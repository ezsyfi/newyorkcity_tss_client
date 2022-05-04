use super::Rotation;
use curv::cryptographic_primitives::twoparty::coin_flip_optimal_rounds;
use curv::elliptic::curves::{Scalar, Secp256k1};
use sha2::Sha256;

pub struct Rotation1 {}

impl Rotation1 {
    //TODO: implmenet sid / state machine
    pub fn key_rotate_first_message() -> (
        coin_flip_optimal_rounds::Party1FirstMessage<Secp256k1, Sha256>,
        Scalar<Secp256k1>,
        Scalar<Secp256k1>,
    ) {
        coin_flip_optimal_rounds::Party1FirstMessage::commit()
    }

    pub fn key_rotate_second_message(
        party2_first_message: &coin_flip_optimal_rounds::Party2FirstMessage<Secp256k1>,
        m1: &Scalar<Secp256k1>,
        r1: &Scalar<Secp256k1>,
    ) -> (
        coin_flip_optimal_rounds::Party1SecondMessage<Secp256k1, Sha256>,
        Rotation,
    ) {
        let (res1, res2) =
            coin_flip_optimal_rounds::Party1SecondMessage::<Secp256k1, Sha256>::reveal(
                &party2_first_message.seed,
                m1,
                r1,
            );

        (res1, Rotation { rotation: res2 })
    }
}
