#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use crate::{error::ContractError, msg::MigrateMsg};
use cosmwasm_std::{DepsMut, Env, Event, Response};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    let event = Event::new("OtcWasm.v1.MsgMigrateContract");
    Ok(Response::new().add_event(event))
}
