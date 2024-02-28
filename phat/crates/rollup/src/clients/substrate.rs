use crate::{Action, Error, Result, RollupTx};

use alloc::{borrow::ToOwned, vec::Vec};
use scale::{Decode, Encode};

use kv_session::{
    rollup,
    traits::{BumpVersion, KvSnapshot, QueueIndexCodec},
    RwTracker, Session,
};
use pink::ResultExt;
use pink::AccountId;
use primitive_types::H256;

const METHOD_CLAIM_NAME: u8 = 0u8;
const METHOD_ROLLUP: u8 = 1u8;

pub struct SubstrateSnapshot<'a> {
    rpc: &'a str,
    contract_id: &'a AccountId,
    at: H256,
}

impl<'a> SubstrateSnapshot<'a> {
    pub fn new(rpc: &'a str, contract_id: &'a AccountId) -> Result<Self> {
        let hash = subrpc::get_block_hash(rpc, None).or(Err(Error::FailedToGetBlockHash))?;
        Ok(SubstrateSnapshot {
            rpc,
            contract_id,
            at: hash,
        })
    }
}

impl<'a> KvSnapshot for SubstrateSnapshot<'a> {
    fn get(&self, key: &[u8]) -> kv_session::Result<Option<Vec<u8>>> {
        let prefix = subrpc::storage::storage_prefix("PhatRollupAnchor", "States");
        let key1: &[u8] = self.contract_id.as_ref();
        let key2: &[u8] = &key.to_owned().encode();
        let storage_key = subrpc::storage::storage_double_map_prefix::<
            subrpc::hasher::Blake2_128Concat,
            subrpc::hasher::Blake2_128Concat,
        >(&prefix, key1, key2);
        let value = subrpc::get_storage(self.rpc, &storage_key, None)
            .log_err("rollup snapshot: get storage failed")
            .or(Err(kv_session::Error::FailedToGetStorage))?;

        #[cfg(feature = "logging")]
        pink::warn!(
            "Storage[{}][{}] = {:?}",
            hex::encode(key1),
            hex::encode(key2),
            value.clone().map(|data| hex::encode(&data))
        );

        match value {
            Some(raw) => Ok(Some(
                Vec::<u8>::decode(&mut &raw[..]).or(Err(kv_session::Error::FailedToDecode))?,
            )),
            None => Ok(None),
        }
    }

    fn snapshot_id(&self) -> kv_session::Result<Vec<u8>> {
        Ok(self.at.encode())
    }
}
impl<'a> BumpVersion for SubstrateSnapshot<'a> {
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
    fn encode(number: u32) -> Vec<u8> {
        number.encode()
    }

    fn decode(raw: impl AsRef<[u8]>) -> kv_session::Result<u32> {
        // QueueIndex is stored as a value in the rollup kv store. When the value is empty, it's
        // treated as the default value (0 for u32). However, this function only handles the
        // non-empty case (empty value != zero length bytes). So here, `[]` is not considered.
        Decode::decode(&mut raw.as_ref()).or(Err(kv_session::Error::FailedToDecode))
    }
}

pub struct SubstrateRollupClient<'a> {
    rpc: &'a str,
    pallet_id: u8,
    contract_id: &'a AccountId,
    actions: Vec<Vec<u8>>,
    session: Session<SubstrateSnapshot<'a>, RwTracker, ScaleCodec>,
}

pub struct SubmittableRollupTx<'a> {
    rpc: &'a str,
    pallet_id: u8,
    contract_id: &'a AccountId,
    tx: RollupTx,
}

impl<'a> SubstrateRollupClient<'a> {
    pub fn new(
        rpc: &'a str,
        pallet_id: u8,
        contract_id: &'a AccountId,
        queue_prefix: &[u8],
    ) -> Result<Self> {
        let kvdb = SubstrateSnapshot::new(rpc, contract_id)?;
        let access_tracker = RwTracker::new();
        Ok(SubstrateRollupClient {
            rpc,
            pallet_id,
            contract_id,
            actions: Default::default(),
            session: Session::new(kvdb, access_tracker, queue_prefix)
                .map_err(Error::SessionError)?,
        })
    }

    pub fn session(&mut self) -> &mut Session<SubstrateSnapshot<'a>, RwTracker, ScaleCodec> {
        &mut self.session
    }

    pub fn action(&mut self, action: Action) -> &mut Self {
        self.actions.push(action.encode());
        self
    }

    pub fn commit(self) -> Result<Option<SubmittableRollupTx<'a>>> {
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

        if raw_tx.updates.is_empty() && self.actions.is_empty() {
            return Ok(None);
        }

        // Consider:
        // if let Some(head_idx) = raw_tx.queue_head {
        //     self.action(Action::ProcessedTo(head_idx));
        // }

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
            rpc: self.rpc,
            pallet_id: self.pallet_id,
            contract_id: self.contract_id,
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

impl<'a> SubmittableRollupTx<'a> {
    pub fn submit(self, secret_key: &[u8; 32], nonce: u128) -> Result<Vec<u8>> {
        let signed_tx = subrpc::create_transaction(
            secret_key,
            "khala",
            self.rpc,
            self.pallet_id,                     // pallet idx
            METHOD_ROLLUP,                      // method 1: rollup
            (self.contract_id, self.tx, nonce), // (name, tx, nonce)
            subrpc::ExtraParam::default(),
        )
        .or(Err(Error::FailedToCreateTransaction))?;

        #[cfg(feature = "logging")]
        {
            pink::warn!("ContractId = {}", hex::encode(self.contract_id),);
            pink::warn!("SignedTx = {}", hex::encode(&signed_tx),);
        }

        let tx_hash = subrpc::send_transaction(self.rpc, &signed_tx)
            .or(Err(Error::FailedToSendTransaction))?;

        #[cfg(feature = "logging")]
        pink::warn!("Sent = {}", hex::encode(&tx_hash),);
        Ok(tx_hash)
    }
}

pub fn get_name_owner(rpc: &str, contract_id: &AccountId) -> Result<Option<AccountId>> {
    // Build key
    let prefix = subrpc::storage::storage_prefix("PhatRollupAnchor", "SubmitterByNames");
    let map_key: &[u8] = contract_id.as_ref();
    let storage_key =
        subrpc::storage::storage_map_prefix::<subrpc::hasher::Blake2_128Concat>(&prefix, map_key);
    // Get storage
    let value = subrpc::get_storage(rpc, &storage_key, None).or(Err(Error::FailedToGetStorage))?;
    if let Some(value) = value {
        let owner: AccountId = Decode::decode(&mut &value[..]).or(Err(Error::FailedToDecode))?;
        return Ok(Some(owner));
    }
    return Ok(None);
}

pub fn claim_name(
    rpc: &str,
    pallet_id: u8,
    contract_id: &AccountId,
    secret_key: &[u8; 32],
) -> Result<Vec<u8>> {
    let signed_tx = subrpc::create_transaction(
        secret_key,
        "khala",
        rpc,
        pallet_id,
        METHOD_CLAIM_NAME,
        contract_id,
        subrpc::ExtraParam::default(),
    )
    .or(Err(Error::FailedToCreateTransaction))?;

    let tx_hash =
        subrpc::send_transaction(rpc, &signed_tx).or(Err(Error::FailedToSendTransaction))?;

    #[cfg(feature = "logging")]
    pink::warn!("Sent = {}", hex::encode(&tx_hash),);
    Ok(tx_hash)
}
