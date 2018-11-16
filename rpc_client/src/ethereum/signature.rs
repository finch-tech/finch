#[derive(Clone)]
pub struct Signature([u8; 65]);

impl Signature {
    pub fn new(data: [u8; 65]) -> Self {
        Signature(data)
    }

    pub fn r(&self) -> &[u8] {
        &self.0[0..32]
    }

    pub fn s(&self) -> &[u8] {
        &self.0[32..64]
    }

    pub fn v(&self) -> u8 {
        self.0[64]
    }
}
