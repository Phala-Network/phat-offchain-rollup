#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use core::fmt::Debug;

use alloc::vec::Vec;
use scale::{Decode, Encode};

pub mod clients;
pub mod lock;
pub mod platforms;

#[derive(Debug)]
pub enum Error {
    UnknownLock,
    FailedToReadVersion,
    FailedToDecode,
    DecodeOverflow,
    BadEvmAnchorAbi,
    FailedToGetStorage,
    FailedToDecodeStorage,
    FailedToGetBlockHash,
    FailedToCreateTransaction,
    FailedToSendTransaction,
    SessionFailedToDecode,
    SessionFailedToGetStorage,
    EvmFailedToSubmitTx(pink_web3::Error),
    EvmFailedToEstimateGas(pink_web3::contract::Error),
    EvmFailedToGetStorage(pink_web3::contract::Error),
    QueueIndexOverflow,
    LockVersionOverflow,
    RpcNetworkError,
}
pub type Result<T> = core::result::Result<T, Error>;

#[derive(Clone, PartialEq, Eq, Encode, Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub struct Raw(Vec<u8>);
impl Debug for Raw {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(fmt, "0x{}", hex::encode(&self.0))
    }
}
impl From<Vec<u8>> for Raw {
    fn from(data: Vec<u8>) -> Raw {
        Raw(data)
    }
}
impl From<Raw> for Vec<u8> {
    fn from(r: Raw) -> Vec<u8> {
        r.0
    }
}

#[derive(Debug, Encode, Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub struct RollupResult {
    pub tx: RollupTx,
    pub signature: Option<Vec<u8>>,
    pub target: Option<Vec<u8>>,
}

#[derive(Debug, Default, PartialEq, Eq, Encode, Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub struct RollupTx {
    pub conds: Vec<Cond>,
    pub actions: Vec<Raw>,
    pub updates: Vec<(Raw, Option<Raw>)>,
}

impl RollupTx {
    pub fn action(&mut self, act: impl Into<Vec<u8>>) -> &mut Self {
        self.actions.push(Into::<Vec<u8>>::into(act).into());
        self
    }
}

#[derive(Debug, PartialEq, Eq, Encode, Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum Action {
    Reply(Vec<u8>),
    ProcessedTo(u32),
}

#[derive(Debug, PartialEq, Eq, Encode, Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum Cond {
    Eq(Raw, Option<Raw>),
}

#[ink::trait_definition]
pub trait RollupHandler {
    #[ink(message)]
    fn handle_rollup(&self) -> core::result::Result<Option<RollupResult>, Vec<u8>>;
}

// Make it easier to call an arbitrary contract that implements RollupHandler
use ink::{codegen::TraitCallForwarder, reflect::TraitDefinitionRegistry};
use ink_lang as ink;
pub type RollupHandlerForwarder<Env> = <
    <TraitDefinitionRegistry<Env> as RollupHandler>::__ink_TraitInfo
    as TraitCallForwarder
>::Forwarder;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
