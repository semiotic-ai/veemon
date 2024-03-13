use clap::Parser;
use flat_head::s3::s3_fetch;

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

#[tokio::main]
async fn main() {
    let args = Args::parse();
    s3_fetch(args.start_epoch, args.end_epoch, args.endpoint).await;
}
