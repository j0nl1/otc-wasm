use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Coin;

use crate::state::{Deal, DealStatus, Id};

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: String,
    pub duration_range: Vec<u64>,
}

#[cw_serde]
pub struct MigrateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    Claim(Id),
    Withdraw(Id),
    CreateDeal(CreateDealMsg),
    ExecuteDeal(Id),
    CancelDeal(Id),
    UpdateConfig {
        owner: Option<String>,
        duration_range: Option<Vec<u64>>,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Deal)]
    DealById(u64),
    #[returns(Vec<Deal>)]
    DealsByExpiration {
        options: Option<QueryOptions>,
        show_expired: bool,
    },
    #[returns(Vec<Deal>)]
    DealsByFilters {
        filters: QueryFilter,
        options: Option<QueryOptions>,
    },
}

/// QueryOptions are used to paginate contract queries
#[cw_serde]
#[derive(Default)]
pub struct QueryOptions {
    /// Whether to sort items in ascending or descending order
    pub descending: Option<bool>,
    /// The id to start the query after
    pub start_after: Option<u64>,
    // The number of items that will be returned
    pub limit: Option<u32>,
}

#[cw_serde]
pub struct QueryFilter {
    pub seller: Option<String>,
    pub status: Option<DealStatus>,
}

#[cw_serde]
pub struct CreateDealMsg {
    pub offer: Coin,
    pub ask: Coin,
    pub duration: u64,
}
