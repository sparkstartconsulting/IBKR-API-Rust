#![allow(clippy::too_many_arguments)]
//! Examples of populating fields of various order types
use num_traits::FromPrimitive;

use crate::core::common::TagValue;
use crate::core::order::{AuctionStrategy, Order, OrderComboLeg};
use crate::core::order_condition::{
    create_condition, ConditionType, ExecutionCondition, MarginCondition, PercentChangeCondition,
    PriceCondition, TimeCondition, VolumeCondition,
};

/// An auction order is entered into the electronic trading system during the pre-market opening period for execution at the
/// Calculated Opening Price (COP). If your order is not filled on the open, the order is re-submitted as a limit order with
/// the limit price set to the COP or the best bid/ask after the market opens.
/// Products: FUT, STK *///
//==================================================================================================
pub fn at_auction(action: &str, quantity: f64, price: f64) -> Order {
    Order {
        action: action.to_string(),
        tif: "AUC".to_string(),
        order_type: "MTL".to_string(),
        total_quantity: quantity,
        lmt_price: price,
        ..Default::default()
    }
}

//==================================================================================================
/// A discretionary order is a limit order submitted with a hidden, specified 'discretionary' amount off the limit price which
/// may be used to increase the price range over which the limit order is eligible to execute. The market sees only the limit price.
/// Products: STK
pub fn discretionary(action: &str, quantity: f64, price: f64, discretionary_amount: f64) -> Order {
    Order {
        action: action.to_string(),
        order_type: "LMT".to_string(),
        total_quantity: quantity,
        lmt_price: price,
        discretionary_amt: discretionary_amount,
        ..Default::default()
    }
}

//==================================================================================================
/// A Market order is an order to buy or sell at the market bid or offer price. A market order may increase the likelihood of a fill
/// and the speed of execution, but unlike the Limit order a Market order provides no price protection and may fill at a price far
/// lower/higher than the current displayed bid/ask.
/// Products: BOND, CFD, EFP, CASH, FUND, FUT, FOP, OPT, STK, WAR
pub fn market_order(action: &str, quantity: f64) -> Order {
    Order {
        action: action.to_string(),
        order_type: "MKT".to_string(),
        total_quantity: quantity,
        ..Default::default()
    }
}

//==================================================================================================
/// A Market if Touched (MIT) is an order to buy (or sell) a contract below (or above) the market. Its purpose is to take advantage
/// of sudden or unexpected changes in share or other prices and provides investors with a trigger price to set an order in motion.
/// Investors may be waiting for excessive strength (or weakness) to cease, which might be represented by a specific price point.
/// MIT orders can be used to determine whether or not to enter the market once a specific price level has been achieved. This order
/// is held in the system until the trigger price is touched, and is then submitted as a market  An MIT order is similar to a
/// stop order, except that an MIT sell order is placed above the current market price, and a stop sell order is placed below
/// Products: BOND, CFD, CASH, FUT, FOP, OPT, STK, WAR
pub fn market_if_touched(action: &str, quantity: f64, price: f64) -> Order {
    Order {
        action: action.to_string(),
        order_type: "MIT".to_string(),
        total_quantity: quantity,
        aux_price: price,
        ..Default::default()
    }
}

//==================================================================================================
/// A Market-on-Close (MOC) order is a market order that is submitted to execute as close to the closing price as possible.
/// Products: CFD, FUT, STK, WAR
pub fn market_on_close(action: &str, quantity: f64) -> Order {
    Order {
        action: action.to_string(),
        order_type: "MOC".to_string(),
        total_quantity: quantity,
        ..Default::default()
    }
}
//==================================================================================================
/// A Market-on-Open (MOO) order combines a market order with the OPG time in force to create an order that is automatically
/// submitted at the market's open and fills at the market price.
/// Products: CFD, STK, OPT, WAR
pub fn market_on_open(action: &str, quantity: f64) -> Order {
    Order {
        action: action.to_string(),
        order_type: "MKT".to_string(),
        total_quantity: quantity,
        tif: "OPG".to_string(),
        ..Default::default()
    }
}

//==================================================================================================
/// ISE MidpoMatch:i32 (MPM) orders always execute at the midpoof:the:i32 NBBO. You can submit market and limit orders direct-routed
/// to ISE for MPM execution. Market orders execute at the midpowhenever:an:i32 eligible contra-order is available. Limit orders
/// execute only when the midpoprice:is:i32 better than the limit price. Standard MPM orders are completely anonymous.
/// Products: STK
pub fn midpoint_match(action: &str, quantity: f64) -> Order {
    Order {
        action: action.to_string(),
        order_type: "MKT".to_string(),
        total_quantity: quantity,
        ..Default::default()
    }
}

//==================================================================================================
/// A midprice order is designed to split the difference between the bid and ask prices, and fill at the current midpoint of
/// the NBBO or better. Set an optional price cap to define the highest price (for a buy order) or the lowest price (for a sell
/// order) you are willing to accept. Requires TWS 975+. Smart-routing to US stocks only.
pub fn midprice(action: &str, quantity: f64, price_cap: f64) -> Order {
    Order {
        action: action.to_string(),
        order_type: "MIDPRICE".to_string(),
        total_quantity: quantity,
        lmt_price: price_cap, // optional
        //midprice]
        ..Default::default()
    }
}

//==================================================================================================
/// A pegged-to-market order is designed to maintain a purchase price relative to the national best offer (NBO) or a sale price
/// relative to the national best bid (NBB). Depending on the width of the quote, this order may be passive or aggressive.
/// The trader creates the order by entering a limit price which defines the worst limit price that they are willing to accept.
/// Next, the trader enters an offset amount which computes the active limit price as follows:
///     Sell order price: Bid price + offset amount
///     Buy order price: Ask price - offset amount
/// Products: STK
pub fn pegged_to_market(action: &str, quantity: f64, market_offset: f64) -> Order {
    //pegged_market]
    Order {
        action: action.to_string(),
        order_type: "PEG MKT".to_string(),
        total_quantity: quantity,
        aux_price: market_offset, //Offset price
        //pegged_market]
        ..Default::default()
    }
}

//==================================================================================================
/// A Pegged to Stock order continually adjusts the option order price by the product of a signed user-define delta and the change of
/// the option's underlying stock price. The delta is entered as an absolute and assumed to be positive for calls and negative for puts.
/// A buy or sell call order price is determined by adding the delta times a change in an underlying stock price to a specified starting
/// price for the call. To determine the change in price, the stock reference price is subtracted from the current NBBO midpoint.
/// The Stock Reference Price can be defined by the user, or defaults to the NBBO midpoat:the:i32 time of the order if no reference price
/// is entered. You may also enter a high/low stock price range which cancels the order when reached. The delta times the change in stock
/// price will be rounded to the nearest penny in favor of the
/// Products: OPT
pub fn pegged_to_stock(
    action: &str,
    quantity: f64,
    delta: f64,
    stock_reference_price: f64,
    starting_price: f64,
) -> Order {
    //pegged_stock]
    Order {
        action: action.to_string(),
        order_type: "PEG STK".to_string(),
        total_quantity: quantity,
        delta,
        stock_ref_price: stock_reference_price,
        starting_price,
        //pegged_stock]
        ..Default::default()
    }
}

/// Relative (a.k.a. Pegged-to-Primary) orders provide a means for traders to seek a more aggressive price than the National Best Bid
/// and Offer (NBBO). By acting as liquidity providers, and placing more aggressive bids and offers than the current best bids and offers,
/// traders increase their odds of filling their  Quotes are automatically adjusted as the markets move, to remain aggressive.
/// For a buy order, your bid is pegged to the NBB by a more aggressive offset, and if the NBB moves up, your bid will also move up.
/// If the NBB moves down, there will be no adjustment because your bid will become even more aggressive and execute. For sales, your
/// offer is pegged to the NBO by a more aggressive offset, and if the NBO moves down, your offer will also move down. If the NBO moves up,
/// there will be no adjustment because your offer will become more aggressive and execute. In addition to the offset, you can define an
/// absolute cap, which works like a limit price, and will prevent your order from being executed above or below a specified level.
/// Stocks, Options and Futures - not available on paper trading
/// Products: CFD, STK, OPT, FUT
//==================================================================================================
pub fn relative_pegged_to_primary(
    action: &str,
    quantity: f64,
    price_cap: f64,
    offset_amount: f64,
) -> Order {
    //relative_pegged_primary]
    Order {
        action: action.to_string(),
        order_type: "REL".to_string(),
        total_quantity: quantity,
        lmt_price: price_cap,
        aux_price: offset_amount,
        //relative_pegged_primary]
        ..Default::default()
    }
}

/// Sweep-to-fill orders are useful when a trader values speed of execution over price. A sweep-to-fill order identifies the best price
/// and the exact quantity offered/available at that price, and transmits the corresponding portion of your order for immediate execution.
/// Simultaneously it identifies the next best price and quantity offered/available, and submits the matching quantity of your order for
/// immediate execution.
/// Products: CFD, STK, WAR
//==================================================================================================
pub fn sweep_to_fill(action: &str, quantity: f64, price: f64) -> Order {
    //sweep_to_fill]
    Order {
        action: action.to_string(),
        order_type: "LMT".to_string(),
        total_quantity: quantity,
        lmt_price: price,
        sweep_to_fill: true,
        //sweep_to_fill]
        ..Default::default()
    }
}

/// For option orders routed to the Boston Options Exchange (BOX) you may elect to participate in the BOX's price improvement auction in
/// pennies. All BOX-directed price improvement orders are immediately sent from Interactive Brokers to the BOX order book, and when the
/// terms allow, IB will evaluate it for inclusion in a price improvement auction based on price and volume priority. In the auction, your
/// order will have priority over broker-dealer price improvement orders at the same price.
/// An Auction Limit order at a specified price. Use of a limit order ensures that you will not receive an execution at a price less favorable
/// than the limit price. Enter limit orders in penny increments with your auction improvement amount computed as the difference between your
/// limit order price and the nearest listed increment.
/// Products: OPT
/// Supported Exchanges: BOX
//==================================================================================================
pub fn auction_limit(
    action: &str,
    quantity: f64,
    price: f64,
    auction_strategy: AuctionStrategy,
) -> Order {
    //auction_limit]
    Order {
        action: action.to_string(),
        order_type: "LMT".to_string(),
        total_quantity: quantity,
        lmt_price: price,
        auction_strategy,
        //auction_limit]
        ..Default::default()
    }
}

/// For option orders routed to the Boston Options Exchange (BOX) you may elect to participate in the BOX's price improvement auction in pennies.
/// All BOX-directed price improvement orders are immediately sent from Interactive Brokers to the BOX order book, and when the terms allow,
/// IB will evaluate it for inclusion in a price improvement auction based on price and volume priority. In the auction, your order will have
/// priority over broker-dealer price improvement orders at the same price.
/// An Auction Pegged to Stock order adjusts the order price by the product of a signed delta (which is entered as an absolute and assumed to be
/// positive for calls, negative for puts) and the change of the option's underlying stock price. A buy or sell call order price is determined
/// by adding the delta times a change in an underlying stock price change to a specified starting price for the call. To determine the change
/// in price, a stock reference price (NBBO midpoat:the:i32 time of the order is assumed if no reference price is entered) is subtracted from
/// the current NBBO midpoint. A stock range may also be entered that cancels an order when reached. The delta times the change in stock price
/// will be rounded to the nearest penny in favor of the order and will be used as your auction improvement amount.
/// Products: OPT
/// Supported Exchanges: BOX
//==================================================================================================
pub fn auction_pegged_to_stock(
    action: &str,
    quantity: f64,
    starting_price: f64,
    delta: f64,
) -> Order {
    //auction_pegged_stock]
    Order {
        action: action.to_string(),
        order_type: "PEG STK".to_string(),
        total_quantity: quantity,
        delta,
        starting_price,
        //auction_pegged_stock]
        ..Default::default()
    }
}

/// For option orders routed to the Boston Options Exchange (BOX) you may elect to participate in the BOX's price improvement auction in pennies.
/// All BOX-directed price improvement orders are immediately sent from Interactive Brokers to the BOX order book, and when the terms allow,
/// IB will evaluate it for inclusion in a price improvement auction based on price and volume priority. In the auction, your order will have
/// priority over broker-dealer price improvement orders at the same price.
/// An Auction Relative order that adjusts the order price by the product of a signed delta (which is entered as an absolute and assumed to be
/// positive for calls, negative for puts) and the change of the option's underlying stock price. A buy or sell call order price is determined
/// by adding the delta times a change in an underlying stock price change to a specified starting price for the call. To determine the change
/// in price, a stock reference price (NBBO midpoat:the:i32 time of the order is assumed if no reference price is entered) is subtracted from
/// the current NBBO midpoint. A stock range may also be entered that cancels an order when reached. The delta times the change in stock price
/// will be rounded to the nearest penny in favor of the order and will be used as your auction improvement amount.
/// Products: OPT
/// Supported Exchanges: BOX
//==================================================================================================
pub fn auction_relative(action: &str, quantity: f64, offset: f64) -> Order {
    //auction_relative]
    Order {
        action: action.to_string(),
        order_type: "REL".to_string(),
        total_quantity: quantity,
        aux_price: offset,
        //auction_relative]
        ..Default::default()
    }
}

/// The block attribute is used for large volume option orders on ISE that consist of at least 50 contracts. To execute large-volume
/// orders over time without moving the market, use the Accumulate/Distribute algorithm.
/// Products: OPT
//==================================================================================================
pub fn block(action: &str, quantity: f64, price: f64) -> Order {
    //block]
    Order {
        action: action.to_string(),
        order_type: "LMT".to_string(),
        total_quantity: quantity, //Large volumes!
        lmt_price: price,
        block_order: true,
        //block]
        ..Default::default()
    }
}

/// A Box Top order executes as a market order at the current best price. If the order is only partially filled, the remainder is submitted as
/// a limit order with the limit price equal to the price at which the filled portion of the order executed.
/// Products: OPT
/// Supported Exchanges: BOX
//==================================================================================================
pub fn box_top(action: &str, quantity: f64) -> Order {
    //boxtop]
    Order {
        action: action.to_string(),
        order_type: "BOX TOP".to_string(),
        total_quantity: quantity,
        //boxtop]
        ..Default::default()
    }
}

/// A Limit order is an order to buy or sell at a specified price or better. The Limit order ensures that if the order fills,
/// it will not fill at a price less favorable than your limit price, but it does not guarantee a fill.
/// Products: BOND, CFD, CASH, FUT, FOP, OPT, STK, WAR
//==================================================================================================
pub fn limit_order(action: &str, quantity: f64, limit_price: f64) -> Order {
    //limitorder]
    Order {
        action: action.to_string(),
        order_type: "LMT".to_string(),
        total_quantity: quantity,
        lmt_price: limit_price,
        transmit: true,
        //limitorder]
        ..Default::default()
    }
}

/// Forex orders can be placed in demonination of second currency in pair using cashQty field
/// Requires TWS or IBG 963+
/// <https://www.interactivebrokers.com/en/index.php?f=23876#963-02>
//==================================================================================================
pub fn limit_order_with_cash_qty(
    action: &str,
    quantity: f64,
    limit_price: f64,
    cash_qty: f64,
) -> Order {
    Order {
        action: action.to_string(),
        order_type: "LMT".to_string(),
        total_quantity: quantity,
        lmt_price: limit_price,
        cash_qty,
        ..Default::default()
    }
}

/// A Limit if Touched is an order to buy (or sell) a contract at a specified price or better, below (or above) the market. This order is
/// held in the system until the trigger price is touched. An LIT order is similar to a stop limit order, except that an LIT sell order is
/// placed above the current market price, and a stop limit sell order is placed below.
/// Products: BOND, CFD, CASH, FUT, FOP, OPT, STK, WAR
//==================================================================================================
pub fn limit_if_touched(
    action: &str,
    quantity: f64,
    limit_price: f64,
    trigger_price: f64,
) -> Order {
    //limitiftouched]
    Order {
        action: action.to_string(),
        order_type: "LIT".to_string(),
        total_quantity: quantity,
        lmt_price: limit_price,
        aux_price: trigger_price,
        //limitiftouched]
        ..Default::default()
    }
}

/// A Limit-on-close (LOC) order will be submitted at the close and will execute if the closing price is at or better than the submitted
/// limit price.
/// Products: CFD, FUT, STK, WAR
//==================================================================================================
pub fn limit_on_close(action: &str, quantity: f64, limit_price: f64) -> Order {
    Order {
        action: action.to_string(),
        order_type: "LOC".to_string(),
        total_quantity: quantity,
        lmt_price: limit_price,
        ..Default::default()
    }
}

/// A Limit-on-Open (LOO) order combines a limit order with the OPG time in force to create an order that is submitted at the market's open,
/// and that will only execute at the specified limit price or better. Orders are filled in accordance with specific exchange rules.
/// Products: CFD, STK, OPT, WAR
//==================================================================================================
pub fn limit_on_open(action: &str, quantity: f64, limit_price: f64) -> Order {
    Order {
        action: action.to_string(),
        tif: "OPG".to_string(),
        order_type: "LMT".to_string(),
        total_quantity: quantity,
        lmt_price: limit_price,
        ..Default::default()
    }
}

/// Passive Relative orders provide a means for traders to seek a less aggressive price than the National Best Bid and Offer (NBBO) while
/// keeping the order pegged to the best bid (for a buy) or ask (for a sell). The order price is automatically adjusted as the markets move
/// to keep the order less aggressive. For a buy order, your order price is pegged to the NBB by a less aggressive offset, and if the NBB
/// moves up, your bid will also move up. If the NBB moves down, there will be no adjustment because your bid will become aggressive and execute.
/// For a sell order, your price is pegged to the NBO by a less aggressive offset, and if the NBO moves down, your offer will also move down.
/// If the NBO moves up, there will be no adjustment because your offer will become aggressive and execute. In addition to the offset, you can
/// define an absolute cap, which works like a limit price, and will prevent your order from being executed above or below a specified level.
/// The Passive Relative order is similar to the Relative/Pegged-to-Primary order, except that the Passive relative subtracts the offset from
/// the bid and the Relative adds the offset to the bid.
/// Products: STK, WAR
//==================================================================================================
pub fn passive_relative(action: &str, quantity: f64, offset: f64) -> Order {
    Order {
        action: action.to_string(),
        order_type: "PASSV REL".to_string(),
        total_quantity: quantity,
        aux_price: offset,
        ..Default::default()
    }
}

/// A pegged-to-midpoorder:provides:i32 a means for traders to seek a price at the midpoof:the:i32 National Best Bid and Offer (NBBO).
/// The price automatically adjusts to peg the midpoas:the:i32 markets move, to remain aggressive. For a buy order, your bid is pegged to
/// the NBBO midpoand:the:i32 order price adjusts automatically to continue to peg the midpoif:the:i32 market moves. The price only adjusts
/// to be more aggressive. If the market moves in the opposite direction, the order will execute.
/// Products: STK
//==================================================================================================
pub fn pegged_to_midpoint(action: &str, quantity: f64, offset: f64, limit_price: f64) -> Order {
    //pegged_midpoint]
    Order {
        action: action.to_string(),
        order_type: "PEG MID".to_string(),
        total_quantity: quantity,
        aux_price: offset,
        lmt_price: limit_price,
        //pegged_midpoint]
        ..Default::default()
    }
}

/// Bracket orders are designed to help limit your loss and lock in a profit by "bracketing" an order with two opposite-side orders.
/// A BUY order is bracketed by a high-side sell limit order and a low-side sell stop  A SELL order is bracketed by a high-side buy
/// stop order and a low side buy limit
/// Products: CFD, BAG, FOP, CASH, FUT, OPT, STK, WAR
//==================================================================================================
pub fn bracket_order(
    parent_order_id: i32,
    action: &str,
    quantity: f64,
    limit_price: f64,
    take_profit_limit_price: f64,
    stop_loss_price: f64,
) -> (Order, Order, Order) {
    // This will be our main or "parent" ..Default::default()

    let parent = Order {
        order_id: parent_order_id,
        action: action.to_string(),
        order_type: "LMT".to_string(),
        total_quantity: quantity,
        lmt_price: limit_price,
        // The parent and children orders will need this attribute set to False to prevent accidental executions.
        // The LAST CHILD will have it set to True,
        transmit: false,
        ..Default::default()
    };

    let take_profit = Order {
        order_id: parent.order_id + 1,
        action: (if action == "BUY" { "SELL" } else { "BUY" }).to_string(),
        order_type: "LMT".to_string(),
        total_quantity: quantity,
        lmt_price: take_profit_limit_price,
        parent_id: parent_order_id,
        transmit: false,
        ..Default::default()
    };

    let stop_loss = Order {
        order_id: parent.order_id + 2,
        action: (if action == "BUY" { "SELL" } else { "BUY" }).to_string(),
        order_type: "STP".to_string(),
        // stop trigger price
        aux_price: stop_loss_price,
        total_quantity: quantity,
        parent_id: parent_order_id,
        // In this case, the low side order will be the last child being sent. Therefore, it needs to set this attribute to True
        // to activate all its predecessors
        transmit: true,
        ..Default::default()
    };

    (parent, take_profit, stop_loss)
}

/// Products:CFD, FUT, FOP, OPT, STK, WAR
/// A Market-to-Limit (MTL) order is submitted as a market order to execute at the current best market price. If the order is only
/// partially filled, the remainder of the order is canceled and re-submitted as a limit order with the limit price equal to the price
/// at which the filled portion of the order executed.
//==================================================================================================
pub fn market_to_limit(action: &str, quantity: f64) -> Order {
    Order {
        action: action.to_string(),
        order_type: "MTL".to_string(),
        total_quantity: quantity,
        ..Default::default()
    }
}

/// This order type is useful for futures traders using Globex. A Market with Protection order is a market order that will be cancelled and
/// resubmitted as a limit order if the entire order does not immediately execute at the market price. The limit price is set by Globex to be
/// close to the current market price, slightly higher for a sell order and lower for a buy
/// Products: FUT, FOP
//==================================================================================================
pub fn market_with_protection(action: &str, quantity: f64) -> Order {
    Order {
        action: action.to_string(),
        order_type: "MKT PRT".to_string(),
        total_quantity: quantity,
        ..Default::default()
    }
}

/// A stop order is an instruction to submit a buy or sell market order if and when the user-specified stop trigger price is attained or
/// penetrated. A stop order is not guaranteed a specific execution price and may execute significantly away from its stop price. A Sell
/// stop order is always placed below the current market price and is typically used to limit a loss or protect a profit on a long stock
/// position. A Buy stop order is always placed above the current market price. It is typically used to limit a loss or help protect a
/// profit on a short sale.
/// Products: CFD, BAG, CASH, FUT, FOP, OPT, STK, WAR
//==================================================================================================
pub fn stop(action: &str, quantity: f64, stop_price: f64) -> Order {
    Order {
        action: action.to_string(),
        order_type: "STP".to_string(),
        aux_price: stop_price,
        total_quantity: quantity,
        ..Default::default()
    }
}

/// A stop-Limit order is an instruction to submit a buy or sell limit order when the user-specified stop trigger price is attained or
/// penetrated. The order has two basic components: the stop price and the limit price. When a trade has occurred at or through the stop
/// price, the order becomes executable and enters the market as a limit order, which is an order to buy or sell at a specified price or better.
/// Products: CFD, CASH, FUT, FOP, OPT, STK, WAR
//==================================================================================================
pub fn stop_limit(action: &str, quantity: f64, limit_price: f64, stop_price: f64) -> Order {
    Order {
        action: action.to_string(),
        order_type: "STP LMT".to_string(),
        total_quantity: quantity,
        lmt_price: limit_price,
        aux_price: stop_price,
        ..Default::default()
    }
}

/// A stop with Protection order combines the functionality of a stop limit order with a market with protection  The order is set
/// to trigger at a specified stop price. When the stop price is penetrated, the order is triggered as a market with protection order,
/// which means that it will fill within a specified protected price range equal to the trigger price +/- the exchange-defined protection
/// porange:i32. Any portion of the order that does not fill within this protected range is submitted as a limit order at the exchange-defined
/// trigger price +/- the protection points.
/// Products: FUT
//==================================================================================================
pub fn stop_with_protection(action: &str, quantity: f64, stop_price: f64) -> Order {
    Order {
        total_quantity: quantity,
        action: action.to_string(),
        order_type: "STP PRT".to_string(),
        aux_price: stop_price,
        ..Default::default()
    }
}

/// A sell trailing stop order sets the stop price at a fixed amount below the market price with an attached "trailing" amount. As the
/// market price rises, the stop price rises by the trail amount, but if the stock price falls, the stop loss price doesn't change,
/// and a market order is submitted when the stop price is hit. This technique is designed to allow an investor to specify a limit on the
/// maximum possible loss, without setting a limit on the maximum possible gain. "Buy" trailing stop orders are the mirror image of sell
/// trailing stop orders, and are most appropriate for use in falling markets.
/// Products: CFD, CASH, FOP, FUT, OPT, STK, WAR
//==================================================================================================
pub fn trailing_stop(
    action: &str,
    quantity: f64,
    trailing_percent: f64,
    trail_stop_price: f64,
) -> Order {
    Order {
        action: action.to_string(),
        order_type: "TRAIL".to_string(),
        total_quantity: quantity,
        trailing_percent,
        trail_stop_price,
        ..Default::default()
    }
}

/// A trailing stop limit order is designed to allow an investor to specify a limit on the maximum possible loss, without setting a limit
/// on the maximum possible gain. A SELL trailing stop limit moves with the market price, and continually recalculates the stop trigger
/// price at a fixed amount below the market price, based on the user-defined "trailing" amount. The limit order price is also continually
/// recalculated based on the limit offset. As the market price rises, both the stop price and the limit price rise by the trail amount and
/// limit offset respectively, but if the stock price falls, the stop price remains unchanged, and when the stop price is hit a limit order
/// is submitted at the last calculated limit price. A "Buy" trailing stop limit order is the mirror image of a sell trailing stop limit,
/// and is generally used in falling markets.
/// Products: BOND, CFD, CASH, FUT, FOP, OPT, STK, WAR
//==================================================================================================
pub fn trailing_stop_limit(
    action: &str,
    quantity: f64,
    lmt_price_offset: f64,
    trailing_amount: f64,
    trail_stop_price: f64,
) -> Order {
    Order {
        action: action.to_string(),
        order_type: "TRAIL LIMIT".to_string(),
        total_quantity: quantity,
        trail_stop_price,
        lmt_price_offset,
        aux_price: trailing_amount,
        ..Default::default()
    }
}

/// Create combination orders that include options, stock and futures legs (stock legs can be included if the order is routed
/// through SmartRouting). Although a combination/spread order is constructed of separate legs, it is executed as a single transaction
/// if it is routed directly to an exchange. For combination orders that are SmartRouted, each leg may be executed separately to ensure
/// best execution.
/// Products: OPT, STK, FUT
//==================================================================================================
pub fn combo_limit_order(
    action: &str,
    quantity: f64,
    limit_price: f64,
    non_guaranteed: bool,
) -> Order {
    let mut order = Order {
        action: action.to_string(),
        order_type: "LMT".to_string(),
        tif: "GTC".to_string(),
        total_quantity: quantity,
        lmt_price: limit_price,

        ..Default::default()
    };

    if non_guaranteed {
        order
            .smart_combo_routing_params
            .push(TagValue::new("NonGuaranteed".to_string(), "1".to_string()));
    }

    order
}

/// Create combination orders that include options, stock and futures legs (stock legs can be included if the order is routed
/// through SmartRouting). Although a combination/spread order is constructed of separate legs, it is executed as a single transaction
/// if it is routed directly to an exchange. For combination orders that are SmartRouted, each leg may be executed separately to ensure
/// best execution.
/// Products: OPT, STK, FUT
//==================================================================================================
pub fn combo_market_order(action: &str, quantity: f64, non_guaranteed: bool) -> Order {
    let mut order = Order {
        action: action.to_string(),
        order_type: "MKT".to_string(),
        total_quantity: quantity,
        ..Default::default()
    };

    if non_guaranteed {
        order
            .smart_combo_routing_params
            .push(TagValue::new("NonGuaranteed".to_string(), "1".to_string()));
    }

    order
}

/// Create combination orders that include options, stock and futures legs (stock legs can be included if the order is routed
/// through SmartRouting). Although a combination/spread order is constructed of separate legs, it is executed as a single transaction
/// if it is routed directly to an exchange. For combination orders that are SmartRouted, each leg may be executed separately to ensure
/// best execution.
/// Products: OPT, STK, FUT
//==================================================================================================
pub fn limit_order_for_combo_with_leg_prices(
    action: &str,
    quantity: f64,
    leg_prices: Vec<f64>,
    non_guaranteed: bool,
) -> Order {
    let mut order = Order {
        action: action.to_string(),
        order_type: "LMT".to_string(),
        total_quantity: quantity,
        order_combo_legs: leg_prices
            .iter()
            .map(|&price| OrderComboLeg { price })
            .collect::<Vec<_>>(),
        ..Default::default()
    };

    if non_guaranteed {
        order
            .smart_combo_routing_params
            .push(TagValue::new("NonGuaranteed".to_string(), "1".to_string()));
    }

    order
}

/// Create combination orders that include options, stock and futures legs (stock legs can be included if the order is routed
/// through SmartRouting). Although a combination/spread order is constructed of separate legs, it is executed as a single transaction
/// if it is routed directly to an exchange. For combination orders that are SmartRouted, each leg may be executed separately to ensure
/// best execution.
/// Products: OPT, STK, FUT
//==================================================================================================
pub fn relative_limit_combo(
    action: &str,
    quantity: f64,
    limit_price: f64,
    non_guaranteed: bool,
) -> Order {
    let mut order = Order {
        action: action.to_string(),
        total_quantity: quantity,
        order_type: "REL + LMT".to_string(),
        lmt_price: limit_price,
        ..Default::default()
    };

    if non_guaranteed {
        order
            .smart_combo_routing_params
            .push(TagValue::new("NonGuaranteed".to_string(), "1".to_string()));
    }

    order
}

/// Create combination orders that include options, stock and futures legs (stock legs can be included if the order is routed
/// through SmartRouting). Although a combination/spread order is constructed of separate legs, it is executed as a single transaction
/// if it is routed directly to an exchange. For combination orders that are SmartRouted, each leg may be executed separately to ensure
/// best execution.
/// Products: OPT, STK, FUT
//==================================================================================================
pub fn relative_market_combo(action: &str, quantity: f64, non_guaranteed: bool) -> Order {
    let mut order = Order {
        action: action.to_string(),
        total_quantity: quantity,
        order_type: "REL + MKT".to_string(),

        ..Default::default()
    };

    if non_guaranteed {
        order
            .smart_combo_routing_params
            .push(TagValue::new("NonGuaranteed".to_string(), "1".to_string()));
    }

    order
}

/// One-Cancels All (OCA) order type allows an investor to place multiple and possibly unrelated orders assigned to a group. The aim is
/// to complete just one of the orders, which in turn will cause TWS to cancel the remaining orders. The investor may submit several
/// orders aimed at taking advantage of the most desirable price within the group. Completion of one piece of the group order causes
/// cancellation of the remaining group orders while partial completion causes the group to rebalance. An investor might desire to sell
/// 1000 shares of only ONE of three positions held above prevailing market prices. The OCA order group allows the investor to enter prices
/// at specified target levels and if one is completed, the other two will automatically cancel. Alternatively, an investor may wish to take
/// a LONG position in eMini S&P stock index futures in a falling market or else SELL US treasury futures at a more favorable price.
/// Grouping the two orders using an OCA order type offers the investor two chance to enter a similar position, while only running the risk
/// of taking on a single position.
/// Products: BOND, CASH, FUT, FOP, STK, OPT, WAR
//==================================================================================================
pub fn one_cancels_all(oca_group: &str, oca_orders: Vec<Order>, oca_type: i32) {
    for mut order in oca_orders {
        order.oca_group = oca_group.to_string();
        order.oca_type = oca_type;
    }
}

/// Specific to US options, investors are able to create and enter volatility-type orders for options and combinations rather than price orders.
/// Option traders may wish to trade and position for movements in the price of the option determined by its implied volatility. Because
/// implied volatility is a key determinant of the premium on an option, traders position in specific contract months in an effort to take
/// advantage of perceived changes in implied volatility arising before, during or after earnings or when company specific or broad market
/// volatility is predicted to change. In order to create a volatility order, clients must first create a volatility Trader page from the
/// Trading Tools menu and as they enter option contracts, premiums will display in percentage terms rather than premium. The buy/sell process
/// is the same as for regular orders priced in premium terms except that the client can limit the volatility level they are willing to pay or
/// receive.
/// Products: FOP, OPT
//==================================================================================================
pub fn volatility(
    action: &str,
    quantity: f64,
    volatility_percent: f64,
    volatility_type: i32,
) -> Order {
    Order {
        action: action.to_string(),
        order_type: "VOL".to_string(),
        total_quantity: quantity,
        volatility: volatility_percent, //Expressed in percentage (40%)
        volatility_type,                // 1=daily, 2=annual
        //volatility]
        ..Default::default()
    }
}

//==================================================================================================
pub fn market_fhedge(parent_order_id: i32, action: &str) -> Order {
    // FX Hedge orders can only have a quantity of 0
    let mut order = market_order(action, 0.0);

    order.parent_id = parent_order_id;
    order.hedge_type = "F".to_string();

    order
}

//==================================================================================================
pub fn pegged_to_benchmark(
    action: &str,
    quantity: f64,
    starting_price: f64,
    pegged_change_amount_decrease: bool,
    pegged_change_amount: f64,
    reference_change_amount: f64,
    reference_con_id: i32,
    reference_exchange: &str,
    stock_reference_price: f64,
    reference_contract_lower_range: f64,
    reference_contract_upper_range: f64,
) -> Order {
    Order {
        order_type: "PEG BENCH".to_string(),
        // BUY or SELL
        action: action.to_string(),
        total_quantity: quantity,
        // Beginning with price...
        starting_price,
        // increase/decrease price..
        is_pegged_change_amount_decrease: pegged_change_amount_decrease,
        // by... (and likewise for price moving in opposite direction)
        pegged_change_amount,
        // whenever there is a price change of...
        reference_change_amount,
        // in the reference contract...
        reference_contract_id: reference_con_id,
        // being traded at...
        reference_exchange_id: reference_exchange.parse().unwrap(),
        //starting reference price is...
        stock_ref_price: stock_reference_price,
        // Keep order active as long as reference contract trades between...
        stock_range_lower: reference_contract_lower_range,
        // and...
        stock_range_upper: reference_contract_upper_range,
        //pegged_benchmark]
        ..Default::default()
    }
}

//==================================================================================================
pub fn attach_adjustable_to_stop(
    parent: Order,
    attached_order_stop_price: f64,
    trigger_price: f64,
    adjust_stop_price: f64,
) -> Order {
    // Attached order is a conventional STP order in opposite direction
    let mut order = stop(
        if parent.action == "BUY" {
            "SELL"
        } else {
            "BUY"
        },
        parent.total_quantity,
        attached_order_stop_price,
    );

    order.parent_id = parent.order_id;
    // When trigger price is penetrated
    order.trigger_price = trigger_price;
    // The parent order will be turned into a STP ..Default::default()
    order.adjusted_order_type = "STP".to_string();
    // With the given STP price
    order.adjusted_stop_price = adjust_stop_price;
    //adjustable_stop]
    order
}

//==================================================================================================
pub fn attach_adjustable_to_stop_limit(
    parent: Order,
    attached_order_stop_price: f64,
    trigger_price: f64,
    adjusted_stop_price: f64,
    adjusted_stop_limit_price: f64,
) -> Order {
    // Attached order is a conventional STP ..Default::default()
    let mut order = stop(
        if parent.action == "BUY" {
            "SELL"
        } else {
            "BUY"
        },
        parent.total_quantity,
        attached_order_stop_price,
    );
    order.parent_id = parent.order_id;
    // When trigger price is penetrated
    order.trigger_price = trigger_price;
    // The parent order will be turned into a STP LMT ..Default::default()
    order.adjusted_order_type = "STP LMT".to_string();
    // With the given stop price
    order.adjusted_stop_price = adjusted_stop_price;
    // And the given limit price
    order.adjusted_stop_limit_price = adjusted_stop_limit_price;
    //adjustable_stop_limit]
    order
}

//==================================================================================================
pub fn attach_adjustable_to_trail(
    parent: Order,
    attached_order_stop_price: f64,
    trigger_price: f64,
    adjusted_stop_price: f64,
    adjusted_trail_amount: f64,
    trail_unit: i32,
) -> Order {
    // Attached order is a conventional STP ..Default::default()
    let mut order = stop(
        if parent.action == "BUY" {
            "SELL"
        } else {
            "BUY"
        },
        parent.total_quantity,
        attached_order_stop_price,
    );
    order.parent_id = parent.order_id;
    // When trigger price is penetrated
    order.trigger_price = trigger_price;
    // The parent order will be turned into a TRAIL ..Default::default()
    order.adjusted_order_type = "TRAIL".to_string();
    // With a stop price of...
    order.adjusted_stop_price = adjusted_stop_price;
    // traling by and amount (0) or a percent (1)...
    order.adjustable_trailing_unit = trail_unit;
    // of...
    order.adjusted_trailing_amount = adjusted_trail_amount;

    order
}

//==================================================================================================
pub fn price_condition(
    trigger_method: i32,
    con_id: i32,
    exchange: &str,
    price: f64,
    is_more: bool,
    is_conjunction: bool,
) -> PriceCondition {
    // Conditions have to be created via the OrderCondition.create
    let mut price_condition: PriceCondition = create_condition(ConditionType::Price).into();
    // When this contract...
    price_condition.contract_condition.con_id = con_id;
    // traded on this exchange
    price_condition.contract_condition.exchange = exchange.to_string();
    // has a price above/below
    price_condition
        .contract_condition
        .operator_condition
        .is_more = is_more;
    price_condition.trigger_method = FromPrimitive::from_i32(trigger_method).unwrap();
    // this quantity
    price_condition.price = price;
    // AND | OR next condition (will be ignored if no more conditions are added)
    price_condition
        .contract_condition
        .operator_condition
        .order_condition
        .is_conjunction_connection = is_conjunction;

    price_condition
}

//==================================================================================================
pub fn execution_condition(
    symbol: &str,
    sec_type: &str,
    exchange: &str,
    is_conjunction: bool,
) -> ExecutionCondition {
    let mut exec_condition: ExecutionCondition = create_condition(ConditionType::Execution).into();
    // When an execution on symbol
    exec_condition.symbol = symbol.to_string();
    // at exchange
    exec_condition.exchange = exchange.to_string();
    // for this sec_type
    exec_condition.sec_type = sec_type.to_string();
    // AND | OR next condition (will be ignored if no more conditions are added)
    exec_condition.order_condition.is_conjunction_connection = is_conjunction;
    exec_condition.order_condition.cond_type = ConditionType::Execution;

    exec_condition
}

//==================================================================================================
pub fn margin_condition(percent: f64, is_more: bool, is_conjunction: bool) -> MarginCondition {
    let mut margin_condition: MarginCondition = create_condition(ConditionType::Margin).into();
    // If margin is above/below
    margin_condition.operator_condition.is_more = is_more;
    // given percent
    margin_condition.percent = percent;
    // AND | OR next condition (will be ignored if no more conditions are added)
    margin_condition
        .operator_condition
        .order_condition
        .is_conjunction_connection = is_conjunction;
    margin_condition
        .operator_condition
        .order_condition
        .cond_type = ConditionType::Margin;

    margin_condition
}

//==================================================================================================
pub fn percentage_change_condition(
    pct_change: f64,
    con_id: i32,
    exchange: &str,
    is_more: bool,
    is_conjunction: bool,
) -> PercentChangeCondition {
    let mut pct_change_condition: PercentChangeCondition =
        create_condition(ConditionType::Execution).into();
    // If there is a price percent change measured against last close price above or below...
    pct_change_condition
        .contract_condition
        .operator_condition
        .is_more = is_more;
    // this amount...
    pct_change_condition.change_percent = pct_change;
    // on this contract
    pct_change_condition.contract_condition.con_id = con_id;
    // when traded on this exchange...
    pct_change_condition.contract_condition.exchange = exchange.to_string();
    // AND | OR next condition (will be ignored if no more conditions are added)
    pct_change_condition
        .contract_condition
        .operator_condition
        .order_condition
        .is_conjunction_connection = is_conjunction;
    pct_change_condition
        .contract_condition
        .operator_condition
        .order_condition
        .cond_type = ConditionType::PercentChange;

    pct_change_condition
}

//==================================================================================================
pub fn time_condition(time: &str, is_more: bool, is_conjunction: bool) -> TimeCondition {
    let mut time_condition: TimeCondition = create_condition(ConditionType::Time).into();
    // Before or after...
    time_condition.operator_condition.is_more = is_more;
    // this time..
    time_condition.time = time.to_string();
    // AND | OR next condition (will be ignored if no more conditions are added)
    time_condition
        .operator_condition
        .order_condition
        .is_conjunction_connection = is_conjunction;
    time_condition.operator_condition.order_condition.cond_type = ConditionType::Time;

    time_condition
}

//==================================================================================================
pub fn volume_condition(
    con_id: i32,
    exchange: &str,
    is_more: bool,
    volume: i32,
    is_conjunction: bool,
) -> VolumeCondition {
    let mut vol_cond: VolumeCondition = create_condition(ConditionType::Volume).into();
    // Whenever contract...
    vol_cond.contract_condition.con_id = con_id;
    // When traded at
    vol_cond.contract_condition.exchange = exchange.to_string();
    // reaches a volume higher/lower
    vol_cond.contract_condition.operator_condition.is_more = is_more;
    // than this...
    vol_cond.volume = volume;
    // AND | OR next condition (will be ignored if no more conditions are added)
    vol_cond
        .contract_condition
        .operator_condition
        .order_condition
        .is_conjunction_connection = is_conjunction;
    vol_cond
        .contract_condition
        .operator_condition
        .order_condition
        .cond_type = ConditionType::Volume;

    vol_cond
}

//==================================================================================================
pub fn what_if_limit_order(action: &str, quantity: f64, limit_price: f64) -> Order {
    let mut order = limit_order(action, quantity, limit_price);
    order.what_if = true;

    order
}
