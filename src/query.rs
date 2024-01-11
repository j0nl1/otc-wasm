#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use cosmwasm_std::{to_json_binary, Addr, Binary, Deps, Env, Order, StdError, StdResult};
use cw_storage_plus::Bound;

use crate::{
    msg::{QueryFilter, QueryMsg, QueryOptions},
    state::{deals, Deal},
};

const DEFAULT_QUERY_LIMIT: u32 = 10;
const MAX_QUERY_LIMIT: u32 = 100;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::DealById(id) => to_json_binary(&query_deal_by_id(deps, id)?),
        QueryMsg::DealsByExpiration {
            options,
            show_expired,
        } => to_json_binary(&query_deals_by_expiration(
            deps,
            env,
            show_expired,
            options,
        )?),
        QueryMsg::DealsByFilters { options, filters } => {
            to_json_binary(&query_deals_by_filters(deps, filters, options)?)
        }
    }
}

pub fn query_deal_by_id(deps: Deps, id: u64) -> StdResult<Deal> {
    deals().load(deps.storage, id)
}

pub fn query_deals_by_filters(
    deps: Deps,
    filter: QueryFilter,
    query_options: Option<QueryOptions>,
) -> StdResult<Vec<Deal>> {
    let options = query_options.unwrap_or_default();

    let mut order = Order::Ascending;
    if let Some(descending) = options.descending {
        if descending {
            order = Order::Descending;
        }
    };

    let limit = options
        .limit
        .unwrap_or(DEFAULT_QUERY_LIMIT)
        .min(MAX_QUERY_LIMIT);

    let (min, max) = match order {
        Order::Ascending => (options.start_after.map(Bound::exclusive), None),
        Order::Descending => (None, options.start_after.map(Bound::exclusive)),
    };

    match (filter.seller, filter.status) {
        (Some(seller), None) => {
            let result = deals()
                .idx
                .seller
                .prefix(Addr::unchecked(seller))
                .range(deps.storage, min, max, order)
                .take(limit as usize)
                .map(|item| item.map(|(_, v)| v))
                .collect::<StdResult<_>>()?;
            Ok(result)
        }
        (None, Some(status)) => {
            let result = deals()
                .idx
                .status
                .prefix(status.as_string())
                .range(deps.storage, min, max, order)
                .take(limit as usize)
                .map(|item| item.map(|(_, v)| v))
                .collect::<StdResult<_>>()?;
            Ok(result)
        }
        (Some(seller), Some(status)) => {
            let result = deals()
                .idx
                .seller_status
                .prefix((Addr::unchecked(seller), status.as_string()))
                .range(deps.storage, min, max, order)
                .take(limit as usize)
                .map(|item| item.map(|(_, v)| v))
                .collect::<StdResult<_>>()?;
            Ok(result)
        }
        (None, None) => Err(StdError::generic_err("No filters provided")),
    }
}

pub fn query_deals_by_expiration(
    deps: Deps,
    env: Env,
    show_expired: bool,
    options: Option<QueryOptions>,
) -> StdResult<Vec<Deal>> {
    let options = options.unwrap_or_default();

    let mut order = Order::Ascending;
    if let Some(descending) = options.descending {
        if descending {
            order = Order::Descending;
        }
    };

    let limit = options
        .limit
        .unwrap_or(DEFAULT_QUERY_LIMIT)
        .min(MAX_QUERY_LIMIT);

    let now = env.block.time.seconds();

    let (min, max) = match show_expired {
        true => match order {
            Order::Ascending => (Some(Bound::exclusive((0, 0))), None),
            Order::Descending => (None, Some(Bound::exclusive((u64::MAX, 0)))),
        },
        false => match order {
            Order::Ascending => (Some(Bound::exclusive((now, 0))), None),
            Order::Descending => (None, Some(Bound::exclusive((now, 0)))),
        },
    };

    let result = deals()
        .idx
        .end_time
        .range(deps.storage, min, max, order)
        .take(limit as usize)
        .map(|item| item.map(|(_, v)| v))
        .collect::<StdResult<_>>()?;

    Ok(result)
}
