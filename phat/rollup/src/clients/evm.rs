const ANCHOR_ABI: &[u8] = include_bytes!("../../res/anchor.abi.json");

pub mod read {
    use crate::{
        lock::{LockId, LockVersion, LockVersionReader, Locks},
        platforms::Evm,
        Error, Result, RollupResult, RollupTx,
    };
    use alloc::{string::String, vec::Vec};
    use pink_web3::api::{Eth, Namespace};
    use pink_web3::contract::{Contract, Options};
    use pink_web3::transports::{resolve_ready, PinkHttp};
    use pink_web3::types::{Bytes, H160};
    use primitive_types::U256;
    use scale::Decode;

    // TODO: move out out EVM since it's generic
    pub enum Action {
        Reply(Vec<u8>),
        ProcessedTo(u32),
    }

    // Converts to Vec<u8> for EVM rollup anchor
    impl From<Action> for Vec<u8> {
        fn from(action: Action) -> Vec<u8> {
            use core::iter::once;
            match action {
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

    impl AnchorQueryClient {
        /// Connects to an Ethereum RPC endpoint and load the anchor contract
        pub fn connect(rpc: &String, address: H160) -> Result<Self> {
            let eth = Eth::new(PinkHttp::new(rpc));
            let contract = Contract::from_json(eth, address, super::ANCHOR_ABI)
                .or(Err(Error::BadEvmAnchorAbi))?;

            Ok(Self { address, contract })
        }

        /// Reads the raw bytes from the anchor kv store
        pub fn read_raw(&self, key_slice: &[u8]) -> Result<Vec<u8>> {
            let key: Bytes = key_slice.into();
            let value: Bytes = resolve_ready(self.contract.query(
                "getStorage",
                (key,),
                self.address,
                Options::default(),
                None,
            ))
            .unwrap();
            #[cfg(feature = "std")]
            println!(
                "Read(0x{}) = 0x{}",
                hex::encode(key_slice),
                hex::encode(&value.0)
            );
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

        /// Reads an u256 value from the anchor raw kv store
        pub fn read_raw_u256(&self, key: &[u8]) -> Result<U256> {
            let data = self.read_raw(key)?;
            if data.is_empty() {
                return Ok(Default::default());
            }
            if data.len() != 32 {
                return Err(Error::FailedToDecodeStorage);
            }
            Ok(U256::from_big_endian(&data))
        }

        /// Reads the raw bytes value from the QueuedAnchor.getBytes API
        pub fn read_queue_bytes(&self, key_slice: &[u8]) -> Result<Vec<u8>> {
            let key: Bytes = key_slice.into();
            let value: Bytes = resolve_ready(self.contract.query(
                "getBytes",
                (key,),
                self.address,
                Options::default(),
                None,
            ))
            .unwrap();
            Ok(value.0)
        }

        /// Reads an u256 value from the QueuedAnchor.getBytes API
        pub fn read_queue_u256(&self, key_slice: &[u8]) -> Result<U256> {
            let data = self.read_queue_bytes(key_slice)?;
            if data.is_empty() {
                return Ok(Default::default());
            }
            if data.len() != 32 {
                return Err(Error::FailedToDecodeStorage);
            }
            Ok(U256::from_big_endian(&data))
        }
    }

    /// Implements LockVersionReader to read version from the anchor contract
    struct BlockingVersionStore<'a> {
        anchor: &'a AnchorQueryClient,
    }
    impl<'a> LockVersionReader for BlockingVersionStore<'a> {
        fn get_version(&self, id: LockId) -> crate::Result<LockVersion> {
            let id: Vec<u8> = crate::lock::EvmLocks::key(id).into();
            let value = self
                .anchor
                .read_raw_u256(&id)
                .expect("FIXME: assume successful");
            let value: u32 = value.try_into().expect("version musn't exceed u32");
            Ok(value)
        }
    }

    /// The client to handle a QueuedRollupAnchor rollup session
    pub struct QueuedRollupSession {
        anchor: AnchorQueryClient,
        locks: Locks<Evm>,
        tx: RollupTx,
    }

    impl QueuedRollupSession {
        /// Creates a new session that connects to the EVM anchor contract
        pub fn new<F>(rpc: &String, address: H160, lock_def: F) -> Self
        where
            F: FnOnce(&mut Locks<Evm>),
        {
            let anchor =
                AnchorQueryClient::connect(rpc, address).expect("FIXME: failed to connect");
            let mut locks = Locks::default();
            lock_def(&mut locks);
            Self {
                anchor,
                locks,
                tx: RollupTx::default(),
            }
        }

        /// Gets the RollupTx to write
        pub fn tx_mut(&mut self) -> &mut RollupTx {
            &mut self.tx
        }

        /// Builds the final RollupResult
        pub fn build(self) -> RollupResult {
            RollupResult {
                tx: self.tx,
                signature: None,
                target: None,
            }
        }

        /// Requests a write lock
        pub fn lock_write(&mut self, lock: &str) -> Result<()> {
            let vstore = BlockingVersionStore {
                anchor: &self.anchor,
            };
            self.locks.tx_write(&mut self.tx, &vstore, lock)
        }

        /// Requests a read lock
        pub fn lock_read(&mut self, lock: &str) -> Result<()> {
            let vstore = BlockingVersionStore {
                anchor: &self.anchor,
            };
            self.locks.tx_read(&mut self.tx, &vstore, lock)
        }

        /// Gets the start index of the queue (inclusive)
        pub fn queue_start(&self) -> Result<u32> {
            Ok(self
                .anchor
                .read_queue_u256(b"start")?
                .try_into()
                .expect("queue index overflow"))
        }

        /// Gets the end index of the queue (non-inclusive)
        pub fn queue_end(&self) -> Result<u32> {
            Ok(self
                .anchor
                .read_queue_u256(b"end")?
                .try_into()
                .expect("queue index overflow"))
        }

        /// Gets the i-th element of the queue
        pub fn queue_get(&self, i: u32) -> Result<Vec<u8>> {
            let mut be_idx = [0u8; 32];
            U256::from(i).to_big_endian(&mut be_idx);
            self.anchor.read_queue_bytes(&be_idx)
        }

        /// Gets the first element of the queue
        ///
        /// Return `(element, idx)`
        pub fn queue_head(&self) -> Result<(Option<Vec<u8>>, u32)> {
            let start: u32 = self.queue_start()?;
            let end: u32 = self.queue_end()?;
            if start == end {
                return Ok((None, start));
            }
            self.queue_get(start).map(|v| (Some(v), start))
        }
    }
}

pub mod write {
    use crate::{Error, Result};
    use alloc::{string::String, vec::Vec};
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
        /// Connects to an Ethereum RPC endpoint and load the anchor contract
        pub fn connect(rpc: &String, address: H160) -> Result<AnchorTxClient> {
            let eth = Eth::new(PinkHttp::new(rpc));
            let contract = Contract::from_json(eth, address, super::ANCHOR_ABI)
                .or(Err(Error::BadEvmAnchorAbi))?;

            Ok(AnchorTxClient { contract })
        }

        /// Submits a RollupTx to the anchor with a key pair
        ///
        /// Return the transaction hash but don't wait for the transaction confirmation.
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
