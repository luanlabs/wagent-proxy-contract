use soroban_sdk::{contracttype, Address, String};

#[contracttype]
#[derive(Clone, Debug)]
pub struct Order {
    pub id: String,
    pub amount: i128,
    pub token: Address,
    pub sender: Address,
    pub is_lockup: bool,
    pub submit_date: u64,
    pub receiver: Address,
    pub lockup_id: Option<u64>,
}

#[contracttype]
#[derive(Clone, Debug)]
pub enum DataKey {
    Admin,
    Order(String),
    IsInitialized,
    FluxityAddress,
}
