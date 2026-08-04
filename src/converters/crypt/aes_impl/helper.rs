use digest::Digest;
use crate::app::result::ConvertResult;

pub(crate) const AES_BLOCK_LEN: usize = 16;

pub(crate) fn pkcs7_pad(mut bytes: Vec<u8>) -> Vec<u8> {
    let padding_len = AES_BLOCK_LEN - (bytes.len() % AES_BLOCK_LEN);
    bytes.extend(std::iter::repeat_n(padding_len as u8, padding_len));
    bytes
}

pub(crate) fn pkcs7_unpad(bytes: &[u8]) -> Result<Vec<u8>, String> {
    let Some(&padding_len) = bytes.last() else {
        return Err("empty plaintext".to_string());
    };

    let padding_len = padding_len as usize;

    if padding_len == 0 || padding_len > AES_BLOCK_LEN || padding_len > bytes.len() {
        return Err("invalid PKCS#7 padding".to_string());
    }

    if !bytes[bytes.len() - padding_len..]
        .iter()
        .all(|&byte| byte as usize == padding_len)
    {
        return Err("invalid PKCS#7 padding".to_string());
    }

    Ok(bytes[..bytes.len() - padding_len].to_vec())
}

pub(crate) fn hash_input<D: Digest>(input: &str) -> ConvertResult {
    let mut hasher = D::new();
    hasher.update(input.as_bytes());
    let hash = hasher.finalize();
    let hex_string = base16ct::lower::encode_string(&hash);
    ConvertResult::Text(hex_string)
}