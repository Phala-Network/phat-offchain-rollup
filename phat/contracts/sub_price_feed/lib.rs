#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use ink_lang as ink;

pub use crate::sub_price_feed::*;

#[ink::contract(env = pink_extension::PinkEnvironment)]
mod sub_price_feed {
    use alloc::{format, string::String, vec, vec::Vec};
    use ink_storage::traits::{PackedLayout, SpreadLayout};
    use pink_extension as pink;
    use scale::{Decode, Encode};
    use serde::Deserialize;

    // To enable `(result).log_err("Reason")?`
    use pink::ResultExt;

    use phat_offchain_rollup::{
        clients::substrate::{claim_name, get_name_owner, SubstrateRollupClient},
        Action,
    };

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
        FailedToGetNameOwner,
        FailedToClaimName,

        FailedToGetStorage,
        FailedToCreateTransaction,
        FailedToSendTransaction,
        FailedToGetBlockHash,
        FailedToDecode,
        RollupAlreadyInitialized,
        RollupConfiguredByAnotherAccount,
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
            let actual_owner = get_name_owner(&config.rpc, &contract_id)
                .log_err("failed to get name owner")
                .or(Err(Error::FailedToGetNameOwner))?;
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
            .log_err("failed to claim name")
            .map(Some)
            .or(Err(Error::FailedToClaimName))
        }

        /// Fetches the price of a trading pair from CoinGecko
        fn fetch_coingecko_price(token0: &str, token1: &str) -> Result<u128> {
            use fixed::types::U64F64 as Fp;

            // Fetch the price from CoinGecko.
            //
            // Supported tokens are listed in the detailed documentation:
            // <https://www.coingecko.com/en/api/documentation>
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

        /// Feeds a price by a rollup transaction
        #[ink(message)]
        pub fn feed_price(&self) -> Result<Option<Vec<u8>>> {
            // Initialize a rollup client. The client tracks a "rollup transaction" that allows you
            // to read, write, and execute actions on the target chain with atomicity.
            let config = self.ensure_configured()?;
            let contract_id = self.env().account_id();
            let mut client =
                SubstrateRollupClient::new(&config.rpc, config.pallet_id, &contract_id)
                    .log_err("failed to create rollup client")
                    .or(Err(Error::FailedToCreateClient))?;

            // Business logic starts from here.

            // Get the price and respond as a rollup action.
            let price = Self::fetch_coingecko_price(&config.token0, &config.token1)?;
            let response = ResponseRecord {
                owner: self.owner.clone(),
                contract_id: contract_id.clone(),
                pair: format!("{}_{}", config.token0, config.token1),
                price,
                timestamp_ms: self.env().block_timestamp(),
            };
            // Attach an action to the tx by:
            client.action(Action::Reply(response.encode()));

            // An offchain rollup contract will get a dedicated kv store on the target blockchain.
            // The kv store can be accessed by the Phat Contract by:
            // - client.session.get(key)
            // - client.session.put(key, value)
            //
            // Note that all of the read, write, and custom actions are grouped as a transaction,
            // which is applied on the target blockchain atomically.

            // Business logic ends here.

            // Submit the transaction if it's not empty
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

        /// Returns the config reference or raise the error `NotConfigured`
        fn ensure_configured(&self) -> Result<&Config> {
            self.config.as_ref().ok_or(Error::NotConfigured)
        }
    }

    /// An price feed response to the chain
    ///
    /// It includes the data point (`pair`, `price`), and additional supportive information for the
    /// receiver to process the data. Must be aligned with the receiver on the substrate chain.
    #[derive(Debug, PartialEq, Eq, Encode, Decode, Clone)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct ResponseRecord {
        /// Owner of the oracle Phat Contract
        owner: AccountId,
        /// The contract address
        contract_id: AccountId,
        /// The trading pair name
        pair: String,
        /// The price, represented by a u128 integer (usually with 12 decimals)
        price: u128,
        /// The timestampe of the creation time
        timestamp_ms: u64,
    }

    // Define the structures to parse json like `{"token0":{"token1":1.23}}`
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
            // Secret key of test account `//Alice`
            let sk_alice = hex_literal::hex!(
                "e5be9a5092b81bca64be81d212e7f2f9eba183bb7a90954f7b76361f6edb5c0a"
            );

            let mut price_feed = SubPriceFeed::default();
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
        }
    }
}
