use ink::prelude::vec::Vec;
pub use kv_session::traits::{Key, QueueIndex, Value};
use openbrush::contracts::access_control::{self, AccessControlError, RoleType};
use openbrush::storage::Mapping;
use openbrush::traits::{AccountId, Storage};
use scale::{Decode, Encode};

pub const ATTESTOR_ROLE: RoleType = ink::selector_id!("ATTESTOR_ROLE");

const QUEUE_PREFIX: &[u8] = b"q/";
const QUEUE_HEAD_KEY: &[u8] = b"_head";
const QUEUE_TAIL_KEY: &[u8] = b"_tail";

#[derive(Default, Debug)]
#[openbrush::storage_item]
pub struct Data {
    pub kv_store: Mapping<Key, Value>,
}

pub trait MessageHandler {
    fn on_message_received(&mut self, action: Vec<u8>) -> Result<(), RollupAnchorError>;
}

pub trait EventBroadcaster {
    fn emit_event_message_queued(&self, id: QueueIndex, data: Vec<u8>);

    fn emit_event_message_processed_to(&self, id: QueueIndex);
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
    InvalidPopTarget,
    ConditionNotMet,
    FailedToDecode,
    UnsupportedAction,
    AccessControlError(AccessControlError),
}

/// convertor from AccessControlError to RollupAnchorError
impl From<AccessControlError> for RollupAnchorError {
    fn from(error: AccessControlError) -> Self {
        RollupAnchorError::AccessControlError(error)
    }
}

pub type RollupCondEqMethodParams = (
    Vec<(Key, Option<Value>)>,
    Vec<(Key, Option<Value>)>,
    Vec<HandleActionInput>,
);

macro_rules! get_key {
    ($id:ident) => {
        [QUEUE_PREFIX, &$id.encode()].concat()
    };
}

macro_rules! get_tail_key {
    () => {
        [QUEUE_PREFIX, QUEUE_TAIL_KEY].concat()
    };
}

macro_rules! get_head_key {
    () => {
        [QUEUE_PREFIX, QUEUE_HEAD_KEY].concat()
    };
}

macro_rules! get_queue_index {
    ($kv:ident, $key:ident) => {{
        match $kv.inner_get_value(&$key) {
            Some(v) => QueueIndex::decode(&mut v.as_slice())
                .map_err(|_| RollupAnchorError::FailedToDecode)?,
            _ => 0,
        }
    }};
}

#[openbrush::trait_definition]
pub trait RollupAnchor:
    Storage<Data>
    + MessageHandler
    + EventBroadcaster
    + access_control::AccessControl
    + access_control::Internal
{
    #[ink(message)]
    fn get_value(&self, key: Key) -> Option<Value> {
        self.inner_get_value(&key)
    }

    fn inner_get_value(&self, key: &Key) -> Option<Value> {
        self.data::<Data>().kv_store.get(key)
    }

    fn set_value(&mut self, key: &Key, value: Option<&Value>) {
        match value {
            None => self.data::<Data>().kv_store.remove(key),
            Some(v) => self.data::<Data>().kv_store.insert(key, v),
        }
    }

    fn push_message<M: scale::Encode>(
        &mut self,
        data: &M,
    ) -> Result<QueueIndex, RollupAnchorError> {
        let id = self.get_queue_tail()?;
        let key = get_key!(id);
        let encoded_value = data.encode();
        self.set_value(&key, Some(&encoded_value));

        self.set_queue_tail(id + 1);
        self.emit_event_message_queued(id, encoded_value);

        Ok(id)
    }

    fn get_message<M: scale::Decode>(
        &self,
        id: QueueIndex,
    ) -> Result<Option<M>, RollupAnchorError> {
        let key = get_key!(id);
        match self.inner_get_value(&key) {
            Some(v) => {
                let message =
                    M::decode(&mut v.as_slice()).map_err(|_| RollupAnchorError::FailedToDecode)?;
                Ok(Some(message))
            }
            _ => Ok(None),
        }
    }

    fn get_queue_tail(&self) -> Result<QueueIndex, RollupAnchorError> {
        let key = get_tail_key!();
        let index = get_queue_index!(self, key);
        Ok(index)
    }

    fn get_queue_head(&self) -> Result<QueueIndex, RollupAnchorError> {
        let key = get_head_key!();
        let index = get_queue_index!(self, key);
        Ok(index)
    }

    fn pop_to(&mut self, target_id: QueueIndex) -> Result<(), RollupAnchorError> {
        let current_tail_id = self.get_queue_tail()?;
        if target_id > current_tail_id {
            return Err(RollupAnchorError::InvalidPopTarget);
        }

        let current_head_id = self.get_queue_head()?;
        if target_id < current_head_id {
            return Err(RollupAnchorError::InvalidPopTarget);
        }

        if target_id == current_head_id {
            // nothing to do
            return Ok(());
        }

        for id in current_head_id..target_id {
            let key = get_key!(id);
            self.set_value(&key, None);
        }

        self.set_queue_head(target_id);
        self.emit_event_message_processed_to(target_id);

        Ok(())
    }

    fn set_queue_tail(&mut self, id: QueueIndex) {
        let key = get_tail_key!();
        self.set_value(&key, Some(&id.encode()));
    }

    fn set_queue_head(&mut self, id: QueueIndex) {
        let key = get_head_key!();
        self.set_value(&key, Some(&id.encode()));
    }

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
