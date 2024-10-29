# Forrestrie Examples

Here's an example of how to run one of the examples in the `forrestrie-examples` crate:

```terminal
cd crates/forrestrie-examples && cargo run -- --examples historical_state_roots_proof
```

Use environment variables to provide Firehose Ethereum and Firehose
Beacon providers of your choice.

To do this, place a `.env` file in the root of `veemon`. See the
`.env.example` file in the root of this repository for what you'll need,
depending on your requirements.
