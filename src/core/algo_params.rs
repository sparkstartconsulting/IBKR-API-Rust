use crate::core::common::TagValue;
use crate::core::order::Order;

pub fn fill_scale_params(
    base_order: &mut Order,
    scale_init_level_size: i32,
    scale_subs_level_size: i32,
    scale_random_percent: bool,
    scale_price_increment: f64,
    scale_price_adjust_value: f64,
    scale_price_adjust_interval: i32,
    scale_profit_offset: f64,
    scale_auto_reset: bool,
    scale_init_position: i32,
    scale_init_fill_qty: i32,
) {
    base_order.scale_init_level_size = scale_init_level_size; // Initial Component Size
    base_order.scale_subs_level_size = scale_subs_level_size; // Subsequent Comp. Size
    base_order.scale_random_percent = scale_random_percent; // Randomize size by +/-55%
    base_order.scale_price_increment = scale_price_increment; // Price Increment

    // Auto Price adjustment
    base_order.scale_price_adjust_value = scale_price_adjust_value; // starting price by
    base_order.scale_price_adjust_interval = scale_price_adjust_interval; // in seconds

    // Profit Orders
    base_order.scale_profit_offset = scale_profit_offset; // Create profit taking order Profit Offset
    base_order.scale_auto_reset = scale_auto_reset; // Restore size after taking profit
    base_order.scale_init_position = scale_init_position; // Initial Position
    base_order.scale_init_fill_qty = scale_init_fill_qty; // Filled initial Component Size
}

//==================================================================================================
pub fn fill_arrival_price_params(
    base_order: &mut Order,
    max_pct_vol: f64,
    risk_aversion: &str,
    start_time: &str,
    end_time: &str,
    force_completion: bool,
    allow_past_time: bool,
    monetary_value: f64,
) {
    base_order.algo_strategy = "ArrivalPx".to_string();

    base_order.algo_params.push(TagValue::new(
        "maxPctVol".to_string(),
        max_pct_vol.to_string(),
    ));
    base_order.algo_params.push(TagValue::new(
        "riskAversion".to_string(),
        risk_aversion.to_string(),
    ));
    base_order.algo_params.push(TagValue::new(
        "startTime".to_string(),
        start_time.to_string(),
    ));
    base_order
        .algo_params
        .push(TagValue::new("endTime".to_string(), end_time.to_string()));
    base_order.algo_params.push(TagValue::new(
        "forceCompletion".to_string(),
        force_completion.to_string(),
    ));
    base_order.algo_params.push(TagValue::new(
        "allowPastEndTime".to_string(),
        allow_past_time.to_string(),
    ));
    base_order.algo_params.push(TagValue::new(
        "monetaryValue".to_string(),
        monetary_value.to_string(),
    ));
}

pub fn fill_dark_ice_params(
    base_order: &mut Order,
    display_size: i32,
    start_time: &str,
    end_time: &str,
    allow_past_end_time: bool,
    monetary_value: f64,
) {
    base_order.algo_strategy = "DarkIce".to_string();
    base_order.algo_params.push(TagValue::new(
        "displaySize".to_string(),
        display_size.to_string(),
    ));
    base_order.algo_params.push(TagValue::new(
        "startTime".to_string(),
        start_time.to_string(),
    ));
    base_order
        .algo_params
        .push(TagValue::new("endTime".to_string(), end_time.to_string()));
    base_order.algo_params.push(TagValue::new(
        "allowPastEndTime".to_string(),
        (allow_past_end_time as i32).to_string(),
    ));
    base_order.algo_params.push(TagValue::new(
        "monetaryValue".to_string(),
        monetary_value.to_string(),
    ));
}

//==================================================================================================
pub fn fill_pct_vol_params(
    base_order: &mut Order,
    pct_vol: f64,
    start_time: &str,
    end_time: &str,
    no_take_liq: bool,
    monetary_value: f64,
) {
    base_order.algo_strategy = "PctVol".to_string();

    base_order
        .algo_params
        .push(TagValue::new("pctVol".to_string(), pct_vol.to_string()));
    base_order.algo_params.push(TagValue::new(
        "startTime".to_string(),
        start_time.to_string(),
    ));
    base_order
        .algo_params
        .push(TagValue::new("endTime".to_string(), end_time.to_string()));
    base_order.algo_params.push(TagValue::new(
        "noTakeLiq".to_string(),
        (no_take_liq as i32).to_string(),
    ));
    base_order.algo_params.push(TagValue::new(
        "monetaryValue".to_string(),
        monetary_value.to_string(),
    ));
}

// ! [twap_params]
//==================================================================================================
pub fn fill_twap_params(
    base_order: &mut Order,
    strategy_type: &str,
    start_time: &str,
    end_time: &str,
    allow_past_end_time: bool,
    monetary_value: f64,
) {
    base_order.algo_strategy = "Twap".to_string();

    base_order.algo_params.push(TagValue::new(
        "strategyType".to_string(),
        strategy_type.to_string(),
    ));
    base_order.algo_params.push(TagValue::new(
        "startTime".to_string(),
        start_time.to_string(),
    ));
    base_order
        .algo_params
        .push(TagValue::new("endTime".to_string(), end_time.to_string()));
    base_order.algo_params.push(TagValue::new(
        "allowPastEndTime".to_string(),
        (allow_past_end_time as i32).to_string(),
    ));
    base_order.algo_params.push(TagValue::new(
        "monetaryValue".to_string(),
        monetary_value.to_string(),
    ));
}

// ! [twap_params]

// ! [vwap_params]
//==================================================================================================
pub fn fill_vwap_params(
    base_order: &mut Order,
    max_pct_vol: f64,
    start_time: &str,
    end_time: &str,
    allow_past_end_time: bool,
    no_take_liq: bool,
    monetary_value: f64,
) {
    base_order.algo_strategy = "Vwap".to_string();

    base_order.algo_params.push(TagValue::new(
        "maxPctVol".to_string(),
        max_pct_vol.to_string(),
    ));
    base_order.algo_params.push(TagValue::new(
        "startTime".to_string(),
        start_time.to_string(),
    ));
    base_order
        .algo_params
        .push(TagValue::new("endTime".to_string(), end_time.to_string()));
    base_order.algo_params.push(TagValue::new(
        "allowPastEndTime".to_string(),
        (allow_past_end_time as i32).to_string(),
    ));
    base_order.algo_params.push(TagValue::new(
        "noTakeLiq".to_string(),
        (no_take_liq as i32).to_string(),
    ));
    base_order.algo_params.push(TagValue::new(
        "monetaryValue".to_string(),
        monetary_value.to_string(),
    ));
}

// ! [vwap_params]

// ! [ad_params]
//==================================================================================================
pub fn fill_accumulate_distribute_params(
    base_order: &mut Order,
    component_size: i32,
    time_between_orders: i32,
    randomize_time_20: bool,
    randomize_size_55: bool,
    give_up: i32,
    catch_up: bool,
    wait_for_fill: bool,
    start_time: &str,
    end_time: &str,
) {
    base_order.algo_strategy = "AD".to_string();

    base_order.algo_params.push(TagValue::new(
        "componentSize".to_string(),
        component_size.to_string(),
    ));
    base_order.algo_params.push(TagValue::new(
        "timeBetweenOrders".to_string(),
        time_between_orders.to_string(),
    ));
    base_order.algo_params.push(TagValue::new(
        "randomizeTime20".to_string(),
        (randomize_time_20 as i32).to_string(),
    ));
    base_order.algo_params.push(TagValue::new(
        "randomizeSize55".to_string(),
        (randomize_size_55 as i32).to_string(),
    ));
    base_order
        .algo_params
        .push(TagValue::new("giveUp".to_string(), give_up.to_string()));
    base_order.algo_params.push(TagValue::new(
        "catchUp".to_string(),
        (catch_up as i32).to_string(),
    ));
    base_order.algo_params.push(TagValue::new(
        "waitForFill".to_string(),
        (wait_for_fill as i32).to_string(),
    ));
    base_order.algo_params.push(TagValue::new(
        "activeTimeStart".to_string(),
        start_time.to_string(),
    ));
    base_order.algo_params.push(TagValue::new(
        "activeTimeEnd".to_string(),
        end_time.to_string(),
    ));
}

// ! [ad_params]

// ! [balanceimpactrisk_params]
//==================================================================================================
pub fn fill_balance_impact_risk_params(
    base_order: &mut Order,
    max_pct_vol: f64,
    risk_aversion: &str,
    force_completion: bool,
) {
    base_order.algo_strategy = "BalanceImpactRisk".to_string();

    base_order.algo_params.push(TagValue::new(
        "maxPctVol".to_string(),
        max_pct_vol.to_string(),
    ));
    base_order.algo_params.push(TagValue::new(
        "riskAversion".to_string(),
        risk_aversion.to_string(),
    ));
    base_order.algo_params.push(TagValue::new(
        "forceCompletion".to_string(),
        force_completion.to_string(),
    ));
}

// ! [balanceimpactrisk_params]

// ! [minimpact_params]
//==================================================================================================
pub fn fill_min_impact_params(base_order: &mut Order, max_pct_vol: f64) {
    base_order.algo_strategy = "MinImpact".to_string();

    base_order.algo_params.push(TagValue::new(
        "maxPctVol".to_string(),
        max_pct_vol.to_string(),
    ));
}

// ! [minimpact_params]

// ! [adaptive_params]
//==================================================================================================
pub fn fill_adaptive_params(base_order: &mut Order, priority: &str) {
    base_order.algo_strategy = "Adaptive".to_string();

    base_order.algo_params.push(TagValue::new(
        "adaptivePriority".to_string(),
        priority.to_string(),
    ));
}

// ! [adaptive_params]

// ! [closepx_params]
//==================================================================================================
pub fn fill_close_price_params(
    base_order: &mut Order,
    max_pct_vol: f64,
    risk_aversion: &str,
    start_time: &str,
    force_completion: bool,
    monetary_value: f64,
) {
    base_order.algo_strategy = "ClosePx".to_string();

    base_order.algo_params.push(TagValue::new(
        "maxPctVol".to_string(),
        max_pct_vol.to_string(),
    ));
    base_order.algo_params.push(TagValue::new(
        "riskAversion".to_string(),
        risk_aversion.to_string(),
    ));
    base_order.algo_params.push(TagValue::new(
        "startTime".to_string(),
        start_time.to_string(),
    ));
    base_order.algo_params.push(TagValue::new(
        "forceCompletion".to_string(),
        (force_completion as i32).to_string(),
    ));
    base_order.algo_params.push(TagValue::new(
        "monetaryValue".to_string(),
        monetary_value.to_string(),
    ));
}

// ! [closepx_params]

// ! [pctvolpx_params]
//==================================================================================================
pub fn fill_price_variant_pct_vol_params(
    base_order: &mut Order,
    pct_vol: f64,
    delta_pct_vol: f64,
    min_pct_vol_4px: f64,
    max_pct_vol_4px: f64,
    start_time: &str,
    end_time: &str,
    no_take_liq: bool,
    monetary_value: f64,
) {
    base_order.algo_strategy = "PctVolPx".to_string();

    base_order
        .algo_params
        .push(TagValue::new("pctVol".to_string(), pct_vol.to_string()));
    base_order.algo_params.push(TagValue::new(
        "deltaPctVol".to_string(),
        delta_pct_vol.to_string(),
    ));
    base_order.algo_params.push(TagValue::new(
        "minPctVol4Px".to_string(),
        min_pct_vol_4px.to_string(),
    ));
    base_order.algo_params.push(TagValue::new(
        "maxPctVol4Px".to_string(),
        max_pct_vol_4px.to_string(),
    ));
    base_order.algo_params.push(TagValue::new(
        "startTime".to_string(),
        start_time.to_string(),
    ));
    base_order
        .algo_params
        .push(TagValue::new("endTime".to_string(), end_time.to_string()));
    base_order.algo_params.push(TagValue::new(
        "noTakeLiq".to_string(),
        (no_take_liq as i32).to_string(),
    ));
    base_order.algo_params.push(TagValue::new(
        "monetaryValue".to_string(),
        monetary_value.to_string(),
    ));
}

// ! [pctvolpx_params]

// ! [pctvolsz_params]
//==================================================================================================
pub fn fill_size_variant_pct_vol_params(
    base_order: &mut Order,
    start_pct_vol: f64,
    end_pct_vol: f64,
    start_time: &str,
    end_time: &str,
    no_take_liq: bool,
    monetary_value: f64,
) {
    base_order.algo_strategy = "PctVolSz".to_string();

    base_order.algo_params.push(TagValue::new(
        "startPctVol".to_string(),
        start_pct_vol.to_string(),
    ));
    base_order.algo_params.push(TagValue::new(
        "endPctVol".to_string(),
        end_pct_vol.to_string(),
    ));
    base_order.algo_params.push(TagValue::new(
        "startTime".to_string(),
        start_time.to_string(),
    ));
    base_order
        .algo_params
        .push(TagValue::new("endTime".to_string(), end_time.to_string()));
    base_order.algo_params.push(TagValue::new(
        "noTakeLiq".to_string(),
        (no_take_liq as i32).to_string(),
    ));
    base_order.algo_params.push(TagValue::new(
        "monetaryValue".to_string(),
        monetary_value.to_string(),
    ));
}

// ! [pctvolsz_params]

// ! [pctvoltm_params]
//==================================================================================================
pub fn fill_time_variant_pct_vol_params(
    base_order: &mut Order,
    start_pct_vol: f64,
    end_pct_vol: f64,
    start_time: &str,
    end_time: &str,
    no_take_liq: bool,
    monetary_value: f64,
) {
    base_order.algo_strategy = "PctVolTm".to_string();

    base_order.algo_params.push(TagValue::new(
        "startPctVol".to_string(),
        start_pct_vol.to_string(),
    ));
    base_order.algo_params.push(TagValue::new(
        "endPctVol".to_string(),
        end_pct_vol.to_string(),
    ));
    base_order.algo_params.push(TagValue::new(
        "startTime".to_string(),
        start_time.to_string(),
    ));
    base_order
        .algo_params
        .push(TagValue::new("endTime".to_string(), end_time.to_string()));
    base_order.algo_params.push(TagValue::new(
        "noTakeLiq".to_string(),
        (no_take_liq as i32).to_string(),
    ));
    base_order.algo_params.push(TagValue::new(
        "monetary_value".to_string(),
        monetary_value.to_string(),
    ));
}
// ! [pctvoltm_params]

// ! [jefferies_vwap_params]
//==================================================================================================
pub fn fill_jefferies_vwapparams(
    base_order: &mut Order,
    start_time: &str,
    end_time: &str,
    relative_limit: f64,
    max_volume_rate: f64,
    exclude_auctions: &str,
    trigger_price: f64,
    wow_price: f64,
    min_fill_size: i32,
    wow_order_pct: f64,
    wow_mode: &str,
    is_buy_back: bool,
    wow_reference: &str,
) {
    // must be direct-routed to "JEFFALGO"
    base_order.algo_strategy = "VWAP".to_string();

    base_order.algo_params.push(TagValue::new(
        "StartTime".to_string(),
        start_time.to_string(),
    ));
    base_order
        .algo_params
        .push(TagValue::new("EndTime".to_string(), end_time.to_string()));
    base_order.algo_params.push(TagValue::new(
        "RelativeLimit".to_string(),
        relative_limit.to_string(),
    ));
    base_order.algo_params.push(TagValue::new(
        "MaxVolumeRate".to_string(),
        max_volume_rate.to_string(),
    ));
    base_order.algo_params.push(TagValue::new(
        "ExcludeAuctions".to_string(),
        exclude_auctions.to_string(),
    ));
    base_order.algo_params.push(TagValue::new(
        "TriggerPrice".to_string(),
        trigger_price.to_string(),
    ));
    base_order
        .algo_params
        .push(TagValue::new("WowPrice".to_string(), wow_price.to_string()));
    base_order.algo_params.push(TagValue::new(
        "MinFillSize".to_string(),
        min_fill_size.to_string(),
    ));
    base_order.algo_params.push(TagValue::new(
        "WowOrderPct".to_string(),
        wow_order_pct.to_string(),
    ));
    base_order
        .algo_params
        .push(TagValue::new("WowMode".to_string(), wow_mode.to_string()));
    base_order.algo_params.push(TagValue::new(
        "IsBuyBack".to_string(),
        (is_buy_back as i32).to_string(),
    ));
    base_order.algo_params.push(TagValue::new(
        "WowReference".to_string(),
        wow_reference.to_string(),
    ));
}
// ! [jefferies_vwap_params]

// ! [csfb_inline_params]
//==================================================================================================
pub fn fill_csfbinline_params(
    base_order: &mut Order,
    start_time: &str,
    end_time: &str,
    exec_style: &str,
    min_percent: i32,
    max_percent: i32,
    display_size: i32,
    auction: &str,
    block_finder: bool,
    block_price: f64,
    min_block_size: i32,
    max_block_size: i32,
    i_would_price: f64,
) {
    // must be direct-routed to "CSFBALGO"
    base_order.algo_strategy = "INLINE".to_string();

    base_order.algo_params.push(TagValue::new(
        "StartTime".to_string(),
        start_time.to_string(),
    ));
    base_order
        .algo_params
        .push(TagValue::new("EndTime".to_string(), end_time.to_string()));
    base_order.algo_params.push(TagValue::new(
        "ExecStyle".to_string(),
        exec_style.to_string(),
    ));
    base_order.algo_params.push(TagValue::new(
        "MinPercent".to_string(),
        min_percent.to_string(),
    ));
    base_order.algo_params.push(TagValue::new(
        "MaxPercent".to_string(),
        max_percent.to_string(),
    ));
    base_order.algo_params.push(TagValue::new(
        "DisplaySize".to_string(),
        display_size.to_string(),
    ));
    base_order
        .algo_params
        .push(TagValue::new("Auction".to_string(), auction.to_string()));
    base_order.algo_params.push(TagValue::new(
        "BlockFinder".to_string(),
        (block_finder as i32).to_string(),
    ));
    base_order.algo_params.push(TagValue::new(
        "BlockPrice".to_string(),
        block_price.to_string(),
    ));
    base_order.algo_params.push(TagValue::new(
        "MinBlockSize".to_string(),
        min_block_size.to_string(),
    ));
    base_order.algo_params.push(TagValue::new(
        "MaxBlockSize".to_string(),
        max_block_size.to_string(),
    ));
    base_order.algo_params.push(TagValue::new(
        "IWouldPrice".to_string(),
        i_would_price.to_string(),
    ));
}
// ! [csfb_inline_params]

// ! [qbalgo_strobe_params]
//==================================================================================================
pub fn fill_qbalgo_in_line_params(
    base_order: &mut Order,
    start_time: &str,
    end_time: &str,
    duration: f64,
    benchmark: &str,
    percent_volume: f64,
    no_clean_up: bool,
) {
    // must be direct-routed to "QBALGO"
    base_order.algo_strategy = "STROBE".to_string();

    base_order.algo_params.push(TagValue::new(
        "StartTime".to_string(),
        start_time.to_string(),
    ));
    base_order
        .algo_params
        .push(TagValue::new("EndTime".to_string(), end_time.to_string()));
    //This example uses end_time instead of duration
    //base_order.algo_params.push(TagValue::new("Duration".to_string(), str(duration.to_string())
    base_order.algo_params.push(TagValue::new(
        "Benchmark".to_string(),
        benchmark.to_string(),
    ));
    base_order.algo_params.push(TagValue::new(
        "PercentVolume".to_string(),
        percent_volume.to_string(),
    ));
    base_order.algo_params.push(TagValue::new(
        "NoCleanUp".to_string(),
        (no_clean_up as i32).to_string(),
    ));
}
// ! [qbalgo_strobe_params]
