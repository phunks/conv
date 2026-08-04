use crate::app::result::ConvertResult;
use crate::lazy_regex;
use crate::util::ext::{SliceExt, StringExt};
use crate::widgets::menu::Base64Menu;
use base64::alphabet::{STANDARD, URL_SAFE};
use base64::engine::general_purpose::{NO_PAD, PAD};
use base64::{Engine, engine};
use flate2::Compression;
use flate2::write::DeflateEncoder;
use inflate::InflateWriter;
use std::io::Write;
use std::sync::LazyLock;

#[derive(Default, Clone, PartialEq)]
pub struct Base64Options {
    pub mode: Base64Menu,
}

lazy_regex!(
    RE_LF:  r"\n",
    RE_PAD: r"=+$"
);

pub(crate) fn convert(input: &str, options: &Base64Options) -> ConvertResult {
    match options.mode {
        Base64Menu::ToBase64 => ConvertResult::Text(
            RE_LF
                .replace_all(input, "")
                .as_bytes()
                .t_base64(STANDARD, PAD),
        ),
        Base64Menu::ToBase64Url => ConvertResult::Text(
            RE_LF
                .replace_all(input, "")
                .as_bytes()
                .t_base64(STANDARD, NO_PAD),
        ),
        Base64Menu::FromBase64 => {
            let text = RE_PAD.replace_all(input, "").to_string().tr_safe_url();

            match text.as_bytes().f_base64(URL_SAFE, NO_PAD) {
                Ok(bytes) => ConvertResult::Text(bytes.t_utf8_string()),
                Err(e) => ConvertResult::Error(format!("warn: {e}")),
            }
        }
        Base64Menu::ToDeflatedSaml => {
            let mut buf = vec![];

            {
                let mut enc = DeflateEncoder::new(&mut buf, Compression::default());
                if let Err(e) = enc.write_all(input.as_ref()) {
                    return ConvertResult::Error(format!("warn: {e}"));
                }
            }

            ConvertResult::Text(base64::engine::general_purpose::STANDARD.encode(&buf))
        }
        Base64Menu::FromDeflatedSaml => {
            let text = RE_PAD.replace_all(input, "");
            let mut inf = InflateWriter::new(Vec::new());

            match engine::GeneralPurpose::new(&STANDARD, NO_PAD).decode(&*text) {
                Ok(bytes) => match inf.write(bytes.as_ref()) {
                    Ok(_) => match inf.finish() {
                        Ok(bytes) => ConvertResult::Text(bytes.t_utf8_string()),
                        Err(e) => ConvertResult::Error(format!("warn: {e}")),
                    },
                    Err(e) => ConvertResult::Error(format!("warn: {e}")),
                },
                Err(e) => ConvertResult::Error(format!("warn: {e}")),
            }
        }
    }
}