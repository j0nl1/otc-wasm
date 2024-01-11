use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Coin, StdResult, Storage, Timestamp};
use cw_storage_macro::index_list;
use cw_storage_plus::{IndexedMap, Item, MultiIndex, UniqueIndex};

#[cw_serde]
pub struct Config {
    pub owner: Addr,
    /// Range of time in seconds that a deal can be open
    pub duration_range: Vec<u64>,
}

pub const CONFIG: Item<Config> = Item::new("config");

pub type Id = u64;

pub const ID_COUNT: Item<Id> = Item::new("id_count");

pub fn next_id(store: &mut dyn Storage) -> StdResult<Id> {
    let id = ID_COUNT.may_load(store)?.unwrap_or(1);
    ID_COUNT.save(store, &(id + 1))?;
    Ok(id)
}

#[cw_serde]
pub enum DealStatus {
    Open,
    Claimable,
    Cancelled,
    Closed,
    Expired,
}

impl DealStatus {
    pub fn as_string(&self) -> String {
        match self {
            DealStatus::Open => "open".to_string(),
            DealStatus::Closed => "closed".to_string(),
            DealStatus::Claimable => "claimable".to_string(),
            DealStatus::Cancelled => "cancelled".to_string(),
            DealStatus::Expired => "expired".to_string(),
        }
    }
}

#[cw_serde]
pub struct Deal {
    pub id: Id,
    pub seller: Addr,
    pub buyer: Option<Addr>,
    pub offer: Coin,
    pub ask: Coin,
    pub status: DealStatus,
    pub creation_time: Timestamp,
    pub end_time: Timestamp,
}

#[index_list(Deal)]
pub struct DealIndexer<'a> {
    pub id: UniqueIndex<'a, Id, Deal, Id>,
    pub seller: MultiIndex<'a, Addr, Deal, Id>,
    pub status: MultiIndex<'a, String, Deal, Id>,
    pub end_time: MultiIndex<'a, u64, Deal, Id>,
    pub seller_status: MultiIndex<'a, (Addr, String), Deal, Id>,
}

pub fn deals<'a>() -> IndexedMap<'a, u64, Deal, DealIndexer<'a>> {
    let indexes = DealIndexer {
        id: UniqueIndex::new(|d| d.id, "deals__id"),
        seller: MultiIndex::new(
            |_pk: &[u8], d: &Deal| d.seller.clone(),
            "deals",
            "deals__seller",
        ),
        status: MultiIndex::new(
            |_pk: &[u8], d: &Deal| d.status.as_string(),
            "deals",
            "deals__status",
        ),
        end_time: MultiIndex::new(
            |_pk: &[u8], d: &Deal| d.end_time.seconds(),
            "deals",
            "deals__end_time",
        ),
        seller_status: MultiIndex::new(
            |_pk: &[u8], d: &Deal| (d.seller.clone(), d.status.as_string()),
            "deals",
            "deals__seller__status",
        ),
    };
    IndexedMap::new("deals", indexes)
}
