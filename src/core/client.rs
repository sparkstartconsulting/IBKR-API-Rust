use std::borrow::{Borrow, BorrowMut, Cow};
use std::io::Write;
use std::marker::Sync;
use std::net::Shutdown;
use std::net::TcpStream;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use std::thread;

use encoding::Encoding;
use from_ascii::FromAscii;
use log::*;
use log4rs::append::Append;
use num_derive::FromPrimitive;
// 0.2.4 (the derive)
use num_traits::FromPrimitive;

use crate::core::common::*;
use crate::core::contract::Contract;
use crate::core::decoder::Decoder;
use crate::core::errors::{IBKRApiLibError, TwsError};
use crate::core::execution::ExecutionFilter;
use crate::core::messages::make_field;
use crate::core::messages::{make_field_handle_empty, read_msg};
use crate::core::messages::{make_message, read_fields, OutgoingMessageIds};
use crate::core::order::Order;
use crate::core::order_condition::Condition;
use crate::core::reader::Reader;
use crate::core::scanner::ScannerSubscription;
use crate::core::server_versions::*;
use crate::core::wrapper::Wrapper;

// 0.2.6 (the trait)

#[repr(i32)]
#[derive(FromPrimitive, Copy, Clone)]
pub enum ConnStatus {
    DISCONNECTED,
    CONNECTING,
    CONNECTED,
    REDIRECT,
}

pub struct EClient<T: Wrapper + Sync + Send> {
    //decoder: Decoder<'a, T>,
    wrapper: Arc<Mutex<T>>,
    done: bool,
    n_keyb_int_hard: i32,
    stream: Option<TcpStream>,
    host: String,
    port: u32,
    extra_auth: bool,
    client_id: i32,
    server_version: i32,
    conn_time: String,
    pub conn_state: Arc<Mutex<ConnStatus>>,
    opt_capab: String,
    asynchronous: bool,
    disconnect_requested: Arc<AtomicBool>,
}

impl<T> EClient<T>
where
    T: Wrapper + Sync + Send + 'static,
{
    pub fn new(the_wrapper: Arc<Mutex<T>>) -> Self {
        EClient {
            wrapper: the_wrapper,
            //decoder: Decoder::new(the_wrapper, 0),
            done: false,
            n_keyb_int_hard: 0,
            stream: None,
            host: "".to_string(),
            port: 0,
            extra_auth: false,
            client_id: 0,
            server_version: 0,
            conn_time: "".to_string(),
            conn_state: Arc::new(Mutex::new(ConnStatus::DISCONNECTED)),
            opt_capab: "".to_string(),
            asynchronous: false,
            disconnect_requested: Arc::new(AtomicBool::new(false)),
        }
    }
    fn send_request(&self, request: &str) {
        let bytes = make_message(request);
        self.send_bytes(bytes.as_slice());
    }

    fn send_bytes(&self, bytes: &[u8]) {
        self.stream.as_ref().unwrap().write(bytes);
    }

    //----------------------------------------------------------------------------------------------
    pub fn connect(
        &mut self,
        host: String,
        port: u32,
        client_id: i32,
    ) -> Result<(), IBKRApiLibError> {
        self.host = host;
        self.port = port;
        self.client_id = client_id;
        debug!("Connecting");
        self.disconnect_requested.store(false, Ordering::Release);
        *self.conn_state.lock().unwrap().deref_mut() = ConnStatus::CONNECTING;
        let thestream = TcpStream::connect(format!("{}:{}", self.host.to_string(), port)).unwrap();
        *self.conn_state.lock().unwrap().deref_mut() = ConnStatus::CONNECTED;
        debug!("Connected");

        self.stream = Option::from(thestream.try_clone().unwrap());

        let _reader_stream = thestream.try_clone().unwrap();

        let (tx, rx) = channel::<String>();
        let mut reader = Reader::new(thestream, tx.clone(), self.disconnect_requested.clone());

        //self.msg_queue = Option::from(Mutex::new(rx));

        let v_100_prefix = "API\0";
        let v_100_version = format!("v{}..{}", MIN_CLIENT_VER, MAX_CLIENT_VER);

        let msg = make_message(v_100_version.as_str());

        let mut bytearray: Vec<u8> = Vec::new();
        bytearray.extend_from_slice(v_100_prefix.as_bytes());
        bytearray.extend_from_slice(msg.as_slice());

        self.send_bytes(bytearray.as_slice());
        let mut fields: Vec<String> = Vec::new();

        //let mut decoder = Decoder::new(self.wrapper.clone(), rx, self.server_version);
        let mut decoder = Decoder::new(
            self.wrapper.clone(),
            rx,
            self.server_version,
            self.conn_state.clone(),
        );
        //sometimes I get news before the server version, thus the loop
        while fields.len() != 2 {
            if fields.len() > 0 {
                decoder.interpret(fields.as_slice())?;
            }

            let buf = reader.recv_packet()?;

            if buf.len() > 0 {
                let (_size, msg, _remaining_messages) = read_msg(buf.as_slice());

                fields.clear();
                fields.extend_from_slice(read_fields(msg.as_ref()).as_slice());
            } else {
                fields.clear();
            }
        }

        self.server_version = i32::from_ascii(fields.get(0).unwrap().as_bytes()).unwrap();

        info!("Server version: {}", self.server_version);

        self.conn_time = fields.get(1).unwrap().to_string();
        decoder.server_version = self.server_version;

        thread::spawn(move || {
            reader.run();
        });

        thread::spawn(move || {
            decoder.run();
        });
        self.start_api();
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    pub fn is_connected(&self) -> bool {
        //info!("checking connected...");
        let connected = match *self.conn_state.lock().unwrap().deref() {
            ConnStatus::DISCONNECTED => false,
            ConnStatus::CONNECTED => true,
            ConnStatus::CONNECTING => false,
            ConnStatus::REDIRECT => false,
        };

        //info!("finished checking connected...");
        connected
    }

    //----------------------------------------------------------------------------------------------
    pub fn server_version(&self) -> i32 {
        //        Returns the version of the TWS instance to which the API
        //        application is connected.

        self.server_version
    }

    //----------------------------------------------------------------------------------------------
    pub fn set_server_log_level(&self, log_evel: i32) {
        //The pub fnault detail level is ERROR. For more details, see API
        //        Logging.
        //TODO Make log_level an enum
        debug!("set_server_log_level -- log_evel: {}", log_evel);

        if !self.is_connected() {
            self.wrapper.lock().unwrap().error(
                NO_VALID_ID,
                TwsError::NotConnected.code(),
                TwsError::NotConnected.message(),
            );
            return;
        }

        let version = 1;
        let _log_level = log_evel;

        let mut msg = "".to_string();

        let message_id = OutgoingMessageIds::SetServerLoglevel as i32;
        let _x = message_id.to_be_bytes();
        msg.push_str(&make_field(&message_id));
        msg.push_str(&make_field(&version));
        msg.push_str(&make_field(&_log_level));

        self.send_request(msg.as_str())
    }

    //----------------------------------------------------------------------------------------------
    pub fn tws_connection_time(&mut self) -> String {
        //"""Returns the time the API application made a connection to TWS."""

        self.conn_time.clone()
    }

    //----------------------------------------------------------------------------------------------
    pub fn req_current_time(&self) {
        let version = 2;

        let message_id: i32 = OutgoingMessageIds::ReqCurrentTime as i32;
        let mut msg = "".to_string();
        msg.push_str(&make_field(&message_id));
        msg.push_str(&make_field(&version));

        debug!("Requesting current time: {}", msg.as_str());
        self.send_request(msg.as_str())
    }

    //----------------------------------------------------------------------------------------------
    pub fn disconnect(&mut self) {
        if !self.is_connected() {
            info!("Already disconnected...");
            return;
        }
        info!("Disconnect requested.  Shutting down stream...");
        self.disconnect_requested.store(true, Ordering::Release);
        self.stream.as_mut().unwrap().shutdown(Shutdown::Both);
        *self.conn_state.lock().unwrap().deref_mut() = ConnStatus::DISCONNECTED;
    }

    //----------------------------------------------------------------------------------------------
    fn start_api(&mut self) {
        //Initiates the message exchange between the core application and
        //the TWS/IB Gateway. """

        if !self.is_connected() {
            self.wrapper.lock().unwrap().deref_mut().error(
                NO_VALID_ID,
                TwsError::NotConnected.code(),
                TwsError::NotConnected.message(),
            );
            return;
        }

        let version = 2;
        let mut opt_capab = "".to_string();
        if self.server_version >= MIN_SERVER_VER_OPTIONAL_CAPABILITIES as i32 {
            opt_capab = make_field(&self.opt_capab);
        }

        let msg = format!(
            "{}{}{}{}",
            make_field(&mut (Some(OutgoingMessageIds::StartApi).unwrap() as i32)),
            make_field(&mut version.to_string()),
            make_field(&mut self.client_id.to_string()),
            opt_capab
        );

        self.send_request(msg.as_str())
    }

    //##############################################################################################
    //################################### Market Data
    //##############################################################################################

    pub fn req_mkt_data(
        &mut self,
        req_id: i32,
        contract: &Contract,
        generic_tick_list: &'static str,
        snapshot: bool,
        regulatory_snapshot: bool,
        mkt_data_options: Vec<TagValue>,
    ) {
        //        """Call this function to request market data. The market data
        //                will be returned by the tickPrice and tickSize events.
        //
        //                req_id: TickerId - The ticker id. Must be a unique value. When the
        //                    market data returns, it will be identified by this tag. This is
        //                    also used when canceling the market data.
        //                contract:&Contract - This structure contains a description of the
        //                    Contractt for which market data is being requested.
        //                generic_tick_list:&'static str - A commma delimited list of generic tick types.
        //                    Tick types can be found in the Generic Tick Types page.
        //                    Prefixing w/ 'mdoff' indicates that top mkt data shouldn't tick.
        //                    You can specify the news source by postfixing w/ ':<source>.
        //                    Example: "mdoff, 292: FLY + BRF"
        //                snapshot:bool - Check to return a single snapshot of Market data and
        //                    have the market data subscription cancel. Do not enter any
        //                    genericTicklist values if you use snapshots.
        //                regulatory_snapshot: bool - With the US Value Snapshot Bundle for stocks,
        //                    regulatory snapshots are available for 0.01 USD each.
        //                mktDataOptions:Vec<TagValue> - For internal use only.
        //                    Use pub fnault value XYZ. """

        //// self.logRequest(current_fn_name()); vars())

        if !self.is_connected() {
            self.wrapper.lock().unwrap().deref_mut().error(
                NO_VALID_ID,
                TwsError::NotConnected.code(),
                TwsError::NotConnected.message(),
            );
            return;
        }

        if self.server_version() < MIN_SERVER_VER_DELTA_NEUTRAL {
            if let Some(_value) = &contract.delta_neutral_contract {
                self.wrapper.lock().unwrap().error(
                    req_id,
                    TwsError::UpdateTws.code(),
                    format!(
                        "{}{}",
                        TwsError::UpdateTws.message(),
                        "  It does not support delta-neutral orders."
                    )
                    .as_ref(),
                );
                return;
            }
        }

        if self.server_version() < MIN_SERVER_VER_REQ_MKT_DATA_CONID && contract.con_id > 0 {
            self.wrapper.lock().unwrap().error(
                req_id,
                TwsError::UpdateTws.code(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    "  It does not support con_id parameter."
                )
                .as_ref(),
            );
            return;
        }

        if self.server_version() < MIN_SERVER_VER_TRADING_CLASS && "" != contract.trading_class {
            self.wrapper.lock().unwrap().error(
                req_id,
                TwsError::UpdateTws.code(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    "  It does not support trading_class parameter in req_mkt_data."
                )
                .as_ref(),
            );
            return;
        }

        let version = 11;

        let message_id: i32 = OutgoingMessageIds::ReqMktData as i32;

        let mut msg = "".to_string();

        // send req mkt data msg
        msg.push_str(&make_field(&message_id));
        msg.push_str(&make_field(&version));
        msg.push_str(&make_field(&req_id));

        // send contract fields
        if self.server_version() >= MIN_SERVER_VER_REQ_MKT_DATA_CONID {
            msg.push_str(&make_field(&contract.con_id));
            msg.push_str(&make_field(&contract.symbol));

            msg.push_str(&make_field(&contract.sec_type));
            msg.push_str(&make_field(&contract.last_trade_date_or_contract_month));
            msg.push_str(&make_field(&contract.strike));
            msg.push_str(&make_field(&contract.right));
            msg.push_str(&make_field(&contract.multiplier)); // srv v15 and above
            msg.push_str(&make_field(&contract.exchange));
            msg.push_str(&make_field(&contract.primary_exchange)); // srv v14 and above
            msg.push_str(&make_field(&contract.currency));
            msg.push_str(&make_field(&contract.local_symbol)); //  srv v2 and above
        }

        if self.server_version() >= MIN_SERVER_VER_TRADING_CLASS {
            msg.push_str(&make_field(&contract.trading_class));
        }
        // Send combo legs for BAG requests(srv v8 and above)
        if contract.sec_type == "BAG" {
            let combo_legs_count = contract.combo_legs.len();
            msg.push_str(&make_field(&combo_legs_count));
            for combo_leg in &contract.combo_legs {
                msg.push_str(&make_field(&combo_leg.con_id));
                msg.push_str(&make_field(&combo_leg.ratio));
                msg.push_str(&make_field(&combo_leg.action));
                msg.push_str(&make_field(&combo_leg.exchange));
            }
        }

        if self.server_version() >= MIN_SERVER_VER_DELTA_NEUTRAL {
            if contract.delta_neutral_contract.is_some() {
                msg.push_str(&make_field(&true));
                msg.push_str(&make_field(
                    &contract.delta_neutral_contract.as_ref().unwrap().con_id,
                ));
                msg.push_str(&make_field(
                    &contract.delta_neutral_contract.as_ref().unwrap().delta,
                ));
                msg.push_str(&make_field(
                    &contract.delta_neutral_contract.as_ref().unwrap().price,
                ));
            } else {
                msg.push_str(&make_field(&false));
            }

            msg.push_str(&make_field(&generic_tick_list)); // srv v31 and above
            msg.push_str(&make_field(&snapshot)); // srv v35 and above
        }

        if self.server_version() >= MIN_SERVER_VER_REQ_SMART_COMPONENTS {
            msg.push_str(&make_field(&regulatory_snapshot));
        }

        // send mktDataOptions parameter
        if self.server_version() >= MIN_SERVER_VER_LINKING {
            // current doc says this part is for "internal use only" -> won't support it
            if mkt_data_options.len() > 0 {
                self.wrapper.lock().unwrap().error(
                    req_id,
                    TwsError::Unsupported.code(),
                    format!(
                        "{}{}",
                        TwsError::Unsupported.message(),
                        "  Internal use only.  mkt_data_options not supported."
                    )
                    .as_ref(),
                );
                return;
            }
            let mkt_data_options_str = "";
            msg.push_str(&make_field(&mkt_data_options_str));
        }

        self.send_request(msg.as_str());
    }

    //----------------------------------------------------------------------------------------------
    pub fn cancel_mkt_data(&mut self, req_id: i32) {
        //        """After calling this function, market data for the specified id
        //        will stop flowing.
        //
        //        reqId: TickerId - The ID that was specified in the call to
        //            reqMktData(). """

        //// self.logRequest(current_fn_name()); vars())

        if !self.is_connected() {
            self.wrapper.lock().unwrap().deref_mut().error(
                NO_VALID_ID,
                TwsError::NotConnected.code(),
                TwsError::NotConnected.message(),
            );
            return;
        }

        let version = 2;

        let message_id: i32 = OutgoingMessageIds::CancelMktData as i32;
        let mut msg = "".to_string();
        msg.push_str(&make_field(&message_id));
        msg.push_str(&make_field(&version));
        msg.push_str(&make_field(&req_id));

        self.send_request(msg.as_str())
    }

    //----------------------------------------------------------------------------------------------
    pub fn req_market_data_type(&mut self, market_data_type: i32) {
        // The API can receive frozen market data from Trader \
        // Workstation. Frozen market data is the last data recorded in our system. \
        // During normal trading hours, the API receives real-time market data. If \
        // you use this function, you are telling TWS to automatically switch to \
        // frozen market data after the close. Then, before the opening of the next \
        // trading day, market data will automatically switch back to real-time \
        // market data. \
        //
        // marketDataType:i32 - 1 for real-time streaming market data || 2 for \
        // frozen market data"

        // // self.logRequest(current_fn_name()); vars())

        if !self.is_connected() {
            self.wrapper.lock().unwrap().deref_mut().error(
                NO_VALID_ID,
                TwsError::NotConnected.code(),
                TwsError::NotConnected.message(),
            );
            return;
        }

        if self.server_version() < MIN_SERVER_VER_REQ_MARKET_DATA_TYPE {
            self.wrapper.lock().unwrap().error(
                NO_VALID_ID,
                TwsError::UpdateTws.code(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    "  It does not support market data type requests."
                )
                .as_ref(),
            );
            return;
        }

        let mut msg = "".to_string();
        let version = 1;
        let message_id = OutgoingMessageIds::ReqMarketDataType as i32;

        msg.push_str(&make_field(&message_id));
        msg.push_str(&make_field(&version));
        msg.push_str(&make_field(&market_data_type));

        self.send_request(msg.as_str());
    }

    //----------------------------------------------------------------------------------------------
    pub fn req_smart_components(&mut self, req_id: i32, bbo_exchange: &'static str) {
        //// self.logRequest(current_fn_name()); vars())

        if !self.is_connected() {
            self.wrapper.lock().unwrap().deref_mut().error(
                NO_VALID_ID,
                TwsError::NotConnected.code(),
                TwsError::NotConnected.message(),
            );
            return;
        }

        if self.server_version() < MIN_SERVER_VER_REQ_SMART_COMPONENTS {
            self.wrapper.lock().unwrap().error(
                NO_VALID_ID,
                TwsError::UpdateTws.code(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    "  It does not support smart components request."
                )
                .as_ref(),
            );
            return;
        }

        let mut msg = "".to_string();

        let message_id = OutgoingMessageIds::ReqSmartComponents as i32;

        msg.push_str(&make_field(&message_id));
        msg.push_str(&make_field(&req_id));
        msg.push_str(&make_field(&bbo_exchange));

        self.send_request(msg.as_str());
    }

    //----------------------------------------------------------------------------------------------
    pub fn req_market_rule(&mut self, market_rule_id: i32) {
        //// self.logRequest(current_fn_name()); vars())

        if !self.is_connected() {
            self.wrapper.lock().unwrap().deref_mut().error(
                NO_VALID_ID,
                TwsError::NotConnected.code(),
                TwsError::NotConnected.message(),
            );
            return;
        }

        if self.server_version() < MIN_SERVER_VER_MARKET_RULES {
            self.wrapper.lock().unwrap().error(
                NO_VALID_ID,
                TwsError::UpdateTws.code(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    " It does not support market rule requests."
                )
                .as_ref(),
            );
            return;
        }

        let mut msg = "".to_string();

        let message_id = OutgoingMessageIds::ReqMarketRule as i32;

        msg.push_str(&make_field(&message_id));
        msg.push_str(&make_field(&market_rule_id));

        self.send_request(msg.as_str());
    }

    //----------------------------------------------------------------------------------------------
    pub fn req_tick_by_tick_data(
        &mut self,
        req_id: i32,
        contract: Contract,
        tick_type: TickType,
        number_of_ticks: i32,
        ignore_size: bool,
    ) {
        //// self.logRequest(current_fn_name()); vars())

        if !self.is_connected() {
            self.wrapper.lock().unwrap().deref_mut().error(
                NO_VALID_ID,
                TwsError::NotConnected.code(),
                TwsError::NotConnected.message(),
            );
            return;
        }

        if self.server_version() < MIN_SERVER_VER_TICK_BY_TICK {
            self.wrapper.lock().unwrap().error(
                NO_VALID_ID,
                TwsError::UpdateTws.code(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    " It does not support tick-by-tick data requests."
                )
                .as_ref(),
            );
            return;
        }

        if self.server_version() < MIN_SERVER_VER_TICK_BY_TICK_IGNORE_SIZE {
            self.wrapper.lock().unwrap().error(NO_VALID_ID,
                                               TwsError::UpdateTws.code(),
                                               format!("{}{}", TwsError::UpdateTws.message(),
                                                               " It does not support ignore_size && number_of_ticks parameters in tick-by-tick data requests.").as_ref());
            return;
        }

        let mut msg = "".to_string();

        let message_id = OutgoingMessageIds::ReqTickByTickData as i32;

        msg.push_str(&make_field(&message_id));

        //    msg.push_str(&make_field(&OUT.REQ_TICK_BY_TICK_DATA)\
        msg.push_str(&make_field(&req_id));
        msg.push_str(&make_field(&contract.con_id));
        msg.push_str(&make_field(&contract.symbol));
        msg.push_str(&make_field(&contract.sec_type));
        msg.push_str(&make_field(&contract.last_trade_date_or_contract_month));
        msg.push_str(&make_field(&contract.strike));
        msg.push_str(&make_field(&contract.right));
        msg.push_str(&make_field(&contract.multiplier));
        msg.push_str(&make_field(&contract.exchange));
        msg.push_str(&make_field(&contract.primary_exchange));
        msg.push_str(&make_field(&contract.currency));
        msg.push_str(&make_field(&contract.local_symbol));
        msg.push_str(&make_field(&contract.trading_class));
        msg.push_str(&make_field(&tick_type.value().to_string()));

        if self.server_version() >= MIN_SERVER_VER_TICK_BY_TICK_IGNORE_SIZE {
            msg.push_str(&make_field(&number_of_ticks));
            msg.push_str(&make_field(&ignore_size));
        }

        self.send_request(msg.as_str());
    }

    //----------------------------------------------------------------------------------------------
    pub fn cancel_tick_by_tick_data(&mut self, req_id: i32) {
        //// self.logRequest(current_fn_name()); vars())

        if !self.is_connected() {
            self.wrapper.lock().unwrap().deref_mut().error(
                NO_VALID_ID,
                TwsError::NotConnected.code(),
                TwsError::NotConnected.message(),
            );
            return;
        }

        if self.server_version() < MIN_SERVER_VER_TICK_BY_TICK {
            self.wrapper.lock().unwrap().error(
                NO_VALID_ID,
                TwsError::UpdateTws.code(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    " It does not support tick-by-tick data requests."
                )
                .as_ref(),
            );
            return;
        }

        let mut msg = "".to_string();

        let message_id = OutgoingMessageIds::CancelTickByTickData as i32;

        msg.push_str(&make_field(&message_id));
        msg.push_str(&make_field(&req_id));

        self.send_request(msg.as_str());
    }

    //##########################################################################
    //################## Options
    //##########################################################################

    pub fn calculate_implied_volatility(
        &mut self,
        req_id: i32,
        contract: Contract,
        option_price: f64,
        under_price: f64,
        impl_vol_options: Vec<TagValue>,
    ) {
        //        Call this function to calculate volatility for a supplied
        //        option price and underlying price. Result will be delivered
        //        via EWrapper.tickOptionComputation()
        //
        //        reqId:i32 -  The request id.
        //        contract:&Contract -  Describes the contract.
        //        optionPrice:double - The price of the option.
        //        underPrice:double - Price of the underlying.

        // // self.logRequest(current_fn_name()); vars())

        if !self.is_connected() {
            self.wrapper.lock().unwrap().deref_mut().error(
                NO_VALID_ID,
                TwsError::NotConnected.code(),
                TwsError::NotConnected.message(),
            );
            return;
        }

        if self.server_version() < MIN_SERVER_VER_REQ_CALC_IMPLIED_VOLAT {
            self.wrapper.lock().unwrap().error(
                req_id,
                TwsError::UpdateTws.code(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    "  It does not support calculate_implied_volatility req."
                )
                .as_ref(),
            );
            return;
        }

        if self.server_version() < MIN_SERVER_VER_TRADING_CLASS && "" != contract.trading_class {
            self.wrapper.lock().unwrap().error(
                req_id,
                TwsError::UpdateTws.code(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    "  It does not support trading_class parameter in calculate_implied_volatility."
                )
                .as_ref(),
            );
            return;
        }

        let version = 3;

        let mut msg = "".to_string();

        let message_id = OutgoingMessageIds::CancelCalcImpliedVolat as i32;

        msg.push_str(&make_field(&message_id));

        msg.push_str(&make_field(&version));
        msg.push_str(&make_field(&req_id));

        // send contract fields
        msg.push_str(&make_field(&contract.con_id));
        msg.push_str(&make_field(&contract.symbol));
        msg.push_str(&make_field(&contract.sec_type));
        msg.push_str(&make_field(&contract.last_trade_date_or_contract_month));
        msg.push_str(&make_field(&contract.strike));
        msg.push_str(&make_field(&contract.right));
        msg.push_str(&make_field(&contract.multiplier));
        msg.push_str(&make_field(&contract.exchange));
        msg.push_str(&make_field(&contract.primary_exchange));
        msg.push_str(&make_field(&contract.currency));
        msg.push_str(&make_field(&contract.local_symbol));

        if self.server_version() >= MIN_SERVER_VER_TRADING_CLASS {
            msg.push_str(&make_field(&contract.trading_class));
        }

        msg.push_str(&make_field(&option_price));
        msg.push_str(&make_field(&under_price));

        if self.server_version() >= MIN_SERVER_VER_LINKING {
            let _impl_vol_opt_str = "".to_string();
            let tag_values_count = impl_vol_options.len();
            if tag_values_count > 0 {
                let impl_vol_opt_str = impl_vol_options
                    .iter()
                    .map(|x| format!("{}={};", x.tag, x.value))
                    .collect::<String>();

                msg.push_str(&make_field(&tag_values_count));
                msg.push_str(&make_field(&impl_vol_opt_str));
            }
        }

        self.send_request(msg.as_str());
    }

    //----------------------------------------------------------------------------------------------
    pub fn calculate_option_price(
        &mut self,
        req_id: i32,
        contract: &Contract,
        volatility: f64,
        under_price: f64,
        opt_prc_options: Vec<TagValue>,
    ) {
        //        Call this function to calculate option price and greek values
        //        for a supplied volatility and underlying price.
        //
        //        req_id:i32 -    The ticker ID.
        //        contract:&Contract - Describes the contract.
        //        volatility:double - The volatility.
        //        under_price:double - Price of the underlying.
        //// self.logRequest(current_fn_name()); vars())

        if !self.is_connected() {
            self.wrapper.lock().unwrap().deref_mut().error(
                NO_VALID_ID,
                TwsError::NotConnected.code(),
                TwsError::NotConnected.message(),
            );
            return;
        }

        if self.server_version() < MIN_SERVER_VER_REQ_CALC_IMPLIED_VOLAT {
            self.wrapper.lock().unwrap().error(
                req_id,
                TwsError::UpdateTws.code(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    "  It does not support calculateImpliedVolatility req."
                )
                .as_ref(),
            );
            return;
        }

        if self.server_version() < MIN_SERVER_VER_TRADING_CLASS {
            if "" != contract.trading_class {
                self.wrapper.lock().unwrap().error(req_id, TwsError::UpdateTws.code(),
                                                   format!("{}{}", TwsError::UpdateTws.message(),
                                                                   "  It does not support trading_class parameter in calculateImpliedVolatility.").as_ref());
                return;
            }
        }

        let version = 3;

        // send req mkt data msg
        let mut msg = "".to_string();

        let message_id = OutgoingMessageIds::ReqCalcOptionPrice as i32;

        msg.push_str(&make_field(&message_id));
        msg.push_str(&make_field(&version));
        msg.push_str(&make_field(&req_id));
        // send contract fields
        msg.push_str(&make_field(&contract.con_id));
        msg.push_str(&make_field(&contract.symbol));
        msg.push_str(&make_field(&contract.sec_type));
        msg.push_str(&make_field(&contract.last_trade_date_or_contract_month));
        msg.push_str(&make_field(&contract.strike));
        msg.push_str(&make_field(&contract.right));
        msg.push_str(&make_field(&contract.multiplier));
        msg.push_str(&make_field(&contract.exchange));
        msg.push_str(&make_field(&contract.primary_exchange));
        msg.push_str(&make_field(&contract.currency));
        msg.push_str(&make_field(&contract.local_symbol));

        if self.server_version() >= MIN_SERVER_VER_TRADING_CLASS {
            msg.push_str(&make_field(&contract.trading_class));
        }

        msg.push_str(&make_field(&volatility));
        msg.push_str(&make_field(&under_price));

        if self.server_version() >= MIN_SERVER_VER_LINKING {
            let _opt_prc_opt_str = "".to_string();
            let tag_values_count = opt_prc_options.len();
            if tag_values_count > 0 {
                let opt_prc_opt_str = opt_prc_options
                    .iter()
                    .map(|x| format!("{}={};", x.tag, x.value))
                    .collect::<String>();

                msg.push_str(&make_field(&tag_values_count));
                msg.push_str(&make_field(&opt_prc_opt_str));
            }
        }
        self.send_request(msg.as_str());
    }

    //----------------------------------------------------------------------------------------------
    pub fn cancel_calculate_option_price(&mut self, req_id: i32) {
        //        Call this function to cancel a request to calculate the option
        //        price and greek values for a supplied volatility and underlying price.
        //
        //        req_id:i32 - The request ID.

        //// self.logRequest(current_fn_name()); vars())

        if !self.is_connected() {
            self.wrapper.lock().unwrap().deref_mut().error(
                NO_VALID_ID,
                TwsError::NotConnected.code(),
                TwsError::NotConnected.message(),
            );
            return;
        }

        if self.server_version() < MIN_SERVER_VER_REQ_CALC_IMPLIED_VOLAT {
            self.wrapper.lock().unwrap().error(
                req_id,
                TwsError::UpdateTws.code(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    "  It does not support calculateImpliedVolatility req."
                )
                .as_ref(),
            );
            return;
        }

        let version = 1;

        let mut msg = "".to_string();

        let message_id = OutgoingMessageIds::CancelCalcOptionPrice as i32;

        msg.push_str(&make_field(&message_id));
        msg.push_str(&make_field(&version));
        msg.push_str(&make_field(&req_id));

        self.send_request(msg.as_str());
    }

    //----------------------------------------------------------------------------------------------
    pub fn exercise_options(
        &mut self,
        req_id: i32,
        contract: &Contract,
        exercise_action: i32,
        exercise_quantity: i32,
        account: &'static str,
        over_ride: i32,
    ) {
        //        req_id:i32 - The ticker id. multipleust be a unique value.
        //        contract:&Contract - This structure contains a description of the
        //            contract to be exercised
        //        exercise_action:i32 - Specifies whether you want the option to lapse
        //            || be exercised.
        //            Values are 1 = exercise, 2 = lapse.
        //        exercise_quantity:i32 - The quantity you want to exercise.
        //        account:&'static str - destination account
        //        override:i32 - Specifies whether your setting will override the system's
        //            natural action. For example, if your action is "exercise" and the
        //            option is not in-the-money, by natural action the option would not
        //            exercise. If you have override set to "yes" the natural action would
        //             be overridden and the out-of-the money option would be exercised.
        //            Values are: 0 = no, 1 = yes.

        //// self.logRequest(current_fn_name()); vars())

        if !self.is_connected() {
            self.wrapper.lock().unwrap().deref_mut().error(
                NO_VALID_ID,
                TwsError::NotConnected.code(),
                TwsError::NotConnected.message(),
            );
            return;
        }

        if self.server_version() < MIN_SERVER_VER_TRADING_CLASS {
            if !contract.trading_class.is_empty() {
                self.wrapper.lock().unwrap().error(
                    req_id, TwsError::UpdateTws.code(),
                    format!("{}{}", TwsError::UpdateTws.message(),
                            "  It does not support con_id, multiplier, trading_class parameter in exercise_options.").as_ref());
                return;
            }
        }

        let version = 2;

        // send req mkt data msg
        let mut msg = "".to_string();

        let message_id = OutgoingMessageIds::ExerciseOptions as i32;

        msg.push_str(&make_field(&message_id));

        msg.push_str(&make_field(&version));
        msg.push_str(&make_field(&req_id));

        // send contract fields
        if self.server_version() >= MIN_SERVER_VER_TRADING_CLASS {
            msg.push_str(&make_field(&contract.con_id));
        }
        msg.push_str(&make_field(&contract.symbol));
        msg.push_str(&make_field(&contract.sec_type));
        msg.push_str(&make_field(&contract.last_trade_date_or_contract_month));
        msg.push_str(&make_field(&contract.strike));
        msg.push_str(&make_field(&contract.right));
        msg.push_str(&make_field(&contract.multiplier));
        msg.push_str(&make_field(&contract.exchange));
        msg.push_str(&make_field(&contract.currency));
        msg.push_str(&make_field(&contract.local_symbol));
        if self.server_version() >= MIN_SERVER_VER_TRADING_CLASS {
            msg.push_str(&make_field(&contract.trading_class));
        }
        msg.push_str(&make_field(&exercise_action));
        msg.push_str(&make_field(&exercise_quantity));
        msg.push_str(&make_field(&account));
        msg.push_str(&make_field(&over_ride));

        self.send_request(msg.as_str());
    }

    //#########################################################################
    //################## Orders
    //########################################################################

    pub fn place_order(&mut self, order_id: i32, contract: &Contract, order: &Order) {
        //        Call this function to place an order. The order status will
        //        be returned by the orderStatus event.
        //
        //        order_id:OrderId - The order id. You must specify a unique value. When the
        //        order START_APItus returns, it will be identified by this tag.
        //            This tag is also used when canceling the order.
        //        contract:&Contract - This structure contains a description of the
        //            contract which is being traded.
        //        order:Order - This structure contains the details of tradedhe order.
        //            Note: Each core MUST connect with a unique clientId.

        //// self.logRequest(current_fn_name()); vars())

        if !self.is_connected() {
            self.wrapper.lock().unwrap().deref_mut().error(
                NO_VALID_ID,
                TwsError::NotConnected.code(),
                TwsError::NotConnected.message(),
            );
            return;
        }

        if self.server_version() < MIN_SERVER_VER_DELTA_NEUTRAL {
            if contract.delta_neutral_contract.is_some() {
                self.wrapper.lock().unwrap().error(
                    order_id,
                    TwsError::UpdateTws.code(),
                    format!(
                        "{}{}",
                        TwsError::UpdateTws.message(),
                        "  It does not support delta-neutral orders."
                    )
                    .as_ref(),
                );
                return;
            }
        }

        if self.server_version() < MIN_SERVER_VER_SCALE_ORDERS2
            && order.scale_subs_level_size != UNSET_INTEGER
        {
            self.wrapper.lock().unwrap().error(
                order_id,
                TwsError::UpdateTws.code(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    "  It does not support Subsequent Level Size for Scale orders."
                )
                .as_ref(),
            );
            return;
        }

        if self.server_version() < MIN_SERVER_VER_ALGO_ORDERS && !order.algo_strategy.is_empty() {
            self.wrapper.lock().unwrap().error(
                order_id,
                TwsError::UpdateTws.code(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    "  It does not support algo orders."
                )
                .as_ref(),
            );
            return;
        }

        if self.server_version() < MIN_SERVER_VER_NOT_HELD && order.not_held {
            self.wrapper.lock().unwrap().error(
                order_id,
                TwsError::UpdateTws.code(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    "  It does not support notHeld parameter."
                )
                .as_ref(),
            );
            return;
        }

        if self.server_version() < MIN_SERVER_VER_SEC_ID_TYPE
            && (!contract.sec_id_type.is_empty() || !contract.sec_id.is_empty())
        {
            self.wrapper.lock().unwrap().error(
                order_id,
                TwsError::UpdateTws.code(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    "  It does not support sec_id_type && secId parameters."
                )
                .as_ref(),
            );
            return;
        }

        if self.server_version() < MIN_SERVER_VER_PLACE_ORDER_CONID && contract.con_id > 0 {
            self.wrapper.lock().unwrap().error(
                order_id,
                TwsError::UpdateTws.code(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    "  It does not support con_id parameter."
                )
                .as_ref(),
            );
            return;
        }

        if self.server_version() < MIN_SERVER_VER_SSHORTX {
            if order.exempt_code != -1 {
                self.wrapper.lock().unwrap().error(
                    order_id,
                    TwsError::UpdateTws.code(),
                    format!(
                        "{}{}",
                        TwsError::UpdateTws.message(),
                        "  It does not support exempt_code parameter."
                    )
                    .as_ref(),
                );
                return;
            }
            if contract.combo_legs.len() > 0
                && contract.combo_legs.iter().any(|x| x.exempt_code != -1)
            {
                self.wrapper.lock().unwrap().error(
                    order_id,
                    TwsError::UpdateTws.code(),
                    format!(
                        "{}{}",
                        TwsError::UpdateTws.message(),
                        "  It does not support exempt_code parameter."
                    )
                    .as_ref(),
                );
                return;
            }
        }
        if self.server_version() < MIN_SERVER_VER_HEDGE_ORDERS && !order.hedge_type.is_empty() {
            self.wrapper.lock().unwrap().error(
                order_id,
                TwsError::UpdateTws.code(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    "  It does not support hedge orders."
                )
                .as_ref(),
            );
            return;
        }

        if self.server_version() < MIN_SERVER_VER_OPT_OUT_SMART_ROUTING
            && order.opt_out_smart_routing
        {
            self.wrapper.lock().unwrap().error(
                order_id,
                TwsError::UpdateTws.code(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    "  It does not support optOutSmartRouting parameter."
                )
                .as_ref(),
            );
            return;
        }

        if self.server_version() < MIN_SERVER_VER_DELTA_NEUTRAL_CONID
            && (order.delta_neutral_con_id > 0
                || !order.delta_neutral_settling_firm.is_empty()
                || !order.delta_neutral_clearing_account.is_empty()
                || !order.delta_neutral_clearing_intent.is_empty())
        {
            self.wrapper.lock().unwrap().error(
                order_id, TwsError::UpdateTws.code(),
                format!("{}{}", TwsError::UpdateTws.message(),
                        "  It does not support deltaNeutral parameters: con_id, SettlingFirm, ClearingAccount, ClearingIntent.").as_ref());
            return;
        }

        if self.server_version() < MIN_SERVER_VER_DELTA_NEUTRAL_OPEN_CLOSE
            && (!order.delta_neutral_open_close.is_empty()
                || order.delta_neutral_short_sale
                || order.delta_neutral_short_sale_slot > 0
                || !order.delta_neutral_designated_location.is_empty())
        {
            self.wrapper.lock().unwrap().error(
                order_id,
                TwsError::UpdateTws.code(),
                format!("{}{}", TwsError::UpdateTws.message(),
                        "  It does not support deltaNeutral parameters: open_close, ShortSale, short_sale_slot, designated_location.").as_ref());
            return;
        }

        if self.server_version() < MIN_SERVER_VER_SCALE_ORDERS3
            && order.scale_price_increment > 0 as f64
            && order.scale_price_increment != UNSET_DOUBLE
            && (order.scale_price_adjust_value != UNSET_DOUBLE
                || order.scale_price_adjust_interval != UNSET_INTEGER
                || order.scale_profit_offset != UNSET_DOUBLE
                || order.scale_auto_reset
                || order.scale_init_position != UNSET_INTEGER
                || order.scale_init_fill_qty != UNSET_INTEGER
                || order.scale_random_percent)
        {
            self.wrapper.lock().unwrap().error(
                order_id,
                TwsError::UpdateTws.code(),
                format!("{}{}", TwsError::UpdateTws.message(),
                        "  It does not support Scale order parameters: PriceAdjustValue, \
                PriceAdjustInterval, ProfitOffset, AutoReset, InitPosition, InitFillQty && RandomPercent").as_ref());
            return;
        }

        if self.server_version() < MIN_SERVER_VER_ORDER_COMBO_LEGS_PRICE
            && contract.sec_type == "BAG"
            && order.order_combo_legs.len() > 0
            && order
                .order_combo_legs
                .iter()
                .any(|x| x.price != UNSET_DOUBLE)
        {
            self.wrapper.lock().unwrap().error(
                order_id,
                TwsError::UpdateTws.code(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    "  It does not support per-leg prices for order combo legs."
                )
                .as_ref(),
            );
            return;
        }

        if self.server_version() < MIN_SERVER_VER_TRAILING_PERCENT
            && order.trailing_percent != UNSET_DOUBLE
        {
            self.wrapper.lock().unwrap().error(
                order_id,
                TwsError::UpdateTws.code(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    "  It does not support trailing percent parameter"
                )
                .as_ref(),
            );
            return;
        }

        if self.server_version() < MIN_SERVER_VER_TRADING_CLASS
            && !contract.trading_class.is_empty()
        {
            self.wrapper.lock().unwrap().error(
                order_id,
                TwsError::UpdateTws.code(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    "  It does not support trading_class parameter in placeOrder."
                )
                .as_ref(),
            );
            return;
        }

        if self.server_version() < MIN_SERVER_VER_SCALE_TABLE
            && (!order.scale_table.is_empty()
                || !order.active_start_time.is_empty()
                || !order.active_stop_time.is_empty())
        {
            self.wrapper.lock().unwrap().error(
                order_id,
                TwsError::UpdateTws.code(),
                format!("{}{}", TwsError::UpdateTws.message(),
                        "  It does not support scaleTable, activeStartTime && activeStopTime parameters").as_ref());
            return;
        }

        if self.server_version() < MIN_SERVER_VER_ALGO_ID && order.algo_id != "" {
            self.wrapper.lock().unwrap().error(
                order_id,
                TwsError::UpdateTws.code(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    "  It does not support algoId parameter"
                )
                .as_ref(),
            );
            return;
        }

        if self.server_version() < MIN_SERVER_VER_ORDER_SOLICITED && order.solicited {
            self.wrapper.lock().unwrap().error(
                order_id,
                TwsError::UpdateTws.code(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    "  It does not support order solicited parameter."
                )
                .as_ref(),
            );
            return;
        }

        if self.server_version() < MIN_SERVER_VER_MODELS_SUPPORT && !order.model_code.is_empty() {
            self.wrapper.lock().unwrap().error(
                order_id,
                TwsError::UpdateTws.code(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    "  It does not support model code parameter."
                )
                .as_ref(),
            );
            return;
        }

        if self.server_version() < MIN_SERVER_VER_EXT_OPERATOR && !order.ext_operator.is_empty() {
            self.wrapper.lock().unwrap().error(
                order_id,
                TwsError::UpdateTws.code(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    "  It does not support ext operator parameter"
                )
                .as_ref(),
            );
            return;
        }

        if self.server_version() < MIN_SERVER_VER_SOFT_DOLLAR_TIER
            && (!order.soft_dollar_tier.name.is_empty() || !order.soft_dollar_tier.val.is_empty())
        {
            self.wrapper.lock().unwrap().error(
                order_id,
                TwsError::UpdateTws.code(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    " It does not support soft dollar tier"
                )
                .as_ref(),
            );
            return;
        }

        if self.server_version() < MIN_SERVER_VER_CASH_QTY && order.cash_qty != 0.0 {
            self.wrapper.lock().unwrap().error(
                order_id,
                TwsError::UpdateTws.code(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    " It does not support cash quantity parameter"
                )
                .as_ref(),
            );
            return;
        }

        if self.server_version() < MIN_SERVER_VER_DECISION_MAKER
            && (!order.mifid2decision_maker.is_empty() || !order.mifid2decision_algo.is_empty())
        {
            self.wrapper.lock().unwrap().error(
                order_id,
                TwsError::UpdateTws.code(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    " It does not support MIFID II decision maker parameters"
                )
                .as_ref(),
            );
            return;
        }

        if self.server_version() < MIN_SERVER_VER_MIFID_EXECUTION
            && (!order.mifid2execution_trader.is_empty() || !order.mifid2execution_algo.is_empty())
        {
            self.wrapper.lock().unwrap().error(
                order_id,
                TwsError::UpdateTws.code(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    " It does not support MIFID II execution parameters"
                )
                .as_ref(),
            );
            return;
        }

        if self.server_version() < MIN_SERVER_VER_AUTO_PRICE_FOR_HEDGE
            && order.dont_use_auto_price_for_hedge
        {
            self.wrapper.lock().unwrap().error(
                order_id,
                TwsError::UpdateTws.code(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    " It does not support dontUseAutoPriceForHedge parameter"
                )
                .as_ref(),
            );
            return;
        }

        if self.server_version() < MIN_SERVER_VER_ORDER_CONTAINER && order.is_oms_container {
            self.wrapper.lock().unwrap().error(
                order_id,
                TwsError::UpdateTws.code(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    " It does not support oms container parameter"
                )
                .as_str(),
            );
            return;
        }

        if self.server_version() < MIN_SERVER_VER_PRICE_MGMT_ALGO && order.use_price_mgmt_algo {
            self.wrapper.lock().unwrap().error(
                order_id,
                TwsError::UpdateTws.code(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    " It does not support Use price management algo requests"
                )
                .as_ref(),
            );
            return;
        }

        let version: i32 = if self.server_version() < MIN_SERVER_VER_NOT_HELD {
            27
        } else {
            45
        };

        //send place order msg
        let mut msg = "".to_string();

        let message_id = OutgoingMessageIds::PlaceOrder as i32;

        msg.push_str(&make_field(&message_id));

        if self.server_version() < MIN_SERVER_VER_ORDER_CONTAINER {
            msg.push_str(&make_field(&version));
        }

        msg.push_str(&make_field(&order_id));

        // send contract fields
        if self.server_version() >= MIN_SERVER_VER_PLACE_ORDER_CONID {
            msg.push_str(&make_field(&contract.con_id));
        }
        msg.push_str(&make_field(&contract.symbol));
        msg.push_str(&make_field(&contract.sec_type));
        msg.push_str(&make_field(&contract.last_trade_date_or_contract_month));
        msg.push_str(&make_field(&contract.strike));
        msg.push_str(&make_field(&contract.right));
        msg.push_str(&make_field(&contract.multiplier)); // srv v15 && above
        msg.push_str(&make_field(&contract.exchange));
        msg.push_str(&make_field(&contract.primary_exchange)); // srv v14 && above
        msg.push_str(&make_field(&contract.currency));
        msg.push_str(&make_field(&contract.local_symbol)); // srv v2 && above

        if self.server_version() >= MIN_SERVER_VER_TRADING_CLASS {
            msg.push_str(&make_field(&contract.trading_class));
        }

        if self.server_version() >= MIN_SERVER_VER_SEC_ID_TYPE {
            msg.push_str(&make_field(&contract.sec_id_type));
            msg.push_str(&make_field(&contract.sec_id));
        }

        // send main order fields
        msg.push_str(&make_field(&order.action));

        if self.server_version() >= MIN_SERVER_VER_FRACTIONAL_POSITIONS {
            msg.push_str(&make_field(&order.total_quantity));
        } else {
            msg.push_str(&make_field(&(order.total_quantity as i32)));
        }

        msg.push_str(&make_field(&order.order_type));

        if self.server_version() < MIN_SERVER_VER_ORDER_COMBO_LEGS_PRICE {
            msg.push_str(&make_field(if order.lmt_price != UNSET_DOUBLE {
                &order.lmt_price
            } else {
                &0
            }));
        } else {
            msg.push_str(&make_field_handle_empty(&order.lmt_price));
        }

        if self.server_version() < MIN_SERVER_VER_TRAILING_PERCENT {
            msg.push_str(&make_field(if order.aux_price != UNSET_DOUBLE {
                &order.aux_price
            } else {
                &0
            }));
        } else {
            msg.push_str(&make_field_handle_empty(&order.aux_price));
        }

        // send extended order fields
        msg.push_str(&make_field(&order.tif));
        msg.push_str(&make_field(&order.oca_group));
        msg.push_str(&make_field(&order.account));
        msg.push_str(&make_field(&order.open_close));
        msg.push_str(&make_field(&(order.origin as i32)));
        msg.push_str(&make_field(&order.order_ref));
        msg.push_str(&make_field(&order.transmit));
        msg.push_str(&make_field(&order.parent_id)); // srv v4 && above
        msg.push_str(&make_field(&order.block_order)); // srv v5 && above
        msg.push_str(&make_field(&order.sweep_to_fill)); // srv v5 && above
        msg.push_str(&make_field(&order.display_size)); // srv v5 && above
        msg.push_str(&make_field(&order.trigger_method)); // srv v5 && above
        msg.push_str(&make_field(&order.outside_rth)); // srv v5 && above
        msg.push_str(&make_field(&order.hidden)); // srv v7 && above

        // Send combo legs for BAG requests (srv v8 && above)
        if contract.sec_type == "BAG" {
            let combo_legs_count = contract.combo_legs.len();
            msg.push_str(&make_field(&combo_legs_count));
            if combo_legs_count > 0 {
                for combo_leg in &contract.combo_legs {
                    msg.push_str(&make_field(&combo_leg.con_id));
                    msg.push_str(&make_field(&combo_leg.ratio));
                    msg.push_str(&make_field(&combo_leg.action));
                    msg.push_str(&make_field(&combo_leg.exchange));
                    msg.push_str(&make_field(&combo_leg.open_close));
                    msg.push_str(&make_field(&combo_leg.short_sale_slot)); //srv v35 && above
                    msg.push_str(&make_field(&combo_leg.designated_location)); // srv v35 && above
                    if self.server_version() >= MIN_SERVER_VER_SSHORTX_OLD {
                        msg.push_str(&make_field(&combo_leg.exempt_code));
                    }
                }
            }
        }

        // Send order combo legs for BAG requests
        if self.server_version() >= MIN_SERVER_VER_ORDER_COMBO_LEGS_PRICE
            && contract.sec_type == "BAG"
        {
            let order_combo_legs_count = order.order_combo_legs.len();
            msg.push_str(&make_field(&order_combo_legs_count));
            if order_combo_legs_count > 0 {
                for order_combo_leg in &order.order_combo_legs {
                    msg.push_str(&make_field_handle_empty(&order_combo_leg.price));
                }
            }
        }

        if self.server_version() >= MIN_SERVER_VER_SMART_COMBO_ROUTING_PARAMS
            && contract.sec_type == "BAG"
        {
            let smart_combo_routing_params_count = order.smart_combo_routing_params.len();
            msg.push_str(&make_field(&smart_combo_routing_params_count));
            if smart_combo_routing_params_count > 0 {
                for tagValue in &order.smart_combo_routing_params {
                    msg.push_str(&make_field(&tagValue.tag));
                    msg.push_str(&make_field(&tagValue.value));
                }
            }
        }

        //    ######################################################################
        //    // Send the shares allocation.
        //    #
        //    # This specifies the number of order shares allocated to each Financial
        //    # Advisor managed account. The format of the allocation string is as
        //    # follows:
        //    #                      <account_code1>/<number_shares1>,<account_code2>/<number_shares2>,...N
        //    # E.g.
        //    #              To allocate 20 shares of a 100 share order to account 'U101' && the
        //    #      residual 80 to account 'U203' enter the following share allocation string:
        //    #          U101/20,U203/80
        //    #####################################################################

        // send deprecated sharesAllocation field
        msg.push_str(&make_field(&"")); // srv v9 && above

        msg.push_str(&make_field(&order.discretionary_amt)); // srv v10 && above
        msg.push_str(&make_field(&order.good_after_time)); // srv v11 && above
        msg.push_str(&make_field(&order.good_till_date)); // srv v12 && above

        msg.push_str(&make_field(&order.fa_group)); // srv v13 && above
        msg.push_str(&make_field(&order.fa_method)); // srv v13 && above
        msg.push_str(&make_field(&order.fa_percentage)); // srv v13 && above
        msg.push_str(&make_field(&order.fa_profile)); // srv v13 && above

        if self.server_version() >= MIN_SERVER_VER_MODELS_SUPPORT {
            msg.push_str(&make_field(&order.model_code));
        }

        // institutional short saleslot data (srv v18 && above)
        msg.push_str(&make_field(&order.short_sale_slot)); // 0 for retail, 1 || 2 for institutions
        msg.push_str(&make_field(&order.designated_location)); // populate only when shortSaleSlot = 2.

        if self.server_version() >= MIN_SERVER_VER_SSHORTX_OLD {
            msg.push_str(&make_field(&order.exempt_code));
        }

        // not needed anymore
        //bool isVolOrder = (order.orderType.CompareNoCase("VOL").as_ref() == 0)

        // srv v19 && above fields
        msg.push_str(&make_field(&order.oca_type));
        //if( self.server_version() < 38) {
        // will never happen
        //      send( /* order.rthOnly */ false);
        //}
        msg.push_str(&make_field(&order.rule80a));
        msg.push_str(&make_field(&order.settling_firm));
        msg.push_str(&make_field(&order.all_or_none));
        msg.push_str(&make_field_handle_empty(&order.min_qty));
        msg.push_str(&make_field_handle_empty(&order.percent_offset));
        msg.push_str(&make_field(&order.e_trade_only));
        msg.push_str(&make_field(&order.firm_quote_only));
        msg.push_str(&make_field_handle_empty(&order.nbbo_price_cap));
        msg.push_str(&make_field(&(order.auction_strategy as i32))); // AUCTION_MATCH, AUCTION_IMPROVEMENT, AUCTION_TRANSPARENT
        msg.push_str(&make_field_handle_empty(&order.starting_price));
        msg.push_str(&make_field_handle_empty(&order.stock_ref_price));
        msg.push_str(&make_field_handle_empty(&order.delta));
        msg.push_str(&make_field_handle_empty(&order.stock_range_lower));
        msg.push_str(&make_field_handle_empty(&order.stock_range_upper));

        msg.push_str(&make_field(&order.override_percentage_constraints)); //srv v22 && above

        // volatility orders (srv v26 && above)
        msg.push_str(&make_field_handle_empty(&order.volatility));
        msg.push_str(&make_field_handle_empty(&order.volatility_type));
        msg.push_str(&make_field(&order.delta_neutral_order_type)); // srv v28 && above
        msg.push_str(&make_field_handle_empty(&order.delta_neutral_aux_price)); // srv v28 && above

        if self.server_version() >= MIN_SERVER_VER_DELTA_NEUTRAL_CONID
            && !order.delta_neutral_order_type.is_empty()
        {
            msg.push_str(&make_field(&order.delta_neutral_con_id));
            msg.push_str(&make_field(&order.delta_neutral_settling_firm));
            msg.push_str(&make_field(&order.delta_neutral_clearing_account));
            msg.push_str(&make_field(&order.delta_neutral_clearing_intent));
        }

        if self.server_version() >= MIN_SERVER_VER_DELTA_NEUTRAL_OPEN_CLOSE
            && order.delta_neutral_order_type != ""
        {
            msg.push_str(&make_field(&order.delta_neutral_open_close));
            msg.push_str(&make_field(&order.delta_neutral_short_sale));
            msg.push_str(&make_field(&order.delta_neutral_short_sale_slot));
            msg.push_str(&make_field(&order.delta_neutral_designated_location));
        }

        msg.push_str(&make_field(&order.continuous_update));
        msg.push_str(&make_field_handle_empty(&order.reference_price_type));
        msg.push_str(&make_field_handle_empty(&order.trail_stop_price)); // srv v30 && above

        if self.server_version() >= MIN_SERVER_VER_TRAILING_PERCENT {
            msg.push_str(&make_field_handle_empty(&order.trailing_percent));
        }

        // SCALE orders
        if self.server_version() >= MIN_SERVER_VER_SCALE_ORDERS2 {
            msg.push_str(&make_field_handle_empty(&order.scale_init_level_size));
            msg.push_str(&make_field_handle_empty(&order.scale_subs_level_size));
        } else {
            // srv v35 && above)
            msg.push_str(&make_field(&"")); // for not supported scaleNumComponents
            msg.push_str(&make_field_handle_empty(&order.scale_init_level_size));
            // for scaleComponentSize
        }

        msg.push_str(&make_field_handle_empty(&order.scale_price_increment));

        if self.server_version() >= MIN_SERVER_VER_SCALE_ORDERS3
            && order.scale_price_increment != UNSET_DOUBLE
            && order.scale_price_increment > 0.0
        {
            msg.push_str(&make_field_handle_empty(&order.scale_price_adjust_value));
            msg.push_str(&make_field_handle_empty(&order.scale_price_adjust_interval));
            msg.push_str(&make_field_handle_empty(&order.scale_profit_offset));
            msg.push_str(&make_field(&order.scale_auto_reset));
            msg.push_str(&make_field_handle_empty(&order.scale_init_position));
            msg.push_str(&make_field_handle_empty(&order.scale_init_fill_qty));
            msg.push_str(&make_field(&order.scale_random_percent));
        }

        if self.server_version() >= MIN_SERVER_VER_SCALE_TABLE {
            msg.push_str(&make_field(&order.scale_table));
            msg.push_str(&make_field(&order.active_start_time));
            msg.push_str(&make_field(&order.active_stop_time));
        }

        // HEDGE orders
        if self.server_version() >= MIN_SERVER_VER_HEDGE_ORDERS {
            msg.push_str(&make_field(&order.hedge_type));

            if !order.hedge_type.is_empty() {
                msg.push_str(&make_field(&order.hedge_param));
            }
        }

        if self.server_version() >= MIN_SERVER_VER_OPT_OUT_SMART_ROUTING {
            msg.push_str(&make_field(&order.opt_out_smart_routing));
        }

        if self.server_version() >= MIN_SERVER_VER_PTA_ORDERS {
            msg.push_str(&make_field(&order.clearing_account));
            msg.push_str(&make_field(&order.clearing_intent));
        }

        if self.server_version() >= MIN_SERVER_VER_NOT_HELD {
            msg.push_str(&make_field(&order.not_held));
        }

        if self.server_version() >= MIN_SERVER_VER_DELTA_NEUTRAL {
            if contract.delta_neutral_contract.is_some() {
                msg.push_str(&make_field(&true));
                msg.push_str(&make_field(
                    &contract.delta_neutral_contract.as_ref().unwrap().con_id,
                ));
                msg.push_str(&make_field(
                    &contract.delta_neutral_contract.as_ref().unwrap().delta,
                ));
                msg.push_str(&make_field(
                    &contract.delta_neutral_contract.as_ref().unwrap().price,
                ));
            } else {
                msg.push_str(&make_field(&false))
            }
        }

        if self.server_version() >= MIN_SERVER_VER_ALGO_ORDERS {
            msg.push_str(&make_field(&order.algo_strategy));
            if !order.algo_strategy.is_empty() {
                let algo_params_count = order.algo_params.len();
                msg.push_str(&make_field(&algo_params_count));
                if algo_params_count > 0 {
                    for algo_param in &order.algo_params {
                        msg.push_str(&make_field(&algo_param.tag));
                        msg.push_str(&make_field(&algo_param.value));
                    }
                }
            }
        }

        if self.server_version() >= MIN_SERVER_VER_ALGO_ID {
            msg.push_str(&make_field(&order.algo_id));
        }

        msg.push_str(&make_field(&order.what_if)); // srv v36 && above

        // send miscOptions parameter
        if self.server_version() >= MIN_SERVER_VER_LINKING {
            let misc_options_str = order
                .order_misc_options
                .iter()
                .map(|x| format!("{}={};", x.tag, x.value))
                .collect::<String>();
            msg.push_str(&make_field(&misc_options_str));
        }

        if self.server_version() >= MIN_SERVER_VER_ORDER_SOLICITED {
            msg.push_str(&make_field(&order.solicited));
        }

        if self.server_version() >= MIN_SERVER_VER_RANDOMIZE_SIZE_AND_PRICE {
            msg.push_str(&make_field(&order.randomize_size));
            msg.push_str(&make_field(&order.randomize_price));
        }

        if self.server_version() >= MIN_SERVER_VER_PEGGED_TO_BENCHMARK {
            if order.order_type == "PEG BENCH" {
                msg.push_str(&make_field(&order.reference_contract_id));
                msg.push_str(&make_field(&order.is_pegged_change_amount_decrease));
                msg.push_str(&make_field(&order.pegged_change_amount));
                msg.push_str(&make_field(&order.reference_change_amount));
                msg.push_str(&make_field(&order.reference_exchange_id));
            }

            msg.push_str(&make_field(&order.conditions.len()));

            if order.conditions.len() > 0 {
                for cond in &order.conditions {
                    msg.push_str(&make_field(&(cond.get_type() as i32)));
                    let mut vals = cond.make_fields();
                    let vals_string = vals.iter().map(|val| val.clone()).collect::<String>();
                    msg.push_str(vals_string.as_ref());
                }

                msg.push_str(&make_field(&order.conditions_ignore_rth));
                msg.push_str(&make_field(&order.conditions_cancel_order));
            }

            msg.push_str(&make_field(&order.adjusted_order_type));
            msg.push_str(&make_field(&order.trigger_price));
            msg.push_str(&make_field(&order.lmt_price_offset));
            msg.push_str(&make_field(&order.adjusted_stop_price));
            msg.push_str(&make_field(&order.adjusted_stop_limit_price));
            msg.push_str(&make_field(&order.adjusted_trailing_amount));
            msg.push_str(&make_field(&order.adjustable_trailing_unit));
        }

        if self.server_version() >= MIN_SERVER_VER_EXT_OPERATOR {
            msg.push_str(&make_field(&order.ext_operator));
        }

        if self.server_version() >= MIN_SERVER_VER_SOFT_DOLLAR_TIER {
            msg.push_str(&make_field(&order.soft_dollar_tier.name));
            msg.push_str(&make_field(&order.soft_dollar_tier.val));
        }

        if self.server_version() >= MIN_SERVER_VER_CASH_QTY {
            msg.push_str(&make_field(&order.cash_qty));
        }

        if self.server_version() >= MIN_SERVER_VER_DECISION_MAKER {
            msg.push_str(&make_field(&order.mifid2decision_maker));
            msg.push_str(&make_field(&order.mifid2decision_algo));
        }

        if self.server_version() >= MIN_SERVER_VER_MIFID_EXECUTION {
            msg.push_str(&make_field(&order.mifid2execution_trader));
            msg.push_str(&make_field(&order.mifid2execution_algo));
        }

        if self.server_version() >= MIN_SERVER_VER_AUTO_PRICE_FOR_HEDGE {
            msg.push_str(&make_field(&order.dont_use_auto_price_for_hedge));
        }

        if self.server_version() >= MIN_SERVER_VER_ORDER_CONTAINER {
            msg.push_str(&make_field(&order.is_oms_container));
        }

        if self.server_version() >= MIN_SERVER_VER_D_PEG_ORDERS {
            msg.push_str(&make_field(&order.discretionary_up_to_limit_price));
        }

        if self.server_version() >= MIN_SERVER_VER_PRICE_MGMT_ALGO {
            msg.push_str(&make_field_handle_empty(&order.use_price_mgmt_algo))
        }

        info!("Placing order {:?}", msg);
        self.send_request(msg.as_str());
    }

    //----------------------------------------------------------------------------------------------
    pub fn cancel_order(&mut self, order_id: i32) {
        //"""Call this function to cancel an order.

        //orderId:OrderId - The order ID that was specified previously in the call
        //to placeOrder()"""

        //// self.logRequest(current_fn_name()); vars())

        if !self.is_connected() {
            self.wrapper.lock().unwrap().error(
                NO_VALID_ID,
                TwsError::NotConnected.code(),
                TwsError::NotConnected.message(),
            );
            return;
        }

        let version = 2;

        let mut msg = "".to_string();

        let message_id = OutgoingMessageIds::CancelOrder as i32;

        msg.push_str(&make_field(&message_id));
        msg.push_str(&make_field(&version));
        msg.push_str(&make_field(&order_id));

        self.send_request(msg.as_str());
    }

    //----------------------------------------------------------------------------------------------
    pub fn req_open_orders(&mut self) {
        //        Call this function to request the open orders that were
        //        placed from this core. Each open order will be fed back through the
        //        openOrder() and orderStatus() functions on the EWrapper.
        //
        //        Note:  The core with a clientId of 0 will also receive the TWS-owned
        //        open orders. These orders will be associated with the core and a new
        //        orderId will be generated. This association will persist over multiple
        //        API and TWS sessions

        //// self.logRequest(current_fn_name()); vars())

        if !self.is_connected() {
            self.wrapper.lock().unwrap().error(
                NO_VALID_ID,
                TwsError::NotConnected.code(),
                TwsError::NotConnected.message(),
            );
            return;
        }

        let version = 1;

        let mut msg = "".to_string();

        let message_id = OutgoingMessageIds::ReqOpenOrders as i32;

        msg.push_str(&make_field(&message_id));
        msg.push_str(&make_field(&version));

        self.send_request(msg.as_str());
    }

    //----------------------------------------------------------------------------------------------
    pub fn req_auto_open_orders(&mut self, b_auto_bind: bool) {
        //        Call this function to request that newly created TWS orders
        //        be implicitly associated with the core.When a new TWS order is
        //        created, the order will be associated with the core, and fed back
        //        through the openOrder() and orderStatus() functions on the EWrapper.
        //
        //            Note: This request can only be made from a core with clientId of 0.
        //
        //        b_auto_bind: If set to TRUE, newly created TWS orders will be implicitly
        //        associated with the core.If set to FALSE, no association will be
        //        made.

        //// self.logRequest(current_fn_name()); vars())

        if !self.is_connected() {
            self.wrapper.lock().unwrap().error(
                NO_VALID_ID,
                TwsError::NotConnected.code(),
                TwsError::NotConnected.message(),
            );
            return;
        }

        let version = 1;

        let mut msg = "".to_string();

        let message_id = OutgoingMessageIds::ReqAutoOpenOrders as i32;

        msg.push_str(&make_field(&message_id));
        msg.push_str(&make_field(&version));
        msg.push_str(&make_field(&b_auto_bind)); // TRUE = subscribe, FALSE = unsubscribe

        self.send_request(msg.as_str());
    }

    //----------------------------------------------------------------------------------------------
    pub fn req_all_open_orders(&mut self) {
        //        Call this function to request the open orders placed from all
        //        clients and also from TWS. Each open order will be fed back through the
        //        openOrder() and orderStatus() functions on the EWrapper.
        //
        //        Note:  No association is made between the returned orders and the
        //        requesting core.

        //// self.logRequest(current_fn_name()); vars())

        if !self.is_connected() {
            self.wrapper.lock().unwrap().error(
                NO_VALID_ID,
                TwsError::NotConnected.code(),
                TwsError::NotConnected.message(),
            );
            return;
        }

        let version = 1;

        let mut msg = "".to_string();

        let message_id = OutgoingMessageIds::ReqAllOpenOrders as i32;

        msg.push_str(&make_field(&message_id));
        msg.push_str(&make_field(&version));

        self.send_request(msg.as_str());
    }

    //----------------------------------------------------------------------------------------------
    pub fn req_global_cancel(&mut self) {
        //        Use this function to cancel all open orders globally. It
        //        cancels both API and TWS open orders.
        //
        //        If the order was created in TWS, it also gets canceled. If the order
        //        was initiated in the API, it also gets canceled.

        //// self.logRequest(current_fn_name()); vars())

        if !self.is_connected() {
            self.wrapper.lock().unwrap().error(
                NO_VALID_ID,
                TwsError::NotConnected.code(),
                TwsError::NotConnected.message(),
            );
            return;
        }

        let version = 1;

        let mut msg = "".to_string();

        let message_id = OutgoingMessageIds::ReqGlobalCancel as i32;

        msg.push_str(&make_field(&message_id));
        msg.push_str(&make_field(&version));

        self.send_request(msg.as_str());
    }

    //----------------------------------------------------------------------------------------------
    pub fn req_ids(&self, num_ids: i32) {
        //        Call this function to request from TWS the next valid ID that
        //        can be used when placing an order.  After calling this function, the
        //        nextValidId() event will be triggered, and the id returned is that next
        //        valid ID. That ID will reflect any autobinding that has occurred (which
        //        generates new IDs and increments the next valid ID therein).
        //
        //        numIds:i32 - deprecated

        //// self.logRequest(current_fn_name()); vars())

        if !self.is_connected() {
            info!("Not connected, sending error...");
            self.wrapper.lock().unwrap().error(
                NO_VALID_ID,
                TwsError::NotConnected.code(),
                TwsError::NotConnected.message(),
            );
            info!("Not connected, returning...");
            return;
        }
        info!("req_ids is connected...");
        let version = 1;

        let mut msg = "".to_string();

        let message_id = OutgoingMessageIds::ReqIds as i32;

        msg.push_str(&make_field(&message_id));
        msg.push_str(&make_field(&version));
        msg.push_str(&make_field(&num_ids));
        info!("req_ids... sending request...");
        self.send_request(msg.as_str());
    }

    //#########################################################################
    //################## Account and Portfolio
    //########################################################################
    pub fn req_account_updates(&mut self, subscribe: bool, acct_code: String) {
        /*Call this function to start getting account values, portfolio,
        and last update time information via EWrapper.updateAccountValue());
        EWrapperi.updatePortfolio() and Wrapper.updateAccountTime().

        subscribe:bool - If set to TRUE, the core will start receiving account
            and Portfoliolio updates. If set to FALSE, the core will stop
            receiving this information.
        acctCode:&'static str -The account code for which to receive account and
            portfolio updates.*/

        info!("subscribe: {}, acct_code: {}", subscribe, acct_code);
        debug!(
            "req_account_updates: subscribe: {}, acct_code: {}",
            subscribe, acct_code
        );

        if !self.is_connected() {
            self.wrapper.lock().unwrap().error(
                NO_VALID_ID,
                TwsError::NotConnected.code(),
                TwsError::NotConnected.message(),
            );
            return;
        }

        let version = 2;

        let mut msg = "".to_string();

        let message_id = OutgoingMessageIds::ReqAcctData as i32;

        msg.push_str(&make_field(&message_id));
        msg.push_str(&make_field(&version));
        msg.push_str(&make_field(&subscribe)); // TRUE = subscribe, FALSE = unsubscribe
        msg.push_str(&make_field(&acct_code)); // srv v9 and above, the account code.This will only be used for FA clients

        self.send_request(msg.as_str());
    }

    //----------------------------------------------------------------------------------------------
    pub fn req_account_summary(&self, req_id: i32, group_name: String, tags: String) {
        /* Call this method to request and keep up to date the data that appears
        on the TWS Account Window Summary tab. The data is returned by
        accountSummary().

        Note:   This request is designed for an FA managed account but can be
        used for any multi-account structure.

        req_id:i32 - The ID of the data request. Ensures that responses are matched
            to requests If several requests are in process.
        group_name:&'static str - Set to All to returnrn account summary data for all
            accounts, || set to a specific Advisor Account Group name that has
            already been created in TWS Global Configuration.
        tags:&'static str - A comma-separated list of account tags.  Available tags are:
            accountountType
            NetLiquidation,
            TotalCashValue - Total cash including futures pnl
            SettledCash - For cash accounts, this is the same as
            TotalCashValue
            AccruedCash - Net accrued interest
            BuyingPower - The maximum amount of marginable US stocks the
                account can buy
            EquityWithLoanValue - Cash + stocks + bonds + mutual funds
            PreviousDayEquityWithLoanValue,
            GrossPositionValue - The sum of the absolute value of all stock
                and equity option positions
            RegTEquity,
            RegTMargin,
            SMA - Special Memorandum Account
            InitMarginReq,
            MaintMarginReq,
            AvailableFunds,
            ExcessLiquidity,
            Cushion - Excess liquidity as a percentage of net liquidation value
            FullInitMarginReq,
            FullMaintMarginReq,
            FullAvailableFunds,
            FullExcessLiquidity,
            LookAheadNextChange - Time when look-ahead values take effect
            LookAheadInitMarginReq,
            LookAheadMaintMarginReq,
            LookAheadAvailableFunds,
            LookAheadExcessLiquidity,
            HighestSeverity - A measure of how close the account is to liquidation
            DayTradesRemaining - The Number of Open/Close trades a user
                could put on before Pattern Day Trading is detected. A value of " - 1"
                means that the user can put on unlimited day trades.
            Leverage - GrossPositionValue / NetLiquidation
            $LEDGER - Single flag to relay all cash balance tags*, only in base
                currency.
            $LEDGER:CURRENCY - Single flag to relay all cash balance tags*, only in
                the specified currency.
            $LEDGER:ALL - Single flag to relay all cash balance tags* in all
            currencies.*/

        //// self.logRequest(current_fn_name()); vars())

        if !self.is_connected() {
            self.wrapper.try_lock().unwrap().deref_mut().error(
                NO_VALID_ID,
                TwsError::NotConnected.code(),
                TwsError::NotConnected.message(),
            );
            return;
        }

        let version = 2;
        let _req_id = req_id;
        let _group_name = group_name;
        let _tags = tags;
        let message_id: i32 = OutgoingMessageIds::ReqAccountSummary as i32;
        let mut msg = "".to_string();

        msg.push_str(&make_field(&message_id));
        msg.push_str(&make_field(&version));
        msg.push_str(&make_field(&_req_id));
        msg.push_str(&make_field(&_group_name));
        msg.push_str(&make_field(&_tags));

        self.send_request(msg.as_str())
    }

    //----------------------------------------------------------------------------------------------
    pub fn cancel_account_summary(&mut self, req_id: i32) {
        //        """Cancels the request for Account Window Summary tab data.
        //
        //        reqId:i32 - The ID of the data request being canceled."""

        //// self.logRequest(current_fn_name()); vars())

        if !self.is_connected() {
            self.wrapper.lock().unwrap().deref_mut().error(
                NO_VALID_ID,
                TwsError::NotConnected.code(),
                TwsError::NotConnected.message(),
            );
            return;
        }
        let version = 1;

        let message_id: i32 = OutgoingMessageIds::CancelAccountSummary as i32;
        let mut msg = "".to_string();
        msg.push_str(&make_field(&message_id));
        msg.push_str(&make_field(&version));
        msg.push_str(&make_field(&req_id));

        self.send_request(msg.as_str())
    }

    //----------------------------------------------------------------------------------------------
    pub fn req_positions(&mut self) {
        //"""Requests real-time position data for all accounts."""

        //// self.logRequest(current_fn_name()); vars())

        if !self.is_connected() {
            self.wrapper.lock().unwrap().deref_mut().error(
                NO_VALID_ID,
                TwsError::NotConnected.code(),
                TwsError::NotConnected.message(),
            );
            return;
        }

        if self.server_version() < MIN_SERVER_VER_POSITIONS {
            self.wrapper.lock().unwrap().error(
                NO_VALID_ID,
                TwsError::UpdateTws.code(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    "  It does not support positions request.",
                )
                .as_ref(),
            );
            return;
        }

        let version = 1;

        let message_id: i32 = OutgoingMessageIds::ReqPositions as i32;
        let mut msg = "".to_string();
        msg.push_str(&make_field(&message_id));
        msg.push_str(&make_field(&version));

        self.send_request(msg.as_str())
    }

    //----------------------------------------------------------------------------------------------
    pub fn cancel_positions(&mut self) {
        //"""Cancels real-time position updates."""

        //// self.logRequest(current_fn_name()); vars())

        if !self.is_connected() {
            self.wrapper.lock().unwrap().deref_mut().error(
                NO_VALID_ID,
                TwsError::NotConnected.code(),
                TwsError::NotConnected.message(),
            );
            return;
        }

        if self.server_version() < MIN_SERVER_VER_POSITIONS {
            self.wrapper.lock().unwrap().error(
                NO_VALID_ID,
                TwsError::UpdateTws.code(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    "  It does not support positions request.",
                )
                .as_ref(),
            );
            return;
        }

        let version = 1;

        let message_id: i32 = OutgoingMessageIds::CancelPositions as i32;
        let mut msg = "".to_string();
        msg.push_str(&make_field(&message_id));
        msg.push_str(&make_field(&version));
        self.send_request(msg.as_str())
    }

    //----------------------------------------------------------------------------------------------
    pub fn req_positions_multi(&mut self, req_id: i32, account: &String, model_code: &String) {
        //        """Requests positions for account and/or model.
        //                Results are delivered via EWrapper.positionMulti() and
        //                EWrapper.positionMultiEnd() """

        //// self.logRequest(current_fn_name()); vars())

        if !self.is_connected() {
            self.wrapper.lock().unwrap().deref_mut().error(
                NO_VALID_ID,
                TwsError::NotConnected.code(),
                TwsError::NotConnected.message(),
            );
            return;
        }

        if self.server_version() < MIN_SERVER_VER_POSITIONS {
            self.wrapper.lock().unwrap().error(
                NO_VALID_ID,
                TwsError::UpdateTws.code(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    "  It does not support positions multi request.",
                )
                .as_ref(),
            );
            return;
        }

        let version = 1;
        let mut_req_id = req_id;
        let mut_account = account;
        let mut_model_code = model_code;
        let message_id: i32 = OutgoingMessageIds::ReqPositionsMulti as i32;
        let mut msg = "".to_string();
        msg.push_str(&make_field(&message_id));
        msg.push_str(&make_field(&version));
        msg.push_str(&make_field(&mut_req_id));
        msg.push_str(&make_field(mut_account));
        msg.push_str(&make_field(mut_model_code));

        self.send_request(msg.as_str())
    }

    //----------------------------------------------------------------------------------------------
    pub fn cancel_positions_multi(&mut self, req_id: i32) {
        //// self.logRequest(current_fn_name()); vars())

        if !self.is_connected() {
            self.wrapper.lock().unwrap().deref_mut().error(
                NO_VALID_ID,
                TwsError::NotConnected.code(),
                TwsError::NotConnected.message(),
            );
            return;
        }

        if self.server_version() < MIN_SERVER_VER_POSITIONS {
            self.wrapper.lock().unwrap().error(
                NO_VALID_ID,
                TwsError::UpdateTws.code(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    "  It does not support positions multi request.",
                )
                .as_ref(),
            );
            return;
        }

        let version = 1;
        let mut_req_id = req_id;
        let message_id: i32 = OutgoingMessageIds::CancelPositionsMulti as i32;
        let mut msg = "".to_string();
        msg.push_str(&make_field(&message_id));
        msg.push_str(&make_field(&version));
        msg.push_str(&make_field(&mut_req_id));

        self.send_request(msg.as_str())
    }

    //----------------------------------------------------------------------------------------------
    pub fn req_account_updates_multi(
        &mut self,
        req_id: i32,
        account: String,
        model_code: String,
        ledger_and_nlv: bool,
    ) {
        //"""Requests account updates for account and/or model."""

        //// self.logRequest(current_fn_name()); vars())

        if !self.is_connected() {
            self.wrapper.lock().unwrap().deref_mut().error(
                NO_VALID_ID,
                TwsError::NotConnected.code(),
                TwsError::NotConnected.message(),
            );
            return;
        }

        if self.server_version() < MIN_SERVER_VER_MODELS_SUPPORT {
            self.wrapper.lock().unwrap().error(
                NO_VALID_ID,
                TwsError::UpdateTws.code(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    "  It does not support account updates multi request.",
                )
                .as_ref(),
            );
            return;
        }

        let version = 1;
        let mut_req_id = req_id;
        let mut_account = account;
        let mut_model_code = model_code;
        let mut_ledger_and_nlv = ledger_and_nlv;

        let message_id: i32 = OutgoingMessageIds::ReqAccountUpdatesMulti as i32;
        let mut msg = "".to_string();
        msg.push_str(&make_field(&message_id));
        msg.push_str(&make_field(&version));
        msg.push_str(&make_field(&mut_req_id));
        msg.push_str(&make_field(&mut_account));
        msg.push_str(&make_field(&mut_model_code));
        msg.push_str(&make_field(&mut_ledger_and_nlv));

        self.send_request(msg.as_str())
    }

    //----------------------------------------------------------------------------------------------
    pub fn cancel_account_updates_multi(&mut self, req_id: i32) {
        //// self.logRequest(current_fn_name()); vars())

        if !self.is_connected() {
            self.wrapper.lock().unwrap().deref_mut().error(
                NO_VALID_ID,
                TwsError::NotConnected.code(),
                TwsError::NotConnected.message(),
            );
            return;
        }

        if self.server_version() < MIN_SERVER_VER_MODELS_SUPPORT {
            self.wrapper.lock().unwrap().error(
                NO_VALID_ID,
                TwsError::UpdateTws.code(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    "  It does not support account updates multi request.",
                )
                .as_ref(),
            );
            return;
        }

        let version = 1;
        let mut_req_id = req_id;
        let message_id: i32 = OutgoingMessageIds::CancelAccountUpdatesMulti as i32;
        let mut msg = "".to_string();
        msg.push_str(&make_field(&message_id));
        msg.push_str(&make_field(&version));
        msg.push_str(&make_field(&mut_req_id));

        self.send_request(msg.as_str())
    }

    //#########################################################################
    //################## Daily PnL
    //#########################################################################

    pub fn req_pn_l(&mut self, req_id: i32, account: &'static str, model_code: &'static str) {
        //// self.logRequest(current_fn_name()); vars())

        if !self.is_connected() {
            self.wrapper.lock().unwrap().deref_mut().error(
                NO_VALID_ID,
                TwsError::NotConnected.code(),
                TwsError::NotConnected.message(),
            );
            return;
        }

        if self.server_version() < MIN_SERVER_VER_PNL {
            self.wrapper.lock().unwrap().error(
                NO_VALID_ID,
                TwsError::UpdateTws.code(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    "  It does not support PnL request."
                )
                .as_ref(),
            );
            return;
        }

        let message_id: i32 = OutgoingMessageIds::ReqPnl as i32;
        let mut msg = "".to_string();
        msg.push_str(&make_field(&message_id));
        msg.push_str(&make_field(&req_id));
        msg.push_str(&make_field(&account));
        msg.push_str(&make_field(&model_code));

        self.send_request(msg.as_str())
    }

    //----------------------------------------------------------------------------------------------
    pub fn cancel_pnl(&mut self, req_id: i32) {
        //// self.logRequest(current_fn_name()); vars())

        if !self.is_connected() {
            self.wrapper.lock().unwrap().deref_mut().error(
                NO_VALID_ID,
                TwsError::NotConnected.code(),
                TwsError::NotConnected.message(),
            );
            return;
        }

        if self.server_version() < MIN_SERVER_VER_PNL {
            self.wrapper.lock().unwrap().error(
                NO_VALID_ID,
                TwsError::UpdateTws.code(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    "  It does not support PnL request."
                )
                .as_ref(),
            );
            return;
        }

        let message_id: i32 = OutgoingMessageIds::CancelPnl as i32;
        let mut msg = "".to_string();
        msg.push_str(&make_field(&message_id));
        msg.push_str(&make_field(&req_id));

        self.send_request(msg.as_str())
    }

    //----------------------------------------------------------------------------------------------
    pub fn req_pnl_single(
        &mut self,
        req_id: i32,
        account: &'static str,
        model_code: &'static str,
        con_id: i32,
    ) {
        //// self.logRequest(current_fn_name()); vars())

        if !self.is_connected() {
            self.wrapper.lock().unwrap().deref_mut().error(
                NO_VALID_ID,
                TwsError::NotConnected.code(),
                TwsError::NotConnected.message(),
            );
            return;
        }

        if self.server_version() < MIN_SERVER_VER_PNL {
            self.wrapper.lock().unwrap().error(
                NO_VALID_ID,
                TwsError::UpdateTws.code(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    "  It does not support PnL request."
                )
                .as_ref(),
            );
            return;
        }

        let message_id: i32 = OutgoingMessageIds::ReqPnlSingle as i32;
        let mut msg = "".to_string();
        msg.push_str(&make_field(&message_id));
        msg.push_str(&make_field(&req_id));
        msg.push_str(&make_field(&account));
        msg.push_str(&make_field(&model_code));
        msg.push_str(&make_field(&con_id));

        self.send_request(msg.as_str())
    }

    //----------------------------------------------------------------------------------------------
    pub fn cancel_pnl_single(&mut self, req_id: i32) {
        //// self.logRequest(current_fn_name()); vars())

        if !self.is_connected() {
            self.wrapper.lock().unwrap().deref_mut().error(
                NO_VALID_ID,
                TwsError::NotConnected.code(),
                TwsError::NotConnected.message(),
            );
            return;
        }

        if self.server_version() < MIN_SERVER_VER_PNL {
            self.wrapper.lock().unwrap().error(
                NO_VALID_ID,
                TwsError::UpdateTws.code(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    "  It does not support PnL request."
                )
                .as_ref(),
            );
            return;
        }

        let message_id: i32 = OutgoingMessageIds::CancelPnlSingle as i32;
        let mut msg = "".to_string();
        msg.push_str(&make_field(&message_id));
        msg.push_str(&make_field(&req_id));

        self.send_request(msg.as_str())
    }

    //#########################################################################
    //################## Executions
    //#########################################################################

    pub fn req_executions(&mut self, req_id: i32, exec_filter: &ExecutionFilter) {
        //    When this function is called, the execution reports that meet the
        //    filter criteria are downloaded to the core via the execDetails()
        //    function. To view executions beyond the past 24 hours, open the
        //    Trade Log in TWS and, while the Trade Log is displayed, request
        //    the executions again from the API.
        //
        //    req_id:i32 - The ID of the data request. Ensures that responses are
        //        matched to requests if several requests are in process.
        //    exec_filter:ExecutionFilter - This object contains attributes that
        //        describe the filter criteria used to determine which execution
        //        reports are returned.
        //
        //    NOTE: Time format must be 'yyyymmdd-hh:mm:ss' Eg: '20030702-14:55'

        // // self.logRequest(current_fn_name()); vars())

        if !self.is_connected() {
            self.wrapper.lock().unwrap().deref_mut().error(
                NO_VALID_ID,
                TwsError::NotConnected.code(),
                TwsError::NotConnected.message(),
            );
            return;
        }

        let version = 3;
        let message_id: i32 = OutgoingMessageIds::ReqExecutions as i32;
        let mut msg = "".to_string();
        msg.push_str(&make_field(&message_id));
        msg.push_str(&make_field(&version));
        if self.server_version() >= MIN_SERVER_VER_EXECUTION_DATA_CHAIN {
            msg.push_str(&make_field(&req_id));
        }
        msg.push_str(&make_field(&exec_filter.client_id));
        msg.push_str(&make_field(&exec_filter.acct_code));
        msg.push_str(&make_field(&exec_filter.time));
        msg.push_str(&make_field(&exec_filter.symbol));
        msg.push_str(&make_field(&exec_filter.sec_type));
        msg.push_str(&make_field(&exec_filter.exchange));
        msg.push_str(&make_field(&exec_filter.side));

        self.send_request(msg.as_str())
    }

    //#########################################################################
    //################## Contract Details
    //#########################################################################

    pub fn req_contract_details(&mut self, req_id: i32, contract: &Contract) {
        //    Call this function to download all details for a particular
        //    underlying. The contract details will be received via the contractDetails()
        //    function on the EWrapper.
        //
        //    req_id:i32 - The ID of the data request. Ensures that responses are
        //    make_fieldatched to requests if several requests are in process.
        //    contract:&Contract - The summary description of the contract being looked up.

        //// self.logRequest(current_fn_name()); vars())

        if !self.is_connected() {
            self.wrapper.lock().unwrap().deref_mut().error(
                NO_VALID_ID,
                TwsError::NotConnected.code(),
                TwsError::NotConnected.message(),
            );
            return;
        }

        if self.server_version() < MIN_SERVER_VER_SEC_ID_TYPE {
            if contract.sec_id_type != "" || contract.sec_id != "" {
                self.wrapper.lock().unwrap().error(
                    req_id,
                    TwsError::UpdateTws.code(),
                    format!(
                        "{}{}",
                        TwsError::UpdateTws.message(),
                        "  It does not support sec_id_type and secId parameters."
                    )
                    .as_ref(),
                );
                return;
            }
        }

        if self.server_version() < MIN_SERVER_VER_TRADING_CLASS {
            if contract.trading_class != "" {
                self.wrapper.lock().unwrap().error(
                    req_id,
                    TwsError::UpdateTws.code(),
                    format!(
                        "{}{}",
                        TwsError::UpdateTws.message(),
                        "  It does not support trading_class parameter in req_contract_details."
                    )
                    .as_ref(),
                );
                return;
            }
        }

        if self.server_version() < MIN_SERVER_VER_LINKING {
            if contract.primary_exchange != "" {
                self.wrapper.lock().unwrap().error(
                    req_id,
                    TwsError::UpdateTws.code(),
                    format!(
                        "{}{}",
                        TwsError::UpdateTws.message(),
                        "  It does not support primary_exchange parameter in req_contract_details."
                    )
                    .as_ref(),
                );
                return;
            }
        }

        let version = 8;

        let message_id: i32 = OutgoingMessageIds::ReqContractData as i32;
        let mut msg = "".to_string();
        msg.push_str(&make_field(&message_id));
        msg.push_str(&make_field(&version));

        if self.server_version() >= MIN_SERVER_VER_CONTRACT_DATA_CHAIN {
            msg.push_str(&make_field(&req_id));
        }

        // send contract fields
        msg.push_str(&make_field(&req_id));
        msg.push_str(&make_field(&contract.con_id)); // srv v37 and above
        msg.push_str(&make_field(&contract.symbol));

        msg.push_str(&make_field(&contract.sec_type));
        msg.push_str(&make_field(&contract.last_trade_date_or_contract_month));
        msg.push_str(&make_field(&contract.strike));
        msg.push_str(&make_field(&contract.right));
        msg.push_str(&make_field(&contract.multiplier)); // srv v15 and above

        if self.server_version() >= MIN_SERVER_VER_PRIMARYEXCH {
            msg.push_str(&make_field(&contract.exchange));
            msg.push_str(&make_field(&contract.primary_exchange));
        } else if self.server_version() >= MIN_SERVER_VER_LINKING {
            if contract.primary_exchange != ""
                && (contract.exchange == "BEST" || contract.exchange == "SMART")
            {
                msg.push_str(&make_field(&format!(
                    "{}:{}",
                    &contract.exchange, &contract.primary_exchange
                )));
            }
        } else {
            msg.push_str(&make_field(&contract.exchange));
        }

        msg.push_str(&make_field(&contract.currency));
        msg.push_str(&make_field(&contract.local_symbol));

        if self.server_version() >= MIN_SERVER_VER_TRADING_CLASS {
            msg.push_str(&make_field(&contract.trading_class));
            msg.push_str(&make_field(&contract.include_expired)); // srv v31 and above
        }

        if self.server_version() >= MIN_SERVER_VER_SEC_ID_TYPE {
            msg.push_str(&make_field(&contract.sec_id_type));
            msg.push_str(&make_field(&contract.sec_id));
        }

        self.send_request(msg.as_str())
    }

    //#########################################################################
    //################## Market Depth
    //#########################################################################

    pub fn req_mkt_depth_exchanges(&mut self) {
        //// self.logRequest(current_fn_name()); vars())

        if !self.is_connected() {
            self.wrapper.lock().unwrap().deref_mut().error(
                NO_VALID_ID,
                TwsError::NotConnected.code(),
                TwsError::NotConnected.message(),
            );
            return;
        }

        if self.server_version() < MIN_SERVER_VER_REQ_MKT_DEPTH_EXCHANGES {
            self.wrapper.lock().unwrap().error(
                NO_VALID_ID,
                TwsError::UpdateTws.code(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    "  It does not support market depth exchanges request."
                )
                .as_ref(),
            );
            return;
        }

        let message_id: i32 = OutgoingMessageIds::ReqMktDepthExchanges as i32;
        let mut msg = "".to_string();
        msg.push_str(&make_field(&message_id));

        self.send_request(msg.as_str())
    }

    //----------------------------------------------------------------------------------------------
    pub fn req_mkt_depth(
        &mut self,
        req_id: i32,
        contract: &Contract,
        num_rows: i32,
        is_smart_depth: bool,
        mkt_depth_options: Vec<TagValue>,
    ) {
        //    Call this function to request market depth for a specific
        //    contract. The market depth will be returned by the updateMktDepth() and
        //    updateMktDepthL2() events.
        //
        //    Requests the contract's market depth (order book). Note this request must be
        //    direct-routed to an exchange and not smart-routed. The number of simultaneous
        //    market depth requests allowed in an account is calculated based on a formula
        //    that looks at an accounts equity, commissions, and quote booster packs.
        //
        //    req_id:i32 - The ticker id. Must be a unique value. When the market
        //        depth data returns, it will be identified by this tag. This is
        //        also used when canceling the market depth
        //    contract:Contact - This structure contains a description of the contract
        //        for which market depth data is being requested.
        //    num_rows:i32 - Specifies the numRowsumber of market depth rows to display.
        //    is_smart_depth:bool - specifies SMART depth request
        //    mkt_depth_options:Vec<TagValue> - For internal use only. Use pub fnault value
        //        XYZ.

        //// self.logRequest(current_fn_name()); vars())

        if !self.is_connected() {
            self.wrapper.lock().unwrap().error(
                NO_VALID_ID,
                TwsError::NotConnected.code(),
                TwsError::NotConnected.message(),
            );
            return;
        }

        if self.server_version() < MIN_SERVER_VER_TRADING_CLASS {
            if &contract.trading_class != "" || *&contract.con_id > 0 {
                self.wrapper.lock().unwrap().error(
                    req_id,
                    TwsError::UpdateTws.code(),
                    format!(
                        "{}{}",
                        TwsError::UpdateTws.message(),
                        "  It does not support con_id and trading_class parameters in req_mkt_depth.")
                        .as_ref(),
                );
                return;
            }
        }

        if self.server_version() < MIN_SERVER_VER_SMART_DEPTH && is_smart_depth {
            self.wrapper.lock().unwrap().error(
                req_id,
                TwsError::UpdateTws.code(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    " It does not support SMART depth request."
                )
                .as_ref(),
            );
            return;
        }

        if self.server_version() < MIN_SERVER_VER_MKT_DEPTH_PRIM_EXCHANGE
            && contract.primary_exchange != ""
        {
            self.wrapper.lock().unwrap().error(
                req_id,
                TwsError::UpdateTws.code(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    " It does not support primary_exchange parameter in req_mkt_depth."
                )
                .as_ref(),
            );
            return;
        }

        let version = 5;

        // send req mkt depth msg

        let message_id: i32 = OutgoingMessageIds::ReqMktDepth as i32;
        let mut msg = "".to_string();
        msg.push_str(&make_field(&message_id));
        msg.push_str(&make_field(&version));
        msg.push_str(&make_field(&req_id));

        // send contract fields
        if self.server_version() >= MIN_SERVER_VER_TRADING_CLASS {
            msg.push_str(&make_field(&contract.con_id));
            msg.push_str(&make_field(&contract.symbol));
            msg.push_str(&make_field(&contract.sec_type));
            msg.push_str(&make_field(&contract.last_trade_date_or_contract_month));
            msg.push_str(&make_field(&contract.strike));
            msg.push_str(&make_field(&contract.right));
            msg.push_str(&make_field(&contract.multiplier)); // srv v15 and above
            msg.push_str(&make_field(&contract.exchange));
        }
        if self.server_version() >= MIN_SERVER_VER_MKT_DEPTH_PRIM_EXCHANGE {
            msg.push_str(&make_field(&contract.primary_exchange));
            msg.push_str(&make_field(&contract.currency));
            msg.push_str(&make_field(&contract.local_symbol));
        }
        if self.server_version() >= MIN_SERVER_VER_TRADING_CLASS {
            msg.push_str(&make_field(&contract.trading_class));
        }
        msg.push_str(&make_field(&num_rows)); // srv v19 and above

        if self.server_version() >= MIN_SERVER_VER_SMART_DEPTH {
            msg.push_str(&make_field(&is_smart_depth));
        }
        // send mkt_depth_options parameter
        if self.server_version() >= MIN_SERVER_VER_LINKING {
            // current doc says this part if for "internal use only" -> won't support it
            if mkt_depth_options.len() > 0 {
                self.wrapper.lock().unwrap().error(
                    req_id,
                    TwsError::Unsupported.code(),
                    TwsError::Unsupported.message(),
                );
                return;
            }
            let mkt_data_options_str = "";
            msg.push_str(&make_field(&mkt_data_options_str));
        }

        self.send_request(msg.as_str())
    }

    //----------------------------------------------------------------------------------------------
    pub fn cancel_mkt_depth(&mut self, req_id: i32, is_smart_depth: bool) {
        //    After calling this function, market depth data for the specified id
        //    will stop flowing.
        //
        //    req_id:i32 - The ID that was specified in the call to reqMktDepth().
        //    is_smart_depth:bool - specifies SMART depth request

        // // self.logRequest(current_fn_name()); vars())

        if !self.is_connected() {
            self.wrapper.lock().unwrap().error(
                NO_VALID_ID,
                TwsError::NotConnected.code(),
                TwsError::NotConnected.message(),
            );
            return;
        }

        if self.server_version() < MIN_SERVER_VER_SMART_DEPTH && is_smart_depth {
            self.wrapper.lock().unwrap().error(
                req_id,
                TwsError::UpdateTws.code(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    "  It does not support SMART depth cancel."
                )
                .as_ref(),
            );
            return;
        }

        let version = 1;

        let message_id: i32 = OutgoingMessageIds::CancelMktDepth as i32;
        let mut msg = "".to_string();
        msg.push_str(&make_field(&message_id));
        msg.push_str(&make_field(&version));
        msg.push_str(&make_field(&req_id));

        if self.server_version() >= MIN_SERVER_VER_SMART_DEPTH {
            msg.push_str(&make_field(&is_smart_depth));
        }

        self.send_request(msg.as_str())
    }

    //#########################################################################
    //################## News Bulletins
    //#########################################################################

    pub fn req_news_bulletins(&mut self, all_msgs: bool) {
        //    Call this function to start receiving news bulletins. Each bulletin
        //    will be returned by the updateNewsBulletin() event.
        //
        //    all_msgs:bool - If set to TRUE, returns all the existing bulletins for
        //    the currencyent day and any new ones. If set to FALSE, will only
        //    return new bulletins.

        //// self.logRequest(current_fn_name()); vars())

        if !self.is_connected() {
            self.wrapper.lock().unwrap().error(
                NO_VALID_ID,
                TwsError::NotConnected.code(),
                TwsError::NotConnected.message(),
            );
            return;
        }

        let version = 1;

        let message_id: i32 = OutgoingMessageIds::ReqNewsBulletins as i32;
        let mut msg = "".to_string();
        msg.push_str(&make_field(&message_id));
        msg.push_str(&make_field(&version));
        msg.push_str(&make_field(&all_msgs));

        self.send_request(msg.as_str())
    }

    //#########################################################################
    //################## Financial Advisors
    //#########################################################################

    pub fn req_managed_accts(&mut self) {
        //    Call this function to request the list of managed accounts. The list
        //    will be returned by the managedAccounts() function on the EWrapper.
        //
        //    Note:  This request can only be made when connected to a FA managed account.

        // self.logRequest(current_fn_name()); vars())

        if !self.is_connected() {
            self.wrapper.lock().unwrap().error(
                NO_VALID_ID,
                TwsError::NotConnected.code(),
                TwsError::NotConnected.message(),
            );
            return;
        }

        let version = 1;
        let message_id: i32 = OutgoingMessageIds::ReqManagedAccts as i32;
        let mut msg = "".to_string();
        msg.push_str(&make_field(&message_id));
        msg.push_str(&make_field(&version));
        self.send_request(msg.as_str())
    }

    //----------------------------------------------------------------------------------------------
    pub fn request_fa(&mut self, fa_data: FaDataType) {
        //    Call this function to request FA configuration information from TWS.
        //    The data returns in an XML string via a "receiveFA" ActiveX event.
        //
        //    fa_data:FaDataType - Specifies the type of Financial Advisor
        //        configuration data beingingg requested. Valid values include:
        //        1 = GROUPS
        //        2 = PROFILE
        //        3 = ACCOUNT ALIASES

        // self.logRequest(current_fn_name()); vars())

        if !self.is_connected() {
            self.wrapper.lock().unwrap().error(
                NO_VALID_ID,
                TwsError::NotConnected.code(),
                TwsError::NotConnected.message(),
            );
            return;
        }

        let version = 1;
        let message_id: i32 = OutgoingMessageIds::ReqFa as i32;
        let mut msg = "".to_string();
        msg.push_str(&make_field(&message_id));
        msg.push_str(&make_field(&version));
        msg.push_str(&make_field(&fa_data));

        self.send_request(msg.as_str());
    }

    //----------------------------------------------------------------------------------------------
    pub fn replace_fa(&mut self, fa_data: FaDataType, cxml: &'static str) {
        //    Call this function to modify FA configuration information from the
        //    API. Note that this can also be done manually in TWS itself.
        //
        //    fa_data:FaDataType - Specifies the type of Financial Advisor
        //        configuration data beingingg requested. Valid values include:
        //        1 = GROUPS
        //        2 = PROFILE
        //        3 = ACCOUNT ALIASES
        //    cxml: str - The XML string containing the new FA configuration
        //        information.

        // self.logRequest(current_fn_name()); vars())

        if !self.is_connected() {
            self.wrapper.lock().unwrap().error(
                NO_VALID_ID,
                TwsError::NotConnected.code(),
                TwsError::NotConnected.message(),
            );
            return;
        }

        let version = 1;
        let message_id: i32 = OutgoingMessageIds::ReplaceFa as i32;
        let mut msg = "".to_string();
        msg.push_str(&make_field(&message_id));

        msg.push_str(&make_field(&version));
        msg.push_str(&make_field(&fa_data));
        msg.push_str(&make_field(&cxml));

        self.send_request(msg.as_str())
    }

    //#########################################################################
    //################## Historical Data
    //#########################################################################

    pub fn req_historical_data(
        &mut self,
        req_id: i32,
        contract: &Contract,
        end_date_time: String,
        duration_str: String,
        bar_size_setting: String,
        what_to_show: String,
        use_rth: i32,
        format_date: i32,
        keep_up_to_date: bool,
        chart_options: Vec<TagValue>,
    ) {
        //    Requests contracts' historical data. When requesting historical data, a
        //    finishing time and date is required along with a duration string. The
        //    resulting bars will be returned in EWrapper.historicalData()
        //
        //    req_id:i32 - The id of the request. Must be a unique value. When the
        //        market data returns, it whatToShowill be identified by this tag. This is also
        //        used when canceling the market data.
        //    contract:&Contract - This object contains a description of the contract for which
        //        market data is being requested.
        //    end_date_time:&'static str - Defines a query end date and time at any point during the past 6 mos.
        //        Valid values include any date/time within the past six months in the format:
        //        yyyymmdd HH:mm:ss ttt
        //
        //        where "ttt" is the optional time zone.
        //    duration_str:&'static str - Set the query duration up to one week, using a time unit
        //        of seconds, days or weeks. Valid values include any integer followed by a space
        //        and then S (seconds)); D (days) or W (week). If no unit is specified, seconds is used.
        //    bar_size_setting:&'static str - Specifies the size of the bars that will be returned (within IB/TWS listimits).
        //        Valid values include:
        //        1 sec
        //        5 secs
        //        15 secs
        //        30 secs
        //        1 min
        //        2 mins
        //        3 mins
        //        5 mins
        //        15 mins
        //        30 mins
        //        1 hour
        //        1 day
        //    what_to_show:&'static str - Determines the nature of data beinging extracted. Valid values include:
        //
        //        TRADES
        //        MIDPOINT
        //        BID
        //        ASK
        //        BID_ASK
        //        HISTORICAL_VOLATILITY
        //        OPTION_IMPLIED_VOLATILITY
        //    use_rth:i32 - Determines whether to return all data available during the requested time span,
        //        or only data that falls within regular trading hours. Valid values include:
        //
        //        0 - all data is returned even where the market in question was outside of its
        //        regular trading hours.
        //        1 - only data within the regular trading hours is returned, even if the
        //        requested time span falls partially or completely outside of the RTH.
        //    format_date: int - Determines the date format applied to returned bars. validd values include:
        //
        //        1 - dates applying to bars returned in the format: yyyymmdd{space}{space}hh:mm:dd
        //        2 - dates are returned as a long integer specifying the number of seconds since
        //            1/1/1970 GMT.
        //    chart_options: - For internal use only. Use pub fnault value XYZ.

        // self.logRequest(current_fn_name()); vars())

        if !self.is_connected() {
            self.wrapper.lock().unwrap().error(
                NO_VALID_ID,
                TwsError::NotConnected.code(),
                TwsError::NotConnected.message(),
            );
            return;
        }

        if self.server_version() < MIN_SERVER_VER_TRADING_CLASS {
            if &contract.trading_class != "" || contract.con_id > 0 {
                self.wrapper.lock().unwrap().error(req_id, TwsError::UpdateTws.code(),
                                                   format!("{}{}", TwsError::UpdateTws.message(), "  It does not support con_id and trading_class parameters in req_historical_data.").as_ref());
                return;
            }
        }

        let version = 6;

        // send req mkt data msg
        let message_id: i32 = OutgoingMessageIds::ReqHistoricalData as i32;
        let mut msg = "".to_string();
        msg.push_str(&make_field(&message_id));

        if self.server_version() < MIN_SERVER_VER_SYNT_REALTIME_BARS {
            msg.push_str(&make_field(&version));
        }

        msg.push_str(&make_field(&req_id));

        // Send contract fields
        if self.server_version() >= MIN_SERVER_VER_TRADING_CLASS {
            msg.push_str(&make_field(&contract.con_id));
            msg.push_str(&make_field(&contract.symbol));
            msg.push_str(&make_field(&contract.sec_type));
            msg.push_str(&make_field(&contract.last_trade_date_or_contract_month));
            msg.push_str(&make_field(&contract.strike));
            msg.push_str(&make_field(&contract.right));
            msg.push_str(&make_field(&contract.multiplier));
            msg.push_str(&make_field(&contract.exchange));
            msg.push_str(&make_field(&contract.primary_exchange));
            msg.push_str(&make_field(&contract.currency));
            msg.push_str(&make_field(&contract.local_symbol));
        }
        if self.server_version() >= MIN_SERVER_VER_TRADING_CLASS {
            msg.push_str(&make_field(&contract.trading_class));
        }
        msg.push_str(&make_field(&contract.include_expired)); // srv v31 and above

        msg.push_str(&make_field(&end_date_time)); // srv v20 and above
        msg.push_str(&make_field(&bar_size_setting)); // srv v20 and above
        msg.push_str(&make_field(&duration_str));
        msg.push_str(&make_field(&use_rth));
        msg.push_str(&make_field(&what_to_show));
        msg.push_str(&make_field(&format_date)); // srv v16 and above

        // Send combo legs for BAG requests
        if contract.sec_type == "BAG" {
            msg.push_str(&make_field(&contract.combo_legs.len()));
            for combo_leg in &contract.combo_legs {
                msg.push_str(&make_field(&combo_leg.con_id));
                msg.push_str(&make_field(&combo_leg.ratio));
                msg.push_str(&make_field(&combo_leg.action));
                msg.push_str(&make_field(&combo_leg.exchange));
            }
        }
        if self.server_version() >= MIN_SERVER_VER_SYNT_REALTIME_BARS {
            msg.push_str(&make_field(&keep_up_to_date));
        }
        // Send chart_options parameter
        if self.server_version() >= MIN_SERVER_VER_LINKING {
            let chart_options_str = chart_options
                .iter()
                .map(|x| format!("{}={};", x.tag, x.value))
                .collect::<String>();
            msg.push_str(&make_field(&chart_options_str));
        }

        self.send_request(msg.as_str());
    }

    //----------------------------------------------------------------------------------------------
    pub fn cancel_historical_data(&mut self, req_id: i32) {
        /*Used if an internet disconnect has occurred or the results of a query
        are otherwise delayed and the application is no longer interested in receiving
        the data.

        req_id:i32 - The ticker ID. Must be a unique value*/

        // self.logRequest(current_fn_name()); vars())

        if !self.is_connected() {
            self.wrapper.lock().unwrap().error(
                NO_VALID_ID,
                TwsError::NotConnected.code(),
                TwsError::NotConnected.message(),
            );
            return;
        }

        let version = 1;

        let message_id: i32 = OutgoingMessageIds::CancelHistoricalData as i32;
        let mut msg = "".to_string();
        msg.push_str(&make_field(&message_id));

        msg.push_str(&make_field(&version));
        msg.push_str(&make_field(&req_id));

        self.send_request(msg.as_str());
    }

    // Note that formatData parameter affects intraday bars only
    // 1-day bars always return with date in YYYYMMDD format
    //----------------------------------------------------------------------------------------------
    pub fn req_head_time_stamp(
        &mut self,
        req_id: i32,
        contract: &Contract,
        what_to_show: &'static str,
        use_rth: i32,
        format_date: i32,
    ) {
        // self.logRequest(current_fn_name()); vars())
        if !self.is_connected() {
            self.wrapper.lock().unwrap().error(
                NO_VALID_ID,
                TwsError::NotConnected.code(),
                TwsError::NotConnected.message(),
            );
            return;
        }

        if self.server_version() < MIN_SERVER_VER_REQ_HEAD_TIMESTAMP {
            self.wrapper.lock().unwrap().error(
                req_id,
                TwsError::UpdateTws.code(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    "  It does not support head time stamp requests."
                )
                .as_ref(),
            );
            return;
        }

        let message_id: i32 = OutgoingMessageIds::ReqHeadTimestamp as i32;
        let mut msg = "".to_string();
        msg.push_str(&make_field(&message_id));

        msg.push_str(&make_field(&req_id));
        msg.push_str(&make_field(&contract.con_id));
        msg.push_str(&make_field(&contract.symbol));
        msg.push_str(&make_field(&contract.sec_type));
        msg.push_str(&make_field(&contract.last_trade_date_or_contract_month));
        msg.push_str(&make_field(&contract.strike));
        msg.push_str(&make_field(&contract.right));
        msg.push_str(&make_field(&contract.multiplier));
        msg.push_str(&make_field(&contract.exchange));
        msg.push_str(&make_field(&contract.primary_exchange));
        msg.push_str(&make_field(&contract.currency));
        msg.push_str(&make_field(&contract.local_symbol));
        msg.push_str(&make_field(&contract.trading_class));
        msg.push_str(&make_field(&contract.include_expired));
        msg.push_str(&make_field(&use_rth));
        msg.push_str(&make_field(&what_to_show));
        msg.push_str(&make_field(&format_date));

        self.send_request(msg.as_str());
    }

    //----------------------------------------------------------------------------------------------
    pub fn cancel_head_time_stamp(&mut self, req_id: i32) {
        // self.logRequest(current_fn_name()); vars())

        if !self.is_connected() {
            self.wrapper.lock().unwrap().error(
                NO_VALID_ID,
                TwsError::NotConnected.code(),
                TwsError::NotConnected.message(),
            );
            return;
        }

        if self.server_version() < MIN_SERVER_VER_CANCEL_HEADTIMESTAMP {
            self.wrapper.lock().unwrap().error(
                req_id,
                TwsError::UpdateTws.code(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    "  It does not support head time stamp requests."
                )
                .as_ref(),
            );
            return;
        }

        let message_id: i32 = OutgoingMessageIds::CancelHeadTimestamp as i32;
        let mut msg = "".to_string();
        msg.push_str(&make_field(&message_id));

        msg.push_str(&make_field(&req_id));

        self.send_request(msg.as_str());
    }

    //----------------------------------------------------------------------------------------------
    pub fn req_histogram_data(
        &mut self,
        ticker_id: i32,
        contract: Contract,
        use_rth: bool,
        time_period: &'static str,
    ) {
        // self.logRequest(current_fn_name()); vars())

        if !self.is_connected() {
            self.wrapper.lock().unwrap().error(
                NO_VALID_ID,
                TwsError::NotConnected.code(),
                TwsError::NotConnected.message(),
            );
            return;
        }

        if self.server_version() < MIN_SERVER_VER_REQ_HISTOGRAM {
            self.wrapper.lock().unwrap().error(
                NO_VALID_ID,
                TwsError::UpdateTws.code(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    "  It does not support histogram requests.."
                )
                .as_ref(),
            );
            return;
        }

        let message_id: i32 = OutgoingMessageIds::ReqHistogramData as i32;
        let mut msg = "".to_string();
        msg.push_str(&make_field(&message_id));

        msg.push_str(&make_field(&ticker_id));
        msg.push_str(&make_field(&contract.con_id));
        msg.push_str(&make_field(&contract.symbol));
        msg.push_str(&make_field(&contract.sec_type));
        msg.push_str(&make_field(&contract.last_trade_date_or_contract_month));
        msg.push_str(&make_field(&contract.strike));
        msg.push_str(&make_field(&contract.right));
        msg.push_str(&make_field(&contract.multiplier));
        msg.push_str(&make_field(&contract.exchange));
        msg.push_str(&make_field(&contract.primary_exchange));
        msg.push_str(&make_field(&contract.currency));
        msg.push_str(&make_field(&contract.local_symbol));
        msg.push_str(&make_field(&contract.trading_class));
        msg.push_str(&make_field(&contract.include_expired));
        msg.push_str(&make_field(&use_rth));
        msg.push_str(&make_field(&time_period));

        self.send_request(msg.as_str());
    }

    //----------------------------------------------------------------------------------------------
    pub fn cancel_histogram_data(&mut self, ticker_id: i32) {
        // self.logRequest(current_fn_name()); vars())

        if !self.is_connected() {
            self.wrapper.lock().unwrap().error(
                NO_VALID_ID,
                TwsError::NotConnected.code(),
                TwsError::NotConnected.message(),
            );
            return;
        }

        if self.server_version() < MIN_SERVER_VER_REQ_HISTOGRAM {
            self.wrapper.lock().unwrap().error(
                NO_VALID_ID,
                TwsError::UpdateTws.code(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    "  It does not support histogram requests.."
                )
                .as_ref(),
            );
            return;
        }

        let message_id: i32 = OutgoingMessageIds::CancelHistogramData as i32;
        let mut msg = "".to_string();
        msg.push_str(&make_field(&message_id));
        msg.push_str(&make_field(&ticker_id));

        self.send_request(msg.as_str())
    }

    //----------------------------------------------------------------------------------------------
    pub fn req_historical_ticks(
        &mut self,
        req_id: i32,
        contract: &Contract,
        start_date_time: &'static str,
        end_date_time: &'static str,
        number_of_ticks: i32,
        what_to_show: &'static str,
        use_rth: i32,
        ignore_size: bool,
        misc_options: Vec<TagValue>,
    ) {
        // self.logRequest(current_fn_name()); vars())

        if !self.is_connected() {
            self.wrapper.lock().unwrap().error(
                NO_VALID_ID,
                TwsError::NotConnected.code(),
                TwsError::NotConnected.message(),
            );
            return;
        }

        if self.server_version() < MIN_SERVER_VER_HISTORICAL_TICKS {
            self.wrapper.lock().unwrap().error(
                NO_VALID_ID,
                TwsError::UpdateTws.code(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    "  It does not support historical ticks requests.."
                )
                .as_ref(),
            );
            return;
        }

        let message_id: i32 = OutgoingMessageIds::ReqHistoricalTicks as i32;
        let mut msg = "".to_string();
        msg.push_str(&make_field(&message_id));

        msg.push_str(&make_field(&req_id));
        msg.push_str(&make_field(&contract.con_id));
        msg.push_str(&make_field(&contract.symbol));
        msg.push_str(&make_field(&contract.sec_type));
        msg.push_str(&make_field(&contract.last_trade_date_or_contract_month));
        msg.push_str(&make_field(&contract.strike));
        msg.push_str(&make_field(&contract.right));
        msg.push_str(&make_field(&contract.multiplier));
        msg.push_str(&make_field(&contract.exchange));
        msg.push_str(&make_field(&contract.primary_exchange));
        msg.push_str(&make_field(&contract.currency));
        msg.push_str(&make_field(&contract.local_symbol));
        msg.push_str(&make_field(&contract.trading_class));
        msg.push_str(&make_field(&contract.include_expired));
        msg.push_str(&make_field(&start_date_time));
        msg.push_str(&make_field(&end_date_time));
        msg.push_str(&make_field(&number_of_ticks));
        msg.push_str(&make_field(&what_to_show));
        msg.push_str(&make_field(&use_rth));
        msg.push_str(&make_field(&ignore_size));

        let misc_options_string = misc_options
            .iter()
            .map(|x| format!("{}={};", x.tag, x.value))
            .collect::<String>();

        msg.push_str(&make_field(&misc_options_string));

        self.send_request(msg.as_str())
    }

    //#########################################################################
    //################## Market Scanners
    //#########################################################################

    pub fn req_scanner_parameters(&mut self) {
        /*Requests an XML string that describes all possible scanner queries*/

        // self.logRequest(current_fn_name()); vars())

        if !self.is_connected() {
            self.wrapper.lock().unwrap().error(
                NO_VALID_ID,
                TwsError::NotConnected.code(),
                TwsError::NotConnected.message(),
            );
            return;
        }

        let version = 1;
        let message_id: i32 = OutgoingMessageIds::ReqScannerParameters as i32;
        let mut msg = "".to_string();
        msg.push_str(&make_field(&message_id));
        msg.push_str(&make_field(&version));

        self.send_request(msg.as_str())
    }

    //----------------------------------------------------------------------------------------------
    pub fn req_scanner_subscription(
        &mut self,
        req_id: i32,
        subscription: ScannerSubscription,
        scanner_subscription_options: Vec<TagValue>,
        scanner_subscription_filter_options: Vec<TagValue>,
    ) {
        /*reqId:i32 - The ticker ID. Must be a unique value.
        scannerSubscription:ScannerSubscription - This structure contains
            possible parameters used to filter results.
        scanner_subscription_options: - For internal use only.
            Use pub fnault value XYZ*/

        // self.logRequest(current_fn_name()); vars())

        if !self.is_connected() {
            self.wrapper.lock().unwrap().error(
                NO_VALID_ID,
                TwsError::NotConnected.code(),
                TwsError::NotConnected.message(),
            );
            return;
        }

        if self.server_version() < MIN_SERVER_VER_SCANNER_GENERIC_OPTS
            && scanner_subscription_filter_options.len() > 0
        {
            self.wrapper.lock().unwrap().error(
                NO_VALID_ID,
                TwsError::UpdateTws.code(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    " It does not support API scanner subscription generic filter options"
                )
                .as_ref(),
            );
            return;
        }

        let version = 4;

        let message_id: i32 = OutgoingMessageIds::ReqScannerSubscription as i32;
        let mut msg = "".to_string();
        msg.push_str(&make_field(&message_id));

        if self.server_version() < MIN_SERVER_VER_SCANNER_GENERIC_OPTS {
            msg.push_str(&make_field(&version));
        }
        msg.push_str(&make_field(&req_id));
        msg.push_str(&make_field_handle_empty(&subscription.number_of_rows));
        msg.push_str(&make_field(&subscription.instrument));
        msg.push_str(&make_field(&subscription.location_code));
        msg.push_str(&make_field(&subscription.scan_code));
        msg.push_str(&make_field_handle_empty(&subscription.above_price));
        msg.push_str(&make_field_handle_empty(&subscription.below_price));
        msg.push_str(&make_field_handle_empty(&subscription.above_volume));
        msg.push_str(&make_field_handle_empty(&subscription.market_cap_above));
        msg.push_str(&make_field_handle_empty(&subscription.market_cap_below));
        msg.push_str(&make_field(&subscription.moody_rating_above));
        msg.push_str(&make_field(&subscription.moody_rating_below));
        msg.push_str(&make_field(&subscription.sp_rating_above));
        msg.push_str(&make_field(&subscription.sp_rating_below));
        msg.push_str(&make_field(&subscription.maturity_date_above));
        msg.push_str(&make_field(&subscription.maturity_date_below));
        msg.push_str(&make_field_handle_empty(&subscription.coupon_rate_above));
        msg.push_str(&make_field_handle_empty(&subscription.coupon_rate_below));
        msg.push_str(&make_field(&subscription.exclude_convertible));
        msg.push_str(&make_field_handle_empty(
            &subscription.average_option_volume_above,
        )); // srv v25 and above
        msg.push_str(&make_field(&subscription.scanner_setting_pairs)); // srv v25 and above
        msg.push_str(&make_field(&subscription.stock_type_filter)); // srv v27 and above

        // Send scanner_subscription_filter_options parameter
        if self.server_version() >= MIN_SERVER_VER_SCANNER_GENERIC_OPTS {
            let scanner_subscription_filter = scanner_subscription_filter_options
                .iter()
                .map(|x| format!("{}={};", x.tag, x.value))
                .collect::<String>();

            msg.push_str(&make_field(&scanner_subscription_filter));
        }
        // Send scanner_subscription_options parameter
        if self.server_version() >= MIN_SERVER_VER_LINKING {
            let scanner_subscription_options = scanner_subscription_options
                .iter()
                .map(|x| format!("{}={};", x.tag, x.value))
                .collect::<String>();
            msg.push_str(&make_field(&scanner_subscription_options));
        }

        self.send_request(msg.as_str())
    }

    //----------------------------------------------------------------------------------------------
    pub fn cancel_scanner_subscription(&mut self, req_id: i32) {
        /*reqId:i32 - The ticker ID. Must be a unique value*/

        // self.logRequest(current_fn_name()); vars())

        if !self.is_connected() {
            self.wrapper.lock().unwrap().error(
                NO_VALID_ID,
                TwsError::NotConnected.code(),
                TwsError::NotConnected.message(),
            );
            return;
        }

        let version = 1;

        let message_id: i32 = OutgoingMessageIds::CancelScannerSubscription as i32;
        let mut msg = "".to_string();
        msg.push_str(&make_field(&message_id));

        msg.push_str(&make_field(&version));
        msg.push_str(&make_field(&req_id));

        self.send_request(msg.as_str())
    }

    //#########################################################################
    //################## Real Time Bars
    //#########################################################################

    pub fn req_real_time_bars(
        &mut self,
        req_id: i32,
        contract: &Contract,
        bar_size: i32,
        what_to_show: &'static str,
        use_rth: bool,
        real_time_bars_options: Vec<TagValue>,
    ) {
        /*Call the req_real_time_bars() function to start receiving real time bar
        results through the realtimeBar() EWrapper function.

        reqId:i32 - The Id for the request. Must be a unique value. When the
            data is received, it will be identified by this Id. This is also
            used when canceling the request.
        contract:&Contract - This object contains a description of the contract
            for which real time bars are being requested
        bar_size:i32 - Currently only 5 second bars are supported, if any other
            value is used, an exception will be thrown.
        what_to_show:&'static str - Determines the nature of the data extracted. Valid
            values include:
            TRADES
            BID
            ASK
            MIDPOINT
        use_rth:bool - Regular Trading Hours only. Valid values include:
            0 = all data available during the time span requested is returned,
                including time intervals when the market in question was
                outside of regular trading hours.
            1 = only data within the regular trading hours for the product
                requested is returned, even if the time time span falls
                partially or completely outside.
        realTimeBarOptions: - For internal use only. Use pub fnault value XYZ*/

        // self.logRequest(current_fn_name()); vars())

        if !self.is_connected() {
            self.wrapper.lock().unwrap().error(
                NO_VALID_ID,
                TwsError::NotConnected.code(),
                TwsError::NotConnected.message(),
            );
            return;
        }

        if self.server_version() < MIN_SERVER_VER_TRADING_CLASS {
            if !contract.trading_class.is_empty() {
                self.wrapper.lock().unwrap().error(req_id, TwsError::UpdateTws.code(),
                                                   format!("{}{}", TwsError::UpdateTws.message(), "  It does not support con_id and trading_class parameter in req_real_time_bars.").as_ref());
                return;
            }
        }

        let version = 3;

        let message_id: i32 = OutgoingMessageIds::ReqRealTimeBars as i32;
        let mut msg = "".to_string();
        msg.push_str(&make_field(&message_id));

        msg.push_str(&make_field(&version));
        msg.push_str(&make_field(&req_id));

        // Send contract fields
        if self.server_version() >= MIN_SERVER_VER_TRADING_CLASS {
            msg.push_str(&make_field(&contract.con_id));
        }
        msg.push_str(&make_field(&contract.symbol));
        msg.push_str(&make_field(&contract.sec_type));
        msg.push_str(&make_field(&contract.last_trade_date_or_contract_month));
        msg.push_str(&make_field(&contract.strike));
        msg.push_str(&make_field(&contract.right));
        msg.push_str(&make_field(&contract.multiplier));
        msg.push_str(&make_field(&contract.exchange));
        msg.push_str(&make_field(&contract.primary_exchange));
        msg.push_str(&make_field(&contract.currency));
        msg.push_str(&make_field(&contract.local_symbol));
        if self.server_version() >= MIN_SERVER_VER_TRADING_CLASS {
            msg.push_str(&make_field(&contract.trading_class));
        }
        msg.push_str(&make_field(&bar_size));
        msg.push_str(&make_field(&what_to_show));
        msg.push_str(&make_field(&use_rth));

        // Send real_time_bars_options parameter
        if self.server_version() >= MIN_SERVER_VER_LINKING {
            let real_time_bars_options_str = real_time_bars_options
                .iter()
                .map(|x| format!("{}={};", x.tag, x.value))
                .collect::<String>();

            msg.push_str(&make_field(&real_time_bars_options_str));
        }

        self.send_request(msg.as_str())
    }

    //----------------------------------------------------------------------------------------------
    pub fn cancel_real_time_bars(&mut self, req_id: i32) {
        /*Call the cancel_real_time_bars() function to stop receiving real time bar results.

        reqId:i32 - The Id that was specified in the call to req_real_time_bars(). */

        // self.logRequest(current_fn_name()); vars())

        if !self.is_connected() {
            self.wrapper.lock().unwrap().error(
                NO_VALID_ID,
                TwsError::NotConnected.code(),
                TwsError::NotConnected.message(),
            );
            return;
        }

        let version = 1;

        // Send req mkt data msg
        let message_id: i32 = OutgoingMessageIds::CancelRealTimeBars as i32;
        let mut msg = "".to_string();
        msg.push_str(&make_field(&message_id));

        msg.push_str(&make_field(&version));
        msg.push_str(&make_field(&req_id));

        self.send_request(msg.as_str())
    }

    //#########################################################################
    //################## Fundamental Data
    //#########################################################################

    pub fn req_fundamental_data(
        &mut self,
        req_id: i32,
        contract: &Contract,
        report_type: &'static str,
        fundamental_data_options: Vec<TagValue>,
    ) {
        /*Call this function to receive fundamental data for
        stocks. The appropriate market data subscription must be set up in
        Account Management before you can receive this data.
        Fundamental data will be returned at EWrapper.fundamentalData().

        req_fundamental_data() can handle conid specified in the Contract object,
        but not trading_class or multiplier. This is because req_fundamental_data()
        is used only for stocks and stocks do not have a multiplier and
        trading class.

        reqId:ticker_id - The ID of the data request. Ensures that responses are
             matched to requests if several requests are in process.
        contract:&Contract - This structure contains a description of the
            contract for which fundamental data is being requested.
        report_type:&'static str - One of the following XML reports:
            ReportSnapshot (company overview)
            ReportsFinSummary (financial summary)
            ReportRatios (financial ratios)
            ReportsFinStatements (financial statements)
            RESC (analyst estimates)
            CalendarReport (company calendar)*/

        // self.logRequest(current_fn_name()); vars())

        if !self.is_connected() {
            self.wrapper.lock().unwrap().error(
                NO_VALID_ID,
                TwsError::NotConnected.code(),
                TwsError::NotConnected.message(),
            );
            return;
        }

        let version = 2;

        if self.server_version() < MIN_SERVER_VER_FUNDAMENTAL_DATA {
            self.wrapper.lock().unwrap().error(
                NO_VALID_ID,
                TwsError::UpdateTws.code(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    "  It does not support fundamental data request."
                )
                .as_ref(),
            );
            return;
        }

        if self.server_version() < MIN_SERVER_VER_TRADING_CLASS {
            self.wrapper.lock().unwrap().error(
                NO_VALID_ID,
                TwsError::UpdateTws.code(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    "  It does not support con_id parameter in req_fundamental_data."
                )
                .as_ref(),
            );
            return;
        }

        let message_id: i32 = OutgoingMessageIds::ReqFundamentalData as i32;
        let mut msg = "".to_string();
        msg.push_str(&make_field(&message_id));

        msg.push_str(&make_field(&version));
        msg.push_str(&make_field(&req_id));

        // Send contract fields
        if self.server_version() >= MIN_SERVER_VER_TRADING_CLASS {
            msg.push_str(&make_field(&contract.con_id));
        }
        msg.push_str(&make_field(&contract.symbol));
        msg.push_str(&make_field(&contract.sec_type));
        msg.push_str(&make_field(&contract.exchange));
        msg.push_str(&make_field(&contract.primary_exchange));
        msg.push_str(&make_field(&contract.currency));
        msg.push_str(&make_field(&contract.local_symbol));
        msg.push_str(&make_field(&report_type));

        if self.server_version() >= MIN_SERVER_VER_LINKING {
            let tags_value_count = fundamental_data_options.len();
            let fund_data_opt_str = fundamental_data_options
                .iter()
                .map(|x| format!("{}={};", x.tag, x.value))
                .collect::<String>();

            msg.push_str(&make_field(&tags_value_count));
            msg.push_str(&make_field(&fund_data_opt_str));
        }

        self.send_request(msg.as_str())
    }

    //----------------------------------------------------------------------------------------------
    pub fn cancel_fundamental_data(&mut self, req_id: i32) {
        /*Call this function to stop receiving fundamental data.

        reqId:i32 - The ID of the data request*/

        // self.logRequest(current_fn_name()); vars())

        if !self.is_connected() {
            self.wrapper.lock().unwrap().error(
                NO_VALID_ID,
                TwsError::NotConnected.code(),
                TwsError::NotConnected.message(),
            );
            return;
        }

        if self.server_version() < MIN_SERVER_VER_FUNDAMENTAL_DATA {
            self.wrapper.lock().unwrap().error(
                NO_VALID_ID,
                TwsError::UpdateTws.code(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    "  It does not support fundamental data request."
                )
                .as_ref(),
            );
            return;
        }

        let version = 1;

        let message_id: i32 = OutgoingMessageIds::CancelFundamentalData as i32;
        let mut msg = "".to_string();
        msg.push_str(&make_field(&message_id));

        msg.push_str(&make_field(&version));
        msg.push_str(&make_field(&req_id));

        self.send_request(msg.as_str())
    }

    //########################################################################
    //################## News
    //#########################################################################

    pub fn req_news_providers(&mut self) {
        // self.logRequest(current_fn_name()); vars())

        if !self.is_connected() {
            self.wrapper.lock().unwrap().error(
                NO_VALID_ID,
                TwsError::NotConnected.code(),
                TwsError::NotConnected.message(),
            );
            return;
        }

        if self.server_version() < MIN_SERVER_VER_REQ_NEWS_PROVIDERS {
            self.wrapper.lock().unwrap().error(
                NO_VALID_ID,
                TwsError::UpdateTws.code(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    "  It does not support news providers request."
                )
                .as_ref(),
            );
            return;
        }

        let message_id: i32 = OutgoingMessageIds::ReqNewsProviders as i32;
        let mut msg = "".to_string();
        msg.push_str(&make_field(&message_id));

        self.send_request(msg.as_str())
    }

    //----------------------------------------------------------------------------------------------
    pub fn req_news_article(
        &mut self,
        req_id: i32,
        provider_code: &'static str,
        article_id: &'static str,
        news_article_options: Vec<TagValue>,
    ) {
        // self.logRequest(current_fn_name()); vars())

        if !self.is_connected() {
            self.wrapper.lock().unwrap().error(
                NO_VALID_ID,
                TwsError::NotConnected.code(),
                TwsError::NotConnected.message(),
            );
            return;
        }

        if self.server_version() < MIN_SERVER_VER_REQ_NEWS_ARTICLE {
            self.wrapper.lock().unwrap().error(
                NO_VALID_ID,
                TwsError::UpdateTws.code(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    "  It does not support news article request."
                )
                .as_ref(),
            );
            return;
        }

        let message_id: i32 = OutgoingMessageIds::ReqNewsArticle as i32;
        let mut msg = "".to_string();
        msg.push_str(&make_field(&message_id));

        msg.push_str(&make_field(&req_id));
        msg.push_str(&make_field(&provider_code));
        msg.push_str(&make_field(&article_id));

        // Send news_article_options parameter
        if self.server_version() >= MIN_SERVER_VER_NEWS_QUERY_ORIGINS {
            let news_article_options_str = news_article_options
                .iter()
                .map(|x| format!("{}={};", x.tag, x.value))
                .collect::<String>();
            msg.push_str(&make_field(&news_article_options_str));
        }

        self.send_request(msg.as_str())
    }

    //----------------------------------------------------------------------------------------------
    pub fn req_historical_news(
        &mut self,
        req_id: i32,
        con_id: i32,
        provider_codes: &'static str,
        start_date_time: &'static str,
        end_date_time: &'static str,
        total_results: i32,
        historical_news_options: Vec<TagValue>,
    ) {
        // self.logRequest(current_fn_name()); vars())

        if !self.is_connected() {
            self.wrapper.lock().unwrap().error(
                NO_VALID_ID,
                TwsError::NotConnected.code(),
                TwsError::NotConnected.message(),
            );
            return;
        }

        if self.server_version() < MIN_SERVER_VER_REQ_HISTORICAL_NEWS {
            self.wrapper.lock().unwrap().error(
                NO_VALID_ID,
                TwsError::UpdateTws.code(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    "  It does not support historical news request."
                )
                .as_ref(),
            );
            return;
        }

        let message_id: i32 = OutgoingMessageIds::ReqHistoricalNews as i32;
        let mut msg = "".to_string();
        msg.push_str(&make_field(&message_id));

        msg.push_str(&make_field(&req_id));
        msg.push_str(&make_field(&con_id));
        msg.push_str(&make_field(&provider_codes));
        msg.push_str(&make_field(&start_date_time));
        msg.push_str(&make_field(&end_date_time));
        msg.push_str(&make_field(&total_results));

        // Send historical_news_options parameter
        if self.server_version() >= MIN_SERVER_VER_NEWS_QUERY_ORIGINS {
            let historical_news_options_str = historical_news_options
                .iter()
                .map(|x| format!("{}={};", x.tag, x.value))
                .collect::<String>();
            msg.push_str(&make_field(&historical_news_options_str));
        }

        self.send_request(msg.as_str())
    }

    //#########################################################################
    //################## Display Groups
    //#########################################################################

    pub fn query_display_groups(&mut self, req_id: i32) {
        /*API requests used to integrate with TWS color-grouped windows (display groups).
        TWS color-grouped windows are identified by an integer number. Currently that number ranges from 1 to 7 and are mapped to specific colors, as indicated in TWS.

        reqId:i32 - The unique number that will be associated with the
            response */

        // self.logRequest(current_fn_name()); vars())

        if !self.is_connected() {
            self.wrapper.lock().unwrap().error(
                NO_VALID_ID,
                TwsError::NotConnected.code(),
                TwsError::NotConnected.message(),
            );
            return;
        }

        if self.server_version() < MIN_SERVER_VER_LINKING {
            self.wrapper.lock().unwrap().error(
                NO_VALID_ID,
                TwsError::UpdateTws.code(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    "  It does not support query_display_groups request."
                )
                .as_ref(),
            );
            return;
        }

        let version = 1;

        let message_id: i32 = OutgoingMessageIds::QueryDisplayGroups as i32;
        let mut msg = "".to_string();
        msg.push_str(&make_field(&message_id));
        msg.push_str(&make_field(&version));
        msg.push_str(&make_field(&req_id));

        self.send_request(msg.as_str())
    }

    //----------------------------------------------------------------------------------------------
    pub fn subscribe_to_group_events(&mut self, req_id: i32, group_id: i32) {
        /*reqId:i32 - The unique number associated with the notification.
        group_id:i32 - The ID of the group, currently it is a number from 1 to 7.
            This is the display group subscription request sent by the API to TWS*/

        // self.logRequest(current_fn_name()); vars())

        if !self.is_connected() {
            self.wrapper.lock().unwrap().error(
                NO_VALID_ID,
                TwsError::NotConnected.code(),
                TwsError::NotConnected.message(),
            );
            return;
        }

        if self.server_version() < MIN_SERVER_VER_LINKING {
            self.wrapper.lock().unwrap().error(
                NO_VALID_ID,
                TwsError::UpdateTws.code(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    "  It does not support subscribe_to_group_events request."
                )
                .as_ref(),
            );
            return;
        }

        let version = 1;

        let message_id: i32 = OutgoingMessageIds::SubscribeToGroupEvents as i32;
        let mut msg = "".to_string();
        msg.push_str(&make_field(&message_id));

        msg.push_str(&make_field(&version));
        msg.push_str(&make_field(&req_id));
        msg.push_str(&make_field(&group_id));

        self.send_request(msg.as_str())
    }

    //----------------------------------------------------------------------------------------------
    pub fn update_display_group(&mut self, req_id: i32, contract_info: &'static str) {
        /*reqId:i32 - The requestId specified in subscribe_to_group_events().
        contract_info:&'static str - The encoded value that uniquely represents the
            contract in IB. Possible values include:

            none = empty selection
            contractID@exchange - any non-combination contract.
                Examples: 8314@SMART for IBM SMART; 8314@ARCA for IBM @ARCA.
            combo = if any combo is selected*/

        // self.logRequest(current_fn_name()); vars())

        if !self.is_connected() {
            self.wrapper.lock().unwrap().error(
                NO_VALID_ID,
                TwsError::NotConnected.code(),
                TwsError::NotConnected.message(),
            );
            return;
        }

        if self.server_version() < MIN_SERVER_VER_LINKING {
            self.wrapper.lock().unwrap().error(
                NO_VALID_ID,
                TwsError::UpdateTws.code(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    "  It does not support update_display_group request."
                )
                .as_ref(),
            );
            return;
        }

        let version = 1;

        let message_id: i32 = OutgoingMessageIds::UpdateDisplayGroup as i32;
        let mut msg = "".to_string();
        msg.push_str(&make_field(&message_id));

        msg.push_str(&make_field(&version));
        msg.push_str(&make_field(&req_id));
        msg.push_str(&make_field(&contract_info));

        self.send_request(msg.as_str())
    }

    //----------------------------------------------------------------------------------------------
    pub fn unsubscribe_from_group_events(&mut self, req_id: i32) {
        /*reqId:i32 - The requestId specified in subscribe_to_group_events()*/

        // self.logRequest(current_fn_name()); vars())

        if !self.is_connected() {
            self.wrapper.lock().unwrap().error(
                NO_VALID_ID,
                TwsError::NotConnected.code(),
                TwsError::NotConnected.message(),
            );
            return;
        }

        if self.server_version() < MIN_SERVER_VER_LINKING {
            self.wrapper.lock().unwrap().error(
                NO_VALID_ID,
                TwsError::UpdateTws.code(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    "  It does not support unsubscribe_from_group_events request."
                )
                .as_str(),
            );
            return;
        }

        let version = 1;

        let message_id: i32 = OutgoingMessageIds::UnsubscribeFromGroupEvents as i32;
        let mut msg = "".to_string();
        msg.push_str(&make_field(&message_id));

        msg.push_str(&make_field(&version));
        msg.push_str(&make_field(&req_id));

        self.send_request(msg.as_str())
    }

    //----------------------------------------------------------------------------------------------
    pub fn verify_request(&mut self, api_name: &'static str, api_version: &'static str) {
        /*For IB's internal purpose. Allows to provide means of verification
        between the TWS and third party programs*/

        // self.logRequest(current_fn_name()); vars())

        if !self.is_connected() {
            self.wrapper.lock().unwrap().error(
                NO_VALID_ID,
                TwsError::NotConnected.code(),
                TwsError::NotConnected.message(),
            );
            return;
        }

        if self.server_version() < MIN_SERVER_VER_LINKING {
            self.wrapper.lock().unwrap().error(
                NO_VALID_ID,
                TwsError::UpdateTws.code(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    "  It does not support verification request."
                )
                .as_ref(),
            );
            return;
        }

        if !self.extra_auth {
            self.wrapper.lock().unwrap().error(
                NO_VALID_ID,
                TwsError::BadMessage.code(),
                format!("{}{}", TwsError::BadMessage.message(),
                        "  Intent to authenticate needs to be expressed during initial connect request.")
                    .as_ref(),
            );
            return;
        }

        let version = 1;

        let message_id: i32 = OutgoingMessageIds::VerifyRequest as i32;
        let mut msg = "".to_string();
        msg.push_str(&make_field(&message_id));

        msg.push_str(&make_field(&version));
        msg.push_str(&make_field(&api_name));
        msg.push_str(&make_field(&api_version));

        self.send_request(msg.as_str())
    }

    //----------------------------------------------------------------------------------------------
    pub fn verify_message(&mut self, api_data: &'static str) {
        /*For IB's internal purpose. Allows to provide means of verification
        between the TWS and third party programs*/

        // self.logRequest(current_fn_name()); vars())

        if !self.is_connected() {
            self.wrapper.lock().unwrap().error(
                NO_VALID_ID,
                TwsError::NotConnected.code(),
                TwsError::NotConnected.message(),
            );
            return;
        }

        if self.server_version() < MIN_SERVER_VER_LINKING {
            self.wrapper.lock().unwrap().error(
                NO_VALID_ID,
                TwsError::UpdateTws.code(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    "  It does not support verification request."
                )
                .as_ref(),
            );
            return;
        }

        let version = 1;

        let message_id: i32 = OutgoingMessageIds::VerifyMessage as i32;
        let mut msg = "".to_string();
        msg.push_str(&make_field(&message_id));

        msg.push_str(&make_field(&version));
        msg.push_str(&make_field(&api_data));

        self.send_request(msg.as_str())
    }

    //----------------------------------------------------------------------------------------------
    pub fn verify_and_auth_request(
        &mut self,
        api_name: &'static str,
        api_version: &'static str,
        opaque_isv_key: &'static str,
    ) {
        /*For IB's internal purpose. Allows to provide means of verification
        between the TWS and third party programs*/

        // self.logRequest(current_fn_name()); vars())

        if !self.is_connected() {
            self.wrapper.lock().unwrap().error(
                NO_VALID_ID,
                TwsError::NotConnected.code(),
                TwsError::NotConnected.message(),
            );
            return;
        }

        if self.server_version() < MIN_SERVER_VER_LINKING {
            self.wrapper.lock().unwrap().error(
                NO_VALID_ID,
                TwsError::UpdateTws.code(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    "  It does not support verification request."
                )
                .as_ref(),
            );
            return;
        }

        if !self.extra_auth {
            self.wrapper.lock().unwrap().error(
                NO_VALID_ID,
                TwsError::BadMessage.code(),
                format!("{}{}", TwsError::BadMessage.message(),
                        "  Intent to authenticate needs to be expressed during initial connect request.")
                    .as_ref(),
            );
            return;
        }

        let version = 1;

        let message_id: i32 = OutgoingMessageIds::VerifyAndAuthRequest as i32;
        let mut msg = "".to_string();
        msg.push_str(&make_field(&message_id));

        msg.push_str(&make_field(&version));
        msg.push_str(&make_field(&api_name));
        msg.push_str(&make_field(&api_version));
        msg.push_str(&make_field(&opaque_isv_key));

        self.send_request(msg.as_str())
    }

    //----------------------------------------------------------------------------------------------
    pub fn verify_and_auth_message(&mut self, api_data: &'static str, xyz_response: &'static str) {
        /*For IB's internal purpose. Allows to provide means of verification
        between the TWS and third party programs*/

        // self.logRequest(current_fn_name()); vars())

        if !self.is_connected() {
            self.wrapper.lock().unwrap().error(
                NO_VALID_ID,
                TwsError::NotConnected.code(),
                TwsError::NotConnected.message(),
            );
            return;
        }

        if self.server_version() < MIN_SERVER_VER_LINKING {
            self.wrapper.lock().unwrap().error(
                NO_VALID_ID,
                TwsError::UpdateTws.code(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    "  It does not support verification request."
                )
                .as_ref(),
            );
            return;
        }

        let version = 1;

        let message_id: i32 = OutgoingMessageIds::VerifyAndAuthMessage as i32;
        let mut msg = "".to_string();
        msg.push_str(&make_field(&message_id));

        msg.push_str(&make_field(&version));
        msg.push_str(&make_field(&api_data));
        msg.push_str(&make_field(&xyz_response));

        self.send_request(msg.as_str())
    }

    //----------------------------------------------------------------------------------------------
    pub fn req_sec_def_opt_params(
        &mut self,
        req_id: i32,
        underlying_symbol: &'static str,
        fut_fop_exchange: &'static str,
        underlying_sec_type: &'static str,
        underlying_con_id: i32,
    ) {
        /*Requests security pub fninition option parameters for viewing a
        contract's option chain reqId the ID chosen for the request
        underlying_symbol fut_fop_exchange The exchange on which the returned
        options are trading. Can be set to the empty string "" for all
        exchanges. underlying_sec_type The type of the underlying security,
        i.e. STK underlying_con_id the contract ID of the underlying security.
        Response comes via EWrapper.securityDefinitionOptionParameter()*/

        // self.logRequest(current_fn_name()); vars())

        if !self.is_connected() {
            self.wrapper.lock().unwrap().error(
                NO_VALID_ID,
                TwsError::NotConnected.code(),
                TwsError::NotConnected.message(),
            );
            return;
        }

        if self.server_version() < MIN_SERVER_VER_SEC_DEF_OPT_PARAMS_REQ {
            self.wrapper.lock().unwrap().error(
                NO_VALID_ID,
                TwsError::UpdateTws.code(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    "  It does not support security pub fninition option request."
                )
                .as_ref(),
            );
            return;
        }

        let message_id: i32 = OutgoingMessageIds::ReqSecDefOptParams as i32;
        let mut msg = "".to_string();
        msg.push_str(&make_field(&message_id));

        msg.push_str(&make_field(&req_id));
        msg.push_str(&make_field(&underlying_symbol));
        msg.push_str(&make_field(&fut_fop_exchange));
        msg.push_str(&make_field(&underlying_sec_type));
        msg.push_str(&make_field(&underlying_con_id));

        self.send_request(msg.as_str())
    }

    //----------------------------------------------------------------------------------------------
    pub fn req_soft_dollar_tiers(&mut self, req_id: i32) {
        /*Requests pre-pub fnined Soft Dollar Tiers. This is only supported for
        registered professional advisors and hedge and mutual funds who have
        configured Soft Dollar Tiers in Account Management*/

        // self.logRequest(current_fn_name()); vars())

        if !self.is_connected() {
            self.wrapper.lock().unwrap().error(
                NO_VALID_ID,
                TwsError::NotConnected.code(),
                TwsError::NotConnected.message(),
            );
            return;
        }

        let message_id: i32 = OutgoingMessageIds::ReqSoftDollarTiers as i32;
        let mut msg = "".to_string();
        msg.push_str(&make_field(&message_id));

        msg.push_str(&make_field(&req_id));

        self.send_request(msg.as_str())
    }

    //----------------------------------------------------------------------------------------------
    pub fn req_family_codes(&mut self) {
        // self.logRequest(current_fn_name()); vars())

        if !self.is_connected() {
            self.wrapper.lock().unwrap().error(
                NO_VALID_ID,
                TwsError::NotConnected.code(),
                TwsError::NotConnected.message(),
            );
            return;
        }

        if self.server_version() < MIN_SERVER_VER_REQ_FAMILY_CODES {
            self.wrapper.lock().unwrap().error(
                NO_VALID_ID,
                TwsError::UpdateTws.code(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    "  It does not support family codes request."
                )
                .as_ref(),
            );
            return;
        }

        let message_id: i32 = OutgoingMessageIds::ReqFamilyCodes as i32;
        let mut msg = "".to_string();
        msg.push_str(&make_field(&message_id));

        self.send_request(msg.as_str())
    }

    //----------------------------------------------------------------------------------------------
    pub fn req_matching_symbols(&mut self, req_id: i32, pattern: &'static str) {
        // self.logRequest(current_fn_name()); vars())

        if !self.is_connected() {
            self.wrapper.lock().unwrap().error(
                NO_VALID_ID,
                TwsError::NotConnected.code(),
                TwsError::NotConnected.message(),
            );
            return;
        }

        if self.server_version() < MIN_SERVER_VER_REQ_MATCHING_SYMBOLS {
            self.wrapper.lock().unwrap().error(
                NO_VALID_ID,
                TwsError::UpdateTws.code(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    "  It does not support matching symbols request."
                )
                .as_ref(),
            );
            return;
        }

        let message_id: i32 = OutgoingMessageIds::ReqMatchingSymbols as i32;
        let mut msg = "".to_string();
        msg.push_str(&make_field(&message_id));

        msg.push_str(&make_field(&req_id));
        msg.push_str(&make_field(&pattern));

        self.send_request(msg.as_str())
    }

    //----------------------------------------------------------------------------------------------
    pub fn req_completed_orders(&mut self, api_only: bool) {
        /*Call this function to request the completed orders. If api_only parameter
        is true, then only completed orders placed from API are requested.
        Each completed order will be fed back through the
        completedOrder() function on the EWrapper*/

        // self.logRequest(current_fn_name()); vars())

        if !self.is_connected() {
            self.wrapper.lock().unwrap().error(
                NO_VALID_ID,
                TwsError::NotConnected.code(),
                TwsError::NotConnected.message(),
            );
            return;
        }

        let message_id: i32 = OutgoingMessageIds::ReqCompletedOrders as i32;
        let mut msg = "".to_string();
        msg.push_str(&make_field(&message_id));

        msg.push_str(&make_field(&api_only));

        self.send_request(msg.as_str())
    }
}
