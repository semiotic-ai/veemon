// Copyright 2024-, Semiotic AI, Inc.
// SPDX-License-Identifier: Apache-2.0

use std::{
    fs::{self, File},
    io::BufReader,
};

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use flat_files_decoder::{read_block_from_reader, ContentType};
use prost::Message;

const ITERS_PER_FILE: usize = 10;

#[allow(deprecated)]
fn read_decode_check_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("read-decode-check");
    group.sample_size(ITERS_PER_FILE);

    group.bench_function("read-message-stream", |b| {
        let files = fs::read_dir("tests/benchmark_files/pre_merge").expect("Failed to read dir");
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
                let message: Vec<u8> = Vec::new();
                let content_type: ContentType = String::from("").as_str().try_into().unwrap();
                let mut result: Result<(Vec<u8>, ContentType), _> = Ok((message, content_type));

                b.iter(|| {
                    result = black_box(read_block_from_reader(&mut reader));
                });

                if result.is_err() {
                    break;
                }
            }
        }
    });

    group.bench_function("decode-bstream", |b| {
        let files = fs::read_dir("tests/benchmark_files/pre_merge").expect("Failed to read dir");
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
                let (message, _content_type) = match read_block_from_reader(&mut reader) {
                    Ok((message, content_type)) => (message, content_type),
                    Err(_) => {
                        break;
                    }
                };
                b.iter(|| {
                    black_box(firehose_protos::BstreamBlock::decode(message.as_slice())).unwrap();
                });
            }
        }
    });

    group.bench_function("decode-block", |b| {
        let files = fs::read_dir("tests/benchmark_files/pre_merge").expect("Failed to read dir");
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
                let (message, _content_type) = match read_block_from_reader(&mut reader) {
                    Ok((message, content_type)) => (message, content_type),
                    Err(_) => {
                        break;
                    }
                };
                let block_stream =
                    firehose_protos::BstreamBlock::decode(message.as_slice()).unwrap();
                b.iter(|| {
                    black_box(firehose_protos::EthBlock::decode(
                        block_stream.payload_buffer.as_slice(),
                    ))
                    .unwrap();
                });
            }
        }
    });

    group.bench_function("receipts-check", |b| {
        let files = fs::read_dir("tests/benchmark_files/pre_merge").expect("Failed to read dir");
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
                let (message, _content_type) = match read_block_from_reader(&mut reader) {
                    Ok((message, content_type)) => (message, content_type),
                    Err(_) => {
                        break;
                    }
                };
                let block_stream =
                    firehose_protos::BstreamBlock::decode(message.as_slice()).unwrap();
                let block =
                    firehose_protos::EthBlock::decode(block_stream.payload_buffer.as_slice())
                        .unwrap();
                b.iter(|| {
                    black_box(block.receipt_root_is_verified());
                });
            }
        }
    });

    group.bench_function("transactions-check", |b| {
        let files = fs::read_dir("tests/benchmark_files/pre_merge").expect("Failed to read dir");
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
                let (message, _content_type) = match read_block_from_reader(&mut reader) {
                    Ok((message, content_type)) => (message, content_type),
                    Err(_) => {
                        break;
                    }
                };
                let block_stream =
                    firehose_protos::BstreamBlock::decode(message.as_slice()).unwrap();
                let block =
                    firehose_protos::EthBlock::decode(block_stream.payload_buffer.as_slice())
                        .unwrap();
                b.iter(|| {
                    black_box(block.transaction_root_is_verified());
                });
            }
        }
    });

    group.finish();
}

criterion_group!(benches, read_decode_check_bench);
criterion_main!(benches);
