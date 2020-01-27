use std::fmt::Display;

use serde::export::fmt::Error;
use serde::export::Formatter;

use crate::core::account_summary_tags::AccountSummaryTags::*;

//==================================================================================================
pub enum AccountSummaryTags {
    AccountType,
    NetLiquidation,
    TotalCashValue,
    SettledCash,
    AccruedCash,
    BuyingPower,
    EquityWithLoanValue,
    PreviousEquityWithLoanValue,
    GrossPositionValue,
    ReqTEquity,
    ReqTMargin,
    SMA,
    InitMarginReq,
    MaintMarginReq,
    AvailableFunds,
    ExcessLiquidity,
    Cushion,
    FullInitMarginReq,
    FullMaintMarginReq,
    FullAvailableFunds,
    FullExcessLiquidity,
    LookAheadNextChange,
    LookAheadInitMarginReq,
    LookAheadMaintMarginReq,
    LookAheadAvailableFunds,
    LookAheadExcessLiquidity,
    HighestSeverity,
    DayTradesRemaining,
    Leverage,
    AllTags,
}

impl AccountSummaryTags {
    fn display(&self) -> &str {
        match (self) {
            AccountType => "AccountType",
            NetLiquidation => "NetLiquidation",
            TotalCashValue => "TotalCashValue",
            SettledCash => "SettledCash",
            AccruedCash => "AccruedCash",
            BuyingPower => "BuyingPower",
            EquityWithLoanValue => "EquityWithLoanValue",
            PreviousEquityWithLoanValue => "PreviousEquityWithLoanValue",
            GrossPositionValue => "GrossPositionValue",
            ReqTEquity => "ReqTEquity",
            ReqTMargin => "ReqTMargin",
            SMA => "SMA",
            InitMarginReq => "InitMarginReq",
            MaintMarginReq => "MaintMarginReq",
            AvailableFunds => "AvailableFunds",
            ExcessLiquidity => "ExcessLiquidity",
            Cushion => "Cushion",
            FullInitMarginReq => "FullInitMarginReq",
            FullMaintMarginReq => "FullMaintMarginReq",
            FullAvailableFunds => "FullAvailableFunds",
            FullExcessLiquidity => "FullExcessLiquidity",
            LookAheadNextChange => "LookAheadNextChange",
            LookAheadInitMarginReq => "LookAheadInitMarginReq",
            LookAheadMaintMarginReq => "LookAheadMaintMarginReq",
            LookAheadAvailableFunds => "LookAheadAvailableFunds",
            LookAheadExcessLiquidity => "LookAheadExcessLiquidity",
            HighestSeverity => "HighestSeverity",
            DayTradesRemaining => "DayTradesRemaining",
            Leverage => "Leverage",
            AllTags => {
                "AccountType,
            NetLiquidation,
            TotalCashValue,
            SettledCash,
            AccruedCash,
            BuyingPower,
            EquityWithLoanValue,
            PreviousEquityWithLoanValue,
            GrossPositionValue,
            ReqTEquity,
            ReqTMargin,
            SMA,
            InitMarginReq,
            MaintMarginReq,
            AvailableFunds,
            ExcessLiquidity,
            Cushion,Cushion,
            FullInitMarginReq,
            FullMaintMarginReq,
            FullAvailableFunds,
            FullExcessLiquidity,
            LookAheadNextChange,
            LookAheadInitMarginReq,
            LookAheadMaintMarginReq,
            LookAheadAvailableFunds,
            LookAheadExcessLiquidity,
            HighestSeverity,
            DayTradesRemaining,
            Leverage"
            }
        }
    }
}

impl Display for AccountSummaryTags {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}", self.display())
    }
}
