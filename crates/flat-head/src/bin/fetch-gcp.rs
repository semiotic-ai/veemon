use clap::Parser;
use futures::stream::StreamExt;
use object_store::{gcp::GoogleCloudStorageBuilder, path::Path, ObjectStore};

/// This program is intended for fetching
/// flat files from a google cloud storage and verifying them. It skips fetching files
/// that were already verified or are already present. Flat files are stored
/// in 100's blocks,
#[derive(Parser, Debug)]
#[command(version, about = "a flat files google clouid storage fetch and verify", long_about = None)]
struct Args {
    // name of the bucket where files are sotred
    #[arg(short, long)]
    bucket_name: String,

    /// epoch to start fetching blocks flat files from
    #[arg(short, long)]
    start_epoch: u64,

    /// Number of times to greet
    #[arg(short, long)]
    end_epoch: u64,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    // let files: Vec<String> = gen_dbin_filenames(args.start_epoch, args.end_epoch);
    // print!("{:?}", files);

    let gcs = GoogleCloudStorageBuilder::from_env()
        .with_bucket_name(args.bucket_name)
        .build()
        .unwrap();

    let prefix = Path::from("fireeth-merged-blocks");
    let mut list_stream = gcs.list(Some(&prefix));
    // Get an `async` stream of Metadata objects:

    // Print a line about each object
    while let Some(meta) = list_stream.next().await.transpose().unwrap() {
        println!("Name: {}, size: {}", meta.location, meta.size);
    }
}
