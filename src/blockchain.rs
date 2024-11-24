use std::collections::{HashMap, HashSet};
use log::{info};
use crate::block::Block;
use crate::errors::Result;
use crate::transaction::{Transaction};
use crate::tx::TxOutput;

#[derive(Debug)]
pub struct Blockchain {
    db: sled::Db,
    current_hash: String,
}

impl Blockchain {
    pub fn new() -> Result<Blockchain> {
        let db = sled::open("data/blocks")?;
        match db.get("LAST")? {
            Some(hash) => {
                Ok(Blockchain{db, current_hash: String::from_utf8(hash.to_vec())?})
            }
            None => {
                panic!("must create a new blockchain first")
            }
        }
    }

    pub fn create_blockchain(address: String) -> Result<Blockchain> {
        info!("Creating new blockchain at address {}", address);
        let db = sled::open("data/blocks")?;
        let coinbase = Transaction::new_coinbase(address, String::from("Genesis Block"))?;
        let block = Block::new_genesis_block(coinbase);
        db.insert(block.get_hash(), bincode::serialize(&block)?)?;
        db.insert("LAST", block.get_hash().as_bytes())?;
        db.flush()?;
        Ok(Blockchain{db, current_hash: block.get_hash()})
    }

    pub fn add_block(&mut self, data: Vec<Transaction>) -> Result<()>{
        let value = self.db.get("LAST")?.unwrap();
        let current_hash = String::from_utf8(value.to_vec())?;
        if let Some(value) = self.db.get(&current_hash)? {
            let block = bincode::deserialize::<Block>(&value)?;
            let new_block = Block::new_block(data, current_hash.clone(), block.get_height()+1)?;
            self.db.insert(new_block.get_hash(), bincode::serialize(&new_block)?)?;
            self.db.insert("LAST", new_block.get_hash().as_bytes())?;
            self.current_hash = new_block.get_hash();
        }

        Ok(())
    }

    pub fn find_utxo(&self, address:&[u8]) -> Vec<TxOutput> {
        // store id+index of all spent txoutput
        let mut spent_ids = HashMap::<String,Vec<i32>>::new();
        let mut utxos = Vec::<TxOutput>::new();
        for block in self.iter() {
            for transaction in block.get_transactions() {
                for (idx,tout) in transaction.vout.iter().enumerate() {
                    if !tout.can_be_unlock_with(address) {
                        continue;
                    }
                    if !spent_ids.contains_key(&transaction.id)
                        || !spent_ids[&transaction.id].contains(&(idx as i32)) {
                        utxos.push(tout.clone());
                    }
                }

                if transaction.is_coinbase() {
                    continue;
                }
                for tin in &transaction.vin {
                    match spent_ids.get_mut(&tin.txid) {
                        Some(ids) => ids.push(tin.vout),
                        None => { spent_ids.insert(tin.txid.clone(),vec![tin.vout]); }
                    }
                }
            }
        }
        dbg!(&spent_ids);
        utxos
    }

    pub fn find_spendable_outputs(&self, address:&[u8], amount:i32) -> (i32,HashMap<String, Vec<i32>>) {
        let mut accum = 0;
        let mut spent_map = HashMap::<String,Vec<i32>>::new();
        let mut utxos = HashMap::<String,Vec<i32>>::new();
        for block in self.iter() {
            for transaction in block.get_transactions() {
                for (idx,tout) in transaction.vout.iter().enumerate() {
                    if !tout.can_be_unlock_with(address) {
                        continue;
                    }
                    if !spent_map.contains_key(&transaction.id) || !spent_map[&transaction.id].contains(&(idx as i32)) {
                        utxos.insert(transaction.id.clone(), vec![idx as i32]);
                        accum += tout.get_value();
                        if accum >= amount {
                            return (accum,utxos);
                        }
                    }
                }

                if transaction.is_coinbase() {
                    continue;
                }
                for tin in &transaction.vin {
                    if !tin.can_be_unlock_output_with(address) {
                        continue;
                    }
                    match spent_map.get_mut(&transaction.id) {
                        Some(val) => val.push(tin.vout),
                        None => {
                            spent_map.insert(transaction.id.clone(), vec![tin.vout]);
                        }
                    }
                }
            }
        }
        (accum, spent_map)
    }

    pub fn iter(&self) -> BlockChainIterator {
        BlockChainIterator {
            chain: &self,
            current_hash: self.current_hash.clone(),
        }
    }

    pub fn sign_transaction(&self, tx: &mut Transaction, private_key: &[u8]) ->Result<()> {
        let prev_txs = self.get_prev_txs(tx)?;
        tx.sign(private_key, prev_txs)?;
        Ok(())
    }

    pub fn verify_transaction(&self, tx: &Transaction) -> Result<bool> {
        let prev_txs = self.get_prev_txs(tx)?;
        tx.verify(prev_txs)
    }

    fn get_prev_txs(&self, tx: &Transaction) -> Result<HashMap<String,Transaction>> {
        let mut txs = HashMap::<String,Transaction>::new();
        let ids: HashSet<String> = tx.vin.iter().map(|vin| vin.txid.clone()).collect();
        for block in self.iter() {
            for transaction in block.get_transactions() {
                if ids.contains(&transaction.id) {
                    txs.insert(transaction.id.clone(), transaction.clone());
                }
            }
        }
        Ok(txs)
    }
}

pub struct BlockChainIterator<'a> {
    chain: &'a Blockchain,
    current_hash: String,
}
impl<'a> Iterator for BlockChainIterator<'a> {
    type Item = Block;

    fn next(&mut self) -> Option<Self::Item> {
        // get block using iter.current_hash
        match self.chain.db.get(&self.current_hash) {
            Ok(Some(value)) => {
                let block = bincode::deserialize::<Block>(&value).unwrap();
                self.current_hash = block.get_prev_hash();
                Some(block)
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    //use super::*;

}