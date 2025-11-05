use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Config {
    pub admin: Addr,
    pub minter: Addr,
    pub price: Uint128,
    pub max_supply: u64,
    pub paused: bool,
    pub base_uri: String,
    pub provenance_hash: Option<String>,
    pub royalty_bps: u64,
    pub royalty_receiver: Addr,
}

pub const CONFIG: Item<Config> = Item::new("config");

pub const TOKEN_COUNT: Item<u64> = Item::new("token_count");

// For potential per-token URIs if needed later
pub const TOKEN_URI: Map<u64, String> = Map::new("token_uri");
