use eframe::egui;
use eframe::egui::TextFormat;
use eframe::egui::text::LayoutJob;
use egui::TextStyle;

#[derive(Default)]
pub struct LayoutCache {
    style: egui::Style,
    code: String,
    output: LayoutJob,
}

impl LayoutCache {
    pub fn memorise(&mut self, egui_style: &egui::Style, code: &str) -> LayoutJob {
        if (&self.style, self.code.as_str()) != (egui_style, code) {
            self.style = egui_style.clone();
            code.clone_into(&mut self.code);
            self.output = layout_job(egui_style, code);
        }
        self.output.clone()
    }
}

pub fn layout_job(egui_style: &egui::Style, text: &str) -> LayoutJob {
    let mut job = LayoutJob::default();

    if !text.is_empty() {
        job.append(text, 0.0, format_from_style(egui_style));
    }
    job
}

fn format_from_style(egui_style: &egui::Style) -> TextFormat {
    TextFormat {
        font_id: TextStyle::Body.resolve(egui_style),
        ..Default::default()
    }
}
