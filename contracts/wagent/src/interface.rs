use soroban_sdk::{Address, Env, String};

use crate::{data_key::Order, error::WagentErrors};

pub trait IWagent {
    fn initialize(e: Env, admin: Address, fluxity_address: Address) -> Result<(), WagentErrors>;

    fn fluxity_address(e: Env) -> Result<Address, WagentErrors>;

    fn admin(e: Env) -> Result<Address, WagentErrors>;

    fn order(e: Env, order_id: String) -> Result<Order, WagentErrors>;

    fn pay(
        e: Env,
        token_address: Address,
        sender: Address,
        receiver: Address,
        amount: i128,
        order_id: String,
    ) -> Result<(), WagentErrors>;

    fn pay_stream(
        e: Env,
        token_address: Address,
        sender: Address,
        receiver: Address,
        amount: i128,
        order_id: String,
        duration: u64,
        min_cancellable_duration: u64,
    ) -> Result<u64, WagentErrors>;
}
