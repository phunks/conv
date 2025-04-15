use crate::conv::Editor;
use crate::conv::converter::error;
use crate::conv::util::ext::{SliceExt, StringExt};
use crate::lazy_regex;
use base64::alphabet::{STANDARD, URL_SAFE};
use base64::engine::general_purpose;
use base64::engine::general_purpose::{NO_PAD, PAD};
use base64::{Engine, engine};
use eframe::egui::Ui;
use flate2::Compression;
use flate2::write::DeflateEncoder;
use inflate::InflateWriter;
use std::io::Write;
use std::sync::LazyLock;

lazy_regex!(
    RE_LF:  r"\n",
    RE_PAD: r"=+$"
);

pub fn to_base64(editor: &mut Editor) {
    editor.output = RE_LF
        .replace_all(&editor.code, "")
        .as_bytes()
        .t_base64(STANDARD, PAD);
}

pub fn to_base64_url(editor: &mut Editor) {
    editor.output = RE_LF
        .replace_all(&editor.code, "")
        .as_bytes()
        .t_base64(STANDARD, NO_PAD);
}

pub fn from_base64(ui: &mut Ui, editor: &mut Editor) {
    editor.output = match RE_PAD
        .replace_all(&editor.code, "")
        .to_string()
        .tr_safe_url()
        .as_bytes()
        .f_base64(URL_SAFE, NO_PAD)
    {
        Ok(a) => a.t_utf8_string(),
        Err(e) => error(ui, "warn", &e.to_string()),
    };
}

pub fn to_deflated_saml(editor: &mut Editor) {
    let mut buf = vec![];
    {
        let mut enc = DeflateEncoder::new(&mut buf, Compression::default());
        enc.write_all(editor.code.as_ref()).unwrap();
    }
    editor.output = general_purpose::STANDARD.encode(&buf);
}

pub fn from_deflated_saml(ui: &mut Ui, editor: &mut Editor) {
    let text = RE_PAD.replace_all(&editor.code, "");
    let mut inf = InflateWriter::new(Vec::new());
    editor.output = match engine::GeneralPurpose::new(&STANDARD, NO_PAD).decode(&*text) {
        Ok(a) => match inf.write(a.as_ref()) {
            Err(e) => error(ui, "warn", &e.to_string()),
            _ => inf.finish().unwrap().t_utf8_string(),
        },
        Err(e) => error(ui, "warn", &e.to_string()),
    };
}
