use ckb_sdk::{Address, ScriptId};
use ckb_types::core::Capacity;
use ckb_types::{H256, h256};
use ckb_types::packed::{Bytes, CellOutput};
use ckb_types::prelude::*;
use molecule::prelude::*;
use spore_types::{NativeNFTData, SporeData};



pub const SPORE_CODE_HASH: H256 = h256!("0xbbad126377d45f90a8ee120da988a2d7332c78ba8fd679aab478a19d6c133494");
pub const CLUSTER_CODE_HASH: H256 = h256!("0x598d793defef36e2eeba54a9b45130e4ca92822e1d193671f490950c3b856080");

pub fn build_spore_data(data: Vec<u8>) -> SporeData {
    SporeData::from(NativeNFTData {
        content_type: "image/png".to_string(),
        content: data,
        cluster_id: None,
    })
}


pub fn build_output_and_data(sender: &Address, data: Vec<u8>) -> (CellOutput, Bytes) {
    let spore_data = build_spore_data(data);
    let spore_data_vec = spore_data.as_slice().to_vec();
    let data_capacity = Capacity::bytes(spore_data_vec.len()).unwrap();

    let type_script =
        ScriptId::new_type(SPORE_CODE_HASH.clone()).dummy_type_id_script();
    let dummy_output = CellOutput::new_builder()
        .lock(sender.into())
        .type_(Some(type_script).pack())
        .build();
    let required_capacity = dummy_output
        .occupied_capacity(data_capacity)
        .unwrap()
        .pack();
    let output = dummy_output
        .as_builder()
        .capacity(required_capacity)
        .build();
    (output, spore_data_vec.pack())
}