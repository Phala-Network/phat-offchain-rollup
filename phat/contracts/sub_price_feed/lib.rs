#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use ink_lang as ink;

pub use crate::sub_price_feed::*;

#[ink::contract(env = pink_extension::PinkEnvironment)]
mod sub_price_feed {
    use alloc::{string::String, vec::Vec};
    use ink_storage::traits::{PackedLayout, SpreadLayout};
    use pink_extension as pink;
    use scale::{Decode, Encode};
    use serde::Deserialize;

    // To enable `(result).log_err("Reason")?`
    use pink::ResultExt;

    use phat_offchain_rollup::{Action, RollupTx};

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
        /// The RPC endpoint of the target blockchain
        rpc: String,
        /// The rollup anchor pallet id on the target blockchain
        pallet_id: u8,
        /// Key for submiting rollup transaction
        submit_key: [u8; 32],
        /// The first token in the trading pair
        token0: String,
        /// The sedon token in the trading pair
        token1: String,
    }

    #[derive(Encode, Decode, Debug)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        BadOrigin,
        NotConfigured,
        FailedToCreateClient,
        FailedToCommitTx,
        FailedToFetchPrice,

        FailedToGetStorage,
        FailedToCreateTransaction,
        FailedToSendTransaction,
        FailedToGetBlockHash,
        FailedToDecode,
        RollupAlreadyInitialized,
        RollupConfiguredByAnotherAccount,

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
        pub fn config(
            &mut self,
            rpc: String,
            pallet_id: u8,
            submit_key: [u8; 32],
            token0: String,
            token1: String,
        ) -> Result<()> {
            self.ensure_owner()?;
            self.config = Some(Config {
                rpc,
                pallet_id,
                submit_key,
                token0,
                token1,
            });
            Ok(())
        }

        /// Initializes the rollup on the target blockchain if it's not done yet
        ///
        /// First, look up if the name (contract id) is already claimed on the target chain. If
        /// not, just claim it with the `submit_key` and return the tx hash. If it's claimed, check
        /// if it's claimed by the `submit_key`. Return an error if the actual owner account is not
        /// the `submit_key`.
        #[ink(message)]
        pub fn maybe_init_rollup(&self) -> Result<Option<Vec<u8>>> {
            let config = self.ensure_configured()?;
            let contract_id = self.env().account_id();
            // Check if the rollup is initialized properly
            let actual_owner = get_name_owner(&config.rpc, &contract_id)?;
            if let Some(owner) = actual_owner {
                let pubkey = pink::ext()
                    .get_public_key(pink::chain_extension::SigType::Sr25519, &config.submit_key);
                if owner.encode() == pubkey {
                    return Ok(None);
                } else {
                    return Err(Error::RollupConfiguredByAnotherAccount);
                }
            }
            // Not initialized. Let's claim the name.
            claim_name(
                &config.rpc,
                config.pallet_id,
                &contract_id,
                &config.submit_key,
            )
            .map(Some)
        }

        fn fetch_price(token0: &str, token1: &str) -> Result<u128> {
            use fixed::types::U64F64 as Fp;

            // Fetch the price from CoinGecko.
            // (Detailed documentation: https://www.coingecko.com/en/api/documentation)
            let url = format!(
                "https://api.coingecko.com/api/v3/simple/price?ids={token0}&vs_currencies={token1}"
            );
            let headers = vec![("accept".into(), "application/json".into())];
            let resp = pink::http_get!(url, headers);
            if resp.status_code != 200 {
                return Err(Error::FailedToFetchPrice);
            }
            // The response looks like:
            //  {"polkadot":{"usd":5.41}}
            //
            // serde-json-core doesn't do well with dynamic keys. Therefore we play a trick here.
            // We replace the first token name by "token0" and the second token name by "token1".
            // Then we can get the json with constant field names. After the replacement, the above
            // sample json becomes:
            //  {"token0":{"token1":5.41}}
            let json = String::from_utf8(resp.body)
                .or(Err(Error::FailedToDecode))?
                .replace(token0, "token0")
                .replace(token1, "token1");
            let parsed: PriceResponse = pink_json::from_str(&json)
                .log_err("failed to parse json")
                .or(Err(Error::FailedToDecode))?;
            // Parse to a fixed point and convert to u128 by rebasing to 1e12
            let fp = Fp::from_str(parsed.token0.token1)
                .log_err("failed to parse real number")
                .or(Err(Error::FailedToDecode))?;
            let f = fp * Fp::from_num(1_000_000_000_000u64);
            Ok(f.to_num())
        }

        /// Feeds a price by a rollup tx
        #[ink(message)]
        pub fn feed_price(&self) -> Result<Option<Vec<u8>>> {
            let config = self.ensure_configured()?;
            let contract_id = self.env().account_id();
            let mut client =
                SubstrateRollupClient::new(&config.rpc, config.pallet_id, &contract_id)
                    .log_err("failed to create rollup client")
                    .or(Err(Error::FailedToCreateClient))?;

            // Get the price and respond as a rollup action.
            let price = Self::fetch_price(&config.token0, &config.token1)?;
            let response = ResponseRecord {
                owner: self.owner.clone(),
                contract_id: contract_id.clone(),
                pair: format!("{}_{}", config.token0, config.token1),
                price,
                timestamp_ms: self.env().block_timestamp(),
            };
            client.action(Action::Reply(response.encode()));

            // Submit the transaction
            let maybe_submittable = client
                .commit()
                .log_err("failed to commit")
                .or(Err(Error::FailedToCommitTx))?;
            if let Some(submittable) = maybe_submittable {
                let tx_id = submittable
                    .submit(&config.submit_key, 0)
                    .log_err("failed to submit rollup tx")
                    .or(Err(Error::FailedToSendTransaction))?;
                return Ok(Some(tx_id));
            }
            Ok(None)
        }

        /// Returns BadOrigin error if the caller is not the owner
        fn ensure_owner(&self) -> Result<()> {
            if self.env().caller() == self.owner {
                Ok(())
            } else {
                Err(Error::BadOrigin)
            }
        }

        fn ensure_configured(&self) -> Result<&Config> {
            self.config.as_ref().ok_or(Error::NotConfigured)
        }
    }

    #[derive(Debug, PartialEq, Eq, Encode, Decode, Clone)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct ResponseRecord {
        pub owner: AccountId,
        pub contract_id: AccountId,
        pub pair: String,
        pub price: u128,
        pub timestamp_ms: u64,
    }

    #[derive(Deserialize)]
    struct PriceResponse<'a> {
        #[serde(borrow)]
        token0: PriceReponseInner<'a>,
    }
    #[derive(Deserialize)]
    struct PriceReponseInner<'a> {
        #[serde(borrow)]
        token1: &'a str,
    }

    // #[derive(Deserialize)]
    // struct PriceResponse1 {
    //     token0: PriceReponseInner1,
    // }
    // #[derive(Deserialize)]
    // struct PriceReponseInner1 {
    //     token1: String,
    // }

    // -------------------------------------------------------------------------------------------

    use kv_session::{rollup, RwTracker, Session};

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
                .log_err("rollup snapshot: get storage failed")
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

        pub fn action(&mut self, action: Action) -> &mut Self {
            self.actions.push(action.encode());
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
                METHOD_ROLLUP,                      // method 1: rollup
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

    pub fn get_name_owner(rpc: &str, contract_id: &AccountId) -> Result<Option<AccountId>> {
        // Build key
        let prefix = subrpc::storage::storage_prefix("PhatRollupAnchor", "SubmitterByNames");
        let map_key: &[u8] = contract_id.as_ref();
        let storage_key = subrpc::storage::storage_map_blake2_128_prefix(&prefix, map_key);
        // Get storage
        let value =
            subrpc::get_storage(rpc, &storage_key, None).or(Err(Error::FailedToGetStorage))?;
        if let Some(value) = value {
            let owner: AccountId =
                Decode::decode(&mut &value[..]).or(Err(Error::FailedToDecode))?;
            return Ok(Some(owner));
        }
        return Ok(None);
    }

    pub fn claim_name(
        rpc: &str,
        pallet_id: u8,
        contract_id: &AccountId,
        secret_key: &[u8; 32],
    ) -> Result<Vec<u8>> {
        let signed_tx = subrpc::create_transaction(
            secret_key,
            "khala",
            rpc,
            pallet_id,
            METHOD_CLAIM_NAME,
            contract_id,
        )
        .or(Err(Error::FailedToCreateTransaction))?;

        let tx_hash =
            subrpc::send_transaction(rpc, &signed_tx).or(Err(Error::FailedToSendTransaction))?;

        pink::warn!("Sent = {}", hex::encode(&tx_hash),);
        Ok(tx_hash)
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use ink_lang as ink;

        #[ink::test]
        fn fixed_parse() {
            let _ = env_logger::try_init();
            pink_extension_runtime::mock_ext::mock_all_ext();
            let p = SubPriceFeed::fetch_price("polkadot", "usd").unwrap();
            pink::warn!("Price: {p:?}");
        }

        #[ink::test]
        fn default_works() {
            let _ = env_logger::try_init();
            pink_extension_runtime::mock_ext::mock_all_ext();

            let mut price_feed = SubPriceFeed::default();

            let account1 = AccountId::from([1u8; 32]);
            let sk_alice = hex_literal::hex!(
                "e5be9a5092b81bca64be81d212e7f2f9eba183bb7a90954f7b76361f6edb5c0a"
            );

            price_feed
                .config(
                    "http://127.0.0.1:39933".to_string(),
                    100,
                    sk_alice,
                    "polkadot".to_string(),
                    "usd".to_string(),
                )
                .unwrap();
            let r = price_feed.maybe_init_rollup().expect("failed to init");
            pink::warn!("init rollup: {r:?}");

            let r = price_feed.feed_price().expect("failed to feed price");
            pink::warn!("feed price: {r:?}");

            // price_feed.send_tx(&sk_alice);
            // price_feed.read_storage(&[1, 1]);

            // price_feed.config(rpc, anchor_addr).unwrap();

            // let res = price_feed.handle_req().unwrap();
            // println!("res: {:#?}", res);
        }
    }
}
