#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

extern crate core;

pub mod impls;
pub mod traits;

#[cfg(test)]
pub mod tests;
