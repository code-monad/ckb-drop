// This file contains how we measure a contract is a airdrop request

use std::error;
use std::str::FromStr;
use ckb_jsonrpc_types::{CellWithStatus, Status};
use ckb_sdk::{Address, AddressPayload, CkbRpcClient, NetworkType, RpcError};
use ckb_sdk::rpc::ckb_indexer::TxWithCells;
use ckb_types::core::error::TransactionError;
use ckb_types::H160;
use ckb_types::packed::{OutPoint, OutPointBuilder};
use ckb_types::prelude::*;
use molecule::error::Error;

pub fn validate_capacity(cell: &CellWithStatus) -> bool {
    match &cell.cell {
        None => { return false }
        Some(cell) => {
            return cell.output.capacity >= 10.into()
        }
    }
}

pub fn validate_cell_data(cell: &CellWithStatus) -> Option<Address> {
    if let Some(cell) = &cell.cell {
        if let Some(data) = &cell.data {
            if let Ok(address) = std::str::from_utf8(data.content.as_bytes()) {
                if let Ok(addr) = Address::from_str(address) {
                    return Some(addr);
                }
            }
        }
    }
    None
}


pub fn validate_tx(tx: &TxWithCells, client: &mut CkbRpcClient) -> Vec<Address> {
    let mut target_address = Vec::new();
    for (cell_type, index) in &tx.cells {
        let out = OutPoint::new_builder().index(index.value().pack()).tx_hash(tx.tx_hash.pack()).build();
        if let Ok(cell) = client.get_live_cell(
            out.into(),
            true) {
            if cell.status == "live" {
                if !validate_capacity(&cell) {
                    continue
                }
                // filter cell that does not contains a valid addr data
                if let Some(addr) =  validate_cell_data(&cell) {
                    target_address.push(addr);
                }
            }
        }
    }
    target_address
}