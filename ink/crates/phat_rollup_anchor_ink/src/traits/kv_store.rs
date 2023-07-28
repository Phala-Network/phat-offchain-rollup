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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::test_contract::MyContract;
    use openbrush::test_utils::accounts;
    use scale::Encode;

    #[ink::test]
    fn test_get_no_value() {
        let accounts = accounts();
        let contract = MyContract::new(accounts.alice);

        let key = b"0x123".to_vec();
        assert_eq!(None, contract.get_value(key));
    }

    #[ink::test]
    fn test_set_encoded_values() {
        let accounts = accounts();
        let mut contract = MyContract::new(accounts.alice);

        let key_1 = b"0x123".to_vec();
        let some_value_1 = "0x456".encode();
        contract.set_value(&key_1, Some(&some_value_1));

        let key_2 = b"0x124".to_vec();
        let some_value_2 = "0x457".encode();
        contract.set_value(&key_2, Some(&some_value_2));

        match contract.get_value(key_1.clone()) {
            Some(v) => assert_eq!(some_value_1, v),
            _ => panic!("We should find a value for the key {:?}", key_1),
        }

        match contract.get_value(key_2.clone()) {
            Some(v) => assert_eq!(some_value_2, v),
            _ => panic!("We should find a value for the key {:?}", key_2),
        }

        let key_3 = b"0x125".to_vec();
        if let Some(_) = contract.get_value(key_3.clone()) {
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
        contract.set_value(&key, Some(&some_value));
        match contract.inner_get_value(&key) {
            Some(v) => assert_eq!(some_value, v),
            _ => panic!("We should find a value for the key {:?}", key),
        }
        // update the value
        let another_value = "0x457".encode();
        contract.set_value(&key, Some(&another_value));
        match contract.inner_get_value(&key) {
            Some(v) => assert_eq!(another_value, v),
            _ => panic!("We should find a value for the key {:?}", key),
        }
        // remove the value
        contract.set_value(&key, None);
        if let Some(_) = contract.inner_get_value(&key) {
            panic!("We should not find a value for the key {:?}", key);
        }
        // update the value
        let another_value = "0x458".encode();
        contract.set_value(&key, Some(&another_value));
        match contract.inner_get_value(&key) {
            Some(v) => assert_eq!(another_value, v),
            _ => panic!("We should find a value for the key {:?}", key),
        }
    }
}
