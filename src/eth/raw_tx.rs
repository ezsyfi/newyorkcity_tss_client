use super::utils::eth_to_wei;
use crate::ecdsa::{sign, PrivateShare};
use crate::eth::transaction::{Transaction, EIP1559_TX_ID};
use crate::eth::utils::establish_web3_connection;
use crate::utilities::requests::ClientShim;

use anyhow::Result;
use curv::arithmetic::traits::Converter; // Need for signing
use curv::BigInt;
use hex;
use kms::ecdsa::two_party::MasterKey2;
use tokio::sync::mpsc;
use tokio::task;
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
    from: Address,
    to: Address,
    eth_value: f64,
    client_shim: &ClientShim,
    pos: u32,
    private_share: &PrivateShare,
    mk: &MasterKey2,
) -> Result<()> {
    let tx_params = create_eth_transaction(to, eth_value).await?;

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
    let web3 = establish_web3_connection(web3_connection_url).await?;
    let (nonce, gas_price, chain_id) = futures::future::try_join3(
        maybe!(tx_params.nonce, web3.eth().transaction_count(from, None)),
        maybe!(gas_price, web3.eth().gas_price()),
        maybe!(tx_params.chain_id.map(U256::from), web3.eth().chain_id()),
    )
    .await?;
    let chain_id = chain_id.as_u64();
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

    let sig_hash = tx.get_hash(chain_id);

    // let sig = sign(
    //     client_shim,
    //     BigInt::from_hex(&hex::encode(&sig_hash[..])).unwrap(),
    //     mk,
    //     BigInt::from(0),
    //     BigInt::from(pos),
    //     &private_share.id,
    // )?;

    // let (txc, mut rxc) = mpsc::channel(2);

    // let worker = task::spawn_blocking(move || {
    //     txc.blocking_send(sign(
    //         cl,
    //         BigInt::from_hex(&hex::encode(&sig_hash[..])).unwrap(),
    //         mk,
    //         BigInt::from(0),
    //         BigInt::from(pos),
    //         &private_share.id,
    //     ))
    //     .unwrap();
    // });
    // let sig = rxc.recv().await.unwrap()?;
    // worker.await.unwrap();

    let r = H256::from_slice(&BigInt::to_bytes(&sig.r));
    let s = H256::from_slice(&BigInt::to_bytes(&sig.s));
    let v = sig.recid as u64;
    let signature = Signature { r, s, v };

    let signed = tx.sign(signature, chain_id);

    let transaction_result = web3
        .eth()
        .send_raw_transaction(signed.raw_transaction)
        .await?;
    println!("{:?}", transaction_result);
    // Ok(transaction_result)
    Ok(())
}