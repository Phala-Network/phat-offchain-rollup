#![cfg_attr(not(feature = "std"), no_std, no_main)]

extern crate alloc;
extern crate core;

#[ink::contract(env = pink_extension::PinkEnvironment)]
mod ink_price_feed {
    use alloc::{format, string::String, vec, vec::Vec};
    use ink::env::debug_println;

    use pink_extension::chain_extension::signing;
    use pink_extension::{debug, error, info, warn, ResultExt};
    use scale::{Decode, Encode};
    use serde::Deserialize;

    use phat_offchain_rollup::clients::ink::{Action, ContractId, InkRollupClient};

    pub type TradingPairId = u32;

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

    /// Type of response when the offchain rollup communicate with this contract
    const TYPE_ERROR: u8 = 0;
    const TYPE_RESPONSE: u8 = 10;
    const TYPE_FEED: u8 = 11;

    #[ink(storage)]
    pub struct InkPriceFeed {
        owner: AccountId,
        config: Option<Config>,
        /// Key for signing the rollup tx.
        attest_key: [u8; 32],
    }

    #[derive(Encode, Decode, Debug)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    struct Config {
        /// The RPC endpoint of the target blockchain
        rpc: String,
        pallet_id: u8,
        call_id: u8,
        /// The rollup anchor address on the target blockchain
        contract_id: ContractId,
        /// Key for sending out the rollup meta-tx. None to fallback to the wallet based auth.
        sender_key: Option<[u8; 32]>,
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
        FailedToCallRollup,
    }

    type Result<T> = core::result::Result<T, Error>;

    impl From<phat_offchain_rollup::Error> for Error {
        fn from(error: phat_offchain_rollup::Error) -> Self {
            error!("error in the rollup: {:?}", error);
            debug_println!("error in the rollup: {:?}", error);
            Error::FailedToCallRollup
        }
    }

    impl InkPriceFeed {
        #[ink(constructor)]
        pub fn default() -> Self {
            const NONCE: &[u8] = b"attest_key";
            let private_key = signing::derive_sr25519_key(NONCE);
            Self {
                owner: Self::env().caller(),
                attest_key: private_key[..32].try_into().expect("Invalid Key Length"),
                config: None,
            }
        }

        /// Gets the owner of the contract
        #[ink(message)]
        pub fn owner(&self) -> AccountId {
            self.owner
        }

        /// Gets the attestor address used by this rollup
        #[ink(message)]
        pub fn get_attest_address(&self) -> Vec<u8> {
            signing::get_public_key(&self.attest_key, signing::SigType::Sr25519)
        }

        /// Gets the attestor address used by this rollup in the meta transaction
        #[ink(message)]
        pub fn get_attest_address_meta_tx(&self) -> Vec<u8> {
            use ink::env::hash;
            let input = signing::get_public_key(&self.attest_key, signing::SigType::Ecdsa);
            let mut output = <hash::Blake2x256 as hash::HashOutput>::Type::default();
            ink::env::hash_bytes::<hash::Blake2x256>(&input, &mut output);
            output.to_vec()
        }

        /// Set attestor key.
        ///
        /// For dev purpose.
        #[ink(message)]
        pub fn set_attest_key(&mut self, attest_key: Option<Vec<u8>>) -> Result<()> {
            self.attest_key = match attest_key {
                Some(key) => key.try_into().or(Err(Error::InvalidKeyLength))?,
                None => {
                    const NONCE: &[u8] = b"attest_key";
                    let private_key = signing::derive_sr25519_key(NONCE);
                    private_key[..32]
                        .try_into()
                        .or(Err(Error::InvalidKeyLength))?
                }
            };
            Ok(())
        }

        /// Gets the sender address used by this rollup
        #[ink(message)]
        pub fn get_sender_address(&self) -> Option<Vec<u8>> {
            if let Some(Some(sender_key)) = self.config.as_ref().map(|c| c.sender_key.as_ref()) {
                let sender_key = signing::get_public_key(sender_key, signing::SigType::Sr25519);
                Some(sender_key)
            } else {
                None
            }
        }

        /// Gets the config
        #[ink(message)]
        pub fn get_target_contract(&self) -> Option<(String, u8, u8, ContractId)> {
            self.config
                .as_ref()
                .map(|c| (c.rpc.clone(), c.pallet_id, c.call_id, c.contract_id))
        }

        /// Configures the rollup target (admin only)
        #[ink(message)]
        pub fn config(
            &mut self,
            rpc: String,
            pallet_id: u8,
            call_id: u8,
            contract_id: Vec<u8>,
            sender_key: Option<Vec<u8>>,
        ) -> Result<()> {
            self.ensure_owner()?;
            self.config = Some(Config {
                rpc,
                pallet_id,
                call_id,
                contract_id: contract_id
                    .try_into()
                    .or(Err(Error::InvalidAddressLength))?,
                sender_key: match sender_key {
                    Some(key) => Some(key.try_into().or(Err(Error::InvalidKeyLength))?),
                    None => None,
                },
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
            let resp = pink_extension::http_get!(url, headers);
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
        pub fn feed_price_from_coingecko(
            &self,
            trading_pair_id: TradingPairId,
            token0: String,
            token1: String,
        ) -> Result<Option<Vec<u8>>> {
            let price = Self::fetch_coingecko_price(&token0, &token1)?;
            debug!("price: {}", price);
            self.feed_custom_price(trading_pair_id, price)
        }

        /// Feeds a price data point to a customized rollup target.
        ///
        /// For dev purpose.
        #[ink(message)]
        pub fn feed_custom_price(
            &self,
            trading_pair_id: TradingPairId,
            price: u128,
        ) -> Result<Option<Vec<u8>>> {
            // Initialize a rollup client. The client tracks a "rollup transaction" that allows you
            // to read, write, and execute actions on the target chain with atomicity.
            let config = self.ensure_configured()?;
            let mut client = connect(config)?;

            let payload = PriceResponseMessage {
                resp_type: TYPE_FEED,
                trading_pair_id,
                price: Some(price),
                err_no: None,
            };

            client.action(Action::Reply(payload.encode()));

            maybe_submit_tx(client, &self.attest_key, config.sender_key.as_ref())
        }

        /// Processes a price request by a rollup transaction
        #[ink(message)]
        pub fn answer_price(&self) -> Result<Option<Vec<u8>>> {
            let config = self.ensure_configured()?;
            let mut client = connect(config)?;

            // Get a request if presents
            let request: PriceRequestMessage = client
                .pop()
                .log_err("answer_price: failed to read queue")?
                .ok_or(Error::NoRequestInQueue)?;

            let response = Self::handle_request(&request)?;
            // Attach an action to the tx by:
            client.action(Action::Reply(response.encode()));

            maybe_submit_tx(client, &self.attest_key, config.sender_key.as_ref())
        }

        /// Processes a price request by a rollup transaction
        #[ink(message)]
        pub fn answer_price_with_config(
            &self,
            rpc: String,
            pallet_id: u8,
            call_id: u8,
            contract_id: Vec<u8>,
            sender_key: Option<Vec<u8>>,
        ) -> Result<Option<Vec<u8>>> {
            let config = &Config {
                rpc,
                pallet_id,
                call_id,
                contract_id: contract_id
                    .try_into()
                    .or(Err(Error::InvalidAddressLength))?,
                sender_key: match sender_key {
                    Some(key) => Some(key.try_into().or(Err(Error::InvalidKeyLength))?),
                    None => None,
                },
            };

            let mut client = connect(config)?;

            // Get a request if presents
            let request: PriceRequestMessage = client
                .pop()
                .log_err("answer_price: failed to read queue")?
                .ok_or(Error::NoRequestInQueue)?;

            let response = Self::handle_request(&request)?;
            // Attach an action to the tx by:
            client.action(Action::Reply(response.encode()));

            maybe_submit_tx(client, &self.attest_key, config.sender_key.as_ref())
        }

        fn handle_request(request: &PriceRequestMessage) -> Result<PriceResponseMessage> {
            let trading_pair_id = request.trading_pair_id;
            let token0 = request.token0.as_str();
            let token1 = request.token1.as_str();

            info!("Request received: ({trading_pair_id}, {token0}, {token1})");
            // Get the price and respond as a rollup action.
            match Self::fetch_coingecko_price(token0, token1) {
                Ok(price) => {
                    // Respond
                    info!("Price: {price}");
                    let response = PriceResponseMessage {
                        resp_type: TYPE_RESPONSE,
                        trading_pair_id,
                        price: Some(price),
                        err_no: None,
                    };
                    Ok(response)
                }
                // Error when fetching the price. Could be
                Err(Error::FailedToDecode) => {
                    warn!("Fail to decode the price");
                    let response = PriceResponseMessage {
                        resp_type: TYPE_ERROR,
                        trading_pair_id,
                        price: None,
                        err_no: Some(0),
                    };
                    Ok(response)
                }
                Err(e) => Err(e),
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

    fn connect(config: &Config) -> Result<InkRollupClient> {
        InkRollupClient::new(
            &config.rpc,
            config.pallet_id,
            config.call_id,
            &config.contract_id,
        )
        .log_err("failed to create rollup client")
        .or(Err(Error::FailedToCreateClient))
    }

    fn maybe_submit_tx(
        client: InkRollupClient,
        attest_key: &[u8; 32],
        sender_key: Option<&[u8; 32]>,
    ) -> Result<Option<Vec<u8>>> {
        let maybe_submittable = client
            .commit()
            .log_err("failed to commit")
            .map_err(|_| Error::FailedToCommitTx)?;

        if let Some(submittable) = maybe_submittable {
            let tx_id = if let Some(sender_key) = sender_key {
                // Prefer to meta-tx
                submittable
                    .submit_meta_tx(attest_key, sender_key)
                    .log_err("failed to submit rollup meta-tx")?
            } else {
                // Fallback to account-based authentication
                submittable
                    .submit(attest_key)
                    .log_err("failed to submit rollup tx")?
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
        use ink::env::debug_println;
        use pink_extension::chain_extension::SigType;

        use super::*;

        struct EnvVars {
            /// The RPC endpoint of the target blockchain
            rpc: String,
            pallet_id: u8,
            call_id: u8,
            /// The rollup anchor address on the target blockchain
            contract_id: ContractId,
            /// When we want to manually set the attestor key for signing the message (only dev purpose)
            attest_key: Vec<u8>,
            /// When we want to use meta tx
            sender_key: Option<Vec<u8>>,
        }

        fn get_env(key: &str) -> String {
            std::env::var(key).expect("env not found")
        }

        fn config() -> EnvVars {
            dotenvy::dotenv().ok();
            let rpc = get_env("RPC");
            let pallet_id: u8 = get_env("PALLET_ID").parse().expect("u8 expected");
            let call_id: u8 = get_env("CALL_ID").parse().expect("u8 expected");
            let contract_id: ContractId = hex::decode(get_env("CONTRACT_ID"))
                .expect("hex decode failed")
                .try_into()
                .expect("incorrect length");
            let attest_key = hex::decode(get_env("ATTEST_KEY")).expect("hex decode failed");
            let sender_key = std::env::var("SENDER_KEY")
                .map(|s| hex::decode(s).expect("hex decode failed"))
                .ok();

            EnvVars {
                rpc: rpc.to_string(),
                pallet_id,
                call_id,
                contract_id: contract_id.into(),
                attest_key,
                sender_key,
            }
        }

        fn init_contract() -> InkPriceFeed {
            let EnvVars {
                rpc,
                pallet_id,
                call_id,
                contract_id,
                attest_key,
                sender_key,
            } = config();

            let mut price_feed = InkPriceFeed::default();
            price_feed
                .config(rpc, pallet_id, call_id, contract_id.into(), sender_key)
                .unwrap();
            price_feed.set_attest_key(Some(attest_key)).unwrap();

            price_feed
        }

        #[ink::test]
        fn test_update_attestor_key() {
            let _ = env_logger::try_init();
            pink_extension_runtime::mock_ext::mock_all_ext();

            let mut price_feed = InkPriceFeed::default();

            // Secret key and address of Alice in localhost
            let sk_alice: [u8; 32] = [0x01; 32];
            let address_alice = hex_literal::hex!(
                "189dac29296d31814dc8c56cf3d36a0543372bba7538fa322a4aebfebc39e056"
            );

            let initial_attestor_address = price_feed.get_attest_address();
            assert_ne!(address_alice, initial_attestor_address.as_slice());

            price_feed.set_attest_key(Some(sk_alice.into())).unwrap();

            let attestor_address = price_feed.get_attest_address();
            assert_eq!(address_alice, attestor_address.as_slice());

            price_feed.set_attest_key(None).unwrap();

            let attestor_address = price_feed.get_attest_address();
            assert_eq!(initial_attestor_address, attestor_address);
        }

        #[ink::test]
        #[ignore = "the target contract must be deployed in local node or shibuya"]
        fn feed_custom_price() {
            let _ = env_logger::try_init();
            pink_extension_runtime::mock_ext::mock_all_ext();

            let price_feed = init_contract();

            let _token0 = "pha".to_string();
            let _token1 = "usd".to_string();
            let trading_pair_id: TradingPairId = 13;
            let value: u128 = 1_500_000_000_000_000_000;

            price_feed
                .feed_custom_price(trading_pair_id, value)
                .unwrap();
        }

        #[ink::test]
        fn fetch_coingecko_price() {
            let _ = env_logger::try_init();
            pink_extension_runtime::mock_ext::mock_all_ext();

            let token0 = "polkadot".to_string();
            let token1 = "usd".to_string();

            let value =
                InkPriceFeed::fetch_coingecko_price(token0.as_str(), token1.as_str()).unwrap();
            debug_println!("value {}/{} = {}", token0, token1, value);
        }

        #[ink::test]
        #[ignore = "the target contract must be deployed in local node or shibuya"]
        fn feed_price_from_coingecko() {
            let _ = env_logger::try_init();
            pink_extension_runtime::mock_ext::mock_all_ext();

            let price_feed = init_contract();

            let token0 = "polkadot".to_string();
            let token1 = "usd".to_string();
            let trading_pair_id: TradingPairId = 11;

            price_feed
                .feed_price_from_coingecko(trading_pair_id, token0, token1)
                .unwrap();
        }

        #[ink::test]
        #[ignore = "the target contract must be deployed in local node or shibuya"]
        fn answer_price_request() {
            let _ = env_logger::try_init();
            pink_extension_runtime::mock_ext::mock_all_ext();

            let price_feed = init_contract();

            let r = price_feed.answer_price().expect("failed to answer price");
            debug_println!("answer price: {r:?}");
        }

        #[ink::test]
        fn test_sign_and_ecdsa_recover() {
            let _ = env_logger::try_init();
            pink_extension_runtime::mock_ext::mock_all_ext();

            // Secret key of test account `//Alice`
            let private_key = hex_literal::hex!(
                "e5be9a5092b81bca64be81d212e7f2f9eba183bb7a90954f7b76361f6edb5c0a"
            );

            let message = hex_literal::hex!(
                "01e552298e47454041ea31273b4b630c64c104e4514aa3643490b8aaca9cf8edffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff000000000000000000000000000000000405"
            );
            debug_println!("message: {:02x?}", message);

            let hash = hex_literal::hex!(
                "9eb948928cf669f05801b791e5770419f1184637cf2ff3e8124c92e44d45e76f"
            );
            debug_println!("hash: {:02x?}", hash);

            let signature1 = signing::ecdsa_sign_prehashed(&private_key, hash);
            debug_println!("signature1: {:02x?}", signature1);

            let signature2 = signing::sign(&message, &private_key, SigType::Ecdsa);
            debug_println!("signature2: {:02x?}", signature2);

            assert_eq!(signature1.to_vec(), signature2);

            // at the moment we can only verify ecdsa signatures
            let mut public_key = [0u8; 33];
            ink::env::ecdsa_recover(&signature1.try_into().unwrap(), &hash, &mut public_key)
                .unwrap();
            debug_println!("public_key: {:02x?}", public_key);

            let ecdsa_public_key = signing::get_public_key(&private_key, SigType::Ecdsa);
            debug_println!("public_key (ecdsa): {:02x?}", ecdsa_public_key);

            let ecdsa_public_key: [u8; 33] = ecdsa_public_key.try_into().unwrap();
            assert_eq!(public_key, ecdsa_public_key);
        }
    }
}
