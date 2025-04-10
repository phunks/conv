use crate::conv::Editor;
use crate::conv::enum_variants::DigestMenu;
use crate::conv::util::ext::{SliceExt, StringExt};
use aead::{Aead, KeyInit};
use aes_gcm::{AesGcm, Nonce};
use base64::alphabet::STANDARD;
use base64::engine::general_purpose::PAD;
use cipher::BlockCipherEncrypt;
use eframe::egui::{Align, Color32, Layout, RichText, Ui};
use hybrid_array::Array;
use rustc_serialize::hex::{FromHex, ToHex};
use strum::EnumMessage;

pub fn aes<Aes>(editor: &mut Editor) -> Vec<String>
where
    Aes: BlockCipherEncrypt,
    AesGcm<Aes, aead::consts::U16>: KeyInit + Aead,
    AesGcm<Aes, aead::consts::U15>: KeyInit + Aead,
    AesGcm<Aes, aead::consts::U14>: KeyInit + Aead,
    AesGcm<Aes, aead::consts::U13>: KeyInit + Aead,
    AesGcm<Aes, aead::consts::U12>: KeyInit + Aead,
{
    match editor.menu.aes.tag {
        GcmTagLen::Tag128 => aes_gcm_enc::<Aes, aead::consts::U16>(editor),
        GcmTagLen::Tag120 => aes_gcm_enc::<Aes, aead::consts::U15>(editor),
        GcmTagLen::Tag112 => aes_gcm_enc::<Aes, aead::consts::U14>(editor),
        GcmTagLen::Tag104 => aes_gcm_enc::<Aes, aead::consts::U13>(editor),
        GcmTagLen::Tag96 => aes_gcm_enc::<Aes, aead::consts::U12>(editor),
    }
}

fn aes_gcm_enc<Aes, N>(editor: &mut Editor) -> Vec<String>
where
    AesGcm<Aes, N>: KeyInit + Aead,
{
    let mut v = vec![];
    let n = match editor.menu.digest {
        DigestMenu::Aes128 => 16,
        DigestMenu::Aes192 => 24,
        DigestMenu::Aes256 => 32,
        _ => 0,
    };

    let key = editor.aes.key.validator_len(n, "key").unwrap_or_else(|e| {
        v.push(e);
        Default::default()
    });

    let n = match editor.menu.aes.tag.get_message() {
        None => 0,
        Some(a) => (a.parse::<usize>().unwrap() / 8) as i8,
    };
    let nonce = Array::try_from(
        editor
            .aes
            .iv
            .validator_len(n, "nonce")
            .unwrap_or_else(|e| {
                v.push(e);
                Default::default()
            })
            .as_bytes(),
    )
    .unwrap_or_else(|_e| Nonce::default());

    let key = aes_gcm::Key::<AesGcm<Aes, N>>::try_from(key.as_bytes()).unwrap_or_else(|e| {
        editor.output.clear();
        v.push(format!("warn: {}", e));
        Default::default()
    });

    let cipher = <AesGcm<Aes, N>>::new(&key);

    match editor.menu.aes.encdec {
        AesEncDec::AesEnc => {
            let ciphertext = cipher.encrypt(&nonce, editor.code.as_ref());
            if v.is_empty() {
                v.push(match editor.menu.aes.enc_fmt {
                    EncTextFormat::Hex => ciphertext.unwrap().to_hex(),
                    EncTextFormat::Base64 => ciphertext.unwrap().t_base64(STANDARD, PAD),
                });
            }
        }
        AesEncDec::AesDec => {
            let ciphertext = editor.code.from_hex().unwrap_or_else(|e| {
                let prev = e.to_string();
                match editor.code.as_bytes().f_base64(STANDARD, PAD) {
                    Ok(a) => a,
                    Err(e) => {
                        [prev, e]
                            .iter()
                            .for_each(|i| v.push(format!("warn: {}", i)));
                        Default::default()
                    }
                }
            });
            let ciphertext = cipher
                .decrypt(nonce.as_ref(), ciphertext.as_ref())
                .unwrap_or_else(|e| {
                    v.push(format!("warn: {}", e));
                    Default::default()
                });
            if v.is_empty() {
                v.push(match editor.menu.aes.dec_fmt {
                    DecTextFormat::Text => ciphertext.t_utf8_string(),
                    DecTextFormat::Base64 => ciphertext.t_base64(STANDARD, PAD),
                });
            }
        }
    }
    v
}

pub trait Colorize {
    fn color(&self, ui: &mut Ui, editor: &mut Editor);
}

impl Colorize for Vec<String> {
    fn color(&self, ui: &mut Ui, editor: &mut Editor) {
        editor.output.clear();
        ui.with_layout(Layout::left_to_right(Align::TOP), |ui| {
            ui.horizontal_wrapped(|ui| {
                self.iter().for_each(|x| {
                    if x.starts_with("warn:") {
                        let s = x.split(": ").collect::<Vec<_>>();
                        ui.label(RichText::new(format!("{}: ", s[0])).color(Color32::ORANGE));
                        ui.label(
                            RichText::new(format!("{}\n", s[1]))
                                .color(Color32::from_rgb(180, 190, 120)),
                        );
                    } else {
                        editor.output = x.to_string();
                    }
                });
            });
        });
    }
}

use crate::conv::modules::crypt::enum_crypt::{AesEncDec, DecTextFormat, EncTextFormat, GcmTagLen};
use crypto2::blockmode::Aes128Ecb;

fn aes_ecb_enc<A>(editor: &mut Editor, a: A) -> Vec<String>
// where
//     Aes: KeyInit + Aead,
{
    let mut v = vec![];
    let n = match editor.menu.digest {
        DigestMenu::Aes128 => 16,
        DigestMenu::Aes192 => 24,
        DigestMenu::Aes256 => 32,
        _ => 0,
    };

    let key = editor.aes.key.validator_len(n, "key").unwrap_or_else(|e| {
        v.push(e);
        Default::default()
    });

    let mut cipher = Aes128Ecb::new(key.as_bytes());

    match editor.menu.aes.encdec {
        AesEncDec::AesEnc => {
            let mut ciphertext = editor.code.as_bytes().to_vec();
            cipher.encrypt(ciphertext.as_mut());
            if v.is_empty() {
                v.push(match editor.menu.aes.enc_fmt {
                    EncTextFormat::Hex => ciphertext.to_hex(),
                    EncTextFormat::Base64 => ciphertext.t_base64(STANDARD, PAD),
                });
            }
        },
        AesEncDec::AesDec => {
            let ciphertext = editor.code.from_hex().unwrap_or_else(|e| {
                let prev = e.to_string();
                match editor.code.as_bytes().f_base64(STANDARD, PAD) {
                    Ok(a) => a,
                    Err(e) => {
                        [prev, e]
                            .iter()
                            .for_each(|i| v.push(format!("warn: {}", i)));
                        Default::default()
                    }
                }
            });
            // let ciphertext = cipher.decrypt(nonce.as_ref(), ciphertext.as_ref())
            //     .unwrap_or_else(|e| {
            //         v.push(format!("warn: {}", e));
            //         Default::default()
            //     });
            if v.is_empty() {
                v.push(match editor.menu.aes.dec_fmt {
                    DecTextFormat::Text => ciphertext.t_utf8_string(),
                    DecTextFormat::Base64 => ciphertext.t_base64(STANDARD, PAD),
                });
            }
        }
    }
    v
}

#[cfg(test)]
mod tests {
    use cipher::block_padding::{Padding, Pkcs7};
    use crypto2::blockmode::Aes128Ecb;
    use hybrid_array::{Array, typenum::U8};
    use rustc_serialize::hex::FromHex;

    #[test]
    fn test_pkcs7() {
        let msg = b"testtes";
        let pos = msg.len();
        let mut block: Array<u8, U8> = Array([0xff; 8]);
        block[..pos].copy_from_slice(msg);
        Pkcs7::pad(&mut block, pos);

        println!("{:?}", &block[..]);
        // assert_eq!(&block[..], b"test\x04\x04\x04\x04");
        // let res = Pkcs7::unpad(&block).unwrap();
        // assert_eq!(res,msg);
    }

    #[test]
    fn test_aes128_ecb_enc() {
        // F.1.1  ECB-AES128.Encrypt, (Page-31)
        // https://nvlpubs.nist.gov/nistpubs/Legacy/SP/nistspecialpublication800-38a.pdf

        let binding = "2b7e151628aed2a6abf7158809cf4f3c".from_hex().unwrap();
        let key = binding;
        let mut cipher = Aes128Ecb::new(&key);

        let mut binding = "\
6bc1bee22e409f96e93d7e117393172a\
ae2d8a571e03ac9c9eb76fac45af8e51\
30c81c46a35ce411e5fbc1191a0a52ef\
f69f2445df4f9b17ad2b417be66c3710"
            .from_hex()
            .unwrap();
        let mut ciphertext = binding.as_mut_slice();
        cipher.encrypt(&mut ciphertext);

        assert_eq!(
            &ciphertext[..],
            "\
3ad77bb40d7a3660a89ecaf32466ef97\
f5d3d58503b9699de785895a96fdbaaf\
43b1cd7f598ece23881b00e3ed030688\
7b0c785e27e8ad3f8223207104725dd4"
                .from_hex()
                .unwrap()
                .as_slice()
        );
    }

    #[test]
    fn test_aes128_ecb_dec() {
        // F.1.2  ECB-AES128.Decrypt, (Page-31)
        // https://nvlpubs.nist.gov/nistpubs/Legacy/SP/nistspecialpublication800-38a.pdf
        let binding = "2b7e151628aed2a6abf7158809cf4f3c".from_hex().unwrap();
        let key = binding.as_slice();

        let mut cipher = Aes128Ecb::new(&key);

        let mut binding = "\
3ad77bb40d7a3660a89ecaf32466ef97\
f5d3d58503b9699de785895a96fdbaaf\
43b1cd7f598ece23881b00e3ed030688\
7b0c785e27e8ad3f8223207104725dd4"
            .from_hex()
            .unwrap();
        let mut ciphertext = binding.as_mut_slice();

        cipher.decrypt(&mut ciphertext);

        assert_eq!(
            &ciphertext[..],
            "\
6bc1bee22e409f96e93d7e117393172a\
ae2d8a571e03ac9c9eb76fac45af8e51\
30c81c46a35ce411e5fbc1191a0a52ef\
f69f2445df4f9b17ad2b417be66c3710"
                .from_hex()
                .unwrap()
                .as_slice()
        );
    }
}
