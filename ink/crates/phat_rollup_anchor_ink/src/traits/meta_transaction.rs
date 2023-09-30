use crate::traits::rollup_anchor::{RollupAnchor, RollupAnchorError, RollupCondEqMethodParams};
use ink::env::hash::{Blake2x256, HashOutput};
use ink::prelude::vec::Vec;
use openbrush::storage::Mapping;
use openbrush::traits::{AccountId, Hash, Storage};

pub type Nonce = u128;
pub type PrepareResult = (ForwardRequest, Hash);
pub type MetatTxRollupCondEqMethodParams = (ForwardRequest, [u8; 65]);

#[derive(Debug, Eq, PartialEq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum MetaTransactionError {
    InvalidDestination,
    NonceTooLow,
    IncorrectSignature,
    PublicKeyNotMatch,
    PublicKeyIncorrect,
    RollupAnchorError(RollupAnchorError),
}

/// convertor from RollupAnchorError to MetaTxError
impl From<RollupAnchorError> for MetaTransactionError {
    fn from(error: RollupAnchorError) -> Self {
        MetaTransactionError::RollupAnchorError(error)
    }
}

#[derive(Debug, Eq, PartialEq, Clone, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub struct ForwardRequest {
    pub from: AccountId,
    pub to: AccountId,
    pub nonce: Nonce,
    pub data: Vec<u8>,
}

#[derive(Default, Debug)]
#[openbrush::storage_item]
pub struct Data {
    nonces: Mapping<AccountId, Nonce>,
}

#[openbrush::trait_definition]
pub trait MetaTransaction: Storage<Data> + EventBroadcaster + RollupAnchor {
    #[ink(message)]
    fn prepare(
        &self,
        from: AccountId,
        data: Vec<u8>,
    ) -> Result<(ForwardRequest, Hash), MetaTransactionError> {
        let nonce = self.get_nonce(from);
        let to = Self::env().account_id();

        let request = ForwardRequest {
            from,
            to,
            nonce,
            data,
        };
        let mut hash = <Blake2x256 as HashOutput>::Type::default();
        ink::env::hash_encoded::<Blake2x256, _>(&request, &mut hash);

        Ok((request, hash.into()))
    }

    fn get_nonce(&self, from: AccountId) -> Nonce {
        self.data::<Data>().nonces.get(&from).unwrap_or(0)
    }

    fn verify(
        &self,
        request: &ForwardRequest,
        signature: &[u8; 65],
    ) -> Result<(), MetaTransactionError> {
        let to = Self::env().account_id();
        if request.to != to {
            return Err(MetaTransactionError::InvalidDestination);
        }

        let nonce_from = self.get_nonce(request.from);
        if request.nonce != nonce_from {
            return Err(MetaTransactionError::NonceTooLow);
        }

        // at the moment we can only verify ecdsa signatures
        let mut hash = <Blake2x256 as HashOutput>::Type::default();
        ink::env::hash_encoded::<Blake2x256, _>(&request, &mut hash);

        let mut public_key = [0u8; 33];
        ink::env::ecdsa_recover(signature, &hash, &mut public_key)
            .map_err(|_| MetaTransactionError::IncorrectSignature)?;

        if request.from != get_ecdsa_account_id(&public_key) {
            return Err(MetaTransactionError::PublicKeyNotMatch);
        }
        Ok(())
    }

    fn ensure_meta_tx_valid(
        &mut self,
        request: &ForwardRequest,
        signature: &[u8; 65],
    ) -> Result<(), MetaTransactionError> {
        // verify the signature
        self.verify(request, signature)?;
        // update the nonce
        let nonce = request.nonce + 1;
        self.data::<Data>().nonces.insert(&request.from, &nonce);
        Ok(())
    }

    #[ink(message)]
    fn meta_tx_rollup_cond_eq(
        &mut self,
        request: ForwardRequest,
        signature: [u8; 65],
    ) -> Result<(), MetaTransactionError> {
        // check the signature
        self.ensure_meta_tx_valid(&request, &signature)?;

        // check the attestor role
        self.check_attestor_role(request.from)?;

        // decode the data
        let data: RollupCondEqMethodParams = scale::Decode::decode(&mut request.data.as_slice())
            .map_err(|_| RollupAnchorError::FailedToDecode)?;

        // emit the event
        self.emit_event_meta_tx_decoded();

        // call the rollup
        self.inner_rollup_cond_eq(data.0, data.1, data.2)?;

        Ok(())
    }
}

pub trait EventBroadcaster {
    fn emit_event_meta_tx_decoded(&self);
}

/// Hashing function for bytes
fn hash_blake2b256(input: &[u8]) -> [u8; 32] {
    use ink::env::hash;
    let mut output = <hash::Blake2x256 as hash::HashOutput>::Type::default();
    ink::env::hash_bytes::<hash::Blake2x256>(input, &mut output);
    output
}

/// Converts a compressed ECDSA public key to AccountId
fn get_ecdsa_account_id(pub_key: &[u8; 33]) -> AccountId {
    AccountId::from(hash_blake2b256(pub_key))
}
