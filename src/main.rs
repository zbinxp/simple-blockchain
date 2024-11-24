
mod block;
mod errors;
mod blockchain;
mod cli;
mod transaction;
mod tx;
mod wallet;

use crate::errors::Result;

fn main() -> Result<()>{
    let mut cli = cli::Cli::new()?;
    cli.run()?;

    Ok(())
}
