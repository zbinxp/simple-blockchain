use std::collections::HashMap;
use crypto::digest::Digest;
use crypto::ed25519;
use crypto::sha2::Sha256;
use failure::format_err;
use log::error;
use rand::RngCore;
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use crate::errors::Result;
use crate::tx::{TxInput, TxOutput};
use crate::utxoset::UTXOSet;
use crate::wallet::{Wallet};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Transaction {
    pub id: String,
    pub vin: Vec<TxInput>,
    pub vout: Vec<TxOutput>,
}

impl Transaction {
    pub fn new_coinbase(to: String, mut data: String) -> Result<Transaction> {
        let mut key = [0u8; 32];
        if data.is_empty() {
            OsRng::default().fill_bytes(&mut key);
            data += &format!("Reward to '{}'", to);
        }
        let mut pub_key = Vec::from(data.as_bytes());
        pub_key.append(&mut Vec::from(key));

        let output = TxOutput::new(100, to)?;
        let mut transaction = Transaction {
            id: String::new(),
            vin: vec![TxInput {
                txid: String::new(),
                vout: -1,
                signature: Vec::new(),
                pub_key
            }],
            vout: vec![output],
        };
        transaction.id = transaction.hash()?;
        Ok(transaction)
    }

    pub fn new_utxo(from:&Wallet, amount:i32, to:&Wallet, utxo_set:&UTXOSet) -> Result<Transaction> {
        let mut pub_key_hash = from.public_key.clone();
        Wallet::hash_pub_key(&mut pub_key_hash);

        let (accum,utxos) = utxo_set.find_spendable_outputs(&pub_key_hash, amount)?;
        if accum < amount {
            error!("can't fulfill the transaction");
            return Err(format_err!("not enough outputs to fulfill transaction, spentable:{accum} < {amount}"));
        }

        let mut vin = Vec::new();
        for (txid, vout) in utxos {
            for out in vout {
                let input = TxInput{
                    txid: txid.clone(),
                    vout: out,
                    signature: Vec::new(),
                    pub_key: from.public_key.clone(),
                };
                vin.push(input);
            }
        }
        let mut vout = Vec::new();
        vout.push(TxOutput::new(amount, to.get_address())?);
        if accum > amount {
           vout.push(TxOutput::new(accum - amount, from.get_address())?);
        }
        let mut tx = Transaction{
            id: String::new(),
            vin,
            vout
        };
        tx.id = tx.hash()?;
        utxo_set.blockchain.sign_transaction(&mut tx, &from.private_key)?;
        Ok(tx)
    }

    pub fn hash(&self) -> Result<String> {
        let mut hasher = Sha256::new();
        hasher.input(&bincode::serialize(&self)?);
        Ok(hasher.result_str())
    }

    pub fn is_coinbase(&self) -> bool {
        self.vin.len() == 1 && self.vin[0].txid.is_empty() && self.vin[0].vout == -1
    }

    pub fn sign(&mut self, private_key: &[u8], prev_tx: HashMap<String, Transaction>) -> Result<()> {
        if self.is_coinbase() {
            return Ok(());
        }
        for vin in &self.vin {
            if prev_tx.get(&vin.txid).unwrap().id.is_empty() {
                return Err(format_err!("can't find transaction with id:{}",vin.txid));
            }
        }
        let mut tx_copy = self.trim_copy();
        for idx in 0..tx_copy.vin.len() {
            let prev_tx = prev_tx.get(&tx_copy.vin[idx].txid).unwrap();
            tx_copy.vin[idx].signature.clear();
            tx_copy.vin[idx].pub_key = prev_tx.vout[tx_copy.vin[idx].vout as usize]
                .pub_key_hash.clone();
            tx_copy.id = tx_copy.hash()?;
            tx_copy.vin[idx].pub_key = Vec::new();
            let sig = ed25519::signature(tx_copy.id.as_bytes(), private_key);
            self.vin[idx].signature = sig.to_vec();
        }
        Ok(())
    }

    pub fn verify(&self, prev_tx: HashMap<String, Transaction>) -> Result<bool> {
        if self.is_coinbase() {
            return Ok(true);
        }
        for vin in &self.vin {
            if prev_tx.get(&vin.txid).unwrap().id.is_empty() {
                return Err(format_err!("can't find transaction with id:{}", vin.txid));
            }
        }
        let mut tx_copy = self.trim_copy();
        for idx in 0..tx_copy.vin.len() {
            let prev_tx = prev_tx.get(&tx_copy.vin[idx].txid).unwrap();
            tx_copy.vin[idx].signature.clear();
            tx_copy.vin[idx].pub_key = prev_tx.vout[tx_copy.vin[idx].vout as usize]
                .pub_key_hash.clone();
            tx_copy.id = tx_copy.hash()?;
            tx_copy.vin[idx].pub_key = Vec::new();
            if !ed25519::verify(tx_copy.id.as_bytes(), &self.vin[idx].pub_key, &self.vin[idx].signature) {
                return Ok(false);
            }
        }
        Ok(true)
    }

    fn trim_copy(&self) -> Transaction {
        let mut vin = Vec::new();
        let mut vout = Vec::new();
        for item in &self.vin {
            let input = TxInput{
                txid: item.txid.clone(),
                vout: item.vout,
                signature: Vec::new(),
                pub_key: Vec::new()
            };
            vin.push(input);
        }
        for item in &self.vout {
            let out = TxOutput{
                value: item.value,
                pub_key_hash: item.pub_key_hash.clone()
            };
            vout.push(out);
        }

        Transaction{
            id: self.id.clone(),
            vin,
            vout
        }
    }
}