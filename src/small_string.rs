pub struct SmallString {
    pub data: [u8; 128],
    pub length: usize,
}

impl SmallString {
    pub fn new(s: &str) -> SmallString {
        let bytes = s.as_bytes();
        let mut data = [0u8; 128];
        data[..bytes.len()].copy_from_slice(bytes);
        SmallString {
            data,
            length: bytes.len(),
        }
    }
}
