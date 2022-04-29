pub mod party1;
pub mod party2;

#[derive(Debug, Serialize, Deserialize)]
pub struct Rotation {
    pub rotation: FE,
}
