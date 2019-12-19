//==================================================================================================
pub struct Execution {
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

//def __str__(self):
//return "ExecId: %s, Time: %s, Account: %s, Exchange: %s, Side: %s, Shares: %f, Price: %f, PermId: %d, " \
//"ClientId: %d, OrderId: %d, Liquidation: %d, CumQty: %f, AvgPrice: %f, OrderRef: %s, EvRule: %s, " \
//"EvMultiplier: %f, ModelCode: %s, LastLiquidity: %d" % (self.exec_id, self.time, self.acct_number,
//self.exchange, self.side, self.shares, self.price, self.perm_id, self.client_id, self.order_id, self.liquidation,
//self.cum_qty, self.avg_price, self.order_ref, self.ev_rule, self.ev_multiplier, self.model_code, self.lastLiquidity)

//==================================================================================================
pub struct ExecutionFilter {
    client_id: i32,
    acct_code: String,
    time: String,
    symbol: String,
    sec_type: String,
    exchange: String,
    side: String,
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
