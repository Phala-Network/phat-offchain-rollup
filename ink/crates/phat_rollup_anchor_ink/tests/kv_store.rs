use openbrush::test_utils::accounts;
use phat_rollup_anchor_ink::traits::kv_store::KvStore;
use scale::Encode;

mod contract;
use contract::test_contract::MyContract;

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

    assert_eq!(
        contract.inner_get_value(&key),
        Some(some_value),
        "We should find a value for the key {key:?}"
    );

    // update the value
    let another_value = "0x457".encode();
    contract.set_value(&key, Some(&another_value));
    assert_eq!(
        contract.inner_get_value(&key),
        Some(another_value),
        "We should find a value for the key {key:?}"
    );

    // remove the value
    contract.set_value(&key, None);
    assert_eq!(
        contract.inner_get_value(&key),
        None,
        "We should not find a value for the key {key:?}"
    );

    // update the value
    let another_value = "0x458".encode();
    contract.set_value(&key, Some(&another_value));
    assert_eq!(
        contract.inner_get_value(&key),
        Some(another_value),
        "We should find a value for the key {key:?}"
    );
}
