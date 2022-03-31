use crate::btc::utils::{get_bitcoin_network, get_new_bitcoin_address, to_bitcoin_public_key};
// // iOS bindings
use crate::ecdsa::{sign, PrivateShare};

use crate::utilities::dto::{
    BalanceAggregator, BlockCypherAddress, MKPosAddressDto, MKPosDto, UtxoAggregator,
};
use crate::utilities::err_handling::{error_to_c_string, ErrorFFIKind};
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

use super::utils::{to_bitcoin_address, BTC_TESTNET};

pub const BLOCK_CYPHER_HOST: &str = "https://api.blockcypher.com/v1/btc/test3"; // TODO: Centralize the config constants

#[derive(Serialize, Deserialize)]
pub struct BtcRawTxFFIResp {
    pub raw_tx_hex: String,
    pub change_address_payload: MKPosAddressDto,
}

pub fn create_raw_tx(
    to_address: String,
    amount_btc: f32,
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
    let amount_satoshi = (amount_btc * 100_000_000 as f32) as u64;

    let (change_pos, change_mk) = derive_new_key(private_share, last_derived_pos);

    let change_address = match get_new_bitcoin_address(private_share, last_derived_pos) {
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
    println!("{} - back", total_selected - amount_satoshi);

    let to_btc_adress = bitcoin::Address::from_str(&to_address)?;
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
    for i in 0..transaction.input.len() {
        let address_derivation = match addresses_derivation_map.get(&selected[i].address) {
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
            (selected[i].value as u32).into(),
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
    // (hex::encode(serialize(&signed_transaction)), Some(change_addr_resp))
    Ok(Some(BtcRawTxFFIResp {
        raw_tx_hex: hex::encode(serialize(&signed_transaction)),
        change_address_payload,
    }))
}

// TODO: handle fees
// Select all txin enough to pay the amount
fn select_tx_in(
    amount_btc: f32,
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

fn get_all_addresses_balance(
    last_derived_pos: u32,
    private_share: &PrivateShare,
) -> Result<Vec<BalanceAggregator>> {
    let response: Result<Vec<BalanceAggregator>> =
        get_all_addresses(last_derived_pos, private_share)?
            .into_iter()
            .map(|a| get_address_balance(&a))
            .collect();

    // println!("get_all_addresses_balance {:#?}", response);
    response
}

fn get_all_addresses(
    last_derived_pos: u32,
    private_share: &PrivateShare,
) -> Result<Vec<bitcoin::Address>> {
    let init = 0;
    let last_pos = last_derived_pos;

    let mut response: Vec<bitcoin::Address> = Vec::new();

    for n in init..=last_pos {
        let mk = private_share
            .master_key
            .get_child(vec![BigInt::from(0), BigInt::from(n)]);

        let bitcoin_address = to_bitcoin_address(BTC_TESTNET, &mk)?;

        response.push(bitcoin_address);
    }

    Ok(response)
}

fn get_address_balance(address: &bitcoin::Address) -> Result<BalanceAggregator> {
    let balance_url = BLOCK_CYPHER_HOST.to_owned() + "/addrs/" + &address.to_string() + "/balance";
    let res = reqwest::blocking::get(balance_url)?.text()?;
    let address_balance: BlockCypherAddress = serde_json::from_str(res.as_str())?;

    Ok(BalanceAggregator {
        confirmed: address_balance.balance,
        unconfirmed: address_balance.unconfirmed_balance,
        address: address.to_string(),
    })
}

fn list_unspent_for_addresss(address: String) -> Result<Vec<UtxoAggregator>> {
    let unspent_tx_url = BLOCK_CYPHER_HOST.to_owned() + "/addrs/" + &address + "?unspentOnly=true";
    let res = reqwest::blocking::get(unspent_tx_url)?.text()?;
    let address_balance_with_tx_refs: BlockCypherAddress = serde_json::from_str(res.as_str())?;
    if let Some(tx_refs) = address_balance_with_tx_refs.txrefs {
        Ok(tx_refs
            .iter()
            .map(|u| UtxoAggregator {
                value: u.value,
                height: u.block_height,
                tx_hash: u.tx_hash.clone(),
                tx_pos: u.tx_output_n,
                address: address.clone(),
            })
            .collect())
    } else {
        Ok(Vec::new())
    }
}

#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn get_raw_btc_tx(
    c_endpoint: *const c_char,
    c_auth_token: *const c_char,
    c_user_id: *const c_char,
    c_to_address: *const c_char,
    c_amount_btc: f32,
    c_last_derived_pos: u32,
    c_private_share_json: *const c_char,
    c_addresses_derivation_map: *const c_char,
) -> *mut c_char {
    let raw_endpoint_json = unsafe { CStr::from_ptr(c_endpoint) };
    let endpoint = match raw_endpoint_json.to_str() {
        Ok(s) => s,
        Err(e) => {
            return error_to_c_string(ErrorFFIKind::E100 {
                msg: "endpoint".to_owned(),
                e: e.to_string(),
            })
        }
    };

    let raw_auth_token_json = unsafe { CStr::from_ptr(c_auth_token) };
    let auth_token = match raw_auth_token_json.to_str() {
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

    let raw_private_share_json = unsafe { CStr::from_ptr(c_private_share_json) };
    let private_share_json = match raw_private_share_json.to_str() {
        Ok(s) => s,
        Err(e) => {
            return error_to_c_string(ErrorFFIKind::E100 {
                msg: "private_share".to_owned(),
                e: e.to_string(),
            })
        }
    };
    let private_share: PrivateShare = match serde_json::from_str(private_share_json) {
        Ok(s) => s,
        Err(e) => {
            return error_to_c_string(ErrorFFIKind::E104 {
                msg: "private_share".to_owned(),
                e: e.to_string(),
            })
        }
    };

    let raw_addresses_derivation_map_json = unsafe { CStr::from_ptr(c_addresses_derivation_map) };
    let addresses_derivation_map_json = match raw_addresses_derivation_map_json.to_str() {
        Ok(s) => s,
        Err(e) => {
            return error_to_c_string(ErrorFFIKind::E100 {
                msg: "addresses_derivation_map".to_owned(),
                e: e.to_string(),
            })
        }
    };
    let addresses_derivation_map: HashMap<String, MKPosDto> =
        match serde_json::from_str(addresses_derivation_map_json) {
            Ok(s) => s,
            Err(e) => {
                return error_to_c_string(ErrorFFIKind::E104 {
                    msg: "addresses_derivation_map".to_owned(),
                    e: e.to_string(),
                })
            }
        };

    let client_shim = ClientShim::new(
        endpoint.to_owned(),
        Some(auth_token.to_owned()),
        user_id.to_owned(),
    );

    let raw_tx_opt = match create_raw_tx(
        to_address.to_owned(),
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
    fn test_get_all_addresses() -> Result<()> {
        let private_share: PrivateShare = get_test_private_share();
        let address_list = super::get_all_addresses(0, &private_share)?;
        assert!(!address_list.is_empty());
        Ok(())
    }

    #[test]
    fn test_get_all_addresses_balance() -> Result<()> {
        let private_share: PrivateShare = get_test_private_share();
        let address_balance_list = super::get_all_addresses_balance(0, &private_share)?;
        assert!(!address_balance_list.is_empty());

        let address_balance = address_balance_list.get(0).unwrap();
        assert_eq!(address_balance.confirmed, 0);
        assert_eq!(address_balance.unconfirmed, 0);
        assert_eq!(
            address_balance.address,
            "tb1qkr66k03t0d0ep8kmkl0zg8du45y2mfer0pflh5"
        );
        Ok(())
    }

    #[test]
    fn test_select_tx_in() -> Result<()> {
        let private_share: PrivateShare = get_test_private_share();
        let unspent_list = super::select_tx_in(0.0, 0, &private_share)?;
        assert!(unspent_list.is_empty());
        Ok(())
    }
}
