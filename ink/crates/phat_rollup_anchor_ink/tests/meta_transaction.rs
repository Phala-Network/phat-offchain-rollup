use ink::env::debug_println;
use openbrush::contracts::access_control::{AccessControl, AccessControlError};
use openbrush::test_utils::accounts;
use openbrush::traits::AccountId;
use phat_rollup_anchor_ink::traits::meta_transaction::*;
use phat_rollup_anchor_ink::traits::rollup_anchor::*;
use scale::Encode;

mod contract;
use contract::test_contract::MyContract;

#[ink::test]
fn test_get_nonce() {
    let accounts = accounts();
    let contract = MyContract::new(accounts.bob);

    // no nonce (ie 0) for new account
    assert_eq!(0, contract.get_nonce(accounts.bob));
}

#[ink::test]
fn test_prepare() {
    let accounts = accounts();
    let mut contract = MyContract::new(accounts.bob);

    // Alice
    let from = AccountId::from(hex_literal::hex!(
        "d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d"
    ));
    let ecdsa_public_key: [u8; 33] =
        hex_literal::hex!("037051bed73458951b45ca6376f4096c85bf1a370da94d5336d04867cfaaad019e");

    let data = u8::encode(&5);

    // register the ecda public key because I am not able to retrieve if from the account id
    contract
        .register_ecdsa_public_key(from, ecdsa_public_key)
        .expect("Error when registering ecdsa public key");

    // prepare the meta transaction
    let (request, hash) = contract
        .prepare(from, data.clone())
        .expect("Error when preparing meta tx");

    assert_eq!(0, request.nonce);
    assert_eq!(from, request.from);
    assert_eq!(&data, &request.data);

    debug_println!("code hash: {:02x?}", hash);
    let expected_hash =
        hex_literal::hex!("17cb4f6eae2f95ba0fbaee9e0e51dc790fe752e7386b72dcd93b9669450c2ccf");
    assert_eq!(&expected_hash, &hash.as_ref());
}

#[ink::test]
fn test_verify() {
    let accounts = accounts();
    let mut contract = MyContract::new(accounts.bob);

    // Alice
    let from = AccountId::from(hex_literal::hex!(
        "d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d"
    ));
    let ecdsa_public_key: [u8; 33] =
        hex_literal::hex!("037051bed73458951b45ca6376f4096c85bf1a370da94d5336d04867cfaaad019e");

    // register the ecda public key because I am not able to retrieve if from the account id
    contract
        .register_ecdsa_public_key(from, ecdsa_public_key)
        .expect("Error when registering ecdsa public key");

    let nonce: Nonce = 0;
    let data = u8::encode(&5);
    let request = ForwardRequest {
        from,
        nonce,
        data: data.clone(),
    };

    // signature by Alice of hash : 17cb4f6eae2f95ba0fbaee9e0e51dc790fe752e7386b72dcd93b9669450c2ccf
    let signature = hex_literal::hex!("ce68d0383bd8f521a2243415add58ed0aed58c246229f15672ed6f99ba6c6c823a6d5fe7503703423e46206196c499d132533a151e2e7d9754b497a9d3014d9301");

    // the verification must succeed
    assert_eq!(Ok(()), contract.verify(&request, &signature));

    // incorrect 'from' => the verification must fail
    let request = ForwardRequest {
        from: accounts.bob,
        nonce,
        data: data.clone(),
    };
    assert_eq!(
        Err(MetaTransactionError::PublicKeyNotRegistered),
        contract.verify(&request, &signature)
    );

    // incorrect nonce => the verification must fail
    let request = ForwardRequest {
        from,
        nonce: 1,
        data: data.clone(),
    };
    assert_eq!(
        Err(MetaTransactionError::PublicKeyNotMatch),
        contract.verify(&request, &signature)
    );

    // incorrect data => the verification must fail
    let request = ForwardRequest {
        from,
        nonce,
        data: u8::encode(&55),
    };
    assert_eq!(
        Err(MetaTransactionError::PublicKeyNotMatch),
        contract.verify(&request, &signature)
    );

    // register another ecda public key
    let ecdsa_public_key =
        hex_literal::hex!("037051bed73458951b45ca6376f4096c85bf1a370da94d5336d04867cfaaad019f");
    contract
        .register_ecdsa_public_key(from, ecdsa_public_key)
        .expect("Error when registering ecdsa public key");
    // incorrect ecdsa public key => the verification must fail
    let request = ForwardRequest {
        from,
        nonce,
        data: data.clone(),
    };
    assert_eq!(
        Err(MetaTransactionError::PublicKeyNotMatch),
        contract.verify(&request, &signature)
    );
}

#[ink::test]
fn test_use_meta_tx() {
    let accounts = accounts();
    let mut contract = MyContract::new(accounts.bob);

    // Alice
    let from = AccountId::from(hex_literal::hex!(
        "d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d"
    ));
    let ecdsa_public_key: [u8; 33] =
        hex_literal::hex!("037051bed73458951b45ca6376f4096c85bf1a370da94d5336d04867cfaaad019e");

    // register the ecda public key
    contract
        .register_ecdsa_public_key(from, ecdsa_public_key)
        .expect("Error when registering ecdsa public key");

    let nonce: Nonce = 0;
    let data = u8::encode(&5);
    let request = ForwardRequest {
        from,
        nonce,
        data: data.clone(),
    };

    // signature by Alice
    let signature = hex_literal::hex!("ce68d0383bd8f521a2243415add58ed0aed58c246229f15672ed6f99ba6c6c823a6d5fe7503703423e46206196c499d132533a151e2e7d9754b497a9d3014d9301");

    // the verification must succeed
    contract
        .use_meta_tx(&request, &signature)
        .expect("Error when using meta tx");

    // check if the nonce has been updated
    assert_eq!(1, contract.get_nonce(from));

    // test we cannot reuse the same call
    // the verification must fail
    assert_eq!(
        Err(MetaTransactionError::NonceTooLow),
        contract.use_meta_tx(&request, &signature)
    );
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
    let data = RollupCondEqMethodParams::encode(&(vec![], vec![], vec![]));

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
        .grant_role(ATTESTOR_ROLE, Some(request.from))
        .expect("Error when grant the role Attestor");
    assert_eq!(
        Ok(()),
        contract.meta_tx_rollup_cond_eq(request.clone(), signature)
    );

    // do it again => it must failed
    assert_eq!(
        Err(MetaTransactionError::NonceTooLow),
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
    let data = RollupCondEqMethodParams::encode(&(vec![], vec![], vec![]));

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
        Err(MetaTransactionError::RollupAnchorError(
            RollupAnchorError::AccessControlError(AccessControlError::MissingRole)
        )),
        contract.meta_tx_rollup_cond_eq(request.clone(), signature)
    );
}
