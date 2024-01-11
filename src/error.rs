use cosmwasm_std::StdError;
use cw_denom::DenomError;
use cw_utils::PaymentError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),
    #[error("{0}")]
    Payment(#[from] PaymentError),
    #[error("{0}")]
    Denom(#[from] DenomError),
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Deal not found")]
    DealNotFound,
    #[error("Duration must be between {0} and {1} seconds")]
    InvalidDuration(u64, u64),
    #[error("Deal is expired")]
    DealExpired,
    #[error("Insufficient Amount expected {0}")]
    InsufficientAmount(String),
}
