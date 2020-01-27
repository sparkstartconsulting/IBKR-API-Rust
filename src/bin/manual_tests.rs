use std::borrow::{Borrow, BorrowMut};
use std::collections::HashSet;
use std::marker::{Send, Sync};
use std::ops::Deref;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, UNIX_EPOCH};

use bigdecimal::BigDecimal;
use chrono;
use chrono::{DateTime, Duration as ChronoDuration, TimeZone, Utc};
use log::*;

use twsapi::core::account_summary_tags::AccountSummaryTags;
use twsapi::core::algo_params::{
    fill_accumulate_distribute_params, fill_adaptive_params, fill_arrival_price_params,
    fill_balance_impact_risk_params, fill_close_price_params, fill_csfbinline_params,
    fill_dark_ice_params, fill_jefferies_vwapparams, fill_min_impact_params, fill_pct_vol_params,
    fill_price_variant_pct_vol_params, fill_qbalgo_in_line_params, fill_scale_params,
    fill_size_variant_pct_vol_params, fill_time_variant_pct_vol_params, fill_twap_params,
    fill_vwap_params,
};
use twsapi::core::client::EClient;
use twsapi::core::common::{
    BarData, CommissionReport, DepthMktDataDescription, FaDataType, FamilyCode, HistogramData,
    HistoricalTick, HistoricalTickBidAsk, HistoricalTickLast, MarketDataTypeEnum, NewsProvider,
    PriceIncrement, SmartComponent, TagValue, TickAttrib, TickAttribBidAsk, TickAttribLast,
    TickByTickType, TickType, UNSET_DOUBLE,
};
use twsapi::core::contract::{
    Contract, ContractDescription, ContractDetails, DeltaNeutralContract,
};
use twsapi::core::errors::IBKRApiLibError;
use twsapi::core::execution::{Execution, ExecutionFilter};
use twsapi::core::messages::IncomingMessageIds::MarketDataType;
use twsapi::core::order::{Order, OrderState, SoftDollarTier};
use twsapi::core::order_condition::{PriceCondition, TriggerMethod};
use twsapi::core::wrapper::Wrapper;
use twsapi::examples::contract_samples;
use twsapi::examples::fa_allocation_samples;
use twsapi::examples::order_samples;
use twsapi::examples::scanner_subscription_samples;

//==================================================================================================
/// Example implementation of the Wrapper callback trait.  Just logs callback methods
pub struct TestWrapper {
    pub client: Option<Arc<Mutex<EClient<TestWrapper>>>>,
    pub next_order_id: i32,
    account: String,
}

impl TestWrapper {
    pub fn new() -> Self {
        TestWrapper {
            client: None,
            next_order_id: -1,
            account: "".to_string(),
        }
    }

    //----------------------------------------------------------------------------------------------
    pub fn start_requests(&mut self) -> Result<(), IBKRApiLibError> {
        //self.order_operations_req(); //tested ok
        //self.what_if_order_operations(); //tested ok
        //self.account_operations_req(); //tested ok
        //self.market_data_type_operations(); //tested ok
        //self.tick_data_operations_req(); //tested ok
        //self.market_depth_operations_req(); //tested ok
        //self.real_time_bars_operations_req(); // Tested ok
        //self.historical_data_operations_req(); // Tested ok
        //self.options_operations_req(); tested ok
        // self.market_scanners_perations_req(); testd ok
        // self.fundamentals_operations_req(); //retest with research data subscription
        //self.contract_operations(); tested ok
        //self.tick_by_tick_operations_req(); //tested ok
        // self.historical_ticks_operations(); //tested ok
        //self.histogram_operations_req(); //tested ok
        // self.continuous_futures_operations_req(); //tested ok
        // self.pnl_operations_req(); //tested ok
        // self.market_rule_operations(); //testd ok
        // self.reroute_cfd_operations(); //tested ok
        //self.financial_advisor_operations(); ****************************RETEST
        // self.news_operations_req(); //retest with news subscription.
        // self.bulletins_operations_req(); //tested ok
        //self.miscelaneous_operations(); //tested ok
        //self.linking_operations(); //tested ok
        //self.financial_advisor_operations();
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    fn account_operations_req(&self) {
        // Requesting managed accounts
        // ! [reqmanagedaccts]
        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .req_managed_accts();
        // ! [reqmanagedaccts]

        // Requesting family codes
        // ! [reqfamilycodes]
        {
            self.client
                .as_ref()
                .unwrap()
                .lock()
                .unwrap()
                .req_family_codes();
        }
        // ! [reqfamilycodes]
        //
        // Requesting accounts' summary
        // ! [reqaaccountsummary]
        {
            let all_tags = AccountSummaryTags::AllTags.to_string();
            self.client
                .as_ref()
                .unwrap()
                .lock()
                .unwrap()
                .req_account_summary(9001, "All", all_tags.as_str());
        }
        //        // ! [reqaaccountsummary]
        //
        //        // ! [reqaaccountsummaryledger]
        {
            self.client
                .as_ref()
                .unwrap()
                .lock()
                .unwrap()
                .req_account_summary(9002, "All", "$LEDGER");
        }
        //        // ! [reqaaccountsummaryledger]
        //
        //        // ! [reqaaccountsummaryledgercurrency]
        {
            self.client
                .as_ref()
                .unwrap()
                .lock()
                .unwrap()
                .req_account_summary(9003, "All", "$LEDGER:EUR");
        }
        //        // ! [reqaaccountsummaryledgercurrency]
        //
        //        // ! [reqaaccountsummaryledgerall]
        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .req_account_summary(9004, "All", "$LEDGER:ALL");
        //        // ! [reqaaccountsummaryledgerall]
        //
        //        // Subscribing to an account's information.Only one at a time!
        //        // ! [reqaaccountupdates]
        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .req_account_updates(true, self.account.as_str());
        //        // ! [reqaaccountupdates]
        //
        //        // ! [reqaaccountupdatesmulti]
        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .req_account_updates_multi(9005, self.account.as_str(), "", true);

        //        // Requesting all accounts' positions.
        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .req_positions();
        //        // ! [reqpositions]
        //
        //        // ! [reqpositionsmulti]
        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .req_positions_multi(9006, &self.account, "");
        //        // ! [reqpositionsmulti]
    }

    //----------------------------------------------------------------------------------------------
    pub fn real_time_bars_operations_req(&self) {
        // Requesting real time bars
        // // ![reqrealtimebars]
        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .req_real_time_bars(
                3001,
                contract_samples::us_stock_at_smart().borrow(),
                1,
                "TRADES",
                true,
                vec![],
            );
        // // ![reqrealtimebars]
    }

    //----------------------------------------------------------------------------------------------
    fn order_operations_req(&mut self) {
        // Requesting the next valid id
        // ! [reqids]
        // The parameter is always ignored.
        info!("order_operations_req");
        //        {
        //            info!("req_ids...");
        //            self.client.as_ref().unwrap().lock().unwrap().req_ids(-1);
        //            info!("finished req_ids...");
        //        }
        // ! [reqids]

        // Requesting all open orders
        // ! [reqallopenorders]
        {
            info!("req_all_open_orders...");
            self.client
                .as_ref()
                .unwrap()
                .lock()
                .unwrap()
                .req_all_open_orders();
        }
        // ! [reqallopenorders]

        // Taking over orders to be submitted via TWS
        // ! [reqautoopenorders]
        info!("req_auto_open_orders...");
        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .req_auto_open_orders(true);
        // ! [reqautoopenorders]

        // Requesting this API client's orders
        // ! [reqopenorders]
        info!("req_open_orders...");
        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .req_open_orders();
        // ! [reqopenorders]

        // Placing/ modifying an order - remember to ALWAYS increment the
        // nextValidId after placing an order so it can be used for the next one!
        // Note if there are multiple clients connected to an account, the
        // order ID must also be greater than all order IDs returned for orders
        // to orderStatus and openOrder to this client.

        // ! [order_submission]
        {
            let next_id = self.next_order_id();
            info!("Placing order... {}", next_id);

            self.client.as_ref().unwrap().lock().unwrap().place_order(
                next_id,
                &contract_samples::usstock().borrow(),
                order_samples::limit_order("SELL", 1.0, 50.0).borrow(),
            );
        }

        //thread::sleep(Duration::from_secs(2));

        //        // ! [order_submission]

        // ! [faorderoneaccount]
        let mut fa_order_one_account = order_samples::market_order("BUY", 100.0);
        // Specify the Account Number directly
        fa_order_one_account.account = "DU228250".to_string();
        {
            let next_id = self.next_order_id();
            self.client.as_ref().unwrap().lock().unwrap().place_order(
                next_id,
                &contract_samples::usstock().borrow(),
                fa_order_one_account.borrow(),
            );
        }

        // ! [faorderoneaccount]
        //
        //        // ! [faordergroupequalquantity]
        let mut fa_order_group_eq = order_samples::limit_order("SELL", 200.0, 2000.0);
        fa_order_group_eq.fa_group = "Group_Equal_Quantity".to_string();
        fa_order_group_eq.fa_method = "EqualQuantity".to_string();

        let next_id = self.next_order_id();
        self.client.as_ref().unwrap().lock().unwrap().place_order(
            next_id,
            &contract_samples::simple_future(),
            fa_order_group_eq.borrow(),
        );
        //        // ! [faordergroupequalquantity]
        //
        //        // ! [faordergrouppctchange]
        let mut fa_order_group_pc = order_samples::market_order("BUY", 0.0);
        // You should not specify any order quantity for PctChange allocation method
        fa_order_group_pc.fa_group = "Pct_Change".to_string();
        fa_order_group_pc.fa_method = "PctChange".to_string();
        fa_order_group_pc.fa_percentage = "100".to_string();

        let next_id = self.next_order_id();
        self.client.as_ref().unwrap().lock().unwrap().place_order(
            next_id,
            &contract_samples::eur_gbp_fx(),
            fa_order_group_pc.borrow(),
        );
        //        // ! [faordergrouppctchange]
        //
        //        // ! [faorderprofile]
        let mut fa_order_profile = order_samples::limit_order("BUY", 200.0, 100.0);
        fa_order_profile.fa_profile = "Percent_60_40".to_string();

        let next_id = self.next_order_id();
        self.client.as_ref().unwrap().lock().unwrap().place_order(
            next_id,
            &contract_samples::european_stock(),
            fa_order_profile.borrow(),
        );
        //        // ! [faorderprofile]
        //
        //        // ! [modelorder]
        let mut model_order = order_samples::limit_order("BUY", 200.0, 100.0);
        model_order.account = "DF12345".to_string();
        model_order.model_code = "Technology".to_string(); // model for tech stocks first created in TWS

        let next_id = self.next_order_id();
        self.client.as_ref().unwrap().lock().unwrap().place_order(
            next_id,
            &contract_samples::usstock().borrow(),
            model_order.borrow(),
        );
        //        // ! [modelorder]
        //

        let next_id = self.next_order_id();
        self.client.as_ref().unwrap().lock().unwrap().place_order(
            next_id,
            &contract_samples::option_at_box(),
            order_samples::block("BUY", 50.0, 20.0).borrow(),
        );

        let next_id = self.next_order_id();
        self.client.as_ref().unwrap().lock().unwrap().place_order(
            next_id,
            &contract_samples::option_at_box(),
            order_samples::box_top("SELL", 10.0).borrow(),
        );

        let next_id = self.next_order_id();
        self.client.as_ref().unwrap().lock().unwrap().place_order(
            next_id,
            &contract_samples::future_combo_contract(),
            order_samples::combo_limit_order("SELL", 1.0, 1.0, false).borrow(),
        );

        let next_id = self.next_order_id();
        self.client.as_ref().unwrap().lock().unwrap().place_order(
            next_id,
            &contract_samples::stock_combo_contract(),
            order_samples::combo_market_order("BUY", 1.0, true).borrow(),
        );

        let next_id = self.next_order_id();
        self.client.as_ref().unwrap().lock().unwrap().place_order(
            next_id,
            &contract_samples::option_combo_contract(),
            order_samples::combo_market_order("BUY", 1.0, false).borrow(),
        );

        let next_id = self.next_order_id();
        self.client.as_ref().unwrap().lock().unwrap().place_order(
            next_id,
            &contract_samples::stock_combo_contract(),
            &order_samples::limit_order_for_combo_with_leg_prices(
                "BUY",
                1.0,
                vec![10.0, 5.0],
                true,
            ),
        );

        let next_id = self.next_order_id();
        self.client.as_ref().unwrap().lock().unwrap().place_order(
            next_id,
            &contract_samples::usstock().borrow(),
            order_samples::discretionary("SELL", 1.0, 45.0, 0.5).borrow(),
        );

        let next_id = self.next_order_id();
        self.client.as_ref().unwrap().lock().unwrap().place_order(
            next_id,
            &contract_samples::option_at_box(),
            order_samples::limit_if_touched("BUY", 1.0, 30.0, 34.0).borrow(),
        );

        let next_id = self.next_order_id();
        self.client.as_ref().unwrap().lock().unwrap().place_order(
            next_id,
            &contract_samples::usstock().borrow(),
            order_samples::limit_on_close("SELL", 1.0, 34.0).borrow(),
        );

        let next_id = self.next_order_id();
        self.client.as_ref().unwrap().lock().unwrap().place_order(
            next_id,
            &contract_samples::usstock().borrow(),
            order_samples::limit_on_open("BUY", 1.0, 35.0).borrow(),
        );

        let next_id = self.next_order_id();
        self.client.as_ref().unwrap().lock().unwrap().place_order(
            next_id,
            &contract_samples::usstock().borrow(),
            order_samples::market_if_touched("BUY", 1.0, 30.0).borrow(),
        );

        let next_id = self.next_order_id();
        self.client.as_ref().unwrap().lock().unwrap().place_order(
            next_id,
            &contract_samples::usstock().borrow(),
            order_samples::market_on_close("SELL", 1.0).borrow(),
        );

        let next_id = self.next_order_id();
        self.client.as_ref().unwrap().lock().unwrap().place_order(
            next_id,
            &contract_samples::usstock().borrow(),
            order_samples::market_on_open("BUY", 1.0).borrow(),
        );

        let next_id = self.next_order_id();
        self.client.as_ref().unwrap().lock().unwrap().place_order(
            next_id,
            &contract_samples::usstock().borrow(),
            order_samples::market_order("SELL", 1.0).borrow(),
        );

        let next_id = self.next_order_id();
        self.client.as_ref().unwrap().lock().unwrap().place_order(
            next_id,
            &contract_samples::usstock().borrow(),
            order_samples::market_to_limit("BUY", 1.0).borrow(),
        );

        let next_id = self.next_order_id();
        self.client.as_ref().unwrap().lock().unwrap().place_order(
            next_id,
            &contract_samples::option_at_ise(),
            order_samples::midpoint_match("BUY", 1.0).borrow(),
        );

        let next_id = self.next_order_id();
        self.client.as_ref().unwrap().lock().unwrap().place_order(
            next_id,
            &contract_samples::usstock().borrow(),
            order_samples::market_to_limit("BUY", 1.0).borrow(),
        );

        let next_id = self.next_order_id();
        self.client.as_ref().unwrap().lock().unwrap().place_order(
            next_id,
            &contract_samples::usstock().borrow(),
            order_samples::stop("SELL", 1.0, 34.4).borrow(),
        );

        let next_id = self.next_order_id();
        self.client.as_ref().unwrap().lock().unwrap().place_order(
            next_id,
            &contract_samples::usstock().borrow(),
            &order_samples::stop_limit("BUY", 1.0, 35.0, 33.0),
        );

        let next_id = self.next_order_id();
        self.client.as_ref().unwrap().lock().unwrap().place_order(
            next_id,
            &contract_samples::simple_future(),
            order_samples::stop_with_protection("SELL", 1.0, 45.0).borrow(),
        );

        let next_id = self.next_order_id();
        self.client.as_ref().unwrap().lock().unwrap().place_order(
            next_id,
            &contract_samples::usstock().borrow(),
            order_samples::sweep_to_fill("BUY", 1.0, 35.0).borrow(),
        );

        let next_id = self.next_order_id();
        self.client.as_ref().unwrap().lock().unwrap().place_order(
            next_id,
            &contract_samples::usstock().borrow(),
            order_samples::trailing_stop("SELL", 1.0, 0.5, 30.0).borrow(),
        );

        let next_id = self.next_order_id();
        self.client.as_ref().unwrap().lock().unwrap().place_order(
            next_id,
            &contract_samples::usstock().borrow(),
            order_samples::trailing_stop_limit("BUY", 1.0, 2.0, 5.0, 50.0).borrow(),
        );

        let next_id = self.next_order_id();
        self.client.as_ref().unwrap().lock().unwrap().place_order(
            next_id,
            &contract_samples::us_option_contract(),
            &order_samples::volatility("SELL", 1.0, 5.0, 2),
        );

        //Interactive Broker's has a 50 messages per second limit, so sleep for 1 sec and continue placing orders
        thread::sleep(Duration::from_secs(1));

        self.algo_samples();
        self.bracket_sample();

        self.condition_samples();

        self.hedge_sample();
        //
        //        // NOTE: the following orders are not supported for Paper Trading
        //        // self.client.unwrap().lock().unwrap().place_order( self.next_order_id(), &contract_samples::usstock().borrow(), order_samples::AtAuction("BUY", 100, 30.0))
        //        // self.client.unwrap().lock().unwrap().place_order( self.next_order_id(), &contract_samples::OptionAtBOX(), order_samples::AuctionLimit("SELL", 10, 30.0, 2))
        //        // self.client.unwrap().lock().unwrap().place_order( self.next_order_id(), &contract_samples::OptionAtBOX(), order_samples::AuctionPeggedToStock("BUY", 10, 30, 0.5))
        //        // self.client.unwrap().lock().unwrap().place_order( self.next_order_id(), &contract_samples::OptionAtBOX(), order_samples::AuctionRelative("SELL", 10, 0.6))
        //        // self.client.unwrap().lock().unwrap().place_order( self.next_order_id(), &contract_samples::simple_future(), order_samples::MarketWithProtection("BUY", 1))
        //        // self.client.unwrap().lock().unwrap().place_order( self.next_order_id(), &contract_samples::usstock().borrow(), order_samples::PassiveRelative("BUY", 1, 0.5))
        //
        //        // 208813720 (GOOG)
        //        // self.client.unwrap().lock().unwrap().place_order( self.next_order_id(), &contract_samples::usstock().borrow(),
        //        // order_samples::PeggedToBenchmark("SELL", 100, 33, True, 0.1, 1, 208813720, "ISLAND", 750, 650, 800))
        //
        //        // STOP ADJUSTABLE ORDERS
        //        // Order stpParent = order_samples::Stop("SELL", 100, 30)
        //        // stpParent.OrderId = self.next_order_id()
        //        // self.client.unwrap().lock().unwrap().place_order(stpParent.OrderId, &contract_samples::EuropeanStock(), stpParent)
        //        // self.client.unwrap().lock().unwrap().place_order( self.next_order_id(), &contract_samples::EuropeanStock(), order_samples::AttachAdjustableToStop(stpParent, 35, 32, 33))
        //        // self.client.unwrap().lock().unwrap().place_order( self.next_order_id(), &contract_samples::EuropeanStock(), order_samples::AttachAdjustableToStopLimit(stpParent, 35, 33, 32, 33))
        //        // self.client.unwrap().lock().unwrap().place_order( self.next_order_id(), &contract_samples::EuropeanStock(), order_samples::AttachAdjustableToTrail(stpParent, 35, 32, 32, 1, 0))
        //
        //        // Order lmtParent = order_samples::limit_order("BUY", 100, 30)
        //        // lmtParent.OrderId = self.next_order_id()
        //        // self.client.unwrap().lock().unwrap().place_order(lmtParent.OrderId, &contract_samples::EuropeanStock(), lmtParent)
        //        // Attached TRAIL adjusted can only be attached to LMT parent orders.
        //        // self.client.unwrap().lock().unwrap().place_order( self.next_order_id(), &contract_samples::EuropeanStock(), order_samples::AttachAdjustableToTrailAmount(lmtParent, 34, 32, 33, 0.008))
        //        self.algo_samples();
        //
        self.oca_sample();

        // Request the day's executions
        // ! [reqexecutions]
        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .req_executions(10001, ExecutionFilter::default().borrow());
        // ! [reqexecutions]
        //
        //        // Requesting completed orders
        //        // ! [reqcompletedorders]
        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .req_completed_orders(false);
        // ! [reqcompletedorders]
    }

    //----------------------------------------------------------------------------------------------
    fn order_operations_cancel(&mut self) {
        if self.next_order_id != -1 {
            // ! [cancelorder]
            self.client
                .as_ref()
                .unwrap()
                .lock()
                .unwrap()
                .cancel_order(self.next_order_id);
            // ! [cancelorder]

            // Cancel all orders for all accounts
            // ! [reqglobalcancel]
            self.req_global_cancel();
            // ! [reqglobalcancel]
        }
    }

    //----------------------------------------------------------------------------------------------
    fn bracket_sample(&mut self) -> Result<(), IBKRApiLibError> {
        // BRACKET ORDER
        // ! [bracketsubmit]
        let bracket =
            order_samples::bracket_order(self.next_order_id(), "BUY", 100.0, 30.0, 40.0, 20.0);

        self.client.as_ref().unwrap().lock().unwrap().place_order(
            bracket.0.order_id,
            contract_samples::european_stock().borrow(),
            bracket.0.borrow(),
        );
        self.next_order_id();
        self.client.as_ref().unwrap().lock().unwrap().place_order(
            bracket.1.order_id,
            contract_samples::european_stock().borrow(),
            bracket.1.borrow(),
        );
        self.next_order_id();
        self.client.as_ref().unwrap().lock().unwrap().place_order(
            bracket.2.order_id,
            contract_samples::european_stock().borrow(),
            bracket.2.borrow(),
        );
        self.next_order_id();
        // ! [bracketsubmit]
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    fn condition_samples(&mut self) -> Result<(), IBKRApiLibError> {
        let mut mkt = order_samples::market_order("BUY", 100.0);
        // Order will become active if conditioning criteria is met
        mkt.conditions
            .push(twsapi::core::order_condition::OrderConditionEnum::Price(
                order_samples::price_condition(
                    twsapi::core::order_condition::TriggerMethod::Default as i32,
                    208813720,
                    "SMART",
                    600.0,
                    false,
                    false,
                ),
            ));
        mkt.conditions.push(
            twsapi::core::order_condition::OrderConditionEnum::Execution(
                order_samples::execution_condition("EUR.USD", "CASH", "IDEALPRO", true),
            ),
        );
        mkt.conditions
            .push(twsapi::core::order_condition::OrderConditionEnum::Margin(
                order_samples::margin_condition(30.0, true, false),
            ));
        mkt.conditions.push(
            twsapi::core::order_condition::OrderConditionEnum::PercentChange(
                order_samples::percentage_change_condition(15.0, 208813720, "SMART", true, true),
            ),
        );
        mkt.conditions
            .push(twsapi::core::order_condition::OrderConditionEnum::Time(
                order_samples::time_condition("20160118 23:59:59", true, false),
            ));
        mkt.conditions
            .push(twsapi::core::order_condition::OrderConditionEnum::Volume(
                order_samples::volume_condition(208813720, "SMART", true, 100000, true),
            ));
        let next_id = self.next_order_id();
        self.client.as_ref().unwrap().lock().unwrap().place_order(
            next_id,
            contract_samples::european_stock().borrow(),
            mkt.borrow(),
        );

        // ! [order_conditioning_activate]

        // Conditions can make the order active or cancel it. Only LMT orders can be conditionally canceled.
        // ! [order_conditioning_cancel]
        //        let mut lmt = order_samples::limit_order("BUY", 100.0, 20.0);
        //        // The active order will be cancelled if conditioning criteria is met
        //        lmt.conditions_cancel_order = true;
        //        lmt.conditions
        //            .push(twsapi::core::order_condition::OrderConditionEnum::Price(
        //                order_samples::price_condition(
        //                    TriggerMethod::Last as i32,
        //                    208813720,
        //                    "SMART",
        //                    600.0,
        //                    false,
        //                    false,
        //                ),
        //            ));

        //        let next_id = self.next_order_id();
        //        self.client.as_ref().unwrap().lock().unwrap().place_order(
        //            next_id,
        //            contract_samples::european_stock().borrow(),
        //            lmt.borrow(),
        //        );
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    fn hedge_sample(&mut self) -> Result<(), IBKRApiLibError> {
        let mut parent = order_samples::limit_order("BUY", 100.0, 10.0);
        parent.order_id = self.next_order_id();
        parent.transmit = false;
        // Hedge on the currency conversion
        let hedge = order_samples::market_fhedge(parent.order_id, "BUY");
        // Place the parent first...
        self.client.as_ref().unwrap().lock().unwrap().place_order(
            parent.order_id,
            contract_samples::european_stock().borrow(),
            parent.borrow(),
        );

        // Then the hedge order
        let next_id = self.next_order_id();
        self.client.as_ref().unwrap().lock().unwrap().place_order(
            next_id,
            contract_samples::eur_gbp_fx().borrow(),
            hedge.borrow(),
        );
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    fn algo_samples(&mut self) -> Result<(), IBKRApiLibError> {
        // ! [scale_order]
        let next_id = self.next_order_id();
        let mut scale_order = Order::default();
        order_samples::relative_pegged_to_primary("BUY", 70000.0, 189.0, 0.01);
        fill_scale_params(
            scale_order.borrow_mut(),
            2000,
            500,
            true,
            0.02,
            189.00,
            3600,
            2.00,
            true,
            10,
            40,
        );
        self.client.as_ref().unwrap().lock().unwrap().place_order(
            next_id,
            contract_samples::us_stock_at_smart().borrow(),
            scale_order.borrow(),
        );
        // ! [scale_order]

        thread::sleep(Duration::from_secs(1));

        // ! [algo_base_order]
        let mut base_order = order_samples::limit_order("BUY", 1000.0, 1.0);
        // ! [algo_base_order]

        // ! [arrivalpx]
        let next_id = self.next_order_id();
        fill_arrival_price_params(
            &mut base_order,
            0.1,
            "Aggressive",
            "09:00:00 CET",
            "16:00:00 CET",
            true,
            true,
            100000.0,
        );
        self.client.as_ref().unwrap().lock().unwrap().place_order(
            next_id,
            contract_samples::us_stock_at_smart().borrow(),
            base_order.borrow(),
        );
        // ! [arrivalpx]

        // ! [darkice]
        let next_id = self.next_order_id();
        fill_dark_ice_params(
            &mut base_order,
            10,
            "09:00:00 CET",
            "16:00:00 CET",
            true,
            100000.0,
        );
        self.client.as_ref().unwrap().lock().unwrap().place_order(
            next_id,
            contract_samples::us_stock_at_smart().borrow(),
            base_order.borrow(),
        );
        // ! [darkice]

        // ! [place_midprice]
        let next_id = self.next_order_id();
        self.client.as_ref().unwrap().lock().unwrap().place_order(
            next_id,
            contract_samples::us_stock_at_smart().borrow(),
            &order_samples::midprice("BUY", 1.0, 150.0),
        );
        // ! [place_midprice]

        // ! [ad]
        // The Time Zone in "StartTime" and "EndTime" attributes is ignored and always defaulted to GMT
        let next_id = self.next_order_id();
        fill_accumulate_distribute_params(
            &mut base_order,
            10,
            60,
            true,
            true,
            1,
            true,
            true,
            "20161010-12:00:00 GMT",
            "20161010-16:00:00 GMT",
        );
        self.client.as_ref().unwrap().lock().unwrap().place_order(
            next_id,
            contract_samples::us_stock_at_smart().borrow(),
            base_order.borrow(),
        );
        // ! [ad]

        // ! [twap]
        let next_id = self.next_order_id();
        fill_twap_params(
            &mut base_order,
            "Marketable",
            "09:00:00 CET",
            "16:00:00 CET",
            true,
            100000.0,
        );
        self.client.as_ref().unwrap().lock().unwrap().place_order(
            next_id,
            contract_samples::us_stock_at_smart().borrow(),
            base_order.borrow(),
        );
        // ! [twap]

        // ! [vwap]
        let next_id = self.next_order_id();
        fill_vwap_params(
            &mut base_order,
            0.2,
            "09:00:00 CET",
            "16:00:00 CET",
            true,
            true,
            100000.0,
        );
        self.client.as_ref().unwrap().lock().unwrap().place_order(
            next_id,
            contract_samples::us_stock_at_smart().borrow(),
            base_order.borrow(),
        );
        // ! [vwap]

        // ! [balanceimpactrisk]
        let next_id = self.next_order_id();
        fill_balance_impact_risk_params(&mut base_order, 0.1, "Aggressive", true);
        self.client.as_ref().unwrap().lock().unwrap().place_order(
            next_id,
            contract_samples::us_option_contract().borrow(),
            base_order.borrow(),
        );
        // ! [balanceimpactrisk]

        // ! [minimpact]
        let next_id = self.next_order_id();
        fill_min_impact_params(&mut base_order, 0.3);
        self.client.as_ref().unwrap().lock().unwrap().place_order(
            next_id,
            contract_samples::us_option_contract().borrow(),
            base_order.borrow(),
        );
        // ! [minimpact]

        // ! [adaptive]
        let next_id = self.next_order_id();
        fill_adaptive_params(&mut base_order, "Normal");
        self.client.as_ref().unwrap().lock().unwrap().place_order(
            next_id,
            contract_samples::us_stock_at_smart().borrow(),
            base_order.borrow(),
        );
        // ! [adaptive]

        // ! [closepx]
        let next_id = self.next_order_id();
        fill_close_price_params(
            &mut base_order,
            0.4,
            "Neutral",
            "20180926-06:06:49",
            true,
            100000.0,
        );
        self.client.as_ref().unwrap().lock().unwrap().place_order(
            next_id,
            contract_samples::us_stock_at_smart().borrow(),
            base_order.borrow(),
        );
        // ! [closepx]

        // ! [pctvol]
        let next_id = self.next_order_id();
        fill_pct_vol_params(
            &mut base_order,
            0.5,
            "12:00:00 EST",
            "14:00:00 EST",
            true,
            100000.0,
        );
        self.client.as_ref().unwrap().lock().unwrap().place_order(
            next_id,
            contract_samples::us_stock_at_smart().borrow(),
            base_order.borrow(),
        );
        // ! [pctvol]

        // ! [pctvolpx]
        let next_id = self.next_order_id();
        fill_price_variant_pct_vol_params(
            &mut base_order,
            0.1,
            0.05,
            0.01,
            0.2,
            "12:00:00 EST",
            "14:00:00 EST",
            true,
            100000.0,
        );
        self.client.as_ref().unwrap().lock().unwrap().place_order(
            next_id,
            contract_samples::us_stock_at_smart().borrow(),
            base_order.borrow(),
        );
        // ! [pctvolpx]

        // ! [pctvolsz]
        let next_id = self.next_order_id();
        fill_size_variant_pct_vol_params(
            &mut base_order,
            0.2,
            0.4,
            "12:00:00 EST",
            "14:00:00 EST",
            true,
            100000.0,
        );
        self.client.as_ref().unwrap().lock().unwrap().place_order(
            next_id,
            contract_samples::us_stock_at_smart().borrow(),
            base_order.borrow(),
        );
        // ! [pctvolsz]

        // ! [pctvoltm]
        let next_id = self.next_order_id();
        fill_time_variant_pct_vol_params(
            &mut base_order,
            0.2,
            0.4,
            "12:00:00 EST",
            "14:00:00 EST",
            true,
            100000.0,
        );
        self.client.as_ref().unwrap().lock().unwrap().place_order(
            next_id,
            contract_samples::us_stock_at_smart().borrow(),
            base_order.borrow(),
        );
        // ! [pctvoltm]

        // ! [jeff_vwap_algo]
        let next_id = self.next_order_id();
        fill_jefferies_vwapparams(
            &mut base_order,
            "10:00:00 EST",
            "16:00:00 EST",
            10.0,
            10.0,
            "Exclude_Both",
            130.0,
            135.0,
            1,
            10.0,
            "Patience",
            false,
            "Midpoint",
        );
        self.client.as_ref().unwrap().lock().unwrap().place_order(
            next_id,
            contract_samples::jefferies_contract().borrow(),
            base_order.borrow(),
        );
        // ! [jeff_vwap_algo]

        // ! [csfb_inline_algo]
        let next_id = self.next_order_id();
        fill_csfbinline_params(
            &mut base_order,
            "10:00:00 EST",
            "16:00:00 EST",
            "Patient",
            10,
            20,
            100,
            "Default",
            false,
            40.0,
            100,
            100,
            35.0,
        );
        self.client.as_ref().unwrap().lock().unwrap().place_order(
            next_id,
            contract_samples::csfbcontract().borrow(),
            base_order.borrow(),
        );
        // ! [csfb_inline_algo]

        // ! [qbalgo_strobe_algo]
        let next_id = self.next_order_id();
        fill_qbalgo_in_line_params(
            &mut base_order,
            "10:00:00 EST",
            "16:00:00 EST",
            99.0,
            "TWAP",
            0.25,
            true,
        );
        self.client.as_ref().unwrap().lock().unwrap().place_order(
            next_id,
            contract_samples::qbalgo_contract().borrow(),
            base_order.borrow(),
        );
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    fn oca_sample(&mut self) -> Result<(), IBKRApiLibError> {
        let oca_orders = vec![
            order_samples::limit_order("BUY", 1.0, 10.0),
            order_samples::limit_order("BUY", 1.0, 11.0),
            order_samples::limit_order("BUY", 1.0, 12.0),
        ];
        order_samples::one_cancels_all(
            format!("TestOCA_{}", self.next_order_id()).as_ref(),
            oca_orders.clone(),
            2,
        );
        for o in oca_orders {
            let next_id = self.next_order_id();
            self.client.as_ref().unwrap().lock().unwrap().place_order(
                next_id,
                contract_samples::us_stock_at_smart().borrow(),
                o.borrow(),
            );
        }
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    fn what_if_order_operations(&mut self) {
        //// ! [whatiflimitorder]
        let mut what_if_order = order_samples::limit_order("SELL", 5.0, 70.0);
        what_if_order.what_if = true;
        let next_id = self.next_order_id();
        self.client.as_ref().unwrap().lock().unwrap().place_order(
            next_id,
            contract_samples::us_stock_at_smart().borrow(),
            what_if_order.borrow(),
        );
        //// ! [whatiflimitorder]
        thread::sleep(Duration::from_secs(2));
    }

    //----------------------------------------------------------------------------------------------
    fn req_global_cancel(&self) -> Result<(), IBKRApiLibError> {
        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .req_global_cancel();
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    fn next_order_id(&mut self) -> i32 {
        let oid = self.next_order_id;
        self.next_order_id += 1;
        oid
    }

    //----------------------------------------------------------------------------------------------
    pub fn historical_data_operations_req(&self) {
        // // Requesting historical data
        // // ![reqHeadTimeStamp]
        self.client
            .as_ref()
            .unwrap()
            .try_lock()
            .unwrap()
            .req_head_time_stamp(
                4101,
                contract_samples::simple_future().borrow(),
                "TRADES",
                0,
                1,
            );
        //// ![reqHeadTimeStamp]

        //// ![reqhistoricaldata]
        let dt = Utc::now();
        let query_time = dt.format("%Y%m%d %H:%M:%S").to_string();
        info!("Request Time:  {}", query_time);
        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .req_historical_data(
                4102,
                contract_samples::simple_future().borrow(),
                query_time.as_str(),
                "1 M",
                "1 day",
                "MIDPOINT",
                1,
                1,
                false,
                vec![],
            );
        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .req_historical_data(
                4103,
                contract_samples::simple_future().borrow(),
                query_time.as_str(),
                "10 D",
                "1 min",
                "TRADES",
                1,
                1,
                false,
                vec![],
            );
        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .req_historical_data(
                4104,
                contract_samples::eur_gbp_fx().borrow(),
                "",
                "1 M",
                "1 day",
                "MIDPOINT",
                1,
                1,
                true,
                vec![],
            );
        //// ![reqhistoricaldata]
    }

    fn market_data_type_operations(&self) {
        // ! [reqmarketdatatype]
        // Switch to live (1) frozen (2) delayed (3) delayed frozen (4).
        let result = self
            .client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .req_market_data_type(MarketDataTypeEnum::Realtime as i32);

        if result.is_err() {
            error!("{}", result.unwrap_err())
        }
        // ! [reqmarketdatatype]
    }

    fn tick_data_operations_req(&self) {
        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .req_market_data_type(MarketDataTypeEnum::DelayedFrozen as i32);

        // Requesting real time market data

        // ! [reqmktdata]
        self.client.as_ref().unwrap().lock().unwrap().req_mkt_data(
            1000,
            contract_samples::us_stock_at_smart().borrow(),
            "",
            false,
            false,
            vec![],
        );
        self.client.as_ref().unwrap().lock().unwrap().req_mkt_data(
            1001,
            contract_samples::stock_combo_contract().borrow(),
            "",
            false,
            false,
            vec![],
        );
        // ! [reqmktdata]

        // ! [reqmktdata_snapshot]
        self.client.as_ref().unwrap().lock().unwrap().req_mkt_data(
            1002,
            contract_samples::future_combo_contract().borrow(),
            "",
            true,
            false,
            vec![],
        );
        // ! [reqmktdata_snapshot]

        // ! [regulatorysnapshot]
        // Each regulatory snapshot request incurs a 0.01 USD fee
        self.client.as_ref().unwrap().lock().unwrap().req_mkt_data(
            1003,
            contract_samples::usstock().borrow(),
            "",
            false,
            true,
            vec![],
        );
        // ! [regulatorysnapshot]

        // ! [reqmktdata_genticks]
        // Requesting RTVolume (Time & Sales), shortable and Fundamental Ratios generic ticks
        self.client.as_ref().unwrap().lock().unwrap().req_mkt_data(
            1004,
            contract_samples::us_stock_at_smart().borrow(),
            "233,236,258",
            false,
            false,
            vec![],
        );
        // ! [reqmktdata_genticks]

        // ! [reqmktdata_contractnews]
        // Without the API news subscription this will generate an "invalid tick type" error
        self.client.as_ref().unwrap().lock().unwrap().req_mkt_data(
            1005,
            contract_samples::us_stock_at_smart().borrow(),
            "100",
            false,
            false,
            vec![],
        );
        self.client.as_ref().unwrap().lock().unwrap().req_mkt_data(
            1006,
            contract_samples::us_stock_at_smart().borrow(),
            "mdoff,292:BRFG+DJNL",
            false,
            false,
            vec![],
        );
        self.client.as_ref().unwrap().lock().unwrap().req_mkt_data(
            1007,
            contract_samples::us_stock_at_smart().borrow(),
            "mdoff,292:BRFUPDN",
            false,
            false,
            vec![],
        );
        self.client.as_ref().unwrap().lock().unwrap().req_mkt_data(
            1008,
            contract_samples::us_stock_at_smart().borrow(),
            "mdoff,292:DJ-RT",
            false,
            false,
            vec![],
        );
        // ! [reqmktdata_contractnews]

        // ! [reqmktdata_broadtapenews]
        self.client.as_ref().unwrap().lock().unwrap().req_mkt_data(
            1009,
            contract_samples::brfgbroadtape_news_feed().borrow(),
            "mdoff,292",
            false,
            false,
            vec![],
        );
        self.client.as_ref().unwrap().lock().unwrap().req_mkt_data(
            1010,
            contract_samples::djnlbroadtape_news_feed().borrow(),
            "mdoff,292",
            false,
            false,
            vec![],
        );
        self.client.as_ref().unwrap().lock().unwrap().req_mkt_data(
            1011,
            contract_samples::djtopbroadtape_news_feed().borrow(),
            "mdoff,292",
            false,
            false,
            vec![],
        );
        self.client.as_ref().unwrap().lock().unwrap().req_mkt_data(
            1012,
            contract_samples::brfupdnbroadtape_news_feed().borrow(),
            "mdoff,292",
            false,
            false,
            vec![],
        );
        // ! [reqmktdata_broadtapenews]

        // ! [reqoptiondatagenticks]
        // Requesting data for an option contract will return the greek values
        self.client.as_ref().unwrap().lock().unwrap().req_mkt_data(
            1013,
            contract_samples::option_with_local_symbol().borrow(),
            "",
            false,
            false,
            vec![],
        );
        self.client.as_ref().unwrap().lock().unwrap().req_mkt_data(
            1014,
            contract_samples::futures_on_options().borrow(),
            "",
            false,
            false,
            vec![],
        );

        // ! [reqoptiondatagenticks]

        // ! [reqfuturesopeninterest]
        self.client.as_ref().unwrap().lock().unwrap().req_mkt_data(
            1015,
            contract_samples::simple_future().borrow(),
            "mdoff,588",
            false,
            false,
            vec![],
        );
        // ! [reqfuturesopeninterest]

        // ! [reqmktdatapreopenbidask]
        self.client.as_ref().unwrap().lock().unwrap().req_mkt_data(
            1016,
            contract_samples::simple_future().borrow(),
            "",
            false,
            false,
            vec![],
        );
        // ! [reqmktdatapreopenbidask]

        // ! [reqavgoptvolume]
        self.client.as_ref().unwrap().lock().unwrap().req_mkt_data(
            1017,
            contract_samples::us_stock_at_smart().borrow(),
            "mdoff,105",
            false,
            false,
            vec![],
        );
        // ! [reqavgoptvolume]

        // ! [reqsmartcomponents]
        // Requests description of map of single letter exchange codes to full exchange names
        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .req_smart_components(1018, "a6");
        // ! [reqsmartcomponents]
    }

    fn market_depth_operations_req(&self) {
        // Requesting the Deep Book
        // ! [reqmarketdepth]
        self.client.as_ref().unwrap().lock().unwrap().req_mkt_depth(
            2001,
            contract_samples::eur_gbp_fx().borrow(),
            5,
            false,
            vec![],
        );
        // ! [reqmarketdepth]

        // ! [reqmarketdepth]
        self.client.as_ref().unwrap().lock().unwrap().req_mkt_depth(
            2002,
            contract_samples::us_stock_at_smart().borrow(),
            5,
            false,
            vec![],
        );
        // ! [reqmarketdepth]

        // Request list of exchanges sending market depth to UpdateMktDepthL2()
        // ! [reqMktDepthExchanges]
        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .req_mkt_depth_exchanges();
        // ! [reqMktDepthExchanges]
    }

    //----------------------------------------------------------------------------------------------
    fn options_operations_req(&self) {
        // ! [reqsecdefoptparams]
        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .req_sec_def_opt_params(0, "IBM", "", "STK", 8314);
        // ! [reqsecdefoptparams]

        // Calculating implied volatility
        // ! [calculateimpliedvolatility]
        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .calculate_implied_volatility(
                5001,
                contract_samples::option_at_box().borrow(),
                5.0,
                85.0,
                vec![],
            );
        // ! [calculateimpliedvolatility]

        // Calculating option's price
        // ! [calculateoptionprice]
        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .calculate_option_price(
                5002,
                contract_samples::option_at_box().borrow(),
                0.22,
                85.0,
                vec![],
            );
        // ! [calculateoptionprice]

        // Exercising options
        // ! [exercise_options]
        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .exercise_options(
                5003,
                contract_samples::option_with_trading_class().borrow(),
                1,
                1,
                &self.account,
                1,
            );
    }

    fn market_scanners_perations_req(&mut self) {
        // Requesting list of valid scanner parameters which can be used in TWS
        // ! [reqscannerparameters]
        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .req_scanner_parameters();
        // ! [reqscannerparameters]

        // Triggering a scanner subscription
        // ! [reqscannersubscription]
        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .req_scanner_subscription(
                7001,
                scanner_subscription_samples::high_opt_volume_pcratio_usindexes(),
                vec![],
                vec![],
            );
        //
        //        // Generic Filters
        let mut tagvalues = vec![];
        tagvalues.push(TagValue::new(
            "usdMarketCapAbove".to_string(),
            "10000".to_string(),
        ));
        tagvalues.push(TagValue::new(
            "optVolumeAbove".to_string(),
            "1000".to_string(),
        ));
        tagvalues.push(TagValue::new(
            "avgVolumeAbove".to_string(),
            "10000".to_string(),
        ));

        let result = self
            .client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .req_scanner_subscription(
                7002,
                scanner_subscription_samples::hot_usstk_by_volume(),
                vec![],
                tagvalues,
            ); // requires TWS v973 +
        if result.is_err() {
            match result.unwrap_err() {
                IBKRApiLibError::ApiError(err) => self.error(
                    err.req_id,
                    err.code.as_str().parse().unwrap(),
                    err.description.as_ref(),
                ),
                _ => {}
            }
        }
        // ! [reqscannersubscription]

        // ! [reqcomplexscanner]
        let aaplcon_idtag = vec![TagValue::new(
            "underConID".to_string(),
            "265598".to_string(),
        )];
        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .req_scanner_subscription(
                7003,
                scanner_subscription_samples::complex_orders_and_trades(),
                vec![],
                aaplcon_idtag,
            ); // requires TWS v975 +
    }

    //----------------------------------------------------------------------------------------------
    fn fundamentals_operations_req(&self) {
        // Requesting Fundamentals
        // ! [reqfundamentaldata]
        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .req_fundamental_data(
                8001,
                contract_samples::usstock().borrow(),
                "ReportsFinSummary",
                vec![],
            );
        // ! [reqfundamentaldata]

        // ! [fundamentalexamples]
        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .req_fundamental_data(
                8002,
                contract_samples::us_stock_at_smart().borrow(),
                "ReportSnapshot",
                vec![],
            ); // for company overview
        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .req_fundamental_data(
                8003,
                contract_samples::us_stock_at_smart().borrow(),
                "ReportRatios",
                vec![],
            ); // for financial ratios
        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .req_fundamental_data(
                8004,
                contract_samples::us_stock_at_smart().borrow(),
                "ReportsFinStatements",
                vec![],
            ); // for financial statements
        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .req_fundamental_data(
                8005,
                contract_samples::us_stock_at_smart().borrow(),
                "RESC",
                vec![],
            ); // for analyst estimates
        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .req_fundamental_data(
                8006,
                contract_samples::us_stock_at_smart().borrow(),
                "CalendarReport",
                vec![],
            );
        // for company calendar
    }

    //----------------------------------------------------------------------------------------------
    fn contract_operations(&self) {
        // ! [reqcontractdetails]
        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .req_contract_details(210, contract_samples::option_for_query().borrow());
        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .req_contract_details(211, contract_samples::eur_gbp_fx().borrow());
        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .req_contract_details(212, contract_samples::bond().borrow());
        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .req_contract_details(213, contract_samples::futures_on_options().borrow());
        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .req_contract_details(214, contract_samples::simple_future().borrow());
        // ! [reqcontractdetails]

        // ! [reqmatchingsymbols]
        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .req_matching_symbols(211, "IB");
    }

    //----------------------------------------------------------------------------------------------
    fn tick_by_tick_operations_req(&self) {
        // Requesting tick - by - tick data (only refresh)
        // ! [reqtickbytick]
        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .req_tick_by_tick_data(
                19001,
                contract_samples::usstock().borrow(),
                TickByTickType::AllLast,
                0,
                false,
            );
        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .req_tick_by_tick_data(
                19002,
                contract_samples::european_stock2().borrow(),
                TickByTickType::Last,
                0,
                false,
            );
        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .req_tick_by_tick_data(
                19003,
                contract_samples::european_stock2().borrow(),
                TickByTickType::Last,
                0,
                true,
            );
        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .req_tick_by_tick_data(
                19004,
                contract_samples::eur_gbp_fx().borrow(),
                TickByTickType::Last,
                0,
                false,
            );
        // ! [reqtickbytick]

        // Requesting tick - by - tick data (refresh + historicalticks)
        // ! [reqtickbytickwithhist]
        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .req_tick_by_tick_data(
                19005,
                contract_samples::european_stock2().borrow(),
                TickByTickType::Last,
                10,
                false,
            );
        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .req_tick_by_tick_data(
                19006,
                contract_samples::european_stock2().borrow(),
                TickByTickType::Last,
                10,
                false,
            );
        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .req_tick_by_tick_data(
                19007,
                contract_samples::european_stock2().borrow(),
                TickByTickType::Last,
                10,
                false,
            );
        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .req_tick_by_tick_data(
                19008,
                contract_samples::eur_gbp_fx().borrow(),
                TickByTickType::Last,
                10,
                true,
            );
    }

    //----------------------------------------------------------------------------------------------
    fn historical_ticks_operations(&self) {
        // ! [reqhistoricalticks]
        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .req_historical_ticks(
                18001,
                contract_samples::usstock().borrow(),
                "20170712 21:39:33",
                "",
                10,
                "TRADES",
                1,
                true,
                vec![],
            );
        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .req_historical_ticks(
                18002,
                contract_samples::us_stock_at_smart().borrow(),
                "20170712 21:39:33",
                "",
                10,
                "BID_ASK",
                1,
                true,
                vec![],
            );
        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .req_historical_ticks(
                18003,
                contract_samples::us_stock_at_smart().borrow(),
                "20170712 21:39:33",
                "",
                10,
                "MIDPOINT",
                1,
                true,
                vec![],
            );
    }

    //----------------------------------------------------------------------------------------------
    fn histogram_operations_req(&self) {
        // ![reqhistogramdata]
        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .req_histogram_data(4002, contract_samples::usstock().borrow(), false, "3 days");
        // ![reqhistogramdata]
    }

    //----------------------------------------------------------------------------------------------
    fn histogram_operations_cancel(&self) {
        // ! [cancelhistogramdata]
        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .cancel_histogram_data(4002);
        // ! [cancelhistogramdata]
    }

    //----------------------------------------------------------------------------------------------
    fn continuous_futures_operations_req(&self) {
        // ! [reqcontractdetailscontfut]
        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .req_contract_details(18001, contract_samples::cont_fut().borrow());
        // ! [reqcontractdetailscontfut]

        // ! [reqhistoricaldatacontfut]
        let time_str = Utc::now().format("%Y%m%d %H:%M:%S");
        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .req_historical_data(
                18002,
                contract_samples::cont_fut().borrow(),
                time_str.to_string().as_str(),
                "1 Y",
                "1 month",
                "TRADES",
                0,
                1,
                false,
                vec![],
            );
    }

    //----------------------------------------------------------------------------------------------
    fn pnl_operations_req(&self) {
        // ! [reqpnl]
        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .req_pnl(17001, "DU228243", "");
        // ! [reqpnl]

        // ! [reqpnlsingle]
        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .req_pnl_single(17002, "DU228243", "", 8314);
        // ! [reqpnlsingle]
    }

    //----------------------------------------------------------------------------------------------
    fn pnl_operations_cancel(&self) {
        // ! [cancelpnl]
        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .cancel_pnl(17001);
        // ! [cancelpnl]

        // ! [cancelpnlsingle]
        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .cancel_pnl_single(17002);
        // ! [cancelpnlsingle]
    }

    //--------------------------------------------------------------------------------------------------
    fn market_rule_operations(&self) {
        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .req_contract_details(17001, contract_samples::usstock().borrow());
        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .req_contract_details(17002, contract_samples::bond().borrow());

        // ! [reqmarketrule]
        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .req_market_rule(26);
        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .req_market_rule(239);
    }

    //----------------------------------------------------------------------------------------------
    fn reroute_cfd_operations(&self) {
        // ![reqmktdatacfd]
        self.client.as_ref().unwrap().lock().unwrap().req_mkt_data(
            16001,
            contract_samples::usstock_cfd().borrow(),
            "",
            false,
            false,
            vec![],
        );
        self.client.as_ref().unwrap().lock().unwrap().req_mkt_data(
            16002,
            contract_samples::european_stock_cfd().borrow(),
            "",
            false,
            false,
            vec![],
        );
        self.client.as_ref().unwrap().lock().unwrap().req_mkt_data(
            16003,
            contract_samples::cash_cfd().borrow(),
            "",
            false,
            false,
            vec![],
        );
        // ![reqmktdatacfd]

        // ![reqmktdepthcfd]
        self.client.as_ref().unwrap().lock().unwrap().req_mkt_depth(
            16004,
            contract_samples::usstock_cfd().borrow(),
            10,
            false,
            vec![],
        );
        self.client.as_ref().unwrap().lock().unwrap().req_mkt_depth(
            16005,
            contract_samples::european_stock_cfd().borrow(),
            10,
            false,
            vec![],
        );
        self.client.as_ref().unwrap().lock().unwrap().req_mkt_depth(
            16006,
            contract_samples::cash_cfd().borrow(),
            10,
            false,
            vec![],
        );
    }

    //----------------------------------------------------------------------------------------------
    fn financial_advisor_operations(&self) {
        // Requesting FA information

        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .request_fa(FaDataType::ALIASES);

        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .request_fa(FaDataType::GROUPS);

        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .request_fa(FaDataType::PROFILES);

        // Replacing FA information - Fill in with the appropriate XML string.

        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .replace_fa(FaDataType::GROUPS, fa_allocation_samples::FA_ONE_GROUP);

        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .replace_fa(FaDataType::GROUPS, fa_allocation_samples::FA_TWO_GROUPS);

        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .replace_fa(FaDataType::PROFILES, fa_allocation_samples::FA_ONE_PROFILE);

        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .replace_fa(FaDataType::PROFILES, fa_allocation_samples::FA_TWO_PROFILES);

        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .req_soft_dollar_tiers(14001);
    }

    //----------------------------------------------------------------------------------------------
    fn news_operations_req(&self) {
        // Requesting news ticks
        // ! [reqNewsTicks]
        //        self.client.as_ref().unwrap().lock().unwrap().req_mkt_data(
        //            10001,
        //            contract_samples::usstock().borrow(),
        //            "mdoff,258",
        //            false,
        //            false,
        //            vec![],
        //        );
        // ! [reqNewsTicks]

        // Returns list of subscribed news providers
        // ! [reqNewsProviders]
        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .req_news_providers();
        // ! [reqNewsProviders]

        // Returns body of news article given article ID
        // ! [reqNewsArticle]
        //        self.client
        //            .as_ref()
        //            .unwrap()
        //            .lock()
        //            .unwrap()
        //            .req_news_article(10002, "BRFG", "BRFG$04fb9da2", vec![]);
        // ! [reqNewsArticle]

        // Returns list of historical news headlines with IDs
        // ! [reqHistoricalNews]
        //        self.client
        //            .as_ref()
        //            .unwrap()
        //            .lock()
        //            .unwrap()
        //            .req_historical_news(10003, 8314, "BRFG", "", "", 10, vec![]);
        // ! [reqHistoricalNews]

        // ! [reqcontractdetailsnews]
        //        self.client
        //            .as_ref()
        //            .unwrap()
        //            .lock()
        //            .unwrap()
        //            .req_contract_details(10004, contract_samples::news_feed_for_query().borrow());
    }

    //----------------------------------------------------------------------------------------------
    fn news_operations_cancel(&self) {
        // Canceling news ticks
        // ! [cancelNewsTicks]
        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .cancel_mkt_data(10001);
        // ! [cancelNewsTicks]
    }

    //----------------------------------------------------------------------------------------------
    fn bulletins_operations_req(&self) {
        // Requesting Interactive Broker's news bulletins_operations_req
        // ! [reqnewsbulletins]
        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .req_news_bulletins(true);
        // ! [reqnewsbulletins]
    }

    //----------------------------------------------------------------------------------------------
    fn bulletins_operations_cancel(&self) {
        // Canceling IB's news bulletins_operations_req
        // ! [cancelnewsbulletins]
        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .cancel_news_bulletins();
        // ! [cancelnewsbulletins]
    }

    //----------------------------------------------------------------------------------------------
    fn miscelaneous_operations(&self) {
        // Request TWS' current time
        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .req_current_time();
        // Setting TWS logging level
        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .set_server_log_level(1);
    }

    fn linking_operations(&self) {
        // ! [querydisplaygroups]
        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .query_display_groups(19001);
        // ! [querydisplaygroups]

        // ! [subscribetogroupevents]
        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .subscribe_to_group_events(19002, 1);
        // ! [subscribetogroupevents]

        // ! [updatedisplaygroup]
        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .update_display_group(19002, "8314@SMART");
        // ! [updatedisplaygroup]

        // ! [subscribefromgroupevents]
        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .unsubscribe_from_group_events(19002);
    }

    //----------------------------------------------------------------------------------------------
    fn tick_by_tick_operations_cancel(&self) {
        // ! [canceltickbytick]
        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .cancel_tick_by_tick_data(19001);
        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .cancel_tick_by_tick_data(19002);
        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .cancel_tick_by_tick_data(19003);
        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .cancel_tick_by_tick_data(19004);
        // ! [canceltickbytick]

        // ! [canceltickbytickwithhist]
        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .cancel_tick_by_tick_data(19005);
        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .cancel_tick_by_tick_data(19006);
        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .cancel_tick_by_tick_data(19007);
        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .cancel_tick_by_tick_data(19008);
    }

    //----------------------------------------------------------------------------------------------
    fn continuous_futures_operations_cancel(&self) {
        // ! [cancelhistoricaldatacontfut]
        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .cancel_historical_data(18002);
        // ! [cancelhistoricaldatacontfut]
    }

    //----------------------------------------------------------------------------------------------
    fn fundamentals_operations_cancel(&self) {
        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .cancel_fundamental_data(8001);
        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .cancel_fundamental_data(8002);
        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .cancel_fundamental_data(8003);
        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .cancel_fundamental_data(8004);
        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .cancel_fundamental_data(8005);
        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .cancel_fundamental_data(8006);
    }

    //----------------------------------------------------------------------------------------------
    fn market_scanners_cancel(&self) {
        // Canceling the scanner subscription
        // ! [cancelscannersubscription]
        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .cancel_scanner_subscription(7001);
        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .cancel_scanner_subscription(7002);
        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .cancel_scanner_subscription(7003);
    }

    //----------------------------------------------------------------------------------------------
    fn options_operations_cancel(&self) {
        // Canceling implied volatility
        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .cancel_calculate_implied_volatility(5001);
        // Canceling option's price calculation
        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .cancel_calculate_option_price(5002);
    }

    //----------------------------------------------------------------------------------------------
    fn historical_data_operations_cancel(&self) {
        // ! [cancelHeadTimestamp]
        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .cancel_head_time_stamp(4101);
        // ! [cancelHeadTimestamp]

        // Canceling historical data requests
        // ! [cancelhistoricaldata]
        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .cancel_historical_data(4102);
        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .cancel_historical_data(4103);
        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .cancel_historical_data(4104);
    }

    //----------------------------------------------------------------------------------------------
    fn real_time_bars_operations_cancel(&self) {
        // Canceling real time bars
        // ! [cancelrealtimebars]
        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .cancel_real_time_bars(3001);
    }

    //----------------------------------------------------------------------------------------------
    fn market_depth_operations_cancel(&self) {
        // Canceling the Deep Book request
        // ! [cancelmktdepth]
        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .cancel_mkt_depth(2001, false);
        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .cancel_mkt_depth(2002, true);
    }

    //----------------------------------------------------------------------------------------------
    fn tick_data_operations_cancel(&self) {
        // Canceling the market data subscription
        // ! [cancelmktdata]
        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .cancel_mkt_data(1000);
        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .cancel_mkt_data(1001);
        // ! [cancelmktdata]

        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .cancel_mkt_data(1004);

        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .cancel_mkt_data(1005);
        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .cancel_mkt_data(1006);
        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .cancel_mkt_data(1007);
        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .cancel_mkt_data(1008);

        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .cancel_mkt_data(1009);
        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .cancel_mkt_data(1010);
        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .cancel_mkt_data(1011);
        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .cancel_mkt_data(1012);

        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .cancel_mkt_data(1013);
        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .cancel_mkt_data(1014);

        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .cancel_mkt_data(1015);

        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .cancel_mkt_data(1016);

        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .cancel_mkt_data(1017);
    }

    //----------------------------------------------------------------------------------------------
    fn account_operations_cancel(&self) {
        // ! [cancelaaccountsummary]
        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .cancel_account_summary(9001);
        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .cancel_account_summary(9002);
        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .cancel_account_summary(9003);
        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .cancel_account_summary(9004);
        // ! [cancelaaccountsummary]

        // ! [cancelaaccountupdates]
        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .req_account_updates(false, self.account.as_str());
        // ! [cancelaaccountupdates]

        // ! [cancelaaccountupdatesmulti]
        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .cancel_account_updates_multi(9005);
        // ! [cancelaaccountupdatesmulti]

        // ! [cancelpositions]
        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .cancel_positions();
        // ! [cancelpositions]

        // ! [cancelpositionsmulti]
        self.client
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .cancel_positions_multi(9006);
        // ! [cancelpositionsmulti]
    }
}

impl Wrapper for TestWrapper {
    fn error(&mut self, req_id: i32, error_code: i32, error_string: &str) {
        error!(
            "req_id: {} ,error_code: {} , error_string:{}",
            req_id, error_code, error_string
        );
    }

    //----------------------------------------------------------------------------------------------
    fn win_error(&mut self, text: &str, last_error: i32) {
        error!("text: {} , last_error:{}", text, last_error);
    }

    //----------------------------------------------------------------------------------------------
    fn connect_ack(&mut self) {
        info!("Connected.");
    }

    //----------------------------------------------------------------------------------------------
    fn market_data_type(&mut self, req_id: i32, market_data_type: i32) {
        info!(
            "market_data_type -- req_id: {}, market_data_type: {}",
            req_id, market_data_type
        );
    }

    //----------------------------------------------------------------------------------------------
    fn tick_price(&mut self, req_id: i32, tick_type: TickType, price: f64, attrib: TickAttrib) {
        info!(
            "tick_size -- req_id: {}, tick_type: {}, price: {}, attrib: {}",
            req_id, tick_type, price, attrib
        );
    }

    //----------------------------------------------------------------------------------------------
    fn tick_size(&mut self, req_id: i32, tick_type: TickType, size: i32) {
        info!(
            "tick_size -- req_id: {}, tick_type: {}, size: {}",
            req_id, tick_type, size
        );
    }

    //----------------------------------------------------------------------------------------------
    fn tick_snapshot_end(&mut self, req_id: i32) {
        info!("tick_snapshot_end -- req_id: {}", req_id);
    }

    //----------------------------------------------------------------------------------------------
    fn tick_generic(&mut self, req_id: i32, tick_type: TickType, value: f64) {
        info!(
            "tick_generic -- req_id: {}, tick_type: {}, value: {}",
            req_id, tick_type, value
        );
    }

    //----------------------------------------------------------------------------------------------
    fn tick_string(&mut self, req_id: i32, tick_type: TickType, value: &str) {
        info!(
            "tick_string -- req_id: {}, tick_type: {}, value: {}",
            req_id, tick_type, value
        );
    }

    //----------------------------------------------------------------------------------------------
    fn tick_efp(
        &mut self,
        req_id: i32,
        tick_type: TickType,
        basis_points: f64,
        formatted_basis_points: &str,
        total_dividends: f64,
        hold_days: i32,
        future_last_trade_date: &str,
        dividend_impact: f64,
        dividends_to_last_trade_date: f64,
    ) {
        info!(
            "tick_efp -- req_id: {},
             tick_type: {},
             basis_points: {},
             formatted_basis_points: {},
             total_dividends: {},
             hold_days: {},
             future_last_trade_date: {},
             dividend_impact: {},
             dividends_to_last_trade_date: {},",
            req_id,
            tick_type,
            basis_points,
            formatted_basis_points,
            total_dividends,
            hold_days,
            future_last_trade_date,
            dividend_impact,
            dividends_to_last_trade_date,
        );
    }

    //----------------------------------------------------------------------------------------------
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
    ) {
        info!(
            "order_status -- order_id: {}, status: {}, filled: {}, remaining: {}, avg_fill_price: {}, \
            perm_id: {}, parent_id: {}, last_fill_price: {}, client_id: {}, why_held: {}, mkt_cap_price: {}",
            order_id, status, filled, remaining, avg_fill_price, perm_id, parent_id, last_fill_price,
            client_id, why_held, mkt_cap_price
        );
    }

    //----------------------------------------------------------------------------------------------
    fn open_order(
        &mut self,
        order_id: i32,
        contract: Contract,
        order: Order,
        order_state: OrderState,
    ) {
        info!(
            "open_order -- order_id: {}, contract: {}, order: {}, order_state: {}",
            order_id, contract, order, order_state
        );
    }

    //----------------------------------------------------------------------------------------------
    fn open_order_end(&mut self) {
        info!("open_order_end. (no parmeters passed)");
    }

    //----------------------------------------------------------------------------------------------
    fn connection_closed(&mut self) {
        info!("connection_closed. (no parmeters passed)");
    }

    //----------------------------------------------------------------------------------------------
    fn update_account_value(&mut self, key: &str, val: &str, currency: &str, account_name: &str) {
        info!(
            "key: {}, value: {}, ccy: {}, account: {}.",
            key, val, currency, account_name
        );
    }

    //----------------------------------------------------------------------------------------------
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
    ) {
        info!(
            "update_portfolio -- contract: {}, position: {}, market_price: {}, market_value: {}, 
             average_cost: {}, unrealized_pnl: {},  realized_pnl: {},  account_name: {}",
            contract,
            position,
            market_price,
            market_value,
            average_cost,
            unrealized_pnl,
            realized_pnl,
            account_name
        );
    }

    //----------------------------------------------------------------------------------------------
    fn update_account_time(&mut self, time_stamp: &str) {
        info!("update_account_time: {}.", time_stamp);
    }

    //----------------------------------------------------------------------------------------------
    fn account_download_end(&mut self, account_name: &str) {
        info!("account_download_end: {}.", account_name);
    }

    //----------------------------------------------------------------------------------------------
    fn next_valid_id(&mut self, order_id: i32) {
        self.next_order_id = order_id;
        info!("next_valid_id -- order_id: {}", order_id);

        self.start_requests();
    }

    //----------------------------------------------------------------------------------------------
    fn contract_details(&mut self, req_id: i32, contract_details: ContractDetails) {
        info!(
            "contract_details -- req_id: {}, contract_details: {}",
            req_id, contract_details
        );
    }

    //----------------------------------------------------------------------------------------------
    fn bond_contract_details(&mut self, req_id: i32, contract_details: ContractDetails) {
        info!(
            "bond_contract_details -- req_id: {}, contract_details: {}",
            req_id, contract_details
        );
    }

    //----------------------------------------------------------------------------------------------
    fn contract_details_end(&mut self, req_id: i32) {
        info!("contract_details_end -- req_id: {}", req_id);
    }

    fn exec_details(&mut self, req_id: i32, contract: Contract, execution: Execution) {
        info!(
            "exec_details -- req_id: {}, contract: {}, execution: {}",
            req_id, contract, execution
        );
    }

    //----------------------------------------------------------------------------------------------
    fn exec_details_end(&mut self, req_id: i32) {
        info!("exec_details_end -- req_id: {}", req_id);
    }

    //----------------------------------------------------------------------------------------------
    fn update_mkt_depth(
        &mut self,
        req_id: i32,
        position: i32,
        operation: i32,
        side: i32,
        price: f64,
        size: i32,
    ) {
        info!(
            "update_mkt_depth -- req_id: {}, position: {}, operation: {}, side: {}, price: {}, size: {}",
            req_id, position, operation, side, price, size
        );
    }

    //----------------------------------------------------------------------------------------------
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
    ) {
        info!(
            "update_mkt_depth_l2 -- req_id: {}, position: {}, market_maker: {}, operation: {}, side: {}, price: {}, size: {}, is_smart_depth: {},",
            req_id, position, market_maker, operation, side, price, size, is_smart_depth
        );
    }

    //----------------------------------------------------------------------------------------------
    fn update_news_bulletin(
        &mut self,
        msg_id: i32,
        msg_type: i32,
        news_message: &str,
        origin_exch: &str,
    ) {
        info!(
            "update_news_bulletin -- msg_id: {}, msg_type: {}, news_message: {}, origin_exch: {}",
            msg_id, msg_type, news_message, origin_exch
        );
    }

    //----------------------------------------------------------------------------------------------
    fn managed_accounts(&mut self, accounts_list: &str) {
        info!("managed_accounts -- accounts_list: {}", accounts_list);
        let split = accounts_list.split(",");
        //self.account = split;
    }

    //----------------------------------------------------------------------------------------------
    fn receive_fa(&mut self, fa_data: FaDataType, cxml: &str) {
        info!("receive_fa -- fa_data: {}, cxml: {}", fa_data, cxml);
    }

    //----------------------------------------------------------------------------------------------
    fn historical_data(&mut self, req_id: i32, bar: BarData) {
        info!("historical_data -- req_id: {}, bar: {}", req_id, bar);
    }

    //----------------------------------------------------------------------------------------------
    fn historical_data_end(&mut self, req_id: i32, start: &str, end: &str) {
        info!(
            "historical_data_end -- req_id: {}, start: {}, end: {}",
            req_id, start, end
        );
    }

    //----------------------------------------------------------------------------------------------
    fn scanner_parameters(&mut self, xml: &str) {
        info!("scanner_parameters -- xml: {}", xml);
    }

    //----------------------------------------------------------------------------------------------
    fn scanner_data(
        &mut self,
        req_id: i32,
        rank: i32,
        contract_details: ContractDetails,
        distance: &str,
        benchmark: &str,
        projection: &str,
        legs_str: &str,
    ) {
        info!(
            "scanner_data -- req_id: {}, rank: {},
             contract_details: {},
             distance: {},
             benchmark: {},
             projection: {},
             legs_str: {}",
            req_id, rank, contract_details, distance, benchmark, projection, legs_str
        );
    }

    //----------------------------------------------------------------------------------------------
    fn scanner_data_end(&mut self, req_id: i32) {
        info!("scanner_data_end -- req_id: {}", req_id);
    }

    //----------------------------------------------------------------------------------------------
    fn realtime_bar(
        &mut self,
        req_id: i32,
        time: i32,
        open_: f64,
        high: f64,
        low: f64,
        close: f64,
        volume: i64,
        wap: f64,
        count: i32,
    ) {
        info!(
            "realtime_bar -- req_id: {}, time: {}, open_: {}, high: {}, low: {}, close: {}, volume: {}, wap: {}, count: {}",
            req_id,
            time,
            open_,
            high,
            low,
            close,
            volume,
            wap,
            count,
        );
    }

    //----------------------------------------------------------------------------------------------
    fn current_time(&mut self, time: i64) {
        // Creates a new SystemTime from the specified number of whole seconds
        let d = UNIX_EPOCH + Duration::from_secs(time as u64);
        // Create DateTime from SystemTime
        let datetime = DateTime::<Utc>::from(d);
        // Formats the combined date and time with the specified format string.
        let timestamp_str = datetime.format("%Y-%m-%d %H:%M:%S.%f").to_string();
        info!("current_time -- time: {}", timestamp_str);
    }

    //----------------------------------------------------------------------------------------------
    fn fundamental_data(&mut self, req_id: i32, data: &str) {
        info!(
            "fundamental_data -- req_id: {}, delta_neutral_contract: {}",
            req_id, data
        );
    }

    //----------------------------------------------------------------------------------------------
    fn delta_neutral_validation(
        &mut self,
        req_id: i32,
        delta_neutral_contract: DeltaNeutralContract,
    ) {
        info!(
            "delta_neutral_validation -- req_id: {}, delta_neutral_contract: {}",
            req_id, delta_neutral_contract
        );
    }

    //----------------------------------------------------------------------------------------------
    fn commission_report(&mut self, commission_report: CommissionReport) {
        info!(
            "commission_report -- commission_report: {}",
            commission_report
        );
    }

    //----------------------------------------------------------------------------------------------
    fn position(&mut self, account: &str, contract: Contract, position: f64, avg_cost: f64) {
        info!(
            "position -- account: {}, contract: [{}], position: {}, avg_cost: {}",
            account, contract, position, avg_cost
        );
    }

    //----------------------------------------------------------------------------------------------
    fn position_end(&mut self) {
        info!("position_end -- (no params are passed in this one)");
    }

    //----------------------------------------------------------------------------------------------
    fn account_summary(
        &mut self,
        req_id: i32,
        account: &str,
        tag: &str,
        value: &str,
        currency: &str,
    ) {
        info!(
            "account_summary -- req_id: {}, account: {}, tag: {}, value: {}, currency: {}",
            req_id, account, tag, value, currency
        );
    }

    //----------------------------------------------------------------------------------------------
    fn account_summary_end(&mut self, req_id: i32) {
        info!("account_summary_end -- req_id: {}", req_id);
    }

    //----------------------------------------------------------------------------------------------
    fn verify_message_api(&mut self, api_data: &str) {
        info!("verify_message_api -- api_data: {}", api_data);
    }

    //----------------------------------------------------------------------------------------------
    fn verify_completed(&mut self, is_successful: bool, error_text: &str) {
        info!(
            "verify_completed -- is_successful: {}, error_text: {}",
            is_successful, error_text
        );
    }

    //----------------------------------------------------------------------------------------------
    fn verify_and_auth_message_api(&mut self, api_data: &str, xyz_challange: &str) {
        info!(
            "verify_and_auth_message_api -- api_data: {}, xyz_challange: {}",
            api_data, xyz_challange
        );
    }

    //----------------------------------------------------------------------------------------------
    fn verify_and_auth_completed(&mut self, is_successful: bool, error_text: &str) {
        info!(
            "verify_and_auth_completed -- is_successful: {}, error_text: {}",
            is_successful, error_text
        );
    }

    //----------------------------------------------------------------------------------------------
    fn display_group_list(&mut self, req_id: i32, groups: &str) {
        info!(
            "display_group_list -- req_id: {}, error_text: {}",
            req_id, groups
        );
    }

    //----------------------------------------------------------------------------------------------
    fn display_group_updated(&mut self, req_id: i32, contract_info: &str) {
        info!(
            "display_group_updated -- req_id: {}, contract_info: {}",
            req_id, contract_info
        );
    }

    //----------------------------------------------------------------------------------------------
    fn position_multi(
        &mut self,
        req_id: i32,
        account: &str,
        model_code: &str,
        contract: Contract,
        pos: f64,
        avg_cost: f64,
    ) {
        info!(
            "position_multi -- req_id: {}, account: {}, model_code: {}, contract: {}, pos: {}, \
             avg_cost: {}",
            req_id, account, model_code, contract, pos, avg_cost
        );
    }

    //----------------------------------------------------------------------------------------------
    fn position_multi_end(&mut self, req_id: i32) {
        info!("position_multi_end -- req_id: {}", req_id);
    }

    //----------------------------------------------------------------------------------------------
    fn account_update_multi(
        &mut self,
        req_id: i32,
        account: &str,
        model_code: &str,
        key: &str,
        value: &str,
        currency: &str,
    ) {
        info!(
            "account_update_multi -- req_id: {}, account: {}, model_code: {}, key: {}, value: {}, currency: {}",
            req_id, account, model_code, key, value, currency
        );
    }

    //----------------------------------------------------------------------------------------------
    fn account_update_multi_end(&mut self, req_id: i32) {
        info!("account_update_multi_end -- req_id: {}", req_id);
    }

    //----------------------------------------------------------------------------------------------
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
    ) {
        info!(
            "tick_option_computation -- req_id: {}, tick_type: {}, implied_vol: {}, delta: {}, \
             opt_price: {}, pv_dividend: {},  gamma: {}, vega: {}, theta: {}, und_price: {}",
            req_id,
            tick_type,
            implied_vol,
            delta,
            opt_price,
            pv_dividend,
            gamma,
            vega,
            theta,
            und_price
        );
    }

    //----------------------------------------------------------------------------------------------
    fn security_definition_option_parameter(
        &mut self,
        req_id: i32,
        exchange: &str,
        underlying_con_id: i32,
        trading_class: &str,
        multiplier: &str,
        expirations: HashSet<String>,
        strikes: HashSet<BigDecimal>,
    ) {
        info!(
            "tick_option_computation -- req_id: {}, exchange: {}, underlying_con_id: {}, \
             trading_class: {}, multiplier: {}, expirations: {:?},  strikes: {:?}",
            req_id,
            exchange,
            underlying_con_id,
            trading_class,
            multiplier,
            expirations
                .iter()
                .map(|x| x.as_str())
                .collect::<Vec<&str>>(),
            strikes
                .iter()
                .map(|x| x.clone())
                .collect::<Vec<BigDecimal>>()
        );
    }

    //----------------------------------------------------------------------------------------------
    fn security_definition_option_parameter_end(&mut self, req_id: i32) {
        info!(
            "security_definition_option_parameter_end -- req_id: {}",
            req_id
        );
    }

    //----------------------------------------------------------------------------------------------
    fn soft_dollar_tiers(&mut self, req_id: i32, tiers: Vec<SoftDollarTier>) {
        info!(
            "soft_dollar_tiers -- req_id: {}, tiers: {:?}",
            req_id, tiers
        );
    }

    //----------------------------------------------------------------------------------------------
    fn family_codes(&mut self, family_codes: Vec<FamilyCode>) {
        info!("family_codes -- family_codes: {:?}", family_codes);
    }

    //----------------------------------------------------------------------------------------------
    fn symbol_samples(&mut self, req_id: i32, contract_descriptions: Vec<ContractDescription>) {
        info!(
            "symbol_samples -- req_id: {}, contract_descriptions: {:?}",
            req_id, contract_descriptions
        );
    }

    //----------------------------------------------------------------------------------------------
    fn mkt_depth_exchanges(&mut self, depth_mkt_data_descriptions: Vec<DepthMktDataDescription>) {
        info!(
            "mkt_depth_exchanges -- depth_mkt_data_descriptions: {:?}",
            depth_mkt_data_descriptions
        );
    }

    //----------------------------------------------------------------------------------------------
    fn tick_news(
        &mut self,
        ticker_id: i32,
        time_stamp: i32,
        provider_code: &str,
        article_id: &str,
        headline: &str,
        extra_data: &str,
    ) {
        info!(
            "tick_news -- ticker_id: {}, time_stamp: {}, provider_code: {}, article_id: {}, \
             headline: {}, extra_data: {},",
            ticker_id, time_stamp, provider_code, article_id, headline, extra_data
        );
    }

    //----------------------------------------------------------------------------------------------
    fn smart_components(&mut self, req_id: i32, smart_components: Vec<SmartComponent>) {
        info!(
            "smart_components -- req_id: {}, smart_components: {:?}",
            req_id, smart_components
        );
    }

    //----------------------------------------------------------------------------------------------
    fn tick_req_params(
        &mut self,
        ticker_id: i32,
        min_tick: f64,
        bbo_exchange: &str,
        snapshot_permissions: i32,
    ) {
        info!(
            "tick_req_params -- ticker_id: {}, min_tick: {}, bbo_exchange: {}, snapshot_permissions: {}",
            ticker_id, min_tick, bbo_exchange, snapshot_permissions
        );
    }

    //----------------------------------------------------------------------------------------------
    fn news_providers(&mut self, news_providers: Vec<NewsProvider>) {
        info!("news_providers -- news_providers: {:?}", news_providers);
    }

    //----------------------------------------------------------------------------------------------
    fn news_article(&mut self, request_id: i32, article_type: i32, article_text: &str) {
        info!(
            "news_article -- request_id: {}, article_type: {}, article_text: {}",
            request_id, article_type, article_text
        );
    }

    //----------------------------------------------------------------------------------------------
    fn historical_news(
        &mut self,
        request_id: i32,
        time: &str,
        provider_code: &str,
        article_id: &str,
        headline: &str,
    ) {
        info!(
            "historical_news -- request_id: {}, time: {}, provider_code: {}, article_id: {}, headline: {}",
            request_id, time, provider_code, article_id, headline
        );
    }

    //----------------------------------------------------------------------------------------------
    fn historical_news_end(&mut self, request_id: i32, has_more: bool) {
        info!(
            "historical_news_end -- request_id: {}, has_more: {}",
            request_id, has_more
        );
    }

    //----------------------------------------------------------------------------------------------
    fn head_timestamp(&mut self, req_id: i32, head_timestamp: &str) {
        info!(
            "head_timestamp -- req_id: {}, head_timestamp: {}",
            req_id, head_timestamp
        );
    }

    //----------------------------------------------------------------------------------------------
    fn histogram_data(&mut self, req_id: i32, items: Vec<HistogramData>) {
        info!("histogram_data -- req_id: {}, items: {:?}", req_id, items);
    }

    fn historical_data_update(&mut self, req_id: i32, bar: BarData) {
        info!("historical_data_update -- req_id: {}, bar: {}", req_id, bar);
    }

    //----------------------------------------------------------------------------------------------
    fn reroute_mkt_data_req(&mut self, req_id: i32, con_id: i32, exchange: &str) {
        info!(
            "reroute_mkt_data_req -- req_id: {}, con_id: {}, exchange: {}",
            req_id, con_id, exchange
        );
    }

    //----------------------------------------------------------------------------------------------
    fn reroute_mkt_depth_req(&mut self, req_id: i32, con_id: i32, exchange: &str) {
        info!(
            "reroute_mkt_depth_req -- req_id: {}, con_id: {}, exchange: {}",
            req_id, con_id, exchange
        );
    }

    //----------------------------------------------------------------------------------------------
    fn market_rule(&mut self, market_rule_id: i32, price_increments: Vec<PriceIncrement>) {
        info!(
            "market_rule -- market_rule_id: {}, price_increments: {:?}",
            market_rule_id, price_increments
        );
    }

    //----------------------------------------------------------------------------------------------
    fn pnl(&mut self, req_id: i32, daily_pn_l: f64, unrealized_pn_l: f64, realized_pn_l: f64) {
        info!(
            "pnl -- req_id: {}, daily_pn_l: {}, unrealized_pn_l: {}, realized_pn_l: {})",
            req_id, daily_pn_l, unrealized_pn_l, realized_pn_l
        );
    }

    //----------------------------------------------------------------------------------------------
    fn pnl_single(
        &mut self,
        req_id: i32,
        pos: i32,
        daily_pn_l: f64,
        unrealized_pn_l: f64,
        realized_pn_l: f64,
        value: f64,
    ) {
        info!(
            "pnl_single -- req_id: {}, pos: {}, daily_pn_l: {}, unrealized_pn_l: {}, realized_pn_l: {}, value: {})",
            req_id, pos, daily_pn_l, unrealized_pn_l, realized_pn_l, value
        );
    }

    //----------------------------------------------------------------------------------------------
    fn historical_ticks(&mut self, req_id: i32, ticks: Vec<HistoricalTick>, done: bool) {
        info!(
            "historical_ticks -- req_id: {}, ticks: {:?}, done: {}",
            req_id, ticks, done
        );
    }

    //----------------------------------------------------------------------------------------------
    fn historical_ticks_bid_ask(
        &mut self,
        req_id: i32,
        ticks: Vec<HistoricalTickBidAsk>,
        done: bool,
    ) {
        info!(
            "historical_ticks_bid_ask -- req_id: {}, ticks: {:?}, done: {}",
            req_id, ticks, done
        );
    }

    //----------------------------------------------------------------------------------------------
    fn historical_ticks_last(&mut self, req_id: i32, ticks: Vec<HistoricalTickLast>, done: bool) {
        info!(
            "historical_ticks_last -- req_id: {}, ticks: {:?}, done: {}",
            req_id, ticks, done
        );
    }

    //----------------------------------------------------------------------------------------------
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
    ) {
        info!(
            "tick_by_tick_all_last -- req_id: {}, tick_type: {:?}, time: {}, price: {}, size: {}, \
             tick_attrib_last: {}, exchange: {}, special_conditions: {}",
            req_id, tick_type, time, price, size, tick_attrib_last, exchange, special_conditions
        );
    }

    //----------------------------------------------------------------------------------------------
    fn tick_by_tick_bid_ask(
        &mut self,
        req_id: i32,
        time: i64,
        bid_price: f64,
        ask_price: f64,
        bid_size: i32,
        ask_size: i32,
        tick_attrib_bid_ask: TickAttribBidAsk,
    ) {
        info!(
            "tick_by_tick_bid_ask -- req_id: {}, time: {}, bid_price: {}, ask_price: {}, bid_size: {}, \
             ask_size: {}, tick_attrib_last: {}",
            req_id, time, bid_price, ask_price, bid_size, ask_size, tick_attrib_bid_ask
        );
    }

    //----------------------------------------------------------------------------------------------
    fn tick_by_tick_mid_point(&mut self, req_id: i32, time: i64, mid_point: f64) {
        info!(
            "tick_by_tick_mid_point -- req_id: {}, time: {}, mid_point: {}",
            req_id, time, mid_point
        );
    }

    //----------------------------------------------------------------------------------------------
    fn order_bound(&mut self, req_id: i32, api_client_id: i32, api_order_id: i32) {
        info!(
            "order_bound -- req_id: {}, api_client_id: {}, api_order_id: {}",
            req_id, api_client_id, api_order_id
        );
    }

    //----------------------------------------------------------------------------------------------
    fn completed_order(&mut self, contract: Contract, order: Order, order_state: OrderState) {
        info!(
            "completed_order -- contract: [{}], order: [{}], order_state: [{}]",
            contract, order, order_state
        );
    }

    //----------------------------------------------------------------------------------------------
    fn completed_orders_end(&mut self) {
        info!("completed_orders_end -- (no parameters for this message)");
    }
}

unsafe impl Send for TestWrapper {}

unsafe impl Sync for TestWrapper {}

/// Example of using client and wrapper.
/// Upon connecting, TWS will send the next valid order ID which will cause the wrapper callback method
/// next_valid_id to be called, which will start sending tests requests to TWS (see the
/// start_requests function inn TestWrapper which is called by next_valid_id

//==================================================================================================
fn main() -> Result<(), IBKRApiLibError> {
    log4rs::init_file("log_config.yml", Default::default()).unwrap();

    let wrapper = Arc::new(Mutex::new(TestWrapper::new()));
    let app = Arc::new(Mutex::new(EClient::new(wrapper.clone())));

    info!("getting connection...");
    wrapper.lock().unwrap().client = Option::from(app.clone());

    // Upon connecting, TWS will send the next valid order ID which will cause the wrapper callback method
    // next_valid_id to be called, which will start sending tests requests to TWS (see the
    // start_requests function inn TestWrapper which is called by next_valid_id
    app.lock().unwrap().connect("127.0.0.1", 7497, 0);

    thread::sleep(Duration::new(18600, 0));

    Ok(())
}
