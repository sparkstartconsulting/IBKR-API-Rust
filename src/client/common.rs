pub const NO_VALID_ID: i32 = -1;
pub const MAX_MSG_LEN: i32 = 0xFFFFFF; //16Mb - 1byte

pub const UNSET_INTEGER: i32 = std::i32::MAX;
pub const UNSET_DOUBLE: f32 = std::f32::MAX;
pub const UNSET_LONG: i64 = std::i64::MAX;

pub struct TickAttrib {
    can_auto_execute: bool,
    past_limit: bool,
    pre_open: bool,
}

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

impl OrderState {}

pub struct Order {}

pub struct Contract {}
pub struct ContractDetails {}
pub struct Execution {}
pub struct FaDataType {}
pub struct BarData {}
pub struct DeltaNeutralContract {}
pub struct CommissionReport {}
pub struct TickAttribBidAsk {}
pub struct TickAttribLast {}
pub struct HistogramData {}
pub struct HistoricalTick {}
pub struct HistoricalTickBidAsk {}
pub struct NewsProvider {}
pub struct SimpleEntry {}
pub struct FamilyCode {}
pub struct ContractDescription {}
pub struct DepthExchanges {}
pub struct PriceIncrement {}
pub struct SoftDollarTier {}
