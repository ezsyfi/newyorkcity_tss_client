use crate::ecdsa::PrivateShare;
use std::fs;

pub const RINKEBY_TEST_API: &str =
    "wss://eth-rinkeby.alchemyapi.io/v2/UmSDyVix3dL4CtIxC2zlKkSuk2UoRw1J";

pub fn get_test_private_share() -> PrivateShare {
    const PRIVATE_SHARE_FILENAME: &str = "test-assets/private_share.json";
    let data =
        fs::read_to_string(PRIVATE_SHARE_FILENAME).expect("Unable to load test private_share!");
    serde_json::from_str(&data).unwrap()
}
