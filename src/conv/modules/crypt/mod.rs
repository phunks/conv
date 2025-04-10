use crate::conv::Editor;
use crate::conv::modules::crypt::aes::{Colorize, aes};
use crate::conv::modules::crypt::hasher::hasher;
use aes_gcm::aes::{Aes128, Aes192, Aes256};
use eframe::egui::Ui;
use rustc_serialize::hex::ToHex;
use sha1::{Digest, Sha1};

pub mod aes;
pub mod enum_crypt;
pub mod hasher;

pub fn digest_md5(editor: &mut Editor) {
    editor.output = md5::compute(&editor.code).to_hex();
}

pub fn digest_sha1(editor: &mut Editor) {
    let mut h = Sha1::new();
    sha1::Digest::update(&mut h, &editor.code);
    editor.output = h.finalize().to_hex();
}

pub fn digest_sha224(editor: &mut Editor) {
    editor.output = hasher("sha224", &editor.code);
}

pub fn digest_sha256(editor: &mut Editor) {
    editor.output = hasher("sha256", &editor.code);
}

pub fn digest_sha384(editor: &mut Editor) {
    editor.output = hasher("sha384", &editor.code);
}

pub fn digest_sha512(editor: &mut Editor) {
    editor.output = hasher("sha512", &editor.code);
}

pub fn digest_aes128(ui: &mut Ui, editor: &mut Editor) {
    aes::<Aes128>(editor).color(ui, editor);
}

pub fn digest_aes192(ui: &mut Ui, editor: &mut Editor) {
    aes::<Aes192>(editor).color(ui, editor);
}

pub fn digest_aes256(ui: &mut Ui, editor: &mut Editor) {
    aes::<Aes256>(editor).color(ui, editor);
}
