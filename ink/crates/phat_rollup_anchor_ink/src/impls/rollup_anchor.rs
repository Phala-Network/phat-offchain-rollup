use ink::prelude::vec::Vec;
use kv_session::traits::{Key, Value};
use openbrush::contracts::access_control;
use openbrush::traits::Storage;

use crate::traits::kv_store;
use crate::traits::message_queue;
use crate::traits::meta_transaction;
pub use crate::traits::rollup_anchor::{self, *};

pub type RolupCondEqMethodParams = (
    Vec<(Key, Option<Value>)>,
    Vec<(Key, Option<Value>)>,
    Vec<HandleActionInput>,
);
pub type MetatTxRolupCondEqMethodParams = (meta_transaction::ForwardRequest, [u8; 65]);

impl<T> RollupAnchor for T
where
    T: rollup_anchor::Internal,
    T: rollup_anchor::EventBroadcaster,
    T: Storage<access_control::Data>,
    T: access_control::AccessControl,
    T: meta_transaction::Internal,
{
    #[openbrush::modifiers(access_control::only_role(ATTESTOR_ROLE))]
    default fn rollup_cond_eq(
        &mut self,
        conditions: Vec<(Key, Option<Value>)>,
        updates: Vec<(Key, Option<Value>)>,
        actions: Vec<HandleActionInput>,
    ) -> Result<bool, RollupAnchorError> {
        self._rollup_cond_eq(conditions, updates, actions)
    }

    default fn meta_tx_rollup_cond_eq(
        &mut self,
        request: meta_transaction::ForwardRequest,
        signature: [u8; 65],
    ) -> Result<bool, RollupAnchorError> {
        // check the signature
        self._use_meta_tx(&request, &signature)?;

        // check the attestor role
        if !self.has_role(ATTESTOR_ROLE, request.from) {
            return Err(RollupAnchorError::AccessControlError(
                access_control::AccessControlError::MissingRole,
            ));
        }

        // decode the data
        let data: RolupCondEqMethodParams = scale::Decode::decode(&mut request.data.as_slice())
            .map_err(|_| RollupAnchorError::FailedToDecode)?;

        // emit the event
        self._emit_event_meta_tx_decoded();

        // call the rollup
        self._rollup_cond_eq(data.0, data.1, data.2)
    }
}

impl<T> Internal for T
where
    T: rollup_anchor::MessageHandler,
    T: kv_store::Internal,
    T: message_queue::Internal,
    T: access_control::AccessControl,
{
    default fn _rollup_cond_eq(
        &mut self,
        conditions: Vec<(Key, Option<Value>)>,
        updates: Vec<(Key, Option<Value>)>,
        actions: Vec<HandleActionInput>,
    ) -> Result<bool, RollupAnchorError> {
        // check the conditions
        for cond in conditions {
            let key = cond.0;
            let current_value = self._get_value(&key);
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
            self._set_value(&update.0, update.1.as_ref());
        }

        // apply the actions
        for action in actions {
            self._handle_action(action)?;
        }

        Ok(true)
    }

    default fn _handle_action(
        &mut self,
        input: HandleActionInput,
    ) -> Result<(), RollupAnchorError> {
        match input.action_type {
            ACTION_REPLY => {
                self._on_message_received(input.action.ok_or(RollupAnchorError::MissingData)?)?
            }
            ACTION_SET_QUEUE_HEAD => {
                self._pop_to(input.id.ok_or(RollupAnchorError::MissingData)?)?
            }
            ACTION_GRANT_ATTESTOR => self.grant_role(
                ATTESTOR_ROLE,
                input.address.ok_or(RollupAnchorError::MissingData)?,
            )?,
            ACTION_REVOKE_ATTESTOR => self.revoke_role(
                ATTESTOR_ROLE,
                input.address.ok_or(RollupAnchorError::MissingData)?,
            )?,
            _ => return Err(RollupAnchorError::UnsupportedAction),
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use openbrush::contracts::access_control::AccessControl;
    use openbrush::test_utils::{accounts, change_caller};
    use openbrush::traits::AccountId;
    use scale::Encode;

    use crate::impls::message_queue::*;
    use crate::impls::meta_transaction::MetaTxError;
    use crate::impls::rollup_anchor::*;
    use crate::tests::test_contract::MyContract;
    use crate::traits::meta_transaction::MetaTxReceiver;

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
        contract._push_message(&message).unwrap();
        contract._push_message(&message).unwrap();

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
            contract.grant_role(ATTESTOR_ROLE, accounts.bob)
        );

        // alice, the owner, can do it
        change_caller(accounts.alice);
        assert_eq!(Ok(()), contract.grant_role(ATTESTOR_ROLE, accounts.bob));
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
            .grant_role(ATTESTOR_ROLE, accounts.bob)
            .expect("Error when grant the role Attestor");

        change_caller(accounts.bob);
        assert_eq!(Ok(true), contract.rollup_cond_eq(vec![], vec![], vec![]));
    }

    #[ink::test]
    fn test_meta_tx_rollup_cond_eq() {
        let accounts = accounts();
        let mut contract = MyContract::new(accounts.alice);

        // Alice
        let from = AccountId::from(hex_literal::hex!(
            "d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d"
        ));
        let ecdsa_public_key: [u8; 33] =
            hex_literal::hex!("037051bed73458951b45ca6376f4096c85bf1a370da94d5336d04867cfaaad019e");
        let data = RolupCondEqMethodParams::encode(&(vec![], vec![], vec![]));

        // register the ecdsa public key
        contract
            .register_ecdsa_public_key(from, ecdsa_public_key)
            .expect("Error when registering ecdsa public key");

        let (request, hash) = contract
            .prepare(from, data)
            .expect("Error when preparing meta tx");

        let expected_hash =
            hex_literal::hex!("c91f57305dc05a66f1327352d55290a250eb61bba8e3cf8560a4b8e7d172bb54");
        assert_eq!(&expected_hash, &hash.as_ref());

        // signature by Alice of previous hash
        let signature : [u8; 65] = hex_literal::hex!("c9a899bc8daa98fd1e819486c57f9ee889d035e8d0e55c04c475ca32bb59389b284d18d785a9db1bdd72ce74baefe6a54c0aa2418b14f7bc96232fa4bf42946600");

        // add the role => it should be succeed
        contract
            .grant_role(ATTESTOR_ROLE, request.from)
            .expect("Error when grant the role Attestor");
        assert_eq!(
            Ok(true),
            contract.meta_tx_rollup_cond_eq(request.clone(), signature)
        );

        // do it again => it must failed
        assert_eq!(
            Err(RollupAnchorError::MetaTxError(MetaTxError::NonceTooLow)),
            contract.meta_tx_rollup_cond_eq(request.clone(), signature)
        );
    }

    #[ink::test]
    fn test_meta_tx_rollup_cond_eq_missing_role() {
        let accounts = accounts();
        let mut contract = MyContract::new(accounts.alice);

        // Alice
        let from = AccountId::from(hex_literal::hex!(
            "d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d"
        ));
        let ecdsa_public_key: [u8; 33] =
            hex_literal::hex!("037051bed73458951b45ca6376f4096c85bf1a370da94d5336d04867cfaaad019e");
        let data = RolupCondEqMethodParams::encode(&(vec![], vec![], vec![]));

        // register the ecdsa public key
        contract
            .register_ecdsa_public_key(from, ecdsa_public_key)
            .expect("Error when registering ecdsa public key");

        let (request, hash) = contract
            .prepare(from, data)
            .expect("Error when preparing meta tx");

        let expected_hash =
            hex_literal::hex!("c91f57305dc05a66f1327352d55290a250eb61bba8e3cf8560a4b8e7d172bb54");
        assert_eq!(&expected_hash, &hash.as_ref());

        // signature by Alice of previous hash
        let signature : [u8; 65] = hex_literal::hex!("c9a899bc8daa98fd1e819486c57f9ee889d035e8d0e55c04c475ca32bb59389b284d18d785a9db1bdd72ce74baefe6a54c0aa2418b14f7bc96232fa4bf42946600");

        // missing role
        assert_eq!(
            Err(RollupAnchorError::AccessControlError(
                access_control::AccessControlError::MissingRole
            )),
            contract.meta_tx_rollup_cond_eq(request.clone(), signature)
        );
    }
}
