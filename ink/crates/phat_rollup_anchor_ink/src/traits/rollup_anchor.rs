pub use crate::traits::message_queue::MessageQueueError;
use crate::traits::meta_transaction::{ForwardRequest, MetaTxError};
use ink::prelude::vec::Vec;
pub use kv_session::traits::{Key, QueueIndex, Value};
use openbrush::contracts::access_control::{AccessControlError, RoleType};
use openbrush::traits::AccountId;

pub const ATTESTOR_ROLE: RoleType = ink::selector_id!("ATTESTOR_ROLE");

pub const ACTION_REPLY: u8 = 0;
pub const ACTION_SET_QUEUE_HEAD: u8 = 1;
pub const ACTION_GRANT_ATTESTOR: u8 = 10;
pub const ACTION_REVOKE_ATTESTOR: u8 = 11;

#[derive(scale::Encode, scale::Decode, Debug, Eq, PartialEq, Clone)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub struct HandleActionInput {
    pub action_type: u8,
    pub action: Option<Vec<u8>>,
    pub address: Option<AccountId>,
    pub id: Option<QueueIndex>,
}

#[derive(Debug, Eq, PartialEq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum RollupAnchorError {
    FailedToDecode,
    UnsupportedAction,
    ConditionNotMet,
    MissingData,
    MessageQueueError(MessageQueueError),
    AccessControlError(AccessControlError),
    MetaTxError(MetaTxError),
}

/// convertor from AccessControlError to RollupAnchorError
impl From<AccessControlError> for RollupAnchorError {
    fn from(error: AccessControlError) -> Self {
        RollupAnchorError::AccessControlError(error)
    }
}

/// convertor from MessageQueueError to RollupAnchorError
impl From<MessageQueueError> for RollupAnchorError {
    fn from(error: MessageQueueError) -> Self {
        RollupAnchorError::MessageQueueError(error)
    }
}

/// convertor from MetaTxError to RollupAnchorError
impl From<MetaTxError> for RollupAnchorError {
    fn from(error: MetaTxError) -> Self {
        RollupAnchorError::MetaTxError(error)
    }
}

#[openbrush::trait_definition]
pub trait RollupAnchor {
    #[ink(message)]
    fn rollup_cond_eq(
        &mut self,
        conditions: Vec<(Key, Option<Value>)>,
        updates: Vec<(Key, Option<Value>)>,
        actions: Vec<HandleActionInput>,
    ) -> Result<bool, RollupAnchorError>;

    #[ink(message)]
    fn meta_tx_rollup_cond_eq(
        &mut self,
        request: ForwardRequest,
        signature: [u8; 65],
    ) -> Result<bool, RollupAnchorError>;
}

pub trait Internal {
    fn _rollup_cond_eq(
        &mut self,
        conditions: Vec<(Key, Option<Value>)>,
        updates: Vec<(Key, Option<Value>)>,
        actions: Vec<HandleActionInput>,
    ) -> Result<bool, RollupAnchorError>;

    fn _handle_action(&mut self, input: HandleActionInput) -> Result<(), RollupAnchorError>;
}

pub trait MessageHandler {
    fn _on_message_received(&mut self, action: Vec<u8>) -> Result<(), RollupAnchorError>;
}

pub trait EventBroadcaster {
    fn _emit_event_meta_tx_decoded(&self);
}
