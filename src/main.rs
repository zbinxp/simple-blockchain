use crate::blockchain::Blockchain;

mod block;
mod errors;
mod blockchain;

fn main() {
    println!("Hello, world!");

    let mut blockchain = Blockchain::new();
    blockchain.add_block(String::from("data")).unwrap();
    // blockchain.add_block(String::from("test2")).unwrap();
    // blockchain.add_block(String::from("test3")).unwrap();
    dbg!(blockchain);
}
