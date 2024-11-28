# file structure

- block.rs: block implementation
- blockchain.rs: manage a linkedlist of blocks
- cli.rs: command line testing tool
- errors.rs: custom Result type for error handling
- transaction.rs: implement `trasnfer` logic
- tx.rs: block internal structure
- wallet.rs: implement wallet(similar to account)

# how to run

1. create two wallets in separate terminal: `cargo run createwallet`, take notes of the addresses, for example addr1 and addr2
2. create blockchain: `cargo run create addr1`, now there is 100 coins at addr1
3. check the balance of both wallets: `cargo run getbalance addr1`, you should see 100 and 0 separately
4. transfer 20 from addr1 to addr2: `cargo run transfer addr1 addr2 20`, check out the balance, you should see 80 and 20 separately

# v2: add UTXOSet (similar to BitCoin's ChainState)

why?

1. Performance Optimization: instead of scanning the entire blockchain to find unspent outputs, the UTXOSet maintains a cache of only unspent transactions, making balance checks and transaction creation much faster.
2. Balance Tracking: make it efficient to calculate an address's balance
3. Transaction Validation: help prevent double-spending by maintaining a current state of what outputs can be spent.