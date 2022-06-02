use anyhow::{anyhow, Result};
use curv::arithmetic::traits::Converter;
use curv::{elliptic::curves::traits::ECPoint, BigInt};

use crate::dto::eth::{Erc20ReqBody, Erc20Resp};
use crate::eth::transaction::Transaction;
use crate::{
    dto::{
        ecdsa::{MKPosDto, PrivateShare},
        eth::{EthSendTxReqBody, EthSendTxResp, EthTxParamsReqBody, EthTxParamsResp},
    },
    ecdsa::sign,
    utilities::requests::{self, ClientShim},
};
use futures::future::try_join_all;
use kms::ecdsa::two_party::MasterKey2;
use std::{collections::HashMap, str::FromStr};
use web3::{
    self,
    signing::{keccak256, Signature},
    transports::{self, WebSocket},
    types::{Address, H160, H256, U256},
    Web3,
};

pub fn sign_send_raw_tx(
    chain_id: u64,
    tx: Transaction,
    pos_mk: &MKPosDto,
    private_share: &PrivateShare,
    client_shim: &ClientShim,
) -> Result<EthSendTxResp> {
    let msg = tx.get_hash(chain_id);

    let sig = sign(
        client_shim,
        BigInt::from_hex(&hex::encode(&msg[..])).unwrap(),
        &pos_mk.mk,
        BigInt::from(0),
        BigInt::from(pos_mk.pos),
        &private_share.id,
    )?;

    let r = H256::from_slice(&BigInt::to_bytes(&sig.r));
    let s = H256::from_slice(&BigInt::to_bytes(&sig.s));
    let v = sig.recid as u64 + 35 + chain_id * 2;
    let signature = Signature { r, s, v };
    let signed = tx.sign(signature, chain_id);

    let tx_send_body = EthSendTxReqBody {
        raw_tx: signed.raw_transaction,
    };

    match requests::postb(client_shim, "eth/tx/send", tx_send_body)? {
        Some(s) => Ok(s),
        None => return Err(anyhow!("send ETH tx request failed")),
    }
}

pub fn get_contract(name: &str, network: &str, client_shim: &ClientShim) -> Result<Erc20Resp> {
    let erc20_body = Erc20ReqBody {
        name: name.to_owned(),
        network: network.to_owned(),
    };

    match requests::postb(client_shim, "eth/contract", erc20_body)? {
        Some(s) => Ok(s),
        None => return Err(anyhow!("get ERC20 contract request failed")),
    }
}

pub fn get_tx_params(
    from_address: H160,
    to_address: H160,
    eth_value: f64,
    client_shim: &ClientShim,
) -> Result<EthTxParamsResp> {
    let tx_params_body = EthTxParamsReqBody {
        from_address,
        to_address,
        eth_value,
    };

    match requests::postb(client_shim, "eth/tx/params", tx_params_body)? {
        Some(s) => Ok(s),
        None => return Err(anyhow!("get ETH tx params request failed")),
    }
}

pub fn get_pos_mk_dto<'a>(
    address: &str,
    addresses_derivation_map: &'a HashMap<String, MKPosDto>,
) -> Result<&'a MKPosDto> {
    match addresses_derivation_map.get(address.to_lowercase().as_str()) {
        Some(pos_mk) => Ok(pos_mk),
        None => {
            return Err(anyhow!(
                "from address not found in addresses_derivation_map"
            ))
        }
    }
}

pub async fn get_all_addresses_balance(
    web3_connection_url: &str,
    last_derived_pos: u32,
    private_share: &PrivateShare,
) -> Result<Vec<f64>> {
    let web3_connection = establish_web3_connection(web3_connection_url).await?;
    let addresses = get_all_addresses(last_derived_pos, private_share).unwrap();
    let result: Vec<f64> = try_join_all(
        addresses
            .iter()
            .map(|a| get_balance_in_eth(format!("{:?}", a), &web3_connection)),
    )
    .await?;
    Ok(result)
}

pub fn get_all_addresses(
    last_derived_pos: u32,
    private_share: &PrivateShare,
) -> Result<Vec<Address>> {
    let init = 0;
    let last_pos = last_derived_pos;

    let mut response: Vec<Address> = Vec::new();

    for n in init..=last_pos {
        let mk = private_share
            .master_key
            .get_child(vec![BigInt::from(0), BigInt::from(n)]);

        let eth_address = pubkey_to_eth_address(&mk);
        response.push(eth_address);
    }

    Ok(response)
}

pub fn pubkey_to_eth_address(mk: &MasterKey2) -> Address {
    let pub_k = mk.public.q.get_element().serialize_uncompressed();
    let hash = keccak256(&pub_k[1..]);
    Address::from_slice(&hash[12..])
}

pub async fn get_balance_in_eth(
    public_address: String,
    web3_connection: &Web3<transports::WebSocket>,
) -> Result<f64> {
    let wei_balance = get_balance(public_address, web3_connection).await?;
    Ok(wei_to_eth(wei_balance))
}

async fn get_balance(public_address: String, web3_connection: &Web3<WebSocket>) -> Result<U256> {
    let wallet_address = handle_eth_address_conversion(public_address.as_str())?;
    let balance = web3_connection.eth().balance(wallet_address, None).await?;
    Ok(balance)
}

pub fn wei_to_eth(wei_val: U256) -> f64 {
    let res = wei_val.as_u128() as f64;
    res / 1_000_000_000_000_000_000.0
}

pub fn eth_to_wei(eth_val: f64) -> U256 {
    let result = eth_val * 1_000_000_000_000_000_000.0;
    let result = result as u128;

    U256::from(result)
}

pub async fn establish_web3_connection(url: &str) -> Result<Web3<transports::WebSocket>> {
    let transport = transports::WebSocket::new(url).await?;
    Ok(Web3::new(transport))
}

pub fn handle_eth_address_conversion(address: &str) -> Result<H160> {
    match Address::from_str(address) {
        Ok(address) => Ok(address),
        Err(e) => {
            let error_msg = format!("Error converting to ETH address: {}", e);
            println!("{}", error_msg);
            Err(anyhow!(error_msg))
        }
    }
}
