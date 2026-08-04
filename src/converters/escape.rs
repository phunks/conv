use crate::app::result::ConvertResult;
use crate::widgets::menu::EscapeMenu;

#[derive(Default, Clone, PartialEq)]
pub struct EscapeOptions {
    pub mode: EscapeMenu,
}

pub(crate) fn convert(input: &str, options: &EscapeOptions) -> ConvertResult {
    match options.mode {
        EscapeMenu::UrlEncode => ConvertResult::Text(url_escape::encode_component(input).to_string()),
        EscapeMenu::UrlDecode => ConvertResult::Text(url_escape::decode(input).to_string()),
        EscapeMenu::ToJsString => ConvertResult::Text(to_js_string(input)),
        EscapeMenu::FromJsString => ConvertResult::Text(from_js_string(input)),
        EscapeMenu::ToHtmlNumEntities => ConvertResult::Text(to_html_numeric_entities(input)),
        EscapeMenu::FromHtmlNumEntities => {
            ConvertResult::Text(html_escape::decode_html_entities(input).to_string())
        }
        EscapeMenu::ToHtmlSanitise => ConvertResult::Text(html_escape::encode_safe(input).to_string()),
        EscapeMenu::FromHtmlSanitise => {
            ConvertResult::Text(html_escape::decode_html_entities(input).to_string())
        }
        EscapeMenu::ToUtf7 => ConvertResult::Text(utf7_imap::encode_utf7_imap(input.to_owned())),
        EscapeMenu::FromUtf7 => ConvertResult::Text(utf7_imap::decode_utf7_imap(input.to_owned())),
    }
}

fn to_js_string(input: &str) -> String {
    let mut output = String::with_capacity(input.len().saturating_mul(6));

    for c in input.chars() {
        output.push_str("\\u{");
        output.push_str(&format!("{:x}", c as u32));
        output.push('}');
    }

    output
}

fn from_js_string(input: &str) -> String {
    let mut output = String::new();
    let mut rest = input;

    while let Some(pos) = rest.find("\\u{") {
        output.push_str(&rest[..pos]);
        rest = &rest[pos + 3..];

        if let Some(end) = rest.find('}') {
            let hex = &rest[..end];

            if let Ok(code) = u32::from_str_radix(hex, 16)
                && let Some(c) = char::from_u32(code) {
                    output.push(c);
            }

            rest = &rest[end + 1..];
        } else {
            output.push_str("\\u{");
            break;
        }
    }

    output.push_str(rest);
    output
}

fn to_html_numeric_entities(input: &str) -> String {
    let mut output = String::with_capacity(input.len().saturating_mul(8));

    for c in input.chars() {
        output.push_str("&#x");
        output.push_str(&format!("{:x}", c as u32));
        output.push(';');
    }

    output
}