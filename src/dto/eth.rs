use web3::types::{Address, Bytes, H256, U256, U64};

#[derive(Serialize, Deserialize)]
pub struct EthTxParamsReqBody {
    pub from_address: Address,
    pub to_address: Address,
    pub eth_value: f64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EthTxParamsResp {
    pub nonce: U256,
    pub gas: U256,
    pub gas_price: U256,
    pub transaction_type: Option<U64>,
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

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Erc20ReqBody {
    pub name: String,
    pub network: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Erc20Resp {
    pub contract: web3::ethabi::Contract,
}
