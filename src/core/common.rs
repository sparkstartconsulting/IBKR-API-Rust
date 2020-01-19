use std::fmt;

use num_derive::FromPrimitive;
// 0.2.4 (the derive)
use num_traits::FromPrimitive;
use serde::{Deserialize, Serialize};
// 0.2.6 (the trait)
use serde::export::fmt::Error;
use serde::export::Formatter;

pub const NO_VALID_ID: i32 = -1;
pub const MAX_MSG_LEN: i64 = 0xFFFFFF; //16Mb - 1byte

pub const UNSET_INTEGER: i32 = std::i32::MAX;
pub const UNSET_DOUBLE: f64 = 1.7976931348623157E308_f64;
//std::f64::MAX;
pub const UNSET_LONG: i64 = std::i64::MAX;

const BID_SIZE: (i32, &str) = (0, "BidSize");
const BID: (i32, &str) = (1, "BID");
const ASK: (i32, &str) = (2, "ASK");
const ASK_SIZE: (i32, &str) = (3, "AskSize");
const LAST: (i32, &str) = (4, "LAST");
const LAST_SIZE: (i32, &str) = (5, "LAST_SIZE");
const HIGH: (i32, &str) = (6, "HIGH");
const LOW: (i32, &str) = (7, "LOW");
const VOLUME: (i32, &str) = (8, "VOLUME");
const CLOSE: (i32, &str) = (9, "CLOSE");
const BID_OPTION_COMPUTATION: (i32, &str) = (10, "BID_OPTION_COMPUTATION");
const ASK_OPTION_COMPUTATION: (i32, &str) = (11, "ASK_OPTION_COMPUTATION");
const LAST_OPTION_COMPUTATION: (i32, &str) = (12, "LAST_OPTION_COMPUTATION");
const MODEL_OPTION: (i32, &str) = (13, "MODEL_OPTION");
const OPEN: (i32, &str) = (14, "OPEN");
const LOW_13_WEEK: (i32, &str) = (15, "LOW_13_WEEK");
const HIGH_13_WEEK: (i32, &str) = (16, "HIGH_13_WEEK");
const LOW_26_WEEK: (i32, &str) = (17, "LOW_26_WEEK");
const HIGH_26_WEEK: (i32, &str) = (18, "HIGH_26_WEEK");
const LOW_52_WEEK: (i32, &str) = (19, "LOW_52_WEEK");
const HIGH_52_WEEK: (i32, &str) = (20, "HIGH_52_WEEK");
const AVG_VOLUME: (i32, &str) = (21, "AVG_VOLUME");
const OPEN_INTEREST: (i32, &str) = (22, "OPEN_INTEREST");
const OPTION_HISTORICAL_VOL: (i32, &str) = (23, "OPTION_HISTORICAL_VOL");
const OPTION_IMPLIED_VOL: (i32, &str) = (24, "OPTION_IMPLIED_VOL");
const OPTION_BID_EXCH: (i32, &str) = (25, "OPTION_BID_EXCH");
const OPTION_ASK_EXCH: (i32, &str) = (26, "OPTION_ASK_EXCH");
const OPTION_CALL_OPEN_INTEREST: (i32, &str) = (27, "OPTION_CALL_OPEN_INTEREST");
const OPTION_PUT_OPEN_INTEREST: (i32, &str) = (28, "OPTION_PUT_OPEN_INTEREST");
const OPTION_CALL_VOLUME: (i32, &str) = (29, "OPTION_CALL_VOLUME");
const OPTION_PUT_VOLUME: (i32, &str) = (30, "OPTION_PUT_VOLUME");
const INDEX_FUTURE_PREMIUM: (i32, &str) = (31, "INDEX_FUTURE_PREMIUM");
const BID_EXCH: (i32, &str) = (32, "BID_EXCH");
const ASK_EXCH: (i32, &str) = (33, "ASK_EXCH");
const AUCTION_VOLUME: (i32, &str) = (34, "AUCTION_VOLUME");
const AUCTION_PRICE: (i32, &str) = (35, "AUCTION_PRICE");
const AUCTION_IMBALANCE: (i32, &str) = (36, "AUCTION_IMBALANCE");
const MARK_PRICE: (i32, &str) = (37, "MARK_PRICE");
const BID_EFP_COMPUTATION: (i32, &str) = (38, "BID_EFP_COMPUTATION");
const ASK_EFP_COMPUTATION: (i32, &str) = (39, "ASK_EFP_COMPUTATION");
const LAST_EFP_COMPUTATION: (i32, &str) = (40, "LAST_EFP_COMPUTATION");
const OPEN_EFP_COMPUTATION: (i32, &str) = (41, "OPEN_EFP_COMPUTATION");
const HIGH_EFP_COMPUTATION: (i32, &str) = (42, "HIGH_EFP_COMPUTATION");
const LOW_EFP_COMPUTATION: (i32, &str) = (43, "LOW_EFP_COMPUTATION");
const CLOSE_EFP_COMPUTATION: (i32, &str) = (44, "CLOSE_EFP_COMPUTATION");
const LAST_TIMESTAMP: (i32, &str) = (45, "LAST_TIMESTAMP");
const SHORTABLE: (i32, &str) = (46, "SHORTABLE");
const FUNDAMENTAL_RATIOS: (i32, &str) = (47, "FUNDAMENTAL_RATIOS");
const RT_VOLUME: (i32, &str) = (48, "RT_VOLUME");
const HALTED: (i32, &str) = (49, "HALTED");
const BID_YIELD: (i32, &str) = (50, "BID_YIELD");
const ASK_YIELD: (i32, &str) = (51, "ASK_YIELD");
const LAST_YIELD: (i32, &str) = (52, "LAST_YIELD");
const CUST_OPTION_COMPUTATION: (i32, &str) = (53, "CUST_OPTION_COMPUTATION");
const TRADE_COUNT: (i32, &str) = (54, "TRADE_COUNT");
const TRADE_RATE: (i32, &str) = (55, "TRADE_RATE");
const VOLUME_RATE: (i32, &str) = (56, "VOLUME_RATE");
const LAST_RTH_TRADE: (i32, &str) = (57, "LAST_RTH_TRADE");
const RT_HISTORICAL_VOL: (i32, &str) = (58, "RT_HISTORICAL_VOL");
const IB_DIVIDENDS: (i32, &str) = (59, "IB_DIVIDENDS");
const BOND_FACTOR_MULTIPLIER: (i32, &str) = (60, "BOND_FACTOR_MULTIPLIER");
const REGULATORY_IMBALANCE: (i32, &str) = (62, "REGULATORY_IMBALANCE");
const NEWS_TICK: (i32, &str) = (63, "NEWS_TICK");
const SHORT_TERM_VOLUME_3_MIN: (i32, &str) = (64, "SHORT_TERM_VOLUME_3_MIN");
const SHORT_TERM_VOLUME_5_MIN: (i32, &str) = (65, "SHORT_TERM_VOLUME_5_MIN");
const SHORT_TERM_VOLUME_10_MIN: (i32, &str) = (66, "SHORT_TERM_VOLUME_10_MIN");
const DELAYED_BID: (i32, &str) = (67, "DELAYED_BID");
const DELAYED_ASK: (i32, &str) = (68, "DELAYED_ASK");
const DELAYED_LAST: (i32, &str) = (69, "DELAYED_LAST");
const DELAYED_BID_SIZE: (i32, &str) = (70, "DELAYED_BID_SIZE");
const DELAYED_ASK_SIZE: (i32, &str) = (71, "DELAYED_ASK_SIZE");
const DELAYED_LAST_SIZE: (i32, &str) = (72, "DELAYED_LAST_SIZE");
const DELAYED_HIGH: (i32, &str) = (73, "DELAYED_HIGH");
const DELAYED_LOW: (i32, &str) = (74, "DELAYED_LOW");
const DELAYED_VOLUME: (i32, &str) = (75, "DELAYED_VOLUME");
const DELAYED_CLOSE: (i32, &str) = (76, "DELAYED_CLOSE");
const DELAYED_OPEN: (i32, &str) = (77, "DELAYED_OPEN");
const RT_TRD_VOLUME: (i32, &str) = (78, "RT_TRD_VOLUME");
const CREDITMAN_MARK_PRICE: (i32, &str) = (79, "");
const CREDITMAN_SLOW_MARK_PRICE: (i32, &str) = (80, "CREDITMAN_MARK_PRICE");
const DELAYED_BID_OPTION: (i32, &str) = (81, "DELAYED_BID_OPTION");
const DELAYED_ASK_OPTION: (i32, &str) = (82, "DELAYED_ASK_OPTION");
const DELAYED_LAST_OPTION: (i32, &str) = (83, "DELAYED_LAST_OPTION");
const DELAYED_MODEL_OPTION: (i32, &str) = (84, "DELAYED_MODEL_OPTION");
const LAST_EXCH: (i32, &str) = (85, "LAST_EXCH");
const LAST_REG_TIME: (i32, &str) = (86, "LAST_REG_TIME");
const FUTURES_OPEN_INTEREST: (i32, &str) = (87, "FUTURES_OPEN_INTEREST");
const AVG_OPT_VOLUME: (i32, &str) = (89, "AVG_OPT_VOLUME");
const DELAYED_LAST_TIMESTAMP: (i32, &str) = (90, "DELAYED_LAST_TIMESTAMP");
const SHORTABLE_SHARES: (i32, &str) = (91, "SHORTABLE_SHARES");
const NOT_SET: (i32, &str) = (92, "NOT_SET");

#[repr(i32)]
#[derive(Serialize, Deserialize, Clone, Debug, FromPrimitive, Copy)]
pub enum TickType {
    BidSize,
    Bid,
    Ask,
    AskSize,
    Last,
    LastSize,
    High,
    Low,
    Volume,
    Close,
    BidOptionComputation,
    AskOptionComputation,
    LastOptionComputation,
    ModelOption,
    Open,
    Low13Week,
    High13Week,
    Low26Week,
    High26Week,
    Low52Week,
    High52Week,
    AvgVolume,
    OpenInterest,
    OptionHistoricalVol,
    OptionImpliedVol,
    OptionBidExch,
    OptionAskExch,
    OptionCallOpenInterest,
    OptionPutOpenInterest,
    OptionCallVolume,
    OptionPutVolume,
    IndexFuturePremium,
    BidExch,
    AskExch,
    AuctionVolume,
    AuctionPrice,
    AuctionImbalance,
    MarkPrice,
    BidEfpComputation,
    AskEfpComputation,
    LastEfpComputation,
    OpenEfpComputation,
    HighEfpComputation,
    LowEfpComputation,
    CloseEfpComputation,
    LastTimestamp,
    Shortable,
    FundamentalRatios,
    RtVolume,
    Halted,
    BidYield,
    AskYield,
    LastYield,
    CustOptionComputation,
    TradeCount,
    TradeRate,
    VolumeRate,
    LastRthTrade,
    RtHistoricalVol,
    IbDividends,
    BondFactorMultiplier,
    RegulatoryImbalance,
    NewsTick,
    ShortTermVolume3Min,
    ShortTermVolume5Min,
    ShortTermVolume10Min,
    DelayedBid,
    DelayedAsk,
    DelayedLast,
    DelayedBidSize,
    DelayedAskSize,
    DelayedLastSize,
    DelayedHigh,
    DelayedLow,
    DelayedVolume,
    DelayedClose,
    DelayedOpen,
    RtTrdVolume,
    CreditmanMarkPrice,
    CreditmanSlowMarkPrice,
    DelayedBidOption,
    DelayedAskOption,
    DelayedLastOption,
    DelayedModelOption,
    LastExch,
    LastRegTime,
    FuturesOpenInterest,
    AvgOptVolume,
    DelayedLastTimestamp,
    ShortableShares,
    NotSet,
}

impl TickType {
    pub fn code(&self) -> i32 {
        match *self {
            TickType::BidSize => BID_SIZE.0,
            TickType::Bid => BID.0,
            TickType::Ask => ASK.0,
            TickType::AskSize => ASK_SIZE.0,
            TickType::Last => LAST.0,
            TickType::LastSize => LAST_SIZE.0,
            TickType::High => HIGH.0,
            TickType::Low => LOW.0,
            TickType::Volume => VOLUME.0,
            TickType::Close => CLOSE.0,
            TickType::BidOptionComputation => BID_OPTION_COMPUTATION.0,
            TickType::AskOptionComputation => ASK_OPTION_COMPUTATION.0,
            TickType::LastOptionComputation => LAST_OPTION_COMPUTATION.0,
            TickType::ModelOption => MODEL_OPTION.0,
            TickType::Open => OPEN.0,
            TickType::Low13Week => LOW_13_WEEK.0,
            TickType::High13Week => HIGH_13_WEEK.0,
            TickType::Low26Week => LOW_26_WEEK.0,
            TickType::High26Week => HIGH_26_WEEK.0,
            TickType::Low52Week => LOW_52_WEEK.0,
            TickType::High52Week => HIGH_52_WEEK.0,
            TickType::AvgVolume => AVG_VOLUME.0,
            TickType::OpenInterest => OPEN_INTEREST.0,
            TickType::OptionHistoricalVol => OPTION_HISTORICAL_VOL.0,
            TickType::OptionImpliedVol => OPTION_IMPLIED_VOL.0,
            TickType::OptionBidExch => OPTION_BID_EXCH.0,
            TickType::OptionAskExch => OPTION_ASK_EXCH.0,
            TickType::OptionCallOpenInterest => OPTION_CALL_OPEN_INTEREST.0,
            TickType::OptionPutOpenInterest => OPTION_PUT_OPEN_INTEREST.0,
            TickType::OptionCallVolume => OPTION_CALL_VOLUME.0,
            TickType::OptionPutVolume => OPTION_PUT_VOLUME.0,
            TickType::IndexFuturePremium => INDEX_FUTURE_PREMIUM.0,
            TickType::BidExch => BID_EXCH.0,
            TickType::AskExch => ASK_EXCH.0,
            TickType::AuctionVolume => AUCTION_VOLUME.0,
            TickType::AuctionPrice => AUCTION_PRICE.0,
            TickType::AuctionImbalance => AUCTION_IMBALANCE.0,
            TickType::MarkPrice => MARK_PRICE.0,
            TickType::BidEfpComputation => BID_EFP_COMPUTATION.0,
            TickType::AskEfpComputation => ASK_EFP_COMPUTATION.0,
            TickType::LastEfpComputation => LAST_EFP_COMPUTATION.0,
            TickType::OpenEfpComputation => OPEN_EFP_COMPUTATION.0,
            TickType::HighEfpComputation => HIGH_EFP_COMPUTATION.0,
            TickType::LowEfpComputation => LOW_EFP_COMPUTATION.0,
            TickType::CloseEfpComputation => CLOSE_EFP_COMPUTATION.0,
            TickType::LastTimestamp => LAST_TIMESTAMP.0,
            TickType::Shortable => SHORTABLE.0,
            TickType::FundamentalRatios => FUNDAMENTAL_RATIOS.0,
            TickType::RtVolume => RT_VOLUME.0,
            TickType::Halted => HALTED.0,
            TickType::BidYield => BID_YIELD.0,
            TickType::AskYield => ASK_YIELD.0,
            TickType::LastYield => LAST_YIELD.0,
            TickType::CustOptionComputation => CUST_OPTION_COMPUTATION.0,
            TickType::TradeCount => TRADE_COUNT.0,
            TickType::TradeRate => TRADE_RATE.0,
            TickType::VolumeRate => VOLUME_RATE.0,
            TickType::LastRthTrade => LAST_RTH_TRADE.0,
            TickType::RtHistoricalVol => RT_HISTORICAL_VOL.0,
            TickType::IbDividends => IB_DIVIDENDS.0,
            TickType::BondFactorMultiplier => BOND_FACTOR_MULTIPLIER.0,
            TickType::RegulatoryImbalance => REGULATORY_IMBALANCE.0,
            TickType::NewsTick => NEWS_TICK.0,
            TickType::ShortTermVolume3Min => SHORT_TERM_VOLUME_3_MIN.0,
            TickType::ShortTermVolume5Min => SHORT_TERM_VOLUME_5_MIN.0,
            TickType::ShortTermVolume10Min => SHORT_TERM_VOLUME_10_MIN.0,
            TickType::DelayedBid => DELAYED_BID.0,
            TickType::DelayedAsk => DELAYED_ASK.0,
            TickType::DelayedLast => DELAYED_LAST.0,
            TickType::DelayedBidSize => DELAYED_BID_SIZE.0,
            TickType::DelayedAskSize => DELAYED_ASK_SIZE.0,
            TickType::DelayedLastSize => DELAYED_LAST_SIZE.0,
            TickType::DelayedHigh => DELAYED_HIGH.0,
            TickType::DelayedLow => DELAYED_LOW.0,
            TickType::DelayedVolume => DELAYED_VOLUME.0,
            TickType::DelayedClose => DELAYED_CLOSE.0,
            TickType::DelayedOpen => DELAYED_OPEN.0,
            TickType::RtTrdVolume => RT_TRD_VOLUME.0,
            TickType::CreditmanMarkPrice => CREDITMAN_MARK_PRICE.0,
            TickType::CreditmanSlowMarkPrice => CREDITMAN_SLOW_MARK_PRICE.0,
            TickType::DelayedBidOption => DELAYED_BID_OPTION.0,
            TickType::DelayedAskOption => DELAYED_ASK_OPTION.0,
            TickType::DelayedLastOption => DELAYED_LAST_OPTION.0,
            TickType::DelayedModelOption => DELAYED_MODEL_OPTION.0,
            TickType::LastExch => LAST_EXCH.0,
            TickType::LastRegTime => LAST_REG_TIME.0,
            TickType::FuturesOpenInterest => FUTURES_OPEN_INTEREST.0,
            TickType::AvgOptVolume => AVG_OPT_VOLUME.0,
            TickType::DelayedLastTimestamp => DELAYED_LAST_TIMESTAMP.0,
            TickType::ShortableShares => SHORTABLE_SHARES.0,
            TickType::NotSet => NOT_SET.0,
        }
    }

    pub fn value(&self) -> &str {
        match *self {
            TickType::BidSize => BID_SIZE.1,
            TickType::Bid => BID.1,
            TickType::Ask => ASK.1,
            TickType::AskSize => ASK_SIZE.1,
            TickType::Last => LAST.1,
            TickType::LastSize => LAST_SIZE.1,
            TickType::High => HIGH.1,
            TickType::Low => LOW.1,
            TickType::Volume => VOLUME.1,
            TickType::Close => CLOSE.1,
            TickType::BidOptionComputation => BID_OPTION_COMPUTATION.1,
            TickType::AskOptionComputation => ASK_OPTION_COMPUTATION.1,
            TickType::LastOptionComputation => LAST_OPTION_COMPUTATION.1,
            TickType::ModelOption => MODEL_OPTION.1,
            TickType::Open => OPEN.1,
            TickType::Low13Week => LOW_13_WEEK.1,
            TickType::High13Week => HIGH_13_WEEK.1,
            TickType::Low26Week => LOW_26_WEEK.1,
            TickType::High26Week => HIGH_26_WEEK.1,
            TickType::Low52Week => LOW_52_WEEK.1,
            TickType::High52Week => HIGH_52_WEEK.1,
            TickType::AvgVolume => AVG_VOLUME.1,
            TickType::OpenInterest => OPEN_INTEREST.1,
            TickType::OptionHistoricalVol => OPTION_HISTORICAL_VOL.1,
            TickType::OptionImpliedVol => OPTION_IMPLIED_VOL.1,
            TickType::OptionBidExch => OPTION_BID_EXCH.1,
            TickType::OptionAskExch => OPTION_ASK_EXCH.1,
            TickType::OptionCallOpenInterest => OPTION_CALL_OPEN_INTEREST.1,
            TickType::OptionPutOpenInterest => OPTION_PUT_OPEN_INTEREST.1,
            TickType::OptionCallVolume => OPTION_CALL_VOLUME.1,
            TickType::OptionPutVolume => OPTION_PUT_VOLUME.1,
            TickType::IndexFuturePremium => INDEX_FUTURE_PREMIUM.1,
            TickType::BidExch => BID_EXCH.1,
            TickType::AskExch => ASK_EXCH.1,
            TickType::AuctionVolume => AUCTION_VOLUME.1,
            TickType::AuctionPrice => AUCTION_PRICE.1,
            TickType::AuctionImbalance => AUCTION_IMBALANCE.1,
            TickType::MarkPrice => MARK_PRICE.1,
            TickType::BidEfpComputation => BID_EFP_COMPUTATION.1,
            TickType::AskEfpComputation => ASK_EFP_COMPUTATION.1,
            TickType::LastEfpComputation => LAST_EFP_COMPUTATION.1,
            TickType::OpenEfpComputation => OPEN_EFP_COMPUTATION.1,
            TickType::HighEfpComputation => HIGH_EFP_COMPUTATION.1,
            TickType::LowEfpComputation => LOW_EFP_COMPUTATION.1,
            TickType::CloseEfpComputation => CLOSE_EFP_COMPUTATION.1,
            TickType::LastTimestamp => LAST_TIMESTAMP.1,
            TickType::Shortable => SHORTABLE.1,
            TickType::FundamentalRatios => FUNDAMENTAL_RATIOS.1,
            TickType::RtVolume => RT_VOLUME.1,
            TickType::Halted => HALTED.1,
            TickType::BidYield => BID_YIELD.1,
            TickType::AskYield => ASK_YIELD.1,
            TickType::LastYield => LAST_YIELD.1,
            TickType::CustOptionComputation => CUST_OPTION_COMPUTATION.1,
            TickType::TradeCount => TRADE_COUNT.1,
            TickType::TradeRate => TRADE_RATE.1,
            TickType::VolumeRate => VOLUME_RATE.1,
            TickType::LastRthTrade => LAST_RTH_TRADE.1,
            TickType::RtHistoricalVol => RT_HISTORICAL_VOL.1,
            TickType::IbDividends => IB_DIVIDENDS.1,
            TickType::BondFactorMultiplier => BOND_FACTOR_MULTIPLIER.1,
            TickType::RegulatoryImbalance => REGULATORY_IMBALANCE.1,
            TickType::NewsTick => NEWS_TICK.1,
            TickType::ShortTermVolume3Min => SHORT_TERM_VOLUME_3_MIN.1,
            TickType::ShortTermVolume5Min => SHORT_TERM_VOLUME_5_MIN.1,
            TickType::ShortTermVolume10Min => SHORT_TERM_VOLUME_10_MIN.1,
            TickType::DelayedBid => DELAYED_BID.1,
            TickType::DelayedAsk => DELAYED_ASK.1,
            TickType::DelayedLast => DELAYED_LAST.1,
            TickType::DelayedBidSize => DELAYED_BID_SIZE.1,
            TickType::DelayedAskSize => DELAYED_ASK_SIZE.1,
            TickType::DelayedLastSize => DELAYED_LAST_SIZE.1,
            TickType::DelayedHigh => DELAYED_HIGH.1,
            TickType::DelayedLow => DELAYED_LOW.1,
            TickType::DelayedVolume => DELAYED_VOLUME.1,
            TickType::DelayedClose => DELAYED_CLOSE.1,
            TickType::DelayedOpen => DELAYED_OPEN.1,
            TickType::RtTrdVolume => RT_TRD_VOLUME.1,
            TickType::CreditmanMarkPrice => CREDITMAN_MARK_PRICE.1,
            TickType::CreditmanSlowMarkPrice => CREDITMAN_SLOW_MARK_PRICE.1,
            TickType::DelayedBidOption => DELAYED_BID_OPTION.1,
            TickType::DelayedAskOption => DELAYED_ASK_OPTION.1,
            TickType::DelayedLastOption => DELAYED_LAST_OPTION.1,
            TickType::DelayedModelOption => DELAYED_MODEL_OPTION.1,
            TickType::LastExch => LAST_EXCH.1,
            TickType::LastRegTime => LAST_REG_TIME.1,
            TickType::FuturesOpenInterest => FUTURES_OPEN_INTEREST.1,
            TickType::AvgOptVolume => AVG_OPT_VOLUME.1,
            TickType::DelayedLastTimestamp => DELAYED_LAST_TIMESTAMP.1,
            TickType::ShortableShares => SHORTABLE_SHARES.1,
            TickType::NotSet => NOT_SET.1,
        }
    }
}

impl fmt::Display for TickType {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.value())
    }
}

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
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct RealTimeBar {
    pub time: String,
    pub end_time: String,
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
        time: String,
        end_time: String,
        open: f64,
        high: f64,
        low: f64,
        close: f64,
        volume: i64,
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
    pub yield_redemption_date: i32, //YYYYMMDD format
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
/*
def __str__(self):
# this is not only used for Python dump but when encoding to send
# so don't change it lightly !
return "%s=%s;" % (self.tag, self.value)
*/
