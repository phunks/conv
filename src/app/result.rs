use eframe::egui::{text, vec2, Align, Color32, Layout, TextStyle, Ui};
use crate::app::cache::TextGalleyCache;

#[derive(Clone, Default)]
pub enum ConvertResult {
    #[default]
    Empty,
    Text(String),
    Error(String),
    #[allow(unused)]
    TextWithWarnings {
        text: String,
        warnings: Vec<String>,
    },
    RichText(text::LayoutJob),
    Warnings(Vec<String>),
}

impl ConvertResult {
    pub fn has_non_empty_plain_text(&self) -> bool {
        match self {
            ConvertResult::Text(text) => !text.is_empty(),
            ConvertResult::TextWithWarnings { text, .. } => !text.is_empty(),
            ConvertResult::RichText(job) => !job.text.is_empty(),
            ConvertResult::Empty | ConvertResult::Error(_) | ConvertResult::Warnings(_) => false,
        }
    }

    pub fn plain_text_owned(&self) -> Option<String> {
        match self {
            ConvertResult::Text(text) => Some(text.clone()),
            ConvertResult::TextWithWarnings { text, .. } => Some(text.clone()),
            ConvertResult::RichText(job) => Some(job.text.clone()),
            ConvertResult::Empty | ConvertResult::Error(_) | ConvertResult::Warnings(_) => None,
        }
    }

    pub fn render(&self, ui: &mut Ui, _cache: &mut TextGalleyCache) {
        match self {
            ConvertResult::Empty => {}
            ConvertResult::Text(text) => {
                render_old_style_text(ui, text);
            }
            ConvertResult::Error(error) => {
                render_warning_line(ui, error);
            }
            ConvertResult::TextWithWarnings { text, warnings } => {
                render_old_style_text(ui, text);

                for warning in warnings {
                    render_warning_line(ui, warning);
                }
            }
            ConvertResult::RichText(job) => {
                render_old_style_rich_text(ui, job);
            }
            ConvertResult::Warnings(warnings) => {
                for warning in warnings {
                    render_warning_line(ui, warning);
                }
            }
        }
    }
}

fn render_old_style_text(ui: &mut Ui, text: &str) {
    let initial_size = vec2(
        ui.available_width(),
        ui.spacing().interact_size.y,
    );

    let layout = Layout::left_to_right(Align::BOTTOM)
        .with_main_wrap(true)
        .with_main_justify(false);

    ui.allocate_ui_with_layout(initial_size, layout, |ui| {
        ui.spacing_mut().item_spacing.x = 0.0;

        let row_height = ui.text_style_height(&TextStyle::Body);
        ui.set_row_height(row_height);

        ui.label(text);
    });
}

fn render_old_style_rich_text(ui: &mut Ui, job: &text::LayoutJob) {
    let initial_size = vec2(
        ui.available_width(),
        ui.spacing().interact_size.y,
    );

    let layout = Layout::left_to_right(Align::BOTTOM)
        .with_main_wrap(true)
        .with_main_justify(false);

    ui.allocate_ui_with_layout(initial_size, layout, |ui| {
        ui.spacing_mut().item_spacing.x = 0.0;

        let row_height = ui.text_style_height(&TextStyle::Body);
        ui.set_row_height(row_height);

        ui.label(job.clone());
    });
}

fn render_warning_line(ui: &mut Ui, warning: &str) {
    let warning = warning.strip_prefix("warn: ").unwrap_or(warning);

    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.colored_label(Color32::ORANGE, "warn: ");
        ui.colored_label(Color32::from_rgb(180, 190, 120), warning);
    });
}