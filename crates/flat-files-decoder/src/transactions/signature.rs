use reth_primitives::{hex, Signature, U256};
use sf_protos::ethereum::r#type::v2::TransactionTrace;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum InvalidSignatureError {
    #[error("Invalid R: {0}")]
    R(String),
    #[error("Invalid S: {0}")]
    S(String),
    #[error("Invalid V: {0}")]
    V(u8),
}

pub fn signature_from_trace(trace: &TransactionTrace) -> Result<Signature, InvalidSignatureError> {
    let r_bytes: [u8; 32] = trace
        .r
        .as_slice()
        .try_into()
        .map_err(|_| InvalidSignatureError::R(hex::encode(&trace.r)))?;
    let r = U256::from_be_bytes(r_bytes);

    let s_bytes: [u8; 32] = trace
        .s
        .as_slice()
        .try_into()
        .map_err(|_| InvalidSignatureError::S(hex::encode(&trace.s)))?;
    let s = U256::from_be_bytes(s_bytes);

    let odd_y_parity = get_y_parity(trace)?;

    Ok(Signature { r, s, odd_y_parity })
}

fn get_y_parity(trace: &TransactionTrace) -> Result<bool, InvalidSignatureError> {
    let v: u8 = if trace.v.is_empty() { 0 } else { trace.v[0] };

    if v == 0 || v == 1 {
        Ok(v == 1)
    } else if v == 27 || v == 28 {
        Ok(v - 27 == 1)
    } else if v == 37 || v == 38 {
        Ok(v - 37 == 1)
    } else {
        Err(InvalidSignatureError::V(v))
    }
}
