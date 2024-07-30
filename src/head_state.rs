use primitive_types::H256;
use serde::{Deserialize, Serialize};
use types::{BeaconState, Error, EthSpec, MainnetEthSpec};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct HeadState<E: EthSpec> {
    version: String,
    execution_optimistic: bool,
    data: BeaconState<E>,
}

impl HeadState<MainnetEthSpec> {
    pub fn compute_merkle_proof(&self, index: usize) -> Result<Vec<H256>, Error> {
        self.data.compute_merkle_proof(index)
    }

    pub fn execution_optimistic(&self) -> bool {
        self.execution_optimistic
    }

    pub fn version(&self) -> &str {
        &self.version
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use types::light_client_update::HISTORICAL_ROOTS_INDEX;

    const HEAD_STATE_JSON: &str = include_str!("../head-state.json");

    #[test]
    fn spike_deserialize_head_state_and_compute_merkle_proof() {
        let state: HeadState<MainnetEthSpec> = serde_json::from_str(HEAD_STATE_JSON).unwrap();

        let proof = state.compute_merkle_proof(HISTORICAL_ROOTS_INDEX).unwrap();

        insta::assert_debug_snapshot!(proof, @r###"
        [
            0xe81a79506c46b126f75a08cdd5cbc35052b61ca944c6c3becf32432e2ee6373a,
            0xcfb49cd7eb0051153685e5e6124b635c6b9bcc69a6ead6af0ef7d9885fcc16e2,
            0x29c2e1f6d96493e9b49517cb78123990038429e4c3574688a48f9abe69238449,
            0xdb329a01d9114f087155633b36b498c8e60028c0acedc8e3b64e013dbbd4fa06,
            0x53b107024e402f616f8f348d900e0d62f4b6f0558d2bfbd09200e68620a5b9c2,
        ]
        "###);
    }
}
