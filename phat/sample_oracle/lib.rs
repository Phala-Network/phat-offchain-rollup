#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use ink_lang as ink;

pub use crate::sample_oracle::*;

#[ink::contract(env = pink_extension::PinkEnvironment)]
mod sample_oracle {
    use alloc::{string::String, vec::Vec};
    use ink_storage::traits::{PackedLayout, SpreadLayout};
    use phat_offchain_rollup::{
        clients::evm::read::{
            queue_key, // FIXME
            Action,
            AnchorQueryClient,
            BlockingVersionStore,
        },
        lock::{Locks, GLOBAL as GLOBAL_LOCK},
        platforms::Evm,
        RollupHandler, RollupResult, RollupTx,
    };
    use primitive_types::H160;
    use scale::{Decode, Encode};

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
            let anchor =
                AnchorQueryClient::connect(rpc, anchor.into()).expect("FIXME: failed to connect");
            let vstore = BlockingVersionStore { anchor: &anchor };

            // Declare write to global lock since it pops an element from the queue
            locks
                .tx_write(&mut tx, &vstore, GLOBAL_LOCK)
                .expect("lock must succeed");

            // Read the queue pointer from the Anchor Contract
            let start: u32 = anchor.read_u256(b"qstart").unwrap().try_into().unwrap();
            let end: u32 = anchor.read_u256(b"qend").unwrap().try_into().unwrap();
            #[cfg(feature = "std")]
            {
                println!("start: {}", start);
                println!("end: {}", end);
            }
            if start == end {
                return Ok(None);
            }

            // Read the queue content
            let queue_data = anchor
                .read_raw(&queue_key(b"q", start))
                .expect("FIXME: failed to read queue data");

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
                signature: None,
                target: None,
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

    fn locks() -> Locks<Evm> {
        let mut locks = Locks::default();
        locks
            .add("queue", GLOBAL_LOCK)
            .expect("defining lock should succeed");
        locks
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
            let rpc = env::var("RPC").unwrap().to_string();
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
            pink_extension_runtime::mock_ext::mock_all_ext();

            let (rpc, anchor_addr) = consts();

            let mut sample_oracle = SampleOracle::default();
            sample_oracle.config(rpc, anchor_addr).unwrap();

            let res = sample_oracle.handle_req().unwrap();
            println!("res: {:#?}", res);
        }
    }
}
