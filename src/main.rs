use ckb_types::packed::{CellOutput, CellOutputBuilder, OutPoint, Script, Uint64};
use ckb_sdk::Address;
use ckb_sdk::{CkbRpcClient, NetworkInfo};
use ckb_sdk::transaction::builder::{CkbTransactionBuilder, SimpleTransactionBuilder};
use ckb_types::{H256, h256};
use std::{error::Error as StdErr, str::FromStr};
use std::thread::sleep;
use std::time::Duration;
use ckb_jsonrpc_types::{Status, TxStatus};
use ckb_sdk::rpc::ckb_indexer::{CellType, ScriptType, SearchKey, Tx};
use ckb_sdk::rpc::ckb_indexer::Order::Asc;
use ckb_sdk::rpc::ckb_indexer::ScriptSearchMode::Exact;
use ckb_sdk::transaction::input::InputIterator;
use ckb_sdk::transaction::signer::{SignContexts, TransactionSigner};
use ckb_sdk::transaction::TransactionBuilderConfiguration;
use ckb_types::prelude::*;
use ckb_sdk::tx_builder::*;
use crate::rules::validate_tx;
use rand::seq::SliceRandom;
use std::fs;
use crate::utils::{build_output_and_data, build_spore_data};

mod rules;
mod utils;


fn main() {
    let mut client = CkbRpcClient::new("http://127.0.0.1:8114");

    // the target addr we searched
    let AIRDROP_ADDR = Address::from_str("ckt1qzda0cr08m85hc8jlnfp3zer7xulejywt49kt2rr0vthywaa50xwsqw75xssu4vt3032jjeh03zp4h0vvwl47aghggnp9").unwrap();

    let network_info = NetworkInfo::testnet();
    let sender = Address::from_str("ckt1qzda0cr08m85hc8jlnfp3zer7xulejywt49kt2rr0vthywaa50xwsqth6sasjn677g5p30yggt0yyh0whq4ducqk3wmsv").unwrap();
    let script: Script = (&AIRDROP_ADDR).into();

    let key = std::env::var("CKB_PRIVATE_KEY").expect(
        "Failed to get private key! please check if you have set env CKB_PRIVATE_KEY!"
    );

    let search_key = SearchKey {
        script: script.into(),
        script_type: ScriptType::Lock,
        script_search_mode: Some(Exact),
        filter: None,
        with_data: None,
        group_by_transaction: Some(true),
    };

    let mut sent_related_tx = Vec::new();

    let entries = fs::read_dir("./res")
        .expect("Failed to read directory")
        .map(|res| res.expect("Failed to read directory entry"))
        .collect::<Vec<_>>();


    loop {
        let mut page = client.get_transactions(search_key.clone(), Asc, 10.into(), None).expect("Failed to get transaction!");
        while !page.objects.is_empty() {
            println!("{} transaction founded!", page.objects.len());
            for tx in page.objects.iter() {
                match tx {
                    Tx::Ungrouped(_) => {
                        unreachable!()
                    }
                    Tx::Grouped(tx) => {
                        let tx_hash = tx.tx_hash.clone();
                        if sent_related_tx.contains(&tx_hash) {
                            println!("{} Already sent! skip", tx_hash);
                            continue
                        }
                        if let Ok(tx_status) = client.get_transaction_status(tx.clone().tx_hash) {
                            if tx_status.tx_status.status != Status::Committed {
                                continue
                            }
                        } else {
                            continue
                        }

                        println!("Detecting tx {}", tx_hash);

                        let addresses = validate_tx(tx, &mut client);
                        if addresses.is_empty() {
                            println!("No matching cell! skiping...");
                        }

                        for receiver in addresses.iter() {
                            let configuration = TransactionBuilderConfiguration::new_with_network(network_info.clone()).expect("Error Config");

                            let iter = InputIterator::new_with_address(&[sender.clone()], &network_info);
                            let mut builder = SimpleTransactionBuilder::new(configuration, iter);
                            // Randomly select a file
                            let selected_entry = entries
                                .choose(&mut rand::thread_rng())
                                .expect("No files found in directory!");
                            let file_path = selected_entry.path();
                            let buffer = fs::read(file_path)
                                .expect("Failed to read file");

                            let (cellout, data) = build_output_and_data(&sender, buffer);

                            builder.add_output_and_data(cellout, data);


                            let mut tx_with_groups = builder.build(&Default::default()).expect("Failed to build tx");

                            let json_tx = ckb_jsonrpc_types::TransactionView::from(tx_with_groups.get_tx_view().clone());
                            let private_keys = vec![H256::from_str(&key).unwrap()];
                            TransactionSigner::new(&NetworkInfo::testnet()).sign_transaction(
                                &mut tx_with_groups,
                                &SignContexts::new_sighash_h256(private_keys).expect("Failed to sign, please check if you set env_var CKB_PRIVATE_KEY"),
                            ).expect("failed to sign transaction");
                            let tx_hash = client
                                .send_transaction(json_tx.inner, None)
                                .expect("send transaction");
                            println!(">>> tx {} sent! <<<", tx_hash);
                            sent_related_tx.push(tx_hash.clone());
                        }
                    }
                }
            }

            page = client.get_transactions(search_key.clone(), Asc, 10.into(), Some(page.last_cursor)).expect("Failed to get transaction!");
        }

        println!("next update will trigger in 5 seconds...");
        sleep(Duration::from_secs(5));
    }
}
