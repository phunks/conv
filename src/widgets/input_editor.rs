
use eframe::egui::{FontFamily, FontId, ScrollArea, Style, TextEdit, TextFormat, TextStyle, Ui};
use eframe::egui::text::LayoutJob;
use crate::app::state::{AppState, SelectedTool};
use crate::converters::crypt::{AesMode, DigestMenu};
use crate::widgets::menu::{InputFormat, RegexMenu};

#[derive(Default, Clone)]
pub struct LayoutCache {
    style: Style,
    text: String,
    output: LayoutJob,
}

impl LayoutCache {
    pub fn memorise(&mut self, style: &Style, text: &str) -> LayoutJob {
        if (&self.style, self.text.as_str()) != (style, text) {
            self.style = style.clone();
            text.clone_into(&mut self.text);
            self.output = layout_job(style, text);
        }

        self.output.clone()
    }
}


pub fn input_editor_ui(ui: &mut Ui, state: &mut AppState, cache: &mut LayoutCache) -> bool {
    let mut changed = option_inputs_ui(ui, state);

    ui.add_space(ui.spacing().item_spacing.y);

    let editor_changed = ScrollArea::vertical()
        .id_salt("source")
        .auto_shrink([false, true])
        .show(ui, |ui| {
            let desired_size = ui.available_size();

            let mut layouter = |ui: &Ui, text: &str, wrap_width: f32| {
                let mut layout_job = cache.memorise(ui.style(), text);
                layout_job.wrap.max_width = wrap_width;
                ui.fonts(|fonts| fonts.layout_job(layout_job))
            };

            ui.add_sized(
                desired_size,
                TextEdit::multiline(&mut state.input)
                    .id_salt("input.text")
                    .desired_width(f32::INFINITY)
                    .font(TextStyle::Monospace)
                    .layouter(&mut layouter),
            )
                .changed()
        })
        .inner;

    changed |= editor_changed;
    changed
}

fn option_inputs_ui(ui: &mut Ui, state: &mut AppState) -> bool {
    match state.selected {
        SelectedTool::Crypt => crypt_inputs_ui(ui, state),
        SelectedTool::Regex => regex_inputs_ui(ui, state),
        SelectedTool::Data => jq_inputs_ui(ui, state),
        SelectedTool::Base64 | SelectedTool::Binary | SelectedTool::Escape => false,
    }
}

fn crypt_inputs_ui(ui: &mut Ui, state: &mut AppState) -> bool {
    match state.options.crypt.digest {
        DigestMenu::Aes128 | DigestMenu::Aes192 | DigestMenu::Aes256 => {
            let mut changed = false;

            changed |= inputbox(ui, "input.crypt.key", "key", &mut state.options.crypt.key);

            if state.options.crypt.aes_mode != AesMode::Ecb {
                changed |= inputbox(ui, "input.crypt.iv", "iv", &mut state.options.crypt.iv);
            }

            changed
        }
        DigestMenu::Md5
        | DigestMenu::Sha1
        | DigestMenu::Sha224
        | DigestMenu::Sha256
        | DigestMenu::Sha384
        | DigestMenu::Sha512 => false,
    }
}

fn regex_inputs_ui(ui: &mut Ui, state: &mut AppState) -> bool {
    match state.options.regex.mode {
        RegexMenu::Regex => {
            let mut changed = false;

            changed |= inputbox(
                ui,
                "input.regex.pattern",
                "regex\n(?-ismx)",
                &mut state.options.regex.pattern,
            );

            if state.options.regex.replace_enabled {
                changed |= inputbox(
                    ui,
                    "input.regex.replace",
                    "replace text",
                    &mut state.options.regex.replace,
                );
            }

            changed
        }
        RegexMenu::Grep => inputbox(
            ui,
            "input.regex.pattern",
            "match regex",
            &mut state.options.regex.pattern,
        ),
    }
}

fn jq_inputs_ui(ui: &mut Ui, state: &mut AppState) -> bool {
    if state.options.data.input_format != InputFormat::Json {
        return false;
    }

    inputbox(
        ui,
        "input.data.filter",
        "jq filter",
        &mut state.options.data.filter,
    )
}

fn inputbox(ui: &mut Ui, salt: &str, tooltip: &str, text: &mut String) -> bool {
    ui.add(
        TextEdit::singleline(text)
            .id_salt(salt)
            .desired_width(f32::INFINITY)
            .font(FontId {
                size: 13.0,
                family: FontFamily::Proportional,
            }),
    )
        .on_hover_text(tooltip)
        .changed()
}

fn layout_job(style: &Style, text: &str) -> LayoutJob {
    let mut job = LayoutJob::default();

    if !text.is_empty() {
        job.append(
            text,
            0.0,
            TextFormat {
                font_id: TextStyle::Monospace.resolve(style),
                ..Default::default()
            },
        );
    }

    job
}