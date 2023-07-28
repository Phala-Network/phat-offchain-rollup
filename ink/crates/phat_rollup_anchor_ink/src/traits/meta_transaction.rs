use crate::traits::rollup_anchor::{RollupAnchor, RollupAnchorError, RolupCondEqMethodParams};
use ink::env::hash::{Blake2x256, HashOutput};
use ink::prelude::vec::Vec;
use openbrush::contracts::access_control::{self, AccessControlError, RoleType};
use openbrush::storage::Mapping;
use openbrush::traits::{AccountId, Hash, Storage};

pub type Nonce = u128;
pub type PrepareResult = (ForwardRequest, Hash);
pub type MetatTxRolupCondEqMethodParams = (ForwardRequest, [u8; 65]);

pub const MANAGER_ROLE: RoleType = ink::selector_id!("MANAGER_ROLE");

#[derive(Debug, Eq, PartialEq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum MetaTransactionError {
    NonceTooLow,
    IncorrectSignature,
    PublicKeyNotMatch,
    PublicKeyNotRegistered,
    AccessControlError(AccessControlError),
    RollupAnchorError(RollupAnchorError),
}

/// convertor from AccessControlError to MetaTxError
impl From<AccessControlError> for MetaTransactionError {
    fn from(error: AccessControlError) -> Self {
        MetaTransactionError::AccessControlError(error)
    }
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
    pub nonce: Nonce,
    pub data: Vec<u8>,
}

#[derive(Default, Debug)]
#[openbrush::storage_item]
pub struct Data {
    nonces: Mapping<AccountId, Nonce>,
    ecdsa_public_keys: Mapping<AccountId, Vec<u8>>,
}

#[openbrush::trait_definition]
pub trait MetaTransaction:
    Storage<Data> + EventBroadcaster + access_control::Internal + RollupAnchor
{
    #[ink(message)]
    fn get_ecdsa_public_key(&self, from: AccountId) -> [u8; 33] {
        self.data::<Data>()
            .ecdsa_public_keys
            .get(&from)
            .map(|k| k.try_into().unwrap_or([0; 33]))
            .unwrap_or([0; 33])
    }

    #[ink(message)]
    #[openbrush::modifiers(access_control::only_role(MANAGER_ROLE))]
    fn register_ecdsa_public_key(
        &mut self,
        from: AccountId,
        ecdsa_public_key: [u8; 33],
    ) -> Result<(), MetaTransactionError> {
        self.data::<Data>()
            .ecdsa_public_keys
            .insert(&from, &ecdsa_public_key.into());
        Ok(())
    }

    #[ink(message)]
    fn prepare(
        &self,
        from: AccountId,
        data: Vec<u8>,
    ) -> Result<(ForwardRequest, Hash), MetaTransactionError> {
        let nonce = self.get_nonce(from);

        let request = ForwardRequest { from, nonce, data };
        let mut hash = <Blake2x256 as HashOutput>::Type::default();
        ink::env::hash_encoded::<Blake2x256, _>(&request, &mut hash);

        Ok((request, hash.into()))
    }

    fn get_nonce(&self, from: AccountId) -> Nonce {
        self.data::<Data>().nonces.get(&from).unwrap_or(0)
    }

    fn verify(&self, request: &ForwardRequest, signature: &[u8; 65]) -> Result<(), MetaTransactionError> {
        let ecdsa_public_key : [u8; 33]  = self
            .data::<Data>()
            .ecdsa_public_keys
            .get(&request.from)
            .ok_or(MetaTransactionError::PublicKeyNotRegistered)?
            .try_into()
            .map_err(|_| MetaTransactionError::PublicKeyNotRegistered)?;

        let nonce_from = self.get_nonce(request.from);

        if request.nonce < nonce_from {
            return Err(MetaTransactionError::NonceTooLow);
        }

        let mut hash = <Blake2x256 as HashOutput>::Type::default();
        ink::env::hash_encoded::<Blake2x256, _>(&request, &mut hash);

        // at the moment we can only verify ecdsa signatures
        let mut public_key = [0u8; 33];
        ink::env::ecdsa_recover(signature, &hash, &mut public_key)
            .map_err(|_| MetaTransactionError::IncorrectSignature)?;

        if public_key != ecdsa_public_key {
            return Err(MetaTransactionError::PublicKeyNotMatch);
        }
        Ok(())
    }

    fn use_meta_tx(
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
    ) -> Result<bool, MetaTransactionError> {
        // check the signature
        self.use_meta_tx(&request, &signature)?;

        // check the attestor role
        self.check_attestor_role(request.from)?;

        // decode the data
        let data: RolupCondEqMethodParams = scale::Decode::decode(&mut request.data.as_slice())
            .map_err(|_| RollupAnchorError::FailedToDecode)?;

        // emit the event
        self.emit_event_meta_tx_decoded();

        // call the rollup
        let result = self.inner_rollup_cond_eq(data.0, data.1, data.2)?;

        Ok(result)
    }
}

pub trait EventBroadcaster {
    fn emit_event_meta_tx_decoded(&self);
}
