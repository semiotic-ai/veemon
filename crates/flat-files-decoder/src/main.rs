use decoder::decode_flat_files;

fn main() {
    let blocks = decode_flat_files("input_files").expect("Failed to decode files");

    println!("Total blocks: {}", blocks.len());
}

