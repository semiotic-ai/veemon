use accumulator::{EpochAccumulator, HeaderRecord};
use alloy::hex::FromHex;
use alloy::primitives::U256;
use substreams::pb::substreams::store_delta::Operation;
use substreams::scalar::BigInt;
use substreams::store::{DeltaString, Deltas, StoreDelete, StoreNew};
mod pb;
use pb::mydata::v1::{Hashes, Header, HeaderAccumulator};

use substreams::store::{
    Appender, StoreAppend, StoreGet, StoreGetString, StoreSet, StoreSetString,
};
use substreams::Hex;
use substreams_ethereum::pb::eth::v2::Block;

#[allow(unused_imports)]
use num_traits::cast::ToPrimitive;
use tree_hash::{Hash256, TreeHash};

mod accumulator;

substreams_ethereum::init!();

#[substreams::handlers::map]
fn map_my_data(blk: Block) -> Header {
    Header {
        block_hash: Hex(&blk.hash).to_string(),
        block_number: blk.number.to_u64().unwrap(),
        total_difficulty: blk.header.unwrap().total_difficulty,
    }
}

const BLOCKS: u64 = 8192;

#[substreams::handlers::store]
pub fn store_block_headers(header: Header, output_append: StoreAppend<String>) {
    let header_record: HeaderRecord = header.clone().into();
    // TODO append bytes instead of strings
    let epoch = header.block_number / BLOCKS;
    output_append.append(
        1,
        epoch.to_string(),
        serde_json::to_string(&header_record).unwrap(),
    );
}

// TODO move the delete logic to store_block_headers
#[substreams::handlers::store]
pub fn store_tick_epoch(
    header: Header,
    epoch_hashes: StoreGetString,
    output_append: StoreSetString,
) {
    let epoch = header.block_number / BLOCKS;
    if epoch > 0 {
        let previous_epoch = epoch - 1;
        output_append.delete_prefix(1, &previous_epoch.to_string());
    }
    output_append.set(
        1,
        epoch.to_string(),
        &epoch_hashes
            .get_at(1, epoch.to_string())
            .unwrap_or_default(),
    );
}

// TODO use bytes
#[substreams::handlers::map]
pub fn map_hashes(hashes_delta: Deltas<DeltaString>) -> Hashes {
    let hashes_delta = hashes_delta.deltas;
    hashes_delta
        .into_iter()
        .find(|e| e.operation == Operation::Delete)
        .map(|hashes| Hashes {
            hashes: hashes
                .old_value
                .split(";")
                .map(ToString::to_string)
                .filter(|f| !f.is_empty())
                .collect(),
        })
        .unwrap_or_default()
}

// TODO use better conversions and not string conversion
impl From<Header> for HeaderRecord {
    fn from(value: Header) -> Self {
        let block_hash = value.block_hash;
        let block_hash = Hash256::from_hex(&block_hash).unwrap();

        let total_difficulty = value.total_difficulty.unwrap();
        let total_difficulty: BigInt = total_difficulty.into();
        let total_difficulty = total_difficulty.to_string();
        let total_difficulty = U256::from_str_radix(&total_difficulty, 10).unwrap();
        Self {
            block_hash,
            total_difficulty,
        }
    }
}

// TODO receive and decode bytes
#[substreams::handlers::map]
pub fn map_accumulator(hashes: Hashes) -> HeaderAccumulator {
    let header_records: Vec<HeaderRecord> = hashes
        .hashes
        .into_iter()
        .map(|hash| serde_json::from_str(&hash).unwrap())
        .collect();
    let epoch_accumulator = EpochAccumulator::from(header_records);

    let root = epoch_accumulator.tree_hash_root();
    HeaderAccumulator {
        root: root.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use alloy::hex::FromHex;
    use tree_hash::Hash256;

    #[test]
    fn convert_header() {
        let hash = "d4e56740f876aef8c010b86a40d5f56745a118d0906a34e69aec8c0db1cb8fa3";
        Hash256::from_hex(hash).unwrap();
    }

    // TODO: compute an header accumulator pre-merge, and check if header accumulator matches with
    // pre-computed frozen header accumulators we have
    // pub const MERGE_BLOCK: usize = 15537394;

    // Compute an accumulator  post merge, and pre-capella, checks if it matches frozen header accumulator
    // pub const CAPELLA_START_EPOCH: usize = 194048;

    // compute an accumulator post-capella (deneb)
    // pub const FIRST_EXECUTION_BLOCK_DENEB: usize = 19426587;

    // pub const FIRST_DENEB_EPOCH: usize = ?;
}
