#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Network {
    Main,
    Ropsten,
}

impl Network {
    pub fn chain_id(&self) -> u64 {
        match self {
            Network::Main => 1,
            Network::Ropsten => 3,
        }
    }
}
