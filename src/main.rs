use crate::conv::Editor;
use eframe::egui::{Context, FontData, FontDefinitions, FontFamily};
use eframe::{egui, Frame};

mod conv;

fn main() -> eframe::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([500.0, 190.0]),
        ..Default::default()
    };
    eframe::run_native(
        "conv",
        options,
        Box::new(|cc| {
            add_font(&cc.egui_ctx);
            Ok(Box::<Editor>::default())
        }),
    )
}

impl eframe::App for Editor {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        self.panels(ctx);
    }
}

fn add_font(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();
    let aa = include_bytes!("../assets/HackGen-Regular.ttf");
    fonts
        .font_data
        .insert("hackgen".to_owned(), FontData::from_static(aa));
    fonts
        .families
        .get_mut(&FontFamily::Proportional)
        .unwrap()
        .insert(0, "hackgen".to_owned());
    ctx.set_fonts(fonts);
}
