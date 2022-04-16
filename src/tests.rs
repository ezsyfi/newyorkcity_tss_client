#[cfg(test)]
mod btc_test_suite {

    use crate::{
        btc::{
            raw_tx::select_tx_in,
            utils::{
                get_all_addresses, get_all_addresses_balance, get_bitcoin_network, get_new_address,
                BTC_TESTNET,
            },
        },
        ecdsa::PrivateShare,
        utilities::{derive_new_key, tests::get_test_private_share},
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
    fn test_get_all_addresses_balance() -> Result<()> {
        let private_share: PrivateShare = get_test_private_share();
        let address_balance_list = get_all_addresses_balance(0, &private_share)?;
        assert!(!address_balance_list.is_empty());

        let address_balance = address_balance_list.get(0).unwrap();
        // assert_eq!(address_balance.confirmed, 0);
        assert!(address_balance.confirmed > 0);
        assert_eq!(address_balance.unconfirmed, 0);
        assert_eq!(
            address_balance.address,
            "tb1qkr66k03t0d0ep8kmkl0zg8du45y2mfer0pflh5"
        );
        Ok(())
    }

    #[test]
    fn test_select_tx_in() -> Result<()> {
        let private_share: PrivateShare = get_test_private_share();
        let unspent_list = select_tx_in(0.0, 0, &private_share)?;
        assert!(!unspent_list.is_empty());
        Ok(())
    }

    // TODO: Find a reliable way of doing integration testing over the blockchain.
    // TODO: Ideally we would like to do the whole flow of receiving and sending. PR welcome ;)
    //    #[test]
    //    fn send_test() {
    //        // expect the server running
    //        let client_shim : api::ClientShim = api::ClientShim::new(
    //            "http://localhost:8001".to_string(), None);

    //        let  mut w : Wallet = Wallet::load_from(TEST_WALLET_FILENAME);
    //        let b = w.get_balance();
    //        assert!(b.confirmed > 0);

    //        let available_balance = b.confirmed as f32 / 100000000 as f32;
    //        let to_send = 0.00000001;
    //        let delta_pessimistic_fees = 0.00013; // 0.5 usd - 03/14/2019
    //        assert!(available_balance > to_send + delta_pessimistic_fees, "You need to refund the wallet");

    //        let to_address = w.get_new_address(); // inner wallet tx
    //        let txid = w.send(to_address.to_string(), to_send, &client_shim);
    //        assert!(!txid.is_empty());
    //    }
}

#[cfg(test)]
mod eth_test_suite {
    use anyhow::Result;
    use curv::BigInt;
    use web3::types::U256;

    use crate::{
        ecdsa::PrivateShare,
        eth::utils::{
            get_all_addresses, get_all_addresses_balance, pubkey_to_eth_address, wei_to_eth,
        },
        utilities::tests::{get_test_private_share, RINKEBY_TEST_API},
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
}
