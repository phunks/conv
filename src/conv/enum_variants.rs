
use strum::{EnumDiscriminants, EnumMessage, EnumString, VariantArray, VariantNames};
use strum_macros::Display;

#[derive(
    Copy,
    Clone,
    Debug,
    PartialEq,
    EnumString,
    VariantNames,
    VariantArray,
    EnumMessage,
    EnumDiscriminants,
)]
#[strum(serialize_all = "kebab-case")]
pub enum Conv {
    #[strum(message = "To Base64")]
    /// to base 64 (rfc 4648)
    ToBase64,
    #[strum(message = "To Base64URL")]
    /// to base 64 url
    /// rfc 4648 url safe
    ToBase64Url,
    #[strum(message = "From Base64")]
    /// from base 64
    FromBase64,
    #[strum(message = "Url Encode")]
    /// url encode
    /// TODO rfc 3986
    UrlEncode,
    #[strum(message = "Url Decode")]
    /// url decode
    UrlDecode,
    #[strum(message = "Hex Encode")]
    /// From UTF-8 to Hex ex: '☺' = 'e298ba'
    HexEncode,
    #[strum(message = "Hex Decode")]
    /// From Hex to UTF-8 ex: 'e298ba' = '☺'
    HexDecode,
    #[strum(message = "To byte string")]
    /// To byte string  ex: 0x31, 0x34
    ToByteString,
    #[strum(message = "To JS String")]
    /// To js string. ex: \u3042\u3042..
    ToJsString,

    // TODO
    // #[strum(message = "From JS String")]
    // /// from js string
    // FromJsString,

    #[strum(message = "To Html Numeric Entities")]
    /// To HTML Numeric Entities ex: '𝕊' = &#120138
    ToHtmlNumEntities,
    // TODO
    // #[strum(message = "From Html Numeric Entities")]
    // /// From HTML Numeric Entities ex: &#120138 = '𝕊'
    // FromHtmlNumEntities,

    #[strum(message = "To Html Sanitise")]
    /// To Html sanitise ex: '<' = '&lt;'
    ToHtmlSanitise,
    #[strum(message = "From Html Sanitise")]
    /// From Html sanitise ex: '&#9787;' = '☻'
    FromHtmlSanitise,
    #[strum(message = "To UTF-7")]
    /// to utf-7 (rfc 3501)
    ToUtf7,
    #[strum(message = "From UTF-7")]
    /// from utf-7 (rfc 3501)
    FromUtf7,
    #[strum(message = "To Deflated Saml")]
    /// to deflated saml auth
    ToDeflatedSaml,
    #[strum(message = "From Deflated Saml")]
    /// from deflated saml auth
    FromDeflatedSaml,
    #[strum(message = "Crypt")]
    /// Crypt
    Crypt,
}

#[derive(
    Default,
    Copy,
    Clone,
    Debug,
    PartialEq,
    EnumString,
    VariantNames,
    VariantArray,
    EnumMessage,
    EnumDiscriminants,
    Display
)]
#[strum(serialize_all = "kebab-case")]
pub enum Digest {
    #[default]
    #[strum(message = "MD5")]
    /// md5 digest
    Md5,
    #[strum(message = "SHA-1")]
    /// sha1
    Sha1,
    #[strum(message = "SHA-224")]
    /// sha224
    Sha224,
    #[strum(message = "SHA-256")]
    /// sha256
    Sha256,
    #[strum(message = "SHA-384")]
    /// sha384
    Sha384,
    #[strum(message = "SHA-512")]
    /// sha512
    Sha512,
}