use ethereum_types::H256 as Hash256;
use ethereum_types::{Bloom, H160, H64, U256 as EthereumU256};
use ethportal_api::types::execution::accumulator::{EpochAccumulator, HeaderRecord};
use ethportal_api::Header;
use sf_protos::ethereum::r#type::v2::{Block, BlockHeader};

use crate::errors::EraValidateError;
use crate::types::ExtHeaderRecord;

pub const MAX_EPOCH_SIZE: usize = 8192;
pub const FINAL_EPOCH: usize = 01896;
pub const MERGE_BLOCK: u64 = 15537394;

pub fn header_record_from_header(
    block_header: &BlockHeader,
) -> Result<Vec<HeaderRecord>, EraValidateError> {
    let mut header_records = Vec::<HeaderRecord>::new();
    let header_record = HeaderRecord {
        block_hash: Hash256::from_slice(block_header.hash.as_slice()),
        total_difficulty: EthereumU256::try_from(
            block_header
                .total_difficulty
                .as_ref()
                .ok_or(EraValidateError::HeaderDecodeError)?
                .bytes
                .as_slice(),
        )
        .map_err(|_| EraValidateError::HeaderDecodeError)?,
    };
    header_records.push(header_record);

    Ok(header_records)
}

pub fn compute_epoch_accumulator(
    header_records: &Vec<HeaderRecord>,
) -> Result<EpochAccumulator, EraValidateError> {
    if header_records.len() > MAX_EPOCH_SIZE {
        Err(EraValidateError::TooManyHeaderRecords)?;
    }

    let mut epoch_accumulator =
        EpochAccumulator::new(Vec::new()).map_err(|_| EraValidateError::EpochAccumulatorError)?;
    for header_record in header_records {
        let _ = epoch_accumulator.push(*header_record);
    }
    Ok(epoch_accumulator)
}

pub fn header_from_block(block: &Block) -> Result<Header, EraValidateError> {
    let block_header = block
        .header
        .as_ref()
        .ok_or(EraValidateError::HeaderDecodeError)?;
    let parent_hash = Hash256::from_slice(block_header.parent_hash.as_slice());
    let uncles_hash = Hash256::from_slice(block_header.uncle_hash.as_slice());
    let author = H160::from_slice(block_header.coinbase.as_slice());
    let state_root = Hash256::from_slice(block_header.state_root.as_slice());
    let transactions_root = Hash256::from_slice(block_header.transactions_root.as_slice());
    let receipts_root = Hash256::from_slice(block_header.receipt_root.as_slice());
    let logs_bloom = Bloom::from_slice(block_header.logs_bloom.as_slice());
    let difficulty = EthereumU256::try_from(
        block_header
            .difficulty
            .as_ref()
            .ok_or(EraValidateError::HeaderDecodeError)?
            .bytes
            .as_slice(),
    )
    .map_err(|_| EraValidateError::HeaderDecodeError)?;
    let number = block_header.number;
    let gas_limit = EthereumU256::try_from(block_header.gas_limit)
        .map_err(|_| EraValidateError::HeaderDecodeError)?;
    let gas_used = EthereumU256::try_from(block_header.gas_used)
        .map_err(|_| EraValidateError::HeaderDecodeError)?;
    let timestamp = block_header
        .timestamp
        .as_ref()
        .ok_or(EraValidateError::HeaderDecodeError)?
        .seconds as u64;
    let extra_data = block_header.extra_data.clone();
    let mix_hash = Some(Hash256::from_slice(block_header.mix_hash.as_slice()));
    let nonce = Some(H64::from_slice(&block_header.nonce.to_be_bytes()));
    let base_fee_per_gas = match block_header.base_fee_per_gas.as_ref() {
        Some(base_fee_per_gas) => Some(EthereumU256::from_big_endian(
            base_fee_per_gas.bytes.as_slice(),
        )),
        None => None,
    };
    let withdrawals_root = match block_header.withdrawals_root.is_empty() {
        true => None,
        false => Some(Hash256::from_slice(
            block_header.withdrawals_root.as_slice(),
        )),
    };

    let header = Header {
        parent_hash,
        uncles_hash,
        author,
        state_root,
        transactions_root,
        receipts_root,
        logs_bloom,
        difficulty,
        number,
        gas_limit,
        gas_used,
        timestamp,
        extra_data,
        mix_hash,
        nonce,
        base_fee_per_gas,
        withdrawals_root,
    };

    Ok(header)
}
