use crate::conv::enum_variants::{Base64Menu, BinaryMenu, Conv, EscapeMenu, DigestMenu, RegexMenu};
use crate::conv::modules::base64::*;
use crate::conv::modules::binary::*;
use crate::conv::modules::crypt::*;
use crate::conv::modules::escape::*;
use crate::conv::modules::regex::*;
use crate::conv::Editor;
use eframe::egui;
use eframe::egui::{Color32, RichText};
use egui::{Align, Layout, TextStyle, Ui, vec2};
use crate::conv::modules::jaq::jq;

pub fn convert(ui: &mut Ui, editor: &mut Editor) {
    let initial_size = vec2(
        ui.available_width(),
        ui.spacing().interact_size.y, // Assume there will be
    );

    let layout = Layout::left_to_right(Align::BOTTOM).with_main_wrap(true);

    ui.allocate_ui_with_layout(initial_size, layout, |ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        let row_height = ui.text_style_height(&TextStyle::Body);
        ui.set_row_height(row_height);

        item_ui(ui, editor);
    });
}

pub fn item_ui(ui: &mut Ui, editor: &mut Editor) {
    match editor.menu.converter {
        Conv::Base64 => match editor.menu.base64 {
            Base64Menu::ToBase64 => to_base64(editor),
            Base64Menu::ToBase64Url => to_base64_url(editor),
            Base64Menu::FromBase64 => from_base64(ui, editor),
            Base64Menu::ToDeflatedSaml => to_deflated_saml(editor),
            Base64Menu::FromDeflatedSaml => from_deflated_saml(ui, editor),
        },
        Conv::Binary => match editor.menu.binary {
            BinaryMenu::HexEncode => hex_encode(editor),
            BinaryMenu::HexDecode => hex_decode(ui, editor),
            BinaryMenu::ToByteString => to_byte_string(editor),
            BinaryMenu::FromByteString => from_byte_string(ui, editor),
            BinaryMenu::ToHexDecimalString => to_hex_decimal_string(editor),
            BinaryMenu::FromHexDecimalString => from_hex_decimal_string(editor),
        },
        Conv::Escape => match editor.menu.escape {
            EscapeMenu::UrlEncode => url_encode(editor),
            EscapeMenu::UrlDecode => url_decode(editor),
            EscapeMenu::ToJsString => to_js_string(editor),
            EscapeMenu::FromJsString => from_js_string(editor),
            EscapeMenu::ToHtmlNumEntities => to_html_num_entities(editor),
            EscapeMenu::FromHtmlNumEntities => from_html_num_entities(editor),
            EscapeMenu::ToHtmlSanitise => to_html_sanitise(editor),
            EscapeMenu::FromHtmlSanitise => from_html_sanitise(editor),
            EscapeMenu::ToUtf7 => to_utf7(editor),
            EscapeMenu::FromUtf7 => from_utf7(editor),
        },
        Conv::Crypt => match editor.menu.digest {
            DigestMenu::Md5 => digest_md5(editor),
            DigestMenu::Sha1 => digest_sha1(editor),
            DigestMenu::Sha224 => digest_sha224(editor),
            DigestMenu::Sha256 => digest_sha256(editor),
            DigestMenu::Sha384 => digest_sha384(editor),
            DigestMenu::Sha512 => digest_sha512(editor),
            DigestMenu::Aes128 => digest_aes128(ui, editor),
            DigestMenu::Aes192 => digest_aes192(ui, editor),
            DigestMenu::Aes256 => digest_aes256(ui, editor),
        },
        Conv::Regex => match editor.menu.regex {
            RegexMenu::Regex => regex_replace(ui, editor),
            RegexMenu::Grep => regex_grep(ui, editor),
        },
        Conv::Jq => {
            jq(ui, editor)
        }
    }
    ui.label(&editor.output);
}

#[inline]
pub fn error<T>(ui: &mut Ui, s: &str, e: &T) -> String
where
    String: for<'a> From<&'a T>,
{
    ui.label(RichText::new(format!("{}: ", s)).color(Color32::ORANGE));
    ui.label(RichText::new(e).color(Color32::from_rgb(180, 190, 120)));
    "".to_string()
}
