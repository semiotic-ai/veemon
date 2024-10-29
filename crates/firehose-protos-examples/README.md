# Firehose Protos Examples

Here's an example of how to run one of the examples in the `firehose-protos-examples` crate:

```terminal
cd crates/firehose-protos-examples && cargo run -- --examples receipt_root
```

Use environment variables to provide Firehose Ethereum and Firehose
Beacon providers of your choice.

To do this, place a `.env` file in the root of `veemon`. See the
`.env.example` file in the root of this repository for what you'll need,
depending on your requirements.
