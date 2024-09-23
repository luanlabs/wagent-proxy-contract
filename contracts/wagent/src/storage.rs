use soroban_sdk::{Address, Env, String};

use crate::{
    data_key::{DataKey, Order},
    extend::{self, extend_data_ttl},
};

pub fn set_fluxity_address(e: &Env, fluxity_address: Address) {
    e.storage()
        .instance()
        .set(&DataKey::FluxityAddress, &fluxity_address);

    extend::extend_contract_ttl(e);
}

pub fn get_fluxity_address(e: &Env) -> Option<Address> {
    e.storage().instance().get(&DataKey::FluxityAddress)
}

pub fn initialize_contract(e: &Env) {
    e.storage().instance().set(&DataKey::IsInitialized, &true);

    extend::extend_contract_ttl(e);
}

pub fn is_contract_initialized(e: &Env) -> bool {
    e.storage()
        .instance()
        .get(&DataKey::IsInitialized)
        .unwrap_or(false)
}

pub fn set_admin(e: &Env, admin: Address) {
    e.storage().instance().set(&DataKey::Admin, &admin);
}

pub fn get_admin(e: &Env) -> Option<Address> {
    e.storage().instance().get(&DataKey::Admin)
}

pub fn set_order(e: &Env, order: Order) {
    e.storage()
        .persistent()
        .set(&DataKey::Order(order.id.clone()), &order);

    extend_data_ttl(e, &DataKey::Order(order.id.clone()));
}

pub fn get_order(e: &Env, order_id: String) -> Option<Order> {
    e.storage().persistent().get(&DataKey::Order(order_id))
}
