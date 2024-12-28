use crate::conv::Editor;
use eframe::egui::SizeHint::Size;
use eframe::egui::{Context, FontData, FontDefinitions, FontFamily};
use eframe::epaint::ColorImage;
use eframe::{egui, Frame};
use egui_extras::image::load_svg_bytes_with_size;

mod conv;

const CONV_ICON: &[u8; 2132] = include_bytes!("../assets/icon_conv.svg");
fn main() -> eframe::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let icon = load_svg_bytes_with_size(CONV_ICON, Option::from(Size(128, 128))).unwrap();
    let size = icon.width() as u32;

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([500.0, 193.0])
            .with_resizable(false)
            .with_icon(egui::IconData {
                rgba: Vec::from(icon.as_raw()),
                width: size,
                height: size,
            }),
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

pub fn load_image_from_memory(image_data: &[u8]) -> Result<ColorImage, image::ImageError> {
    let image = image::load_from_memory(image_data)?;
    let size = [image.width() as _, image.height() as _];
    let image_buffer = image.to_rgba8();
    let pixels = image_buffer.as_flat_samples();
    Ok(ColorImage::from_rgba_unmultiplied(size, pixels.as_slice()))
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
        .insert("hackgen".to_owned(), FontData::from_static(aa).into());
    fonts
        .families
        .get_mut(&FontFamily::Proportional)
        .unwrap()
        .insert(0, "hackgen".to_owned());
    ctx.set_fonts(fonts);
}
