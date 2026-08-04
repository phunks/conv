use base64::alphabet::Alphabet;
use base64::engine::GeneralPurposeConfig;
use base64::{Engine, engine};

pub trait SliceExt {
    #[allow(unused)]
    fn t_hex(&self) -> Vec<String>;
    #[allow(unused)]
    fn t_dec(&self) -> Vec<u32>;
    fn t_utf8_string(&self) -> String;
    fn t_base64(&self, engine: Alphabet, pad: GeneralPurposeConfig) -> String;
    fn f_base64(&self, engine: Alphabet, pad: GeneralPurposeConfig) -> Result<Vec<u8>, String>;
}

impl SliceExt for [u8] {
    #[inline]
    fn t_hex(&self) -> Vec<String> {
        self.iter()
            .map(|x| format!("{:02x}", x))
            .collect::<Vec<_>>()
    }
    #[inline]
    fn t_dec(&self) -> Vec<u32> {
        self.t_utf8_string()
            .chars()
            .map(|x| x as u32)
            .collect::<Vec<_>>()
    }
    #[inline]
    fn t_utf8_string(&self) -> String {
        String::from_utf8_lossy(self).into_owned()
    }
    #[inline]
    fn t_base64(&self, engine: Alphabet, pad: GeneralPurposeConfig) -> String {
        engine::GeneralPurpose::new(&engine, pad).encode(self)
    }
    #[inline]
    fn f_base64(&self, engine: Alphabet, pad: GeneralPurposeConfig) -> Result<Vec<u8>, String> {
        match engine::GeneralPurpose::new(&engine, pad).decode(self) {
            Ok(a) => Ok(a),
            Err(e) => Err(e.to_string()),
        }
    }
}
const TR_SAFE_URL: [char; 4] = ['/', '+', '_', '-'];

pub trait StringExt {
    #[allow(unused)]
    fn validator_len(&self, n: i8, t: &str) -> Result<Self, String>
    where
        Self: Sized;
    #[allow(unused)]
    fn char_bytestring(&self) -> Vec<u32>;
    #[allow(unused)]
    fn parse_unicode(&self) -> Option<char>;
    fn tr_safe_url(&self) -> String;
    #[allow(unused)]
    fn utf8_bytestring(&self) -> Vec<u8>;
    #[allow(unused)]
    fn utf16_bytestring(&self) -> Vec<u16>;
}

impl StringExt for String {
    #[inline]
    fn validator_len(&self, n: i8, t: &str) -> Result<Self, String> {
        let a = self.chars().count() as i8;
        if a != n && a != 0 {
            return Err(format!("warn: {} length {}/{}", t, a, n));
        };
        Ok(self.into())
    }
    #[inline]
    fn char_bytestring(&self) -> Vec<u32> {
        self.chars().map(|x| x as u32).collect::<Vec<_>>()
    }
    #[inline]
    fn parse_unicode(&self) -> Option<char> {
        let unicode = u32::from_str_radix(self, 16).ok();
        char::from_u32(unicode?)
    }
    #[inline]
    fn tr_safe_url(&self) -> String {
        let mut buf: String = String::with_capacity(self.len());
        for c in self.chars() {
            if let Some(idx) = TR_SAFE_URL.iter().take(2).position(|x| x == &c) {
                buf.push(TR_SAFE_URL[idx + 2]);
                continue;
            }
            buf.push(c);
        }
        buf
    }
    #[inline]
    fn utf8_bytestring(&self) -> Vec<u8> {
        self.chars()
            .map(|x| {
                let mut b = [0; 4];
                x.encode_utf8(&mut b);
                b
            })
            .flat_map(|x| x.into_iter().filter(|x| x != &0))
            .collect::<Vec<_>>()
    }
    #[inline]
    fn utf16_bytestring(&self) -> Vec<u16> {
        self.chars()
            .map(|x| {
                let mut b = [0; 2];
                x.encode_utf16(&mut b);
                b
            })
            .flat_map(|x| x.into_iter().filter(|x| x != &0))
            .collect::<Vec<_>>()
    }
}
