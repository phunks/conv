
use crate::app::result::ConvertResult;
use aes_gcm::aes::{Aes128, Aes192, Aes256};
use strum::{EnumMessage, VariantArray};
use crate::converters::crypt::aes_impl::cbc::aes_cbc;
use crate::converters::crypt::aes_impl::ecb::aes_ecb;
use crate::converters::crypt::aes_impl::gcm::aes_gcm;
use crate::converters::crypt::aes_impl::helper::hash_input;

pub(crate) mod aes_impl;
#[derive(Default, Copy, Clone, Debug, PartialEq, Eq, VariantArray, EnumMessage)]
#[strum(serialize_all = "kebab-case")]
pub enum DigestMenu {
    #[default]
    /// md5 digest
    #[strum(message = "MD5")]
    Md5,
    /// sha1
    #[strum(message = "SHA-1")]
    Sha1,
    /// sha224
    #[strum(message = "SHA-224")]
    Sha224,
    /// sha256
    #[strum(message = "SHA-256")]
    Sha256,
    /// sha384
    #[strum(message = "SHA-384")]
    Sha384,
    /// sha512
    #[strum(message = "SHA-512")]
    Sha512,
    /// aes128
    #[strum(message = "AES-128")]
    Aes128,
    /// aes192
    #[strum(message = "AES-192")]
    Aes192,
    /// aes256
    #[strum(message = "AES-256")]
    Aes256,
}

#[derive(Default, Copy, Clone, Debug, PartialEq, Eq, VariantArray, EnumMessage)]
#[strum(serialize_all = "kebab-case")]
pub enum AesMode {
    #[default]
    /// GCM authenticated encryption
    #[strum(message = "GCM")]
    Gcm,
    /// CBC with PKCS#7 padding
    #[strum(message = "CBC")]
    Cbc,
    /// ECB with PKCS#7 padding, not recommended
    #[strum(message = "ECB")]
    Ecb,
}

#[derive(Default, Copy, Clone, Debug, PartialEq, Eq, VariantArray, EnumMessage)]
#[strum(serialize_all = "kebab-case")]
pub enum GcmNonceLen {
    /// 128 bit nonce / 16 byte IV
    #[strum(message = "128")]
    Nonce128,
    /// 120 bit nonce / 15 byte IV
    #[strum(message = "120")]
    Nonce120,
    /// 112 bit nonce / 14 byte IV
    #[strum(message = "112")]
    Nonce112,
    /// 104 bit nonce / 13 byte IV
    #[strum(message = "104")]
    Nonce104,
    /// 96 bit nonce / 12 byte IV, recommended for AES-GCM
    #[default]
    #[strum(message = "96")]
    Nonce96,
}

#[derive(Default, Copy, Clone, Debug, PartialEq, Eq, VariantArray, EnumMessage)]
#[strum(serialize_all = "kebab-case")]
pub enum AesEncDec {
    #[default]
    /// encrypt
    #[strum(message = "Enc")]
    Encrypt,
    /// decrypt
    #[strum(message = "Dec")]
    Decrypt,
}

#[derive(Default, Copy, Clone, Debug, PartialEq, Eq, VariantArray, EnumMessage)]
#[strum(serialize_all = "kebab-case")]
pub enum CryptOutputFormat {
    /// base64
    #[default]
    #[strum(message = "Base64")]
    Base64,
    /// hex
    #[strum(message = "Hex")]
    Hex,
    /// text
    #[strum(message = "Text")]
    Text,
}

#[derive(Default, Clone, PartialEq)]
pub struct CryptOptions {
    pub digest: DigestMenu,
    pub aes_mode: AesMode,
    pub gcm_nonce: GcmNonceLen,
    pub encdec: AesEncDec,
    pub output_format: CryptOutputFormat,
    pub key: String,
    pub iv: String,
}



pub(crate) fn convert(input: &str, options: &CryptOptions) -> ConvertResult {
    match options.digest {
        DigestMenu::Md5 => ConvertResult::Text(format!("{:x}", md5::compute(input.as_bytes()))),
        DigestMenu::Sha1 => hash_input::<sha1::Sha1>(input),
        DigestMenu::Sha224 => hash_input::<sha2::Sha224>(input),
        DigestMenu::Sha256 => hash_input::<sha2::Sha256>(input),
        DigestMenu::Sha384 => hash_input::<sha2::Sha384>(input),
        DigestMenu::Sha512 => hash_input::<sha2::Sha512>(input),
        DigestMenu::Aes128 | DigestMenu::Aes192 | DigestMenu::Aes256 => aes(input, options),
    }
}

fn aes(input: &str, options: &CryptOptions) -> ConvertResult {
    let key_len = match options.digest {
        DigestMenu::Aes128 => 16,
        DigestMenu::Aes192 => 24,
        DigestMenu::Aes256 => 32,
        _ => unreachable!(),
    };

    match options.aes_mode {
        AesMode::Gcm => match options.digest {
            DigestMenu::Aes128 => aes_gcm::<Aes128>(input, options, key_len),
            DigestMenu::Aes192 => aes_gcm::<Aes192>(input, options, key_len),
            DigestMenu::Aes256 => aes_gcm::<Aes256>(input, options, key_len),
            _ => unreachable!(),
        },
        AesMode::Cbc => match options.digest {
            DigestMenu::Aes128 => aes_cbc::<Aes128>(input, options, key_len),
            DigestMenu::Aes192 => aes_cbc::<Aes192>(input, options, key_len),
            DigestMenu::Aes256 => aes_cbc::<Aes256>(input, options, key_len),
            _ => unreachable!(),
        },
        AesMode::Ecb => match options.digest {
            DigestMenu::Aes128 => aes_ecb::<Aes128>(input, options, key_len),
            DigestMenu::Aes192 => aes_ecb::<Aes192>(input, options, key_len),
            DigestMenu::Aes256 => aes_ecb::<Aes256>(input, options, key_len),
            _ => unreachable!(),
        },
    }
}





