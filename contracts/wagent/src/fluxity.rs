use soroban_sdk::Env;

use crate::storage::get_fluxity_address;

pub mod fluxity_contract {
    use soroban_sdk::contractimport;

    contractimport!(file = "../../fluxity_v1_core.wasm");
}

pub fn fluxity_contract(e: &Env) -> fluxity_contract::Client {
    let fluxity_address = get_fluxity_address(e).unwrap();

    fluxity_contract::Client::new(e, &fluxity_address)
}
