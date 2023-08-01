use kv_session::traits::{Key, Value};
use openbrush::storage::Mapping;
use openbrush::traits::Storage;

#[derive(Default, Debug)]
#[openbrush::storage_item]
pub struct Data {
    pub kv_store: Mapping<Key, Value>,
}

#[openbrush::trait_definition]
pub trait KvStore: Storage<Data> {
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
}
