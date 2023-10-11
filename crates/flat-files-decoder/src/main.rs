use clap::Parser;
use decoder::decode_flat_files;

#[derive(Parser, Debug)]
struct Args {
    #[clap(short, long, default_value = "input_files")]
    input: String,
    #[clap(long)]
    headers_dir: Option<String>,
    #[clap(short, long)]
    output: Option<String>,
}

fn main() {
    let args = Args::parse();

    let blocks = decode_flat_files(
        &args.input,
        args.output.as_deref(),
        args.headers_dir.as_deref(),
    )
    .expect("Failed to decode files");

    println!("Total blocks: {}", blocks.len());
}
