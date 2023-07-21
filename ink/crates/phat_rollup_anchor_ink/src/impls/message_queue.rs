use kv_session::traits::QueueIndex;
use scale::{Decode, Encode};

use crate::traits::kv_store;
pub use crate::traits::message_queue::{self, *};

const QUEUE_PREFIX: &[u8] = b"q/";
const QUEUE_HEAD_KEY: &[u8] = b"_head";
const QUEUE_TAIL_KEY: &[u8] = b"_tail";

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
        match $kv._get_value(&$key) {
            Some(v) => QueueIndex::decode(&mut v.as_slice())
                .map_err(|_| MessageQueueError::FailedToDecode)?,
            _ => 0,
        }
    }};
}
pub trait MessageQueueImpl: message_queue::Internal + message_queue::EventBroadcaster + kv_store::Internal {

    fn push_message<M: Encode>(
        &mut self,
        data: &M,
    ) -> Result<QueueIndex, MessageQueueError> {
        let id = self._get_queue_tail()?;
        let key = get_key!(id);
        let encoded_value = data.encode();
        self._set_value(&key, Some(&encoded_value));

        self._set_queue_tail(id + 1);
        self.emit_event_message_queued(id, data.encode());

        Ok(id)
    }
}

pub trait InternalImpl: kv_store::Internal + message_queue::EventBroadcaster {

    fn _get_message<M: Decode>(&self, id: QueueIndex) -> Result<Option<M>, MessageQueueError> {
        let key = get_key!(id);
        match self._get_value(&key) {
            Some(v) => {
                let message =
                    M::decode(&mut v.as_slice()).map_err(|_| MessageQueueError::FailedToDecode)?;
                Ok(Some(message))
            }
            _ => Ok(None),
        }
    }

    fn _get_queue_tail(&self) -> Result<QueueIndex, MessageQueueError> {
        let key = get_tail_key!();
        let index = get_queue_index!(self, key);
        Ok(index)
    }

    fn _get_queue_head(&self) -> Result<QueueIndex, MessageQueueError> {
        let key = get_head_key!();
        let index = get_queue_index!(self, key);
        Ok(index)
    }

    fn _pop_to(&mut self, target_id: QueueIndex) -> Result<(), MessageQueueError> {
        let current_tail_id = self._get_queue_tail()?;
        if target_id > current_tail_id {
            return Err(MessageQueueError::InvalidPopTarget);
        }

        let current_head_id = self._get_queue_head()?;
        if target_id < current_head_id {
            return Err(MessageQueueError::InvalidPopTarget);
        }

        if target_id == current_head_id {
            // nothing to do
            return Ok(());
        }

        for id in current_head_id..target_id {
            let key = get_key!(id);
            self._set_value(&key, None);
        }

        self._set_queue_head(target_id);
        self.emit_event_message_processed_to(target_id);

        Ok(())
    }

    fn _set_queue_tail(&mut self, id: QueueIndex) {
        let key = get_tail_key!();
        self._set_value(&key, Some(&id.encode()));
    }

    fn _set_queue_head(&mut self, id: QueueIndex) {
        let key = get_head_key!();
        self._set_value(&key, Some(&id.encode()));
    }
}

#[cfg(test)]
mod tests {

    use crate::impls::message_queue::*;
    use crate::tests::test_contract::MyContract;
    use openbrush::test_utils::accounts;

    #[ink::test]
    fn test_push_and_pop_message() {
        let accounts = accounts();
        let mut contract = MyContract::new(accounts.alice);

        assert_eq!(0, contract.get_queue_tail().unwrap());
        assert_eq!(0, contract.get_queue_head().unwrap());

        // push the first message in the queue
        let message1 = 123456u128;
        let queue_index = contract.push_message(&message1).unwrap();
        assert_eq!(0, queue_index);
        assert_eq!(0, contract.get_queue_head().unwrap());
        assert_eq!(1, contract.get_queue_tail().unwrap());

        // push the second message in the queue
        let message2 = 4589u16;
        let queue_index = contract.push_message(&message2).unwrap();
        assert_eq!(1, queue_index);
        assert_eq!(0, contract.get_queue_head().unwrap());
        assert_eq!(2, contract.get_queue_tail().unwrap());

        // get the first message
        let message_in_queue: Option<u128> = contract.get_message(0).unwrap();
        assert_eq!(
            message1,
            message_in_queue.expect("we expect a message in the queue")
        );

        // get the seconde message
        let message_in_queue: Option<u16> = contract.get_message(1).unwrap();
        assert_eq!(
            message2,
            message_in_queue.expect("we expect a message in the queue")
        );

        // pop the two messages
        contract._pop_to(2).unwrap();
        assert_eq!(2, contract.get_queue_head().unwrap());
        assert_eq!(2, contract.get_queue_tail().unwrap());
    }

    #[ink::test]
    fn test_pop_messages() {
        let accounts = accounts();
        let mut contract = MyContract::new(accounts.alice);

        // pop to the future => error
        assert_eq!(
            Err(MessageQueueError::InvalidPopTarget),
            contract._pop_to(2)
        );

        let message = 4589u16;
        contract.push_message(&message).unwrap();
        contract.push_message(&message).unwrap();
        contract.push_message(&message).unwrap();
        contract.push_message(&message).unwrap();
        contract.push_message(&message).unwrap();

        assert_eq!(0, contract.get_queue_head().unwrap());
        assert_eq!(5, contract.get_queue_tail().unwrap());

        assert_eq!(Ok(()), contract._pop_to(2));

        assert_eq!(2, contract.get_queue_head().unwrap());
        assert_eq!(5, contract.get_queue_tail().unwrap());

        // we do nothing
        assert_eq!(Ok(()), contract._pop_to(2));

        assert_eq!(2, contract.get_queue_head().unwrap());
        assert_eq!(5, contract.get_queue_tail().unwrap());

        // pop to the past => error
        assert_eq!(
            Err(MessageQueueError::InvalidPopTarget),
            contract._pop_to(1)
        );

        // we do nothing
        assert_eq!(Ok(()), contract._pop_to(5));

        assert_eq!(5, contract.get_queue_head().unwrap());
        assert_eq!(5, contract.get_queue_tail().unwrap());
    }
}
