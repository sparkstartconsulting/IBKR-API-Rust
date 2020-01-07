use std::slice::Iter;

use num_traits::FromPrimitive;

use crate::core::common::{TagValue, UNSET_DOUBLE};
use crate::core::contract::{ComboLeg, Contract, DeltaNeutralContract};
use crate::core::decoder::{
    decode_bool, decode_f64, decode_f64_show_unset, decode_i32, decode_i32_show_unset,
    decode_string,
};
use crate::core::errors::IBKRApiLibError;
use crate::core::order::{Order, OrderComboLeg, OrderState, SoftDollarTier};
use crate::core::order_condition::{create_condition, Condition};
use crate::core::server_versions::{
    MIN_SERVER_VER_AUTO_PRICE_FOR_HEDGE, MIN_SERVER_VER_CASH_QTY, MIN_SERVER_VER_D_PEG_ORDERS,
    MIN_SERVER_VER_FRACTIONAL_POSITIONS, MIN_SERVER_VER_MODELS_SUPPORT,
    MIN_SERVER_VER_ORDER_CONTAINER, MIN_SERVER_VER_PEGGED_TO_BENCHMARK,
    MIN_SERVER_VER_PRICE_MGMT_ALGO, MIN_SERVER_VER_SOFT_DOLLAR_TIER, MIN_SERVER_VER_SSHORTX_OLD,
    MIN_SERVER_VER_WHAT_IF_EXT_FIELDS,
};

pub struct OrderDecoder<'a> {
    contract: &'a mut Contract,
    order: &'a mut Order,
    order_state: &'a mut OrderState,
    version: i32,
    server_version: i32,
}

impl<'a> OrderDecoder<'a> {
    pub fn new(
        contract: &'a mut Contract,
        order: &'a mut Order,
        order_state: &'a mut OrderState,
        version: i32,
        server_version: i32,
    ) -> Self {
        OrderDecoder {
            contract,
            order,
            order_state,
            version,
            server_version,
        }
    }

    pub fn decode_completed(
        &mut self,
        fields_iter: &mut Iter<String>,
    ) -> Result<(), IBKRApiLibError> {
        // read contract fields
        self.decode_contract_fields(fields_iter);

        // read order fields
        self.decode_action(fields_iter);
        self.decode_total_quantity(fields_iter);
        self.decode_order_type(fields_iter);
        self.decode_lmt_price(fields_iter);
        self.decode_aux_price(fields_iter);
        self.decode_tif(fields_iter);
        self.decode_oca_group(fields_iter);
        self.decode_account(fields_iter);
        self.decode_open_close(fields_iter);
        self.decode_origin(fields_iter);
        self.decode_order_ref(fields_iter);
        self.decode_perm_id(fields_iter);
        self.decode_outside_rth(fields_iter);
        self.decode_hidden(fields_iter);
        self.decode_discretionary_amt(fields_iter);
        self.decode_good_after_time(fields_iter);
        self.decode_faparams(fields_iter);
        self.decode_model_code(fields_iter);
        self.decode_good_till_date(fields_iter);
        self.decode_rule80a(fields_iter);
        self.decode_percent_offset(fields_iter);
        self.decode_settling_firm(fields_iter);
        self.decode_short_sale_params(fields_iter);
        self.decode_box_order_params(fields_iter);
        self.decode_peg_to_stk_or_vol_order_params(fields_iter);
        self.decode_display_size(fields_iter);
        self.decode_sweep_to_fill(fields_iter);
        self.decode_all_or_none(fields_iter);
        self.decode_min_qty(fields_iter);
        self.decode_oca_type(fields_iter);
        self.decode_trigger_method(fields_iter);
        self.decode_vol_order_params(fields_iter, false);
        self.decode_trail_params(fields_iter);
        self.decode_combo_legs(fields_iter);
        self.decode_smart_combo_routing_params(fields_iter);
        self.decode_scale_order_params(fields_iter);
        self.decode_hedge_params(fields_iter);
        self.decode_clearing_params(fields_iter);
        self.decode_not_held(fields_iter);
        self.decode_delta_neutral(fields_iter);
        self.decode_algo_params(fields_iter);
        self.decode_solicited(fields_iter);
        self.decode_order_status(fields_iter);
        self.decode_vol_randomize_flags(fields_iter);
        self.decode_peg_to_bench_params(fields_iter);
        self.decode_conditions(fields_iter);
        self.decode_stop_price_and_lmt_price_offset(fields_iter);
        self.decode_cash_qty(fields_iter);
        self.decode_dont_use_auto_price_for_hedge(fields_iter);
        self.decode_is_oms_containers(fields_iter);
        self.decode_auto_cancel_date(fields_iter);
        self.decode_filled_quantity(fields_iter);
        self.decode_ref_futures_con_id(fields_iter);
        self.decode_auto_cancel_parent(fields_iter);
        self.decode_shareholder(fields_iter);
        self.decode_imbalance_only(fields_iter);
        self.decode_route_marketable_to_bbo(fields_iter);
        self.decode_parent_perm_id(fields_iter);
        self.decode_completed_time(fields_iter);
        self.decode_completed_status(fields_iter);

        Ok(())
    }

    pub fn decode_open(&mut self, fields_iter: &mut Iter<String>) -> Result<(), IBKRApiLibError> {
        self.decode_order_id(fields_iter);

        // read contract fields
        self.decode_contract_fields(fields_iter);

        // read order fields
        self.decode_action(fields_iter);
        self.decode_total_quantity(fields_iter);
        self.decode_order_type(fields_iter);
        self.decode_lmt_price(fields_iter);
        self.decode_aux_price(fields_iter);
        self.decode_tif(fields_iter);
        self.decode_oca_group(fields_iter);
        self.decode_account(fields_iter);
        self.decode_open_close(fields_iter);
        self.decode_origin(fields_iter);
        self.decode_order_ref(fields_iter);
        self.decode_client_id(fields_iter);
        self.decode_perm_id(fields_iter);
        self.decode_outside_rth(fields_iter);
        self.decode_hidden(fields_iter);
        self.decode_discretionary_amt(fields_iter);
        self.decode_good_after_time(fields_iter);
        self.skip_shares_allocation(fields_iter);
        self.decode_faparams(fields_iter);
        self.decode_model_code(fields_iter);
        self.decode_good_till_date(fields_iter);
        self.decode_rule80a(fields_iter);
        self.decode_percent_offset(fields_iter);
        self.decode_settling_firm(fields_iter);
        self.decode_short_sale_params(fields_iter);
        self.decode_auction_strategy(fields_iter);
        self.decode_box_order_params(fields_iter);
        self.decode_peg_to_stk_or_vol_order_params(fields_iter);
        self.decode_display_size(fields_iter);
        self.decode_block_order(fields_iter);
        self.decode_sweep_to_fill(fields_iter);
        self.decode_all_or_none(fields_iter);
        self.decode_min_qty(fields_iter);
        self.decode_oca_type(fields_iter);
        self.decode_etrade_only(fields_iter);
        self.decode_firm_quote_only(fields_iter);
        self.decode_nbbo_price_cap(fields_iter);
        self.decode_parent_id(fields_iter);
        self.decode_trigger_method(fields_iter);
        self.decode_vol_order_params(fields_iter, true);
        self.decode_trail_params(fields_iter);
        self.decode_basis_points(fields_iter);
        self.decode_combo_legs(fields_iter);
        self.decode_smart_combo_routing_params(fields_iter);
        self.decode_scale_order_params(fields_iter);
        self.decode_hedge_params(fields_iter);
        self.decode_opt_out_smart_routing(fields_iter);
        self.decode_clearing_params(fields_iter);
        self.decode_not_held(fields_iter);
        self.decode_delta_neutral(fields_iter);
        self.decode_algo_params(fields_iter);
        self.decode_solicited(fields_iter);
        self.decode_what_if_info_and_commission(fields_iter);
        self.decode_vol_randomize_flags(fields_iter);
        self.decode_peg_to_bench_params(fields_iter);
        self.decode_conditions(fields_iter);
        self.decode_adjusted_order_params(fields_iter);
        self.decode_soft_dollar_tier(fields_iter);
        self.decode_cash_qty(fields_iter);
        self.decode_dont_use_auto_price_for_hedge(fields_iter);
        self.decode_is_oms_containers(fields_iter);
        self.decode_discretionary_up_to_limit_price(fields_iter);
        self.decode_use_price_mgmt_algo(fields_iter);

        Ok(())
    }
    fn decode_order_id(&mut self, fields_iter: &mut Iter<String>) -> Result<(), IBKRApiLibError> {
        self.order.order_id = decode_i32(fields_iter)?;
        Ok(())
    }

    fn decode_contract_fields(
        &mut self,
        fields_iter: &mut Iter<String>,
    ) -> Result<(), IBKRApiLibError> {
        self.contract.con_id = decode_i32(fields_iter)?;
        self.contract.symbol = decode_string(fields_iter)?;
        self.contract.sec_type = decode_string(fields_iter)?;
        self.contract.last_trade_date_or_contract_month = decode_string(fields_iter)?;
        self.contract.strike = decode_f64(fields_iter)?;
        self.contract.right = decode_string(fields_iter)?;
        if self.version >= 32 {
            self.contract.multiplier = decode_string(fields_iter)?;
        }
        self.contract.exchange = decode_string(fields_iter)?;
        self.contract.currency = decode_string(fields_iter)?;
        self.contract.local_symbol = decode_string(fields_iter)?;
        if self.version >= 32 {
            self.contract.trading_class = decode_string(fields_iter)?;
        }

        Ok(())
    }

    fn decode_action(&mut self, fields_iter: &mut Iter<String>) -> Result<(), IBKRApiLibError> {
        self.order.action = decode_string(fields_iter)?;
        Ok(())
    }

    fn decode_total_quantity(
        &mut self,
        fields_iter: &mut Iter<String>,
    ) -> Result<(), IBKRApiLibError> {
        if self.server_version >= MIN_SERVER_VER_FRACTIONAL_POSITIONS {
            self.order.total_quantity = decode_f64(fields_iter)?;
        } else {
            self.order.total_quantity = decode_i32(fields_iter)? as f64;
        }
        Ok(())
    }

    fn decode_order_type(&mut self, fields_iter: &mut Iter<String>) -> Result<(), IBKRApiLibError> {
        self.order.order_type = decode_string(fields_iter)?;
        Ok(())
    }

    fn decode_lmt_price(&mut self, fields_iter: &mut Iter<String>) -> Result<(), IBKRApiLibError> {
        if self.version < 29 {
            self.order.lmt_price = decode_f64(fields_iter)?;
        } else {
            self.order.lmt_price = decode_f64_show_unset(fields_iter)?;
        }
        Ok(())
    }

    fn decode_aux_price(&mut self, fields_iter: &mut Iter<String>) -> Result<(), IBKRApiLibError> {
        if self.version < 30 {
            self.order.aux_price = decode_f64(fields_iter)?;
        } else {
            self.order.aux_price = decode_f64_show_unset(fields_iter)?;
        }

        Ok(())
    }

    fn decode_tif(&mut self, fields_iter: &mut Iter<String>) -> Result<(), IBKRApiLibError> {
        self.order.tif = decode_string(fields_iter)?;

        Ok(())
    }
    fn decode_oca_group(&mut self, fields_iter: &mut Iter<String>) -> Result<(), IBKRApiLibError> {
        self.order.oca_group = decode_string(fields_iter)?;
        Ok(())
    }
    fn decode_account(&mut self, fields_iter: &mut Iter<String>) -> Result<(), IBKRApiLibError> {
        self.order.account = decode_string(fields_iter)?;
        Ok(())
    }
    fn decode_open_close(&mut self, fields_iter: &mut Iter<String>) -> Result<(), IBKRApiLibError> {
        self.order.open_close = decode_string(fields_iter)?;

        Ok(())
    }
    fn decode_origin(&mut self, fields_iter: &mut Iter<String>) -> Result<(), IBKRApiLibError> {
        self.order.origin = FromPrimitive::from_i32(decode_i32(fields_iter)?).unwrap();
        Ok(())
    }
    fn decode_order_ref(&mut self, fields_iter: &mut Iter<String>) -> Result<(), IBKRApiLibError> {
        self.order.order_ref = decode_string(fields_iter)?;
        Ok(())
    }
    fn decode_client_id(&mut self, fields_iter: &mut Iter<String>) -> Result<(), IBKRApiLibError> {
        self.order.client_id = decode_i32(fields_iter)?;
        Ok(())
    }
    fn decode_perm_id(&mut self, fields_iter: &mut Iter<String>) -> Result<(), IBKRApiLibError> {
        self.order.perm_id = decode_i32(fields_iter)?;
        Ok(())
    }
    fn decode_outside_rth(
        &mut self,
        fields_iter: &mut Iter<String>,
    ) -> Result<(), IBKRApiLibError> {
        self.order.outside_rth = decode_bool(fields_iter)?;
        Ok(())
    }
    fn decode_hidden(&mut self, fields_iter: &mut Iter<String>) -> Result<(), IBKRApiLibError> {
        self.order.hidden = decode_bool(fields_iter)?;
        Ok(())
    }
    fn decode_discretionary_amt(
        &mut self,
        fields_iter: &mut Iter<String>,
    ) -> Result<(), IBKRApiLibError> {
        self.order.discretionary_amt = decode_f64(fields_iter)?;
        Ok(())
    }

    fn decode_good_after_time(
        &mut self,
        fields_iter: &mut Iter<String>,
    ) -> Result<(), IBKRApiLibError> {
        self.order.good_after_time = decode_string(fields_iter)?;
        Ok(())
    }
    fn skip_shares_allocation(
        &mut self,
        fields_iter: &mut Iter<String>,
    ) -> Result<(), IBKRApiLibError> {
        decode_string(fields_iter)?; // deprecated
        Ok(())
    }

    fn decode_faparams(&mut self, fields_iter: &mut Iter<String>) -> Result<(), IBKRApiLibError> {
        self.order.fa_group = decode_string(fields_iter)?;
        self.order.fa_method = decode_string(fields_iter)?;
        self.order.fa_percentage = decode_string(fields_iter)?;
        self.order.fa_profile = decode_string(fields_iter)?;
        Ok(())
    }

    fn decode_model_code(&mut self, fields_iter: &mut Iter<String>) -> Result<(), IBKRApiLibError> {
        if self.server_version >= MIN_SERVER_VER_MODELS_SUPPORT {
            self.order.model_code = decode_string(fields_iter)?;
        }
        Ok(())
    }

    fn decode_good_till_date(
        &mut self,
        fields_iter: &mut Iter<String>,
    ) -> Result<(), IBKRApiLibError> {
        self.order.good_till_date = decode_string(fields_iter)?;
        Ok(())
    }

    fn decode_rule80a(&mut self, fields_iter: &mut Iter<String>) -> Result<(), IBKRApiLibError> {
        self.order.rule80a = decode_string(fields_iter)?;
        Ok(())
    }

    fn decode_percent_offset(
        &mut self,
        fields_iter: &mut Iter<String>,
    ) -> Result<(), IBKRApiLibError> {
        self.order.percent_offset = decode_f64_show_unset(fields_iter)?;
        Ok(())
    }

    fn decode_settling_firm(
        &mut self,
        fields_iter: &mut Iter<String>,
    ) -> Result<(), IBKRApiLibError> {
        self.order.settling_firm = decode_string(fields_iter)?;
        Ok(())
    }

    fn decode_short_sale_params(
        &mut self,
        fields_iter: &mut Iter<String>,
    ) -> Result<(), IBKRApiLibError> {
        self.order.short_sale_slot = decode_i32(fields_iter)?;
        self.order.designated_location = decode_string(fields_iter)?;
        if self.server_version == MIN_SERVER_VER_SSHORTX_OLD {
            decode_i32(fields_iter)?;
        } else if self.version >= 23 {
            self.order.exempt_code = decode_i32(fields_iter)?;
        }
        Ok(())
    }

    fn decode_auction_strategy(
        &mut self,
        fields_iter: &mut Iter<String>,
    ) -> Result<(), IBKRApiLibError> {
        self.order.auction_strategy = FromPrimitive::from_i32(decode_i32(fields_iter)?).unwrap();
        Ok(())
    }

    fn decode_box_order_params(
        &mut self,
        fields_iter: &mut Iter<String>,
    ) -> Result<(), IBKRApiLibError> {
        self.order.starting_price = decode_f64_show_unset(fields_iter)?;
        self.order.stock_ref_price = decode_f64_show_unset(fields_iter)?;
        self.order.delta = decode_f64_show_unset(fields_iter)?;
        Ok(())
    }

    fn decode_peg_to_stk_or_vol_order_params(
        &mut self,
        fields_iter: &mut Iter<String>,
    ) -> Result<(), IBKRApiLibError> {
        self.order.stock_range_lower = decode_f64_show_unset(fields_iter)?;
        self.order.stock_range_upper = decode_f64_show_unset(fields_iter)?;
        Ok(())
    }

    fn decode_display_size(
        &mut self,
        fields_iter: &mut Iter<String>,
    ) -> Result<(), IBKRApiLibError> {
        self.order.display_size = decode_i32(fields_iter)?;
        Ok(())
    }

    fn decode_block_order(
        &mut self,
        fields_iter: &mut Iter<String>,
    ) -> Result<(), IBKRApiLibError> {
        self.order.block_order = decode_bool(fields_iter)?;
        Ok(())
    }

    fn decode_sweep_to_fill(
        &mut self,
        fields_iter: &mut Iter<String>,
    ) -> Result<(), IBKRApiLibError> {
        self.order.sweep_to_fill = decode_bool(fields_iter)?;
        Ok(())
    }

    fn decode_all_or_none(
        &mut self,
        fields_iter: &mut Iter<String>,
    ) -> Result<(), IBKRApiLibError> {
        self.order.all_or_none = decode_bool(fields_iter)?;
        Ok(())
    }

    fn decode_min_qty(&mut self, fields_iter: &mut Iter<String>) -> Result<(), IBKRApiLibError> {
        self.order.min_qty = decode_i32_show_unset(fields_iter)?;
        Ok(())
    }

    fn decode_oca_type(&mut self, fields_iter: &mut Iter<String>) -> Result<(), IBKRApiLibError> {
        self.order.oca_type = decode_i32(fields_iter)?;
        Ok(())
    }

    fn decode_etrade_only(
        &mut self,
        fields_iter: &mut Iter<String>,
    ) -> Result<(), IBKRApiLibError> {
        self.order.e_trade_only = decode_bool(fields_iter)?;
        Ok(())
    }

    fn decode_firm_quote_only(
        &mut self,
        fields_iter: &mut Iter<String>,
    ) -> Result<(), IBKRApiLibError> {
        self.order.firm_quote_only = decode_bool(fields_iter)?;
        Ok(())
    }

    fn decode_nbbo_price_cap(
        &mut self,
        fields_iter: &mut Iter<String>,
    ) -> Result<(), IBKRApiLibError> {
        self.order.nbbo_price_cap = decode_f64_show_unset(fields_iter)?;
        Ok(())
    }
    fn decode_parent_id(&mut self, fields_iter: &mut Iter<String>) -> Result<(), IBKRApiLibError> {
        self.order.parent_id = decode_i32(fields_iter)?;
        Ok(())
    }
    fn decode_trigger_method(
        &mut self,
        fields_iter: &mut Iter<String>,
    ) -> Result<(), IBKRApiLibError> {
        self.order.trigger_method = decode_i32(fields_iter)?;
        Ok(())
    }

    fn decode_vol_order_params(
        &mut self,
        fields_iter: &mut Iter<String>,
        read_open_order_attribs: bool,
    ) -> Result<(), IBKRApiLibError> {
        self.order.volatility = decode_f64_show_unset(fields_iter)?;
        self.order.volatility_type = decode_i32(fields_iter)?;
        self.order.delta_neutral_order_type = decode_string(fields_iter)?;
        self.order.delta_neutral_aux_price = decode_f64_show_unset(fields_iter)?;

        if self.version >= 27 && self.order.delta_neutral_order_type != "" {
            self.order.delta_neutral_con_id = decode_i32(fields_iter)?;
            if read_open_order_attribs {
                self.order.delta_neutral_settling_firm = decode_string(fields_iter)?;
                self.order.delta_neutral_clearing_account = decode_string(fields_iter)?;
                self.order.delta_neutral_clearing_intent = decode_string(fields_iter)?;
            }
        }

        if self.version >= 31 && self.order.delta_neutral_order_type != "" {
            if read_open_order_attribs {
                self.order.delta_neutral_open_close = decode_string(fields_iter)?;
            }
            self.order.delta_neutral_short_sale = decode_bool(fields_iter)?;
            self.order.delta_neutral_short_sale_slot = decode_i32(fields_iter)?;
            self.order.delta_neutral_designated_location = decode_string(fields_iter)?;
        }

        self.order.continuous_update = decode_bool(fields_iter)?;
        self.order.reference_price_type = decode_i32(fields_iter)?;
        Ok(())
    }

    fn decode_trail_params(
        &mut self,
        fields_iter: &mut Iter<String>,
    ) -> Result<(), IBKRApiLibError> {
        self.order.trail_stop_price = decode_f64_show_unset(fields_iter)?;
        if self.version >= 30 {
            self.order.trailing_percent = decode_f64_show_unset(fields_iter)?;
        }
        Ok(())
    }

    fn decode_basis_points(
        &mut self,
        fields_iter: &mut Iter<String>,
    ) -> Result<(), IBKRApiLibError> {
        self.order.basis_points = decode_f64_show_unset(fields_iter)?;
        self.order.basis_points_type = decode_i32_show_unset(fields_iter)?;
        Ok(())
    }

    fn decode_combo_legs(&mut self, fields_iter: &mut Iter<String>) -> Result<(), IBKRApiLibError> {
        self.contract.combo_legs_descrip = decode_string(fields_iter)?;

        if self.version >= 29 {
            let combo_legs_count = decode_i32(fields_iter)?;

            if combo_legs_count > 0 {
                self.contract.combo_legs = vec![];
                for _ in 0..combo_legs_count {
                    let mut combo_leg: ComboLeg = ComboLeg::default();
                    combo_leg.con_id = decode_i32(fields_iter)?;
                    combo_leg.ratio = decode_i32(fields_iter)?;
                    combo_leg.action = decode_string(fields_iter)?;
                    combo_leg.exchange = decode_string(fields_iter)?;
                    combo_leg.open_close =
                        FromPrimitive::from_i32(decode_i32(fields_iter)?).unwrap();
                    combo_leg.short_sale_slot = decode_i32(fields_iter)?;
                    combo_leg.designated_location = decode_string(fields_iter)?;
                    combo_leg.exempt_code = decode_i32(fields_iter)?;
                    self.contract.combo_legs.push(combo_leg);
                }
            }
        }
        let order_combo_legs_count = decode_i32(fields_iter)?;
        if order_combo_legs_count > 0 {
            self.order.order_combo_legs = vec![];
            for _ in 0..order_combo_legs_count {
                let mut order_combo_leg = OrderComboLeg::default();
                order_combo_leg.price = decode_f64_show_unset(fields_iter)?;
                self.order.order_combo_legs.push(order_combo_leg);
            }
        }
        Ok(())
    }

    fn decode_smart_combo_routing_params(
        &mut self,
        fields_iter: &mut Iter<String>,
    ) -> Result<(), IBKRApiLibError> {
        if self.version >= 26 {
            let smart_combo_routing_params_count = decode_i32(fields_iter)?;
            if smart_combo_routing_params_count > 0 {
                self.order.smart_combo_routing_params = vec![];
                for _ in 0..smart_combo_routing_params_count {
                    let mut tagValue = TagValue::default();
                    tagValue.tag = decode_string(fields_iter)?;
                    tagValue.value = decode_string(fields_iter)?;
                    self.order.smart_combo_routing_params.push(tagValue)
                }
            }
        }
        Ok(())
    }

    fn decode_scale_order_params(
        &mut self,
        fields_iter: &mut Iter<String>,
    ) -> Result<(), IBKRApiLibError> {
        if self.version >= 20 {
            self.order.scale_init_level_size = decode_i32_show_unset(fields_iter)?;
            self.order.scale_subs_level_size = decode_i32_show_unset(fields_iter)?;
        } else {
            // self.order.scale_num_components = decode_i32_show_unset(fields_iter)?;
            self.order.scale_init_level_size = decode_i32_show_unset(fields_iter)?;
        }
        self.order.scale_price_increment = decode_f64_show_unset(fields_iter)?;

        if self.version >= 28
            && self.order.scale_price_increment != UNSET_DOUBLE
            && self.order.scale_price_increment > 0.0
        {
            self.order.scale_price_adjust_value = decode_f64_show_unset(fields_iter)?;
            self.order.scale_price_adjust_interval = decode_i32_show_unset(fields_iter)?;
            self.order.scale_profit_offset = decode_f64_show_unset(fields_iter)?;
            self.order.scale_auto_reset = decode_bool(fields_iter)?;
            self.order.scale_init_position = decode_i32_show_unset(fields_iter)?;
            self.order.scale_init_fill_qty = decode_i32_show_unset(fields_iter)?;
            self.order.scale_random_percent = decode_bool(fields_iter)?;
        }
        Ok(())
    }

    fn decode_hedge_params(
        &mut self,
        fields_iter: &mut Iter<String>,
    ) -> Result<(), IBKRApiLibError> {
        if self.version >= 24 {
            self.order.hedge_type = decode_string(fields_iter)?;
        }
        if self.order.hedge_type != "" {
            self.order.hedge_param = decode_string(fields_iter)?;
        }
        Ok(())
    }

    fn decode_opt_out_smart_routing(
        &mut self,
        fields_iter: &mut Iter<String>,
    ) -> Result<(), IBKRApiLibError> {
        if self.version >= 25 {
            self.order.opt_out_smart_routing = decode_bool(fields_iter)?;
        }
        Ok(())
    }

    fn decode_clearing_params(
        &mut self,
        fields_iter: &mut Iter<String>,
    ) -> Result<(), IBKRApiLibError> {
        self.order.clearing_account = decode_string(fields_iter)?;
        self.order.clearing_intent = decode_string(fields_iter)?;
        Ok(())
    }

    fn decode_not_held(&mut self, fields_iter: &mut Iter<String>) -> Result<(), IBKRApiLibError> {
        if self.version >= 22 {
            self.order.not_held = decode_bool(fields_iter)?;
        }
        Ok(())
    }

    fn decode_delta_neutral(
        &mut self,
        fields_iter: &mut Iter<String>,
    ) -> Result<(), IBKRApiLibError> {
        if self.version >= 20 {
            let delta_neutral_contract_present = decode_bool(fields_iter)?;
            if delta_neutral_contract_present {
                self.contract.delta_neutral_contract =
                    Option::from(DeltaNeutralContract::default());
                self.contract
                    .delta_neutral_contract
                    .as_mut()
                    .unwrap()
                    .con_id = decode_i32(fields_iter)?;
                self.contract.delta_neutral_contract.as_mut().unwrap().delta =
                    decode_f64(fields_iter)?;
                self.contract.delta_neutral_contract.as_mut().unwrap().price =
                    decode_f64(fields_iter)?;
            }
        }
        Ok(())
    }

    fn decode_algo_params(
        &mut self,
        fields_iter: &mut Iter<String>,
    ) -> Result<(), IBKRApiLibError> {
        if self.version >= 21 {
            self.order.algo_strategy = decode_string(fields_iter)?;
            if self.order.algo_strategy != "" {
                let algo_params_count = decode_i32(fields_iter)?;
                if algo_params_count > 0 {
                    self.order.algo_params = vec![];
                    for _ in 0..algo_params_count {
                        let mut tag_value = TagValue::default();
                        tag_value.tag = decode_string(fields_iter)?;
                        tag_value.value = decode_string(fields_iter)?;
                        self.order.algo_params.push(tag_value);
                    }
                }
            }
        }
        Ok(())
    }

    fn decode_solicited(&mut self, fields_iter: &mut Iter<String>) -> Result<(), IBKRApiLibError> {
        if self.version >= 33 {
            self.order.solicited = decode_bool(fields_iter)?;
        }
        Ok(())
    }

    fn decode_order_status(
        &mut self,
        fields_iter: &mut Iter<String>,
    ) -> Result<(), IBKRApiLibError> {
        self.order_state.status = decode_string(fields_iter)?;
        Ok(())
    }

    fn decode_what_if_info_and_commission(
        &mut self,
        fields_iter: &mut Iter<String>,
    ) -> Result<(), IBKRApiLibError> {
        self.order.what_if = decode_bool(fields_iter)?;
        self.decode_order_status(fields_iter);
        if self.server_version >= MIN_SERVER_VER_WHAT_IF_EXT_FIELDS {
            self.order_state.init_margin_before = decode_string(fields_iter)?;
            self.order_state.maint_margin_before = decode_string(fields_iter)?;
            self.order_state.equity_with_loan_before = decode_string(fields_iter)?;
            self.order_state.init_margin_change = decode_string(fields_iter)?;
            self.order_state.maint_margin_change = decode_string(fields_iter)?;
            self.order_state.equity_with_loan_change = decode_string(fields_iter)?;
        }

        self.order_state.init_margin_after = decode_string(fields_iter)?;
        self.order_state.maint_margin_after = decode_string(fields_iter)?;
        self.order_state.equity_with_loan_after = decode_string(fields_iter)?;

        self.order_state.commission = decode_f64_show_unset(fields_iter)?;
        self.order_state.min_commission = decode_f64_show_unset(fields_iter)?;
        self.order_state.max_commission = decode_f64_show_unset(fields_iter)?;
        self.order_state.commission_currency = decode_string(fields_iter)?;
        self.order_state.warning_text = decode_string(fields_iter)?;
        Ok(())
    }

    fn decode_vol_randomize_flags(
        &mut self,
        fields_iter: &mut Iter<String>,
    ) -> Result<(), IBKRApiLibError> {
        if self.version >= 34 {
            self.order.randomize_size = decode_bool(fields_iter)?;
            self.order.randomize_price = decode_bool(fields_iter)?;
        }
        Ok(())
    }

    fn decode_peg_to_bench_params(
        &mut self,
        fields_iter: &mut Iter<String>,
    ) -> Result<(), IBKRApiLibError> {
        if self.server_version >= MIN_SERVER_VER_PEGGED_TO_BENCHMARK {
            if self.order.order_type == "PEG BENCH" {
                self.order.reference_contract_id = decode_i32(fields_iter)?;
                self.order.is_pegged_change_amount_decrease = decode_bool(fields_iter)?;
                self.order.pegged_change_amount = decode_f64(fields_iter)?;
                self.order.reference_change_amount = decode_f64(fields_iter)?;
                self.order.reference_exchange_id = decode_string(fields_iter)?;
            }
        }
        Ok(())
    }

    fn decode_conditions(&mut self, fields_iter: &mut Iter<String>) -> Result<(), IBKRApiLibError> {
        if self.server_version >= MIN_SERVER_VER_PEGGED_TO_BENCHMARK {
            let conditions_size = decode_i32(fields_iter)?;
            if conditions_size > 0 {
                self.order.conditions = vec![];
                for _ in 0..conditions_size {
                    let condition_type = FromPrimitive::from_i32(decode_i32(fields_iter)?).unwrap();
                    let mut condition = create_condition(condition_type);
                    condition.decode(fields_iter);
                    self.order.conditions.push(condition);

                    self.order.conditions_ignore_rth = decode_bool(fields_iter)?;
                    self.order.conditions_cancel_order = decode_bool(fields_iter)?;
                }
            }
        }
        Ok(())
    }

    fn decode_adjusted_order_params(
        &mut self,
        fields_iter: &mut Iter<String>,
    ) -> Result<(), IBKRApiLibError> {
        if self.server_version >= MIN_SERVER_VER_PEGGED_TO_BENCHMARK {
            self.order.adjusted_order_type = decode_string(fields_iter)?;
            self.order.trigger_price = decode_f64(fields_iter)?;
            self.decode_stop_price_and_lmt_price_offset(fields_iter)?;
            self.order.adjusted_stop_price = decode_f64(fields_iter)?;
            self.order.adjusted_stop_limit_price = decode_f64(fields_iter)?;
            self.order.adjusted_trailing_amount = decode_f64(fields_iter)?;
            self.order.adjustable_trailing_unit = decode_i32(fields_iter)?;
        }
        Ok(())
    }

    fn decode_stop_price_and_lmt_price_offset(
        &mut self,
        fields_iter: &mut Iter<String>,
    ) -> Result<(), IBKRApiLibError> {
        self.order.trail_stop_price = decode_f64(fields_iter)?;
        self.order.lmt_price_offset = decode_f64(fields_iter)?;
        Ok(())
    }

    fn decode_soft_dollar_tier(
        &mut self,
        fields_iter: &mut Iter<String>,
    ) -> Result<(), IBKRApiLibError> {
        if self.server_version >= MIN_SERVER_VER_SOFT_DOLLAR_TIER {
            let name = decode_string(fields_iter)?;
            let value = decode_string(fields_iter)?;
            let display_name = decode_string(fields_iter)?;
            self.order.soft_dollar_tier = SoftDollarTier::new(name, value, display_name)
        }
        Ok(())
    }

    fn decode_cash_qty(&mut self, fields_iter: &mut Iter<String>) -> Result<(), IBKRApiLibError> {
        if self.server_version >= MIN_SERVER_VER_CASH_QTY {
            self.order.cash_qty = decode_f64(fields_iter)?;
        }
        Ok(())
    }

    fn decode_dont_use_auto_price_for_hedge(
        &mut self,
        fields_iter: &mut Iter<String>,
    ) -> Result<(), IBKRApiLibError> {
        if self.server_version >= MIN_SERVER_VER_AUTO_PRICE_FOR_HEDGE {
            self.order.dont_use_auto_price_for_hedge = decode_bool(fields_iter)?;
        }
        Ok(())
    }

    fn decode_is_oms_containers(
        &mut self,
        fields_iter: &mut Iter<String>,
    ) -> Result<(), IBKRApiLibError> {
        if self.server_version >= MIN_SERVER_VER_ORDER_CONTAINER {
            self.order.is_oms_container = decode_bool(fields_iter)?;
        }
        Ok(())
    }

    fn decode_discretionary_up_to_limit_price(
        &mut self,
        fields_iter: &mut Iter<String>,
    ) -> Result<(), IBKRApiLibError> {
        if self.server_version >= MIN_SERVER_VER_D_PEG_ORDERS {
            self.order.discretionary_up_to_limit_price = decode_bool(fields_iter)?;
        }
        Ok(())
    }

    fn decode_auto_cancel_date(
        &mut self,
        fields_iter: &mut Iter<String>,
    ) -> Result<(), IBKRApiLibError> {
        self.order.auto_cancel_date = decode_string(fields_iter)?;
        Ok(())
    }

    fn decode_filled_quantity(
        &mut self,
        fields_iter: &mut Iter<String>,
    ) -> Result<(), IBKRApiLibError> {
        self.order.filled_quantity = decode_f64(fields_iter)?;
        Ok(())
    }

    fn decode_ref_futures_con_id(
        &mut self,
        fields_iter: &mut Iter<String>,
    ) -> Result<(), IBKRApiLibError> {
        self.order.ref_futures_con_id = decode_i32(fields_iter)?;
        Ok(())
    }

    fn decode_auto_cancel_parent(
        &mut self,
        fields_iter: &mut Iter<String>,
    ) -> Result<(), IBKRApiLibError> {
        self.order.auto_cancel_parent = decode_bool(fields_iter)?;
        Ok(())
    }

    fn decode_shareholder(
        &mut self,
        fields_iter: &mut Iter<String>,
    ) -> Result<(), IBKRApiLibError> {
        self.order.shareholder = decode_string(fields_iter)?;
        Ok(())
    }

    fn decode_imbalance_only(
        &mut self,
        fields_iter: &mut Iter<String>,
    ) -> Result<(), IBKRApiLibError> {
        self.order.imbalance_only = decode_bool(fields_iter)?;
        Ok(())
    }

    fn decode_route_marketable_to_bbo(
        &mut self,
        fields_iter: &mut Iter<String>,
    ) -> Result<(), IBKRApiLibError> {
        self.order.route_marketable_to_bbo = decode_bool(fields_iter)?;
        Ok(())
    }

    fn decode_parent_perm_id(
        &mut self,
        fields_iter: &mut Iter<String>,
    ) -> Result<(), IBKRApiLibError> {
        self.order.parent_perm_id = decode_i32(fields_iter)?;
        Ok(())
    }

    fn decode_completed_time(
        &mut self,
        fields_iter: &mut Iter<String>,
    ) -> Result<(), IBKRApiLibError> {
        self.order_state.completed_time = decode_string(fields_iter)?;
        Ok(())
    }

    fn decode_completed_status(
        &mut self,
        fields_iter: &mut Iter<String>,
    ) -> Result<(), IBKRApiLibError> {
        self.order_state.completed_status = decode_string(fields_iter)?;
        Ok(())
    }

    fn decode_use_price_mgmt_algo(
        &mut self,
        fields_iter: &mut Iter<String>,
    ) -> Result<(), IBKRApiLibError> {
        if self.server_version >= MIN_SERVER_VER_PRICE_MGMT_ALGO {
            self.order.use_price_mgmt_algo = decode_bool(fields_iter)?;
        }
        Ok(())
    }
}
