use prost::Message;
use crate::Encoder;

/// Encode a sequence of blocks into a single DBIN stream using the ETH content type.
///
/// - `blocks`: An iterator of items that implement `prost::Message` (e.g., `firehose_protos::EthBlock`).
/// - Returns a DBIN byte vector containing all encoded blocks in a single stream.
pub fn encode_blocks_to_dbin<T, I>(blocks: I) -> Vec<u8>
where
    T: Message,
    I: IntoIterator<Item = T>,
{
    // Collect encoded blocks into a Vec<Vec<u8>> for the DBIN writer
    let encoded_blocks: Vec<Vec<u8>> = blocks.into_iter().map(|b| b.encode_to_vec()).collect();
    // Use the ETH content-type encoder to wrap the stream with a header and frames
    let encoder = Encoder::new_v1("ETH");
    encoder.wrap_stream(encoded_blocks)
}
