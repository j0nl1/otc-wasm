use cw_multi_test::{App, AppResponse, Executor};

use crate::tests::common::*;

pub struct OTCSuite {
    pub app: App,
    // The account that deploys everything
    pub deployer: Addr,
    // The account that is owner
    pub executor: Addr,
    // seller address
    pub seller: Addr,
    // buyer address
    pub buyer: Addr,
    // contract otd address
    pub otc: Addr,
}

impl OTCSuite {
    pub fn init() -> Result<OTCSuite> {
        let genesis_funds = vec![coin(300000, DENOM_1), coin(30000, DENOM_2)];
        let deployer = Addr::unchecked(DEPLOYER);
        let executor = Addr::unchecked(EXECUTOR);
        let seller = Addr::unchecked(SELLER);
        let buyer = Addr::unchecked(BUYER);

        let mut app = App::new(|router, _, storage| {
            router
                .bank
                .init_balance(storage, &deployer, genesis_funds)
                .unwrap();
        });
        app.send_tokens(
            deployer.clone(),
            seller.clone(),
            &[coin(50000, DENOM_1), coin(5000, DENOM_2)],
        )?;
        app.send_tokens(
            deployer.clone(),
            buyer.clone(),
            &[coin(50000, DENOM_1), coin(5000, DENOM_2)],
        )?;
        app.send_tokens(
            deployer.clone(),
            executor.clone(),
            &[coin(50000, DENOM_1), coin(5000, DENOM_2)],
        )?;

        let otc_id = app.store_code(contract_otc());

        let otc = app.instantiate_contract(
            otc_id,
            executor.clone(),
            &InstantiateMsg {
                owner: executor.to_string(),
                duration_range: [20000, 40000, 60000].to_vec(),
            },
            &[],
            "otc_contract",
            Some(executor.to_string()),
        )?;

        Ok(OTCSuite {
            app,
            deployer,
            executor,
            seller,
            buyer,
            otc,
        })
    }

    pub fn query_balance(&self, addr: &Addr, denom: &str) -> StdResult<Coin> {
        Ok(self.app.wrap().query_balance(addr.as_str(), denom)?)
    }

    pub fn query_deal_by_id(&self, id: Id) -> StdResult<Deal> {
        let msg = QueryMsg::DealById(id);
        self.app.wrap().query_wasm_smart(self.otc.clone(), &msg)
    }

    pub fn create_deal(
        &mut self,
        sender: &Addr,
        msg: CreateDealMsg,
    ) -> Result<AppResponse, ContractError> {
        self.app
            .execute_contract(
                sender.clone(),
                self.otc.clone(),
                &ExecuteMsg::CreateDeal(msg.clone()),
                &[msg.offer],
            )
            .map_err(|err| err.downcast().unwrap())
    }

    pub fn fast_forward_block_time(&mut self, forward_time_sec: u64) {
        let block = self.app.block_info();

        let mock_block = BlockInfo {
            height: block.height + 10,
            chain_id: block.chain_id,
            time: block.time.plus_seconds(forward_time_sec),
        };

        self.app.set_block(mock_block);
    }
}
