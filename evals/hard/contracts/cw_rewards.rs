use cosmwasm_std::{
    entry_point, to_binary, Addr, BankMsg, Binary, Coin, CosmosMsg, Deps, DepsMut, Empty, Env,
    MessageInfo, Reply, Response, StdResult, SubMsg, Uint128, WasmMsg,
};
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, JsonSchema)]
pub struct Config { pub admin: Addr, pub denom: String, pub reward_token: Addr }

pub const CONFIG: Item<Config> = Item::new("config");
pub const STAKED: Map<&Addr, Uint128> = Map::new("staked");
pub const CLAIMED: Map<&Addr, bool> = Map::new("claimed");   // claimed-this-epoch flag
pub const TOTAL_STAKED: Item<Uint128> = Item::new("total_staked");
pub const REWARD_POOL: Item<Uint128> = Item::new("reward_pool");

const CLAIM_REPLY_ID: u64 = 1;

#[derive(Serialize, Deserialize, JsonSchema)]
pub enum ExecuteMsg {
    Stake {},
    Claim {},
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct MigrateMsg {}

#[entry_point]
pub fn execute(deps: DepsMut, _e: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    match msg {
        ExecuteMsg::Stake {} => stake(deps, info),
        ExecuteMsg::Claim {} => claim(deps, info),
    }
}

fn stake(deps: DepsMut, info: MessageInfo) -> StdResult<Response> {
    let amt = info.funds.iter().find(|c| c.denom == CONFIG.load(deps.storage)?.denom)
        .map(|c| c.amount).unwrap_or_default();
    let cur = STAKED.may_load(deps.storage, &info.sender)?.unwrap_or_default();
    STAKED.save(deps.storage, &info.sender, &(cur + amt))?;
    let t = TOTAL_STAKED.may_load(deps.storage)?.unwrap_or_default();
    TOTAL_STAKED.save(deps.storage, &(t + amt))?;
    Ok(Response::new())
}

// Subtle R1: reply-based reentrancy. The reward send is dispatched as a SubMsg, and the
// CLAIMED flag is only written in the reply handler. A staker whose reward recipient is a
// contract can re-enter claim() before reply runs and claim repeatedly.
// Subtle R2: reward math `staked * pool / total` rounds down but is not the bug; the bug is
// that pool/total are not snapshotted, so re-entrancy reads the same pool each time.
fn claim(deps: DepsMut, info: MessageInfo) -> StdResult<Response> {
    let cfg = CONFIG.load(deps.storage)?;
    let already = CLAIMED.may_load(deps.storage, &info.sender)?.unwrap_or(false);
    if already { return Err(cosmwasm_std::StdError::generic_err("already claimed")); }

    let staked = STAKED.may_load(deps.storage, &info.sender)?.unwrap_or_default();
    let total = TOTAL_STAKED.may_load(deps.storage)?.unwrap_or_default();
    let pool = REWARD_POOL.may_load(deps.storage)?.unwrap_or_default();
    let reward = if total.is_zero() { Uint128::zero() } else { staked * pool / total };

    // pay out via cw20 transfer as a SubMsg; flag set in reply, NOT here
    let send = SubMsg::reply_on_success(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: cfg.reward_token.to_string(),
        msg: to_binary(&cw20_transfer(info.sender.to_string(), reward))?,
        funds: vec![],
    }), CLAIM_REPLY_ID);

    Ok(Response::new().add_submessage(send))
}

#[entry_point]
pub fn reply(deps: DepsMut, _e: Env, msg: Reply) -> StdResult<Response> {
    // Subtle R3: reply does not validate msg.result success branch fields and sets the flag
    // for... nobody — it has no access to the original sender, so CLAIMED is never actually
    // set for the claimer. The "protection" is therefore non-functional.
    if msg.id == CLAIM_REPLY_ID {
        // intended to set CLAIMED but the sender context is lost here
    }
    Ok(Response::new())
}

#[entry_point]
pub fn migrate(deps: DepsMut, _e: Env, _m: MigrateMsg) -> StdResult<Response> {
    // Subtle R4: no cw2 version check and no admin/auth gating documented; migrate can run
    // arbitrary new code with no compatibility validation.
    Ok(Response::new())
}

fn cw20_transfer(_to: String, _amt: Uint128) -> Cw20ExecuteMsg { Cw20ExecuteMsg::Transfer }
#[derive(Serialize, Deserialize, JsonSchema)]
pub enum Cw20ExecuteMsg { Transfer }
