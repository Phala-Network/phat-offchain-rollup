#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use ink_lang as ink;

pub use sample_oracle::*;

#[ink::contract(env = pink_extension::PinkEnvironment)]
mod sample_oracle {
    use alloc::{
        string::{String, ToString},
        vec::Vec,
    };
    use ink_storage::traits::{PackedLayout, SpreadLayout};
    use phat_offchain_rollup::{
        lock::{Locks, GLOBAL as GLOBAL_LOCK},
        platforms::Evm,
        RollupHandler, RollupResult, RollupTx, Target as RollupTarget,
    };
    use primitive_types::U256;
    use scale::{Decode, Encode};
    // use pink_extension as pink;

    /// Defines the storage of your contract.
    /// Add new fields to the below struct in order
    /// to add new static storage fields to your contract.
    #[ink(storage)]
    pub struct SampleOracle {
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
        anchor: [u8; 20],
    }

    #[derive(Encode, Decode, Debug)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        BadOrigin,
        NotConfigurated,
        BadAbi,
        FailedToGetStorage,
        FailedToDecodeStorage,
        FailedToEstimateGas,
    }

    type Result<T> = core::result::Result<T, Error>;

    impl SampleOracle {
        #[ink(constructor)]
        pub fn default() -> Self {
            Self {
                owner: Self::env().caller(),
                config: None,
            }
        }

        /// Configures the rollup target
        #[ink(message)]
        pub fn config(&mut self, rpc: String, anchor: H160) -> Result<()> {
            self.ensure_owner()?;
            self.config = Some(Config {
                rpc,
                anchor: anchor.into(),
            });
            Ok(())
        }

        fn handle_req(&self) -> Result<Option<RollupResult>> {
            let Config { rpc, anchor } = self.config.as_ref().ok_or(Error::NotConfigurated)?;

            let mut tx = RollupTx::default();
            let locks = locks();

            // Connect to Ethereum RPC
            let anchor = AnchorQueryClient::connect(rpc, anchor.into())?;
            let vstore = BlockingVersionStore { anchor: &anchor };

            // Declare write to global lock since it pops an element from the queue
            locks
                .tx_write(&mut tx, &vstore, GLOBAL_LOCK)
                .expect("lock must succeed");

            // Read the queue pointer from the Anchor Contract
            let start: u32 = anchor.read_u256(b"qstart")?.try_into().unwrap();
            let end: u32 = anchor.read_u256(b"qend")?.try_into().unwrap();
            #[cfg(feature = "std")]
            {
                println!("start: {}", start);
                println!("end: {}", end);
            }
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
            #[cfg(feature = "std")]
            println!("Got req ({}, {})", rid, pair);

            // Get the price from somewhere
            // let price = get_price(pair);
            // let encoded_price = encode(price);

            // Apply the response to request
            let payload = ethabi::encode(&[
                ethabi::Token::Uint(*rid),
                ethabi::Token::Uint(19800_000000_000000_000000u128.into()),
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

        /// Returns BadOrigin error if the caller is not the owner
        fn ensure_owner(&self) -> Result<()> {
            if self.env().caller() == self.owner {
                Ok(())
            } else {
                Err(Error::BadOrigin)
            }
        }
    }

    impl RollupHandler for SampleOracle {
        #[ink(message)]
        fn handle_rollup(&self) -> core::result::Result<Option<RollupResult>, Vec<u8>> {
            self.handle_req().map_err(|e| Encode::encode(&e))
        }
    }

    use pink_web3::api::{Eth, Namespace};
    use pink_web3::contract::{Contract, Options};
    use pink_web3::transports::{resolve_ready, PinkHttp};
    use pink_web3::types::{Bytes, H160};

    enum Action {
        Reply(Vec<u8>),
        ProcessedTo(u32),
    }

    // conver to Vec<u8> for EVM
    impl Into<Vec<u8>> for Action {
        fn into(self) -> Vec<u8> {
            use core::iter::once;
            match self {
                Action::Reply(data) => once(1u8).chain(data.into_iter()).collect(),
                Action::ProcessedTo(n) => [2u8, 0u8]
                    .into_iter()
                    .chain(u256_be(n.into()).into_iter())
                    .collect(),
            }
        }
    }

    fn u256_be(n: U256) -> [u8; 32] {
        let mut r = [0u8; 32];
        n.to_big_endian(&mut r);
        r
    }

    /// The client to query anchor contract states
    struct AnchorQueryClient {
        address: H160,
        contract: Contract<PinkHttp>,
    }

    fn queue_key(prefix: &[u8], idx: u32) -> Vec<u8> {
        let mut be_idx = [0u8; 32];
        U256::from(idx).to_big_endian(&mut be_idx);
        let mut key = Vec::from(prefix);
        key.extend(&be_idx);
        key
    }

    impl AnchorQueryClient {
        fn connect(rpc: &String, address: H160) -> Result<Self> {
            let eth = Eth::new(PinkHttp::new(rpc));
            let contract = Contract::from_json(eth, address, include_bytes!("res/anchor.abi.json"))
                .or(Err(Error::BadAbi))?;

            Ok(Self { address, contract })
        }

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
            #[cfg(feature = "std")]
            println!("{:?}", value);
            // FIXME
            // ).or(Err(Error::FailedToGetStorage))?;

            Ok(value.0)
        }

        fn _read_typed<T: Decode + Default>(&self, key: &[u8]) -> Result<T> {
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
    }

    // TODO: mock locks
    fn locks() -> Locks<Evm> {
        let mut locks = Locks::default();
        locks
            .add("queue", GLOBAL_LOCK)
            .expect("defining lock should succeed");
        locks
    }

    use phat_offchain_rollup::lock::{LockId, LockVersion, LockVersionReader};

    struct BlockingVersionStore<'a> {
        anchor: &'a AnchorQueryClient,
    }
    impl<'a> LockVersionReader for BlockingVersionStore<'a> {
        fn get_version(&self, id: LockId) -> phat_offchain_rollup::Result<LockVersion> {
            let id: Vec<u8> = phat_offchain_rollup::lock::EvmLocks::key(id).into();
            let value = self
                .anchor
                .read_u256(&id)
                .expect("FIXME: assume successful");
            let value: u32 = value.try_into().expect("version musn't exceed u32");
            Ok(value)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        use ink_lang as ink;

        #[ink::test]
        fn default_works() {
            pink_extension_runtime::mock_ext::mock_all_ext();

            /*
             Deployed {
                anchor: '0xb3083F961C729f1007a6A1265Ae6b97dC2Cc16f2',
                oracle: '0x8Bf50F8d0B62017c9B83341CB936797f6B6235dd'
            }
            */

            let rpc =
                "https://eth-goerli.g.alchemy.com/v2/68LXpUy0t0sLfZT2U-iYY5xh5OA8L6RV".to_string();
            let anchor_addr: H160 = hex!("b3083F961C729f1007a6A1265Ae6b97dC2Cc16f2").into();

            let mut sample_oracle = SampleOracle::default();
            sample_oracle.config(rpc, anchor_addr).unwrap();

            let res = sample_oracle.handle_req().unwrap();
            println!("res: {:#?}", res);
        }
    }
}
