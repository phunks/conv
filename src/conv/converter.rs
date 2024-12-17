use std::io::Write;
use base64::engine::general_purpose;
use base64::{Engine as _, engine, alphabet};
use eframe::egui;
use egui::{
    vec2, Align, Layout, TextStyle,
    Ui,
};

use std::sync::LazyLock;
use flate2::Compression;
use flate2::write::DeflateEncoder;
use inflate::InflateWriter;
use itertools::Itertools;
use regex::Regex;
use rustc_serialize::hex::{ToHex, FromHex};
use sha1::{Digest, Sha1};
use crate::conv::{Editor, enum_variants};
use crate::conv::hasher::hasher;
use crate::conv::enum_variants::{Base64Kind, BinaryKind, Conv, EscapeKind};
use crate::lazy_regex;

pub fn convert(ui: &mut Ui, editor: &Editor) {
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

lazy_regex!(
    RE_LF:  r"\n",
    RE_PAD: r"=+$",
    RE_0X:  r"0[x|X](?<b>[0-9a-fA-F]{2})", // 2 digit hex string ex: 0x0a
    RE_BSU: r"\\u(?<b>[0-9a-fA-F]+)",
    RE_HS:  r"&#[x|X](?<b>[0-9a-fA-F]+)"
);

pub fn item_ui(ui: &mut Ui, editor: &Editor) {

    match editor.menu.converter {
        Conv::Base64 => {
            match editor.menu.base64 {
                Base64Kind::ToBase64 => {
                    // rfc 4648
                    let text = RE_LF.replace_all(&editor.code, "");
                    let enc = engine::GeneralPurpose::new(
                        &alphabet::STANDARD,
                        general_purpose::PAD).encode(&*text);
                    ui.label(enc);
                }
                Base64Kind::ToBase64Url => {
                    // rfc 4648 url safe
                    let text = RE_LF.replace_all(&editor.code, "");
                    let enc = engine::GeneralPurpose::new(
                        &alphabet::URL_SAFE,
                        general_purpose::NO_PAD).encode(&*text);
                    ui.label(enc);
                }
                Base64Kind::FromBase64 => {
                    let text = tr_safe_url(&RE_PAD.replace_all(&editor.code, ""));
                    let dec = match engine::GeneralPurpose::new(
                        &alphabet::URL_SAFE,
                        general_purpose::NO_PAD)
                        .decode(&*text)  {
                        Ok(a) => String::from_utf8_lossy(&a).into_owned(),
                        Err(e) => e.to_string(),
                    };
                    ui.label(dec);
                }
                Base64Kind::ToDeflatedSaml => {
                    let mut buf = vec![];
                    {
                        let mut enc =
                            DeflateEncoder::new(&mut buf, Compression::default());
                        enc.write_all(editor.code.as_ref()).unwrap();
                    }
                    let enc = general_purpose::STANDARD.encode(&buf);
                    ui.label(enc);
                }
                Base64Kind::FromDeflatedSaml => {
                    let text = RE_PAD.replace_all(&editor.code, "");
                    let mut inf = InflateWriter::new(Vec::new());
                    let dec = match engine::GeneralPurpose::new(
                        &alphabet::STANDARD,
                        general_purpose::NO_PAD)
                        .decode(&*text)  {
                        Ok(a) => {
                            match inf.write(a.as_ref()) {
                                Err(e) => e.to_string(),
                                _ => String::from_utf8_lossy(&inf.finish().unwrap()).into_owned()
                            }
                        },
                        Err(e) => e.to_string(),
                    };
                    ui.label(dec);
                }
            }
        }
        Conv::Binary => {
            match editor.menu.binary {
                BinaryKind::HexEncode => {
                    ui.label(editor.code.as_bytes().to_hex());
                }
                BinaryKind::HexDecode => {
                    let a = match &editor.code.from_hex() {
                        Ok(dec) => String::from_utf8_lossy(dec).into_owned(),
                        Err(e) => e.to_string(),
                    };
                    ui.label(a);
                }
                BinaryKind::ToByteString => {
                    // 0x31, 0x34
                    ui.label(format!(r"0x{}", utf8_bytestring(&editor.code)
                        .iter().map(|x|format!("{:02x}",x)).join(r", 0x")));
                }
                BinaryKind::FromByteString => {
                    // 0x31, 0x34
                    let a = RE_0X.captures_iter(&editor.code)
                        .map(|cap| cap["b"].to_owned())
                        .collect::<Vec<_>>()
                        .to_owned().join("");

                    let a = match a.from_hex() {
                        Ok(dec) => {
                            String::from_utf8_lossy(&dec).into_owned()
                        },
                        Err(e) => e.to_string(),
                    };
                    ui.label(a);
                }
            }
        }
        Conv::Escape => {
            match editor.menu.escape {
                EscapeKind::UrlEncode => {
                    // TODO rfc 3986
                    ui.label(url_escape::encode_www_form_urlencoded(&editor.code));
                }
                EscapeKind::UrlDecode => {
                    ui.label(url_escape::decode(&editor.code));
                }
                EscapeKind::ToJsString => {
                    // \u3042\u3042..
                    ui.label(format!(r"\u{}", char_bytestring(&editor.code)
                        .iter().map(|x|format!("{:x}",x)).join(r"\u")));
                }
                EscapeKind::FromJsString => {
                    let a = RE_BSU.captures_iter(&editor.code)
                        .map(|cap| cap["b"].to_owned())
                        .map(|x| parse_unicode(&x))
                        .filter_map(|x|x.map(|a| a))
                        .collect::<Vec<_>>().iter().join("");

                    ui.label(a);
                }
                EscapeKind::ToHtmlNumEntities => {
                    ui.label(format!(r"&#x{}", char_bytestring(&editor.code)
                        .iter().map(|x|format!("{:x}",x)).join(r", &#x")));
                }
                EscapeKind::FromHtmlNumEntities => {
                    let a = RE_HS.captures_iter(&editor.code)
                        .map(|cap| cap["b"].to_owned())
                        .map(|x| parse_unicode(&x))
                        .filter_map(|x|x.map(|a| a))
                        .collect::<Vec<_>>().iter().join("");

                    ui.label(a);
                }
                EscapeKind::ToHtmlSanitise => {
                    ui.label(html_escape::encode_safe(&editor.code));
                }
                EscapeKind::FromHtmlSanitise => {
                    ui.label(html_escape::decode_html_entities(&editor.code));
                }
            }
        }

        Conv::ToUtf7 => {
            // rfc 3501
            let enc = utf7_imap::encode_utf7_imap(editor.code.to_string());
            ui.label(enc);
        }
        Conv::FromUtf7 => {
            let dec = utf7_imap::decode_utf7_imap(editor.code.to_string());
            ui.label(dec);
        }
        Conv::Crypt => {
            match editor.menu.digest {
                enum_variants::Digest::Md5 => {
                    let digest = md5::compute(&editor.code);
                    ui.label(digest.to_hex());
                }
                enum_variants::Digest::Sha1 => {
                    let mut h = Sha1::new();
                    sha1::Digest::update(&mut h, <str as AsRef<[u8]>>::as_ref(&editor.code));
                    ui.label(String::from_utf8_lossy((*h.finalize().to_hex()).as_ref()));
                }
                enum_variants::Digest::Sha224 => {
                    ui.label(hasher("sha224", &editor.code));
                }
                enum_variants::Digest::Sha256 => {
                    ui.label(hasher("sha256", &editor.code));
                }
                enum_variants::Digest::Sha384 => {
                    ui.label(hasher("sha384", &editor.code));
                }
                enum_variants::Digest::Sha512 => {
                    ui.label(hasher("sha512", &editor.code));
                }
            }
        }
    }
}

const TR_SAFE_URL: [char; 4] = [
  '/', '+', '_', '-'
];

#[inline]
fn tr_safe_url(text: &str) -> String {
    let mut buf: String = String::with_capacity(text.len());
    for c in text.chars() {
        if let Some(idx) = TR_SAFE_URL.iter()
            .take(2)
            .position(|x| x == &c)
            {buf.push(TR_SAFE_URL[idx + 2]); continue}
        buf.push(c);
    }
    buf
}

#[inline]
fn utf8_bytestring(text: &str) -> Vec<u8> {
    text.chars()
        .map(|x| {
            let mut b = [0; 4];
            x.encode_utf8(&mut b);
            b
        })
        .flat_map(|x| x.into_iter()
            .filter(|x| x != &0))
        .collect::<Vec<_>>()
}

#[allow(unused)]
#[inline]
fn utf16_bytestring(text: &str) -> Vec<u16> {
    text.chars()
        .map(|x| {
            let mut b = [0; 2];
            x.encode_utf16(&mut b);
            b
        })
        .flat_map(|x| x.into_iter()
            .filter(|x| x != &0))
        .collect::<Vec<_>>()
}

#[inline]
fn char_bytestring(text: &str) -> Vec<u32> {
    text.chars()
        .map(|x| {
            x as u32
        })
        .collect::<Vec<_>>()
}


fn parse_unicode(input: &str) -> Option<char> {
    let unicode = u32::from_str_radix(input, 16).ok();
    char::from_u32(unicode?)
}

#[allow(unused)]
fn parse_unicode_digit(input: &str) -> Option<char> {
    let unicode = u32::from_str_radix(input, 16).ok();
    char::from_digit(unicode?, 16)
}
