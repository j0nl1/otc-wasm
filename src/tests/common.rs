pub use anyhow::Result;
pub use derivative::Derivative;

pub use cosmwasm_std::{coin, Addr, BlockInfo, Coin, Empty, StdResult};
pub use cw_multi_test::{Contract, ContractWrapper};

pub use crate::error::ContractError;
pub use crate::execute::execute;
pub use crate::instantiate::instantiate;
pub use crate::msg::*;
pub use crate::query::query;
pub use crate::state::*;

pub const SELLER: &str = "seller";
pub const BUYER: &str = "buyer";
pub const EXECUTOR: &str = "executor";
pub const DEPLOYER: &str = "deployer";
pub const DENOM_1: &str = "ucosm";
pub const DENOM_2: &str = "ustake";

pub fn contract_otc() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(execute, instantiate, query);
    Box::new(contract)
}
