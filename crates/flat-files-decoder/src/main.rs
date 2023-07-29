mod protos;
mod dbin;


use std::any::Any;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use protobuf::Message;
use crate::dbin::DbinFile;
use crate::protos::block::{Block, TransactionTraceStatus};

use reth_blockchain_tree::post_state::PostState;
use reth_primitives::{Address, Bytes, H256, Log, Receipt, TxType};
use crate::protos::block::transaction_trace::Type;

fn handle_block(message: Vec<u8>) -> anyhow::Result<Block> {
    let message: protos::bstream::Block = Message::parse_from_bytes(&message)?;

    let block: Block = Message::parse_from_bytes(&message.payload_buffer)?;
    let mut post_state = PostState::new();


    // TODO: Extract to separate functions / modules
    for trace in &block.transaction_traces {
        let success = trace.status.enum_value()? == TransactionTraceStatus::SUCCEEDED;

        // TODO: check -> nothing maps to TxType::EIP4844
        let tx_type = match trace.type_.enum_value()? {
            Type::TRX_TYPE_LEGACY => TxType::Legacy,
            Type::TRX_TYPE_ACCESS_LIST => TxType::EIP2930,
            Type::TRX_TYPE_DYNAMIC_FEE => TxType::EIP1559 // TODO: check correctness
        };

        let logs = trace.receipt.logs.iter().map(|log| {
            let slice: [u8;20] = <[u8; 20]>::try_from(log.address.as_slice())?; // TODO: Better way ?
            let address = Address::from(slice);
            let topics = log.topics.iter().map(|topic| {
                let slice: [u8;32] = <[u8; 32]>::try_from(topic.as_slice())?; // TODO: Better way?
                H256::from(slice)
            }).collect();
            Log {
                address,
                topics,
                data: Bytes::from(log.data.as_slice())
            }
        }).collect();

        let receipt = Receipt {
            tx_type,
            success,
            cumulative_gas_used: trace.receipt.cumulative_gas_used,
            logs,
        };

        post_state.add_receipt(block.number, receipt);
    }

    let computed_root = post_state.receipts_root(block.number);

    if computed_root.as_bytes() != block.header.receipt_root.as_slice() {
        panic!("Root mismatch: {} != {:?}", computed_root, block.header.receipt_root.as_slice());
    }

    let file_name = format!("out/block-{}.json", block.number);
    let mut out_file = File::create(file_name)?;

    let block_json = protobuf_json_mapping::print_to_string(&block)?;

    out_file.write_all(block_json.as_bytes())?;

    Ok(block)
}

fn handle_file(path: PathBuf) -> anyhow::Result<Vec<Block>> {
    let input_file = File::open(path)
        .expect("Failed to open file");

    let dbin_file = DbinFile::from_file(input_file)
        .expect("Invalid dbin file");

    if dbin_file.content_type != "ETH" {
        panic!("Invalid content type: {}", dbin_file.content_type);
    }

    let mut blocks: Vec<Block> = vec![];

    for message in dbin_file.messages {
        blocks.push(handle_block(message)?);
    }

    Ok(blocks)
}

fn main() {
    let paths = fs::read_dir("input_files")
        .expect("Failed to read input_files directory");

    let mut blocks: Vec<Block> = vec![];
    for path in paths {
        let path = path.expect("Failed to read path");
        match path.path().extension() {
            Some(ext) => {
                if ext != "dbin" {
                    continue;
                }
            },
            None => continue
        };

        println!("Processing file: {}", path.path().display());
        match handle_file(path.path()) {
            Ok(file_blocks) => {
                blocks.extend(file_blocks);
            },
            Err(err) => {
                println!("Failed to process file: {}", err);
            }
        }
    }

    println!("Total blocks: {}", blocks.len());

}


