use alloc::vec::Vec;

use ink::primitives::Hash;
use kv_session::{
    rollup,
    traits::{
        BumpVersion, Key, KvSession, KvSnapshot, QueueIndex, QueueIndexCodec, QueueSession, Value,
    },
    RwTracker, Session,
};
use pink_extension::chain_extension::signing;
use pink_extension::ResultExt;

#[cfg(feature = "logging")]
use pink_extension::debug;

use primitive_types::H256;
use scale::{Decode, Encode};
use subrpc::contracts::*;

pub use crate::{Action, Error, Result};

const DEFAULT_QUEUE_PREFIX: &[u8] = b"q/";

pub type ContractId = [u8; 32];

pub struct InkSnapshot<'a> {
    rpc: &'a str,
    pallet_id: u8,
    call_id: u8,
    contract_id: &'a ContractId,
    at: H256,
}

impl<'a> InkSnapshot<'a> {
    pub fn new(
        rpc: &'a str,
        pallet_id: u8,
        call_id: u8,
        contract_id: &'a ContractId,
    ) -> Result<Self> {
        let hash = subrpc::get_block_hash(rpc, None).or(Err(Error::FailedToGetBlockHash))?;
        Ok(InkSnapshot {
            rpc,
            pallet_id,
            call_id,
            contract_id,
            at: hash,
        })
    }
}

impl<'a> KvSnapshot for InkSnapshot<'a> {
    fn get(&self, key: &[u8]) -> kv_session::Result<Option<Value>> {
        let contract = InkContract::new(self.rpc, self.pallet_id, self.call_id, self.contract_id);

        // result of the query
        type QueryResult = Option<Vec<u8>>;
        // call the method
        let value: QueryResult = contract
            .query_at(
                *self.contract_id,
                ink::selector_bytes!("KvStore::get_value"),
                Some(&key),
                0,
                Some(self.at),
            )
            .log_err("Rollup snapshot: failed to get storage")
            .map_err(|_| kv_session::Error::FailedToGetStorage)?;

        #[cfg(feature = "logging")]
        debug!("Snapshot - key: {:02x?} - value: {:02x?}", &key, &value);

        Ok(value)
    }

    fn snapshot_id(&self) -> kv_session::Result<Vec<u8>> {
        Ok(self.at.encode())
    }
}

impl<'a> BumpVersion for InkSnapshot<'a> {
    fn bump_version(&self, version: Option<Vec<u8>>) -> kv_session::Result<Vec<u8>> {
        match version {
            Some(v) => {
                let ver = u32::decode(&mut &v[..]).or(Err(kv_session::Error::FailedToDecode))?;
                Ok((ver + 1).encode())
            }
            None => Ok(1u32.encode()),
        }
    }
}

pub struct ScaleCodec;
impl QueueIndexCodec for ScaleCodec {
    fn encode(number: QueueIndex) -> Vec<u8> {
        number.encode()
    }

    fn decode(raw: impl AsRef<[u8]>) -> kv_session::Result<QueueIndex> {
        // QueueIndex is stored as a value in the rollup kv store. When the value is empty, it's
        // treated as the default value (0 for u32). However, this function only handles the
        // non-empty case (empty value != zero length bytes). So here, `[]` is not considered.
        QueueIndex::decode(&mut raw.as_ref()).or(Err(kv_session::Error::FailedToDecode))
    }
}

pub struct InkRollupClient<'a> {
    rpc: &'a str,
    pallet_id: u8,
    call_id: u8,
    contract_id: &'a ContractId,
    actions: Vec<Action>,
    session: Session<InkSnapshot<'a>, RwTracker, ScaleCodec>,
}

pub struct SubmittableRollupTx<'a> {
    rpc: &'a str,
    pallet_id: u8,
    call_id: u8,
    contract_id: &'a ContractId,
    tx: InkRollupTx,
    _at: H256,
}

#[derive(Debug, Default, PartialEq, Eq, Encode, Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub struct InkRollupTx {
    conditions: Vec<(Key, Option<Value>)>,
    updates: Vec<(Key, Option<Value>)>,
    actions: Vec<Action>,
}

#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode)]
pub struct HandleActionInput {
    pub action_type: u8,
    pub action: Option<Vec<u8>>,
    pub address: Option<ContractId>,
    pub id: Option<QueueIndex>,
}

const ACTION_REPLY: u8 = 0;
const ACTION_SET_QUEUE_HEAD: u8 = 1;

impl Action {
    fn encode_into_ink(self) -> HandleActionInput {
        match self {
            Action::Reply(data) => HandleActionInput {
                action_type: ACTION_REPLY,
                action: Some(data),
                address: None,
                id: None,
            },
            Action::ProcessedTo(n) => HandleActionInput {
                action_type: ACTION_SET_QUEUE_HEAD,
                action: None,
                address: None,
                id: Some(n),
            },
        }
    }
}

impl<'a> InkRollupClient<'a> {
    pub fn new(
        rpc: &'a str,
        pallet_id: u8,
        call_id: u8,
        contract_id: &'a ContractId,
    ) -> Result<Self> {
        let kvdb = InkSnapshot::new(rpc, pallet_id, call_id, contract_id)?;
        let access_tracker = RwTracker::new();
        Ok(InkRollupClient {
            rpc,
            pallet_id,
            call_id,
            contract_id,
            actions: Default::default(),
            session: Session::new(kvdb, access_tracker, DEFAULT_QUEUE_PREFIX)
                .map_err(Error::SessionError)?,
        })
    }

    pub fn get<K: scale::Encode, V: scale::Decode>(&mut self, key: &K) -> Result<Option<V>> {
        let v = self.session.get(&key.encode())?;

        if let Some(v) = v {
            let v = V::decode(&mut v.as_slice())?;
            return Ok(Some(v));
        }

        Ok(None)
    }

    pub fn put<K: scale::Encode, V: scale::Encode>(&mut self, key: &K, value: &V) {
        self.session.put(&key.encode(), value.encode());
    }

    pub fn delete<K: scale::Encode>(&mut self, key: K) {
        self.session.delete(&key.encode());
    }

    pub fn pop<V: scale::Codec>(&mut self) -> Result<Option<V>> {
        let v = self.session.pop().map_err(Self::convert_err)?;

        if let Some(v) = v {
            let v = V::decode(&mut v.as_slice())?;
            return Ok(Some(v));
        }

        Ok(None)
    }

    pub fn action(&mut self, action: Action) -> &mut Self {
        self.actions.push(action);
        self
    }

    pub fn commit(mut self) -> Result<Option<SubmittableRollupTx<'a>>> {
        let (session_tx, kvdb) = self.session.commit();
        let raw_tx = rollup::rollup(
            &kvdb,
            session_tx,
            rollup::VersionLayout::Standalone {
                key_postfix: b":ver".to_vec(),
            },
        )
        .map_err(Self::convert_err)?;

        if let Some(head_idx) = raw_tx.queue_head {
            self.actions.push(Action::ProcessedTo(head_idx));
        }

        if raw_tx.updates.is_empty() && self.actions.is_empty() {
            return Ok(None);
        }

        let tx = InkRollupTx {
            conditions: raw_tx.conditions,
            updates: raw_tx.updates,
            actions: self.actions,
        };

        Ok(Some(SubmittableRollupTx {
            rpc: self.rpc,
            pallet_id: self.pallet_id,
            call_id: self.call_id,
            contract_id: self.contract_id,
            tx,
            _at: kvdb.at,
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
    pub fn submit(self, secret_key: &[u8; 32]) -> Result<Vec<u8>> {
        let params = self.tx.into_params();

        let contract = InkContract::new(self.rpc, self.pallet_id, self.call_id, self.contract_id);

        let result = contract
            .dry_run_and_send_transaction(
                ink::selector_bytes!("RollupAnchor::rollup_cond_eq"),
                Some(&params),
                0,
                secret_key,
            )
            .log_err("dry run and send transaction failed")
            .map_err(Error::InkFailedToCallContract)?;

        #[cfg(feature = "logging")]
        debug!("Sent = {}", hex::encode(&result));

        Ok(result)
    }

    pub fn submit_meta_tx(self, attestor_key: &[u8; 32], relay_key: &[u8; 32]) -> Result<Vec<u8>> {
        let params = self.tx.into_params();

        let origin: [u8; 32] = signing::get_public_key(attestor_key, signing::SigType::Sr25519)
            .try_into()
            .map_err(|_| Error::InvalidAddressLength)?;

        let meta_params = (origin, params.encode());

        #[cfg(feature = "logging")]
        {
            debug!("query method prepare");
            debug!("origin: {:?}", &origin);
            debug!("encoded params {:02x?}", params.encode());
        }

        let contract = InkContract::new(self.rpc, self.pallet_id, self.call_id, self.contract_id);

        // result of the query
        type PrepareResult = core::result::Result<(ForwardRequest, Hash), ContractError>;
        // call the method
        let result: PrepareResult = contract
            .query(
                origin,
                ink::selector_bytes!("MetaTransaction::prepare"),
                Some(&meta_params),
                0,
            )
            .log_err("dry run and send transaction failed")
            .map_err(Error::InkFailedToQueryContract)?;

        let (forward_request, hash) = result.map_err(|_| Error::InkFailedToPrepareMetaTx)?;

        #[cfg(feature = "logging")]
        {
            debug!("forwardRequest: {:02x?}", &forward_request);
            debug!("hash: {:02x?}", &hash);
        }

        // the attestor sign the hash
        //let signature = signing::sign(hash.as_ref(), attestor_key, signing::SigType::Ecdsa);
        let message: [u8; 32] = hash
            .as_ref()
            .to_vec()
            .try_into()
            .expect("Hash should be of length 32");
        let signature = signing::ecdsa_sign_prehashed(attestor_key, message);

        #[cfg(feature = "logging")]
        debug!("signature: {:02x?}", signature);

        let params = (forward_request, signature);

        let result = contract
            .dry_run_and_send_transaction(
                ink::selector_bytes!("MetaTransaction::meta_tx_rollup_cond_eq"),
                Some(&params),
                0,
                relay_key,
            )
            .log_err("dry run and send transaction failed")
            .map_err(Error::InkFailedToCallContract)?;

        #[cfg(feature = "logging")]
        debug!("Sent = {}", hex::encode(&result));

        Ok(result)
    }
}

///
/// Struct use in the meta transactions
///
#[derive(Debug, Eq, PartialEq, Clone, Encode, Decode)]
struct ForwardRequest {
    from: ink::primitives::AccountId,
    nonce: u128,
    data: Vec<u8>,
}

type RollupParamsType = (
    Vec<(Vec<u8>, Option<Vec<u8>>)>,
    Vec<(Vec<u8>, Option<Vec<u8>>)>,
    Vec<HandleActionInput>,
);

trait IntoRollupParams {
    /// Converts a RollupTx into the Ink contract arguments.
    fn into_params(self) -> RollupParamsType;
}

impl IntoRollupParams for InkRollupTx {
    fn into_params(self) -> RollupParamsType {
        #[cfg(feature = "logging")]
        {
            debug!("conditions ------");
            self.conditions.clone().into_iter().for_each(|(k, v)| {
                debug!("k: {:02x?}", &k);
                debug!("v: {:02x?}", &v);
            });

            debug!("updates ------");
            self.updates.clone().into_iter().for_each(|(k, v)| {
                debug!("k: {:02x?}", &k);
                debug!("v: {:02x?}", &v);
            });

            debug!("actions ------");
        }

        let actions: Vec<HandleActionInput> = self
            .actions
            .into_iter()
            .map(|action| {
                #[cfg(feature = "logging")]
                {
                    let a = action.encode_into_ink();
                    debug!("action: {:02x?}", &a);
                    a
                }
                #[cfg(not(feature = "logging"))]
                {
                    action.encode_into_ink()
                }
            })
            .collect();
        (self.conditions, self.updates, actions)
    }
}

/// convertor from scale::Error to Error
impl From<scale::Error> for Error {
    fn from(error: scale::Error) -> Self {
        Error::InkFailedToDecode(error)
    }
}
impl From<kv_session::Error> for Error {
    fn from(error: kv_session::Error) -> Self {
        Error::KVError(error)
    }
}
