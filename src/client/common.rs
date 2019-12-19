use std::fmt;

use serde::export::fmt::Error;
use serde::export::Formatter;

pub const NO_VALID_ID: i32 = -1;
pub const MAX_MSG_LEN: i64 = 0xFFFFFF; //16Mb - 1byte

pub const UNSET_INTEGER: i32 = std::i32::MAX;
pub const UNSET_DOUBLE: f64 = std::f64::MAX;
pub const UNSET_LONG: i64 = std::i64::MAX;

pub enum FaDataType {
    NA = 0,
    GROUPS = 1,
    PROFILES = 2,
    ALIASES = 3,
}

//==================================================================================================
#[derive()]
pub struct OrderState {
    status: String,
    init_margin_before: String,
    maint_margin_before: String,
    equity_with_loan_before: String,
    init_margin_change: String,
    maint_margin_change: String,
    equity_with_loan_change: String,
    init_margin_after: String,
    maint_margin_after: String,
    equity_with_loan_after: String,
    commission: f64,
    min_commission: f64,
    max_commission: f64,
    commission_currency: String,
    warning_text: String,
    completed_time: String,
    completed_status: String,
}

impl OrderState {
    pub fn new(
        status: String,
        init_margin_before: String,
        maint_margin_before: String,
        equity_with_loan_before: String,
        init_margin_change: String,
        maint_margin_change: String,
        equity_with_loan_change: String,
        init_margin_after: String,
        maint_margin_after: String,
        equity_with_loan_after: String,
        commission: f64,
        min_commission: f64,
        max_commission: f64,
        commission_currency: String,
        warning_text: String,
        completed_time: String,
        completed_status: String,
    ) -> Self {
        OrderState {
            status,
            init_margin_before,
            maint_margin_before,
            equity_with_loan_before,
            init_margin_change,
            maint_margin_change,
            equity_with_loan_change,
            init_margin_after,
            maint_margin_after,
            equity_with_loan_after,
            commission,
            min_commission,
            max_commission,
            commission_currency,
            warning_text,
            completed_time,
            completed_status,
        }
    }
}

impl fmt::Display for OrderState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "status: {},
            init_margin_before: {},
            maint_margin_before: {},
            equity_with_loan_before: {},
            init_margin_change: {},
            maint_margin_change: {},
            equity_with_loan_change: {},
            init_margin_after: {},
            maint_margin_after: {},
            equity_with_loan_after: {},
            commission: {},
            min_commission: {},
            max_commission: {},
            commission_currency: {},
            warning_text: {},
            completed_time: {},
            completed_status: {},",
            self.status,
            self.init_margin_before,
            self.maint_margin_before,
            self.equity_with_loan_before,
            self.init_margin_change,
            self.maint_margin_change,
            self.equity_with_loan_change,
            self.init_margin_after,
            self.maint_margin_after,
            self.equity_with_loan_after,
            self.commission,
            self.min_commission,
            self.max_commission,
            self.commission_currency,
            self.warning_text,
            self.completed_time,
            self.completed_status,
        )
    }
}

//==================================================================================================
pub struct BarData {
    date: String,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: i32,
    bar_count: i32,
    average: f64,
}

impl BarData {
    pub fn new(
        date: String,
        open: f64,
        high: f64,
        low: f64,
        close: f64,
        volume: i32,
        bar_count: i32,
        average: f64,
    ) -> Self {
        BarData {
            date,
            open,
            high,
            low,
            close,
            volume,
            bar_count,
            average,
        }
    }
}

impl fmt::Display for BarData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "date: {}, open: {}, high: {}, low: {}, close: {}, volume: {}, average: {}, barcount: {}", self.date, self.open, self.high,
               self.low, self.close, self.volume, self.average, self.bar_count)
    }
}

//==================================================================================================
pub struct RealTimeBar {
    time: String,
    end_time: String,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: i32,
    wap: f64,
    count: i32,
}

impl RealTimeBar {
    pub fn new(
        time: String,
        end_time: String,
        open: f64,
        high: f64,
        low: f64,
        close: f64,
        volume: i32,
        wap: f64,
        count: i32,
    ) -> Self {
        RealTimeBar {
            time,
            end_time,
            open,
            high,
            low,
            close,
            volume,
            wap,
            count,
        }
    }
}

impl fmt::Display for RealTimeBar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "time: {}, end_time: {},open: {}, high: {}, low: {}, close: {}, volume: {}, wap: {}, count: {}", self.time, self.end_time, self.open, self.high,
               self.low, self.close, self.volume, self.wap, self.count)
    }
}

//==================================================================================================
pub struct HistogramData {
    price: f64,
    count: i32,
}

impl HistogramData {
    pub fn new(price: f64, count: i32) -> Self {
        HistogramData { price, count }
    }
}

impl fmt::Display for HistogramData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "price: {}, count: {}", self.price, self.count)
    }
}

//==================================================================================================
pub struct DepthMktDataDescription {
    exchange: String,
    sec_type: String,
    listing_exch: String,
    service_data_type: String,
    agg_group: i32,
}

impl DepthMktDataDescription {
    pub fn new(
        exchange: String,
        sec_type: String,
        listing_exch: String,
        service_data_type: String,
        agg_group: i32,
    ) -> Self {
        DepthMktDataDescription {
            exchange,
            sec_type,
            listing_exch,
            service_data_type,
            agg_group,
        }
    }
}

impl fmt::Display for DepthMktDataDescription {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "exchange: {}, sectype: {}, listing_exchange: {}, service_data_type: {}, agggroup: {}, ",
            self.exchange,
            self.sec_type,
            self.listing_exch,
            self.service_data_type,
            if self.agg_group != UNSET_INTEGER {
                format!("{}", self.agg_group)
            } else {
                "".to_string()
            }
        )
    }
}

//==================================================================================================
pub struct SmartComponent {
    bit_number: i32,
    exchange: String,
    exchange_letter: String,
}

impl SmartComponent {
    pub fn new(bit_number: i32, exchange: String, exchange_letter: String) -> Self {
        SmartComponent {
            bit_number,
            exchange,
            exchange_letter,
        }
    }
}

impl fmt::Display for SmartComponent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "bit_number: {}, exchange: {}, exchange_letter: {}",
            self.bit_number, self.exchange, self.exchange_letter
        )
    }
}

//==================================================================================================
pub struct TickAttrib {
    can_auto_execute: bool,
    past_limit: bool,
    pre_open: bool,
}

impl TickAttrib {
    pub fn new(can_auto_execute: bool, past_limit: bool, pre_open: bool) -> Self {
        TickAttrib {
            can_auto_execute,
            past_limit,
            pre_open,
        }
    }
}

impl fmt::Display for TickAttrib {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "can_auto_execute: {}, past_limit: {}, pre_open: {}",
            self.can_auto_execute, self.past_limit, self.pre_open
        )
    }
}

//==================================================================================================
pub struct TickAttribBidAsk {
    bid_past_low: bool,
    ask_past_high: bool,
}

impl TickAttribBidAsk {
    pub fn new(bid_past_low: bool, ask_past_high: bool) -> Self {
        TickAttribBidAsk {
            bid_past_low,
            ask_past_high,
        }
    }
}

impl fmt::Display for TickAttribBidAsk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "bid_past_low: {}, ask_past_high: {}",
            self.bid_past_low, self.ask_past_high
        )
    }
}

//==================================================================================================
pub struct TickAttribLast {
    past_limit: bool,
    unreported: bool,
}

impl TickAttribLast {
    pub fn new(past_limit: bool, unreported: bool) -> Self {
        TickAttribLast {
            past_limit,
            unreported,
        }
    }
}

impl fmt::Display for TickAttribLast {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "past_limit: {}, unreported: {}",
            self.past_limit, self.unreported
        )
    }
}

//==================================================================================================
pub struct FamilyCode {
    account_id: String,
    family_code_str: String,
}

impl FamilyCode {
    pub fn new(account_id: String, family_code_str: String) -> Self {
        FamilyCode {
            account_id,
            family_code_str,
        }
    }
}

impl fmt::Display for FamilyCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "account_id: {}, family_code_str: {}",
            self.account_id, self.family_code_str
        )
    }
}

//==================================================================================================
pub struct PriceIncrement {
    low_edge: f64,
    increment: f64,
}

impl PriceIncrement {
    pub fn new(low_edge: f64, increment: f64) -> Self {
        PriceIncrement {
            low_edge,
            increment,
        }
    }
}

impl fmt::Display for PriceIncrement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "low_edge: {}, increment: {}",
            self.low_edge, self.increment
        )
    }
}

//==================================================================================================
pub struct HistoricalTick {
    time: i32,
    price: f64,
    size: i32,
}

impl HistoricalTick {
    pub fn new(time: i32, price: f64, size: i32) -> Self {
        HistoricalTick { time, price, size }
    }
}

impl fmt::Display for HistoricalTick {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "time: {}, price: {}, size: {}",
            self.time, self.price, self.size
        )
    }
}

//==================================================================================================
pub struct HistoricalTickBidAsk {
    time: i32,
    tick_attrib_bid_ask: TickAttribBidAsk,
    price_bid: f64,
    price_ask: f64,
    size_bid: i32,
    size_ask: i32,
}

impl HistoricalTickBidAsk {
    pub fn new(
        time: i32,
        tick_attrib_bid_ask: TickAttribBidAsk,
        price_bid: f64,
        price_ask: f64,
        size_bid: i32,
        size_ask: i32,
    ) -> Self {
        HistoricalTickBidAsk {
            time,
            tick_attrib_bid_ask,
            price_bid,
            price_ask,
            size_bid,
            size_ask,
        }
    }
}

impl fmt::Display for HistoricalTickBidAsk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "time: {}, tick_attrib_bid_ask: {}, price_bid: {}, price_ask: {}, size_bid: {}, size_ask: {}",
            self.time,
            self.tick_attrib_bid_ask,
            self.price_bid,
            self.price_ask,
            self.size_bid,
            self.size_ask
        )
    }
}

//==================================================================================================
pub struct HistoricalTickLast {
    time: i32,
    tick_attrib_last: TickAttribLast,
    price: f64,
    size: i32,
    exchange: String,
    special_conditions: String,
}

impl HistoricalTickLast {
    pub fn new(
        time: i32,
        tick_attrib_last: TickAttribLast,
        price: f64,
        size: i32,
        exchange: String,
        special_conditions: String,
    ) -> Self {
        HistoricalTickLast {
            time,
            tick_attrib_last,
            price,
            size,
            exchange,
            special_conditions,
        }
    }
}

impl fmt::Display for HistoricalTickLast {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "time: {}, tick_attrib_last: {}, price: {}, size: {}, exchange: {}, special_conditions: {}",
               self.time,
               self.tick_attrib_last,
               self.price,
               self.size,
               self.exchange,
               self.special_conditions)
    }
}

//==================================================================================================
pub struct CommissionReport {
    exec_id: String,
    commission: f64,
    currency: String,
    realized_pnl: f64,
    yield_: f64,
    yield_redemption_date: i32, //YYYYMMDD format
}

impl CommissionReport {
    pub fn new(
        exec_id: String,
        commission: f64,
        currency: String,
        realized_pnl: f64,
        yield_: f64,
        yield_redemption_date: i32,
    ) -> Self {
        CommissionReport {
            exec_id,
            commission,
            currency,
            realized_pnl,
            yield_,
            yield_redemption_date,
        }
    }
}

impl fmt::Display for CommissionReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "exec_id: {}, commission: {}, currency: {}, realized_pnl: {}, yield_: {}, yield_redemption_date: {}",
               self.exec_id,
               self.commission,
               self.currency,
               if self.realized_pnl != UNSET_DOUBLE { format!("{}", self.realized_pnl) } else { "".to_string() },
               if self.yield_ != UNSET_DOUBLE { format!("{}", self.yield_) } else { "".to_string() },
               self.yield_redemption_date)
    }
}

//==================================================================================================
pub struct NewsProvider {
    code: String,
    name: String,
}

impl NewsProvider {
    pub fn new(code: String, name: String) -> Self {
        NewsProvider { code, name }
    }
}

impl fmt::Display for NewsProvider {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "code: {}, name: {}", self.code, self.name,)
    }
}

//==================================================================================================
pub struct TagValue {
    tag: String,
    value: String,
}

impl TagValue {
    pub fn new(tag: String, value: String) -> Self {
        TagValue { tag, value }
    }
}

impl fmt::Display for TagValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "tag: {}, value: {}", self.tag, self.value,)
    }
}
/*
def __str__(self):
# this is not only used for Python dump but when encoding to send
# so don't change it lightly !
return "%s=%s;" % (self.tag, self.value)
*/
