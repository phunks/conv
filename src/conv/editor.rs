
use crate::conv::enum_variants::{Conv, Digest};
use eframe::egui;
use eframe::egui::{ ScrollArea};
use strum::{EnumMessage, VariantArray};

pub struct Editor {
    code: String,
    converter: Conv,
    digest: Digest,
    highlighter: crate::conv::LayoutCache,
}

impl PartialEq for Editor {
    fn eq(&self, other: &Self) -> bool {
        self.code == other.code
    }
}

impl Default for Editor {
    fn default() -> Self {
        Self {
            code: "".to_owned(),
            converter: Conv::ToBase64,
            digest: Digest::Md5,
            highlighter: Default::default(),
        }
    }
}

impl Editor {
    pub fn panels(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.ui(ui);
        });
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) {
        let Self { converter, digest, .. } = self;

        let conv_name = |c: Conv| c.get_message().unwrap();
        let conv_comment = |c: Conv| c.get_documentation().unwrap();
        let crypt_name = |c: Digest| c.get_message().unwrap();
        let crypt_comment = |c: Digest| c.get_documentation().unwrap();

        ui.horizontal(|ui| {
            egui::ComboBox::from_id_salt("converter")
                .selected_text(conv_name(*converter))
                .show_ui(ui, |ui| {
                    for conv in Conv::VARIANTS {
                        ui.selectable_value(converter, *conv, conv_name(*conv)).on_hover_ui(|ui| {
                            ui.style_mut().interaction.selectable_labels = true;
                            ui.label(conv_comment(*conv));
                        });
                    }
                });
            match converter {
                Conv::Crypt => {
                    egui::ComboBox::from_id_salt("crypt")
                        .selected_text(crypt_name(*digest))
                        .show_ui(ui, |ui| {
                            for crypt in Digest::VARIANTS {
                                ui.selectable_value(digest, *crypt, crypt_name(*crypt)).on_hover_ui(|ui| {
                                    ui.style_mut().interaction.selectable_labels = true;
                                    ui.label(crypt_comment(*crypt));
                                });
                            }
                        });
                },
                _ => {}
            }
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
                    crate::conv::convert(ui, &self.code, &self.converter, &self.digest);
                });
        });
    }

    fn editor_ui(&mut self, ui: &mut egui::Ui) {
        let Self {
            code, highlighter, ..
        } = self;

        let mut layouter = |ui: &egui::Ui, text: &str, wrap_width: f32| {
            let mut layout_job = highlighter.memorise(ui.style(), text);
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
