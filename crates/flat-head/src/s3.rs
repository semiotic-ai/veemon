use dotenvy::dotenv;
use header_accumulator::{Epoch, EraValidator, ExtHeaderRecord, MAX_EPOCH_SIZE};
use std::env;
use trin_validation::accumulator::PreMergeAccumulator;

use flat_files_decoder::decoder::{handle_reader, Compression};

use object_store::{aws::AmazonS3Builder, path::Path, ObjectStore};

use crate::utils::gen_dbin_filenames;

fn handle_var(var_name: &str) -> String {
    match env::var(var_name) {
        Ok(value) => value,
        Err(e) => {
            println!("Error reading environment variable {}: {}", var_name, e);
            std::process::exit(1);
        }
    }
}

pub async fn s3_fetch(
    start_epoch: u64,
    end_epoch: u64,
    endpoint: Option<String>,
    decompress: Option<bool>,
) {
    dotenv().ok();

    let aws_region = handle_var("AWS_REGION");
    let bucket_name = handle_var("BUCKET_NAME");
    let access_key_id = handle_var("ACCESS_KEY_ID");
    let secret_key = handle_var("SECRET_KEY");

    let mut builder = AmazonS3Builder::new()
        .with_region(aws_region)
        .with_bucket_name(bucket_name)
        .with_access_key_id(access_key_id)
        .with_secret_access_key(secret_key)
        .with_allow_http(true);

    if let Some(endpoint) = endpoint {
        builder = builder.with_endpoint(endpoint);
    }

    let s3 = builder.build().unwrap();

    let file_names = gen_dbin_filenames(start_epoch, end_epoch, decompress);

    let era_validator: EraValidator = PreMergeAccumulator::default().into();

    let mut headers: Vec<ExtHeaderRecord> = Vec::new();
    for file_name in file_names {
        let path_string = format!("/{}", file_name);
        let path = Path::from(path_string);
        let result = s3.get(&path).await.unwrap();

        let bytes = result.bytes().await.unwrap();

        // Use `as_ref` to get a &[u8] from `bytes` and pass it to `handle_buf`
        match handle_reader(bytes.as_ref(), Compression::None) {
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
                // Handle the successfully decoded blocks
            }
            Err(e) => {
                log::error!("error: {:?}", e);
                // Handle the decoding error
            }
        }
        if headers.len() >= MAX_EPOCH_SIZE {
            let epoch: Epoch = headers
                .drain(0..MAX_EPOCH_SIZE)
                .collect::<Vec<_>>()
                .try_into()
                .unwrap();
            let valid_blocks = era_validator.validate_era(&epoch);
            println!("{:?} valid epochs", valid_blocks);
        }
    }
}
