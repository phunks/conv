use crate::app::result::ConvertResult;
use crate::widgets::menu::BinaryMenu;

#[derive(Default, Clone, PartialEq)]
pub struct BinaryOptions {
    pub mode: BinaryMenu,
}

pub(crate) fn convert(input: &str, options: &BinaryOptions) -> ConvertResult {
    match options.mode {
        BinaryMenu::HexEncode => {
            let output = hex_encode(input.as_bytes());
            ConvertResult::Text(output)
        }
        BinaryMenu::HexDecode => match hex_decode(input) {
            Ok(output) => ConvertResult::Text(output),
            Err(e) => ConvertResult::Error(format!("warn: {e}")),
        },
        BinaryMenu::ToByteString => {
            let bytes = input.as_bytes();
            let mut output = String::with_capacity(bytes.len().saturating_mul(6));

            for (index, byte) in bytes.iter().enumerate() {
                if index > 0 {
                    output.push_str(", ");
                }
                output.push_str("0x");
                push_hex_byte(&mut output, *byte);
            }

            ConvertResult::Text(output)
        }
        BinaryMenu::FromByteString => {
            let bytes = input
                .split(',')
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(|s| s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")).unwrap_or(s))
                .map(|s| u8::from_str_radix(s, 16))
                .collect::<Result<Vec<_>, _>>();

            match bytes {
                Ok(bytes) => ConvertResult::Text(String::from_utf8_lossy(&bytes).into_owned()),
                Err(e) => ConvertResult::Error(format!("warn: {e}")),
            }
        }
        BinaryMenu::ToDecimalString => {
            let mut output = String::with_capacity(input.len().saturating_mul(3));

            for (index, ch) in input.chars().enumerate() {
                if index > 0 {
                    output.push(' ');
                }
                output.push_str(&(ch as u32).to_string());
            }

            ConvertResult::Text(output)
        }
        BinaryMenu::FromDecimalString => {
            ConvertResult::Text(from_decimal_codepoint_string(input))
        }
    }
}

fn push_hex_byte(out: &mut String, byte: u8) {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    out.push(HEX[(byte >> 4) as usize] as char);
    out.push(HEX[(byte & 0x0f) as usize] as char);
}

fn hex_encode(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(bytes.len() * 2);

    for &byte in bytes {
        push_hex_byte(&mut output, byte);
    }

    output
}

fn from_decimal_codepoint_string(input: &str) -> String {
    let mut output = String::new();
    let mut digits = String::new();

    for character in input.chars() {
        if character.is_ascii_digit() {
            digits.push(character);
            continue;
        }

        if !digits.is_empty() {
            if let Ok(codepoint) = digits.parse::<u32>()
                && let Some(character) = char::from_u32(codepoint) {
                    output.push(character);
            }

            digits.clear();
        }
    }

    if !digits.is_empty()
        && let Ok(codepoint) = digits.parse::<u32>()
        && let Some(character) = char::from_u32(codepoint) {
        output.push(character);
    }

    output
}

fn hex_decode(input: &str) -> Result<String, String> {
    let input = input.trim().replace([' ', '\n', '\r', '\t'], "");

    if !input.len().is_multiple_of(2) {
        return Err("hex input length must be even".to_string());
    }

    if let Some(character) = input.chars().find(|character| !character.is_ascii_hexdigit()) {
        return Err(format!("invalid hex character: {character}"));
    }

    #[allow(clippy::chunks_exact_to_as_chunks)]
    let bytes = input
        .as_bytes()
        .chunks_exact(2)
        .map(|chunk| {
            let hex = std::str::from_utf8(chunk).map_err(|error| error.to_string())?;
            u8::from_str_radix(hex, 16).map_err(|error| error.to_string())
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(String::from_utf8_lossy(&bytes).into_owned())
}