use curv::cryptographic_primitives::twoparty::coin_flip_optimal_rounds;
use curv::elliptic::curves::secp256_k1::GE;

use super::Rotation;

pub struct Rotation2 {}

impl Rotation2 {
    pub fn key_rotate_first_message(
        party1_first_message: &coin_flip_optimal_rounds::Party1FirstMessage<GE>,
    ) -> coin_flip_optimal_rounds::Party2FirstMessage<GE> {
        coin_flip_optimal_rounds::Party2FirstMessage::share(&party1_first_message.proof)
    }

    pub fn key_rotate_second_message(
        party1_second_message: &coin_flip_optimal_rounds::Party1SecondMessage<GE>,
        party2_first_message: &coin_flip_optimal_rounds::Party2FirstMessage<GE>,
        party1_first_message: &coin_flip_optimal_rounds::Party1FirstMessage<GE>,
    ) -> Rotation {
        let rotation = coin_flip_optimal_rounds::finalize(
            &party1_second_message.proof,
            &party2_first_message.seed,
            &party1_first_message.proof.com,
        );
        Rotation { rotation }
    }
}
