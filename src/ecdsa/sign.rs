use anyhow::{anyhow, Result};
use curv::BigInt;
use kms::ecdsa::two_party::party2;
use kms::ecdsa::two_party::MasterKey2;
use multi_party_ecdsa::protocols::two_party_ecdsa::lindell_2017::party_one;
use multi_party_ecdsa::protocols::two_party_ecdsa::lindell_2017::party_two;

use super::super::utilities::requests;
use crate::utilities::a_requests;
use crate::utilities::a_requests::AsyncClientShim;
use crate::utilities::err_handling::error_to_c_string;
use crate::utilities::err_handling::ErrorFFIKind;
use crate::utilities::requests::ClientShim;

// iOS bindings
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

#[derive(Serialize, Deserialize, Debug)]
pub struct SignSecondMsgRequest {
    pub message: BigInt,
    pub party_two_sign_message: party2::SignMessage,
    pub x_pos_child_key: BigInt,
    pub y_pos_child_key: BigInt,
}

pub fn sign(
    client_shim: &ClientShim,
    message: BigInt,
    mk: &MasterKey2,
    x_pos: BigInt,
    y_pos: BigInt,
    id: &str,
) -> Result<party_one::SignatureRecid> {
    // Choose ephemeral key
    let (eph_key_gen_first_message_party_two, eph_comm_witness, eph_ec_key_pair_party2) =
        MasterKey2::sign_first_message();

    let request: party_two::EphKeyGenFirstMsg = eph_key_gen_first_message_party_two;

    // Repeat Key Generation protocol for ephemeral key to obtain random point on curve that will be used in generating signature
    let sign_party_one_first_message: party_one::EphKeyGenFirstMsg =
        match requests::postb(client_shim, &format!("/ecdsa/sign/{}/first", id), &request)? {
            Some(s) => s,
            None => return Err(anyhow!("party1 sign first message request failed")),
        };

    // Generate encryption of derivative of the signature, called c3
    let party_two_sign_message = mk.sign_second_message(
        &eph_ec_key_pair_party2,
        eph_comm_witness,
        &sign_party_one_first_message,
        &message,
    );

    // Send c3 to P1 to verify and get valid signature
    let signature = match get_signature(
        client_shim,
        message,
        party_two_sign_message,
        x_pos,
        y_pos,
        id,
    ) {
        Ok(s) => s,
        Err(e) => return Err(anyhow!("ecdsa::get_signature failed failed: {}", e)),
    };

    Ok(signature)
}

pub async fn a_sign(
    client_shim: &AsyncClientShim,
    message: BigInt,
    mk: &MasterKey2,
    x_pos: BigInt,
    y_pos: BigInt,
    id: &str,
) -> Result<party_one::SignatureRecid> {
    let (eph_key_gen_first_message_party_two, eph_comm_witness, eph_ec_key_pair_party2) =
        MasterKey2::sign_first_message();

    let request: party_two::EphKeyGenFirstMsg = eph_key_gen_first_message_party_two;
    let sign_party_one_first_message: party_one::EphKeyGenFirstMsg =
        match a_requests::a_postb(client_shim, &format!("/ecdsa/sign/{}/first", id), &request)
            .await?
        {
            Some(s) => s,
            None => return Err(anyhow!("party1 sign first message request failed")),
        };

    let party_two_sign_message = mk.sign_second_message(
        &eph_ec_key_pair_party2,
        eph_comm_witness,
        &sign_party_one_first_message,
        &message,
    );

    let signature = match a_get_signature(
        client_shim,
        message,
        party_two_sign_message,
        x_pos,
        y_pos,
        id,
    )
    .await
    {
        Ok(s) => s,
        Err(e) => return Err(anyhow!("ecdsa::get_signature failed failed: {}", e)),
    };

    Ok(signature)
}

fn get_signature(
    client_shim: &ClientShim,
    message: BigInt,
    party_two_sign_message: party2::SignMessage,
    x_pos_child_key: BigInt,
    y_pos_child_key: BigInt,
    id: &str,
) -> Result<party_one::SignatureRecid> {
    let request: SignSecondMsgRequest = SignSecondMsgRequest {
        message,
        party_two_sign_message,
        x_pos_child_key,
        y_pos_child_key,
    };

    let signature: party_one::SignatureRecid =
        match requests::postb(client_shim, &format!("/ecdsa/sign/{}/second", id), &request)? {
            Some(s) => s,
            None => return Err(anyhow!("party1 sign second message request failed",)),
        };

    Ok(signature)
}

async fn a_get_signature(
    client_shim: &AsyncClientShim,
    message: BigInt,
    party_two_sign_message: party2::SignMessage,
    x_pos_child_key: BigInt,
    y_pos_child_key: BigInt,
    id: &str,
) -> Result<party_one::SignatureRecid> {
    let request: SignSecondMsgRequest = SignSecondMsgRequest {
        message,
        party_two_sign_message,
        x_pos_child_key,
        y_pos_child_key,
    };

    let signature: party_one::SignatureRecid =
        match a_requests::a_postb(client_shim, &format!("/ecdsa/sign/{}/second", id), &request)
            .await?
        {
            Some(s) => s,
            None => return Err(anyhow!("party1 sign second message request failed",)),
        };

    Ok(signature)
}

#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn sign_message(
    c_endpoint: *const c_char,
    c_auth_token: *const c_char,
    c_user_id: *const c_char,
    c_message_le_hex: *const c_char,
    c_master_key_json: *const c_char,
    c_x_pos: i32,
    c_y_pos: i32,
    c_id: *const c_char,
) -> *mut c_char {
    let raw_endpoint = unsafe { CStr::from_ptr(c_endpoint) };
    let endpoint = match raw_endpoint.to_str() {
        Ok(s) => s,
        Err(e) => {
            return error_to_c_string(ErrorFFIKind::E100 {
                msg: "endpoint".to_owned(),
                e: e.to_string(),
            })
        }
    };

    let raw_auth_token = unsafe { CStr::from_ptr(c_auth_token) };
    let auth_token = match raw_auth_token.to_str() {
        Ok(s) => s,
        Err(e) => {
            return error_to_c_string(ErrorFFIKind::E100 {
                msg: "auth_token".to_owned(),
                e: e.to_string(),
            })
        }
    };

    let user_id_json = unsafe { CStr::from_ptr(c_user_id) };
    let user_id = match user_id_json.to_str() {
        Ok(s) => s,
        Err(e) => {
            return error_to_c_string(ErrorFFIKind::E100 {
                msg: "user_id".to_owned(),
                e: e.to_string(),
            })
        }
    };

    let raw_message_hex = unsafe { CStr::from_ptr(c_message_le_hex) };
    let message_hex = match raw_message_hex.to_str() {
        Ok(s) => s,
        Err(e) => {
            return error_to_c_string(ErrorFFIKind::E100 {
                msg: "message_hex".to_owned(),
                e: e.to_string(),
            })
        }
    };

    let raw_master_key_json = unsafe { CStr::from_ptr(c_master_key_json) };
    let master_key_json = match raw_master_key_json.to_str() {
        Ok(s) => s,
        Err(e) => {
            return error_to_c_string(ErrorFFIKind::E100 {
                msg: "master_key_json".to_owned(),
                e: e.to_string(),
            })
        }
    };

    let raw_id = unsafe { CStr::from_ptr(c_id) };
    let id = match raw_id.to_str() {
        Ok(s) => s,
        Err(e) => {
            return error_to_c_string(ErrorFFIKind::E100 {
                msg: "id".to_owned(),
                e: e.to_string(),
            })
        }
    };

    let x: BigInt = BigInt::from(c_x_pos);

    let y: BigInt = BigInt::from(c_y_pos);

    let client_shim = ClientShim::new(
        endpoint.to_owned(),
        Some(auth_token.to_owned()),
        user_id.to_owned(),
    );

    let mk: MasterKey2 = serde_json::from_str(master_key_json).unwrap();

    let mk_child: MasterKey2 = mk.get_child(vec![x.clone(), y.clone()]);

    let message: BigInt = serde_json::from_str(message_hex).unwrap();

    let sig = match sign(&client_shim, message, &mk_child, x, y, id) {
        Ok(s) => s,
        Err(e) => {
            return error_to_c_string(ErrorFFIKind::E103 {
                msg: "sig".to_owned(),
                e: e.to_string(),
            })
        }
    };

    let signature_json = match serde_json::to_string(&sig) {
        Ok(share) => share,
        Err(e) => {
            return error_to_c_string(ErrorFFIKind::E103 {
                msg: "signature_json".to_owned(),
                e: e.to_string(),
            })
        }
    };

    CString::new(signature_json).unwrap().into_raw()
}
