
use crate::conv::enum_variants::{Base64Kind, BinaryKind, Conv, Digest, EscapeKind};
use eframe::egui;
use eframe::egui::{ScrollArea, Ui};
use strum::{EnumMessage, VariantArray};

#[derive(Default)]
pub struct Editor {
    pub code: String,
    pub menu: Selected,
    highlighter: crate::conv::LayoutCache,
}

#[derive(Default)]
pub struct Selected {
    pub converter: Conv,
    pub digest: Digest,
    pub base64: Base64Kind,
    pub binary: BinaryKind,
    pub escape: EscapeKind
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
                    combobox::<BinaryKind>(ui, "base64", &mut menu.binary);
                },
                Conv::Escape => {
                    combobox::<EscapeKind>(ui, "base64", &mut menu.escape);
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
                    crate::conv::convert(ui, self);
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

fn combobox<T>(ui: &mut Ui, salt: &str, var: &mut T)
    where T: EnumMessage + VariantArray + PartialEq + Clone
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