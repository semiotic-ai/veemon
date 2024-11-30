# Firehose Protos Examples

Examples that use methods implemented on the Rust-compiled Firehose
protobuffer types from [Firehose Protos](../firehose_protos/index.html).

## Running Examples

To run the examples, you need access to a Firehose provider for each chain from which you want
to extract data - an endpoint and an API key (if the latter is required).

If you need access to a Firehose provider, we suggest using [Pinax](https://app.pinax.network/).

Add your endpoint and API key to a `.env` file in the root of the repository. See `.env.examples` for
a configuration template.

To run individual examples, use the following command:

```terminal
cargo run -p firehose-protos-examples --example <example_name>
```

So, for example, to run the `receipt_root` example:

```terminal
cargo run -p firehose-protos-examples --example receipt_root
```
