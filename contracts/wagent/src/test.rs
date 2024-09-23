#![cfg(test)]

use super::*;
use fluxity::fluxity_contract;
use soroban_sdk::{
    testutils::{Address as _, Events},
    token::{Client, StellarAssetClient},
    Address, Env, IntoVal, String,
};

struct SetupValues<'a> {
    env: Env,
    amount: i128,
    user: Address,
    admin: Address,
    xlm: Client<'a>,
    token: Client<'a>,
    wagent: WagentClient<'a>,
    fluxity: fluxity_contract::Client<'a>,
}

fn setup(should_initialize: bool) -> SetupValues<'static> {
    let env = Env::default();

    env.mock_all_auths();

    let amount = 1000000000000_i128; // 100_000 TOKEN

    let admin = Address::generate(&env);
    let user = Address::generate(&env);

    let fluxity_id = env.register_contract_wasm(None, fluxity_contract::WASM);
    let fluxity_client = fluxity_contract::Client::new(&env, &fluxity_id);

    let contract_id = env.register_contract(None, Wagent);
    let contract_client = WagentClient::new(&env, &contract_id);

    let token_id = env.register_stellar_asset_contract_v2(admin.clone());
    let token_client = Client::new(&env, &token_id.address());
    let token_admin_client = StellarAssetClient::new(&env, &token_id.address());

    let xlm_id = env.register_stellar_asset_contract_v2(admin.clone());
    let xlm_client = Client::new(&env, &xlm_id.address());
    let xlm_admin_client = StellarAssetClient::new(&env, &xlm_id.address());

    xlm_admin_client.mint(&user, &amount);
    xlm_admin_client.mint(&admin, &amount);
    token_admin_client.mint(&user, &amount);
    token_admin_client.mint(&admin, &amount);

    fluxity_client.initialize(&admin, &xlm_id.address());

    if should_initialize {
        contract_client.initialize(&admin, &fluxity_id);
    }

    SetupValues {
        env,
        user,
        admin,
        amount,
        xlm: xlm_client,
        token: token_client,
        fluxity: fluxity_client,
        wagent: contract_client,
    }
}

#[test]
fn test_initialize_contract() {
    let vars = setup(false);

    vars.wagent.initialize(&vars.admin, &vars.fluxity.address);

    assert_eq!(vars.wagent.fluxity_address(), vars.fluxity.address);
}

#[test]
fn test_initialize_contract_emits_event() {
    let vars = setup(false);

    vars.wagent.initialize(&vars.admin, &vars.fluxity.address);

    let events = vars.env.events().all();
    let event = events.last().unwrap();

    assert_eq!(event.0, vars.wagent.address);
    assert_eq!(
        event.1,
        (symbol_short!("CONTRACT"), symbol_short!("INIT")).into_val(&vars.env)
    );
}

#[test]
fn test_initialize_contract_sets_fluxity_address() {
    let vars = setup(false);

    vars.wagent.initialize(&vars.admin, &vars.fluxity.address);

    assert_eq!(vars.wagent.fluxity_address(), vars.fluxity.address);
}

#[test]
fn test_initialize_contract_called_twice_reverts() {
    let vars = setup(false);

    vars.wagent.initialize(&vars.admin, &vars.fluxity.address);

    assert_eq!(
        vars.wagent
            .try_initialize(&vars.admin, &vars.fluxity.address),
        Err(Ok(WagentErrors::ContractAlreadyInitialized))
    );
}

#[test]
fn test_initialize_contract_sets_admin() {
    let vars = setup(false);

    vars.wagent.initialize(&vars.admin, &vars.fluxity.address);

    assert_eq!(vars.wagent.admin(), vars.admin);
}

#[test]
fn test_uninitialized_contract_throws_error_if_admin_is_called() {
    let vars = setup(false);

    assert_eq!(
        vars.wagent.try_admin(),
        Err(Ok(WagentErrors::ContractNotInitialized))
    );
}

#[test]
fn test_pay() {
    let vars = setup(true);

    let amount: i128 = 1_0000000;

    vars.token
        .approve(&vars.user, &vars.wagent.address, &amount, &6311000);

    let user_balance_before = vars.token.balance(&vars.user);
    let admin_balance_before = vars.token.balance(&vars.admin);

    vars.wagent.pay(
        &vars.token.address,
        &vars.user,
        &vars.admin,
        &amount,
        &String::from_str(&vars.env, "66aea870d40479b0f9624a44"),
    );

    let user_balance_after = vars.token.balance(&vars.user);
    let admin_balance_after = vars.token.balance(&vars.admin);

    assert_eq!(user_balance_before, user_balance_after + amount);
    assert_eq!(admin_balance_before, admin_balance_after - amount);
}

#[test]
fn test_pay_when_contract_is_not_initialized_reverts() {
    let vars = setup(false);

    let amount: i128 = 1_0000000;

    vars.token
        .approve(&vars.user, &vars.wagent.address, &amount, &6311000);

    let result = vars.wagent.try_pay(
        &vars.token.address,
        &vars.user,
        &vars.admin,
        &amount,
        &String::from_str(&vars.env, "fkfhdkfh"),
    );

    assert_eq!(result, Err(Ok(WagentErrors::ContractNotInitialized)));
}

#[test]
fn test_pay_should_set_order() {
    let vars = setup(true);

    let amount: i128 = 1_0000000;
    let order_id = String::from_str(&vars.env, "fkfhdkfh");

    vars.token
        .approve(&vars.user, &vars.wagent.address, &amount, &6311000);

    vars.wagent.pay(
        &vars.token.address,
        &vars.user,
        &vars.admin,
        &amount,
        &order_id,
    );

    let order = vars.wagent.order(&order_id);

    assert_eq!(order.id, order_id);
    assert_eq!(order.amount, amount);
    assert_eq!(order.lockup_id, None);
    assert_eq!(order.is_lockup, false);
    assert_eq!(order.sender, vars.user);
    assert_eq!(order.receiver, vars.admin);
    assert_eq!(order.token, vars.token.address);
    assert_eq!(order.submit_date, vars.env.ledger().timestamp());
}

#[test]
fn test_pay_twice_with_the_same_order_id_reverts() {
    let vars = setup(true);

    let amount: i128 = 1_0000000;
    let order_id = String::from_str(&vars.env, "fkfhdkfh");

    vars.token
        .approve(&vars.user, &vars.wagent.address, &amount, &6311000);

    vars.wagent.pay(
        &vars.token.address,
        &vars.user,
        &vars.admin,
        &amount,
        &order_id,
    );

    let result = vars.wagent.try_pay(
        &vars.token.address,
        &vars.user,
        &vars.admin,
        &amount,
        &order_id,
    );

    assert_eq!(result, Err(Ok(WagentErrors::OrderAlreadyExists)));
}

#[test]
fn test_pay_emits_event() {
    let vars = setup(true);

    let amount: i128 = 1_0000000;
    let order_id = String::from_str(&vars.env, "fkfhdkfh");

    vars.token
        .approve(&vars.user, &vars.wagent.address, &amount, &6311000);

    vars.wagent.pay(
        &vars.token.address,
        &vars.user,
        &vars.admin,
        &amount,
        &order_id,
    );

    let events = vars.env.events().all();
    let event = events.last().unwrap();

    assert_eq!(event.0, vars.wagent.address);
    assert_eq!(
        event.1,
        (symbol_short!("PAY"), symbol_short!("DIRECT")).into_val(&vars.env)
    );
}

#[test]
fn test_pay_stream() {
    let vars = setup(true);

    let amount: i128 = 1_0000000;
    let order_id = String::from_str(&vars.env, "fkfhdkfh");

    vars.token
        .approve(&vars.user, &vars.wagent.address, &amount, &6311000);

    let user_balance_before = vars.token.balance(&vars.user);
    let admin_balance_before = vars.token.balance(&vars.admin);

    vars.wagent.pay_stream(
        &vars.token.address,
        &vars.user,
        &vars.admin,
        &amount,
        &order_id,
        &1000,
        &0,
    );

    let user_balance_after = vars.token.balance(&vars.user);
    let admin_balance_after = vars.token.balance(&vars.admin);

    assert_eq!(admin_balance_before, admin_balance_after);
    assert_eq!(user_balance_before, user_balance_after + amount);
}

#[test]
fn test_pay_stream_sets_order() {
    let vars = setup(true);

    let amount: i128 = 1_0000000;
    let order_id = String::from_str(&vars.env, "fkfhdkfh");

    vars.token
        .approve(&vars.user, &vars.wagent.address, &amount, &6311000);

    let id = vars.wagent.pay_stream(
        &vars.token.address,
        &vars.user,
        &vars.admin,
        &amount,
        &order_id,
        &1000,
        &0,
    );

    let order = vars.wagent.order(&order_id);

    assert_eq!(order.id, order_id);
    assert_eq!(order.amount, amount);
    assert_eq!(order.lockup_id, Some(id));
    assert_eq!(order.is_lockup, true);
    assert_eq!(order.sender, vars.user);
    assert_eq!(order.receiver, vars.admin);
    assert_eq!(order.token, vars.token.address);
    assert_eq!(order.submit_date, vars.env.ledger().timestamp());
}

#[test]
fn test_pay_stream_emits_event() {
    let vars = setup(true);

    let amount: i128 = 1_0000000;
    let order_id = String::from_str(&vars.env, "fkfhdkfh");

    vars.token
        .approve(&vars.user, &vars.wagent.address, &amount, &6311000);

    vars.wagent.pay_stream(
        &vars.token.address,
        &vars.user,
        &vars.admin,
        &amount,
        &order_id,
        &1000,
        &0,
    );

    let events = vars.env.events().all();
    let event = events.last().unwrap();

    assert_eq!(event.0, vars.wagent.address);
    assert_eq!(
        event.1,
        (symbol_short!("PAY"), symbol_short!("STREAM")).into_val(&vars.env)
    );
}

// TODO: fee tests (when fluxity's monthly fee is set)
// TODO: admin fee (implementation and testing)
// TODO: fluxity integration tests
