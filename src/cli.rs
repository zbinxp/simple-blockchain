use bitcoincash_addr::Address;
use crate::blockchain::Blockchain;
use crate::errors::Result;
use clap::{arg, Command};
use crate::transaction::Transaction;
use crate::utxoset::UTXOSet;
use crate::wallet::WalletManager;

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
            .subcommand(Command::new("createwallet").about("create wallet"))
            .subcommand(Command::new("listaddresses").about("list addresses of the wallet"))
            .subcommand(Command::new("reindex").about("reindex wallet"))
            .subcommand(Command::new("printutxo").about("printutxo transactions"))
            .get_matches();

        if let Some(_) = matches.subcommand_matches("printchain") {
            self.cmd_print_chain()?;
        }
        if let Some(matches) = matches.subcommand_matches("create") {
            if let Some(addr) = matches.get_one::<String>("ADDRESS") {
                let addr = String::from(addr);
                let bc = Blockchain::create_blockchain(addr.clone())?;
                let utxo_set = UTXOSet{blockchain:bc};
                utxo_set.reindex()?;
                println!("Created blockchain at {}", addr);
            }
        }
        if let Some(matches) = matches.subcommand_matches("transfer") {
            let from = matches.get_one::<String>("FROM").unwrap();
            let to = matches.get_one::<String>("TO").unwrap();
            let amount = matches.get_one::<String>("AMOUNT").unwrap();
            let amount = amount.parse::<i32>()?;
            let bc = Blockchain::new()?;
            let mut utxo_set = UTXOSet {blockchain: bc};
            let wm = WalletManager::new()?;
            let wallet_from = wm.get_wallet(from.as_str()).unwrap();
            let wallet_to = wm.get_wallet(to.as_str()).unwrap();
            let tx = Transaction::new_utxo(wallet_from, amount, wallet_to, &utxo_set)?;
            // let cbtx = Transaction::new_coinbase(String::from(to), String::new())?;
            let new_block = utxo_set.blockchain.add_block(vec![tx])?;
            utxo_set.update(&new_block)?;
            println!("Transferred amount {} from {} to {}", amount, *from, *to);
        }
        if let Some(matches) = matches.subcommand_matches("getbalance") {
            if let Some(addr) = matches.get_one::<String>("ADDRESS") {
                let pub_key_hash = Address::decode(addr).unwrap().body;
                let bc = Blockchain::new()?;
                // let utxos = bc.find_utxo(&pub_key_hash);
                let utxo_set = UTXOSet{blockchain: bc};
                let utxos = utxo_set.find_utxo(&pub_key_hash)?;
                let balance:i32 = utxos.iter()
                    .map(|item| item.get_value())
                    .sum();
                println!("UTXO {} balance is {}", addr, balance);
            }
        }

        if let Some(_) = matches.subcommand_matches("createwallet") {
            let mut wm = WalletManager::new()?;
            let address = wm.new_wallet();
            wm.save_all()?;
            println!("Created wallet at {}", address);
        }
        if let Some(_) = matches.subcommand_matches("listaddresses") {
            let wm = WalletManager::new()?;
            let addresses = wm.get_all_addresses();
            println!("Addresses {:?}", addresses);
        }
        if let Some(_) = matches.subcommand_matches("reindex") {
            let bc = Blockchain::new()?;
            let utxo_set = UTXOSet{blockchain: bc};
            utxo_set.reindex()?;
            let count = utxo_set.count_transactions()?;
            println!("After reindex, there are {} transactions", count);
        }
        if let Some(_) = matches.subcommand_matches("printutxo") {
            let bc = Blockchain::new()?;
            let utxo_set = UTXOSet{blockchain: bc};
            println!("{:?}", utxo_set);
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