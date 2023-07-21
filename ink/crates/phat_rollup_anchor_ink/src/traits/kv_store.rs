pub use kv_session::traits::{Key, Value};

#[openbrush::trait_definition]
pub trait KVStore {
    #[ink(message)]
    fn get_value(&self, key: Key) -> Option<Value>;
}

pub trait Internal {
    fn _get_value(&self, key: &Key) -> Option<Value>;

    fn _set_value(&mut self, key: &Key, value: Option<&Value>);
}
