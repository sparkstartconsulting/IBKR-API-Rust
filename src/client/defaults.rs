use crate::client::common::{
    BarData, CommissionReport, Contract, ContractDescription, ContractDetails,
    DeltaNeutralContract, DepthExchanges, Execution, FaDataType, FamilyCode, HistogramData,
    HistoricalTick, HistoricalTickBidAsk, NewsProvider, Order, OrderState, PriceIncrement,
    SimpleEntry, SoftDollarTier, TickAttrib, TickAttribBidAsk, TickAttribLast,
};
use crate::client::wrapper::Wrapper;
use std::collections::hash_map::RandomState;
use std::collections::{HashMap, HashSet};

pub struct DefaultWrapper {}

impl DefaultWrapper {
    pub fn new() -> Self {
        DefaultWrapper {}
    }
}
impl Wrapper for DefaultWrapper {
    fn error(&self, req_id: i32, error_code: i32, error_string: &str) {
        error!("Code: {} , Message:{}", error_code, error_string);
        println!("Code: {} , Message:{}", error_code, error_string);
    }

    fn win_error(&self, text: &str, last_error: i32) {
        unimplemented!()
    }

    fn connect_ack(&self) {
        info!("Connected.");
    }

    fn market_data_type(&self, req_id: i32, market_data_type: i32) {
        unimplemented!()
    }

    fn tick_price(&self, req_id: i32, tick_type: i32, price: f64, attrib: TickAttrib) {
        unimplemented!()
    }

    fn tick_size(&self, req_id: i32, tick_type: i32, size: i32) {
        unimplemented!()
    }

    fn tick_snapshot_end(&self, req_id: i32) {
        unimplemented!()
    }

    fn tick_generic(&self, req_id: i32, tick_type: i32, value: f64) {
        unimplemented!()
    }

    fn tick_string(&self, req_id: i32, tick_type: i32, value: &str) {
        unimplemented!()
    }

    fn tick_efp(
        &self,
        req_id: i32,
        tick_type: i32,
        basis_points: f64,
        formatted_basis_points: &str,
        total_dividends: f64,
        hold_days: i32,
        future_last_trade_date: &str,
        dividend_impact: f64,
        dividends_to_last_trade_date: f64,
    ) {
        unimplemented!()
    }

    fn order_status(
        &self,
        order_id: i32,
        status: &str,
        filled: f64,
        remaining: f64,
        avg_fill_price: f64,
        perm_id: i32,
        parent_id: i32,
        last_fill_price: f64,
        client_id: i32,
        why_held: &str,
        mkt_cap_price: f64,
    ) {
        unimplemented!()
    }

    fn open_order(&self, order_id: i32, contract: Contract, order: Order, order_state: OrderState) {
        unimplemented!()
    }

    fn open_order_end(&self) {
        unimplemented!()
    }

    fn connection_closed(&self) {
        unimplemented!()
    }

    fn update_account_value(&self, key: &str, val: &str, currency: &str, account_name: &str) {
        info!(
            "key: {}, value: {}, ccy: {}, account: {}.",
            key, val, currency, account_name
        );

        println!(
            "key: {}, value: {}, ccy: {}, account: {}.",
            key, val, currency, account_name
        );
    }

    fn update_portfolio(
        &self,
        contract: Contract,
        position: f64,
        market_price: f64,
        market_value: f64,
        average_cost: f64,
        unrealized_pnl: f64,
        realized_pnl: f64,
        account_name: &str,
    ) {
        unimplemented!()
    }

    fn update_account_time(&self, time_stamp: &str) {
        info!("update_account_time: {}.", time_stamp);
        println!("update_account_time: {}.", time_stamp);
    }

    fn account_download_end(&self, account_name: &str) {
        info!("account_download_end: {}.", account_name);
        println!("account_download_end: {}.", account_name);
    }

    fn next_valid_id(&self, order_id: i32) {
        unimplemented!()
    }

    fn contract_details(&self, req_id: i32, contract_details: ContractDetails) {
        unimplemented!()
    }

    fn bond_contract_details(&self, req_id: i32, contract_details: ContractDetails) {
        unimplemented!()
    }

    fn contract_details_end(&self, req_id: i32) {
        unimplemented!()
    }

    fn exec_details(&self, req_id: i32, contract: Contract, execution: Execution) {
        unimplemented!()
    }

    fn exec_details_end(&self, req_id: i32) {
        unimplemented!()
    }

    fn update_mkt_depth(
        &self,
        req_id: i32,
        position: i32,
        operation: i32,
        side: i32,
        price: f64,
        size: i32,
    ) {
        unimplemented!()
    }

    fn update_mkt_depth_l2(
        &self,
        req_id: i32,
        position: i32,
        market_maker: &str,
        operation: i32,
        side: i32,
        price: f64,
        size: i32,
        is_smart_depth: bool,
    ) {
        unimplemented!()
    }

    fn update_news_bulletin(
        &self,
        msg_id: i32,
        msg_type: i32,
        news_message: &str,
        origin_exch: &str,
    ) {
        unimplemented!()
    }

    fn managed_accounts(&self, accounts_list: &str) {
        unimplemented!()
    }

    fn receive_fa(&self, fa_data: FaDataType, cxml: &str) {
        unimplemented!()
    }

    fn historical_data(&self, req_id: i32, bar: BarData) {
        unimplemented!()
    }

    fn historical_data_end(&self, req_id: i32, start: &str, end: &str) {
        unimplemented!()
    }

    fn scanner_parameters(&self, xml: &str) {
        unimplemented!()
    }

    fn scanner_data(
        &self,
        req_id: i32,
        rank: i32,
        contract_details: ContractDetails,
        distance: &str,
        benchmark: &str,
        projection: &str,
        legs_str: &str,
    ) {
        unimplemented!()
    }

    fn scanner_data_end(&self, req_id: i32) {
        unimplemented!()
    }

    fn realtime_bar(
        &self,
        req_id: i32,
        time: i32,
        open_: f64,
        high: f64,
        low: f64,
        close: f64,
        volume: i32,
        wap: f64,
        count: i32,
    ) {
        unimplemented!()
    }

    fn current_time(&self, time: i32) {
        unimplemented!()
    }

    fn fundamental_data(&self, req_id: i32, data: &str) {
        unimplemented!()
    }

    fn delta_neutral_validation(&self, req_id: i32, delta_neutral_contract: DeltaNeutralContract) {
        unimplemented!()
    }

    fn commission_report(&self, commission_report: CommissionReport) {
        unimplemented!()
    }

    fn position(&self, account: &str, contract: Contract, position: f64, avg_cost: f64) {
        unimplemented!()
    }

    fn position_end(&self) {
        unimplemented!()
    }

    fn account_summary(&self, req_id: i32, account: &str, tag: &str, value: &str, currency: &str) {
        unimplemented!()
    }

    fn account_summary_end(&self, req_id: i32) {
        unimplemented!()
    }

    fn verify_message_api(&self, api_data: &str) {
        unimplemented!()
    }

    fn verify_completed(&self, is_successful: bool, error_text: &str) {
        unimplemented!()
    }

    fn verify_and_auth_message_api(&self, api_data: &str, xyz_challange: &str) {
        unimplemented!()
    }

    fn verify_and_auth_completed(&self, is_successful: bool, error_text: &str) {
        unimplemented!()
    }

    fn display_group_list(&self, req_id: i32, groups: &str) {
        unimplemented!()
    }

    fn display_group_updated(&self, req_id: i32, contract_info: &str) {
        unimplemented!()
    }

    fn position_multi(
        &self,
        req_id: i32,
        account: &str,
        model_code: &str,
        contract: Contract,
        pos: f64,
        avg_cost: f64,
    ) {
        unimplemented!()
    }

    fn position_multi_end(&self, req_id: i32) {
        unimplemented!()
    }

    fn account_update_multi(
        &self,
        req_id: i32,
        account: &str,
        model_code: &str,
        key: &str,
        value: &str,
        currency: &str,
    ) {
        unimplemented!()
    }

    fn account_update_multi_end(&self, req_id: i32) {
        unimplemented!()
    }

    fn tick_option_computation(
        &self,
        req_id: i32,
        tick_type: i32,
        implied_vol: f64,
        delta: f64,
        opt_price: f64,
        pv_dividend: f64,
        gamma: f64,
        vega: f64,
        theta: f64,
        und_price: f64,
    ) {
        unimplemented!()
    }

    fn security_definition_option_parameter(
        &self,
        req_id: i32,
        exchange: &str,
        underlying_con_id: i32,
        trading_class: &str,
        multiplier: &str,
        expirations: HashSet<String, RandomState>,
        strikes: HashSet<f64, RandomState>,
    ) {
        unimplemented!()
    }

    fn security_definition_option_parameter_end(&self, req_id: i32) {
        unimplemented!()
    }

    fn soft_dollar_tiers(&self, req_id: i32, tiers: Vec<SoftDollarTier>) {
        unimplemented!()
    }

    fn family_codes(&self, family_codes: Vec<FamilyCode>) {
        unimplemented!()
    }

    fn symbol_samples(&self, req_id: i32, contract_descriptions: Vec<ContractDescription>) {
        unimplemented!()
    }

    fn mkt_depth_exchanges(&self, depth_mkt_data_descriptions: Vec<DepthExchanges>) {
        unimplemented!()
    }

    fn tick_news(
        &self,
        ticker_id: i32,
        time_stamp: i32,
        provider_code: &str,
        article_id: &str,
        headline: &str,
        extra_data: &str,
    ) {
        unimplemented!()
    }

    fn smart_components(
        &self,
        req_id: i32,
        smart_component_map: HashMap<i32, SimpleEntry, RandomState>,
    ) {
        unimplemented!()
    }

    fn tick_req_params(
        &self,
        ticker_id: i32,
        min_tick: f64,
        bbo_exchange: &str,
        snapshot_permissions: i32,
    ) {
        unimplemented!()
    }

    fn news_providers(&self, news_providers: Vec<NewsProvider>) {
        unimplemented!()
    }

    fn news_article(&self, request_id: i32, article_type: i32, article_text: &str) {
        unimplemented!()
    }

    fn historical_news(
        &self,
        request_id: i32,
        time: &str,
        provider_code: &str,
        article_id: &str,
        headline: &str,
    ) {
        unimplemented!()
    }

    fn historical_news_end(&self, request_id: i32, has_more: bool) {
        unimplemented!()
    }

    fn head_timestamp(&self, req_id: i32, head_timestamp: &str) {
        unimplemented!()
    }

    fn histogram_data(&self, req_id: i32, items: HistogramData) {
        unimplemented!()
    }

    fn historical_data_update(&self, req_id: i32, bar: BarData) {
        unimplemented!()
    }

    fn reroute_mkt_data_req(&self, req_id: i32, con_id: i32, exchange: &str) {
        unimplemented!()
    }

    fn reroute_mkt_depth_req(&self, req_id: i32, con_id: i32, exchange: &str) {
        unimplemented!()
    }

    fn market_rule(&self, market_rule_id: i32, price_increments: Vec<PriceIncrement>) {
        unimplemented!()
    }

    fn pnl(&self, req_id: i32, daily_pn_l: f64, unrealized_pn_l: f64, realized_pn_l: f64) {
        unimplemented!()
    }

    fn pnl_single(
        &self,
        req_id: i32,
        pos: i32,
        daily_pn_l: f64,
        unrealized_pn_l: f64,
        realized_pn_l: f64,
        value: f64,
    ) {
        unimplemented!()
    }

    fn historical_ticks(&self, req_id: i32, ticks: Vec<HistoricalTick>, done: bool) {
        unimplemented!()
    }

    fn historical_ticks_bid_ask(&self, req_id: i32, ticks: Vec<HistoricalTickBidAsk>, done: bool) {
        unimplemented!()
    }

    fn tick_by_tick_all_last(
        &self,
        req_id: i32,
        tick_type: i32,
        time: i32,
        price: f64,
        size: i32,
        tick_attrib_last: TickAttribLast,
        exchange: &str,
        special_conditions: &str,
    ) {
        unimplemented!()
    }

    fn tick_by_tick_bid_ask(
        &self,
        req_id: i32,
        time: i32,
        bid_price: f64,
        ask_price: f64,
        bid_size: i32,
        ask_size: i32,
        tick_attrib_bid_ask: TickAttribBidAsk,
    ) {
        unimplemented!()
    }

    fn tick_by_tick_mid_point(&self, req_id: i32, time: i32, mid_point: f64) {
        unimplemented!()
    }

    fn order_bound(&self, req_id: i32, api_client_id: i32, api_order_id: i32) {
        unimplemented!()
    }

    fn completed_order(&self, contract: Contract, order: Order, order_state: OrderState) {
        unimplemented!()
    }

    fn completed_orders_end(&self) {
        unimplemented!()
    }
}
