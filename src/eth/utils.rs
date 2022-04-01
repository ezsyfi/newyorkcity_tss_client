use anyhow::Result;
use curv::{elliptic::curves::traits::ECPoint, BigInt};
use futures::future::try_join_all;
use kms::ecdsa::two_party::MasterKey2;
use std::str::FromStr;
use web3::{
    self,
    signing::keccak256,
    transports::{self, WebSocket},
    types::{Address, U256},
    Web3,
};

use crate::ecdsa::PrivateShare;

pub async fn get_all_addresses_balance(
    last_derived_pos: u32,
    private_share: &PrivateShare,
    web3_connection: &Web3<transports::WebSocket>,
) -> Result<Vec<f64>> {
    let addresses = get_all_addresses(last_derived_pos, private_share).unwrap();
    let result: Vec<f64> = try_join_all(
        addresses
            .iter()
            .map(|a| get_balance_in_eth(a.to_string(), web3_connection)),
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

        let eth_address = to_eth_address(&mk);

        response.push(eth_address);
    }

    Ok(response)
}

pub fn to_eth_address(mk: &MasterKey2) -> Address {
    let pub_k = mk.public.q.get_element().serialize_uncompressed();
    let hash = keccak256(&pub_k[1..]);
    Address::from_slice(&hash[12..])
}

pub async fn establish_web3_connection(url: &str) -> Result<Web3<transports::WebSocket>> {
    let transport = transports::WebSocket::new(url).await?;
    Ok(Web3::new(transport))
}

pub async fn get_balance_in_eth(
    public_address: String,
    web3_connection: &Web3<transports::WebSocket>,
) -> Result<f64> {
    let wei_balance = get_balance(public_address, web3_connection).await?;
    Ok(wei_to_eth(wei_balance))
}

pub async fn get_balance(
    public_address: String,
    web3_connection: &Web3<WebSocket>,
) -> Result<U256> {
    let wallet_address = Address::from_str(public_address.as_str())?;
    let balance = web3_connection.eth().balance(wallet_address, None).await?;
    Ok(balance)
}

pub fn wei_to_eth(wei_val: U256) -> f64 {
    let res = wei_val.as_u128() as f64;
    res / 1_000_000_000_000_000_000.0
}

pub async fn check_address_info(
    last_derived_pos: u32,
    private_share: &PrivateShare,
) -> Result<f64> {
    let web3_con = establish_web3_connection(
        "wss://eth-rinkeby.alchemyapi.io/v2/UmSDyVix3dL4CtIxC2zlKkSuk2UoRw1J",
    )
    .await?;

    let block_number = web3_con.eth().block_number().await?;
    println!("block number: {}", &block_number);

    let balance_l = get_all_addresses_balance(last_derived_pos, private_share, &web3_con).await?;
    let mut total = 0.0;
    for b in balance_l {
       total += b
    }
    Ok(total)
}
