use std::{fs::File, io::BufReader};

use firehose_protos::EthBlock as Block;
use flat_files_decoder::{read_blocks_from_reader, Compression};
use header_accumulator::{
    generate_inclusion_proof, verify_inclusion_proof, EraValidateError, ExtHeaderRecord,
};

fn create_test_reader(path: &str) -> BufReader<File> {
    BufReader::new(File::open(path).unwrap())
}

fn main() -> Result<(), EraValidateError> {
    let mut headers: Vec<ExtHeaderRecord> = Vec::new();
    let mut all_blocks: Vec<Block> = Vec::new();

    for flat_file_number in (0..=8200).step_by(100) {
        let file = format!(
            "your-test-assets/ethereum_firehose_first_8200/{:010}.dbin",
            flat_file_number
        );
        match read_blocks_from_reader(create_test_reader(&file), Compression::None) {
            Ok(blocks) => {
                headers.extend(
                    blocks
                        .iter()
                        .map(|block| ExtHeaderRecord::try_from(block).unwrap())
                        .collect::<Vec<ExtHeaderRecord>>(),
                );
                all_blocks.extend(blocks);
            }
            Err(e) => {
                eprintln!("error: {:?}", e);
                break;
            }
        }
    }

    let start_block = 301;
    let end_block = 402;
    let inclusion_proof =
        generate_inclusion_proof(headers, start_block, end_block).unwrap_or_else(|e| {
            println!("Error occurred: {}", e);
            std::process::exit(1);
        });
    assert_eq!(
        inclusion_proof.len() as usize,
        (end_block - start_block + 1) as usize
    );

    // Verify inclusion proof
    let proof_blocks: Vec<Block> = all_blocks[start_block as usize..=end_block as usize].to_vec();
    assert!(verify_inclusion_proof(proof_blocks, None, inclusion_proof.clone()).is_ok());

    // Verify if inclusion proof fails on not proven blocks
    let proof_blocks: Vec<Block> = all_blocks[302..=403].to_vec();
    assert!(verify_inclusion_proof(proof_blocks, None, inclusion_proof.clone()).is_err());

    println!("Inclusion proof verified successfully!");

    Ok(())
}
