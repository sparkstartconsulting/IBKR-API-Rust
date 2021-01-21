//! Account summary tags
use std::fmt::{Display, Error, Formatter};



use crate::core::account_summary_tags::AccountSummaryTags::*;

//==================================================================================================
/// AccountType — Identifies the IB account structure
/// NetLiquidation — The basis for determining the price of the assets in your account. Total cash value + stock value + options value + bond value
/// TotalCashValue — Total cash balance recognized at the time of trade + futures PNL
/// SettledCash — Cash recognized at the time of settlement - purchases at the time of trade - commissions - taxes - fees
/// AccruedCash — Total accrued cash value of stock, commodities and securities
/// BuyingPower — Buying power serves as a measurement of the dollar value of securities that one may purchase in a securities account without depositing additional funds
/// EquityWithLoanValue — Forms the basis for determining whether a client has the necessary assets to either initiate or maintain security positions. Cash + stocks + bonds + mutual funds
/// PreviousEquityWithLoanValue — Marginable Equity with Loan value as of 16:00 ET the previous day
/// GrossPositionValue — The sum of the absolute value of all stock and equity option positions
/// RegTEquity — Regulation T equity for universal account
/// RegTMargin — Regulation T margin for universal account
/// SMA — Special Memorandum Account: Line of credit created when the market value of securities in a Regulation T account increase in value
/// InitMarginReq — Initial Margin requirement of whole portfolio
/// MaintMarginReq — Maintenance Margin requirement of whole portfolio
/// AvailableFunds — This value tells what you have available for trading
/// ExcessLiquidity — This value shows your margin cushion, before liquidation
/// Cushion — Excess liquidity as a percentage of net liquidation value
/// FullInitMarginReq — Initial Margin of whole portfolio with no discounts or intraday credits
/// FullMaintMarginReq — Maintenance Margin of whole portfolio with no discounts or intraday credits
/// FullAvailableFunds — Available funds of whole portfolio with no discounts or intraday credits
/// FullExcessLiquidity — Excess liquidity of whole portfolio with no discounts or intraday credits
/// LookAheadNextChange — Time when look-ahead values take effect
/// LookAheadInitMarginReq — Initial Margin requirement of whole portfolio as of next period's margin change
/// LookAheadMaintMarginReq — Maintenance Margin requirement of whole portfolio as of next period's margin change
/// LookAheadAvailableFunds — This value reflects your available funds at the next margin change
/// LookAheadExcessLiquidity — This value reflects your excess liquidity at the next margin change
/// HighestSeverity — A measure of how close the account is to liquidation
/// DayTradesRemaining — The Number of Open/Close trades a user could put on before Pattern Day Trading is detected. A value of "-1" means that the user can put on unlimited day trades.
/// Leverage — GrossPositionValue / NetLiquidation
/// $LEDGER — Single flag to relay all cash balance tags*, only in base currency.
/// $LEDGER:CURRENCY — Single flag to relay all cash balance tags*, only in the specified currency.
/// $LEDGER:ALL — Single flag to relay all cash balance tags* in all currencies.

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
    Ledger,
    LedgerCurrency,
    LedgerAll,
    AllTags,
}

impl AccountSummaryTags {
    fn display(&self) -> &str {
        match self {
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
            Ledger => "$LEDGER",
            LedgerCurrency => "$LEDGER:CURRENCY",
            LedgerAll => "$LEDGER:ALL",
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
            Leverage,
            $LEDGER,
            $LEDGER:CURRENCY,
            $LEDGER:ALL"
            }
        }
    }
}

impl Display for AccountSummaryTags {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}", self.display())
    }
}
