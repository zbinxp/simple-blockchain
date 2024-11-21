
use crate::blockchain::Blockchain;
use crate::errors::Result;
use clap::{arg, Command};
use crate::transaction::Transaction;

pub struct Cli {

}

impl Cli {
    pub fn new() -> Result<Self> {
        Ok(Self {})
    }

    pub fn run(&mut self) -> Result<()> {
        let matches = Command::new("blockchain-rust-demo")
            .version("0.1.0")
            .about("a simple blockchain for learning")
            .subcommand(
                Command::new("printchain").about("print all the block chain")
            )
            .subcommand(
                Command::new("getbalance").about("get balance in the chain")
                    .arg(arg!(<ADDRESS>"'the address of the balance'")))
            .subcommand(Command::new("create").about("create new chain")
                .arg(arg!(<ADDRESS>"'the address to send the genesis block to'")))
            .subcommand(Command::new("transfer").about("transfer in the chain")
                .arg(arg!(<FROM>"'the address to send the transaction from'").required(true))
                .arg(arg!(<TO>"'the address to send the transaction to'").required(true))
                .arg(arg!(<AMOUNT>"'the amount of the transaction'").required(true)))
            .get_matches();

        if let Some(_) = matches.subcommand_matches("printchain") {
            self.cmd_print_chain()?;
        }
        if let Some(matches) = matches.subcommand_matches("create") {
            if let Some(addr) = matches.get_one::<String>("ADDRESS") {
                let addr = String::from(addr);
                Blockchain::create_blockchain(addr.clone())?;
                println!("Created blockchain at {}", addr);
            }
        }
        if let Some(matches) = matches.subcommand_matches("transfer") {
            let from = matches.get_one::<String>("FROM").unwrap();
            let to = matches.get_one::<String>("TO").unwrap();
            let amount = matches.get_one::<String>("AMOUNT").unwrap();
            let amount = amount.parse::<i32>()?;
            let mut bc = Blockchain::new()?;
            let tx = Transaction::new_utxo(from, amount, to, &bc)?;
            bc.add_block(vec![tx])?;
            println!("Transferred amount {} from {} to {}", amount, *from, *to);
        }
        if let Some(matches) = matches.subcommand_matches("getbalance") {
            if let Some(addr) = matches.get_one::<String>("ADDRESS") {
                let addr = String::from(addr);
                let bc = Blockchain::new()?;
                let utxos = bc.find_utxo(&addr);
                let balance:i32 = utxos.iter()
                    .map(|item| item.get_value())
                    .sum();
                println!("UTXO {} balance is {}", addr, balance);
            }
        }
        Ok(())
    }

    fn cmd_print_chain(&self) -> Result<()> {
        let blockchain = Blockchain::new()?;
        for block in blockchain.iter() {
            println!("{:#?}", block);
        }
        Ok(())
    }
}