#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub mod ffi_utils {
    use anyhow::{anyhow, Result};
    use std::{
        collections::HashMap,
        ffi::{CStr, CString},
        os::raw::c_char,
    };

    use crate::{
        ecdsa::PrivateShare, utilities::requests::ClientShim, dto::ecdsa::MKPosDto,
    };

    pub fn get_client_shim_from_raw(
        c_endpoint: *const c_char,
        c_auth_token: *const c_char,
        c_user_id: *const c_char,
    ) -> Result<ClientShim> {
        let raw_endpoint_json = unsafe { CStr::from_ptr(c_endpoint) };
        let endpoint = match raw_endpoint_json.to_str() {
            Ok(s) => s,
            Err(e) => return Err(anyhow!("Decode C string endpoint failed: {}", e)),
        };

        let raw_auth_token_json = unsafe { CStr::from_ptr(c_auth_token) };
        let auth_token = match raw_auth_token_json.to_str() {
            Ok(s) => s,
            Err(e) => return Err(anyhow!("Decode C string auth_token failed: {}", e)),
        };

        let user_id_json = unsafe { CStr::from_ptr(c_user_id) };
        let user_id = match user_id_json.to_str() {
            Ok(s) => s,
            Err(e) => return Err(anyhow!("Decode C string user_id failed: {}", e)),
        };

        Ok(ClientShim::new(
            endpoint.to_owned(),
            Some(auth_token.to_owned()),
            user_id.to_owned(),
        ))
    }

    pub fn get_private_share_from_raw(c_private_share_json: *const c_char) -> Result<PrivateShare> {
        let raw_private_share_json = unsafe { CStr::from_ptr(c_private_share_json) };
        let private_share_json = match raw_private_share_json.to_str() {
            Ok(s) => s,
            Err(e) => return Err(anyhow!("Decode C string private_share failed: {}", e)),
        };

        let private_share: PrivateShare = match serde_json::from_str(private_share_json) {
            Ok(s) => s,
            Err(e) => return Err(anyhow!("Deserialize private_share failed: {}", e)),
        };

        Ok(private_share)
    }

    pub fn get_addresses_derivation_map_from_raw(
        c_addresses_derivation_map: *const c_char,
    ) -> Result<HashMap<String, MKPosDto>> {
        let raw_addresses_derivation_map_json =
            unsafe { CStr::from_ptr(c_addresses_derivation_map) };
        let addresses_derivation_map_json = match raw_addresses_derivation_map_json.to_str() {
            Ok(s) => s,
            Err(e) => {
                return Err(anyhow!(
                    "Decode C string addresses_derivation_map failed: {}",
                    e
                ))
            }
        };

        let addresses_derivation_map: HashMap<String, MKPosDto> =
            match serde_json::from_str(addresses_derivation_map_json) {
                Ok(s) => s,
                Err(e) => {
                    return Err(anyhow!(
                        "Deserialize addresses_derivation_map failed: {}",
                        e
                    ))
                }
            };

        Ok(addresses_derivation_map)
    }

    #[no_mangle]
    pub extern "C" fn cstring_free(cstring: *mut c_char) {
        if cstring.is_null() {
            return;
        }
        unsafe { CString::from_raw(cstring) };
    }
}
