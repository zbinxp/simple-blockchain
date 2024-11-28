use std::collections::{HashMap, HashSet};
use std::fmt;
use std::fmt::Debug;
use log::info;
use crate::block::Block;
use crate::blockchain::Blockchain;
use crate::errors::Result;
use crate::tx::TxOutput;
// use crate::tx::TxOutputs;

pub struct UTXOSet {
    pub blockchain: Blockchain,
}

impl Debug for UTXOSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let db = sled::open(UTXOSet::PATH).map_err(|_| std::fmt::Error)?;
        writeln!(f, "UTXOSet {{ ")?;
        for kv in db.iter() {
            let (k,v) = kv.map_err(|_| std::fmt::Error)?;
            let key = String::from_utf8_lossy(&k);
            writeln!(f, "  {}: [", key)?;
            let value:TxOutput = bincode::deserialize(&v).map_err(|_|fmt::Error)?;
            writeln!(f, "    value:{}, pub_key:{:?}", value.value, value.pub_key_hash)?;
            writeln!(f, "]")?;
        }
        write!(f, "}}")
    }
}

impl UTXOSet {
    const PATH: &'static str = "data/utxos";
    // rebuild the UTXO set
    pub fn reindex(&self) -> Result<()> {
        // storage:
        // key: txid-index, value: binary of TxOutput
        if let Err(_) = std::fs::remove_dir_all(UTXOSet::PATH) {
            info!("directory not found: {}", UTXOSet::PATH);
        }
        let db = sled::open(UTXOSet::PATH)?;
        let utxos = self.blockchain.find_all_utxos();
        for (txid, outs) in &utxos {
            for (idx, out) in outs.outputs.iter().enumerate() {
                let key = Self::construct_key(txid, idx);
                db.insert(key.as_bytes(), bincode::serialize(out)?)?;
            }
        }
        Ok(())
    }

    fn construct_key(txid:&str, index:usize) -> String {
        format!("{txid}-{index}")
    }

    pub fn update(&self, block: &Block) -> Result<()> {
        let db = sled::open(UTXOSet::PATH)?;
        for tx in block.get_transactions() {
            // add TxOutput from block.vout
            for (idx, item) in tx.vout.iter().enumerate() {
                let key = Self::construct_key(tx.id.as_str(), idx);
                db.insert(key.as_bytes(), bincode::serialize(item)?)?;
            }

            if tx.is_coinbase() {
                continue;
            }

            // prev transactions in block.vin are now spent, so remove them
            for item in &tx.vin {
                let key = Self::construct_key(item.txid.as_str(), item.vout as usize);
                db.remove(key.as_bytes())?;
            }
        }
        Ok(())
    }

    pub fn count_transactions(&self) -> Result<usize> {
        let db = sled::open(UTXOSet::PATH)?;
        let mut set = HashSet::new();
        for kv in db.iter() {
            let (k,_) = kv?;
            let txid = String::from_utf8(k.to_vec())?.split('-').next().unwrap().to_string();
            set.insert(txid);
        }
        Ok(set.iter().count())
    }

    pub fn find_spendable_outputs(&self, address:&[u8], amount:i32) -> Result<(i32,HashMap<String, Vec<i32>>)> {
        let mut accum = 0;
        let mut spent_map = HashMap::<String,Vec<i32>>::new();
        let db = sled::open(UTXOSet::PATH)?;
        for kv in db.iter() {
            let (k ,v) = kv?;
            let key = String::from_utf8(k.to_vec())?;
            let itr = key.split('-').collect::<Vec<&str>>();
            let txid = itr[0];
            let idx = itr[1].parse::<i32>()?;
            let out: TxOutput = bincode::deserialize(&v)?;

            if out.can_be_unlock_with(address) && accum < amount {
                accum += out.value;

                match spent_map.get_mut(txid) {
                    Some(val) => val.push(idx),
                    None => {
                        spent_map.insert(txid.to_string(), vec![idx]);
                    }
                }
            }
        }
        Ok((accum, spent_map))
    }

    pub fn find_utxo(&self, pub_key_hash: &[u8]) -> Result<Vec<TxOutput>> {
        let mut utxos = Vec::new();
        let db = sled::open(UTXOSet::PATH)?;
        for kv in db.iter() {
            let (_ ,v) = kv?;
            // let txid = String::from_utf8(k.to_vec())?;
            let out: TxOutput = bincode::deserialize(&v)?;
            if out.can_be_unlock_with(pub_key_hash) {
                utxos.push(out);
            }
        }
        Ok(utxos)
    }
}