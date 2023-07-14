use kv_session::traits::{Key, Value};
use openbrush::storage::Mapping;
use openbrush::traits::Storage;

pub use crate::traits::kv_store::*;

pub const STORAGE_KEY: u32 = openbrush::storage_unique_key!(Data);

#[derive(Default, Debug)]
#[openbrush::upgradeable_storage(STORAGE_KEY)]
pub struct Data {
    kv_store: Mapping<Key, Value>,
}

impl<T> KVStore for T
where
    T: Storage<Data>,
{
    default fn get_value(&self, key: Key) -> Option<Value> {
        self._get_value(&key)
    }

    default fn _get_value(&self, key: &Key) -> Option<Value> {
        self.data::<Data>().kv_store.get(key)
    }

    default fn _set_value(&mut self, key: &Key, value: Option<&Value>) {
        match value {
            None => self.data::<Data>().kv_store.remove(key),
            Some(v) => self.data::<Data>().kv_store.insert(key, v),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::impls::kv_store::*;
    use crate::tests::test_contract::MyContract;
    use openbrush::test_utils::accounts;
    use scale::Encode;

    #[ink::test]
    fn test_get_no_value() {
        let accounts = accounts();
        let contract = MyContract::new(accounts.alice);

        let key = b"0x123".to_vec();
        assert_eq!(None, contract._get_value(&key));
    }

    #[ink::test]
    fn test_set_encoded_values() {
        let accounts = accounts();
        let mut contract = MyContract::new(accounts.alice);

        let key_1 = b"0x123".to_vec();
        let some_value_1 = "0x456".encode();
        contract._set_value(&key_1, Some(&some_value_1));

        let key_2 = b"0x124".to_vec();
        let some_value_2 = "0x457".encode();
        contract._set_value(&key_2, Some(&some_value_2));

        match contract._get_value(&key_1) {
            Some(v) => assert_eq!(some_value_1, v),
            _ => panic!("We should find a value for the key {:?}", key_1),
        }

        match contract._get_value(&key_2) {
            Some(v) => assert_eq!(some_value_2, v),
            _ => panic!("We should find a value for the key {:?}", key_2),
        }

        let key_3 = b"0x125".to_vec();
        if let Some(_) = contract._get_value(&key_3) {
            panic!("We should not find a value for the key {:?}", key_3);
        }
    }

    #[ink::test]
    fn test_update_same_key() {
        let accounts = accounts();
        let mut contract = MyContract::new(accounts.alice);

        let key = b"0x123".to_vec();

        // update the value
        let some_value = "0x456".encode();
        contract._set_value(&key, Some(&some_value));
        match contract._get_value(&key) {
            Some(v) => assert_eq!(some_value, v),
            _ => panic!("We should find a value for the key {:?}", key),
        }
        // update the value
        let another_value = "0x457".encode();
        contract._set_value(&key, Some(&another_value));
        match contract._get_value(&key) {
            Some(v) => assert_eq!(another_value, v),
            _ => panic!("We should find a value for the key {:?}", key),
        }
        // remove the value
        contract._set_value(&key, None);
        if let Some(_) = contract._get_value(&key) {
            panic!("We should not find a value for the key {:?}", key);
        }
        // update the value
        let another_value = "0x458".encode();
        contract._set_value(&key, Some(&another_value));
        match contract._get_value(&key) {
            Some(v) => assert_eq!(another_value, v),
            _ => panic!("We should find a value for the key {:?}", key),
        }
    }
}
