use std::collections::HashMap;

use crate::dto::ecdsa::MKPosDto;
use crate::dto::ecdsa::PrivateShare;
use crate::eth::transaction::Transaction;

use crate::eth::utils::get_contract;
use crate::eth::utils::handle_eth_address_conversion;
use crate::eth::utils::pubkey_to_eth_address;
use crate::utilities::err_handling::{error_to_c_string, ErrorFFIKind};
use crate::utilities::ffi::ffi_utils::get_str_from_c_char;
use crate::utilities::ffi::ffi_utils::{
    get_addresses_derivation_map_from_raw, get_client_shim_from_raw, get_private_share_from_raw,
};
use crate::utilities::requests::ClientShim;
use anyhow::Result;
use web3::ethabi::Token;
use web3::types::TransactionParameters;

use std::ffi::CString;
use std::os::raw::c_char;
use web3::types::H256;

use super::utils::eth_to_wei;
use super::utils::get_pos_mk_dto;
use super::utils::get_tx_params;
use super::utils::sign_send_raw_tx;

#[allow(clippy::too_many_arguments)]
pub fn send_erc20(
    from: &str,
    to: &str,
    token_name: &str,
    network: &str,
    token_amount: usize,
    client_shim: &ClientShim,
    private_share: &PrivateShare,
    addresses_derivation_map: &HashMap<String, MKPosDto>,
) -> Result<H256> {
    let erc20_resp = get_contract(token_name, network, client_shim)?;
    let contract_abi = erc20_resp.contract;
    let from_address = handle_eth_address_conversion(from)?;
    let to_address = handle_eth_address_conversion(to)?;
    let func = contract_abi.function("transferFrom")?;
    let data = func.encode_input(&[
        Token::Address(from_address),
        Token::Address(to_address),
        Token::Uint(token_amount.into()),
    ])?;
    println!("send_erc20 data: {:?}", data);
    let tx_hash = sign_and_send(
        from,
        to,
        0.0,
        data,
        client_shim,
        private_share,
        addresses_derivation_map,
    )?;
    println!("send_erc20 tx_hash: {}", format!("{:?}", tx_hash));
    Ok(tx_hash)
}

pub fn sign_and_send(
    from: &str,
    to: &str,
    eth_value: f64,
    data: Vec<u8>,
    client_shim: &ClientShim,
    private_share: &PrivateShare,
    addresses_derivation_map: &HashMap<String, MKPosDto>,
) -> Result<H256> {
    let pos_mk = get_pos_mk_dto(from, addresses_derivation_map)?;
    let mk = &pos_mk.mk;

    let from_address = pubkey_to_eth_address(mk);
    let to_address = handle_eth_address_conversion(to)?;

    let tx_params = get_tx_params(from_address, to_address, eth_value, client_shim)?;

    let tx = Transaction {
        to: Some(to_address),
        nonce: tx_params.nonce,
        gas: tx_params.gas,
        gas_price: tx_params.gas_price,
        value: eth_to_wei(eth_value),
        data,
        transaction_type: tx_params.transaction_type,
        access_list: TransactionParameters::default()
            .access_list
            .unwrap_or_default(),
        max_priority_fee_per_gas: tx_params.max_priority_fee_per_gas,
    };
    let chain_id = tx_params.chain_id;
    let transaction_result = sign_send_raw_tx(chain_id, tx, pos_mk, private_share, client_shim)?;
    Ok(transaction_result.tx_hash)
}

#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn send_eth_tx(
    c_endpoint: *const c_char,
    c_auth_token: *const c_char,
    c_user_id: *const c_char,
    c_from_address: *const c_char,
    c_to_address: *const c_char,
    c_amount_eth: f64,
    c_private_share_json: *const c_char,
    c_addresses_derivation_map: *const c_char,
) -> *mut c_char {
    let from_address = match get_str_from_c_char(c_from_address, "from_address") {
        Ok(s) => s,
        Err(e) => return error_to_c_string(e),
    };

    let to_address = match get_str_from_c_char(c_to_address, "to_address") {
        Ok(s) => s,
        Err(e) => return error_to_c_string(e),
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
        Err(e) => return error_to_c_string(e),
    };

    let addresses_derivation_map =
        match get_addresses_derivation_map_from_raw(c_addresses_derivation_map) {
            Ok(s) => s,
            Err(e) => return error_to_c_string(e),
        };

    let tx_hash = match sign_and_send(
        &from_address,
        &to_address,
        c_amount_eth,
        Vec::new(),
        &client_shim,
        &private_share,
        &addresses_derivation_map,
    ) {
        Ok(s) => s,
        Err(e) => {
            return error_to_c_string(ErrorFFIKind::E103 {
                msg: "tx_hash".to_owned(),
                e: e.to_string(),
            })
        }
    };

    let tx_hash_json = match serde_json::to_string(&tx_hash) {
        Ok(tx_resp) => tx_resp,
        Err(e) => {
            return error_to_c_string(ErrorFFIKind::E102 {
                msg: "tx_hash".to_owned(),
                e: e.to_string(),
            })
        }
    };

    match CString::new(tx_hash_json) {
        Ok(s) => s.into_raw(),
        Err(e) => error_to_c_string(ErrorFFIKind::E101 {
            msg: "tx_hash".to_owned(),
            e: e.to_string(),
        }),
    }
}
