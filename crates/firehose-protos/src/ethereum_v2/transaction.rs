// Copyright 2024-, Semiotic AI, Inc.
// SPDX-License-Identifier: Apache-2.0

use std::fmt::Display;

use alloy_consensus::{TxEip1559, TxEip2930, TxLegacy};
use alloy_eip2930::{AccessList, AccessListItem};
use alloy_primitives::{
    hex, Address, Bytes, ChainId, FixedBytes, Parity, TxKind, Uint, U128, U256,
};
use reth_primitives::{Signature, Transaction, TransactionSigned, TxType};
use tracing::debug;

use crate::error::ProtosError;

use super::{transaction_trace::Type, BigInt, CallType, TransactionReceipt, TransactionTrace};

impl From<Type> for TxType {
    fn from(tx_type: Type) -> Self {
        use TxType::*;
        use Type::*;

        match tx_type {
            TrxTypeLegacy => Legacy,
            TrxTypeAccessList => Eip2930,
            TrxTypeDynamicFee => Eip1559,
            TrxTypeBlob => Eip4844,
            TrxTypeArbitrumDeposit => unimplemented!(),
            TrxTypeArbitrumUnsigned => unimplemented!(),
            TrxTypeArbitrumContract => unimplemented!(),
            TrxTypeArbitrumRetry => unimplemented!(),
            TrxTypeArbitrumSubmitRetryable => unimplemented!(),
            TrxTypeArbitrumInternal => unimplemented!(),
            TrxTypeArbitrumLegacy => unimplemented!(),
            TrxTypeOptimismDeposit => unimplemented!(),
        }
    }
}

/// Ethereum mainnet chain ID.
pub const CHAIN_ID: ChainId = 1;

/// Determines the chain ID for legacy Ethereum transactions based on the `v` value in the transaction trace.
///
/// In Ethereum, the `v` value within a transaction's signature component can indicate whether the transaction
/// is a legacy (pre-EIP-155) transaction or an EIP-155 transaction that includes a chain ID. Legacy transactions
/// have `v` values of `27` or `28`, which do not encode a chain ID. For such transactions, this function returns `None`.
/// For non-legacy transactions where `v` encodes a chain ID, this function returns the constant mainnet chain ID.
///
fn get_legacy_chain_id(trace: &TransactionTrace) -> Option<ChainId> {
    let v = trace.v();
    if v == 27 || v == 28 {
        None
    } else {
        Some(CHAIN_ID)
    }
}

impl TransactionTrace {
    /// Returns true if the transaction's status is successful.
    pub(crate) fn is_success(&self) -> bool {
        self.status == 1
    }

    fn parity(&self) -> Result<Parity, ProtosError> {
        // Extract the first byte of the V value (Ethereum's V value).
        let v = self.v();

        let parity = match v {
            // V values 0 and 1 directly indicate Y parity.
            0 | 1 => v == 1,

            // V values 27 and 28 are commonly used in Ethereum and indicate Y parity.
            27 | 28 => v - 27 == 1,

            // V values 37 and 38 are less common but still valid and represent Y parity.
            37 | 38 => v - 37 == 1,

            // If V is outside the expected range, return an error.
            _ => {
                return Err(ProtosError::TraceSignatureInvalid(
                    EcdsaComponent::V.to_string(),
                    v.to_string(),
                ))
            }
        };

        Ok(parity.into())
    }

    pub(crate) fn receipt(&self) -> Result<&TransactionReceipt, ProtosError> {
        self.receipt
            .as_ref()
            .ok_or(ProtosError::TransactionTraceMissingReceipt)
    }

    fn v(&self) -> u8 {
        if self.v.is_empty() {
            0
        } else {
            self.v[0]
        }
    }
}

#[derive(Clone, Debug)]
enum EcdsaComponent {
    R,
    S,
    V,
}

impl Display for EcdsaComponent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EcdsaComponent::R => write!(f, "R"),
            EcdsaComponent::S => write!(f, "S"),
            EcdsaComponent::V => write!(f, "V"),
        }
    }
}

impl TryFrom<&TransactionTrace> for TxKind {
    type Error = ProtosError;

    fn try_from(trace: &TransactionTrace) -> Result<Self, Self::Error> {
        let first_call = trace
            .calls
            .first()
            .ok_or(ProtosError::TransactionMissingCall)?;

        match first_call.call_type() {
            CallType::Create => Ok(TxKind::Create),
            _ => {
                // The `alloy_primitives` `TxKind` classifies all non-`Create` call types as a `Call`.
                debug!(
                    "Transaction trace has a non-`Create` call type: {:?}",
                    first_call
                );
                let address = Address::from_slice(trace.to.as_slice());
                Ok(TxKind::Call(address))
            }
        }
    }
}

impl TryFrom<&TransactionTrace> for Signature {
    type Error = ProtosError;

    fn try_from(trace: &TransactionTrace) -> Result<Self, Self::Error> {
        use EcdsaComponent::*;

        // Extract the R value from the trace and ensure it's a valid 32-byte array.
        let r_bytes: [u8; 32] = trace.r.as_slice().try_into().map_err(|_| {
            Self::Error::TraceSignatureInvalid(R.to_string(), hex::encode(&trace.r))
        })?;
        let r = U256::from_be_bytes(r_bytes);

        // Extract the S value from the trace and ensure it's a valid 32-byte array.
        let s_bytes: [u8; 32] = trace.s.as_slice().try_into().map_err(|_| {
            Self::Error::TraceSignatureInvalid(S.to_string(), hex::encode(&trace.s))
        })?;
        let s = U256::from_be_bytes(s_bytes);

        // Extract the Y parity from the V value.
        let odd_y_parity = trace.parity()?;

        Ok(Signature::new(r, s, odd_y_parity))
    }
}

impl TryFrom<&TransactionTrace> for reth_primitives::TxType {
    type Error = ProtosError;

    fn try_from(trace: &TransactionTrace) -> Result<Self, Self::Error> {
        match Type::try_from(trace.r#type) {
            Ok(tx_type) => Ok(TxType::from(tx_type)),
            Err(e) => Err(ProtosError::TxTypeConversion(e.to_string())),
        }
    }
}

impl TryFrom<&TransactionTrace> for Transaction {
    type Error = ProtosError;

    fn try_from(trace: &TransactionTrace) -> Result<Self, Self::Error> {
        let tx_type = reth_primitives::TxType::try_from(trace)?;
        let nonce = trace.nonce;
        let gas_price = get_u128_or_default(&trace.gas_price)?;
        let gas_limit = trace.gas_limit;
        let to = TxKind::try_from(trace)?;
        let value = Uint::from(get_u128_or_default(&trace.value)?);
        let input = Bytes::copy_from_slice(trace.input.as_slice());

        let transaction: Transaction = match tx_type {
            TxType::Legacy => Transaction::Legacy(TxLegacy {
                chain_id: get_legacy_chain_id(trace),
                nonce,
                gas_price,
                gas_limit,
                to,
                value,
                input,
            }),
            TxType::Eip2930 => Transaction::Eip2930(TxEip2930 {
                chain_id: CHAIN_ID,
                nonce,
                gas_price,
                gas_limit,
                to,
                value,
                access_list: AccessList::try_from(trace)?,
                input,
            }),
            TxType::Eip1559 => Transaction::Eip1559(TxEip1559 {
                chain_id: CHAIN_ID,
                nonce,
                gas_limit,
                max_fee_per_gas: get_u128_or_default(&trace.max_fee_per_gas)?,
                max_priority_fee_per_gas: get_u128_or_default(&trace.max_priority_fee_per_gas)?,
                to,
                value,
                access_list: AccessList::try_from(trace)?,
                input,
            }),
            TxType::Eip4844 => unimplemented!(),
            TxType::Eip7702 => unimplemented!(),
        };

        Ok(transaction)
    }
}

fn get_u128_or_default(opt_big_int: &Option<BigInt>) -> Result<u128, ProtosError> {
    let big_int = match opt_big_int {
        Some(gas_price) => gas_price,
        None => &BigInt { bytes: vec![0] },
    };
    u128::try_from(big_int)
}

impl TryFrom<&TransactionTrace> for TransactionSigned {
    type Error = ProtosError;

    fn try_from(trace: &TransactionTrace) -> Result<Self, Self::Error> {
        let transaction = Transaction::try_from(trace)?;
        let signature = Signature::try_from(trace)?;
        let hash = FixedBytes::from_slice(trace.hash.as_slice());

        Ok(TransactionSigned {
            transaction,
            signature,
            hash,
        })
    }
}

impl TryFrom<&TransactionTrace> for AccessList {
    type Error = ProtosError;

    fn try_from(trace: &TransactionTrace) -> Result<Self, Self::Error> {
        let access_list_items = trace
            .access_list
            .iter()
            .map(AccessListItem::try_from)
            .collect::<Result<Vec<AccessListItem>, Self::Error>>()?;

        Ok(AccessList(access_list_items))
    }
}

impl TryFrom<&BigInt> for u128 {
    type Error = ProtosError;

    fn try_from(value: &BigInt) -> Result<Self, Self::Error> {
        let slice = value.bytes.as_slice();
        let n =
            U128::try_from_be_slice(slice).ok_or(ProtosError::BigIntInvalid(hex::encode(slice)))?;
        Ok(u128::from_le_bytes(n.to_le_bytes()))
    }
}

#[cfg(test)]
mod tests {
    use crate::ethereum_v2::Call;

    use super::*;
    use alloy_primitives::Address;

    #[test]
    fn test_get_u128_or_default() {
        let valid_bigint = BigInt {
            bytes: vec![0, 0, 0, 1],
        };
        let result = get_u128_or_default(&Some(valid_bigint)).unwrap();
        assert_eq!(result, 1);

        let result = get_u128_or_default(&None).unwrap();
        assert_eq!(result, 0);
    }

    #[test]
    fn test_get_legacy_chain_id() {
        let mut trace = TransactionTrace {
            v: vec![27],
            ..Default::default()
        };
        // For 27 and 28, no chain ID
        assert_eq!(get_legacy_chain_id(&trace), None);

        trace.v = vec![37];
        // Check chain ID for other cases
        assert_eq!(get_legacy_chain_id(&trace), Some(CHAIN_ID));
    }

    #[test]
    fn test_transaction_trace_to_txkind() {
        let trace = TransactionTrace {
            to: Address::from_slice(&[0x00; 20]).to_vec(),
            calls: vec![Call::default()],
            ..Default::default()
        };
        let tx_kind = TxKind::try_from(&trace).unwrap();
        assert_eq!(tx_kind, TxKind::Call(Address::from_slice(&[0x00; 20])));
    }

    #[test]
    fn test_transaction_trace_to_signature() {
        let mut trace = TransactionTrace {
            r: {
                let mut vec = vec![0x00; 31];
                vec.push(0x01);
                vec
            },
            s: {
                let mut vec = vec![0x00; 31];
                vec.push(0x01);
                vec
            },
            v: vec![27],
            ..Default::default()
        };

        let signature = Signature::try_from(&trace).unwrap();
        assert_eq!(signature.r(), U256::from(1));
        assert_eq!(signature.s(), U256::from(1));
        assert!(!trace.parity().unwrap().y_parity());

        trace.v = vec![28];
        assert!(trace.parity().unwrap().y_parity());
    }

    #[test]
    fn test_transaction_trace_conversion() {
        // Test each `TxType` case with representative data

        // Legacy transaction
        let mut trace = TransactionTrace {
            r#type: Type::TrxTypeLegacy as i32,
            nonce: 1,
            gas_price: Some(BigInt {
                bytes: vec![0, 0, 1],
            }),
            gas_limit: 21000,
            to: Address::from_slice(&[0x02; 20]).to_vec(),
            value: Some(BigInt {
                bytes: vec![0, 0, 5],
            }),
            input: vec![0x01, 0x02, 0x03],
            calls: vec![Call::default()],
            ..Default::default()
        };

        let tx = Transaction::try_from(&trace).unwrap();
        match tx {
            Transaction::Legacy(tx) => {
                assert_eq!(tx.nonce, 1);
                assert_eq!(tx.gas_price, 1);
                assert_eq!(tx.gas_limit, 21000);
                assert_eq!(tx.value, Uint::from(5));
            }
            _ => panic!("Expected Legacy transaction"),
        }

        // EIP-2930 transaction
        trace.r#type = Type::TrxTypeAccessList as i32;
        let tx = Transaction::try_from(&trace).unwrap();
        assert!(matches!(tx, Transaction::Eip2930(_)));

        // EIP-1559 transaction
        trace.r#type = Type::TrxTypeDynamicFee as i32;
        let tx = Transaction::try_from(&trace).unwrap();
        assert!(matches!(tx, Transaction::Eip1559(_)));
    }

    #[test]
    fn test_access_list_conversion() {
        let trace = TransactionTrace::default();
        let access_list = AccessList::try_from(&trace).unwrap();
        assert_eq!(access_list.0.len(), trace.access_list.len());
    }

    #[test]
    fn test_invalid_bigint_conversion() {
        let invalid_bigint = BigInt {
            bytes: vec![0xFF; 17],
        }; // More than 16 bytes, should fail
        let result: Result<u128, ProtosError> = u128::try_from(&invalid_bigint);
        assert!(result.is_err());
    }
}
