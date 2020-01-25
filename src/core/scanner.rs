use std::fmt::Display;

use serde::export::fmt::Error;
use serde::export::Formatter;
use serde::{Deserialize, Serialize};

use crate::core::contract::ContractDetails;

//==================================================================================================

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct ScanData {
    pub contract: ContractDetails,
    pub rank: i32,
    pub distance: String,
    pub benchmark: String,
    pub projection: String,
    pub legs: String,
}

impl ScanData {
    pub fn new(
        contract: ContractDetails,
        rank: i32,
        distance: String,
        benchmark: String,
        projection: String,
        legs: String,
    ) -> Self {
        ScanData {
            contract,
            rank,
            distance,
            benchmark,
            projection,
            legs,
        }
    }
}

impl Display for ScanData {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "Rank: {}, Symbol: {}, SecType: {}, Currency: {}, Distance: {}, Benchmark: {}, Projection: {}, Legs: {}",
               self.rank, self.contract.contract.symbol, self.contract.contract.sec_type, self.contract.contract.currency,
               self.distance, self.benchmark, self.projection, self.legs)
    }
}

//==================================================================================================
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ScannerSubscription {
    pub number_of_rows: i32,
    pub instrument: String,
    pub location_code: String,
    pub scan_code: String,
    pub above_price: f64,
    pub below_price: f64,
    pub above_volume: i32,
    pub market_cap_above: f64,
    pub market_cap_below: f64,
    pub moody_rating_above: String,
    pub moody_rating_below: String,
    pub sp_rating_above: String,
    pub sp_rating_below: String,
    pub maturity_date_above: String,
    pub maturity_date_below: String,
    pub coupon_rate_above: f64,
    pub coupon_rate_below: f64,
    pub exclude_convertible: bool,
    pub average_option_volume_above: i32,
    pub scanner_setting_pairs: String,
    pub stock_type_filter: String,
}

impl ScannerSubscription {
    pub fn new(
        number_of_rows: i32,
        instrument: String,
        location_code: String,
        scan_code: String,
        above_price: f64,
        below_price: f64,
        above_volume: i32,
        market_cap_above: f64,
        market_cap_below: f64,
        moody_rating_above: String,
        moody_rating_below: String,
        sp_rating_above: String,
        sp_rating_below: String,
        maturity_date_above: String,
        maturity_date_below: String,
        coupon_rate_above: f64,
        coupon_rate_below: f64,
        exclude_convertible: bool,
        average_option_volume_above: i32,
        scanner_setting_pairs: String,
        stock_type_filter: String,
    ) -> Self {
        ScannerSubscription {
            number_of_rows,
            instrument,
            location_code,
            scan_code,
            above_price,
            below_price,
            above_volume,
            market_cap_above,
            market_cap_below,
            moody_rating_above,
            moody_rating_below,
            sp_rating_above,
            sp_rating_below,
            maturity_date_above,
            maturity_date_below,
            coupon_rate_above,
            coupon_rate_below,
            exclude_convertible,
            average_option_volume_above,
            scanner_setting_pairs,
            stock_type_filter,
        }
    }
}

impl Display for ScannerSubscription {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(
            f,
            "nstrument: {}, LocationCode: {}, ScanCode: {}",
            self.instrument, self.location_code, self.scan_code
        )
    }
}
