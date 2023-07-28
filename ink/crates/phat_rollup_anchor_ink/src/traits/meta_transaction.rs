use crate::traits::rollup_anchor::{RollupAnchor, RollupAnchorError, RolupCondEqMethodParams};
use ink::env::hash::{Blake2x256, HashOutput};
use ink::prelude::vec::Vec;
use openbrush::contracts::access_control::{self, AccessControlError, RoleType};
use openbrush::storage::Mapping;
use openbrush::traits::{AccountId, Hash, Storage};

pub type Nonce = u128;
type NonceAndEcdsaPk = (Nonce, Vec<u8>);
pub type PrepareResult = (ForwardRequest, Hash);
pub type MetatTxRolupCondEqMethodParams = (ForwardRequest, [u8; 65]);

pub const MANAGER_ROLE: RoleType = ink::selector_id!("MANAGER_ROLE");

#[derive(Debug, Eq, PartialEq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum MetaTxError {
    NonceTooLow,
    IncorrectSignature,
    PublicKeyNotMatch,
    PublicKeyNotRegistered,
    AccessControlError(AccessControlError),
    RollupAnchorError(RollupAnchorError),
}

/// convertor from AccessControlError to MetaTxError
impl From<AccessControlError> for MetaTxError {
    fn from(error: AccessControlError) -> Self {
        MetaTxError::AccessControlError(error)
    }
}

/// convertor from RollupAnchorError to MetaTxError
impl From<RollupAnchorError> for MetaTxError {
    fn from(error: RollupAnchorError) -> Self {
        MetaTxError::RollupAnchorError(error)
    }
}

#[derive(Debug, Eq, PartialEq, Clone, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub struct ForwardRequest {
    pub from: AccountId,
    pub nonce: Nonce,
    pub data: Vec<u8>,
}

#[derive(Default, Debug)]
#[openbrush::storage_item]
pub struct Data {
    nonces_and_ecdsa_public_key: Mapping<AccountId, NonceAndEcdsaPk>,
}

#[openbrush::trait_definition]
pub trait MetaTxReceiver: Storage<Data> + access_control::Internal + RollupAnchor {
    #[ink(message)]
    fn get_ecdsa_public_key(&self, from: AccountId) -> [u8; 33] {
        match self.data::<Data>().nonces_and_ecdsa_public_key.get(&from) {
            None => [0; 33],
            Some((_, p)) => p.try_into().unwrap_or([0; 33]),
        }
    }

    #[ink(message)]
    #[openbrush::modifiers(access_control::only_role(MANAGER_ROLE))]
    fn register_ecdsa_public_key(
        &mut self,
        from: AccountId,
        ecdsa_public_key: [u8; 33],
    ) -> Result<(), MetaTxError> {
        match self.data::<Data>().nonces_and_ecdsa_public_key.get(&from) {
            None => self
                .data::<Data>()
                .nonces_and_ecdsa_public_key
                .insert(&from, &(0, ecdsa_public_key.into())),
            Some((n, _)) => self
                .data::<Data>()
                .nonces_and_ecdsa_public_key
                .insert(&from, &(n, ecdsa_public_key.into())),
        }
        Ok(())
    }

    #[ink(message)]
    fn prepare(
        &self,
        from: AccountId,
        data: Vec<u8>,
    ) -> Result<(ForwardRequest, Hash), MetaTxError> {
        let nonce = self.get_nonce(from);

        let request = ForwardRequest { from, nonce, data };
        let mut hash = <Blake2x256 as HashOutput>::Type::default();
        ink::env::hash_encoded::<Blake2x256, _>(&request, &mut hash);

        Ok((request, hash.into()))
    }

    fn get_nonce(&self, from: AccountId) -> Nonce {
        self.data::<Data>()
            .nonces_and_ecdsa_public_key
            .get(&from)
            .map(|(n, _)| n)
            .unwrap_or(0)
    }

    fn verify(&self, request: &ForwardRequest, signature: &[u8; 65]) -> Result<(), MetaTxError> {
        let (nonce_from, ecdsa_public_key) = match self
            .data::<Data>()
            .nonces_and_ecdsa_public_key
            .get(&request.from)
        {
            Some((n, p)) => (n, p),
            _ => return Err(MetaTxError::PublicKeyNotRegistered),
        };
        let ecdsa_public_key: [u8; 33] = ecdsa_public_key
            .try_into()
            .map_err(|_| MetaTxError::PublicKeyNotRegistered)?;

        if request.nonce < nonce_from {
            return Err(MetaTxError::NonceTooLow);
        }

        let mut hash = <Blake2x256 as HashOutput>::Type::default();
        ink::env::hash_encoded::<Blake2x256, _>(&request, &mut hash);

        // at the moment we can only verify ecdsa signatures
        let mut public_key = [0u8; 33];
        ink::env::ecdsa_recover(signature, &hash, &mut public_key)
            .map_err(|_| MetaTxError::IncorrectSignature)?;

        if public_key != ecdsa_public_key {
            return Err(MetaTxError::PublicKeyNotMatch);
        }
        Ok(())
    }

    fn use_meta_tx(
        &mut self,
        request: &ForwardRequest,
        signature: &[u8; 65],
    ) -> Result<(), MetaTxError> {
        // verify the signature
        self.verify(request, signature)?;
        // update the nonce
        match self
            .data::<Data>()
            .nonces_and_ecdsa_public_key
            .get(&request.from)
        {
            Some((_, p)) => self
                .data::<Data>()
                .nonces_and_ecdsa_public_key
                .insert(&request.from, &(request.nonce + 1, p)),
            None => return Err(MetaTxError::PublicKeyNotRegistered),
        }
        Ok(())
    }

    #[ink(message)]
    fn meta_tx_rollup_cond_eq(
        &mut self,
        request: ForwardRequest,
        signature: [u8; 65],
    ) -> Result<bool, MetaTxError> {
        // check the signature
        self.use_meta_tx(&request, &signature)?;

        // check the attestor role
        self.check_attestor_role(request.from)?;

        // decode the data
        let data: RolupCondEqMethodParams = scale::Decode::decode(&mut request.data.as_slice())
            .map_err(|_| RollupAnchorError::FailedToDecode)?;

        // emit the event
        self.emit_event_meta_tx_decoded();

        // call the rollup
        let result = self.inner_rollup_cond_eq(data.0, data.1, data.2)?;

        Ok(result)
    }
}

#[macro_export]
macro_rules! use_meta_tx {
    ($metaTxReceiver:ident, $request:ident, $signature:ident) => {{
        $metaTxReceiver._use_meta_tx(&$request, &$signature)?
    }};
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::test_contract::MyContract;
    use crate::traits::rollup_anchor::ATTESTOR_ROLE;
    use ink::env::debug_println;
    use openbrush::contracts::access_control::AccessControl;
    use openbrush::test_utils::accounts;
    use scale::Encode;

    #[ink::test]
    fn test_get_nonce() {
        let accounts = accounts();
        let contract = MyContract::new(accounts.bob);

        // no nonce (ie 0) for new account
        assert_eq!(0, contract.get_nonce(accounts.bob));
    }

    #[ink::test]
    fn test_prepare() {
        let accounts = accounts();
        let mut contract = MyContract::new(accounts.bob);

        // Alice
        let from = AccountId::from(hex_literal::hex!(
            "d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d"
        ));
        let ecdsa_public_key: [u8; 33] =
            hex_literal::hex!("037051bed73458951b45ca6376f4096c85bf1a370da94d5336d04867cfaaad019e");

        let data = u8::encode(&5);

        // register the ecda public key because I am not able to retrieve if from the account id
        contract
            .register_ecdsa_public_key(from, ecdsa_public_key)
            .expect("Error when registering ecdsa public key");

        // prepare the meta transaction
        let (request, hash) = contract
            .prepare(from, data.clone())
            .expect("Error when preparing meta tx");

        assert_eq!(0, request.nonce);
        assert_eq!(from, request.from);
        assert_eq!(&data, &request.data);

        debug_println!("code hash: {:02x?}", hash);
        let expected_hash =
            hex_literal::hex!("17cb4f6eae2f95ba0fbaee9e0e51dc790fe752e7386b72dcd93b9669450c2ccf");
        assert_eq!(&expected_hash, &hash.as_ref());
    }

    #[ink::test]
    fn test_verify() {
        let accounts = accounts();
        let mut contract = MyContract::new(accounts.bob);

        // Alice
        let from = AccountId::from(hex_literal::hex!(
            "d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d"
        ));
        let ecdsa_public_key: [u8; 33] =
            hex_literal::hex!("037051bed73458951b45ca6376f4096c85bf1a370da94d5336d04867cfaaad019e");

        // register the ecda public key because I am not able to retrieve if from the account id
        contract
            .register_ecdsa_public_key(from, ecdsa_public_key)
            .expect("Error when registering ecdsa public key");

        let nonce: Nonce = 0;
        let data = u8::encode(&5);
        let request = ForwardRequest {
            from,
            nonce,
            data: data.clone(),
        };

        // signature by Alice of hash : 17cb4f6eae2f95ba0fbaee9e0e51dc790fe752e7386b72dcd93b9669450c2ccf
        let signature = hex_literal::hex!("ce68d0383bd8f521a2243415add58ed0aed58c246229f15672ed6f99ba6c6c823a6d5fe7503703423e46206196c499d132533a151e2e7d9754b497a9d3014d9301");

        // the verification must succeed
        assert_eq!(Ok(()), contract.verify(&request, &signature));

        // incorrect 'from' => the verification must fail
        let request = ForwardRequest {
            from: accounts.bob,
            nonce,
            data: data.clone(),
        };
        assert_eq!(
            Err(MetaTxError::PublicKeyNotRegistered),
            contract.verify(&request, &signature)
        );

        // incorrect nonce => the verification must fail
        let request = ForwardRequest {
            from,
            nonce: 1,
            data: data.clone(),
        };
        assert_eq!(
            Err(MetaTxError::PublicKeyNotMatch),
            contract.verify(&request, &signature)
        );

        // incorrect data => the verification must fail
        let request = ForwardRequest {
            from,
            nonce,
            data: u8::encode(&55),
        };
        assert_eq!(
            Err(MetaTxError::PublicKeyNotMatch),
            contract.verify(&request, &signature)
        );

        // register another ecda public key
        let ecdsa_public_key =
            hex_literal::hex!("037051bed73458951b45ca6376f4096c85bf1a370da94d5336d04867cfaaad019f");
        contract
            .register_ecdsa_public_key(from, ecdsa_public_key)
            .expect("Error when registering ecdsa public key");
        // incorrect ecdsa public key => the verification must fail
        let request = ForwardRequest {
            from,
            nonce,
            data: data.clone(),
        };
        assert_eq!(
            Err(MetaTxError::PublicKeyNotMatch),
            contract.verify(&request, &signature)
        );
    }

    #[ink::test]
    fn test_use_meta_tx() {
        let accounts = accounts();
        let mut contract = MyContract::new(accounts.bob);

        // Alice
        let from = AccountId::from(hex_literal::hex!(
            "d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d"
        ));
        let ecdsa_public_key: [u8; 33] =
            hex_literal::hex!("037051bed73458951b45ca6376f4096c85bf1a370da94d5336d04867cfaaad019e");

        // register the ecda public key
        contract
            .register_ecdsa_public_key(from, ecdsa_public_key)
            .expect("Error when registering ecdsa public key");

        let nonce: Nonce = 0;
        let data = u8::encode(&5);
        let request = ForwardRequest {
            from,
            nonce,
            data: data.clone(),
        };

        // signature by Alice
        let signature = hex_literal::hex!("ce68d0383bd8f521a2243415add58ed0aed58c246229f15672ed6f99ba6c6c823a6d5fe7503703423e46206196c499d132533a151e2e7d9754b497a9d3014d9301");

        // the verification must succeed
        contract
            .use_meta_tx(&request, &signature)
            .expect("Error when using meta tx");

        // check if the nonce has been updated
        assert_eq!(1, contract.get_nonce(from));

        // test we cannot reuse the same call
        // the verification must fail
        assert_eq!(
            Err(MetaTxError::NonceTooLow),
            contract.use_meta_tx(&request, &signature)
        );
    }

    #[ink::test]
    fn test_meta_tx_rollup_cond_eq() {
        let accounts = accounts();
        let mut contract = MyContract::new(accounts.alice);

        // Alice
        let from = AccountId::from(hex_literal::hex!(
            "d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d"
        ));
        let ecdsa_public_key: [u8; 33] =
            hex_literal::hex!("037051bed73458951b45ca6376f4096c85bf1a370da94d5336d04867cfaaad019e");
        let data = RolupCondEqMethodParams::encode(&(vec![], vec![], vec![]));

        // register the ecdsa public key
        contract
            .register_ecdsa_public_key(from, ecdsa_public_key)
            .expect("Error when registering ecdsa public key");

        let (request, hash) = contract
            .prepare(from, data)
            .expect("Error when preparing meta tx");

        let expected_hash =
            hex_literal::hex!("c91f57305dc05a66f1327352d55290a250eb61bba8e3cf8560a4b8e7d172bb54");
        assert_eq!(&expected_hash, &hash.as_ref());

        // signature by Alice of previous hash
        let signature : [u8; 65] = hex_literal::hex!("c9a899bc8daa98fd1e819486c57f9ee889d035e8d0e55c04c475ca32bb59389b284d18d785a9db1bdd72ce74baefe6a54c0aa2418b14f7bc96232fa4bf42946600");

        // add the role => it should be succeed
        contract
            .grant_role(ATTESTOR_ROLE, Some(request.from))
            .expect("Error when grant the role Attestor");
        assert_eq!(
            Ok(true),
            contract.meta_tx_rollup_cond_eq(request.clone(), signature)
        );

        // do it again => it must failed
        assert_eq!(
            Err(MetaTxError::NonceTooLow),
            contract.meta_tx_rollup_cond_eq(request.clone(), signature)
        );
    }

    #[ink::test]
    fn test_meta_tx_rollup_cond_eq_missing_role() {
        let accounts = accounts();
        let mut contract = MyContract::new(accounts.alice);

        // Alice
        let from = AccountId::from(hex_literal::hex!(
            "d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d"
        ));
        let ecdsa_public_key: [u8; 33] =
            hex_literal::hex!("037051bed73458951b45ca6376f4096c85bf1a370da94d5336d04867cfaaad019e");
        let data = RolupCondEqMethodParams::encode(&(vec![], vec![], vec![]));

        // register the ecdsa public key
        contract
            .register_ecdsa_public_key(from, ecdsa_public_key)
            .expect("Error when registering ecdsa public key");

        let (request, hash) = contract
            .prepare(from, data)
            .expect("Error when preparing meta tx");

        let expected_hash =
            hex_literal::hex!("c91f57305dc05a66f1327352d55290a250eb61bba8e3cf8560a4b8e7d172bb54");
        assert_eq!(&expected_hash, &hash.as_ref());

        // signature by Alice of previous hash
        let signature : [u8; 65] = hex_literal::hex!("c9a899bc8daa98fd1e819486c57f9ee889d035e8d0e55c04c475ca32bb59389b284d18d785a9db1bdd72ce74baefe6a54c0aa2418b14f7bc96232fa4bf42946600");

        // missing role
        assert_eq!(
            Err(MetaTxError::RollupAnchorError(
                RollupAnchorError::AccessControlError(
                    access_control::AccessControlError::MissingRole
                )
            )),
            contract.meta_tx_rollup_cond_eq(request.clone(), signature)
        );
    }
}
