#[cfg(test)]
mod btc_test_suite {

    use crate::{
        btc::{
            raw_tx::select_tx_in,
            utils::{
                get_all_addresses, get_bitcoin_network, get_new_address,
                BTC_TESTNET,
            },
        },
        dto::ecdsa::PrivateShare,
        utilities::{derive_new_key, tests::{get_test_private_share, mock_client_shim, BTC_TEST_WALLET_FILE}}, wallet::Wallet,
    };
    use anyhow::Result;
    use bitcoin::Network;
    use curv::elliptic::curves::traits::ECPoint;

    #[test]
    fn test_get_bitcoin_network() -> Result<()> {
        let network = get_bitcoin_network(BTC_TESTNET)?;
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
        let addrs = get_new_address(&private_share, 0)?;
        let exp = "tb1qxyjt450heqv4ql8k7rp2qfmd4vrmncaquzw37r".to_string();
        assert_eq!(addrs.to_string(), exp);
        Ok(())
    }

    #[test]
    fn test_get_all_addresses() -> Result<()> {
        let private_share: PrivateShare = get_test_private_share();
        let address_list = get_all_addresses(0, &private_share)?;
        assert!(!address_list.is_empty());
        Ok(())
    }
    #[test]
    fn test_get_all_unspent() -> Result<()> {
        let private_share: PrivateShare = get_test_private_share();
        let tx_ins = select_tx_in(0, &private_share)?;
        let utxo = tx_ins.get(0).unwrap();
        assert!(utxo.value > 0);
        assert!(!utxo.address.is_empty());
        assert!(!utxo.tx_hash.is_empty());
        Ok(())
    }
    #[test]
    fn test_select_tx_in() -> Result<()> {
        let private_share: PrivateShare = get_test_private_share();
        let unspent_list = select_tx_in(0, &private_share)?;
        assert!(!unspent_list.is_empty());
        Ok(())
    }

    // TODO: To test `send` feature on created account, we need to complete recover feature first.
    //       So maybe we can restore the wallet from the backup file.
    #[test]
    fn send_test() {
        // expect the server running

        let client_shim = mock_client_shim("BTC_TEST_MAIL","BTC_TEST_PW");

        let mut w: Wallet = Wallet::load_from(BTC_TEST_WALLET_FILE);

        let unspent_amount = w.get_balance();
        if unspent_amount <= 10000 {
            return;
        }
        println!("We have enough balance to test {:?}", w.get_balance());
        let to_send = 0.00001; // 1000 satoshi
        let txid = w.send(
            "",
            "tb1qcs5whgs59ywsgert834jhk2mr84sdnv0d84jw8", // to address in our btc_w
            to_send,
            &client_shim,
        );
        assert!(!txid.is_empty());
        w.save_to(BTC_TEST_WALLET_FILE);
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
        utilities::tests::{get_test_private_share, RINKEBY_TEST_API, ETH_TEST_WALLET_FILE, mock_client_shim}, wallet::Wallet,
    };

    #[test]
    fn test_pubkey_to_eth_address() -> Result<()> {
        let private_share: PrivateShare = get_test_private_share();
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
        let private_share: PrivateShare = get_test_private_share();

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
        let private_share: PrivateShare = get_test_private_share();
        let balance_l = get_all_addresses_balance(RINKEBY_TEST_API, 1, &private_share).await?;
        let mut total = 0.0;
        for b in balance_l {
            total += b
        }

        assert!(total > 0.0);
        Ok(())
    }

    #[test]
    fn send_test() {
        // expect the server running
        let client_shim = mock_client_shim("ETH_TEST_MAIL","ETH_TEST_PW");
        let mut w: Wallet = Wallet::load_from(ETH_TEST_WALLET_FILE);
        if w.get_balance() == 0 {
            return;
        }
        println!("We have enough balance to test {:?}", w.get_balance());
        let to_send = 0.001; // 1_000_000_000_000_000 wei
        let txid = w.send(
            "0xd4606c1470580e6ded4b3a8a983f24ca86ca12ad", // from address in eth_w
            "0xc3e8a75c0f162b2243c15095cd27a8f2c109e7aa", // to address in eth_w
            to_send,
            &client_shim,
        );
        assert!(!txid.is_empty());
    }
}

