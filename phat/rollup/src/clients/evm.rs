const ANCHOR_ABI: &[u8] = include_bytes!("../../res/anchor.abi.json");

pub mod read {
    use crate::{Error, Result};
    use alloc::vec::Vec;
    use pink_web3::api::{Eth, Namespace};
    use pink_web3::contract::{Contract, Options};
    use pink_web3::transports::{resolve_ready, PinkHttp};
    use pink_web3::types::{Bytes, H160};
    use primitive_types::U256;
    use scale::Decode;

    pub enum Action {
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
    pub struct AnchorQueryClient {
        address: H160,
        contract: Contract<PinkHttp>,
    }

    pub fn queue_key(prefix: &[u8], idx: u32) -> Vec<u8> {
        let mut be_idx = [0u8; 32];
        U256::from(idx).to_big_endian(&mut be_idx);
        let mut key = Vec::from(prefix);
        key.extend(&be_idx);
        key
    }

    impl AnchorQueryClient {
        pub fn connect(rpc: &String, address: H160) -> Result<Self> {
            let eth = Eth::new(PinkHttp::new(rpc));
            let contract = Contract::from_json(eth, address, super::ANCHOR_ABI)
                .or(Err(Error::BadEvmAnchorAbi))?;

            Ok(Self { address, contract })
        }

        pub fn read_raw(&self, key: &[u8]) -> Result<Vec<u8>> {
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

        pub fn _read_typed<T: Decode + Default>(&self, key: &[u8]) -> Result<T> {
            let data = self.read_raw(key)?;
            if data.is_empty() {
                return Ok(Default::default());
            }
            T::decode(&mut &data[..]).or(Err(Error::FailedToDecodeStorage))
        }

        pub fn read_u256(&self, key: &[u8]) -> Result<U256> {
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

    use crate::lock::{LockId, LockVersion, LockVersionReader};

    pub struct BlockingVersionStore<'a> {
        pub anchor: &'a AnchorQueryClient,
    }
    impl<'a> LockVersionReader for BlockingVersionStore<'a> {
        fn get_version(&self, id: LockId) -> crate::Result<LockVersion> {
            let id: Vec<u8> = crate::lock::EvmLocks::key(id).into();
            let value = self
                .anchor
                .read_u256(&id)
                .expect("FIXME: assume successful");
            let value: u32 = value.try_into().expect("version musn't exceed u32");
            Ok(value)
        }
    }
}

pub mod write {
    use crate::{Error, Result};
    use pink_web3::contract::{Contract, Options};
    use pink_web3::transports::{resolve_ready, PinkHttp};
    use pink_web3::types::H160;
    use pink_web3::{
        api::{Eth, Namespace},
        keys::pink::KeyPair,
    };

    /// The client to submit transaction to the Evm anchor contract
    pub struct AnchorTxClient {
        contract: Contract<PinkHttp>,
    }

    impl AnchorTxClient {
        pub fn connect(rpc: &String, address: H160) -> Result<AnchorTxClient> {
            let eth = Eth::new(PinkHttp::new(rpc));
            let contract = Contract::from_json(eth, address, super::ANCHOR_ABI)
                .or(Err(Error::BadEvmAnchorAbi))?;

            Ok(AnchorTxClient { contract })
        }

        pub fn submit_rollup(
            &self,
            tx: crate::RollupTx,
            pair: KeyPair,
        ) -> Result<primitive_types::H256> {
            use ethabi::Token;
            use pink_web3::signing::Key;

            // Prepare rollupU256CondEq params
            let (cond_keys, cond_values): (Vec<Vec<u8>>, Vec<Vec<u8>>) = tx
                .conds
                .into_iter()
                .map(|cond| {
                    let crate::Cond::Eq(k, v) = cond;
                    (k.into(), v.map(Into::into).unwrap_or_default())
                })
                .unzip();
            let (update_keys, update_values): (Vec<Vec<u8>>, Vec<Vec<u8>>) = tx
                .updates
                .into_iter()
                .map(|(k, v)| (k.into(), v.map(Into::into).unwrap_or_default()))
                .unzip();
            let actions = tx.actions.into_iter().map(Into::<Vec<u8>>::into);
            let params = (
                Token::Array(cond_keys.into_iter().map(Token::Bytes).collect()),
                Token::Array(cond_values.into_iter().map(Token::Bytes).collect()),
                Token::Array(update_keys.into_iter().map(Token::Bytes).collect()),
                Token::Array(update_values.into_iter().map(Token::Bytes).collect()),
                Token::Array(actions.map(Token::Bytes).collect()),
            );

            // Estiamte gas before submission
            let gas = resolve_ready(self.contract.estimate_gas(
                "rollupU256CondEq",
                params.clone(),
                pair.address(),
                Options::default(),
            ))
            .expect("FIXME: failed to estiamte gas");

            // Actually submit the tx (no guarantee for success)
            let tx_id = resolve_ready(self.contract.signed_call(
                "rollupU256CondEq",
                params,
                Options::with(|opt| opt.gas = Some(gas)),
                pair,
            ))
            .expect("FIXME: submit failed");
            Ok(tx_id)
        }
    }
}
