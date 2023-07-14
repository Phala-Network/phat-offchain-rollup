#![cfg_attr(not(feature = "std"), no_std, no_main)]
#![feature(min_specialization)]

#[openbrush::contract]
pub mod test_oracle {
    use ink::codegen::{EmitEvent, Env};
    use ink::prelude::string::String;
    use ink::prelude::vec::Vec;
    use ink::storage::Mapping;
    use openbrush::contracts::access_control::*;
    use openbrush::contracts::ownable::*;
    use openbrush::traits::Storage;
    use scale::{Decode, Encode};

    use phat_rollup_anchor_ink::impls::{
        kv_store, kv_store::*, message_queue, message_queue::*, meta_transaction,
        meta_transaction::*, rollup_anchor, rollup_anchor::*,
    };

    pub type TradingPairId = u32;

    /// Events emitted when a price is received
    #[ink(event)]
    pub struct PriceReceived {
        trading_pair_id: TradingPairId,
        price: u128,
    }

    /// Events emitted when a error is received
    #[ink(event)]
    pub struct ErrorReceived {
        trading_pair_id: TradingPairId,
        err_no: u128,
    }

    /// Errors occurred in the contract
    #[derive(Encode, Decode, Debug)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum ContractError {
        AccessControlError(AccessControlError),
        MessageQueueError(MessageQueueError),
        MissingTradingPair,
    }
    /// convertor from MessageQueueError to ContractError
    impl From<MessageQueueError> for ContractError {
        fn from(error: MessageQueueError) -> Self {
            ContractError::MessageQueueError(error)
        }
    }
    /// convertor from MessageQueueError to ContractError
    impl From<AccessControlError> for ContractError {
        fn from(error: AccessControlError) -> Self {
            ContractError::AccessControlError(error)
        }
    }

    /// Message to request the price of the trading pair
    /// message pushed in the queue by this contract and read by the offchain rollup
    #[derive(Encode, Decode)]
    struct PriceRequestMessage {
        /// id of the pair (use as key in the Mapping)
        trading_pair_id: TradingPairId,
        /// trading pair like 'polkdatot/usd'
        /// Note: it will be better to not save this data in the storage
        token0: String,
        token1: String,
    }
    /// Message sent to provide the price of the trading pair
    /// response pushed in the queue by the offchain rollup and read by this contract
    #[derive(Encode, Decode)]
    struct PriceResponseMessage {
        /// Type of response
        resp_type: u8,
        /// id of the pair
        trading_pair_id: TradingPairId,
        /// price of the trading pair
        price: Option<u128>,
        /// when the price is read
        err_no: Option<u128>,
    }

    /// Type of response when the offchain rollup communicates with this contract
    const TYPE_ERROR: u8 = 0;
    const TYPE_RESPONSE: u8 = 10;
    const TYPE_FEED: u8 = 11;

    /// Data storage
    #[derive(Encode, Decode, Default, Eq, PartialEq, Clone, Debug)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub struct TradingPair {
        /// trading pair like 'polkdatot/usd'
        /// Note: it will be better to not save this data outside of the storage
        token0: String,
        token1: String,
        /// value of the trading pair
        value: u128,
        /// number of updates of the value
        nb_updates: u16,
        /// when the last value has been updated
        last_update: u64,
    }

    #[ink(storage)]
    #[derive(Default, Storage)]
    pub struct TestOracle {
        #[storage_field]
        ownable: ownable::Data,
        #[storage_field]
        access: access_control::Data,
        #[storage_field]
        kv_store: kv_store::Data,
        #[storage_field]
        meta_transaction: meta_transaction::Data,
        trading_pairs: Mapping<TradingPairId, TradingPair>,
    }

    impl Ownable for TestOracle {}
    impl AccessControl for TestOracle {}
    impl KVStore for TestOracle {}
    impl MessageQueue for TestOracle {}
    impl MetaTxReceiver for TestOracle {}
    impl RollupAnchor for TestOracle {}

    impl TestOracle {
        #[ink(constructor)]
        pub fn new() -> Self {
            let mut instance = Self::default();
            let caller = instance.env().caller();
            // set the owner of this contract
            instance._init_with_owner(caller);
            // set the admin of this contract
            instance._init_with_admin(caller);
            // grant the role manager to teh given address
            instance
                .grant_role(MANAGER_ROLE, caller)
                .expect("Should grant the role manager");
            instance
        }

        #[ink(message)]
        #[openbrush::modifiers(access_control::only_role(MANAGER_ROLE))]
        pub fn create_trading_pair(
            &mut self,
            trading_pair_id: TradingPairId,
            token0: String,
            token1: String,
        ) -> Result<(), ContractError> {
            // we create a new trading pair or override an existing one
            let trading_pair = TradingPair {
                token0,
                token1,
                value: 0,
                nb_updates: 0,
                last_update: 0,
            };
            self.trading_pairs.insert(trading_pair_id, &trading_pair);
            Ok(())
        }

        #[ink(message)]
        #[openbrush::modifiers(access_control::only_role(MANAGER_ROLE))]
        pub fn request_price(
            &mut self,
            trading_pair_id: TradingPairId,
        ) -> Result<QueueIndex, ContractError> {
            let index = match self.trading_pairs.get(trading_pair_id) {
                Some(t) => {
                    // push the message in the queue
                    let message = PriceRequestMessage {
                        trading_pair_id,
                        token0: t.token0,
                        token1: t.token1,
                    };
                    self._push_message(&message)?
                }
                _ => return Err(ContractError::MissingTradingPair),
            };

            Ok(index)
        }

        #[ink(message)]
        pub fn get_trading_pair(&self, trading_pair_id: TradingPairId) -> Option<TradingPair> {
            self.trading_pairs.get(trading_pair_id)
        }

        #[ink(message)]
        pub fn register_attestor(
            &mut self,
            account_id: AccountId,
            ecdsa_public_key: [u8; 33],
        ) -> Result<(), RollupAnchorError> {
            self.grant_role(ATTESTOR_ROLE, account_id)?;
            self.register_ecdsa_public_key(account_id, ecdsa_public_key)?;
            Ok(())
        }

        #[ink(message)]
        pub fn get_attestor_role(&self) -> RoleType {
            ATTESTOR_ROLE
        }

        #[ink(message)]
        pub fn get_manager_role(&self) -> RoleType {
            MANAGER_ROLE
        }
    }

    impl rollup_anchor::Internal for TestOracle {
        fn _on_message_received(&mut self, action: Vec<u8>) -> Result<(), RollupAnchorError> {
            // parse the response
            let message: PriceResponseMessage =
                Decode::decode(&mut &action[..]).or(Err(RollupAnchorError::FailedToDecode))?;

            // handle the response
            if message.resp_type == TYPE_RESPONSE || message.resp_type == TYPE_FEED {
                // we received the price
                // register the info
                let mut trading_pair = self
                    .trading_pairs
                    .get(message.trading_pair_id)
                    .unwrap_or_default();
                trading_pair.value = message.price.unwrap_or_default();
                trading_pair.nb_updates += 1;
                trading_pair.last_update = self.env().block_timestamp();
                self.trading_pairs
                    .insert(message.trading_pair_id, &trading_pair);

                // emmit te event
                self.env().emit_event(PriceReceived {
                    trading_pair_id: message.trading_pair_id,
                    price: message.price.unwrap_or_default(),
                });
            } else if message.resp_type == TYPE_ERROR {
                // we received an error
                self.env().emit_event(ErrorReceived {
                    trading_pair_id: message.trading_pair_id,
                    err_no: message.err_no.unwrap_or_default(),
                });
            } else {
                // response type unknown
                return Err(RollupAnchorError::UnsupportedAction);
            }

            Ok(())
        }

        fn _emit_event_meta_tx_decoded(&self) {
            self.env().emit_event(MetaTxDecoded {});
        }
    }

    /// Events emitted when a meta transaction is decoded
    #[ink(event)]
    pub struct MetaTxDecoded {}

    /// Events emitted when a message is pushed in the queue
    #[ink(event)]
    pub struct MessageQueued {
        pub id: u32,
        pub data: Vec<u8>,
    }

    /// Events emitted when a message is proceed
    #[ink(event)]
    pub struct MessageProcessedTo {
        pub id: u32,
    }

    impl message_queue::Internal for TestOracle {
        fn _emit_event_message_queued(&self, id: u32, data: Vec<u8>) {
            self.env().emit_event(MessageQueued { id, data });
        }

        fn _emit_event_message_processed_to(&self, id: u32) {
            self.env().emit_event(MessageProcessedTo { id });
        }
    }

    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        use super::*;
        use openbrush::contracts::access_control::accesscontrol_external::AccessControl;

        use ink_e2e::{build_message, PolkadotConfig};
        use phat_rollup_anchor_ink::impls::{
            meta_transaction::metatxreceiver_external::MetaTxReceiver,
            rollup_anchor::rollupanchor_external::RollupAnchor,
        };

        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        #[ink_e2e::test]
        async fn test_create_trading_pair(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // given
            let constructor = TestOracleRef::new();
            let contract_acc_id = client
                .instantiate("test_oracle", &ink_e2e::alice(), constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            let trading_pair_id = 10;

            // read the trading pair and check it doesn't exist yet
            let get_trading_pair = build_message::<TestOracleRef>(contract_acc_id.clone())
                .call(|oracle| oracle.get_trading_pair(trading_pair_id));
            let get_res = client
                .call_dry_run(&ink_e2e::bob(), &get_trading_pair, 0, None)
                .await;
            assert_eq!(None, get_res.return_value());

            // bob is not granted as manager => it should not be able to create the trading pair
            let create_trading_pair =
                build_message::<TestOracleRef>(contract_acc_id.clone()).call(|oracle| {
                    oracle.create_trading_pair(
                        trading_pair_id,
                        String::from("polkadot"),
                        String::from("usd"),
                    )
                });
            let result = client
                .call(&ink_e2e::bob(), create_trading_pair, 0, None)
                .await;
            assert!(
                result.is_err(),
                "only manager should not be able to create trading pair"
            );

            // bob is granted as manager
            let bob_address =
                ink::primitives::AccountId::from(ink_e2e::bob::<PolkadotConfig>().account_id().0);
            let grant_role = build_message::<TestOracleRef>(contract_acc_id.clone())
                .call(|oracle| oracle.grant_role(MANAGER_ROLE, bob_address));
            client
                .call(&ink_e2e::alice(), grant_role, 0, None)
                .await
                .expect("grant bob as attestor failed");

            // create the trading pair
            let create_trading_pair =
                build_message::<TestOracleRef>(contract_acc_id.clone()).call(|oracle| {
                    oracle.create_trading_pair(
                        trading_pair_id,
                        String::from("polkadot"),
                        String::from("usd"),
                    )
                });
            client
                .call(&ink_e2e::bob(), create_trading_pair, 0, None)
                .await
                .expect("create trading pair failed");

            // then check if the trading pair exists
            let get_trading_pair = build_message::<TestOracleRef>(contract_acc_id.clone())
                .call(|oracle| oracle.get_trading_pair(trading_pair_id));
            let get_res = client
                .call_dry_run(&ink_e2e::bob(), &get_trading_pair, 0, None)
                .await;
            let expected_trading_pair = TradingPair {
                token0: String::from("polkadot"),
                token1: String::from("usd"),
                value: 0,
                nb_updates: 0,
                last_update: 0,
            };
            assert_eq!(Some(expected_trading_pair), get_res.return_value());

            Ok(())
        }

        #[ink_e2e::test]
        async fn test_feed_price(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // given
            let constructor = TestOracleRef::new();
            let contract_acc_id = client
                .instantiate("test_oracle", &ink_e2e::alice(), constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            let trading_pair_id = 10;

            // create the trading pair
            let create_trading_pair =
                build_message::<TestOracleRef>(contract_acc_id.clone()).call(|oracle| {
                    oracle.create_trading_pair(
                        trading_pair_id,
                        String::from("polkadot"),
                        String::from("usd"),
                    )
                });
            client
                .call(&ink_e2e::alice(), create_trading_pair, 0, None)
                .await
                .expect("create trading pair failed");

            // bob is granted as attestor
            let bob_address =
                ink::primitives::AccountId::from(ink_e2e::bob::<PolkadotConfig>().account_id().0);
            let grant_role = build_message::<TestOracleRef>(contract_acc_id.clone())
                .call(|oracle| oracle.grant_role(ATTESTOR_ROLE, bob_address));
            client
                .call(&ink_e2e::alice(), grant_role, 0, None)
                .await
                .expect("grant bob as attestor failed");

            // then bob feeds the price
            let value: u128 = 150_000_000_000_000_000_000;
            let payload = PriceResponseMessage {
                resp_type: TYPE_FEED,
                trading_pair_id,
                price: Some(value),
                err_no: None,
            };
            let actions = vec![HandleActionInput {
                action_type: ACTION_REPLY,
                id: None,
                action: Some(payload.encode()),
                address: None,
            }];
            let rollup_cond_eq = build_message::<TestOracleRef>(contract_acc_id.clone())
                .call(|oracle| oracle.rollup_cond_eq(vec![], vec![], actions.clone()));
            let result = client
                .call(&ink_e2e::bob(), rollup_cond_eq, 0, None)
                .await
                .expect("rollup cond eq failed");
            // events PriceReceived
            assert!(result.contains_event("Contracts", "ContractEmitted"));

            // and check if the price is filled
            let get_trading_pair = build_message::<TestOracleRef>(contract_acc_id.clone())
                .call(|oracle| oracle.get_trading_pair(trading_pair_id));
            let get_res = client
                .call_dry_run(&ink_e2e::bob(), &get_trading_pair, 0, None)
                .await;
            let trading_pair = get_res.return_value().expect("Trading pair not found");

            assert_eq!(value, trading_pair.value);
            assert_eq!(1, trading_pair.nb_updates);
            assert_ne!(0, trading_pair.last_update);

            Ok(())
        }

        #[ink_e2e::test]
        async fn test_receive_reply(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // given
            let constructor = TestOracleRef::new();
            let contract_acc_id = client
                .instantiate("test_oracle", &ink_e2e::alice(), constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            let trading_pair_id = 10;

            // create the trading pair
            let create_trading_pair =
                build_message::<TestOracleRef>(contract_acc_id.clone()).call(|oracle| {
                    oracle.create_trading_pair(
                        trading_pair_id,
                        String::from("polkadot"),
                        String::from("usd"),
                    )
                });
            client
                .call(&ink_e2e::alice(), create_trading_pair, 0, None)
                .await
                .expect("create trading pair failed");

            // bob is granted as attestor
            let bob_address =
                ink::primitives::AccountId::from(ink_e2e::bob::<PolkadotConfig>().account_id().0);
            let grant_role = build_message::<TestOracleRef>(contract_acc_id.clone())
                .call(|oracle| oracle.grant_role(ATTESTOR_ROLE, bob_address));
            client
                .call(&ink_e2e::alice(), grant_role, 0, None)
                .await
                .expect("grant bob as attestor failed");

            // a price request is sent
            let request_price = build_message::<TestOracleRef>(contract_acc_id.clone())
                .call(|oracle| oracle.request_price(trading_pair_id));
            let result = client
                .call(&ink_e2e::alice(), request_price, 0, None)
                .await
                .expect("Request price should be sent");
            // event MessageQueued
            assert!(result.contains_event("Contracts", "ContractEmitted"));

            let request_id = result.return_value().expect("Request id not found");

            // then a response is received
            let value: u128 = 150_000_000_000_000_000_000;
            let payload = PriceResponseMessage {
                resp_type: TYPE_RESPONSE,
                trading_pair_id,
                price: Some(value),
                err_no: None,
            };
            let actions = vec![
                HandleActionInput {
                    action_type: ACTION_REPLY,
                    id: None,
                    action: Some(payload.encode()),
                    address: None,
                },
                HandleActionInput {
                    action_type: ACTION_SET_QUEUE_HEAD,
                    id: Some(request_id + 1),
                    action: None,
                    address: None,
                },
            ];
            let rollup_cond_eq = build_message::<TestOracleRef>(contract_acc_id.clone())
                .call(|oracle| oracle.rollup_cond_eq(vec![], vec![], actions.clone()));
            let result = client
                .call(&ink_e2e::bob(), rollup_cond_eq, 0, None)
                .await
                .expect("rollup cond eq should be ok");
            // two events : MessageProcessedTo and PricesRecieved
            assert!(result.contains_event("Contracts", "ContractEmitted"));

            // and check if the price is filled
            let get_trading_pair = build_message::<TestOracleRef>(contract_acc_id.clone())
                .call(|oracle| oracle.get_trading_pair(trading_pair_id));
            let get_res = client
                .call_dry_run(&ink_e2e::bob(), &get_trading_pair, 0, None)
                .await;
            let trading_pair = get_res.return_value().expect("Trading pair not found");

            assert_eq!(value, trading_pair.value);
            assert_eq!(1, trading_pair.nb_updates);
            assert_ne!(0, trading_pair.last_update);

            // reply in the future should fail
            let actions = vec![
                HandleActionInput {
                    action_type: ACTION_REPLY,
                    id: None,
                    action: Some(payload.encode()),
                    address: None,
                },
                HandleActionInput {
                    action_type: ACTION_SET_QUEUE_HEAD,
                    id: Some(request_id + 2),
                    action: None,
                    address: None,
                },
            ];
            let rollup_cond_eq = build_message::<TestOracleRef>(contract_acc_id.clone())
                .call(|oracle| oracle.rollup_cond_eq(vec![], vec![], actions.clone()));
            let result = client.call(&ink_e2e::bob(), rollup_cond_eq, 0, None).await;
            assert!(
                result.is_err(),
                "Rollup should fail because we try to pop in the future"
            );

            // reply in the past should fail
            let actions = vec![
                HandleActionInput {
                    action_type: ACTION_REPLY,
                    id: None,
                    action: Some(payload.encode()),
                    address: None,
                },
                HandleActionInput {
                    action_type: ACTION_SET_QUEUE_HEAD,
                    id: Some(request_id),
                    action: None,
                    address: None,
                },
            ];
            let rollup_cond_eq = build_message::<TestOracleRef>(contract_acc_id.clone())
                .call(|oracle| oracle.rollup_cond_eq(vec![], vec![], actions.clone()));
            let result = client.call(&ink_e2e::bob(), rollup_cond_eq, 0, None).await;
            assert!(
                result.is_err(),
                "Rollup should fail because we try to pop in the past"
            );

            Ok(())
        }

        #[ink_e2e::test]
        async fn test_receive_error(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // given
            let constructor = TestOracleRef::new();
            let contract_acc_id = client
                .instantiate("test_oracle", &ink_e2e::alice(), constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            let trading_pair_id = 10;

            // create the trading pair
            let create_trading_pair =
                build_message::<TestOracleRef>(contract_acc_id.clone()).call(|oracle| {
                    oracle.create_trading_pair(
                        trading_pair_id,
                        String::from("polkadot"),
                        String::from("usd"),
                    )
                });
            client
                .call(&ink_e2e::alice(), create_trading_pair, 0, None)
                .await
                .expect("create trading pair failed");

            // bob is granted as attestor
            let bob_address =
                ink::primitives::AccountId::from(ink_e2e::bob::<PolkadotConfig>().account_id().0);
            let grant_role = build_message::<TestOracleRef>(contract_acc_id.clone())
                .call(|oracle| oracle.grant_role(ATTESTOR_ROLE, bob_address));
            client
                .call(&ink_e2e::alice(), grant_role, 0, None)
                .await
                .expect("grant bob as attestor failed");

            // a price request is sent
            let request_price = build_message::<TestOracleRef>(contract_acc_id.clone())
                .call(|oracle| oracle.request_price(trading_pair_id));
            let result = client
                .call(&ink_e2e::alice(), request_price, 0, None)
                .await
                .expect("Request price should be sent");
            // event : MessageQueued
            assert!(result.contains_event("Contracts", "ContractEmitted"));

            let request_id = result.return_value().expect("Request id not found");

            // then a response is received
            let payload = PriceResponseMessage {
                resp_type: TYPE_ERROR,
                trading_pair_id,
                price: None,
                err_no: Some(12356),
            };
            let actions = vec![
                HandleActionInput {
                    action_type: ACTION_REPLY,
                    id: None,
                    action: Some(payload.encode()),
                    address: None,
                },
                HandleActionInput {
                    action_type: ACTION_SET_QUEUE_HEAD,
                    id: Some(request_id + 1),
                    action: None,
                    address: None,
                },
            ];
            let rollup_cond_eq = build_message::<TestOracleRef>(contract_acc_id.clone())
                .call(|oracle| oracle.rollup_cond_eq(vec![], vec![], actions.clone()));
            let result = client
                .call(&ink_e2e::bob(), rollup_cond_eq, 0, None)
                .await
                .expect("we should proceed error message");
            // two events : MessageProcessedTo and PricesReceived
            assert!(result.contains_event("Contracts", "ContractEmitted"));

            Ok(())
        }

        #[ink_e2e::test]
        async fn test_bad_attestor(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // given
            let constructor = TestOracleRef::new();
            let contract_acc_id = client
                .instantiate("test_oracle", &ink_e2e::alice(), constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            // bob is not granted as attestor => it should not be able to send a message
            let rollup_cond_eq = build_message::<TestOracleRef>(contract_acc_id.clone())
                .call(|oracle| oracle.rollup_cond_eq(vec![], vec![], vec![]));
            let result = client.call(&ink_e2e::bob(), rollup_cond_eq, 0, None).await;
            assert!(
                result.is_err(),
                "only attestor should be able to send messages"
            );

            // bob is granted as attestor
            let bob_address =
                ink::primitives::AccountId::from(ink_e2e::bob::<PolkadotConfig>().account_id().0);
            let grant_role = build_message::<TestOracleRef>(contract_acc_id.clone())
                .call(|oracle| oracle.grant_role(ATTESTOR_ROLE, bob_address));
            client
                .call(&ink_e2e::alice(), grant_role, 0, None)
                .await
                .expect("grant bob as attestor failed");

            // then bob is abel to send a message
            let rollup_cond_eq = build_message::<TestOracleRef>(contract_acc_id.clone())
                .call(|oracle| oracle.rollup_cond_eq(vec![], vec![], vec![]));
            let result = client
                .call(&ink_e2e::bob(), rollup_cond_eq, 0, None)
                .await
                .expect("rollup cond eq failed");
            // no event
            assert!(!result.contains_event("Contracts", "ContractEmitted"));

            Ok(())
        }

        #[ink_e2e::test]
        async fn test_bad_messages(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // given
            let constructor = TestOracleRef::new();
            let contract_acc_id = client
                .instantiate("test_oracle", &ink_e2e::alice(), constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            let trading_pair_id = 10;

            // create the trading pair
            let create_trading_pair =
                build_message::<TestOracleRef>(contract_acc_id.clone()).call(|oracle| {
                    oracle.create_trading_pair(
                        trading_pair_id,
                        String::from("polkadot"),
                        String::from("usd"),
                    )
                });
            client
                .call(&ink_e2e::alice(), create_trading_pair, 0, None)
                .await
                .expect("create trading pair failed");

            // bob is granted as attestor
            let bob_address =
                ink::primitives::AccountId::from(ink_e2e::bob::<PolkadotConfig>().account_id().0);
            let grant_role = build_message::<TestOracleRef>(contract_acc_id.clone())
                .call(|oracle| oracle.grant_role(ATTESTOR_ROLE, bob_address));
            client
                .call(&ink_e2e::alice(), grant_role, 0, None)
                .await
                .expect("grant bob as attestor failed");

            let actions = vec![HandleActionInput {
                action_type: ACTION_REPLY,
                id: None,
                action: Some(58u128.encode()),
                address: None,
            }];
            let rollup_cond_eq = build_message::<TestOracleRef>(contract_acc_id.clone())
                .call(|oracle| oracle.rollup_cond_eq(vec![], vec![], actions.clone()));
            let result = client.call(&ink_e2e::bob(), rollup_cond_eq, 0, None).await;
            assert!(
                result.is_err(),
                "we should not be able to proceed bad messages"
            );

            Ok(())
        }

        #[ink_e2e::test]
        async fn test_optimistic_locking(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // given
            let constructor = TestOracleRef::new();
            let contract_acc_id = client
                .instantiate("test_oracle", &ink_e2e::alice(), constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            // bob is granted as attestor
            let bob_address =
                ink::primitives::AccountId::from(ink_e2e::bob::<PolkadotConfig>().account_id().0);
            let grant_role = build_message::<TestOracleRef>(contract_acc_id.clone())
                .call(|oracle| oracle.grant_role(ATTESTOR_ROLE, bob_address));
            client
                .call(&ink_e2e::alice(), grant_role, 0, None)
                .await
                .expect("grant bob as attestor failed");

            // then bob sends a message
            // from v0 to v1 => it's ok
            let conditions = vec![(123u8.encode(), None)];
            let updates = vec![(123u8.encode(), Some(1u128.encode()))];
            let rollup_cond_eq = build_message::<TestOracleRef>(contract_acc_id.clone())
                .call(|oracle| oracle.rollup_cond_eq(conditions.clone(), updates.clone(), vec![]));
            let result = client.call(&ink_e2e::bob(), rollup_cond_eq, 0, None).await;
            result.expect("This message should be proceed because the condition is met");

            // test idempotency it should fail because the conditions are not met
            let rollup_cond_eq = build_message::<TestOracleRef>(contract_acc_id.clone())
                .call(|oracle| oracle.rollup_cond_eq(conditions.clone(), updates.clone(), vec![]));
            let result = client.call(&ink_e2e::bob(), rollup_cond_eq, 0, None).await;
            assert!(
                result.is_err(),
                "This message should not be proceed because the condition is not met"
            );

            // from v1 to v2 => it's ok
            let conditions = vec![(123u8.encode(), Some(1u128.encode()))];
            let updates = vec![(123u8.encode(), Some(2u128.encode()))];
            let rollup_cond_eq = build_message::<TestOracleRef>(contract_acc_id.clone())
                .call(|oracle| oracle.rollup_cond_eq(conditions.clone(), updates.clone(), vec![]));
            let result = client.call(&ink_e2e::bob(), rollup_cond_eq, 0, None).await;
            result.expect("This message should be proceed because the condition is met");

            // test idempotency it should fail because the conditions are not met
            let rollup_cond_eq = build_message::<TestOracleRef>(contract_acc_id.clone())
                .call(|oracle| oracle.rollup_cond_eq(conditions.clone(), updates.clone(), vec![]));
            let result = client.call(&ink_e2e::bob(), rollup_cond_eq, 0, None).await;
            assert!(
                result.is_err(),
                "This message should not be proceed because the condition is not met"
            );

            Ok(())
        }

        #[ink_e2e::test]
        async fn test_prepare_meta_tx(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            let constructor = TestOracleRef::new();
            let contract_acc_id = client
                .instantiate("test_oracle", &ink_e2e::bob(), constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            // register the ecda public key because I am not able to retrieve if from the account id
            // Alice
            let from =
                ink::primitives::AccountId::from(ink_e2e::alice::<PolkadotConfig>().account_id().0);
            let ecdsa_public_key: [u8; 33] = hex_literal::hex!(
                "037051bed73458951b45ca6376f4096c85bf1a370da94d5336d04867cfaaad019e"
            );

            let register_ecdsa_public_key = build_message::<TestOracleRef>(contract_acc_id.clone())
                .call(|oracle| oracle.register_ecdsa_public_key(from, ecdsa_public_key));
            client
                .call(&ink_e2e::bob(), register_ecdsa_public_key, 0, None)
                .await
                .expect("We should be able to register the ecdsa public key");

            // prepare the meta transaction
            let data = u8::encode(&5);
            let prepare_meta_tx = build_message::<TestOracleRef>(contract_acc_id.clone())
                .call(|oracle| oracle.prepare(from, data.clone()));
            let result = client
                .call(&ink_e2e::bob(), prepare_meta_tx, 0, None)
                .await
                .expect("We should be able to prepare the meta tx");

            let (request, hash) = result
                .return_value()
                .expect("Expected value when preparing meta tx");

            assert_eq!(0, request.nonce);
            assert_eq!(from, request.from);
            assert_eq!(&data, &request.data);

            let expected_hash = hex_literal::hex!(
                "17cb4f6eae2f95ba0fbaee9e0e51dc790fe752e7386b72dcd93b9669450c2ccf"
            );
            assert_eq!(&expected_hash, &hash.as_ref());

            Ok(())
        }

        ///
        /// Test the meta transactions
        /// Charlie is the owner
        /// Alice is the attestor
        /// Bob is the sender (ie the payer)
        ///
        #[ink_e2e::test]
        async fn test_meta_tx_rollup_cond_eq(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            let constructor = TestOracleRef::new();
            let contract_acc_id = client
                .instantiate("test_oracle", &ink_e2e::charlie(), constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            // register the ecda public key because I am not able to retrieve if from the account id
            // Alice is the attestor
            let from =
                ink::primitives::AccountId::from(ink_e2e::alice::<PolkadotConfig>().account_id().0);
            let ecdsa_public_key: [u8; 33] = hex_literal::hex!(
                "037051bed73458951b45ca6376f4096c85bf1a370da94d5336d04867cfaaad019e"
            );

            let register_ecdsa_public_key = build_message::<TestOracleRef>(contract_acc_id.clone())
                .call(|oracle| oracle.register_ecdsa_public_key(from, ecdsa_public_key));
            client
                .call(&ink_e2e::charlie(), register_ecdsa_public_key, 0, None)
                .await
                .expect("We should be able to register the ecdsa public key");

            // add the role => it should be succeed
            let grant_role = build_message::<TestOracleRef>(contract_acc_id.clone())
                .call(|oracle| oracle.grant_role(ATTESTOR_ROLE, from));
            client
                .call(&ink_e2e::charlie(), grant_role, 0, None)
                .await
                .expect("grant the attestor failed");

            // prepare the meta transaction
            let data = RolupCondEqMethodParams::encode(&(vec![], vec![], vec![]));
            let prepare_meta_tx = build_message::<TestOracleRef>(contract_acc_id.clone())
                .call(|oracle| oracle.prepare(from, data.clone()));
            let result = client
                .call(&ink_e2e::bob(), prepare_meta_tx, 0, None)
                .await
                .expect("We should be able to prepare the meta tx");

            let (request, hash) = result
                .return_value()
                .expect("Expected value when preparing meta tx");

            assert_eq!(0, request.nonce);
            assert_eq!(from, request.from);
            assert_eq!(&data, &request.data);

            let expected_hash = hex_literal::hex!(
                "c91f57305dc05a66f1327352d55290a250eb61bba8e3cf8560a4b8e7d172bb54"
            );
            assert_eq!(&expected_hash, &hash.as_ref());

            // signature by Alice of previous hash
            let signature : [u8; 65] = hex_literal::hex!("c9a899bc8daa98fd1e819486c57f9ee889d035e8d0e55c04c475ca32bb59389b284d18d785a9db1bdd72ce74baefe6a54c0aa2418b14f7bc96232fa4bf42946600");

            // do the meta tx
            let meta_tx_rollup_cond_eq = build_message::<TestOracleRef>(contract_acc_id.clone())
                .call(|oracle| oracle.meta_tx_rollup_cond_eq(request.clone(), signature));
            client
                .call(&ink_e2e::bob(), meta_tx_rollup_cond_eq, 0, None)
                .await
                .expect("meta tx rollup cond eq should not failed");

            // do it again => it must failed
            let meta_tx_rollup_cond_eq = build_message::<TestOracleRef>(contract_acc_id.clone())
                .call(|oracle| oracle.meta_tx_rollup_cond_eq(request.clone(), signature));
            let result = client
                .call(&ink_e2e::bob(), meta_tx_rollup_cond_eq, 0, None)
                .await;
            assert!(
                result.is_err(),
                "This message should not be proceed because the nonce is obsolete"
            );

            Ok(())
        }
    }
}
