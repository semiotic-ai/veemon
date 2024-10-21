//! Module for Firehose Ethereum-related data structures and operations.
//! Currently contains the `.proto` defined [here](https://github.com/streamingfast/firehose-ethereum/blob/d9ec696423c2288db640f00026ae29a6cc4c2121/proto/sf/ethereum/type/v2/type.proto#L9)    

use alloy_primitives::{Address, Bloom, FixedBytes, Uint};
use ethportal_api::types::execution::header::Header;
use prost::Message;
use reth_primitives::TxType;
use transaction_trace::Type;

use crate::{
    error::ProtosError,
    firehose::v2::{Response, SingleBlockResponse},
};

tonic::include_proto!("sf.ethereum.r#type.v2");

impl TryFrom<&Block> for Header {
    type Error = ProtosError;

    fn try_from(block: &Block) -> Result<Self, Self::Error> {
        let block_header = block
            .header
            .as_ref()
            .ok_or(ProtosError::BlockConversionError)?;

        let parent_hash = FixedBytes::from_slice(block_header.parent_hash.as_slice());
        let uncles_hash = FixedBytes::from_slice(block_header.uncle_hash.as_slice());
        let author = Address::from_slice(block_header.coinbase.as_slice());
        let state_root = FixedBytes::from_slice(block_header.state_root.as_slice());
        let transactions_root = FixedBytes::from_slice(block_header.transactions_root.as_slice());
        let receipts_root = FixedBytes::from_slice(block_header.receipt_root.as_slice());
        let logs_bloom = Bloom::from_slice(block_header.logs_bloom.as_slice());
        let difficulty = Uint::from_be_slice(
            block_header
                .difficulty
                .as_ref()
                .ok_or(ProtosError::BlockConversionError)?
                .bytes
                .as_slice(),
        );
        let number = block_header.number;
        let gas_limit = Uint::from(block_header.gas_limit);
        let gas_used = Uint::from(block_header.gas_used);
        let timestamp = block_header
            .timestamp
            .as_ref()
            .ok_or(ProtosError::BlockConversionError)?
            .seconds as u64;
        let extra_data = block_header.extra_data.clone();
        let mix_hash = Some(FixedBytes::from_slice(block_header.mix_hash.as_slice()));
        let nonce = Some(FixedBytes::from_slice(&block_header.nonce.to_be_bytes()));
        let base_fee_per_gas = block_header
            .base_fee_per_gas
            .as_ref()
            .map(|base_fee_per_gas| Uint::from_be_slice(base_fee_per_gas.bytes.as_slice()));
        let withdrawals_root = match block_header.withdrawals_root.is_empty() {
            true => None,
            false => Some(FixedBytes::from_slice(
                block_header.withdrawals_root.as_slice(),
            )),
        };
        let blob_gas_used = block_header.blob_gas_used.map(Uint::from);
        let excess_blob_gas = block_header.excess_blob_gas.map(Uint::from);
        let parent_beacon_block_root = match block_header.parent_beacon_root.is_empty() {
            true => None,
            false => Some(FixedBytes::from_slice(
                block_header.parent_beacon_root.as_slice(),
            )),
        };

        Ok(Header {
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
            blob_gas_used,
            excess_blob_gas,
            parent_beacon_block_root,
        })
    }
}

impl From<Type> for TxType {
    fn from(tx_type: Type) -> Self {
        use TxType::*;
        use Type::*;

        match tx_type {
            TrxTypeLegacy => Legacy,
            TrxTypeAccessList => Eip2930,
            TrxTypeDynamicFee => Eip1559,
            TrxTypeBlob => Eip4844,
            TrxTypeArbitrumDeposit => unimplemented!(),
            TrxTypeArbitrumUnsigned => unimplemented!(),
            TrxTypeArbitrumContract => unimplemented!(),
            TrxTypeArbitrumRetry => unimplemented!(),
            TrxTypeArbitrumSubmitRetryable => unimplemented!(),
            TrxTypeArbitrumInternal => unimplemented!(),
            TrxTypeArbitrumLegacy => unimplemented!(),
            TrxTypeOptimismDeposit => unimplemented!(),
        }
    }
}

impl TryFrom<SingleBlockResponse> for Block {
    type Error = ProtosError;

    fn try_from(response: SingleBlockResponse) -> Result<Self, Self::Error> {
        let any = response.block.ok_or(ProtosError::NullBlock)?;
        let block = Block::decode(any.value.as_ref())?;
        Ok(block)
    }
}

impl TryFrom<Response> for Block {
    type Error = ProtosError;

    fn try_from(response: Response) -> Result<Self, Self::Error> {
        let any = response.block.ok_or(ProtosError::NullBlock)?;
        let block = Block::decode(any.value.as_ref())?;
        Ok(block)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_to_header() {
        let reader = std::fs::File::open("tests/data/block-20562650-header.json").unwrap();
        let block: Block = serde_json::from_reader(reader).unwrap();

        // Confirm block number and hash.
        assert_eq!(&block.number, &20562650);
        assert_eq!(
            format!("0x{}", hex::encode(&block.hash)).as_str(),
            "0xf218f8b4f7879b1c4a44b658a32d4a338db85c85c2916229d8b1c7728b448382"
        );

        let header = Header::try_from(&block).unwrap();

        // Calculate the block hash from the header.
        // `hash()` calls `keccak256(alloy_rlp::encode(self))`.
        let block_hash = header.hash();

        assert_eq!(
            block_hash.to_string().as_str(),
            "0xf218f8b4f7879b1c4a44b658a32d4a338db85c85c2916229d8b1c7728b448382"
        );
    }
}
