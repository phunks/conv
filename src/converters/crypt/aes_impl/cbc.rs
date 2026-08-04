use crate::app::result::ConvertResult;
use crate::converters::crypt::aes_impl::helper::{pkcs7_pad, pkcs7_unpad, AES_BLOCK_LEN};
use crate::converters::crypt::{AesEncDec, CryptOptions};
use crate::converters::crypt::aes_impl::gcm::{format_decrypted_bytes, format_encrypted_bytes, parse_ciphertext};
use aes_gcm::aes::cipher::{Block, BlockCipherEncrypt, BlockCipherDecrypt, KeyInit as BlockKeyInit};

pub(crate) fn aes_cbc<Aes>(input: &str, options: &CryptOptions, key_len: usize) -> ConvertResult
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

    let iv = if options.iv.is_empty() {
        [0u8; AES_BLOCK_LEN]
    } else {
        match <[u8; AES_BLOCK_LEN]>::try_from(options.iv.as_bytes()) {
            Ok(iv) => iv,
            Err(_) => {
                return ConvertResult::Error(format!(
                    "warn: iv length {}/{} bytes",
                    options.iv.len(),
                    AES_BLOCK_LEN,
                ));
            }
        }
    };

    let cipher = match Aes::new_from_slice(options.key.as_bytes()) {
        Ok(cipher) => cipher,
        Err(error) => return ConvertResult::Error(format!("warn: key error: {error}")),
    };

    match options.encdec {
        AesEncDec::Encrypt => {
            let mut bytes = pkcs7_pad(input.as_bytes().to_vec());
            encrypt_cbc_blocks(&cipher, &iv, &mut bytes);
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

            decrypt_cbc_blocks(&cipher, &iv, &mut bytes);

            let plaintext = match pkcs7_unpad(&bytes) {
                Ok(plaintext) => plaintext,
                Err(error) => return ConvertResult::Error(format!("warn: {error}")),
            };

            ConvertResult::Text(format_decrypted_bytes(&plaintext, options.output_format))
        }
    }
}

fn encrypt_cbc_blocks<Aes>(cipher: &Aes, iv: &[u8; AES_BLOCK_LEN], bytes: &mut [u8])
where
    Aes: BlockCipherEncrypt,
    for<'a> &'a mut [u8]: TryInto<&'a mut Block<Aes>>,
{
    let mut previous = *iv;
    #[allow(clippy::chunks_exact_to_as_chunks)]
    for chunk in bytes.chunks_exact_mut(AES_BLOCK_LEN) {
        for index in 0..AES_BLOCK_LEN {
            chunk[index] ^= previous[index];
        }

        if let Ok(block) = chunk.try_into() {
            cipher.encrypt_block(block);
        }
        previous.copy_from_slice(chunk);
    }
}

fn decrypt_cbc_blocks<Aes>(cipher: &Aes, iv: &[u8; AES_BLOCK_LEN], bytes: &mut [u8])
where
    Aes: BlockCipherDecrypt,
    for<'a> &'a mut [u8]: TryInto<&'a mut Block<Aes>>,
    Block<Aes>: Default,
{
    let mut previous = *iv;

    let mut ciphertext_block = Block::<Aes>::default();
    #[allow(clippy::chunks_exact_to_as_chunks)]
    for chunk in bytes.chunks_exact_mut(AES_BLOCK_LEN) {
        ciphertext_block.copy_from_slice(chunk);

        if let Ok(block) = chunk.try_into() {
            cipher.decrypt_block(block);
        }

        for index in 0..AES_BLOCK_LEN {
            chunk[index] ^= previous[index];
        }

        previous.copy_from_slice(ciphertext_block.as_slice());
    }
}