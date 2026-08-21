
use std::time::Duration;
use std::time::Instant;
use eframe::egui::{Align, Button, CentralPanel, Context, Key, Layout, Modifiers, Response, ScrollArea, Ui, Visuals};
use egui_json_tree::{DefaultExpand, JsonTree, JsonTreeStyle};
use crate::app::cache::AppCache;
use crate::app::colors::JsonTreeColorScheme;
use crate::app::state::{AppState, SelectedTool};
use crate::converters::Converters;
use crate::converters::format::formatters::FormatLanguage;
use crate::widgets::diff_lang::DiffLanguage;

#[derive(Default)]
pub struct App {
    pub state: AppState,
    pub cache: AppCache,
    pub converters: Converters,
}

impl eframe::App for App {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        ctx.set_visuals(Visuals::dark());
        CentralPanel::default().show(ctx, |ui| {
            self.app_ui(ui);
        });
    }
}

impl App {
    fn app_ui(&mut self, ui: &mut Ui) {
        if self.state.selected == SelectedTool::Diff {
            self.diff_ui(ui);
            return;
        }

        if self.state.selected == SelectedTool::Spreadsheet {
            self.spreadsheet_ui(ui);
            return;
        }

        let selected_before_toolbar = self.state.selected;
        let toolbar_changed = self.toolbar_ui(ui);

        if self.state.selected != selected_before_toolbar {
            self.cache.output_layout.clear();
        } else if toolbar_changed {
            self.cache.outputs.mark_current_dirty(self.state.selected);
            self.cache.output_layout.clear();
        }

        ui.separator();

        ui.columns(2, |columns| {
            if self.input_ui(&mut columns[0]) {
                self.cache.outputs.mark_all_dirty();
                self.cache.output_layout.clear();
            }
            self.output_ui(&mut columns[1]);
        });
    }

    fn open_data_csv_in_spreadsheet(&mut self) {
        let Some(csv) = self.converters.jq.copy_output_text(&self.state) else {
            return;
        };

        self.state.options.spreadsheet.open_csv(&csv);
        self.state.selected = SelectedTool::Spreadsheet;
        self.cache.output_layout.clear();
    }

    fn spreadsheet_ui(&mut self, ui: &mut Ui) {
        let cancel_requested = ui.ctx().input_mut(|input| {
            input.consume_key(Modifiers::NONE, Key::Escape)
        });

        if cancel_requested {
            self.state.selected = SelectedTool::Data;
            self.cache.output_layout.clear();
            return;
        }

        ui.horizontal(|ui| {
            if ui
                .button("Cancel")
                .on_hover_text("return to Data without changing the input")
                .clicked()
            {
                self.state.selected = SelectedTool::Data;
                self.cache.output_layout.clear();
            }

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let enabled = self.state.options.spreadsheet.has_table();

                if ui
                    .add_enabled(enabled, Button::new("← input"))
                    .on_hover_text("copy the current CSV table to input and return to Data")
                    .clicked()
                    && let Ok(csv) = self.state.options.spreadsheet.csv_text()
                {
                    self.state.input = csv;

                    let options = &mut self.state.options.data;
                    let input_format = options.input_format;
                    options.input_format = options.output_format.into();
                    options.output_format = input_format.into();

                    self.state.selected = SelectedTool::Data;
                    self.cache.outputs.mark_all_dirty();
                    self.cache.output_layout.clear();
                }
            });
        });

        ui.separator();

        crate::widgets::spreadsheet::spreadsheet_ui(
            ui,
            &mut self.state.options.spreadsheet,
        );
    }

    fn input_ui(&mut self, ui: &mut Ui) -> bool {
        crate::widgets::input_editor::input_editor_ui(
            ui,
            &mut self.state,
            &mut self.cache.input_layout,
        )
    }

    fn toolbar_ui(&mut self, ui: &mut Ui) -> bool {
        ui.horizontal(|ui| {
            let changed = crate::widgets::menu::menu_ui(ui, &mut self.state);

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let selected = self.state.selected;
                let data_outputs_csv = selected == SelectedTool::Data
                    && self.state.options.data.output_format
                    == crate::widgets::menu::OutputFormat::Csv;

                let enabled = match selected {
                    SelectedTool::Data => {
                        self.state.options.data.input_format
                            == crate::widgets::menu::InputFormat::Xml
                            || !self.state.options.data.filter.is_empty()
                    }
                    _ => self
                        .cache
                        .outputs
                        .current(selected)
                        .result
                        .has_non_empty_plain_text(),
                };

                if ui
                    .add_enabled(enabled, Button::new("← input"))
                    .on_hover_text("copy output to input")
                    .clicked()
                {
                    let output = match selected {
                        SelectedTool::Data => self.converters.jq.copy_output_text(&self.state),
                        _ => self
                            .cache
                            .outputs
                            .current(selected)
                            .result
                            .plain_text_owned(),
                    };

                    if let Some(output) = output {
                        self.state.input = output;

                        if selected == SelectedTool::Data {
                            let options = &mut self.state.options.data;
                            let input_format = options.input_format;
                            options.input_format = options.output_format.into();
                            options.output_format = input_format.into();
                        }

                        self.cache.outputs.mark_all_dirty();
                        self.cache.output_layout.clear();
                    }
                }

                if data_outputs_csv
                    && ui
                    .button("Spreadsheet…")
                    .on_hover_text("open the current CSV output in the spreadsheet")
                    .clicked()
                {
                    self.open_data_csv_in_spreadsheet();
                    return;
                }
            });

            changed
        })
            .inner
    }

    fn output_ui(&mut self, ui: &mut Ui) {
        ScrollArea::vertical()
            .id_salt("output")
            .auto_shrink([false, true])
            .show(ui, |ui| {
                ui.with_layout(
                    Layout::top_down(Align::LEFT).with_cross_justify(false),
                    |ui| {
                        if self.state.selected == SelectedTool::Data {
                            self.converters.jq.render(ui, &self.state);
                            return;
                        }

                        self.ensure_output();

                        // if self.state.selected == SelectedTool::Format
                        //     && self.state.options.format.language == FormatLanguage::Json
                        // {
                        //     self.format_json_output_ui(ui);
                        //     return;
                        // }

                        let selected = self.state.selected;
                        let result = &self.cache.outputs.current(selected).result;
                        result.render(ui, &mut self.cache.output_layout);
                    },
                );
            });
    }

    fn format_json_output_ui(&mut self, ui: &mut Ui) {
        let result = &self
            .cache
            .outputs
            .current(SelectedTool::Format)
            .result;

        match result {
            crate::app::result::ConvertResult::Text(text) => {
                match serde_json::from_str::<serde_json::Value>(text) {
                    Ok(value) => {
                        let tree_style = JsonTreeStyle::new()
                            .visuals(JsonTreeColorScheme::new());
                        JsonTree::new("format.json_tree", &value)
                            .style(tree_style)
                            .default_expand(DefaultExpand::All)
                            .show(ui);
                    }
                    Err(error) => {
                        ui.colored_label(
                            ui.visuals().error_fg_color,
                            format!("formatted JSON could not be parsed: {error}"),
                        );
                    }
                }
            }
            _ => result.render(ui, &mut self.cache.output_layout),
        }
    }

    fn ensure_output(&mut self) {
        let output = self.cache.outputs.current_mut(self.state.selected);

        if !output.dirty {
            return;
        }

        output.result = self.converters.convert(&self.state);
        output.dirty = false;
        self.cache.output_layout.clear();
    }

    fn diff_ui(&mut self, ui: &mut Ui) {
        const DIFF_DEBOUNCE: Duration = Duration::from_millis(30);

        let toolbar_changed = crate::widgets::menu::menu_ui(ui, &mut self.state);

        if toolbar_changed || self.cache.diff.result.is_none() {
            self.cache.diff.update_deadline = None;
            self.update_diff();
        }

        ui.separator();

        let editor_changed = crate::widgets::diff_editor::diff_editor_ui(
            ui,
            &mut self.state.options.diff,
            &mut self.cache.diff,
        );

        if editor_changed {
            self.cache.diff.update_deadline = Some(Instant::now() + DIFF_DEBOUNCE);
        }

        if let Some(deadline) = self.cache.diff.update_deadline {
            let now = Instant::now();

            if now >= deadline {
                self.cache.diff.update_deadline = None;
                self.update_diff();
            } else {
                ui.ctx()
                    .request_repaint_after(deadline.saturating_duration_since(now));
            }
        }
    }

    fn update_diff(&mut self) {
        use crate::widgets::diff_editor::MAX_DIFF_INPUT_BYTES;

        self.cache.diff.update_deadline = None;

        let options = &self.state.options.diff;

        if options.left.len() > MAX_DIFF_INPUT_BYTES {
            self.cache.diff.error = Some("left input exceeds 1 MiB; diff was not updated".to_owned());
            return;
        }

        if options.right.len() > MAX_DIFF_INPUT_BYTES {
            self.cache.diff.error = Some("right input exceeds 1 MiB; diff was not updated".to_owned());
            return;
        }

        if options.left.is_empty() && options.right.is_empty() {
            self.cache.diff.result = None;
            self.cache.diff.aligned = None;
            return;
        }

        let structural_input_limit = options.language.structural_diff_input_limit();
        let structural_input_too_large = structural_input_limit.is_some_and(|limit| {
            options.left.len() > limit || options.right.len() > limit
        });

        let (virtual_path, fallback_message) = if structural_input_too_large {
            let limit = structural_input_limit.expect("limit must exist when fallback is enabled");

            (
                DiffLanguage::Text.virtual_path(),
                Some(format!(
                    "input exceeds {} KiB for this structural diff; showing a plain-text diff",
                    limit / 1024,
                )),
            )
        } else {
            (options.language.virtual_path(), None)
        };

        self.cache.diff.result = Some(difftastic::clipboard::diff_text(
            virtual_path,
            &options.left,
            &options.right,
            difftastic::clipboard::ClipboardDiffOptions {
                context_lines: options.context_lines,
                ignore_comments: options.ignore_comments,
            },
        ));

        self.state.options.diff.change_index = usize::MAX;
        self.state.options.diff.pending_change_delta = 0;
        self.cache.diff.aligned = None;
        self.cache.diff.layout.left.clear();
        self.cache.diff.layout.right.clear();
        self.cache.diff.error = fallback_message;
    }
}

pub fn handle_file_drop(
    ui: &Ui,
    left_response: &Response,
    right_response: &Response,
    left_text: &mut String,
    right_text: &mut String,
) -> bool {
    let dropped_files = ui.ctx().input(|input| input.raw.dropped_files.clone());

    if dropped_files.is_empty() {
        return false;
    }

    let target_text = if right_response.has_focus() {
        right_text
    } else if left_response.has_focus() {
        left_text
    } else {
        return false;
    };

    for file in dropped_files {
        if let Some(path) = file.path
            && let Ok(content) = std::fs::read_to_string(path)
        {
            *target_text = content;
            return true;
        }
    }

    false
}