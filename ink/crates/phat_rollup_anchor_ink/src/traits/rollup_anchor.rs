use crate::traits::kv_store;
use crate::traits::message_queue::{self, MessageQueueError};
use ink::prelude::vec::Vec;
use kv_session::traits::{Key, QueueIndex, Value};
use openbrush::contracts::access_control::{self, AccessControlError, RoleType};
use openbrush::traits::AccountId;

pub const ATTESTOR_ROLE: RoleType = ink::selector_id!("ATTESTOR_ROLE");

pub const ACTION_REPLY: u8 = 0;
pub const ACTION_SET_QUEUE_HEAD: u8 = 1;
pub const ACTION_GRANT_ATTESTOR: u8 = 10;
pub const ACTION_REVOKE_ATTESTOR: u8 = 11;

pub trait MessageHandler {
    fn on_message_received(&mut self, action: Vec<u8>) -> Result<(), RollupAnchorError>;
}

pub trait EventBroadcaster {
    fn emit_event_meta_tx_decoded(&self);
}

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

pub type RolupCondEqMethodParams = (
    Vec<(Key, Option<Value>)>,
    Vec<(Key, Option<Value>)>,
    Vec<HandleActionInput>,
);

#[openbrush::trait_definition]
pub trait RollupAnchor:
    EventBroadcaster
    + MessageHandler
    + kv_store::KvStore
    + message_queue::MessageQueue
    //+ meta_transaction::MetaTxReceiver
    + access_control::AccessControl
    + access_control::Internal
{
    #[ink(message)]
    #[openbrush::modifiers(access_control::only_role(ATTESTOR_ROLE))]
    fn rollup_cond_eq(
        &mut self,
        conditions: Vec<(Key, Option<Value>)>,
        updates: Vec<(Key, Option<Value>)>,
        actions: Vec<HandleActionInput>,
    ) -> Result<bool, RollupAnchorError> {
        self.inner_rollup_cond_eq(conditions, updates, actions)
    }

    fn check_attestor_role(
        &self,
        attestor: AccountId,
    ) -> Result<(), RollupAnchorError> {

        if !self.has_role(ATTESTOR_ROLE, Some(attestor)) {
            return Err(RollupAnchorError::AccessControlError(
                access_control::AccessControlError::MissingRole,
            ));
        }

        Ok(())
    }

    fn inner_rollup_cond_eq(
        &mut self,
        conditions: Vec<(Key, Option<Value>)>,
        updates: Vec<(Key, Option<Value>)>,
        actions: Vec<HandleActionInput>,
    ) -> Result<bool, RollupAnchorError> {
        // check the conditions
        for cond in conditions {
            let key = cond.0;
            let current_value = self.inner_get_value(&key);
            let expected_value = cond.1;
            match (current_value, expected_value) {
                (None, None) => {}
                (Some(v1), Some(v2)) => {
                    if v1.ne(&v2) {
                        // condition is not met
                        return Err(RollupAnchorError::ConditionNotMet);
                    }
                }
                (_, _) => return Err(RollupAnchorError::ConditionNotMet),
            }
        }

        // apply the updates
        for update in updates {
            self.set_value(&update.0, update.1.as_ref());
        }

        // apply the actions
        for action in actions {
            self.handle_action(action)?;
        }

        Ok(true)
    }

    fn handle_action(&mut self, input: HandleActionInput) -> Result<(), RollupAnchorError> {
        match input.action_type {
            ACTION_REPLY => {
                self.on_message_received(input.action.ok_or(RollupAnchorError::MissingData)?)?
            }
            ACTION_SET_QUEUE_HEAD => {
                self.pop_to(input.id.ok_or(RollupAnchorError::MissingData)?)?
            }
            ACTION_GRANT_ATTESTOR => self.grant_role(
                ATTESTOR_ROLE,
                Some(input.address.ok_or(RollupAnchorError::MissingData)?),
            )?,
            ACTION_REVOKE_ATTESTOR => self.revoke_role(
                ATTESTOR_ROLE,
                Some(input.address.ok_or(RollupAnchorError::MissingData)?),
            )?,
            _ => return Err(RollupAnchorError::UnsupportedAction),
        }

        Ok(())
    }
}
