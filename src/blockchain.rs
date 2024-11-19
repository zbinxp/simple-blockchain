use crate::block::Block;
use crate::errors::Result;

#[derive(Debug)]
pub struct Blockchain {
    blocks: Vec<Block>
}

impl Blockchain {
    pub fn new() -> Blockchain {
        Blockchain {
            blocks: vec![Block::new_genesis_block()]
        }
    }

    pub fn add_block(&mut self, data: String) -> Result<()>{
        let prev = self.blocks.last().unwrap();
        let new_block = Block::new_block(data, prev.get_hash(),
                                         prev.get_height() + 1)?;
        self.blocks.push(new_block);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn add_block() {
        let mut blockchain = Blockchain::new();
        blockchain.add_block(String::from("data")).unwrap();
        blockchain.add_block(String::from("test2")).unwrap();
        blockchain.add_block(String::from("test3")).unwrap();
        dbg!(blockchain);
    }
}