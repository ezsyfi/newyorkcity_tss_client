# Steps to test

## 1. Fills wallet fund with faucets
BTC: https://bitcoinfaucet.uo1.net/send.php, https://testnet-faucet.mempool.co/, https://coinfaucet.eu/en/btc-testnet/
ETH: https://rinkebyfaucet.com/, https://rinkeby-faucet.com/

## 2. Run test
```bash
cargo test --verbose # all test suites
cargo test --verbose -- --nocapture # all test suites with logs
cargo test --verbose <mod name> -- --nocapture # specific test suite
```

## 3. Check address balance on explorers
BTC: https://live.blockcypher.com/btc-testnet/ 
- the `output addresses` of this TX must include the `lowest pos address` (to address) & `highest pos address` (change address) of our `test wallet` (btc_w.json) 

ETH: https://rinkeby.etherscan.io/ 