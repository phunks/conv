use digest::DynDigest;
use rustc_serialize::hex::ToHex;

fn use_hasher(mut hasher: Box<dyn DynDigest>, data: &[u8]) -> Box<[u8]> {
    hasher.update(data);
    hasher.finalize_reset()
}

fn select_hasher(s: &str) -> Box<dyn DynDigest> {
    match s {
        "sha224" => Box::new(sha2::Sha224::default()),
        "sha256" => Box::new(sha2::Sha256::default()),
        "sha384" => Box::new(sha2::Sha384::default()),
        "sha512" => Box::new(sha2::Sha512::default()),
        _ => unimplemented!("unsupported digest: {}", s),
    }
}

pub fn hasher<'a>(s: &'a str, text: &'a str) -> String {
    use_hasher(select_hasher(s), text.as_ref()).to_hex()
}
