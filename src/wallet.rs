use crate::errors::Result;
use std::collections::HashMap;
use bitcoincash_addr::{Address, HashType, Scheme};
use crypto::digest::Digest;
use serde::{Deserialize, Serialize};
use crypto::ed25519;
use crypto::ripemd160::Ripemd160;
use crypto::sha2::Sha256;
use log::info;
use rand::RngCore;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Wallet {
    pub private_key: Vec<u8>,      // account password
    pub public_key: Vec<u8>,       // account id
}

impl Wallet {

    pub fn new() -> Self {
        let mut key = [0u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut key);
        let (private_key, public_key) = ed25519::keypair(&key);
        Wallet {
            private_key: private_key.to_vec(),
            public_key: public_key.to_vec()
        }
    }

    /*
    Address creation:
    A private-public key pair is generated using cryptographic algorithms.
    The public key is hashed (e.g., SHA-256 for Bitcoin)
    and encoded (e.g., Base58Check for Bitcoin).
     */
    pub fn get_address(&self) -> String {
        let mut pub_key = self.public_key.clone();
        Wallet::hash_pub_key(&mut pub_key);
        let address = Address {
            body: pub_key,
            scheme: Scheme::Base58,
            hash_type: HashType::Script,
            ..Default::default()
        };
        address.encode().unwrap()
    }

    pub fn hash_pub_key(pub_key: &mut Vec<u8>) {
        let mut hasher = Sha256::new();
        hasher.input(pub_key);
        hasher.result(pub_key);
        let mut hasher2 = Ripemd160::new();
        hasher2.input(pub_key);
        pub_key.resize(20,0);
        hasher2.result(pub_key);
    }
}

pub struct WalletManager {
    pub wallets: HashMap<String, Wallet>,
}

impl WalletManager {
    pub fn new() -> Result<Self> {
        let mut wallets = HashMap::new();
        let db = sled::open("data/wallets")?;
        for item in db.iter() {
            let (key, value) = item?;
            let address = String::from_utf8(key.to_vec())?;
            let wallet = bincode::deserialize::<Wallet>(&value)?;
            wallets.insert(address, wallet);
        }
        drop(db);
        Ok(WalletManager { wallets })
    }

    pub fn new_wallet(&mut self) -> String {
        let wallet = Wallet::new();
        let address = wallet.get_address();
        self.wallets.insert(address.clone(), wallet);
        info!("create wallet at address {}", address);
        address
    }

    pub fn get_wallet(&self, name: &str) -> Option<&Wallet> {
        self.wallets.get(name)
    }

    pub fn get_all_addresses(&self) -> Vec<String> {
        self.wallets.keys().cloned().collect()
    }

    pub fn save_all(&self) -> Result<()> {
        let db = sled::open("data/wallets")?;
        for (address, wallet) in &self.wallets {
            db.insert(address, bincode::serialize(wallet)?)?;
        }
        db.flush()?;
        drop(db);
        Ok(())
    }
}