#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::{
    vec::Vec,
    string::String
};
use scale::{Encode, Decode};

pub mod lock;

#[derive(Debug)]
pub enum Error {
    UnknownLock,
    FailedToReadVersion,
}
pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Encode, Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub struct RollupResult {
    pub tx: RollupTx,
    pub signature: Vec<u8>,
    pub target: Target,
}

#[derive(Debug, Default, PartialEq, Eq, Encode, Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub struct RollupTx {
    conds: Vec<Cond>,
    actions: Vec<Vec<u8>>,
    updates: Vec<(Vec<u8>, Option<Vec<u8>>)>,
}

#[derive(Debug, PartialEq, Eq, Encode, Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum Cond {
    Eq(Vec<u8>, Option<Vec<u8>>),
}

#[derive(Debug, Encode, Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum Target {
    Evm {
        chain_id: String,
        contract: String,
    },
    Pallte {
        chain_id: String,
        // add more
    },
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
