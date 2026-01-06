// SPDX-FileCopyrightText: 2024- Semiotic AI, Inc.
// SPDX-License-Identifier: Apache-2.0

use std::{
    fs::{self, File},
    io::BufReader,
};

use criterion::{criterion_group, criterion_main, Criterion};
use flat_files_decoder::read_block_from_reader;
use prost::Message;
use std::hint::black_box;

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
                let mut message: Result<Vec<u8>, _> = Ok(Vec::new());

                b.iter(|| {
                    message = black_box(read_block_from_reader(&mut reader));
                });

                if message.is_err() {
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
            while let Ok(message) = read_block_from_reader(&mut reader) {
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
            while let Ok(message) = read_block_from_reader(&mut reader) {
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
            while let Ok(message) = read_block_from_reader(&mut reader) {
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
            while let Ok(message) = read_block_from_reader(&mut reader) {
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
