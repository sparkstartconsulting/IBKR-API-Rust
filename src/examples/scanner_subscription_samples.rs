use crate::core::scanner::ScannerSubscription;

pub fn hot_usstk_by_volume() -> ScannerSubscription {
    // ! [hotusvolume]
    //Hot US stocks by volume
    let mut scan_sub = ScannerSubscription::default();
    scan_sub.instrument = "STK".to_string();
    scan_sub.location_code = "STK.US.MAJOR".to_string();
    scan_sub.scan_code = "HOT_BY_VOLUME".to_string();
    // ! [hotusvolume]
    scan_sub
}

pub fn top_percent_gainers_ibis() -> ScannerSubscription {
    //! [toppercentgaineribis]
    // Top % gainers at IBIS
    let mut scan_sub = ScannerSubscription::default();
    scan_sub.instrument = "STOCK.EU".to_string();
    scan_sub.location_code = "STK.EU.IBIS".to_string();
    scan_sub.scan_code = "TOP_PERC_GAIN".to_string();
    // ! [toppercentgaineribis]
    scan_sub
}

pub fn most_active_fut_soffex() -> ScannerSubscription {
    // ! [mostactivefutsoffex]
    // Most active futures at SOFFEX
    let mut scan_sub = ScannerSubscription::default();
    scan_sub.instrument = "FUT.EU".to_string();
    scan_sub.location_code = "FUT.EU.SOFFEX".to_string();
    scan_sub.scan_code = "MOST_ACTIVE".to_string();
    // ! [mostactivefutsoffex]
    scan_sub
}

pub fn high_opt_volume_pcratio_usindexes() -> ScannerSubscription {
    // ! [highoptvolume]
    // High option volume P/C ratio US indexes
    let mut scan_sub = ScannerSubscription::default();
    scan_sub.instrument = "IND.US".to_string();
    scan_sub.location_code = "IND.US".to_string();
    scan_sub.scan_code = "HIGH_OPT_VOLUME_PUT_CALL_RATIO".to_string();
    // ! [highoptvolume]
    scan_sub
}

pub fn complex_orders_and_trades() -> ScannerSubscription {
    //! [combolatesttrade]
    // High option volume P/C ratio US indexes
    let mut scan_sub = ScannerSubscription::default();
    scan_sub.instrument = "NATCOMB".to_string();
    scan_sub.location_code = "NATCOMB.OPT.US".to_string();
    scan_sub.scan_code = "COMBO_LATEST_TRADE".to_string();
    // ! [combolatesttrade]
    scan_sub
}
