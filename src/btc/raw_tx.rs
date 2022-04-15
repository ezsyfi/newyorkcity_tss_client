use super::utils::{get_all_addresses_balance, list_unspent_for_addresss, BTC_TESTNET};
use crate::btc::utils::{get_bitcoin_network, get_new_address, to_bitcoin_public_key};
use crate::ecdsa::{sign, PrivateShare};
use crate::utilities::dto::{MKPosAddressDto, MKPosDto, UtxoAggregator};
use crate::utilities::err_handling::{error_to_c_string, ErrorFFIKind};
use crate::utilities::ffi::ffi_utils::{
    get_addresses_derivation_map_from_raw, get_client_shim_from_raw, get_private_share_from_raw,
};
use crate::utilities::hd_wallet::derive_new_key;
use crate::utilities::requests::ClientShim;

use anyhow::{anyhow, Result};
use bitcoin::util::bip143::SigHashCache;
use curv::arithmetic::traits::Converter; // Need for signing
use curv::elliptic::curves::traits::ECPoint;
use curv::BigInt;
use itertools::Itertools;

use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

use bitcoin::consensus::encode::serialize;
use bitcoin::hashes::{hex::FromHex, sha256d};
use bitcoin::secp256k1::Signature;
use bitcoin::{self, SigHashType};
use bitcoin::{TxIn, TxOut, Txid};
use serde_json;

use hex;
use std::str::FromStr;

#[derive(Serialize, Deserialize)]
pub struct BtcRawTxFFIResp {
    pub raw_tx_hex: String,
    pub change_address_payload: MKPosAddressDto,
}

pub fn create_raw_tx(
    to_address: &str,
    amount_btc: f64,
    client_shim: &ClientShim,
    last_derived_pos: u32,
    private_share: &PrivateShare,
    addresses_derivation_map: &HashMap<String, MKPosDto>,
) -> Result<Option<BtcRawTxFFIResp>> {
    let selected = select_tx_in(amount_btc, last_derived_pos, private_share)?;

    if selected.is_empty() {
        return Err(anyhow!("Not enough fund"));
    }

    /* Specify "vin" array aka Transaction Inputs */
    let txs_in: Vec<TxIn> = selected
        .clone()
        .into_iter()
        .map(|s| bitcoin::TxIn {
            previous_output: bitcoin::OutPoint {
                txid: Txid::from_hash(sha256d::Hash::from_hex(&s.tx_hash).unwrap()),
                vout: s.tx_pos as u32,
            },
            script_sig: bitcoin::Script::default(),
            sequence: 0xFFFFFFFF,
            witness: Vec::default(),
        })
        .collect();

    /* Specify "vout" array aka Transaction Outputs */
    let relay_fees = 10_000; // Relay fees for miner
    let amount_satoshi = (amount_btc * 100_000_000.0) as u64;

    let (change_pos, change_mk) = derive_new_key(private_share, last_derived_pos);

    let change_address = match get_new_address(private_share, last_derived_pos) {
        Ok(s) => s,
        Err(e) => {
            return Err(anyhow!("Error while get new btc address: {}", e));
        }
    };

    let change_address_payload = MKPosAddressDto {
        address: change_address.to_string(),
        pos: change_pos,
        mk: change_mk,
    };

    let total_selected = selected
        .clone()
        .into_iter()
        .fold(0, |sum, val| sum + val.value) as u64;

    println!(
        "amount_satoshi: {} - total_selected: {}  ",
        amount_satoshi, total_selected
    );

    let to_btc_adress = bitcoin::Address::from_str(to_address)?;
    let txs_out = vec![
        TxOut {
            value: amount_satoshi,
            script_pubkey: to_btc_adress.script_pubkey(),
        },
        TxOut {
            value: total_selected - amount_satoshi - relay_fees,
            script_pubkey: change_address.script_pubkey(),
        },
    ];

    let mut transaction = bitcoin::Transaction {
        version: 2,
        lock_time: 0,
        input: txs_in,
        output: txs_out,
    };

    let mut signed_transaction = transaction.clone();

    /* Signing transaction */
    for (i, txi) in selected.iter().enumerate().take(transaction.input.len()) {
        let address_derivation = match addresses_derivation_map.get(&txi.address) {
            Some(s) => s,
            None => {
                return Err(anyhow!(
                    "Error while get address from addresses_derivation_map"
                ));
            }
        };

        let mk = &address_derivation.mk;
        let pk = mk.public.q.get_element();

        let mut sig_hasher = SigHashCache::new(&mut transaction);
        let sig_hash = sig_hasher.signature_hash(
            i,
            &bitcoin::Address::p2pkh(
                &to_bitcoin_public_key(pk),
                get_bitcoin_network(BTC_TESTNET)?,
            )
            .script_pubkey(),
            (txi.value as u32).into(),
            SigHashType::All,
        );

        let signature = sign(
            client_shim,
            BigInt::from_hex(&hex::encode(&sig_hash[..])).unwrap(),
            mk,
            BigInt::from(0),
            BigInt::from(address_derivation.pos),
            &private_share.id,
        )?;

        let mut v = BigInt::to_bytes(&signature.r);
        v.extend(BigInt::to_bytes(&signature.s));

        // Serialize the (R,S) value of ECDSA Signature
        let mut sig_vec = Signature::from_compact(&v[..])?.serialize_der().to_vec();
        sig_vec.push(1);

        let pk_vec = pk.serialize().to_vec();

        signed_transaction.input[i].witness = vec![sig_vec, pk_vec];
    }
    Ok(Some(BtcRawTxFFIResp {
        raw_tx_hex: hex::encode(serialize(&signed_transaction)),
        change_address_payload,
    }))
}

// TODO: handle fees
// Select all txin enough to pay the amount
fn select_tx_in(
    amount_btc: f64,
    last_derived_pos: u32,
    private_share: &PrivateShare,
) -> Result<Vec<UtxoAggregator>> {
    // greedy selection
    let list_unspent: Vec<UtxoAggregator> =
        get_all_addresses_balance(last_derived_pos, private_share)?
            .into_iter()
            // .filter(|b| b.confirmed > 0)
            .filter_map(|a| list_unspent_for_addresss(a.address).ok())
            .flatten()
            .sorted_by(|a, b| a.value.partial_cmp(&b.value).unwrap())
            .into_iter()
            .collect();

    // println!("list_unspent {:#?}", list_unspent);

    let mut remaining: i64 = (amount_btc * 100_000_000.0) as i64;
    let mut selected: Vec<UtxoAggregator> = Vec::new();
    for unspent in list_unspent {
        selected.push(unspent.clone());
        remaining -= unspent.value as i64;
        if remaining < 0 {
            break;
        }
    }
    Ok(selected)
}

#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn get_raw_btc_tx(
    c_endpoint: *const c_char,
    c_auth_token: *const c_char,
    c_user_id: *const c_char,
    c_to_address: *const c_char,
    c_amount_btc: f64,
    c_last_derived_pos: u32,
    c_private_share_json: *const c_char,
    c_addresses_derivation_map: *const c_char,
) -> *mut c_char {
    let raw_to_address = unsafe { CStr::from_ptr(c_to_address) };
    let to_address = match raw_to_address.to_str() {
        Ok(s) => s,
        Err(e) => {
            return error_to_c_string(ErrorFFIKind::E100 {
                msg: "to_address".to_owned(),
                e: e.to_string(),
            })
        }
    };

    let client_shim = match get_client_shim_from_raw(c_endpoint, c_auth_token, c_user_id) {
        Ok(s) => s,
        Err(e) => {
            return error_to_c_string(ErrorFFIKind::E100 {
                msg: "client_shim".to_owned(),
                e: e.to_string(),
            })
        }
    };

    let private_share = match get_private_share_from_raw(c_private_share_json) {
        Ok(s) => s,
        Err(e) => {
            return error_to_c_string(ErrorFFIKind::E104 {
                msg: "private_share".to_owned(),
                e: e.to_string(),
            })
        }
    };

    let addresses_derivation_map =
        match get_addresses_derivation_map_from_raw(c_addresses_derivation_map) {
            Ok(s) => s,
            Err(e) => {
                return error_to_c_string(ErrorFFIKind::E104 {
                    msg: "addresses_derivation_map".to_owned(),
                    e: e.to_string(),
                })
            }
        };

    let raw_tx_opt = match create_raw_tx(
        to_address,
        c_amount_btc,
        &client_shim,
        c_last_derived_pos,
        &private_share,
        &addresses_derivation_map,
    ) {
        Ok(s) => s,
        Err(e) => {
            return error_to_c_string(ErrorFFIKind::E103 {
                msg: "raw_tx".to_owned(),
                e: e.to_string(),
            })
        }
    };

    let raw_tx = match raw_tx_opt {
        Some(tx) => tx,
        None => return std::ptr::null_mut(),
    };

    let raw_tx_json = match serde_json::to_string(&raw_tx) {
        Ok(tx_resp) => tx_resp,
        Err(e) => {
            return error_to_c_string(ErrorFFIKind::E102 {
                msg: "raw_tx".to_owned(),
                e: e.to_string(),
            })
        }
    };

    match CString::new(raw_tx_json) {
        Ok(s) => s.into_raw(),
        Err(e) => error_to_c_string(ErrorFFIKind::E101 {
            msg: "raw_tx".to_owned(),
            e: e.to_string(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use crate::{btc::utils::get_test_private_share, ecdsa::PrivateShare};

    #[test]
    fn test_select_tx_in() -> Result<()> {
        let private_share: PrivateShare = get_test_private_share();
        let unspent_list = super::select_tx_in(0.0, 0, &private_share)?;
        assert!(unspent_list.is_empty());
        Ok(())
    }
}
