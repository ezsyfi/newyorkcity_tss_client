use anyhow::Result;
use curv::elliptic::curves::traits::ECPoint;
use kms::ecdsa::two_party::MasterKey2;
use std::str::FromStr;
use web3::{
    self,
    signing::keccak256,
    transports::{self, WebSocket},
    types::{Address, U256},
    Web3,
};

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
    public_address: &str,
    web3_connection: &Web3<transports::WebSocket>,
) -> Result<f64> {
    let wei_balance = get_balance(public_address, web3_connection).await?;
    Ok(wei_to_eth(wei_balance))
}

pub async fn get_balance(public_address: &str, web3_connection: &Web3<WebSocket>) -> Result<U256> {
    let wallet_address = Address::from_str(public_address)?;
    let balance = web3_connection.eth().balance(wallet_address, None).await?;
    Ok(balance)
}

pub fn wei_to_eth(wei_val: U256) -> f64 {
    let res = wei_val.as_u128() as f64;
    res / 1_000_000_000_000_000_000.0
}

pub async fn check_address_info(public_address: &str) -> Result<f64> {
    let web3_con = establish_web3_connection(
        "wss://eth-rinkeby.alchemyapi.io/v2/UmSDyVix3dL4CtIxC2zlKkSuk2UoRw1J",
    )
    .await?;
    let block_number = web3_con.eth().block_number().await?;
    println!("block number: {}", &block_number);

    let balance = get_balance_in_eth(public_address, &web3_con).await?;
    Ok(balance)
}
