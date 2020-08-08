//! Types related to Contracts
use num_derive::FromPrimitive;
use serde::{Deserialize, Serialize};
// 0.2.6 (the trait)
use serde::export::fmt::{Display, Error};
use serde::export::Formatter;

use crate::core::common::TagValue;

//==================================================================================================
#[repr(i32)]
#[derive(Serialize, Deserialize, Clone, FromPrimitive, Debug)]
pub enum PositionType {
    SamePos = 0,
    //open/close leg value is same as combo
    OpenPos = 1,
    //open
    ClosePos = 2,
    //close
    UnknownPos = 3, //unknown
}

impl Display for PositionType {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match *self {
            PositionType::SamePos => write!(f, "SamePos"),
            PositionType::OpenPos => write!(f, "OpenPos"),
            PositionType::ClosePos => write!(f, "ClosePos"),
            PositionType::UnknownPos => write!(f, "UnknownPos"),
        }
    }
}

impl Default for PositionType {
    fn default() -> Self {
        PositionType::SamePos
    }
}

//==================================================================================================
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ComboLeg {
    pub con_id: i32,
    pub ratio: f64,
    pub action: String,
    // BUY /SELL / SSHORT
    pub exchange: String,
    pub open_close: PositionType,
    // for stock legs when doing short sale
    pub short_sale_slot: i32,
    pub designated_location: String,
    pub exempt_code: i32,
}

impl ComboLeg {
    pub fn new(
        con_id: i32,
        ratio: f64,
        action: String,
        exchange: String,
        open_close: PositionType,
        short_sale_slot: i32,
        designated_location: String,
        exempt_code: i32,
    ) -> Self {
        ComboLeg {
            con_id,
            ratio,
            action,
            exchange,
            open_close,
            short_sale_slot,
            designated_location,
            exempt_code,
        }
    }
}

impl Display for ComboLeg {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(
            f,
            "con_id: {},
            ratio: {},
            action: {},
            exchange: {},
            open_close: {},
            short_sale_slot: {},
            designated_location: {},
            exempt_code: {}",
            self.con_id,
            self.ratio,
            self.action,
            self.exchange,
            self.open_close,
            self.short_sale_slot,
            self.designated_location,
            self.exempt_code
        )
    }
}

//==================================================================================================
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct DeltaNeutralContract {
    pub con_id: i32,
    pub delta: f64,
    pub price: f64,
}

impl DeltaNeutralContract {
    pub fn new(con_id: i32, delta: f64, price: f64) -> Self {
        DeltaNeutralContract {
            con_id,
            delta,
            price,
        }
    }
}

impl Display for DeltaNeutralContract {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(
            f,
            "con_id: {}, delta: {}, price: {},",
            self.con_id, self.delta, self.price,
        )
    }
}

//==================================================================================================
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Contract {
    pub con_id: i32,
    pub symbol: String,
    pub sec_type: String,
    pub last_trade_date_or_contract_month: String,
    pub strike: f64,
    pub right: String,
    pub multiplier: String,
    pub exchange: String,
    pub primary_exchange: String,
    // pick an actual (ie non - aggregate) exchange that the contract trades on.DO NOT SET TO SMART.
    pub currency: String,
    pub local_symbol: String,
    pub trading_class: String,
    pub include_expired: bool,
    pub sec_id_type: String,
    // CUSIP; SEDOL; ISIN;RIC
    pub sec_id: String,

    //combos
    pub combo_legs_descrip: String,
    // received in open order 14 and up for all combos
    pub combo_legs: Vec<ComboLeg>,
    pub delta_neutral_contract: Option<DeltaNeutralContract>,
}

impl Contract {
    pub fn new(
        con_id: i32,
        symbol: String,
        sec_type: String,
        last_trade_date_or_contract_month: String,
        strike: f64,
        right: String,
        multiplier: String,
        exchange: String,
        primary_exchange: String,
        currency: String,
        local_symbol: String,
        trading_class: String,
        include_expired: bool,
        sec_id_type: String,
        sec_id: String,
        combo_legs_descrip: String,
        combo_legs: Vec<ComboLeg>,
        delta_neutral_contract: Option<DeltaNeutralContract>,
    ) -> Self {
        Contract {
            con_id,
            symbol,
            sec_type,
            last_trade_date_or_contract_month,
            strike,
            right,
            multiplier,
            exchange,
            primary_exchange,
            currency,
            local_symbol,
            trading_class,
            include_expired,
            sec_id_type,
            sec_id,
            combo_legs_descrip,
            combo_legs,
            delta_neutral_contract,
        }
    }
}

impl Display for Contract {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(
            f,
            "con_id: {},
            symbol: {},
            sec_type: {},
            last_trade_date_or_contract_month: {},
            strike: {},
            right: {},
            multiplier: {},
            exchange: {},
            primary_exchange: {},
            currency: {},
            local_symbol: {},
            trading_class: {},
            include_expired: {},
            sec_id_type: {},
            sec_id: {},
            combo_legs_descrip: {},
            combo_legs: [{}],
            delta_neutral_contract: [{:?}]
            ",
            self.con_id,
            self.symbol,
            self.sec_type,
            self.last_trade_date_or_contract_month,
            self.strike,
            self.right,
            self.multiplier,
            self.exchange,
            self.primary_exchange,
            self.currency,
            self.local_symbol,
            self.trading_class,
            self.include_expired,
            self.sec_id_type,
            self.sec_id,
            self.combo_legs_descrip,
            self.combo_legs
                .iter()
                .map(|x| { format!("{}", x.to_string()) })
                .collect::<Vec<String>>()
                .join(","),
            self.delta_neutral_contract
        )
    }
}

//==================================================================================================
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ContractDetails {
    pub contract: Contract,
    pub market_name: String,
    pub min_tick: f64,
    pub order_types: String,
    pub valid_exchanges: String,
    pub price_magnifier: i32,
    pub under_con_id: i32,
    pub long_name: String,
    pub contract_month: String,
    pub industry: String,
    pub category: String,
    pub subcategory: String,
    pub time_zone_id: String,
    pub trading_hours: String,
    pub liquid_hours: String,
    pub ev_rule: String,
    pub ev_multiplier: f64,
    pub md_size_multiplier: i32,
    pub agg_group: i32,
    pub under_symbol: String,
    pub under_sec_type: String,
    pub market_rule_ids: String,
    pub sec_id_list: Vec<TagValue>,
    pub real_expiration_date: String,
    pub last_trade_time: String,

    // BOND values
    pub cusip: String,
    pub ratings: String,
    pub desc_append: String,
    pub bond_type: String,
    pub coupon_type: String,
    pub callable: bool,
    pub putable: bool,
    pub coupon: f64,
    pub convertible: bool,
    pub maturity: String,
    pub issue_date: String,
    pub next_option_date: String,
    pub next_option_type: String,
    pub next_option_partial: bool,
    pub notes: String,
}

impl ContractDetails {
    pub fn new(
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
        ev_multiplier: f64,
        md_size_multiplier: i32,
        agg_group: i32,
        under_symbol: String,
        under_sec_type: String,
        market_rule_ids: String,
        sec_id_list: Vec<TagValue>,
        real_expiration_date: String,
        last_trade_time: String,
        cusip: String,
        ratings: String,
        desc_append: String,
        bond_type: String,
        coupon_type: String,
        callable: bool,
        putable: bool,
        coupon: f64,
        convertible: bool,
        maturity: String,
        issue_date: String,
        next_option_date: String,
        next_option_type: String,
        next_option_partial: bool,
        notes: String,
    ) -> Self {
        ContractDetails {
            contract,
            market_name,
            min_tick,
            order_types,
            valid_exchanges,
            price_magnifier,
            under_con_id,
            long_name,
            contract_month,
            industry,
            category,
            subcategory,
            time_zone_id,
            trading_hours,
            liquid_hours,
            ev_rule,
            ev_multiplier,
            md_size_multiplier,
            agg_group,
            under_symbol,
            under_sec_type,
            market_rule_ids,
            sec_id_list,
            real_expiration_date,
            last_trade_time,
            cusip,
            ratings,
            desc_append,
            bond_type,
            coupon_type,
            callable,
            putable,
            coupon,
            convertible,
            maturity,
            issue_date,
            next_option_date,
            next_option_type,
            next_option_partial,
            notes,
        }
    }
}

impl Display for ContractDetails {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(
            f,
            "contract: {},
            market_name: {},
            min_tick: {},
            order_types: {},
            valid_exchanges: {},
            price_magnifier: {},
            under_con_id: {},
            long_name: {},
            contract_month: {},
            industry: {},
            category: {},
            subcategory: {},
            time_zone_id: {},
            trading_hours: {},
            liquid_hours: {},
            ev_rule: {},
            ev_multiplier: {},
            md_size_multiplier: {},
            agg_group: {},
            under_symbol: {},
            under_sec_type: {},
            market_rule_ids: {},
            sec_id_list: {},
            real_expiration_date: {},
            last_trade_time: {},
            cusip: {},
            ratings: {},
            desc_append: {},
            bond_type: {},
            coupon_type: {},
            callable: {},
            putable: {},
            coupon: {},
            convertible: {},
            maturity: {},
            issue_date: {},
            next_option_date: {},
            next_option_type: {},
            next_option_partial: {},
            notes: {},",
            self.contract,
            self.market_name,
            self.min_tick,
            self.order_types,
            self.valid_exchanges,
            self.price_magnifier,
            self.under_con_id,
            self.long_name,
            self.contract_month,
            self.industry,
            self.category,
            self.subcategory,
            self.time_zone_id,
            self.trading_hours,
            self.liquid_hours,
            self.ev_rule,
            self.ev_multiplier,
            self.md_size_multiplier,
            self.agg_group,
            self.under_symbol,
            self.under_sec_type,
            self.market_rule_ids,
            self.sec_id_list
                .iter()
                .map(|x| { format!("{}", x.to_string()) })
                .collect::<Vec<String>>()
                .join(","),
            self.real_expiration_date,
            self.last_trade_time,
            self.cusip,
            self.ratings,
            self.desc_append,
            self.bond_type,
            self.coupon_type,
            self.callable,
            self.putable,
            self.coupon,
            self.convertible,
            self.maturity,
            self.issue_date,
            self.next_option_date,
            self.next_option_type,
            self.next_option_partial,
            self.notes
        )
    }
}

//==================================================================================================
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ContractDescription {
    pub contract: Contract,
    pub derivative_sec_types: Vec<String>, // type: list of strings
}

impl ContractDescription {
    pub fn new(contract: Contract, derivative_sec_types: Vec<String>) -> Self {
        ContractDescription {
            contract,
            derivative_sec_types,
        }
    }
}

impl Display for ContractDescription {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(
            f,
            "contract: {}, derivative_sec_types: ({})",
            self.contract,
            self.derivative_sec_types
                .iter()
                .map(|x| { x.to_owned() })
                .collect::<Vec<String>>()
                .join(",")
        )
    }
}
