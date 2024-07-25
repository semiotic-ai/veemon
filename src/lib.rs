use beacon_node::beacon_chain::types::{BeaconState, EthSpec};
use serde::Deserialize;

#[allow(unused)]
#[derive(Deserialize)]
struct HeadState<E: EthSpec> {
    version: String,
    execution_optimistic: bool,
    data: BeaconState<E>,
}

#[cfg(test)]
mod tests {
    use super::*;

    use beacon_node::beacon_chain::types::MainnetEthSpec;
    use types::light_client_update::FINALIZED_ROOT_INDEX;

    const HEAD_STATE_JSON: &str = include_str!("../head-state.json");

    #[test]
    fn spike_deserialize_head_state_and_compute_merkle_proof() {
        let state: HeadState<MainnetEthSpec> = serde_json::from_str(HEAD_STATE_JSON).unwrap();

        let proof = state
            .data
            .compute_merkle_proof(FINALIZED_ROOT_INDEX)
            .unwrap();

        insta::assert_debug_snapshot!(proof, @r###"
        [
            0x2084040000000000000000000000000000000000000000000000000000000000,
            0x29fd8c86a3d314172f4d273d90691fdb8c73acb464a2acdf865d370a5a91f6ea,
            0x985137d97513dd35feaf1617f7e8d985fa251b68244b63373cf9c50376fd2bfd,
            0xf0e45527b9da9c9266caeabcaf03ffb9cbc3576888b3f8628f9372d9d03f1feb,
            0x50d36fdf9425494f04a6e5cd996855b7c1b7fb581bab17e3e97c5e48d232d1f2,
            0x2150a88f205759c59817f42dc307620c67d3d23417959286928d186c639a0948,
        ]
        "###);
    }
}
