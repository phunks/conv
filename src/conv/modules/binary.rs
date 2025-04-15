use crate::conv::Editor;
use crate::conv::converter::error;
use crate::conv::util::ext::{SliceExt, StringExt};
use crate::lazy_regex;
use eframe::egui::Ui;
use itertools::Itertools;
use rustc_serialize::hex::{FromHex, ToHex};
use std::sync::LazyLock;

lazy_regex!(
    RE_0X:  r"0[x|X](?<b>[0-9a-fA-F]{2})", // 2 digit hex string ex: 0x0a
    RE_DEC: r"(?<b>\d+)"
);
pub fn hex_encode(editor: &mut Editor) {
    editor.output = editor.code.as_bytes().to_hex();
}

pub fn hex_decode(ui: &mut Ui, editor: &mut Editor) {
    editor.output = match &editor.code.from_hex() {
        Ok(dec) => dec.t_utf8_string(),
        Err(e) => error(ui, "warn", &e.to_string()),
    };
}

pub fn to_byte_string(editor: &mut Editor) {
    // 0x31, 0x34
    editor.output = editor
        .code
        .utf8_bytestring()
        .iter()
        .map(|x| format!("{:#02x}", x))
        .join(r", ");
}

pub fn from_byte_string(ui: &mut Ui, editor: &mut Editor) {
    // 0x31, 0x34
    let a = RE_0X
        .captures_iter(&editor.code)
        .map(|cap| cap["b"].to_owned())
        .collect::<Vec<_>>()
        .join("");

    editor.output = match a.from_hex() {
        Ok(dec) => dec.t_utf8_string(),
        Err(e) => error(ui, "warn", &e.to_string()),
    };
}

pub fn to_hex_decimal_string(editor: &mut Editor) {
    editor.output = editor.code.char_bytestring().into_iter().join(" ");
}

pub fn from_hex_decimal_string(editor: &mut Editor) {
    editor.output = RE_DEC
        .captures_iter(&editor.code)
        .map(|cap| cap["b"].to_owned())
        .filter_map(|x| x.parse::<u32>().ok())
        .filter_map(char::from_u32)
        .collect::<String>();
}
