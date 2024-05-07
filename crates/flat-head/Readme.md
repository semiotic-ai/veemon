# Flat head

Flat head is developed to be a crate that verifies flat files generated from Firehose against header accumulators.


## Getting Started

### Prerequisites
- [Rust (stable)](https://www.rust-lang.org/tools/install)
- Cargo (Comes with Rust by default)
- [protoc](https://grpc.io/docs/protoc-installation/)
- Firehose dbin files to decode

## Running

There are a few different binaries to run, depending on the desired functionality:

`flat_head` is for general usage on flat files in a local folder. Run 
`cargo run --bin flat_head help` for commands and options. 

`fetch-s3` is supposed to be used to fetch flat files from an s3-like object storage.

## Usage Examples

Here are some examples of how to use the commands:

1.  To validate flat files in a folder, a start epoch and a end epoch must be provided. `-d` flag can be used for debugging or log information.

```
 cargo run --bin flat_head -- era-validate --store-url file:///<full-path-to-folder> -s 0   
```


2. To fetch flat files from a s3 bucket and validate each epoch as they arrive:

```
 cargo run --bin flat_head -- era-validate --store-url s3:///<full-path-to-folder> -s 0   

```

3. To fetch files from a seaweed-fs s3 compatible bucket and validate epochs:


```
cargo run --bin flat_head -- era-validate --store-url http://localhost:8333/newbucket3  -s 0 -e 1 --compatible s3

 ```

Note that in this case it is using seaweed-fs s3 compatible API.

4. To fetch flat files from a Webdav server and validate each file as they arrive:

```
 cargo run --bin flat_head -- era-validate --store-url http:///<full-path-to-folder> -s 0   
```


5. To fetch flat files from a gcloud bucket, and validate each epoch as they arrive:

```
 cargo run --bin flat_head -- era-validate --store-url gs:///<full-path-to-folder> -s 0   
```


### notice about usage

Flat files should come compressed with Zstandard (zstd) from Firehose. Flat_head handles decompression by default, but if it is necessary to disable it pass to the args: `-c false`. This is the same for all other binaries.

Passing `--end-epoch` is not necessary, although without it, `flat_head` will only validate the start epoch passed as param.

`era-validate` will skip the files that were already verified and written into `lockfile.json`.
It stops abruptly if verification of any file fails. If files are compressed as `.zst` it is also capable
of decompressing them.

An optional endpoint can be provided if running in a local environment or in another s3 compatible API.

Environment variables for aws have to be set for s3 in this scenario. An example is provided in `.env.example`

## Goals

Our goal is to provide The Graph's Indexers the tools to trustlessly share flat files with cryptographic guarantees 
that the data in the flat files is part of the canonical history of the Ethereum blockchain, 
enabling Indexers to quickly sync all historical data and begin serving data with minimal effort.


## Integration tests

### with webdav

running some commands to fetch flat files from server might require an instance with flat files running:

```
docker run --restart always -v /webdav/:/var/lib/dav \
  -e AUTH_TYPE=Digest -e USERNAME=alice -e PASSWORD=secret1234 -e ANONYMOUS_METHODS=GET,POST,OPTIONS,PROPFIND \
  --publish 80:80 -d bytemark/webdav
```

Then files must be fed into the webdav folder, either via interacting with the server directly or storing them into the volume.

### With S3

There is a minio `docker-compose` script which can be used to run a local s3 instance with [minio](https://github.com/minio/minio?tab=readme-ov-file) for development, with a mock access id and key and a bucket for development on `/dev` folder. Run `docker-compose up -d` to set it up, clone the `minio.env` to the root folder as `.env` and populate the bucket with flat files to test. Populating the bucket ca be done manually by accessing the 

**minio does not work for large scale flat files, use it only for testing purposes.**


### With seaweed-fs

It is possible to run seaweed-fs with an s3 compatible API locally:

```
docker run -p 8333:8333 chrislusf/seaweedfs server -s3
```

To test seaweed-fs locally it is necessary to add some flat files inside it first. There are instructions [here, on seaweed-fs wiki, on how to use the aws CLI to do that.](https://github.com/seaweedfs/seaweedfs/wiki/AWS-CLI-with-SeaweedFS).

To put it simply, first make sure to have dummy credentials at least. Then, create a bucket and populate it with the files in the test folder of this repo with a command similar to this: `aws --endpoint-url http://localhost:8333 s3 cp ./tests/compressed/  s3://newbucket3 --recursive`




### Coverage

Generate code coverage reports with `cargo llvm-cov --html` and open them with `open ./target/llvm-cov/html/index.html`. 