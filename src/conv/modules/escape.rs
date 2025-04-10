use crate::conv::{Editor, util::ext::StringExt};
use crate::lazy_regex;
use itertools::Itertools;
use regex::Regex;
use std::sync::LazyLock;

lazy_regex!(
    RE_BSU: r"\\u\{?(?<b>[0-9a-fA-F]+)\}?",
    RE_HS:  r"&#[x|X](?<b>[0-9a-fA-F]+)"
);

fn collector(a: &LazyLock<Regex>, b: &str) -> String {
    a.captures_iter(b)
        .map(|cap| cap["b"].to_owned())
        .filter_map(|x| x.parse_unicode())
        .collect::<Vec<_>>()
        .iter()
        .join("")
}

pub fn url_encode(editor: &mut Editor) {
    editor.output = url_escape::encode_www_form_urlencoded(&editor.code).into();
}

pub fn url_decode(editor: &mut Editor) {
    editor.output = url_escape::decode(&editor.code).into();
}

pub fn to_js_string(editor: &mut Editor) {
    editor.output = editor
        .code
        .char_bytestring()
        .iter()
        .map(|x| format!("\\u{{{:x}}}", x))
        .join("");
}

pub fn from_js_string(editor: &mut Editor) {
    editor.output = collector(&RE_BSU, &editor.code);
}

pub fn to_html_num_entities(editor: &mut Editor) {
    editor.output = editor
        .code
        .char_bytestring()
        .iter()
        .map(|x| format!("&#x{:x}", x))
        .join(", ");
}

pub fn from_html_num_entities(editor: &mut Editor) {
    editor.output = collector(&RE_HS, &editor.code);
}

pub fn to_html_sanitise(editor: &mut Editor) {
    editor.output = html_escape::encode_safe(&editor.code).into();
}

pub fn from_html_sanitise(editor: &mut Editor) {
    editor.output = html_escape::decode_html_entities(&editor.code).into();
}

pub fn to_utf7(editor: &mut Editor) {
    editor.output = utf7_imap::encode_utf7_imap(editor.code.to_string());
}

pub fn from_utf7(editor: &mut Editor) {
    editor.output = utf7_imap::decode_utf7_imap(editor.code.to_string());
}
