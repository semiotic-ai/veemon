use decoder::decode_flat_files;
use header_accumulator::{
    self,
    errors::EraValidateError,
    inclusion_proof::{generate_inclusion_proof, verify_inclusion_proof},
    types::ExtHeaderRecord,
};
use sf_protos::ethereum::r#type::v2::Block;

#[test]
fn test_inclusion_proof() -> Result<(), EraValidateError> {
    let mut headers: Vec<ExtHeaderRecord> = Vec::new();
    let mut all_blocks: Vec<Block> = Vec::new(); // Vector to hold all blocks

    for flat_file_number in (0..=8200).step_by(100) {
        let file_name = format!(
            "tests/ethereum_firehose_first_8200/{:010}.dbin",
            flat_file_number
        );
        match decode_flat_files(file_name, None, None, Some(false)) {
            Ok(blocks) => {
                headers.extend(
                    blocks
                        .iter()
                        .map(|block| ExtHeaderRecord::try_from(block).unwrap())
                        .collect::<Vec<ExtHeaderRecord>>(),
                );
                all_blocks.extend(blocks); // Extend the all_blocks vector with the decoded blocks
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
            // Handle the error, e.g., by exiting the program or returning a default value
            std::process::exit(1); // Exiting the program, for example
        });
    assert_eq!(
        inclusion_proof.len() as usize,
        (end_block - start_block + 1) as usize
    );

    // Verify inclusion proof
    let proof_blocks: Vec<Block> = all_blocks[start_block as usize..=end_block as usize].to_vec();
    assert!(verify_inclusion_proof(proof_blocks, None, inclusion_proof.clone()).is_ok());

    // verify if inclusion proof fails on not proven blocks
    let proof_blocks: Vec<Block> = all_blocks[302..=403].to_vec();
    assert!(verify_inclusion_proof(proof_blocks, None, inclusion_proof.clone()).is_err());

    Ok(())
}
