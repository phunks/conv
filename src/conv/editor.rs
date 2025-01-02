
use crate::conv::enum_variants::{Base64Kind, BinaryKind, Conv, Digest, EscapeKind};
use eframe::egui;
use eframe::egui::SizeHint::Size;
use eframe::egui::{Align, Image, Response, ScrollArea, Sense, Ui};
use eframe::epaint::ColorImage;
use egui_extras::image::load_svg_bytes_with_size;
use strum::{EnumMessage, VariantArray};

#[derive(Default)]
pub struct Editor {
    pub code: String,
    pub menu: Selected,
    pub text: String,
    cache: crate::conv::LayoutCache,
}

#[derive(Default)]
pub struct Selected {
    pub converter: Conv,
    pub digest: Digest,
    pub base64: Base64Kind,
    pub binary: BinaryKind,
    pub escape: EscapeKind,
}

impl PartialEq for Editor {
    fn eq(&self, other: &Self) -> bool {
        self.code == other.code
    }
}

impl Editor {
    pub fn panels(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.ui(ui);
        });
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) {
        let Self { menu, .. } = self;

        ui.horizontal(|ui| {
            combobox::<Conv>(ui, "converter", &mut menu.converter);

            match menu.converter {
                Conv::Crypt => {
                    combobox::<Digest>(ui, "crypt", &mut menu.digest);
                },
                Conv::Base64 => {
                    combobox::<Base64Kind>(ui, "base64", &mut menu.base64);
                },
                Conv::Binary => {
                    combobox::<BinaryKind>(ui, "binary", &mut menu.binary);
                },
                Conv::Escape => {
                    combobox::<EscapeKind>(ui, "escape", &mut menu.escape);
                },
            }

            ui.with_layout(egui::Layout::right_to_left(Align::RIGHT), |ui| {
                let mut icon = LoadIcon { texture: None };

                let response = icon.ui(ui);
                if response.clicked() {
                    self.code = self.text.clone();
                }
            })
        });

        ui.separator();

        ui.columns(2, |columns| {
            ScrollArea::vertical()
                .id_salt("source")
                .show(&mut columns[0], |ui| self.editor_ui(ui));
            ScrollArea::vertical()
                .id_salt("rendered")
                .show(&mut columns[1], |ui| {
                    ui.set_min_width(240.0);
                    crate::conv::convert(ui, self);
                });
        });
    }

    fn editor_ui(&mut self, ui: &mut egui::Ui) {
        let Self { code, cache, .. } = self;

        let mut layouter = |ui: &egui::Ui, text: &str, wrap_width: f32| {
            let mut layout_job = cache.memorise(ui.style(), text);
            layout_job.wrap.max_width = wrap_width;
            ui.fonts(|f| f.layout_job(layout_job))
        };

        ui.add(
            egui::TextEdit::multiline(code)
                .desired_rows(10)
                .desired_width(f32::INFINITY)
                .font(egui::TextStyle::Monospace) // for cursor height
                .layouter(&mut layouter),
        );
    }
}

fn combobox<T>(ui: &mut Ui, salt: &str, var: &mut T)
where
    T: EnumMessage + VariantArray + PartialEq + Clone,
{
    egui::ComboBox::from_id_salt(salt)
        .selected_text(var.get_message().unwrap())
        .show_ui(ui, |ui| {
            for v in T::VARIANTS {
                ui.selectable_value(var, v.clone(), v.get_message().unwrap())
                    .on_hover_ui(|ui| {
                        ui.style_mut().interaction.selectable_labels = true;
                        ui.label(v.get_documentation().unwrap());
                    });
            }
        });
}

struct LoadIcon {
    texture: Option<egui::TextureHandle>,
}

impl LoadIcon {
    fn ui(&mut self, ui: &mut egui::Ui) -> Response {
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
