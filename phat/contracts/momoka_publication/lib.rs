#![cfg_attr(not(feature = "std"), no_std, no_main)]

extern crate alloc;

pub use crate::momoka_publication::*;

#[ink::contract(env = pink_extension::PinkEnvironment)]
mod momoka_publication {
    use alloc::{format, string::String, vec, vec::Vec};
    use ethabi::Token;
    use ink::storage::traits::StorageLayout;
    use pink_extension as pink;
    use pink_extension::chain_extension::signing;
    use pink_json as json;
    use pink_web3::{
        api::{Eth, Namespace},
        contract::{Contract, Options},
        keys::pink::KeyPair,
        signing::Key,
        transports::{resolve_ready, PinkHttp},
        types::{Bytes, H160, U256},
    };
    use scale::{Decode, Encode};
    use serde::Deserialize;
    use this_crate::{version_tuple, VersionTuple};

    // To enable `(result).log_err("Reason")?`
    use pink::ResultExt;

    const ANCHOR_ABI: &[u8] = include_bytes!("./res/anchor.abi.json");

    #[ink(storage)]
    pub struct MomokaPublication {
        owner: AccountId,
        /// Key for signing the rollup tx
        attest_key: [u8; 32],
        client: Option<Client>,
    }

    #[derive(Clone, Encode, Decode, Debug)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, StorageLayout))]
    pub struct Client {
        /// The RPC endpoint of the target blockchain
        rpc: String,
        /// The client smart contract address on the target blockchain
        client_addr: [u8; 20],
    }

    #[derive(Encode, Decode, PartialEq, Debug)]
    #[repr(u8)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        BadOrigin,
        ClientNotConfigured,
        InvalidClientAddress,

        FailedToFetchData,
        FailedToDecode,
        FailedToParseJson,
        PublicationNotExists,
        FailedToParseId,

        BadEvmAnchorAbi,
        EvmFailedToPrepareMetaTx,
    }

    impl From<Error> for U256 {
        fn from(err: Error) -> U256 {
            (err as u8).into()
        }
    }

    type Result<T> = core::result::Result<T, Error>;

    impl MomokaPublication {
        #[ink(constructor)]
        pub fn default() -> Self {
            const NONCE_ATTEST_KEY: &[u8] = b"attest_key";
            let random_attest_key = signing::derive_sr25519_key(NONCE_ATTEST_KEY);
            Self::new(
                random_attest_key[..32]
                    .try_into()
                    .expect("should be long enough; qed."),
            )
        }

        #[ink(constructor)]
        pub fn new(sk: [u8; 32]) -> Self {
            Self {
                owner: Self::env().caller(),
                attest_key: sk,
                client: None,
            }
        }

        #[ink(message)]
        pub fn version(&self) -> VersionTuple {
            version_tuple!()
        }

        /// Gets the owner of the contract
        #[ink(message)]
        pub fn owner(&self) -> AccountId {
            self.owner
        }

        /// Get the identity of offchain rollup
        #[ink(message)]
        pub fn get_attest_address(&self) -> H160 {
            let sk = KeyPair::from(self.attest_key);
            sk.address()
        }

        #[ink(message)]
        pub fn get_client(&self) -> Result<Client> {
            self.client.clone().ok_or(Error::ClientNotConfigured)
        }

        /// Configures the rollup target (admin only)
        #[ink(message)]
        pub fn config_client(&mut self, rpc: String, client_addr: Vec<u8>) -> Result<()> {
            self.ensure_owner()?;
            self.client = Some(Client {
                rpc,
                client_addr: client_addr
                    .try_into()
                    .or(Err(Error::InvalidClientAddress))?,
            });
            Ok(())
        }

        /// Return abi::encode(forwardRequest, sig)
        #[ink(message)]
        pub fn check_lens_publication(
            &self,
            publication_id: String,
            mainnet: bool,
        ) -> Result<Vec<u8>> {
            let client = self.ensure_client_configured()?;

            let (profile_id, publication_id) =
                Self::fetch_lens_publication(publication_id, mainnet)?;
            let data = ethabi::encode(&[
                Token::Uint(profile_id.into()),
                Token::Uint(publication_id.into()),
            ]);

            let attest_key = KeyPair::from(self.attest_key);
            let (forward_request, sig) = sign_meta_tx(
                client.rpc.clone(),
                H160(client.client_addr),
                &data,
                &attest_key,
            )?;

            let r = ethabi::encode(&[forward_request, Token::Bytes(sig.0)]);
            Ok(r)
        }

        /// Return (profileId, publicationId)
        fn fetch_lens_publication(publication_id: String, mainnet: bool) -> Result<(u64, u64)> {
            let lens_api = if mainnet {
                "https://api.lens.dev/"
            } else {
                "https://api-mumbai.lens.dev/"
            };
            let headers = vec![
                ("Content-Type".into(), "application/json".into()),
                ("user-agent".into(), "phat-contract".into()),
            ];
            let body = format!("{{\"query\":\"query Publication {{\\n  publication(request: {{\\n    publicationId: \\\"{publication_id}\\\",\\n  }}) {{\\n    ... on Post {{\\n      ...PostFields\\n    }}\\n  }}\\n}}\\n\\nfragment PostFields on Post {{\\n  id\\n}}\"}}");

            let resp = pink::http_post!(lens_api, body.as_bytes(), headers);
            if resp.status_code != 200 {
                pink::warn!(
                    "Fail to read Lens api with status code: {}, reason: {}, body: {:?}",
                    resp.status_code,
                    resp.reason_phrase,
                    resp.body
                );
                return Err(Error::FailedToFetchData);
            }

            // Response body examples:
            // {
            //   "data": {
            //     "publication": {
            //       "id": "0x814a-0x01-DA-0e18b370"
            //     }
            //   }
            // },s
            //   "data": {
            //     "publication": null
            //   }
            // }
            let resp_body = String::from_utf8(resp.body).or(Err(Error::FailedToDecode))?;
            let parsed: PublicationResponse = json::from_str(&resp_body)
                .log_err("failed to parse json")
                .or(Err(Error::FailedToParseJson))?;

            // e.g. "0x814a-0x01-DA-0e18b370"
            let publication_id = parsed
                .data
                .publication
                .ok_or_else(|| Error::PublicationNotExists)?
                .id;

            let to_u64 = |s: String| -> Result<u64> {
                u64::from_str_radix(s.trim_start_matches("0x"), 16)
                    .map_err(|_| Error::FailedToParseId)
            };
            let tokens = publication_id
                .split('-')
                .take(2)
                .map(String::from)
                .map(to_u64)
                .collect::<Result<Vec<u64>>>()?;
            if tokens.len() != 2 {
                return Err(Error::FailedToParseId);
            }

            Ok((tokens[0], tokens[1]))
        }

        /// Returns BadOrigin error if the caller is not the owner
        fn ensure_owner(&self) -> Result<()> {
            if self.env().caller() == self.owner {
                Ok(())
            } else {
                Err(Error::BadOrigin)
            }
        }

        /// Returns the client config reference or raise the error `ClientNotConfigured`
        fn ensure_client_configured(&self) -> Result<&Client> {
            self.client.as_ref().ok_or(Error::ClientNotConfigured)
        }
    }

    /// Signes a meta tx with the help of the MetaTx contract
    ///
    /// Return (ForwardRequest, Sig)
    pub fn sign_meta_tx(
        rpc: String,
        contract_id: H160,
        data: &[u8],
        pair: &KeyPair,
    ) -> Result<(Token, Bytes)> {
        let eth = Eth::new(PinkHttp::new(rpc));
        let contract =
            Contract::from_json(eth, contract_id, ANCHOR_ABI).or(Err(Error::BadEvmAnchorAbi))?;

        let data: Bytes = data.into();
        let (req, hash): (Token, Token) = resolve_ready(contract.query(
            "metaTxPrepare",
            (pair.address(), data),
            contract.address(),
            Options::default(),
            None,
        ))
        .log_err("rollup snapshot: get storage failed")
        .map_err(|_| Error::EvmFailedToPrepareMetaTx)?;
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

    // Define the structures to parse json like `{"data":{"publication":{"id":"0x814a-0x01-DA-0e18b370"}}}}`
    #[derive(Deserialize)]
    struct PublicationResponse<'a> {
        #[serde(borrow)]
        data: PublicationResponsePayload<'a>,
    }
    #[derive(Deserialize)]
    struct PublicationResponsePayload<'a> {
        #[serde(borrow)]
        publication: Option<PublicationResponseInner<'a>>,
    }
    #[derive(Deserialize)]
    struct PublicationResponseInner<'a> {
        #[serde(borrow)]
        id: &'a str,
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        struct EnvVars {
            rpc: String,
            client_addr: Vec<u8>,
            attest_key: Vec<u8>,
        }

        fn get_env(key: &str) -> String {
            std::env::var(key).expect("env not found")
        }
        fn config() -> EnvVars {
            dotenvy::dotenv().ok();
            let rpc = get_env("RPC");
            let client_addr = hex::decode(get_env("CLIENT_ADDR")).expect("hex decode failed");
            let attest_key = hex::decode(get_env("ATTEST_KEY")).expect("hex decode failed");
            EnvVars {
                rpc,
                client_addr,
                attest_key,
            }
        }

        #[ink::test]
        fn fixed_parse() {
            let _ = env_logger::try_init();
            pink_extension_runtime::mock_ext::mock_all_ext();

            let (profile_id, publication_id): (u64, u64) =
                MomokaPublication::fetch_lens_publication(
                    String::from("0x814a-0x01-DA-0e18b370"),
                    false,
                )
                .unwrap();
            assert_eq!(profile_id, 0x814a);
            assert_eq!(publication_id, 0x01);
            pink::warn!("Profile: {profile_id:?}, Publication: {publication_id:?}");

            let res = MomokaPublication::fetch_lens_publication(
                String::from("0x814a-0x01-DA-0e18b37"),
                true,
            );
            assert_eq!(res, Err(Error::PublicationNotExists));

            let res = MomokaPublication::fetch_lens_publication(
                String::from("0x814a-0x01-DA-0e18b37"),
                false,
            );
            assert_eq!(res, Err(Error::PublicationNotExists));
        }

        #[ink::test]
        #[ignore]
        fn check_lens_publication() {
            let _ = env_logger::try_init();
            pink_extension_runtime::mock_ext::mock_all_ext();
            let EnvVars {
                rpc,
                client_addr,
                attest_key,
            } = config();

            let mut momoka_publication = MomokaPublication::new(attest_key.try_into().unwrap());
            momoka_publication.config_client(rpc, client_addr).unwrap();

            let r = momoka_publication
                .check_lens_publication(String::from("0x814a-0x01-DA-0e18b370"), false)
                .expect("failed to check publication");
            pink::warn!("publication proof: {}", hex::encode(&r));
        }
    }
}
