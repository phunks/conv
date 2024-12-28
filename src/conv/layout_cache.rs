use eframe::egui;
use egui::TextStyle;

#[derive(Default)]
pub struct LayoutCache {
    style: egui::Style,
    code: String,
    output: egui::text::LayoutJob,
}

impl LayoutCache {
    pub fn memorise(&mut self, egui_style: &egui::Style, code: &str) -> egui::text::LayoutJob {
        if (&self.style, self.code.as_str()) != (egui_style, code) {
            self.style = egui_style.clone();
            code.clone_into(&mut self.code);
            self.output = layout_job(egui_style, code);
        }
        self.output.clone()
    }
}

pub fn layout_job(egui_style: &egui::Style, text: &str) -> egui::text::LayoutJob {
    let mut job = egui::text::LayoutJob::default();

    if !text.is_empty() {
        job.append(text, 0.0, format_from_style(egui_style));
    }
    job
}

fn format_from_style(egui_style: &egui::Style) -> egui::text::TextFormat {
    egui::text::TextFormat {
        font_id: TextStyle::Body.resolve(egui_style),
        ..Default::default()
    }
}
