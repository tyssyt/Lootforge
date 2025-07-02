use super::ser_v1;

pub const MAGIC: [u8;8] = [b'l', b'o', b'o', b't', b'\xF0', b'\x01', b'\xFF', b'\n'];

pub struct ValidatedBytes {
    pub bytes: Vec<u8>,
    pub version: u8,
    pub ts: u64,
}
impl ValidatedBytes {
    pub fn validate(bytes: Vec<u8>) -> Option<ValidatedBytes> {
        if bytes.len() < 9 || &bytes[0..8] != MAGIC {
            return None
        }
        let version = bytes[8];
        let ts = match version {
            1 => ser_v1::validate(&bytes)?,
            _ => panic!("unknown file version: {}", version)
        };

        Some(ValidatedBytes { bytes, version, ts })
    }
}