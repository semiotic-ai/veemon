use std::{
    fs::File,
    io::{BufReader, BufWriter, Cursor, Write},
};

use flat_files_decoder::decoder::{stream_blocks, Reader};
use futures::StreamExt;

const TEST_ASSET_PATH: &str = "test-assets";

#[tokio::main]
async fn main() {
    env_logger::init();
    let mut buffer = Vec::new();
    let cursor: Cursor<&mut Vec<u8>> = Cursor::new(&mut buffer);
    let inputs = vec![
        format!("{TEST_ASSET_PATH}/example-create-17686085.dbin"),
        format!("{TEST_ASSET_PATH}/example0017686312.dbin"),
    ];
    {
        let mut writer = BufWriter::new(cursor);
        for i in inputs {
            let mut input = File::open(i).expect("couldn't read input file");

            std::io::copy(&mut input, &mut writer).expect("couldn't copy");
            writer.flush().expect("failed to flush output");
        }
    }
    let mut cursor = Cursor::new(buffer);
    cursor.set_position(0);

    let reader = BufReader::new(cursor);

    let mut blocks = Vec::new();

    let mut stream = stream_blocks(Reader::Buf(reader), None.into())
        .await
        .unwrap();

    while let Some(block) = stream.next().await {
        blocks.push(block);
    }

    assert_eq!(blocks.len(), 2);
    assert_eq!(blocks[0].number, 17686164);
    assert_eq!(blocks[1].number, 17686312);
    println!("Done");
}
