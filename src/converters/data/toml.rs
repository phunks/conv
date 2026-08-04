use eframe::egui::text::LayoutJob;
use eframe::egui::{Color32, FontFamily, FontId, TextFormat};

fn append_toml_text(job: &mut LayoutJob, text: &str) {
    if text.is_empty() {
        return;
    }

    job.append(
        text,
        0.0,
        TextFormat {
            font_id: FontId::new(11.5, FontFamily::Monospace),
            color: Color32::GRAY,
            ..Default::default()
        },
    );
}