use decoder::{decode_flat_files, Decompression};
use header_accumulator::{
    epoch::Epoch, era_validator::EraValidator, errors::EraValidateError, types::ExtHeaderRecord,
};
use trin_validation::accumulator::PreMergeAccumulator;

#[test]
fn test_era_validate() -> Result<(), EraValidateError> {
    let mut headers: Vec<ExtHeaderRecord> = Vec::new();
    for number in (0..=8200).step_by(100) {
        let file_name = format!("tests/ethereum_firehose_first_8200/{:010}.dbin", number);
        match decode_flat_files(file_name, None, None, Decompression::None) {
            Ok(blocks) => {
                let (successful_headers, _): (Vec<_>, Vec<_>) = blocks
                    .iter()
                    .cloned()
                    .map(|block| ExtHeaderRecord::try_from(&block))
                    .fold((Vec::new(), Vec::new()), |(mut succ, mut errs), res| {
                        match res {
                            Ok(header) => succ.push(header),
                            Err(e) => {
                                // Log the error or handle it as needed
                                eprintln!("Error converting block: {:?}", e);
                                errs.push(e);
                            }
                        };
                        (succ, errs)
                    });

                headers.extend(successful_headers);
            }
            Err(e) => {
                eprintln!("error: {:?}", e);
                break;
            }
        }
    }
    assert_eq!(headers.len(), 8300);
    assert_eq!(headers[0].block_number, 0);

    let premerge_accumulator: EraValidator = PreMergeAccumulator::default().into();
    let epoch: Epoch = headers.try_into().unwrap();

    let result = premerge_accumulator.validate_eras(&[&epoch])?;

    let expected = [
        94, 193, 255, 184, 195, 177, 70, 244, 38, 6, 199, 76, 237, 151, 61, 193, 110, 197, 161, 7,
        192, 52, 88, 88, 195, 67, 252, 148, 120, 11, 66, 24,
    ];
    assert_eq!(result.first(), Some(&expected));

    Ok(())
}

// #[test]
// fn test_era_validate_compressed() -> Result<(), HeaderAccumulatorError> {
//     // clean up before tests
//     if let Err(e) = fs::remove_file("lockfile.json") {
//         eprintln!("Error deleting lockfile.json: {}", e);
//     }
//
//     let mut headers: Vec<ExtHeaderRecord> = Vec::new();
//     for number in (0..=8200).step_by(100) {
//         let file_name = format!("tests/compressed/{:010}.dbin.zst", number);
//         match decode_flat_files(file_name, None, None, Decompression::Zstd) {
//             Ok(blocks) => {
//                 let (successful_headers, _): (Vec<_>, Vec<_>) = blocks
//                     .iter()
//                     .cloned()
//                     .map(|block| ExtHeaderRecord::try_from(&block))
//                     .fold((Vec::new(), Vec::new()), |(mut succ, mut errs), res| {
//                         match res {
//                             Ok(header) => succ.push(header),
//                             Err(e) => {
//                                 // Log the error or handle it as needed
//                                 eprintln!("Error converting block: {:?}", e);
//                                 errs.push(e);
//                             }
//                         };
//                         (succ, errs)
//                     });
//
//                 headers.extend(successful_headers);
//             }
//             Err(e) => {
//                 eprintln!("error: {:?}", e);
//                 break;
//             }
//         }
//     }
//
//     assert_eq!(headers.len(), 8300);
//     assert_eq!(headers[0].block_number, 0);
//
//     let premerge_accumulator = PreMergeAccumulator::default();
//
//     let result = premerge_accumulator.era_validate(headers.clone(), 0, None, false)?;
//     println!("result 1: {:?}", result);
//
//     assert!(result.contains(&0), "The vector does not contain 0");
//
//     // Test with creating a lockfile
//     let result = premerge_accumulator.era_validate(headers.clone(), 0, None, true)?;
//     println!("result 2: {:?}", result);
//
//     assert!(result.contains(&0), "The vector does not contain 0");
//
//     // test with the lockfile created before.
//
//     let result = premerge_accumulator.era_validate(headers.clone(), 0, None, true)?;
//
//     // already validated epochs are not included in the array.
//     assert_eq!(result.len(), 0);
//
//     // clean up after tests
//     if let Err(e) = fs::remove_file("lockfile.json") {
//         eprintln!("Error deleting lockfile.json: {}", e);
//     }
//
//     Ok(())
// }
