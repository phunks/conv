
use eframe::egui::{Align, Button, CentralPanel, Context, Layout, ScrollArea, Ui, Visuals};
use crate::app::cache::AppCache;
use crate::app::state::AppState;
use crate::converters::Converters;

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
                let enabled = match selected {
                    crate::app::state::SelectedTool::Data => {
                        !self.state.options.data.filter.is_empty()
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
                        crate::app::state::SelectedTool::Data => {
                            self.converters.jq.copy_output_text(&self.state)
                        }
                        _ => self
                            .cache
                            .outputs
                            .current(selected)
                            .result
                            .plain_text_owned(),
                    };

                    if let Some(output) = output {
                        self.state.input = output;
                        self.cache.outputs.mark_all_dirty();
                    }
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
                        if self.state.selected == crate::app::state::SelectedTool::Data {
                            self.converters.jq.render(ui, &self.state);
                            return;
                        }

                        self.ensure_output();

                        let selected = self.state.selected;
                        let result = &self.cache.outputs.current(selected).result;
                        result.render(ui, &mut self.cache.output_layout);
                    },
                );
            });
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
}

