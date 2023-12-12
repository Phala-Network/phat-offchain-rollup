#![cfg_attr(not(feature = "std"), no_std, no_main)]

extern crate alloc;

pub use crate::momoka_publication::*;

#[ink::contract(env = pink_extension::PinkEnvironment)]
mod momoka_publication {
    use alloc::{string::String, vec, vec::Vec};
    use ethabi::Token;
    use ink::storage::traits::StorageLayout;
    use pink_extension as pink;
    use pink_extension::chain_extension::signing;
    use pink_web3::{
        api::{Eth, Namespace},
        contract::{Contract, Options},
        keys::pink::KeyPair,
        signing::Key,
        transports::{resolve_ready, PinkHttp},
        types::{Bytes, H160, U256},
    };
    use scale::{Decode, Encode};
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
        NoProofForQuote,
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
            let lens_api = if mainnet {
                "https://api-v2.lens.dev/"
            } else {
                "https://api-mumbai-v2.lens.dev/"
            };
            let headers = vec![
                ("Content-Type".into(), "application/json".into()),
                ("User-Agent".into(), "phat-contract".into()),
            ];
            let body = String::from(r#"{"query":"query Publication {\n  publication(request: { forId: \"${publicationId}\" }) {\n    __typename\n    ... on Post {\n      ...PostFields\n    }\n    ... on Mirror {\n      ...MirrorFields\n    }\n  }\n}\n\nfragment PostFields on Post {\n  id\n  metadata {\n    __typename\n    ... on TextOnlyMetadataV3 {\n      id\n      rawURI\n      contentWarning\n      content\n    }\n  }\n  openActionModules {\n    __typename\n  }\n}\n\nfragment MirrorFields on Mirror {\n  id\n  mirrorOn {\n    __typename\n    ... on Post {\n      ...PostFields\n    }\n  }\n}\n"}"#)
                .replace("${publicationId}", &publication_id);

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

            let resp_body = String::from_utf8(resp.body).or(Err(Error::FailedToDecode))?;

            let script: &str = r"
            (function() {
                const [ jsonStr ] = scriptArgs;
                const obj = JSON.parse(jsonStr);
                let mirrorOn = '', post, pubType;
                if (obj.data.publication.__typename == 'Mirror') {
                    pubType = 3;
                    mirrorOn = obj.data.publication.mirrorOn.id;
                    post = obj.data.publication.mirrorOn;
                } else {
                    pubType = 1;
                    post = obj.data.publication;
                }
                if (post.__typename != 'Post' 
                  || post.metadata.__typename != 'TextOnlyMetadataV3') {
                    throw new Exception('Unsupported publication');
                }
                const contentUri = post.metadata.rawURI;
                // ---- codec
                const typedef = 'PublicationData = {pubType:u8, pointedTo:str, contentUri:str}';
                const output = {pubType, contentUri, pointedTo: mirrorOn};
                return Pink.SCALE.encode(output, 'PublicationData', typedef);
            })()
            ";

            let js_output = phat_js::eval_async_js(script, &[resp_body]);
            let phat_js::JsValue::Bytes(encoded) = js_output else {
                return Err(Error::FailedToParseJson);
            };
            let pub_data = PublicationData::decode(&mut &encoded[..]).expect("encoded by js; qed.");
            
            let (profile_id, pub_id) = Self::extract_ids(publication_id)?;
            let publication = match pub_data.pub_type {
                1 => {
                    evm_publication_memory(
                        U256::from(0),
                        U256::from(0),
                        pub_data.content_uri,
                        1,
                        profile_id,
                        pub_id,
                    )
                }
                3 => {
                    let (root_profile_id, root_pub_id) = Self::extract_ids(String::from(pub_data.pointed_to))?;
                    evm_publication_memory(
                        root_profile_id,
                        root_pub_id,
                        pub_data.content_uri,
                        3,
                        root_profile_id,
                        root_pub_id,
                    )
                }
                2 => Err(Error::NoProofForComment)?,
                4 => Err(Error::NoProofForQuote)?,
                _ => Err(Error::UnknownPublicationType)?,
            };

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

    // #[repr(u8)]
    // pub enum PublicationType {
    //     Nonexistent,
    //     Post,
    //     Comment,
    //     Mirror,
    //     Quote,
    // }

    #[derive(Decode)]
    struct PublicationData {
        pub_type: u8,
        pointed_to: String,
        content_uri: String,
    }

    pub fn evm_publication_memory(
        pointed_profile_id: U256,
        pointed_pub_id: U256,
        content_uri: String,
        pub_type: u8,
        root_profile_id: U256,
        root_pub_id: U256,
    ) -> Token {
        Token::Tuple(vec![
            Token::Uint(pointed_profile_id),
            Token::Uint(pointed_pub_id),
            Token::String(content_uri),
            // TODO: reference_module
            Token::Address(H160::default()),
            // deprecated collect_module
            Token::Address(H160::default()),
            // deprecated collect_nft
            Token::Address(H160::default()),
            Token::Uint(U256::from(pub_type as u8)),
            Token::Uint(root_profile_id),
            Token::Uint(root_pub_id),
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
}

#[cfg(test)]
mod tests {
    use crate::momoka_publication::*;

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

    use ::ink::env::call::{
        utils::{ReturnType, Set, Unset},
        Call, ExecutionInput,
    };
    use drink::{errors::MessageResult, runtime::Runtime, session::Session, ContractBundle};
    use drink_pink_runtime::{ExecMode, PinkRuntime};
    use ink::codegen::TraitCallBuilder;
    use ink::env::{
        call::{CallBuilder, CreateBuilder, FromAccountId},
        Environment,
    };
    use ink::primitives::Hash;
    use pink_extension::Balance;
    use pink_extension::ConvertTo;
    use scale::{Decode, Encode};

    const DEFAULT_GAS_LIMIT: u64 = 1_000_000_000_000_000;

    #[drink::contract_bundle_provider]
    enum BundleProvider {}

    fn deploy_bundle<Env, Contract, Args>(
        session: &mut Session<PinkRuntime>,
        bundle: ContractBundle,
        constructor: CreateBuilder<
            Env,
            Contract,
            Unset<Hash>,
            Unset<u64>,
            Unset<Balance>,
            Set<ExecutionInput<Args>>,
            Set<Vec<u8>>,
            Set<ReturnType<Contract>>,
        >,
    ) -> core::result::Result<Contract, String>
    where
        Env: Environment<Hash = Hash, Balance = Balance>,
        Contract: FromAccountId<Env>,
        Args: Encode,
        Env::AccountId: From<[u8; 32]>,
    {
        session.execute_with(move || {
            let caller = PinkRuntime::default_actor();
            let code_hash = PinkRuntime::upload_code(caller.clone(), bundle.wasm, true)?;
            let constructor = constructor
                .code_hash(code_hash.0.into())
                .endowment(0)
                .gas_limit(DEFAULT_GAS_LIMIT);
            let params = constructor.params();
            let input_data = params.exec_input().encode();
            let account_id = PinkRuntime::instantiate(
                caller,
                0,
                params.gas_limit(),
                None,
                code_hash,
                input_data,
                params.salt_bytes().clone(),
            )?;
            Ok(Contract::from_account_id(account_id.convert_to()))
        })
    }

    fn call<Env, Args, Ret>(
        session: &mut Session<PinkRuntime>,
        call_builder: CallBuilder<
            Env,
            Set<Call<Env>>,
            Set<ExecutionInput<Args>>,
            Set<ReturnType<Ret>>,
        >,
        deterministic: bool,
    ) -> core::result::Result<Ret, String>
    where
        Env: Environment<Hash = Hash, Balance = Balance>,
        Args: Encode,
        Ret: Decode,
    {
        session.execute_with(move || {
            let origin = PinkRuntime::default_actor();
            let params = call_builder.params();
            let data = params.exec_input().encode();
            let callee = params.callee();
            let address: [u8; 32] = callee.as_ref().try_into().or(Err("Invalid callee"))?;
            let result = PinkRuntime::call(
                origin,
                address.into(),
                0,
                DEFAULT_GAS_LIMIT,
                None,
                data,
                deterministic,
            )?;
            let ret = MessageResult::<Ret>::decode(&mut &result[..])
                .map_err(|e| format!("Failed to decode result: {}", e))?
                .map_err(|e| format!("Failed to execute call: {}", e))?;
            Ok(ret)
        })
    }

    #[drink::test]
    fn check_lens_publication_works() -> Result<(), Box<dyn std::error::Error>> {
        let _ = env_logger::try_init();
        let EnvVars {
            rpc,
            client_addr,
            attest_key,
        } = config();

        let mut session = Session::<PinkRuntime>::new()?;
        session.execute_with(|| {
            PinkRuntime::setup_cluster().expect("Failed to setup cluster");
        });

        let bundle = BundleProvider::local()?;
        let mut contract = deploy_bundle(
            &mut session,
            bundle,
            MomokaPublicationRef::default().salt_bytes(vec![]),
        )?;

        let result = call(
            &mut session,
            contract.call_mut().config_client(rpc, client_addr),
            true,
        )?;

        let result = call(
            &mut session,
            contract.call().check_lens_publication(String::from("0x01-0x01ef-DA-eb395e21"), true),
            false,
        )?
        .expect("check_lens_publication failed");
        println!("publication proof: {}", hex::encode(&result));
        Ok(())
    }

    /*
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
    fn fetch_lens_publication_negatives() {
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
*/
}
