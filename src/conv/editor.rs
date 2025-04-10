use crate::conv::converter::convert;
use crate::conv::enum_variants::{Base64Menu, BinaryMenu, Conv, DigestMenu, EscapeMenu, RegexMenu};
use crate::conv::modules::crypt::enum_crypt::{
    AesEncDec, AesMode, AesPadding, DecTextFormat, EncTextFormat, GcmTagLen,
};
use eframe::egui::{Align, Response, ScrollArea, TextBuffer, Ui};
use eframe::{egui, emath};
use strum::{EnumMessage, VariantArray};
use crate::conv::icon::LoadIcon;

#[derive(Default)]
pub struct Editor {
    pub code: String,
    pub menu: Selected,
    pub output: String,
    pub copy_buf: String,
    pub aes: AesStore,
    pub regex: RegexStore,
    pub jq: JqStore,
    cache: crate::conv::LayoutCache,
}

#[derive(Default)]
pub struct JqStore {
    /// filter
    pub filter: String,
}

#[derive(Default)]
pub struct RegexStore {
    /// regex
    pub re: String,
    /// text
    pub text: String,
    /// regex flag
    pub regex_flag: RegexFlag,
    /// grep flag
    pub grep_flag: GrepFlag,
}

#[derive(Default)]
pub struct RegexFlag {
    /// single line
    pub single: bool,
    /// match/replace
    pub replace: bool,
    /// ignore case
    pub ignore: bool,
}

#[derive(better_default::Default)]
pub struct GrepFlag {
    /// invert
    pub invert: bool,
    /// ignore case
    #[default(true)]
    pub ignore: bool,
}

#[derive(Default)]
pub struct AesStore {
    /// initial vec
    pub iv: String,
    /// secret key
    pub key: String,
    #[allow(unused)]
    /// encrypt text
    pub text: String,
}

#[derive(Default)]
pub struct AesCipher {
    /// encryption, decryption
    pub encdec: AesEncDec,
    /// cipher mode
    pub mode: AesMode,
    /// padding
    pub pad: AesPadding,
    /// encryption output text format
    pub enc_fmt: EncTextFormat,
    /// decryption output text format
    pub dec_fmt: DecTextFormat,
    /// gcm tag len
    pub tag: GcmTagLen,
}

#[derive(Default)]
pub struct Selected {
    pub converter: Conv,
    pub digest: DigestMenu,
    pub base64: Base64Menu,
    pub binary: BinaryMenu,
    pub escape: EscapeMenu,
    pub aes: AesCipher,
    pub regex: RegexMenu,
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

    pub fn ui(&mut self, ui: &mut Ui) {
        let Self { menu, .. } = self;
        ui.horizontal(|ui| {
            combobox::<Conv>(ui, "converter", &mut menu.converter);

            match menu.converter {
                Conv::Crypt => {
                    combobox::<DigestMenu>(ui, "crypt", &mut menu.digest);
                    match menu.digest {
                        DigestMenu::Aes128 | DigestMenu::Aes192 | DigestMenu::Aes256 => {
                            combobox::<AesMode>(ui, "mode", &mut menu.aes.mode);
                            match menu.aes.mode {
                                // AesMode::Ecb | AesMode::Cbc => {
                                //     combobox::<AesPadding>(ui, "pad", &mut menu.aes.pad)
                                // }
                                AesMode::Gcm => combobox::<GcmTagLen>(ui, "tag", &mut menu.aes.tag),
                                _ => {}
                            }
                            combobox::<AesEncDec>(ui, "enc/dec", &mut menu.aes.encdec);
                            match menu.aes.encdec {
                                AesEncDec::AesEnc => {
                                    combobox::<EncTextFormat>(ui, "enc fmt", &mut menu.aes.enc_fmt)
                                }
                                AesEncDec::AesDec => {
                                    combobox::<DecTextFormat>(ui, "dec fmt", &mut menu.aes.dec_fmt)
                                }
                            }
                        }
                        _ => {}
                    }
                }
                Conv::Base64 => combobox::<Base64Menu>(ui, "base64", &mut menu.base64),
                Conv::Binary => combobox::<BinaryMenu>(ui, "binary", &mut menu.binary),
                Conv::Escape => combobox::<EscapeMenu>(ui, "escape", &mut menu.escape),
                Conv::Regex => {
                    combobox::<RegexMenu>(ui, "regex", &mut menu.regex);

                    match menu.regex {
                        RegexMenu::Regex => {
                            self.regex.regex_flag.single =
                                checkbox(ui, self.regex.regex_flag.single, "single line");
                            self.regex.regex_flag.replace =
                                checkbox(ui, self.regex.regex_flag.replace, "replace");
                            self.regex.regex_flag.ignore =
                                checkbox(ui, self.regex.regex_flag.ignore, "ignore case");
                        }
                        RegexMenu::Grep => {
                            self.regex.grep_flag.invert =
                                checkbox(ui, self.regex.grep_flag.invert, "invert");
                            self.regex.grep_flag.ignore =
                                checkbox(ui, self.regex.grep_flag.ignore, "ignore case");
                        }
                    }
                },
                Conv::Jq => {

                }
            }

            ui.with_layout(egui::Layout::right_to_left(Align::RIGHT), |ui| {
                let mut icon = LoadIcon { texture: None };

                let response = icon.ui(ui).on_hover_text("swap text");
                if response.clicked() && !self.output.is_empty() {
                    self.code = self.output.clone();
                }
            });
        });

        ui.separator();

        ui.columns(2, |columns| {
            let left_pain = egui::Frame::new().show(&mut columns[0], |ui| {
                ScrollArea::vertical().id_salt("source").show(ui, |ui| {
                    let rows = self.inputbox_ui(ui);
                    self.editor_ui(ui, rows);
                });
            });

            ScrollArea::vertical()
                .id_salt("rendered")
                .show(&mut columns[1], |ui| {
                    ui.set_min_width(window_size(left_pain.response.rect).x);
                    convert(ui, self);
                });
        });
    }

    fn inputbox_ui(&mut self, ui: &mut Ui) -> usize {
        let Self {
            menu, aes, regex, jq, ..
        } = self;

        let pos = window_size(ui.ctx().input(|i| i.viewport().outer_rect).unwrap());
        match menu.converter {
            Conv::Crypt => match menu.digest {
                DigestMenu::Aes128 | DigestMenu::Aes192 | DigestMenu::Aes256 => {
                    inputbox(ui, "secret key", &mut aes.key);
                    inputbox(ui, "initial vector", &mut aes.iv);
                }
                _ => return ((pos.y - 78.5) / 14.05).round() as usize,
            },
            Conv::Regex => match menu.regex {
                RegexMenu::Regex => {
                    inputbox(ui, "regex\n(?-ismx)", &mut regex.re);
                    inputbox(ui, "replace text", &mut regex.text);
                }
                RegexMenu::Grep => {
                    inputbox(ui, "match regex", &mut regex.re);
                    return ((pos.y - 100.5) / 14.05).round() as usize;
                }
            },
            Conv::Jq => {
                inputbox(ui, "jq filter", &mut jq.filter);
                return ((pos.y - 100.5) / 14.05).round() as usize;
            }
            _ => {
                menu.digest = Default::default();
                return ((pos.y - 78.5) / 14.05).round() as usize;
            }
        }

        ((pos.y - 123.) / 14.05).round() as usize
    }

    fn editor_ui(&mut self, ui: &mut Ui, rows: usize) {
        let Self { code, cache, .. } = self;

        let mut layouter = |ui: &Ui, text: &str, wrap_width: f32| {
            let mut layout_job = cache.memorise(ui.style(), text);
            layout_job.wrap.max_width = wrap_width;
            ui.fonts(|f| f.layout_job(layout_job))
        };

        let _response = ui.add(
            egui::TextEdit::multiline(code)
                .id_salt("textedit")
                .desired_rows(rows)
                .desired_width(f32::INFINITY)
                .font(egui::TextStyle::Monospace)
                .layouter(&mut layouter),
        );
    }
}

fn window_size(rect: emath::Rect) -> egui::Pos2 {
    (rect.max - rect.min).to_pos2()
}

#[allow(dead_code)]
fn selected_text(ui: &mut Ui, res: Response, code: &str) -> Option<String> {
    if let Some(state) = egui::TextEdit::load_state(ui.ctx(), res.id) {
        if let Some(ccursor_range) = state.cursor.char_range() {
            let [primary, secondary] = ccursor_range.sorted();
            if primary != secondary {
                return match code.char_indices().nth(primary.index) {
                    None => None,
                    Some(a) => code
                        .char_indices()
                        .nth(secondary.index)
                        .map(|b| code[a.0..b.0].to_string()),
                };
            }
        }
    };
    None
}

#[allow(unused)]
fn show_example_multiline_hover(ui: &mut Ui) {
    let mut text = "Hello world!\nI can do tooltips!\nPretty neat.";
    let text_edit_output = egui::TextEdit::multiline(&mut text).show(ui);
    let hover_pos = ui.input(|i| i.pointer.hover_pos());
    if let Some(hover_pos) = hover_pos {
        if text_edit_output.response.rect.contains(hover_pos) {
            let hover_pos = hover_pos - text_edit_output.response.rect.left_top();
            let hover_cursor = text_edit_output.galley.cursor_from_pos(hover_pos).pcursor;
            if let Some(line) = text.lines().nth(hover_cursor.paragraph) {
                egui::show_tooltip_at_pointer(
                    ui.ctx(),
                    ui.layer_id(),
                    egui::Id::new("hover tooltip"),
                    |ui| {
                        ui.label(line);
                    },
                );
            }
        }
    }
}

fn inputbox(ui: &mut Ui, salt: &str, text: &mut dyn TextBuffer) {
    ui.add(
        egui::TextEdit::multiline(text)
            .id_salt(salt)
            .desired_width(32.0)
            .desired_rows(1)
            // .font(egui::TextStyle::Monospace),
            .font(egui::FontId {
                size: 12.5,
                family: egui::FontFamily::Proportional,
            }),
    )
    .on_hover_text(salt);
}

fn checkbox(ui: &mut Ui, mut check: bool, text: &str) -> bool {
    ui.checkbox(&mut check, text).changed();
    check
}

fn combobox<T>(ui: &mut Ui, salt: &str, var: &mut T)
where
    T: EnumMessage + VariantArray + PartialEq + Clone,
{
    egui::ComboBox::from_id_salt(salt)
        .width(0.0)
        .selected_text(var.get_message().unwrap())
        .show_ui(ui, |ui| {
            for v in T::VARIANTS {
                ui.selectable_value(var, v.clone(), v.get_message().unwrap())
                    .on_hover_text(v.get_documentation().unwrap());
            }
        })
        .response
        .on_hover_text(var.get_documentation().unwrap());
}

// use egui::text::LayoutJob;
//
// impl<T: Editor> egui::util::cache::ComputerMut<(&T, &str), LayoutJob> for Token {
//     fn compute(&mut self, (cache, text): (&T, &str)) -> LayoutJob {
//         self.highlight(cache, text)
//     }
// }
//
// pub type HighlightCache = egui::util::cache::FrameCache<LayoutJob, Token>;
//
// pub fn highlight<T: Editor>(ctx: &egui::Context, cache: &T, text: &str) -> LayoutJob {
//     ctx.memory_mut(|mem| mem.caches.cache::<HighlightCache>().get((cache, text)))
// }