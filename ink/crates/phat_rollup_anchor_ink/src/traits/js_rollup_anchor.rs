use crate::traits::rollup_anchor::RollupAnchorError;
use ink::prelude::vec::Vec;
use ink::storage::Lazy;
use openbrush::contracts::access_control::{self, AccessControlError, RoleType};
use openbrush::traits::Storage;
use scale::{Decode, Encode};


pub const MANAGER_ROLE: RoleType = ink::selector_id!("JS_MANAGER_ROLE");

#[derive(Debug, Eq, PartialEq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum JsRollupAnchorError {
    AccessControlError(AccessControlError),
}

/// convertor from AccessControlError to JsRollupAnchorError
impl From<AccessControlError> for JsRollupAnchorError {
    fn from(error: AccessControlError) -> Self {
        JsRollupAnchorError::AccessControlError(error)
    }
}

type CodeHash = [u8; 32];

/// Message sent to provide the data
/// response pushed in the queue by the offchain rollup and read by the Ink! smart contract
#[derive(Encode, Decode)]
#[cfg_attr(
    feature = "std",
    derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
)]
pub enum ResponseMessage {
    JsResponse {
        /// hash of js script executed to get the data
        js_script_hash: CodeHash,
        /// hash of data in input of js
        input_hash: CodeHash,
        /// hash of settings of js
        settings_hash: CodeHash,
        /// response value
        output_value: Vec<u8>,
    },
    Error {
        /// hash of js script
        js_script_hash: CodeHash,
        /// input in js
        input_value: Vec<u8>,
        /// hash of settings of js
        settings_hash: CodeHash,
        /// when an error occurs
        error: Vec<u8>,
    },
}

/// Enum used in the function on_message_received to return the result
/// INPUT is the request type sent from the smart contract to phat offchain rollup
/// OUTPUT is the response type receive by the smart contract
pub enum MessageReceived<INPUT, OUTPUT> {
    Ok { output: OUTPUT },
    Error { input: INPUT, error: Vec<u8> },
}

#[derive(Default, Debug)]
#[openbrush::storage_item]
pub struct Data {
    /// hash of js script executed to query the data
    js_script_hash: Lazy<CodeHash>,
    /// hash of settings given in parameter to js runner
    settings_hash: Lazy<CodeHash>,
}

#[openbrush::trait_definition]
pub trait JsRollupAnchor:
    Storage<Data> + access_control::Internal
{
    #[ink(message)]
    #[openbrush::modifiers(access_control::only_role(MANAGER_ROLE))]
    fn set_js_script_hash(&mut self, js_script_hash: CodeHash) -> Result<(), JsRollupAnchorError> {
        self.data::<Data>().js_script_hash.set(&js_script_hash);
        Ok(())
    }

    #[ink(message)]
    fn get_js_script_hash(&self) -> Option<CodeHash> {
        self.data::<Data>().js_script_hash.get()
    }

    #[ink(message)]
    #[openbrush::modifiers(access_control::only_role(MANAGER_ROLE))]
    fn set_settings_hash(&mut self, settings_hash: CodeHash) -> Result<(), JsRollupAnchorError> {
        self.data::<Data>().settings_hash.set(&settings_hash);
        Ok(())
    }

    #[ink(message)]
    fn get_settings_hash(&self) -> Option<CodeHash> {
        self.data::<Data>().settings_hash.get()
    }

    fn on_message_received<I: Decode, O: Decode>(
        &mut self,
        action: Vec<u8>,
    ) -> Result<MessageReceived<I, O>, RollupAnchorError> {
        // parse the response
        let response: ResponseMessage =
            Decode::decode(&mut &action[..]).or(Err(RollupAnchorError::FailedToDecode))?;

        match response {
            ResponseMessage::JsResponse {
                js_script_hash,
                settings_hash,
                output_value,
                ..
            } => {
                // check the js code hash
                match self.data::<Data>().js_script_hash.get() {
                    Some(expected_js_hash) => {
                        if js_script_hash != expected_js_hash {
                            return Err(RollupAnchorError::ConditionNotMet); // improve the error
                        }
                    }
                    None => {}
                }

                // check the settings hash
                match self.data::<Data>().settings_hash.get() {
                    Some(expected_settings_hash) => {
                        if settings_hash != expected_settings_hash {
                            return Err(RollupAnchorError::ConditionNotMet); // improve the error
                        }
                    }
                    None => {}
                }

                // we received the data
                let output = O::decode(&mut output_value.as_slice())
                    .map_err(|_| RollupAnchorError::FailedToDecode)?;
                Ok(MessageReceived::<I,O>::Ok { output })
            }
            ResponseMessage::Error {
                error, input_value, ..
            } => {
                // we received an error
                let input = I::decode(&mut input_value.as_slice())
                    .map_err(|_| RollupAnchorError::FailedToDecode)?;
                Ok(MessageReceived::<I,O>::Error { input, error })
            }
        }
    }
}
