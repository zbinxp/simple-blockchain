use crypto::digest::Digest;
use crypto::sha2::Sha256;
use failure::format_err;
use log::error;
use serde::{Deserialize, Serialize};
use crate::blockchain::Blockchain;
use crate::errors::Result;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Transaction {
    pub id: String,
    pub vin: Vec<TxInput>,
    pub vout: Vec<TxOutput>,
}

impl Transaction {
    pub fn new_coinbase(to: String, mut data: String) -> Result<Transaction> {
        if data.is_empty() {
            data += &format!("Reward to '{}'", to);
        }
        let output = TxOutput {
            value: 10,
            pub_key: to.clone(),
        };
        let mut transaction = Transaction {
            id: String::new(),
            vin: vec![TxInput {
                txid: String::new(),
                vout: -1,
                // signature: String::new(),
                pub_key: data
            }],
            vout: vec![output],
        };
        transaction.id = transaction.hash()?;
        Ok(transaction)
    }

    pub fn new_utxo(from:&str, amount:i32, to:&str, bc:&Blockchain) -> Result<Transaction> {
        let (accum,utxos) = bc.find_spendable_outputs(from, amount);
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
                    pub_key: String::from(from)
                };
                vin.push(input);
            }
        }
        let mut vout = Vec::new();
        vout.push(TxOutput{
            value: amount,
            pub_key: String::from(to)
        });
        if accum > amount {
           vout.push(TxOutput{
               value: accum - amount,
               pub_key: String::from(from)
           });
        }
        let mut tx = Transaction{
            id: String::new(),
            vin,
            vout
        };
        tx.id = tx.hash()?;
        Ok(tx)
    }

    fn hash(&self) -> Result<String> {
        let mut hasher = Sha256::new();
        hasher.input(&bincode::serialize(&self)?);
        Ok(hasher.result_str())
    }

    pub fn is_coinbase(&self) -> bool {
        self.vin.len() == 1 && self.vin[0].txid.is_empty() && self.vin[0].vout == -1
    }
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TxInput {
    pub txid: String,
    pub vout: i32,          // specify the utxo in transaction txid
    // signature: String,
    pub_key: String,    // sender's address
}

impl TxInput {
    pub fn can_be_unlock_output_with(&self, unlock_data: &str) -> bool {
        self.pub_key == unlock_data
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TxOutput {
    value: i32,         // amount
    pub_key: String,    // recipient address
}

impl TxOutput {
    pub fn can_be_unlock_with(&self, unlock_data: &str) -> bool {
        self.pub_key == unlock_data
    }

    pub fn get_value(&self) -> i32 {
        self.value
    }
}