#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum EthNetwork {
    Main,
    Ropsten,
}

impl EthNetwork {
    pub fn chain_id(&self) -> u64 {
        match self {
            EthNetwork::Main => 1,
            EthNetwork::Ropsten => 3,
        }
    }
}
