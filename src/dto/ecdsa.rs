use kms::ecdsa::two_party::{party2, MasterKey2};
use two_party_ecdsa::BigInt;

#[derive(Serialize, Deserialize)]
pub struct SignSecondMsgRequest {
    pub message: BigInt,
    pub party_two_sign_message: party2::SignMessage,
    pub pos_child_key: u32,
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

