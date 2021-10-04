//! Scanner subscription examples
use crate::core::scanner::ScannerSubscription;

///Hot US stocks by volume
pub fn hot_usstk_by_volume() -> ScannerSubscription {
    ScannerSubscription {
        instrument: "STK".to_string(),
        location_code: "STK.US.MAJOR".to_string(),
        scan_code: "HOT_BY_VOLUME".to_string(),
        ..Default::default()
    }
}

/// Top % gainers at IBIS
pub fn top_percent_gainers_ibis() -> ScannerSubscription {
    ScannerSubscription {
        instrument: "STOCK.EU".to_string(),
        location_code: "STK.EU.IBIS".to_string(),
        scan_code: "TOP_PERC_GAIN".to_string(),
        ..Default::default()
    }
}

/// Most active futures at SOFFEX
pub fn most_active_fut_soffex() -> ScannerSubscription {
    ScannerSubscription {
        instrument: "FUT.EU".to_string(),
        location_code: "FUT.EU.SOFFEX".to_string(),
        scan_code: "MOST_ACTIVE".to_string(),
        ..Default::default()
    }
}

/// High option volume P/C ratio US indexes
pub fn high_opt_volume_pcratio_usindexes() -> ScannerSubscription {
    ScannerSubscription {
        instrument: "IND.US".to_string(),
        location_code: "IND.US".to_string(),
        scan_code: "HIGH_OPT_VOLUME_PUT_CALL_RATIO".to_string(),
        ..Default::default()
    }
}

/// Combination order latest trade
pub fn complex_orders_and_trades() -> ScannerSubscription {
    ScannerSubscription {
        instrument: "NATCOMB".to_string(),
        location_code: "NATCOMB.OPT.US".to_string(),
        scan_code: "COMBO_LATEST_TRADE".to_string(),
        ..Default::default()
    }
}
