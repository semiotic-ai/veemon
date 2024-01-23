use std::{
    fs::{self, File},
    io::BufReader,
};

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use decoder::{
    dbin::{error::DbinFileError, DbinFile},
    receipts::check_receipt_root,
    sf,
    transactions::check_transaction_root,
};
use prost::Message;

const ITERS_PER_FILE: usize = 10;
// const ITERS_PER_BLOCK: usize = 1;

fn read_decode_check_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("read-decode-check");
    group.sample_size(ITERS_PER_FILE);

    group.bench_function("read-message-stream", |b| {
        let files = fs::read_dir("benchmark_files/pre_merge").expect("Failed to read dir");
        for file in files {
            let path = file.expect("Failed to get path").path();
            match path.extension() {
                None => continue,
                Some(ext) => {
                    if ext != "dbin" {
                        continue;
                    }
                }
            }
            let file = File::open(&path).expect("Failed to open file");
            let mut reader = BufReader::new(file);
            let mut message: Result<Vec<u8>, decoder::dbin::error::DbinFileError> =
                Err(DbinFileError::InvalidDBINBytes);
            loop {
                b.iter(|| {
                    message = black_box(DbinFile::read_message_stream(&mut reader));
                });
                match message {
                    Ok(_) => continue,
                    Err(_) => {
                        break;
                    }
                }
            }
        }
    });

    group.bench_function("decode-bstream", |b| {
        let files = fs::read_dir("benchmark_files/pre_merge").expect("Failed to read dir");
        for file in files {
            let path = file.expect("Failed to get path").path();
            match path.extension() {
                None => continue,
                Some(ext) => {
                    if ext != "dbin" {
                        continue;
                    }
                }
            }
            let file = File::open(&path).expect("Failed to open file");
            let mut reader = BufReader::new(file);
            loop {
                let message = match DbinFile::read_message_stream(&mut reader) {
                    Ok(message) => message,
                    Err(_) => {
                        break;
                    }
                };
                b.iter(|| {
                    black_box(sf::bstream::v1::Block::decode(message.as_slice())).unwrap();
                });
            }
        }
    });

    group.bench_function("decode-block", |b| {
        let files = fs::read_dir("benchmark_files/pre_merge").expect("Failed to read dir");
        for file in files {
            let path = file.expect("Failed to get path").path();
            match path.extension() {
                None => continue,
                Some(ext) => {
                    if ext != "dbin" {
                        continue;
                    }
                }
            }
            let file = File::open(&path).expect("Failed to open file");
            let mut reader = BufReader::new(file);
            loop {
                let message = match DbinFile::read_message_stream(&mut reader) {
                    Ok(message) => message,
                    Err(_) => {
                        break;
                    }
                };
                let block_stream = sf::bstream::v1::Block::decode(message.as_slice()).unwrap();
                b.iter(|| {
                    black_box(sf::ethereum::r#type::v2::Block::decode(
                        block_stream.payload_buffer.as_slice(),
                    ))
                    .unwrap();
                });
            }
        }
    });

    group.bench_function("receipts-check", |b| {
        let files = fs::read_dir("benchmark_files/pre_merge").expect("Failed to read dir");
        for file in files {
            let path = file.expect("Failed to get path").path();
            match path.extension() {
                None => continue,
                Some(ext) => {
                    if ext != "dbin" {
                        continue;
                    }
                }
            }
            let file = File::open(&path).expect("Failed to open file");
            let mut reader = BufReader::new(file);
            loop {
                let message = match DbinFile::read_message_stream(&mut reader) {
                    Ok(message) => message,
                    Err(_) => {
                        break;
                    }
                };
                let block_stream = sf::bstream::v1::Block::decode(message.as_slice()).unwrap();
                let block =
                    sf::ethereum::r#type::v2::Block::decode(block_stream.payload_buffer.as_slice())
                        .unwrap();
                b.iter(|| {
                    black_box(check_receipt_root(&block)).unwrap();
                });
            }
        }
    });

    group.bench_function("transactions-check", |b| {
        let files = fs::read_dir("benchmark_files/pre_merge").expect("Failed to read dir");
        for file in files {
            let path = file.expect("Failed to get path").path();
            match path.extension() {
                None => continue,
                Some(ext) => {
                    if ext != "dbin" {
                        continue;
                    }
                }
            }
            let file = File::open(&path).expect("Failed to open file");
            let mut reader = BufReader::new(file);
            loop {
                let message = match DbinFile::read_message_stream(&mut reader) {
                    Ok(message) => message,
                    Err(_) => {
                        break;
                    }
                };
                let block_stream = sf::bstream::v1::Block::decode(message.as_slice()).unwrap();
                let block =
                    sf::ethereum::r#type::v2::Block::decode(block_stream.payload_buffer.as_slice())
                        .unwrap();
                b.iter(|| {
                    black_box(check_transaction_root(&block)).unwrap();
                });
            }
        }
    });

    group.finish();
}

criterion_group!(benches, read_decode_check_bench);
criterion_main!(benches);
