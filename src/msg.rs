use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;
use cw721::Expiration;

#[cw_serde]
pub struct InstantiateMsg {
    pub base_token_uri: String,
    pub provenance_hash: Option<String>,
}

#[cw_serde]
pub enum ExecuteMsg {
    PublicMint {},
    PublicBatchMint { quantity: u64 },

    Withdraw {},
    PauseMint {},
    UnpauseMint {},
    UpdateBaseUri { base_uri: String },
    SetProvenanceHash { hash: String },

    // Pass-through CW721 execute variants for transfers/approvals
    Approve { spender: String, token_id: String, expires: Option<Expiration> },
    Revoke { spender: String, token_id: String },
    ApproveAll { operator: String, expires: Option<Expiration> },
    RevokeAll { operator: String },
    TransferNft { recipient: String, token_id: String },
    SendNft { contract: String, token_id: String, msg: cosmwasm_std::Binary },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(cw721::OwnerOfResponse)]
    OwnerOf { token_id: String, include_expired: Option<bool> },

    #[returns(cw721::NumTokensResponse)]
    NumTokens {},

    #[returns(cw721::NftInfoResponse)]
    NftInfo { token_id: String },

    #[returns(cw721::TokensResponse)]
    Tokens { owner: String, start_after: Option<String>, limit: Option<u32> },

    #[returns(cw721::ContractInfoResponse)]
    ContractInfo {},

    // CW2981 royalty info
    #[returns(RoyaltiesInfoResponse)]
    RoyaltyInfo { token_id: String, sale_price: Uint128 },

    // custom config
    #[returns(ConfigResponse)]
    GetConfig {},
}

#[cw_serde]
pub struct ConfigResponse {
    pub admin: String,
    pub minter: String,
    pub price: Uint128,
    pub max_supply: u64,
    pub paused: bool,
    pub base_uri: String,
    pub provenance_hash: Option<String>,
    pub royalty_bps: u64,
    pub royalty_receiver: String,
    pub total_minted: u64,
}

// Minimal CW2981-compatible response struct
// Matches cw2981::msg::RoyaltiesInfoResponse fields
#[cw_serde]
pub struct RoyaltiesInfoResponse {
    pub address: String,
    pub royalty_amount: Uint128,
}
