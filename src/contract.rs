use cosmwasm_std::{
    entry_point, to_json_binary, BankMsg, Binary, Coin, Deps, DepsMut, Env, MessageInfo,
    QueryRequest, Response, StdResult, Uint128, Empty,
};
use cw2::set_contract_version;
use cw721_base::{Cw721Contract, InstantiateMsg as Cw721InstantiateMsg};
use cw721::{Cw721Execute, Cw721Query};
use crate::msg::RoyaltiesInfoResponse;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{CONFIG, Config, TOKEN_COUNT};

const CONTRACT_NAME: &str = "paxi-pioneers";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

// Constants per spec
const TOKEN_NAME: &str = "Paxi Pioneers";
const TOKEN_SYMBOL: &str = "PIONEER";
const PRICE_UPAXI: u128 = 10_000_000; // 10 PAXI = 10_000_000 upaxi
const MAX_SUPPLY: u64 = 10_000;
const ROYALTY_BPS: u64 = 750; // 7.5%

// cw721-base contract type (Empty custom msg types)
pub type Base<'a> = Cw721Contract<'a, Empty, Empty, Empty, Empty>;

#[entry_point]
pub fn instantiate(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // instantiate cw721-base
    let cw721_msg = Cw721InstantiateMsg {
        name: TOKEN_NAME.to_string(),
        symbol: TOKEN_SYMBOL.to_string(),
        minter: env.contract.address.to_string(),
    };

    let base: Base<'static> = Base::default();
    base.instantiate(deps.branch(), env.clone(), info.clone(), cw721_msg)?;

    let admin = deps.api.addr_validate(info.sender.as_str())?;
    let minter = deps.api.addr_validate(env.contract.address.as_str())?;

    let cfg = Config {
        admin: admin.clone(),
        minter: minter.clone(),
        price: Uint128::from(PRICE_UPAXI),
        max_supply: MAX_SUPPLY,
        paused: true,
        base_uri: msg.base_token_uri,
        provenance_hash: msg.provenance_hash,
        royalty_bps: ROYALTY_BPS,
        royalty_receiver: admin.clone(),
    };
    CONFIG.save(deps.storage, &cfg)?;
    TOKEN_COUNT.save(deps.storage, &0u64)?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("admin", admin)
        .add_attribute("minter", minter))
}

fn ensure_admin(deps: Deps, info: &MessageInfo) -> Result<(), ContractError> {
    let cfg = CONFIG.load(deps.storage)?;
    if info.sender != cfg.admin { return Err(ContractError::Unauthorized); }
    Ok(())
}

fn must_active(deps: Deps) -> Result<(), ContractError> {
    let cfg = CONFIG.load(deps.storage)?;
    if cfg.paused { return Err(ContractError::Paused); }
    Ok(())
}

#[entry_point]
pub fn execute(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::PublicMint {} => mint_internal(deps, env, info, 1),
        ExecuteMsg::PublicBatchMint { quantity } => mint_internal(deps, env, info, quantity),
        ExecuteMsg::Withdraw {} => withdraw(deps, info),
        ExecuteMsg::PauseMint {} => pause(deps, info),
        ExecuteMsg::UnpauseMint {} => unpause(deps, info),
        ExecuteMsg::UpdateBaseUri { base_uri } => update_base_uri(deps, info, base_uri),
        ExecuteMsg::SetProvenanceHash { hash } => set_provenance(deps, info, hash),

        // pass-throughs to cw721-base
        ExecuteMsg::Approve { spender, token_id, expires } => {
            let base: Base<'static> = Base::default();
            let r = base.approve(deps, env, info, spender, token_id, expires)?;
            Ok(r.into())
        }
        ExecuteMsg::Revoke { spender, token_id } => {
            let base: Base<'static> = Base::default();
            let r = base.revoke(deps, env, info, spender, token_id)?;
            Ok(r.into())
        }
        ExecuteMsg::ApproveAll { operator, expires } => {
            let base: Base<'static> = Base::default();
            let r = base.approve_all(deps, env, info, operator, expires)?;
            Ok(r.into())
        }
        ExecuteMsg::RevokeAll { operator } => {
            let base: Base<'static> = Base::default();
            let r = base.revoke_all(deps, env, info, operator)?;
            Ok(r.into())
        }
        ExecuteMsg::TransferNft { recipient, token_id } => {
            let base: Base<'static> = Base::default();
            let r = base.transfer_nft(deps, env, info, recipient, token_id)?;
            Ok(r.into())
        }
        ExecuteMsg::SendNft { contract, token_id, msg } => {
            let base: Base<'static> = Base::default();
            let r = base.send_nft(deps, env, info, contract, token_id, msg)?;
            Ok(r.into())
        }
    }
}

fn get_sent_upaxi(info: &MessageInfo) -> Uint128 {
    info.funds
        .iter()
        .find(|c| c.denom == "upaxi")
        .map(|c| c.amount)
        .unwrap_or_else(Uint128::zero)
}

fn mint_internal(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    quantity: u64,
) -> Result<Response, ContractError> {
    if quantity == 0 || quantity > 10 { return Err(ContractError::InvalidQuantity); }
    must_active(deps.as_ref())?;

    let mut count = TOKEN_COUNT.load(deps.storage)?;
    let cfg = CONFIG.load(deps.storage)?;

    // supply checks
    if count + quantity > cfg.max_supply { return Err(ContractError::MaxSupply); }

    // payment checks
    let required = cfg.price.checked_mul(Uint128::from(quantity as u128)).unwrap();
    let paid = get_sent_upaxi(&info);
    if paid != required { return Err(ContractError::InvalidPayment); }

    let base: Base<'static> = Base::default();

    let mut resp = Response::new().add_attribute("action", "public_mint");
    for _ in 0..quantity {
        count += 1;
        let token_id = count.to_string();
        let token_uri = format!("{}{}", cfg.base_uri, token_id);
        // Mint must be executed by the contract-as-minter (owner via cw_ownable)
        let minter_info = MessageInfo { sender: env.contract.address.clone(), funds: vec![] };
        base.mint(
            deps.branch(),
            minter_info,
            token_id.clone(),
            info.sender.to_string(),
            Some(token_uri),
            Empty::default(),
        )?;
    }

    TOKEN_COUNT.save(deps.storage, &count)?;

    Ok(resp.add_attribute("quantity", quantity.to_string()))
}

fn withdraw(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
    ensure_admin(deps.as_ref(), &info)?;
    let cfg = CONFIG.load(deps.storage)?;

    // Query contract's own balance of upaxi
    let bal: cosmwasm_std::BalanceResponse = deps.querier.query(&QueryRequest::Bank(
        cosmwasm_std::BankQuery::Balance { address: env_address(deps.as_ref())?, denom: "upaxi".to_string() }
    ))?;
    let balance = bal.amount.amount;

    let msg = BankMsg::Send {
        to_address: cfg.admin.to_string(),
        amount: vec![Coin { denom: "upaxi".to_string(), amount: balance }],
    };
    Ok(Response::new().add_message(msg).add_attribute("action", "withdraw"))
}

// Helper to get contract address string
fn env_address(deps: Deps) -> StdResult<String> {
    // No Env is passed here, but we can’t access env from state; instead we rely on cfg.minter which is set to contract address
    let cfg = CONFIG.load(deps.storage)?;
    Ok(cfg.minter.to_string())
}

fn pause(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
    ensure_admin(deps.as_ref(), &info)?;
    CONFIG.update(deps.storage, |mut c| -> StdResult<_> { c.paused = true; Ok(c) })?;
    Ok(Response::new().add_attribute("action", "pause"))
}

fn unpause(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
    ensure_admin(deps.as_ref(), &info)?;
    CONFIG.update(deps.storage, |mut c| -> StdResult<_> { c.paused = false; Ok(c) })?;
    Ok(Response::new().add_attribute("action", "unpause"))
}

fn update_base_uri(deps: DepsMut, info: MessageInfo, base_uri: String) -> Result<Response, ContractError> {
    ensure_admin(deps.as_ref(), &info)?;
    CONFIG.update(deps.storage, |mut c| -> StdResult<_> { c.base_uri = base_uri; Ok(c) })?;
    Ok(Response::new().add_attribute("action", "update_base_uri"))
}

fn set_provenance(deps: DepsMut, info: MessageInfo, hash: String) -> Result<Response, ContractError> {
    ensure_admin(deps.as_ref(), &info)?;
    CONFIG.update(deps.storage, |mut c| -> StdResult<_> { c.provenance_hash = Some(hash); Ok(c) })?;
    Ok(Response::new().add_attribute("action", "set_provenance"))
}

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::OwnerOf { token_id, include_expired } => {
            let base: Base<'static> = Base::default();
            to_json_binary(&base.owner_of(deps, env, token_id, include_expired.unwrap_or(false))?)
        }
        QueryMsg::NumTokens {} => {
            let base: Base<'static> = Base::default();
            to_json_binary(&base.num_tokens(deps)?)
        }
        QueryMsg::NftInfo { token_id } => {
            let base: Base<'static> = Base::default();
            to_json_binary(&base.nft_info(deps, token_id)?)
        }
        QueryMsg::Tokens { owner, start_after, limit } => {
            let base: Base<'static> = Base::default();
            to_json_binary(&base.tokens(deps, owner, start_after, limit)?)
        }
        QueryMsg::ContractInfo {} => {
            let base: Base<'static> = Base::default();
            to_json_binary(&base.contract_info(deps)?)
        }
        QueryMsg::RoyaltyInfo { token_id, sale_price } => {
            // Implement simple CW2981-like computation based on global config
            let cfg = CONFIG.load(deps.storage)?;
            let amount = sale_price.multiply_ratio(cfg.royalty_bps as u128, 10_000u128);
            let resp = RoyaltiesInfoResponse {
                address: cfg.royalty_receiver.to_string(),
                royalty_amount: amount,
            };
            to_json_binary(&resp)
        }
        QueryMsg::GetConfig {} => {
            let cfg = CONFIG.load(deps.storage)?;
            let total_minted = TOKEN_COUNT.load(deps.storage)?;
            to_json_binary(&crate::msg::ConfigResponse {
                admin: cfg.admin.to_string(),
                minter: cfg.minter.to_string(),
                price: cfg.price,
                max_supply: cfg.max_supply,
                paused: cfg.paused,
                base_uri: cfg.base_uri,
                provenance_hash: cfg.provenance_hash,
                royalty_bps: cfg.royalty_bps,
                royalty_receiver: cfg.royalty_receiver.to_string(),
                total_minted,
            })
        }
    }
}
