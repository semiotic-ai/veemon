use std::{fs::File, io::BufReader};

use flat_files_decoder::{read_blocks_from_reader, Compression};
use header_accumulator::{
    generate_inclusion_proofs, verify_inclusion_proofs, Epoch, EraValidateError, ExtHeaderRecord,
};

fn create_test_reader(path: &str) -> BufReader<File> {
    BufReader::new(File::open(path).unwrap())
}

fn main() -> Result<(), EraValidateError> {
    let mut headers: Vec<ExtHeaderRecord> = Vec::new();

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
            }
            Err(e) => {
                eprintln!("error: {:?}", e);
                break;
            }
        }
    }

    let start_block = 301;
    let end_block = 402;
    let headers_to_prove: Vec<_> = headers[start_block..end_block]
        .iter()
        .map(|ext| ext.full_header.as_ref().unwrap().clone())
        .collect();
    let epoch: Epoch = headers.try_into().unwrap();

    let inclusion_proof = generate_inclusion_proofs(vec![epoch], headers_to_prove.clone())
        .unwrap_or_else(|e| {
            println!("Error occurred: {}", e);
            std::process::exit(1);
        });
    assert_eq!(inclusion_proof.len(), headers_to_prove.len());

    let proof_headers = headers_to_prove
        .into_iter()
        .zip(inclusion_proof)
        .map(|(header, proof)| proof.with_header(header))
        .collect::<Result<Vec<_>, _>>()?;

    // Verify inclusion proof
    assert!(verify_inclusion_proofs(None, proof_headers).is_ok());

    println!("Inclusion proof verified successfully!");

    Ok(())
}
