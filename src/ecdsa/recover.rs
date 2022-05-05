use anyhow::Result;
use centipede::juggling::proof_system::{Helgamalsegmented, Proof};
use centipede::juggling::segmentation::Msegmentation;
use curv::arithmetic::{Converter, Modulo};
use curv::elliptic::curves::secp256_k1::{FE, GE};
use curv::elliptic::curves::traits::{ECPoint, ECScalar};
use curv::BigInt;
use kms::ecdsa::two_party::{MasterKey1, MasterKey2};
use serde_json;
// iOS bindings
use std::ffi::CString;
use std::os::raw::c_char;

use crate::dto::ecdsa::PrivateShare;
use crate::escrow::{self, Escrow};
use crate::utilities::err_handling::{error_to_c_string, ErrorFFIKind};
use crate::utilities::ffi::ffi_utils::{get_private_share_from_raw, get_str_from_c_char};

pub fn backup_client_mk(private_share: &PrivateShare) -> Result<String, ErrorFFIKind> {
    let escrow = Escrow::new();

    let g: GE = ECPoint::generator();
    let y = escrow.get_public_key();
    let (segments, encryptions) = private_share.master_key.private.to_encrypted_segment(
        escrow::SEGMENT_SIZE,
        escrow::NUM_SEGMENTS,
        &y,
        &g,
    );

    let proof = Proof::prove(&segments, &encryptions, &g, &y, &escrow::SEGMENT_SIZE);

    match serde_json::to_string(&(
        encryptions,
        proof,
        private_share.master_key.public.clone(),
        private_share.master_key.chain_code.clone(),
        &private_share.id,
    )) {
        Ok(s) => Ok(s),
        Err(e) => Err(ErrorFFIKind::E102 {
            msg: "client_backup".to_owned(),
            e: e.to_string(),
        }),
    }
}

#[no_mangle]
pub extern "C" fn backup(c_private_share_json: *const c_char) -> *mut c_char {
    let private_share: PrivateShare = match get_private_share_from_raw(c_private_share_json) {
        Ok(s) => s,
        Err(e) => return error_to_c_string(e),
    };

    let client_backup_json = match backup_client_mk(&private_share) {
        Ok(s) => s,
        Err(e) => return error_to_c_string(e),
    };

    match CString::new(client_backup_json) {
        Ok(s) => s.into_raw(),
        Err(e) => error_to_c_string(ErrorFFIKind::E101 {
            msg: "client_backup".to_owned(),
            e: e.to_string(),
        }),
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "C" fn decrypt_party_one_master_key(
    c_master_key_two_json: *const c_char,
    c_helgamal_segmented_json: *const c_char,
    c_private_key: *const c_char,
) -> *mut c_char {
    let segment_size = 8; // This is hardcoded on both client and server side

    let G: GE = GE::generator();
    let master_key_two_json =
        match get_str_from_c_char(c_master_key_two_json, "master_key_two_json") {
            Ok(s) => s,
            Err(e) => return error_to_c_string(e),
        };

    let party_two_master_key: MasterKey2 = match serde_json::from_str(&master_key_two_json) {
        Ok(s) => s,
        Err(e) => {
            return error_to_c_string(ErrorFFIKind::E104 {
                msg: "party_two_master_key".to_owned(),
                e: e.to_string(),
            })
        }
    };
    let helgamal_segmented_json =
        match get_str_from_c_char(c_helgamal_segmented_json, "helgamal_segmented_json") {
            Ok(s) => s,
            Err(e) => return error_to_c_string(e),
        };

    let encryptions_secret_party1: Helgamalsegmented =
        match serde_json::from_str(&helgamal_segmented_json) {
            Ok(s) => s,
            Err(e) => {
                return error_to_c_string(ErrorFFIKind::E104 {
                    msg: "encryptions_secret_party1".to_owned(),
                    e: e.to_string(),
                })
            }
        };
    let private_key = match get_str_from_c_char(c_private_key, "private_key") {
        Ok(s) => s,
        Err(e) => return error_to_c_string(e),
    };

    let y_b: BigInt = match serde_json::from_str(&private_key) {
        Ok(s) => s,
        Err(e) => {
            return error_to_c_string(ErrorFFIKind::E104 {
                msg: "encryptions_secret_party1".to_owned(),
                e: e.to_string(),
            })
        }
    };

    let y: FE = ECScalar::from(&y_b);

    let r = match Msegmentation::decrypt(&encryptions_secret_party1, &G, &y, &segment_size) {
        Ok(s) => s,
        Err(_e) => {
            return error_to_c_string(ErrorFFIKind::E103 {
                msg: "secp256k1_scalar".to_owned(),
                e: "Secp256k1 Error Decrypting".to_owned(),
            })
        }
    };
    let party_one_master_key_recovered =
        party_two_master_key.counter_master_key_from_recovered_secret(r);

    let s = match serde_json::to_string(&party_one_master_key_recovered) {
        Ok(s) => s,
        Err(e) => {
            return error_to_c_string(ErrorFFIKind::E102 {
                msg: "party_one_master_key_recovered".to_owned(),
                e: e.to_string(),
            })
        }
    };

    match CString::new(s) {
        Ok(s) => s.into_raw(),
        Err(e) => error_to_c_string(ErrorFFIKind::E101 {
            msg: "client_backup".to_owned(),
            e: e.to_string(),
        }),
    }
}

#[no_mangle]
pub extern "C" fn get_child_mk1(
    c_master_key_one_json: *const c_char,
    c_x_pos: i32,
    c_y_pos: i32,
) -> *mut c_char {
    let master_key_one_json =
        match get_str_from_c_char(c_master_key_one_json, "master_key_one_json") {
            Ok(s) => s,
            Err(e) => return error_to_c_string(e),
        };

    let party_one_master_key: MasterKey1 = match serde_json::from_str(&master_key_one_json) {
        Ok(s) => s,
        Err(e) => {
            return error_to_c_string(ErrorFFIKind::E104 {
                msg: "party_one_master_key".to_owned(),
                e: e.to_string(),
            })
        }
    };

    let x: BigInt = BigInt::from(c_x_pos);
    let y: BigInt = BigInt::from(c_y_pos);

    let derived_mk1 = party_one_master_key.get_child(vec![x, y]);

    let derived_mk1_json = match serde_json::to_string(&derived_mk1) {
        Ok(share) => share,
        Err(e) => {
            return error_to_c_string(ErrorFFIKind::E102 {
                msg: "derived_mk1_json".to_owned(),
                e: e.to_string(),
            })
        }
    };

    CString::new(derived_mk1_json).unwrap().into_raw()
}

#[no_mangle]
pub extern "C" fn get_child_mk2(
    c_master_key_two_json: *const c_char,
    c_x_pos: i32,
    c_y_pos: i32,
) -> *mut c_char {
    let master_key_two_json =
        match get_str_from_c_char(c_master_key_two_json, "master_key_two_json") {
            Ok(s) => s,
            Err(e) => return error_to_c_string(e),
        };

    let party_two_master_key: MasterKey2 = match serde_json::from_str(&master_key_two_json) {
        Ok(s) => s,
        Err(e) => {
            return error_to_c_string(ErrorFFIKind::E104 {
                msg: "party_two_master_key".to_owned(),
                e: e.to_string(),
            })
        }
    };

    let x: BigInt = BigInt::from(c_x_pos);

    let y: BigInt = BigInt::from(c_y_pos);

    let derived_mk2 = party_two_master_key.get_child(vec![x, y]);

    let derived_mk2_json = match serde_json::to_string(&derived_mk2) {
        Ok(share) => share,
        Err(e) => {
            return error_to_c_string(ErrorFFIKind::E102 {
                msg: "derived_mk2_json".to_owned(),
                e: e.to_string(),
            })
        }
    };

    match CString::new(derived_mk2_json) {
        Ok(s) => s.into_raw(),
        Err(e) => error_to_c_string(ErrorFFIKind::E101 {
            msg: "derived_mk2_json".to_owned(),
            e: e.to_string(),
        }),
    }
}

#[no_mangle]
pub extern "C" fn construct_single_private_key(
    c_mk1_x1: *const c_char,
    c_mk2_x2: *const c_char,
) -> *mut c_char {
    let mk1_x1_str = match get_str_from_c_char(c_mk1_x1, "mk1_x1") {
        Ok(s) => s,
        Err(e) => return error_to_c_string(e),
    };
    let mk1_x1: BigInt = BigInt::from_hex(&mk1_x1_str).unwrap();

    let mk2_x2_str = match get_str_from_c_char(c_mk2_x2, "mk2_x2") {
        Ok(s) => s,
        Err(e) => return error_to_c_string(e),
    };
    let mk2_x2: BigInt = BigInt::from_hex(&mk2_x2_str).unwrap();

    let sk = BigInt::mod_mul(&mk1_x1, &mk2_x2, &FE::q());

    let sk_json = match serde_json::to_string(&sk) {
        Ok(share) => share,
        Err(_) => panic!("Error while construct_single_private_key"),
    };

    CString::new(sk_json).unwrap().into_raw()
}
