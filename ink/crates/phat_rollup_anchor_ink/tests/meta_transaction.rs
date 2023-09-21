use ink::env::test::set_callee;
use ink::env::{debug_println, DefaultEnvironment};
use openbrush::contracts::access_control::{AccessControl, AccessControlError};
use openbrush::test_utils::accounts;
use openbrush::traits::AccountId;
use phat_rollup_anchor_ink::traits::meta_transaction::*;
use phat_rollup_anchor_ink::traits::rollup_anchor::*;
use scale::Encode;

mod contract;
use contract::test_contract::MyContract;
use ink_e2e::subxt::tx::Signer;
use ink_e2e::PolkadotConfig;

#[ink::test]
fn test_get_nonce() {
    let accounts = accounts();
    let contract = MyContract::new(accounts.bob);

    // no nonce (ie 0) for new account
    assert_eq!(0, contract.get_nonce(accounts.bob));
}

#[ink::test]
fn test_prepare() {
    let contract_address = AccountId::from([0xFF as u8; 32]);
    set_callee::<DefaultEnvironment>(contract_address);

    let accounts = accounts();
    let contract = MyContract::new(accounts.bob);

    // ecdsa public key d'Alice
    let from = ink::primitives::AccountId::from(
        Signer::<PolkadotConfig>::account_id(&subxt_signer::ecdsa::dev::alice()).0,
    );

    let data = u8::encode(&5);

    // prepare the meta transaction
    let (request, hash) = contract
        .prepare(from, data.clone())
        .expect("Error when preparing meta tx");

    assert_eq!(0, request.nonce);
    assert_eq!(from, request.from);
    assert_eq!(contract_address, request.to);
    assert_eq!(&data, &request.data);

    debug_println!("message: {:02x?}", &scale::Encode::encode(&request));

    debug_println!("code hash: {:02x?}", hash);
    let expected_hash =
        hex_literal::hex!("9eb948928cf669f05801b791e5770419f1184637cf2ff3e8124c92e44d45e76f");
    assert_eq!(&expected_hash, &hash.as_ref());
}

#[ink::test]
fn test_verify() {
    let contract_address = AccountId::from([0xFF as u8; 32]);
    set_callee::<DefaultEnvironment>(contract_address);

    let accounts = accounts();
    let contract = MyContract::new(accounts.bob);

    // ecdsa public key d'Alice
    let keypair = subxt_signer::ecdsa::dev::alice();
    let from = AccountId::from(Signer::<PolkadotConfig>::account_id(&keypair).0);

    let nonce: Nonce = 0;
    let data = u8::encode(&5);
    let request = ForwardRequest {
        from,
        to: contract_address,
        nonce,
        data: data.clone(),
    };

    let message = scale::Encode::encode(&request);
    debug_println!("message: {:02x?}", &message);
    // Alice signs the message
    let signature = keypair.sign(&message).0;
    debug_println!("signature: {:02x?}", &signature);

    // the verification must succeed
    assert_eq!(Ok(()), contract.verify(&request, &signature));

    // incorrect 'from' => the verification must fail
    let request = ForwardRequest {
        from: accounts.bob,
        to: contract_address,
        nonce,
        data: data.clone(),
    };
    assert_eq!(
        Err(MetaTransactionError::PublicKeyNotMatch),
        contract.verify(&request, &signature)
    );

    // incorrect 'to' => the verification must fail
    let request = ForwardRequest {
        from,
        to: accounts.bob,
        nonce,
        data: data.clone(),
    };
    assert_eq!(
        Err(MetaTransactionError::InvalidDestination),
        contract.verify(&request, &signature)
    );

    // incorrect nonce => the verification must fail
    let request = ForwardRequest {
        from,
        to: contract_address,
        nonce: 1,
        data: data.clone(),
    };
    assert_eq!(
        Err(MetaTransactionError::NonceTooLow),
        contract.verify(&request, &signature)
    );

    // incorrect data => the verification must fail
    let request = ForwardRequest {
        from,
        to: contract_address,
        nonce,
        data: u8::encode(&55),
    };
    assert_eq!(
        Err(MetaTransactionError::PublicKeyNotMatch),
        contract.verify(&request, &signature)
    );
}

#[ink::test]
fn test_ensure_meta_tx_valid() {
    let contract_address = AccountId::from([0xFF as u8; 32]);
    set_callee::<DefaultEnvironment>(contract_address);

    let accounts = accounts();
    let mut contract = MyContract::new(accounts.bob);

    // ecdsa public key d'Alice
    let keypair = subxt_signer::ecdsa::dev::alice();
    let from = AccountId::from(Signer::<PolkadotConfig>::account_id(&keypair).0);

    let nonce: Nonce = 0;
    let data = u8::encode(&5);
    let request = ForwardRequest {
        from,
        to: contract_address,
        nonce,
        data: data.clone(),
    };

    // Alice signs the message
    let signature = keypair.sign(&scale::Encode::encode(&request)).0;
    debug_println!("signature: {:02x?}", &signature);

    // the verification must succeed
    contract
        .ensure_meta_tx_valid(&request, &signature)
        .expect("Error when using meta tx");

    // check if the nonce has been updated
    assert_eq!(1, contract.get_nonce(from));

    // test we cannot reuse the same call
    // the verification must fail
    assert_eq!(
        Err(MetaTransactionError::NonceTooLow),
        contract.ensure_meta_tx_valid(&request, &signature)
    );
}

#[ink::test]
fn test_meta_tx_rollup_cond_eq() {
    let contract_address = AccountId::from([0xFF as u8; 32]);
    set_callee::<DefaultEnvironment>(contract_address);

    let accounts = accounts();
    let mut contract = MyContract::new(accounts.alice);

    // ecdsa public key d'Alice
    let keypair = subxt_signer::ecdsa::dev::alice();
    let from = AccountId::from(Signer::<PolkadotConfig>::account_id(&keypair).0);

    let data = RollupCondEqMethodParams::encode(&(vec![], vec![], vec![]));

    let (request, hash) = contract
        .prepare(from, data)
        .expect("Error when preparing meta tx");

    debug_println!("code hash: {:02x?}", hash);
    let expected_hash =
        hex_literal::hex!("8e00f5d6a0f721acb9f4244a1c28787f7d1cb628176b132b2010c880de153e2e");
    assert_eq!(&expected_hash, &hash.as_ref());

    // Alice signs the message
    let signature = keypair.sign(&scale::Encode::encode(&request)).0;
    debug_println!("signature: {:02x?}", &signature);

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
    let contract_address = AccountId::from([0xFF as u8; 32]);
    set_callee::<DefaultEnvironment>(contract_address);

    let accounts = accounts();
    let mut contract = MyContract::new(accounts.alice);

    // ecdsa public key d'Alice
    let keypair = subxt_signer::ecdsa::dev::alice();
    let from = AccountId::from(Signer::<PolkadotConfig>::account_id(&keypair).0);

    let data = RollupCondEqMethodParams::encode(&(vec![], vec![], vec![]));

    let (request, hash) = contract
        .prepare(from, data)
        .expect("Error when preparing meta tx");

    debug_println!("code hash: {:02x?}", hash);
    let expected_hash =
        hex_literal::hex!("8e00f5d6a0f721acb9f4244a1c28787f7d1cb628176b132b2010c880de153e2e");
    assert_eq!(&expected_hash, &hash.as_ref());

    // Alice signs the message
    let signature = keypair.sign(&scale::Encode::encode(&request)).0;
    debug_println!("signature: {:02x?}", &signature);

    // missing role
    assert_eq!(
        Err(MetaTransactionError::RollupAnchorError(
            RollupAnchorError::AccessControlError(AccessControlError::MissingRole)
        )),
        contract.meta_tx_rollup_cond_eq(request.clone(), signature)
    );
}
