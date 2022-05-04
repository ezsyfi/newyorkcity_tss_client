use curv::{elliptic::curves::{Scalar, Secp256k1}, BigInt};
use kms::ecdsa::two_party::MasterKey2;
use kzen_paillier::EncryptionKey;
use two_party_ecdsa::party_one::DLogProof;
use zk_paillier::zkproofs::{CompositeDLogProof};
use multi_party_ecdsa::{utilities::zk_pdl_with_slack::PDLwSlackProof, protocols::two_party_ecdsa::lindell_2017::party_two};
use multi_party_ecdsa::utilities::zk_pdl_with_slack::PDLwSlackStatement;

pub mod party1;
pub mod party2;
#[derive(Debug, Serialize, Deserialize)]
pub struct Rotation {
    pub rotation: Scalar<Secp256k1>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RotationParty1Message1 {
    pub ek: EncryptionKey,
    pub c_key_new: BigInt,
    pub correct_key_proof: DLogProof,
    pub pdl_statement: PDLwSlackStatement,
    pub pdl_proof: PDLwSlackProof,
    pub composite_dlog_proof: CompositeDLogProof,
}


pub fn rotate_first_message(
    mk2: MasterKey2,
    cf: &Rotation,
    party_one_rotation_first_message: &RotationParty1Message1,
    party_one_rotation_first_message_salt: &[u8]
) -> Result<MasterKey2, ()> {
    let party_two_paillier = party_two::PaillierPublic {
        ek: party_one_rotation_first_message.ek.clone(),
        encrypted_secret_share: party_one_rotation_first_message.c_key_new.clone(),
    };

    let pdl_verify = party_two::PaillierPublic::pdl_verify(
        &party_one_rotation_first_message.composite_dlog_proof,
        &party_one_rotation_first_message.pdl_statement,
        &party_one_rotation_first_message.pdl_proof,
        &party_two_paillier,
        &(mk2.public.p1 * &cf.rotation),
    );

    let correct_key_verify = party_one_rotation_first_message
        .correct_key_proof
        .verify(&party_two_paillier.ek, party_one_rotation_first_message_salt);

    let master_key = mk2.rotate(cf, &party_two_paillier);

    match pdl_verify {
        Ok(_proof) => match correct_key_verify {
            Ok(_proof) => Ok(master_key),
            Err(_correct_key_error) => Err(()),
        },
        Err(_range_proof_error) => Err(()),
    }
}