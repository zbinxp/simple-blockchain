use bitcoincash_addr::Address;
use serde::{Deserialize, Serialize};
use crate::errors::Result;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TxOutputs {
    pub outputs: Vec<TxOutput>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TxInput {
    pub txid: String,
    pub vout: i32,          // specify the utxo in transaction txid
    pub signature: Vec<u8>,
    pub pub_key: Vec<u8>,    // sender's address
}

impl TxInput {
    // pub fn can_be_unlock_output_with(&self, unlock_data: &[u8]) -> bool {
    //     let mut pub_key = self.pub_key.clone();
    //     Wallet::hash_pub_key(&mut pub_key);
    //     self.pub_key == unlock_data
    // }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TxOutput {
    pub value: i32,         // amount
    pub pub_key_hash: Vec<u8>,    // recipient address
}

impl TxOutput {
    pub fn can_be_unlock_with(&self, unlock_data: &[u8]) -> bool {
        self.pub_key_hash == unlock_data
    }

    pub fn get_value(&self) -> i32 {
        self.value
    }

    pub fn new(value: i32, address: String) -> Result<Self> {
        let mut output = TxOutput{
            value,
            pub_key_hash: Vec::new(),
        };
        output.lock(&address)?;
        Ok(output)
    }

    fn lock(&mut self, address: &str) -> Result<()> {
        let pub_key_hash = Address::decode(address).unwrap().body;
        self.pub_key_hash = pub_key_hash;
        Ok(())
    }
}