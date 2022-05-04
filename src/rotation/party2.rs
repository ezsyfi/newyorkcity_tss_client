use curv::{cryptographic_primitives::twoparty::coin_flip_optimal_rounds, elliptic::curves::Secp256k1};
use sha2::Sha256;

use super::Rotation;

pub struct Rotation2 {}

impl Rotation2 {
    pub fn key_rotate_first_message(
        party1_first_message: &coin_flip_optimal_rounds::Party1FirstMessage<Secp256k1, Sha256>,
    ) -> coin_flip_optimal_rounds::Party2FirstMessage<Secp256k1> {
        coin_flip_optimal_rounds::Party2FirstMessage::share(&party1_first_message.proof)
    }

    pub fn key_rotate_second_message(
        party1_second_message: &coin_flip_optimal_rounds::Party1SecondMessage<Secp256k1, Sha256>,
        party2_first_message: &coin_flip_optimal_rounds::Party2FirstMessage<Secp256k1>,
        party1_first_message: &coin_flip_optimal_rounds::Party1FirstMessage<Secp256k1, Sha256>,
    ) -> Rotation {
        let rotation = coin_flip_optimal_rounds::finalize(
            &party1_second_message.proof,
            &party2_first_message.seed,
            &party1_first_message.proof.com,
        );
        Rotation { rotation }
    }
}
