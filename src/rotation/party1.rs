use super::Rotation;
use curv::cryptographic_primitives::twoparty::coin_flip_optimal_rounds;
use curv::elliptic::curves::secp256_k1::{Secp256k1Scalar, GE};


pub struct Rotation1 {}

impl Rotation1 {
    //TODO: implmenet sid / state machine
    pub fn key_rotate_first_message() -> (
        coin_flip_optimal_rounds::Party1FirstMessage<GE>,
        Secp256k1Scalar,
        Secp256k1Scalar,
    ) {
        coin_flip_optimal_rounds::Party1FirstMessage::commit()
    }

    pub fn key_rotate_second_message(
        party2_first_message: &coin_flip_optimal_rounds::Party2FirstMessage<GE>,
        m1: &Secp256k1Scalar,
        r1: &Secp256k1Scalar,
    ) -> (coin_flip_optimal_rounds::Party1SecondMessage<GE>, Rotation) {
        let (res1, res2) = coin_flip_optimal_rounds::Party1SecondMessage::reveal(
            &party2_first_message.seed,
            m1,
            r1,
        );

        (res1, Rotation { rotation: res2 })
    }
}
