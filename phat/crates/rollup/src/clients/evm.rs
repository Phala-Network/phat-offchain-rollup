use crate::{Action, Error, Result, RollupTx};

use alloc::{borrow::ToOwned, vec::Vec};
use primitive_types::{H160, U256};
use scale::Encode;

use kv_session::{
    rollup,
    traits::{BumpVersion, KvSnapshot, QueueIndexCodec},
    RwTracker, Session,
};
use pink::ResultExt;
use pink_extension as pink;
use pink_web3::{
    api::{Eth, Namespace},
    contract::{Contract, Options},
    keys::pink::KeyPair,
    transports::{resolve_ready, PinkHttp},
    types::{BlockId, BlockNumber, Bytes, U64},
};

const ANCHOR_ABI: &[u8] = include_bytes!("../../res/anchor.abi.json");

pub struct EvmSnapshot {
    contract_id: H160,
    contract: Contract<PinkHttp>,
    at: u64,
}

impl EvmSnapshot {
    pub fn new(rpc: &str, contract_id: H160) -> Result<Self> {
        let eth = Eth::new(PinkHttp::new(rpc));
        let at: U64 = eth
            .block_number()
            .resolve()
            .log_err("rollup snapshot: failed to get block number")
            .or(Err(Error::FailedToGetBlockNumber))?;
        let contract =
            Contract::from_json(eth, contract_id, ANCHOR_ABI).or(Err(Error::BadEvmAnchorAbi))?;
        Ok(EvmSnapshot {
            contract,
            contract_id,
            at: at.0[0],
        })
    }
    pub fn destruct(self) -> Contract<PinkHttp> {
        self.contract
    }
}

impl KvSnapshot for EvmSnapshot {
    fn get(&self, key: &[u8]) -> kv_session::Result<Option<Vec<u8>>> {
        let key: Bytes = key.to_owned().into();
        let value: Bytes = resolve_ready(self.contract.query(
            "getStorage",
            (key.clone(),),
            self.contract_id,
            Options::default(),
            Some(BlockId::Number(BlockNumber::Number(self.at.into()))),
        ))
        .log_err("rollup snapshot: get storage failed")
        .or(Err(kv_session::Error::FailedToGetStorage))?;

        #[cfg(feature = "logging")]
        pink::warn!(
            "Storage[{}] = {:?}",
            hex::encode(&key.0),
            hex::encode(&value.0)
        );

        Ok(Some(value.0))
    }

    fn snapshot_id(&self) -> kv_session::Result<Vec<u8>> {
        Ok(self.at.encode())
    }
}
impl BumpVersion for EvmSnapshot {
    fn bump_version(&self, version: Option<Vec<u8>>) -> kv_session::Result<Vec<u8>> {
        // u32 is stored in U256 in EVM. Here we parse it as u32, inc, and return in U256 again
        let old: u32 = match version {
            Some(v) => RlpCodec::decode(&v)?,
            None => 0,
        };
        let new = old + 1;
        let mut encoded = [0u8; 32];
        U256::from(new).to_big_endian(&mut encoded);
        Ok(encoded.to_vec())
    }
}

pub struct RlpCodec;
impl QueueIndexCodec for RlpCodec {
    fn encode(number: u32) -> Vec<u8> {
        let mut encoded = [0u8; 32];
        U256::from(number).to_big_endian(&mut encoded);
        encoded.to_vec()
    }

    fn decode(raw: impl AsRef<[u8]>) -> kv_session::Result<u32> {
        // Unlike the decode function for Substrate, EVM contract always returns the raw bytes.
        // Even if the storage value doesn't exist, it returns a zero length bytes array. So here
        // we must handle the default value case (`v.len() == 0`).
        let v = raw.as_ref();
        if v.len() == 0 {
            Ok(0)
        } else if v.len() != 32 {
            Err(kv_session::Error::FailedToDecode)
        } else {
            Ok(U256::from_big_endian(v).low_u32())
        }
    }
}

pub struct EvmRollupClient {
    actions: Vec<Vec<u8>>,
    session: Session<EvmSnapshot, RwTracker, RlpCodec>,
}

pub struct SubmittableRollupTx {
    contract: Contract<PinkHttp>,
    tx: RollupTx,
}

impl Action {
    fn encode_into_evm(self) -> Vec<u8> {
        match self {
            Action::Reply(mut data) => {
                data.insert(0, 0);
                data
            }
            Action::ProcessedTo(n) => {
                let mut data = RlpCodec::encode(n);
                data.insert(0, 1);
                data
            }
        }
    }
}

impl EvmRollupClient {
    pub fn new(rpc: &str, contract_id: H160, queue_prefix: &[u8]) -> Result<Self> {
        let kvdb = EvmSnapshot::new(rpc, contract_id)?;
        let access_tracker = RwTracker::new();
        Ok(Self {
            actions: Default::default(),
            session: Session::new(kvdb, access_tracker, queue_prefix)
                .map_err(Error::SessionError)?,
        })
    }

    pub fn session(&mut self) -> &mut Session<EvmSnapshot, RwTracker, RlpCodec> {
        &mut self.session
    }

    pub fn action(&mut self, action: Action) -> &mut Self {
        self.actions.push(action.encode_into_evm());
        self
    }

    pub fn commit(mut self) -> Result<Option<SubmittableRollupTx>> {
        let (session_tx, kvdb) = self.session.commit();
        let raw_tx = rollup::rollup(
            &kvdb,
            session_tx,
            rollup::VersionLayout::Standalone {
                key_postfix: b":ver".to_vec(),
            },
        )
        .map_err(Self::convert_err)?;

        // #[cfg(feature = "logging")]
        // pink::warn!("RawTx: {raw_tx:?}");

        if let Some(head_idx) = raw_tx.queue_head {
            self.actions
                .push(Action::ProcessedTo(head_idx).encode_into_evm());
        }

        if raw_tx.updates.is_empty() && self.actions.is_empty() {
            return Ok(None);
        }

        let tx = crate::RollupTx {
            conds: raw_tx
                .conditions
                .into_iter()
                .map(|(k, v)| crate::Cond::Eq(k.into(), v.map(Into::into)))
                .collect(),
            actions: self.actions.into_iter().map(Into::into).collect(),
            updates: raw_tx
                .updates
                .into_iter()
                .map(|(k, v)| (k.into(), v.map(Into::into)))
                .collect(),
        };

        Ok(Some(SubmittableRollupTx {
            contract: kvdb.destruct(),
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

impl SubmittableRollupTx {
    pub fn submit(self, pair: KeyPair) -> Result<Vec<u8>> {
        use ethabi::Token;
        use pink_web3::signing::Key;

        // Prepare rollupU256CondEq params
        let (cond_keys, cond_values): (Vec<Vec<u8>>, Vec<Vec<u8>>) = self
            .tx
            .conds
            .into_iter()
            .map(|cond| {
                let crate::Cond::Eq(k, v) = cond;
                (k.into(), v.map(Into::into).unwrap_or_default())
            })
            .unzip();
        let (update_keys, update_values): (Vec<Vec<u8>>, Vec<Vec<u8>>) = self
            .tx
            .updates
            .into_iter()
            .map(|(k, v)| (k.into(), v.map(Into::into).unwrap_or_default()))
            .unzip();
        let actions = self.tx.actions.into_iter().map(Into::<Vec<u8>>::into);
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
        .map_err(Error::EvmFailedToEstimateGas)?;

        // Actually submit the tx (no guarantee for success)
        let tx_id = resolve_ready(self.contract.signed_call(
            "rollupU256CondEq",
            params,
            Options::with(|opt| opt.gas = Some(gas)),
            pair,
        ))
        .map_err(Error::EvmFailedToSubmitTx)?;

        #[cfg(feature = "logging")]
        pink::warn!("Sent = {}", hex::encode(&tx_id));

        Ok(tx_id.encode())
    }
}
