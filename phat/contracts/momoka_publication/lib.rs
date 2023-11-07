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
        FailedToParseAddress,
        NoProofForComment,
        UnknownPublicationType,
        MissingMirrorField,
        MissingCollectModule,

        BadEvmAnchorAbi,
        EvmFailedToPrepareMetaTx,
        BadPublicationId,
        BadDaId,
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
            let act_oracle_resp = Self::fetch_lens_publication(publication_id, mainnet)?;
            let data = ethabi::encode(&[act_oracle_resp]);

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
        fn fetch_lens_publication(
            publication_id: String,
            mainnet: bool,
        ) -> Result<Token> {
            use alloc::string::ToString;
            let lens_api = if mainnet {
                "https://api.lens.dev/"
            } else {
                "https://api-mumbai.lens.dev/"
            };
            let headers = vec![
                ("Content-Type".into(), "application/json".into()),
                ("User-Agent".into(), "phat-contract".into()),
            ];
            let body = format!("{{\"query\": \"query Publication {{\\n  publication(request: {{\\n    publicationId: \\\"{publication_id}\\\"\\n  }}) {{\\n   __typename\\n    ... on Post {{\\n      ...PostFields\\n    }}\\n    ... on Mirror {{\\n      ...MirrorFields\\n    }}\\n  }}\\n}}\\n\\nfragment PostFields on Post {{\\n  id\\n  onChainContentURI\\n  collectModule {{\\n    ...CollectModuleFields\\n  }}\\n}}\\n\\nfragment MirrorBaseFields on Mirror {{\\n  id\\n}}\\n\\nfragment MirrorFields on Mirror {{\\n  ...MirrorBaseFields\\n  mirrorOf {{\\n   ... on Post {{\\n      ...PostFields\\n   }}\\n  }}\\n}}\\n\\nfragment CollectModuleFields on CollectModule {{\\n  ... on FreeCollectModuleSettings {{\\n    type\\n    contractAddress\\n  }}\\n  ... on FeeCollectModuleSettings {{\\n    type\\n    contractAddress\\n  }}\\n  ... on LimitedFeeCollectModuleSettings {{\\n    type\\n    contractAddress\\n  }}\\n  ... on LimitedTimedFeeCollectModuleSettings {{\\n    type\\n    contractAddress\\n  }}\\n  ... on RevertCollectModuleSettings {{\\n    type\\n    contractAddress\\n  }}\\n  ... on TimedFeeCollectModuleSettings {{\\n    type\\n    contractAddress\\n  }}\\n  ... on MultirecipientFeeCollectModuleSettings {{\\n    type\\n    contractAddress\\n  }}\\n  ... on SimpleCollectModuleSettings {{\\n    type\\n    contractAddress\\n  }}\\n  ... on ERC4626FeeCollectModuleSettings {{\\n    type\\n    contractAddress\\n  }}\\n  ... on AaveFeeCollectModuleSettings {{\\n    type\\n    contractAddress\\n  }}\\n  ... on UnknownCollectModuleSettings {{\\n    type\\n    contractAddress\\n  }}\\n}}\"}}");

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
            //     "data": {
            //         "publication": {
            //             "__typename": "Post",
            //             "id": "0x01-0x01",
            //             "collectModule": {
            //                 "type": "FreeCollectModule",
            //                 "contractAddress": "0x23b9467334bEb345aAa6fd1545538F3d54436e96"
            //             }
            //         }
            //     }
            // },
            // {
            //     "data": {
            //         "publication": {
            //             "__typename": "Mirror",
            //             "id": "0x9d72-0x0457-DA-64abf0b0",
            //             "mirrorOf": {
            //                 "id": "0x05-0x1e8a-DA-6d1b60c9",
            //                 "collectModule": {
            //                     "type": "RevertCollectModule",
            //                     "contractAddress": "0xa31FF85E840ED117E172BC9Ad89E55128A999205"
            //                 }
            //             }
            //         }
            //     }
            // },
            // {
            //     "data": {
            //         "publication": {
            //             "__typename": "Comment"
            //         }
            //     }
            // }
            let resp_body = String::from_utf8(resp.body).or(Err(Error::FailedToDecode))?;
            let parsed: Response = json::from_str(&resp_body)
                .log_err("failed to parse json")
                .or(Err(Error::FailedToParseJson))?;

            let pub_info = parsed.data.publication.ok_or(Error::PublicationNotExists)?;
            let publication = match pub_info.__typename {
                "Post" => {
                    let id = pub_info.id.ok_or(Error::PublicationNotExists)?;
                    let (profile_id, pub_id) = Self::extract_ids(String::from(id))?;
                    let content_uri = pub_info.on_chain_content_uri.expect("no post content uri");
                    evm_publication(
                        U256::from(0),
                        U256::from(0),
                        content_uri.to_string(),
                        PublicationType::Post,
                        profile_id,
                        pub_id,
                    )
                }
                "Mirror" => {
                    // let id = pub_info.id.ok_or(Error::PublicationNotExists)?;
                    // let (profile_id, pub_id) = Self::extract_ids(String::from(id))?;
                    let mirror_of = pub_info.mirror_of.ok_or(Error::MissingMirrorField)?;
                    let root_id = mirror_of.id;
                    let (root_profile_id, root_pub_id) = Self::extract_ids(String::from(root_id))?;
                    // let root_collect_module = mirror_of.collect_module.contract_address;
                    // let root_collect_module: [u8; 20] = Self::decode_hex(root_collect_module)?
                    //     .try_into()
                    //     .or(Err(Error::FailedToParseAddress))?;
                    evm_publication(
                        root_profile_id,
                        root_pub_id,
                        mirror_of.on_chain_content_uri.to_string(),
                        PublicationType::Mirror,
                        root_profile_id,
                        root_pub_id,
                    )
                }
                "Comment" => Err(Error::NoProofForComment)?,
                _ => Err(Error::UnknownPublicationType)?,
            };

            let (profile_id, pub_id) = Self::extract_ids(publication_id)?;
            // free collect action
            let collect_act = H160::from(match mainnet {
                true => hex_literal::hex!("efBa1032bB5f9bEC79e022f52D89C2cc9090D1B8"),
                false => hex_literal::hex!("027AfbD7628221A0222eD4851EE63dF449d9dAE7"),
            });
            let free_collect = H160::from(match mainnet {
                true => hex_literal::hex!("c9205abC4A4fceC25E15446A8c2DD19ab60e1149"),
                false => hex_literal::hex!("2adb3d8fC5E5BB5552a342A0FB9fC23Ffb5d1Eee"),
            });
            Ok(evm_act_oracle_response_with_collect_act(
                profile_id,
                pub_id,
                publication,
                collect_act,
                free_collect,
            ))
        }

        fn extract_ids(publication_id: String) -> Result<(U256, U256)> {
            // e.g. "0x814a-0x01-DA-0e18b370"
            fn to_u32(s: &str) -> Result<u32> {
                u32::from_str_radix(s.trim_start_matches("0x"), 16)
                    .or(Err(Error::FailedToParseId))
            }
            let tokens: Vec<&str> = publication_id.split('-').collect();
            if tokens.len() != 4 {
                return Err(Error::FailedToParseId);
            }

            let profile_id = U256::from(to_u32(tokens[0])?);
            let pub_ref_id = to_u32(tokens[1])?;
            let da_id = to_u32(tokens[3])?;
            let pub_id = U256::from(pub_ref_id) | (U256::from(da_id) << 128);
            Ok((profile_id, pub_id))
        }

        pub fn decode_hex(s: &str) -> Result<Vec<u8>> {
            let stripped = s.strip_prefix("0x").unwrap_or(s);
            hex::decode(stripped).or(Err(Error::FailedToParseAddress))
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

    #[repr(u8)]
    pub enum PublicationType {
        Nonexistent,
        Post,
        Comment,
        Mirror,
        Quote,
    }

    pub fn evm_publication(
        pointed_profile_id: U256,
        pointed_pub_id: U256,
        content_uri: String,
        pub_type: PublicationType,
        root_profile_id: U256,
        root_pub_id: U256,
    ) -> Token {
        Token::Tuple(vec![
            Token::Uint(pointed_profile_id),
            Token::Uint(pointed_pub_id),
            Token::String(content_uri),
            // reference_module
            Token::Address(H160::default()),
            // deprecated collect_module
            Token::Address(H160::default()),
            // deprecated collect_nft
            Token::Address(H160::default()),
            Token::Uint(U256::from(pub_type as u8)),
            Token::Uint(root_profile_id),
            Token::Uint(root_pub_id),
            // enabled_action_modules_bitmap
            Token::Uint(U256::default())
        ])
    }

    pub fn evm_act_oracle_response_with_collect_act(
        profile_id: U256,
        pub_id: U256,
        publication: Token,
        collect_act: H160,
        free_collect_addr: H160
    ) -> Token {
        Token::Tuple(vec![
            Token::Uint(profile_id),
            Token::Uint(pub_id),
            publication,
            // referrer_pub_types
            Token::Array(vec![]),
            // action_modules
            Token::Array(vec![
                Token::Address(collect_act),
            ]),
            // action_modules_init_data
            Token::Array(vec![
                Token::Bytes(ethabi::encode(&[
                    Token::Address(free_collect_addr),
                    Token::Bytes(vec![])
                ]))
            ]),
            // referrer_profile_ids
            Token::Array(vec![]),
            // referrer_pub_ids
            Token::Array(vec![])
        ])
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

    // Define the structures to parse json response
    #[derive(Deserialize, Clone)]
    struct Response<'a> {
        #[serde(borrow)]
        data: ResponsePayload<'a>,
    }
    #[derive(Deserialize, Clone)]
    struct ResponsePayload<'a> {
        #[serde(borrow)]
        publication: Option<ResponseInner<'a>>,
    }
    #[derive(Deserialize, Clone)]
    struct ResponseInner<'a> {
        __typename: &'a str,
        #[serde(borrow)]
        id: Option<&'a str>,
        #[serde(borrow, alias = "onChainContentURI")]
        on_chain_content_uri: Option<&'a str>,
        #[serde(borrow, alias = "mirrorOf")]
        mirror_of: Option<MirrorOf<'a>>,
        #[serde(borrow, alias = "collectModule")]
        collect_module: Option<CollectModule<'a>>,
    }

    #[derive(Deserialize, Clone)]
    struct MirrorOf<'a> {
        id: &'a str,
        #[serde(alias = "onChainContentURI")]
        on_chain_content_uri: &'a str,
        #[serde(borrow, alias = "collectModule")]
        collect_module: CollectModule<'a>,
    }

    #[derive(Deserialize, Clone)]
    struct CollectModule<'a> {
        #[serde(alias = "type")]
        _module_type: &'a str,
        #[serde(alias = "contractAddress")]
        contract_address: &'a str,
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
        fn can_parse_lens_publication() {
            use std::str::FromStr;
            let _ = env_logger::try_init();
            pink_extension_runtime::mock_ext::mock_all_ext();

            let pub_resp = MomokaPublication::fetch_lens_publication(
                String::from("0x814a-0x01-DA-0e18b370"),
                false,
            )
            .unwrap();
            assert_eq!(
                pub_resp,
                Token::Tuple(vec![
                    Token::Uint(U256::from_str("0x814a").unwrap()),
                    Token::Uint(U256::from_str("0xe18b37000000000000000000000000000000001").unwrap()),
                    Token::Tuple(vec![
                        Token::Uint(U256::from(0)),
                        Token::Uint(U256::from(0)),
                        Token::String("ar://YhErKXFGi8pe4vR4w7vUc__KSmShdJ5_hLQJ7M9BTRU".to_string()),
                        Token::Address(H160::default()),
                        Token::Address(H160::default()),
                        Token::Address(H160::default()),
                        Token::Uint(U256::from(1)),
                        Token::Uint(U256::from_str("0x814a").unwrap()),
                        Token::Uint(U256::from_str("0xe18b37000000000000000000000000000000001").unwrap()),
                        Token::Uint(U256::from(0))
                    ]),
                    Token::Array(vec![]),
                    Token::Array(vec![Token::Address(H160::from(hex_literal::hex!("f4054e308f7804e34713c114a0c9e48e786a9a4c")))]),
                    Token::Array(vec![Token::Bytes(vec![])]),
                    Token::Array(vec![]),
                    Token::Array(vec![]),
                ])
            );

            let pub_resp = MomokaPublication::fetch_lens_publication(
                String::from("0x9d72-0x0457-DA-64abf0b0"),
                true,
            )
            .unwrap();
            assert_eq!(
                pub_resp,
                Token::Tuple(vec![
                    Token::Uint(U256::from_str("0x9d72").unwrap()),
                    Token::Uint(U256::from_str("0x64abf0b000000000000000000000000000000457").unwrap()),
                    Token::Tuple(vec![
                        Token::Uint(U256::from_str("0x5").unwrap()),
                        Token::Uint(U256::from_str("0x6d1b60c900000000000000000000000000001e8a").unwrap()),
                        Token::String("ar://s7-KUGt9F0TuJ4xTP01kbybqz0QLsk7NKp4zy4day1M".to_string()),
                        Token::Address(H160::default()),
                        Token::Address(H160::default()),
                        Token::Address(H160::default()),
                        Token::Uint(U256::from(3)),
                        Token::Uint(U256::from_str("0x5").unwrap()),
                        Token::Uint(U256::from_str("0x6d1b60c900000000000000000000000000001e8a").unwrap()),
                        Token::Uint(U256::from(0))
                    ]),
                    Token::Array(vec![]),
                    Token::Array(vec![Token::Address(H160::from(hex_literal::hex!("f4054e308f7804e34713c114a0c9e48e786a9a4c")))]),
                    Token::Array(vec![Token::Bytes(vec![])]),
                    Token::Array(vec![]),
                    Token::Array(vec![]),
                ])
            );
        }

        #[ink::test]
        fn fetch_lens_publicatino_negatives() {
            let _ = env_logger::try_init();
            pink_extension_runtime::mock_ext::mock_all_ext();

            let res = MomokaPublication::fetch_lens_publication(
                String::from("0x73b1-0x2b05-DA-ebdf984e"),
                true,
            );
            assert_eq!(res, Err(Error::NoProofForComment));

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
                .check_lens_publication(String::from("0x01-0x01ef-DA-eb395e21"), true)
                .expect("failed to check publication");
            pink::warn!("publication proof: {}", hex::encode(&r));
        }
    }
}
