use eframe::egui::{ComboBox, Ui};
use strum::{EnumMessage, VariantArray};
use crate::app::state::{AppState, SelectedTool};
use crate::converters::base64::Base64Options;
use crate::converters::binary::BinaryOptions;
use crate::converters::escape::EscapeOptions;
use crate::converters::data::DataOptions;
use crate::converters::regex;
use crate::converters::crypt::{
    AesEncDec, AesMode, CryptOptions, CryptOutputFormat, DigestMenu,
};

#[derive(Default, Copy, Clone, Debug, PartialEq, Eq, VariantArray, EnumMessage)]
#[strum(serialize_all = "kebab-case")]
pub enum Conv {
    #[default]
    /// modules
    #[strum(message = "Base64")]
    Base64,
    /// binary
    #[strum(message = "Binary")]
    Binary,
    /// escape
    #[strum(message = "Escape")]
    Escape,
    /// crypt
    #[strum(message = "Crypt")]
    Crypt,
    /// regex
    #[strum(message = "Regex")]
    Regex,
    /// structured-data converter
    #[strum(message = "Data")]
    Data,
    /// clipboard structural diff
    #[strum(message = "Diff")]
    Diff,
}

#[derive(Default, Copy, Clone, Debug, PartialEq, Eq, VariantArray, EnumMessage)]
pub enum InputFormat {
    #[default]
    /// input Json
    #[strum(message = "JSON")]
    Json,
    /// input Toml
    #[strum(message = "TOML")]
    Toml,
    /// input Yaml
    #[strum(message = "YAML")]
    Yaml,
    /// input Csv
    #[strum(message = "CSV")]
    Csv,
}

#[derive(Default, Copy, Clone, Debug, PartialEq, Eq, VariantArray, EnumMessage)]
pub enum OutputFormat {
    #[default]
    /// output Json
    #[strum(message = "JSON")]
    Json,
    /// output Toml
    #[strum(message = "TOML")]
    Toml,
    /// output Yaml
    #[strum(message = "YAML")]
    Yaml,
    /// output Csv
    #[strum(message = "CSV")]
    Csv,
}

impl From<SelectedTool> for Conv {
    fn from(value: SelectedTool) -> Self {
        match value {
            SelectedTool::Base64 => Self::Base64,
            SelectedTool::Binary => Self::Binary,
            SelectedTool::Escape => Self::Escape,
            SelectedTool::Crypt => Self::Crypt,
            SelectedTool::Regex => Self::Regex,
            SelectedTool::Data => Self::Data,
            SelectedTool::Diff => Self::Diff,
        }
    }
}

impl From<Conv> for SelectedTool {
    fn from(value: Conv) -> Self {
        match value {
            Conv::Base64 => Self::Base64,
            Conv::Binary => Self::Binary,
            Conv::Escape => Self::Escape,
            Conv::Crypt => Self::Crypt,
            Conv::Regex => Self::Regex,
            Conv::Data => Self::Data,
            Conv::Diff => Self::Diff,
        }
    }
}

#[derive(Default, Copy, Clone, Debug, PartialEq, Eq, VariantArray, EnumMessage)]
#[strum(serialize_all = "kebab-case")]
pub enum RegexMenu {
    #[default]
    /// regex replace
    #[strum(message = "Regex")]
    Regex,
    /// regex grep
    #[strum(message = "Grep")]
    Grep,
}

#[derive(Default, Copy, Clone, Debug, PartialEq, Eq, VariantArray, EnumMessage)]
#[strum(serialize_all = "kebab-case")]
pub enum Base64Menu {
    #[default]
    /// to base64 (rfc 4648)
    #[strum(message = "To Base64")]
    ToBase64,
    /// to base64 url (rfc 4648 url safe)
    #[strum(message = "To Base64URL")]
    ToBase64Url,
    /// from base64
    #[strum(message = "From Base64")]
    FromBase64,
    /// to deflated saml auth
    #[strum(message = "To Deflated Saml")]
    ToDeflatedSaml,
    /// from deflated saml auth
    #[strum(message = "From Deflated Saml")]
    FromDeflatedSaml,
}

#[derive(Default, Copy, Clone, Debug, PartialEq, Eq, VariantArray, EnumMessage)]
#[strum(serialize_all = "kebab-case")]
pub enum BinaryMenu {
    #[default]
    /// from UTF-8 to hex
    /// ex: '𝕊☺a' = 'f09d958ae298ba61'
    #[strum(message = "Hex Encode")]
    HexEncode,
    /// from hex to UTF-8
    /// ex: 'f09d958ae298ba61' = '𝕊☺a'
    #[strum(message = "Hex Decode")]
    HexDecode,
    /// to byte string
    /// ex: '𝕊☺a' = '0xf0, 0x9d, 0x95, 0x8a, 0xe2, 0x98, 0xba, 0x61'
    #[strum(message = "To byte string")]
    ToByteString,
    /// from byte string
    /// ex: '0xf0, 0x9d, 0x95, 0x8a, 0xe2, 0x98, 0xba, 0x61' = '𝕊☺a'
    #[strum(message = "From byte string")]
    FromByteString,
    /// to decimal character code string
    /// ex: '𝕊☺a' = '120138 9786 97'
    #[strum(message = "To Decimal String")]
    ToDecimalString,
    /// from decimal character code string
    /// ex: '120138 9786 97' = '𝕊☺a'
    #[strum(message = "From Decimal String")]
    FromDecimalString,
}

#[derive(Default, Copy, Clone, Debug, PartialEq, Eq, VariantArray, EnumMessage)]
#[strum(serialize_all = "kebab-case")]
pub enum EscapeMenu {
    #[default]
    /// url encode
    /// ex: '𝕊☺a' = '%F0%9D%95%8A%E2%98%BAa'
    #[strum(message = "Url Encode")]
    UrlEncode,
    /// url decode
    /// ex: '%F0%9D%95%8A%E2%98%BAa' = '𝕊☺a'
    #[strum(message = "Url Decode")]
    UrlDecode,
    /// to js string (JS6)
    /// ex: '𝕊☺a' = '\u{1d54a}\u{263a}\u{61}'
    #[strum(message = "To JS String")]
    ToJsString,
    /// from js string
    /// ex: '\u{1d54a}\u{263a}\u61' = '𝕊☺a'
    #[strum(message = "From JS String")]
    FromJsString,
    /// to HTML numeric entities
    /// ex: '𝕊☺a' = '&#x1d54a, &#x263a, &#x61'
    #[strum(message = "To Html Numeric Entities")]
    ToHtmlNumEntities,
    /// from HTML numeric entities
    /// ex: '&#x1d54a, &#x263a, &#x61' = '𝕊☺a'
    #[strum(message = "From Html Numeric Entities")]
    FromHtmlNumEntities,
    /// to HTML sanitize
    /// ex: '<' = '&lt;'
    #[strum(message = "To Html Sanitise")]
    ToHtmlSanitise,
    /// from HTML sanitize
    /// ex: '&#9787;' = '☻'
    #[strum(message = "From Html Sanitise")]
    FromHtmlSanitise,
    /// to utf-7 (rfc 3501)
    #[strum(message = "To UTF-7")]
    ToUtf7,
    /// from utf-7 (rfc 3501)
    #[strum(message = "From UTF-7")]
    FromUtf7,
}

pub fn menu_ui(ui: &mut Ui, state: &mut AppState) -> bool {
    ui.horizontal(|ui| {
        let mut changed = selected_tool_ui(ui, state);

        changed |= match state.selected {
            SelectedTool::Base64 => base64_menu_ui(ui, &mut state.options.base64),
            SelectedTool::Binary => binary_menu_ui(ui, &mut state.options.binary),
            SelectedTool::Escape => escape_menu_ui(ui, &mut state.options.escape),
            SelectedTool::Crypt => crypt_menu_ui(ui, &mut state.options.crypt),
            SelectedTool::Regex => regex_menu_ui(ui, state),
            SelectedTool::Data => jq_menu_ui(ui, &mut state.options.data),
            SelectedTool::Diff => diff_menu_ui(ui, state),
        };

        changed
    })
        .inner
}

fn selected_tool_ui(ui: &mut Ui, state: &mut AppState) -> bool {
    let mut selected = Conv::from(state.selected);
    let changed = combobox(ui, "menu.converter", &mut selected);

    if changed {
        state.selected = SelectedTool::from(selected);
    }

    changed
}

fn base64_menu_ui(ui: &mut Ui, options: &mut Base64Options) -> bool {
    combobox(ui, "menu.base64.mode", &mut options.mode)
}

fn binary_menu_ui(ui: &mut Ui, options: &mut BinaryOptions) -> bool {
    combobox(ui, "menu.binary.mode", &mut options.mode)
}

fn escape_menu_ui(ui: &mut Ui, options: &mut EscapeOptions) -> bool {
    combobox(ui, "menu.escape.mode", &mut options.mode)
}

fn crypt_menu_ui(ui: &mut Ui, options: &mut CryptOptions) -> bool {
    let mut changed = false;

    changed |= combobox(ui, "menu.crypt.digest", &mut options.digest);

    match options.digest {
        DigestMenu::Md5
        | DigestMenu::Sha1
        | DigestMenu::Sha224
        | DigestMenu::Sha256
        | DigestMenu::Sha384
        | DigestMenu::Sha512 => {}
        DigestMenu::Aes128 | DigestMenu::Aes192 | DigestMenu::Aes256 => {
            changed |= combobox(ui, "menu.crypt.aes.mode", &mut options.aes_mode);

            match options.aes_mode {
                AesMode::Gcm => {
                    ui.label("IV bits:");
                    changed |= combobox(ui, "menu.crypt.aes.gcm_tag", &mut options.gcm_nonce);
                }
                AesMode::Cbc => {}
                AesMode::Ecb => {}
            }

            changed |= combobox(ui, "menu.crypt.aes.encdec", &mut options.encdec);
            changed |= crypt_output_format_ui(ui, options);
        }
    }

    changed
}

fn crypt_output_format_ui(ui: &mut Ui, options: &mut CryptOptions) -> bool {
    if options.encdec == AesEncDec::Encrypt && options.output_format == CryptOutputFormat::Text {
        options.output_format = CryptOutputFormat::Base64;
    }

    let mut changed = false;

    ComboBox::from_id_salt("menu.crypt.output_format")
        .width(0.0)
        .selected_text(options.output_format.get_message().unwrap_or_default())
        .show_ui(ui, |ui| {
            for variant in CryptOutputFormat::VARIANTS {
                if options.encdec == AesEncDec::Encrypt && *variant == CryptOutputFormat::Text {
                    continue;
                }

                changed |= ui
                    .selectable_value(
                        &mut options.output_format,
                        *variant,
                        variant.get_message().unwrap_or_default(),
                    )
                    .on_hover_text(variant.get_documentation().unwrap_or_default())
                    .changed();
            }
        })
        .response
        .on_hover_text(options.output_format.get_documentation().unwrap_or_default());

    changed
}

fn regex_menu_ui(ui: &mut Ui, state: &mut AppState) -> bool {
    let mut changed = false;

    {
        let options = &mut state.options.regex;

        changed |= combobox(ui, "menu.regex.mode", &mut options.mode);

        match options.mode {
            RegexMenu::Regex => {
                changed |= ui.checkbox(&mut options.single_line, "single line").changed();
                changed |= ui.checkbox(&mut options.replace_enabled, "replace").changed();
                changed |= ui.checkbox(&mut options.ignore_case, "ignore case").changed();
            }
            RegexMenu::Grep => {
                changed |= ui.checkbox(&mut options.invert, "invert").changed();
                changed |= ui.checkbox(&mut options.ignore_case, "ignore case").changed();
                changed |= ui.checkbox(&mut options.sort, "sort").changed();
                changed |= ui.checkbox(&mut options.uniq, "uniq").changed();
                changed |= ui.checkbox(&mut options.unique, "unique").changed();
            }
        }
    }

    if state.options.regex.mode == RegexMenu::Grep
        && let Some(count) = regex::grep_result_count(&state.input, &state.options.regex)
    {
        ui.label(format!("{count} lines"))
            .on_hover_text("number of resulting lines");
    }

    changed
}

fn jq_menu_ui(ui: &mut Ui, options: &mut DataOptions) -> bool {
    let mut changed = false;

    ui.label("from:");
    changed |= combobox(ui, "menu.data.input_format", &mut options.input_format);

    ui.label("to:");
    changed |= combobox(ui, "menu.data.output_format", &mut options.output_format);

    if matches!(options.input_format, InputFormat::Json | InputFormat::Csv) {
        changed |= ui
            .checkbox(&mut options.slurp, "slurp")
            .on_hover_text("combine all input values into a single array")
            .changed();
    }

    if options.output_format == OutputFormat::Json {
        changed |= ui
            .checkbox(&mut options.compact, "compact")
            .on_hover_text("compact JSON output (-c)")
            .changed();
    }

    changed
}

fn combobox<T>(ui: &mut Ui, salt: &str, value: &mut T) -> bool
where
    T: EnumMessage + VariantArray + PartialEq + Clone,
{
    let mut changed = false;

    ComboBox::from_id_salt(salt)
        .width(0.0)
        .selected_text(value.get_message().unwrap_or_default())
        .show_ui(ui, |ui| {
            for variant in T::VARIANTS {
                changed |= ui
                    .selectable_value(value, variant.clone(), variant.get_message().unwrap_or_default())
                    .on_hover_text(variant.get_documentation().unwrap_or_default())
                    .changed();
            }
        })
        .response
        .on_hover_text(value.get_documentation().unwrap_or_default());

    changed
}

fn diff_menu_ui(ui: &mut Ui, state: &mut AppState) -> bool {
    let options = &mut state.options.diff;
    let mut changed = false;

    ui.label("language:");
    changed |= combobox(ui, "menu.diff.language", &mut options.language);

    if !options.language.supports_pretty_print() {
        options.pretty_print = false;
    }

    let structural_fallback = options
        .language
        .structural_diff_input_limit()
        .is_some_and(|limit| options.left.len() > limit || options.right.len() > limit);

    changed |= ui
        .add_enabled(
            options.language.supports_pretty_print() && !structural_fallback,
            eframe::egui::Checkbox::new(&mut options.pretty_print, "pretty"),
        )
        .on_hover_text("format both inputs before comparing them")
        .changed();

    changed |= ui
        .checkbox(&mut options.ignore_comments, "ignore comments")
        .changed();

    changed |= ui
        .checkbox(&mut options.show_hex, "hex view")
        .on_hover_text("show the current source line as UTF-8 bytes")
        .changed();

    ui.separator();

    if ui.button("↑ Prev").clicked() {
        options.pending_change_delta -= 1;
    }

    if ui.button("↓ Next").clicked() {
        options.pending_change_delta += 1;
    }

    if ui.button("Swap").clicked() {
        std::mem::swap(&mut options.left, &mut options.right);
        changed = true;
    }

    if ui
        .add_enabled(
            !options.left.is_empty() || !options.right.is_empty(),
            eframe::egui::Button::new("Clear"),
        )
        .clicked()
    {
        options.left.clear();
        options.right.clear();
        changed = true;
    }

    changed
}