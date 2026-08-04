#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
extern crate core;

use eframe::egui::{Context, FontData, FontDefinitions, FontFamily};
use eframe::egui;
use eframe::egui::SizeHint::Size;
use egui_extras::image::load_svg_bytes_with_size;

mod app;
mod converters;
mod widgets;
mod util;
use crate::app::ui::App;


const CONV_ICON: &[u8; 2132] = include_bytes!("../assets/icon_conv.svg");
fn main() -> eframe::Result {
    env_logger::init();

    let icon = load_svg_bytes_with_size(CONV_ICON, Option::from(Size(128, 128))).unwrap();
    let size = icon.width() as u32;

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
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
            Ok(Box::<App>::default())
        }),
    )
}

fn add_font(ctx: &Context) {
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

    fonts
        .families
        .get_mut(&FontFamily::Monospace)
        .unwrap()
        .push("hackgen".to_owned());

    ctx.set_fonts(fonts);
}
