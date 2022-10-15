#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use ink_lang as ink;

#[ink::contract(env = pink_extension::PinkEnvironment)]
mod sample_oracle {
    use hex_literal::hex;
    use phat_offchain_rollup::{
        lock::{Locks, GLOBAL as GLOBAL_LOCK},
        platforms::Evm,
        RollupResult, RollupTx, Target as RollupTarget,
    };
    use primitive_types::U256;
    use scale::{Decode, Encode};
    // use pink_extension as pink;

    /// Defines the storage of your contract.
    /// Add new fields to the below struct in order
    /// to add new static storage fields to your contract.
    #[ink(storage)]
    pub struct SampleOracle {
        /// Stores a single `bool` value on the storage.
        value: bool,
    }

    #[derive(Encode, Decode, Debug)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        BadAbi,
        FailedToGetStorage,
        FailedToDecodeStorage,
    }

    type Result<T> = core::result::Result<T, Error>;

    impl SampleOracle {
        /// Constructor that initializes the `bool` value to the given `init_value`.
        #[ink(constructor)]
        pub fn new(init_value: bool) -> Self {
            Self { value: init_value }
        }

        /// Constructor that initializes the `bool` value to `false`.
        ///
        /// Constructors can delegate to other constructors.
        #[ink(constructor)]
        pub fn default() -> Self {
            Self::new(Default::default())
        }

        /// Simply returns the current value of our `bool`.
        #[ink(message)]
        pub fn handle_req(&self) -> Result<Option<RollupResult>> {
            // TODO: change to return `core::result::Result<RollupResult, Vec<u8>>`
            let mut tx = RollupTx::default();
            let locks = locks();
            let vstore = MockVersionStore::default();

            // Declare write to global lock since it pops an element from the queue
            locks
                .tx_write(&mut tx, &vstore, GLOBAL_LOCK)
                .expect("lock must succeed");

            /**
             Deployed {
                anchor: '0xb3083F961C729f1007a6A1265Ae6b97dC2Cc16f2',
                oracle: '0x8Bf50F8d0B62017c9B83341CB936797f6B6235dd'
            }
            */
            // Connect to Ethereum RPC
            let addr: H160 = hex!("b3083F961C729f1007a6A1265Ae6b97dC2Cc16f2").into();
            let anchor = connect(
                "https://eth-goerli.g.alchemy.com/v2/jIHxR3mmAe_x36nOylBoWRgPL_XoxaUb".to_string(),
                addr,
            )?;

            // Read the queue pointer from the Anchor Contract
            let start: u32 = anchor.read_u256(b"qstart")?.try_into().unwrap();
            let end: u32 = anchor.read_u256(b"qend")?.try_into().unwrap();
            println!("start: {}", start);
            println!("end: {}", end);
            if start == end {
                return Ok(None);
            }

            // Read the queue content
            let queue_data = anchor.read_raw(&queue_key(b"q", start))?;

            // Decode the queue data by ethabi (u256, bytes)
            use pink_web3::ethabi;
            let decoded = ethabi::decode(
                &[ethabi::ParamType::Uint(32), ethabi::ParamType::Bytes],
                &queue_data,
            )
            .or(Err(Error::FailedToDecodeStorage))?;
            let (rid, pair) = match decoded.as_slice() {
                [ethabi::Token::Uint(reqid), ethabi::Token::Bytes(content)] => (reqid, content),
                _ => return Err(Error::FailedToDecodeStorage),
            };
            // Print the human readable request
            let pair = String::from_utf8(pair.clone()).unwrap();
            println!("Got req ({}, {})", rid, pair);

            // Get the price from somewhere
            // let price = get_price(pair);
            // let encoded_price = encode(price);
            let encoded_price =
                ethabi::encode(&[ethabi::Token::Uint(19800_000000_000000_000000u128.into())]);

            // Apply the response to request
            let payload = ethabi::encode(&[
                ethabi::Token::Uint(*rid),
                ethabi::Token::Bytes(encoded_price),
            ]);

            tx.action(Action::Reply(payload))
                .action(Action::ProcessedTo(start + 1));

            let result = RollupResult {
                tx,
                signature: Vec::new(),
                target: RollupTarget::Evm {
                    chain_id: "Ethereum".to_string(),
                    contract: "0xDEADBEEF".to_string(),
                },
            };
            Ok(Some(result))
        }
    }

    use pink_web3::api::{Eth, Namespace};
    use pink_web3::contract::{Contract, Options};
    use pink_web3::transports::{resolve_ready, PinkHttp};
    use pink_web3::types::TransactionParameters;
    use pink_web3::types::{Bytes, FilterBuilder, H160};

    enum Action {
        Reply(Vec<u8>),
        ProcessedTo(u32),
    }

    // conver to Vec<u8> for EVM
    impl Into<Vec<u8>> for Action {
        fn into(self) -> Vec<u8> {
            use core::iter::once;
            match self {
                Action::Reply(data) => once(0u8).chain(data.into_iter()).collect(),
                Action::ProcessedTo(n) => once(1u8).chain(u256_be(n.into()).into_iter()).collect(),
            }
        }
    }

    fn u256_be(n: U256) -> [u8; 32] {
        let mut r = [0u8; 32];
        n.to_big_endian(&mut r);
        r
    }

    struct Anchor {
        address: H160,
        contract: Contract<PinkHttp>,
    }

    fn connect(rpc: String, address: H160) -> Result<Anchor> {
        let eth = Eth::new(PinkHttp::new(rpc));
        let contract = Contract::from_json(eth, address, include_bytes!("res/anchor.abi.json"))
            .or(Err(Error::BadAbi))?;

        Ok(Anchor { address, contract })
    }

    fn queue_key(prefix: &[u8], idx: u32) -> Vec<u8> {
        let mut be_idx = [0u8; 32];
        U256::from(idx).to_big_endian(&mut be_idx);
        let mut key = Vec::from(prefix);
        key.extend(&be_idx);
        key
    }

    impl Anchor {
        fn read_raw(&self, key: &[u8]) -> Result<Vec<u8>> {
            let key: Bytes = key.into();
            let value: Bytes = resolve_ready(self.contract.query(
                "getStorage",
                (key,),
                self.address,
                Options::default(),
                None,
            ))
            .unwrap();
            println!("{:?}", value);
            // ).or(Err(Error::FailedToGetStorage))?;

            Ok(value.0)
        }

        fn read_typed<T: Decode + Default>(&self, key: &[u8]) -> Result<T> {
            let data = self.read_raw(key)?;
            if data.is_empty() {
                return Ok(Default::default());
            }
            T::decode(&mut &data[..]).or(Err(Error::FailedToDecodeStorage))
        }

        fn read_u256(&self, key: &[u8]) -> Result<U256> {
            let data = self.read_raw(key)?;
            if data.is_empty() {
                return Ok(Default::default());
            }
            if data.len() != 32 {
                return Err(Error::FailedToDecodeStorage);
            }
            Ok(U256::from_big_endian(&data))
        }

        fn submit_rollup(&self, tx: RollupTx) -> Result<()> {
            let pair = pink_web3::keys::pink::KeyPair::from(hex![
                "4c5d4f158b3d691328a1237d550748e019fe499ebf3df7467db6fa02a0818821"
            ]);
            // self.contract.signed_call("rollupU256CondEq", (), Options::default(), key)
            Ok(())
        }
    }

    // TODO: mock locks
    fn locks() -> Locks<Evm> {
        let mut locks = Locks::default();
        locks
            .add("queue", GLOBAL_LOCK)
            .expect("defining lock should succeed");
        locks
    }

    use alloc::collections::BTreeMap;
    use phat_offchain_rollup::lock::{LockId, LockVersion, LockVersionReader};
    // TODO: mock version reader
    #[derive(Default)]
    struct MockVersionStore {
        versions: BTreeMap<LockId, LockVersion>,
    }
    impl LockVersionReader for MockVersionStore {
        fn get_version(&self, id: LockId) -> phat_offchain_rollup::Result<LockVersion> {
            Ok(self.versions.get(&id).cloned().unwrap_or(0))
        }
    }

    /// Unit tests in Rust are normally defined within such a `#[cfg(test)]`
    /// module and test functions are marked with a `#[test]` attribute.
    /// The below code is technically just normal Rust code.
    #[cfg(test)]
    mod tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        /// Imports `ink_lang` so we can use `#[ink::test]`.
        use ink_lang as ink;

        /// We test if the default constructor does its job.
        #[ink::test]
        fn default_works() {
            pink_extension_runtime::mock_ext::mock_all_ext();
            let sample_oracle = SampleOracle::default();
            let res = sample_oracle.handle_req().unwrap();
            println!("res: {:#?}", res);
        }

        /// We test a simple use case of our contract.
        #[ink::test]
        fn it_works() {
            // let mut sample_oracle = SampleOracle::new(false);
            // sample_oracle.flip();
            // assert_eq!(sample_oracle.get(), true);
        }
    }
}
