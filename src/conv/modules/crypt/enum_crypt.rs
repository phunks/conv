use strum_macros::{EnumMessage, VariantArray};

#[derive(Default, Copy, Clone, Debug, PartialEq, VariantArray, EnumMessage)]
#[strum(serialize_all = "kebab-case")]
pub enum AesEncDec {
    #[default]
    /// AES Encryption
    #[strum(message = "Enc")]
    AesEnc,
    /// AES Decryption
    #[strum(message = "Dec")]
    AesDec,
}

#[derive(Default, Copy, Clone, Debug, PartialEq, VariantArray, EnumMessage)]
#[strum(serialize_all = "kebab-case")]
pub enum AesMode {
    /// GCM
    #[default]
    #[strum(message = "GCM")]
    Gcm,
    // /// ECB
    // #[strum(message = "ECB")]
    // Ecb,
    // /// CBC
    // #[strum(message = "CBC")]
    // Cbc,
    // /// CTR
    // #[strum(message = "CTR")]
    // Ctr,
}

#[derive(Default, Copy, Clone, Debug, PartialEq, VariantArray, EnumMessage)]
#[strum(serialize_all = "kebab-case")]
pub enum AesPadding {
    #[default]
    /// PKCS5
    #[strum(message = "PKCS5")]
    Pkcs5,
    /// None
    #[strum(message = "None")]
    None,
}

#[derive(Default, Copy, Clone, Debug, PartialEq, VariantArray, EnumMessage)]
#[strum(serialize_all = "kebab-case")]
pub enum GcmTagLen {
    #[default]
    /// tag 128 bit, 16 char length
    #[strum(message = "128")]
    Tag128,
    /// tag 120 bit
    #[strum(message = "120")]
    Tag120,
    /// tag 112 bit
    #[strum(message = "112")]
    Tag112,
    /// tag 104 bit
    #[strum(message = "104")]
    Tag104,
    /// tag 96 bit, 12 char length
    #[strum(message = "96")]
    Tag96,
}

#[derive(Default, Copy, Clone, Debug, PartialEq, VariantArray, EnumMessage)]
#[strum(serialize_all = "kebab-case")]
pub enum EncTextFormat {
    #[default]
    /// Base64
    #[strum(message = "Base64")]
    Base64,
    /// Hex
    #[strum(message = "Hex")]
    Hex,
}

#[derive(Default, Copy, Clone, Debug, PartialEq, VariantArray, EnumMessage)]
#[strum(serialize_all = "kebab-case")]
pub enum DecTextFormat {
    #[default]
    /// Plain Text
    #[strum(message = "Text")]
    Text,
    /// Base64
    #[strum(message = "Base64")]
    Base64,
}
