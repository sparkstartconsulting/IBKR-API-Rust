use num_derive::FromPrimitive;
// 0.2.4 (the derive)
use num_traits::FromPrimitive;
use serde::{Deserialize, Serialize};
// 0.2.6 (the trait)
use serde::export::fmt::{Display, Error};
use serde::export::Formatter;

use crate::core::common::TagValue;
use crate::core::order_condition::OrderConditionEnum;

// enum Origin
//==================================================================================================
#[repr(i32)]
#[derive(Serialize, Deserialize, Clone, Debug, FromPrimitive)]
pub enum Origin {
    Customer = 0,
    Firm = 1,
    Unknown = 2,
}

impl Default for Origin {
    fn default() -> Self {
        Origin::Unknown
    }
}

// enum AuctionStrategy
//==================================================================================================
#[repr(i32)]
#[derive(Serialize, Deserialize, Clone, Debug, FromPrimitive)]
pub enum AuctionStrategy {
    AuctionUnset = 0,
    AuctionMatch = 1,
    AuctionImprovement = 2,
    AuctionTransparent = 3,
}

impl Default for AuctionStrategy {
    fn default() -> Self {
        AuctionStrategy::AuctionUnset
    }
}

//==================================================================================================
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct SoftDollarTier {
    pub name: String,
    pub val: String,
    pub display_name: String,
}

impl SoftDollarTier {
    pub fn new(name: String, val: String, display_name: String) -> Self {
        SoftDollarTier {
            name,
            val,
            display_name,
        }
    }
}

impl Display for SoftDollarTier {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(
            f,
            "name: {}, value: {}, display_name: {}",
            self.name, self.val, self.display_name
        )
    }
}

//==================================================================================================
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct OrderState {
    pub status: String,
    pub init_margin_before: String,
    pub maint_margin_before: String,
    pub equity_with_loan_before: String,
    pub init_margin_change: String,
    pub maint_margin_change: String,
    pub equity_with_loan_change: String,
    pub init_margin_after: String,
    pub maint_margin_after: String,
    pub equity_with_loan_after: String,
    pub commission: f64,
    pub min_commission: f64,
    pub max_commission: f64,
    pub commission_currency: String,
    pub warning_text: String,
    pub completed_time: String,
    pub completed_status: String,
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

impl Display for OrderState {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
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
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct OrderComboLeg {
    pub(crate) price: f64, // type: float
}

impl OrderComboLeg {
    pub fn new(price: f64) -> Self {
        OrderComboLeg { price }
    }
}

impl Display for OrderComboLeg {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}", self.price)
    }
}

//==================================================================================================
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Order {
    pub soft_dollar_tier: SoftDollarTier,
    // order identifier
    pub order_id: i32,
    pub client_id: i32,
    pub perm_id: i32,

    // main order fields
    pub action: String,
    pub total_quantity: f64,
    pub order_type: String,
    pub lmt_price: f64,
    pub(crate) aux_price: f64,

    // extended order fields
    pub tif: String,
    // "Time in Force" - DAY, GTC, etc.
    pub active_start_time: String,
    // for GTC orders
    pub active_stop_time: String,
    // for GTC orders
    pub oca_group: String,
    // one cancels all group name
    pub oca_type: i32,
    // 1 = CANCEL_WITH_BLOCK, 2 = REDUCE_WITH_BLOCK, 3 = REDUCE_NON_BLOCK
    pub order_ref: String,
    pub transmit: bool,
    // if false, order will be created but not transmited
    pub parent_id: i32,
    // Parent order Id, to associate Auto STP or TRAIL orders with the original order.
    pub block_order: bool,
    pub sweep_to_fill: bool,
    pub display_size: i32,
    pub trigger_method: i32,
    // 0=Default, 1=Double_Bid_Ask, 2=Last, 3=Double_Last, 4=Bid_Ask, 7=Last_or_Bid_Ask, 8=Mid-point
    pub outside_rth: bool,
    pub hidden: bool,
    pub good_after_time: String,
    // Format: 20060505 08:00:00 {time zone}
    pub good_till_date: String,
    // Format: 20060505 08:00:00 {time zone}
    pub rule80a: String,
    // Individual = 'I', Agency = 'A', AgentOtherMember = 'W', IndividualPTIA = 'J', AgencyPTIA = 'U', AgentOtherMemberPTIA = 'M', IndividualPT = 'K', AgencyPT = 'Y', AgentOtherMemberPT = 'N'
    pub all_or_none: bool,
    pub min_qty: i32,
    //type: int
    pub percent_offset: f64,
    // type: float; REL orders only
    pub override_percentage_constraints: bool,
    pub trail_stop_price: f64,
    // type: float
    pub trailing_percent: f64, // type: float; TRAILLIMIT orders only

    // financial advisors only
    pub fa_group: String,
    pub fa_profile: String,
    pub fa_method: String,
    pub fa_percentage: String,

    // institutional (ie non-cleared) only
    pub designated_location: String,
    //used only when short_sale_slot=2
    pub open_close: String,
    // O=Open, C=Close
    pub origin: Origin,
    // 0=Customer, 1=Firm
    pub short_sale_slot: i32,
    // type: int; 1 if you hold the shares, 2 if they will be delivered from elsewhere.  Only for Action=SSHORT
    pub exempt_code: i32,

    // SMART routing only
    pub discretionary_amt: f64,
    pub e_trade_only: bool,
    pub firm_quote_only: bool,
    pub nbbo_price_cap: f64,
    // type: float
    pub opt_out_smart_routing: bool,

    // BOX exchange orders only
    pub auction_strategy: AuctionStrategy,
    // type: int; AuctionMatch, AuctionImprovement, AuctionTransparent
    pub starting_price: f64,
    // type: float
    pub stock_ref_price: f64,
    // type: float
    pub delta: f64, // type: float

    // pegged to stock and VOL orders only
    pub stock_range_lower: f64,
    // type: float
    pub stock_range_upper: f64, // type: float

    pub randomize_price: bool,
    pub randomize_size: bool,

    // VOLATILITY ORDERS ONLY
    pub volatility: f64,
    // type: float
    pub volatility_type: i32,
    // type: int   // 1=daily, 2=annual
    pub delta_neutral_order_type: String,
    pub delta_neutral_aux_price: f64,
    // type: float
    pub delta_neutral_con_id: i32,
    pub delta_neutral_settling_firm: String,
    pub delta_neutral_clearing_account: String,
    pub delta_neutral_clearing_intent: String,
    pub delta_neutral_open_close: String,
    pub delta_neutral_short_sale: bool,
    pub delta_neutral_short_sale_slot: i32,
    pub delta_neutral_designated_location: String,
    pub continuous_update: bool,
    pub reference_price_type: i32, // type: int; 1=Average, 2 = BidOrAsk

    // COMBO ORDERS ONLY
    pub basis_points: f64,
    // type: float; EFP orders only
    pub basis_points_type: i32, // type: int;  EFP orders only

    // SCALE ORDERS ONLY
    pub scale_init_level_size: i32,
    // type: int
    pub scale_subs_level_size: i32,
    // type: int
    pub scale_price_increment: f64,
    // type: float
    pub scale_price_adjust_value: f64,
    // type: float
    pub scale_price_adjust_interval: i32,
    // type: int
    pub scale_profit_offset: f64,
    // type: float
    pub scale_auto_reset: bool,
    pub scale_init_position: i32,
    // type: int
    pub scale_init_fill_qty: i32,
    // type: int
    pub scale_random_percent: bool,
    pub scale_table: String,

    // HEDGE ORDERS
    pub hedge_type: String,
    // 'D' - delta, 'B' - beta, 'F' - FX, 'P' - pair
    pub hedge_param: String, // 'beta=X' value for beta hedge, 'ratio=Y' for pair hedge

    // Clearing info
    pub account: String,
    // IB account
    pub settling_firm: String,
    pub clearing_account: String,
    //True beneficiary of the order
    pub clearing_intent: String, // "" (Default), "IB", "Away", "PTA" (PostTrade)

    // ALGO ORDERS ONLY
    pub algo_strategy: String,

    pub algo_params: Vec<TagValue>,
    //TagValueList
    pub smart_combo_routing_params: Vec<TagValue>, //TagValueList

    pub algo_id: String,

    // What-if
    pub what_if: bool,

    // Not Held
    pub not_held: bool,
    pub solicited: bool,

    // models
    pub model_code: String,

    // order combo legs
    pub order_combo_legs: Vec<OrderComboLeg>, // OrderComboLegListSPtr

    pub order_misc_options: Vec<TagValue>, // TagValueList

    // VER PEG2BENCH fields:
    pub reference_contract_id: i32,
    pub pegged_change_amount: f64,
    pub is_pegged_change_amount_decrease: bool,
    pub reference_change_amount: f64,
    pub reference_exchange_id: String,
    pub adjusted_order_type: String,

    pub trigger_price: f64,
    pub adjusted_stop_price: f64,
    pub adjusted_stop_limit_price: f64,
    pub adjusted_trailing_amount: f64,
    pub adjustable_trailing_unit: i32,
    pub lmt_price_offset: f64,

    pub conditions: Vec<OrderConditionEnum>,
    // std::vector<std::shared_ptr<OrderCondition>>
    pub conditions_cancel_order: bool,
    pub conditions_ignore_rth: bool,

    // ext operator
    pub ext_operator: String,

    // native cash quantity
    pub cash_qty: f64,

    pub mifid2decision_maker: String,
    pub mifid2decision_algo: String,
    pub mifid2execution_trader: String,
    pub mifid2execution_algo: String,

    pub dont_use_auto_price_for_hedge: bool,

    pub is_oms_container: bool,

    pub discretionary_up_to_limit_price: bool,

    pub auto_cancel_date: String,
    pub filled_quantity: f64,
    pub ref_futures_con_id: i32,
    pub auto_cancel_parent: bool,
    pub shareholder: String,
    pub imbalance_only: bool,
    pub route_marketable_to_bbo: bool,
    pub parent_perm_id: i32,

    pub use_price_mgmt_algo: bool,
}

impl Order {
    pub fn new(
        soft_dollar_tier: SoftDollarTier,
        order_id: i32,
        client_id: i32,
        perm_id: i32,
        action: String,
        total_quantity: f64,
        order_type: String,
        lmt_price: f64,
        aux_price: f64,
        tif: String,
        active_start_time: String,
        active_stop_time: String,
        oca_group: String,
        oca_type: i32,
        order_ref: String,
        transmit: bool,
        parent_id: i32,
        block_order: bool,
        sweep_to_fill: bool,
        display_size: i32,
        trigger_method: i32,
        outside_rth: bool,
        hidden: bool,
        good_after_time: String,
        good_till_date: String,
        rule80a: String,
        all_or_none: bool,
        min_qty: i32,
        percent_offset: f64,
        override_percentage_constraints: bool,
        trail_stop_price: f64,
        trailing_percent: f64,
        fa_group: String,
        fa_profile: String,
        fa_method: String,
        fa_percentage: String,
        designated_location: String,
        open_close: String,
        origin: Origin,
        short_sale_slot: i32,
        exempt_code: i32,
        discretionary_amt: f64,
        e_trade_only: bool,
        firm_quote_only: bool,
        nbbo_price_cap: f64,
        opt_out_smart_routing: bool,
        auction_strategy: AuctionStrategy,
        starting_price: f64,
        stock_ref_price: f64,
        delta: f64,
        stock_range_lower: f64,
        stock_range_upper: f64,
        randomize_price: bool,
        randomize_size: bool,
        volatility: f64,
        volatility_type: i32,
        delta_neutral_order_type: String,
        delta_neutral_aux_price: f64,
        delta_neutral_con_id: i32,
        delta_neutral_settling_firm: String,
        delta_neutral_clearing_account: String,
        delta_neutral_clearing_intent: String,
        delta_neutral_open_close: String,
        delta_neutral_short_sale: bool,
        delta_neutral_short_sale_slot: i32,
        delta_neutral_designated_location: String,
        continuous_update: bool,
        reference_price_type: i32,
        basis_points: f64,
        basis_points_type: i32,
        scale_init_level_size: i32,
        scale_subs_level_size: i32,
        scale_price_increment: f64,
        scale_price_adjust_value: f64,
        scale_price_adjust_interval: i32,
        scale_profit_offset: f64,
        scale_auto_reset: bool,
        scale_init_position: i32,
        scale_init_fill_qty: i32,
        scale_random_percent: bool,
        scale_table: String,
        hedge_type: String,
        hedge_param: String,
        account: String,
        settling_firm: String,
        clearing_account: String,
        clearing_intent: String,
        algo_strategy: String,
        algo_params: Vec<TagValue>,
        smart_combo_routing_params: Vec<TagValue>,
        algo_id: String,
        what_if: bool,
        not_held: bool,
        solicited: bool,
        model_code: String,
        order_combo_legs: Vec<OrderComboLeg>,
        order_misc_options: Vec<TagValue>,
        reference_contract_id: i32,
        pegged_change_amount: f64,
        is_pegged_change_amount_decrease: bool,
        reference_change_amount: f64,
        reference_exchange_id: String,
        adjusted_order_type: String,
        trigger_price: f64,
        adjusted_stop_price: f64,
        adjusted_stop_limit_price: f64,
        adjusted_trailing_amount: f64,
        adjustable_trailing_unit: i32,
        lmt_price_offset: f64,
        conditions: Vec<OrderConditionEnum>,
        conditions_cancel_order: bool,
        conditions_ignore_rth: bool,
        ext_operator: String,
        cash_qty: f64,
        mifid2decision_maker: String,
        mifid2decision_algo: String,
        mifid2execution_trader: String,
        mifid2execution_algo: String,
        dont_use_auto_price_for_hedge: bool,
        is_oms_container: bool,
        discretionary_up_to_limit_price: bool,
        auto_cancel_date: String,
        filled_quantity: f64,
        ref_futures_con_id: i32,
        auto_cancel_parent: bool,
        shareholder: String,
        imbalance_only: bool,
        route_marketable_to_bbo: bool,
        parent_perm_id: i32,
        use_price_mgmt_algo: bool,
    ) -> Self {
        Order {
            soft_dollar_tier,
            order_id,
            client_id,
            perm_id,
            action,
            total_quantity,
            order_type,
            lmt_price,
            aux_price,
            tif,
            active_start_time,
            active_stop_time,
            oca_group,
            oca_type,
            order_ref,
            transmit,
            parent_id,
            block_order,
            sweep_to_fill,
            display_size,
            trigger_method,
            outside_rth,
            hidden,
            good_after_time,
            good_till_date,
            rule80a,
            all_or_none,
            min_qty,
            percent_offset,
            override_percentage_constraints,
            trail_stop_price,
            trailing_percent,
            fa_group,
            fa_profile,
            fa_method,
            fa_percentage,
            designated_location,
            open_close,
            origin,
            short_sale_slot,
            exempt_code,
            discretionary_amt,
            e_trade_only,
            firm_quote_only,
            nbbo_price_cap,
            opt_out_smart_routing,
            auction_strategy,
            starting_price,
            stock_ref_price,
            delta,
            stock_range_lower,
            stock_range_upper,
            randomize_price,
            randomize_size,
            volatility,
            volatility_type,
            delta_neutral_order_type,
            delta_neutral_aux_price,
            delta_neutral_con_id,
            delta_neutral_settling_firm,
            delta_neutral_clearing_account,
            delta_neutral_clearing_intent,
            delta_neutral_open_close,
            delta_neutral_short_sale,
            delta_neutral_short_sale_slot,
            delta_neutral_designated_location,
            continuous_update,
            reference_price_type,
            basis_points,
            basis_points_type,
            scale_init_level_size,
            scale_subs_level_size,
            scale_price_increment,
            scale_price_adjust_value,
            scale_price_adjust_interval,
            scale_profit_offset,
            scale_auto_reset,
            scale_init_position,
            scale_init_fill_qty,
            scale_random_percent,
            scale_table,
            hedge_type,
            hedge_param,
            account,
            settling_firm,
            clearing_account,
            clearing_intent,
            algo_strategy,
            algo_params,
            smart_combo_routing_params,
            algo_id,
            what_if,
            not_held,
            solicited,
            model_code,
            order_combo_legs,
            order_misc_options,
            reference_contract_id,
            pegged_change_amount,
            is_pegged_change_amount_decrease,
            reference_change_amount,
            reference_exchange_id,
            adjusted_order_type,
            trigger_price,
            adjusted_stop_price,
            adjusted_stop_limit_price,
            adjusted_trailing_amount,
            adjustable_trailing_unit,
            lmt_price_offset,
            conditions,
            conditions_cancel_order,
            conditions_ignore_rth,
            ext_operator,
            cash_qty,
            mifid2decision_maker,
            mifid2decision_algo,
            mifid2execution_trader,
            mifid2execution_algo,
            dont_use_auto_price_for_hedge,
            is_oms_container,
            discretionary_up_to_limit_price,
            auto_cancel_date,
            filled_quantity,
            ref_futures_con_id,
            auto_cancel_parent,
            shareholder,
            imbalance_only,
            route_marketable_to_bbo,
            parent_perm_id,
            use_price_mgmt_algo,
        }
    }
}

impl Display for Order {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(
            f,
            "order_id = {},client_id = {},perm_id = {},order_type = {},action = {},total_quantity = {},lmt_price = {},tif = {},CMB({}),COND({})",
            self.order_id,
            self.client_id,
            self.perm_id,
            self.order_type,
            self.action,
            self.total_quantity,
            self.lmt_price,
            self.tif,
            if !self.order_combo_legs.is_empty() {
                self.order_combo_legs
                    .iter()
                    .map(|x| format!("{}", x))
                    .collect::<Vec<String>>()
                    .join(",")
            } else {
                "".to_string()
            },
            if !self.conditions.is_empty() {
                self.conditions
                    .iter()
                    .map(|x| format!("{}", x.to_string()))
                    .collect::<Vec<String>>()
                    .join(",")
            } else {
                "".to_string()
            },
        )
    }
}
