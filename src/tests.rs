use std::borrow::BorrowMut;

use cosmwasm_std::testing::{mock_dependencies, mock_env, MockApi, MockQuerier};
use cosmwasm_std::{coin, Addr, Env, MemoryStorage, OwnedDeps, StdError, Timestamp};

use crate::msg::{QueryFilter, QueryOptions};
use crate::query::{query_deal_by_id, query_deals_by_expiration, query_deals_by_filters};
use crate::state::{deals, Deal, DealStatus};

const SELLER: &str = "seller";
const BUYER: &str = "buyer";

fn mock_data(env: &Env, deps: &mut OwnedDeps<MemoryStorage, MockApi, MockQuerier>) {
    let seller = Addr::unchecked(SELLER);
    let buyer: Addr = Addr::unchecked(BUYER);

    let deal = Deal {
        id: 1,
        seller: seller.clone(),
        creation_time: env.block.time,
        end_time: Timestamp::from_seconds(env.block.time.seconds()).plus_hours(1),
        buyer: Some(buyer.clone()),
        ask: coin(12, "ucosm"),
        offer: coin(100, "ustake"),
        status: DealStatus::Closed,
    };

    let res = deals().save(deps.as_mut().storage, deal.id.clone(), &deal);
    assert!(res.is_ok());

    let deal: Deal = Deal {
        id: 2,
        seller: seller.clone(),
        creation_time: env.block.time,
        end_time: Timestamp::from_seconds(env.block.time.seconds()).plus_hours(1),
        buyer: Some(Addr::unchecked("another_buyer")),
        ask: coin(12, "ucosm"),
        offer: coin(100, "ustake"),
        status: DealStatus::Open,
    };

    let res = deals().save(deps.as_mut().storage, deal.id.clone(), &deal);
    assert!(res.is_ok());

    let deal: Deal = Deal {
        id: 3,
        seller: Addr::unchecked("another_seller"),
        creation_time: env.block.time,
        end_time: Timestamp::from_seconds(env.block.time.seconds()).minus_days(1),
        buyer: Some(Addr::unchecked("another_buyer")),
        ask: coin(12, "ucosm"),
        offer: coin(100, "ustake"),
        status: DealStatus::Open,
    };

    let res = deals().save(deps.as_mut().storage, deal.id.clone(), &deal);
    assert!(res.is_ok());

    let deal: Deal = Deal {
        id: 4,
        seller: seller.clone(),
        creation_time: env.block.time,
        end_time: Timestamp::from_seconds(env.block.time.seconds()).minus_days(1),
        buyer: Some(Addr::unchecked("another_buyer")),
        ask: coin(12, "ucosm"),
        offer: coin(100, "ustake"),
        status: DealStatus::Open,
    };

    let res = deals().save(deps.as_mut().storage, deal.id.clone(), &deal);
    assert!(res.is_ok());
}

#[test]
fn try_query_deal_by_id() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    mock_data(&env, deps.borrow_mut());

    let res = query_deal_by_id(deps.as_ref(), 1);
    assert_eq!(res.unwrap().id, 1);
}

#[test]
fn try_query_deals_by_filters() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    mock_data(&env, deps.borrow_mut());

    // If not filter is provided should throw an error.
    let filters = QueryFilter {
        seller: None,
        status: None,
    };
    let res = query_deals_by_filters(deps.as_ref(), filters, None);
    assert_eq!(
        res.unwrap_err(),
        StdError::generic_err("No filters provided")
    );

    // filter by seller should return 3 deals
    let filters = QueryFilter {
        seller: Some(SELLER.to_string()),
        status: None,
    };
    let res = query_deals_by_filters(deps.as_ref(), filters, None);
    assert_eq!(res.unwrap().len(), 3);

    // filter by seller and status open should return 2 deals
    let filters = QueryFilter {
        seller: Some(SELLER.to_string()),
        status: Some(DealStatus::Open),
    };
    let res = query_deals_by_filters(deps.as_ref(), filters, None);
    assert_eq!(res.unwrap().len(), 2);

    // filter by seller and status closed should return 1 deal
    let filters = QueryFilter {
        seller: Some(SELLER.to_string()),
        status: Some(DealStatus::Closed),
    };
    let res = query_deals_by_filters(deps.as_ref(), filters, None);
    assert_eq!(res.unwrap().len(), 1);

    // filter by seller and status open but providing query option with limit 1 should return 1
    let filters = QueryFilter {
        seller: Some(SELLER.to_string()),
        status: Some(DealStatus::Open),
    };
    let query_options = QueryOptions {
        start_after: None,
        limit: Some(1),
        descending: None,
    };

    let res = query_deals_by_filters(deps.as_ref(), filters, Some(query_options));
    assert_eq!(res.unwrap().len(), 1);

    // providing order descending should return a different order than no providing anyhting
    let filters = QueryFilter {
        seller: Some(SELLER.to_string()),
        status: None,
    };

    let query_options = QueryOptions {
        start_after: None,
        limit: None,
        descending: Some(true),
    };

    let res = query_deals_by_filters(deps.as_ref(), filters.clone(), Some(query_options));
    assert_eq!(res.unwrap()[0].id, 4);

    let res_two = query_deals_by_filters(deps.as_ref(), filters, None);
    assert_eq!(res_two.unwrap()[0].id, 1);
}

#[test]
pub fn try_query_deals_by_expiration() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    mock_data(&env, deps.borrow_mut());

    // It should only show 2 since the other 2 are expired
    let res = query_deals_by_expiration(deps.as_ref(), env.clone(), false, None);
    assert_eq!(res.unwrap().len(), 2);

    // It should show 4 since we are showing expired
    let res = query_deals_by_expiration(deps.as_ref(), env.clone(), true, None);
    assert_eq!(res.unwrap().len(), 4);

    // It should show 2 since we are showing expired and providing a limit of 2
    let query_options = QueryOptions {
        start_after: None,
        limit: Some(2),
        descending: None,
    };

    let res = query_deals_by_expiration(deps.as_ref(), env.clone(), true, Some(query_options));
    assert_eq!(res.unwrap().len(), 2);
}
