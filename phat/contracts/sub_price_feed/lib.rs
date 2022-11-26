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
        FailedToGetBlockHash,

        SessionFailedToDecode,
        SessionFailedToGetStorage,
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

        /// Returns BadOrigin error if the caller is not the owner
        fn ensure_owner(&self) -> Result<()> {
            if self.env().caller() == self.owner {
                Ok(())
            } else {
                Err(Error::BadOrigin)
            }
        }
    }

    use kv_session::{rollup, traits::KvSession, RwTracker, Session};

    use kv_session::traits::{BumpVersion, KvSnapshot};
    use primitive_types::H256;

    const METHOD_CLAIM_NAME: u8 = 0u8;
    const METHOD_ROLLUP: u8 = 1u8;
    struct SubstrateSnapshot<'a> {
        rpc: &'a str,
        pallet_id: u8,
        contract_id: &'a AccountId,
        at: H256,
    }

    impl<'a> SubstrateSnapshot<'a> {
        pub fn new(rpc: &'a str, pallet_id: u8, contract_id: &'a AccountId) -> Result<Self> {
            let hash = subrpc::get_block_hash(rpc, None).or(Err(Error::FailedToGetBlockHash))?;
            Ok(SubstrateSnapshot {
                rpc,
                pallet_id,
                contract_id,
                at: hash,
            })
        }
    }

    impl<'a> KvSnapshot for SubstrateSnapshot<'a> {
        type Key = Vec<u8>;
        type Value = Vec<u8>;

        fn get(
            &self,
            key: &impl ToOwned<Owned = Self::Key>,
        ) -> kv_session::Result<Option<Self::Value>> {
            let prefix = subrpc::storage::storage_prefix("PhatRollupAnchor", "States");
            let key1: &[u8] = self.contract_id.as_ref();
            let key2: &[u8] = &key.to_owned().encode();
            let storage_key =
                subrpc::storage::storage_double_map_blake2_128_prefix(&prefix, key1, key2);
            let value = subrpc::get_storage(self.rpc, &storage_key, None)
                .or(Err(kv_session::Error::FailedToGetStorage))?;
            pink::warn!(
                "Storage[{}][{}] = {:?}",
                hex::encode(key1),
                hex::encode(key2),
                value.clone().map(|data| hex::encode(&data))
            );
            match value {
                Some(raw) => Ok(Some(
                    Vec::<u8>::decode(&mut &raw[..]).or(Err(kv_session::Error::FailedToDecode))?,
                )),
                None => Ok(None),
            }
        }

        fn snapshot_id(&self) -> kv_session::Result<Self::Value> {
            Ok(self.at.encode())
        }
    }
    impl<'a> BumpVersion<Vec<u8>> for SubstrateSnapshot<'a> {
        fn bump_version(&self, version: Option<Vec<u8>>) -> kv_session::Result<Vec<u8>> {
            match version {
                Some(v) => {
                    let ver =
                        u32::decode(&mut &v[..]).or(Err(kv_session::Error::FailedToDecode))?;
                    Ok((ver + 1).encode())
                }
                None => Ok(1u32.encode()),
            }
        }
    }

    struct SubstrateRollupClient<'a> {
        rpc: &'a str,
        pallet_id: u8,
        contract_id: &'a AccountId,
        actions: Vec<Vec<u8>>,
        session: Session<SubstrateSnapshot<'a>, Vec<u8>, Vec<u8>, RwTracker<Vec<u8>>>,
    }

    struct SubmittableRollupTx<'a> {
        rpc: &'a str,
        pallet_id: u8,
        contract_id: &'a AccountId,
        tx: RollupTx,
    }

    impl<'a> SubstrateRollupClient<'a> {
        pub fn new(rpc: &'a str, pallet_id: u8, contract_id: &'a AccountId) -> Result<Self> {
            let kvdb = SubstrateSnapshot::new(rpc, pallet_id, contract_id)?;
            let access_tracker = RwTracker::<Vec<u8>>::new();
            Ok(SubstrateRollupClient {
                rpc,
                pallet_id,
                contract_id,
                actions: Default::default(),
                session: Session::new(kvdb, access_tracker),
            })
        }

        pub fn action(&mut self, action: Vec<u8>) -> &mut Self {
            self.actions.push(action);
            self
        }

        pub fn commit(self) -> Result<Option<SubmittableRollupTx<'a>>> {
            let (session_tx, kvdb) = self.session.commit();
            let raw_tx = rollup::rollup(
                kvdb,
                session_tx,
                rollup::VersionLayout::Standalone {
                    key_postfix: b":ver".to_vec(),
                },
            )
            .map_err(Self::convert_err)?;
            pink::warn!("RawTx: {raw_tx:?}");

            if raw_tx.updates.is_empty() && self.actions.is_empty() {
                return Ok(None);
            }

            let tx = phat_offchain_rollup::RollupTx {
                conds: raw_tx
                    .conditions
                    .into_iter()
                    .map(|(k, v)| phat_offchain_rollup::Cond::Eq(k.into(), v.map(Into::into)))
                    .collect(),
                actions: self.actions.into_iter().map(Into::into).collect(),
                updates: raw_tx
                    .updates
                    .into_iter()
                    .map(|(k, v)| (k.into(), v.map(Into::into)))
                    .collect(),
            };

            Ok(Some(SubmittableRollupTx {
                rpc: self.rpc,
                pallet_id: self.pallet_id,
                contract_id: self.contract_id,
                tx,
            }))
        }

        fn convert_err(err: kv_session::Error) -> Error {
            match err {
                kv_session::Error::FailedToDecode => Error::SessionFailedToDecode,
                kv_session::Error::FailedToGetStorage => Error::SessionFailedToGetStorage,
            }
        }
    }

    impl<'a> SubmittableRollupTx<'a> {
        fn submit(self, secret_key: &[u8; 32], nonce: u128) -> Result<Vec<u8>> {
            let signed_tx = subrpc::create_transaction(
                secret_key,
                "khala",
                self.rpc,
                self.pallet_id,                     // pallet idx
                1,                                  // method 1: rollup
                (self.contract_id, self.tx, nonce), // (name, tx, nonce)
            )
            .or(Err(Error::FailedToCreateTransaction))?;

            pink::warn!("ContractId = {}", hex::encode(self.contract_id),);
            pink::warn!("SignedTx = {}", hex::encode(&signed_tx),);

            let tx_hash = subrpc::send_transaction(self.rpc, &signed_tx)
                .or(Err(Error::FailedToSendTransaction))?;

            pink::warn!("Sent = {}", hex::encode(&tx_hash),);
            Ok(tx_hash)
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

            // let mut price_feed = SubPriceFeed::default();

            let account1 = AccountId::from([1u8; 32]);
            let sk_alice = hex_literal::hex!(
                "e5be9a5092b81bca64be81d212e7f2f9eba183bb7a90954f7b76361f6edb5c0a"
            );

            // Create a rollup session
            let mut client = SubstrateRollupClient::new("http://127.0.0.1:39933", 100, &account1)
                .expect("failed to create rollup client");

            // Using the rollup
            {
                let v = client
                    .session
                    .get(&[1u8; 2].to_vec())
                    .expect("failed to read storage");
                client.session.put(&[1u8; 2].to_vec(), [3u8; 2].to_vec());
            }

            // Submit the transaction
            let maybe_submittable = client.commit().expect("failed to commit");
            if let Some(submittable) = maybe_submittable {
                let tx_id = submittable.submit(&sk_alice, 0).expect("failed to submit");
                pink::warn!("Rollup Tx Submitted: {:?}", tx_id);
            }

            // price_feed.send_tx(&sk_alice);
            // price_feed.read_storage(&[1, 1]);

            // price_feed.config(rpc, anchor_addr).unwrap();

            // let res = price_feed.handle_req().unwrap();
            // println!("res: {:#?}", res);
        }
    }
}
