use serde_json::json;

use crate::ecdsa::PrivateShare;
use std::fs;

pub const TEST_WALLET_FILENAME: &str = "test-assets/wallet.json";
pub const RINKEBY_TEST_API: &str =
    "wss://eth-rinkeby.alchemyapi.io/v2/UmSDyVix3dL4CtIxC2zlKkSuk2UoRw1J";

#[derive(Debug, Deserialize)]
#[allow(dead_code, non_snake_case)]
struct AuthToken {
    StatusCode: u16,
    Msg: String,
}

pub struct MockToken {
    pub token: String,
    pub user_id: String,
}

pub fn get_test_private_share() -> PrivateShare {
    const PRIVATE_SHARE_FILENAME: &str = "test-assets/private_share.json";
    let data =
        fs::read_to_string(PRIVATE_SHARE_FILENAME).expect("Unable to load test private_share!");
    serde_json::from_str(&data).unwrap()
}

pub fn mock_sign_in(email: &str, password: &str, signin_url: &str) -> MockToken {
    let http_client = reqwest::blocking::Client::new();
    let auth_body = json!({
        "email": email,
        "password": password
    });
    let http_resp = http_client
        .post(signin_url)
        .json(&auth_body)
        .send()
        .unwrap()
        .json::<AuthToken>()
        .unwrap();

    println!("{:#?}", http_resp);

    MockToken {
        token: http_resp.Msg,
        user_id: email.to_owned(),
    }
}
