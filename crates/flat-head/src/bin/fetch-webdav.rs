use clap::Parser;
use decoder::handle_buf;
use flat_head::{era_verifier::MAX_EPOCH_SIZE, utils::gen_dbin_filenames};
use header_accumulator::{era_validator::era_validate, types::ExtHeaderRecord};
use object_store::{http::HttpBuilder, path::Path, ClientOptions, ObjectStore};

/// This program is intended for fetching
/// flat files from a google cloud storage and verifying them. It skips fetching files
/// that were already verified or are already present. Flat files are stored
/// in 100's blocks,
#[derive(Parser, Debug)]
#[command(version, about = "a flat files google clouid storage fetch and verify", long_about = None)]
struct Args {
    // name of the bucket where files are sotred
    #[arg(short, long)]
    url: String,

    /// epoch to start fetching blocks flat files from
    #[arg(short, long)]
    start_epoch: u64,

    /// Number of times to greet
    #[arg(short, long)]
    end_epoch: u64,

    #[clap(short = 'c', long, default_value = "true")]
    // Where to decompress files from zstd or not.
    decompress: Option<bool>,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let dav = HttpBuilder::new()
        .with_url(args.url)
        .with_client_options(ClientOptions::new().with_allow_http(true))
        .build()
        .expect("server errors");

    // Get an `async` stream of Metadata objects:
    let file_names = gen_dbin_filenames(args.start_epoch, args.end_epoch, args.decompress);

    for file_name in file_names {
        let path_string = format!("/{}", file_name);
        let path = Path::from(path_string);
        let result = dav.get(&path).await.unwrap();

        let bytes = result.bytes().await.unwrap();

        let mut headers: Vec<ExtHeaderRecord> = Vec::new();

        // Use `as_ref` to get a &[u8] from `bytes` and pass it to `handle_buf`
        match handle_buf(bytes.as_ref()) {
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
            Err(_e) => {
                // Handle the decoding error
            }
        }
        if headers.len() > 8192 {
            let epoch_headers: Vec<ExtHeaderRecord> = headers.drain(0..MAX_EPOCH_SIZE).collect();
            let valid_headers = era_validate(
                epoch_headers,
                None,
                args.start_epoch as usize,
                Some(args.end_epoch as usize),
                None,
            );
            println!("{:?} valid epochs", valid_headers);
        }
    }
}
