use kms::ecdsa::two_party::{MasterKey2, Party2Public, MasterKey1};
use two_party_ecdsa::{BigInt, party_two, GE, party_one, FE};

pub fn set_master_key(
    chain_code: &BigInt,
    ec_key_pair_party2: &party_two::EcKeyPair,
    party1_second_message_public_share: &GE,
    paillier_public: &party_two::PaillierPublic,
) -> MasterKey2 {
    let party2_public = Party2Public {
        q: party_two::compute_pubkey(ec_key_pair_party2, party1_second_message_public_share),
        p2: ec_key_pair_party2.public_share.clone(),
        p1: party1_second_message_public_share.clone(),
        paillier_pub: paillier_public.ek.clone(),
        c_key: paillier_public.encrypted_secret_share.clone(),
    };
    let party2_private = party_two::Party2Private::set_private_key(&ec_key_pair_party2);
    MasterKey2 {
        public: party2_public,
        private: party2_private,
        chain_code: chain_code.clone(),
    }
}

//  master key of party one from counter party recovery (party two recovers party one secret share)
pub fn counter_master_key_from_recovered_secret(mk2: MasterKey2, party_one_secret: FE) -> MasterKey1 {
    let (_, _, ec_key_pair_party1) =
        party_one::KeyGenFirstMsg::create_commitments_with_fixed_secret_share(party_one_secret);
    let paillier_key_pair =
        party_one::PaillierKeyPair::generate_keypair_and_encrypted_share(&ec_key_pair_party1);

    let party_one_private =
        party_one::Party1Private::set_private_key(&ec_key_pair_party1, &paillier_key_pair);

    // set master keys:
    MasterKey1::set_master_key(
        &mk2.chain_code,
        party_one_private,
        &ec_key_pair_party1.public_share,
        &mk2.public.p2,
        paillier_key_pair,
    )
}

pub fn recover_master_key(
    recovered_secret: FE,
    party_two_public: Party2Public,
    chain_code: BigInt,
) -> MasterKey2 {
    //  master key of party two from party two secret recovery:
    // q1 (public key of party one), chain code, and public paillier data (c_key, ek) are needed for
    // recovery of party two master key. paillier data can be refreshed but q1 and cc must be the same
    // as before. Therefore there are two options:
    // (1) party 2 kept the public data of the master key and can retrieve it (only private key was lost)
    // (2) party 2 lost the public data as well. in this case only party 1 can help with the public data.
    //     if party 1 becomes malicious it means two failures at the same time from which the system will not be able to recover.
    //     Therefore no point of running any secure protocol with party 1 and just accept the public data as is.

    let (_, ec_key_pair_party2) =
        party_two::KeyGenFirstMsg::create_with_fixed_secret_share(recovered_secret);
    let party2_private = party_two::Party2Private::set_private_key(&ec_key_pair_party2);
    MasterKey2 {
        public: party_two_public,
        private: party2_private,
        chain_code,
    }
}
