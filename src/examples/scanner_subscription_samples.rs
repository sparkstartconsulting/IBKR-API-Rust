//! Scanner subscription examples
use crate::core::scanner::ScannerSubscription;

///Hot US stocks by volume
pub fn hot_usstk_by_volume() -> ScannerSubscription {
    let mut scan_sub = ScannerSubscription::default();
    scan_sub.instrument = "STK".to_string();
    scan_sub.location_code = "STK.US.MAJOR".to_string();
    scan_sub.scan_code = "HOT_BY_VOLUME".to_string();
    scan_sub
}

/// Top % gainers at IBIS
pub fn top_percent_gainers_ibis() -> ScannerSubscription {
    let mut scan_sub = ScannerSubscription::default();
    scan_sub.instrument = "STOCK.EU".to_string();
    scan_sub.location_code = "STK.EU.IBIS".to_string();
    scan_sub.scan_code = "TOP_PERC_GAIN".to_string();
    scan_sub
}

/// Most active futures at SOFFEX
pub fn most_active_fut_soffex() -> ScannerSubscription {
    let mut scan_sub = ScannerSubscription::default();
    scan_sub.instrument = "FUT.EU".to_string();
    scan_sub.location_code = "FUT.EU.SOFFEX".to_string();
    scan_sub.scan_code = "MOST_ACTIVE".to_string();
    scan_sub
}

/// High option volume P/C ratio US indexes
pub fn high_opt_volume_pcratio_usindexes() -> ScannerSubscription {
    let mut scan_sub = ScannerSubscription::default();
    scan_sub.instrument = "IND.US".to_string();
    scan_sub.location_code = "IND.US".to_string();
    scan_sub.scan_code = "HIGH_OPT_VOLUME_PUT_CALL_RATIO".to_string();
    scan_sub
}

/// Combination order latest trade
pub fn complex_orders_and_trades() -> ScannerSubscription {
    let mut scan_sub = ScannerSubscription::default();
    scan_sub.instrument = "NATCOMB".to_string();
    scan_sub.location_code = "NATCOMB.OPT.US".to_string();
    scan_sub.scan_code = "COMBO_LATEST_TRADE".to_string();
    scan_sub
}
