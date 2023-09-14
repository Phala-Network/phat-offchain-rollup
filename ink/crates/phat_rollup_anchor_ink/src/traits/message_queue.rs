use crate::traits::kv_store;
use ink::prelude::vec::Vec;
pub use kv_session::traits::QueueIndex;
use scale::{Decode, Encode};

const QUEUE_PREFIX: &[u8] = b"q/";
const QUEUE_HEAD_KEY: &[u8] = b"_head";
const QUEUE_TAIL_KEY: &[u8] = b"_tail";

#[derive(Debug, Eq, PartialEq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum MessageQueueError {
    InvalidPopTarget,
    FailedToDecode,
}

pub trait EventBroadcaster {
    fn emit_event_message_queued(&self, id: QueueIndex, data: Vec<u8>);

    fn emit_event_message_processed_to(&self, id: QueueIndex);
}

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
                .map_err(|_| MessageQueueError::FailedToDecode)?,
            _ => 0,
        }
    }};
}

pub trait MessageQueue: EventBroadcaster + kv_store::KvStore {
    fn push_message<M: scale::Encode>(
        &mut self,
        data: &M,
    ) -> Result<QueueIndex, MessageQueueError> {
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
    ) -> Result<Option<M>, MessageQueueError> {
        let key = get_key!(id);
        match self.inner_get_value(&key) {
            Some(v) => {
                let message =
                    M::decode(&mut v.as_slice()).map_err(|_| MessageQueueError::FailedToDecode)?;
                Ok(Some(message))
            }
            _ => Ok(None),
        }
    }

    fn get_queue_tail(&self) -> Result<QueueIndex, MessageQueueError> {
        let key = get_tail_key!();
        let index = get_queue_index!(self, key);
        Ok(index)
    }

    fn get_queue_head(&self) -> Result<QueueIndex, MessageQueueError> {
        let key = get_head_key!();
        let index = get_queue_index!(self, key);
        Ok(index)
    }

    fn pop_to(&mut self, target_id: QueueIndex) -> Result<(), MessageQueueError> {
        let current_tail_id = self.get_queue_tail()?;
        if target_id > current_tail_id {
            return Err(MessageQueueError::InvalidPopTarget);
        }

        let current_head_id = self.get_queue_head()?;
        if target_id < current_head_id {
            return Err(MessageQueueError::InvalidPopTarget);
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
}
