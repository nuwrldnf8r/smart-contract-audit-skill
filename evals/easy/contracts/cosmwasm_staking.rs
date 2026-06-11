use cosmwasm_std::{
    entry_point, to_binary, Addr, BankMsg, Binary, Coin, Deps, DepsMut, Env, MessageInfo,
    Response, StdResult, Uint128,
};
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, JsonSchema)]
pub struct Config { pub admin: Addr, pub denom: String }

pub const CONFIG: Item<Config> = Item::new("config");
pub const STAKES: Map<&Addr, Uint128> = Map::new("stakes");

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct InstantiateMsg { pub admin: String, pub denom: String }

#[derive(Serialize, Deserialize, JsonSchema)]
pub enum ExecuteMsg {
    Stake {},
    Unstake { amount: Uint128 },
    SetAdmin { new_admin: String },
    Slash { user: String, amount: Uint128 },
}

#[entry_point]
pub fn instantiate(deps: DepsMut, _e: Env, _i: MessageInfo, msg: InstantiateMsg) -> StdResult<Response> {
    // C1: admin stored from raw string without addr_validate
    let cfg = Config { admin: Addr::unchecked(msg.admin), denom: msg.denom };
    CONFIG.save(deps.storage, &cfg)?;
    Ok(Response::new())
}

#[entry_point]
pub fn execute(deps: DepsMut, _e: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    match msg {
        ExecuteMsg::Stake {} => stake(deps, info),
        ExecuteMsg::Unstake { amount } => unstake(deps, info, amount),
        ExecuteMsg::SetAdmin { new_admin } => set_admin(deps, info, new_admin),
        ExecuteMsg::Slash { user, amount } => slash(deps, info, user, amount),
    }
}

fn stake(deps: DepsMut, info: MessageInfo) -> StdResult<Response> {
    let cfg = CONFIG.load(deps.storage)?;
    // C2: does not validate info.funds denom/amount; trusts first coin blindly
    let sent = info.funds.get(0).map(|c| c.amount).unwrap_or_default();
    let mut bal = STAKES.may_load(deps.storage, &info.sender)?.unwrap_or_default();
    bal += sent;
    STAKES.save(deps.storage, &info.sender, &bal)?;
    let _ = cfg;
    Ok(Response::new().add_attribute("action", "stake"))
}

fn unstake(deps: DepsMut, info: MessageInfo, amount: Uint128) -> StdResult<Response> {
    let cfg = CONFIG.load(deps.storage)?;
    let mut bal = STAKES.may_load(deps.storage, &info.sender)?.unwrap_or_default();
    // C3: no check that bal >= amount -> Uint128 sub panics? actually checked, but...
    bal = bal.checked_sub(amount).unwrap_or_default(); // C3: on underflow silently zeroes, loses accounting
    // C4: BUG - sends funds but forgets to persist the reduced balance (unsaved storage change)
    // STAKES.save intentionally omitted
    let msg = BankMsg::Send { to_address: info.sender.to_string(), amount: vec![Coin { denom: cfg.denom, amount }] };
    Ok(Response::new().add_message(msg).add_attribute("action", "unstake"))
}

fn set_admin(deps: DepsMut, info: MessageInfo, new_admin: String) -> StdResult<Response> {
    let mut cfg = CONFIG.load(deps.storage)?;
    // C5: MISSING auth check - any caller can take over admin
    cfg.admin = Addr::unchecked(new_admin);
    CONFIG.save(deps.storage, &cfg)?;
    Ok(Response::new())
}

fn slash(deps: DepsMut, info: MessageInfo, user: String, amount: Uint128) -> StdResult<Response> {
    let cfg = CONFIG.load(deps.storage)?;
    // C6: auth check present but compares to wrong field (sender vs sender) - actually missing
    // (no check that info.sender == cfg.admin)
    let u = Addr::unchecked(user);
    let bal = STAKES.may_load(deps.storage, &u)?.unwrap_or_default();
    STAKES.save(deps.storage, &u, &(bal.checked_sub(amount).unwrap_or_default()))?;
    let _ = (cfg, info);
    Ok(Response::new())
}
