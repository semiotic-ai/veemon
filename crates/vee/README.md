# Vee

Verifiable Extraction for Blockchain data.

This crate exposes the interfaces from other crates in veemon.

### Verifiabilty for Ethereum
![Full path of proofs diagram](./assets/diagram.svg)

Events recorded in transaction receipt logs are useful for analysis of blockchain data. Ensuring that the receipts recorded in extracted blocks accurately reflect the chain's history is critical for developers. veemon provides the capability to generate inclusion proofs for receipts against the `receipts_root` in the block header as well as to reconstruct the `receipts_root` for validation.

veemon also provides implementations for verification of the block itself against the history of the blockchain. Verifying inclusion of a block in the Ethereum history relies on [Ethereum Portal Network Accumulators](https://github.com/ethereum/portal-accumulators)
which use Header Accumulators built on historical block hash information for pre-Merge, post-Merge & pre-Capella Ethereum blocks. Post-Capella Accumulators rely on the 
`HistoricalSummary` which is extracted from the `HeadState` of the beacon chain, and the associated beacon blocks as well. 


## Examples

### Inclusion proof

inclusion proofs are for verifying specific blocks to be part of canonical epochs. 

```rust,no_run
use std::{fs::File, io::BufReader};
use vee::{
    generate_inclusion_proofs, read_blocks_from_reader, verify_inclusion_proofs,
    AnyBlock, Compression, Epoch, ExtHeaderRecord,
};
use vee::authentication::AuthenticationError;

fn main() -> Result<(), AuthenticationError> {
   let mut headers: Vec<ExtHeaderRecord> = Vec::new();

    for flat_file_number in (0..=8200).step_by(100) {
        let file = format!(
            "your_files/ethereum_firehose_first_8200/{:010}.dbin",
            flat_file_number
        );
        let blocks =  read_blocks_from_reader(
            BufReader::new(File::open(&file).unwrap()),
            Compression::None,
        ).unwrap();
        headers.extend(
            blocks
            .iter()
            .filter_map(|block| {
                if let AnyBlock::Evm(eth_block) = block {
                    ExtHeaderRecord::try_from(eth_block).ok()
                } else {
                    None
                }
            })
            .collect::<Vec<ExtHeaderRecord>>(),
        );
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
    // Note: For post-Capella blocks, pass Some(historical_summaries) as the third argument
    assert!(verify_inclusion_proofs(None, proof_headers, None).is_ok());

    println!("Inclusion proof verified successfully!");

    Ok(())
}    
```

### Era validator

Epochs by themselves can be validated to be canonical
with the example below.

```rust,no_run
use std::{fs::File, io::BufReader};
use tree_hash::Hash256;
use vee::{
    read_blocks_from_reader, AnyBlock, Compression, Epoch, ExtHeaderRecord,
};
use vee::authentication::ethereum::EthereumPreMergeValidator;
use vee::authentication::AuthenticationError;

fn create_test_reader(path: &str) -> BufReader<File> {
    BufReader::new(File::open(path).unwrap())
}

fn main() -> Result<(), AuthenticationError> {
    let mut headers: Vec<ExtHeaderRecord> = Vec::new();
    for number in (0..=8200).step_by(100) {
        let file_name = format!(
            "your-test-assets/ethereum_firehose_first_8200/{:010}.dbin",
            number
        );
        let reader = create_test_reader(&file_name);
        let blocks = read_blocks_from_reader(reader, Compression::None).unwrap();
        let successful_headers = blocks
            .iter()
            .filter_map(|block| {
                if let AnyBlock::Evm(eth_block) = block {
                    ExtHeaderRecord::try_from(eth_block).ok()
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        headers.extend(successful_headers);
    }

    assert_eq!(headers.len(), 8300);
    assert_eq!(headers[0].block_number, 0);
    let era_verifier = EthereumPreMergeValidator::default();
    let epoch: Epoch = headers.try_into().unwrap();
    let result = era_verifier.validate_single_epoch(&epoch)?;
    let expected = Hash256::new([
        94, 193, 255, 184, 195, 177, 70, 244, 38, 6, 199, 76, 237, 151, 61, 193, 110, 197, 161, 7,
        192, 52, 88, 88, 195, 67, 252, 148, 120, 11, 66, 24,
    ]);
    assert_eq!(result, expected);

    println!("Era validated successfully!");

    Ok(())
}
```

### Other available examples

Other examples and tests in veemon depend on  [ve-assets](https://github.com/semiotic-ai/ve-assets) to run. Choose each example carefully given the .dbin or .parquet file, because the blocks have to be within the specific block number range for it to work. These usage examples are:

1. Convert parquet data structure into the same `BlockHeader` 
2. Build receipt proof against receipts
3. Build proof that `receipt_root` is correct against `receipt`.
4. Build proof of pre-Merge and pre-Capella block against Header Accumulaors.
5. Build proof of post-Capella blocks given beacon blocks and Ethereum head state

## Testing

Some testing assets were stored in [ve-assets](https://github.com/semiotic-ai/ve-assets). These include parquet file block headers and .dbin files extracted from Firehose. Part of these are necessary for some tests, but they are heavy so most of them are stored in another repo.

