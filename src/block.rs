use std::time::SystemTime;
use crypto::digest::Digest;
use crypto::sha2::Sha256;
use merkle_cbt::CBMT;
use merkle_cbt::merkle_tree::Merge;
use serde::{Deserialize, Serialize};
use crate::errors::Result;
use crate::transaction::Transaction;

const TARGET_HEX:usize = 4;
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Block {
    timestamp: u128,
    transactions: Vec<Transaction>,
    prev_block_hash: String,
    hash: String,
    height: i32,
    nonce: i32,
}

impl Block {
    pub fn get_hash(&self) -> String {
        self.hash.clone()
    }

    pub fn get_prev_hash(&self) -> String {
        self.prev_block_hash.clone()
    }

    pub fn get_height(&self) -> i32 {
        self.height
    }

    pub fn get_transactions(&self) -> &Vec<Transaction> {
        &self.transactions
    }

    pub(crate) fn new_genesis_block(coinbase: Transaction) -> Block {
        Block::new_block(vec![coinbase], String::new(), 0).unwrap()
    }

    pub fn new_block(data: Vec<Transaction>, prev_block_hash: String, height:i32) -> Result<Block> {
        let timestamp = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?.as_millis();
        let transactions = data.clone();
        let hash = String::new();
        let nonce = 0;
        let mut block = Block {
            timestamp,
            transactions,
            prev_block_hash,
            hash,
            height,
            nonce
        };
        block.run_proof_of_work()?;
        Ok(block)
    }

    fn run_proof_of_work(&mut self) -> Result<()> {
        while !self.validate()? {
            self.nonce += 1;
        }
        // mining success
        let data = self.prepare_hash_data()?;
        let mut hasher = Sha256::new();
        hasher.input(&data[..]);
        self.hash = hasher.result_str();
        Ok(())
    }

    fn validate(&self) -> Result<bool> {
        let data = self.prepare_hash_data()?;
        let mut hasher = Sha256::new();
        hasher.input(&data[..]);
        let res = hasher.result_str()[0..TARGET_HEX].as_bytes()
            .iter().all(|&b| b == '0' as u8);
        Ok(res)
        // let mut vec1: Vec<u8> = Vec::new();
        // vec1.resize(TARGET_HEX, '0' as u8);
        // Ok(&hasher.result_str()[0..TARGET_HEX] == String::from_utf8(vec1)?)

    }

    fn prepare_hash_data(&self) -> Result<Vec<u8>> {
        let data = (
            self.prev_block_hash.clone(),
            self.timestamp,
            self.hash_transaction()?,
            TARGET_HEX,
            self.nonce);
        let bytes = bincode::serialize(&data)?;
        Ok(bytes)
    }

    fn hash_transaction(&self) -> Result<Vec<u8>> {
        let mut txs = Vec::new();
        for tx in &self.transactions {
            txs.push(tx.hash()?.as_bytes().to_vec());
        }
        let tree = CBMT::<Vec<u8>, MergeTX>::build_merkle_tree(&txs);
        Ok(tree.root())
    }
}

struct MergeTX {}

impl Merge for MergeTX{
    type Item = Vec<u8>;

    fn merge(left: &Self::Item, right: &Self::Item) -> Self::Item {
        let mut hasher = Sha256::new();
        let mut input = left.clone();
        input.append(&mut right.clone());
        hasher.input(&input[..]);
        let mut output = [0u8; 32];
        hasher.result(&mut output);
        output.to_vec()
    }
}