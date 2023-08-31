#![cfg_attr(not(feature = "std"), no_std, no_main)]

extern crate alloc;

pub use crate::http_test::*;

#[ink::contract(env = pink_extension::PinkEnvironment)]
mod http_test {
    use alloc::{format, string::String, vec, vec::Vec};
    use ink::storage::traits::StorageLayout;
    use pink_extension as pink;
    use scale::{Decode, Encode};

    // To enable `(result).log_err("Reason")?`
    use pink::ResultExt;

    // Defined in TestOracle.sol
    const TYPE_RESPONSE: u32 = 0;
    const TYPE_FEED: u32 = 1;
    const TYPE_ERROR: u32 = 3;

    #[ink(storage)]
    pub struct HttpTest {
        owner: AccountId,
    }

    #[derive(Encode, Decode, Debug)]
    #[repr(u8)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        BadOrigin,
        NotConfigured,
        InvalidKeyLength,
        InvalidAddressLength,
        NoRequestInQueue,
        FailedToCreateClient,
        FailedToCommitTx,
        FailedToFetchPrice,

        FailedToGetStorage,
        FailedToCreateTransaction,
        FailedToSendTransaction,
        FailedToGetBlockHash,
        FailedToDecode,
        InvalidRequest,
    }

    type Result<T> = core::result::Result<T, Error>;

    impl HttpTest {
        #[ink(constructor)]
        pub fn default() -> Self {
            Self {
                owner: Self::env().caller(),
            }
        }

        /// Gets the owner of the contract
        #[ink(message)]
        pub fn owner(&self) -> AccountId {
            self.owner
        }

        /// Transfers the ownership of the contract (admin only)
        #[ink(message)]
        pub fn transfer_ownership(&mut self, new_owner: AccountId) -> Result<()> {
            self.ensure_owner()?;
            self.owner = new_owner;
            Ok(())
        }

        #[ink(message)]
        pub fn post(&self, url: String, data: Vec<u8>, headers: Vec<(String, String)>) -> Result<(u16, String)> {
            // let headers: Vec<_> = headers.iter().map(|s| s.as_bytes().to_vec()).collect();
            let resp = pink::http_post!(url.clone(), data, headers);
            let body = String::from_utf8(resp.body)
                .or(Err(Error::FailedToDecode))?;
            pink::info!("HTTP_TEST POST url: {url}, code: {}, body: <{body}>", resp.status_code);
            Ok((resp.status_code, body))
        }

        #[ink(message)]
        pub fn post_rpc(&self) -> Result<(u16, String)> {
            use alloc::string::ToString;
            self.post(
                "https://polygon-mumbai.g.alchemy.com/v2/r1xJUpegKtRjj_tKZEJC-dnytofbZ92q".to_string(),
                r#"{"jsonrpc":"2.0","id":1,"method":"eth_call","params":[{"from":"0x0000000000000000000000000000000000000000","data":"0xcaa0ff21000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000000","to":"0x6d3c87968e9f637abfe2ed4d1376242af482debb"},"latest"]}"#
                .to_string().as_bytes().to_vec(),
                vec![
                    ("accept".to_string(), "*/*".to_string()),
                    ("content-type".to_string(), "application/json".to_string()),
                ]
            )
        }

        /// Returns BadOrigin error if the caller is not the owner
        fn ensure_owner(&self) -> Result<()> {
            if self.env().caller() == self.owner {
                Ok(())
            } else {
                Err(Error::BadOrigin)
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[ink::test]
        fn test() {
            let _ = env_logger::try_init();
            pink_extension_runtime::mock_ext::mock_all_ext();
            let p = HttpTest::default().post(
                "https://rpc-mumbai.maticvigil.com/".to_string(),
                r#"{"jsonrpc":"2.0","id":1,"method":"eth_call","params":[{"from":"0x0000000000000000000000000000000000000000","data":"0xcaa0ff21000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000000","to":"0x6d3c87968e9f637abfe2ed4d1376242af482debb"},"latest"]}"#
                    .to_string().as_bytes().to_vec(),
                vec![
                    ("accept".to_string(), "*/*".to_string()),
                    ("content-type".to_string(), "application/json".to_string()),
                ]
            ).unwrap();
            pink::warn!("Price: {p:?}");
        }

    }
}
