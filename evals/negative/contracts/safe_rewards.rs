// Negative-eval fixture (CosmWasm): patterns that LOOK like the classic reply/SubMsg reentrancy
// and migration bugs but are SAFE. A well-calibrated audit should NOT report these.
// Minimized for the eval; not a reference implementation.

use cosmwasm_std::{
    entry_point, to_json_binary, BankMsg, Coin, DepsMut, Env, MessageInfo, Response, StdError,
    StdResult, SubMsg, Uint128,
};
use cw2::{get_contract_version, set_contract_version};
use cw_storage_plus::Map;

const CONTRACT_NAME: &str = "crates.io:safe-rewards";
const CONTRACT_VERSION: &str = "2.0.0";

const CLAIMED: Map<&str, bool> = Map::new("claimed");
const REWARD: Map<&str, Uint128> = Map::new("reward");

pub fn execute_claim(deps: DepsMut, _env: Env, info: MessageInfo) -> StdResult<Response> {
    let who = info.sender.to_string();

    // Replay guard is set BEFORE any SubMsg is dispatched, and the amount is read once and
    // zeroed here. A re-entrant call to claim() during message handling sees claimed=true and
    // a zeroed reward, so double-claim is not possible. This is NOT reply/submsg reentrancy.
    if CLAIMED.may_load(deps.storage, &who)?.unwrap_or(false) {
        return Err(StdError::generic_err("already claimed"));
    }
    let amount = REWARD.may_load(deps.storage, &who)?.unwrap_or_default();
    if amount.is_zero() {
        return Err(StdError::generic_err("nothing to claim"));
    }
    CLAIMED.save(deps.storage, &who, &true)?;
    REWARD.save(deps.storage, &who, &Uint128::zero())?;

    // Effects are fully committed before the payout. A plain BankMsg (no reply needed) is used,
    // so there is no reply-handler state to get wrong.
    let pay = SubMsg::new(BankMsg::Send {
        to_address: who,
        amount: vec![Coin { denom: "ustake".into(), amount }],
    });
    Ok(Response::new().add_submessage(pay))
}

#[entry_point]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    // cw2 version compatibility IS checked: same contract name, and no downgrade. Migrate
    // authorization on CosmWasm is enforced by the chain (only the code admin can migrate),
    // and that admin is the governance multisig per deployment. Not an unguarded migrate.
    let prev = get_contract_version(deps.storage)?;
    if prev.contract != CONTRACT_NAME {
        return Err(StdError::generic_err("wrong contract"));
    }
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(Response::new().add_attribute("action", "migrate"))
}

#[cosmwasm_schema::cw_serde]
pub struct MigrateMsg {}
