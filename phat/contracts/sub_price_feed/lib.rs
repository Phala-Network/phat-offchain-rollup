#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use ink_lang as ink;

pub use crate::sub_price_feed::*;

#[ink::contract(env = pink_extension::PinkEnvironment)]
mod sub_price_feed {
    use alloc::{string::String, vec::Vec};
    use ink_storage::traits::{PackedLayout, SpreadLayout};
    use pink_extension as pink;
    use primitive_types::H160;
    use scale::{Decode, Encode};

    use phat_offchain_rollup::RollupTx;

    #[ink(storage)]
    pub struct SubPriceFeed {
        owner: AccountId,
        config: Option<Config>,
    }

    #[derive(Encode, Decode, Debug, PackedLayout, SpreadLayout)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink_storage::traits::StorageLayout)
    )]
    struct Config {
        rpc: String,
    }

    #[derive(Encode, Decode, Debug)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        BadOrigin,
        NotConfigurated,
        FailedToGetStorage,
        FailedToCreateTransaction,
        FailedToSendTransaction,
    }

    type Result<T> = core::result::Result<T, Error>;

    impl SubPriceFeed {
        #[ink(constructor)]
        pub fn default() -> Self {
            Self {
                owner: Self::env().caller(),
                config: None,
            }
        }

        /// Gets the owner of the contract
        #[ink(message)]
        pub fn owner(&self) -> AccountId {
            self.owner
        }

        /// Configures the rollup target
        #[ink(message)]
        pub fn config(&mut self, rpc: String) -> Result<()> {
            self.ensure_owner()?;
            self.config = Some(Config { rpc });
            Ok(())
        }

        fn read_storage(&self, key: &[u8]) -> Result<Vec<u8>> {
            let rpc = "http://127.0.0.1:39933";
            let prefix = subrpc::storage::storage_prefix("PhatRollupAnchor", "States");
            let contract_id = self.env().account_id();
            let key1: &[u8] = contract_id.as_ref();
            let key2: &[u8] = &key.to_vec().encode();
            let storage_key =
                subrpc::storage::storage_double_map_blake2_128_prefix(&prefix, key1, key2);
            let value =
                subrpc::get_storage(rpc, &storage_key, None).or(Err(Error::FailedToGetStorage))?;
            pink::warn!(
                "Storage[{}][{}] = {:?}",
                hex::encode(key1),
                hex::encode(key2),
                value.map(|data| hex::encode(&data))
            );
            Ok(Default::default())
        }

        fn send_tx(&self, sk: &[u8; 32]) -> Result<Vec<u8>> {
            let rpc = "http://127.0.0.1:39933";
            let key1: Vec<u8> = vec![1, 1];
            let key2: Vec<u8> = vec![2, 2];

            let tx = RollupTx {
                conds: vec![],
                actions: vec![],
                updates: vec![
                    (key1.clone().into(), Some(key1.into())),
                    (key2.clone().into(), Some(key2.into())),
                ],
            };

            let contract_id = self.env().account_id();
            let signed_tx = subrpc::create_transaction(
                sk,
                "khala",
                rpc,
                100,                      // pallet idx
                1,                        // method 1: rollup
                (contract_id, tx, 1u128), // (name, tx, nonce)
            )
            .or(Err(Error::FailedToCreateTransaction))?;

            pink::warn!("ContractId = {}", hex::encode(&contract_id),);
            pink::warn!("SignedTx = {}", hex::encode(&signed_tx),);

            let tx_hash = subrpc::send_transaction(rpc, &signed_tx)
                .or(Err(Error::FailedToSendTransaction))?;

            pink::warn!("Sent = {}", hex::encode(&tx_hash),);

            Ok(Default::default())
        }

        /// Returns BadOrigin error if the caller is not the owner
        fn ensure_owner(&self) -> Result<()> {
            if self.env().caller() == self.owner {
                Ok(())
            } else {
                Err(Error::BadOrigin)
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use ink_lang as ink;

        fn consts() -> (String, H160) {
            use std::env;
            dotenvy::dotenv().ok();
            /*
             Deployed {
                anchor: '0xb3083F961C729f1007a6A1265Ae6b97dC2Cc16f2',
                oracle: '0x8Bf50F8d0B62017c9B83341CB936797f6B6235dd'
            }
            */
            let rpc = env::var("RPC").unwrap();
            let anchor_addr: [u8; 20] =
                hex::decode(env::var("ANCHOR_ADDR").expect("env not found"))
                    .expect("hex decode failed")
                    .try_into()
                    .expect("invald length");
            let anchor_addr: H160 = anchor_addr.into();
            (rpc, anchor_addr)
        }

        #[ink::test]
        fn default_works() {
            let _ = env_logger::try_init();
            pink_extension_runtime::mock_ext::mock_all_ext();

            let (rpc, anchor_addr) = consts();

            let mut price_feed = SubPriceFeed::default();
            let sk_alice = hex_literal::hex!(
                "e5be9a5092b81bca64be81d212e7f2f9eba183bb7a90954f7b76361f6edb5c0a"
            );
            price_feed.send_tx(&sk_alice);
            price_feed.read_storage(&[1, 1]);
            // price_feed.config(rpc, anchor_addr).unwrap();

            // let res = price_feed.handle_req().unwrap();
            // println!("res: {:#?}", res);
        }
    }
}
