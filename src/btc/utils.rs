use std::fs;

use anyhow::{anyhow, Result};
use bitcoin::{self, Network};
use curv::elliptic::curves::secp256_k1::PK;
use curv::elliptic::curves::traits::ECPoint;
use kms::ecdsa::two_party::MasterKey2;

use crate::ecdsa::PrivateShare;
use crate::utilities::hd_wallet::derive_new_key;

pub const BTC_TESTNET: &str = "testnet";

pub fn get_new_bitcoin_address(
    private_share: &PrivateShare,
    last_derived_pos: u32,
) -> Result<bitcoin::Address> {
    let (_pos, mk) = derive_new_key(private_share, last_derived_pos);
    to_bitcoin_address(BTC_TESTNET, &mk)
}

pub fn to_bitcoin_address(network: &str, mk: &MasterKey2) -> Result<bitcoin::Address> {
    let pk = mk.public.q.get_element();
    match bitcoin::Address::p2wpkh(&to_bitcoin_public_key(pk), get_bitcoin_network(network)?) {
        Ok(address) => Ok(address),
        Err(e) => Err(anyhow!("Error while creating bitcoin address: {}", e)),
    }
}

pub fn to_bitcoin_public_key(pk: PK) -> bitcoin::util::key::PublicKey {
    bitcoin::util::key::PublicKey {
        compressed: true,
        key: pk,
    }
}

pub fn get_bitcoin_network(nw: &str) -> Result<Network> {
    let btc_nw = nw.to_owned().parse::<Network>()?;
    Ok(btc_nw)
}

pub fn get_test_private_share() -> PrivateShare {
    const PRIVATE_SHARE_FILENAME: &str = "test-assets/private_share.json";
    let data =
        fs::read_to_string(PRIVATE_SHARE_FILENAME).expect("Unable to load test private_share!");
    serde_json::from_str(&data).unwrap()
}
#[cfg(test)]
mod tests {
    use crate::{
        btc::utils::{get_new_bitcoin_address, get_test_private_share, BTC_TESTNET},
        ecdsa::PrivateShare,
        utilities::hd_wallet::derive_new_key,
    };
    use anyhow::Result;
    use bitcoin::Network;
    use curv::elliptic::curves::traits::ECPoint;

    #[test]
    fn test_get_bitcoin_network() -> Result<()> {
        let network = super::get_bitcoin_network(BTC_TESTNET)?;
        assert_eq!(network, Network::Testnet);
        Ok(())
    }

    #[test]
    fn test_derive_new_key() {
        let private_share: PrivateShare = get_test_private_share();
        let (pos, mk) = derive_new_key(&private_share, 0);
        let pk = mk.public.q.get_element();
        assert!(!pk.to_string().is_empty());
        assert_eq!(pos, 1);
    }

    #[test]
    fn test_get_new_bitcoin_address() -> Result<()> {
        let private_share: PrivateShare = get_test_private_share();
        let addrs = get_new_bitcoin_address(&private_share, 0)?;
        let exp = "tb1qxyjt450heqv4ql8k7rp2qfmd4vrmncaquzw37r".to_string();
        assert_eq!(addrs.to_string(), exp);
        Ok(())
    }
}
