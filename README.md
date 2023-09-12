# Spore Drop

A tiny agent shows how to program an Airdrop agent using CKB's Rust sdk.

It will randomly pick a image fomr [res/](./res/) to make it a Spore and send to the receiver.

## Procedure

Just like other ckb development ,the procedure of this program can be described as:
### 1. Searching.
It searches for the on_chain cells, using CkbRpcClient.
### 2. Filtering
Filter cells you found with rules specified(check [rules.rs](src/rules.rs))
### 3. Processing
Process data contains in cell.
### 4. Build Spore Data
Build the spore data like in [utils.rs](src/utils.rs)
### 5. Build Transaction
Build a transaction and send it.
