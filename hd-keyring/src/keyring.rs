use std::str::FromStr;

use bip39::{Language, Mnemonic, MnemonicType};

use bip32::{DerivationPath, Index, XKeyPair};
use errors::Error;
use wallet::Wallet;

use types::BtcNetwork;

#[derive(Debug)]
pub struct HdKeyring {
    pub mnemonic: Mnemonic,
    pub hd_path: DerivationPath,
    pub wallets: Vec<Wallet>,
    hd_wallet: XKeyPair,
    root: XKeyPair,
    pub btc_network: BtcNetwork,
}

impl HdKeyring {
    pub fn new(
        path: &str,
        number_of_accounts: u32,
        btc_network: BtcNetwork,
    ) -> Result<Self, Error> {
        let path = DerivationPath::from_str(path)?;

        let mnemonic = Mnemonic::new(
            MnemonicType::Type12Words,
            Language::English,
            String::from(""),
        )?;

        let mut keyring = HdKeyring::init_from_mnemonic(mnemonic, &path, btc_network)?;
        keyring.load_wallets(number_of_accounts)?;
        Ok(keyring)
    }

    pub fn from_mnemonic(
        path: &str,
        mnemonic: &str,
        number_of_accounts: u32,
        btc_network: BtcNetwork,
    ) -> Result<Self, Error> {
        let path = DerivationPath::from_str(path)?;

        let mnemonic = Mnemonic::from_string(mnemonic, Language::English, "")?;
        let mut keyring = HdKeyring::init_from_mnemonic(mnemonic, &path, btc_network)?;
        keyring.load_wallets(number_of_accounts)?;
        Ok(keyring)
    }

    fn init_from_mnemonic(
        mnemonic: Mnemonic,
        path: &DerivationPath,
        btc_network: BtcNetwork,
    ) -> Result<Self, Error> {
        let master_node = XKeyPair::from_seed(mnemonic.get_seed(), btc_network)?;
        let root = master_node.from_path(path)?;

        Ok(HdKeyring {
            mnemonic,
            root,
            hd_wallet: master_node,
            hd_path: path.clone(),
            wallets: Vec::new(),
            btc_network,
        })
    }

    fn load_wallets(&mut self, number_of_accounts: u32) -> Result<(), Error> {
        for i in 0..number_of_accounts {
            let key_pair = self.root.derive(&Index::Soft(i))?;
            let wallet = Wallet::from_secret_key(*key_pair.xprv().as_raw(), self.btc_network)?;
            self.wallets.push(wallet);
        }

        Ok(())
    }

    pub fn get_wallet_by_index(&self, index: u32) -> Result<Wallet, Error> {
        let key_pair = self.root.derive(&Index::Soft(index - 1))?;
        Wallet::from_secret_key(*key_pair.xprv().as_raw(), self.btc_network)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use types::H160;

    #[test]
    fn create_new_keyring() {
        HdKeyring::new("m/44'/60'/0'/0", 1, BtcNetwork::MainNet).unwrap();
    }

    #[test]
    fn eth_wallet_from_mnemonic() {
        let addresses = vec![H160::from_str("0xD51CE1261D51DBB00A2CCA7FDC8136ABDFFB76B7").unwrap()];

        let keyring = HdKeyring::from_mnemonic(
            "m/44'/60'/0'/0",
            "addict else general weird gospel excite void debate north include exercise liberty",
            1,
            BtcNetwork::MainNet,
        )
        .unwrap();

        for (i, w) in keyring.wallets.into_iter().enumerate() {
            assert_eq!(addresses[i], w.get_eth_address());
        }
    }

    #[test]
    fn btc_wallet_from_mnemonic() {
        let addresses = vec!["195BqgTp3yH1ZWmt1L9LmMkGbTMAc1vGPN"];

        let keyring = HdKeyring::from_mnemonic(
            "m/44'/0'/0'/0",
            "addict else general weird gospel excite void debate north include exercise liberty",
            1,
            BtcNetwork::MainNet,
        )
        .unwrap();

        for (i, w) in keyring.wallets.into_iter().enumerate() {
            assert_eq!(addresses[i], w.get_btc_address());
        }
    }

    #[test]
    fn get_wallet_at_specific_index() {
        let index = 100;
        let address = H160::from_str("0x41CF7938A02B9B27795A8D28C2DE028AA86E8ECB").unwrap();

        let keyring = HdKeyring::from_mnemonic(
            "m/44'/60'/0'/0",
            "addict else general weird gospel excite void debate north include exercise liberty",
            0,
            BtcNetwork::MainNet,
        )
        .unwrap();

        let wallet = keyring.get_wallet_by_index(index).unwrap();

        assert_eq!(address, wallet.get_eth_address());
    }
}
