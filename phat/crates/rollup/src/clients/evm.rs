use crate::{Action, Error, Result, RollupTx};

use alloc::{borrow::ToOwned, vec::Vec};
use primitive_types::{H160, U256};
use scale::Encode;

use ethabi::Token;
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
    signing::Key,
    transports::{resolve_ready, PinkHttp},
    types::{BlockId, BlockNumber, Bytes, U64},
};

const ANCHOR_ABI: &[u8] = include_bytes!("../../res/anchor.abi.json");
const DEFAULT_QUEUE_PREFIX: &[u8] = b"q/";

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
    at: u64,
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
    pub fn new(rpc: &str, contract_id: H160) -> Result<Self> {
        let kvdb = EvmSnapshot::new(rpc, contract_id)?;
        let access_tracker = RwTracker::new();
        Ok(Self {
            actions: Default::default(),
            session: Session::new(kvdb, access_tracker, DEFAULT_QUEUE_PREFIX)
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

        let at = kvdb.at;
        Ok(Some(SubmittableRollupTx {
            contract: kvdb.destruct(),
            tx,
            at,
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
        // Prepare rollupU256CondEq params
        let params = self.tx.into_params();

        // Estiamte gas before submission
        let gas = resolve_ready(
            self.contract
                .estimate_gas::<(Token, Token, Token, Token, Token)>(
                    "rollupU256CondEq",
                    params.clone(),
                    pair.address(),
                    Options::default(),
                ),
        )
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

    pub fn submit_meta_tx(self, pair: &KeyPair, relay_pair: &KeyPair) -> Result<Vec<u8>> {
        let params = self.tx.into_params();
        let data = ethabi::encode(&[params.0, params.1, params.2, params.3, params.4]);
        let meta_params = sign_meta_tx(&self.contract, self.at, &data, pair).unwrap();

        // Estiamte gas before submission
        let gas = resolve_ready(self.contract.estimate_gas::<(Token, Bytes)>(
            "metaTxRollupU256CondEq",
            meta_params.clone(),
            relay_pair.address(),
            Options::default(),
        ))
        .map_err(Error::EvmFailedToEstimateGas)?;

        // Actually submit the tx (no guarantee for success)
        let tx_id = resolve_ready(self.contract.signed_call(
            "metaTxRollupU256CondEq",
            meta_params,
            Options::with(|opt| opt.gas = Some(gas)),
            relay_pair,
        ))
        .map_err(Error::EvmFailedToSubmitTx)?;

        #[cfg(feature = "logging")]
        pink::warn!("Sent = {}", hex::encode(&tx_id));

        Ok(tx_id.encode())
    }
}

/// Signes a meta tx with the help of the MetaTx contract
///
/// Return (ForwardRequest, Sig)
fn sign_meta_tx(
    contract: &Contract<PinkHttp>,
    at: u64,
    data: &[u8],
    pair: &KeyPair,
) -> Result<(Token, Bytes)> {
    let data: Bytes = data.into();
    let (req, hash): (Token, Token) = resolve_ready(contract.query(
        "metaTxPrepare",
        (pair.address(), data),
        contract.address(),
        Options::default(),
        // Currently the strategy is to stick to the snapthost block (`at`). However, it may not
        // be the best choice depending on the requirement.
        Some(BlockId::Number(BlockNumber::Number(at.into()))),
    ))
    .log_err("rollup snapshot: get storage failed")
    .map_err(Error::EvmFailedToPrepareMetaTx)?;
    let Token::FixedBytes(hash) = hash else {
        unreachable!()
    };
    let hash: [u8; 32] = hash
        .as_slice()
        .try_into()
        .expect("metaTxPrepare must return bytes32; qed.");
    let signature = pair.sign(&hash, None).expect("signing error").sig_encode();

    Ok((req, signature.into()))
}

trait Erc1271SigEncode {
    /// Encodes the secp256k1 signature with [ERC1271](https://eips.ethereum.org/EIPS/eip-1271)
    ///
    /// It always results in 65 bytes (32 bytes r, 32 bytes s, and 1 byte v).
    fn sig_encode(&self) -> Vec<u8>;
}

impl Erc1271SigEncode for pink_web3::signing::Signature {
    fn sig_encode(&self) -> Vec<u8> {
        (&self.r, &self.s, self.v as u8).encode()
    }
}

trait IntoRollupParams {
    /// Converts a RollupTx into the EVM contract arguments.
    ///
    /// `(cond_key, cond_values, update_keys, update_values, actions)`
    fn into_params(self) -> (Token, Token, Token, Token, Token);
}

impl IntoRollupParams for RollupTx {
    fn into_params(self) -> (Token, Token, Token, Token, Token) {
        let (cond_keys, cond_values): (Vec<Vec<u8>>, Vec<Vec<u8>>) = self
            .conds
            .into_iter()
            .map(|cond| {
                let crate::Cond::Eq(k, v) = cond;
                (k.into(), v.map(Into::into).unwrap_or_default())
            })
            .unzip();
        let (update_keys, update_values): (Vec<Vec<u8>>, Vec<Vec<u8>>) = self
            .updates
            .into_iter()
            .map(|(k, v)| (k.into(), v.map(Into::into).unwrap_or_default()))
            .unzip();
        let actions = self.actions.into_iter().map(Into::<Vec<u8>>::into);
        (
            Token::Array(cond_keys.into_iter().map(Token::Bytes).collect()),
            Token::Array(cond_values.into_iter().map(Token::Bytes).collect()),
            Token::Array(update_keys.into_iter().map(Token::Bytes).collect()),
            Token::Array(update_values.into_iter().map(Token::Bytes).collect()),
            Token::Array(actions.map(Token::Bytes).collect()),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sig_encode() {
        pink_extension_runtime::mock_ext::mock_all_ext();
        let seed: [u8; 32] =
            hex_literal::hex!("0000000000000000000000000000000000000000000000000000000000000001");
        let msg: [u8; 32] =
            hex_literal::hex!("1c8aff950685c2ed4bc3174f3472287b56d9517b9c948127319a09a7a36deac8");
        let pair = pink_web3::keys::pink::KeyPair::from(seed);
        let sig = pair.sign(&msg, None).unwrap();
        let der = sig.sig_encode();
        assert_eq!(&der, &hex_literal::hex!("a0b37f8fba683cc68f6574cd43b39f0343a50008bf6ccea9d13231d9e7e2e1e411edc8d307254296264aebfc3dc76cd8b668373a072fd64665b50000e9fcce521c"));
    }

    #[test]
    #[ignore]
    fn meta_tx() {
        pink_extension_runtime::mock_ext::mock_all_ext();
        let seed: [u8; 32] =
            hex_literal::hex!("ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80");
        let pair = pink_web3::keys::pink::KeyPair::from(seed);
        let anchor: H160 = hex_literal::hex!("5FbDB2315678afecb367f032d93F642f64180aa3").into();
        let mut client = EvmRollupClient::new("http://localhost:8545", anchor)
            .expect("failed to connect to testnet anchor");
        client.action(Action::Reply(vec![]));
        let rollup_tx = client.commit().expect("failed to commit").unwrap();
        rollup_tx.submit_meta_tx(&pair, &pair).unwrap();
    }
}
