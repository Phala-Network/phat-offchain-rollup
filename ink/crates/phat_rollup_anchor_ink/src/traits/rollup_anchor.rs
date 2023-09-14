use crate::traits::kv_store;
use crate::traits::message_queue::{self, MessageQueueError};
use ink::prelude::vec::Vec;
use kv_session::traits::{Key, QueueIndex, Value};
use openbrush::contracts::access_control::{self, AccessControlError, RoleType};
use openbrush::traits::AccountId;

pub const ATTESTOR_ROLE: RoleType = ink::selector_id!("ATTESTOR_ROLE");

pub trait MessageHandler {
    fn on_message_received(&mut self, action: Vec<u8>) -> Result<(), RollupAnchorError>;
}

#[derive(scale::Encode, scale::Decode, Debug, Eq, PartialEq, Clone)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum HandleActionInput {
    Reply(Vec<u8>),
    SetQueueHead(QueueIndex),
    GrantAttestor(AccountId),
    RevokeAttestor(AccountId),
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

pub type RollupCondEqMethodParams = (
    Vec<(Key, Option<Value>)>,
    Vec<(Key, Option<Value>)>,
    Vec<HandleActionInput>,
);

#[openbrush::trait_definition]
pub trait RollupAnchor:
    MessageHandler
    + kv_store::KvStore
    + message_queue::MessageQueue
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
    ) -> Result<(), RollupAnchorError> {
        self.inner_rollup_cond_eq(conditions, updates, actions)
    }

    fn check_attestor_role(&self, attestor: AccountId) -> Result<(), RollupAnchorError> {
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
    ) -> Result<(), RollupAnchorError> {
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

        Ok(())
    }

    fn handle_action(&mut self, input: HandleActionInput) -> Result<(), RollupAnchorError> {
        match input {
            HandleActionInput::Reply(action) => self.on_message_received(action)?,
            HandleActionInput::SetQueueHead(id) => self.pop_to(id)?,
            HandleActionInput::GrantAttestor(address) => {
                self.grant_role(ATTESTOR_ROLE, Some(address))?
            }
            HandleActionInput::RevokeAttestor(address) => {
                self.revoke_role(ATTESTOR_ROLE, Some(address))?
            }
        }

        Ok(())
    }
}
