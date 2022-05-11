pub mod common;

#[cfg(test)]
mod ecdsa_test_suite {
    use serde_json::Value;

    use crate::{
        dto::ecdsa::PrivateShare, ecdsa::rotate_private_share, tests::common::mock_client_shim,
        wallet::Wallet,
    };

    fn get_coordinates_of_2_mk(private_share: &PrivateShare) -> (String, String) {
        let mk1 = private_share.master_key.public.p1;
        let mk2 = private_share.master_key.public.p2;
        let mk1_obj: Value =
            serde_json::from_str(serde_json::to_string(&mk1).unwrap().as_str()).unwrap();
        let x1 = mk1_obj["x"].as_str().unwrap();
        let mk2_obj: Value =
            serde_json::from_str(serde_json::to_string(&mk2).unwrap().as_str()).unwrap();
        let x2 = mk2_obj["x"].as_str().unwrap();
        (x1.to_string(), x2.to_string())
    }

    fn get_coordinate_of_p2_private(private_share: &PrivateShare) -> String {
        let private = &private_share.master_key.private;
        let private_obj: Value =
            serde_json::from_str(serde_json::to_string(private).unwrap().as_str()).unwrap();
        let x = private_obj["x2"].as_str().unwrap();
        x.to_string()
    }

    #[test]
    fn test_rotate() {
        let wallet_file: &str = "test-assets/rotate_w.json";
        let mut w: Wallet = Wallet::load_from(wallet_file);
        let private_share = w.private_share;
        let client_shim = mock_client_shim("ROTATE_TEST_MAIL", "ROTATE_TEST_PW");
        let (old_x1, old_x2) = get_coordinates_of_2_mk(&private_share);
        let old_paillier_x = get_coordinate_of_p2_private(&private_share);

        // Act
        let rotated_private_share = rotate_private_share(private_share, &client_shim).unwrap();
        let (new_x1, new_x2) = get_coordinates_of_2_mk(&rotated_private_share);
        let new_paillier_x = get_coordinate_of_p2_private(&rotated_private_share);

        assert_ne!(new_x1, old_x1);
        assert_ne!(new_x2, old_x2);
        assert_ne!(old_paillier_x, new_paillier_x);
        w.private_share = rotated_private_share;
        w.save_to(wallet_file);
    }
}

#[cfg(test)]
mod btc_test_suite {

    use crate::{
        btc::{
            raw_tx::select_tx_in,
            utils::{get_all_addresses, get_bitcoin_network, get_new_address, BTC_TESTNET},
        },
        dto::ecdsa::PrivateShare,
        tests::common::{
            get_test_private_share, mock_client_shim, print_balance, print_tx_hash,
            PRIVATE_SHARE_FILENAME,
        },
        utilities::derive_new_key,
        wallet::Wallet,
    };
    use anyhow::Result;
    use bitcoin::Network;
    use curv::elliptic::curves::traits::ECPoint;
    const SENT_BTC: f64 = 0.000001; // 100 satoshi
    #[test]
    fn test_get_bitcoin_network() -> Result<()> {
        let network = get_bitcoin_network(BTC_TESTNET)?;
        assert_eq!(network, Network::Testnet);
        Ok(())
    }

    #[test]
    fn test_derive_new_key() {
        let private_share: PrivateShare = get_test_private_share(PRIVATE_SHARE_FILENAME);
        let (pos, mk) = derive_new_key(&private_share, 0);
        let pk = mk.public.q.get_element();
        assert!(!pk.to_string().is_empty());
        assert_eq!(pos, 1);
    }

    #[test]
    fn test_get_new_bitcoin_address() -> Result<()> {
        let private_share: PrivateShare = get_test_private_share(PRIVATE_SHARE_FILENAME);
        let addrs = get_new_address(&private_share, 0)?;
        let exp = "tb1qxyjt450heqv4ql8k7rp2qfmd4vrmncaquzw37r".to_string();
        assert_eq!(addrs.to_string(), exp);
        Ok(())
    }

    #[test]
    fn test_get_all_addresses() -> Result<()> {
        let private_share: PrivateShare = get_test_private_share(PRIVATE_SHARE_FILENAME);
        let address_list = get_all_addresses(0, &private_share)?;
        assert!(!address_list.is_empty());
        Ok(())
    }
    #[test]
    fn test_get_all_unspent() -> Result<()> {
        let private_share: PrivateShare = get_test_private_share(PRIVATE_SHARE_FILENAME);
        let tx_ins = select_tx_in(0, &private_share)?;
        let utxo = tx_ins.get(0).unwrap();
        assert!(utxo.value > 0);
        assert!(!utxo.address.is_empty());
        assert!(!utxo.tx_hash.is_empty());
        Ok(())
    }
    #[test]
    fn test_select_tx_in() -> Result<()> {
        let private_share: PrivateShare = get_test_private_share(PRIVATE_SHARE_FILENAME);
        let unspent_list = select_tx_in(0, &private_share)?;
        assert!(!unspent_list.is_empty());
        Ok(())
    }

    #[test]
    fn test_send_btc() {
        // expect the server running
        let wallet_file: &str = "test-assets/btc_w.json";
        let client_shim = mock_client_shim("BTC_TEST_MAIL", "BTC_TEST_PW");
        let mut w: Wallet = Wallet::load_from(wallet_file);
        let unspent_amount = w.get_balance();
        print_balance(unspent_amount);
        if unspent_amount <= 10000 {
            return;
        }
        // Act
        let to_send = SENT_BTC;
        let txid = w.send(
            "",                                           // btc doesn't need to specify from address
            "tb1qrgkqy0yvpyauwj9f4tq7qku8lgfwn4rw3v9ja8", // to address in our btc_w
            to_send,
            &client_shim,
        );
        print_tx_hash(&txid);
        w.save_to(wallet_file);
        assert!(!txid.is_empty());
    }

    #[test]
    fn test_rotate_and_send_btc() {
        // expect the server running
        let w_file: &str = "test-assets/btc_w.json";
        let client_shim = mock_client_shim("BTC_TEST_MAIL", "BTC_TEST_PW");
        let mut w: Wallet = Wallet::load_from(w_file);
        let unspent_amount = w.get_balance();
        print_balance(unspent_amount);
        if unspent_amount <= 10000 {
            return;
        }
        // Act
        w.rotate(&client_shim, w_file);
        let mut new_w = Wallet::load_from(w_file);
        let to_send = SENT_BTC;
        let txid = new_w.send(
            "",                                           // btc doesn't need to specify from address
            "tb1qrgkqy0yvpyauwj9f4tq7qku8lgfwn4rw3v9ja8", // to address in our btc_w
            to_send,
            &client_shim,
        );
        print_tx_hash(&txid);
        new_w.save_to(w_file);
        assert!(!txid.is_empty());
    }
}

#[cfg(test)]
mod eth_test_suite {
    use anyhow::Result;
    use curv::BigInt;
    use web3::types::U256;

    use crate::{
        dto::ecdsa::PrivateShare,
        eth::utils::{
            get_all_addresses, get_all_addresses_balance, pubkey_to_eth_address, wei_to_eth,
        },
        tests::common::{
            get_test_private_share, mock_client_shim, print_balance, print_tx_hash,
            PRIVATE_SHARE_FILENAME, RINKEBY_TEST_API,
        },
        wallet::Wallet,
    };
    const SENT_ETH: f64 = 0.001; // 1_000_000_000_000_000 wei
    #[test]
    fn test_pubkey_to_eth_address() -> Result<()> {
        let private_share: PrivateShare = get_test_private_share(PRIVATE_SHARE_FILENAME);
        let mk = private_share
            .master_key
            .get_child(vec![BigInt::from(0), BigInt::from(1)]);

        let addrs = pubkey_to_eth_address(&mk);
        let exp = "0xa83b17156ce2a750e7550d3b00d7968463bd759a".to_string();
        assert_eq!(format!("{:?}", addrs), exp);
        Ok(())
    }

    #[test]
    fn test_get_all_addresses() -> Result<()> {
        let private_share: PrivateShare = get_test_private_share(PRIVATE_SHARE_FILENAME);
        let addrs = get_all_addresses(0, &private_share)?;
        let exp = "0x1737844cc0d63f1bb6ed5c049a843dd7c2ab22b0".to_string();
        assert_eq!(format!("{:?}", addrs.get(0).unwrap()), exp);
        Ok(())
    }

    #[test]
    fn test_wei_to_eth() -> Result<()> {
        let eth_amount = wei_to_eth(U256::from(1_000_000_000_000_000_000_u64));
        assert_eq!(eth_amount, 1.0);
        Ok(())
    }

    #[tokio::test]
    async fn test_get_all_addresses_balance() -> Result<()> {
        let private_share: PrivateShare = get_test_private_share(PRIVATE_SHARE_FILENAME);
        let balance_l = get_all_addresses_balance(RINKEBY_TEST_API, 1, &private_share).await?;
        let mut total = 0.0;
        for b in balance_l {
            total += b
        }

        assert!(total > 0.0);
        Ok(())
    }

    #[test]
    fn test_send_eth() {
        // expect the server running
        let client_shim = mock_client_shim("ETH_TEST_MAIL", "ETH_TEST_PW");
        let mut w: Wallet = Wallet::load_from("test-assets/eth_w.json");
        print_balance(w.get_balance());
        if w.get_balance() == 0 {
            return;
        }

        // Act
        let to_send = SENT_ETH;
        let txid = w.send(
            "0xd4606c1470580e6ded4b3a8a983f24ca86ca12ad", // from address in eth_w
            "0xc3e8a75c0f162b2243c15095cd27a8f2c109e7aa", // to address in eth_w
            to_send,
            &client_shim,
        );
        print_tx_hash(&txid);
        assert!(!txid.is_empty());
    }

    #[test]
    fn test_rotate_and_send_eth() {
        // expect the server running
        let client_shim = mock_client_shim("ETH_TEST_MAIL", "ETH_TEST_PW");
        let w_file = "test-assets/eth_w.json";
        let mut w: Wallet = Wallet::load_from(w_file);
        print_balance(w.get_balance());
        if w.get_balance() == 0 {
            return;
        }

        // Act
        w.rotate(&client_shim, w_file);
        let mut new_w = Wallet::load_from(w_file);
        let to_send = SENT_ETH;
        let txid = new_w.send(
            "0xd4606c1470580e6ded4b3a8a983f24ca86ca12ad", // from address in eth_w
            "0xc3e8a75c0f162b2243c15095cd27a8f2c109e7aa", // to address in eth_w
            to_send,
            &client_shim,
        );
        print_tx_hash(&txid);
        assert!(!txid.is_empty());
    }
}
