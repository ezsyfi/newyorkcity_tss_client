use anyhow::Result;
use bitcoin::hashes::hex::ToHex;
use bitcoin::{self};
use curv::elliptic::curves::secp256_k1::GE;
use curv::elliptic::curves::traits::ECPoint;
use curv::BigInt;
use kms::ecdsa::two_party::MasterKey2;
use kms::ecdsa::two_party::*;
use serde_json::{self};
use std::fs;
use uuid::Uuid;

use centipede::juggling::proof_system::{Helgamalsegmented, Proof};
use centipede::juggling::segmentation::Msegmentation;
use kms::chain_code::two_party::party2::ChainCode2;

use crate::btc::utils::{get_bitcoin_network, to_bitcoin_address, to_bitcoin_public_key};
use crate::eth::utils::{
    check_address_info, establish_web3_connection, get_balance_in_eth, to_eth_address,
};
use crate::utilities::dto::{BlockCypherRawTx, MKPosDto, UtxoAggregator};
use crate::utilities::hd_wallet::derive_new_key;
use crate::utilities::requests::ClientShim;

use super::btc;

use super::ecdsa;
use super::ecdsa::types::PrivateShare;
use super::escrow;
use super::utilities::requests;
use std::collections::HashMap;

// TODO: move that to a config file and double check electrum server addresses
const WALLET_FILENAME: &str = "wallet/wallet.json";
const BACKUP_FILENAME: &str = "wallet/backup.data";
const BLOCK_CYPHER_HOST: &str = "https://api.blockcypher.com/v1/btc/test3";

#[derive(Serialize, Deserialize)]
pub struct Wallet {
    pub id: String,
    pub coin_type: String,
    pub network: String,
    pub private_share: PrivateShare,
    pub last_derived_pos: u32,
    pub addresses_derivation_map: HashMap<String, MKPosDto>,
}

impl Wallet {
    pub fn new(client_shim: &ClientShim, net: &str, c_type: &str) -> Wallet {
        let id = Uuid::new_v4().to_string();
        let private_share = match ecdsa::get_master_key(client_shim) {
            Ok(p) => p,
            Err(e) => panic!("{}", e),
        };
        let last_derived_pos = 0;
        let addresses_derivation_map = HashMap::new();

        Wallet {
            id,
            coin_type: c_type.to_owned(),
            network: net.to_owned(),
            private_share,
            last_derived_pos,
            addresses_derivation_map,
        }
    }

    pub fn rotate(self, client_shim: &ClientShim) -> Self {
        ecdsa::rotate_master_key(self, client_shim)
    }

    pub fn backup(&self, escrow_service: escrow::Escrow) {
        let g: GE = ECPoint::generator();
        let y = escrow_service.get_public_key();
        let (segments, encryptions) = self.private_share.master_key.private.to_encrypted_segment(
            escrow::SEGMENT_SIZE,
            escrow::NUM_SEGMENTS,
            &y,
            &g,
        );

        let proof = Proof::prove(&segments, &encryptions, &g, &y, &escrow::SEGMENT_SIZE);

        let client_backup_json = serde_json::to_string(&(
            encryptions,
            proof,
            self.private_share.master_key.public.clone(),
            self.private_share.master_key.chain_code.clone(),
            self.private_share.id.clone(),
        ))
        .unwrap();

        fs::write(BACKUP_FILENAME, client_backup_json).expect("Unable to save client backup!");

        debug!("(wallet id: {}) Backup wallet with escrow", self.id);
    }

    pub fn verify_backup(&self, escrow_service: escrow::Escrow) {
        let g: GE = ECPoint::generator();
        let y = escrow_service.get_public_key();

        let data = fs::read_to_string(BACKUP_FILENAME).expect("Unable to load client backup!");
        let (encryptions, proof, client_public, _, _): (
            Helgamalsegmented,
            Proof,
            Party2Public,
            ChainCode2,
            String,
        ) = serde_json::from_str(&data).unwrap();
        let verify = proof.verify(
            &encryptions,
            &g,
            &y,
            &client_public.p2,
            &escrow::SEGMENT_SIZE,
        );
        match verify {
            Ok(_x) => println!("backup verified 🍻"),
            Err(_e) => println!("Backup was not verified correctly 😲"),
        }
    }

    pub fn recover_and_save_share(
        escrow_service: escrow::Escrow,
        net: &str,
        client_shim: &ClientShim,
    ) -> Wallet {
        let g: GE = ECPoint::generator();
        let y_priv = escrow_service.get_private_key();

        let data = fs::read_to_string(BACKUP_FILENAME).expect("Unable to load client backup!");

        let (encryptions, _proof, public_data, chain_code2, key_id): (
            Helgamalsegmented,
            Proof,
            Party2Public,
            BigInt,
            String,
        ) = serde_json::from_str(&data).unwrap();

        let sk = Msegmentation::decrypt(&encryptions, &g, &y_priv, &escrow::SEGMENT_SIZE);

        let client_master_key_recovered =
            MasterKey2::recover_master_key(sk.unwrap(), public_data, chain_code2);
        let pos_old: u32 = requests::post(client_shim, &format!("ecdsa/{}/recover", key_id))
            .unwrap()
            .unwrap();

        let pos_old = if pos_old < 10 { 10 } else { pos_old };
        //TODO: temporary, server will keep updated pos, to do so we need to send update to server for every get_new_address

        let id = Uuid::new_v4().to_string();
        let addresses_derivation_map = HashMap::new(); //TODO: add a fucntion to recreate

        let new_wallet = Wallet {
            id,
            coin_type: "btc".to_owned(),
            network: net.to_owned(),
            private_share: PrivateShare {
                master_key: client_master_key_recovered,
                id: key_id,
            },
            last_derived_pos: pos_old,
            addresses_derivation_map,
        };

        new_wallet.save();
        println!("Recovery Completed Successfully ❤️");

        new_wallet
    }

    pub fn save_to(&self, filepath: &str) {
        let wallet_json = serde_json::to_string(self).unwrap();

        fs::write(filepath, wallet_json).expect("Unable to save wallet!");

        debug!("(wallet id: {}) Saved wallet to disk", self.id);
    }

    pub fn save(&self) {
        self.save_to(WALLET_FILENAME)
    }

    pub fn load_from(filepath: &str) -> Wallet {
        let data = fs::read_to_string(filepath).expect("Unable to load wallet!");

        let wallet: Wallet = serde_json::from_str(&data).unwrap();

        debug!("(wallet id: {}) Loaded wallet to memory", wallet.id);

        wallet
    }

    pub fn load() -> Wallet {
        Wallet::load_from(WALLET_FILENAME)
    }

    pub fn send(
        &mut self,
        to_address: String,
        amount_btc: f32,
        client_shim: &ClientShim,
    ) -> Option<String> {
        let raw_tx_opt = btc::raw_tx::create_raw_tx(
            to_address,
            amount_btc,
            client_shim,
            self.last_derived_pos,
            &self.private_share,
            &self.addresses_derivation_map,
        );

        let raw_tx = match raw_tx_opt {
            Ok(tx) => tx,
            Err(_) => {
                println!("Unable to create raw transaction");
                return Some("".to_owned());
            }
        };

        let raw_tx_url = BLOCK_CYPHER_HOST.to_owned() + "/txs/push";
        let raw_tx = BlockCypherRawTx {
            tx: raw_tx?.raw_tx_hex,
        };
        let res = reqwest::blocking::Client::new()
            .post(raw_tx_url)
            .json(&raw_tx)
            .send()
            .unwrap()
            .text()
            .unwrap();

        print!("{}", res);

        Some(res)
    }

    pub fn get_crypto_address(&mut self) {
        let (pos, mk) = derive_new_key(&self.private_share, self.last_derived_pos);
        let coin_type = &self.coin_type;
        if coin_type == "btc" {
            let address = to_bitcoin_address(&self.network, &mk).unwrap();
            self.addresses_derivation_map
                .insert(address.to_string(), MKPosDto { mk, pos });
            self.last_derived_pos = pos;

            println!("BTC Network: [{}], Address: [{}]", &self.network, address);
        } else if coin_type == "eth" {
            let address = to_eth_address(&mk);
            self.addresses_derivation_map
                .insert(format!("{:?}", address), MKPosDto { mk, pos });
            self.last_derived_pos = pos;

            println!("ETH address: {:?}", address);
        }
    }

    pub fn derived(&mut self) -> Result<()> {
        for i in 0..self.last_derived_pos {
            let (pos, mk) = derive_new_key(&self.private_share, i);

            let address = bitcoin::Address::p2wpkh(
                &to_bitcoin_public_key(mk.public.q.get_element()),
                get_bitcoin_network(&self.network)?,
            )
            .expect("Cannot panic because `to_bitcoin_public_key` creates a compressed address");

            self.addresses_derivation_map
                .insert(address.to_string(), MKPosDto { mk, pos });
        }
        Ok(())
    }

    pub fn get_balance(&mut self) {
        let coin_type = &self.coin_type;
        if coin_type == "btc" {
            let mut unconfirmed = 0;
            let mut confirmed = 0;
            for b in
                btc::utils::get_all_addresses_balance(self.last_derived_pos, &self.private_share)
                    .unwrap()
            {
                unconfirmed += b.unconfirmed;
                confirmed += b.confirmed;
            }
            println!(
                "Network: [{}], Balance: [balance: {}, pending: {}]",
                &self.network, confirmed, unconfirmed
            );
        } else if coin_type == "eth" {
            let total = get_eth_balance(self.last_derived_pos, &self.private_share).unwrap();
            println!("ETH Balance: [{}]", total);
        }
    }

    pub fn list_unspent(&self) -> Vec<UtxoAggregator> {
        let response: Vec<UtxoAggregator> =
            btc::utils::get_all_addresses(self.last_derived_pos, &self.private_share)
                .unwrap()
                .into_iter()
                .map(|a| btc::utils::list_unspent_for_addresss(a.to_string()).unwrap())
                .flatten()
                .collect();

        response
    }
}

#[tokio::main]
async fn get_eth_balance(last_derived_pos: u32, private_share: &PrivateShare) -> Result<f64> {
    let balance = check_address_info(last_derived_pos, private_share).await?;
    Ok(balance)
}

#[cfg(test)]
mod tests {
    use bitcoin::hashes::hex::ToHex;
    use bitcoin::hashes::sha256d;
    use bitcoin::hashes::Hash;
    use curv::arithmetic::traits::Converter;
    use curv::BigInt;

    #[test]
    fn test_message_conv() {
        let message: [u8; 32] = [
            1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        ];

        // 14abf5ed107ff58bf844ee7f447bec317c276b00905c09a45434f8848599597e
        let hash = sha256d::Hash::from_slice(&message).unwrap();

        // 7e59998584f83454a4095c90006b277c31ec7b447fee44f88bf57f10edf5ab14
        let ser = hash.to_hex();

        // 57149727877124134702546803488322951680010683936655914236113461592936003513108
        let b: BigInt = BigInt::from_hex(&ser).unwrap();

        println!("({},{},{})", hash, ser, b.to_hex());
    }
}
