use crate::app::result::ConvertResult;
use aes_gcm::aead::consts::{U12, U13, U14, U15, U16};
use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{AesGcm, Key};
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use crate::converters::crypt::{AesEncDec, CryptOptions, CryptOutputFormat, GcmNonceLen};

pub(crate) fn aes_gcm<Aes>(input: &str, options: &CryptOptions, key_len: usize) -> ConvertResult
where
    AesGcm<Aes, U16>: KeyInit + Aead,
    AesGcm<Aes, U15>: KeyInit + Aead,
    AesGcm<Aes, U14>: KeyInit + Aead,
    AesGcm<Aes, U13>: KeyInit + Aead,
    AesGcm<Aes, U12>: KeyInit + Aead,
{
    // Historical compatibility:
    // `GcmTagLen` was originally used as nonce/IV length, not authentication tag length.
    // AES-GCM tag length remains the aes-gcm crate default.
    match options.gcm_nonce {
        GcmNonceLen::Nonce128 => aes_gcm_with_nonce::<Aes, U16>(input, options, key_len, 16),
        GcmNonceLen::Nonce120 => aes_gcm_with_nonce::<Aes, U15>(input, options, key_len, 15),
        GcmNonceLen::Nonce112 => aes_gcm_with_nonce::<Aes, U14>(input, options, key_len, 14),
        GcmNonceLen::Nonce104 => aes_gcm_with_nonce::<Aes, U13>(input, options, key_len, 13),
        GcmNonceLen::Nonce96  => aes_gcm_with_nonce::<Aes, U12>(input, options, key_len, 12),
    }
}

fn aes_gcm_with_nonce<Aes, N>(
    input: &str,
    options: &CryptOptions,
    key_len: usize,
    nonce_len: usize,
) -> ConvertResult
where
    AesGcm<Aes, N>: KeyInit + Aead,
{
    let mut warnings = Vec::new();

    if options.key.len() != key_len {
        warnings.push(format!(
            "warn: key length {}/{} bytes",
            options.key.len(),
            key_len,
        ));
    }

    if !options.iv.is_empty() && options.iv.len() != nonce_len {
        warnings.push(format!(
            "warn: iv length {}/{} bytes",
            options.iv.len(),
            nonce_len,
        ));
    }

    if !warnings.is_empty() {
        return ConvertResult::Warnings(warnings);
    }

    let key = match Key::<AesGcm<Aes, N>>::try_from(options.key.as_bytes()) {
        Ok(key) => key,
        Err(error) => {
            return ConvertResult::Error(format!("warn: key error: {error}"));
        }
    };

    let nonce = if options.iv.is_empty() {
        // Historical behavior: empty IV means all-zero nonce.
        aes_gcm::aead::Nonce::<AesGcm<Aes, N>>::default()
    } else {
        match aes_gcm::aead::Nonce::<AesGcm<Aes, N>>::try_from(options.iv.as_bytes()) {
            Ok(nonce) => nonce,
            Err(error) => {
                return ConvertResult::Error(format!("warn: nonce error: {error}"));
            }
        }
    };

    let cipher = AesGcm::<Aes, N>::new(&key);

    match options.encdec {
        AesEncDec::Encrypt => {
            let ciphertext = match cipher.encrypt(&nonce, input.as_bytes()) {
                Ok(ciphertext) => ciphertext,
                Err(error) => return ConvertResult::Error(format!("warn: {error:?}")),
            };

            ConvertResult::Text(format_encrypted_bytes(&ciphertext, options.output_format))
        }
        AesEncDec::Decrypt => {
            let ciphertext = match parse_ciphertext(input) {
                Ok(ciphertext) => ciphertext,
                Err(warnings) => return ConvertResult::Warnings(warnings),
            };

            let plaintext = match cipher.decrypt(&nonce, ciphertext.as_ref()) {
                Ok(plaintext) => plaintext,
                Err(error) => return ConvertResult::Error(format!("warn: {error:?}")),
            };

            ConvertResult::Text(format_decrypted_bytes(&plaintext, options.output_format))
        }
    }
}

pub(crate) fn format_encrypted_bytes(bytes: &[u8], format: CryptOutputFormat) -> String {
    match format {
        CryptOutputFormat::Base64 | CryptOutputFormat::Text => STANDARD.encode(bytes),
        CryptOutputFormat::Hex => base16ct::lower::encode_string(bytes),
    }
}

pub(crate) fn format_decrypted_bytes(bytes: &[u8], format: CryptOutputFormat) -> String {
    match format {
        CryptOutputFormat::Base64 => STANDARD.encode(bytes),
        CryptOutputFormat::Hex => base16ct::lower::encode_string(bytes),
        CryptOutputFormat::Text => String::from_utf8_lossy(bytes).into_owned(),
    }
}

pub(crate) fn parse_ciphertext(input: &str) -> Result<Vec<u8>, Vec<String>> {
    let mut warnings = Vec::new();

    match decode_hex(input) {
        Ok(bytes) => return Ok(bytes),
        Err(error) => warnings.push(format!("warn: hex decode: {error}")),
    }

    match STANDARD.decode(input.trim()) {
        Ok(bytes) => Ok(bytes),
        Err(error) => {
            warnings.push(format!("warn: base64 decode: {error}"));
            Err(warnings)
        }
    }
}

fn decode_hex(input: &str) -> Result<Vec<u8>, String> {
    let input = input.trim().replace([' ', '\n', '\r', '\t'], "");

    if !input.len().is_multiple_of(2) {
        return Err("hex input length must be even".to_string());
    }

    if let Some(character) = input.chars().find(|character| !character.is_ascii_hexdigit()) {
        return Err(format!("invalid hex character: {character}"));
    }

    #[allow(clippy::chunks_exact_to_as_chunks)]
    input
        .as_bytes()
        .chunks_exact(2)
        .map(|chunk| {
            let hex = std::str::from_utf8(chunk).map_err(|error| error.to_string())?;
            u8::from_str_radix(hex, 16).map_err(|error| error.to_string())
        })
        .collect()
}