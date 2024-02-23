# Flat head

Flat head is developed to be a crate that verifies flat files generated from Firehose against header accumulators.


## Getting Started

### Prerequisites
- [Rust (stable)](https://www.rust-lang.org/tools/install)
- Cargo (Comes with Rust by default)
- [protoc](https://grpc.io/docs/protoc-installation/)
- Firehose dbin files to decode

## Running

### Commands


### Options


## Usage Examples

Here are some examples of how to use the commands:

1.  To validate flat files in a folder, a start epoch and a end epoch must be provided. `-d` flag can be used for debugging or log information.

```
cargo run --bin flat_head --  -d  era-validate --input  <folder>  --start-epoch 0 --end-epoch 1
```

Flat files can be compressed using Zstandard (zstd). To decompress, use `zstd -d *.zst` in your flat files folder

2. To fetch flat files from a FTP server and validate each file as they arrive:

```
cargo run --bin fetch-ftp --server <server> --fist-epoch 0 --end-epoch 1
```

This command will skip the files that were already verified and written into `lockfile.json`.
It stops abruptly if verification of any file fails. If files are compressed as `.zst` it is also capable
of decompressing them.

3. To fetch flat files from a gcloud bucket, and validate each file as they arreive:

```
cargo run --bin fetch-gcloud --bucket --fist-epoch 0 --end-epoch 1
```

**NOTICE: fetching from gcloud has a price ($0.10/GB currently) so be careful when using this method for many files**


<!-- 4. TODO: fetch from a webdav server -->


## Goals

Our goal is to provide The Graph's Indexers the tools to trustlessly share flat files with cryptographic guarantees 
that the data in the flat files is part of the canonical history of the Ethereum blockchain, 
enabling Indexers to quickly sync all historical data and begin serving data with minimal effort.


## Integration tests

running some commands to fetch flat files from server might require an instance with flat files running:

```
docker run --restart always -v /absolute/path/to/flat_files/:/var/lib/dav \
  -e AUTH_TYPE=Digest -e USERNAME=alice -e PASSWORD=secret1234 -e ANONYMOUS_METHODS=GET,POST,OPTIONS,PROPFIND \
  --publish 80:80 -d bytemark/webdav
```