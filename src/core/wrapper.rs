//! Clients implement the Wrapper trait in this module to receive data and notifications from Trader WorkStation or IB Gateway
use std::collections::HashSet;
use std::marker::{Send, Sync};

use bigdecimal::BigDecimal;

use super::common::RealTimeBar;
use crate::core::common::{
    BarData, CommissionReport, DepthMktDataDescription, FaDataType, FamilyCode, HistogramData,
    HistoricalTick, HistoricalTickBidAsk, HistoricalTickLast, NewsProvider, PriceIncrement,
    SmartComponent, TickAttrib, TickAttribBidAsk, TickAttribLast, TickByTickType, TickType,
};
use crate::core::contract::{Contract, ContractDescription, ContractDetails, DeltaNeutralContract};
use crate::core::execution::Execution;
use crate::core::order::{Order, OrderState, SoftDollarTier};

/// A trait that clients will implement that declares callback functions that get called when the application receives messages from the Trader WorkStation or IB Gateway
pub trait Wrapper: Send + Sync + 'static {
    //----------------------------------------------------------------------------------------------
    /// This event is called when there is an error with the
    /// communication or when TWS wants to send a message to the core.
    fn error(&mut self, req_id: i32, error_code: i32, error_string: &str);

    //----------------------------------------------------------------------------------------------
    fn win_error(&mut self, text: &str, last_error: i32);

    //----------------------------------------------------------------------------------------------
    fn connect_ack(&mut self);

    //----------------------------------------------------------------------------------------------
    /// TWS sends a market_data_type(type) callback to the API, where
    /// type is set to Frozen or RealTime, to announce that market data has been
    /// switched between frozen and real-time. This notification occurs only
    /// when market data switches between real-time and frozen. The
    /// market_data_type() callback accepts a req_id parameter and is sent per
    /// every subscription because different contracts can generally trade on a
    /// different schedule.
    fn market_data_type(&mut self, req_id: i32, market_data_type: i32);

    //----------------------------------------------------------------------------------------------
    /// Market data tick price callback. Handles all price related ticks.
    fn tick_price(&mut self, req_id: i32, tick_type: TickType, price: f64, attrib: TickAttrib);

    //----------------------------------------------------------------------------------------------
    ///Market data tick size callback. Handles all size-related ticks.
    fn tick_size(&mut self, req_id: i32, tick_type: TickType, size: i32);

    //----------------------------------------------------------------------------------------------
    /// When requesting market data snapshots, this market will indicate the
    /// snapshot reception is finished.
    fn tick_snapshot_end(&mut self, req_id: i32);

    //----------------------------------------------------------------------------------------------
    fn tick_generic(&mut self, req_id: i32, tick_type: TickType, value: f64);

    //----------------------------------------------------------------------------------------------
    fn tick_string(&mut self, req_id: i32, tick_type: TickType, value: &str);

    //----------------------------------------------------------------------------------------------
    /// market data call back for Exchange for Physical
    ///
    /// # Arguments
    /// * req_id - The request's identifier.
    /// * tick_type - The type of tick being received.
    /// * basis_points - Annualized basis points, which is representative of
    ///                the financing rate that can be directly compared to broker rates.
    /// * formatted_basis_points - Annualized basis points as a formatted string
    ///                          that depicts them in percentage form.
    /// * implied_future - The implied Futures price.
    /// * hold_days -  The number of hold days until the lastTradeDate of the EFP.
    /// * future_last_trade_date -   The expiration date of the single stock future.
    /// * dividend_impact - The dividend impact upon the annualized basis points
    ///                     interest rate.
    /// * dividends_to_last_trade_date - The dividends expected until the expiration
    ///                                  of the single stock future.
    fn tick_efp(
        &mut self,
        req_id: i32,
        tick_type: TickType,
        basis_points: f64,
        formatted_basis_points: &str,
        implied_future: f64,
        hold_days: i32,
        future_last_trade_date: &str,
        dividend_impact: f64,
        dividends_to_last_trade_date: f64,
    );

    //----------------------------------------------------------------------------------------------
    /// This event is called whenever the status of an order changes. It is
    /// also fired after reconnecting to TWS if the core has any open orders.
    ///
    /// # Arguments
    /// * order_id - The order ID that was specified previously in the
    ///            call to placeOrder()
    /// * status - The order status. Possible values include:
    ///     * PendingSubmit - indicates that you have transmitted the order, but have not  yet received confirmation that it has been accepted by the order destination. NOTE: This order status is not sent by TWS and should be explicitly set by the API developer when an order is submitted.
    ///     * PendingCancel - indicates that you have sent a request to cancel the order but have not yet received cancel confirmation from the order destination. At this point, your order is not confirmed canceled. You may still receive an execution while your cancellation request is pending. NOTE: This order status is not sent by TWS and should be explicitly set by the API developer when an order is canceled.
    ///     * PreSubmitted - indicates that a simulated order type has been accepted by the IB system and that this order has yet to be elected. The order is held in the IB system until the election criteria are met. At that time the order is transmitted to the order destination as specified.
    ///     * Submitted - indicates that your order has been accepted at the order destination and is working.
    ///     * Cancelled - indicates that the balance of your order has been confirmed canceled by the IB system. This could occur unexpectedly when IB or the destination has rejected your order.
    ///     * Filled - indicates that the order has been completely filled.
    ///     * Inactive - indicates that the order has been accepted by the system (simulated orders) or an exchange (native orders) but that currently the order is inactive due to system, exchange or other issues.
    /// * filled - Specifies the number of shares that have been executed.
    ///          For more information about partial fills, see Order Status for Partial Fills.
    /// * remaining -   Specifies the number of shares still outstanding.
    /// * avg_fill_price - The average price of the shares that have been executed. This parameter is valid only if the filled parameter value is greater than zero. Otherwise, the price parameter will be zero.
    /// * perm_id -  The TWS id used to identify orders. Remains the same over TWS sessions.
    /// * parent_id - The order ID of the parent order, used for bracket and auto trailing stop orders.
    /// * lastFilledPrice - The last price of the shares that have been executed. This parameter is valid only if the filled parameter value is greater than zero. Otherwise, the price parameter will be zero.
    /// * client_id - The ID of the core (or TWS) that placed the order. Note that TWS orders have a fixed client_id and order_id of 0 that distinguishes them from API orders.
    /// * why_held - This field is used to identify an order held when TWS is trying to locate shares for a short sell. The value used to indicate this is 'locate'.
    fn order_status(
        &mut self,
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

    //----------------------------------------------------------------------------------------------
    /// This function is called to feed in open orders.
    ///
    /// # Arguments
    /// * order_id - The order ID assigned by TWS. Use to cancel or
    ///           update TWS order.
    /// * contract - The Contract class attributes describe the contract.
    /// * order - The Order class gives the details of the open order.
    /// * order_state - The orderState class includes attributes Used
    ///               for both pre and post trade margin and commission data.
    fn open_order(
        &mut self,
        order_id: i32,
        contract: Contract,
        order: Order,
        order_state: OrderState,
    );

    //----------------------------------------------------------------------------------------------
    /// This is called at the end of a given request for open orders.
    fn open_order_end(&mut self);

    //----------------------------------------------------------------------------------------------
    /// This function is called when TWS closes the sockets
    /// connection with the ActiveX control, or when TWS is shut down.
    fn connection_closed(&mut self);

    //----------------------------------------------------------------------------------------------
    /// This function is called only when req_account_updates on
    /// EClient object has been called.
    fn update_account_value(&mut self, key: &str, val: &str, currency: &str, account_name: &str);

    //----------------------------------------------------------------------------------------------
    /// This function is called only when req_account_updates on
    /// EClient object has been called.
    fn update_portfolio(
        &mut self,
        contract: Contract,
        position: f64,
        market_price: f64,
        market_value: f64,
        average_cost: f64,
        unrealized_pnl: f64,
        realized_pnl: f64,
        account_name: &str,
    );

    //----------------------------------------------------------------------------------------------
    fn update_account_time(&mut self, time_stamp: &str);

    //----------------------------------------------------------------------------------------------
    /// This is called after a batch update_account_value() and
    /// update_portfolio() is sent.
    fn account_download_end(&mut self, account_name: &str);

    //----------------------------------------------------------------------------------------------
    /// Receives next valid order id.
    fn next_valid_id(&mut self, order_id: i32);

    //----------------------------------------------------------------------------------------------
    /// Receives the full contract's definitions. This method will return all
    /// contracts matching the requested via req_contract_details.
    /// For example, one can obtain the whole option chain with it.
    fn contract_details(&mut self, req_id: i32, contract_details: ContractDetails);

    //----------------------------------------------------------------------------------------------
    /// This function is called when req_contract_details function
    /// has been called for bonds.
    fn bond_contract_details(&mut self, req_id: i32, contract_details: ContractDetails);

    //----------------------------------------------------------------------------------------------
    /// This function is called once all contract details for a given
    /// request are received. This helps to define the end of an option chain.
    fn contract_details_end(&mut self, req_id: i32);

    //----------------------------------------------------------------------------------------------
    /// This event is fired when the req_executions() functions is
    /// invoked, or when an order is filled.
    fn exec_details(&mut self, req_id: i32, contract: Contract, execution: Execution);

    //----------------------------------------------------------------------------------------------
    /// This function is called once all executions have been sent to
    /// a core in response to req_executions().
    fn exec_details_end(&mut self, req_id: i32);

    //----------------------------------------------------------------------------------------------
    /// Returns the order book.
    ///
    /// # Arguments       
    /// * req_id -  the request id
    /// * position -  the order book's row being updated
    /// * operation - how to refresh the row:
    ///     * 0 = insert (insert this new order into the row identified by 'position')
    ///     * 1 = update (update the existing order in the row identified by 'position')
    ///     * 2 = delete (delete the existing order at the row identified by 'position').
    /// * side
    ///     * 0 for ask
    ///     * 1 for bid
    /// * price - the order's price
    /// * size -  the order's size
    fn update_mkt_depth(
        &mut self,
        req_id: i32,
        position: i32,
        operation: i32,
        side: i32,
        price: f64,
        size: i32,
    );

    //----------------------------------------------------------------------------------------------
    /// Returns the order book.
    ///
    /// # Arguments
    /// * req_id -  the request id
    /// * position -  the order book's row being updated
    /// * market_maker - the exchange holding the order
    /// * operation - how to refresh the row:
    ///       0 = insert (insert this new order into the row identified by 'position')
    ///       1 = update (update the existing order in the row identified by 'position')
    ///       2 = delete (delete the existing order at the row identified by 'position').
    /// * side
    ///     * 0 for ask
    ///     * 1 for bid
    /// * price - the order's price
    /// * size -  the order's size
    /// * is_smart_depth - is SMART Depth request
    fn update_mkt_depth_l2(
        &mut self,
        req_id: i32,
        position: i32,
        market_maker: &str,
        operation: i32,
        side: i32,
        price: f64,
        size: i32,
        is_smart_depth: bool,
    );

    //----------------------------------------------------------------------------------------------
    /// provides IB's bulletins
    ///
    /// # Arguments
    /// * msg_id - the bulletin's identifier
    /// * msg_type - one of: 
    ///     * 1 - Regular news bulletin
    ///     * 2 - Exchange no longer available for trading 
    ///     * 3 - Exchange is available for trading
    /// * news_message - the message
    /// * origin_exch -    the exchange where the message comes from.
    fn update_news_bulletin(
        &mut self,
        msg_id: i32,
        msg_type: i32,
        news_message: &str,
        origin_exch: &str,
    );

    //----------------------------------------------------------------------------------------------
    /// Receives a comma-separated string with the managed account ids.
    fn managed_accounts(&mut self, accounts_list: &str);

    //----------------------------------------------------------------------------------------------
    ///  receives the Financial Advisor's configuration available in the TWS
    ///
    /// # Arguments
    /// * fa_data - one of:
    ///     * Groups: offer traders a way to create a group of accounts and apply
    ///              a single allocation method to all accounts in the group.
    ///     * Profiles: let you allocate shares on an account-by-account basis
    ///               using a predefined calculation value.
    ///     * Account Aliases: let you easily identify the accounts by meaningful
    ///               names rather than account numbers.
    ///     * faXmlData -  the xml-formatted configuration
    fn receive_fa(&mut self, fa_data: FaDataType, cxml: &str);

    //----------------------------------------------------------------------------------------------
    ///  returns the requested historical data bars
    ///
    /// # Arguments
    /// * req_id - the request's identifier
    /// * bar - BarData struct containing historical bar data information
    fn historical_data(&mut self, req_id: i32, bar: BarData);

    //----------------------------------------------------------------------------------------------
    /// Marks the ending of the historical bars reception.
    fn historical_data_end(&mut self, req_id: i32, start: &str, end: &str);

    //----------------------------------------------------------------------------------------------
    /// Provides the xml-formatted parameters available to create a market scanner.
    ///
    /// # Arguments
    /// * xml -   the xml-formatted string with the available parameters.
    fn scanner_parameters(&mut self, xml: &str);

    //----------------------------------------------------------------------------------------------
    ///  Provides the data resulting from the market scanner request.
    ///
    /// # Arguments
    /// * req_id - the request's identifier.
    /// * rank -  the ranking within the response of this bar.
    /// * contract_details - the data's ContractDetails
    /// * distance - according to query.
    /// * benchmark - according to query.
    /// * projection - according to query.
    /// * legs_str - describes the combo legs when the scanner is returning EFP
    fn scanner_data(
        &mut self,
        req_id: i32,
        rank: i32,
        contract_details: ContractDetails,
        distance: &str,
        benchmark: &str,
        projection: &str,
        legs_str: &str,
    );

    //----------------------------------------------------------------------------------------------
    /// Indicates the scanner data reception has terminated.
    fn scanner_data_end(&mut self, req_id: i32);

    //----------------------------------------------------------------------------------------------
    /// Updates the real time 5 seconds bars
    ///
    /// # Arguments
    /// * req_id - the request's identifier
    /// * bar - RealTimeBar data
    fn realtime_bar(&mut self, req_id: i32, bar: RealTimeBar);

    //----------------------------------------------------------------------------------------------
    /// Server's current time. This method will receive IB server's system
    /// time resulting after the invokation of req_current_time.
    fn current_time(&mut self, time: i64);

    //----------------------------------------------------------------------------------------------
    /// This function is called to receive fundamental
    /// market data. The appropriate market data subscription must be set
    /// up in Account Management before you can receive this data.
    fn fundamental_data(&mut self, req_id: i32, data: &str);

    //----------------------------------------------------------------------------------------------
    /// Upon accepting a Delta-Neutral RFQ(request for quote), the
    /// server sends a delta_neutral_validation() message with the DeltaNeutralContract
    /// structure. If the delta and price fields are empty in the original
    /// request, the confirmation will contain the current values from the
    /// server. These values are locked when the RFQ is processed and remain
    /// locked until the RFQ is canceled.
    fn delta_neutral_validation(
        &mut self,
        req_id: i32,
        delta_neutral_contract: DeltaNeutralContract,
    );

    //----------------------------------------------------------------------------------------------
    /// The commission_report() callback is triggered as follows:
    /// immediately after a trade execution
    /// by calling req_executions().
    fn commission_report(&mut self, commission_report: CommissionReport);

    //----------------------------------------------------------------------------------------------
    /// This event returns real-time positions for all accounts in
    /// response to the reqPositions() method.
    fn position(&mut self, account: &str, contract: Contract, position: f64, avg_cost: f64);

    //----------------------------------------------------------------------------------------------
    /// This is called once all position data for a given request are
    /// received and functions as an end marker for the position data.
    fn position_end(&mut self);

    //----------------------------------------------------------------------------------------------
    /// Returns the data from the TWS Account Window Summary tab in
    /// response to req_account_summary().
    fn account_summary(
        &mut self,
        req_id: i32,
        account: &str,
        tag: &str,
        value: &str,
        currency: &str,
    );

    //----------------------------------------------------------------------------------------------
    /// This method is called once all account summary data for a
    /// given request are received.
    fn account_summary_end(&mut self, req_id: i32);

    //----------------------------------------------------------------------------------------------
    /// Deprecated Function
    fn verify_message_api(&mut self, api_data: &str);

    //----------------------------------------------------------------------------------------------
    /// Deprecated Function
    fn verify_completed(&mut self, is_successful: bool, error_text: &str);

    //----------------------------------------------------------------------------------------------
    /// Deprecated Function
    fn verify_and_auth_message_api(&mut self, api_data: &str, xyz_challange: &str);

    //----------------------------------------------------------------------------------------------
    /// Deprecated Function
    fn verify_and_auth_completed(&mut self, is_successful: bool, error_text: &str);

    //----------------------------------------------------------------------------------------------
    /// This callback is a one-time response to query_display_groups().
    ///
    /// # Arguments
    /// * req_id - The requestId specified in query_display_groups().
    /// * groups - A list of integers representing visible group ID's separated by
    ///            the | character, and sorted by most used group first. This list will
    ///            not change during TWS session (in other words, user cannot add a
    ///            new group; sorting can change though).
    fn display_group_list(&mut self, req_id: i32, groups: &str);

    //----------------------------------------------------------------------------------------------
    /// This is sent by TWS to the API core once after receiving
    /// the subscription request subscribe_to_group_events(), and will be sent
    /// again if the selected contract in the subscribed display group has
    /// changed.
    ///
    /// # Arguments
    /// * req_id - The requestId specified in subscribe_to_group_events().
    /// * contract_info - The encoded value that uniquely represents the contract
    ///                 in IB. Possible values include:
    ///                 none = empty selection
    ///                 contractID@exchange = any non-combination contract.
    ///                
    ///                 Examples: 8314@SMART for IBM SMART; 8314@ARCA for IBM @ARCA.
    ///                 combo = if any combo is selected.
    fn display_group_updated(&mut self, req_id: i32, contract_info: &str);

    //----------------------------------------------------------------------------------------------
    /// same as position() except it can be for a certain account/model
    fn position_multi(
        &mut self,
        req_id: i32,
        account: &str,
        model_code: &str,
        contract: Contract,
        pos: f64,
        avg_cost: f64,
    );

    //----------------------------------------------------------------------------------------------
    /// same as position_end() except it can be for a certain
    /// account/model
    fn position_multi_end(&mut self, req_id: i32);

    //----------------------------------------------------------------------------------------------
    /// same as update_account_value() except it can be for a certain
    /// account/model
    fn account_update_multi(
        &mut self,
        req_id: i32,
        account: &str,
        model_code: &str,
        key: &str,
        value: &str,
        currency: &str,
    );

    //----------------------------------------------------------------------------------------------
    /// same as account_download_end() except it can be for a certain
    /// account/model
    fn account_update_multi_end(&mut self, req_id: i32);

    //----------------------------------------------------------------------------------------------
    /// This function is called when the market in an option or its
    /// underlier moves. TWS's option model volatilities, prices, and
    /// deltas, along with the present value of dividends expected on that
    /// options underlier are received.
    fn tick_option_computation(
        &mut self,
        req_id: i32,
        tick_type: TickType,
        implied_vol: f64,
        delta: f64,
        opt_price: f64,
        pv_dividend: f64,
        gamma: f64,
        vega: f64,
        theta: f64,
        und_price: f64,
    );

    //----------------------------------------------------------------------------------------------
    /// Returns the option chain for an underlying on an exchange
    /// specified in req_sec_def_opt_params There will be multiple callbacks to
    /// security_definition_option_parameter if multiple exchanges are specified
    /// in req_sec_def_opt_params
    //
    /// # Arguments
    /// * req_id - ID of the request initiating the callback
    /// * underlying_con_id - The conID of the underlying security
    /// * trading_class -  the option trading class
    /// * multiplier -    the option multiplier
    /// * expirations - a list of the expiries for the options of this underlying on this exchange
    /// * strikes - a list of the possible strikes for options of this underlying on this exchange
    ///
    fn security_definition_option_parameter(
        &mut self,
        req_id: i32,
        exchange: &str,
        underlying_con_id: i32,
        trading_class: &str,
        multiplier: &str,
        expirations: HashSet<String>,
        strikes: HashSet<BigDecimal>,
    );

    //----------------------------------------------------------------------------------------------
    /// Called when all callbacks to security_definition_option_parameter are complete
    ///
    /// * req_id - the ID used in the call to security_definition_option_parameter
    fn security_definition_option_parameter_end(&mut self, req_id: i32);

    //----------------------------------------------------------------------------------------------
    /// Called when receives Soft Dollar Tier configuration information
    ///
    /// * req_id - The request ID used in the call to EEClient::req_soft_dollar_tiers
    /// * tiers - Stores a list of SoftDollarTier that contains all Soft Dollar
    ///          Tiers information
    fn soft_dollar_tiers(&mut self, req_id: i32, tiers: Vec<SoftDollarTier>);

    //----------------------------------------------------------------------------------------------
    /// returns array of family codes
    fn family_codes(&mut self, family_codes: Vec<FamilyCode>);

    //----------------------------------------------------------------------------------------------
    /// returns array of sample contract descriptions
    fn symbol_samples(&mut self, req_id: i32, contract_descriptions: Vec<ContractDescription>);

    //----------------------------------------------------------------------------------------------
    /// returns array of exchanges which return depth to UpdateMktDepthL2
    fn mkt_depth_exchanges(&mut self, depth_mkt_data_descriptions: Vec<DepthMktDataDescription>);

    //----------------------------------------------------------------------------------------------
    /// returns news headlines
    fn tick_news(
        &mut self,
        ticker_id: i32,
        time_stamp: i32,
        provider_code: &str,
        article_id: &str,
        headline: &str,
        extra_data: &str,
    );

    //----------------------------------------------------------------------------------------------
    /// returns exchange component mapping
    fn smart_components(&mut self, req_id: i32, smart_components: Vec<SmartComponent>);

    //----------------------------------------------------------------------------------------------
    /// returns exchange map of a particular contract
    fn tick_req_params(
        &mut self,
        ticker_id: i32,
        min_tick: f64,
        bbo_exchange: &str,
        snapshot_permissions: i32,
    );

    //----------------------------------------------------------------------------------------------
    /// returns available, subscribed API news providers
    fn news_providers(&mut self, news_providers: Vec<NewsProvider>);

    //----------------------------------------------------------------------------------------------
    /// returns body of news article
    fn news_article(&mut self, request_id: i32, article_type: i32, article_text: &str);

    //----------------------------------------------------------------------------------------------
    /// returns historical news headlines
    fn historical_news(
        &mut self,
        request_id: i32,
        time: &str,
        provider_code: &str,
        article_id: &str,
        headline: &str,
    );

    //----------------------------------------------------------------------------------------------
    /// signals end of historical news
    fn historical_news_end(&mut self, request_id: i32, has_more: bool);

    //----------------------------------------------------------------------------------------------
    /// returns earliest available data of a type of data for a particular contract
    fn head_timestamp(&mut self, req_id: i32, head_timestamp: &str);

    //----------------------------------------------------------------------------------------------
    /// returns histogram data for a contract
    fn histogram_data(&mut self, req_id: i32, items: Vec<HistogramData>);

    //----------------------------------------------------------------------------------------------
    /// returns updates in real time when keepUpToDate is set to True
    fn historical_data_update(&mut self, req_id: i32, bar: BarData);

    //----------------------------------------------------------------------------------------------
    /// returns reroute cfd contract information for market data request
    fn reroute_mkt_data_req(&mut self, req_id: i32, con_id: i32, exchange: &str);

    //----------------------------------------------------------------------------------------------
    /// returns reroute cfd contract information for market depth request
    fn reroute_mkt_depth_req(&mut self, req_id: i32, con_id: i32, exchange: &str);

    //----------------------------------------------------------------------------------------------
    /// returns minimum price increment structure for a particular market rule ID
    fn market_rule(&mut self, market_rule_id: i32, price_increments: Vec<PriceIncrement>);

    //----------------------------------------------------------------------------------------------
    /// returns the daily PnL for the account
    fn pnl(&mut self, req_id: i32, daily_pn_l: f64, unrealized_pn_l: f64, realized_pn_l: f64);

    //----------------------------------------------------------------------------------------------
    /// returns the daily PnL for a single position in the account
    fn pnl_single(
        &mut self,
        req_id: i32,
        pos: i32,
        daily_pn_l: f64,
        unrealized_pn_l: f64,
        realized_pn_l: f64,
        value: f64,
    );

    //----------------------------------------------------------------------------------------------
    /// returns historical tick data when what_to_how=MIDPOINT
    fn historical_ticks(&mut self, req_id: i32, ticks: Vec<HistoricalTick>, done: bool);

    //----------------------------------------------------------------------------------------------
    /// returns historical tick data when what_to_how=BID_ASK
    fn historical_ticks_bid_ask(
        &mut self,
        req_id: i32,
        ticks: Vec<HistoricalTickBidAsk>,
        done: bool,
    );

    //----------------------------------------------------------------------------------------------
    /// returns historical tick data when what_to_how=TRADES
    fn historical_ticks_last(&mut self, req_id: i32, ticks: Vec<HistoricalTickLast>, done: bool);

    //----------------------------------------------------------------------------------------------
    /// returns tick-by-tick data for tickType = "Last" or "AllLast"
    fn tick_by_tick_all_last(
        &mut self,
        req_id: i32,
        tick_type: TickByTickType,
        time: i64,
        price: f64,
        size: i32,
        tick_attrib_last: TickAttribLast,
        exchange: &str,
        special_conditions: &str,
    );

    //----------------------------------------------------------------------------------------------
    /// returns tick-by-tick data for TickAttribBidAsk
    fn tick_by_tick_bid_ask(
        &mut self,
        req_id: i32,
        time: i64,
        bid_price: f64,
        ask_price: f64,
        bid_size: i32,
        ask_size: i32,
        tick_attrib_bid_ask: TickAttribBidAsk,
    );

    //----------------------------------------------------------------------------------------------
    /// returns tick-by-tick data for tickType = "MidPoint"
    fn tick_by_tick_mid_point(&mut self, req_id: i32, time: i64, mid_point: f64);

    //----------------------------------------------------------------------------------------------
    /// returns order_bound notification
    fn order_bound(&mut self, req_id: i32, api_client_id: i32, api_order_id: i32);

    //----------------------------------------------------------------------------------------------
    /// This function is called to feed in completed orders.
    ///
    /// # Arguments
    /// * contract - The Contract class attributes describe the contract.
    /// * order - The Order class gives the details of the completed order.
    /// * orderState: OrderState - The orderState class includes completed order status details.
    ///
    fn completed_order(&mut self, contract: Contract, order: Order, order_state: OrderState);

    //----------------------------------------------------------------------------------------------
    /// This is called at the end of a given request for completed orders.
    fn completed_orders_end(&mut self);
}
