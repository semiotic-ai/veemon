use protobuf::MessageField;
use reth_primitives::{Signature, Transaction, TransactionSigned, TxHash};
use crate::protos::block::{BigInt, TransactionTrace};


impl TryFrom<&TransactionTrace> for TransactionSigned {
    type Error = anyhow::Error;

    fn try_from(trace: &TransactionTrace) -> Result<Self, Self::Error> {
        let hash_bytes: [u8; 32] = trace.hash.as_slice().try_into()?;
        let hash = TxHash::from(hash_bytes);

        let transaction = Transaction::try_from(trace)?;
        let signature = Signature::try_from(trace)?;

        Ok(TransactionSigned {
            hash,
            signature,
            transaction,
        })
    }
}

pub fn u128_from_field(field: &MessageField<BigInt>) -> anyhow::Result<u128> {
    field.get_or_default().clone().try_into()
}