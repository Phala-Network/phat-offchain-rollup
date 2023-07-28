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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::test_contract::MyContract;
    use crate::traits::message_queue::MessageQueue;
    use openbrush::contracts::access_control::AccessControl;
    use openbrush::test_utils::{accounts, change_caller};
    use scale::Encode;

    #[ink::test]
    fn test_conditions() {
        let accounts = accounts();
        let mut contract = MyContract::new(accounts.alice);

        // no condition, no update, no action => it should work
        assert_eq!(contract.rollup_cond_eq(vec![], vec![], vec![]), Ok(true));

        // test with correct condition
        let conditions = vec![(123u8.encode(), None)];
        assert_eq!(
            contract.rollup_cond_eq(conditions, vec![], vec![]),
            Ok(true)
        );

        // update a value
        let updates = vec![(123u8.encode(), Some(456u128.encode()))];
        assert_eq!(contract.rollup_cond_eq(vec![], updates, vec![]), Ok(true));

        // test with the correct condition
        let conditions = vec![(123u8.encode(), Some(456u128.encode()))];
        assert_eq!(
            contract.rollup_cond_eq(conditions, vec![], vec![]),
            Ok(true)
        );

        // test with incorrect condition (incorrect value)
        let conditions = vec![(123u8.encode(), Some(789u128.encode()))];
        assert_eq!(
            contract.rollup_cond_eq(conditions, vec![], vec![]),
            Err(RollupAnchorError::ConditionNotMet)
        );

        // test with incorrect condition (incorrect value)
        let conditions = vec![(123u8.encode(), None)];
        assert_eq!(
            contract.rollup_cond_eq(conditions, vec![], vec![]),
            Err(RollupAnchorError::ConditionNotMet)
        );

        // test with incorrect condition (incorrect key)
        let conditions = vec![
            (123u8.encode(), Some(456u128.encode())),
            (124u8.encode(), Some(456u128.encode())),
        ];
        assert_eq!(
            contract.rollup_cond_eq(conditions, vec![], vec![]),
            Err(RollupAnchorError::ConditionNotMet)
        );
    }

    #[ink::test]
    fn test_actions_missing_data() {
        let accounts = accounts();
        let mut contract = MyContract::new(accounts.alice);

        let actions = vec![HandleActionInput {
            action_type: ACTION_SET_QUEUE_HEAD,
            id: None, // missing data
            action: None,
            address: None,
        }];
        assert_eq!(
            contract.rollup_cond_eq(vec![], vec![], actions),
            Err(RollupAnchorError::MissingData)
        );

        let actions = vec![HandleActionInput {
            action_type: ACTION_REPLY,
            id: None,
            action: None, // missing data
            address: None,
        }];
        assert_eq!(
            contract.rollup_cond_eq(vec![], vec![], actions),
            Err(RollupAnchorError::MissingData)
        );
    }

    #[ink::test]
    fn test_action_pop_to() {
        let accounts = accounts();
        let mut contract = MyContract::new(accounts.alice);

        // no condition, no update, no action
        let mut actions = Vec::new();
        actions.push(HandleActionInput {
            action_type: ACTION_SET_QUEUE_HEAD,
            id: Some(2),
            action: None,
            address: None,
        });

        assert_eq!(
            contract.rollup_cond_eq(vec![], vec![], actions.clone()),
            Err(RollupAnchorError::MessageQueueError(
                MessageQueueError::InvalidPopTarget
            ))
        );

        let message = 4589u16;
        contract.push_message(&message).unwrap();
        contract.push_message(&message).unwrap();

        assert_eq!(contract.rollup_cond_eq(vec![], vec![], actions), Ok(true));
    }

    #[ink::test]
    fn test_action_reply() {
        let accounts = accounts();
        let mut contract = MyContract::new(accounts.alice);

        let actions = vec![HandleActionInput {
            action_type: ACTION_REPLY,
            id: Some(2),
            action: Some(012u8.encode()),
            address: None,
        }];

        assert_eq!(contract.rollup_cond_eq(vec![], vec![], actions), Ok(true));
    }

    #[ink::test]
    fn test_grant_role() {
        let accounts = accounts();
        let mut contract = MyContract::new(accounts.alice);

        // bob cannot grant the role
        change_caller(accounts.bob);
        assert_eq!(
            Err(access_control::AccessControlError::MissingRole),
            contract.grant_role(ATTESTOR_ROLE, Some(accounts.bob))
        );

        // alice, the owner, can do it
        change_caller(accounts.alice);
        assert_eq!(
            Ok(()),
            contract.grant_role(ATTESTOR_ROLE, Some(accounts.bob))
        );
    }

    #[ink::test]
    fn test_rollup_cond_eq_role_attestor() {
        let accounts = accounts();
        let mut contract = MyContract::new(accounts.alice);

        change_caller(accounts.bob);

        assert_eq!(
            Err(RollupAnchorError::AccessControlError(
                access_control::AccessControlError::MissingRole
            )),
            contract.rollup_cond_eq(vec![], vec![], vec![])
        );

        change_caller(accounts.alice);
        contract
            .grant_role(ATTESTOR_ROLE, Some(accounts.bob))
            .expect("Error when grant the role Attestor");

        change_caller(accounts.bob);
        assert_eq!(Ok(true), contract.rollup_cond_eq(vec![], vec![], vec![]));
    }
}
