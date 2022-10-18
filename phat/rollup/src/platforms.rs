use crate::{Error, Result};
use alloc::vec::Vec;
use primitive_types::U256;

pub trait Platform {
    fn encode_u32(n: u32) -> Vec<u8>;
    fn decode_u32(data: &[u8]) -> Result<u32>;

    fn encode_u256(n: U256) -> Vec<u8>;
    fn decode_u256(data: &[u8]) -> Result<U256>;
}

pub struct Evm;
impl Platform for Evm {
    fn encode_u32(n: u32) -> Vec<u8> {
        Self::encode_u256(n.into())
    }
    fn decode_u32(data: &[u8]) -> Result<u32> {
        let n256 = Self::decode_u256(data)?;
        n256.try_into().or(Err(Error::DecodeOverflow))
    }

    fn encode_u256(n: U256) -> Vec<u8> {
        u256_be(n).to_vec()
    }
    fn decode_u256(data: &[u8]) -> Result<U256> {
        if data.len() < 32 {
            return Err(Error::FailedToDecode);
        }
        Ok(U256::from_big_endian(data))
    }
}

fn u256_be(n: U256) -> [u8; 32] {
    let mut r = [0u8; 32];
    n.to_big_endian(&mut r);
    r
}
