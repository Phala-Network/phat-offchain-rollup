use ink::prelude::vec::Vec;
use openbrush::contracts::access_control::{AccessControlError, RoleType};
use openbrush::traits::{AccountId, Hash};
use scale::{Decode, Encode};

pub type Nonce = u128;

pub const MANAGER_ROLE: RoleType = ink::selector_id!("MANAGER_ROLE");

#[derive(Debug, Eq, PartialEq, Encode, Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum MetaTxError {
    NonceTooLow,
    IncorrectSignature,
    PublicKeyNotMatch,
    PublicKeyNotRegistered,
    AccessControlError(AccessControlError),
}

/// convertor from AccessControlError to MetaTxError
impl From<AccessControlError> for MetaTxError {
    fn from(error: AccessControlError) -> Self {
        MetaTxError::AccessControlError(error)
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Encode, Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub struct ForwardRequest {
    pub from: AccountId,
    pub nonce: Nonce,
    pub data: Vec<u8>,
}

pub type PrepareResult = (ForwardRequest, Hash);

#[openbrush::trait_definition]
pub trait MetaTxReceiver {
    fn _get_nonce(&self, from: AccountId) -> Nonce;

    #[ink(message)]
    fn get_ecdsa_public_key(&self, from: AccountId) -> [u8; 33];

    #[ink(message)]
    fn register_ecdsa_public_key(
        &mut self,
        from: AccountId,
        ecdsa_public_key: [u8; 33],
    ) -> Result<(), MetaTxError>;

    #[ink(message)]
    fn prepare(&self, from: AccountId, data: Vec<u8>) -> Result<PrepareResult, MetaTxError>;

    fn _verify(&self, request: &ForwardRequest, signature: &[u8; 65]) -> Result<(), MetaTxError>;

    fn _use_meta_tx(
        &mut self,
        request: &ForwardRequest,
        signature: &[u8; 65],
    ) -> Result<(), MetaTxError>;
}
