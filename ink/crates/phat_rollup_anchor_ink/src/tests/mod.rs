#![cfg_attr(not(feature = "std"), no_std, no_main)]
#![allow(clippy::inline_fn_without_body)]


#[openbrush::implementation(Ownable, AccessControl, KvStore, MetaTxReceiver, RollupAnchor)]
#[openbrush::contract]
pub mod test_contract {

    use crate::impls::kv_store::{self, *};
    use crate::impls::message_queue::{self, *};
    use crate::impls::meta_transaction::{self, *};
    use crate::impls::rollup_anchor::{self, *};
    use ink::env::debug_println;
    use openbrush::contracts::access_control::*;
    use openbrush::contracts::ownable::*;
    use openbrush::traits::Storage;

    #[ink(storage)]
    #[derive(Default, Storage)]
    pub struct MyContract {
        #[storage_field]
        ownable: ownable::Data,
        #[storage_field]
        access: access_control::Data,
        #[storage_field]
        kv_store: kv_store::Data,
        #[storage_field]
        meta_transaction: meta_transaction::Data,
    }

    impl MyContract {
        #[ink(constructor)]
        pub fn new(phat_attestor: AccountId) -> Self {
            let mut instance = Self::default();
            let caller = instance.env().caller();
            // set the owner of this contract
            instance._init_with_owner(caller);
            // set the admin of this contract
            instance._init_with_admin(caller);
            // grant the role manager
            instance
                .grant_role(MANAGER_ROLE, caller)
                .expect("Should grant the role MANAGER_ROLE");
            // grant the role attestor to the given address
            instance
                .grant_role(ATTESTOR_ROLE, phat_attestor)
                .expect("Should grant the role ATTESTOR_ROLE");
            instance
        }
    }

    #[default_impl(MetaTxReceiver)]
    #[openbrush::modifiers(access_control::only_role(MANAGER_ROLE))]
    fn register_ecdsa_public_key(){}

    #[default_impl(RollupAnchor)]
    #[openbrush::modifiers(access_control::only_role(ATTESTOR_ROLE))]
    fn rollup_cond_eq(){}

    impl rollup_anchor::MessageHandler for MyContract {
        fn on_message_received(&mut self, action: Vec<u8>) -> Result<(), RollupAnchorError> {
            debug_println!("Message received {:?}'", action);
            Ok(())
        }
    }

    impl rollup_anchor::EventBroadcaster for MyContract {
        fn emit_event_meta_tx_decoded(&self) {
            debug_println!("Meta transaction decoded");
        }
    }

    impl message_queue::EventBroadcaster for MyContract {
        fn emit_event_message_queued(&self, id: u32, data: Vec<u8>) {
            debug_println!(
                "Emit event 'message queued {{ id: {:?}, data: {:2x?} }}",
                id,
                data
            );
        }
        fn emit_event_message_processed_to(&self, id: u32) {
            debug_println!("Emit event 'message processed to {:?}'", id);
        }
    }
}
