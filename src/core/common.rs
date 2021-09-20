#![allow(clippy::too_many_arguments)]
//! Common types
use std::fmt::Display;
use std::fmt::{self, Error, Formatter};

use num_derive::FromPrimitive;

use serde::{Deserialize, Serialize};

pub const NO_VALID_ID: i32 = -1;
pub const MAX_MSG_LEN: i64 = 0xFFFFFF; //16Mb - 1byte

pub const UNSET_INTEGER: i32 = std::i32::MAX;
pub const UNSET_DOUBLE: f64 = 1.7976931348623157E308_f64;
pub const UNSET_LONG: i64 = std::i64::MAX;

//==================================================================================================
/// Tick types
#[repr(i32)]
#[derive(Serialize, Deserialize, Clone, Debug, FromPrimitive, Copy)]
pub enum TickType {
    BidSize = 0,
    Bid = 1,
    Ask = 2,
    AskSize = 3,
    Last = 4,
    LastSize = 5,
    High = 6,
    Low = 7,
    Volume = 8,
    Close = 9,
    BidOptionComputation = 10,
    AskOptionComputation = 11,
    LastOptionComputation = 12,
    ModelOption = 13,
    Open = 14,
    Low13Week = 15,
    High13Week = 16,
    Low26Week = 17,
    High26Week = 18,
    Low52Week = 19,
    High52Week = 20,
    AvgVolume = 21,
    OpenInterest = 22,
    OptionHistoricalVol = 23,
    OptionImpliedVol = 24,
    OptionBidExch = 25,
    OptionAskExch = 26,
    OptionCallOpenInterest = 27,
    OptionPutOpenInterest = 28,
    OptionCallVolume = 29,
    OptionPutVolume = 30,
    IndexFuturePremium = 31,
    BidExch = 32,
    AskExch = 33,
    AuctionVolume = 34,
    AuctionPrice = 35,
    AuctionImbalance = 36,
    MarkPrice = 37,
    BidEfpComputation = 38,
    AskEfpComputation = 39,
    LastEfpComputation = 40,
    OpenEfpComputation = 41,
    HighEfpComputation = 42,
    LowEfpComputation = 43,
    CloseEfpComputation = 44,
    LastTimestamp = 45,
    Shortable = 46,
    FundamentalRatios = 47,
    RtVolume = 48,
    Halted = 49,
    BidYield = 50,
    AskYield = 51,
    LastYield = 52,
    CustOptionComputation = 53,
    TradeCount = 54,
    TradeRate = 55,
    VolumeRate = 56,
    LastRthTrade = 57,
    RtHistoricalVol = 58,
    IbDividends = 59,
    BondFactorMultiplier = 60,
    RegulatoryImbalance = 61,
    NewsTick = 62,
    ShortTermVolume3Min = 63,
    ShortTermVolume5Min = 64,
    ShortTermVolume10Min = 65,
    DelayedBid = 66,
    DelayedAsk = 67,
    DelayedLast = 68,
    DelayedBidSize = 69,
    DelayedAskSize = 70,
    DelayedLastSize = 71,
    DelayedHigh = 72,
    DelayedLow = 73,
    DelayedVolume = 74,
    DelayedClose = 75,
    DelayedOpen = 76,
    RtTrdVolume = 77,
    CreditmanMarkPrice = 78,
    CreditmanSlowMarkPrice = 79,
    DelayedBidOption = 80,
    DelayedAskOption = 81,
    DelayedLastOption = 82,
    DelayedModelOption = 83,
    LastExch = 84,
    LastRegTime = 85,
    FuturesOpenInterest = 86,
    AvgOptVolume = 87,
    DelayedLastTimestamp = 88,
    ShortableShares = 89,
    NotSet = UNSET_INTEGER,
}

impl fmt::Display for TickType {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            TickType::BidSize => write!(fmt, "bidSize"),
            TickType::Bid => write!(fmt, "bidPrice"),
            TickType::Ask => write!(fmt, "askPrice"),
            TickType::AskSize => write!(fmt, "askSize"),
            TickType::Last => write!(fmt, "lastPrice"),
            TickType::LastSize => write!(fmt, "lastSize"),
            TickType::High => write!(fmt, "high"),
            TickType::Low => write!(fmt, "low"),
            TickType::Volume => write!(fmt, "volume"),
            TickType::Close => write!(fmt, "close"),
            TickType::BidOptionComputation => write!(fmt, "bidOptComp"),
            TickType::AskOptionComputation => write!(fmt, "askOptComp"),
            TickType::LastOptionComputation => write!(fmt, "lastOptComp"),
            TickType::ModelOption => write!(fmt, "modelOptComp"),
            TickType::Open => write!(fmt, "open"),
            TickType::Low13Week => write!(fmt, "13WeekLow"),
            TickType::High13Week => write!(fmt, "13WeekHigh"),
            TickType::Low26Week => write!(fmt, "26WeekLow"),
            TickType::High26Week => write!(fmt, "26WeekHigh"),
            TickType::Low52Week => write!(fmt, "52WeekLow"),
            TickType::High52Week => write!(fmt, "52WeekHigh"),
            TickType::AvgVolume => write!(fmt, "AvgVolume"),
            TickType::OpenInterest => write!(fmt, "OpenInterest"),
            TickType::OptionHistoricalVol => write!(fmt, "OptionHistoricalVolatility"),
            TickType::OptionImpliedVol => write!(fmt, "OptionImpliedVolatility"),
            TickType::OptionBidExch => write!(fmt, "OptionBidExch"),
            TickType::OptionAskExch => write!(fmt, "OptionAskExch"),
            TickType::OptionCallOpenInterest => write!(fmt, "OptionCallOpenInterest"),
            TickType::OptionPutOpenInterest => write!(fmt, "OptionPutOpenInterest"),
            TickType::OptionCallVolume => write!(fmt, "OptionCallVolume"),
            TickType::OptionPutVolume => write!(fmt, "OptionPutVolume"),
            TickType::IndexFuturePremium => write!(fmt, "IndexFuturePremium"),
            TickType::BidExch => write!(fmt, "bidExch"),
            TickType::AskExch => write!(fmt, "askExch"),
            TickType::AuctionVolume => write!(fmt, "auctionVolume"),
            TickType::AuctionPrice => write!(fmt, "auctionPrice"),
            TickType::AuctionImbalance => write!(fmt, "auctionImbalance"),
            TickType::MarkPrice => write!(fmt, "markPrice"),
            TickType::BidEfpComputation => write!(fmt, "bidEFP"),
            TickType::AskEfpComputation => write!(fmt, "askEFP"),
            TickType::LastEfpComputation => write!(fmt, "lastEFP"),
            TickType::OpenEfpComputation => write!(fmt, "openEFP"),
            TickType::HighEfpComputation => write!(fmt, "highEFP"),
            TickType::LowEfpComputation => write!(fmt, "lowEFP"),
            TickType::CloseEfpComputation => write!(fmt, "closeEFP"),
            TickType::LastTimestamp => write!(fmt, "lastTimestamp"),
            TickType::Shortable => write!(fmt, "shortable"),
            TickType::FundamentalRatios => write!(fmt, "fundamentals"),
            TickType::RtVolume => write!(fmt, "RTVolume"),
            TickType::Halted => write!(fmt, "halted"),
            TickType::BidYield => write!(fmt, "bidYield"),
            TickType::AskYield => write!(fmt, "askYield"),
            TickType::LastYield => write!(fmt, "lastYield"),
            TickType::CustOptionComputation => write!(fmt, "custOptComp"),
            TickType::TradeCount => write!(fmt, "tradeCount"),
            TickType::TradeRate => write!(fmt, "tradeRate"),
            TickType::VolumeRate => write!(fmt, "volumeRate"),
            TickType::LastRthTrade => write!(fmt, "lastRTHTrade"),
            TickType::RtHistoricalVol => write!(fmt, "RTHistoricalVol"),
            TickType::IbDividends => write!(fmt, "IBDividends"),
            TickType::BondFactorMultiplier => write!(fmt, "bondFactorMultiplier"),
            TickType::RegulatoryImbalance => write!(fmt, "regulatoryImbalance"),
            TickType::NewsTick => write!(fmt, "newsTick"),
            TickType::ShortTermVolume3Min => write!(fmt, "shortTermVolume3Min"),
            TickType::ShortTermVolume5Min => write!(fmt, "shortTermVolume5Min"),
            TickType::ShortTermVolume10Min => write!(fmt, "shortTermVolume10Min"),
            TickType::DelayedBid => write!(fmt, "delayedBid"),
            TickType::DelayedAsk => write!(fmt, "delayedAsk"),
            TickType::DelayedLast => write!(fmt, "delayedLast"),
            TickType::DelayedBidSize => write!(fmt, "delayedBidSize"),
            TickType::DelayedAskSize => write!(fmt, "delayedAskSize"),
            TickType::DelayedLastSize => write!(fmt, "delayedLastSize"),
            TickType::DelayedHigh => write!(fmt, "delayedHigh"),
            TickType::DelayedLow => write!(fmt, "delayedLow"),
            TickType::DelayedVolume => write!(fmt, "delayedVolume"),
            TickType::DelayedClose => write!(fmt, "delayedClose"),
            TickType::DelayedOpen => write!(fmt, "delayedOpen"),
            TickType::RtTrdVolume => write!(fmt, "rtTrdVolume"),
            TickType::CreditmanMarkPrice => write!(fmt, "creditmanMarkPrice"),
            TickType::CreditmanSlowMarkPrice => write!(fmt, "creditmanSlowMarkPrice"),
            TickType::DelayedBidOption => write!(fmt, "delayedBidOptComp"),
            TickType::DelayedAskOption => write!(fmt, "delayedAskOptComp"),
            TickType::DelayedLastOption => write!(fmt, "delayedLastOptComp"),
            TickType::DelayedModelOption => write!(fmt, "delayedModelOptComp"),
            TickType::LastExch => write!(fmt, "lastExchange"),
            TickType::LastRegTime => write!(fmt, "lastRegTime"),
            TickType::FuturesOpenInterest => write!(fmt, "futuresOpenInterest"),
            TickType::AvgOptVolume => write!(fmt, "avgOptVolume"),
            TickType::DelayedLastTimestamp => write!(fmt, "delayedLastTimestamp"),
            TickType::ShortableShares => write!(fmt, "shortableShares"),
            TickType::NotSet => write!(fmt, "unknown"),
        }
    }
}

//==================================================================================================
/// Financial advisor data types
#[repr(i32)]
#[derive(Serialize, Deserialize, Clone, FromPrimitive, Debug)]
pub enum FaDataType {
    NA = 0,
    GROUPS = 1,
    PROFILES = 2,
    ALIASES = 3,
}

impl fmt::Display for FaDataType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Type: {}", self)
    }
}

//==================================================================================================
/// Tick by tick types
#[repr(i32)]
#[derive(Serialize, Deserialize, Clone, FromPrimitive, Debug)]
pub enum TickByTickType {
    NA = 0,
    Last = 1,
    AllLast = 2,
    BidAsk = 3,
    MidPoint = 4,
}

impl fmt::Display for TickByTickType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TickByTickType::NA => write!(f, "N/A"),
            TickByTickType::Last => write!(f, "Last"),
            TickByTickType::AllLast => write!(f, "AllLast"),
            TickByTickType::BidAsk => write!(f, "BidAsk"),
            TickByTickType::MidPoint => write!(f, "MidPoint"),
        }
    }
}

//==================================================================================================
/// date - the bar's date and time (either as a yyyymmss hh:mm:ssformatted
///        string or as system time according to the request)
/// open  - the bar's open point
/// high  - the bar's high point
/// low   - the bar's low point
/// close - the bar's closing point
/// volume - the bar's traded volume if available
/// count - the number of trades during the bar's timespan (only available
///         for TRADES).
/// bar_count - running count of the bars received for this request
/// average - average price of the bar
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct BarData {
    pub date: String,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: i64,
    pub bar_count: i32,
    pub average: f64,
}

impl BarData {
    pub fn new(
        date: String,
        open: f64,
        high: f64,
        low: f64,
        close: f64,
        volume: i64,
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
/// date_time - the bar's date and time (either as a yyyymmss hh:mm:ssformatted
///        string or as system time according to the request)
/// open  - the bar's open point
/// high  - the bar's high point
/// low   - the bar's low point
/// close - the bar's closing point
/// volume - the bar's traded volume if available
/// count - the number of trades during the bar's timespan (only available
///         for TRADES).
/// wap -   the bar's Weighted Average Price
/// count - running count of the bars for this request
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct RealTimeBar {
    pub date_time: String,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: i64,
    pub wap: f64,
    pub count: i32,
}

impl RealTimeBar {
    pub fn new(
        date_time: String,
        open: f64,
        high: f64,
        low: f64,
        close: f64,
        volume: i64,
        wap: f64,
        count: i32,
    ) -> Self {
        RealTimeBar {
            date_time,
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
        write!(
            f,
            "date_time: {},open: {}, high: {}, low: {}, close: {}, volume: {}, wap: {}, count: {}",
            self.date_time,
            self.open,
            self.high,
            self.low,
            self.close,
            self.volume,
            self.wap,
            self.count
        )
    }
}

//==================================================================================================
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct HistogramData {
    pub price: f64,
    pub count: i32,
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
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct DepthMktDataDescription {
    pub exchange: String,
    pub sec_type: String,
    pub listing_exch: String,
    pub service_data_type: String,
    pub agg_group: i32,
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
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct SmartComponent {
    pub bit_number: i32,
    pub exchange: String,
    pub exchange_letter: String,
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
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct TickAttrib {
    pub can_auto_execute: bool,
    pub past_limit: bool,
    pub pre_open: bool,
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
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct TickAttribBidAsk {
    pub bid_past_low: bool,
    pub ask_past_high: bool,
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
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct TickAttribLast {
    pub past_limit: bool,
    pub unreported: bool,
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
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct FamilyCode {
    pub account_id: String,
    pub family_code_str: String,
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
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct PriceIncrement {
    pub low_edge: f64,
    pub increment: f64,
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
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct HistoricalTick {
    pub time: i32,
    pub price: f64,
    pub size: i32,
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
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct HistoricalTickBidAsk {
    pub time: i32,
    pub tick_attrib_bid_ask: TickAttribBidAsk,
    pub price_bid: f64,
    pub price_ask: f64,
    pub size_bid: i32,
    pub size_ask: i32,
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
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct HistoricalTickLast {
    pub time: i32,
    pub tick_attrib_last: TickAttribLast,
    pub price: f64,
    pub size: i32,
    pub exchange: String,
    pub special_conditions: String,
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
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct CommissionReport {
    pub exec_id: String,
    pub commission: f64,
    pub currency: String,
    pub realized_pnl: f64,
    pub yield_: f64,
    pub yield_redemption_date: String, //YYYYMMDD format
}

impl CommissionReport {
    pub fn new(
        exec_id: String,
        commission: f64,
        currency: String,
        realized_pnl: f64,
        yield_: f64,
        yield_redemption_date: String,
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
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct NewsProvider {
    pub code: String,
    pub name: String,
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
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct TagValue {
    pub tag: String,
    pub value: String,
}

impl TagValue {
    pub fn new(tag: String, value: String) -> Self {
        TagValue { tag, value }
    }
}

impl fmt::Display for TagValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}={};", self.tag, self.value,)
    }
}

//==================================================================================================
#[repr(i32)]
#[derive(Serialize, Deserialize, Clone, FromPrimitive, Debug)]
pub enum ComboParam {
    NonGuaranteed,
    PriceCondConid,
    CondPriceMax,
    CondPriceMin,
    ChangeToMktTime1,
    ChangeToMktTime2,
    DiscretionaryPct,
    DontLeginNext,
    LeginPrio,
    MaxSegSize,
}

impl Display for ComboParam {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match *self {
            ComboParam::NonGuaranteed => write!(f, "NonGuaranteed"),
            ComboParam::PriceCondConid => write!(f, "PriceCondConid"),
            ComboParam::CondPriceMax => write!(f, "CondPriceMax"),
            ComboParam::CondPriceMin => write!(f, "CondPriceMin"),
            ComboParam::ChangeToMktTime1 => write!(f, "ChangeToMktTime1"),
            ComboParam::ChangeToMktTime2 => write!(f, "ChangeToMktTime2"),
            ComboParam::DiscretionaryPct => write!(f, "DiscretionaryPct"),
            ComboParam::DontLeginNext => write!(f, "DontLeginNext"),
            ComboParam::LeginPrio => write!(f, "LeginPrio"),
            ComboParam::MaxSegSize => write!(f, "MaxSegSize"),
        }
    }
}

//==================================================================================================
#[repr(i32)]
#[derive(Serialize, Deserialize, Clone, FromPrimitive, Debug)]
pub enum HedgeType {
    None,
    Delta,
    Beta,
    Fx,
    Pair,
}

impl Display for HedgeType {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match *self {
            HedgeType::None => write!(f, ""),
            HedgeType::Delta => write!(f, "Delta"),
            HedgeType::Beta => write!(f, "Beta"),
            HedgeType::Fx => write!(f, "Fx"),
            HedgeType::Pair => write!(f, "Pair"),
        }
    }
}

//==================================================================================================
#[repr(i32)]
#[derive(Serialize, Deserialize, Clone, FromPrimitive, Debug)]
pub enum Right {
    None,
    Put,
    Call,
}

impl Display for Right {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match *self {
            Right::None => write!(f, ""),
            Right::Put => write!(f, "P"),
            Right::Call => write!(f, "C"),
        }
    }
}

//==================================================================================================
#[repr(i32)]
#[derive(Serialize, Deserialize, Clone, FromPrimitive, Debug)]
pub enum VolatilityType {
    None,
    Daily,
    Annual,
}

impl Display for VolatilityType {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match *self {
            VolatilityType::None => write!(f, "None"),
            VolatilityType::Daily => write!(f, "Daily"),
            VolatilityType::Annual => write!(f, "Annual"),
        }
    }
}

//==================================================================================================
#[repr(i32)]
#[derive(Serialize, Deserialize, Clone, FromPrimitive, Debug)]
pub enum ReferencePriceType {
    None,
    Midpoint,
    BidOrAsk,
}

impl Display for ReferencePriceType {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match *self {
            ReferencePriceType::None => write!(f, "None"),
            ReferencePriceType::Midpoint => write!(f, "Midpoint"),
            ReferencePriceType::BidOrAsk => write!(f, "BidOrAsk"),
        }
    }
}

//==================================================================================================
#[repr(i32)]
#[derive(Serialize, Deserialize, Clone, FromPrimitive, Debug)]
pub enum TriggerMethod {
    Default,
    DoubleBidAsk,
    Last,
    DoubleLast,
    BidAsk,
    LastOrBidAsk,
    Midpoint,
}

impl Display for TriggerMethod {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match *self {
            TriggerMethod::Default => write!(f, "Default"),
            TriggerMethod::DoubleBidAsk => write!(f, "DoubleBidAsk"),
            TriggerMethod::Last => write!(f, "Last"),
            TriggerMethod::DoubleLast => write!(f, "DoubleLast"),
            TriggerMethod::BidAsk => write!(f, "BidAsk"),
            TriggerMethod::LastOrBidAsk => write!(f, "LastOrBidAsk"),
            TriggerMethod::Midpoint => write!(f, "Midpoint"),
        }
    }
}

//==================================================================================================
#[repr(i32)]
#[derive(Serialize, Deserialize, Clone, FromPrimitive, Debug)]
pub enum Action {
    BUY,
    SELL,
    SSHORT,
}

impl Display for Action {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match *self {
            Action::BUY => write!(f, "BUY"),
            Action::SELL => write!(f, "SELL"),
            Action::SSHORT => write!(f, "SSHORT"),
        }
    }
}

//==================================================================================================
#[repr(i32)]
#[derive(Serialize, Deserialize, Clone, FromPrimitive, Debug)]
pub enum Rule80A {
    None,
    Individual,
    Agency,
    AgentOtherMember,
    IndividualPTIA,
    AgencyPTIA,
    AgentOtherMemberPTIA,
    IndividualPT,
    AgencyPT,
    AgentOtherMemberPT,
}

impl Display for Rule80A {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            Rule80A::None => write!(f, ""),
            Rule80A::Individual => write!(f, "I"),
            Rule80A::Agency => write!(f, "A"),
            Rule80A::AgentOtherMember => write!(f, "W"),
            Rule80A::IndividualPTIA => write!(f, "J"),
            Rule80A::AgencyPTIA => write!(f, "U"),
            Rule80A::AgentOtherMemberPTIA => write!(f, "M"),
            Rule80A::IndividualPT => write!(f, "K"),
            Rule80A::AgencyPT => write!(f, "Y"),
            Rule80A::AgentOtherMemberPT => write!(f, "N"),
        }
    }
}

//==================================================================================================
#[repr(i32)]
#[derive(Serialize, Deserialize, Clone, FromPrimitive, Debug)]
pub enum OcaType {
    None,
    CancelWithBlocking,
    ReduceWithBlocking,
    ReduceWithoutBlocking,
}

impl Display for OcaType {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match *self {
            OcaType::None => write!(f, "None"),
            OcaType::CancelWithBlocking => write!(f, "CancelWithBlocking"),
            OcaType::ReduceWithBlocking => write!(f, "ReduceWithBlocking"),
            OcaType::ReduceWithoutBlocking => write!(f, "ReduceWithoutBlocking"),
        }
    }
}

//==================================================================================================
#[repr(i32)]
#[derive(Serialize, Deserialize, Clone, FromPrimitive, Debug)]
pub enum TimeInForce {
    DAY,
    GTC,
    OPG,
    IOC,
    GTD,
    GTT,
    AUC,
    FOK,
    GTX,
    DTC,
}

impl Display for TimeInForce {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match *self {
            TimeInForce::DAY => write!(f, "DAY"),
            TimeInForce::GTC => write!(f, "GTC"),
            TimeInForce::OPG => write!(f, "OPG"),
            TimeInForce::IOC => write!(f, "IOC"),
            TimeInForce::GTD => write!(f, "GTD"),
            TimeInForce::GTT => write!(f, "GTT"),
            TimeInForce::AUC => write!(f, "AUC"),
            TimeInForce::FOK => write!(f, "FOK"),
            TimeInForce::GTX => write!(f, "GTX"),
            TimeInForce::DTC => write!(f, "DTC"),
        }
    }
}

//==================================================================================================
#[repr(i32)]
#[derive(Serialize, Deserialize, Clone, FromPrimitive, Debug)]
pub enum ExerciseType {
    None,
    Exercise,
    Lapse,
}

impl Display for ExerciseType {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match *self {
            ExerciseType::None => write!(f, ""),
            ExerciseType::Exercise => write!(f, "Exercise"),
            ExerciseType::Lapse => write!(f, "Lapse"),
        }
    }
}

//==================================================================================================
#[repr(i32)]
#[derive(Serialize, Deserialize, Clone, FromPrimitive, Debug)]
pub enum FundamentalType {
    ReportSnapshot,
    ReportsFinSummary,
    ReportRatios,
    ReportsFinStatements,
    RESC,
    CalendarReport,
    ReportsOwnership,
}

impl Display for FundamentalType {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match *self {
            FundamentalType::ReportSnapshot => write!(f, "Company overview"),
            FundamentalType::ReportsFinSummary => write!(f, "Financial summary"),
            FundamentalType::ReportRatios => write!(f, "Financial ratios"),
            FundamentalType::ReportsFinStatements => write!(f, "Financial statements"),
            FundamentalType::RESC => write!(f, "Analyst estimates"),
            FundamentalType::CalendarReport => write!(f, "Company calendar"),
            FundamentalType::ReportsOwnership => write!(f, "Company ownership"),
        }
    }
}

//==================================================================================================
#[repr(i32)]
#[derive(Serialize, Deserialize, Clone, FromPrimitive, Debug)]
pub enum WhatToShow {
    Trades,
    Midpoint,
    Bid,
    Ask,
    // << only these are valid for real-time bars
    BidAsk,
    HistoricalVolatility,
    OptionImpliedVolatility,
    YieldAsk,
    YieldBid,
    YieldBidAsk,
    YieldLast,
    AdjustedLast,
}

impl Display for WhatToShow {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match *self {
            WhatToShow::Trades => write!(f, "TRADES"),
            WhatToShow::Midpoint => write!(f, "MIDPOINT"),
            WhatToShow::Bid => write!(f, "BID"),
            WhatToShow::Ask => write!(f, "ASK"),
            WhatToShow::BidAsk => write!(f, "BID_ASK"),
            WhatToShow::HistoricalVolatility => write!(f, "HISTORICAL_VOLATILITY"),
            WhatToShow::OptionImpliedVolatility => write!(f, "OPTION_IMPLIED_VOLATILITY"),
            WhatToShow::YieldAsk => write!(f, "YIELD_ASK"),
            WhatToShow::YieldBid => write!(f, "YIELD_ASK"),
            WhatToShow::YieldBidAsk => write!(f, "YIELD_BID_ASK"),
            WhatToShow::YieldLast => write!(f, "YIELD_LAST"),
            WhatToShow::AdjustedLast => write!(f, "ADJUSTED_LAST"),
        }
    }
}

//==================================================================================================
#[repr(i32)]
#[derive(Serialize, Deserialize, Clone, FromPrimitive, Debug)]
pub enum BarSize {
    _1Secs,
    _5Secs,
    _10Secs,
    _15Secs,
    _30Secs,
    _1Min,
    _2Mins,
    _3Mins,
    _5Mins,
    _10Mins,
    _15Mins,
    _20Mins,
    _30Mins,
    _1Hour,
    _4Hours,
    _1Day,
    _1Week,
    _1Month,
}

impl Display for BarSize {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match *self {
            BarSize::_1Secs => write!(f, "1 secs"),
            BarSize::_5Secs => write!(f, "5 secs"),
            BarSize::_10Secs => write!(f, "10 secs"),
            BarSize::_15Secs => write!(f, "15 secs"),
            BarSize::_30Secs => write!(f, "30 secs"),
            BarSize::_1Min => write!(f, "1 min"),
            BarSize::_2Mins => write!(f, "2 mins"),
            BarSize::_3Mins => write!(f, "3 mins"),
            BarSize::_5Mins => write!(f, "5 mins"),
            BarSize::_10Mins => write!(f, "10 mins"),
            BarSize::_15Mins => write!(f, "15 mins"),
            BarSize::_20Mins => write!(f, "20 mins"),
            BarSize::_30Mins => write!(f, "30 mins"),
            BarSize::_1Hour => write!(f, "1 hour"),
            BarSize::_4Hours => write!(f, "4 hours"),
            BarSize::_1Day => write!(f, "1 day"),
            BarSize::_1Week => write!(f, "1 week"),
            BarSize::_1Month => write!(f, "1 month"),
        }
    }
}

//==================================================================================================
#[repr(i32)]
#[derive(Serialize, Deserialize, Clone, FromPrimitive, Debug)]
pub enum DurationUnit {
    SECOND,
    DAY,
    WEEK,
    MONTH,
    YEAR,
}

impl Display for DurationUnit {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match *self {
            DurationUnit::SECOND => write!(f, "SECOND"),
            DurationUnit::DAY => write!(f, "DAY"),
            DurationUnit::WEEK => write!(f, "WEEK"),
            DurationUnit::MONTH => write!(f, "MONTH"),
            DurationUnit::YEAR => write!(f, "YEAR"),
        }
    }
}

//==================================================================================================
#[repr(i32)]
#[derive(Serialize, Deserialize, Clone, FromPrimitive, Debug)]
pub enum DeepType {
    INSERT,
    UPDATE,
    DELETE,
}

impl Display for DeepType {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match *self {
            DeepType::INSERT => write!(f, "INSERT"),
            DeepType::UPDATE => write!(f, "UPDATE"),
            DeepType::DELETE => write!(f, "DELETE"),
        }
    }
}

#[repr(i32)]
#[derive(Serialize, Deserialize, Clone, FromPrimitive, Debug)]
pub enum DeepSide {
    Sell,
    Buy,
}

impl Display for DeepSide {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match *self {
            DeepSide::Buy => write!(f, "BUY"),
            DeepSide::Sell => write!(f, "SELL"),
        }
    }
}

//==================================================================================================
#[repr(i32)]
#[derive(Serialize, Deserialize, Clone, FromPrimitive, Debug)]
pub enum NewsType {
    Unknown,
    BBS,
    LiveExch,
    DeadExch,
    HTML,
    PopupText,
    PopupHtml,
}

impl Display for NewsType {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match *self {
            NewsType::Unknown => write!(f, "UNKNOWN"),
            NewsType::BBS => write!(f, "BBS"),
            NewsType::LiveExch => write!(f, "LIVE_EXCH"),
            NewsType::DeadExch => write!(f, "DEAD_EXCH"),
            NewsType::HTML => write!(f, "HTML"),
            NewsType::PopupText => write!(f, "POPUP_TEXT"),
            NewsType::PopupHtml => write!(f, "POPUP_HTML"),
        }
    }
}

//==================================================================================================
#[repr(i32)]
#[derive(Serialize, Deserialize, Clone, FromPrimitive, Debug)]
pub enum SecIdType {
    None,
    CUSIP,
    SEDOL,
    ISIN,
    RIC,
}

impl Display for SecIdType {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match *self {
            SecIdType::None => write!(f, ""),
            SecIdType::CUSIP => write!(f, "CUSIP"),
            SecIdType::SEDOL => write!(f, "SEDOL"),
            SecIdType::ISIN => write!(f, "ISIN"),
            SecIdType::RIC => write!(f, "RIC"),
        }
    }
}

//==================================================================================================
#[repr(i32)]
#[derive(Serialize, Deserialize, Clone, FromPrimitive, Debug)]
pub enum SecType {
    None,
    STK,
    OPT,
    FUT,
    CONTFUT,
    CASH,
    BOND,
    CFD,
    FOP,
    WAR,
    IOPT,
    FWD,
    BAG,
    IND,
    BILL,
    FUND,
    FIXED,
    SLB,
    NEWS,
    CMDTY,
    BSK,
    ICU,
    ICS,
}

impl Display for SecType {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match *self {
            SecType::None => write!(f, ""),
            SecType::STK => write!(f, "STK"),
            SecType::OPT => write!(f, "OPT"),
            SecType::FUT => write!(f, "FUT"),
            SecType::CONTFUT => write!(f, "CONTFUT"),
            SecType::CASH => write!(f, "CASH"),
            SecType::BOND => write!(f, "BOND"),
            SecType::CFD => write!(f, "CFD"),
            SecType::FOP => write!(f, "FOP"),
            SecType::WAR => write!(f, "WAR"),
            SecType::IOPT => write!(f, "IOPT"),
            SecType::FWD => write!(f, "FWD"),
            SecType::BAG => write!(f, "BAG"),
            SecType::IND => write!(f, "IND"),
            SecType::BILL => write!(f, "BILL"),
            SecType::FUND => write!(f, "FUND"),
            SecType::FIXED => write!(f, "FIXED"),
            SecType::SLB => write!(f, "SLB"),
            SecType::NEWS => write!(f, "NEWS"),
            SecType::CMDTY => write!(f, "CMDTY"),
            SecType::BSK => write!(f, "BSK"),
            SecType::ICU => write!(f, "ICU"),
            SecType::ICS => write!(f, "ICS"),
        }
    }
}

//==================================================================================================
#[repr(i32)]
#[derive(Serialize, Deserialize, Clone, FromPrimitive, Debug)]
pub enum MarketDataTypeEnum {
    Unknown,
    Realtime,
    Frozen,
    Delayed,
    DelayedFrozen,
}

impl Display for MarketDataTypeEnum {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            MarketDataTypeEnum::Unknown => write!(f, "N/A"),
            MarketDataTypeEnum::Realtime => write!(f, "REALTIME"),
            MarketDataTypeEnum::Frozen => write!(f, "FROZEN"),
            MarketDataTypeEnum::Delayed => write!(f, "DELAYED"),
            MarketDataTypeEnum::DelayedFrozen => write!(f, "DELAYED_FROZEN"),
        }
    }
}

//==================================================================================================
#[repr(i32)]
#[derive(Serialize, Deserialize, Clone, FromPrimitive, Debug)]
pub enum Method {
    None,
    EqualQuantity,
    AvailableEquity,
    NetLiq,
    PctChange,
}

impl Display for Method {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            Method::None => write!(f, ""),
            Method::EqualQuantity => write!(f, "EqualQuantity"),
            Method::AvailableEquity => write!(f, "AvailableEquity"),
            Method::NetLiq => write!(f, "NetLiq"),
            Method::PctChange => write!(f, "PctChange"),
        }
    }
}

//==================================================================================================
#[repr(i32)]
#[derive(Serialize, Deserialize, Clone, FromPrimitive, Debug)]
pub enum UsePriceMgmtAlgo {
    Default,
    NotUse,
    Use,
}

impl UsePriceMgmtAlgo {
    pub fn value(&self) -> Option<bool> {
        match *self {
            UsePriceMgmtAlgo::Default => None,
            UsePriceMgmtAlgo::NotUse => Some(false),
            UsePriceMgmtAlgo::Use => Some(true),
        }
    }
}
