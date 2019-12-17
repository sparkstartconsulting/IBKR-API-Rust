use std::io::Write;
use std::net::TcpStream;
use std::string::ToString;
use std::u8;

use crate::client;
use crate::client::messages::IncomingMessageIds;
use crate::client::wrapper::Wrapper;
use bytebuffer::ByteBuffer;
use num_traits::FromPrimitive;

const SEP: u8 = '\0' as u8;
const EMPTY_LENGTH_HEADER: [u8; 4] = [0; 4];

trait Sender<T> {
    fn send(&mut self, a: T);
}

pub struct Builder {
    server_version: i32,
    buffer: ByteBuffer,
}

impl Builder {
    pub fn new(server_version: i32) -> Self {
        Builder {
            server_version,
            buffer: ByteBuffer::new(),
        }
    }

    pub fn write_out(&self, mut stream: &mut dyn std::io::Write) {
        stream.write(self.buffer.to_bytes().as_slice());
    }

    pub fn reset(&mut self) {
        self.buffer.clear();
    }
}

impl Sender<i32> for Builder {
    fn send(&mut self, a: i32) {
        self.buffer.write_i32(a);
        self.buffer.write_u8(SEP);
    }
}

impl Sender<i64> for Builder {
    fn send(&mut self, a: i64) {
        self.buffer.write_i64(a);
        self.buffer.write_u8(SEP);
    }
}

impl Sender<f32> for Builder {
    fn send(&mut self, a: f32) {
        self.buffer.write_f32(a);
        self.buffer.write_u8(SEP);
    }
}

impl Sender<f64> for Builder {
    fn send(&mut self, a: f64) {
        self.buffer.write_f64(a);
        self.buffer.write_u8(SEP);
    }
}

impl Sender<bool> for Builder {
    fn send(&mut self, a: bool) {
        self.buffer.write_bit(a);
        self.buffer.write_u8(SEP);
    }
}

impl Sender<&str> for Builder {
    fn send(&mut self, a: &str) {
        self.buffer.write_string(a);
        self.buffer.write_u8(SEP);
    }
}

pub struct Decoder<'a, T: Wrapper> {
    wrapper: &'a mut T,
    server_version: i32,
}

impl<'a, T> Decoder<'a, T>
where
    T: client::wrapper::Wrapper,
{
    pub fn new(wrapper: &'a mut T, server_version: i32) -> Self {
        Decoder {
            wrapper,
            server_version,
        }
    }

    pub fn interpret(&mut self, fields: &[String]) {
        if fields.is_empty() {
            return;
        }

        let msg_id = fields.get(0).unwrap().parse::<i32>().unwrap();

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

    fn process_tick_price(&mut self, fields: &[String]) {}
    fn process_account_summary(&mut self, fields: &[String]) {}
    fn process_account_summary_end(&mut self, fields: &[String]) {}
    fn process_account_update_multi(&mut self, fields: &[String]) {}
    fn process_account_update_multi_end(&mut self, fields: &[String]) {}
    fn process_account_download_end(&mut self, fields: &[String]) {}
    fn process_account_update_time(&mut self, fields: &[String]) {}
    fn process_account_value(&mut self, fields: &[String]) {}
    fn process_bond_contract_data(&mut self, fields: &[String]) {}
    fn process_commission_report(&mut self, fields: &[String]) {}
    fn process_completed_order(&mut self, fields: &[String]) {}
    fn process_complete_orders_end(&mut self, fields: &[String]) {}
    fn process_contract_data(&mut self, fields: &[String]) {}
    fn process_contract_data_end(&mut self, fields: &[String]) {}
    fn process_current_time(&mut self, fields: &[String]) {}
    fn process_delta_neutral_validation(&mut self, fields: &[String]) {}
    fn process_display_group_list(&mut self, fields: &[String]) {}
    fn process_display_group_updated(&mut self, fields: &[String]) {}
    fn process_error_message(&mut self, fields: &[String]) {}
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
    fn process_tick_string(&mut self, fields: &[String]) {}
    fn process_verify_and_auth_completed(&mut self, fields: &[String]) {}
    fn process_verify_and_auth_message_api(&mut self, fields: &[String]) {}
    fn process_verify_completed(&mut self, fields: &[String]) {}
    fn process_verify_message_api(&mut self, fields: &[String]) {}
}
