use prost::Message;
use crate::Encoder;

/// Extension trait to DBIN-encode prost messages without pulling in encoder logic into downstream crates.
pub trait DbinEncodeExt {
    /// Encode the message as a DBIN stream using ETH content type (V1).
    fn encode_to_dbin(&self) -> Vec<u8>;
}

// ETH block encoder: firehose_protos::EthBlock
impl DbinEncodeExt for firehose_protos::EthBlock {
    fn encode_to_dbin(&self) -> Vec<u8> {
        let payload = self.encode_to_vec();
        let encoder = Encoder::new_v1("ETH");
        encoder.wrap_stream(std::iter::once(payload))
    }
}

// Solana block encoder: firehose_protos::SolBlock
impl DbinEncodeExt for firehose_protos::SolBlock {
    fn encode_to_dbin(&self) -> Vec<u8> {
        let payload = self.encode_to_vec();
        let encoder = Encoder::new_v1("type.googleapis.com/sf.solana.type.v1.Block");
        encoder.wrap_stream(std::iter::once(payload))
    }
}
