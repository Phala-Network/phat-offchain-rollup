use crate::traits::rollup_anchor::{RollupAnchor, RollupAnchorError, RolupCondEqMethodParams};
use ink::env::hash::{Blake2x256, HashOutput};
use ink::prelude::vec::Vec;
use openbrush::contracts::access_control::{self, AccessControlError, RoleType};
use openbrush::storage::Mapping;
use openbrush::traits::{AccountId, Hash, Storage};

pub type Nonce = u128;
type NonceAndEcdsaPk = (Nonce, Vec<u8>);
pub type PrepareResult = (ForwardRequest, Hash);
pub type MetatTxRolupCondEqMethodParams = (ForwardRequest, [u8; 65]);

pub const MANAGER_ROLE: RoleType = ink::selector_id!("MANAGER_ROLE");

#[derive(Debug, Eq, PartialEq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum MetaTxError {
    NonceTooLow,
    IncorrectSignature,
    PublicKeyNotMatch,
    PublicKeyNotRegistered,
    AccessControlError(AccessControlError),
    RollupAnchorError(RollupAnchorError),
}

/// convertor from AccessControlError to MetaTxError
impl From<AccessControlError> for MetaTxError {
    fn from(error: AccessControlError) -> Self {
        MetaTxError::AccessControlError(error)
    }
}

/// convertor from RollupAnchorError to MetaTxError
impl From<RollupAnchorError> for MetaTxError {
    fn from(error: RollupAnchorError) -> Self {
        MetaTxError::RollupAnchorError(error)
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
    nonces_and_ecdsa_public_key: Mapping<AccountId, NonceAndEcdsaPk>,
}

#[openbrush::trait_definition]
pub trait MetaTxReceiver: Storage<Data> + access_control::Internal + RollupAnchor {
    #[ink(message)]
    fn get_ecdsa_public_key(&self, from: AccountId) -> [u8; 33] {
        match self.data::<Data>().nonces_and_ecdsa_public_key.get(&from) {
            None => [0; 33],
            Some((_, p)) => p.try_into().unwrap_or([0; 33]),
        }
    }

    #[ink(message)]
    #[openbrush::modifiers(access_control::only_role(MANAGER_ROLE))]
    fn register_ecdsa_public_key(
        &mut self,
        from: AccountId,
        ecdsa_public_key: [u8; 33],
    ) -> Result<(), MetaTxError> {
        match self.data::<Data>().nonces_and_ecdsa_public_key.get(&from) {
            None => self
                .data::<Data>()
                .nonces_and_ecdsa_public_key
                .insert(&from, &(0, ecdsa_public_key.into())),
            Some((n, _)) => self
                .data::<Data>()
                .nonces_and_ecdsa_public_key
                .insert(&from, &(n, ecdsa_public_key.into())),
        }
        Ok(())
    }

    #[ink(message)]
    fn prepare(
        &self,
        from: AccountId,
        data: Vec<u8>,
    ) -> Result<(ForwardRequest, Hash), MetaTxError> {
        let nonce = self.get_nonce(from);

        let request = ForwardRequest { from, nonce, data };
        let mut hash = <Blake2x256 as HashOutput>::Type::default();
        ink::env::hash_encoded::<Blake2x256, _>(&request, &mut hash);

        Ok((request, hash.into()))
    }

    fn get_nonce(&self, from: AccountId) -> Nonce {
        self.data::<Data>()
            .nonces_and_ecdsa_public_key
            .get(&from)
            .map(|(n, _)| n)
            .unwrap_or(0)
    }

    fn verify(&self, request: &ForwardRequest, signature: &[u8; 65]) -> Result<(), MetaTxError> {
        let (nonce_from, ecdsa_public_key) = match self
            .data::<Data>()
            .nonces_and_ecdsa_public_key
            .get(&request.from)
        {
            Some((n, p)) => (n, p),
            _ => return Err(MetaTxError::PublicKeyNotRegistered),
        };
        let ecdsa_public_key: [u8; 33] = ecdsa_public_key
            .try_into()
            .map_err(|_| MetaTxError::PublicKeyNotRegistered)?;

        if request.nonce < nonce_from {
            return Err(MetaTxError::NonceTooLow);
        }

        let mut hash = <Blake2x256 as HashOutput>::Type::default();
        ink::env::hash_encoded::<Blake2x256, _>(&request, &mut hash);

        // at the moment we can only verify ecdsa signatures
        let mut public_key = [0u8; 33];
        ink::env::ecdsa_recover(signature, &hash, &mut public_key)
            .map_err(|_| MetaTxError::IncorrectSignature)?;

        if public_key != ecdsa_public_key {
            return Err(MetaTxError::PublicKeyNotMatch);
        }
        Ok(())
    }

    fn use_meta_tx(
        &mut self,
        request: &ForwardRequest,
        signature: &[u8; 65],
    ) -> Result<(), MetaTxError> {
        // verify the signature
        self.verify(request, signature)?;
        // update the nonce
        match self
            .data::<Data>()
            .nonces_and_ecdsa_public_key
            .get(&request.from)
        {
            Some((_, p)) => self
                .data::<Data>()
                .nonces_and_ecdsa_public_key
                .insert(&request.from, &(request.nonce + 1, p)),
            None => return Err(MetaTxError::PublicKeyNotRegistered),
        }
        Ok(())
    }

    #[ink(message)]
    fn meta_tx_rollup_cond_eq(
        &mut self,
        request: ForwardRequest,
        signature: [u8; 65],
    ) -> Result<bool, MetaTxError> {
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

#[macro_export]
macro_rules! use_meta_tx {
    ($metaTxReceiver:ident, $request:ident, $signature:ident) => {{
        $metaTxReceiver._use_meta_tx(&$request, &$signature)?
    }};
}
