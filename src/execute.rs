#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use cosmwasm_std::{
    ensure, ensure_eq, BankMsg, CosmosMsg, DepsMut, Env, Event, MessageInfo, Response,
};
use cw_denom::validate_native_denom;
use cw_utils::{must_pay, one_coin, Expiration};

use crate::error::ContractError;
use crate::msg::{CreateDealMsg, ExecuteMsg};
use crate::state::{deals, next_id, Deal, DealStatus, Id, CONFIG};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Claim(id) => claim(deps, info, id),
        ExecuteMsg::Withdraw(id) => withdraw(deps, env, info, id),
        ExecuteMsg::CreateDeal(created_deal_msg) => create_deal(deps, env, info, created_deal_msg),
        ExecuteMsg::ExecuteDeal(id) => execute_deal(deps, env, info, id),
        ExecuteMsg::CancelDeal(id) => cancel_deal(deps, env, info, id),
        ExecuteMsg::UpdateConfig {
            owner,
            duration_range,
        } => update_config(deps, info, owner, duration_range),
    }
}

pub fn withdraw(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    id: Id,
) -> Result<Response, ContractError> {
    let deal = deals().load(deps.storage, id)?;

    ensure_eq!(info.sender, deal.seller, ContractError::Unauthorized);

    ensure!(deal.status == DealStatus::Open, ContractError::Unauthorized);

    ensure!(
        Expiration::AtTime(deal.end_time).is_expired(&env.block),
        ContractError::Unauthorized
    );

    deals().update(deps.storage, id, |d| -> Result<Deal, ContractError> {
        let mut deal = d.unwrap();
        deal.status = DealStatus::Expired;
        Ok(deal)
    })?;

    let msg = BankMsg::Send {
        to_address: info.sender.to_string(),
        amount: vec![deal.offer],
    };

    let event = Event::new("OtcWasm.v1.MsgWithdraw")
        .add_attribute("seller", info.sender)
        .add_attribute("id", id.to_string());

    deals().update(deps.storage, id, |d| -> Result<Deal, ContractError> {
        let mut deal = d.unwrap();
        deal.status = DealStatus::Closed;
        Ok(deal)
    })?;

    Ok(Response::new().add_event(event).add_message(msg))
}

pub fn execute_deal(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    id: Id,
) -> Result<Response, ContractError> {
    let deal = deals().load(deps.storage, id)?;

    ensure_eq!(deal.status, DealStatus::Open, ContractError::Unauthorized);

    ensure!(
        !Expiration::AtTime(deal.end_time).is_expired(&env.block),
        ContractError::DealExpired
    );

    let payment = must_pay(&info, &deal.ask.denom)?;

    ensure_eq!(
        payment,
        deal.ask.amount,
        ContractError::InsufficientAmount(payment.to_string())
    );

    deals().update(deps.storage, id, |d| -> Result<Deal, ContractError> {
        let mut deal = d.unwrap();
        deal.status = DealStatus::Claimable;
        deal.buyer = Some(info.sender.clone());
        Ok(deal)
    })?;

    let msg = CosmosMsg::Bank(cosmwasm_std::BankMsg::Send {
        to_address: info.sender.to_string(),
        amount: vec![deal.offer],
    });

    let event = Event::new("OtcWasm.v1.MsgExecuteDeal")
        .add_attribute("buyer", info.sender)
        .add_attribute("id", id.to_string());

    deals().update(deps.storage, id, |d| -> Result<Deal, ContractError> {
        let mut deal = d.unwrap();
        deal.status = DealStatus::Claimable;
        Ok(deal)
    })?;

    Ok(Response::new().add_event(event).add_message(msg))
}

pub fn create_deal(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: CreateDealMsg,
) -> Result<Response, ContractError> {
    let offer = one_coin(&info)?;

    validate_native_denom(msg.ask.denom.clone())?;

    let config = CONFIG.load(deps.storage)?;

    ensure!(
        config.duration_range.contains(&msg.duration),
        ContractError::InvalidDuration(
            config.duration_range[0],
            config.duration_range[config.duration_range.len() - 1]
        )
    );

    let id = next_id(deps.storage)?;

    let deal = Deal {
        id,
        offer,
        seller: info.sender.clone(),
        buyer: None,
        ask: msg.ask,
        status: DealStatus::Open,
        creation_time: env.block.time,
        end_time: env.block.time.plus_seconds(msg.duration),
    };

    deals().save(deps.storage, id, &deal)?;

    let event = Event::new("OtcWasm.v1.MsgCreateDeal")
        .add_attribute("seller", info.sender.to_string())
        .add_attribute("id", id.to_string());

    Ok(Response::new().add_event(event))
}

pub fn claim(deps: DepsMut, info: MessageInfo, id: Id) -> Result<Response, ContractError> {
    let deal = deals().load(deps.storage, id)?;
    ensure_eq!(info.sender, deal.seller, ContractError::Unauthorized);
    ensure_eq!(
        deal.status,
        DealStatus::Claimable,
        ContractError::Unauthorized
    );
    let msg = BankMsg::Send {
        to_address: info.sender.to_string(),
        amount: vec![deal.ask],
    };

    let event = Event::new("OtcWasm.v1.MsgClaim")
        .add_attribute("claimer", info.sender)
        .add_attribute("id", id.to_string());

    deals().update(deps.storage, id, |d| -> Result<Deal, ContractError> {
        let mut deal = d.unwrap();
        deal.status = DealStatus::Closed;
        Ok(deal)
    })?;

    Ok(Response::new().add_event(event).add_message(msg))
}

pub fn cancel_deal(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    id: Id,
) -> Result<Response, ContractError> {
    let deal = deals().update(deps.storage, id, |d| -> Result<Deal, ContractError> {
        let mut deal = d.ok_or(ContractError::DealNotFound)?;
        ensure_eq!(info.sender, deal.seller, ContractError::Unauthorized);
        ensure_eq!(deal.status, DealStatus::Open, ContractError::Unauthorized);
        ensure!(
            !Expiration::AtTime(deal.end_time).is_expired(&env.block),
            ContractError::DealExpired
        );
        deal.status = DealStatus::Cancelled;
        Ok(deal)
    })?;

    let msg = BankMsg::Send {
        to_address: info.sender.to_string(),
        amount: vec![deal.offer],
    };

    let event = Event::new("OtcWasm.v1.MsgCancelDeal").add_attribute("id", id.to_string());
    Ok(Response::new().add_event(event).add_message(msg))
}

pub fn update_config(
    deps: DepsMut,
    info: MessageInfo,
    owner: Option<String>,
    duration_range: Option<Vec<u64>>,
) -> Result<Response, ContractError> {
    CONFIG.update(deps.storage, |mut config| -> Result<_, ContractError> {
        ensure_eq!(info.sender, config.owner, ContractError::Unauthorized);
        if let Some(owner) = owner {
            config.owner = deps.api.addr_validate(&owner)?;
        }
        if let Some(duration_range) = duration_range {
            config.duration_range = duration_range;
        }
        Ok(config)
    })?;

    let event = Event::new("OtcWasm.v1.MsgUpdateConfig");

    Ok(Response::new().add_event(event))
}
