use super::super::utilities::requests;
use super::super::wallet;
use super::super::ClientShim;
use super::types::PrivateShare;
use curv::cryptographic_primitives::twoparty::coin_flip_optimal_rounds;
use curv::elliptic::curves::secp256_k1::GE;

use kms::ecdsa::two_party::*;
use kms::rotation::two_party::party2::Rotation2;
use std::collections::HashMap;
use zk_paillier::zkproofs::SALT_STRING;

const ROT_PATH_PRE: &str = "ecdsa/rotate";

pub fn rotate_master_key(wallet: wallet::Wallet, client_shim: &ClientShim) -> wallet::Wallet {
    let id = &wallet.private_share.id.clone();
    let coin_flip_party1_first_message: coin_flip_optimal_rounds::Party1FirstMessage<GE> =
        requests::post(client_shim, &format!("{}/{}/first", ROT_PATH_PRE, id)).unwrap();

    let coin_flip_party2_first_message =
        Rotation2::key_rotate_first_message(&coin_flip_party1_first_message);

    let body = &coin_flip_party2_first_message;

    let (coin_flip_party1_second_message, rotation_party1_first_message): (
        coin_flip_optimal_rounds::Party1SecondMessage<GE>,
        party1::RotationParty1Message1,
    ) = requests::postb(
        client_shim,
        &format!("{}/{}/second", ROT_PATH_PRE, id.clone()),
        body,
    )
    .unwrap();

    let random2 = Rotation2::key_rotate_second_message(
        &coin_flip_party1_second_message,
        &coin_flip_party2_first_message,
        &coin_flip_party1_first_message,
    );

    let result_masterkey2_new = wallet.private_share.master_key.rotate_first_message(
        &random2,
        &rotation_party1_first_message,
        SALT_STRING,
    );
    if result_masterkey2_new.is_err() {
        panic!("rotation failed");
    }

    let party_two_master_key_rotated = result_masterkey2_new.unwrap();

    let private_share = PrivateShare {
        id: wallet.private_share.id.clone(),
        master_key: party_two_master_key_rotated,
    };

    let addresses_derivation_map = HashMap::new();
    let mut wallet_after_rotate = wallet::Wallet {
        id: wallet.id.clone(),
        network: wallet.network.clone(),
        private_share,
        last_derived_pos: wallet.last_derived_pos,
        addresses_derivation_map,
    };
    wallet_after_rotate.derived();

    wallet_after_rotate
}
