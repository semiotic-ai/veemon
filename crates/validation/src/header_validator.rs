// SPDX-FileCopyrightText: 2021-2025 Trin Contributors
// SPDX-License-Identifier: MIT

use alloy_consensus::Header;
use alloy_primitives::B256;
use anyhow::anyhow;
use ethportal_api::{
    consensus::historical_summaries::HistoricalSummaries,
    types::execution::header_with_proof::{
        BlockHeaderProof, BlockProofHistoricalRoots, BlockProofHistoricalSummariesCapella,
        BlockProofHistoricalSummariesDeneb, HeaderWithProof,
    },
};

use crate::{
    constants::{DENEB_BLOCK_NUMBER, EPOCH_SIZE, MERGE_BLOCK_NUMBER, SHANGHAI_BLOCK_NUMBER},
    historical_roots::HistoricalRootsAccumulator,
    merkle::proof::verify_merkle_proof,
    post_capella_types::{
        BeaconSlot, BlockRootIndex, ValidatedHistoricalSummaryIndex, BEACON_BLOCK_PROOF_DEPTH,
        EXECUTION_BLOCK_GENERALIZED_INDEX,
    },
    PreMergeAccumulator,
};

#[cfg(test)]
use crate::constants::{CAPELLA_FORK_EPOCH, SLOTS_PER_EPOCH};

/// Wrapper enum for post-Capella proof variants (Capella and Deneb eras).
///
/// This enum provides a unified interface for verifying execution headers after the
/// Capella fork, abstracting over structural differences between fork variants while
/// maintaining type safety and era validation.
///
/// # Structural Differences
///
/// The primary difference between variants is the execution block proof length:
/// - **Capella**: 11 sibling hashes ([`ExecutionBlockProofBellatrix`])
/// - **Deneb**: 12 sibling hashes ([`ExecutionBlockProofDeneb`])
///
/// This reflects changes in the beacon block structure introduced in the Deneb upgrade.
///
/// # Era Boundaries
///
/// - **Capella era**: Blocks [`SHANGHAI_BLOCK_NUMBER`] (17,034,870) to [`DENEB_BLOCK_NUMBER`] (19,426,587)
/// - **Deneb era**: Blocks >= [`DENEB_BLOCK_NUMBER`] (19,426,587)
///
/// # Usage
///
/// Wrap proof structs from [`ethportal_api`] in the appropriate enum variant:
///
/// ```rust
/// use validation::{PostCapellaProof, HeaderValidator};
/// # use ethportal_api::types::execution::header_with_proof::{
/// #     BlockProofHistoricalSummariesDeneb,
/// #     BlockProofHistoricalSummariesCapella,
/// # };
/// # use alloy_primitives::B256;
/// # use validation::constants::{DENEB_BLOCK_NUMBER, SHANGHAI_BLOCK_NUMBER};
/// # let deneb_proof = BlockProofHistoricalSummariesDeneb {
/// #     beacon_block_root: B256::ZERO,
/// #     execution_block_proof: vec![B256::ZERO; 12].into(),
/// #     beacon_block_proof: vec![B256::ZERO; 13].into(),
/// #     slot: 6209536,
/// # };
/// # let capella_proof = BlockProofHistoricalSummariesCapella {
/// #     beacon_block_root: B256::ZERO,
/// #     execution_block_proof: vec![B256::ZERO; 11].into(),
/// #     beacon_block_proof: vec![B256::ZERO; 13].into(),
/// #     slot: 6209536,
/// # };
///
/// // Wrap Deneb-era proof
/// let proof = PostCapellaProof::Deneb(&deneb_proof);
///
/// // Access proof data through unified interface
/// let beacon_root = proof.beacon_block_root();
/// let slot = proof.slot();
///
/// // Validate era matches block number
/// proof.validate_era(DENEB_BLOCK_NUMBER).unwrap();
///
/// // Wrap Capella-era proof
/// let capella = PostCapellaProof::Capella(&capella_proof);
/// capella.validate_era(SHANGHAI_BLOCK_NUMBER).unwrap();
/// ```
///
/// # Verification Flow
///
/// Both variants follow the same verification algorithm, proving a chain of inclusion:
/// ```text
/// Execution Block Header → Beacon Block → Historical Summary
/// ```
///
/// Use with [`HeaderValidator::verify_post_capella_header`] for full verification.
///
/// [`ExecutionBlockProofBellatrix`]: ethportal_api::types::execution::header_with_proof::ExecutionBlockProofBellatrix
/// [`ExecutionBlockProofDeneb`]: ethportal_api::types::execution::header_with_proof::ExecutionBlockProofDeneb
/// [`SHANGHAI_BLOCK_NUMBER`]: crate::constants::SHANGHAI_BLOCK_NUMBER
/// [`DENEB_BLOCK_NUMBER`]: crate::constants::DENEB_BLOCK_NUMBER
#[derive(Debug, Clone, Copy)]
pub enum PostCapellaProof<'a> {
    /// Proof for blocks in the Capella era (Shanghai to Deneb).
    ///
    /// Valid for blocks in the range [`SHANGHAI_BLOCK_NUMBER`] to [`DENEB_BLOCK_NUMBER`] - 1.
    /// Uses 11-element execution block proofs ([`ExecutionBlockProofBellatrix`]).
    ///
    /// [`SHANGHAI_BLOCK_NUMBER`]: crate::constants::SHANGHAI_BLOCK_NUMBER
    /// [`DENEB_BLOCK_NUMBER`]: crate::constants::DENEB_BLOCK_NUMBER
    /// [`ExecutionBlockProofBellatrix`]: ethportal_api::types::execution::header_with_proof::ExecutionBlockProofBellatrix
    Capella(&'a BlockProofHistoricalSummariesCapella),

    /// Proof for blocks in the Deneb era (Cancun/Deneb onwards).
    ///
    /// Valid for blocks >= [`DENEB_BLOCK_NUMBER`].
    /// Uses 12-element execution block proofs ([`ExecutionBlockProofDeneb`]).
    ///
    /// [`DENEB_BLOCK_NUMBER`]: crate::constants::DENEB_BLOCK_NUMBER
    /// [`ExecutionBlockProofDeneb`]: ethportal_api::types::execution::header_with_proof::ExecutionBlockProofDeneb
    Deneb(&'a BlockProofHistoricalSummariesDeneb),
}

impl<'a> PostCapellaProof<'a> {
    /// Returns the beacon block root that bridges execution and consensus layers.
    ///
    /// This root identifies the specific beacon block that includes the execution payload
    /// being verified. It serves as both the target in execution block verification and
    /// the leaf in historical summary verification.
    ///
    /// # Returns
    ///
    /// The 32-byte SSZ hash tree root of the beacon block.
    pub fn beacon_block_root(&self) -> B256 {
        match self {
            Self::Capella(p) => p.beacon_block_root,
            Self::Deneb(p) => p.beacon_block_root,
        }
    }

    /// Returns the Merkle proof for execution block inclusion in the beacon block.
    ///
    /// This proof demonstrates that the execution block header is correctly embedded
    /// within the beacon block's execution payload at the path:
    /// `BeaconBlock.body.execution_payload.block_hash`
    ///
    /// # Returns
    ///
    /// A slice of sibling hashes forming the Merkle proof path:
    /// - **Capella**: 11 hashes
    /// - **Deneb**: 12 hashes
    pub fn execution_block_proof(&self) -> &[B256] {
        match self {
            Self::Capella(p) => &p.execution_block_proof,
            Self::Deneb(p) => &p.execution_block_proof,
        }
    }

    /// Returns the Merkle proof for beacon block inclusion in historical summaries.
    ///
    /// This proof demonstrates that the beacon block root is correctly included in
    /// a historical summary's `block_summary_root`, which covers 8192 beacon block roots.
    ///
    /// # Returns
    ///
    /// A slice of exactly 13 sibling hashes (since 2^13 = 8192).
    pub fn beacon_block_proof(&self) -> &[B256] {
        match self {
            Self::Capella(p) => &p.beacon_block_proof,
            Self::Deneb(p) => &p.beacon_block_proof,
        }
    }

    /// Returns the beacon chain slot number.
    ///
    /// The slot number is used to calculate which historical summary contains the beacon
    /// block and where within that summary to find it. Each slot represents 12 seconds
    /// on the beacon chain.
    ///
    /// # Returns
    ///
    /// The beacon chain slot number as a 64-bit unsigned integer.
    pub fn slot(&self) -> u64 {
        match self {
            Self::Capella(p) => p.slot,
            Self::Deneb(p) => p.slot,
        }
    }

    /// Validates that the proof variant matches the block era.
    ///
    /// This method ensures that the correct proof type is used for the given block number,
    /// preventing misuse of Capella proofs for Deneb-era blocks and vice versa.
    ///
    /// # Era Validation Rules
    ///
    /// - **Capella proofs**: Valid for blocks in [`SHANGHAI_BLOCK_NUMBER`] to [`DENEB_BLOCK_NUMBER`] - 1
    /// - **Deneb proofs**: Valid for blocks >= [`DENEB_BLOCK_NUMBER`]
    ///
    /// # Arguments
    ///
    /// * `block_number` - The execution layer block number to validate against
    ///
    /// # Returns
    ///
    /// - `Ok(())` if the proof type is appropriate for the block number
    /// - `Err` with a descriptive message if there's a mismatch
    pub fn validate_era(&self, block_number: u64) -> anyhow::Result<()> {
        match self {
            Self::Capella(_) => {
                if block_number < SHANGHAI_BLOCK_NUMBER {
                    return Err(anyhow!(
                        "capella proof used for pre-shanghai block {}",
                        block_number
                    ));
                }
                if block_number >= DENEB_BLOCK_NUMBER {
                    return Err(anyhow!(
                        "capella proof used for deneb-era block {} (>= {})",
                        block_number,
                        DENEB_BLOCK_NUMBER
                    ));
                }
                Ok(())
            }
            Self::Deneb(_) => {
                if block_number < DENEB_BLOCK_NUMBER {
                    return Err(anyhow!(
                        "deneb proof used for pre-deneb block {} (< {})",
                        block_number,
                        DENEB_BLOCK_NUMBER
                    ));
                }
                Ok(())
            }
        }
    }
}

fn calculate_generalized_index(header: &Header) -> u64 {
    // Calculate generalized index for header
    // https://github.com/ethereum/consensus-specs/blob/v0.11.1/ssz/merkle-proofs.md#generalized-merkle-tree-index
    let hr_index = header.number % EPOCH_SIZE;
    (EPOCH_SIZE * 2 * 2) + (hr_index * 2)
}

/// HeaderValidator is responsible for validating pre-merge and post-merge headers with their
/// respective proofs.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct HeaderValidator {
    /// Pre-merge accumulator used to validate pre-merge headers.
    pub pre_merge_acc: PreMergeAccumulator,
    /// Historical roots accumulator used to validate post-merge/pre-Capella headers.
    pub historical_roots_acc: HistoricalRootsAccumulator,
    /// Historical summaries used to validate post-Capella headers.
    pub historical_summaries: Option<HistoricalSummaries>,
}

impl HeaderValidator {
    pub fn new() -> Self {
        let pre_merge_acc = PreMergeAccumulator::default();
        let historical_roots_acc = HistoricalRootsAccumulator::default();

        Self {
            pre_merge_acc,
            historical_roots_acc,
            historical_summaries: None,
        }
    }

    pub fn new_with_historical_summaries(historical_summaries: HistoricalSummaries) -> Self {
        let pre_merge_acc = PreMergeAccumulator::default();
        let historical_roots_acc = HistoricalRootsAccumulator::default();

        Self {
            pre_merge_acc,
            historical_roots_acc,
            historical_summaries: Some(historical_summaries),
        }
    }

    pub fn validate_header_with_proof(&self, hwp: &HeaderWithProof) -> anyhow::Result<()> {
        match &hwp.proof {
            BlockHeaderProof::HistoricalHashes(proof) => {
                if hwp.header.number > MERGE_BLOCK_NUMBER {
                    return Err(anyhow!("invalid proof type found for post-merge header"));
                }
                // look up historical epoch hash for header from pre-merge accumulator
                let gen_index = calculate_generalized_index(&hwp.header);
                let epoch_index =
                    self.pre_merge_acc.get_epoch_index_of_header(&hwp.header) as usize;
                let epoch_hash = self.pre_merge_acc.historical_epochs[epoch_index];

                match verify_merkle_proof(
                    hwp.header.hash_slow(),
                    proof,
                    15,
                    gen_index as usize,
                    epoch_hash,
                ) {
                    true => Ok(()),
                    false => Err(anyhow!(
                        "merkle proof validation failed for pre-merge header"
                    )),
                }
            }
            BlockHeaderProof::HistoricalRoots(proof) => self.verify_post_merge_pre_capella_header(
                hwp.header.number,
                hwp.header.hash_slow(),
                proof,
            ),
            BlockHeaderProof::HistoricalSummariesDeneb(proof) => {
                let summaries = self.historical_summaries.as_ref().ok_or_else(|| {
                    anyhow!(
                        "historical summaries required for post-capella validation of block {} but not provided",
                        hwp.header.number
                    )
                })?;
                self.verify_post_capella_header(
                    hwp.header.number,
                    hwp.header.hash_slow(),
                    PostCapellaProof::Deneb(proof),
                    summaries,
                )
            }
            BlockHeaderProof::HistoricalSummariesCapella(proof) => {
                let summaries = self.historical_summaries.as_ref().ok_or_else(|| {
                    anyhow!(
                        "historical summaries required for post-capella validation of block {} but not provided",
                        hwp.header.number
                    )
                })?;
                self.verify_post_capella_header(
                    hwp.header.number,
                    hwp.header.hash_slow(),
                    PostCapellaProof::Capella(proof),
                    summaries,
                )
            }
        }
    }

    /// A method to verify the chain of proofs for post-merge/pre-Capella execution headers.
    fn verify_post_merge_pre_capella_header(
        &self,
        block_number: u64,
        header_hash: B256,
        proof: &BlockProofHistoricalRoots,
    ) -> anyhow::Result<()> {
        if block_number <= MERGE_BLOCK_NUMBER {
            return Err(anyhow!(
                "invalid HistoricalRootsBlockProof found for pre-merge header"
            ));
        }
        if block_number >= SHANGHAI_BLOCK_NUMBER {
            return Err(anyhow!(
                "invalid HistoricalRootsBlockProof found for post-shanghai header"
            ));
        }

        // verify the chain of proofs for post-merge/pre-capella block header
        Self::verify_beacon_block_proof(
            header_hash,
            &proof.execution_block_proof,
            proof.beacon_block_root,
        )?;

        let block_root_index = proof.slot % EPOCH_SIZE;
        let gen_index = 2 * EPOCH_SIZE + block_root_index;
        let historical_root_index = proof.slot / EPOCH_SIZE;
        let historical_root =
            self.historical_roots_acc.historical_roots[historical_root_index as usize];

        if !verify_merkle_proof(
            proof.beacon_block_root,
            &proof.beacon_block_proof,
            14,
            gen_index as usize,
            historical_root,
        ) {
            return Err(anyhow!(
                "merkle proof validation failed for HistoricalRootsProof"
            ));
        }

        Ok(())
    }

    /// Verifies the chain of proofs for post-Capella execution headers.
    ///
    /// # Post-Capella Validation Algorithm
    ///
    /// This method implements the verification chain for execution headers after the Capella
    /// fork, working with both Capella and Deneb era proofs through the `PostCapellaProof` enum.
    ///
    /// ## Validation Steps
    ///
    /// ### Step 0: Era Validation
    /// - Validates proof variant matches block era via [`PostCapellaProof::validate_era`]
    /// - Capella proofs: blocks in [`SHANGHAI_BLOCK_NUMBER`] to [`DENEB_BLOCK_NUMBER`] - 1
    /// - Deneb proofs: blocks >= [`DENEB_BLOCK_NUMBER`]
    ///
    /// ### Step 1: Pre-conditions Check
    /// - Ensures block number is >= Shanghai (17,034,870)
    /// - Rejects pre-Shanghai blocks with post-Capella proofs
    ///
    /// [`SHANGHAI_BLOCK_NUMBER`]: crate::constants::SHANGHAI_BLOCK_NUMBER
    /// [`DENEB_BLOCK_NUMBER`]: crate::constants::DENEB_BLOCK_NUMBER
    ///
    /// ### Step 2: Execution Block -> Beacon Block Verification
    /// - Verifies the execution block header is included in the beacon block body
    /// - Uses a Merkle proof with generalized index 3228
    /// - Path: `BeaconBlock.body.execution_payload.block_hash`
    ///
    /// ### Step 3: Calculate Historical Summary Index
    /// - Computes which epoch the slot belongs to relative to Capella fork
    /// - Formula: `(slot - CAPELLA_FORK_EPOCH * SLOTS_PER_EPOCH) / EPOCH_SIZE`
    /// - Each epoch contains 8192 slots (32 slots/epoch * 256 epochs)
    /// - Validates index is within bounds of historical summaries array
    ///
    /// ### Step 4: Beacon Block -> Historical Summary Verification
    /// - Calculates block root position within its epoch: `slot % EPOCH_SIZE`
    /// - Computes generalized index: `EPOCH_SIZE + block_root_index`
    /// - Verifies beacon block root against the historical summary's block_summary_root
    /// - Uses a Merkle proof with depth 13 (`log2(EPOCH_SIZE)`)
    ///
    /// ## Historical Summaries Structure
    ///
    /// After Capella, the beacon state maintains `historical_summaries` which is a list of
    /// `HistoricalSummary` objects. Each summary covers 8192 slots (one epoch) and contains:
    /// - `block_summary_root`: Root of all block roots in that epoch
    /// - `state_summary_root`: Root of all state roots in that epoch
    ///
    /// # Arguments
    ///
    /// * `block_number` - Execution layer block number
    /// * `header_hash` - Hash of the execution block header
    /// * `proof` - Post-Capella proof (Deneb or Capella variant)
    /// * `historical_summaries` - List of historical summaries from beacon state
    ///
    /// # Returns
    ///
    /// * `Ok(())` if all verification steps pass
    /// * `Err` if any step fails (era validation, pre-conditions, bounds check, or Merkle proof)
    ///
    /// # Examples
    ///
    /// ## Example 1: Capella Era Validation
    ///
    /// Validates a block from the Capella era (blocks 17,034,870 to 19,426,586).
    /// Capella-era proofs use 11-element execution block proofs.
    ///
    /// ```no_run
    /// use alloy_consensus::Header;
    /// use alloy_primitives::B256;
    /// use ethportal_api::{
    ///     consensus::historical_summaries::HistoricalSummaries,
    ///     types::execution::header_with_proof::BlockProofHistoricalSummariesCapella,
    /// };
    /// use validation::{HeaderValidator, header_validator::PostCapellaProof};
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// // create validator with historical summaries from beacon node
    /// let historical_summaries: HistoricalSummaries = load_historical_summaries()?;
    /// let validator = HeaderValidator::new_with_historical_summaries(historical_summaries.clone());
    ///
    /// // example capella-era block (block 17,100,000)
    /// let block_number = 17_100_000;
    /// let header = Header {
    ///     number: block_number,
    ///     // populate with actual header data
    ///     ..Default::default()
    /// };
    /// let header_hash = header.hash_slow();
    ///
    /// // capella proof structure with 11-element execution block proof
    /// // these proofs are obtained from beacon chain data
    /// let capella_proof = BlockProofHistoricalSummariesCapella {
    ///     beacon_block_root: B256::ZERO,  // root of beacon block containing this execution block
    ///     execution_block_proof: vec![B256::ZERO; 11].into(),  // 11 merkle siblings
    ///     beacon_block_proof: vec![B256::ZERO; 13].into(),     // 13 merkle siblings
    ///     slot: 6209536,  // beacon chain slot number
    /// };
    ///
    /// // verify the header using capella-era proof
    /// validator.verify_post_capella_header(
    ///     block_number,
    ///     header_hash,
    ///     PostCapellaProof::Capella(&capella_proof),
    ///     &historical_summaries,
    /// )?;
    ///
    /// println!("capella-era block verified successfully");
    /// # Ok(())
    /// # }
    /// #
    /// # fn load_historical_summaries() -> anyhow::Result<HistoricalSummaries> {
    /// #     Ok(vec![].into())
    /// # }
    /// ```
    ///
    /// ## Example 2: Deneb Era Validation
    ///
    /// Validates a block from the Deneb era (blocks ≥19,426,587).
    /// Deneb-era proofs use 12-element execution block proofs due to changes
    /// in the beacon block structure.
    ///
    /// ```no_run
    /// use alloy_consensus::Header;
    /// use alloy_primitives::B256;
    /// use ethportal_api::{
    ///     consensus::historical_summaries::HistoricalSummaries,
    ///     types::execution::header_with_proof::BlockProofHistoricalSummariesDeneb,
    /// };
    /// use validation::{HeaderValidator, header_validator::PostCapellaProof};
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// // create validator with historical summaries
    /// let historical_summaries: HistoricalSummaries = load_historical_summaries()?;
    /// let validator = HeaderValidator::new_with_historical_summaries(historical_summaries.clone());
    ///
    /// // example deneb-era block (block 19,500,000)
    /// let block_number = 19_500_000;
    /// let header = Header {
    ///     number: block_number,
    ///     ..Default::default()
    /// };
    /// let header_hash = header.hash_slow();
    ///
    /// // deneb proof structure with 12-element execution block proof
    /// // note the extra element compared to capella
    /// let deneb_proof = BlockProofHistoricalSummariesDeneb {
    ///     beacon_block_root: B256::ZERO,
    ///     execution_block_proof: vec![B256::ZERO; 12].into(),  // 12 merkle siblings
    ///     beacon_block_proof: vec![B256::ZERO; 13].into(),     // 13 merkle siblings
    ///     slot: 7123456,
    /// };
    ///
    /// // verify using deneb-era proof
    /// validator.verify_post_capella_header(
    ///     block_number,
    ///     header_hash,
    ///     PostCapellaProof::Deneb(&deneb_proof),
    ///     &historical_summaries,
    /// )?;
    ///
    /// println!("deneb-era block verified successfully");
    /// # Ok(())
    /// # }
    /// #
    /// # fn load_historical_summaries() -> anyhow::Result<HistoricalSummaries> {
    /// #     Ok(vec![].into())
    /// # }
    /// ```
    ///
    /// # See Also
    ///
    /// - [`PostCapellaProof`] for the unified proof interface and era validation
    /// - [`validate_header_with_proof`](Self::validate_header_with_proof) for validating pre-merge and post-merge headers
    pub fn verify_post_capella_header(
        &self,
        block_number: u64,
        header_hash: B256,
        proof: PostCapellaProof<'_>,
        historical_summaries: &HistoricalSummaries,
    ) -> anyhow::Result<()> {
        proof.validate_era(block_number)?;

        Self::verify_beacon_block_proof(
            header_hash,
            proof.execution_block_proof(),
            proof.beacon_block_root(),
        )?;

        let slot = BeaconSlot::new(proof.slot());
        let summary_index = slot.to_historical_summary_index(historical_summaries.len())?;
        let block_root_index = slot.block_root_index();

        Self::verify_beacon_block_in_summary(
            proof.beacon_block_root(),
            proof.beacon_block_proof(),
            block_root_index,
            summary_index,
            historical_summaries,
        )?;

        Ok(())
    }

    /// Verifies beacon block root is in historical summary.
    ///
    /// This method proves that the beacon block root is correctly included in a
    /// historical summary's `block_summary_root`, which covers 8192 beacon block roots.
    ///
    /// # Arguments
    ///
    /// * `beacon_block_root` - The beacon block root to verify
    /// * `beacon_block_proof` - Merkle proof siblings (exactly 13 elements)
    /// * `block_root_index` - Position of the block root within the epoch (0-8191)
    /// * `summary_index` - Validated index into the historical summaries array
    /// * `historical_summaries` - List of historical summaries from beacon state
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the Merkle proof verifies successfully
    /// * `Err` if verification fails
    fn verify_beacon_block_in_summary(
        beacon_block_root: B256,
        beacon_block_proof: &[B256],
        block_root_index: BlockRootIndex,
        summary_index: ValidatedHistoricalSummaryIndex,
        historical_summaries: &HistoricalSummaries,
    ) -> anyhow::Result<()> {
        let gen_index = block_root_index.generalized_index();
        let historical_summary = historical_summaries[summary_index.as_usize()].block_summary_root;

        if !verify_merkle_proof(
            beacon_block_root,
            beacon_block_proof,
            BEACON_BLOCK_PROOF_DEPTH,
            gen_index.as_usize(),
            historical_summary,
        ) {
            return Err(anyhow!(
                "merkle proof validation failed for beacon block in historical summary"
            ));
        }

        Ok(())
    }

    /// Verify that the execution block header is included in the beacon block.
    ///
    /// # Generalized Index Calculation
    ///
    /// This method verifies that an execution layer block header (identified by `header_hash`)
    /// is correctly embedded within a beacon chain block (identified by `block_body_root`).
    ///
    /// ## SSZ Merkle Tree Navigation
    ///
    /// The execution payload is nested three levels deep in the beacon block structure:
    ///
    /// ```text
    /// BeaconBlock (root = block_body_root)
    ///   ├─ slot: u64                           [index 0]
    ///   ├─ proposer_index: u64                 [index 1]
    ///   ├─ parent_root: Root                   [index 2]
    ///   ├─ state_root: Root                    [index 3]
    ///   └─ body: BeaconBlockBody               [index 4] ← We navigate here
    ///        ├─ randao_reveal: BLSSignature    [index 0]
    ///        ├─ eth1_data: Eth1Data             [index 1]
    ///        ├─ graffiti: Bytes32               [index 2]
    ///        ├─ proposer_slashings: List        [index 3]
    ///        ├─ attester_slashings: List        [index 4]
    ///        ├─ attestations: List              [index 5]
    ///        ├─ deposits: List                  [index 6]
    ///        ├─ voluntary_exits: List           [index 7]
    ///        ├─ sync_aggregate: SyncAggregate   [index 8]
    ///        └─ execution_payload: ExecutionPayload [index 9] ← Then here
    ///             ├─ parent_hash: Hash          [index 0]
    ///             ├─ fee_recipient: Address     [index 1]
    ///             ├─ state_root: Root           [index 2]
    ///             ├─ receipts_root: Root        [index 3]
    ///             ├─ logs_bloom: ByteVector     [index 4]
    ///             ├─ prev_randao: Bytes32       [index 5]
    ///             ├─ block_number: u64          [index 6]
    ///             ├─ gas_limit: u64             [index 7]
    ///             ├─ gas_used: u64              [index 8]
    ///             ├─ timestamp: u64             [index 9]
    ///             ├─ extra_data: ByteList       [index 10]
    ///             ├─ base_fee_per_gas: u256     [index 11]
    ///             └─ block_hash: Hash           [index 12] ← Finally here!
    ///                  (This is our execution header hash)
    /// ```
    ///
    /// ## Generalized Index Calculation
    ///
    /// To navigate through nested SSZ structures, we use generalized indices.
    /// Each level multiplies by the next power of 2 that contains all fields:
    ///
    /// 1. **BeaconBlock level**: 5 fields → round up to 8 (2^3)
    ///    - Start at root: generalized index = 1
    ///    - Navigate to field 4 (body): 1 * 8 + 4 = 12
    ///
    /// 2. **BeaconBlockBody level**: 10 fields → round up to 16 (2^4)
    ///    - Current index: 12
    ///    - Navigate to field 9 (execution_payload): 12 * 16 + 9 = 201
    ///
    /// 3. **ExecutionPayload level**: 14+ fields → round up to 16 (2^4)
    ///    - Current index: 201
    ///    - Navigate to field 12 (block_hash): 201 * 16 + 12 = 3228
    ///
    /// Therefore, the generalized index for the execution block hash is **3228**.
    ///
    /// ## Verification Process
    ///
    /// The Merkle proof verifies that:
    /// 1. Starting from `block_body_root` (beacon block root)
    /// 2. Following the path encoded in generalized index 3228
    /// 3. We arrive at `header_hash` (execution block hash)
    ///
    /// This cryptographically proves the execution block is part of this beacon block.
    fn verify_beacon_block_proof(
        header_hash: B256,
        block_body_proof: &[B256],
        block_body_root: B256,
    ) -> anyhow::Result<()> {
        if !verify_merkle_proof(
            header_hash,
            block_body_proof,
            block_body_proof.len(),
            EXECUTION_BLOCK_GENERALIZED_INDEX.as_usize(),
            block_body_root,
        ) {
            return Err(anyhow!(
                "merkle proof validation failed for execution block inclusion in beacon block"
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::B256;
    use ethportal_api::consensus::historical_summaries::{HistoricalSummaries, HistoricalSummary};

    fn mock_historical_summaries() -> HistoricalSummaries {
        vec![HistoricalSummary {
            block_summary_root: B256::ZERO,
            state_summary_root: B256::ZERO,
        }]
        .into()
    }

    fn mock_deneb_proof() -> BlockProofHistoricalSummariesDeneb {
        BlockProofHistoricalSummariesDeneb {
            beacon_block_root: B256::ZERO,
            execution_block_proof: vec![B256::ZERO; 12].into(),
            beacon_block_proof: vec![B256::ZERO; 13].into(),
            slot: CAPELLA_FORK_EPOCH * SLOTS_PER_EPOCH,
        }
    }

    fn mock_capella_proof() -> BlockProofHistoricalSummariesCapella {
        BlockProofHistoricalSummariesCapella {
            beacon_block_root: B256::ZERO,
            execution_block_proof: vec![B256::ZERO; 12].into(),
            beacon_block_proof: vec![B256::ZERO; 13].into(),
            slot: CAPELLA_FORK_EPOCH * SLOTS_PER_EPOCH,
        }
    }

    #[test]
    fn rejects_pre_shanghai_blocks_with_deneb_proof() {
        let validator = HeaderValidator::new();
        let proof = mock_deneb_proof();
        let summaries = mock_historical_summaries();

        let result = validator.verify_post_capella_header(
            SHANGHAI_BLOCK_NUMBER - 1,
            B256::ZERO,
            PostCapellaProof::Deneb(&proof),
            &summaries,
        );

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("pre-deneb"));
    }

    #[test]
    fn rejects_pre_shanghai_blocks_with_capella_proof() {
        let validator = HeaderValidator::new();
        let proof = mock_capella_proof();
        let summaries = mock_historical_summaries();

        let result = validator.verify_post_capella_header(
            SHANGHAI_BLOCK_NUMBER - 1,
            B256::ZERO,
            PostCapellaProof::Capella(&proof),
            &summaries,
        );

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("pre-shanghai"));
    }

    #[test]
    fn requires_historical_summaries_for_post_capella_validation() {
        let validator = HeaderValidator::new();
        let deneb_proof = mock_deneb_proof();

        let hwp = HeaderWithProof {
            header: Header {
                number: SHANGHAI_BLOCK_NUMBER + 1000,
                ..Default::default()
            },
            proof: BlockHeaderProof::HistoricalSummariesDeneb(deneb_proof),
        };

        let result = validator.validate_header_with_proof(&hwp);

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("historical summaries required"));
    }

    #[test]
    fn enum_methods_for_deneb_proof() {
        let proof = mock_deneb_proof();
        let wrapped = PostCapellaProof::Deneb(&proof);
        assert_eq!(wrapped.beacon_block_root(), B256::ZERO);
        assert_eq!(wrapped.slot(), CAPELLA_FORK_EPOCH * SLOTS_PER_EPOCH);
        assert_eq!(wrapped.execution_block_proof().len(), 12);
        assert_eq!(wrapped.beacon_block_proof().len(), 13);
    }

    #[test]
    fn validates_deneb_proof_era() {
        let proof = mock_deneb_proof();
        let wrapped = PostCapellaProof::Deneb(&proof);

        // deneb proofs should work for deneb-era blocks
        assert!(wrapped.validate_era(DENEB_BLOCK_NUMBER).is_ok());
        assert!(wrapped.validate_era(DENEB_BLOCK_NUMBER + 1000).is_ok());

        // deneb proofs should fail for pre-deneb blocks
        assert!(wrapped.validate_era(DENEB_BLOCK_NUMBER - 1).is_err());
        assert!(wrapped.validate_era(SHANGHAI_BLOCK_NUMBER).is_err());
    }

    #[test]
    fn validates_capella_proof_era() {
        let proof = mock_capella_proof();
        let wrapped = PostCapellaProof::Capella(&proof);

        // capella proofs should work for capella-era blocks (shanghai to deneb)
        assert!(wrapped.validate_era(SHANGHAI_BLOCK_NUMBER).is_ok());
        assert!(wrapped.validate_era(DENEB_BLOCK_NUMBER - 1).is_ok());

        // capella proofs should fail for pre-shanghai blocks
        assert!(wrapped.validate_era(SHANGHAI_BLOCK_NUMBER - 1).is_err());

        // capella proofs should fail for deneb-era blocks
        assert!(wrapped.validate_era(DENEB_BLOCK_NUMBER).is_err());
        assert!(wrapped.validate_era(DENEB_BLOCK_NUMBER + 1000).is_err());
    }
}
