use reth_primitives::TxType;

use super::transaction_trace::Type;

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
