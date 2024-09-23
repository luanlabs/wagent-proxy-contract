use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum WagentErrors {
    ContractNotInitialized = 10,
    ContractAlreadyInitialized = 11,

    OrderNotFound = 20,
    OrderAlreadyExists = 21,
}
