use ink::env::hash::{Blake2x256, HashOutput};
use ink::prelude::vec::Vec;
use openbrush::storage::Mapping;
use openbrush::traits::{AccountId, Hash, Storage};

pub use crate::traits::meta_transaction::{self, *};

type NonceAndEcdsaPk = (Nonce, Vec<u8>);

#[derive(Default, Debug)]
#[openbrush::storage_item]
pub struct Data {
    nonces_and_ecdsa_public_key: Mapping<AccountId, NonceAndEcdsaPk>,
}

pub trait MetaTxReceiverImpl: Internal + Storage<Data> {

    fn get_ecdsa_public_key(&self, from: AccountId) -> [u8; 33] {
        match self.data::<Data>().nonces_and_ecdsa_public_key.get(&from) {
            None => [0; 33],
            Some((_, p)) => p.try_into().unwrap_or([0; 33]),
        }
    }

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

    fn prepare(
        &self,
        from: AccountId,
        data: Vec<u8>,
    ) -> Result<(ForwardRequest, Hash), MetaTxError> {
        let nonce = self._get_nonce(from);

        let request = ForwardRequest { from, nonce, data };
        let mut hash = <Blake2x256 as HashOutput>::Type::default();
        ink::env::hash_encoded::<Blake2x256, _>(&request, &mut hash);

        Ok((request, hash.into()))
    }
}

pub trait InternalImpl : Storage<Data> {

    fn _get_nonce(&self, from: AccountId) -> Nonce {
        self.data::<Data>()
            .nonces_and_ecdsa_public_key
            .get(&from)
            .map(|(n, _)| n)
            .unwrap_or(0)
    }

    fn _verify(
        &self,
        request: &ForwardRequest,
        signature: &[u8; 65],
    ) -> Result<(), MetaTxError> {
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

    fn _use_meta_tx(
        &mut self,
        request: &ForwardRequest,
        signature: &[u8; 65],
    ) -> Result<(), MetaTxError> {
        // verify the signature
        self._verify(request, signature)?;
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
}

#[macro_export]
macro_rules! use_meta_tx {
    ($metaTxReceiver:ident, $request:ident, $signature:ident) => {{
        $metaTxReceiver._use_meta_tx(&$request, &$signature)?
    }};
}

#[cfg(test)]
mod tests {
    use ink::env::debug_println;
    use openbrush::test_utils::accounts;
    use scale::Encode;

    use crate::impls::meta_transaction::*;
    use crate::tests::test_contract::MyContract;

    #[ink::test]
    fn test_get_nonce() {
        let accounts = accounts();
        let contract = MyContract::new(accounts.bob);

        // no nonce (ie 0) for new account
        assert_eq!(0, contract._get_nonce(accounts.bob));
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
        assert_eq!(Ok(()), contract._verify(&request, &signature));

        // incorrect 'from' => the verification must fail
        let request = ForwardRequest {
            from: accounts.bob,
            nonce,
            data: data.clone(),
        };
        assert_eq!(
            Err(MetaTxError::PublicKeyNotRegistered),
            contract._verify(&request, &signature)
        );

        // incorrect nonce => the verification must fail
        let request = ForwardRequest {
            from,
            nonce: 1,
            data: data.clone(),
        };
        assert_eq!(
            Err(MetaTxError::PublicKeyNotMatch),
            contract._verify(&request, &signature)
        );

        // incorrect data => the verification must fail
        let request = ForwardRequest {
            from,
            nonce,
            data: u8::encode(&55),
        };
        assert_eq!(
            Err(MetaTxError::PublicKeyNotMatch),
            contract._verify(&request, &signature)
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
            contract._verify(&request, &signature)
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
            ._use_meta_tx(&request, &signature)
            .expect("Error when using meta tx");

        // check if the nonce has been updated
        assert_eq!(1, contract._get_nonce(from));

        // test we cannot reuse the same call
        // the verification must fail
        assert_eq!(
            Err(MetaTxError::NonceTooLow),
            contract._use_meta_tx(&request, &signature)
        );
    }
}
