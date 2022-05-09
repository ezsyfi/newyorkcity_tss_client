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

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct BlockCypherTX {
    pub block_hash: String,
    pub block_height: isize,
    pub hash: String,
    pub addresses: Vec<BlockCypherAddress>,
    pub total: u64,
    pub fees: u64,
    pub size: u64,
    pub vsize: u64,
    pub preference: String,
    pub relayed_by: String,
    pub confirmed: String,
    pub received: String,
    pub ver: u16,
    pub lock_time: u64,
    pub double_spend: bool,
    pub vin_sz: usize,
    pub vout_sz: usize,
    pub confirmations: u64,
    #[serde(skip)]
    pub inputs: String,
    #[serde(skip)]
    pub outputs: String,
}
