use std::error::Error;
use std::io::Write;
use std::marker::Sync;
use std::net::TcpStream;
use std::num::ParseIntError;
use std::ops::{Deref, DerefMut};
use std::slice::Iter;
use std::str::FromStr;
use std::string::ToString;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};
use std::{thread, u8};

use bytebuffer::ByteBuffer;
use from_ascii::{FromAscii, FromAsciiRadix};
use num_traits::FromPrimitive;

use crate::client;
use crate::client::client::{ConnStatus, EClient};
use crate::client::common::{
    CommissionReport, TagValue, TickAttrib, TickType, MAX_MSG_LEN, NO_VALID_ID, UNSET_DOUBLE,
    UNSET_INTEGER,
};
use crate::client::contract::{Contract, ContractDetails};
use crate::client::errors::{IBKRApiLibError, TwsError};
use crate::client::messages::{read_fields, IncomingMessageIds};
use crate::client::order::{Order, OrderState};
use crate::client::order_decoder::OrderDecoder;
use crate::client::server_versions::{
    MIN_SERVER_VER_AGG_GROUP, MIN_SERVER_VER_MARKET_RULES, MIN_SERVER_VER_MD_SIZE_MULTIPLIER,
    MIN_SERVER_VER_PAST_LIMIT, MIN_SERVER_VER_PRE_OPEN_BID_ASK,
};
use crate::client::wrapper::Wrapper;

const SEP: u8 = '\0' as u8;
const EMPTY_LENGTH_HEADER: [u8; 4] = [0; 4];

pub fn decode_i32(iter: &mut Iter<String>) -> Result<i32, IBKRApiLibError> {
    let val: i32 = iter.next().unwrap().parse()?;
    Ok(val)
}

pub fn decode_i32_show_unset(iter: &mut Iter<String>) -> Result<i32, IBKRApiLibError> {
    let retval: i32 = iter.next().unwrap().parse()?;
    Ok(if retval == 0 { UNSET_INTEGER } else { retval })
}

pub fn decode_f64(iter: &mut Iter<String>) -> Result<f64, IBKRApiLibError> {
    let val = iter.next().unwrap().parse()?;
    Ok(val)
}

pub fn decode_f64_show_unset(iter: &mut Iter<String>) -> Result<f64, IBKRApiLibError> {
    let retval: f64 = iter.next().unwrap().parse()?;
    Ok(if retval == 0.0 { UNSET_DOUBLE } else { retval })
}

pub fn decode_string(iter: &mut Iter<String>) -> Result<String, IBKRApiLibError> {
    let val = iter.next().unwrap().to_string();
    Ok(val)
}

pub fn decode_bool(iter: &mut Iter<String>) -> Result<bool, IBKRApiLibError> {
    let retval: i32 = iter.next().unwrap().parse()?;
    Ok(retval != 0)
}

pub struct Decoder<T: Wrapper> {
    msg_queue: Receiver<String>,
    pub wrapper: Arc<Mutex<T>>,
    pub server_version: i32,
    conn_state: Arc<Mutex<ConnStatus>>,
}

impl<T> Decoder<T>
where
    T: Wrapper + Sync,
{
    pub fn new(
        the_wrapper: Arc<Mutex<T>>,
        msg_queue: Receiver<String>,
        server_version: i32,
        conn_state: Arc<Mutex<ConnStatus>>,
    ) -> Self {
        Decoder {
            wrapper: the_wrapper,
            msg_queue: msg_queue,
            server_version,
            conn_state,
        }
    }

    pub fn interpret(&mut self, fields: &[String]) {
        if fields.is_empty() {
            return;
        }
        for field in fields {
            debug!("inside interpret: {:?}", field);
        }
        let msg_id = i32::from_str(fields.get(0).unwrap().as_str()).unwrap();

        match FromPrimitive::from_i32(msg_id) {
            Some(IncomingMessageIds::TickPrice) => self.process_tick_price(fields),
            Some(IncomingMessageIds::AccountSummary) => self.process_account_summary(fields),
            Some(IncomingMessageIds::AccountSummaryEnd) => self.process_account_summary_end(fields),
            Some(IncomingMessageIds::AccountUpdateMulti) => {
                self.process_account_update_multi(fields)
            }
            Some(IncomingMessageIds::AccountUpdateMultiEnd) => {
                self.process_account_update_multi_end(fields)
            }
            Some(IncomingMessageIds::AcctDownloadEnd) => self.process_account_download_end(fields),
            Some(IncomingMessageIds::AcctUpdateTime) => self.process_account_update_time(fields),
            Some(IncomingMessageIds::AcctValue) => self.process_account_value(fields),
            Some(IncomingMessageIds::BondContractData) => self.process_bond_contract_data(fields),
            Some(IncomingMessageIds::CommissionReport) => self.process_commission_report(fields),
            Some(IncomingMessageIds::CompletedOrder) => self.process_completed_order(fields),
            Some(IncomingMessageIds::CompletedOrdersEnd) => {
                self.process_complete_orders_end(fields)
            }
            Some(IncomingMessageIds::ContractData) => self.process_contract_data(fields),
            Some(IncomingMessageIds::ContractDataEnd) => self.process_contract_data_end(fields),
            Some(IncomingMessageIds::CurrentTime) => self.process_current_time(fields),
            Some(IncomingMessageIds::DeltaNeutralValidation) => {
                self.process_delta_neutral_validation(fields)
            }
            Some(IncomingMessageIds::DisplayGroupList) => self.process_display_group_list(fields),
            Some(IncomingMessageIds::DisplayGroupUpdated) => {
                self.process_display_group_updated(fields)
            }
            Some(IncomingMessageIds::ErrMsg) => self.process_error_message(fields),
            Some(IncomingMessageIds::ExecutionData) => self.process_execution_data(fields),
            Some(IncomingMessageIds::ExecutionDataEnd) => self.process_execution_data_end(fields),
            Some(IncomingMessageIds::FamilyCodes) => self.process_family_codes(fields),
            Some(IncomingMessageIds::FundamentalData) => self.process_fundamental_data(fields),
            Some(IncomingMessageIds::HeadTimestamp) => self.process_head_timestamp(fields),
            Some(IncomingMessageIds::HistogramData) => self.process_histogram_data(fields),
            Some(IncomingMessageIds::HistoricalData) => self.process_historical_data(fields),
            Some(IncomingMessageIds::HistoricalDataUpdate) => self.process_historical_data(fields),
            Some(IncomingMessageIds::HistoricalNews) => self.process_historical_news(fields),
            Some(IncomingMessageIds::HistoricalNewsEnd) => self.process_historical_news_end(fields),
            Some(IncomingMessageIds::HistoricalTicks) => self.process_historical_ticks(fields),
            Some(IncomingMessageIds::HistoricalTicksBidAsk) => {
                self.process_historical_ticks_bid_ask(fields)
            }
            Some(IncomingMessageIds::HistoricalTicksLast) => {
                self.process_historical_ticks_last(fields)
            }
            Some(IncomingMessageIds::ManagedAccts) => self.process_managed_accounts(fields),
            Some(IncomingMessageIds::MarketDataType) => self.process_market_data_type(fields),
            Some(IncomingMessageIds::MarketDepth) => self.process_market_depth(fields),
            Some(IncomingMessageIds::MarketDepthL2) => self.process_market_depth_l2(fields),
            Some(IncomingMessageIds::MarketRule) => self.process_market_rule(fields),
            Some(IncomingMessageIds::MktDepthExchanges) => {
                self.process_market_depth_exchanges(fields)
            }
            Some(IncomingMessageIds::NewsArticle) => self.process_news_article(fields),
            Some(IncomingMessageIds::NewsBulletins) => self.process_news_bulletins(fields),
            Some(IncomingMessageIds::NewsProviders) => self.process_news_providers(fields),
            Some(IncomingMessageIds::NextValidId) => self.process_next_valid_id(fields),
            Some(IncomingMessageIds::OpenOrder) => self.process_open_order(fields),
            Some(IncomingMessageIds::OpenOrderEnd) => self.process_open_order_end(fields),
            Some(IncomingMessageIds::OrderStatus) => self.process_order_status(fields),
            Some(IncomingMessageIds::OrderBound) => self.process_order_bound(fields),
            Some(IncomingMessageIds::Pnl) => self.process_pnl(fields),
            Some(IncomingMessageIds::PnlSingle) => self.process_pnl_single(fields),
            Some(IncomingMessageIds::PortfolioValue) => self.process_portfolio_value(fields),
            Some(IncomingMessageIds::PositionData) => self.process_position_data(fields),
            Some(IncomingMessageIds::PositionEnd) => self.process_position_end(fields),
            Some(IncomingMessageIds::RealTimeBars) => self.process_real_time_bars(fields),
            Some(IncomingMessageIds::ReceiveFa) => self.process_receive_fa(fields),
            Some(IncomingMessageIds::RerouteMktDataReq) => {
                self.process_reroute_mkt_data_req(fields)
            }
            Some(IncomingMessageIds::PositionMulti) => self.process_position_multi(fields),
            Some(IncomingMessageIds::PositionMultiEnd) => self.process_position_multi_end(fields),
            Some(IncomingMessageIds::ScannerData) => self.process_scanner_data(fields),
            Some(IncomingMessageIds::ScannerParameters) => self.process_scanner_parameters(fields),
            Some(IncomingMessageIds::SecurityDefinitionOptionParameter) => {
                self.process_security_definition_option_parameter(fields)
            }
            Some(IncomingMessageIds::SecurityDefinitionOptionParameterEnd) => {
                self.process_security_definition_option_parameter_end(fields)
            }
            Some(IncomingMessageIds::SmartComponents) => self.process_smart_components(fields),
            Some(IncomingMessageIds::SoftDollarTiers) => self.process_soft_dollar_tiers(fields),
            Some(IncomingMessageIds::SymbolSamples) => self.process_symbol_samples(fields),
            Some(IncomingMessageIds::TickByTick) => self.process_tick_by_tick(fields),
            Some(IncomingMessageIds::TickEfp) => self.process_tick_by_tick(fields),
            Some(IncomingMessageIds::TickGeneric) => self.process_tick_generic_news(fields),
            Some(IncomingMessageIds::TickNews) => self.process_tick_news(fields),
            Some(IncomingMessageIds::TickOptionComputation) => {
                self.process_tick_option_computation(fields)
            }
            Some(IncomingMessageIds::TickReqParams) => self.process_tick_teq_params(fields),
            Some(IncomingMessageIds::TickSize) => self.process_tick_size(fields),
            Some(IncomingMessageIds::TickSnapshotEnd) => self.process_tick_snapshot_end(fields),
            Some(IncomingMessageIds::TickString) => self.process_tick_string(fields),
            Some(IncomingMessageIds::VerifyAndAuthCompleted) => {
                self.process_verify_and_auth_completed(fields)
            }
            Some(IncomingMessageIds::VerifyCompleted) => self.process_verify_completed(fields),

            Some(IncomingMessageIds::VerifyMessageApi) => self.process_verify_completed(fields),

            Some(IncomingMessageIds::VerifyAndAuthMessageApi) => {
                self.process_verify_and_auth_message_api(fields)
            }

            _ => {}
        }
    }

    fn process_tick_price(&mut self, fields: &[String]) {
        let mut fields_itr = fields.iter();

        //throw away message_id
        fields_itr.next();
        //throw away version
        fields_itr.next();

        let req_id = fields_itr.next().unwrap().parse().unwrap();
        let ticker_id: i32 = fields_itr.next().unwrap().parse().unwrap();
        let tick_type: i32 = fields_itr.next().unwrap().parse().unwrap();
        let price: f64 = fields_itr.next().unwrap().parse().unwrap();
        let mut size = fields_itr.next().unwrap().parse().unwrap();
        let mut attr_mask: i32 = fields_itr.next().unwrap().parse().unwrap();
        let mut tick_arrtibute = TickAttrib::new(false, false, false);

        tick_arrtibute.can_auto_execute = attr_mask == 1;

        if self.server_version >= MIN_SERVER_VER_PAST_LIMIT {
            tick_arrtibute.can_auto_execute = attr_mask & 1 != 0;
            tick_arrtibute.past_limit = attr_mask & 2 != 0;
        }
        if self.server_version >= MIN_SERVER_VER_PRE_OPEN_BID_ASK {
            tick_arrtibute.pre_open = attr_mask & 4 != 0;
        }
        self.wrapper.lock().unwrap().deref_mut().tick_price(
            req_id,
            FromPrimitive::from_i32(tick_type).unwrap(),
            price,
            tick_arrtibute,
        );

        // process ver 2 fields

        let size_tick_type = match FromPrimitive::from_i32(tick_type) {
            Some(TickType::Bid) => TickType::BidSize,
            Some(TickType::Ask) => TickType::AskSize,
            Some(TickType::Last) => TickType::LastSize,
            Some(TickType::DelayedBid) => TickType::DelayedBidSize,
            Some(TickType::DelayedAsk) => TickType::DelayedAskSize,
            Some(TickType::DelayedLast) => TickType::DelayedLastSize,
            _ => TickType::NotSet,
        };

        if size_tick_type as i32 != TickType::NotSet as i32 {
            self.wrapper
                .lock()
                .unwrap()
                .deref_mut()
                .tick_size(req_id, size_tick_type, size);
        }
    }

    fn process_tick_string(&mut self, fields: &[String]) {
        let mut fields_itr = fields.iter();

        //throw away message_id
        fields_itr.next();
        //throw away version
        fields_itr.next();

        let req_id: i32 = fields_itr.next().unwrap().parse().unwrap();
        let tick_type: i32 = fields_itr.next().unwrap().parse().unwrap();
        let value = fields_itr.next().unwrap();

        self.wrapper.lock().unwrap().deref_mut().tick_string(
            req_id,
            FromPrimitive::from_i32(tick_type).unwrap(),
            value,
        )
    }

    fn process_account_summary(&mut self, fields: &[String]) {
        self.wrapper.lock().unwrap().deref_mut().account_summary(
            fields.get(2).unwrap().parse().unwrap(),
            fields.get(3).unwrap(),
            fields.get(4).unwrap(),
            fields.get(5).unwrap(),
            fields.get(6).unwrap(),
        );
    }
    fn process_account_summary_end(&mut self, fields: &[String]) {
        self.wrapper
            .lock()
            .unwrap()
            .deref_mut()
            .account_summary_end(fields.get(2).unwrap().parse().unwrap())
    }

    fn process_account_update_multi(&mut self, fields: &[String]) {
        let mut fields_itr = fields.iter();

        //throw away message_id
        fields_itr.next();
        //throw away version
        fields_itr.next();

        let req_id: i32 = fields_itr.next().unwrap().parse().unwrap();
        let account = fields_itr.next().unwrap();
        let model_code = fields_itr.next().unwrap();
        let key = fields_itr.next().unwrap();
        let value = fields_itr.next().unwrap();
        let currency = fields_itr.next().unwrap();

        self.wrapper
            .lock()
            .unwrap()
            .deref_mut()
            .account_update_multi(req_id, account, model_code, key, value, currency);
    }
    fn process_account_update_multi_end(&mut self, fields: &[String]) {
        let mut fields_itr = fields.iter();

        //throw away message_id
        fields_itr.next();
        //throw away version
        fields_itr.next();

        let req_id: i32 = fields_itr.next().unwrap().parse().unwrap();

        self.wrapper
            .lock()
            .unwrap()
            .deref_mut()
            .account_update_multi_end(req_id);
    }
    fn process_account_download_end(&mut self, fields: &[String]) {
        self.wrapper
            .lock()
            .unwrap()
            .deref_mut()
            .account_download_end(fields.get(1).unwrap())
    }
    fn process_account_update_time(&mut self, fields: &[String]) {
        self.wrapper
            .lock()
            .unwrap()
            .deref_mut()
            .update_account_time(fields.get(1).unwrap())
    }
    fn process_account_value(&mut self, fields: &[String]) {
        self.wrapper
            .lock()
            .unwrap()
            .deref_mut()
            .update_account_value(
                fields.get(1).unwrap(),
                fields.get(2).unwrap(),
                fields.get(3).unwrap(),
                fields.get(4).unwrap(),
            );
    }

    fn process_bond_contract_data(&mut self, fields: &[String]) {
        let mut fields_itr = fields.iter();

        //throw away message_id
        fields_itr.next();

        let version: i32 = fields_itr.next().unwrap().parse().unwrap();

        let mut req_id = -1;
        if (version >= 3) {
            req_id = fields_itr.next().unwrap().parse().unwrap();
        }

        let mut contract = ContractDetails::default();

        contract.contract.symbol = fields_itr.next().unwrap().parse().unwrap();
        contract.contract.sec_type = fields_itr.next().unwrap().parse().unwrap();
        contract.cusip = fields_itr.next().unwrap().parse().unwrap();
        contract.coupon = fields_itr.next().unwrap().parse().unwrap();
        self.read_last_trade_date(&mut contract, true, fields_itr.next().unwrap());
        contract.issue_date = fields_itr.next().unwrap().parse().unwrap();
        contract.ratings = fields_itr.next().unwrap().parse().unwrap();
        contract.bond_type = fields_itr.next().unwrap().parse().unwrap();
        contract.coupon_type = fields_itr.next().unwrap().parse().unwrap();
        contract.convertible = i32::from_str(fields_itr.next().unwrap().as_ref()).unwrap() != 0;
        contract.callable = i32::from_str(fields_itr.next().unwrap().as_ref()).unwrap() != 0;
        contract.putable = i32::from_str(fields_itr.next().unwrap().as_ref()).unwrap() != 0;
        contract.desc_append = fields_itr.next().unwrap().parse().unwrap();
        contract.contract.exchange = fields_itr.next().unwrap().parse().unwrap();
        contract.contract.currency = fields_itr.next().unwrap().parse().unwrap();
        contract.market_name = fields_itr.next().unwrap().parse().unwrap();
        contract.contract.trading_class = fields_itr.next().unwrap().parse().unwrap();
        contract.contract.con_id = fields_itr.next().unwrap().parse().unwrap();
        contract.min_tick = fields_itr.next().unwrap().parse().unwrap();
        if (self.server_version >= MIN_SERVER_VER_MD_SIZE_MULTIPLIER) {
            contract.md_size_multiplier = fields_itr.next().unwrap().parse().unwrap();
        }
        contract.order_types = fields_itr.next().unwrap().parse().unwrap();
        contract.valid_exchanges = fields_itr.next().unwrap().parse().unwrap();
        if (version >= 2) {
            contract.next_option_date = fields_itr.next().unwrap().parse().unwrap();
            contract.next_option_type = fields_itr.next().unwrap().parse().unwrap();
            contract.next_option_partial = fields_itr.next().unwrap().parse().unwrap();
            contract.notes = fields_itr.next().unwrap().parse().unwrap();
        }
        if (version >= 4) {
            contract.long_name = fields_itr.next().unwrap().parse().unwrap();
        }
        if (version >= 6) {
            contract.ev_rule = fields_itr.next().unwrap().parse().unwrap();
            contract.ev_multiplier = fields_itr.next().unwrap().parse().unwrap();
        }
        if (version >= 5) {
            let sec_id_list_count = fields_itr.next().unwrap().parse().unwrap();
            if (sec_id_list_count > 0) {
                contract.sec_id_list = vec![];
                for _ in 0..sec_id_list_count {
                    contract.sec_id_list.push(TagValue::new(
                        fields_itr.next().unwrap().parse().unwrap(),
                        fields_itr.next().unwrap().parse().unwrap(),
                    ));
                }
            }
        }
        if (self.server_version >= MIN_SERVER_VER_AGG_GROUP) {
            contract.agg_group = fields_itr.next().unwrap().parse().unwrap();
        }
        if (self.server_version >= MIN_SERVER_VER_MARKET_RULES) {
            contract.market_rule_ids = fields_itr.next().unwrap().parse().unwrap();
        }

        self.wrapper
            .lock()
            .unwrap()
            .deref_mut()
            .bond_contract_details(req_id, contract.clone());
    }

    fn process_commission_report(&mut self, fields: &[String]) {
        let mut fields_itr = fields.iter();

        //throw away message_id
        fields_itr.next();

        let mut commission_report = CommissionReport::default();
        commission_report.exec_id = fields_itr.next().unwrap().to_string();
        commission_report.commission = fields_itr.next().unwrap().parse().unwrap();
        commission_report.currency = fields_itr.next().unwrap().to_string();
        commission_report.realized_pnl = fields_itr.next().unwrap().parse().unwrap();
        commission_report.yield_ = fields_itr.next().unwrap().parse().unwrap();
        commission_report.yield_redemption_date = fields_itr.next().unwrap().parse().unwrap();

        self.wrapper
            .lock()
            .unwrap()
            .deref_mut()
            .commission_report(commission_report);
    }
    fn process_completed_order(&mut self, fields: &[String]) {
        let mut fields_itr = fields.iter();

        //throw away message_id
        fields_itr.next();

        let mut contract = Contract::default();
        let mut order = Order::default();
        let mut order_state = OrderState::default();

        let mut order_decoder = OrderDecoder::new(
            &mut contract,
            &mut order,
            &mut order_state,
            UNSET_INTEGER,
            self.server_version,
        );

        order_decoder.decode(&mut fields_itr);

        self.wrapper
            .lock()
            .unwrap()
            .deref_mut()
            .completed_order(contract, order, order_state);
    }
    fn process_complete_orders_end(&mut self, fields: &[String]) {}
    fn process_contract_data(&mut self, fields: &[String]) {}
    fn process_contract_data_end(&mut self, fields: &[String]) {}
    fn process_current_time(&mut self, fields: &[String]) {
        print!("$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$  process_current_time");
        print!("{:?}", fields);
    }
    fn process_delta_neutral_validation(&mut self, fields: &[String]) {}
    fn process_display_group_list(&mut self, fields: &[String]) {}
    fn process_display_group_updated(&mut self, fields: &[String]) {}
    fn process_error_message(&mut self, fields: &[String]) {
        print!("{:?}", fields);

        self.wrapper.lock().unwrap().deref_mut().error(
            fields.get(2).unwrap().parse().unwrap(),
            fields.get(3).unwrap().parse().unwrap(),
            fields.get(4).unwrap(),
        )
    }
    fn process_execution_data(&mut self, fields: &[String]) {}
    fn process_execution_data_end(&mut self, fields: &[String]) {}
    fn process_family_codes(&mut self, fields: &[String]) {}
    fn process_fundamental_data(&mut self, fields: &[String]) {}
    fn process_head_timestamp(&mut self, fields: &[String]) {}
    fn process_histogram_data(&mut self, fields: &[String]) {}
    fn process_historical_data(&mut self, fields: &[String]) {}
    fn process_historical_data_update(&mut self, fields: &[String]) {}
    fn process_historical_news(&mut self, fields: &[String]) {}
    fn process_historical_news_end(&mut self, fields: &[String]) {}
    fn process_historical_ticks(&mut self, fields: &[String]) {}
    fn process_historical_ticks_bid_ask(&mut self, fields: &[String]) {}
    fn process_historical_ticks_last(&mut self, fields: &[String]) {}
    fn process_managed_accounts(&mut self, fields: &[String]) {}
    fn process_market_data_type(&mut self, fields: &[String]) {}
    fn process_market_depth(&mut self, fields: &[String]) {}
    fn process_market_depth_l2(&mut self, fields: &[String]) {}
    fn process_market_rule(&mut self, fields: &[String]) {}
    fn process_market_depth_exchanges(&mut self, fields: &[String]) {}
    fn process_news_article(&mut self, fields: &[String]) {}
    fn process_news_bulletins(&mut self, fields: &[String]) {}
    fn process_news_providers(&mut self, fields: &[String]) {}
    fn process_next_valid_id(&mut self, fields: &[String]) {}
    fn process_open_order(&mut self, fields: &[String]) {}
    fn process_open_order_end(&mut self, fields: &[String]) {}
    fn process_order_bound(&mut self, fields: &[String]) {}
    fn process_order_status(&mut self, fields: &[String]) {}
    fn process_pnl(&mut self, fields: &[String]) {}
    fn process_pnl_single(&mut self, fields: &[String]) {}
    fn process_portfolio_value(&mut self, fields: &[String]) {}
    fn process_position_data(&mut self, fields: &[String]) {}
    fn process_position_end(&mut self, fields: &[String]) {}
    fn process_real_time_bars(&mut self, fields: &[String]) {}
    fn process_receive_fa(&mut self, fields: &[String]) {}
    fn process_reroute_mkt_data_req(&mut self, fields: &[String]) {}
    fn process_reroute_mkt_depth_req(&mut self, fields: &[String]) {}
    fn process_position_multi(&mut self, fields: &[String]) {}
    fn process_position_multi_end(&mut self, fields: &[String]) {}
    fn process_scanner_data(&mut self, fields: &[String]) {}
    fn process_scanner_parameters(&mut self, fields: &[String]) {}
    fn process_security_definition_option_parameter(&mut self, fields: &[String]) {}
    fn process_security_definition_option_parameter_end(&mut self, fields: &[String]) {}
    fn process_smart_components(&mut self, fields: &[String]) {}
    fn process_soft_dollar_tiers(&mut self, fields: &[String]) {}
    fn process_symbol_samples(&mut self, fields: &[String]) {}
    fn process_tick_by_tick(&mut self, fields: &[String]) {}
    fn process_tick_efp(&mut self, fields: &[String]) {}
    fn process_tick_generic_news(&mut self, fields: &[String]) {}
    fn process_tick_news(&mut self, fields: &[String]) {}
    fn process_tick_option_computation(&mut self, fields: &[String]) {}
    fn process_tick_teq_params(&mut self, fields: &[String]) {}
    fn process_tick_size(&mut self, fields: &[String]) {}
    fn process_tick_snapshot_end(&mut self, fields: &[String]) {}

    fn process_verify_and_auth_completed(&mut self, fields: &[String]) {}
    fn process_verify_and_auth_message_api(&mut self, fields: &[String]) {}
    fn process_verify_completed(&mut self, fields: &[String]) {}
    fn process_verify_message_api(&mut self, fields: &[String]) {}

    //==============================================================================================
    fn read_last_trade_date(&self, contract: &mut ContractDetails, is_bond: bool, read_date: &str) {
        if read_date != "" {
            let splitted = read_date.split_whitespace().collect::<Vec<&str>>();
            if splitted.len() > 0 {
                if (is_bond) {
                    contract.maturity = splitted.get(0).unwrap().to_string();
                } else {
                    contract.contract.last_trade_date_or_contract_month =
                        splitted.get(0).unwrap().to_string();
                }
            }
            if (splitted.len() > 1) {
                contract.last_trade_time = splitted.get(1).unwrap().to_string();
            }
            if is_bond && splitted.len() > 2 {
                contract.time_zone_id = splitted.get(2).unwrap().to_string();
            }
        }
    }

    //==============================================================================================
    pub fn run(&mut self) {
        //This is the function that has the message loop.

        info!("Starting run...");
        // !self.done &&
        while true {
            info!("Client waiting for message...");

            {
                let text = self.msg_queue.recv();

                match text {
                    Result::Ok(val) => {
                        if val.len() > MAX_MSG_LEN as usize {
                            self.wrapper.lock().unwrap().deref_mut().error(
                                NO_VALID_ID,
                                TwsError::NotConnected.code(),
                                format!(
                                    "{}:{}:{}",
                                    TwsError::NotConnected.message(),
                                    val.len(),
                                    val
                                )
                                .as_str(),
                            );
                            self.wrapper.lock().unwrap().deref_mut().connection_closed();
                            *self.conn_state.lock().unwrap().deref_mut() = ConnStatus::DISCONNECTED;
                            break;
                        } else {
                            let fields = read_fields((&val).as_ref());

                            self.interpret(fields.as_slice());
                        }
                    }
                    Result::Err(err) => {
                        error!("Error receiving message.  Disconnected: {:?}", err);
                        self.wrapper.lock().unwrap().deref_mut().connection_closed();
                        *self.conn_state.lock().unwrap().deref_mut() = ConnStatus::DISCONNECTED;
                        break;
                    }
                }
            }
        }
    }
}
