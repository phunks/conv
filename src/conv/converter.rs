use base64::engine::general_purpose;
use base64::{alphabet, engine, Engine as _};
use eframe::egui;
use egui::{vec2, Align, Layout, TextStyle, Ui};
use std::io::Write;

use crate::conv::enum_variants::{Base64Kind, BinaryKind, Conv, EscapeKind};
use crate::conv::hasher::hasher;
use crate::conv::{enum_variants, Editor};
use crate::lazy_regex;
use flate2::write::DeflateEncoder;
use flate2::Compression;
use inflate::InflateWriter;
use itertools::Itertools;
use regex::Regex;
use rustc_serialize::hex::{FromHex, ToHex};
use sha1::{Digest, Sha1};
use std::sync::LazyLock;

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

lazy_regex!(
    RE_LF:  r"\n",
    RE_PAD: r"=+$",
    RE_0X:  r"0[x|X](?<b>[0-9a-fA-F]{2})", // 2 digit hex string ex: 0x0a
    RE_BSU: r"\\u\{?(?<b>[0-9a-fA-F]+)\}?",
    RE_HS:  r"&#[x|X](?<b>[0-9a-fA-F]+)",
    RE_DEC: r"(?<b>\d+)"
);

pub fn item_ui(ui: &mut Ui, editor: &mut Editor) {
    let collector = |a: &LazyLock<Regex>, b: &String| {
        a.captures_iter(b)
            .map(|cap| cap["b"].to_owned())
            .filter_map(|x| parse_unicode(&x))
            .collect::<Vec<_>>()
            .iter()
            .join("")
    };

    match editor.menu.converter {
        Conv::Base64 => {
            match editor.menu.base64 {
                Base64Kind::ToBase64 => {
                    // rfc 4648
                    let text = RE_LF.replace_all(&editor.code, "");
                    editor.text =
                        engine::GeneralPurpose::new(&alphabet::STANDARD, general_purpose::PAD)
                            .encode(&*text);
                },
                Base64Kind::ToBase64Url => {
                    // rfc 4648 url safe
                    let text = RE_LF.replace_all(&editor.code, "");
                    editor.text =
                        engine::GeneralPurpose::new(&alphabet::URL_SAFE, general_purpose::NO_PAD)
                            .encode(&*text);
                },
                Base64Kind::FromBase64 => {
                    let text = tr_safe_url(&RE_PAD.replace_all(&editor.code, ""));
                    editor.text = match engine::GeneralPurpose::new(
                        &alphabet::URL_SAFE,
                        general_purpose::NO_PAD,
                    )
                    .decode(&*text)
                    {
                        Ok(a) => String::from_utf8_lossy(&a).into_owned(),
                        Err(e) => e.to_string(),
                    };
                },
                Base64Kind::ToDeflatedSaml => {
                    let mut buf = vec![];
                    {
                        let mut enc = DeflateEncoder::new(&mut buf, Compression::default());
                        enc.write_all(editor.code.as_ref()).unwrap();
                    }
                    editor.text = general_purpose::STANDARD.encode(&buf);
                },
                Base64Kind::FromDeflatedSaml => {
                    let text = RE_PAD.replace_all(&editor.code, "");
                    let mut inf = InflateWriter::new(Vec::new());
                    editor.text = match engine::GeneralPurpose::new(
                        &alphabet::STANDARD,
                        general_purpose::NO_PAD,
                    )
                    .decode(&*text)
                    {
                        Ok(a) => match inf.write(a.as_ref()) {
                            Err(e) => e.to_string(),
                            _ => String::from_utf8_lossy(&inf.finish().unwrap()).into_owned(),
                        },
                        Err(e) => e.to_string(),
                    };
                },
            }
        },
        Conv::Binary => {
            match editor.menu.binary {
                BinaryKind::HexEncode => {
                    editor.text = editor.code.as_bytes().to_hex();
                },
                BinaryKind::HexDecode => {
                    editor.text = match &editor.code.from_hex() {
                        Ok(dec) => String::from_utf8_lossy(dec).into_owned(),
                        Err(e) => e.to_string(),
                    };
                },
                BinaryKind::ToByteString => {
                    // 0x31, 0x34
                    editor.text = format!(
                        r"0x{}",
                        utf8_bytestring(&editor.code)
                            .iter()
                            .map(|x| format!("{:02x}", x))
                            .join(r", 0x")
                    );
                },
                BinaryKind::FromByteString => {
                    // 0x31, 0x34
                    let a = RE_0X
                        .captures_iter(&editor.code)
                        .map(|cap| cap["b"].to_owned())
                        .collect::<Vec<_>>()
                        .join("");

                    editor.text = match a.from_hex() {
                        Ok(dec) => String::from_utf8_lossy(&dec).into_owned(),
                        Err(e) => e.to_string(),
                    };
                },
                BinaryKind::ToHexDecimalString => {
                    editor.text = char_bytestring(&editor.code).into_iter().join(" ");
                },
                BinaryKind::FromHexDecimalString => {
                    editor.text = RE_DEC
                        .captures_iter(&editor.code)
                        .map(|cap| cap["b"].to_owned())
                        .filter_map(|x| x.parse::<u32>().ok())
                        .filter_map(char::from_u32)
                        .collect::<String>();
                },
            }
        },
        Conv::Escape => {
            match editor.menu.escape {
                EscapeKind::UrlEncode => {
                    // TODO rfc 3986
                    editor.text = url_escape::encode_www_form_urlencoded(&editor.code).into();
                },
                EscapeKind::UrlDecode => {
                    editor.text = url_escape::decode(&editor.code).into();
                },
                EscapeKind::ToJsString => {
                    editor.text = format!(
                        r"\u{}",
                        char_bytestring(&editor.code)
                            .iter()
                            .map(|x| format!("{{{:x}}}", x))
                            .join(r"\u")
                    );
                },
                EscapeKind::FromJsString => {
                    editor.text = collector(&RE_BSU, &editor.code);
                },
                EscapeKind::ToHtmlNumEntities => {
                    editor.text = format!(
                        r"&#x{}",
                        char_bytestring(&editor.code)
                            .iter()
                            .map(|x| format!("{:x}", x))
                            .join(r", &#x")
                    );
                },
                EscapeKind::FromHtmlNumEntities => {
                    editor.text = collector(&RE_HS, &editor.code);
                },
                EscapeKind::ToHtmlSanitise => {
                    editor.text = html_escape::encode_safe(&editor.code).into();
                },
                EscapeKind::FromHtmlSanitise => {
                    editor.text = html_escape::decode_html_entities(&editor.code).into();
                },
                EscapeKind::ToUtf7 => {
                    // rfc 3501
                    editor.text = utf7_imap::encode_utf7_imap(editor.code.to_string());
                },
                EscapeKind::FromUtf7 => {
                    editor.text = utf7_imap::decode_utf7_imap(editor.code.to_string());
                },
            }
        },

        Conv::Crypt => match editor.menu.digest {
            enum_variants::Digest::Md5 => {
                let digest = md5::compute(&editor.code);
                editor.text = digest.to_hex();
            },
            enum_variants::Digest::Sha1 => {
                let mut h = Sha1::new();
                sha1::Digest::update(&mut h, <str as AsRef<[u8]>>::as_ref(&editor.code));
                editor.text = String::from_utf8_lossy((*h.finalize().to_hex()).as_ref()).into();
            },
            enum_variants::Digest::Sha224 => {
                editor.text = hasher("sha224", &editor.code);
            },
            enum_variants::Digest::Sha256 => {
                editor.text = hasher("sha256", &editor.code);
            },
            enum_variants::Digest::Sha384 => {
                editor.text = hasher("sha384", &editor.code);
            },
            enum_variants::Digest::Sha512 => {
                editor.text = hasher("sha512", &editor.code);
            },
        },
    }
    ui.label(&editor.text);
}

const TR_SAFE_URL: [char; 4] = ['/', '+', '_', '-'];

#[inline]
fn tr_safe_url(text: &str) -> String {
    let mut buf: String = String::with_capacity(text.len());
    for c in text.chars() {
        if let Some(idx) = TR_SAFE_URL.iter().take(2).position(|x| x == &c) {
            buf.push(TR_SAFE_URL[idx + 2]);
            continue;
        }
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
        .flat_map(|x| x.into_iter().filter(|x| x != &0))
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
        .flat_map(|x| x.into_iter().filter(|x| x != &0))
        .collect::<Vec<_>>()
}

#[inline]
fn char_bytestring(text: &str) -> Vec<u32> {
    text.chars().map(|x| x as u32).collect::<Vec<_>>()
}

#[inline]
fn parse_unicode(input: &str) -> Option<char> {
    let unicode = u32::from_str_radix(input, 16).ok();
    char::from_u32(unicode?)
}
