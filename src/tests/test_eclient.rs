#[cfg(test)]
mod tests {
    use crate::core::client::{ConnStatus, EClient, POISONED_MUTEX};

    use crate::core::{
        common::{
            BarData, CommissionReport, DepthMktDataDescription, FaDataType, FamilyCode,
            HistogramData, HistoricalTick, HistoricalTickBidAsk, HistoricalTickLast, NewsProvider,
            PriceIncrement, RealTimeBar, SmartComponent, TickAttrib, TickAttribBidAsk,
            TickAttribLast, TickByTickType, TickType,
        },
        contract::{Contract, ContractDescription, ContractDetails, DeltaNeutralContract},
        execution::{Execution, ExecutionFilter},
        order::{Order, SoftDollarTier},
        streamer::{Streamer, TestStreamer},
        wrapper::Wrapper,
    };
    use crate::{
        core::{
            errors::IBKRApiLibError,
            messages::{read_fields, read_msg, OutgoingMessageIds},
            order::OrderState,
        },
        examples::contract_samples::simple_future,
    };
    use std::sync::{Arc, Mutex};

    pub struct DummyTestWrapper {}

    impl DummyTestWrapper {
        fn new() -> Self {
            DummyTestWrapper {}
        }
    }

    impl Wrapper for DummyTestWrapper {
        fn error(&mut self, _req_id: i32, _error_code: i32, _error_string: &str) {
            todo!()
        }
        fn win_error(&mut self, _text: &str, _last_error: i32) {
            todo!()
        }
        fn connect_ack(&mut self) {
            todo!()
        }
        fn market_data_type(&mut self, _req_id: i32, _market_data_type: i32) {
            todo!()
        }
        fn tick_price(
            &mut self,
            _req_id: i32,
            _tick_type: TickType,
            _price: f64,
            _attrib: TickAttrib,
        ) {
            todo!()
        }
        fn tick_size(&mut self, _req_id: i32, _tick_type: TickType, _size: i32) {
            todo!()
        }
        fn tick_snapshot_end(&mut self, _req_id: i32) {
            todo!()
        }
        fn tick_generic(&mut self, _req_id: i32, _tick_type: TickType, _value: f64) {
            todo!()
        }
        fn tick_string(&mut self, _req_id: i32, _tick_type: TickType, _value: &str) {
            todo!()
        }
        fn tick_efp(
            &mut self,
            _req_id: i32,
            _tick_type: TickType,
            _basis_points: f64,
            _formatted_basis_points: &str,
            _implied_future: f64,
            _hold_days: i32,
            _future_last_trade_date: &str,
            _dividend_impact: f64,
            _dividends_to_last_trade_date: f64,
        ) {
            todo!()
        }
        fn order_status(
            &mut self,
            _order_id: i32,
            _status: &str,
            _filled: f64,
            _remaining: f64,
            _avg_fill_price: f64,
            _perm_id: i32,
            _parent_id: i32,
            _last_fill_price: f64,
            _client_id: i32,
            _why_held: &str,
            _mkt_cap_price: f64,
        ) {
            todo!()
        }
        fn open_order(
            &mut self,
            _order_id: i32,
            _contract: Contract,
            _order: Order,
            _order_state: OrderState,
        ) {
            todo!()
        }
        fn open_order_end(&mut self) {
            todo!()
        }
        fn connection_closed(&mut self) {
            todo!()
        }
        fn update_account_value(
            &mut self,
            _key: &str,
            _val: &str,
            _currency: &str,
            _account_name: &str,
        ) {
            todo!()
        }
        fn update_portfolio(
            &mut self,
            _contract: Contract,
            _position: f64,
            _market_price: f64,
            _market_value: f64,
            _average_cost: f64,
            _unrealized_pnl: f64,
            _realized_pnl: f64,
            _account_name: &str,
        ) {
            todo!()
        }
        fn update_account_time(&mut self, _time_stamp: &str) {
            todo!()
        }
        fn account_download_end(&mut self, _account_name: &str) {
            todo!()
        }
        fn next_valid_id(&mut self, _order_id: i32) {
            todo!()
        }
        fn contract_details(&mut self, _req_id: i32, _contract_details: ContractDetails) {
            todo!()
        }
        fn bond_contract_details(&mut self, _req_id: i32, _contract_details: ContractDetails) {
            todo!()
        }
        fn contract_details_end(&mut self, _req_id: i32) {
            todo!()
        }
        fn exec_details(&mut self, _req_id: i32, _contract: Contract, _execution: Execution) {
            todo!()
        }
        fn exec_details_end(&mut self, _req_id: i32) {
            todo!()
        }
        fn update_mkt_depth(
            &mut self,
            _req_id: i32,
            _position: i32,
            _operation: i32,
            _side: i32,
            _price: f64,
            _size: i32,
        ) {
            todo!()
        }
        fn update_mkt_depth_l2(
            &mut self,
            _req_id: i32,
            _position: i32,
            _market_maker: &str,
            _operation: i32,
            _side: i32,
            _price: f64,
            _size: i32,
            _is_smart_depth: bool,
        ) {
            todo!()
        }
        fn update_news_bulletin(
            &mut self,
            _msg_id: i32,
            _msg_type: i32,
            _news_message: &str,
            _origin_exch: &str,
        ) {
            todo!()
        }
        fn managed_accounts(&mut self, _accounts_list: &str) {
            todo!()
        }
        fn receive_fa(&mut self, _fa_data: FaDataType, _cxml: &str) {
            todo!()
        }
        fn historical_data(&mut self, _req_id: i32, _bar: BarData) {
            todo!()
        }
        fn historical_data_end(&mut self, _req_id: i32, _start: &str, _end: &str) {
            todo!()
        }
        fn scanner_parameters(&mut self, _xml: &str) {
            todo!()
        }
        fn scanner_data(
            &mut self,
            _req_id: i32,
            _rank: i32,
            _contract_details: ContractDetails,
            _distance: &str,
            _benchmark: &str,
            _projection: &str,
            _legs_str: &str,
        ) {
            todo!()
        }
        fn scanner_data_end(&mut self, _req_id: i32) {
            todo!()
        }
        fn realtime_bar(&mut self, _req_id: i32, _bar: RealTimeBar) {
            todo!()
        }
        fn current_time(&mut self, _time: i64) {
            todo!()
        }
        fn fundamental_data(&mut self, _req_id: i32, _data: &str) {
            todo!()
        }
        fn delta_neutral_validation(
            &mut self,
            _req_id: i32,
            _delta_neutral_contract: DeltaNeutralContract,
        ) {
            todo!()
        }
        fn commission_report(&mut self, _commission_report: CommissionReport) {
            todo!()
        }
        fn position(
            &mut self,
            _account: &str,
            _contract: Contract,
            _position: f64,
            _avg_cost: f64,
        ) {
            todo!()
        }
        fn position_end(&mut self) {
            todo!()
        }
        fn account_summary(
            &mut self,
            _req_id: i32,
            _account: &str,
            _tag: &str,
            _value: &str,
            _currency: &str,
        ) {
            todo!()
        }
        fn account_summary_end(&mut self, _req_id: i32) {
            todo!()
        }
        fn verify_message_api(&mut self, _api_data: &str) {
            todo!()
        }
        fn verify_completed(&mut self, _is_successful: bool, _error_text: &str) {
            todo!()
        }
        fn verify_and_auth_message_api(&mut self, _api_data: &str, _xyz_challange: &str) {
            todo!()
        }
        fn verify_and_auth_completed(&mut self, _is_successful: bool, _error_text: &str) {
            todo!()
        }
        fn display_group_list(&mut self, _req_id: i32, _groups: &str) {
            todo!()
        }
        fn display_group_updated(&mut self, _req_id: i32, _contract_info: &str) {
            todo!()
        }
        fn position_multi(
            &mut self,
            _req_id: i32,
            _account: &str,
            _model_code: &str,
            _contract: Contract,
            _pos: f64,
            _avg_cost: f64,
        ) {
            todo!()
        }
        fn position_multi_end(&mut self, _req_id: i32) {
            todo!()
        }
        fn account_update_multi(
            &mut self,
            _req_id: i32,
            _account: &str,
            _model_code: &str,
            _key: &str,
            _value: &str,
            _currency: &str,
        ) {
            todo!()
        }
        fn account_update_multi_end(&mut self, _req_id: i32) {
            todo!()
        }
        fn tick_option_computation(
            &mut self,
            _req_id: i32,
            _tick_type: TickType,
            _implied_vol: f64,
            _delta: f64,
            _opt_price: f64,
            _pv_dividend: f64,
            _gamma: f64,
            _vega: f64,
            _theta: f64,
            _und_price: f64,
        ) {
            todo!()
        }
        fn security_definition_option_parameter(
            &mut self,
            _req_id: i32,
            _exchange: &str,
            _underlying_con_id: i32,
            _trading_class: &str,
            _multiplier: &str,
            _expirations: std::collections::HashSet<String>,
            _strikes: std::collections::HashSet<bigdecimal::BigDecimal>,
        ) {
            todo!()
        }
        fn security_definition_option_parameter_end(&mut self, _req_id: i32) {
            todo!()
        }
        fn soft_dollar_tiers(&mut self, _req_id: i32, _tiers: Vec<SoftDollarTier>) {
            todo!()
        }
        fn family_codes(&mut self, _family_codes: Vec<FamilyCode>) {
            todo!()
        }
        fn symbol_samples(
            &mut self,
            _req_id: i32,
            _contract_descriptions: Vec<ContractDescription>,
        ) {
            todo!()
        }
        fn mkt_depth_exchanges(
            &mut self,
            _depth_mkt_data_descriptions: Vec<DepthMktDataDescription>,
        ) {
            todo!()
        }
        fn tick_news(
            &mut self,
            _ticker_id: i32,
            _time_stamp: i32,
            _provider_code: &str,
            _article_id: &str,
            _headline: &str,
            _extra_data: &str,
        ) {
            todo!()
        }
        fn smart_components(&mut self, _req_id: i32, _smart_components: Vec<SmartComponent>) {
            todo!()
        }
        fn tick_req_params(
            &mut self,
            _ticker_id: i32,
            _min_tick: f64,
            _bbo_exchange: &str,
            _snapshot_permissions: i32,
        ) {
            todo!()
        }
        fn news_providers(&mut self, _news_providers: Vec<NewsProvider>) {
            todo!()
        }
        fn news_article(&mut self, _request_id: i32, _article_type: i32, _article_text: &str) {
            todo!()
        }
        fn historical_news(
            &mut self,
            _request_id: i32,
            _time: &str,
            _provider_code: &str,
            _article_id: &str,
            _headline: &str,
        ) {
            todo!()
        }
        fn historical_news_end(&mut self, _request_id: i32, _has_more: bool) {
            todo!()
        }
        fn head_timestamp(&mut self, _req_id: i32, _head_timestamp: &str) {
            todo!()
        }
        fn histogram_data(&mut self, _req_id: i32, _items: Vec<HistogramData>) {
            todo!()
        }
        fn historical_data_update(&mut self, _req_id: i32, _bar: BarData) {
            todo!()
        }
        fn reroute_mkt_data_req(&mut self, _req_id: i32, _con_id: i32, _exchange: &str) {
            todo!()
        }
        fn reroute_mkt_depth_req(&mut self, _req_id: i32, _con_id: i32, _exchange: &str) {
            todo!()
        }
        fn market_rule(&mut self, _market_rule_id: i32, _price_increments: Vec<PriceIncrement>) {
            todo!()
        }
        fn pnl(
            &mut self,
            _req_id: i32,
            _daily_pn_l: f64,
            _unrealized_pn_l: f64,
            _realized_pn_l: f64,
        ) {
            todo!()
        }
        fn pnl_single(
            &mut self,
            _req_id: i32,
            _pos: i32,
            _daily_pn_l: f64,
            _unrealized_pn_l: f64,
            _realized_pn_l: f64,
            _value: f64,
        ) {
            todo!()
        }
        fn historical_ticks(&mut self, _req_id: i32, _ticks: Vec<HistoricalTick>, _done: bool) {
            todo!()
        }
        fn historical_ticks_bid_ask(
            &mut self,
            _req_id: i32,
            _ticks: Vec<HistoricalTickBidAsk>,
            _done: bool,
        ) {
            todo!()
        }
        fn historical_ticks_last(
            &mut self,
            _req_id: i32,
            _ticks: Vec<HistoricalTickLast>,
            _done: bool,
        ) {
            todo!()
        }
        fn tick_by_tick_all_last(
            &mut self,
            _req_id: i32,
            _tick_type: TickByTickType,
            _time: i64,
            _price: f64,
            _size: i32,
            _tick_attrib_last: TickAttribLast,
            _exchange: &str,
            _special_conditions: &str,
        ) {
            todo!()
        }
        fn tick_by_tick_bid_ask(
            &mut self,
            _req_id: i32,
            _time: i64,
            _bid_price: f64,
            _ask_price: f64,
            _bid_size: i32,
            _ask_size: i32,
            _tick_attrib_bid_ask: TickAttribBidAsk,
        ) {
            todo!()
        }
        fn tick_by_tick_mid_point(&mut self, _req_id: i32, _time: i64, _mid_point: f64) {
            todo!()
        }
        fn order_bound(&mut self, _req_id: i32, _api_client_id: i32, _api_order_id: i32) {
            todo!()
        }
        fn completed_order(
            &mut self,
            _contract: Contract,
            _order: Order,
            _order_state: OrderState,
        ) {
            todo!()
        }
        fn completed_orders_end(&mut self) {
            todo!()
        }
    }

    //------------------------------------------------------------------------------------------------
    trait ClientConnectForTest {
        fn connect_test(&mut self);
    }

    impl ClientConnectForTest for EClient<DummyTestWrapper> {
        fn connect_test(&mut self) {
            *self.conn_state.lock().expect(POISONED_MUTEX) = ConnStatus::CONNECTED;
            let streamer = TestStreamer::new();
            self.set_streamer(Option::from(Box::new(streamer) as Box<dyn Streamer>));
            self.server_version = 151;
        }
    }

    //------------------------------------------------------------------------------------------------
    #[test]
    fn test_req_account_summary() -> Result<(), IBKRApiLibError> {
        let wrapper = Arc::new(Mutex::new(DummyTestWrapper::new()));
        let app = Arc::new(Mutex::new(EClient::<DummyTestWrapper>::new(
            wrapper,
        )));

        let version = 2;
        let req_id = 100;
        let group_name = "MyGroup";
        let tags = "tag1:tag_value1, tag2:tag_value2";
        let mut buf = Vec::<u8>::new();

        let mut locked_app = app.lock().expect("EClient mutex was poisoned");

        locked_app.connect_test();
        locked_app.req_account_summary(req_id, group_name, tags)?;
        locked_app.stream.as_mut().unwrap().read_to_end(&mut buf)?;

        let expected: [u8; 54] = [
            0, 0, 0, 50, 54, 50, 0, 50, 0, 49, 48, 48, 0, 77, 121, 71, 114, 111, 117, 112, 0, 116,
            97, 103, 49, 58, 116, 97, 103, 95, 118, 97, 108, 117, 101, 49, 44, 32, 116, 97, 103,
            50, 58, 116, 97, 103, 95, 118, 97, 108, 117, 101, 50, 0,
        ];

        let msg_data = read_msg(buf.as_slice())?;

        let fields = read_fields(&msg_data.1);
        assert_eq!(expected.as_ref(), buf.as_slice());
        assert_eq!(
            OutgoingMessageIds::ReqAccountSummary as u8,
            fields[0].parse::<u8>().unwrap()
        );
        assert_eq!(version, fields[1].parse::<i32>().unwrap());
        assert_eq!(req_id, fields[2].parse::<i32>().unwrap());
        assert_eq!(group_name, fields[3]);
        assert_eq!(tags, fields[4]);

        Ok(())
    }

    //------------------------------------------------------------------------------------------------
    #[test]
    fn test_req_account_updates() -> Result<(), IBKRApiLibError> {
        let wrapper = Arc::new(Mutex::new(DummyTestWrapper::new()));
        let app = Arc::new(Mutex::new(EClient::<DummyTestWrapper>::new(
            wrapper,
        )));

        let version = 2;
        let subscribe = true;
        let acct_code = "D12345";
        let mut buf = Vec::<u8>::new();

        let mut locked_app = app.lock().expect("EClient mutex was poisoned");

        locked_app.connect_test();
        locked_app.req_account_updates(subscribe, acct_code)?;
        locked_app.stream.as_mut().unwrap().read_to_end(&mut buf)?;

        let expected: [u8; 17] = [0, 0, 0, 13, 54, 0, 50, 0, 49, 0, 68, 49, 50, 51, 52, 53, 0];

        let msg_data = read_msg(buf.as_slice())?;
        let fields = read_fields(&msg_data.1);
        assert_eq!(expected.as_ref(), buf.as_slice());
        assert_eq!(
            OutgoingMessageIds::ReqAcctData as u8,
            fields[0].parse::<u8>().unwrap()
        );
        assert_eq!(version, fields[1].parse::<i32>().unwrap());
        assert_eq!(subscribe as i32, fields[2].parse::<i32>().unwrap());
        assert_eq!(acct_code, fields[3]);

        Ok(())
    }

    //------------------------------------------------------------------------------------------------
    #[test]
    fn test_req_account_updates_multi() -> Result<(), IBKRApiLibError> {
        let wrapper = Arc::new(Mutex::new(DummyTestWrapper::new()));
        let app = Arc::new(Mutex::new(EClient::<DummyTestWrapper>::new(
            wrapper,
        )));

        let version = 1;
        let req_id = 101;
        let acct_code = "D12345";
        let model_code = "ABC";
        let ledger_and_nvl = true;
        let mut buf = Vec::<u8>::new();

        let mut locked_app = app.lock().expect("EClient mutex was poisoned");

        locked_app.connect_test();
        locked_app.req_account_updates_multi(req_id, acct_code, model_code, ledger_and_nvl)?;
        locked_app.stream.as_mut().unwrap().read_to_end(&mut buf)?;

        let expected: [u8; 26] = [
            0, 0, 0, 22, 55, 54, 0, 49, 0, 49, 48, 49, 0, 68, 49, 50, 51, 52, 53, 0, 65, 66, 67, 0,
            49, 0,
        ];

        let msg_data = read_msg(buf.as_slice())?;

        let fields = read_fields(&msg_data.1);
        assert_eq!(expected.as_ref(), buf.as_slice());
        assert_eq!(
            OutgoingMessageIds::ReqAccountUpdatesMulti as u8,
            fields[0].parse::<u8>().unwrap()
        );
        assert_eq!(version, fields[1].parse::<i32>().unwrap());
        assert_eq!(req_id, fields[2].parse::<i32>().unwrap());
        assert_eq!(acct_code, fields[3]);
        assert_eq!(model_code, fields[4]);
        assert_eq!(ledger_and_nvl as i32, fields[5].parse::<i32>().unwrap());

        Ok(())
    }

    //------------------------------------------------------------------------------------------------
    #[test]
    fn test_req_all_open_orders() -> Result<(), IBKRApiLibError> {
        let wrapper = Arc::new(Mutex::new(DummyTestWrapper::new()));
        let app = Arc::new(Mutex::new(EClient::<DummyTestWrapper>::new(
            wrapper,
        )));

        let version = 1;

        let mut buf = Vec::<u8>::new();

        let mut locked_app = app.lock().expect("EClient mutex was poisoned");

        locked_app.connect_test();
        locked_app.req_all_open_orders()?;
        locked_app.stream.as_mut().unwrap().read_to_end(&mut buf)?;

        let expected: [u8; 9] = [0, 0, 0, 5, 49, 54, 0, 49, 0];

        let msg_data = read_msg(buf.as_slice())?;
        let fields = read_fields(&msg_data.1);

        assert_eq!(expected.as_ref(), buf.as_slice());
        assert_eq!(
            OutgoingMessageIds::ReqAllOpenOrders as u8,
            fields[0].parse::<u8>().unwrap()
        );
        assert_eq!(version, fields[1].parse::<i32>().unwrap());

        Ok(())
    }

    //------------------------------------------------------------------------------------------------
    #[test]
    fn test_req_auto_open_orders() -> Result<(), IBKRApiLibError> {
        let wrapper = Arc::new(Mutex::new(DummyTestWrapper::new()));
        let app = Arc::new(Mutex::new(EClient::<DummyTestWrapper>::new(
            wrapper,
        )));

        let version = 1;
        let auto_bind = true;
        let mut buf = Vec::<u8>::new();

        let mut locked_app = app.lock().expect("EClient mutex was poisoned");

        locked_app.connect_test();
        locked_app.req_auto_open_orders(auto_bind)?;
        locked_app.stream.as_mut().unwrap().read_to_end(&mut buf)?;

        let expected: [u8; 11] = [0, 0, 0, 7, 49, 53, 0, 49, 0, 49, 0];

        let msg_data = read_msg(buf.as_slice())?;

        let fields = read_fields(&msg_data.1);

        assert_eq!(expected.as_ref(), buf.as_slice());
        assert_eq!(
            OutgoingMessageIds::ReqAutoOpenOrders as u8,
            fields[0].parse::<u8>().unwrap()
        );
        assert_eq!(version, fields[1].parse::<i32>().unwrap());

        Ok(())
    }

    //------------------------------------------------------------------------------------------------
    #[test]
    fn test_req_completed_orders() -> Result<(), IBKRApiLibError> {
        let wrapper = Arc::new(Mutex::new(DummyTestWrapper::new()));
        let app = Arc::new(Mutex::new(EClient::<DummyTestWrapper>::new(
            wrapper,
        )));

        let version = 1;
        let api_only = true;
        let mut buf = Vec::<u8>::new();

        let mut locked_app = app.lock().expect("EClient mutex was poisoned");

        locked_app.connect_test();
        locked_app.req_completed_orders(api_only)?;
        locked_app.stream.as_mut().unwrap().read_to_end(&mut buf)?;

        let expected: [u8; 9] = [0, 0, 0, 5, 57, 57, 0, 49, 0];

        let msg_data = read_msg(buf.as_slice())?;

        let fields = read_fields(&msg_data.1);

        assert_eq!(expected.as_ref(), buf.as_slice());
        assert_eq!(
            OutgoingMessageIds::ReqCompletedOrders as u8,
            fields[0].parse::<u8>().unwrap()
        );
        assert_eq!(version, fields[1].parse::<i32>().unwrap());

        Ok(())
    }

    //------------------------------------------------------------------------------------------------
    #[test]
    fn test_req_contract_details() -> Result<(), IBKRApiLibError> {
        let wrapper = Arc::new(Mutex::new(DummyTestWrapper::new()));
        let app = Arc::new(Mutex::new(EClient::<DummyTestWrapper>::new(
            wrapper,
        )));

        let version = 8;
        let req_id = 102;
        let mut buf = Vec::<u8>::new();

        let mut locked_app = app.lock().expect("EClient mutex was poisoned");

        locked_app.connect_test();
        let contract = simple_future();
        locked_app.req_contract_details(req_id, &contract)?;
        locked_app.stream.as_mut().unwrap().read_to_end(&mut buf)?;

        let expected: [u8; 50] = [
            0, 0, 0, 46, 57, 0, 56, 0, 49, 48, 50, 0, 48, 0, 69, 83, 0, 70, 85, 84, 0, 50, 48, 50,
            48, 48, 57, 0, 48, 0, 0, 0, 71, 76, 79, 66, 69, 88, 0, 0, 85, 83, 68, 0, 0, 0, 48, 0,
            0, 0,
        ];

        let msg_data = read_msg(buf.as_slice())?;
        let fields = read_fields(&msg_data.1);

        assert_eq!(expected.as_ref(), buf.as_slice());
        assert_eq!(
            OutgoingMessageIds::ReqContractData as u8,
            fields[0].parse::<u8>().unwrap()
        );
        assert_eq!(version, fields[1].parse::<i32>().unwrap());
        assert_eq!(req_id, fields[2].parse::<i32>().unwrap());
        assert_eq!(contract.con_id, fields[3].parse::<i32>().unwrap()); // srv v37 and above
        assert_eq!(contract.symbol, fields[4]);

        assert_eq!(contract.sec_type, fields[5]);
        assert_eq!(contract.last_trade_date_or_contract_month, fields[6]);
        assert_eq!(contract.strike, fields[7].parse::<f64>().unwrap());
        assert_eq!(contract.right, fields[8]);
        assert_eq!(contract.multiplier, fields[9]); // srv v15 and above

        assert_eq!(contract.exchange, fields[10]);
        assert_eq!(contract.primary_exchange, fields[11]);

        assert_eq!(contract.currency, fields[12]);
        assert_eq!(contract.local_symbol, fields[13]);

        assert_eq!(contract.trading_class, fields[14]);
        assert_eq!(
            contract.include_expired as i32,
            fields[15].parse::<i32>().unwrap()
        ); // srv v31 and above

        assert_eq!(contract.sec_id_type, fields[16]);
        assert_eq!(contract.sec_id, fields[17]);

        Ok(())
    }

    //------------------------------------------------------------------------------------------------
    #[test]
    fn test_req_current_time() -> Result<(), IBKRApiLibError> {
        let wrapper = Arc::new(Mutex::new(DummyTestWrapper::new()));
        let app = Arc::new(Mutex::new(EClient::<DummyTestWrapper>::new(
            wrapper,
        )));

        let version = 2;

        let mut buf = Vec::<u8>::new();

        let mut locked_app = app.lock().expect("EClient mutex was poisoned");

        locked_app.connect_test();
        locked_app.req_current_time()?;
        locked_app.stream.as_mut().unwrap().read_to_end(&mut buf)?;

        let expected: [u8; 9] = [0, 0, 0, 5, 52, 57, 0, 50, 0];

        let msg_data = read_msg(buf.as_slice())?;

        let fields = read_fields(&msg_data.1);

        assert_eq!(expected.as_ref(), buf.as_slice());
        assert_eq!(
            OutgoingMessageIds::ReqCurrentTime as u8,
            fields[0].parse::<u8>().unwrap()
        );
        assert_eq!(version, fields[1].parse::<i32>().unwrap());

        Ok(())
    }

    //------------------------------------------------------------------------------------------------
    #[test]
    fn test_req_executions() -> Result<(), IBKRApiLibError> {
        let wrapper = Arc::new(Mutex::new(DummyTestWrapper::new()));
        let app = Arc::new(Mutex::new(EClient::<DummyTestWrapper>::new(
            wrapper,
        )));

        let version = 3;
        let req_id = 102;
        let client_id = 0;
        let acct_code = "D54321";

        //Time from which the executions will be returned yyyymmdd hh:mm:ss Only those executions reported after the specified time will be returned.
        let time = "";
        let symbol = "ES";
        let sec_type = "FUT";
        let exchange = "GLOBEX";
        let side = "BUY";
        let mut buf = Vec::<u8>::new();

        let mut locked_app = app.lock().expect("EClient mutex was poisoned");

        locked_app.connect_test();

        let exec_filter = ExecutionFilter::new(
            client_id,
            acct_code.to_string(),
            time.to_string(),
            symbol.to_string(),
            sec_type.to_string(),
            exchange.to_string(),
            side.to_string(),
        );
        locked_app.req_executions(req_id, &exec_filter)?;
        locked_app.stream.as_mut().unwrap().read_to_end(&mut buf)?;

        let expected: [u8; 40] = [
            0, 0, 0, 36, 55, 0, 51, 0, 49, 48, 50, 0, 48, 0, 68, 53, 52, 51, 50, 49, 0, 0, 69, 83,
            0, 70, 85, 84, 0, 71, 76, 79, 66, 69, 88, 0, 66, 85, 89, 0,
        ];

        let msg_data = read_msg(buf.as_slice())?;
        //println!("read message: {:?}", read_msg(buf.as_slice())?);
        let fields = read_fields(&msg_data.1);
        //println!("read fields: {:?}", read_fields(&msg_data.1));
        assert_eq!(expected.as_ref(), buf.as_slice());
        assert_eq!(
            OutgoingMessageIds::ReqExecutions as u8,
            fields[0].parse::<u8>().unwrap()
        );
        assert_eq!(version, fields[1].parse::<i32>().unwrap());
        assert_eq!(req_id, fields[2].parse::<i32>().unwrap());
        assert_eq!(client_id, fields[3].parse::<i32>().unwrap());
        assert_eq!(acct_code, fields[4]);
        assert_eq!(time, fields[5]);
        assert_eq!(symbol, fields[6]);
        assert_eq!(sec_type, fields[7]);
        assert_eq!(exchange, fields[8]);
        assert_eq!(side, fields[9]);

        Ok(())
    }
}
