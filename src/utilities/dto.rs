use curv::BigInt;
use kms::ecdsa::two_party::MasterKey2;
use kms::ecdsa::two_party::*;
use web3::types::{AccessList, Address, Bytes, H256, U256, U64};

#[derive(Serialize, Deserialize)]
pub struct SignSecondMsgRequest {
    pub message: BigInt,
    pub party_two_sign_message: party2::SignMessage,
    pub pos_child_key: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct BtcBalanceAggregator {
    pub address: String,
    pub confirmed: u64,
    pub unconfirmed: i64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct UtxoAggregator {
    pub height: isize,
    pub tx_hash: String,
    pub tx_pos: usize,
    pub value: usize,
    pub address: String,
}

#[derive(Serialize, Deserialize)]
pub struct MKPosDto {
    pub pos: u32,
    pub mk: MasterKey2,
}

#[derive(Serialize, Deserialize)]
pub struct MKPosAddressDto {
    pub address: String,
    pub pos: u32,
    pub mk: MasterKey2,
}

// BLOCKCYPHER DTO
#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct BlockCypherAddress {
    pub address: String,
    pub total_received: u64,
    pub total_sent: u64,
    pub balance: u64,
    pub unconfirmed_balance: i64,
    pub final_balance: u64,
    pub n_tx: u64,
    pub unconfirmed_n_tx: u64,
    pub final_n_tx: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub txrefs: Option<Vec<BlockCypherTxRef>>,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct BlockCypherTxRef {
    pub tx_hash: String,
    pub block_height: isize,
    pub tx_input_n: isize,
    pub tx_output_n: usize,
    pub value: usize,
    pub ref_balance: u64,
    pub spent: bool,
    pub confirmations: u64,
    pub confirmed: String,
}

#[derive(Serialize, Deserialize)]
pub struct BlockCypherRawTx {
    pub tx: String,
}

#[derive(Serialize, Deserialize)]
pub struct EthTxParamsReqBody {
    pub from_address: Address,
    pub to_address: Address,
    pub eth_value: f64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EthTxParamsResp {
    pub to: Option<Address>,
    pub nonce: U256,
    pub gas: U256,
    pub gas_price: U256,
    pub value: U256,
    pub data: Vec<u8>,
    pub transaction_type: Option<U64>,
    pub access_list: AccessList,
    pub max_priority_fee_per_gas: U256,
    pub chain_id: u64,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct EthSendTxResp {
    pub tx_hash: H256,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct EthSendTxReqBody {
    pub raw_tx: Bytes,
}
