use std::collections::HashMap;
use std::str::FromStr;

use crate::dto::ecdsa::MKPosDto;
use crate::dto::ecdsa::PrivateShare;
use crate::eth::transaction::Transaction;
use crate::eth::utils::pubkey_to_eth_address;
use crate::utilities::err_handling::{error_to_c_string, ErrorFFIKind};
use crate::utilities::ffi::ffi_utils::get_str_from_c_char;
use crate::utilities::ffi::ffi_utils::{
    get_addresses_derivation_map_from_raw, get_client_shim_from_raw, get_private_share_from_raw,
};
use crate::utilities::requests::ClientShim;
use anyhow::Result;

use std::ffi::CString;
use std::os::raw::c_char;
use web3::types::{Address, H256};

use super::utils::get_pos_mk_dto;
use super::utils::get_tx_params;
use super::utils::sign_send_raw_tx;
const ERC20_ADDRESSES_JSON: &str = r#"{
    "usdt": {
      "address": "0x3B00Ef435fA4FcFF5C209a37d1f3dcff37c705aD",
      "network": "rinkeby",
      "abi": [
        {
          "inputs": [
            { "internalType": "string", "name": "name", "type": "string" },
            { "internalType": "string", "name": "symbol", "type": "string" },
            { "internalType": "uint8", "name": "decimals", "type": "uint8" }
          ],
          "payable": false,
          "stateMutability": "nonpayable",
          "type": "constructor"
        },
        {
          "anonymous": false,
          "inputs": [
            {
              "indexed": true,
              "internalType": "address",
              "name": "owner",
              "type": "address"
            },
            {
              "indexed": true,
              "internalType": "address",
              "name": "spender",
              "type": "address"
            },
            {
              "indexed": false,
              "internalType": "uint256",
              "name": "value",
              "type": "uint256"
            }
          ],
          "name": "Approval",
          "type": "event"
        },
        {
          "anonymous": false,
          "inputs": [
            {
              "indexed": true,
              "internalType": "address",
              "name": "from",
              "type": "address"
            },
            {
              "indexed": true,
              "internalType": "address",
              "name": "to",
              "type": "address"
            },
            {
              "indexed": false,
              "internalType": "uint256",
              "name": "value",
              "type": "uint256"
            }
          ],
          "name": "Transfer",
          "type": "event"
        },
        {
          "constant": true,
          "inputs": [
            { "internalType": "address", "name": "owner", "type": "address" },
            { "internalType": "address", "name": "spender", "type": "address" }
          ],
          "name": "allowance",
          "outputs": [
            { "internalType": "uint256", "name": "", "type": "uint256" }
          ],
          "payable": false,
          "stateMutability": "view",
          "type": "function"
        },
        {
          "constant": false,
          "inputs": [
            { "internalType": "address", "name": "spender", "type": "address" },
            { "internalType": "uint256", "name": "amount", "type": "uint256" }
          ],
          "name": "approve",
          "outputs": [{ "internalType": "bool", "name": "", "type": "bool" }],
          "payable": false,
          "stateMutability": "nonpayable",
          "type": "function"
        },
        {
          "constant": true,
          "inputs": [
            { "internalType": "address", "name": "account", "type": "address" }
          ],
          "name": "balanceOf",
          "outputs": [
            { "internalType": "uint256", "name": "", "type": "uint256" }
          ],
          "payable": false,
          "stateMutability": "view",
          "type": "function"
        },
        {
          "constant": true,
          "inputs": [],
          "name": "decimals",
          "outputs": [{ "internalType": "uint8", "name": "", "type": "uint8" }],
          "payable": false,
          "stateMutability": "view",
          "type": "function"
        },
        {
          "constant": false,
          "inputs": [
            { "internalType": "address", "name": "spender", "type": "address" },
            {
              "internalType": "uint256",
              "name": "subtractedValue",
              "type": "uint256"
            }
          ],
          "name": "decreaseAllowance",
          "outputs": [{ "internalType": "bool", "name": "", "type": "bool" }],
          "payable": false,
          "stateMutability": "nonpayable",
          "type": "function"
        },
        {
          "constant": false,
          "inputs": [
            { "internalType": "address", "name": "spender", "type": "address" },
            {
              "internalType": "uint256",
              "name": "addedValue",
              "type": "uint256"
            }
          ],
          "name": "increaseAllowance",
          "outputs": [{ "internalType": "bool", "name": "", "type": "bool" }],
          "payable": false,
          "stateMutability": "nonpayable",
          "type": "function"
        },
        {
          "constant": false,
          "inputs": [
            { "internalType": "address", "name": "_to", "type": "address" },
            { "internalType": "uint256", "name": "_amount", "type": "uint256" }
          ],
          "name": "mint",
          "outputs": [{ "internalType": "bool", "name": "", "type": "bool" }],
          "payable": false,
          "stateMutability": "nonpayable",
          "type": "function"
        },
        {
          "constant": true,
          "inputs": [],
          "name": "name",
          "outputs": [
            { "internalType": "string", "name": "", "type": "string" }
          ],
          "payable": false,
          "stateMutability": "view",
          "type": "function"
        },
        {
          "constant": true,
          "inputs": [],
          "name": "symbol",
          "outputs": [
            { "internalType": "string", "name": "", "type": "string" }
          ],
          "payable": false,
          "stateMutability": "view",
          "type": "function"
        },
        {
          "constant": true,
          "inputs": [],
          "name": "totalSupply",
          "outputs": [
            { "internalType": "uint256", "name": "", "type": "uint256" }
          ],
          "payable": false,
          "stateMutability": "view",
          "type": "function"
        },
        {
          "constant": false,
          "inputs": [
            {
              "internalType": "address",
              "name": "recipient",
              "type": "address"
            },
            { "internalType": "uint256", "name": "amount", "type": "uint256" }
          ],
          "name": "transfer",
          "outputs": [{ "internalType": "bool", "name": "", "type": "bool" }],
          "payable": false,
          "stateMutability": "nonpayable",
          "type": "function"
        },
        {
          "constant": false,
          "inputs": [
            { "internalType": "address", "name": "sender", "type": "address" },
            {
              "internalType": "address",
              "name": "recipient",
              "type": "address"
            },
            { "internalType": "uint256", "name": "amount", "type": "uint256" }
          ],
          "name": "transferFrom",
          "outputs": [{ "internalType": "bool", "name": "", "type": "bool" }],
          "payable": false,
          "stateMutability": "nonpayable",
          "type": "function"
        }
      ]
    }
  }"#;

// pub fn send_erc20(
//     from: &str,
//     to: &str,
//     token_name: &str,
//     network: &str,
//     client_shim: &ClientShim,
//     private_share: &PrivateShare,
//     addresses_derivation_map: &HashMap<String, MKPosDto>,
// ) -> Result<H256> {
    
    // let erc20_addresses_map: HashMap<String, ERC20> = serde::from_str(ERC20_ADDRESSES_JSON).unwrap();
    // let erc20_obj = erc20_addresses_map.get(token_name).unwrap();
    // let contract_address = Address::from_str(&erc20_obj.address)?;
    
    // // let contract_abi = serde_json::to_vec(erc)
    // let contract = Contract::from_json(
    //     WebSocket,
    //     contract_address,
    //     erc20_obj.abi.as_bytes(),
    // )?;
// } 

pub fn sign_and_send(
    from: &str,
    to: &str,
    eth_value: f64,
    client_shim: &ClientShim,
    private_share: &PrivateShare,
    addresses_derivation_map: &HashMap<String, MKPosDto>,
) -> Result<H256> {
    let pos_mk = get_pos_mk_dto(from, addresses_derivation_map)?;
    let mk = &pos_mk.mk;

    let from_address = pubkey_to_eth_address(mk);
    let to_address = Address::from_str(to)?;

    let tx_params = get_tx_params(from_address, to_address, eth_value, client_shim)?;

    let tx = Transaction {
        to: tx_params.to,
        nonce: tx_params.nonce,
        gas: tx_params.gas,
        gas_price: tx_params.gas_price,
        value: tx_params.value,
        data: tx_params.data,
        transaction_type: tx_params.transaction_type,
        access_list: tx_params.access_list,
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
