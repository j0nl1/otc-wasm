use std::borrow::BorrowMut;

use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info, MockApi, MockQuerier};
use cosmwasm_std::{coin, Addr, Env, MemoryStorage, MessageInfo, OwnedDeps, StdError, Timestamp};

use crate::execute::{create_deal, update_config};
use crate::instantiate::instantiate;
use crate::msg::{CreateDealMsg, InstantiateMsg, QueryFilter, QueryOptions};
use crate::query::{
    query_config, query_deal_by_id, query_deals_by_expiration, query_deals_by_filters,
};
use crate::state::{deals, Config, Deal, DealStatus, CONFIG};

const SELLER: &str = "seller";
const BUYER: &str = "buyer";

fn do_instantiate() -> (
    OwnedDeps<MemoryStorage, MockApi, MockQuerier>,
    Env,
    MessageInfo,
) {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info: MessageInfo = mock_info("owner", &[]);

    let msg = InstantiateMsg {
        owner: "owner".to_string(),
        duration_range: vec![500, 300],
    };

    let res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg.clone());
    assert!(res.is_ok());

    let cfg = CONFIG.load(&deps.storage).unwrap();

    // Verify config is right
    assert_eq!(cfg.owner, msg.owner);
    assert_eq!(cfg.duration_range, msg.duration_range);

    (deps, env, info)
}

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
fn test_query_deal_by_id() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    mock_data(&env, deps.borrow_mut());

    let res = query_deal_by_id(deps.as_ref(), 1);
    assert_eq!(res.unwrap().id, 1);
}

#[test]
fn test_query_deals_by_filters() {
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
pub fn test_query_deals_by_expiration() {
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

#[test]
pub fn test_query_config() {
    let (deps, _, _) = do_instantiate();

    let res = query_config(deps.as_ref());
    assert!(res.is_ok());

    let cfg = res.unwrap();
    assert_eq!(cfg.owner, "owner".to_string());
    assert_eq!(cfg.duration_range, vec![500, 300]);
}

#[test]
pub fn test_update_config() {
    let (mut deps, _env, info) = do_instantiate();

    let res = update_config(deps.as_mut(), info.clone(), None, Some(vec![100, 200]));
    assert!(res.is_ok());

    // Config should have changed
    let cfg = CONFIG.load(&deps.storage).unwrap();
    assert_eq!(cfg.duration_range, vec![100, 200]);
    assert_eq!(cfg.owner, "owner".to_string());

    let res = update_config(
        deps.as_mut(),
        info.clone(),
        Some("new_owner".to_string()),
        None,
    );
    assert!(res.is_ok());

    // owner should have changed
    let cfg = CONFIG.load(&deps.storage).unwrap();
    assert_eq!(cfg.owner, "new_owner".to_string());

    // should fail since the sender is not the owner
    let res = update_config(
        deps.as_mut(),
        info.clone(),
        Some("test_new_owner".to_string()),
        None,
    );
    assert!(res.is_err())
}

#[test]
pub fn test_create_deal() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let offer = coin(100, "ustake");
    let info: MessageInfo = mock_info(SELLER, &[offer.clone()]);

    CONFIG
        .save(
            deps.as_mut().storage,
            &Config {
                owner: Addr::unchecked("owner"),
                duration_range: vec![500, 300],
            },
        )
        .unwrap();

    let msg = CreateDealMsg {
        offer,
        ask: coin(12, "ucosm"),
        duration: 500,
    };

    let res = create_deal(deps.as_mut(), env.clone(), info.clone(), msg.clone());
    assert!(res.is_ok());

    let deal = deals().load(deps.as_ref().storage, 1).unwrap();

    // Verify deal is right
    assert_eq!(deal.creation_time, env.block.time);
    assert_eq!(deal.end_time, env.block.time.plus_seconds(msg.duration));
    assert_eq!(deal.status, DealStatus::Open);
    assert_eq!(deal.buyer, None);
    assert_eq!(deal.seller, info.sender);
    assert_eq!(deal.ask, msg.ask);
    assert_eq!(deal.offer, msg.offer);
}
