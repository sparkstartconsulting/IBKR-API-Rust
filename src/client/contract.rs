/*
    SamePos    = open/close leg value is same as combo
    OpenPos    = open
    ClosePos   = close
    UnknownPos = unknown
*/

use crate::client::common::TagValue;

pub enum PositionType {
    SamePos = 0,
    //open/close leg value is same as combo
    OpenPos = 1,
    //open
    ClosePos = 2,
    //close
    UnknownPos = 3, //unknown
}

pub struct ComboLeg {
    con_id: i32,
    // type: int
    ratio: i32,
    // type: int
    action: String,
    // BUY /SELL / SSHORT
    exchange: String,
    open_close: PositionType,
    // type: int; LegOpenClose enum values
    // for stock legs when doing short sale
    short_sale_slot: i32,
    designated_location: String,
    exempt_code: i32,
}

/*def __str__(self):
return ",".join((
str(con_id),
str(self.ratio),
str(self.action),
str(self.exchange),
str(self.open_close),
str(self.short_sale_slot),
str(self.designated_location),
str(self.exempt_code)))*/

pub struct DeltaNeutralContract {
    con_id: i32,
    // type: int
    delta: f64,
    // type: float
    price: f64, // type: float
}

/*def __str__(self):
return ",".join((
str(self.con_id),
str(self.delta),
str(self.price)))*/

pub struct Contract {
    con_id: i32,
    symbol: String,
    sec_type: String,
    last_trade_date_or_contract_month: String,
    strike: f64,
    // float ! !
    right: String,
    multiplier: String,
    exchange: String,
    primary_exchange: String,
    // pick an actual (ie non - aggregate) exchange that the contract trades on.DO NOT SET TO SMART.
    currency: String,
    local_symbol: String,
    trading_class: String,
    include_expired: bool,
    sec_id_type: String,
    // CUSIP; SEDOL; ISIN;RIC
    sec_id: String,

    //combos
    combo_legs_descrip: String,
    // type: str; received in open order 14 and up for all combos
    combo_legs: Vec<ComboLeg>,
    // type: list<ComboLeg>
    delta_neutral_contract: DeltaNeutralContract,
}

/*def __str__(self):
s = ",".join((
str(self.con_id),
str(self.symbol),
str(self.sec_type),
str(self.last_trade_date_or_contract_month),
str(self.strike),
str(self.right),
str(self.multiplier),
str(self.exchange),
str(self.primary_exchange),
str(self.currency),
str(self.local_symbol),
str(self.trading_class),
str(self.include_expired),
str(self.sec_id_type),
str(self.sec_id)))
s += "combo:" + self.combo_legs_descrip

if self.combo_legs:
for leg in self.combo_legs:
s += ";" + str(leg)

if self.delta_neutral_contract:
s += ";" + str(self.delta_neutral_contract)

return s
*/

pub struct ContractDetails {
    contract: Contract,
    market_name: String,
    min_tick: f64,
    order_types: String,
    valid_exchanges: String,
    price_magnifier: i32,
    under_con_id: i32,
    long_name: String,
    contract_month: String,
    industry: String,
    category: String,
    subcategory: String,
    time_zone_id: String,
    trading_hours: String,
    liquid_hours: String,
    ev_rule: String,
    ev_multiplier: i32,
    md_size_multiplier: i32,
    agg_group: i32,
    under_symbol: String,
    under_sec_type: String,
    market_rule_ids: String,
    sec_id_list: Vec<TagValue>,
    real_expiration_date: String,
    last_trade_time: String,

    // BOND values
    cusip: String,
    ratings: String,
    desc_append: String,
    bond_type: String,
    coupon_type: String,
    callable: bool,
    putable: bool,
    coupon: i32,
    convertible: bool,
    maturity: String,
    issue_date: String,
    next_option_date: String,
    next_option_type: String,
    next_option_partial: bool,
    notes: String,
}

/*
def __str__(self):
s = ",".join((
str(self.contract),
str(self.market_name),
str(self.min_tick),
str(self.order_types),
str(self.valid_exchanges),
str(self.price_magnifier),
str(self.under_con_id),
str(self.long_name),
str(self.contract_month),
str(self.industry),
str(self.category),
str(self.subcategory),
str(self.time_zone_id),
str(self.trading_hours),
str(self.liquid_hours),
str(self.ev_rule),
str(self.ev_multiplier),
str(self.md_size_multiplier),
str(self.under_symbol),
str(self.under_sec_type),
str(self.market_rule_ids),
str(self.agg_group),
str(self.sec_id_list),
str(self.real_expiration_date),
str(self.cusip),
str(self.ratings),
str(self.desc_append),
str(self.bond_type),
str(self.coupon_type),
str(self.callable),
str(self.putable),
str(self.coupon),
str(self.convertible),
str(self.maturity),
str(self.issue_date),
str(self.next_option_date),
str(self.next_option_type),
str(self.next_option_partial),
str(self.notes)))
return s
*/

pub struct ContractDescription {
    contract: Contract,
    derivative_sec_types: Vec<String>, // type: list of strings
}
