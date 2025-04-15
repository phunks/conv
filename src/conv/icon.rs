use eframe::egui;
use eframe::egui::{ColorImage, Image, Response, Sense, Ui};
use eframe::egui::SizeHint::Size;
use egui_extras::image::load_svg_bytes_with_size;

pub struct LoadIcon {
    pub texture: Option<egui::TextureHandle>,
}

impl LoadIcon {
    pub fn ui(&mut self, ui: &mut Ui) -> Response {
        let texture: &egui::TextureHandle = self.texture.get_or_insert_with(|| {
            ui.ctx().load_texture(
                "copy_icon",
                load_copy_icon(ui.ctx().style().visuals.dark_mode),
                Default::default(),
            )
        });
        ui.add(Image::new((texture.id(), texture.size_vec2())).sense(Sense::click()))
    }
}

const COPY_ICON_LIGHT: &[u8; 4533] = include_bytes!("../../assets/icon_copy_light.svg");
const COPY_ICON_DARK: &[u8; 4533] = include_bytes!("../../assets/icon_copy_dark.svg");
fn load_copy_icon(dark: bool) -> ColorImage {
    if dark {
        return load_svg_bytes_with_size(COPY_ICON_DARK, Some(Size(21, 21))).unwrap();
    }
    load_svg_bytes_with_size(COPY_ICON_LIGHT, Some(Size(21, 21))).unwrap()
}
