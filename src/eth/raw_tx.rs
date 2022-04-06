use std::str::FromStr;

use super::utils::eth_to_wei;
use crate::ecdsa::sign::a_sign;
use crate::ecdsa::PrivateShare;
use crate::eth::transaction::{Transaction, EIP1559_TX_ID};
use crate::eth::utils::{establish_web3_connection, to_eth_address};
use crate::utilities::a_requests::AsyncClientShim;

use anyhow::Result;
use curv::arithmetic::traits::Converter;
use curv::BigInt;
use hex;
use kms::ecdsa::two_party::MasterKey2;
use web3::types::{Address, TransactionParameters, H256, U256, U64};
use web3::{self, signing::Signature};

async fn create_eth_transaction(to: Address, eth_value: f64) -> Result<TransactionParameters> {
    Ok(TransactionParameters {
        to: Some(to),
        value: eth_to_wei(eth_value),
        ..Default::default()
    })
}

#[allow(clippy::too_many_arguments)]
pub async fn sign_and_send(
    web3_connection_url: &str,
    to: &str,
    eth_value: f64,
    client_shim: &AsyncClientShim,
    pos: u32,
    private_share: &PrivateShare,
    mk: &MasterKey2,
) -> Result<H256> {
    let to_address = Address::from_str(to)?;
    let tx_params = create_eth_transaction(to_address, eth_value).await?;

    macro_rules! maybe {
        ($o: expr, $f: expr) => {
            async {
                match $o {
                    Some(value) => Ok(value),
                    None => $f.await,
                }
            }
        };
    }

    let gas_price = match tx_params.transaction_type {
        Some(tx_type)
            if tx_type == U64::from(EIP1559_TX_ID) && tx_params.max_fee_per_gas.is_some() =>
        {
            tx_params.max_fee_per_gas
        }
        _ => tx_params.gas_price,
    };

    let from_address = to_eth_address(mk);

    let web3 = establish_web3_connection(web3_connection_url).await?;

    let (nonce, gas_price, chain_id) = futures::future::try_join3(
        maybe!(
            tx_params.nonce,
            web3.eth().transaction_count(from_address, None)
        ),
        maybe!(gas_price, web3.eth().gas_price()),
        maybe!(tx_params.chain_id.map(U256::from), web3.eth().chain_id()),
    )
    .await?;

    let max_priority_fee_per_gas = match tx_params.transaction_type {
        Some(tx_type) if tx_type == U64::from(EIP1559_TX_ID) => {
            tx_params.max_priority_fee_per_gas.unwrap_or(gas_price)
        }
        _ => gas_price,
    };
    let tx = Transaction {
        to: tx_params.to,
        nonce,
        gas: tx_params.gas,
        gas_price,
        value: tx_params.value,
        data: tx_params.data.0,
        transaction_type: tx_params.transaction_type,
        access_list: tx_params.access_list.unwrap_or_default(),
        max_priority_fee_per_gas,
    };
    let chain_id = chain_id.as_u64();
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

    let transaction_result = web3
        .eth()
        .send_raw_transaction(signed.raw_transaction)
        .await?;

    Ok(transaction_result)
}
