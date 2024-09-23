pub const DAY_IN_LEDGERS: u32 = 17280;
pub const BUMP_AMOUNT: u32 = 60 * DAY_IN_LEDGERS;
pub const LIFETIME_THRESHOLD: u32 = 30 * DAY_IN_LEDGERS;

use super::data_key::DataKey;
use soroban_sdk::Env;

pub fn extend_data_ttl(e: &Env, key: &DataKey) {
    e.storage()
        .persistent()
        .extend_ttl(key, LIFETIME_THRESHOLD, BUMP_AMOUNT);
}

pub fn extend_contract_ttl(e: &Env) {
    e.storage()
        .instance()
        .extend_ttl(LIFETIME_THRESHOLD, BUMP_AMOUNT);
}
