use std::str::FromStr;

use crate::ecdsa::sign::a_sign;
use crate::ecdsa::PrivateShare;
use crate::eth::transaction::Transaction;
use crate::eth::utils::to_eth_address;
use crate::utilities::a_requests::{self, AsyncClientShim};
use crate::utilities::dto::{EthTxParamsReqBody, EthTxParamsResp, EthSendTxReqBody, EthSendTxResp};

use anyhow::{anyhow, Result};
use curv::arithmetic::traits::Converter;
use curv::BigInt;
use hex;
use kms::ecdsa::two_party::MasterKey2;
use web3::types::{Address, H256};
use web3::{self, signing::Signature};

#[allow(clippy::too_many_arguments)]
pub async fn sign_and_send(
    to: &str,
    eth_value: f64,
    client_shim: &AsyncClientShim,
    pos: u32,
    private_share: &PrivateShare,
    mk: &MasterKey2,
) -> Result<H256> {
    let to_address = Address::from_str(to)?;
    let from_address = to_eth_address(mk);

    let tx_params_body = EthTxParamsReqBody {
        from_address,
        to_address,
        eth_value,
    };

    let tx_params: EthTxParamsResp =
        match a_requests::a_postb(client_shim, "eth/tx/params", tx_params_body).await? {
            Some(s) => s,
            None => return Err(anyhow!("get ETH tx params request failed")),
        };

    println!("{:#?}", tx_params);

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
    let msg = tx.get_hash(chain_id);

    let sig = a_sign(
        client_shim,
        BigInt::from_hex(&hex::encode(&msg[..])).unwrap(),
        mk,
        BigInt::from(0),
        BigInt::from(pos),
        &private_share.id,
    )
    .await?;

    let r = H256::from_slice(&BigInt::to_bytes(&sig.r));
    let s = H256::from_slice(&BigInt::to_bytes(&sig.s));
    let v = sig.recid as u64 + 35 + chain_id * 2;
    let signature = Signature { r, s, v };
    let signed = tx.sign(signature, chain_id);

    let tx_send_body = EthSendTxReqBody {
        raw_tx: signed.raw_transaction,
    };

    let transaction_result: EthSendTxResp =
        match a_requests::a_postb(client_shim, "eth/tx/send", tx_send_body).await? {
            Some(s) => s,
            None => return Err(anyhow!("send ETH tx request failed")),
        };

    Ok(transaction_result.tx_hash)
}
