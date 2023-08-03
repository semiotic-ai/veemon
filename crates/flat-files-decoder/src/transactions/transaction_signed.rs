use protobuf::MessageField;
use reth_primitives::{Signature, Transaction, TransactionSigned};
use crate::protos::block::{BigInt, TransactionTrace};


impl TryFrom<&TransactionTrace> for TransactionSigned {
    type Error = anyhow::Error;

    fn try_from(trace: &TransactionTrace) -> Result<Self, Self::Error> {
        let transaction = Transaction::try_from(trace)?;
        let signature = Signature::try_from(trace)?;

        let tx_signed = TransactionSigned::from_transaction_and_signature(transaction, signature);
        Ok(tx_signed)
    }
}

pub fn u128_from_field(field: &MessageField<BigInt>) -> anyhow::Result<u128> {
    field.get_or_default().clone().try_into()
}