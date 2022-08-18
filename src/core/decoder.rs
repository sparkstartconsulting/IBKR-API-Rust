//! Receives messages from Reader, decodes messages, and feeds them to Wrapper
use std::collections::HashSet;

use std::marker::Sync;
use std::ops::Deref;
use std::slice::Iter;
use std::str::FromStr;
use std::string::ToString;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};

use bigdecimal::BigDecimal;
use float_cmp::*;
use log::*;
use num_traits::float::FloatCore;
use num_traits::FromPrimitive;

use crate::core::client::ConnStatus;
use crate::core::common::{
    BarData, CommissionReport, DepthMktDataDescription, FamilyCode, HistogramData, HistoricalTick,
    HistoricalTickBidAsk, HistoricalTickLast, NewsProvider, PriceIncrement, RealTimeBar,
    SmartComponent, TagValue, TickAttrib, TickAttribBidAsk, TickAttribLast, TickType, MAX_MSG_LEN,
    NO_VALID_ID, UNSET_DOUBLE, UNSET_INTEGER,
};
use crate::core::contract::{Contract, ContractDescription, ContractDetails, DeltaNeutralContract};
use crate::core::errors::{IBKRApiLibError, TwsError};
use crate::core::execution::Execution;
use crate::core::messages::{read_fields, IncomingMessageIds};
use crate::core::order::{Order, OrderState, SoftDollarTier};
use crate::core::order_decoder::OrderDecoder;
use crate::core::scanner::ScanData;
use crate::core::server_versions::{
    MIN_SERVER_VER_AGG_GROUP, MIN_SERVER_VER_FRACTIONAL_POSITIONS, MIN_SERVER_VER_LAST_LIQUIDITY,
    MIN_SERVER_VER_MARKET_CAP_PRICE, MIN_SERVER_VER_MARKET_RULES,
    MIN_SERVER_VER_MD_SIZE_MULTIPLIER, MIN_SERVER_VER_MODELS_SUPPORT,
    MIN_SERVER_VER_ORDER_CONTAINER, MIN_SERVER_VER_PAST_LIMIT, MIN_SERVER_VER_PRE_OPEN_BID_ASK,
    MIN_SERVER_VER_REALIZED_PNL, MIN_SERVER_VER_REAL_EXPIRATION_DATE,
    MIN_SERVER_VER_SERVICE_DATA_TYPE, MIN_SERVER_VER_SMART_DEPTH,
    MIN_SERVER_VER_SYNT_REALTIME_BARS, MIN_SERVER_VER_UNDERLYING_INFO,
    MIN_SERVER_VER_UNREALIZED_PNL,
};
use crate::core::wrapper::Wrapper;

const WRAPPER_POISONED_MUTEX: &str = "Wrapper mutex was poisoned";
//==================================================================================================
pub fn decode_i32(iter: &mut Iter<String>) -> Result<i32, IBKRApiLibError> {
    let next = iter.next();

    let val: i32 = next.unwrap().parse().unwrap_or(0);
    Ok(val)
}

//==================================================================================================
pub fn decode_i32_show_unset(iter: &mut Iter<String>) -> Result<i32, IBKRApiLibError> {
    let next = iter.next();
    //info!("{:?}", next);
    let retval: i32 = next.unwrap().parse().unwrap_or(0);
    Ok(if retval == 0 { UNSET_INTEGER } else { retval })
}

//==================================================================================================
pub fn decode_i64(iter: &mut Iter<String>) -> Result<i64, IBKRApiLibError> {
    let next = iter.next();
    //info!("{:?}", next);
    let val: i64 = next.unwrap().parse().unwrap_or(0);
    Ok(val)
}

//==================================================================================================
pub fn decode_f64(iter: &mut Iter<String>) -> Result<f64, IBKRApiLibError> {
    let next = iter.next();
    //info!("{:?}", next);
    let val = next.unwrap().parse().unwrap_or(0.0);
    Ok(val)
}

//==================================================================================================
pub fn decode_f64_show_unset(iter: &mut Iter<String>) -> Result<f64, IBKRApiLibError> {
    let next = iter.next();
    //info!("{:?}", next);
    let retval: f64 = next.unwrap().parse().unwrap_or(0.0);
    Ok(if retval == 0.0 { UNSET_DOUBLE } else { retval })
}

//==================================================================================================
pub fn decode_string(iter: &mut Iter<String>) -> Result<String, IBKRApiLibError> {
    let next = iter.next();
    //info!("{:?}", next);
    let val = next.unwrap().parse().unwrap_or("".to_string());
    Ok(val)
}

//==================================================================================================
pub fn decode_bool(iter: &mut Iter<String>) -> Result<bool, IBKRApiLibError> {
    let next = iter.next();
    //info!("{:?}", next);
    let retval: i32 = next.unwrap_or(&"0".to_string()).parse().unwrap_or(0);
    Ok(retval != 0)
}

//==================================================================================================
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
            msg_queue,
            server_version,
            conn_state,
        }
    }

    //----------------------------------------------------------------------------------------------
    pub fn interpret(&mut self, fields: &[String]) -> Result<(), IBKRApiLibError> {
        if fields.is_empty() {
            return Ok(());
        }

        let msg_id = i32::from_str(fields.get(0).unwrap().as_str())?;

        match FromPrimitive::from_i32(msg_id) {
            Some(IncomingMessageIds::TickPrice) => self.process_tick_price(fields)?,
            Some(IncomingMessageIds::AccountSummary) => self.process_account_summary(fields)?,
            Some(IncomingMessageIds::AccountSummaryEnd) => {
                self.process_account_summary_end(fields)?
            }
            Some(IncomingMessageIds::AccountUpdateMulti) => {
                self.process_account_update_multi(fields)?
            }
            Some(IncomingMessageIds::AccountUpdateMultiEnd) => {
                self.process_account_update_multi_end(fields)?
            }
            Some(IncomingMessageIds::AcctDownloadEnd) => {
                self.process_account_download_end(fields)?
            }
            Some(IncomingMessageIds::AcctUpdateTime) => self.process_account_update_time(fields)?,
            Some(IncomingMessageIds::AcctValue) => self.process_account_value(fields)?,
            Some(IncomingMessageIds::BondContractData) => {
                self.process_bond_contract_data(fields)?
            }
            Some(IncomingMessageIds::CommissionReport) => self.process_commission_report(fields)?,
            Some(IncomingMessageIds::CompletedOrder) => self.process_completed_order(fields)?,
            Some(IncomingMessageIds::CompletedOrdersEnd) => {
                self.process_complete_orders_end(fields)?
            }
            Some(IncomingMessageIds::ContractData) => self.process_contract_details(fields)?,
            Some(IncomingMessageIds::ContractDataEnd) => {
                self.process_contract_details_end(fields)?
            }
            Some(IncomingMessageIds::CurrentTime) => self.process_current_time(fields)?,
            Some(IncomingMessageIds::DeltaNeutralValidation) => {
                self.process_delta_neutral_validation(fields)?
            }
            Some(IncomingMessageIds::DisplayGroupList) => {
                self.process_display_group_list(fields)?
            }
            Some(IncomingMessageIds::DisplayGroupUpdated) => {
                self.process_display_group_updated(fields)?
            }
            Some(IncomingMessageIds::ErrMsg) => self.process_error_message(fields)?,
            Some(IncomingMessageIds::ExecutionData) => self.process_execution_data(fields)?,
            Some(IncomingMessageIds::ExecutionDataEnd) => {
                self.process_execution_data_end(fields)?
            }
            Some(IncomingMessageIds::FamilyCodes) => self.process_family_codes(fields)?,
            Some(IncomingMessageIds::FundamentalData) => self.process_fundamental_data(fields)?,
            Some(IncomingMessageIds::HeadTimestamp) => self.process_head_timestamp(fields)?,
            Some(IncomingMessageIds::HistogramData) => self.process_histogram_data(fields)?,
            Some(IncomingMessageIds::HistoricalData) => self.process_historical_data(fields)?,
            Some(IncomingMessageIds::HistoricalDataUpdate) => {
                self.process_historical_data_update(fields)?
            }
            Some(IncomingMessageIds::HistoricalNews) => self.process_historical_news(fields)?,
            Some(IncomingMessageIds::HistoricalNewsEnd) => {
                self.process_historical_news_end(fields)?
            }
            Some(IncomingMessageIds::HistoricalTicks) => self.process_historical_ticks(fields)?,
            Some(IncomingMessageIds::HistoricalTicksBidAsk) => {
                self.process_historical_ticks_bid_ask(fields)?
            }

            Some(IncomingMessageIds::HistoricalTicksLast) => {
                self.process_historical_ticks_last(fields)?
            }
            Some(IncomingMessageIds::ManagedAccts) => self.process_managed_accounts(fields)?,
            Some(IncomingMessageIds::MarketDataType) => self.process_market_data_type(fields)?,
            Some(IncomingMessageIds::MarketDepth) => self.process_market_depth(fields)?,
            Some(IncomingMessageIds::MarketDepthL2) => self.process_market_depth_l2(fields)?,
            Some(IncomingMessageIds::MarketRule) => self.process_market_rule(fields)?,
            Some(IncomingMessageIds::MktDepthExchanges) => {
                self.process_market_depth_exchanges(fields)?
            }
            Some(IncomingMessageIds::NewsArticle) => self.process_news_article(fields)?,
            Some(IncomingMessageIds::NewsBulletins) => self.process_news_bulletins(fields)?,
            Some(IncomingMessageIds::NewsProviders) => self.process_news_providers(fields)?,
            Some(IncomingMessageIds::NextValidId) => self.process_next_valid_id(fields)?,
            Some(IncomingMessageIds::OpenOrder) => self.process_open_order(fields)?,
            Some(IncomingMessageIds::OpenOrderEnd) => self.process_open_order_end(fields)?,
            Some(IncomingMessageIds::OrderStatus) => self.process_order_status(fields)?,
            Some(IncomingMessageIds::OrderBound) => self.process_order_bound(fields)?,
            Some(IncomingMessageIds::Pnl) => self.process_pnl(fields)?,
            Some(IncomingMessageIds::PnlSingle) => self.process_pnl_single(fields)?,
            Some(IncomingMessageIds::PortfolioValue) => self.process_portfolio_value(fields)?,
            Some(IncomingMessageIds::PositionData) => self.process_position_data(fields)?,
            Some(IncomingMessageIds::PositionEnd) => self.process_position_end(fields)?,
            Some(IncomingMessageIds::RealTimeBars) => self.process_real_time_bars(fields)?,
            Some(IncomingMessageIds::ReceiveFa) => self.process_receive_fa(fields)?,
            Some(IncomingMessageIds::RerouteMktDataReq) => {
                self.process_reroute_mkt_data_req(fields)?
            }

            Some(IncomingMessageIds::PositionMulti) => self.process_position_multi(fields)?,
            Some(IncomingMessageIds::PositionMultiEnd) => {
                self.process_position_multi_end(fields)?
            }
            Some(IncomingMessageIds::ScannerData) => self.process_scanner_data(fields)?,
            Some(IncomingMessageIds::ScannerParameters) => {
                self.process_scanner_parameters(fields)?
            }
            Some(IncomingMessageIds::SecurityDefinitionOptionParameter) => {
                self.process_security_definition_option_parameter(fields)?
            }
            Some(IncomingMessageIds::SecurityDefinitionOptionParameterEnd) => {
                self.process_security_definition_option_parameter_end(fields)?
            }

            Some(IncomingMessageIds::SmartComponents) => self.process_smart_components(fields)?,
            Some(IncomingMessageIds::SoftDollarTiers) => self.process_soft_dollar_tiers(fields)?,
            Some(IncomingMessageIds::SymbolSamples) => self.process_symbol_samples(fields)?,
            Some(IncomingMessageIds::TickByTick) => self.process_tick_by_tick(fields)?,
            Some(IncomingMessageIds::TickEfp) => self.process_tick_by_tick(fields)?,
            Some(IncomingMessageIds::TickGeneric) => self.process_tick_generic(fields)?,
            Some(IncomingMessageIds::TickNews) => self.process_tick_news(fields)?,
            Some(IncomingMessageIds::TickOptionComputation) => {
                self.process_tick_option_computation(fields)?
            }
            Some(IncomingMessageIds::TickReqParams) => self.process_tick_req_params(fields)?,
            Some(IncomingMessageIds::TickSize) => self.process_tick_size(fields)?,
            Some(IncomingMessageIds::TickSnapshotEnd) => self.process_tick_snapshot_end(fields)?,
            Some(IncomingMessageIds::TickString) => self.process_tick_string(fields)?,
            Some(IncomingMessageIds::VerifyAndAuthCompleted) => {
                self.process_verify_and_auth_completed(fields)?
            }

            Some(IncomingMessageIds::VerifyCompleted) => self.process_verify_completed(fields)?,

            Some(IncomingMessageIds::VerifyMessageApi) => self.process_verify_completed(fields)?,

            Some(IncomingMessageIds::VerifyAndAuthMessageApi) => {
                self.process_verify_and_auth_message_api(fields)?
            }
            Some(IncomingMessageIds::RerouteMktDepthReq) => {
                self.process_reroute_mkt_depth_req(fields)?
            }

            _ => panic!("Received unkown message id!!  Exiting..."),
        }
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    fn process_tick_price(&mut self, fields: &[String]) -> Result<(), IBKRApiLibError> {
        let mut fields_itr = fields.iter();

        //throw away message_id
        fields_itr.next();
        //throw away version
        fields_itr.next();

        let req_id = decode_i32(&mut fields_itr)?;
        let tick_type: i32 = decode_i32(&mut fields_itr)?;
        let price: f64 = decode_f64(&mut fields_itr)?;
        let size = decode_i32(&mut fields_itr)?;
        let attr_mask: i32 = decode_i32(&mut fields_itr)?;
        let mut tick_arrtibute = TickAttrib::new(false, false, false);

        tick_arrtibute.can_auto_execute = attr_mask == 1;

        if self.server_version >= MIN_SERVER_VER_PAST_LIMIT {
            tick_arrtibute.can_auto_execute = attr_mask & 1 != 0;
            tick_arrtibute.past_limit = attr_mask & 2 != 0;
        }
        if self.server_version >= MIN_SERVER_VER_PRE_OPEN_BID_ASK {
            tick_arrtibute.pre_open = attr_mask & 4 != 0;
        }
        self.wrapper
            .lock()
            .expect(WRAPPER_POISONED_MUTEX)
            .tick_price(
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
                .expect(WRAPPER_POISONED_MUTEX)
                .tick_size(req_id, size_tick_type, size);
        }
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    fn process_tick_string(&mut self, fields: &[String]) -> Result<(), IBKRApiLibError> {
        let mut fields_itr = fields.iter();

        //throw away message_id
        fields_itr.next();
        //throw away version
        fields_itr.next();

        let req_id: i32 = decode_i32(&mut fields_itr)?;
        let tick_type: i32 = decode_i32(&mut fields_itr)?;
        let value = decode_string(&mut fields_itr)?;

        self.wrapper
            .lock()
            .expect(WRAPPER_POISONED_MUTEX)
            .tick_string(
                req_id,
                FromPrimitive::from_i32(tick_type).unwrap(),
                value.as_ref(),
            );
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    fn process_account_summary(&mut self, fields: &[String]) -> Result<(), IBKRApiLibError> {
        let mut fields_itr = fields.iter();

        //throw away message_id
        fields_itr.next();
        //throw away version
        fields_itr.next();

        self.wrapper
            .lock()
            .expect(WRAPPER_POISONED_MUTEX)
            .account_summary(
                decode_i32(&mut fields_itr)?,
                decode_string(&mut fields_itr)?.as_ref(),
                decode_string(&mut fields_itr)?.as_ref(),
                decode_string(&mut fields_itr)?.as_ref(),
                decode_string(&mut fields_itr)?.as_ref(),
            );
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    fn process_account_summary_end(&mut self, fields: &[String]) -> Result<(), IBKRApiLibError> {
        let mut fields_itr = fields.iter();

        //throw away message_id
        fields_itr.next();
        //throw away version
        fields_itr.next();

        self.wrapper
            .lock()
            .expect(WRAPPER_POISONED_MUTEX)
            .account_summary_end(decode_i32(&mut fields_itr)?);
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    fn process_account_update_multi(&mut self, fields: &[String]) -> Result<(), IBKRApiLibError> {
        let mut fields_itr = fields.iter();

        //throw away message_id
        fields_itr.next();
        //throw away version
        fields_itr.next();

        let req_id: i32 = decode_i32(&mut fields_itr)?;
        let account = decode_string(&mut fields_itr)?;
        let model_code = decode_string(&mut fields_itr)?;
        let key = decode_string(&mut fields_itr)?;
        let value = decode_string(&mut fields_itr)?;
        let currency = decode_string(&mut fields_itr)?;

        self.wrapper
            .lock()
            .expect(WRAPPER_POISONED_MUTEX)
            .account_update_multi(
                req_id,
                account.as_ref(),
                model_code.as_ref(),
                key.as_ref(),
                value.as_ref(),
                currency.as_ref(),
            );
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    fn process_account_update_multi_end(
        &mut self,
        fields: &[String],
    ) -> Result<(), IBKRApiLibError> {
        let mut fields_itr = fields.iter();

        //throw away message_id
        fields_itr.next();
        //throw away version
        fields_itr.next();

        let req_id: i32 = decode_i32(&mut fields_itr)?;

        self.wrapper
            .lock()
            .expect(WRAPPER_POISONED_MUTEX)
            .account_update_multi_end(req_id);
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    fn process_account_download_end(&mut self, fields: &[String]) -> Result<(), IBKRApiLibError> {
        let mut fields_itr = fields.iter();

        //throw away message_id
        fields_itr.next();
        //throw away version
        fields_itr.next();

        self.wrapper
            .lock()
            .expect(WRAPPER_POISONED_MUTEX)
            .account_download_end(decode_string(&mut fields_itr)?.as_ref());
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    fn process_account_update_time(&mut self, fields: &[String]) -> Result<(), IBKRApiLibError> {
        let mut fields_itr = fields.iter();

        //throw away message_id
        fields_itr.next();
        //throw away version
        fields_itr.next();

        self.wrapper
            .lock()
            .expect(WRAPPER_POISONED_MUTEX)
            .update_account_time(decode_string(&mut fields_itr)?.as_ref());
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    fn process_account_value(&mut self, fields: &[String]) -> Result<(), IBKRApiLibError> {
        let mut fields_itr = fields.iter();

        //throw away message_id
        fields_itr.next();
        //throw away version
        fields_itr.next();

        self.wrapper
            .lock()
            .expect(WRAPPER_POISONED_MUTEX)
            .update_account_value(
                decode_string(&mut fields_itr)?.as_ref(),
                decode_string(&mut fields_itr)?.as_ref(),
                decode_string(&mut fields_itr)?.as_ref(),
                decode_string(&mut fields_itr)?.as_ref(),
            );
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    fn process_bond_contract_data(&mut self, fields: &[String]) -> Result<(), IBKRApiLibError> {
        let mut fields_itr = fields.iter();

        //throw away message_id
        fields_itr.next();

        let version: i32 = decode_i32(&mut fields_itr)?;

        let mut req_id = -1;
        if version >= 3 {
            req_id = decode_i32(&mut fields_itr)?;
        }

        let mut contract = ContractDetails::default();

        contract.contract.symbol = decode_string(&mut fields_itr)?;
        contract.contract.sec_type = decode_string(&mut fields_itr)?;
        contract.cusip = decode_string(&mut fields_itr)?;
        contract.coupon = decode_f64(&mut fields_itr)?;
        self.read_last_trade_date(&mut contract, true, fields_itr.next().unwrap())?;
        contract.issue_date = decode_string(&mut fields_itr)?;
        contract.ratings = decode_string(&mut fields_itr)?;
        contract.bond_type = decode_string(&mut fields_itr)?;
        contract.coupon_type = decode_string(&mut fields_itr)?;
        contract.convertible = i32::from_str(fields_itr.next().unwrap().as_ref())? != 0;
        contract.callable = i32::from_str(fields_itr.next().unwrap().as_ref())? != 0;
        contract.putable = i32::from_str(fields_itr.next().unwrap().as_ref())? != 0;
        contract.desc_append = decode_string(&mut fields_itr)?;
        contract.contract.exchange = decode_string(&mut fields_itr)?;
        contract.contract.currency = decode_string(&mut fields_itr)?;
        contract.market_name = decode_string(&mut fields_itr)?;
        contract.contract.trading_class = decode_string(&mut fields_itr)?;
        contract.contract.con_id = decode_i32(&mut fields_itr)?;
        contract.min_tick = decode_f64(&mut fields_itr)?;
        if self.server_version >= MIN_SERVER_VER_MD_SIZE_MULTIPLIER {
            contract.md_size_multiplier = decode_i32(&mut fields_itr)?;
        }
        contract.order_types = decode_string(&mut fields_itr)?;
        contract.valid_exchanges = decode_string(&mut fields_itr)?;
        if version >= 2 {
            contract.next_option_date = decode_string(&mut fields_itr)?;
            contract.next_option_type = decode_string(&mut fields_itr)?;
            contract.next_option_partial = decode_bool(&mut fields_itr)?;
            contract.notes = decode_string(&mut fields_itr)?;
        }
        if version >= 4 {
            contract.long_name = decode_string(&mut fields_itr)?;
        }
        if version >= 6 {
            contract.ev_rule = decode_string(&mut fields_itr)?;
            contract.ev_multiplier = decode_f64(&mut fields_itr)?;
        }
        if version >= 5 {
            let sec_id_list_count = decode_i32(&mut fields_itr)?;
            if sec_id_list_count > 0 {
                contract.sec_id_list = vec![];
                for _ in 0..sec_id_list_count {
                    contract.sec_id_list.push(TagValue::new(
                        fields_itr.next().unwrap().parse().unwrap(),
                        fields_itr.next().unwrap().parse().unwrap(),
                    ));
                }
            }
        }
        if self.server_version >= MIN_SERVER_VER_AGG_GROUP {
            contract.agg_group = decode_i32(&mut fields_itr)?;
        }
        if self.server_version >= MIN_SERVER_VER_MARKET_RULES {
            contract.market_rule_ids = decode_string(&mut fields_itr)?;
        }

        self.wrapper
            .lock()
            .expect(WRAPPER_POISONED_MUTEX)
            .bond_contract_details(req_id, contract.clone());
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    fn process_commission_report(&mut self, fields: &[String]) -> Result<(), IBKRApiLibError> {
        let mut fields_itr = fields.iter();
        //throw away message_id
        fields_itr.next();
        //throw away version
        fields_itr.next();

        let mut commission_report = CommissionReport::default();
        commission_report.exec_id = fields_itr.next().unwrap().to_string();
        commission_report.commission = decode_f64(&mut fields_itr)?;
        commission_report.currency = fields_itr.next().unwrap().to_string();

        commission_report.realized_pnl = decode_f64(&mut fields_itr)?;

        commission_report.yield_ = decode_f64(&mut fields_itr)?;

        commission_report.yield_redemption_date = decode_string(&mut fields_itr)?;

        self.wrapper
            .lock()
            .expect(WRAPPER_POISONED_MUTEX)
            .commission_report(commission_report);
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    fn process_completed_order(&mut self, fields: &[String]) -> Result<(), IBKRApiLibError> {
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

        order_decoder.decode_completed(&mut fields_itr)?;

        self.wrapper
            .lock()
            .expect(WRAPPER_POISONED_MUTEX)
            .completed_order(contract, order, order_state);
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    fn process_complete_orders_end(&mut self, fields: &[String]) -> Result<(), IBKRApiLibError> {
        let mut fields_itr = fields.iter();

        //throw away message_id
        fields_itr.next();
        self.wrapper
            .lock()
            .expect(WRAPPER_POISONED_MUTEX)
            .completed_orders_end();
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    fn process_contract_details(&mut self, fields: &[String]) -> Result<(), IBKRApiLibError> {
        let mut fields_itr = fields.iter();

        //throw away message_id
        fields_itr.next();

        let version: i32 = decode_i32(&mut fields_itr)?;

        let mut req_id = -1;
        if version >= 3 {
            req_id = decode_i32(&mut fields_itr)?;
        }

        let mut contract = ContractDetails::default();

        contract.contract.symbol = decode_string(&mut fields_itr)?;
        contract.contract.sec_type = decode_string(&mut fields_itr)?;
        self.read_last_trade_date(&mut contract, false, fields_itr.next().unwrap())?;
        contract.contract.strike = decode_f64(&mut fields_itr)?;
        contract.contract.right = decode_string(&mut fields_itr)?;
        contract.contract.exchange = decode_string(&mut fields_itr)?;
        contract.contract.currency = decode_string(&mut fields_itr)?;
        contract.contract.local_symbol = decode_string(&mut fields_itr)?;
        contract.market_name = decode_string(&mut fields_itr)?;
        contract.contract.trading_class = decode_string(&mut fields_itr)?;
        contract.contract.con_id = decode_i32(&mut fields_itr)?;
        contract.min_tick = decode_f64(&mut fields_itr)?;
        if self.server_version >= MIN_SERVER_VER_MD_SIZE_MULTIPLIER {
            contract.md_size_multiplier = decode_i32(&mut fields_itr)?;
        }
        contract.contract.multiplier = decode_string(&mut fields_itr)?;
        contract.order_types = decode_string(&mut fields_itr)?;
        contract.valid_exchanges = decode_string(&mut fields_itr)?;
        contract.price_magnifier = decode_i32(&mut fields_itr)?;
        if version >= 4 {
            contract.under_con_id = decode_i32(&mut fields_itr)?;
        }
        if version >= 5 {
            contract.long_name = decode_string(&mut fields_itr)?;
            contract.contract.primary_exchange = decode_string(&mut fields_itr)?;
        }

        if version >= 6 {
            contract.contract_month = decode_string(&mut fields_itr)?;
            contract.industry = decode_string(&mut fields_itr)?;
            contract.category = decode_string(&mut fields_itr)?;
            contract.subcategory = decode_string(&mut fields_itr)?;
            contract.time_zone_id = decode_string(&mut fields_itr)?;
            contract.trading_hours = decode_string(&mut fields_itr)?;
            contract.liquid_hours = decode_string(&mut fields_itr)?;
        }
        if version >= 8 {
            contract.ev_rule = decode_string(&mut fields_itr)?;
            contract.ev_multiplier = decode_f64(&mut fields_itr)?;
        }

        if version >= 7 {
            let sec_id_list_count = decode_i32(&mut fields_itr)?;
            if sec_id_list_count > 0 {
                contract.sec_id_list = vec![];
                for _ in 0..sec_id_list_count {
                    contract.sec_id_list.push(TagValue::new(
                        decode_string(&mut fields_itr)?,
                        decode_string(&mut fields_itr)?,
                    ));
                }
            }
        }
        if self.server_version >= MIN_SERVER_VER_AGG_GROUP {
            contract.agg_group = decode_i32(&mut fields_itr)?;
        }

        if self.server_version >= MIN_SERVER_VER_UNDERLYING_INFO {
            contract.under_symbol = decode_string(&mut fields_itr)?;
            contract.under_sec_type = decode_string(&mut fields_itr)?;
        }
        if self.server_version >= MIN_SERVER_VER_MARKET_RULES {
            contract.market_rule_ids = decode_string(&mut fields_itr)?;
        }

        if self.server_version >= MIN_SERVER_VER_REAL_EXPIRATION_DATE {
            contract.real_expiration_date = decode_string(&mut fields_itr)?;
        }

        self.wrapper
            .lock()
            .expect(WRAPPER_POISONED_MUTEX)
            .contract_details(req_id, contract.clone());
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    fn process_contract_details_end(&mut self, fields: &[String]) -> Result<(), IBKRApiLibError> {
        let mut fields_itr = fields.iter();

        //throw away message_id
        fields_itr.next();
        //throw away version
        fields_itr.next();

        let req_id = decode_i32(&mut fields_itr)?;

        self.wrapper
            .lock()
            .expect(WRAPPER_POISONED_MUTEX)
            .contract_details_end(req_id);
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    fn process_current_time(&mut self, fields: &[String]) -> Result<(), IBKRApiLibError> {
        let mut fields_itr = fields.iter();

        //throw away message_id
        fields_itr.next();
        //throw away version
        fields_itr.next();

        self.wrapper
            .lock()
            .expect(WRAPPER_POISONED_MUTEX)
            .current_time(decode_i64(&mut fields_itr)?);
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    fn process_delta_neutral_validation(
        &mut self,
        fields: &[String],
    ) -> Result<(), IBKRApiLibError> {
        let mut fields_itr = fields.iter();

        //throw away message_id
        fields_itr.next();
        //throw away version
        fields_itr.next();

        let req_id = decode_i32(&mut fields_itr)?;

        let mut delta_neutral_contract = DeltaNeutralContract::default();

        delta_neutral_contract.con_id = decode_i32(&mut fields_itr)?;
        delta_neutral_contract.delta = decode_f64(&mut fields_itr)?;
        delta_neutral_contract.price = decode_f64(&mut fields_itr)?;

        self.wrapper
            .lock()
            .expect(WRAPPER_POISONED_MUTEX)
            .delta_neutral_validation(req_id, delta_neutral_contract);
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    fn process_display_group_list(&mut self, fields: &[String]) -> Result<(), IBKRApiLibError> {
        let mut fields_itr = fields.iter();

        //throw away message_id
        fields_itr.next();
        //throw away version
        fields_itr.next();

        let req_id = decode_i32(&mut fields_itr)?;

        let groups = decode_string(&mut fields_itr)?;

        self.wrapper
            .lock()
            .expect(WRAPPER_POISONED_MUTEX)
            .display_group_list(req_id, groups.as_ref());
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    fn process_display_group_updated(&mut self, fields: &[String]) -> Result<(), IBKRApiLibError> {
        let mut fields_itr = fields.iter();

        //throw away message_id
        fields_itr.next();
        //throw away version
        fields_itr.next();

        let req_id = decode_i32(&mut fields_itr)?;

        let contract_info = decode_string(&mut fields_itr)?;

        self.wrapper
            .lock()
            .expect(WRAPPER_POISONED_MUTEX)
            .display_group_updated(req_id, contract_info.as_ref());
        Ok(())
    }
    fn process_error_message(&mut self, fields: &[String]) -> Result<(), IBKRApiLibError> {
        let mut fields_itr = fields.iter();

        //throw away message_id
        fields_itr.next();
        //throw away version
        fields_itr.next();

        self.wrapper.lock().expect(WRAPPER_POISONED_MUTEX).error(
            decode_i32(&mut fields_itr)?,
            decode_i32(&mut fields_itr)?,
            decode_string(&mut fields_itr)?.as_ref(),
        );
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    fn process_execution_data(&mut self, fields: &[String]) -> Result<(), IBKRApiLibError> {
        let mut fields_itr = fields.iter();

        //throw away message_id
        fields_itr.next();

        let mut version = self.server_version;

        if self.server_version < MIN_SERVER_VER_LAST_LIQUIDITY {
            version = decode_i32(&mut fields_itr)?;
        }

        let mut req_id = -1;

        if version >= 7 {
            req_id = decode_i32(&mut fields_itr)?;
        }

        let order_id = decode_i32(&mut fields_itr)?;

        // decode contract fields
        let mut contract = Contract::default();
        contract.con_id = decode_i32(&mut fields_itr)?; // ver 5 field
        contract.symbol = decode_string(&mut fields_itr)?;
        contract.sec_type = decode_string(&mut fields_itr)?;
        contract.last_trade_date_or_contract_month = decode_string(&mut fields_itr)?;
        contract.strike = decode_f64(&mut fields_itr)?;
        contract.right = decode_string(&mut fields_itr)?;
        if version >= 9 {
            contract.multiplier = decode_string(&mut fields_itr)?;
        }
        contract.exchange = decode_string(&mut fields_itr)?;
        contract.currency = decode_string(&mut fields_itr)?;
        contract.local_symbol = decode_string(&mut fields_itr)?;
        if version >= 10 {
            contract.trading_class = decode_string(&mut fields_itr)?;
        }

        // decode execution fields
        let mut execution = Execution::default();
        execution.order_id = order_id;
        execution.exec_id = decode_string(&mut fields_itr)?;
        execution.time = decode_string(&mut fields_itr)?;
        execution.acct_number = decode_string(&mut fields_itr)?;
        execution.exchange = decode_string(&mut fields_itr)?;
        execution.side = decode_string(&mut fields_itr)?;

        if self.server_version >= MIN_SERVER_VER_FRACTIONAL_POSITIONS {
            execution.shares = decode_f64(&mut fields_itr)?;
        } else {
            execution.shares = decode_i32(&mut fields_itr)? as f64;
        }

        execution.price = decode_f64(&mut fields_itr)?;
        execution.perm_id = decode_i32(&mut fields_itr)?; // ver 2 field
        execution.client_id = decode_i32(&mut fields_itr)?; // ver 3 field
        execution.liquidation = decode_i32(&mut fields_itr)?; // ver 4 field

        if version >= 6 {
            execution.cum_qty = decode_f64(&mut fields_itr)?;
            execution.avg_price = decode_f64(&mut fields_itr)?;
        }

        if version >= 8 {
            execution.order_ref = decode_string(&mut fields_itr)?;
        }

        if version >= 9 {
            execution.ev_rule = decode_string(&mut fields_itr)?;

            let tmp_ev_mult = (&mut fields_itr).peekable().peek().unwrap().as_str();
            if tmp_ev_mult != "" {
                execution.ev_multiplier = decode_f64(&mut fields_itr)?;
            } else {
                execution.ev_multiplier = 1.0;
            }
        }

        if self.server_version >= MIN_SERVER_VER_MODELS_SUPPORT {
            execution.model_code = decode_string(&mut fields_itr)?;
        }
        if self.server_version >= MIN_SERVER_VER_LAST_LIQUIDITY {
            execution.last_liquidity = decode_i32(&mut fields_itr)?;
        }

        self.wrapper
            .lock()
            .expect(WRAPPER_POISONED_MUTEX)
            .exec_details(req_id, contract, execution);
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    fn process_execution_data_end(&mut self, fields: &[String]) -> Result<(), IBKRApiLibError> {
        let mut fields_itr = fields.iter();

        //throw away message_id
        fields_itr.next();
        //throw away version
        fields_itr.next();

        let req_id = decode_i32(&mut fields_itr)?;

        self.wrapper
            .lock()
            .expect(WRAPPER_POISONED_MUTEX)
            .exec_details_end(req_id);
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    fn process_family_codes(&mut self, fields: &[String]) -> Result<(), IBKRApiLibError> {
        let mut fields_itr = fields.iter();

        //throw away message_id
        fields_itr.next();

        let family_codes_count = decode_i32(&mut fields_itr)?;
        let mut family_codes: Vec<FamilyCode> = vec![];
        for _ in 0..family_codes_count {
            let mut fam_code = FamilyCode::default();
            fam_code.account_id = decode_string(&mut fields_itr)?;
            fam_code.family_code_str = decode_string(&mut fields_itr)?;
            family_codes.push(fam_code);
        }

        self.wrapper
            .lock()
            .expect(WRAPPER_POISONED_MUTEX)
            .family_codes(family_codes);
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    fn process_fundamental_data(&mut self, fields: &[String]) -> Result<(), IBKRApiLibError> {
        let mut fields_itr = fields.iter();

        //throw away message_id
        fields_itr.next();
        //throw away version
        fields_itr.next();

        let req_id = decode_i32(&mut fields_itr)?;
        let data = decode_string(&mut fields_itr)?;
        self.wrapper
            .lock()
            .expect(WRAPPER_POISONED_MUTEX)
            .fundamental_data(req_id, data.as_ref());
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    fn process_head_timestamp(&mut self, fields: &[String]) -> Result<(), IBKRApiLibError> {
        let mut fields_itr = fields.iter();

        //throw away message_id
        fields_itr.next();

        let req_id = decode_i32(&mut fields_itr)?;
        let timestamp = decode_string(&mut fields_itr)?;
        self.wrapper
            .lock()
            .expect(WRAPPER_POISONED_MUTEX)
            .fundamental_data(req_id, timestamp.as_ref());
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    fn process_histogram_data(&mut self, fields: &[String]) -> Result<(), IBKRApiLibError> {
        let mut fields_itr = fields.iter();

        //throw away message_id
        fields_itr.next();

        let req_id = decode_i32(&mut fields_itr)?;
        let num_points = decode_i32(&mut fields_itr)?;

        let mut histogram = vec![];
        for _ in 0..num_points {
            let mut data_point = HistogramData::default();
            data_point.price = decode_f64(&mut fields_itr)?;
            data_point.count = decode_i32(&mut fields_itr)?;
            histogram.push(data_point);
        }

        self.wrapper
            .lock()
            .expect(WRAPPER_POISONED_MUTEX)
            .histogram_data(req_id, histogram);
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    fn process_historical_data(&mut self, fields: &[String]) -> Result<(), IBKRApiLibError> {
        let mut fields_itr = fields.iter();
        //throw away message_id
        fields_itr.next();

        if self.server_version < MIN_SERVER_VER_SYNT_REALTIME_BARS {
            fields_itr.next();
        }

        let req_id = decode_i32(&mut fields_itr)?;
        let start_date = decode_string(&mut fields_itr)?; // ver 2 field
        let end_date = decode_string(&mut fields_itr)?; // ver 2 field

        let _peek = *(fields_itr.clone()).peekable().peek().unwrap();

        let bar_count = decode_i32(&mut fields_itr)?;

        for _ in 0..bar_count {
            let mut bar = BarData::default();
            bar.date = decode_string(&mut fields_itr)?;
            bar.open = decode_f64(&mut fields_itr)?;
            bar.high = decode_f64(&mut fields_itr)?;
            bar.low = decode_f64(&mut fields_itr)?;
            bar.close = decode_f64(&mut fields_itr)?;
            bar.volume = if self.server_version < MIN_SERVER_VER_SYNT_REALTIME_BARS {
                decode_i32(&mut fields_itr)? as i64
            } else {
                decode_i64(&mut fields_itr)?
            };
            bar.average = decode_f64(&mut fields_itr)?;

            if self.server_version < MIN_SERVER_VER_SYNT_REALTIME_BARS {
                decode_string(&mut fields_itr)?; //has_gaps
            }

            bar.bar_count = decode_i32(&mut fields_itr)?; // ver 3 field

            self.wrapper
                .lock()
                .expect(WRAPPER_POISONED_MUTEX)
                .historical_data(req_id, bar);
        }

        // send end of dataset marker
        self.wrapper
            .lock()
            .expect(WRAPPER_POISONED_MUTEX)
            .historical_data_end(req_id, start_date.as_ref(), end_date.as_ref());
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    fn process_historical_data_update(&mut self, fields: &[String]) -> Result<(), IBKRApiLibError> {
        let mut fields_itr = fields.iter();

        //throw away message_id
        fields_itr.next();

        let req_id = decode_i32(&mut fields_itr)?;

        let mut bar = BarData::default();
        bar.bar_count = decode_i32(&mut fields_itr)?;
        bar.date = decode_string(&mut fields_itr)?;
        bar.open = decode_f64(&mut fields_itr)?;
        bar.close = decode_f64(&mut fields_itr)?;
        bar.high = decode_f64(&mut fields_itr)?;
        bar.low = decode_f64(&mut fields_itr)?;
        bar.average = decode_f64(&mut fields_itr)?;
        bar.volume = decode_i64(&mut fields_itr)?;
        self.wrapper
            .lock()
            .expect(WRAPPER_POISONED_MUTEX)
            .historical_data_update(req_id, bar);
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    fn process_historical_news(&mut self, fields: &[String]) -> Result<(), IBKRApiLibError> {
        let mut fields_itr = fields.iter();

        //throw away message_id
        fields_itr.next();

        let req_id = decode_i32(&mut fields_itr)?;
        let time = decode_string(&mut fields_itr)?;
        let provider_code = decode_string(&mut fields_itr)?;
        let article_id = decode_string(&mut fields_itr)?;
        let headline = decode_string(&mut fields_itr)?;
        self.wrapper
            .lock()
            .expect(WRAPPER_POISONED_MUTEX)
            .historical_news(
                req_id,
                time.as_ref(),
                provider_code.as_ref(),
                article_id.as_ref(),
                headline.as_ref(),
            );
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    fn process_historical_news_end(&mut self, fields: &[String]) -> Result<(), IBKRApiLibError> {
        let mut fields_itr = fields.iter();

        //throw away message_id
        fields_itr.next();

        let req_id = decode_i32(&mut fields_itr)?;
        let has_more = decode_bool(&mut fields_itr)?;

        self.wrapper
            .lock()
            .expect(WRAPPER_POISONED_MUTEX)
            .historical_news_end(req_id, has_more);
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    fn process_historical_ticks(&mut self, fields: &[String]) -> Result<(), IBKRApiLibError> {
        let mut fields_itr = fields.iter();

        //throw away message_id
        fields_itr.next();

        let req_id = decode_i32(&mut fields_itr)?;
        let tick_count = decode_i32(&mut fields_itr)?;

        let mut ticks = vec![];

        for _ in 0..tick_count {
            let mut historical_tick = HistoricalTick::default();
            historical_tick.time = decode_i32(&mut fields_itr)?;
            fields_itr.next(); // for consistency
            historical_tick.price = decode_f64(&mut fields_itr)?;
            historical_tick.size = decode_i32(&mut fields_itr)?;
            ticks.push(historical_tick);
        }

        let done = decode_bool(&mut fields_itr)?;

        self.wrapper
            .lock()
            .expect(WRAPPER_POISONED_MUTEX)
            .historical_ticks(req_id, ticks, done);
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    fn process_historical_ticks_bid_ask(
        &mut self,
        fields: &[String],
    ) -> Result<(), IBKRApiLibError> {
        let mut fields_itr = fields.iter();

        //throw away message_id
        fields_itr.next();

        let req_id = decode_i32(&mut fields_itr)?;
        let tick_count = decode_i32(&mut fields_itr)?;

        let mut ticks = vec![];

        for _ in 0..tick_count {
            let mut historical_tick_bid_ask = HistoricalTickBidAsk::default();
            historical_tick_bid_ask.time = decode_i32(&mut fields_itr)?;
            let mask = decode_i32(&mut fields_itr)?;
            let mut tick_attrib_bid_ask = TickAttribBidAsk::default();
            tick_attrib_bid_ask.ask_past_high = mask & 1 != 0;
            tick_attrib_bid_ask.bid_past_low = mask & 2 != 0;
            historical_tick_bid_ask.tick_attrib_bid_ask = tick_attrib_bid_ask;
            historical_tick_bid_ask.price_bid = decode_f64(&mut fields_itr)?;
            historical_tick_bid_ask.price_ask = decode_f64(&mut fields_itr)?;
            historical_tick_bid_ask.size_bid = decode_i32(&mut fields_itr)?;
            historical_tick_bid_ask.size_ask = decode_i32(&mut fields_itr)?;
            ticks.push(historical_tick_bid_ask);
        }

        let done = decode_bool(&mut fields_itr)?;

        self.wrapper
            .lock()
            .expect(WRAPPER_POISONED_MUTEX)
            .historical_ticks_bid_ask(req_id, ticks, done);
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    fn process_historical_ticks_last(&mut self, fields: &[String]) -> Result<(), IBKRApiLibError> {
        let mut fields_itr = fields.iter();

        //throw away message_id
        fields_itr.next();

        let req_id = decode_i32(&mut fields_itr)?;
        let tick_count = decode_i32(&mut fields_itr)?;

        let mut ticks = vec![];

        for _ in 0..tick_count {
            let mut historical_tick_last = HistoricalTickLast::default();
            historical_tick_last.time = decode_i32(&mut fields_itr)?;
            let mask = decode_i32(&mut fields_itr)?;
            let mut tick_attrib_last = TickAttribLast::default();
            tick_attrib_last.past_limit = mask & 1 != 0;
            tick_attrib_last.unreported = mask & 2 != 0;
            historical_tick_last.tick_attrib_last = tick_attrib_last;
            historical_tick_last.price = decode_f64(&mut fields_itr)?;
            historical_tick_last.size = decode_i32(&mut fields_itr)?;
            historical_tick_last.exchange = decode_string(&mut fields_itr)?;
            historical_tick_last.special_conditions = decode_string(&mut fields_itr)?;
            ticks.push(historical_tick_last);
        }

        let done = decode_bool(&mut fields_itr)?;

        self.wrapper
            .lock()
            .expect(WRAPPER_POISONED_MUTEX)
            .historical_ticks_last(req_id, ticks, done);
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    fn process_managed_accounts(&mut self, fields: &[String]) -> Result<(), IBKRApiLibError> {
        let mut fields_itr = fields.iter();

        //throw away message_id
        fields_itr.next();
        //throw away version
        fields_itr.next();

        let accounts_list = decode_string(&mut fields_itr)?;
        info!("calling managed_accounts");
        self.wrapper
            .lock()
            .expect(WRAPPER_POISONED_MUTEX)
            .managed_accounts(accounts_list.as_ref());
        info!("finished calling managed_accounts");
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    fn process_market_data_type(&mut self, fields: &[String]) -> Result<(), IBKRApiLibError> {
        let mut fields_itr = fields.iter();
        //throw away message_id
        fields_itr.next();
        //throw away version
        fields_itr.next();
        let req_id = decode_i32(&mut fields_itr)?;
        let market_data_type = decode_i32(&mut fields_itr)?;
        self.wrapper
            .lock()
            .expect(WRAPPER_POISONED_MUTEX)
            .market_data_type(req_id, market_data_type);
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    fn process_market_depth(&mut self, fields: &[String]) -> Result<(), IBKRApiLibError> {
        let mut fields_itr = fields.iter();
        //throw away message_id
        fields_itr.next();
        //throw away version
        fields_itr.next();

        let req_id = decode_i32(&mut fields_itr)?;

        let position = decode_i32(&mut fields_itr)?;
        let operation = decode_i32(&mut fields_itr)?;
        let side = decode_i32(&mut fields_itr)?;
        let price = decode_f64(&mut fields_itr)?;
        let size = decode_i32(&mut fields_itr)?;

        self.wrapper
            .lock()
            .expect(WRAPPER_POISONED_MUTEX)
            .update_mkt_depth(req_id, position, operation, side, price, size);
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    fn process_market_depth_l2(&mut self, fields: &[String]) -> Result<(), IBKRApiLibError> {
        let mut fields_itr = fields.iter();

        //throw away message_id
        fields_itr.next();
        //throw away version
        fields_itr.next();

        let req_id = decode_i32(&mut fields_itr)?;

        let position = decode_i32(&mut fields_itr)?;
        let market_maker = decode_string(&mut fields_itr)?;
        let operation = decode_i32(&mut fields_itr)?;
        let side = decode_i32(&mut fields_itr)?;
        let price = decode_f64(&mut fields_itr)?;
        let size = decode_i32(&mut fields_itr)?;
        let mut is_smart_depth = false;

        if self.server_version >= MIN_SERVER_VER_SMART_DEPTH {
            is_smart_depth = decode_bool(&mut fields_itr)?;
        }

        self.wrapper
            .lock()
            .expect(WRAPPER_POISONED_MUTEX)
            .update_mkt_depth_l2(
                req_id,
                position,
                market_maker.as_ref(),
                operation,
                side,
                price,
                size,
                is_smart_depth,
            );
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    fn process_market_rule(&mut self, fields: &[String]) -> Result<(), IBKRApiLibError> {
        let mut fields_itr = fields.iter();

        //throw away message_id
        fields_itr.next();

        let market_rule_id = decode_i32(&mut fields_itr)?;

        let price_increments_count = decode_i32(&mut fields_itr)?;
        let mut price_increments = vec![];

        for _ in 0..price_increments_count {
            let mut prc_inc = PriceIncrement::default();
            prc_inc.low_edge = decode_f64(&mut fields_itr)?;
            prc_inc.increment = decode_f64(&mut fields_itr)?;
            price_increments.push(prc_inc);
        }

        self.wrapper
            .lock()
            .expect(WRAPPER_POISONED_MUTEX)
            .market_rule(market_rule_id, price_increments);
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    fn process_market_depth_exchanges(&mut self, fields: &[String]) -> Result<(), IBKRApiLibError> {
        let mut fields_itr = fields.iter();

        //throw away message_id
        fields_itr.next();

        let mut depth_mkt_data_descriptions = vec![];
        let depth_mkt_data_descriptions_count = decode_i32(&mut fields_itr)?;

        for _ in 0..depth_mkt_data_descriptions_count {
            let mut desc = DepthMktDataDescription::default();
            desc.exchange = decode_string(&mut fields_itr)?;
            desc.sec_type = decode_string(&mut fields_itr)?;
            if self.server_version >= MIN_SERVER_VER_SERVICE_DATA_TYPE {
                desc.listing_exch = decode_string(&mut fields_itr)?;
                desc.service_data_type = decode_string(&mut fields_itr)?;
                desc.agg_group = decode_i32(&mut fields_itr)?;
            } else {
                decode_i32(&mut fields_itr)?; // boolean notSuppIsL2
            }
            depth_mkt_data_descriptions.push(desc);
        }

        self.wrapper
            .lock()
            .expect(WRAPPER_POISONED_MUTEX)
            .mkt_depth_exchanges(depth_mkt_data_descriptions);
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    fn process_news_article(&mut self, fields: &[String]) -> Result<(), IBKRApiLibError> {
        let mut fields_itr = fields.iter();

        //throw away message_id
        fields_itr.next();

        let req_id = decode_i32(&mut fields_itr)?;
        let article_type = decode_i32(&mut fields_itr)?;
        let article_text = decode_string(&mut fields_itr)?;
        self.wrapper
            .lock()
            .expect(WRAPPER_POISONED_MUTEX)
            .news_article(req_id, article_type, article_text.as_ref());
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    fn process_news_bulletins(&mut self, fields: &[String]) -> Result<(), IBKRApiLibError> {
        let mut fields_itr = fields.iter();

        //throw away message_id
        fields_itr.next();
        //throw away version
        fields_itr.next();

        let news_msg_id = decode_i32(&mut fields_itr)?;
        let news_msg_type = decode_i32(&mut fields_itr)?;
        let news_message = decode_string(&mut fields_itr)?;
        let originating_exch = decode_string(&mut fields_itr)?;

        self.wrapper
            .lock()
            .expect(WRAPPER_POISONED_MUTEX)
            .update_news_bulletin(
                news_msg_id,
                news_msg_type,
                news_message.as_ref(),
                originating_exch.as_ref(),
            );
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    fn process_news_providers(&mut self, fields: &[String]) -> Result<(), IBKRApiLibError> {
        let mut fields_itr = fields.iter();

        //throw away message_id
        fields_itr.next();

        let mut news_providers = vec![];
        let news_providers_count = decode_i32(&mut fields_itr)?;
        for _ in 0..news_providers_count {
            let mut provider = NewsProvider::default();
            provider.code = decode_string(&mut fields_itr)?;
            provider.name = decode_string(&mut fields_itr)?;
            news_providers.push(provider);
        }

        self.wrapper
            .lock()
            .expect(WRAPPER_POISONED_MUTEX)
            .news_providers(news_providers);
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    fn process_next_valid_id(&mut self, fields: &[String]) -> Result<(), IBKRApiLibError> {
        let mut fields_itr = fields.iter();

        //throw away message_id
        fields_itr.next();
        //throw away version
        fields_itr.next();

        let order_id = decode_i32(&mut fields_itr)?;
        self.wrapper
            .lock()
            .expect(WRAPPER_POISONED_MUTEX)
            .next_valid_id(order_id);
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    fn process_open_order(&mut self, fields: &[String]) -> Result<(), IBKRApiLibError> {
        let mut fields_itr = fields.iter();
        //info!("Processing open order");
        //throw away message_id
        fields_itr.next();

        let mut order = Order::default();
        let mut contract = Contract::default();
        let mut order_state = OrderState::default();

        let mut version = self.server_version;
        if self.server_version < MIN_SERVER_VER_ORDER_CONTAINER {
            version = decode_i32(&mut fields_itr)?;
        }

        let mut order_decoder = OrderDecoder::new(
            &mut contract,
            &mut order,
            &mut order_state,
            version,
            self.server_version,
        );

        order_decoder.decode_open(&mut fields_itr)?;

        self.wrapper
            .lock()
            .expect(WRAPPER_POISONED_MUTEX)
            .open_order(order.order_id, contract, order, order_state);
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    fn process_open_order_end(&mut self, _fields: &[String]) -> Result<(), IBKRApiLibError> {
        self.wrapper
            .lock()
            .expect(WRAPPER_POISONED_MUTEX)
            .open_order_end();
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    fn process_order_bound(&mut self, fields: &[String]) -> Result<(), IBKRApiLibError> {
        let mut fields_itr = fields.iter();

        //throw away message_id
        fields_itr.next();

        let req_id = decode_i32(&mut fields_itr)?;
        let api_client_id = decode_i32(&mut fields_itr)?;
        let api_order_id = decode_i32(&mut fields_itr)?;

        self.wrapper
            .lock()
            .expect(WRAPPER_POISONED_MUTEX)
            .order_bound(req_id, api_client_id, api_order_id);
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    fn process_order_status(&mut self, fields: &[String]) -> Result<(), IBKRApiLibError> {
        let mut fields_itr = fields.iter();

        //throw away message_id
        fields_itr.next();

        if self.server_version < MIN_SERVER_VER_MARKET_CAP_PRICE {
            fields_itr.next();
        }

        let order_id = decode_i32(&mut fields_itr)?;

        let status = decode_string(&mut fields_itr)?;

        let filled;
        if self.server_version >= MIN_SERVER_VER_FRACTIONAL_POSITIONS {
            filled = decode_f64(&mut fields_itr)?;
        } else {
            filled = decode_i32(&mut fields_itr)? as f64;
        }

        let remaining;

        if self.server_version >= MIN_SERVER_VER_FRACTIONAL_POSITIONS {
            remaining = decode_f64(&mut fields_itr)?;
        } else {
            remaining = decode_i32(&mut fields_itr)? as f64;
        }

        let avg_fill_price = decode_f64(&mut fields_itr)?;

        let perm_id = decode_i32(&mut fields_itr)?; // ver 2 field
        let parent_id = decode_i32(&mut fields_itr)?; // ver 3 field
        let last_fill_price = decode_f64(&mut fields_itr)?; // ver 4 field
        let client_id = decode_i32(&mut fields_itr)?; // ver 5 field
        let why_held = decode_string(&mut fields_itr)?; // ver 6 field

        let mut mkt_cap_price = 0.0;
        if self.server_version >= MIN_SERVER_VER_MARKET_CAP_PRICE {
            mkt_cap_price = decode_f64(&mut fields_itr)?;
        }

        self.wrapper
            .try_lock()
            .expect(WRAPPER_POISONED_MUTEX)
            .order_status(
                order_id,
                status.as_ref(),
                filled,
                remaining,
                avg_fill_price,
                perm_id,
                parent_id,
                last_fill_price,
                client_id,
                why_held.as_ref(),
                mkt_cap_price,
            );
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    fn process_pnl(&mut self, fields: &[String]) -> Result<(), IBKRApiLibError> {
        let mut fields_itr = fields.iter();

        //throw away message_id
        fields_itr.next();

        let req_id = decode_i32(&mut fields_itr)?;
        let daily_pnl = decode_f64(&mut fields_itr)?;
        let mut unrealized_pnl = 0.0;
        let mut realized_pnl = 0.0;

        if self.server_version >= MIN_SERVER_VER_UNREALIZED_PNL {
            unrealized_pnl = decode_f64(&mut fields_itr)?;
        }

        if self.server_version >= MIN_SERVER_VER_REALIZED_PNL {
            realized_pnl = decode_f64(&mut fields_itr)?;
        }

        self.wrapper.lock().expect(WRAPPER_POISONED_MUTEX).pnl(
            req_id,
            daily_pnl,
            unrealized_pnl,
            realized_pnl,
        );
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    fn process_pnl_single(&mut self, fields: &[String]) -> Result<(), IBKRApiLibError> {
        let mut fields_itr = fields.iter();

        //throw away message_id
        fields_itr.next();

        let req_id = decode_i32(&mut fields_itr)?;
        let pos = decode_i32(&mut fields_itr)?;
        let daily_pnl = decode_f64(&mut fields_itr)?;
        let mut unrealized_pnl = 0.0;
        let mut realized_pnl = 0.0;

        if self.server_version >= MIN_SERVER_VER_UNREALIZED_PNL {
            unrealized_pnl = decode_f64(&mut fields_itr)?;
        }

        if self.server_version >= MIN_SERVER_VER_REALIZED_PNL {
            realized_pnl = decode_f64(&mut fields_itr)?;
        }

        let value = decode_f64(&mut fields_itr)?;

        self.wrapper
            .lock()
            .expect(WRAPPER_POISONED_MUTEX)
            .pnl_single(req_id, pos, daily_pnl, unrealized_pnl, realized_pnl, value);
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    fn process_portfolio_value(&mut self, fields: &[String]) -> Result<(), IBKRApiLibError> {
        let mut fields_itr = fields.iter();

        //throw away message_id
        fields_itr.next();

        let version = decode_i32(&mut fields_itr)?;

        // read contract fields
        let mut contract = Contract::default();
        contract.con_id = decode_i32(&mut fields_itr)?; // ver 6 field
        contract.symbol = decode_string(&mut fields_itr)?;
        contract.sec_type = decode_string(&mut fields_itr)?;
        contract.last_trade_date_or_contract_month = decode_string(&mut fields_itr)?;
        contract.strike = decode_f64(&mut fields_itr)?;
        contract.right = decode_string(&mut fields_itr)?;

        if version >= 7 {
            contract.multiplier = decode_string(&mut fields_itr)?;
            contract.primary_exchange = decode_string(&mut fields_itr)?;
        }

        contract.currency = decode_string(&mut fields_itr)?;
        contract.local_symbol = decode_string(&mut fields_itr)?; // ver 2 field
        if version >= 8 {
            contract.trading_class = decode_string(&mut fields_itr)?;
        }

        let position;
        if self.server_version >= MIN_SERVER_VER_FRACTIONAL_POSITIONS {
            position = decode_f64(&mut fields_itr)?;
        } else {
            position = decode_i32(&mut fields_itr)? as f64;
        }

        let market_price = decode_f64(&mut fields_itr)?;
        let market_value = decode_f64(&mut fields_itr)?;
        let average_cost = decode_f64(&mut fields_itr)?; // ver 3 field
        let unrealized_pnl = decode_f64(&mut fields_itr)?; // ver 3 field
        let realized_pnl = decode_f64(&mut fields_itr)?; // ver 3 field

        let account_name = decode_string(&mut fields_itr)?; // ver 4 field

        if version == 6 && self.server_version == 39 {
            contract.primary_exchange = decode_string(&mut fields_itr)?;
        }

        self.wrapper
            .lock()
            .expect(WRAPPER_POISONED_MUTEX)
            .update_portfolio(
                contract,
                position,
                market_price,
                market_value,
                average_cost,
                unrealized_pnl,
                realized_pnl,
                account_name.as_ref(),
            );
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    fn process_position_data(&mut self, fields: &[String]) -> Result<(), IBKRApiLibError> {
        let mut fields_itr = fields.iter();

        //throw away message_id
        fields_itr.next();

        let version = decode_i32(&mut fields_itr)?;

        let account = decode_string(&mut fields_itr)?;

        // decode contract fields
        let mut contract = Contract::default();
        contract.con_id = decode_i32(&mut fields_itr)?;
        contract.symbol = decode_string(&mut fields_itr)?;
        contract.sec_type = decode_string(&mut fields_itr)?;
        contract.last_trade_date_or_contract_month = decode_string(&mut fields_itr)?;
        contract.strike = decode_f64(&mut fields_itr)?;
        contract.right = decode_string(&mut fields_itr)?;
        contract.multiplier = decode_string(&mut fields_itr)?;
        contract.exchange = decode_string(&mut fields_itr)?;
        contract.currency = decode_string(&mut fields_itr)?;
        contract.local_symbol = decode_string(&mut fields_itr)?;
        if version >= 2 {
            contract.trading_class = decode_string(&mut fields_itr)?;
        }

        let position;
        if self.server_version >= MIN_SERVER_VER_FRACTIONAL_POSITIONS {
            position = decode_f64(&mut fields_itr)?;
        } else {
            position = decode_i32(&mut fields_itr)? as f64;
        }

        let mut avg_cost = 0.0;
        if version >= 3 {
            avg_cost = decode_f64(&mut fields_itr)?;
        }

        self.wrapper.lock().expect(WRAPPER_POISONED_MUTEX).position(
            account.as_ref(),
            contract,
            position,
            avg_cost,
        );
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    fn process_position_end(&mut self, _fields: &[String]) -> Result<(), IBKRApiLibError> {
        self.wrapper
            .lock()
            .expect(WRAPPER_POISONED_MUTEX)
            .position_end();
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    fn process_position_multi(&mut self, fields: &[String]) -> Result<(), IBKRApiLibError> {
        let mut fields_itr = fields.iter();

        //throw away message_id
        fields_itr.next();
        //throw away version
        fields_itr.next();

        let req_id = decode_i32(&mut fields_itr)?;

        let account = decode_string(&mut fields_itr)?;

        // decode contract fields
        let mut contract = Contract::default();
        contract.con_id = decode_i32(&mut fields_itr)?;
        contract.symbol = decode_string(&mut fields_itr)?;
        contract.sec_type = decode_string(&mut fields_itr)?;
        contract.last_trade_date_or_contract_month = decode_string(&mut fields_itr)?;
        contract.strike = decode_f64(&mut fields_itr)?;
        contract.right = decode_string(&mut fields_itr)?;
        contract.multiplier = decode_string(&mut fields_itr)?;
        contract.exchange = decode_string(&mut fields_itr)?;
        contract.currency = decode_string(&mut fields_itr)?;
        contract.local_symbol = decode_string(&mut fields_itr)?;
        contract.trading_class = decode_string(&mut fields_itr)?;

        let position = decode_f64(&mut fields_itr)?;
        let avg_cost = decode_f64(&mut fields_itr)?;
        let model_code = decode_string(&mut fields_itr)?;

        self.wrapper
            .lock()
            .expect(WRAPPER_POISONED_MUTEX)
            .position_multi(
                req_id,
                account.as_ref(),
                model_code.as_ref(),
                contract,
                position,
                avg_cost,
            );

        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    fn process_position_multi_end(&mut self, fields: &[String]) -> Result<(), IBKRApiLibError> {
        let mut fields_itr = fields.iter();

        //throw away message_id
        fields_itr.next();
        //throw away version
        fields_itr.next();

        let req_id = decode_i32(&mut fields_itr)?;
        self.wrapper
            .lock()
            .expect(WRAPPER_POISONED_MUTEX)
            .position_multi_end(req_id);
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    fn process_real_time_bars(&mut self, fields: &[String]) -> Result<(), IBKRApiLibError> {
        let mut fields_itr = fields.iter();

        //throw away message_id
        fields_itr.next();
        //throw away version
        fields_itr.next();

        let req_id = decode_i32(&mut fields_itr)?;

        let mut bar = RealTimeBar::default();
        bar.date_time = decode_string(&mut fields_itr)?;
        bar.open = decode_f64(&mut fields_itr)?;
        bar.high = decode_f64(&mut fields_itr)?;
        bar.low = decode_f64(&mut fields_itr)?;
        bar.close = decode_f64(&mut fields_itr)?;
        bar.volume = decode_i64(&mut fields_itr)?;
        bar.wap = decode_f64(&mut fields_itr)?;
        bar.count = decode_i32(&mut fields_itr)?;

        self.wrapper
            .lock()
            .expect(WRAPPER_POISONED_MUTEX)
            .realtime_bar(req_id, bar);
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    fn process_receive_fa(&mut self, fields: &[String]) -> Result<(), IBKRApiLibError> {
        let mut fields_itr = fields.iter();

        //throw away message_id
        fields_itr.next();
        //throw away version
        fields_itr.next();

        let fa_data_type = decode_i32(&mut fields_itr)?;
        let xml = decode_string(&mut fields_itr)?;

        self.wrapper
            .lock()
            .expect(WRAPPER_POISONED_MUTEX)
            .receive_fa(FromPrimitive::from_i32(fa_data_type).unwrap(), xml.as_ref());
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    fn process_reroute_mkt_data_req(&mut self, fields: &[String]) -> Result<(), IBKRApiLibError> {
        let mut fields_itr = fields.iter();

        //throw away message_id
        fields_itr.next();

        let req_id = decode_i32(&mut fields_itr)?;
        let con_id = decode_i32(&mut fields_itr)?;
        let exchange = decode_string(&mut fields_itr)?;

        self.wrapper
            .lock()
            .expect(WRAPPER_POISONED_MUTEX)
            .reroute_mkt_data_req(req_id, con_id, exchange.as_ref());
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    fn process_reroute_mkt_depth_req(&mut self, fields: &[String]) -> Result<(), IBKRApiLibError> {
        let mut fields_itr = fields.iter();

        //throw away message_id
        fields_itr.next();

        let req_id = decode_i32(&mut fields_itr)?;
        let con_id = decode_i32(&mut fields_itr)?;
        let exchange = decode_string(&mut fields_itr)?;

        self.wrapper
            .lock()
            .expect(WRAPPER_POISONED_MUTEX)
            .reroute_mkt_depth_req(req_id, con_id, exchange.as_ref());
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    fn process_scanner_data(&mut self, fields: &[String]) -> Result<(), IBKRApiLibError> {
        let mut fields_itr = fields.iter();

        //throw away message_id
        fields_itr.next();
        //throw away version
        fields_itr.next();

        let req_id = decode_i32(&mut fields_itr)?;

        let number_of_elements = decode_i32(&mut fields_itr)?;

        for _ in 0..number_of_elements {
            let mut data = ScanData::default();
            data.contract = ContractDetails::default();

            data.rank = decode_i32(&mut fields_itr)?;
            data.contract.contract.con_id = decode_i32(&mut fields_itr)?; // ver 3 field
            data.contract.contract.symbol = decode_string(&mut fields_itr)?;
            data.contract.contract.sec_type = decode_string(&mut fields_itr)?;
            data.contract.contract.last_trade_date_or_contract_month =
                decode_string(&mut fields_itr)?;
            data.contract.contract.strike = decode_f64(&mut fields_itr)?;
            data.contract.contract.right = decode_string(&mut fields_itr)?;
            data.contract.contract.exchange = decode_string(&mut fields_itr)?;
            data.contract.contract.currency = decode_string(&mut fields_itr)?;
            data.contract.contract.local_symbol = decode_string(&mut fields_itr)?;
            data.contract.market_name = decode_string(&mut fields_itr)?;
            data.contract.contract.trading_class = decode_string(&mut fields_itr)?;
            data.distance = decode_string(&mut fields_itr)?;
            data.benchmark = decode_string(&mut fields_itr)?;
            data.projection = decode_string(&mut fields_itr)?;
            data.legs = decode_string(&mut fields_itr)?;
            self.wrapper
                .lock()
                .expect(WRAPPER_POISONED_MUTEX)
                .scanner_data(
                    req_id,
                    data.rank,
                    data.contract,
                    data.distance.as_ref(),
                    data.benchmark.as_ref(),
                    data.projection.as_ref(),
                    data.legs.as_ref(),
                );
        }

        self.wrapper
            .lock()
            .expect(WRAPPER_POISONED_MUTEX)
            .scanner_data_end(req_id);
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    fn process_scanner_parameters(&mut self, fields: &[String]) -> Result<(), IBKRApiLibError> {
        let mut fields_itr = fields.iter();

        //throw away message_id
        fields_itr.next();
        //throw away version
        fields_itr.next();

        let xml = decode_string(&mut fields_itr)?;
        self.wrapper
            .lock()
            .expect(WRAPPER_POISONED_MUTEX)
            .scanner_parameters(xml.as_ref());
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    fn process_security_definition_option_parameter(
        &mut self,
        fields: &[String],
    ) -> Result<(), IBKRApiLibError> {
        let mut fields_itr = fields.iter();

        //throw away message_id
        fields_itr.next();

        let req_id = decode_i32(&mut fields_itr)?;

        let exchange = decode_string(&mut fields_itr)?;
        let underlying_con_id = decode_i32(&mut fields_itr)?;
        let trading_class = decode_string(&mut fields_itr)?;
        let multiplier = decode_string(&mut fields_itr)?;

        let exp_count = decode_i32(&mut fields_itr)?;
        let mut expirations = HashSet::new();
        for _ in 0..exp_count {
            let expiration = decode_string(&mut fields_itr)?;
            expirations.insert(expiration);
        }

        let strike_count = decode_i32(&mut fields_itr)?;
        let mut strikes = HashSet::new();
        for _ in 0..strike_count {
            let strike = decode_f64(&mut fields_itr)?;
            let big_strike = BigDecimal::from_f64(strike).unwrap();
            strikes.insert(big_strike);
        }

        self.wrapper
            .lock()
            .expect(WRAPPER_POISONED_MUTEX)
            .security_definition_option_parameter(
                req_id,
                exchange.as_ref(),
                underlying_con_id,
                trading_class.as_ref(),
                multiplier.as_ref(),
                expirations,
                strikes,
            );
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    fn process_security_definition_option_parameter_end(
        &mut self,
        fields: &[String],
    ) -> Result<(), IBKRApiLibError> {
        let mut fields_itr = fields.iter();

        //throw away message_id
        fields_itr.next();

        let req_id = decode_i32(&mut fields_itr)?;
        self.wrapper
            .lock()
            .expect(WRAPPER_POISONED_MUTEX)
            .security_definition_option_parameter_end(req_id);
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    fn process_smart_components(&mut self, fields: &[String]) -> Result<(), IBKRApiLibError> {
        let mut fields_itr = fields.iter();

        //throw away message_id
        fields_itr.next();

        let req_id = decode_i32(&mut fields_itr)?;

        let count = decode_i32(&mut fields_itr)?;

        let mut smart_components = vec![];
        for _ in 0..count {
            let mut smart_component = SmartComponent::default();
            smart_component.bit_number = decode_i32(&mut fields_itr)?;
            smart_component.exchange = decode_string(&mut fields_itr)?;
            smart_component.exchange_letter = decode_string(&mut fields_itr)?;
            smart_components.push(smart_component)
        }

        self.wrapper
            .lock()
            .expect(WRAPPER_POISONED_MUTEX)
            .smart_components(req_id, smart_components);
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    fn process_soft_dollar_tiers(&mut self, fields: &[String]) -> Result<(), IBKRApiLibError> {
        let mut fields_itr = fields.iter();

        //throw away message_id
        fields_itr.next();

        let req_id = decode_i32(&mut fields_itr)?;

        let count = decode_i32(&mut fields_itr)?;

        let mut tiers = vec![];
        for _ in 0..count {
            let mut tier = SoftDollarTier::default();
            tier.name = decode_string(&mut fields_itr)?;
            tier.val = decode_string(&mut fields_itr)?;
            tier.display_name = decode_string(&mut fields_itr)?;
            tiers.push(tier);
        }

        self.wrapper
            .lock()
            .expect(WRAPPER_POISONED_MUTEX)
            .soft_dollar_tiers(req_id, tiers);
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    fn process_symbol_samples(&mut self, fields: &[String]) -> Result<(), IBKRApiLibError> {
        let mut fields_itr = fields.iter();

        //throw away message_id
        fields_itr.next();

        let req_id = decode_i32(&mut fields_itr)?;

        let count = decode_i32(&mut fields_itr)?;
        let mut contract_descriptions = vec![];
        for _ in 0..count {
            let mut con_desc = ContractDescription::default();
            con_desc.contract.con_id = decode_i32(&mut fields_itr)?;
            con_desc.contract.symbol = decode_string(&mut fields_itr)?;
            con_desc.contract.sec_type = decode_string(&mut fields_itr)?;
            con_desc.contract.primary_exchange = decode_string(&mut fields_itr)?;
            con_desc.contract.currency = decode_string(&mut fields_itr)?;

            let derivative_sec_types_cnt = decode_i32(&mut fields_itr)?;
            con_desc.derivative_sec_types = vec![];
            for _ in 0..derivative_sec_types_cnt {
                let deriv_sec_type = decode_string(&mut fields_itr)?;
                con_desc.derivative_sec_types.push(deriv_sec_type);
            }
            contract_descriptions.push(con_desc)
        }
        self.wrapper
            .lock()
            .expect(WRAPPER_POISONED_MUTEX)
            .symbol_samples(req_id, contract_descriptions);
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    fn process_tick_by_tick(&mut self, fields: &[String]) -> Result<(), IBKRApiLibError> {
        let mut fields_itr = fields.iter();

        //throw away message_id
        fields_itr.next();

        let req_id = decode_i32(&mut fields_itr)?;

        let tick_type = decode_i32(&mut fields_itr)?;
        let time = decode_i64(&mut fields_itr)?;

        match tick_type {
            0 => return Ok(()), // None
            1..=2 =>
            // Last (1) or AllLast (2)
            {
                let price = decode_f64(&mut fields_itr)?;
                let size = decode_i32(&mut fields_itr)?;
                let mask = decode_i32(&mut fields_itr)?;
                let mut tick_attrib_last = TickAttribLast::default();
                tick_attrib_last.past_limit = mask & 1 != 0;
                tick_attrib_last.unreported = mask & 2 != 0;
                let exchange = decode_string(&mut fields_itr)?;
                let special_conditions = decode_string(&mut fields_itr)?;
                self.wrapper
                    .lock()
                    .expect(WRAPPER_POISONED_MUTEX)
                    .tick_by_tick_all_last(
                        req_id,
                        FromPrimitive::from_i32(tick_type).unwrap(),
                        time,
                        price,
                        size,
                        tick_attrib_last,
                        exchange.as_ref(),
                        special_conditions.as_ref(),
                    );
            }
            3 =>
            // BidAsk
            {
                let bid_price = decode_f64(&mut fields_itr)?;
                let ask_price = decode_f64(&mut fields_itr)?;
                let bid_size = decode_i32(&mut fields_itr)?;
                let ask_size = decode_i32(&mut fields_itr)?;
                let mask = decode_i32(&mut fields_itr)?;
                let mut tick_attrib_bid_ask = TickAttribBidAsk::default();
                tick_attrib_bid_ask.bid_past_low = mask & 1 != 0;
                tick_attrib_bid_ask.ask_past_high = mask & 2 != 0;
                self.wrapper
                    .lock()
                    .expect(WRAPPER_POISONED_MUTEX)
                    .tick_by_tick_bid_ask(
                        req_id,
                        time,
                        bid_price,
                        ask_price,
                        bid_size,
                        ask_size,
                        tick_attrib_bid_ask,
                    );
            }
            4 =>
            // MidPoint
            {
                let mid_point = decode_f64(&mut fields_itr)?;
                self.wrapper
                    .lock()
                    .expect(WRAPPER_POISONED_MUTEX)
                    .tick_by_tick_mid_point(req_id, time, mid_point);
            }
            _ => return Ok(()),
        }
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    #[allow(dead_code)]
    fn process_tick_efp(&mut self, fields: &[String]) -> Result<(), IBKRApiLibError> {
        let mut fields_itr = fields.iter();

        //throw away message_id
        fields_itr.next();
        //throw away version
        fields_itr.next();

        let ticker_id = decode_i32(&mut fields_itr)?;
        let tick_type = decode_i32(&mut fields_itr)?;
        let basis_points = decode_f64(&mut fields_itr)?;
        let formatted_basis_points = decode_string(&mut fields_itr)?;
        let implied_futures_price = decode_f64(&mut fields_itr)?;
        let hold_days = decode_i32(&mut fields_itr)?;
        let future_last_trade_date = decode_string(&mut fields_itr)?;
        let dividend_impact = decode_f64(&mut fields_itr)?;
        let dividends_to_last_trade_date = decode_f64(&mut fields_itr)?;
        self.wrapper.lock().expect(WRAPPER_POISONED_MUTEX).tick_efp(
            ticker_id,
            FromPrimitive::from_i32(tick_type).unwrap(),
            basis_points,
            formatted_basis_points.as_ref(),
            implied_futures_price,
            hold_days,
            future_last_trade_date.as_ref(),
            dividend_impact,
            dividends_to_last_trade_date,
        );
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    fn process_tick_generic(&mut self, fields: &[String]) -> Result<(), IBKRApiLibError> {
        let mut fields_itr = fields.iter();

        //throw away message_id
        fields_itr.next();
        //throw away version
        fields_itr.next();

        let ticker_id = decode_i32(&mut fields_itr)?;
        let tick_type = decode_i32(&mut fields_itr)?;
        let value = decode_f64(&mut fields_itr)?;

        self.wrapper
            .lock()
            .expect(WRAPPER_POISONED_MUTEX)
            .tick_generic(
                ticker_id,
                FromPrimitive::from_i32(tick_type).unwrap(),
                value,
            );
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    fn process_tick_news(&mut self, fields: &[String]) -> Result<(), IBKRApiLibError> {
        let mut fields_itr = fields.iter();

        //throw away message_id
        fields_itr.next();
        let ticker_id = decode_i32(&mut fields_itr)?;
        let time_stamp = decode_i32(&mut fields_itr)?;
        let provider_code = decode_string(&mut fields_itr)?;
        let article_id = decode_string(&mut fields_itr)?;
        let headline = decode_string(&mut fields_itr)?;
        let extra_data = decode_string(&mut fields_itr)?;
        self.wrapper
            .lock()
            .expect(WRAPPER_POISONED_MUTEX)
            .tick_news(
                ticker_id,
                time_stamp,
                provider_code.as_ref(),
                article_id.as_ref(),
                headline.as_ref(),
                extra_data.as_ref(),
            );
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    fn process_tick_option_computation(
        &mut self,
        fields: &[String],
    ) -> Result<(), IBKRApiLibError> {
        let mut fields_itr = fields.iter();

        //throw away message_id
        fields_itr.next();

        let version = decode_i32(&mut fields_itr)?;
        let ticker_id = decode_i32(&mut fields_itr)?;
        let tick_type = decode_i32(&mut fields_itr)?;
        let mut implied_vol = decode_f64(&mut fields_itr)?;
        if approx_eq!(f64, implied_vol, -1.0, ulps = 2) {
            // -1 is the "not yet computed" indicator
            implied_vol = f64::max_value();
        }

        let mut delta = decode_f64(&mut fields_itr)?;
        if approx_eq!(f64, delta, -2.0, ulps = 2) {
            // -2 is the "not yet computed" indicator
            delta = f64::max_value();
        }
        let mut opt_price = f64::max_value();
        let mut pv_dividend = f64::max_value();
        let mut gamma = f64::max_value();
        let mut vega = f64::max_value();
        let mut theta = f64::max_value();
        let mut und_price = f64::max_value();
        if version >= 6
            || tick_type == TickType::ModelOption as i32
            || tick_type == TickType::DelayedModelOption as i32
        {
            // introduced in version == 5
            opt_price = decode_f64(&mut fields_itr)?;
            if approx_eq!(f64, opt_price, -1.0, ulps = 2) {
                // -1 is the "not yet computed" indicator
                opt_price = f64::max_value();
            }
            pv_dividend = decode_f64(&mut fields_itr)?;
            if approx_eq!(f64, pv_dividend, -1.0, ulps = 2) {
                // -1 is the "not yet computed" indicator
                pv_dividend = f64::max_value();
            }
        }
        if version >= 6 {
            gamma = decode_f64(&mut fields_itr)?;
            if approx_eq!(f64, gamma, -2.0, ulps = 2) {
                // -2 is the "not yet computed" indicator
                gamma = f64::max_value();
            }
            vega = decode_f64(&mut fields_itr)?;
            if approx_eq!(f64, vega, -2.0, ulps = 2) {
                // -2 is the "not yet computed" indicator
                vega = f64::max_value();
            }
            theta = decode_f64(&mut fields_itr)?;
            if approx_eq!(f64, theta, -2.0, ulps = 2) {
                // -2 is the "not yet computed" indicator
                theta = f64::max_value();
            }
            und_price = decode_f64(&mut fields_itr)?;
            if approx_eq!(f64, und_price, -1.0, ulps = 2) {
                // -1 is the "not yet computed" indicator
                und_price = f64::max_value();
            }
        }

        self.wrapper
            .lock()
            .expect(WRAPPER_POISONED_MUTEX)
            .tick_option_computation(
                ticker_id,
                FromPrimitive::from_i32(tick_type).unwrap(),
                implied_vol,
                delta,
                opt_price,
                pv_dividend,
                gamma,
                vega,
                theta,
                und_price,
            );
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    fn process_tick_req_params(&mut self, fields: &[String]) -> Result<(), IBKRApiLibError> {
        let mut fields_itr = fields.iter();

        //throw away message_id
        fields_itr.next();

        let ticker_id = decode_i32(&mut fields_itr)?;
        let min_tick = decode_f64(&mut fields_itr)?;
        let bbo_exchange = decode_string(&mut fields_itr)?;
        let snapshot_permissions = decode_i32(&mut fields_itr)?;
        self.wrapper
            .lock()
            .expect(WRAPPER_POISONED_MUTEX)
            .tick_req_params(
                ticker_id,
                min_tick,
                bbo_exchange.as_ref(),
                snapshot_permissions,
            );
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    fn process_tick_size(&mut self, fields: &[String]) -> Result<(), IBKRApiLibError> {
        let mut fields_itr = fields.iter();

        //throw away message_id
        fields_itr.next();
        //throw away version
        fields_itr.next();

        let ticker_id = decode_i32(&mut fields_itr)?;
        let tick_type = decode_i32(&mut fields_itr)?;
        let size = decode_i32(&mut fields_itr)?;

        self.wrapper
            .lock()
            .expect(WRAPPER_POISONED_MUTEX)
            .tick_size(ticker_id, FromPrimitive::from_i32(tick_type).unwrap(), size);
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    fn process_tick_snapshot_end(&mut self, fields: &[String]) -> Result<(), IBKRApiLibError> {
        let mut fields_itr = fields.iter();

        //throw away message_id
        fields_itr.next();
        //throw away version
        fields_itr.next();

        let req_id = decode_i32(&mut fields_itr)?;

        self.wrapper
            .lock()
            .expect(WRAPPER_POISONED_MUTEX)
            .tick_snapshot_end(req_id);
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    fn process_verify_and_auth_completed(
        &mut self,
        fields: &[String],
    ) -> Result<(), IBKRApiLibError> {
        let mut fields_itr = fields.iter();

        //throw away message_id
        fields_itr.next();
        //throw away version
        fields_itr.next();
        let _is_successful_str = decode_string(&mut fields_itr)?;
        let is_successful = "true" == decode_string(&mut fields_itr)?;
        let error_text = decode_string(&mut fields_itr)?;

        self.wrapper
            .lock()
            .expect(WRAPPER_POISONED_MUTEX)
            .verify_and_auth_completed(is_successful, error_text.as_ref());
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    fn process_verify_and_auth_message_api(
        &mut self,
        fields: &[String],
    ) -> Result<(), IBKRApiLibError> {
        let mut fields_itr = fields.iter();

        //throw away message_id
        fields_itr.next();
        //throw away version
        fields_itr.next();

        let api_data = decode_string(&mut fields_itr)?;
        let xyz_challenge = decode_string(&mut fields_itr)?;

        self.wrapper
            .lock()
            .expect(WRAPPER_POISONED_MUTEX)
            .verify_and_auth_message_api(api_data.as_ref(), xyz_challenge.as_ref());
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    fn process_verify_completed(&mut self, fields: &[String]) -> Result<(), IBKRApiLibError> {
        let mut fields_itr = fields.iter();
        //throw away message_id
        fields_itr.next();
        //throw away version
        fields_itr.next();

        let _is_successful_str = decode_string(&mut fields_itr)?;
        let is_successful = "true" == decode_string(&mut fields_itr)?;
        let error_text = decode_string(&mut fields_itr)?;

        self.wrapper
            .lock()
            .expect(WRAPPER_POISONED_MUTEX)
            .verify_completed(is_successful, error_text.as_ref());
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    #[allow(dead_code)]
    fn process_verify_message_api(&mut self, fields: &[String]) -> Result<(), IBKRApiLibError> {
        let mut fields_itr = fields.iter();
        //throw away message_id
        fields_itr.next();
        //throw away version
        fields_itr.next();

        let api_data = decode_string(&mut fields_itr)?;

        self.wrapper
            .lock()
            .expect(WRAPPER_POISONED_MUTEX)
            .verify_message_api(api_data.as_ref());
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    fn read_last_trade_date(
        &self,
        contract: &mut ContractDetails,
        is_bond: bool,
        read_date: &str,
    ) -> Result<(), IBKRApiLibError> {
        if read_date != "" {
            let splitted = read_date.split_whitespace().collect::<Vec<&str>>();
            if !splitted.is_empty() {
                if is_bond {
                    contract.maturity = splitted.get(0).unwrap_or_else(|| &"").to_string();
                } else {
                    contract.contract.last_trade_date_or_contract_month =
                        splitted.get(0).unwrap_or_else(|| &"").to_string();
                }
            }
            if splitted.len() > 1 {
                contract.last_trade_time = splitted.get(1).unwrap_or_else(|| &"").to_string();
            }
            if is_bond && splitted.len() > 2 {
                contract.time_zone_id = splitted.get(2).unwrap_or_else(|| &"").to_string();
            }
        }
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    pub fn run(&mut self) -> Result<(), IBKRApiLibError> {
        //This is the function that has the message loop.
        const CONN_STATE_POISONED: &str = "Connection state mutex was poisoned";
        info!("Starting run...");
        // !self.done &&
        loop {
            // debug!("Client waiting for message...");

            let text = self.msg_queue.recv();
            match text {
                Result::Ok(val) => {
                    if val.len() > MAX_MSG_LEN as usize {
                        self.wrapper.lock().expect(WRAPPER_POISONED_MUTEX).error(
                            NO_VALID_ID,
                            TwsError::NotConnected.code(),
                            format!("{}:{}:{}", TwsError::NotConnected.message(), val.len(), val)
                                .as_str(),
                        );
                        error!("Error receiving message.  Disconnected: Message too big");
                        self.wrapper
                            .lock()
                            .expect(WRAPPER_POISONED_MUTEX)
                            .connection_closed();
                        *self.conn_state.lock().expect(CONN_STATE_POISONED) =
                            ConnStatus::DISCONNECTED;
                        error!("Error receiving message.  Invalid size.  Disconnected.");
                        return Ok(());
                    } else {
                        let fields = read_fields((&val).as_ref());

                        self.interpret(fields.as_slice())?;
                    }
                }
                Result::Err(err) => {
                    if *self.conn_state.lock().expect(CONN_STATE_POISONED).deref() as i32
                        != ConnStatus::DISCONNECTED as i32
                    {
                        info!("Error receiving message.  Disconnected: {:?}", err);
                        self.wrapper
                            .lock()
                            .expect(WRAPPER_POISONED_MUTEX)
                            .connection_closed();
                        *self.conn_state.lock().expect(CONN_STATE_POISONED) =
                            ConnStatus::DISCONNECTED;

                        return Ok(());
                    } else {
                        error!("Disconnected...");
                        return Ok(());
                    }
                }
            }
        }
    }
}
