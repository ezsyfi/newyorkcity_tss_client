// use curv::BigInt;
// use kms::ecdsa::two_party::MasterKey2;
// // use multi_party_ecdsa::{utilities::zk_pdl_with_slack::{PDLwSlackStatement, PDLwSlackProof}};
// use two_party_ecdsa::{party_two::{self, PaillierPublic}, GE, BigInt};
// use zk_paillier::zkproofs::{CompositeDLogProof, DLogStatement};

// use crate::dto::rotate::{Rotation, RotationParty1Message1};

// use super::pdl_with_slack::PDLwSlackStatement;

// pub fn rotate_first_message(
//     self,
//     cf: &Rotation,
//     party_one_rotation_first_message: &RotationParty1Message1,
//     party_one_rotation_first_message_salt: &[u8]
// ) -> Result<MasterKey2, ()> {
//     let party_two_paillier = party_two::PaillierPublic {
//         ek: party_one_rotation_first_message.ek.clone(),
//         encrypted_secret_share: party_one_rotation_first_message.c_key_new.clone(),
//     };

//     let pdl_verify = pdl_verify(
//         &party_one_rotation_first_message.composite_dlog_proof,
//         &party_one_rotation_first_message.pdl_statement,
//         &party_one_rotation_first_message.pdl_proof,
//         &party_two_paillier,
//         &(self.public.p1 * &cf.rotation),
//     );

//     let correct_key_verify = party_one_rotation_first_message
//         .correct_key_proof
//         .verify(&party_two_paillier.ek);

//     let master_key = self.rotate(cf, &party_two_paillier);

//     match pdl_verify {
//         Ok(_proof) => match correct_key_verify {
//             Ok(_proof) => Ok(master_key),
//             Err(_correct_key_error) => Err(()),
//         },
//         Err(_range_proof_error) => Err(()),
//     }
// }

// pub fn pdl_verify(
//     composite_dlog_proof: &CompositeDLogProof,
//     pdl_w_slack_statement: &PDLwSlackStatement,
//     pdl_w_slack_proof: &PDLwSlackProof,
//     paillier_public: &PaillierPublic,
//     q1: &GE,
// ) -> Result<(), ()> {
//     if &pdl_w_slack_statement.ek != &paillier_public.ek
//         || &pdl_w_slack_statement.ciphertext != &paillier_public.encrypted_secret_share
//         || &pdl_w_slack_statement.Q != q1
//     {
//         return Err(());
//     }
//     let dlog_statement = DLogStatement {
//         N: BigInt {
//             gmp: pdl_w_slack_statement.N_tilde.clone(),
//         },
//         g: pdl_w_slack_statement.h1.clone(),
//         ni: pdl_w_slack_statement.h2.clone(),
//     };
//     if composite_dlog_proof.verify(&dlog_statement).is_ok()
//         && pdl_w_slack_proof.verify(&pdl_w_slack_statement).is_ok()
//     {
//         return Ok(());
//     } else {
//         return Err(());
//     }
// }
