use dotenv::dotenv;
use header_accumulator::era_validator::era_validate;
use std::env;

use clap::Parser;
use decoder::{handle_buf, sf::ethereum::r#type::v2::Block};
use flat_head::{era_verifier::MAX_EPOCH_SIZE, utils::gen_dbin_filenames};
use object_store::{aws::AmazonS3Builder, path::Path, ObjectStore};

/// This program is intended for fetching
/// flat files from an FTP server and verifying them. It skips fetching files
/// that were already verified or are already present
#[derive(Parser, Debug)]
#[command(version, about = "a flat files FTP server fetch and verify", long_about = None)]
struct Args {
    /// epoch to start fetching flat files
    #[arg(short, long)]
    start_epoch: u64,

    /// epoch where flat files end
    #[arg(short, long)]
    end_epoch: u64,

    /// directly set an endpoint such as http://locahlost:900
    /// for local development or another s3 compatible API
    #[arg(short = 'p', long)]
    endpoint: Option<String>,
}

fn handle_var(var_name: &str) -> String {
    match env::var(var_name) {
        Ok(value) => value,
        Err(e) => {
            println!("Error reading environment variable {}: {}", var_name, e);
            std::process::exit(1);
        }
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let args = Args::parse();

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

    if let Some(endpoint) = args.endpoint {
        builder = builder.with_endpoint(endpoint);
    }

    let s3 = builder.build().unwrap();

    let file_names = gen_dbin_filenames(args.start_epoch, args.end_epoch);

    let mut blocks: Vec<Block> = Vec::new();
    for file_name in file_names {
        let path_string = format!("/{}", file_name);
        let path = Path::from(path_string);
        let result = s3.get(&path).await.unwrap();

        let bytes = result.bytes().await.unwrap();

        // Use `as_ref` to get a &[u8] from `bytes` and pass it to `handle_buf`
        match handle_buf(bytes.as_ref()) {
            Ok(new_blocks) => {
                blocks.extend(new_blocks);
                // Handle the successfully decoded blocks
            }
            Err(e) => {
                log::error!("error: {:?}", e);
                // Handle the decoding error
            }
        }
        if blocks.len() >= 8192 {
            let epoch_blocks: Vec<Block> = blocks.drain(0..MAX_EPOCH_SIZE).collect();
            let valid_blocks = era_validate(
                epoch_blocks,
                None,
                args.start_epoch as usize,
                Some(args.end_epoch as usize),
            );
            println!("{:?} valid epochs", valid_blocks);
        }
    }
}
