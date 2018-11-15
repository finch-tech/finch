#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum EthNetwork {
    Main,
    Ropsten,
}

impl EthNetwork {
    pub fn chain_id(&self) -> u64 {
        match self {
            Main => 1,
            Ropsten => 3,
        }
    }
}
