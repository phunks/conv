use std::ops::Range;
use eframe::egui::text::LayoutJob;
use eframe::egui::{
    Align, Button, Color32, Layout, RichText, ScrollArea, Stroke, TextEdit, TextFormat,
    TextStyle, Ui,
};
use crate::app::cache::DiffCache;
use crate::app::state::{DiffToolOptions, DiffView};

pub const MAX_DIFF_INPUT_BYTES: usize = 1024 * 1024;

pub fn diff_editor_ui(
    ui: &mut Ui,
    options: &mut DiffToolOptions,
    cache: &DiffCache,
) -> bool {
    let mut changed = false;

    // ui.horizontal(|ui| {
    //     if ui.button("Copy left").clicked() {
    //         ui.ctx().copy_text(options.left.clone());
    //     }
    //
    //     if ui.button("Copy right").clicked() {
    //         ui.ctx().copy_text(options.right.clone());
    //     }
    //
    //     ui.separator();
    //
    //     if ui.button("Swap").clicked() {
    //         std::mem::swap(&mut options.left, &mut options.right);
    //         changed = true;
    //     }
    //
    //     if ui
    //         .add_enabled(
    //             !options.left.is_empty() || !options.right.is_empty(),
    //             Button::new("Clear"),
    //         )
    //         .clicked()
    //     {
    //         options.left.clear();
    //         options.right.clear();
    //         changed = true;
    //     }
    //
    //     ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
    //         ui.label(format!(
    //             "right: {} / 1 MiB",
    //             format_bytes(options.right.len())
    //         ));
    //         ui.label(format!(
    //             "left: {} / 1 MiB",
    //             format_bytes(options.left.len())
    //         ));
    //     });
    // });

    if options.left.len() > MAX_DIFF_INPUT_BYTES {
        ui.colored_label(Color32::ORANGE, "left input exceeds 1 MiB");
    }

    if options.right.len() > MAX_DIFF_INPUT_BYTES {
        ui.colored_label(Color32::ORANGE, "right input exceeds 1 MiB");
    }

    if let Some(error) = &cache.error {
        ui.colored_label(Color32::ORANGE, error);
    }

    diff_status_ui(ui, options, cache);
    ui.add_space(ui.spacing().item_spacing.y);

    match options.view {
        DiffView::Edit => {
            changed |= edit_ui(ui, options, cache);
        }
        DiffView::Diff => {
            diff_ui(ui, cache);
        }
    }

    changed
}

fn edit_ui(
    ui: &mut Ui,
    options: &mut DiffToolOptions,
    cache: &DiffCache,
) -> bool {
    let scroll_offset = options.scroll_offset;
    let ctx = ui.ctx().clone();

    ui.columns(2, |columns| {
        let left = ScrollArea::vertical()
            .id_salt("diff.left.editor.scroll")
            .vertical_scroll_offset(scroll_offset)
            .show(&mut columns[0], |ui| {
                let desired_size = ui.available_size();
                // ui.label(RichText::new("Before").strong());

                let mut layouter = |ui: &Ui, text: &str, wrap_width: f32| {
                    let mut job = diff_edit_layout(
                        ui,
                        text,
                        cache.result.as_ref(),
                        DiffSide::Left,
                    );
                    job.wrap.max_width = wrap_width;
                    ui.fonts(|fonts| fonts.layout_job(job))
                };

                ui.add_sized(
                    desired_size,
                    TextEdit::multiline(&mut options.left)
                        .id_salt("diff.left.editor")
                        .desired_width(f32::INFINITY)
                        // .desired_rows(30)
                        .font(TextStyle::Monospace)
                        .layouter(&mut layouter),
                )
                    .changed()
            });

        let right = ScrollArea::vertical()
            .id_salt("diff.right.editor.scroll")
            .vertical_scroll_offset(scroll_offset)
            .show(&mut columns[1], |ui| {
                let desired_size = ui.available_size();
                // ui.label(RichText::new("After").strong());

                let mut layouter = |ui: &Ui, text: &str, wrap_width: f32| {
                    let mut job = diff_edit_layout(
                        ui,
                        text,
                        cache.result.as_ref(),
                        DiffSide::Right,
                    );
                    job.wrap.max_width = wrap_width;
                    ui.fonts(|fonts| fonts.layout_job(job))
                };

                ui.add_sized(
                    desired_size,
                    TextEdit::multiline(&mut options.right)
                        .id_salt("diff.right.editor")
                        .desired_width(f32::INFINITY)
                        // .desired_rows(30)
                        .font(TextStyle::Monospace)
                        .layouter(&mut layouter),
                )
                    .changed()
            });

        let pointer_pos = ctx.pointer_hover_pos();
        let left_hovered = pointer_pos.is_some_and(|pos| left.inner_rect.contains(pos));
        let right_hovered = pointer_pos.is_some_and(|pos| right.inner_rect.contains(pos));

        let updated_offset = match (left_hovered, right_hovered) {
            (true, false) => left.state.offset.y,
            (false, true) => right.state.offset.y,
            _ => scroll_offset,
        };

        if (updated_offset - options.scroll_offset).abs() > f32::EPSILON {
            options.scroll_offset = updated_offset;
            ctx.request_repaint();
        }

        left.inner || right.inner
    })
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
                    ui.colored_label(Color32::YELLOW, "changes");
                } else {
                    ui.colored_label(Color32::LIGHT_GREEN, "no changes");
                }
            }
            None if options.left.is_empty() && options.right.is_empty() => {
                ui.label(RichText::new("Text").strong());
                ui.label("paste text to compare");
            }
            None => {
                ui.label(RichText::new("Text").strong());
                ui.colored_label(Color32::ORANGE, "not loaded");
            }
        }

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
    });
}

#[derive(Clone, Copy)]
enum DiffSide {
    Left,
    Right,
}

fn diff_edit_layout(
    ui: &Ui,
    text: &str,
    diff: Option<&difftastic::clipboard::ClipboardDiff>,
    side: DiffSide,
) -> LayoutJob {
    let monospace = TextStyle::Monospace.resolve(ui.style());
    let mut job = LayoutJob::default();

    let mut cells_by_line = vec![None; text.split_inclusive('\n').count() + 1];

    if let Some(result) = diff {
        for row in result.rows() {
            let cell = match side {
                DiffSide::Left => row.left_line.as_ref(),
                DiffSide::Right => row.right_line.as_ref(),
            };

            if let Some(cell) = cell
                && cell.line_number < cells_by_line.len()
            {
                cells_by_line[cell.line_number] = Some(cell);
            }
        }
    }

    let (row_background, highlight_background) = match side {
        DiffSide::Left => (
            Color32::from_rgb(78, 35, 35),
            Color32::from_rgb(130, 55, 55),
        ),
        DiffSide::Right => (
            Color32::from_rgb(32, 72, 42),
            Color32::from_rgb(50, 115, 70),
        ),
    };

    let normal = TextFormat {
        font_id: monospace.clone(),
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
        underline: Stroke::new(1.0, Color32::LIGHT_YELLOW),
        ..Default::default()
    };

    let mut source_offset = 0;

    for (line_index, line) in text.split_inclusive('\n').enumerate() {
        let line_text = line.strip_suffix('\n').unwrap_or(line);
        let has_newline = line.ends_with('\n');

        let cell = cells_by_line
            .get(line_index + 1)
            .copied()
            .flatten();

        match cell {
            Some(cell) if cell.changed && cell.text == line_text => {
                append_diff_text(
                    &mut job,
                    line_text,
                    &cell.highlights,
                    changed.clone(),
                    highlighted.clone(),
                );
            }
            _ => {
                job.append(line_text, 0.0, normal.clone());
            }
        }

        if has_newline {
            job.append("\n", 0.0, normal.clone());
        }

        source_offset += line.len();
    }

    if text.is_empty() {
        job.append("", 0.0, normal);
    }

    job
}

// fn find_diff_cell(
//     result: &difftastic::clipboard::ClipboardDiff,
//     side: DiffSide,
//     line_number: usize,
// ) -> Option<&difftastic::clipboard::DiffCell> {
//     result.rows().iter().find_map(|row| {
//         let cell = match side {
//             DiffSide::Left => row.left_line.as_ref(),
//             DiffSide::Right => row.right_line.as_ref(),
//         };
//
//         cell.filter(|cell| cell.line_number == line_number)
//     })
// }

fn diff_ui(ui: &mut Ui, cache: &DiffCache) {
    let Some(result) = &cache.result else {
        ui.label("Paste text in Edit mode, then select Diff.");
        return;
    };

    // ui.horizontal(|ui| {
    //     ui.label(RichText::new(result.language()).strong());
    //
    //     if result.has_changes() {
    //         ui.colored_label(Color32::YELLOW, "changes");
    //     } else {
    //         ui.colored_label(Color32::LIGHT_GREEN, "no changes");
    //     }
    // });
    //
    // ui.separator();

    ScrollArea::vertical()
        .id_salt("diff.rows.scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.columns(2, |columns| {
                columns[0].with_layout(
                    Layout::top_down(Align::LEFT).with_cross_justify(false),
                    |ui| {
                        ui.label(RichText::new("Before").strong());
                    },
                );

                columns[1].with_layout(
                    Layout::top_down(Align::LEFT).with_cross_justify(false),
                    |ui| {
                        ui.label(RichText::new("After").strong());
                    },
                );
            });

            ui.separator();

            for row in result.rows() {
                ui.columns(2, |columns| {
                    columns[0].with_layout(
                        Layout::top_down(Align::LEFT).with_cross_justify(false),
                        |ui| {
                            diff_cell_ui(
                                ui,
                                row.left_line.as_ref(),
                                Color32::from_rgb(78, 35, 35),
                                Color32::from_rgb(130, 55, 55),
                            );
                        },
                    );

                    columns[1].with_layout(
                        Layout::top_down(Align::LEFT).with_cross_justify(false),
                        |ui| {
                            diff_cell_ui(
                                ui,
                                row.right_line.as_ref(),
                                Color32::from_rgb(32, 72, 42),
                                Color32::from_rgb(50, 115, 70),
                            );
                        },
                    );
                });
            }
        });
}

fn diff_cell_ui(
    ui: &mut Ui,
    cell: Option<&difftastic::clipboard::DiffCell>,
    row_background: Color32,
    highlight_background: Color32,
) {
    let Some(cell) = cell else {
        ui.label(" ");
        return;
    };

    let monospace = TextStyle::Monospace.resolve(ui.style());
    let mut job = LayoutJob::default();

    job.append(
        &format!("{:>6}  ", cell.line_number),
        0.0,
        TextFormat {
            font_id: monospace.clone(),
            color: ui.visuals().weak_text_color(),
            ..Default::default()
        },
    );

    let normal_format = TextFormat {
        font_id: monospace.clone(),
        background: cell.changed.then_some(row_background).unwrap_or(Color32::TRANSPARENT),
        ..Default::default()
    };

    let highlighted_format = TextFormat {
        font_id: monospace,
        background: highlight_background,
        underline: Stroke::new(1.0, Color32::LIGHT_YELLOW),
        ..Default::default()
    };

    append_diff_text(
        &mut job,
        &cell.text,
        &cell.highlights,
        normal_format,
        highlighted_format,
    );

    ui.label(job);
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