use std::ops::Range;

use eframe::egui::text::{CCursorRange, LayoutJob};
use eframe::egui::{Align, Color32, Frame, Id, Label, Layout, Margin, OutputCommand, RichText, ScrollArea, Stroke, TextEdit, TextFormat, TextStyle, TextWrapMode, Ui};
use eframe::egui::style::ScrollStyle;
use eframe::egui::text_edit::TextEditState;
use log::debug;
use crate::app::cache::{
    AlignedDiff, AlignedLine, AlignedPane, DiffCache, HexLineCache, VIRTUAL_LINE_MARKER,
};
use crate::app::colors::{DIFF_RIGHT_BG, DIFF_RIGHT_FG, DIFF_BORDER, DIFF_HIGHLIGHT_UNDERLINE, DIFF_LEFT_BG, DIFF_LEFT_FG, DIFF_SELECTION, STATUS_CHANGED, STATUS_UNCHANGED, TEXT_ON_ACCENT, TRANSPARENT, WARNING};
use crate::app::state::DiffToolOptions;

pub const MAX_DIFF_INPUT_BYTES: usize = 1024 * 1024;

const MAX_HEX_LINE_BYTES: usize = 64 * 1024;

#[derive(Clone, Default)]
struct HexSelection {
    line_number: Option<usize>,
    byte_range: Option<Range<usize>>,
}

pub fn diff_editor_ui(
    ui: &mut Ui,
    options: &mut DiffToolOptions,
    cache: &mut DiffCache,
) -> bool {
    let mut changed = false;

    if options.left.len() > MAX_DIFF_INPUT_BYTES {
        ui.colored_label(WARNING, "left input exceeds 1 MiB");
    }

    if options.right.len() > MAX_DIFF_INPUT_BYTES {
        ui.colored_label(WARNING, "right input exceeds 1 MiB");
    }

    if let Some(error) = &cache.error {
        ui.colored_label(WARNING, error);
    }

    diff_status_ui(ui, options, cache);
    ui.add_space(ui.spacing().item_spacing.y);

    changed |= edit_ui(ui, options, cache);

    changed
}

fn edit_ui(
    ui: &mut Ui,
    options: &mut DiffToolOptions,
    cache: &mut DiffCache,
) -> bool {
    const LINE_NUMBER_WIDTH: f32 = 50.0;
    const TEXT_EDIT_HORIZONTAL_MARGIN: f32 = 8.0;
    ui.spacing_mut().item_spacing.x = 0.0;
    let columns_width = (ui.available_width() - ui.spacing().item_spacing.x) * 0.5;
    let wrap_width = (
        columns_width
            - LINE_NUMBER_WIDTH
            - TEXT_EDIT_HORIZONTAL_MARGIN
    )
        .max(1.0);

    let should_rebuild_aligned = cache.update_deadline.is_none()
        && cache.result.is_some()
        && cache.aligned.as_ref().is_none_or(|aligned| {
        (aligned.left_wrap_width - wrap_width).abs() > f32::EPSILON
            || (aligned.right_wrap_width - wrap_width).abs() > f32::EPSILON
    });

    if should_rebuild_aligned
        && let Some(result) = cache.result.as_ref() {
            cache.aligned = Some(aligned_diff(
                ui,
                result,
                &options.left,
                &options.right,
                wrap_width,
                wrap_width,
            ));
    }

    let Some(aligned) = cache.aligned.as_mut() else {
        ui.columns(2, |columns| {
            let left_size = columns[0].available_size();
            let left_content_width = (left_size.x - LINE_NUMBER_WIDTH).max(0.0);

            columns[0].horizontal_top(|ui| {
                line_number_editor(
                    ui,
                    "diff.left.line_numbers",
                    "",
                    LINE_NUMBER_WIDTH,
                    left_size.y,
                    None,
                    None,
                    TRANSPARENT,
                );

                ui.add_sized(
                    [left_content_width, left_size.y],
                    TextEdit::multiline(&mut options.left)
                        .id_salt("diff.left.editor")
                        .desired_width(left_content_width)
                        .font(TextStyle::Monospace),
                );
            });

            let right_size = columns[1].available_size();
            let right_content_width = (right_size.x - LINE_NUMBER_WIDTH).max(0.0);

            columns[1].horizontal_top(|ui| {
                line_number_editor(
                    ui,
                    "diff.right.line_numbers",
                    "",
                    LINE_NUMBER_WIDTH,
                    right_size.y,
                    None,
                    None,
                    TRANSPARENT,
                );

                ui.add_sized(
                    [right_content_width, right_size.y],
                    TextEdit::multiline(&mut options.right)
                        .id_salt("diff.right.editor")
                        .desired_width(right_content_width)
                        .font(TextStyle::Monospace),
                );
            });
        });

        return false;
    };

    if aligned.change_markers.is_empty() {
        options.change_index = usize::MAX;
        options.pending_change_delta = 0;
    } else if options.pending_change_delta != 0 {
        const CHANGE_CONTEXT_ROWS: f32 = 6.0;

        let last_change_index = aligned.change_markers.len() - 1;
        let direction = options.pending_change_delta.signum();

        let next_index = match (options.change_index, direction) {
            (usize::MAX, 1) => 0,
            (usize::MAX, -1) => last_change_index,
            (index, 1) => index.saturating_add(1).min(last_change_index),
            (index, -1) => index.saturating_sub(1),
            (index, _) => index,
        };

        options.change_index = next_index;
        options.pending_change_delta = 0;

        let marker = aligned.change_markers[next_index];
        let context_height =
            CHANGE_CONTEXT_ROWS * ui.text_style_height(&TextStyle::Monospace);

        options.scroll_offset = (marker.scroll_y - context_height).max(0.0);

        debug!(
                "jump: index={}, marker_y={:.3}, context_height={:.3}, offset={:.3}",
                next_index,
                marker.scroll_y,
                context_height,
                options.scroll_offset,
            );

        ui.ctx().request_repaint();
    }

    let scroll_offset = options.scroll_offset;
    let ctx = ui.ctx().clone();
    let left_editor_id = eframe::egui::Id::new("diff.left.editor");
    let right_editor_id = eframe::egui::Id::new("diff.right.editor");
    let left_active_line = source_line_at_cursor(
        &ctx,
        left_editor_id,
        &aligned.left.text,
        &aligned.left.lines,
    );
    let right_active_line = source_line_at_cursor(
        &ctx,
        right_editor_id,
        &aligned.right.text,
        &aligned.right.lines,
    );
    let result = cache.result.as_ref();
    let left_lines = aligned.left.lines.clone();
    let right_lines = aligned.right.lines.clone();
    let left_line_numbers = aligned.left.line_numbers.clone();
    let right_line_numbers = aligned.right.line_numbers.clone();
    let left_diagnostic_line = cache
        .format_diagnostics
        .left
        .as_ref()
        .and_then(|diagnostic| diagnostic.line);
    let right_diagnostic_line = cache
        .format_diagnostics
        .right
        .as_ref()
        .and_then(|diagnostic| diagnostic.line);
    let left_saved_selection = cache.selection.left.clone();
    let right_saved_selection = cache.selection.right.clone();
    let left_layout_cache = &mut cache.layout.left;
    let right_layout_cache = &mut cache.layout.right;

    let hex_height = if options.show_hex {
        ui.text_style_height(&TextStyle::Monospace) * 2.0
            + ui.spacing().item_spacing.y
            + 20.0
    } else {
        0.0
    };
    let editor_height = (ui.available_height() - hex_height).max(0.0);

    let (left_response, right_response) = ui.columns(2, |columns| {
        let left = ScrollArea::vertical()
            .id_salt("diff.left.editor.scroll")
            .vertical_scroll_offset(scroll_offset)
            .max_height(editor_height)
            .show(&mut columns[0], |ui| {
                let desired_size = eframe::egui::vec2(ui.available_width(), editor_height);
                let content_width = (desired_size.x - LINE_NUMBER_WIDTH).max(0.0);

                ui.horizontal_top(|ui| {
                    line_number_editor(
                        ui,
                        "diff.left.line_numbers",
                        &left_line_numbers,
                        LINE_NUMBER_WIDTH,
                        desired_size.y,
                        left_active_line,
                        left_diagnostic_line,
                        DIFF_LEFT_FG,
                    );

                    let mut layouter = |ui: &Ui, text: &str, wrap_width: f32| {
                        left_layout_cache.galley(ui, text, wrap_width, || {
                            let mut job = aligned_diff_layout(
                                ui,
                                text,
                                &left_lines,
                                result,
                                DiffSide::Left,
                                left_saved_selection.as_ref(),
                            );
                            job.wrap.max_width = wrap_width;
                            job.wrap.break_anywhere = true;
                            job
                        })
                    };

                    ui.add_sized(
                        [content_width, desired_size.y],
                        TextEdit::multiline(&mut aligned.left.text)
                            .id(left_editor_id)
                            .desired_width(content_width)
                            .font(TextStyle::Monospace)
                            .layouter(&mut layouter),
                    )
                })
                    .inner
            });

        let right = ScrollArea::vertical()
            .id_salt("diff.right.editor.scroll")
            .vertical_scroll_offset(scroll_offset)
            .max_height(editor_height)
            .show(&mut columns[1], |ui| {
                let desired_size = eframe::egui::vec2(ui.available_width(), editor_height);
                let content_width = (desired_size.x - LINE_NUMBER_WIDTH).max(0.0);

                ui.horizontal_top(|ui| {
                    line_number_editor(
                        ui,
                        "diff.right.line_numbers",
                        &right_line_numbers,
                        LINE_NUMBER_WIDTH,
                        desired_size.y,
                        right_active_line,
                        right_diagnostic_line,
                        DIFF_RIGHT_FG,
                    );

                    let mut layouter = |ui: &Ui, text: &str, wrap_width: f32| {
                        right_layout_cache.galley(ui, text, wrap_width, || {
                            let mut job = aligned_diff_layout(
                                ui,
                                text,
                                &right_lines,
                                result,
                                DiffSide::Right,
                                right_saved_selection.as_ref(),
                            );
                            job.wrap.max_width = wrap_width;
                            job.wrap.break_anywhere = true;
                            job
                        })
                    };

                    ui.add_sized(
                        [content_width, desired_size.y],
                        TextEdit::multiline(&mut aligned.right.text)
                            .id(right_editor_id)
                            .desired_width(content_width)
                            .font(TextStyle::Monospace)
                            .layouter(&mut layouter),
                    )
                })
                    .inner
            });

        let left_offset = left.state.offset.y;
        let right_offset = right.state.offset.y;

        let left_changed = (left_offset - scroll_offset).abs() > f32::EPSILON;
        let right_changed = (right_offset - scroll_offset).abs() > f32::EPSILON;

        let updated_offset = match (left_changed, right_changed) {
            // Left text, left scroll bar, left wheel operation.
            (true, false) => left_offset,

            // Right text, right scroll bar, right wheel operation.
            (false, true) => right_offset,

            // Immediately after specifying the same position to both from the program.
            (false, false) => scroll_offset,

            // Usually unlikely, but if both are updated.
            // Either can be used as long as they have the same value.
            (true, true) => left_offset,
        };

        if (updated_offset - options.scroll_offset).abs() > f32::EPSILON {
            options.scroll_offset = updated_offset;
            ctx.request_repaint();
        }

        (left.inner, right.inner)
    });

    update_saved_selection(
        &ctx,
        left_response.id,
        &mut cache.selection.left,
        &mut cache.layout.left,
    );
    update_saved_selection(
        &ctx,
        right_response.id,
        &mut cache.selection.right,
        &mut cache.layout.right,
    );

    if options.show_hex {
        let left_selection = hex_selection(
            &ctx,
            left_response.id,
            &aligned.left.text,
            &aligned.left.lines,
        );
        let right_selection = hex_selection(
            &ctx,
            right_response.id,
            &aligned.right.text,
            &aligned.right.lines,
        );

        ui.add_space(ui.spacing().item_spacing.y);

        Frame::canvas(ui.style())
            .stroke(Stroke::new(1.5_f32, DIFF_BORDER))
            .inner_margin(8.0)
            .outer_margin(Margin {
                left: 2,
                right: 0,
                ..Default::default()
            })
            .show(ui, |ui| {
                ui.style_mut().spacing.scroll = ScrollStyle {
                    floating: true,
                    bar_width: 2.0,
                    ..ScrollStyle::solid()
                };
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label("L: ");
                        ui.label("R: ");
                    });
                    ScrollArea::horizontal()
                        .id_salt("diff.hex.scroll")
                        .auto_shrink([false, true])
                        .show(ui, |ui| {
                            ui.vertical(|ui| {
                                hex_line_ui(
                                    ui,
                                    &options.left,
                                    left_selection,
                                    DIFF_LEFT_FG,
                                    &mut cache.hex.left,
                                );

                                hex_line_ui(
                                    ui,
                                    &options.right,
                                    right_selection,
                                    DIFF_RIGHT_FG,
                                    &mut cache.hex.right,
                                );
                            });
                        });
                });
            });
    }

    if left_response.changed() {
        options.left = source_from_aligned_display(&aligned.left);
    }

    if right_response.changed() {
        options.right = source_from_aligned_display(&aligned.right);
    }

    let copy_requested = ctx.input(|input| {
        let copy_shortcut = input.key_pressed(eframe::egui::Key::C)
            && if cfg!(target_os = "macos") {
            input.modifiers.command
        } else {
            input.modifiers.ctrl
        };

        copy_shortcut
            || input
            .events
            .iter()
            .any(|event| matches!(event, eframe::egui::Event::Copy))
    });

    if copy_requested && (left_response.has_focus() || right_response.has_focus()) {
        ctx.output_mut(|output| {
            for command in &mut output.commands {
                if let OutputCommand::CopyText(text) = command {
                    *text = copy_text_without_virtual_lines(text);
                }
            }
        });
    }

    left_response.changed() || right_response.changed()
}

fn line_number_editor(
    ui: &mut Ui,
    id_salt: &str,
    line_numbers: &str,
    width: f32,
    height: f32,
    active_line: Option<usize>,
    diagnostic_line: Option<usize>,
    active_background: Color32,
) {
    let mut line_numbers = line_numbers;
    let mut layouter = |ui: &Ui, text: &str, wrap_width: f32| {
        let mut job = LayoutJob::default();
        let normal = TextFormat {
            font_id: TextStyle::Monospace.resolve(ui.style()),
            color: ui.visuals().weak_text_color(),
            ..Default::default()
        };
        let active = TextFormat {
            font_id: TextStyle::Monospace.resolve(ui.style()),
            color: TEXT_ON_ACCENT,
            background: active_background,
            ..Default::default()
        };
        let diagnostic = TextFormat {
            font_id: TextStyle::Monospace.resolve(ui.style()),
            color: ui.visuals().error_fg_color,
            background: ui.visuals().error_fg_color.gamma_multiply(0.2),
            ..Default::default()
        };

        for (index, line) in text.split('\n').enumerate() {
            if index > 0 {
                job.append("\n", 0.0, normal.clone());
            }

            let line_number = line.trim().parse::<usize>().ok();
            let is_active = line_number
                .zip(active_line)
                .is_some_and(|(line_number, active_line)| line_number == active_line);
            let is_diagnostic = line_number
                .zip(diagnostic_line)
                .is_some_and(|(line_number, diagnostic_line)| {
                    line_number == diagnostic_line
                });

            let format = if is_diagnostic {
                diagnostic.clone()
            } else if is_active {
                active.clone()
            } else {
                normal.clone()
            };

            job.append(line, 0.0, format);
        }

        job.wrap.max_width = wrap_width;
        ui.fonts(|fonts| fonts.layout_job(job))
    };

    ui.add_sized(
        [width, height],
        TextEdit::multiline(&mut line_numbers)
            .id_salt(id_salt)
            .desired_width(width)
            .font(TextStyle::Monospace)
            .margin(Margin::ZERO)
            .frame(false)
            .interactive(false)
            .layouter(&mut layouter),
    );
}

fn copy_text_without_virtual_lines(text: &str) -> String {
    let mut copied = String::with_capacity(text.len());

    for line in text.split_inclusive('\n') {
        let (contents, newline) = line
            .strip_suffix('\n')
            .map_or((line, ""), |contents| (contents, "\n"));

        match contents.strip_prefix(VIRTUAL_LINE_MARKER) {
            // Blank lines inserted by difftastic are excluded from copying, including line breaks.
            Some("") => {}
            // The contents entered by the user in the virtual row are copied as real data.
            Some(contents) => {
                copied.push_str(contents);
                copied.push_str(newline);
            }
            // Normal lines and normal blank lines entered by the user are maintained as they are.
            None => {
                copied.push_str(contents);
                copied.push_str(newline);
            }
        }
    }

    copied
}

fn aligned_diff_layout(
    ui: &Ui,
    text: &str,
    lines: &[AlignedLine],
    diff: Option<&difftastic::clipboard::ClipboardDiff>,
    side: DiffSide,
    saved_selection: Option<&Range<usize>>,
) -> LayoutJob {
    let monospace = TextStyle::Monospace.resolve(ui.style());
    let mut cells_by_source_line = Vec::new();

    if let Some(result) = diff {
        for row in result.rows() {
            let cell = match side {
                DiffSide::Left => row.left_line.as_ref(),
                DiffSide::Right => row.right_line.as_ref(),
            };

            if let Some(cell) = cell {
                if cell.line_number >= cells_by_source_line.len() {
                    cells_by_source_line.resize(cell.line_number + 1, None);
                }

                cells_by_source_line[cell.line_number] = Some(cell);
            }
        }
    }

    let (row_background, highlight_background) = match side {
        DiffSide::Left => (DIFF_LEFT_BG, DIFF_LEFT_FG),
        DiffSide::Right => (DIFF_RIGHT_BG, DIFF_RIGHT_FG),
    };

    let normal = TextFormat {
        font_id: monospace.clone(),
        ..Default::default()
    };

    let virtual_marker = TextFormat {
        font_id: monospace.clone(),
        color: highlight_background,
        background: row_background,
        ..Default::default()
    };

    let changed = TextFormat {
        font_id: monospace.clone(),
        background: row_background,
        ..Default::default()
    };

    let highlighted = TextFormat {
        font_id: monospace,
        background: highlight_background,
        underline: Stroke::new(1.0_f32, DIFF_HIGHLIGHT_UNDERLINE),
        ..Default::default()
    };

    let mut job = LayoutJob::default();

    for (display_row, line) in text.split('\n').enumerate() {
        if display_row > 0 {
            job.append("\n", 0.0, normal.clone());
        }

        let metadata = lines.get(display_row).copied();
        let line_without_marker = line.strip_prefix(VIRTUAL_LINE_MARKER).unwrap_or(line);
        let is_virtual_line = metadata.is_some_and(AlignedLine::is_virtual_blank);

        if line.starts_with(VIRTUAL_LINE_MARKER) {
            job.append(
                &VIRTUAL_LINE_MARKER.to_string(),
                0.0,
                virtual_marker.clone(),
            );
        }

        let cell = metadata
            .and_then(|line| line.source_line)
            .and_then(|source_line| {
                cells_by_source_line
                    .get(source_line)
                    .copied()
                    .flatten()
            });

        match cell {
            Some(cell) if cell.changed && cell.text == line_without_marker => {
                append_diff_text(
                    &mut job,
                    line_without_marker,
                    &cell.highlights,
                    changed.clone(),
                    highlighted.clone(),
                );
            }
            Some(cell) if !is_virtual_line && cell.text == line_without_marker => {
                job.append(line_without_marker, 0.0, normal.clone());
            }
            _ => {
                job.append(line_without_marker, 0.0, normal.clone());
            }
        }
    }

    apply_saved_selection(
        &mut job,
        text,
        saved_selection,
        DIFF_SELECTION,
    );

    job
}

fn diff_status_ui(
    ui: &mut Ui,
    options: &DiffToolOptions,
    cache: &DiffCache,
) {
    ui.horizontal(|ui| {
        match &cache.result {
            Some(result) => {
                ui.label(RichText::new(result.language()).strong());

                if result.has_changes() {
                    ui.colored_label(STATUS_CHANGED, "changes");
                } else {
                    ui.colored_label(STATUS_UNCHANGED, "no changes");
                }
            }
            None if options.left.is_empty() && options.right.is_empty() => {
                ui.label(RichText::new("Text").strong());
                ui.label("paste text to compare");
            }
            None => {
                ui.label(RichText::new("Text").strong());
                ui.colored_label(WARNING, "not loaded");
            }
        }

        let show_input_sizes = cache.result.as_ref().is_none_or(|result| {
            result.language().chars().count() <= 10
        });

        if show_input_sizes {
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.label(format!(
                    "right: {} / 1 MiB",
                    format_bytes(options.right.len())
                ));
                ui.label(format!(
                    "left: {} / 1 MiB",
                    format_bytes(options.left.len())
                ));
            });
        }
    });
}

#[derive(Clone, Copy)]
enum DiffSide {
    Left,
    Right,
}

fn update_saved_selection(
    ctx: &eframe::egui::Context,
    editor_id: Id,
    saved_selection: &mut Option<Range<usize>>,
    layout_cache: &mut crate::app::cache::DiffPaneGalleyCache,
) {
    if !ctx.memory(|memory| memory.has_focus(editor_id)) {
        return;
    }

    let selection = TextEditState::load(ctx, editor_id)
        .and_then(|state| state.cursor.char_range())
        .and_then(|range| {
            let start = range.primary.index.min(range.secondary.index);
            let end = range.primary.index.max(range.secondary.index);

            (start < end).then_some(start..end)
        });

    if *saved_selection != selection {
        *saved_selection = selection;
        layout_cache.clear();
        ctx.request_repaint();
    }
}

fn source_line_at_cursor(
    ctx: &eframe::egui::Context,
    editor_id: eframe::egui::Id,
    display_text: &str,
    lines: &[AlignedLine],
) -> Option<usize> {
    let state = TextEditState::load(ctx, editor_id)?;
    let cursor_range = state.cursor.char_range()?;

    hex_selection_from_cursor_range(display_text, lines, cursor_range).line_number
}


fn hex_selection(
    ctx: &eframe::egui::Context,
    editor_id: Id,
    display_text: &str,
    lines: &[AlignedLine],
) -> HexSelection {
    let first_source_line = lines.iter().find_map(|line| line.source_line);

    let Some(state) = TextEditState::load(ctx, editor_id) else {
        return HexSelection {
            line_number: first_source_line,
            byte_range: None,
        };
    };
    let Some(cursor_range) = state.cursor.char_range() else {
        return HexSelection {
            line_number: first_source_line,
            byte_range: None,
        };
    };

    let selection = hex_selection_from_cursor_range(display_text, lines, cursor_range);

    if selection.line_number.is_some() {
        selection
    } else {
        HexSelection {
            line_number: first_source_line,
            byte_range: None,
        }
    }
}

fn hex_selection_from_cursor_range(
    display_text: &str,
    lines: &[AlignedLine],
    cursor_range: CCursorRange,
) -> HexSelection {
    let start = cursor_range.primary.index.min(cursor_range.secondary.index);
    let end = cursor_range.primary.index.max(cursor_range.secondary.index);

    let line_start = display_text[..char_to_byte_offset(display_text, start)]
        .rfind('\n')
        .map_or(0, |offset| offset + 1);
    let line_end = display_text[line_start..]
        .find('\n')
        .map_or(display_text.len(), |offset| line_start + offset);

    let display_line = display_text[..line_start].bytes().filter(|byte| *byte == b'\n').count();

    let Some(source_line) = lines
        .get(display_line)
        .and_then(|line| line.source_line)
    else {
        return HexSelection::default();
    };

    let selected_in_line = if start == end {
        None
    } else {
        let selection_start = char_to_byte_offset(display_text, start).max(line_start);
        let selection_end = char_to_byte_offset(display_text, end).min(line_end);

        (selection_start < selection_end).then_some(
            selection_start.saturating_sub(line_start)..selection_end.saturating_sub(line_start),
        )
    };

    HexSelection {
        line_number: Some(source_line),
        byte_range: selected_in_line,
    }
}

fn char_to_byte_offset(text: &str, char_offset: usize) -> usize {
    text.char_indices()
        .nth(char_offset)
        .map_or(text.len(), |(byte_offset, _)| byte_offset)
}

fn source_line_including_newline(source: &str, line_number: usize) -> Option<&str> {
    if line_number == 0 {
        return None;
    }

    source.split_inclusive('\n').nth(line_number - 1)
}

fn hex_line_ui(
    ui: &mut Ui,
    source: &str,
    selection: HexSelection,
    selection_background: Color32,
    cache: &mut HexLineCache,
) {
    let Some(line_number) = selection.line_number else {
        ui.monospace("—");
        return;
    };

    let Some(line) = source_line_including_newline(source, line_number) else {
        ui.monospace("—");
        return;
    };

    if line.len() > MAX_HEX_LINE_BYTES {
        ui.colored_label(
            WARNING,
            format!(
                "hex view unavailable; line exceeds {} KiB",
                MAX_HEX_LINE_BYTES / 1024
            ),
        );
        return;
    }

    cache_hex_line(cache, line_number, line);

    let byte_range = selection
        .byte_range
        .and_then(|range| valid_text_range(line, &range));

    let mut job = LayoutJob::default();
    let normal = TextFormat {
        font_id: TextStyle::Monospace.resolve(ui.style()),
        ..Default::default()
    };
    let selected = TextFormat {
        font_id: TextStyle::Monospace.resolve(ui.style()),
        background: selection_background,
        ..Default::default()
    };

    for (byte_offset, text_range) in cache.byte_ranges.iter().enumerate() {
        let format = if byte_range
            .as_ref()
            .is_some_and(|range| range.contains(&byte_offset))
                { selected.clone() } else { normal.clone() };
        job.append(&cache.text[text_range.clone()], 0.0, format);
    }

    ui.add(
        Label::new(job)
            .wrap_mode(TextWrapMode::Extend)
            .selectable(true),
    );
}

fn cache_hex_line(cache: &mut HexLineCache, line_number: usize, line: &str) {
    if cache.line_number == Some(line_number) && cache.source_line == line {
        return;
    }

    const HEX: &[u8; 16] = b"0123456789ABCDEF";

    cache.line_number = Some(line_number);
    line.clone_into(&mut cache.source_line);

    cache.text.clear();
    cache.text.reserve(line.len().saturating_mul(3));

    cache.byte_ranges.clear();
    cache.byte_ranges.reserve(line.len());

    for &byte in line.as_bytes() {
        let start = cache.text.len();

        cache.text.push(HEX[(byte >> 4) as usize] as char);
        cache.text.push(HEX[(byte & 0x0F) as usize] as char);
        cache.text.push(' ');

        cache.byte_ranges.push(start..cache.text.len());
    }
}

fn append_diff_text(
    job: &mut LayoutJob,
    text: &str,
    highlights: &[difftastic::clipboard::DiffHighlight],
    normal_format: TextFormat,
    highlighted_format: TextFormat,
) {
    let mut offset = 0;

    for highlight in highlights {
        let Some(range) = valid_text_range(text, &highlight.range) else {
            continue;
        };

        if range.end <= offset {
            continue;
        }

        let range_start = range.start.max(offset);

        if offset < range_start {
            job.append(&text[offset..range_start], 0.0, normal_format.clone());
        }

        let mut format = highlighted_format.clone();

        if !highlight.underline {
            format.underline = Stroke::NONE;
        }

        job.append(&text[range_start..range.end], 0.0, format);
        offset = range.end;
    }

    if offset < text.len() {
        job.append(&text[offset..], 0.0, normal_format);
    }
}

fn apply_saved_selection(
    job: &mut LayoutJob,
    text: &str,
    selection: Option<&Range<usize>>,
    background: Color32,
) {
    let Some(selection) = selection else {
        return;
    };

    let selection_start = char_to_byte_offset(text, selection.start);
    let selection_end = char_to_byte_offset(text, selection.end);

    if selection_start >= selection_end {
        return;
    }

    let selection = selection_start..selection_end;
    let mut sections = Vec::with_capacity(job.sections.len().saturating_add(2));

    for section in job.sections.drain(..) {
        let start = section.byte_range.start;
        let end = section.byte_range.end;

        if end <= selection.start || selection.end <= start {
            sections.push(section);
            continue;
        }

        if start < selection.start {
            let mut before = section.clone();
            before.byte_range.end = selection.start;
            sections.push(before);
        }

        let mut selected = section.clone();
        selected.byte_range.start = start.max(selection.start);
        selected.byte_range.end = end.min(selection.end);
        selected.format.background = background;
        sections.push(selected);

        if selection.end < end {
            let mut after = section;
            after.byte_range.start = selection.end;
            sections.push(after);
        }
    }

    job.sections = sections;
}

fn valid_text_range(text: &str, range: &Range<usize>) -> Option<Range<usize>> {
    let start = range.start.min(text.len());
    let end = range.end.min(text.len());

    if start >= end || !text.is_char_boundary(start) || !text.is_char_boundary(end) {
        return None;
    }

    Some(start..end)
}

fn format_bytes(bytes: usize) -> String {
    if bytes < 1024 {
        format!("{bytes} B")
    } else {
        format!("{:.1} KiB", bytes as f32 / 1024.0)
    }
}

pub fn aligned_diff(
    ui: &Ui,
    result: &difftastic::clipboard::ClipboardDiff,
    left_source: &str,
    right_source: &str,
    left_wrap_width: f32,
    right_wrap_width: f32,
) -> AlignedDiff {
    let mut aligned = AlignedDiff {
        left: AlignedPane {
            had_trailing_newline: left_source.ends_with('\n'),
            ..Default::default()
        },
        right: AlignedPane {
            had_trailing_newline: right_source.ends_with('\n'),
            ..Default::default()
        },
        left_wrap_width,
        right_wrap_width,
        ..Default::default()
    };

    let mut display_row = 0;

    for row in result.rows() {
        let left_rows = row
            .left_line
            .as_ref()
            .map_or(1, |cell| wrapped_row_count(ui, &cell.text, left_wrap_width));

        let right_rows = row
            .right_line
            .as_ref()
            .map_or(1, |cell| wrapped_row_count(ui, &cell.text, right_wrap_width));

        let display_rows = left_rows.max(right_rows);

        let row_has_change = row
            .left_line
            .as_ref()
            .is_some_and(|cell| cell.changed)
            || row
            .right_line
            .as_ref()
            .is_some_and(|cell| cell.changed);

        if row_has_change {
            let end_display_row = display_row + display_rows;

            match aligned.change_markers.last_mut() {
                // If it is immediately after the previous changed block,
                // it will be grouped as the same block.
                Some(previous) if previous.end_display_row == display_row => {
                    previous.end_display_row = end_display_row;

                    debug!(
                            "extend change marker: start={}, end={}",
                            previous.display_row,
                            previous.end_display_row,
                        );
                }
                // A new modified block if it is flanked by non-modified lines.
                _ => {
                    let left_char_offset = aligned.left.text.chars().count()
                        + usize::from(!aligned.left.text.is_empty());

                    debug!(
                            "new change marker #{}: start={}, end={}, left_line={:?}, right_line={:?}",
                            aligned.change_markers.len(),
                            display_row,
                            end_display_row,
                            row.left_line.as_ref().map(|cell| cell.line_number),
                            row.right_line.as_ref().map(|cell| cell.line_number),
                        );

                    aligned.change_markers.push(crate::app::cache::ChangeMarker {
                        display_row,
                        end_display_row,
                        left_char_offset,
                        scroll_y: 0.0,
                    });
                }
            }
        }

        append_aligned_line(
            &mut aligned.left,
            row.left_line.as_ref(),
            left_rows,
        );
        append_aligned_line(
            &mut aligned.right,
            row.right_line.as_ref(),
            right_rows,
        );

        for _ in left_rows..display_rows {
            append_virtual_line(&mut aligned.left);
        }

        for _ in right_rows..display_rows {
            append_virtual_line(&mut aligned.right);
        }

        display_row += display_rows;
    }

    let mut left_job = aligned_diff_layout(
        ui,
        &aligned.left.text,
        &aligned.left.lines,
        Some(result),
        DiffSide::Left,
        None,
    );
    left_job.wrap.max_width = left_wrap_width;
    left_job.wrap.break_anywhere = true;

    let left_galley = ui.fonts(|fonts| fonts.layout_job(left_job));

    for marker in &mut aligned.change_markers {
        marker.scroll_y = galley_row_y(&left_galley, marker.left_char_offset);

        debug!(
                "resolved marker: display_row={}, char_offset={}, scroll_y={:.3}",
                marker.display_row,
                marker.left_char_offset,
                marker.scroll_y,
            );
    }

    aligned.total_display_rows = display_row;
    aligned
}

fn galley_row_y(galley: &eframe::egui::Galley, char_offset: usize) -> f32 {
    galley
        .pos_from_ccursor(eframe::egui::text::CCursor {
            index: char_offset,
            prefer_next_row: false,
        })
        .top()
}

fn wrapped_row_count(ui: &Ui, text: &str, wrap_width: f32) -> usize {
    let mut job = LayoutJob::default();

    job.append(
        text,
        0.0,
        TextFormat {
            font_id: TextStyle::Monospace.resolve(ui.style()),
            ..Default::default()
        },
    );
    job.wrap.max_width = wrap_width;
    job.wrap.break_anywhere = true;

    ui.fonts(|fonts| fonts.layout_job(job).rows.len().max(1))
}

pub fn source_from_aligned_display(pane: &AlignedPane) -> String {
    let mut lines = Vec::new();

    for line in pane.text.split('\n') {
        match line.strip_prefix(VIRTUAL_LINE_MARKER) {
            Some("") => {
                // The virtual blank lines inserted by difftastic
                // are not included in the canonical source.
            }
            Some(contents) => {
                // The content entered into the virtual row
                // becomes a new row of real data.
                lines.push(contents);
            }
            None => {
                // Normal line/normal blank line entered by the user.
                lines.push(line);
            }
        }
    }

    // The empty element created by `split('\n')` in response to a trailing newline
    // will be returned to a newline by restoring had_trailing_newline below, so it is excluded here.
    if pane.had_trailing_newline && lines.last().is_some_and(|line| line.is_empty()) {
        lines.pop();
    }

    let mut source = lines.join("\n");

    if pane.had_trailing_newline {
        source.push('\n');
    }

    source
}

fn append_aligned_line(
    pane: &mut AlignedPane,
    cell: Option<&difftastic::clipboard::DiffCell>,
    wrapped_rows: usize,
) {
    match cell {
        Some(cell) => {
            if !pane.lines.is_empty() {
                pane.text.push('\n');
                pane.line_numbers.push('\n');
            }

            pane.text.push_str(&cell.text);
            pane.line_numbers
                .push_str(&format!("{:>6}", cell.line_number));
            pane.lines.push(AlignedLine::source(cell.line_number));

            // TextEdit wraps the body naturally.
            // Since the line number side is not wrapped,
            // add blank lines for the continuation of wrapping.
            for _ in 1..wrapped_rows {
                pane.line_numbers.push('\n');
                pane.line_numbers.push_str("      ");
            }
        }
        None => append_virtual_line(pane),
    }
}

fn append_virtual_line(pane: &mut AlignedPane) {
    if !pane.lines.is_empty() {
        pane.text.push('\n');
        pane.line_numbers.push('\n');
    }

    pane.text.push(VIRTUAL_LINE_MARKER);
    pane.line_numbers.push_str("      ");
    pane.lines.push(AlignedLine::virtual_blank());
}