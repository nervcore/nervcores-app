use cosmwasm_std::StdError;
use cw721_base as cw721b;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("Std error: {0}")]
    Std(#[from] StdError),

    #[error("CW721 error: {0}")]
    Cw721(#[from] cw721b::ContractError),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Minting is paused")]
    Paused,

    #[error("Invalid payment amount")]
    InvalidPayment,

    #[error("Max supply exceeded")]
    MaxSupply,

    #[error("Quantity out of bounds")]
    InvalidQuantity,
}
