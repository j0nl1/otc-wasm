use crate::tests::common::*;
use crate::tests::suite::*;

#[test]
fn should_create_a_deal() {
    let mut suite = OTCSuite::init().unwrap();
    let msg = CreateDealMsg {
        ask: coin(100, DENOM_1),
        offer: coin(1000, DENOM_2),
        duration: 20000,
    };
    suite
        .create_deal(&suite.seller.clone(), msg.clone())
        .unwrap();

    let deal = suite.query_deal_by_id(1).unwrap();
    assert_eq!(deal.ask, msg.ask);
    assert_eq!(deal.offer, msg.offer);
    assert_eq!(deal.seller, suite.seller);
}
