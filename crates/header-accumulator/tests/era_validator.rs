use decoder::decode_flat_files;
use header_accumulator::{
    era_validator::era_validate, errors::EraValidateError, types::ExtHeaderRecord,
    utils::ext_header_from_block,
};

#[test]
fn test_era_validate() -> Result<(), EraValidateError> {
    let mut headers: Vec<ExtHeaderRecord> = Vec::new();
    for number in (0..=8200).step_by(100) {
        let file_name = format!("tests/ethereum_firehose_first_8200/{:010}.dbin", number);
        match decode_flat_files(file_name, None, None) {
            Ok(blocks) => headers.extend(
                blocks
                    .iter()
                    .map(|block| ext_header_from_block(block).unwrap())
                    .collect::<Vec<ExtHeaderRecord>>(),
            ),
            Err(e) => {
                eprintln!("error: {:?}", e);
                break;
            }
        }
    }
    assert_eq!(headers.len(), 8300);
    assert_eq!(headers[0].block_number, 0);

    let result = era_validate(headers, None, 0, None, Some(false))?;

    assert!(result.contains(&0), "The vector does not contain 0");
    Ok(())
}
