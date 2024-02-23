#![cfg_attr(not(feature = "std"), no_std, no_main)]

extern crate alloc;

pub use crate::evm_price_feed::*;

#[ink::contract(env = pink::PinkEnvironment)]
mod evm_price_feed {
    use alloc::{format, string::String, vec, vec::Vec};
    use ink::storage::traits::StorageLayout;
    use pink_web3::types::{H160, U256};
    use scale::{Decode, Encode};
    use serde::Deserialize;

    // To enable `(result).log_err("Reason")?`
    use pink::ResultExt;

    use phat_offchain_rollup::{clients::evm::EvmRollupClient, Action};

    // Defined in TestOracle.sol
    const TYPE_RESPONSE: u32 = 0;
    const TYPE_FEED: u32 = 1;
    const TYPE_ERROR: u32 = 3;

    #[ink(storage)]
    pub struct EvmPriceFeed {
        owner: AccountId,
        config: Option<Config>,
    }

    #[derive(Encode, Decode, Debug)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, StorageLayout))]
    struct Config {
        /// The RPC endpoint of the target blockchain
        rpc: String,
        /// The rollup anchor address on the target blockchain
        anchor_addr: [u8; 20],
        /// Key for signing the rollup tx. If `sender_key` is None, use it to sign the EVM tx.
        attest_key: [u8; 32],
        /// Key for sending out the rollup meta-tx. None to fallback to the wallet based auth.
        sender_key: Option<[u8; 32]>,
        /// The first token in the trading pair
        token0: String,
        /// The sedon token in the trading pair
        token1: String,
        /// Submit price feed as this feed id
        feed_id: u32,
    }

    #[derive(Encode, Decode, Debug)]
    #[repr(u8)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        BadOrigin,
        NotConfigured,
        InvalidKeyLength,
        InvalidAddressLength,
        NoRequestInQueue,
        FailedToCreateClient,
        FailedToCommitTx,
        FailedToFetchPrice,

        FailedToGetStorage,
        FailedToCreateTransaction,
        FailedToSendTransaction,
        FailedToGetBlockHash,
        FailedToDecode,
        InvalidRequest,
    }

    impl From<Error> for U256 {
        fn from(err: Error) -> U256 {
            (err as u8).into()
        }
    }

    type Result<T> = core::result::Result<T, Error>;

    impl EvmPriceFeed {
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

        /// Configures the rollup target (admin only)
        #[ink(message)]
        pub fn config(
            &mut self,
            rpc: String,
            anchor_addr: Vec<u8>,
            attest_key: Vec<u8>,
            sender_key: Option<Vec<u8>>,
            token0: String,
            token1: String,
            feed_id: u32,
        ) -> Result<()> {
            self.ensure_owner()?;
            self.config = Some(Config {
                rpc,
                anchor_addr: anchor_addr
                    .try_into()
                    .or(Err(Error::InvalidAddressLength))?,
                attest_key: attest_key.try_into().or(Err(Error::InvalidKeyLength))?,
                sender_key: match sender_key {
                    Some(key) => Some(key.try_into().or(Err(Error::InvalidKeyLength))?),
                    None => None,
                },
                token0,
                token1,
                feed_id,
            });
            Ok(())
        }

        /// Transfers the ownership of the contract (admin only)
        #[ink(message)]
        pub fn transfer_ownership(&mut self, new_owner: AccountId) -> Result<()> {
            self.ensure_owner()?;
            self.owner = new_owner;
            Ok(())
        }

        /// Fetches the price of a trading pair from CoinGecko
        fn fetch_coingecko_price(token0: &str, token1: &str) -> Result<u128> {
            use fixed::types::U80F48 as Fp;

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
            // Parse to a fixed point and convert to u128 by rebasing to 1e18
            let fp = Fp::from_str(parsed.token0.token1)
                .log_err("failed to parse real number")
                .or(Err(Error::FailedToDecode))?;
            let f = fp * Fp::from_num(1_000_000_000_000_000_000u128);
            Ok(f.to_num())
        }

        /// Feeds a price by a rollup transaction
        #[ink(message)]
        pub fn feed_price(&self) -> Result<Option<Vec<u8>>> {
            use ethabi::Token;
            // Initialize a rollup client. The client tracks a "rollup transaction" that allows you
            // to read, write, and execute actions on the target chain with atomicity.
            let config = self.ensure_configured()?;
            let mut client = connect(&config)?;

            // Get the price and respond as a rollup action.
            let price = Self::fetch_coingecko_price(&config.token0, &config.token1)?;

            let payload = ethabi::encode(&[
                Token::Uint(TYPE_FEED.into()),
                Token::Uint(config.feed_id.into()),
                Token::Uint(price.into()),
            ]);

            // Attach an action to the tx by:
            client.action(Action::Reply(payload));

            // An offchain rollup contract will get a dedicated kv store on the target blockchain.
            // The kv store and the request queue can be accessed by the Phat Contract by:
            // - client.session.get(key)
            // - client.session.put(key, value)
            // - client.session.pop()
            //
            // Note that all of the read, write, and custom actions are grouped as a transaction,
            // which is applied on the target blockchain atomically.

            // Business logic ends here.

            // Submit the transaction if it's not empty
            maybe_submit_tx(client, &config)
        }

        /// Processes a price request by a rollup transaction
        #[ink(message)]
        pub fn answer_price(&self) -> Result<Option<Vec<u8>>> {
            use ethabi::Token;
            let config = self.ensure_configured()?;
            let mut client = connect(&config)?;
            let action = match Self::answer_price_inner(&mut client)? {
                PriceReponse::Response(rid, price) => ethabi::encode(&[
                    Token::Uint(TYPE_RESPONSE.into()),
                    Token::Uint(rid),
                    Token::Uint(price.into()),
                ]),
                PriceReponse::Error(rid, error) => ethabi::encode(&[
                    Token::Uint(TYPE_ERROR.into()),
                    Token::Uint(rid.unwrap_or_default()),
                    Token::Uint(error.into()),
                ]),
            };
            client.action(Action::Reply(action));
            maybe_submit_tx(client, &config)
        }

        /// Feeds a price data point to a customized rollup target.
        ///
        /// For dev purpose.
        #[ink(message)]
        pub fn feed_custom_price(
            &self,
            rpc: String,
            anchor_addr: [u8; 20],
            attest_key: [u8; 32],
            sender_key: Option<[u8; 32]>,
            feed_id: u32,
            price: u128,
        ) -> Result<Option<Vec<u8>>> {
            use ethabi::Token;
            let custom_config = Config {
                rpc,
                anchor_addr,
                attest_key,
                sender_key,
                token0: Default::default(),
                token1: Default::default(),
                feed_id,
            };
            let mut client = connect(&custom_config)?;
            let payload = ethabi::encode(&[
                Token::Uint(TYPE_FEED.into()),
                Token::Uint(feed_id.into()),
                Token::Uint(price.into()),
            ]);
            client.action(Action::Reply(payload));
            maybe_submit_tx(client, &custom_config)
        }

        fn answer_price_inner(client: &mut EvmRollupClient) -> Result<PriceReponse> {
            use ethabi::{ParamType, Token};
            use pink_kv_session::traits::QueueSession;
            // Get a request if presents
            let raw_req = client
                .session()
                .pop()
                .log_err("answer_price: failed to read queue")
                .or(Err(Error::FailedToGetStorage))?
                .ok_or(Error::NoRequestInQueue)?;
            // Decode the queue data by ethabi (u256, bytes)
            let Ok(decoded) = ethabi::decode(&[ParamType::Uint(32), ParamType::Bytes], &raw_req)
            else {
                return Ok(PriceReponse::Error(None, Error::FailedToDecode));
            };
            let [Token::Uint(rid), Token::Bytes(pair)] = decoded.as_slice() else {
                return Err(Error::FailedToDecode);
            };
            // Extract tokens from "token0/token1" string
            let pair = String::from_utf8_lossy(&pair);
            let pair_components: Vec<&str> = pair.split('/').collect();
            let [token0, token1] = pair_components.as_slice() else {
                return Ok(PriceReponse::Error(Some(*rid), Error::InvalidRequest));
            };
            pink::info!("Request received: ({token0}, {token1})");
            // Get the price and respond as a rollup action.
            let result = Self::fetch_coingecko_price(token0, token1);
            match result {
                Ok(price) => {
                    // Respond
                    pink::info!("Price: {price}");
                    Ok(PriceReponse::Response(*rid, price))
                }
                // Error when fetching the price. Could be
                Err(Error::FailedToDecode) => {
                    Ok(PriceReponse::Error(Some(*rid), Error::FailedToDecode))
                }
                Err(e) => return Err(e),
            }
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

    enum PriceReponse {
        Response(U256, u128),
        Error(Option<U256>, Error),
    }

    fn connect(config: &Config) -> Result<EvmRollupClient> {
        let anchor_addr: H160 = config.anchor_addr.into();
        EvmRollupClient::new(&config.rpc, anchor_addr)
            .log_err("failed to create rollup client")
            .or(Err(Error::FailedToCreateClient))
    }

    fn maybe_submit_tx(client: EvmRollupClient, config: &Config) -> Result<Option<Vec<u8>>> {
        use pink_web3::keys::pink::KeyPair;
        let maybe_submittable = client
            .commit()
            .log_err("failed to commit")
            .or(Err(Error::FailedToCommitTx))?;
        if let Some(submittable) = maybe_submittable {
            let attest_pair = KeyPair::from(config.attest_key);
            let tx_id = if let Some(sender_key) = config.sender_key {
                // Prefer to meta-tx
                let sender_pair = KeyPair::from(sender_key);
                submittable
                    .submit_meta_tx(&attest_pair, &sender_pair)
                    .log_err("failed to submit rollup meta-tx")
                    .or(Err(Error::FailedToSendTransaction))?
            } else {
                // Fallback to account-based authentication
                submittable
                    .submit(attest_pair)
                    .log_err("failed to submit rollup tx")
                    .or(Err(Error::FailedToSendTransaction))?
            };
            return Ok(Some(tx_id));
        }
        Ok(None)
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

        struct EnvVars {
            rpc: String,
            attest_key: Vec<u8>,
            sender_key: Option<Vec<u8>>,
            anchor: Vec<u8>,
        }

        fn get_env(key: &str) -> String {
            std::env::var(key).expect("env not found")
        }
        fn config() -> EnvVars {
            dotenvy::dotenv().ok();
            let rpc = get_env("RPC");
            let attest_key = hex::decode(get_env("ATTEST_KEY")).expect("hex decode failed");
            let sender_key = std::env::var("SENDER_KEY")
                .map(|s| hex::decode(s).expect("hex decode failed"))
                .ok();
            let anchor = hex::decode(get_env("ANCHOR")).expect("hex decode failed");
            EnvVars {
                rpc,
                attest_key,
                sender_key,
                anchor,
            }
        }

        #[ink::test]
        fn fixed_parse() {
            let _ = env_logger::try_init();
            pink_chain_extension::mock_ext::mock_all_ext();
            let p = EvmPriceFeed::fetch_coingecko_price("polkadot", "usd").unwrap();
            pink::warn!("Price: {p:?}");
        }

        #[ink::test]
        #[ignore]
        fn submit_price_feed() {
            let _ = env_logger::try_init();
            pink_chain_extension::mock_ext::mock_all_ext();
            let EnvVars {
                rpc,
                attest_key,
                sender_key,
                anchor,
            } = config();

            let mut price_feed = EvmPriceFeed::default();
            price_feed
                .config(
                    rpc,
                    anchor,
                    attest_key,
                    sender_key,
                    "polkadot".to_string(),
                    "usd".to_string(),
                    0,
                )
                .unwrap();

            let r = price_feed.feed_price().expect("failed to feed price");
            pink::warn!("feed price: {r:?}");
        }

        #[ink::test]
        #[ignore]
        fn answer_price_request() {
            let _ = env_logger::try_init();
            pink_chain_extension::mock_ext::mock_all_ext();
            let EnvVars {
                rpc,
                attest_key,
                sender_key,
                anchor,
            } = config();

            let mut price_feed = EvmPriceFeed::default();
            price_feed
                .config(
                    rpc,
                    anchor,
                    attest_key,
                    sender_key,
                    "polkadot".to_string(),
                    "usd".to_string(),
                    0,
                )
                .unwrap();

            let r = price_feed.answer_price().expect("failed to answer price");
            pink::warn!("answer price: {r:?}");
        }
    }
}
