// Copyright 2024-, Semiotic AI, Inc.
// SPDX-License-Identifier: Apache-2.0

//! # Empty Slot Block Hashes
//!
//! This example demonstrates that empty Beacon slots - slots with no execution block -
//! are represented in the [`BeaconState`] as duplicates of the previous full Beacon slot block hash.

use std::collections::{BTreeMap, HashSet};

use forrestrie::beacon_state::{HeadState, CAPELLA_START_ERA, SLOTS_PER_HISTORICAL_ROOT};
use primitive_types::H256;
use types::MainnetEthSpec;

#[tokio::main]
async fn main() {
    // This slot was chosen because it is the first slot of an era (and an epoch),
    // which we demonstrate by showing that the slot number (see below) modulo 8192 is 0.
    // You may need to update the slot being queried as the state data is updated.
    // Multiply a recent era by 8192 to get the slot number.
    const SLOT: u64 = 10182656;
    let url = format!("https://www.lightclientdata.org/eth/v2/debug/beacon/states/{SLOT}");
    println!("Requesting state for slot {SLOT} ... (this can take a while!)");
    let response = reqwest::get(url).await.unwrap();
    let state: HeadState<MainnetEthSpec> = if response.status().is_success() {
        let json_response: serde_json::Value = response.json().await.unwrap();
        serde_json::from_value(json_response).unwrap()
    } else {
        panic!("Request failed with status: {}", response.status());
    };

    let slot = state.data().slot().as_usize();

    // Every 8192 slots, the era increments by 1, and (after Capella) the historical summaries buffer is updated.
    let current_era = slot / SLOTS_PER_HISTORICAL_ROOT;
    assert_eq!(slot % SLOTS_PER_HISTORICAL_ROOT, 0);

    // The historical summaries buffer is updated every 8192 slots, from the start of the Capella era.
    let num_historical_summaries = state.data().historical_summaries().unwrap().len();
    assert_eq!((current_era - num_historical_summaries), CAPELLA_START_ERA);

    let block_roots = state.data().block_roots().to_vec();

    // Block roots buffer contains duplicates.
    let block_roots_set: HashSet<&H256, std::hash::RandomState> =
        HashSet::from_iter(block_roots.iter());
    assert_ne!(block_roots_set.len(), block_roots.len());

    let duplicate_block_roots_lookup_table = state
        .data()
        .block_roots()
        .to_vec()
        .iter()
        .enumerate()
        // Using BTreeMaps for deterministic order.
        .fold(BTreeMap::<H256, Vec<usize>>::new(), |mut acc, (i, root)| {
            acc.entry(*root).or_default().push(i);
            acc
        })
        // Remove non-duplicate block roots.
        .into_iter()
        .filter(|(_, indices)| indices.len() > 1)
        .collect::<BTreeMap<H256, Vec<usize>>>();

    // The block roots buffer contains duplicates that are consecutive.
    insta::assert_debug_snapshot!(duplicate_block_roots_lookup_table, @r###"
    {
        0x0b181ac43241327561b0c9cb4e070f72989581fb49ed26f5435bef997c42ebf5: [
            991,
            992,
        ],
        0x0e0639696be97e597e0c7ee1acfff59f165ba9f5945e729633cff21c0c635848: [
            6467,
            6468,
        ],
        0x0e26c5f8321a19c33e0e190ad7f72d0c135ab8ba7104cd8a0473242c7064db18: [
            6688,
            6689,
        ],
        0x0f40bc4698ba17aa95faade9e069e95456bcffd3e966c1fb0510df803038df48: [
            3396,
            3397,
        ],
        0x0f57d09bbf7b6a20a76ce3683e555ca86e7a1c38a3d1414bc6afb76894c460c1: [
            5265,
            5266,
        ],
        0x1473d4d768a680e9e61e7c6904df1a798545e14295b664bd7be06951140e4650: [
            7162,
            7163,
        ],
        0x1e30ab8ebd808669cca453279c2d89ed1965c3920f33e2715aca2d3e2756722a: [
            1162,
            1163,
        ],
        0x287c4b53d1b7be5c6c1e42ee596cb6b2803dcdaf17821798f21175b1a7ded5a8: [
            7543,
            7544,
        ],
        0x2cf6c52b3a63f73d8393734ce77540b0ae4914f403128ef2e0b9dcabb36dd443: [
            5087,
            5088,
        ],
        0x2d8144f651ad2c0864d586f43c446570177ae0dc219f15ff9469dfd05fc8de6e: [
            6465,
            6466,
        ],
        0x3514b0a08790ff1047150893234575c86705b9b98ca0a0a109a39da2216a3a4f: [
            2432,
            2433,
        ],
        0x3e12555313ed8ad02e60bdf78688892a54e6e02498fffd5a2ed0dbfc38d97db5: [
            2532,
            2533,
        ],
        0x41eb012e02e62f6e31bf742c709a3e9ec654b9258ff86b2061d124f0839a0188: [
            1799,
            1800,
        ],
        0x4294e233c2ca1055127d1373ffaf96f91386a187f888c9de4742ea79ff2e67f0: [
            3958,
            3959,
        ],
        0x498bb1ca0923c4a56e094e2f6fe2243dff4a9766f317309da7c57be5940f5a56: [
            124,
            125,
        ],
        0x4ca5d89aaa6de795d3432fda64bbecf4aed5fa0966193e77aa1c98599fb08ebe: [
            7807,
            7808,
        ],
        0x4f497aaff8a60ec338bc3fd19e0089d3cfa922bd093f767e6ba34ce0ec0e69e9: [
            6175,
            6176,
        ],
        0x515519a00556388934dd24fd9c528b26af2dce885c4cd3d5b0120b3939808ddc: [
            4410,
            4411,
        ],
        0x56cf582ed2d994dc15c4e4e49cea4e013e5ccb825997958255ebff9a9c70a126: [
            4127,
            4128,
        ],
        0x59ef61abc9d0dee4a8c19d3f636876bc38aa56559bf29315b26ccfd66da73aa9: [
            1510,
            1511,
        ],
        0x5db5cee0a5a63e6f20744bd2da116f0b7ff1346b6b6014cf847977bd6036b17e: [
            5297,
            5298,
        ],
        0x5fe37ef18fdaee618fb869760b20be5f7d04334e93f356b00e313f3f2d4b5eb6: [
            3743,
            3744,
        ],
        0x6808158ef68b250694ebc6cfbd90418a0182ac3d6e0560ad19212dc902a31c2f: [
            1937,
            1938,
        ],
        0x6820e4ea1e925a0198c67e40d4a066778898cd8b2e6fea4c32d7dccec7c548d6: [
            7294,
            7295,
        ],
        0x69dfd5cbd798a693d3deb17ae9bb6c0b075e50d0e414b710f58438dc2a54b51d: [
            3540,
            3541,
        ],
        0x6f0b738c363cc6739178e2c813dc47d7ff9aaef5cda7b838a964ff67aa626ab3: [
            1667,
            1668,
        ],
        0x6fec0abed7cbf72b3c0c0fb00db7fa7e78fdf55c7bc52804971e9997e8c37ef6: [
            5439,
            5440,
        ],
        0x71afc6470dd6ea94a1cfa95d91975d2b2d0efcf261bcf92a37eeb722a10907e5: [
            1518,
            1519,
        ],
        0x99254b3ae83a064a9dd91f68c60630e88727bd2989110a041fed6aacb0780239: [
            3555,
            3556,
        ],
        0x9c91fed096d21a383bf4cba7ee5213c68f5fb662225af74864450d45bdd93e01: [
            6028,
            6029,
        ],
        0xa89ca327b5d989c3066ea390053651a5f8b951829bda21257f7b33503e1c7abc: [
            6240,
            6241,
        ],
        0xaba1fa146c1665c2a3083987e55a9ae15dc04800d527ca98f2baf8692b96d5fd: [
            7167,
            7168,
        ],
        0xb077d4b158fa43c1ac54ee3d608d9430970d10cbc64a219b819fc279f7d3d3e0: [
            3380,
            3381,
        ],
        0xc4153799a620d470ced2bf02f0275f6353ec57be64f34bb06a0bc3a13423a9e3: [
            5453,
            5454,
        ],
        0xcebd2b3111fce7d8f9f1ddcf556d7ba54aa0999342184e7c58fa262131e94283: [
            2894,
            2895,
        ],
        0xd436e0dbe68b089f4dca99cac9ab4dc044b448c3569ff029c230d1539c643b93: [
            1036,
            1037,
        ],
        0xd7e5a02180a5116042684af1b180739609e2424bbb4deb0d030b023b23490438: [
            2050,
            2051,
        ],
        0xda13e985195e44855e08d5bd2d54ca6ac8f4cfaa5668526760d521aeaa9c4178: [
            7478,
            7479,
        ],
        0xde5d5a3b2f2da2b6482adcd4f61c6addbf45dfee24ff938931ac90e56c9e73a9: [
            6430,
            6431,
        ],
        0xdef43bbd5c642857fdbb5fdcf8e566c1e1dffbb543c3a29e8d606c25e60d2bf3: [
            5491,
            5492,
        ],
        0xf406504fad9ec2165294c51a47bf6d258c07f7db212b897ebe5611153fbfcb88: [
            3839,
            3840,
        ],
        0xfe5b350eb4ae790d3c14db582269d3edea28f803a76983ababbf31926a7c9ff3: [
            6784,
            6785,
        ],
    }
    "###);

    println!("Empty slot block hashes example completed successfully");
}
