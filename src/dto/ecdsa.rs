use curv::BigInt;
use kms::ecdsa::two_party::MasterKey2;
use kms::ecdsa::two_party::*;

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
