#![no_std]
mod data_key;
mod error;
mod extend;
mod fluxity;
mod interface;
mod storage;
mod test;

use data_key::Order;
use error::WagentErrors;
use extend::extend_contract_ttl;
use fluxity::{
    fluxity_contract,
    fluxity_contract::{LockupInput, Rate},
};
use interface::IWagent;
use soroban_sdk::{contract, contractimpl, symbol_short, token::Client, Address, Env, String};
use storage::{
    get_admin, get_fluxity_address, get_order, initialize_contract, is_contract_initialized,
    set_admin, set_fluxity_address, set_order,
};

#[contract]
pub struct Wagent;

#[contractimpl]
impl IWagent for Wagent {
    ///
    /// Initializes the contract and sets admin and fluxity contract address
    ///
    fn initialize(e: Env, admin: Address, fluxity_address: Address) -> Result<(), WagentErrors> {
        if is_contract_initialized(&e) {
            return Err(WagentErrors::ContractAlreadyInitialized);
        }

        initialize_contract(&e);
        set_fluxity_address(&e, fluxity_address);
        set_admin(&e, admin.clone());

        e.events()
            .publish((symbol_short!("CONTRACT"), symbol_short!("INIT")), admin);

        Ok(())
    }

    ///
    /// Returns the fluxity_contract address
    ///
    /// Throws if the contract is not initialized
    ///
    fn fluxity_address(e: Env) -> Result<Address, WagentErrors> {
        match get_fluxity_address(&e) {
            Some(x) => Ok(x),
            None => Err(WagentErrors::ContractNotInitialized),
        }
    }

    ///
    /// Returns the admin address
    ///
    /// Throws if the contract is not initialized
    ///
    fn admin(e: Env) -> Result<Address, WagentErrors> {
        match get_admin(&e) {
            Some(x) => Ok(x),
            None => Err(WagentErrors::ContractNotInitialized),
        }
    }

    ///
    /// Returns an order details
    ///
    /// Throws if the order is not found
    ///
    fn order(e: Env, order_id: String) -> Result<Order, WagentErrors> {
        match get_order(&e, order_id) {
            Some(x) => Ok(x),
            None => Err(WagentErrors::OrderNotFound),
        }
    }

    ///
    /// Creates an order and pays the receiver
    ///
    /// Needs approval from the sender
    ///
    /// Throws if the contract is not initialized or an order with the same id exists
    ///
    fn pay(
        e: Env,
        token_address: Address,
        sender: Address,
        receiver: Address,
        amount: i128,
        order_id: String,
    ) -> Result<(), WagentErrors> {
        if !is_contract_initialized(&e) {
            return Err(WagentErrors::ContractNotInitialized);
        }

        if get_order(&e, order_id.clone()).is_some() {
            return Err(WagentErrors::OrderAlreadyExists);
        }

        let token = Client::new(&e, &token_address);

        token.transfer_from(
            &e.current_contract_address(),
            &sender,
            &e.current_contract_address(),
            &amount,
        );

        token.transfer(&e.current_contract_address(), &receiver, &amount);

        let order = Order {
            amount,
            sender,
            receiver,
            lockup_id: None,
            is_lockup: false,
            id: order_id.clone(),
            token: token_address,
            submit_date: e.ledger().timestamp(),
        };

        set_order(&e, order);

        extend_contract_ttl(&e);

        e.events()
            .publish((symbol_short!("PAY"), symbol_short!("DIRECT")), order_id);

        Ok(())
    }

    ///
    /// Creates an order and an stream in Fluxity for the sender/receiver
    ///
    /// Throws if the contract is not initialized or an order with the same id exists
    ///
    fn pay_stream(
        e: Env,
        token_address: Address,
        sender: Address,
        receiver: Address,
        amount: i128,
        order_id: String,
        duration: u64,
        min_cancellable_duration: u64,
    ) -> Result<u64, WagentErrors> {
        if !is_contract_initialized(&e) {
            return Err(WagentErrors::ContractNotInitialized);
        }

        if get_order(&e, order_id.clone()).is_some() {
            return Err(WagentErrors::OrderAlreadyExists);
        }

        let fluxity = fluxity_contract(&e);

        let token = Client::new(&e, &token_address);
        let expiration_ledger = e.ledger().timestamp();

        token.transfer_from(
            &e.current_contract_address(),
            &sender,
            &e.current_contract_address(),
            &amount,
        );

        token.approve(
            &e.current_contract_address(),
            &fluxity.address,
            &amount,
            &(expiration_ledger as u32),
        );

        let params = LockupInput {
            amount,
            is_vesting: false,
            rate: Rate::Monthly,
            sender: sender.clone(),
            receiver: receiver.clone(),
            token: token_address.clone(),
            start_date: expiration_ledger,
            cliff_date: expiration_ledger,
            spender: e.current_contract_address(),
            end_date: expiration_ledger + duration,
            cancellable_date: expiration_ledger + min_cancellable_duration,
        };

        extend_contract_ttl(&e);

        let id = fluxity.create_lockup(&params);

        let order = Order {
            amount,
            sender,
            receiver,
            is_lockup: true,
            lockup_id: Some(id),
            id: order_id.clone(),
            token: token_address,
            submit_date: e.ledger().timestamp(),
        };

        set_order(&e, order);

        e.events().publish(
            (symbol_short!("PAY"), symbol_short!("STREAM")),
            order_id.clone(),
        );

        Ok(id)
    }
}
