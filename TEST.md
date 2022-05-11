# Organization
## Test assets
Folder `test-assets` includes JSON files that represent **test wallet** data.

## Test arrange utilities
`src/tests/common.rs`

## Test suites
- ecdsa
- btc
- eth
# Steps to test

## 1. Fill test wallet funds with faucets in case we run out of money
- BTC: https://bitcoinfaucet.uo1.net/send.php, https://testnet-faucet.mempool.co/, https://coinfaucet.eu/en/btc-testnet/
- ETH: https://rinkebyfaucet.com/, https://rinkeby-faucet.com/

## 2. Run test
```bash
cargo test --verbose # all test suites
cargo test --verbose -- --nocapture # all test suites with logs
cargo test --verbose <mod name> -- --nocapture # specific test suite
```

## 3. Check address balance on explorers
to address: where we send money to
from address: where we keep money

BTC: https://live.blockcypher.com/btc-testnet/ 
- the `output addresses` of the transactions must include: 
    - the to (lowest pos) address: `tb1qrgkqy0yvpyauwj9f4tq7qku8lgfwn4rw3v9ja8`
    - the change (highest pos) address which will become the from address later on 

ETH: https://rinkeby.etherscan.io/ 
- the to address: `0xc3e8a75c0f162b2243c15095cd27a8f2c109e7aa`
- the from address: `0xd4606c1470580e6ded4b3a8a983f24ca86ca12ad`

# NOTES:
- Not have test coverage yet
- Not have negative cases yet (Use `should_panic`)
- Not have error-checking cases yet (Assert returned error)
- Not have bench/bounded cases yet (min/max/0/ranging/repeatability)
