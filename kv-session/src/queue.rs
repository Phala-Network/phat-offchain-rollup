use alloc::{
    string::{String, ToString},
    vec::Vec,
};
use core::marker::PhantomData;

use crate::{
    rollup::RollUpTransaction,
    traits::{KvSession, KvSnapshot, KvSnapshotExt, PrefixedKvSnapshot},
    OneLock, Result, Session,
};

pub trait Codec {
    fn encode_u128(number: u128) -> Vec<u8>;
    fn decode_u128(raw: impl AsRef<[u8]>) -> Result<u128>;
}

type InnerSession<Snap> =
    Session<PrefixedKvSnapshot<String, Snap>, String, Vec<u8>, OneLock<String>>;

pub struct MessageQueueSession<Snap, Cod> {
    prefix: String,
    session: InnerSession<Snap>,
    // The pos of first pushed message
    head: u128,
    // The pos to push the next message
    tail: u128,
    codec: PhantomData<Cod>,
}

impl<S, C> MessageQueueSession<S, C>
where
    S: KvSnapshot<Key = String, Value = Vec<u8>> + Clone,
    C: Codec,
{
    pub fn new(prefix: impl Into<String>, snapshot: S) -> Result<Self> {
        let prefix = prefix.into();
        let session = Session::new(
            snapshot.prefixed(prefix.clone()),
            // Treat the head cursor as lock
            OneLock::new("head".into(), false),
        );

        Self {
            prefix,
            session,
            codec: PhantomData,
            head: 0,
            tail: 0,
        }
        .init()
    }

    fn init(mut self) -> Result<Self> {
        self.head = self.get_number("head")?.unwrap_or(0);
        self.tail = self.get_number("tail")?.unwrap_or(0);
        if self.tail < self.head {
            return Err(crate::Error::FailedToDecode);
        }
        Ok(self)
    }

    fn get_number(&mut self, key: &str) -> Result<Option<u128>> {
        self.session.get(key)?.map(C::decode_u128).transpose()
    }

    pub fn length(&self) -> u128 {
        self.tail - self.head
    }

    pub fn pop(&mut self) -> Result<Option<Vec<u8>>> {
        if self.head == self.tail {
            return Ok(None);
        }
        let front_key = self.head.to_string();
        let data = self.session.get(&front_key)?;
        self.session.delete(&front_key);
        self.head += 1;
        self.session.put("head", C::encode_u128(self.head));
        Ok(data)
    }

    pub fn commit(self) -> Result<RollUpTransaction<String, Vec<u8>>> {
        let (tx, snapshot) = self.session.commit();

        let conditions = snapshot.batch_get(&tx.accessed_keys)?;
        let snapshot_id = snapshot.snapshot_id()?;
        Ok(RollUpTransaction {
            snapshot_id,
            conditions,
            updates: tx.value_updates,
        }
        .prefixed_with(self.prefix))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use alloc::{borrow::ToOwned, collections::BTreeMap, sync::Arc, vec};
    use core::cell::RefCell;
    use scale::{Decode, Encode};

    use crate::Error;

    #[derive(Clone, Default)]
    struct MockSnapshot {
        db: Arc<RefCell<BTreeMap<String, Vec<u8>>>>,
    }

    impl MockSnapshot {
        fn set(&self, key: impl Into<String>, value: &[u8]) {
            self.db.borrow_mut().insert(key.into(), value.to_owned());
        }
    }

    impl KvSnapshot for MockSnapshot {
        type Key = String;

        type Value = Vec<u8>;

        fn get(&self, key: &impl ToOwned<Owned = Self::Key>) -> Result<Option<Self::Value>> {
            let key = key.to_owned();
            Ok(self.db.borrow().get(&key).cloned())
        }

        fn snapshot_id(&self) -> Result<Self::Value> {
            Ok(vec![])
        }
    }

    struct ScaleCodec;
    impl Codec for ScaleCodec {
        fn encode_u128(number: u128) -> Vec<u8> {
            Encode::encode(&number)
        }

        fn decode_u128(raw: impl AsRef<[u8]>) -> Result<u128> {
            let mut buf = raw.as_ref();
            Decode::decode(&mut buf).or(Err(Error::FailedToDecode))
        }
    }

    #[test]
    fn empty_queue_works() {
        let kvdb = MockSnapshot::default();
        let mut queue = MessageQueueSession::<_, ScaleCodec>::new("TestQ/", kvdb).unwrap();
        assert_eq!(queue.length(), 0);
        assert_eq!(queue.pop(), Ok(None));
        let tx = queue.commit().unwrap();
        // Should lock the "TestQ/head"
        assert_eq!(tx.conditions, vec![("TestQ/head".to_owned(), None)]);
        assert_eq!(tx.updates, vec![]);
    }

    #[test]
    fn pop_queue_works() {
        let kvdb = MockSnapshot::default();

        // Set up some test data
        kvdb.set("TestQ/head", &0_u128.encode());
        kvdb.set("TestQ/tail", &2_u128.encode());
        kvdb.set("TestQ/0", b"foo");
        kvdb.set("TestQ/1", b"bar");

        let mut queue = MessageQueueSession::<_, ScaleCodec>::new("TestQ/", kvdb).unwrap();
        assert_eq!(queue.length(), 2);
        assert_eq!(queue.pop(), Ok(Some(b"foo".to_vec())));
        assert_eq!(queue.pop(), Ok(Some(b"bar".to_vec())));
        assert_eq!(queue.pop(), Ok(None));
        let tx = queue.commit().unwrap();

        assert_eq!(
            tx.conditions,
            // Should lock on the head cursor
            vec![("TestQ/head".to_owned(), Some(0_u128.encode()))]
        );
        assert_eq!(
            tx.updates,
            vec![
                // Should remove the poped keys. This is not performant if there are many elements
                // in the queue. Use ACTION_QUEUE_PROCESSED_TO is light weight, .
                ("TestQ/0".to_owned(), None),
                ("TestQ/1".to_owned(), None),
                // Should modify the head cursor (also treat as lock)
                ("TestQ/head".to_owned(), Some(2_u128.encode())),
            ]
        );
    }
}
