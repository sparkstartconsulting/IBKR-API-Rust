use crate::client::common::{
    BarData, CommissionReport, Contract, ContractDescription, ContractDetails,
    DeltaNeutralContract, DepthExchanges, Execution, FaDataType, FamilyCode, HistogramData,
    HistoricalTick, HistoricalTickBidAsk, NewsProvider, Order, OrderState, PriceIncrement,
    SimpleEntry, SoftDollarTier, TickAttrib, TickAttribBidAsk, TickAttribLast,
};
use std::collections::{HashMap, HashSet};

pub trait Wrapper {
    /// This event is called when there is an error with the
    /// communication or when TWS wants to send a message to the client.
    fn error(&self, req_id: i32, error_code: i32, error_string: &str);

    fn win_error(&self, text: &str, last_error: i32);

    fn connect_ack(&self);

    /// TWS sends a market_data_type(type) callback to the API, where
    /// type is set to Frozen or RealTime, to announce that market data has been
    /// switched between frozen and real-time. This notification occurs only
    /// when market data switches between real-time and frozen. The
    ///  market_data_type( ) callback accepts a req_id parameter and is sent per
    /// every subscription because different contracts can generally trade on a
    /// different schedule.
    fn market_data_type(&self, req_id: i32, market_data_type: i32);

    /// Market data tick price callback. Handles all price related ticks.
    fn tick_price(&self, req_id: i32, tick_type: i32, price: f64, attrib: TickAttrib);

    ///Market data tick size callback. Handles all size-related ticks.
    fn tick_size(&self, req_id: i32, tick_type: i32, size: i32);

    /// When requesting market data snapshots, this market will indicate the
    /// snapshot reception is finished.
    fn tick_snapshot_end(&self, req_id: i32);

    fn tick_generic(&self, req_id: i32, tick_type: i32, value: f64);

    fn tick_string(&self, req_id: i32, tick_type: i32, value: &str);

    ///market data call back for Exchange for Physical
    ///        tickerId -      The request's identifier.
    ///        tick_type -      The type of tick being received.
    ///        basis_points -   Annualized basis points, which is representative of
    ///            the financing rate that can be directly compared to broker rates.
    ///        formatted_basis_points -  Annualized basis points as a formatted string
    ///            that depicts them in percentage form.
    ///        impliedFuture - The implied Futures price.
    ///        hold_days -  The number of hold days until the lastTradeDate of the EFP.
    ///        future_last_trade_date -   The expiration date of the single stock future.
    ///        dividend_impact - The dividend impact upon the annualized basis points
    ///            interest rate.
    ///        dividends_to_last_trade_date - The dividends expected until the expiration
    ///            of the single stock future.
    fn tick_efp(
        &self,
        req_id: i32,
        tick_type: i32,
        basis_points: f64,
        formatted_basis_points: &str,
        total_dividends: f64,
        hold_days: i32,
        future_last_trade_date: &str,
        dividend_impact: f64,
        dividends_to_last_trade_date: f64,
    );

    ///        This event is called whenever the status of an order changes. It is
    //        also fired after reconnecting to TWS if the client has any open orders.
    //
    //        order_id: i32 - The order ID that was specified previously in the
    //            call to placeOrder()
    //        status:&str - The order status. Possible values include:
    //            PendingSubmit - indicates that you have transmitted the order, but have not  yet received confirmation that it has been accepted by the order destination. NOTE: This order status is not sent by TWS and should be explicitly set by the API developer when an order is submitted.
    //            PendingCancel - indicates that you have sent a request to cancel the order but have not yet received cancel confirmation from the order destination. At this point, your order is not confirmed canceled. You may still receive an execution while your cancellation request is pending. NOTE: This order status is not sent by TWS and should be explicitly set by the API developer when an order is canceled.
    //            PreSubmitted - indicates that a simulated order type has been accepted by the IB system and that this order has yet to be elected. The order is held in the IB system until the election criteria are met. At that time the order is transmitted to the order destination as specified.
    //            Submitted - indicates that your order has been accepted at the order destination and is working.
    //            Cancelled - indicates that the balance of your order has been confirmed canceled by the IB system. This could occur unexpectedly when IB or the destination has rejected your order.
    //            Filled - indicates that the order has been completely filled.
    //            Inactive - indicates that the order has been accepted by the system (simulated orders) or an exchange (native orders) but that currently the order is inactive due to system, exchange or other issues.
    //        filled:i32 - Specifies the number of shares that have been executed.
    //            For more information about partial fills, see Order Status for Partial Fills.
    //        remaining:i32 -   Specifies the number of shares still outstanding.
    //        avg_fill_price:f64 - The average price of the shares that have been executed. This parameter is valid only if the filled parameter value is greater than zero. Otherwise, the price parameter will be zero.
    //        perm_id:i32 -  The TWS id used to identify orders. Remains the same over TWS sessions.
    //        parent_id:i32 - The order ID of the parent order, used for bracket and auto trailing stop orders.
    //        lastFilledPrice:f64 - The last price of the shares that have been executed. This parameter is valid only if the filled parameter value is greater than zero. Otherwise, the price parameter will be zero.
    //        client_id:i32 - The ID of the client (or TWS) that placed the order. Note that TWS orders have a fixed client_id and order_id of 0 that distinguishes them from API orders.
    //        why_held:&str - This field is used to identify an order held when TWS is trying to locate shares for a short sell. The value used to indicate this is 'locate'.
    fn order_status(
        &self,
        order_id: i32,
        status: &str,
        filled: f64,
        remaining: f64,
        avg_fill_price: f64,
        perm_id: i32,
        parent_id: i32,
        last_fill_price: f64,
        client_id: i32,
        why_held: &str,
        mkt_cap_price: f64,
    );

    /// This function is called to feed in open orders.
    //
    //        orderID: i32 - The order ID assigned by TWS. Use to cancel or
    //            update TWS order.
    //        contract: Contract - The Contract class attributes describe the contract.
    //        order: Order - The Order class gives the details of the open order.
    //        orderState: OrderState - The orderState class includes attributes Used
    //            for both pre and post trade margin and commission data.
    fn open_order(&self, order_id: i32, contract: Contract, order: Order, order_state: OrderState);

    /// This is called at the end of a given request for open orders.
    fn open_order_end(&self);

    /// This function is called when TWS closes the sockets
    /// connection with the ActiveX control, or when TWS is shut down.
    fn connection_closed(&self);

    ///  This function is called only when ReqAccountUpdates on
    //        EEClientSocket object has been called.
    fn update_account_value(&self, key: &str, val: &str, currency: &str, account_name: &str);

    /// This function is called only when req_account_updates on
    //        EEClientSocket object has been called.
    fn update_portfolio(
        &self,
        contract: Contract,
        position: f64,
        market_price: f64,
        market_value: f64,
        average_cost: f64,
        unrealized_pnl: f64,
        realized_pnl: f64,
        account_name: &str,
    );

    fn update_account_time(&self, time_stamp: &str);

    /// This is called after a batch update_account_value() and
    //        update_portfolio() is sent.
    fn account_download_end(&self, account_name: &str);

    /// Receives next valid order id.
    fn next_valid_id(&self, order_id: i32);

    /// Receives the full contract's definitions. This method will return all
    //        contracts matching the requested via EEClientSocket::reqContractDetails.
    //        For example, one can obtain the whole option chain with it.
    fn contract_details(&self, req_id: i32, contract_details: ContractDetails);

    /// This function is called when reqContractDetails function
    /// has been called for bonds.
    fn bond_contract_details(&self, req_id: i32, contract_details: ContractDetails);

    /// This function is called once all contract details for a given
    //  request are received. This helps to define the end of an option chain.
    fn contract_details_end(&self, req_id: i32);

    /// This event is fired when the reqExecutions() functions is
    /// invoked, or when an order is filled.
    fn exec_details(&self, req_id: i32, contract: Contract, execution: Execution);

    /// This function is called once all executions have been sent to
    /// a client in response to reqExecutions().
    fn exec_details_end(&self, req_id: i32);

    /// Returns the order book.
    ///
    ///        tickerId -  the request's identifier
    ///        position -  the order book's row being updated
    ///        operation - how to refresh the row:
    ///            0 = insert (insert this new order into the row identified by 'position')
    ///            1 = update (update the existing order in the row identified by 'position')
    ///            2 = delete (delete the existing order at the row identified by 'position').
    ///        side -  0 for ask, 1 for bid
    ///        price - the order's price
    ///        size -  the order's size
    fn update_mkt_depth(
        &self,
        req_id: i32,
        position: i32,
        operation: i32,
        side: i32,
        price: f64,
        size: i32,
    );

    /// Returns the order book.
    ///
    ///        tickerId -  the request's identifier
    ///        position -  the order book's row being updated
    ///        marketMaker - the exchange holding the order
    ///        operation - how to refresh the row:
    ///            0 = insert (insert this new order into the row identified by 'position')
    ///            1 = update (update the existing order in the row identified by 'position')
    ///            2 = delete (delete the existing order at the row identified by 'position').
    ///        side -  0 for ask, 1 for bid
    ///        price - the order's price
    ///        size -  the order's size
    ///        isSmartDepth - is SMART Depth request
    fn update_mkt_depth_l2(
        &self,
        req_id: i32,
        position: i32,
        market_maker: &str,
        operation: i32,
        side: i32,
        price: f64,
        size: i32,
        is_smart_depth: bool,
    );

    /// provides IB's bulletins
    ///        msgId - the bulletin's identifier
    ///        msgType - one of: 1 - Regular news bulletin 2 - Exchange no longer
    ///            available for trading 3 - Exchange is available for trading
    ///        message - the message
    ///        origExchange -    the exchange where the message comes from.
    fn update_news_bulletin(
        &self,
        msg_id: i32,
        msg_type: i32,
        news_message: &str,
        origin_exch: &str,
    );

    /// Receives a comma-separated string with the managed account ids.
    fn managed_accounts(&self, accounts_list: &str);

    ///  receives the Financial Advisor's configuration available in the TWS
    ///
    ///        faDataType - one of:
    ///            Groups: offer traders a way to create a group of accounts and apply
    ///                 a single allocation method to all accounts in the group.
    ///            Profiles: let you allocate shares on an account-by-account basis
    ///                using a predefined calculation value.
    ///            Account Aliases: let you easily identify the accounts by meaningful
    ///                 names rather than account numbers.
    ///        faXmlData -  the xml-formatted configuration
    fn receive_fa(&self, fa_data: FaDataType, cxml: &str);

    ///  returns the requested historical data bars
    ///
    ///        req_id - the request's identifier
    ///        date  - the bar's date and time (either as a yyyymmss hh:mm:ssformatted
    ///             string or as system time according to the request)
    ///        open  - the bar's open point
    ///        high  - the bar's high point
    ///        low   - the bar's low point
    ///        close - the bar's closing point
    ///        volume - the bar's traded volume if available
    ///        count - the number of trades during the bar's timespan (only available
    ///            for TRADES).
    ///        WAP -   the bar's Weighted Average Price
    ///        hasGaps  -indicates if the data has gaps or not.
    fn historical_data(&self, req_id: i32, bar: BarData);

    /// Marks the ending of the historical bars reception.
    fn historical_data_end(&self, req_id: i32, start: &str, end: &str);

    ///  Provides the xml-formatted parameters available to create a market
    ///        scanner.
    ///
    ///        xml -   the xml-formatted string with the available parameters.
    fn scanner_parameters(&self, xml: &str);

    ///  Provides the data resulting from the market scanner request.
    ///
    ///        reqid - the request's identifier.
    ///        rank -  the ranking within the response of this bar.
    ///        contract_details - the data's ContractDetails
    ///        distance -      according to query.
    ///        benchmark -     according to query.
    ///        projection -    according to query.
    ///        legStr - describes the combo legs when the scanner is returning EFP
    fn scanner_data(
        &self,
        req_id: i32,
        rank: i32,
        contract_details: ContractDetails,
        distance: &str,
        benchmark: &str,
        projection: &str,
        legs_str: &str,
    );

    ///  Indicates the scanner data reception has terminated.
    ///
    ///        req_id - the request's identifier
    fn scanner_data_end(&self, req_id: i32);

    ///  Updates the real time 5 seconds bars
    ///
    ///        req_id - the request's identifier
    ///        bar.time  - start of bar in unix (or 'epoch') time
    ///        bar.endTime - for synthetic bars, the end time (requires TWS v964). Otherwise -1.
    ///        bar.open_  - the bar's open value
    ///        bar.high  - the bar's high value
    ///        bar.low   - the bar's low value
    ///        bar.close - the bar's closing value
    ///        bar.volume - the bar's traded volume if available
    ///        bar.WAP   - the bar's Weighted Average Price
    ///        bar.count - the number of trades during the bar's timespan (only available
    ///            for TRADES).
    fn realtime_bar(
        &self,
        req_id: i32,
        time: i32,
        open_: f64,
        high: f64,
        low: f64,
        close: f64,
        volume: i32,
        wap: f64,
        count: i32,
    );

    ///  Server's current time. This method will receive IB server's system
    ///  time resulting after the invokation of reqCurrentTime.
    fn current_time(&self, time: i32);

    /// This function is called to receive fundamental
    /// market data. The appropriate market data subscription must be set
    /// up in Account Management before you can receive this data.
    fn fundamental_data(&self, req_id: i32, data: &str);

    /// Upon accepting a Delta-Neutral RFQ(request for quote), the
    /// server sends a delta_neutral_validation() message with the DeltaNeutralContract
    /// structure. If the delta and price fields are empty in the original
    /// request, the confirmation will contain the current values from the
    /// server. These values are locked when the RFQ is processed and remain
    /// locked until the RFQ is canceled.
    fn delta_neutral_validation(&self, req_id: i32, delta_neutral_contract: DeltaNeutralContract);

    /// The commission_report() callback is triggered as follows:
    /// immediately after a trade execution
    /// by calling reqExecutions().
    fn commission_report(&self, commission_report: CommissionReport);

    /// This event returns real-time positions for all accounts in
    /// response to the reqPositions() method.
    fn position(&self, account: &str, contract: Contract, position: f64, avg_cost: f64);

    /// This is called once all position data for a given request are
    //  received and functions as an end marker for the position() data.
    fn position_end(&self);

    /// Returns the data from the TWS Account Window Summary tab in
    /// response to req_account_summary().
    fn account_summary(&self, req_id: i32, account: &str, tag: &str, value: &str, currency: &str);

    /// This method is called once all account summary data for a
    //  given request are received.
    fn account_summary_end(&self, req_id: i32);

    /// Deprecated Function
    fn verify_message_api(&self, api_data: &str);

    fn verify_completed(&self, is_successful: bool, error_text: &str);

    fn verify_and_auth_message_api(&self, api_data: &str, xyz_challange: &str);

    fn verify_and_auth_completed(&self, is_successful: bool, error_text: &str);

    /// This callback is a one-time response to queryDisplayGroups().
    ///
    ///        req_id - The requestId specified in queryDisplayGroups().
    ///        groups - A list of integers representing visible group ID separated by
    ///            the | character, and sorted by most used group first. This list will
    ///             not change during TWS session (in other words, user cannot add a
    ///            new group; sorting can change though).
    fn display_group_list(&self, req_id: i32, groups: &str);

    /// This is sent by TWS to the API client once after receiving
    ///        the subscription request subscribeToGroupEvents(), and will be sent
    ///        again if the selected contract in the subscribed display group has
    ///        changed.
    ///
    ///        requestId - The requestId specified in subscribeToGroupEvents().
    ///        contractInfo - The encoded value that uniquely represents the contract
    ///            in IB. Possible values include:
    ///            none = empty selection
    ///            contractID@exchange = any non-combination contract.
    ///                Examples: 8314@SMART for IBM SMART; 8314@ARCA for IBM @ARCA.
    ///            combo = if any combo is selected.
    fn display_group_updated(&self, req_id: i32, contract_info: &str);

    /// same as position() except it can be for a certain
    /// account/model
    fn position_multi(
        &self,
        req_id: i32,
        account: &str,
        model_code: &str,
        contract: Contract,
        pos: f64,
        avg_cost: f64,
    );

    /// same as position_end() except it can be for a certain
    /// account/model
    fn position_multi_end(&self, req_id: i32);

    /// same as update_account_value() except it can be for a certain
    /// account/model
    fn account_update_multi(
        &self,
        req_id: i32,
        account: &str,
        model_code: &str,
        key: &str,
        value: &str,
        currency: &str,
    );

    /// same as account_download_end() except it can be for a certain
    ///  account/model
    fn account_update_multi_end(&self, req_id: i32);

    /// This function is called when the market in an option or its
    ///        underlier moves. TWS's option model volatilities, prices, and
    ///        deltas, along with the present value of dividends expected on that
    ///        options underlier are received.
    fn tick_option_computation(
        &self,
        req_id: i32,
        tick_type: i32,
        implied_vol: f64,
        delta: f64,
        opt_price: f64,
        pv_dividend: f64,
        gamma: f64,
        vega: f64,
        theta: f64,
        und_price: f64,
    );

    /// Returns the option chain for an underlying on an exchange
    //  specified in reqSecDefOptParams There will be multiple callbacks to
    //  security_definition_option_parameter if multiple exchanges are specified
    //  in reqSecDefOptParams
    //
    //  req_id - ID of the request initiating the callback
    //  underlyingConId - The conID of the underlying security
    //  tradingClass -  the option trading class
    //  multiplier -    the option multiplier
    //  expirations - a list of the expiries for the options of this underlying on this exchange
    //  strikes - a list of the possible strikes for options of this underlying on this exchange
    ///
    fn security_definition_option_parameter(
        &self,
        req_id: i32,
        exchange: &str,
        underlying_con_id: i32,
        trading_class: &str,
        multiplier: &str,
        expirations: HashSet<String>,
        strikes: HashSet<f64>,
    );

    /// Called when all callbacks to security_definition_option_parameter are complete
    ///
    /// req_id - the ID used in the call to security_definition_option_parameter
    fn security_definition_option_parameter_end(&self, req_id: i32);

    /// Called when receives Soft Dollar Tier configuration information
    ///
    ///        req_id - The request ID used in the call to EEClient::reqSoftDollarTiers
    ///        tiers - Stores a list of SoftDollarTier that contains all Soft Dollar
    ///            Tiers information
    fn soft_dollar_tiers(&self, req_id: i32, tiers: Vec<SoftDollarTier>);

    /// returns array of family codes
    fn family_codes(&self, family_codes: Vec<FamilyCode>);

    /// returns array of sample contract descriptions
    fn symbol_samples(&self, req_id: i32, contract_descriptions: Vec<ContractDescription>);

    /// returns array of exchanges which return depth to UpdateMktDepthL2
    fn mkt_depth_exchanges(&self, depth_mkt_data_descriptions: Vec<DepthExchanges>);

    /// returns news headlines
    fn tick_news(
        &self,
        ticker_id: i32,
        time_stamp: i32,
        provider_code: &str,
        article_id: &str,
        headline: &str,
        extra_data: &str,
    );

    /// returns exchange component mapping
    fn smart_components(&self, req_id: i32, smart_component_map: HashMap<i32, SimpleEntry>);

    /// returns exchange map of a particular contract
    fn tick_req_params(
        &self,
        ticker_id: i32,
        min_tick: f64,
        bbo_exchange: &str,
        snapshot_permissions: i32,
    );

    /// returns available, subscribed API news providers
    fn news_providers(&self, news_providers: Vec<NewsProvider>);

    /// returns body of news article
    fn news_article(&self, request_id: i32, article_type: i32, article_text: &str);

    /// returns historical news headlines
    fn historical_news(
        &self,
        request_id: i32,
        time: &str,
        provider_code: &str,
        article_id: &str,
        headline: &str,
    );

    /// signals end of historical news
    fn historical_news_end(&self, request_id: i32, has_more: bool);

    /// returns earliest available data of a type of data for a particular contract
    fn head_timestamp(&self, req_id: i32, head_timestamp: &str);

    /// returns histogram data for a contract
    fn histogram_data(&self, req_id: i32, items: HistogramData);

    /// returns updates in real time when keepUpToDate is set to True
    fn historical_data_update(&self, req_id: i32, bar: BarData);

    /// returns reroute CFD contract information for market data request
    fn reroute_mkt_data_req(&self, req_id: i32, con_id: i32, exchange: &str);

    /// returns reroute CFD contract information for market depth request
    fn reroute_mkt_depth_req(&self, req_id: i32, con_id: i32, exchange: &str);

    /// returns minimum price increment structure for a particular market rule ID
    fn market_rule(&self, market_rule_id: i32, price_increments: Vec<PriceIncrement>);

    /// returns the daily PnL for the account
    fn pnl(&self, req_id: i32, daily_pn_l: f64, unrealized_pn_l: f64, realized_pn_l: f64);

    /// returns the daily PnL for a single position in the account
    fn pnl_single(
        &self,
        req_id: i32,
        pos: i32,
        daily_pn_l: f64,
        unrealized_pn_l: f64,
        realized_pn_l: f64,
        value: f64,
    );

    /// returns historical tick data when whatToShow=MIDPOINT
    fn historical_ticks(&self, req_id: i32, ticks: Vec<HistoricalTick>, done: bool);

    /// returns historical tick data when whatToShow=BID_ASK
    fn historical_ticks_bid_ask(&self, req_id: i32, ticks: Vec<HistoricalTickBidAsk>, done: bool);

    /// returns historical tick data when whatToShow=TRADES

    /// returns tick-by-tick data for tickType = "Last" or "AllLast"
    fn tick_by_tick_all_last(
        &self,
        req_id: i32,
        tick_type: i32,
        time: i32,
        price: f64,
        size: i32,
        tick_attrib_last: TickAttribLast,
        exchange: &str,
        special_conditions: &str,
    );

    /// returns tick-by-tick data for tickType = "BidAsk"
    fn tick_by_tick_bid_ask(
        &self,
        req_id: i32,
        time: i32,
        bid_price: f64,
        ask_price: f64,
        bid_size: i32,
        ask_size: i32,
        tick_attrib_bid_ask: TickAttribBidAsk,
    );

    /// returns tick-by-tick data for tickType = "MidPoint"
    fn tick_by_tick_mid_point(&self, req_id: i32, time: i32, mid_point: f64);

    /// returns order_bound notification
    fn order_bound(&self, req_id: i32, api_client_id: i32, api_order_id: i32);

    /// This function is called to feed in completed orders.
    ///
    ///        contract: Contract - The Contract class attributes describe the contract.
    ///       order: Order - The Order class gives the details of the completed order.
    ///        orderState: OrderState - The orderState class includes completed order status details.
    ///
    fn completed_order(&self, contract: Contract, order: Order, order_state: OrderState);

    /// This is called at the end of a given request for completed orders.
    fn completed_orders_end(&self);
}
