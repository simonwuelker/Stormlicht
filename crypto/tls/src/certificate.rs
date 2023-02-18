#[derive(Clone, Debug)]
pub struct X509v3Certificate(Vec<u8>);

impl X509v3Certificate {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }
}
