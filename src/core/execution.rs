//! Types related to executions
use std::fmt::Display;

use serde::export::fmt::Error;
use serde::export::Formatter;
use serde::{Deserialize, Serialize};

//==================================================================================================
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Execution {
    pub exec_id: String,
    pub time: String,
    pub acct_number: String,
    pub exchange: String,
    pub side: String,
    pub shares: f64,
    pub price: f64,
    pub perm_id: i32,
    pub client_id: i32,
    pub order_id: i32,
    pub liquidation: i32,
    pub cum_qty: f64,
    pub avg_price: f64,
    pub order_ref: String,
    pub ev_rule: String,
    pub ev_multiplier: f64,
    pub model_code: String,
    pub last_liquidity: i32,
}

impl Execution {
    pub fn new(
        exec_id: String,
        time: String,
        acct_number: String,
        exchange: String,
        side: String,
        shares: f64,
        price: f64,
        perm_id: i32,
        client_id: i32,
        order_id: i32,
        liquidation: i32,
        cum_qty: f64,
        avg_price: f64,
        order_ref: String,
        ev_rule: String,
        ev_multiplier: f64,
        model_code: String,
        last_liquidity: i32,
    ) -> Self {
        Execution {
            exec_id,
            time,
            acct_number,
            exchange,
            side,
            shares,
            price,
            perm_id,
            client_id,
            order_id,
            liquidation,
            cum_qty,
            avg_price,
            order_ref,
            ev_rule,
            ev_multiplier,
            model_code,
            last_liquidity,
        }
    }
}

impl Display for Execution {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(
            f,
            "exec_id: : {},
            time: : {},
            acct_number: : {},
            exchange: : {},
            side: : {},
            shares: : {},
            price: : {},
            perm_id: : {},
            client_id: : {},
            order_id: : {},
            liquidation: : {},
            cum_qty: : {},
            avg_price: : {},
            order_ref: : {},
            ev_rule: : {},
            ev_multiplier: : {},
            model_code: : {},
            last_liquidity: : {} ",
            self.exec_id,
            self.time,
            self.acct_number,
            self.exchange,
            self.side,
            self.shares,
            self.price,
            self.perm_id,
            self.client_id,
            self.order_id,
            self.liquidation,
            self.cum_qty,
            self.avg_price,
            self.order_ref,
            self.ev_rule,
            self.ev_multiplier,
            self.model_code,
            self.last_liquidity,
        )
    }
}

//==================================================================================================
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ExecutionFilter {
    pub client_id: i32,
    pub acct_code: String,
    pub time: String,
    pub symbol: String,
    pub sec_type: String,
    pub exchange: String,
    pub side: String,
}

impl ExecutionFilter {
    pub fn new(
        client_id: i32,
        acct_code: String,
        time: String,
        symbol: String,
        sec_type: String,
        exchange: String,
        side: String,
    ) -> Self {
        ExecutionFilter {
            client_id,
            acct_code,
            time,
            symbol,
            sec_type,
            exchange,
            side,
        }
    }
}
