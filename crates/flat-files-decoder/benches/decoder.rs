extern crate rand;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use flat_files_decoder::{compression::Compression, decoder::read_flat_file};
use std::fs;

const ITERS_PER_FILE: usize = 10;

fn bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("handle-flat-files");
    group.sample_size(ITERS_PER_FILE);

    group.bench_function("handle-flat-file", |b| {
        let files = fs::read_dir("tests/benchmark_files").expect("Failed to read dir");
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

            b.iter(|| read_flat_file(black_box(&path), Compression::None));
        }
    });

    group.finish();
}

criterion_group!(benches, bench);
criterion_main!(benches);
