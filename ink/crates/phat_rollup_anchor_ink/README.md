# Phat Rollup Anchor for Ink smart contract 

Library for Ink! smart contract to help you build [Phat Rollup Anchor ](https://github.com/Phala-Network/phat-offchain-rollup/
)deployed on the Substrate pallet Contracts.
This library uses the [OpenBrush](https://learn.brushfam.io/docs/OpenBrush) library with teh features `ownable` and `access_control`
It provides the following traits for:
 - kv_store: key-value store that allows offchain Phat Contracts to perform read/write operations.
 - message_queue: Message Queue, enabling a request-response programming model for the smart-contract while ensuring that each request received exactly one response. It uses the KV Store to save the messages. 
 - rollup_anchor: Use the kv-store and the message queue to allow offchain's rollup transactions.
 - meta_transaction : Allow the offchain Phat Contract to do transactions without paying the gas fee. The fee will be paid by a third party (the relayer).


## Build the crate

To build the crate:

```bash
cargo build
```
## Run the integration tests

To run the integration tests:

```bash
cargo test
```

## Use this crate in your library

### Add the dependencies

The default toml of your project

```toml
[dependencies]
ink = { version = "4.2.0", default-features = false }

scale = { package = "parity-scale-codec", version = "3", default-features = false, features = ["derive"] }
scale-info = { version = "2", default-features = false, features = ["derive"], optional = true }

# OpenBrush dependency
openbrush = { git = "https://github.com/727-Ventures/openbrush-contracts", version = "4.0.0-beta", features = ["ownable", "access_control"], default-features = false }

# Phat Rollup Anchor dependency
phat_rollup_anchor_ink = { path = "phat-rollup-anchor-ink", default-features = false}

[features]
default = ["std"]
std = [
    "ink/std",
    "scale/std",
    "scale-info/std",
    "openbrush/std",
    "phat_rollup_anchor_ink/std",
]
```

### Add imports

Use `openbrush::contract` macro instead of `ink::contract`. 
Import everything from `openbrush::contracts::access_control`, `openbrush::contracts::ownable`, `phat_rollup_anchor_ink::traits::kv_store`, `phat_rollup_anchor_ink::traits::message_queue`, `phat_rollup_anchor_ink::traits::meta_transaction`, `phat_rollup_anchor_ink::traits::rollup_anchor`.

```rust
#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[openbrush::implementation(Ownable, AccessControl)]
#[openbrush::contract]
pub mod test_oracle {
    
    use openbrush::contracts::access_control::*;
    use openbrush::contracts::ownable::*;
    use openbrush::traits::Storage;
    use scale::{Decode, Encode};

    use phat_rollup_anchor_ink::traits::{
        kv_store, kv_store::*,
        message_queue, message_queue::*,
        meta_transaction, meta_transaction::*,
        rollup_anchor, rollup_anchor::*
    };
...
```

### Define storage

Declare storage struct and declare the fields related to the modules.

```rust
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
    ...
}
```

### Inherit logic
Inherit implementation of the traits. You can customize (override) methods in this `impl` block.

```rust
impl KVStore for TestOracle {}
impl MessageQueue for TestOracle {}
impl MetaTxReceiver for TestOracle {}
impl RollupAnchor for TestOracle {}
```

### Define constructor
```rust
impl TestOracle {
    #[ink(constructor)]
    pub fn new() -> Self {
        let mut instance = Self::default();
        let caller = instance.env().caller();
        // set the owner of this contract
        ownable::Internal::_init_with_owner(&mut instance, caller);
        // set the admin of this contract
        access_control::Internal::_init_with_admin(&mut instance, Some(caller));
        instance
    }
}
```

### Traits to implement

### Trait for the message queue
Implement the `message_queue::EventBroadcaster` trait to emit the events when a message is pushed in the queue and when a message is proceeded. 
If you don't want to emit the events, you can put an empty block in the methods `emit_event_message_queued` and `emit_event_message_processed_to`.

```rust
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

impl message_queue::EventBroadcaster for TestOracle {

    fn emit_event_message_queued(&self, id: u32, data: Vec<u8>){
        self.env().emit_event(MessageQueued { id, data });
    }

    fn emit_event_message_processed_to(&self, id: u32){
        self.env().emit_event(MessageProcessedTo { id });
    }

}
```
### Traits for the rollup anchor
Implement the `rollup_anchor::MessageHandler` trait to put your business logic when a message is received.
Here an example when the Oracle receives a message with the price feed. 

```rust
impl rollup_anchor::MessageHandler for TestOracle {
    fn on_message_received(&mut self, action: Vec<u8>) -> Result<(), RollupAnchorError> {

        // parse the response
        let message: PriceResponseMessage = Decode::decode(&mut &action[..])
            .or(Err(RollupAnchorError::FailedToDecode))?;

        // handle the response
        if message.resp_type == TYPE_RESPONSE || message.resp_type == TYPE_FEED {  // we received the price
            // register the info
            let mut trading_pair = self.trading_pairs.get(&message.trading_pair_id).unwrap_or_default();
            trading_pair.value = message.price.unwrap_or_default();
            trading_pair.nb_updates += 1;
            trading_pair.last_update = self.env().block_timestamp();
            self.trading_pairs.insert(&message.trading_pair_id, &trading_pair);

            // emmit te event
            self.env().emit_event(
                PriceReceived {
                    trading_pair_id: message.trading_pair_id,
                    price: message.price.unwrap_or_default(),
                }
            );

        } else if message.resp_type == TYPE_ERROR { // we received an error
            self.env().emit_event(
                ErrorReceived {
                    trading_pair_id: message.trading_pair_id,
                    err_no: message.err_no.unwrap_or_default()
                }
            );
        } else {
            // response type unknown
            return Err(RollupAnchorError::UnsupportedAction);
        }

        Ok(())
    }
}
```

### Final code 
Here the final code of the Price Oracle.

```rust
#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[openbrush::implementation(Ownable, AccessControl)]
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

    use phat_rollup_anchor_ink::traits::{
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

    impl TestOracle {
        #[ink(constructor)]
        pub fn new() -> Self {
            let mut instance = Self::default();
            let caller = instance.env().caller();
            // set the owner of this contract
            ownable::Internal::_init_with_owner(&mut instance, caller);
            // set the admin of this contract
            access_control::Internal::_init_with_admin(&mut instance, Some(caller));
            // grant the role manager
            AccessControl::grant_role(&mut instance, MANAGER_ROLE, Some(caller))
                .expect("Should grant the role MANAGER_ROLE");
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
                    self.push_message(&message)?
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
            AccessControl::grant_role(self, ATTESTOR_ROLE, Some(account_id))?;
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

    impl KvStore for TestOracle {}

    impl MessageQueue for TestOracle {}

    impl MetaTxReceiver for TestOracle {}

    impl RollupAnchor for TestOracle {}

    impl rollup_anchor::MessageHandler for TestOracle {
        fn on_message_received(&mut self, action: Vec<u8>) -> Result<(), RollupAnchorError> {
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
    }

    impl rollup_anchor::EventBroadcaster for TestOracle {
        fn emit_event_meta_tx_decoded(&self) {
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

    impl message_queue::EventBroadcaster for TestOracle {
        fn emit_event_message_queued(&self, id: u32, data: Vec<u8>) {
            self.env().emit_event(MessageQueued { id, data });
        }

        fn emit_event_message_processed_to(&self, id: u32) {
            self.env().emit_event(MessageProcessedTo { id });
        }
    }
}
```