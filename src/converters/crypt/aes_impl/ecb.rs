
use crate::app::result::ConvertResult;
use crate::converters::crypt::{AesEncDec, CryptOptions};
use crate::converters::crypt::aes_impl::gcm::{format_decrypted_bytes, format_encrypted_bytes, parse_ciphertext};
use crate::converters::crypt::aes_impl::helper::{pkcs7_pad, pkcs7_unpad, AES_BLOCK_LEN};
use aes_gcm::aes::cipher::{Block, BlockCipherEncrypt, BlockCipherDecrypt, KeyInit as BlockKeyInit};

pub(crate) fn aes_ecb<Aes>(input: &str, options: &CryptOptions, key_len: usize) -> ConvertResult
where
    Aes: BlockKeyInit + BlockCipherEncrypt + BlockCipherDecrypt,
{
    if options.key.len() != key_len {
        return ConvertResult::Error(format!(
            "warn: key length {}/{} bytes",
            options.key.len(),
            key_len,
        ));
    }

    let cipher = match Aes::new_from_slice(options.key.as_bytes()) {
        Ok(cipher) => cipher,
        Err(error) => return ConvertResult::Error(format!("warn: key error: {error}")),
    };

    match options.encdec {
        AesEncDec::Encrypt => {
            let mut bytes = pkcs7_pad(input.as_bytes().to_vec());
            encrypt_ecb_blocks(&cipher, &mut bytes);
            ConvertResult::Text(format_encrypted_bytes(&bytes, options.output_format))
        }
        AesEncDec::Decrypt => {
            let mut bytes = match parse_ciphertext(input) {
                Ok(bytes) => bytes,
                Err(warnings) => return ConvertResult::Warnings(warnings),
            };

            if bytes.len() % AES_BLOCK_LEN != 0 {
                return ConvertResult::Error(format!(
                    "warn: ciphertext length must be multiple of {AES_BLOCK_LEN} bytes",
                ));
            }

            decrypt_ecb_blocks(&cipher, &mut bytes);

            let plaintext = match pkcs7_unpad(&bytes) {
                Ok(plaintext) => plaintext,
                Err(error) => return ConvertResult::Error(format!("warn: {error}")),
            };

            ConvertResult::Text(format_decrypted_bytes(&plaintext, options.output_format))
        }
    }
}

fn encrypt_ecb_blocks<Aes>(cipher: &Aes, bytes: &mut [u8])
where
    Aes: BlockCipherEncrypt,
    for<'a> &'a mut [u8]: TryInto<&'a mut Block<Aes>>,
{
    let block_len = Aes::block_size();
    for chunk in bytes.chunks_exact_mut(block_len) {
        if let Ok(block) = chunk.try_into() {
            cipher.encrypt_block(block);
        }
    }
}

fn decrypt_ecb_blocks<Aes>(cipher: &Aes, bytes: &mut [u8])
where
    Aes: BlockCipherDecrypt,
    for<'a> &'a mut [u8]: TryInto<&'a mut Block<Aes>>,
{
    let block_len = Aes::block_size();
    for chunk in bytes.chunks_exact_mut(block_len) {
        if let Ok(block) = chunk.try_into() {
            cipher.decrypt_block(block);
        }
    }
}
