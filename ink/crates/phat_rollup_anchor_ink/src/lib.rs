#![cfg_attr(not(feature = "std"), no_std, no_main)]
//#![feature(min_specialization)]
#![allow(clippy::inline_fn_without_body)]

extern crate core;

pub mod impls;
pub mod traits;

#[cfg(test)]
pub mod tests;
