use ssz_rs::{Sized, Deserialize, DeserializeError, U256, Merkleized, Vector, Serialize, Node};
use ssz_rs_derive::SimpleSerialize;


pub const MAX_EPOCH_SIZE: usize = 8192;

#[derive(Default, Debug, Clone, PartialEq, SimpleSerialize)]
pub struct HeaderRecord {
    pub hash: U256,
    pub td: U256,
}


pub fn compute_accumulator(hashes: Vec<U256>, tds: Vec<U256>) -> Node {
    if hashes.len() != tds.len() {
        panic!("hashes and tds must be the same length");
    }

    if hashes.len() > MAX_EPOCH_SIZE {
        panic!("hashes and tds must be less than MAX_EPOCH_SIZE");
    }

    let mut epoch_accumulator_bytes = Vec::new();
    for (hash, td) in hashes.iter().zip(tds.iter()) {
        let mut header_record = HeaderRecord {
            hash: hash.clone(),
            td: td.clone(),
        };
        let mut leaf_bytes = Vec::new();
        let _ = header_record.hash_tree_root().unwrap().serialize(&mut leaf_bytes);
        epoch_accumulator_bytes.push(leaf_bytes);
    }
    let mut epoch_accumulator: Vector<u8, 32> = epoch_accumulator_bytes.into_iter().flatten().collect::<Vec<u8>>().try_into().unwrap();
    epoch_accumulator.hash_tree_root().unwrap()
}


