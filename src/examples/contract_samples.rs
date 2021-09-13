//! Examples of populating fields that define various types of contacts

use crate::core::contract::{ComboLeg, Contract, PositionType};

//==================================================================================================
pub fn eur_gbp_fx() -> Contract {
    Contract {
        symbol: "EUR".to_string(),
        sec_type: "CASH".to_string(),
        currency: "GBP".to_string(),
        exchange: "IDEALPRO".to_string(),
        ..Default::default()
    }
}

//==================================================================================================
pub fn index() -> Contract {
    Contract {
        symbol: "DAX".to_string(),
        sec_type: "IND".to_string(),
        currency: "EUR".to_string(),
        exchange: "DTB".to_string(),
        ..Default::default()
    }
}

//==================================================================================================
pub fn cfd() -> Contract {
    Contract {
        symbol: "IBDE30".to_string(),
        sec_type: "cfd".to_string(),
        currency: "EUR".to_string(),
        exchange: "SMART".to_string(),
        ..Default::default()
    }
}

//==================================================================================================
pub fn european_stock() -> Contract {
    Contract {
        symbol: "BMW".to_string(),
        sec_type: "STK".to_string(),
        currency: "EUR".to_string(),
        exchange: "SMART".to_string(),
        primary_exchange: "IBIS".to_string(),
        ..Default::default()
    }
}

//==================================================================================================
pub fn european_stock2() -> Contract {
    Contract {
        symbol: "NOKIA".to_string(),
        sec_type: "STK".to_string(),
        currency: "EUR".to_string(),
        exchange: "SMART".to_string(),
        primary_exchange: "HEX".to_string(),
        ..Default::default()
    }
}

//==================================================================================================
pub fn option_at_ise() -> Contract {
    Contract {
        symbol: "COF".to_string(),
        sec_type: "OPT".to_string(),
        currency: "USD".to_string(),
        exchange: "ISE".to_string(),
        last_trade_date_or_contract_month: "20190315".to_string(),
        right: "P".to_string(),
        strike: 105.0,
        multiplier: "100".to_string(),
        ..Default::default()
    }
}

//==================================================================================================
pub fn bond_with_cusip() -> Contract {
    Contract {
        // enter CUSIP as symbol
        symbol: "912828C57".to_string(),
        sec_type: "BOND".to_string(),
        exchange: "SMART".to_string(),
        currency: "USD".to_string(),
        ..Default::default()
    }
}

//==================================================================================================
pub fn bond() -> Contract {
    Contract {
        con_id: 15960357,
        exchange: "SMART".to_string(),
        ..Default::default()
    }
}

//==================================================================================================
pub fn mutual_fund() -> Contract {
    Contract {
        symbol: "VINIX".to_string(),
        sec_type: "FUND".to_string(),
        exchange: "FUNDSERV".to_string(),
        currency: "USD".to_string(),
        ..Default::default()
    }
}

//==================================================================================================
pub fn commodity() -> Contract {
    Contract {
        symbol: "XAUUSD".to_string(),
        sec_type: "CMDTY".to_string(),
        exchange: "SMART".to_string(),
        currency: "USD".to_string(),
        ..Default::default()
    }
}

//==================================================================================================
pub fn usstock() -> Contract {
    Contract {
        symbol: "AMZN".to_string(),
        sec_type: "STK".to_string(),
        currency: "USD".to_string(),
        //In the API side, NASDAQ is always defined as ISLAND in the exchange field
        exchange: "ISLAND".to_string(),
        //stkcontract]        ..Default::default()
        ..Default::default()
    }
}

//==================================================================================================
pub fn usstock_with_primary_exch() -> Contract {
    Contract {
        symbol: "MSFT".to_string(),
        sec_type: "STK".to_string(),
        currency: "USD".to_string(),
        exchange: "SMART".to_string(),
        //Specify the Primary Exchange attribute to avoid contract ambiguity
        //(there is an ambiguity because there is also a MSFT contract with primary exchange: "AEB")
        primary_exchange: "ISLAND".to_string(),
        //stkcontractwithprimary]        ..Default::default()
        ..Default::default()
    }
}

//==================================================================================================
pub fn us_stock_at_smart() -> Contract {
    Contract {
        symbol: "MSFT".to_string(),
        sec_type: "STK".to_string(),
        currency: "USD".to_string(),
        exchange: "SMART".to_string(),
        ..Default::default()
    }
}

//==================================================================================================
pub fn us_option_contract() -> Contract {
    Contract {
        symbol: "GOOG".to_string(),
        sec_type: "OPT".to_string(),
        exchange: "SMART".to_string(),
        currency: "USD".to_string(),
        last_trade_date_or_contract_month: "20201218".to_string(),
        strike: 1180.0,
        right: "C".to_string(),
        multiplier: "100".to_string(),
        ..Default::default()
    }
}

//==================================================================================================
pub fn option_at_box() -> Contract {
    Contract {
        symbol: "GOOG".to_string(),
        sec_type: "OPT".to_string(),
        exchange: "BOX".to_string(),
        currency: "USD".to_string(),
        last_trade_date_or_contract_month: "20201218".to_string(),
        strike: 1180.0,
        right: "C".to_string(),
        multiplier: "100".to_string(),
        ..Default::default()
    }
}

//==================================================================================================
/// Option contracts require far more information since there are many
/// contracts having the exact same attributes such as symbol, currency,
/// strike, etc. This can be overcome by adding more details such as the
//' trading class
pub fn option_with_trading_class() -> Contract {
    Contract {
        symbol: "SANT".to_string(),
        sec_type: "OPT".to_string(),
        exchange: "MEFFRV".to_string(),
        currency: "EUR".to_string(),
        last_trade_date_or_contract_month: "20190621".to_string(),
        strike: 7.5,
        right: "C".to_string(),
        multiplier: "100".to_string(),
        trading_class: "SANEU".to_string(),
        ..Default::default()
    }
}

//==================================================================================================
/// Using the contract's own symbol (local_symbol) can greatly simplify a
/// contract description. Watch out for the spaces within the local symbol!
pub fn option_with_local_symbol() -> Contract {
    Contract {
        //Watch out for the spaces within the local symbol!
        local_symbol: "C DBK  DEC 20  1600".to_string(),
        sec_type: "OPT".to_string(),
        exchange: "DTB".to_string(),
        currency: "EUR".to_string(),
        ..Default::default()
    }
}

//==================================================================================================
/// Dutch Warrants (IOPTs) can be defined using the local symbol or conid
pub fn dutch_warrant() -> Contract {
    Contract {
        local_symbol: "B881G".to_string(),
        sec_type: "IOPT".to_string(),
        exchange: "SBF".to_string(),
        currency: "EUR".to_string(),
        ..Default::default()
    }
}

//==================================================================================================
/// Future contracts also require an expiration date but are less
/// complicated than options.
pub fn simple_future() -> Contract {
    Contract {
        symbol: "ES".to_string(),
        sec_type: "FUT".to_string(),
        exchange: "GLOBEX".to_string(),
        currency: "USD".to_string(),
        last_trade_date_or_contract_month: "202009".to_string(),
        ..Default::default()
    }
}

//==================================================================================================
/// Rather than giving expiration dates we can also provide the local symbol
/// attributes such as symbol, currency, strike, etc.
pub fn future_with_local_symbol() -> Contract {
    Contract {
        sec_type: "FUT".to_string(),
        exchange: "GLOBEX".to_string(),
        currency: "USD".to_string(),
        local_symbol: "ESU0".to_string(),
        ..Default::default()
    }
}

//==================================================================================================
pub fn future_with_multiplier() -> Contract {
    Contract {
        symbol: "DAX".to_string(),
        sec_type: "FUT".to_string(),
        exchange: "DTB".to_string(),
        currency: "EUR".to_string(),
        last_trade_date_or_contract_month: "201903".to_string(),
        multiplier: "5".to_string(),
        ..Default::default()
    }
}

//==================================================================================================
/// Note the space in the symbol!
pub fn wrong_contract() -> Contract {
    Contract {
        symbol: " IJR ".to_string(),
        con_id: 9579976,
        sec_type: "STK".to_string(),
        exchange: "SMART".to_string(),
        currency: "USD".to_string(),
        ..Default::default()
    }
}

//==================================================================================================
pub fn futures_on_options() -> Contract {
    Contract {
        symbol: "ES".to_string(),
        sec_type: "FOP".to_string(),
        exchange: "GLOBEX".to_string(),
        currency: "USD".to_string(),
        last_trade_date_or_contract_month: "20190315".to_string(),
        strike: 2900.0,
        right: "C".to_string(),
        multiplier: "50".to_string(),
        ..Default::default()
    }
}

//==================================================================================================
/// It is also possible to deine contracts based on their ISIN (IBKR STK
/// sample).
pub fn by_isin() -> Contract {
    Contract {
        sec_id_type: "ISIN".to_string(),
        sec_id: "US45841N1072".to_string(),
        exchange: "SMART".to_string(),
        currency: "USD".to_string(),
        sec_type: "STK".to_string(),
        ..Default::default()
    }
}

//==================================================================================================
/// Or their con_id (EUR.uSD sample).
/// Note: passing a contract containing the con_id can cause problems if one of
/// the other provided attributes does not match 100% with what is in IB's
/// database. This is particularly important for contracts such as Bonds which
/// may change their description from one day to another.
/// If the con_id is provided, it is best not to give too much information as
/// in the example below.
pub fn by_con_id() -> Contract {
    Contract {
        sec_type: "CASH".to_string(),
        con_id: 12087792,
        exchange: "IDEALPRO".to_string(),
        ..Default::default()
    }
}

//==================================================================================================
/// Ambiguous contracts are great to use with
/// Contract::req_contract_details. This way
/// you can query the whole option chain for an underlying. Bear in mind that
/// there are pacing mechanisms in place which will delay any further responses
/// from the TWS to prevent abuse.
pub fn option_for_query() -> Contract {
    Contract {
        symbol: "FISV".to_string(),
        sec_type: "OPT".to_string(),
        exchange: "SMART".to_string(),
        currency: "USD".to_string(),
        ..Default::default()
    }
}

//==================================================================================================
pub fn option_combo_contract() -> Contract {
    Contract {
        symbol: "DBK".to_string(),
        sec_type: "BAG".to_string(),
        currency: "EUR".to_string(),
        exchange: "DTB".to_string(),
        combo_legs: vec![
            ComboLeg {
                con_id: 317960956, //DBK JUN 21 2019 C
                ratio: 1.0,
                action: "BUY".to_string(),
                exchange: "DTB".to_string(),
                ..Default::default()
            },
            ComboLeg {
                con_id: 334216780, //DBK MAR 15 2019 C
                ratio: 1.0,
                action: "SELL".to_string(),
                exchange: "DTB".to_string(),
                ..Default::default()
            },
        ],
        //bagoptcontract]        ..Default::default()
        ..Default::default()
    }
}

//==================================================================================================
/// STK Combo contract
/// Leg 1: 43645865 - IBKR's STK
/// Leg 2: 9408 - McDonald's STK
pub fn stock_combo_contract() -> Contract {
    Contract {
        symbol: "IBKR,MCD".to_string(),
        sec_type: "BAG".to_string(),
        currency: "USD".to_string(),
        exchange: "SMART".to_string(),
        combo_legs: vec![
            ComboLeg {
                con_id: 43645865, //IBKR STK
                ratio: 1.0,
                action: "BUY".to_string(),
                exchange: "SMART".to_string(),
                ..Default::default()
            },
            ComboLeg {
                con_id: 9408, //MCD STK
                ratio: 1.0,
                action: "SELL".to_string(),
                exchange: "SMART".to_string(),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

//==================================================================================================
/// CBOE volatility Index Future combo contract
pub fn future_combo_contract() -> Contract {
    Contract {
        symbol: "VIX".to_string(),
        sec_type: "BAG".to_string(),
        currency: "USD".to_string(),
        exchange: "CFE".to_string(),
        combo_legs: vec![
            ComboLeg {
                con_id: 438391466, // VIX FUT 201903
                ratio: 1.0,
                action: "BUY".to_string(),
                exchange: "CFE".to_string(),
                exempt_code: -1,
                open_close: PositionType::SamePos,
                ..Default::default()
            },
            ComboLeg {
                con_id: 394987014, // VIX FUT 201904
                ratio: 1.0,
                action: "SELL".to_string(),
                exchange: "CFE".to_string(),
                exempt_code: -1,
                open_close: PositionType::SamePos,
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

//==================================================================================================
pub fn smart_future_combo_contract() -> Contract {
    Contract {
        symbol: "WTI".to_string(), // WTI,COIL spread. Symbol can be defined as first leg symbol ("WTI") or currency ("USD")
        sec_type: "BAG".to_string(),
        currency: "USD".to_string(),
        exchange: "SMART".to_string(),
        combo_legs: vec![
            ComboLeg {
                con_id: 55928698, // WTI future June 2017
                ratio: 1.0,
                action: "BUY".to_string(),
                exchange: "IPE".to_string(),
                ..Default::default()
            },
            ComboLeg {
                con_id: 55850663, // COIL future June 2017
                ratio: 1.0,
                action: "SELL".to_string(),
                exchange: "IPE".to_string(),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

//==================================================================================================
pub fn inter_cmdty_futures_contract() -> Contract {
    Contract {
        symbol: "CL.BZ".to_string(), //symbol is 'local symbol' of intercommodity spread.
        sec_type: "BAG".to_string(),
        currency: "USD".to_string(),
        exchange: "NYMEX".to_string(),
        combo_legs: vec![
            ComboLeg {
                con_id: 47207310, //CL Dec'16 @NYMEX
                ratio: 1.0,
                action: "BUY".to_string(),
                exchange: "NYMEX".to_string(),
                ..Default::default()
            },
            ComboLeg {
                con_id: 47195961, //BZ Dec'16 @NYMEX
                ratio: 1.0,
                action: "SELL".to_string(),
                exchange: "NYMEX".to_string(),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

//==================================================================================================
pub fn news_feed_for_query() -> Contract {
    Contract {
        sec_type: "NEWS".to_string(),
        exchange: "BRFG".to_string(), //Briefing Trader
        ..Default::default()
    }
}

//==================================================================================================
pub fn brfgbroadtape_news_feed() -> Contract {
    Contract {
        symbol: "BRFG:BRFG_ALL".to_string(),
        sec_type: "NEWS".to_string(),
        exchange: "BRFG".to_string(),
        ..Default::default()
    }
}

//==================================================================================================
pub fn djnlbroadtape_news_feed() -> Contract {
    Contract {
        symbol: "DJNL:DJNL_ALL".to_string(),
        sec_type: "NEWS".to_string(),
        exchange: "DJNL".to_string(),
        ..Default::default()
    }
}

//==================================================================================================
pub fn djtopbroadtape_news_feed() -> Contract {
    Contract {
        symbol: "DJTOP:ASIAPAC".to_string(),
        sec_type: "NEWS".to_string(),
        exchange: "DJTOP".to_string(),
        ..Default::default()
    }
}

//==================================================================================================
pub fn brfupdnbroadtape_news_feed() -> Contract {
    Contract {
        symbol: "BRFUPDN:BRF_ALL".to_string(),
        sec_type: "NEWS".to_string(),
        exchange: "BRFUPDN".to_string(),
        ..Default::default()
    }
}

//==================================================================================================
pub fn cont_fut() -> Contract {
    Contract {
        symbol: "ES".to_string(),
        sec_type: "CONTFUT".to_string(),
        exchange: "GLOBEX".to_string(),
        ..Default::default()
    }
}

//==================================================================================================
pub fn cont_and_expiring_fut() -> Contract {
    Contract {
        symbol: "ES".to_string(),
        sec_type: "FUT+CONTFUT".to_string(),
        exchange: "GLOBEX".to_string(),
        ..Default::default()
    }
}

//==================================================================================================
pub fn jefferies_contract() -> Contract {
    Contract {
        symbol: "AAPL".to_string(),
        sec_type: "STK".to_string(),
        exchange: "JEFFALGO".to_string(),
        currency: "USD".to_string(),
        ..Default::default()
    }
}

//==================================================================================================
pub fn csfbcontract() -> Contract {
    Contract {
        symbol: "IBKR".to_string(),
        sec_type: "STK".to_string(),
        exchange: "CSFBALGO".to_string(),
        currency: "USD".to_string(),
        ..Default::default()
    }
}

//==================================================================================================
pub fn usstock_cfd() -> Contract {
    Contract {
        symbol: "IBM".to_string(),
        sec_type: "cfd".to_string(),
        currency: "USD".to_string(),
        exchange: "SMART".to_string(),
        ..Default::default()
    }
}

//==================================================================================================
pub fn european_stock_cfd() -> Contract {
    Contract {
        symbol: "BMW".to_string(),
        sec_type: "cfd".to_string(),
        currency: "EUR".to_string(),
        exchange: "SMART".to_string(),
        ..Default::default()
    }
}

//==================================================================================================
pub fn cash_cfd() -> Contract {
    Contract {
        symbol: "EUR".to_string(),
        sec_type: "cfd".to_string(),
        currency: "USD".to_string(),
        exchange: "SMART".to_string(),
        ..Default::default()
    }
}

//==================================================================================================
pub fn qbalgo_contract() -> Contract {
    Contract {
        symbol: "ES".to_string(),
        sec_type: "FUT".to_string(),
        exchange: "QBALGO".to_string(),
        currency: "USD".to_string(),
        last_trade_date_or_contract_month: "202009".to_string(),
        ..Default::default()
    }
}
