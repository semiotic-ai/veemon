/// The maximum number of slots per epoch in Ethereum.
/// In the context of Proof of Stake (PoS) consensus, an epoch is a collection of slots
/// during which validators propose and attest to blocks. The maximum size of an epoch
/// defines the number of slots that can be included in one epoch.
pub const MAX_EPOCH_SIZE: usize = 8192;

/// The final epoch number before the Ethereum network underwent "The Merge."
/// "The Merge" refers to the event where Ethereum transitioned from Proof of Work (PoW)
/// to Proof of Stake (PoS). The final epoch under PoW was epoch 1896.
pub const FINAL_EPOCH: usize = 1896;

/// The block number at which "The Merge" occurred in the Ethereum network.
/// "The Merge" took place at block 15537394, when the Ethereum network fully switched
/// from Proof of Work (PoW) to Proof of Stake (PoS).
pub const MERGE_BLOCK: u64 = 15537394;
