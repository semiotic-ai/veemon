use crate::error::ProtosError;
use crate::firehose::v2::{Response, SingleBlockResponse};
use primitive_types::{H256, U256};
use prost::Message;
use ssz_types::{length::Fixed, Bitfield, FixedVector};
use types::{
    Address, BeaconBlockBodyDeneb, BitList, EthSpec, ExecutionBlockHash, Graffiti,
    IndexedAttestationBase, MainnetEthSpec, GRAFFITI_BYTES_LEN,
};

tonic::include_proto!("sf.beacon.r#type.v1");

impl<E: EthSpec> TryFrom<Attestation> for types::AttestationBase<E> {
    type Error = ProtosError;

    fn try_from(
        Attestation {
            aggregation_bits,
            data,
            signature,
        }: Attestation,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            aggregation_bits: BitList::from_bytes(aggregation_bits.as_slice().into())
                .map_err(|e| ProtosError::SszTypesError(format!("{:?}", e)))?,
            data: data.ok_or(ProtosError::NullAttestationData)?.try_into()?,
            signature: bls::generics::GenericAggregateSignature::deserialize(signature.as_slice())
                .map_err(|e| ProtosError::Bls(format!("{:?}", e)))?,
        })
    }
}

impl TryFrom<AttestationData> for types::AttestationData {
    type Error = ProtosError;

    fn try_from(
        AttestationData {
            slot,
            committee_index,
            beacon_block_root,
            source,
            target,
        }: AttestationData,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            slot: slot.into(),
            index: committee_index,
            beacon_block_root: H256::from_slice(beacon_block_root.as_slice()),
            source: source.ok_or(ProtosError::NullCheckpoint)?.into(),
            target: target.ok_or(ProtosError::NullCheckpoint)?.into(),
        })
    }
}

impl<E: EthSpec> TryFrom<AttesterSlashing> for types::AttesterSlashingBase<E> {
    type Error = ProtosError;

    fn try_from(
        AttesterSlashing {
            attestation_1,
            attestation_2,
        }: AttesterSlashing,
    ) -> Result<Self, Self::Error> {
        let attestation_1 = attestation_1.ok_or(ProtosError::NullSigner)?;
        let attestation_2 = attestation_2.ok_or(ProtosError::NullSigner)?;

        Ok(Self {
            attestation_1: attestation_1.try_into()?,
            attestation_2: attestation_2.try_into()?,
        })
    }
}

impl From<BeaconBlockHeader> for types::BeaconBlockHeader {
    fn from(
        BeaconBlockHeader {
            slot,
            proposer_index,
            parent_root,
            state_root,
            body_root,
        }: BeaconBlockHeader,
    ) -> Self {
        Self {
            slot: slot.into(),
            proposer_index,
            parent_root: H256::from_slice(parent_root.as_slice()),
            state_root: H256::from_slice(state_root.as_slice()),
            body_root: H256::from_slice(body_root.as_slice()),
        }
    }
}

impl TryFrom<BlsToExecutionChange> for types::BlsToExecutionChange {
    type Error = ProtosError;

    fn try_from(
        BlsToExecutionChange {
            validator_index,
            from_bls_pub_key,
            to_execution_address,
        }: BlsToExecutionChange,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            validator_index,
            from_bls_pubkey: bls::generics::GenericPublicKeyBytes::deserialize(
                from_bls_pub_key.as_slice(),
            )
            .map_err(|e| ProtosError::Bls(format!("{e:?}")))?,
            to_execution_address: Address::from_slice(to_execution_address.as_slice()),
        })
    }
}

impl From<Checkpoint> for types::Checkpoint {
    fn from(Checkpoint { epoch, root }: Checkpoint) -> Self {
        Self {
            epoch: epoch.into(),
            root: H256::from_slice(root.as_slice()),
        }
    }
}

impl<E: EthSpec> TryFrom<DenebExecutionPayload> for types::ExecutionPayloadDeneb<E> {
    type Error = ProtosError;

    fn try_from(
        DenebExecutionPayload {
            parent_hash,
            fee_recipient,
            state_root,
            receipts_root,
            logs_bloom,
            prev_randao,
            block_number,
            gas_limit,
            gas_used,
            timestamp,
            extra_data,
            base_fee_per_gas,
            block_hash,
            transactions,
            withdrawals,
            blob_gas_used,
            excess_blob_gas,
        }: DenebExecutionPayload,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            parent_hash: ExecutionBlockHash::from_root(H256::from_slice(parent_hash.as_slice())),
            fee_recipient: Address::from_slice(fee_recipient.as_slice()),
            state_root: H256::from_slice(state_root.as_slice()),
            receipts_root: H256::from_slice(receipts_root.as_slice()),
            logs_bloom: FixedVector::from(logs_bloom),
            prev_randao: H256::from_slice(prev_randao.as_slice()),
            block_number,
            gas_limit,
            gas_used,
            timestamp: timestamp
                .as_ref()
                .ok_or(ProtosError::BlockConversionError)?
                .seconds as u64,
            extra_data: extra_data.into(),
            base_fee_per_gas: U256::from_big_endian(base_fee_per_gas.as_slice()),
            block_hash: ExecutionBlockHash(H256::from_slice(block_hash.as_slice())),
            transactions: transactions
                .into_iter()
                .map(Into::into)
                .collect::<Vec<_>>()
                .into(),
            withdrawals: withdrawals
                .into_iter()
                .map(Into::into)
                .collect::<Vec<_>>()
                .into(),
            blob_gas_used,
            excess_blob_gas,
        })
    }
}

impl TryFrom<Deposit> for types::Deposit {
    type Error = ProtosError;

    fn try_from(Deposit { proof, data }: Deposit) -> Result<Self, Self::Error> {
        Ok(Self {
            proof: proof
                .into_iter()
                .map(|v| H256::from_slice(v.as_slice()))
                .collect::<Vec<_>>()
                .into(),
            data: data.ok_or(ProtosError::NullDepositData)?.try_into()?,
        })
    }
}

impl TryFrom<DepositData> for types::DepositData {
    type Error = ProtosError;

    fn try_from(
        DepositData {
            public_key,
            withdrawal_credentials,
            gwei,
            signature,
        }: DepositData,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            pubkey: bls::generics::GenericPublicKeyBytes::deserialize(public_key.as_slice())
                .map_err(|e| ProtosError::Bls(format!("{:?}", e)))?,
            withdrawal_credentials: H256::from_slice(withdrawal_credentials.as_slice()),
            amount: gwei,
            signature: bls::generics::GenericSignatureBytes::deserialize(signature.as_slice())
                .map_err(|e| ProtosError::Bls(format!("{:?}", e)))?,
        })
    }
}

impl From<Eth1Data> for types::Eth1Data {
    fn from(
        Eth1Data {
            deposit_root,
            deposit_count,
            block_hash,
        }: Eth1Data,
    ) -> Self {
        Self {
            deposit_root: H256::from_slice(deposit_root.as_slice()),
            deposit_count,
            block_hash: H256::from_slice(block_hash.as_slice()),
        }
    }
}

impl<E: EthSpec> TryFrom<IndexedAttestation> for types::IndexedAttestationBase<E> {
    type Error = ProtosError;

    fn try_from(
        IndexedAttestation {
            attesting_indices,
            data,
            signature,
        }: IndexedAttestation,
    ) -> Result<Self, Self::Error> {
        Ok(IndexedAttestationBase {
            attesting_indices: attesting_indices.into(),
            data: data
                .ok_or(ProtosError::NullIndexedAttestationData)?
                .try_into()?,
            signature: bls::generics::GenericAggregateSignature::deserialize(signature.as_slice())
                .map_err(|e| ProtosError::Bls(format!("{:?}", e)))?,
        })
    }
}

impl TryFrom<ProposerSlashing> for types::ProposerSlashing {
    type Error = ProtosError;

    fn try_from(
        ProposerSlashing {
            signed_header_1,
            signed_header_2,
        }: ProposerSlashing,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            signed_header_1: signed_header_1.ok_or(ProtosError::NullSigner)?.try_into()?,
            signed_header_2: signed_header_2.ok_or(ProtosError::NullSigner)?.try_into()?,
        })
    }
}

impl TryFrom<SignedBeaconBlockHeader> for types::SignedBeaconBlockHeader {
    type Error = ProtosError;

    fn try_from(
        SignedBeaconBlockHeader { message, signature }: SignedBeaconBlockHeader,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            message: message
                .ok_or(ProtosError::NullSignedBeaconBlockHeaderMessage)?
                .into(),
            signature: bls::generics::GenericSignature::deserialize(signature.as_slice())
                .map_err(|e| ProtosError::Bls(format!("{:?}", e)))?,
        })
    }
}

impl TryFrom<SignedBlsToExecutionChange> for types::SignedBlsToExecutionChange {
    type Error = ProtosError;

    fn try_from(
        SignedBlsToExecutionChange { message, signature }: SignedBlsToExecutionChange,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            message: message
                .ok_or(ProtosError::NullBlsToExecutionChange)?
                .try_into()?,
            signature: bls::generics::GenericSignature::deserialize(signature.as_slice())
                .expect("Failed to deserialize signature"),
        })
    }
}

impl TryFrom<SignedVoluntaryExit> for types::SignedVoluntaryExit {
    type Error = ProtosError;

    fn try_from(
        SignedVoluntaryExit { message, signature }: SignedVoluntaryExit,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            message: message.ok_or(ProtosError::NullVoluntaryExit)?.into(),
            signature: bls::generics::GenericSignature::deserialize(signature.as_slice())
                .map_err(|e| ProtosError::Bls(format!("{:?}", e)))?,
        })
    }
}

impl TryFrom<SingleBlockResponse> for Block {
    type Error = ProtosError;

    fn try_from(response: SingleBlockResponse) -> Result<Self, Self::Error> {
        let any = response.block.ok_or(ProtosError::NullBlock)?;
        let block = Block::decode(any.value.as_ref())?;
        Ok(block)
    }
}

impl TryFrom<Response> for Block {
    type Error = ProtosError;

    fn try_from(response: Response) -> Result<Self, Self::Error> {
        let any = response.block.ok_or(ProtosError::NullBlock)?;
        let block = Block::decode(any.value.as_ref())?;
        Ok(block)
    }
}

impl<E: EthSpec> TryFrom<SyncAggregate> for types::SyncAggregate<E> {
    type Error = ProtosError;

    fn try_from(
        SyncAggregate {
            sync_commitee_bits,
            sync_comittee_signature,
        }: SyncAggregate,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            sync_committee_bits: Bitfield::<Fixed<<E as EthSpec>::SyncCommitteeSize>>::from_bytes(
                sync_commitee_bits.as_slice().into(),
            )
            .map_err(|e| ProtosError::SszTypesError(format!("{:?}", e)))?,
            sync_committee_signature: bls::generics::GenericAggregateSignature::deserialize(
                sync_comittee_signature.as_slice(),
            )
            .map_err(|e| ProtosError::Bls(format!("{:?}", e)))?,
        })
    }
}

impl From<VoluntaryExit> for types::VoluntaryExit {
    fn from(
        VoluntaryExit {
            epoch,
            validator_index,
        }: VoluntaryExit,
    ) -> Self {
        Self {
            epoch: epoch.into(),
            validator_index,
        }
    }
}

impl From<Withdrawal> for types::Withdrawal {
    fn from(
        Withdrawal {
            withdrawal_index,
            validator_index,
            address,
            gwei,
        }: Withdrawal,
    ) -> Self {
        Self {
            index: withdrawal_index,
            validator_index,
            address: Address::from_slice(address.as_slice()),
            amount: gwei,
        }
    }
}

impl TryFrom<DenebBody> for types::BeaconBlockBodyDeneb<MainnetEthSpec> {
    type Error = ProtosError;

    fn try_from(
        DenebBody {
            rando_reveal,
            eth1_data,
            graffiti,
            proposer_slashings,
            attester_slashings,
            attestations,
            deposits,
            voluntary_exits,
            sync_aggregate,
            execution_payload,
            bls_to_execution_changes,
            blob_kzg_commitments,
            // Blobs not included.
            ..
        }: DenebBody,
    ) -> Result<Self, Self::Error> {
        let beacon_block_body = BeaconBlockBodyDeneb {
            randao_reveal: bls::generics::GenericSignature::deserialize(&rando_reveal)
                .map_err(|e| ProtosError::Bls(format!("{:?}", e)))?,
            eth1_data: eth1_data
                .map(|eth1_data| eth1_data.into())
                .unwrap_or_default(),
            graffiti: Graffiti::from(
                <[u8; GRAFFITI_BYTES_LEN]>::try_from(graffiti.as_slice())
                    .map_err(|_| ProtosError::GraffitiInvalid)?,
            ),
            proposer_slashings: proposer_slashings
                .into_iter()
                .map(|proposer_slashing| proposer_slashing.try_into())
                .collect::<Result<Vec<_>, _>>()?
                .into(),
            attester_slashings: attester_slashings
                .into_iter()
                .map(|attester_slashing| attester_slashing.try_into())
                .collect::<Result<Vec<_>, _>>()?
                .into(),
            attestations: attestations
                .into_iter()
                .map(|attestation| attestation.try_into())
                .collect::<Result<Vec<_>, _>>()?
                .into(),
            deposits: deposits
                .into_iter()
                .map(|deposit| deposit.try_into())
                .collect::<Result<Vec<_>, _>>()?
                .into(),
            voluntary_exits: voluntary_exits
                .into_iter()
                .map(|voluntary_exit| voluntary_exit.try_into())
                .collect::<Result<Vec<_>, _>>()?
                .into(),
            sync_aggregate: sync_aggregate
                .map(|sync_aggregate| sync_aggregate.try_into())
                .transpose()?
                .unwrap_or_else(types::SyncAggregate::new),
            execution_payload: execution_payload
                .ok_or(ProtosError::NullExecutionPayload)
                .and_then(types::ExecutionPayloadDeneb::try_from)?
                .into(),
            bls_to_execution_changes: bls_to_execution_changes
                .into_iter()
                .map(|bls_to_execution_change| bls_to_execution_change.try_into())
                .collect::<Result<Vec<_>, _>>()?
                .into(),
            blob_kzg_commitments: blob_kzg_commitments
                .into_iter()
                .map(|blob_kzg_commitment| {
                    <[u8; 48]>::try_from(blob_kzg_commitment.as_slice())
                        .map(types::KzgCommitment)
                        .map_err(|_| ProtosError::KzgCommitmentInvalid)
                })
                .collect::<Result<Vec<_>, _>>()?
                .into(),
        };
        Ok(beacon_block_body)
    }
}

impl TryFrom<crate::beacon_v1::block::Body> for types::BeaconBlockBodyDeneb<MainnetEthSpec> {
    type Error = ProtosError;

    fn try_from(body: crate::beacon_v1::block::Body) -> Result<Self, Self::Error> {
        match body {
            crate::beacon_v1::block::Body::Deneb(deneb) => Ok(deneb.try_into()?),
            _ => panic!("Invalid body type"),
        }
    }
}

impl TryFrom<Block> for types::BeaconBlock<MainnetEthSpec> {
    type Error = ProtosError;

    fn try_from(
        Block {
            slot,
            proposer_index,
            parent_root,
            state_root,
            body,
            ..
        }: Block,
    ) -> Result<Self, Self::Error> {
        Ok(Self::Deneb(types::BeaconBlockDeneb {
            slot: slot.into(),
            proposer_index,
            parent_root: H256::from_slice(parent_root.as_slice()),
            state_root: H256::from_slice(state_root.as_slice()),
            body: body.ok_or(ProtosError::BlockConversionError)?.try_into()?,
        }))
    }
}
