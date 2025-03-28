# Vee

Verifiable Extraction for Blockchain.

## Examples

### Inclusion proof

```rust,no_run
use std::{fs::File, io::BufReader};
use vee::{
    generate_inclusion_proofs, read_blocks_from_reader, verify_inclusion_proofs, Compression,
    Epoch, EraValidateError, Header,
};

fn main() -> Result<(), EraValidateError> {
    let mut headers: Vec<Header> = Vec::new();

    for flat_file_number in (0..=8200).step_by(100) {
        let file = format!(
            "your_files/ethereum_firehose_first_8200/{:010}.dbin",
            flat_file_number
        );
        match read_blocks_from_reader(
            BufReader::new(File::open(&file).unwrap()),
            Compression::None,
        ) {
            Ok(blocks) => {
                headers.extend(
                    blocks
                        .iter()
                        .map(|block| Header::try_from(block).unwrap())
                        .collect::<Vec<Header>>(),
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
    let headers_to_prove = headers[start_block..end_block].to_vec();
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
```

### Era validator

```rust,no_run
use std::{fs::File, io::BufReader};
use tree_hash::Hash256;
use vee::{
    read_blocks_from_reader, Compression, Epoch, EraValidateError, EraValidator,
    Header,
};

fn create_test_reader(path: &str) -> BufReader<File> {
     BufReader::new(File::open(path).unwrap())
}

fn main() -> Result<(), EraValidateError> {
     let mut headers: Vec<Header> = Vec::new();

     for number in (0..=8200).step_by(100) {
         let file_name = format!(
             "your-test-assets/ethereum_firehose_first_8200/{:010}.dbin",
             number
         );
         let reader = create_test_reader(&file_name);
         let blocks = read_blocks_from_reader(reader, Compression::None).unwrap();
         let successful_headers = blocks
             .iter()
             .cloned()
             .map(|block| Header::try_from(&block))
             .collect::<Result<Vec<_>, _>>()?;
         headers.extend(successful_headers);
     }

     assert_eq!(headers.len(), 8300);
     assert_eq!(headers[0].number, 0);

     let era_verifier = EraValidator::default();
     let epoch: Epoch = headers.try_into().unwrap();
     let result = era_verifier.validate_era(&epoch)?;
     let expected = Hash256::new([
         94, 193, 255, 184, 195, 177, 70, 244, 38, 6, 199, 76, 237, 151, 61, 193, 110, 197, 161, 7,
         192, 52, 88, 88, 195, 67, 252, 148, 120, 11, 66, 24,
     ]);
     assert_eq!(result, expected);

     println!("Era validated successfully!");

     Ok(())
}
```
