# Forrestrie Examples

Here's an example of how to run one of the examples in the `forrestrie-examples` crate:

```terminal
cd crates/forrestrie-examples && cargo run -- --examples historical_state_roots_proof
```

Use environment variables to provide Firehose Ethereum and Firehose
Beacon providers of your choice.

To do this, place a `.env` file in the root of `veemon`. 

Your `.env` file should look like something this, depending on your
requirements:

```shell
FIREHOSE_ETHEREUM_URL=<YOUR-FIREHOSE-ETHEREUM-URL>
FIREHOSE_ETHEREUM_PORT=<YOUR-FIREHOSE-ETHEREUM-PORT>
FIREHOSE_BEACON_URL=<YOUR-FIREHOSE-BEACON-URL>
FIREHOSE_BEACON_PORT=<YOUR-FIREHOSE-BEACON-PORT>
BEACON_API_KEY=<YOUR-API-KEY>
ETHEREUM_API_KEY=<YOUR-API-KEY>
```

Pinax is a great provider for blockchain data, so if you need an API key,
you might want to check them out.
