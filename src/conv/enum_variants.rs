
use strum::{EnumMessage, VariantArray};


#[derive(
    Default, Copy, Clone,
    Debug, PartialEq,
    VariantArray, EnumMessage,
)]
#[strum(serialize_all = "kebab-case")]
pub enum Conv {
    #[default]
    /// base64
    #[strum(message = "Base64          ▸")]
    Base64,
    /// binary
    #[strum(message = "Binary          ▸")]
    Binary,
    /// escape
    #[strum(message = "Escape          ▸")]
    Escape,
    /// to utf-7 (rfc 3501)
    #[strum(message = "To UTF-7")]
    ToUtf7,
    /// from utf-7 (rfc 3501)
    #[strum(message = "From UTF-7")]
    FromUtf7,
    /// Crypt
    #[strum(message = "Crypt           ▸")]
    Crypt,
}

#[derive(
    Default, Copy, Clone,
    Debug, PartialEq,
    VariantArray, EnumMessage,
)]
#[strum(serialize_all = "kebab-case")]
pub enum Digest {
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
}

#[derive(
    Default, Copy, Clone,
    Debug, PartialEq,
    VariantArray, EnumMessage,
    )]
#[strum(serialize_all = "kebab-case")]
pub enum Base64Kind {
    #[default]
    /// to base 64 (rfc 4648)
    #[strum(message = "To Base64")]
    ToBase64,
    /// to base 64 url (rfc 4648 url safe)
    #[strum(message = "To Base64URL")]
    ToBase64Url,
    /// from base 64
    #[strum(message = "From Base64")]
    FromBase64,
    /// to deflated saml auth
    #[strum(message = "To Deflated Saml")]
    ToDeflatedSaml,
    /// from deflated saml auth
    #[strum(message = "From Deflated Saml")]
    FromDeflatedSaml,
}

#[derive(
    Default, Copy, Clone,
    Debug, PartialEq,
    VariantArray, EnumMessage,
)]
#[strum(serialize_all = "kebab-case")]
pub enum BinaryKind {
    #[default]
    /// From UTF-8 to Hex
    /// ex: '𝕊☺a' = 'f09d958ae298ba61'
    #[strum(message = "Hex Encode")]
    HexEncode,
    /// From Hex to UTF-8
    /// ex: 'f09d958ae298ba61' = '𝕊☺a'
    #[strum(message = "Hex Decode")]
    HexDecode,
    /// To byte string
    /// ex: '𝕊☺a' = '0xf0, 0x9d, 0x95, 0x8a, 0xe2, 0x98, 0xba, 0x61'
    #[strum(message = "To byte string")]
    ToByteString,
    /// From byte string
    /// ex: '0xf0, 0x9d, 0x95, 0x8a, 0xe2, 0x98, 0xba, 0x61' = '𝕊☺a'
    #[strum(message = "From byte string")]
    FromByteString,
}

#[derive(
    Default, Copy, Clone,
    Debug, PartialEq,
    VariantArray, EnumMessage,
)]
#[strum(serialize_all = "kebab-case")]
pub enum EscapeKind {
    #[default]
    /// url encode
    /// ex: '𝕊☺a' = '%F0%9D%95%8A%E2%98%BAa'
    #[strum(message = "Url Encode")]
    UrlEncode,
    /// url decode
    /// ex: '%F0%9D%95%8A%E2%98%BAa' = '𝕊☺a'
    #[strum(message = "Url Decode")]
    UrlDecode,
    /// To js string (JS6)
    /// ex: '𝕊☺a' = '\u{1d54a}\u{263a}\u{61}'
    #[strum(message = "To JS String")]
    ToJsString,
    /// from js string (JS6)
    /// ex: '\u{1d54a}\u{263a}\u61' = '𝕊☺a'
    #[strum(message = "From JS String")]
    FromJsString,
    /// To HTML Numeric Entities
    /// ex: '𝕊☺a' = '&#x1d54a, &#x263a, &#x61'
    #[strum(message = "To Html Numeric Entities")]
    ToHtmlNumEntities,
    /// From HTML Numeric Entities
    /// ex: '&#x1d54a, &#x263a, &#x61' = '𝕊☺a'
    #[strum(message = "From Html Numeric Entities")]
    FromHtmlNumEntities,
    /// To Html sanitise
    /// ex: '<' = '&lt;'
    #[strum(message = "To Html Sanitise")]
    ToHtmlSanitise,
    /// From Html sanitise
    /// ex: '&#9787;' = '☻'
    #[strum(message = "From Html Sanitise")]
    FromHtmlSanitise,
}