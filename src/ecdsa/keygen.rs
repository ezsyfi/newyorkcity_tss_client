use floating_duration::TimeFormat;
use serde_json;
use std::time::Instant;

use curv::cryptographic_primitives::twoparty::dh_key_exchange_variant_with_pok_comm::*;
use curv::elliptic::curves::secp256_k1::GE;

use kms::chain_code::two_party as chain_code;
use kms::ecdsa::two_party::*;
use multi_party_ecdsa::protocols::two_party_ecdsa::lindell_2017::*;
use zk_paillier::zkproofs::SALT_STRING;

use crate::utilities::requests::ClientShim;

use super::super::utilities::requests;
use super::types::PrivateShare;

// iOS bindings
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

const KG_PATH_PRE: &str = "ecdsa/keygen";

pub fn get_master_key(client_shim: &ClientShim) -> PrivateShare {
    let start = Instant::now();

    let (id, kg_party_one_first_message): (String, party_one::KeyGenFirstMsg) =
        requests::post(client_shim, &format!("{}/first", KG_PATH_PRE)).unwrap();

    let (kg_party_two_first_message, kg_ec_key_pair_party2) = MasterKey2::key_gen_first_message();

    let body = &kg_party_two_first_message.d_log_proof;

    let kg_party_one_second_message: party1::KeyGenParty1Message2 =
        requests::postb(client_shim, &format!("{}/{}/second", KG_PATH_PRE, id), body).unwrap();

    let key_gen_second_message = MasterKey2::key_gen_second_message(
        &kg_party_one_first_message,
        &kg_party_one_second_message,
        SALT_STRING,
    );

    let (_, party_two_paillier) = key_gen_second_message.unwrap();

    let cc_party_one_first_message: Party1FirstMessage = requests::post(
        client_shim,
        &format!("{}/{}/chaincode/first", KG_PATH_PRE, id),
    )
    .unwrap();

    let (cc_party_two_first_message, cc_ec_key_pair2) =
        chain_code::party2::ChainCode2::chain_code_first_message();

    let body = &cc_party_two_first_message.d_log_proof;

    let cc_party_one_second_message: Party1SecondMessage<GE> = requests::postb(
        client_shim,
        &format!("{}/{}/chaincode/second", KG_PATH_PRE, id),
        body,
    )
    .unwrap();

    let cc_party_two_second_message = chain_code::party2::ChainCode2::chain_code_second_message(
        &cc_party_one_first_message,
        &cc_party_one_second_message,
    );

    assert!(cc_party_two_second_message.is_ok());

    let party2_cc = chain_code::party2::ChainCode2::compute_chain_code(
        &cc_ec_key_pair2,
        &cc_party_one_second_message.comm_witness.public_share,
    )
    .chain_code;

    let master_key = MasterKey2::set_master_key(
        &party2_cc,
        &kg_ec_key_pair_party2,
        &kg_party_one_second_message
            .ecdh_second_message
            .comm_witness
            .public_share,
        &party_two_paillier,
    );

    println!("(id: {}) Took: {}", id, TimeFormat(start.elapsed()));

    PrivateShare { id, master_key }
}

#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn get_client_master_key(
    c_endpoint: *const c_char,
    c_auth_token: *const c_char,
    c_user_id: *const c_char,
) -> *mut c_char {
    let raw_endpoint = unsafe { CStr::from_ptr(c_endpoint) };
    let endpoint = match raw_endpoint.to_str() {
        Ok(s) => s,
        Err(_) => panic!("Error while decoding raw endpoint"),
    };

    let raw_auth_token = unsafe { CStr::from_ptr(c_auth_token) };
    let auth_token = match raw_auth_token.to_str() {
        Ok(s) => s,
        Err(_) => panic!("Error while decoding auth token"),
    };

    let user_id_json = unsafe { CStr::from_ptr(c_user_id) };
    let user_id = match user_id_json.to_str() {
        Ok(s) => s,
        Err(_) => panic!("Error while decoding raw user id"),
    };

    let client_shim = ClientShim::new(
        endpoint.to_owned(),
        Some(auth_token.to_owned()),
        user_id.to_owned(),
    );

    let private_share: PrivateShare = get_master_key(&client_shim);

    let private_share_json = match serde_json::to_string(&private_share) {
        Ok(share) => share,
        Err(_) => panic!("Error while performing keygen to endpoint {}", endpoint),
    };

    CString::new(private_share_json).unwrap().into_raw()
}
