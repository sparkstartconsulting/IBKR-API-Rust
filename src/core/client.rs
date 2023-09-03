//! EClient and supporting structs.  Responsible for connecting to Trader Workstation or IB Gatway and sending requests
use std::io::Write;
use std::marker::Sync;
use std::net::Shutdown;
use std::net::TcpStream;
use std::ops::Deref;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use std::{fmt::Debug, thread};

use from_ascii::FromAscii;
use log::*;

use num_derive::FromPrimitive;

use super::streamer::{Streamer, TcpStreamer};
use crate::core::common::*;
use crate::core::contract::Contract;
use crate::core::decoder::Decoder;
use crate::core::errors::{IBKRApiLibError, TwsApiReportableError, TwsError};
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

pub(crate) static POISONED_MUTEX: &str = "Mutex was poisoned";

//==================================================================================================
/// Connection status
#[repr(i32)]
#[derive(FromPrimitive, Copy, Clone, Debug)]
pub enum ConnStatus {
    DISCONNECTED,
    CONNECTING,
    CONNECTED,
    REDIRECT,
}

//==================================================================================================
/// Struct for sending requests
//#[derive(Debug)]
pub struct EClient<T>
where
    T: Wrapper,
{
    wrapper: Arc<Mutex<T>>,
    pub(crate) stream: Option<Box<dyn Streamer>>,
    host: String,
    port: u32,
    extra_auth: bool,
    client_id: i32,
    pub(crate) server_version: i32,
    conn_time: String,
    pub conn_state: Arc<Mutex<ConnStatus>>,
    opt_capab: String,
    disconnect_requested: Arc<AtomicBool>,
}

impl<T> EClient<T>
where
    T: Wrapper + Send + Sync + 'static,
{
    pub fn new(wrapper: Arc<Mutex<T>>) -> Self {
        EClient {
            wrapper: wrapper,
            stream: None,
            host: "".to_string(),
            port: 0,
            extra_auth: false,
            client_id: 0,
            server_version: 0,
            conn_time: "".to_string(),
            conn_state: Arc::new(Mutex::new(ConnStatus::DISCONNECTED)),
            opt_capab: "".to_string(),
            disconnect_requested: Arc::new(AtomicBool::new(false)),
        }
    }
    fn send_request(&mut self, request: &str) -> Result<(), IBKRApiLibError> {
        let bytes = make_message(request)?;
        self.send_bytes(bytes.as_slice())?;
        Ok(())
    }

    fn send_bytes(&mut self, bytes: &[u8]) -> Result<usize, IBKRApiLibError> {
        let return_val = self.stream.as_mut().unwrap().write(bytes)?;
        Ok(return_val)
    }

    pub(crate) fn set_streamer(&mut self, streamer: Option<Box<dyn Streamer>>) {
        self.stream = streamer;
    }
    //----------------------------------------------------------------------------------------------
    /// Establishes a connection to TWS or IB Gateway
    pub fn connect(
        &mut self,
        host: &str,
        port: u32,
        client_id: i32,
    ) -> Result<(), IBKRApiLibError> {
        if self.is_connected() {
            info!("Already connected...");
            return Err(IBKRApiLibError::ApiError(TwsApiReportableError::new(
                -1,
                TwsError::AlreadyConnected.code().to_string(),
                TwsError::AlreadyConnected.message().to_string(),
            )));
        }
        self.host = host.to_string();
        self.port = port;
        self.client_id = client_id;
        info!("Connecting");
        self.disconnect_requested.store(false, Ordering::Release);
        *self.conn_state.lock().expect(POISONED_MUTEX) = ConnStatus::CONNECTING;
        let tcp_stream = TcpStream::connect(format!("{}:{}", self.host, port))?;
        let streamer = TcpStreamer::new(tcp_stream);
        self.set_streamer(Option::from(Box::new(streamer.clone()) as Box<dyn Streamer>));
        let (tx, rx) = channel::<String>();
        let mut reader = Reader::new(
            Box::new(streamer.clone()),
            tx.clone(),
            self.disconnect_requested.clone(),
        );

        let mut fields: Vec<String> = Vec::new();

        let v_100_prefix = "API\0";
        let v_100_version = format!("v{}..{}", MIN_CLIENT_VER, MAX_CLIENT_VER);

        let msg = make_message(v_100_version.as_str())?;

        let mut bytearray: Vec<u8> = Vec::new();
        bytearray.extend_from_slice(v_100_prefix.as_bytes());
        bytearray.extend_from_slice(msg.as_slice());

        self.send_bytes(bytearray.as_slice())?;

        let mut decoder = Decoder::new(
            self.wrapper.clone(),
            rx,
            self.server_version,
            self.conn_state.clone(),
        );

        //An Interactive Broker's developer's note: "sometimes I get news before the server version, thus the loop"
        while fields.len() != 2 {
            if fields.len() > 0 {
                decoder.interpret(fields.as_slice())?;
            }

            let buf = reader.recv_packet()?;

            if buf.len() > 0 {
                let (_size, msg, _remaining_messages) = read_msg(buf.as_slice())?;

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
            if decoder.run().is_err() {
                panic!("decoder.run() failed!!");
            }
        });
        *self.conn_state.lock().expect(POISONED_MUTEX) = ConnStatus::CONNECTED;
        info!("Connected");
        self.start_api()?;
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    /// Checks connection status
    pub fn is_connected(&self) -> bool {
        let connected = match *self.conn_state.lock().unwrap().deref() {
            ConnStatus::DISCONNECTED => false,
            ConnStatus::CONNECTED => true,
            ConnStatus::CONNECTING => false,
            ConnStatus::REDIRECT => false,
        };

        //debug!("finished checking connected...");
        connected
    }

    //----------------------------------------------------------------------------------------------
    /// Get the server version (important for checking feature flags for different versions)
    pub fn server_version(&self) -> i32 {
        self.server_version
    }

    //----------------------------------------------------------------------------------------------
    /// Sets server logging level
    pub fn set_server_log_level(&mut self, log_evel: i32) -> Result<(), IBKRApiLibError> {
        //The pub default detail level is ERROR. For more details, see API
        //        Logging.
        //TODO Make log_level an enum
        debug!("set_server_log_level -- log_evel: {}", log_evel);

        self.check_connected(NO_VALID_ID)?;

        let version = 1;
        let _log_level = log_evel;

        let mut msg = "".to_string();

        let message_id = OutgoingMessageIds::SetServerLoglevel as i32;
        let _x = message_id.to_be_bytes();
        msg.push_str(&make_field(&message_id)?);
        msg.push_str(&make_field(&version)?);
        msg.push_str(&make_field(&_log_level)?);

        self.send_request(msg.as_str())?;
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    /// Gets the connection time
    pub fn tws_connection_time(&mut self) -> String {
        //"""Returns the time the API client made a connection to TWS."""

        self.conn_time.clone()
    }

    //----------------------------------------------------------------------------------------------
    /// Request the current time according to TWS or IB Gateway
    pub fn req_current_time(&mut self) -> Result<(), IBKRApiLibError> {
        let version = 2;

        let message_id: i32 = OutgoingMessageIds::ReqCurrentTime as i32;
        let mut msg = "".to_string();
        msg.push_str(&make_field(&message_id)?);
        msg.push_str(&make_field(&version)?);

        debug!("Requesting current time: {}", msg.as_str());
        self.send_request(msg.as_str())
    }

    //----------------------------------------------------------------------------------------------
    /// Disconnect from TWS
    pub fn disconnect(&mut self) -> Result<(), IBKRApiLibError> {
        if !self.is_connected() {
            info!("Already disconnected...");
            return Ok(());
        }
        info!("Disconnect requested.  Shutting down stream...");
        self.disconnect_requested.store(true, Ordering::Release);
        self.stream.as_mut().unwrap().shutdown(Shutdown::Both)?;
        *self.conn_state.lock().expect(POISONED_MUTEX) = ConnStatus::DISCONNECTED;
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    /// Initiates the message exchange between the client application and the TWS/IB Gateway
    fn start_api(&mut self) -> Result<(), IBKRApiLibError> {
        self.check_connected(NO_VALID_ID)?;

        let version = 2;
        let mut opt_capab = "".to_string();
        if self.server_version >= MIN_SERVER_VER_OPTIONAL_CAPABILITIES as i32 {
            opt_capab = make_field(&self.opt_capab)?;
        }

        let msg = format!(
            "{}{}{}{}",
            make_field(&mut (Some(OutgoingMessageIds::StartApi).unwrap() as i32))?,
            make_field(&mut version.to_string())?,
            make_field(&mut self.client_id.to_string())?,
            opt_capab
        );

        self.send_request(msg.as_str())?;
        Ok(())
    }

    //##############################################################################################
    //################################### Market Data
    //##############################################################################################
    /// Call this function to request market data. The market data
    /// will be returned by the tick_price and tick_size wrapper events.
    ///
    /// # Arguments
    /// * req_id - The request id. Must be a unique value. When the
    ///            market data returns, it will be identified by this tag. This is
    ///            also used when canceling the market data.
    /// * contract - This structure contains a description of the
    ///              Contract for which market data is being requested.
    /// * generic_tick_list - A commma delimited list of generic tick types.
    ///                       Tick types can be found in the Generic Tick Types page.
    ///                       Prefixing w/ 'mdoff' indicates that top mkt data shouldn't tick.
    ///                       You can specify the news source by postfixing w/ ':<source>.
    ///                       Example: "mdoff, 292: FLY + BRF"
    /// * snapshot - Check to return a single snapshot of Market data and
    ///                    have the market data subscription cancel. Do not enter any
    ///                    generic_tick_list values if you use snapshots.
    /// * regulatory_snapshot - With the US Value Snapshot Bundle for stocks,
    ///                         regulatory snapshots are available for 0.01 USD each.
    /// * mkt_data_options - For internal use only. Use default value XYZ.
    pub fn req_mkt_data(
        &mut self,
        req_id: i32,
        contract: &Contract,
        generic_tick_list: &str,
        snapshot: bool,
        regulatory_snapshot: bool,
        mkt_data_options: Vec<TagValue>,
    ) -> Result<(), IBKRApiLibError> {
        self.check_connected(req_id)?;

        if self.server_version() < MIN_SERVER_VER_DELTA_NEUTRAL {
            if let Some(_value) = &contract.delta_neutral_contract {
                let err = IBKRApiLibError::ApiError(TwsApiReportableError::new(
                    req_id,
                    TwsError::UpdateTws.code().to_string(),
                    format!(
                        "{}{}",
                        TwsError::UpdateTws.message(),
                        " It does not support delta-neutral orders."
                    ),
                ));

                return Err(err);
            }
        }

        if self.server_version() < MIN_SERVER_VER_REQ_MKT_DATA_CONID && contract.con_id > 0 {
            let err = IBKRApiLibError::ApiError(TwsApiReportableError::new(
                req_id,
                TwsError::NotConnected.code().to_string(),
                TwsError::NotConnected.message().to_string(),
            ));
            return Err(err);
        }

        if self.server_version() < MIN_SERVER_VER_TRADING_CLASS && "" != contract.trading_class {
            let err = IBKRApiLibError::ApiError(TwsApiReportableError::new(
                req_id,
                TwsError::UpdateTws.code().to_string(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    " It does not support trading_class parameter in req_mkt_data."
                ),
            ));

            return Err(err);
        }

        let version = 11;

        let message_id: i32 = OutgoingMessageIds::ReqMktData as i32;

        let mut msg = "".to_string();

        // send req mkt data msg
        msg.push_str(&make_field(&message_id)?);
        msg.push_str(&make_field(&version)?);
        msg.push_str(&make_field(&req_id)?);

        // send contract fields
        if self.server_version() >= MIN_SERVER_VER_REQ_MKT_DATA_CONID {
            msg.push_str(&make_field(&contract.con_id)?);
            msg.push_str(&make_field(&contract.symbol)?);

            msg.push_str(&make_field(&contract.sec_type)?);
            msg.push_str(&make_field(&contract.last_trade_date_or_contract_month)?);
            msg.push_str(&make_field(&contract.strike)?);
            msg.push_str(&make_field(&contract.right)?);
            msg.push_str(&make_field(&contract.multiplier)?); // srv v15 and above
            msg.push_str(&make_field(&contract.exchange)?);
            msg.push_str(&make_field(&contract.primary_exchange)?); // srv v14 and above
            msg.push_str(&make_field(&contract.currency)?);
            msg.push_str(&make_field(&contract.local_symbol)?); //  srv v2 and above
        }

        if self.server_version() >= MIN_SERVER_VER_TRADING_CLASS {
            msg.push_str(&make_field(&contract.trading_class)?);
        }
        // Send combo legs for BAG requests(srv v8 and above)
        if contract.sec_type == "BAG" {
            let combo_legs_count = contract.combo_legs.len();
            msg.push_str(&make_field(&combo_legs_count)?);
            for combo_leg in &contract.combo_legs {
                msg.push_str(&make_field(&combo_leg.con_id)?);
                msg.push_str(&make_field(&combo_leg.ratio)?);
                msg.push_str(&make_field(&combo_leg.action)?);
                msg.push_str(&make_field(&combo_leg.exchange)?);
            }
        }

        if self.server_version() >= MIN_SERVER_VER_DELTA_NEUTRAL {
            if contract.delta_neutral_contract.is_some() {
                msg.push_str(&make_field(&true)?);
                msg.push_str(&make_field(
                    &contract.delta_neutral_contract.as_ref().unwrap().con_id,
                )?);
                msg.push_str(&make_field(
                    &contract.delta_neutral_contract.as_ref().unwrap().delta,
                )?);
                msg.push_str(&make_field(
                    &contract.delta_neutral_contract.as_ref().unwrap().price,
                )?);
            } else {
                msg.push_str(&make_field(&false)?);
            }

            msg.push_str(&make_field(&String::from(generic_tick_list))?); // srv v31 and above
            msg.push_str(&make_field(&snapshot)?); // srv v35 and above
        }

        if self.server_version() >= MIN_SERVER_VER_REQ_SMART_COMPONENTS {
            msg.push_str(&make_field(&regulatory_snapshot)?);
        }

        // send mktDataOptions parameter
        if self.server_version() >= MIN_SERVER_VER_LINKING {
            // current doc says this part is for "internal use only" -> won't support it
            if mkt_data_options.len() > 0 {
                let err = IBKRApiLibError::ApiError(TwsApiReportableError::new(
                    req_id,
                    TwsError::UpdateTws.code().to_string(),
                    format!(
                        "{}{}",
                        TwsError::UpdateTws.message(),
                        " Internal use only.  mkt_data_options not supported."
                    ),
                ));

                return Err(err);
            }
            let mkt_data_options_str = "";
            msg.push_str(&make_field(&mkt_data_options_str)?);
        }

        self.send_request(msg.as_str())?;
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    /// After calling this function, market data for the specified id will stop flowing.
    ///
    /// # Arguments
    /// * req_id - The ID that was specified in the call to req_mkt_data()
    pub fn cancel_mkt_data(&mut self, req_id: i32) -> Result<(), IBKRApiLibError> {
        self.check_connected(req_id)?;

        let version = 2;

        let message_id: i32 = OutgoingMessageIds::CancelMktData as i32;
        let mut msg = "".to_string();
        msg.push_str(&make_field(&message_id)?);
        msg.push_str(&make_field(&version)?);
        msg.push_str(&make_field(&req_id)?);

        self.send_request(msg.as_str())?;
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    /// The API can receive frozen market data from Trader
    /// Workstation. Frozen market data is the last data recorded in our system.
    /// During normal trading hours, the API receives real-time market data. If
    /// you use this function, you are telling TWS to automatically switch to
    /// frozen market data after the close. Then, before the opening of the next
    /// trading day, market data will automatically switch back to real-time
    /// market data.
    ///
    /// # Arguments
    /// * market_data_type
    /// * 1 for real-time streaming market data
    /// * 2 for frozen market data
    pub fn req_market_data_type(&mut self, market_data_type: i32) -> Result<(), IBKRApiLibError> {
        self.check_connected(NO_VALID_ID)?;

        if self.server_version() < MIN_SERVER_VER_REQ_MARKET_DATA_TYPE {
            let err = IBKRApiLibError::ApiError(TwsApiReportableError::new(
                NO_VALID_ID,
                TwsError::UpdateTws.code().to_string(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    " It does not support market data type requests."
                ),
            ));

            return Err(err);
        }

        let mut msg = "".to_string();
        let version = 1;
        let message_id = OutgoingMessageIds::ReqMarketDataType as i32;

        msg.push_str(&make_field(&message_id)?);
        msg.push_str(&make_field(&version)?);
        msg.push_str(&make_field(&market_data_type)?);

        self.send_request(msg.as_str())?;
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    /// Returns the mapping of single letter codes to exchange names given the mapping identifier.
    /// # Arguments
    /// * req_id - The request id. Must be a unique value. When the
    ///            market data returns, it will be identified by this tag. This is
    ///            also used when canceling the market data.
    /// * bbo_exchange - mapping identifier received from Wrapper::tick_req_params
    pub fn req_smart_components(
        &mut self,
        req_id: i32,
        bbo_exchange: &str,
    ) -> Result<(), IBKRApiLibError> {
        self.check_connected(req_id)?;

        if self.server_version() < MIN_SERVER_VER_REQ_SMART_COMPONENTS {
            let err = IBKRApiLibError::ApiError(TwsApiReportableError::new(
                req_id,
                TwsError::UpdateTws.code().to_string(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    " It does not support smart components request."
                ),
            ));

            return Err(err);
        }

        let mut msg = "".to_string();

        let message_id = OutgoingMessageIds::ReqSmartComponents as i32;

        msg.push_str(&make_field(&message_id)?);
        msg.push_str(&make_field(&req_id)?);
        msg.push_str(&make_field(&String::from(bbo_exchange))?);

        self.send_request(msg.as_str())?;
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    /// Requests details about a given market rule
    /// The market rule for an instrument on a particular exchange provides details about how the
    /// minimum price increment changes with price
    ///
    /// A list of market rule ids can be obtained by invoking req_contract_details on a particular contract.
    /// The returned market rule ID list will provide the market rule ID for the instrument in the correspond valid exchange list in contractDetails.
    /// # Arguments
    /// * market_rule_id -  the id of market rule
    pub fn req_market_rule(&mut self, market_rule_id: i32) -> Result<(), IBKRApiLibError> {
        self.check_connected(NO_VALID_ID)?;

        if self.server_version() < MIN_SERVER_VER_MARKET_RULES {
            let err = IBKRApiLibError::ApiError(TwsApiReportableError::new(
                NO_VALID_ID,
                TwsError::UpdateTws.code().to_string(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    " It does not support market rule requests."
                ),
            ));

            return Err(err);
        }

        let mut msg = "".to_string();

        let message_id = OutgoingMessageIds::ReqMarketRule as i32;

        msg.push_str(&make_field(&message_id)?);
        msg.push_str(&make_field(&market_rule_id)?);

        self.send_request(msg.as_str())?;
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    /// Request tick by tick data
    ///
    /// # Arguments
    /// * req_id	- unique identifier of the request.
    /// * contract	- the contract for which tick-by-tick data is requested.
    /// * tick_type	- TickByTickType data type: "Last", "AllLast", "BidAsk" or "MidPoint".
    /// * number_of_ticks	- number of ticks.
    /// * ignore_size	- ignore size flag.
    pub fn req_tick_by_tick_data(
        &mut self,
        req_id: i32,
        contract: &Contract,
        tick_type: TickByTickType,
        number_of_ticks: i32,
        ignore_size: bool,
    ) -> Result<(), IBKRApiLibError> {
        self.check_connected(NO_VALID_ID)?;

        if self.server_version() < MIN_SERVER_VER_TICK_BY_TICK {
            let err = IBKRApiLibError::ApiError(TwsApiReportableError::new(
                req_id,
                TwsError::UpdateTws.code().to_string(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    " It does not support tick-by-tick data requests."
                ),
            ));

            return Err(err);
        }

        if self.server_version() < MIN_SERVER_VER_TICK_BY_TICK_IGNORE_SIZE {
            let err = IBKRApiLibError::ApiError(TwsApiReportableError::new(
                req_id,
                TwsError::UpdateTws.code().to_string(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    " It does not support ignore_size and number_of_ticks parameters in tick-by-tick data requests."
                ),
            ));

            return Err(err);
        }

        let mut msg = "".to_string();

        let message_id = OutgoingMessageIds::ReqTickByTickData as i32;

        msg.push_str(&make_field(&message_id)?);

        //    msg.push_str(&make_field(&OUT.REQ_TICK_BY_TICK_DATA)\
        msg.push_str(&make_field(&req_id)?);
        msg.push_str(&make_field(&contract.con_id)?);
        msg.push_str(&make_field(&contract.symbol)?);
        msg.push_str(&make_field(&contract.sec_type)?);
        msg.push_str(&make_field(&contract.last_trade_date_or_contract_month)?);
        msg.push_str(&make_field(&contract.strike)?);
        msg.push_str(&make_field(&contract.right)?);
        msg.push_str(&make_field(&contract.multiplier)?);
        msg.push_str(&make_field(&contract.exchange)?);
        msg.push_str(&make_field(&contract.primary_exchange)?);
        msg.push_str(&make_field(&contract.currency)?);
        msg.push_str(&make_field(&contract.local_symbol)?);
        msg.push_str(&make_field(&contract.trading_class)?);
        msg.push_str(&make_field(&(tick_type.to_string()))?);

        if self.server_version() >= MIN_SERVER_VER_TICK_BY_TICK_IGNORE_SIZE {
            msg.push_str(&make_field(&number_of_ticks)?);
            msg.push_str(&make_field(&ignore_size)?);
        }

        self.send_request(msg.as_str())?;
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    /// Cancel tick by tick data
    ///
    /// # Arguments
    /// * req_id	- The identifier of the original request.
    pub fn cancel_tick_by_tick_data(&mut self, req_id: i32) -> Result<(), IBKRApiLibError> {
        self.check_connected(req_id)?;

        if self.server_version() < MIN_SERVER_VER_TICK_BY_TICK {
            let err = IBKRApiLibError::ApiError(TwsApiReportableError::new(
                req_id,
                TwsError::UpdateTws.code().to_string(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    " It does not support tick-by-tick data requests."
                ),
            ));

            return Err(err);
        }

        let mut msg = "".to_string();

        let message_id = OutgoingMessageIds::CancelTickByTickData as i32;

        msg.push_str(&make_field(&message_id)?);
        msg.push_str(&make_field(&req_id)?);

        self.send_request(msg.as_str())?;
        Ok(())
    }

    //##########################################################################
    //################## Options
    //##########################################################################

    /// Call this function to calculate volatility for a supplied
    /// option price and underlying price. Result will be delivered
    /// via Wrapper::tick_option_computation
    ///
    /// # Arguments
    /// * req_id - The request id.
    /// * contract - Describes the contract.
    /// * option_price - The price of the option.
    /// * under_price - Price of the underlying.
    /// * impl_vol_options - Implied volatility options.
    pub fn calculate_implied_volatility(
        &mut self,
        req_id: i32,
        contract: &Contract,
        option_price: f64,
        under_price: f64,
        impl_vol_options: Vec<TagValue>,
    ) -> Result<(), IBKRApiLibError> {
        self.check_connected(req_id)?;

        if self.server_version() < MIN_SERVER_VER_REQ_CALC_IMPLIED_VOLAT {
            let err = IBKRApiLibError::ApiError(TwsApiReportableError::new(
                req_id,
                TwsError::UpdateTws.code().to_string(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    " It does not support calculate_implied_volatility req."
                ),
            ));

            return Err(err);
        }

        if self.server_version() < MIN_SERVER_VER_TRADING_CLASS && "" != contract.trading_class {
            let err = IBKRApiLibError::ApiError(TwsApiReportableError::new(
                req_id,
                TwsError::UpdateTws.code().to_string(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    " It does not support trading_class parameter in calculate_implied_volatility."
                ),
            ));

            return Err(err);
        }

        let version = 3;

        let mut msg = "".to_string();

        let message_id = OutgoingMessageIds::ReqCalcImpliedVolat as i32;

        msg.push_str(&make_field(&message_id)?);

        msg.push_str(&make_field(&version)?);
        msg.push_str(&make_field(&req_id)?);

        // send contract fields
        msg.push_str(&make_field(&contract.con_id)?);
        msg.push_str(&make_field(&contract.symbol)?);
        msg.push_str(&make_field(&contract.sec_type)?);
        msg.push_str(&make_field(&contract.last_trade_date_or_contract_month)?);
        msg.push_str(&make_field(&contract.strike)?);
        msg.push_str(&make_field(&contract.right)?);
        msg.push_str(&make_field(&contract.multiplier)?);
        msg.push_str(&make_field(&contract.exchange)?);
        msg.push_str(&make_field(&contract.primary_exchange)?);
        msg.push_str(&make_field(&contract.currency)?);
        msg.push_str(&make_field(&contract.local_symbol)?);

        if self.server_version() >= MIN_SERVER_VER_TRADING_CLASS {
            msg.push_str(&make_field(&contract.trading_class)?);
        }

        msg.push_str(&make_field(&option_price)?);
        msg.push_str(&make_field(&under_price)?);

        if self.server_version() >= MIN_SERVER_VER_LINKING {
            let mut impl_vol_opt_str = "".to_string();
            let tag_values_count = impl_vol_options.len();
            if tag_values_count > 0 {
                impl_vol_opt_str = impl_vol_options
                    .iter()
                    .map(|x| format!("{}={};", x.tag, x.value))
                    .collect::<String>();
            }
            msg.push_str(&make_field(&tag_values_count)?);
            msg.push_str(&make_field(&impl_vol_opt_str)?);
        }
        error!("sending calculate_implied_volatility");
        error!("{}", msg);
        self.send_request(msg.as_str())?;
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    /// Call this function to calculate option price and greek values for a supplied volatility and underlying price.
    ///
    /// # Arguments
    /// * req_id - The request id.
    /// * contract - Describes the contract.
    /// * volatility - The volatility.
    /// * under_price - Price of the underlying.
    pub fn calculate_option_price(
        &mut self,
        req_id: i32,
        contract: &Contract,
        volatility: f64,
        under_price: f64,
        opt_prc_options: Vec<TagValue>,
    ) -> Result<(), IBKRApiLibError> {
        self.check_connected(req_id)?;

        if self.server_version() < MIN_SERVER_VER_REQ_CALC_IMPLIED_VOLAT {
            let err = IBKRApiLibError::ApiError(TwsApiReportableError::new(
                req_id,
                TwsError::UpdateTws.code().to_string(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    " It does not support calculateImpliedVolatility req."
                ),
            ));

            return Err(err);
        }

        if self.server_version() < MIN_SERVER_VER_TRADING_CLASS {
            if "" != contract.trading_class {
                let err = IBKRApiLibError::ApiError(TwsApiReportableError::new(
                    req_id,
                    TwsError::UpdateTws.code().to_string(),
                    format!(
                        "{}{}",
                        TwsError::UpdateTws.message(),
                        " It does not support trading_class parameter in calculateImpliedVolatility."
                    ),
                ));

                return Err(err);
            }
        }

        let version = 3;

        // send req mkt data msg
        let mut msg = "".to_string();

        let message_id = OutgoingMessageIds::ReqCalcOptionPrice as i32;

        msg.push_str(&make_field(&message_id)?);
        msg.push_str(&make_field(&version)?);
        msg.push_str(&make_field(&req_id)?);
        // send contract fields
        msg.push_str(&make_field(&contract.con_id)?);
        msg.push_str(&make_field(&contract.symbol)?);
        msg.push_str(&make_field(&contract.sec_type)?);
        msg.push_str(&make_field(&contract.last_trade_date_or_contract_month)?);
        msg.push_str(&make_field(&contract.strike)?);
        msg.push_str(&make_field(&contract.right)?);
        msg.push_str(&make_field(&contract.multiplier)?);
        msg.push_str(&make_field(&contract.exchange)?);
        msg.push_str(&make_field(&contract.primary_exchange)?);
        msg.push_str(&make_field(&contract.currency)?);
        msg.push_str(&make_field(&contract.local_symbol)?);

        if self.server_version() >= MIN_SERVER_VER_TRADING_CLASS {
            msg.push_str(&make_field(&contract.trading_class)?);
        }

        msg.push_str(&make_field(&volatility)?);
        msg.push_str(&make_field(&under_price)?);

        if self.server_version() >= MIN_SERVER_VER_LINKING {
            let _opt_prc_opt_str = "".to_string();
            let tag_values_count = opt_prc_options.len();
            if tag_values_count > 0 {
                let opt_prc_opt_str = opt_prc_options
                    .iter()
                    .map(|x| format!("{}={};", x.tag, x.value))
                    .collect::<String>();

                msg.push_str(&make_field(&tag_values_count)?);
                msg.push_str(&make_field(&opt_prc_opt_str)?);
            }
        }
        self.send_request(msg.as_str())?;
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    /// Call this function to cancel a request to calculate the option
    /// price and greek values for a supplied volatility and underlying price.
    ///
    /// # Arguments
    /// * req_id - The original request id.
    pub fn cancel_calculate_option_price(&mut self, req_id: i32) -> Result<(), IBKRApiLibError> {
        self.check_connected(req_id)?;

        if self.server_version() < MIN_SERVER_VER_REQ_CALC_IMPLIED_VOLAT {
            let err = IBKRApiLibError::ApiError(TwsApiReportableError::new(
                req_id,
                TwsError::UpdateTws.code().to_string(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    " It does not support calculateImpliedVolatility req."
                ),
            ));

            return Err(err);
        }

        let version = 1;

        let mut msg = "".to_string();

        let message_id = OutgoingMessageIds::CancelCalcOptionPrice as i32;

        msg.push_str(&make_field(&message_id)?);
        msg.push_str(&make_field(&version)?);
        msg.push_str(&make_field(&req_id)?);

        self.send_request(msg.as_str())?;
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    /// Call this function to cancel a request to calculate the option implied volatility.
    ///
    /// # Arguments
    /// * req_id - The original request id.
    pub fn cancel_calculate_implied_volatility(
        &mut self,
        req_id: i32,
    ) -> Result<(), IBKRApiLibError> {
        self.check_connected(req_id)?;

        if self.server_version() < MIN_SERVER_VER_REQ_CALC_IMPLIED_VOLAT {
            let err = IBKRApiLibError::ApiError(TwsApiReportableError::new(
                req_id,
                TwsError::UpdateTws.code().to_string(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    " It does not support calculateImpliedVolatility req."
                ),
            ));

            return Err(err);
        }

        let version = 1;

        let mut msg = "".to_string();

        let message_id = OutgoingMessageIds::CancelCalcImpliedVolat as i32;

        msg.push_str(&make_field(&message_id)?);
        msg.push_str(&make_field(&version)?);
        msg.push_str(&make_field(&req_id)?);

        self.send_request(msg.as_str())?;
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    /// Call this function to excercise options
    ///
    /// # Arguments
    /// * req_id - The ticker id. multipleust be a unique value.
    /// * contract - This structure contains a description of the contract to be exercised
    /// * exercise_action - Specifies whether you want the option to lapse or be exercised. Values are:
    ///     * 1 = exercise
    ///     * 2 = lapse.
    /// * exercise_quantity - The quantity you want to exercise.
    /// * account - destination account
    /// * override - Specifies whether your setting will override the system's
    ///              natural action. For example, if your action is "exercise" and the
    ///              option is not in-the-money, by natural action the option would not
    ///              exercise. If you have override set to "yes" the natural action would
    ///              be overridden and the out-of-the money option would be exercised.
    ///              Values are:
    ///      * 0 = no
    ///      * 1 = yes.
    pub fn exercise_options(
        &mut self,
        req_id: i32,
        contract: &Contract,
        exercise_action: i32,
        exercise_quantity: i32,
        account: &String,
        over_ride: i32,
    ) -> Result<(), IBKRApiLibError> {
        self.check_connected(req_id)?;

        if self.server_version() < MIN_SERVER_VER_TRADING_CLASS {
            if !contract.trading_class.is_empty() {
                let err = IBKRApiLibError::ApiError(TwsApiReportableError::new(
                    req_id,
                    TwsError::UpdateTws.code().to_string(),
                    format!(
                        "{}{}",
                        TwsError::UpdateTws.message(),
                        " It does not support con_id, multiplier, trading_class parameter in exercise_options."
                    ),
                ));

                return Err(err);
            }
        }

        let version = 2;

        // send req mkt data msg
        let mut msg = "".to_string();

        let message_id = OutgoingMessageIds::ExerciseOptions as i32;

        msg.push_str(&make_field(&message_id)?);

        msg.push_str(&make_field(&version)?);
        msg.push_str(&make_field(&req_id)?);

        // send contract fields
        if self.server_version() >= MIN_SERVER_VER_TRADING_CLASS {
            msg.push_str(&make_field(&contract.con_id)?);
        }
        msg.push_str(&make_field(&contract.symbol)?);
        msg.push_str(&make_field(&contract.sec_type)?);
        msg.push_str(&make_field(&contract.last_trade_date_or_contract_month)?);
        msg.push_str(&make_field(&contract.strike)?);
        msg.push_str(&make_field(&contract.right)?);
        msg.push_str(&make_field(&contract.multiplier)?);
        msg.push_str(&make_field(&contract.exchange)?);
        msg.push_str(&make_field(&contract.currency)?);
        msg.push_str(&make_field(&contract.local_symbol)?);
        if self.server_version() >= MIN_SERVER_VER_TRADING_CLASS {
            msg.push_str(&make_field(&contract.trading_class)?);
        }
        msg.push_str(&make_field(&exercise_action)?);
        msg.push_str(&make_field(&exercise_quantity)?);
        msg.push_str(&make_field(account)?);
        msg.push_str(&make_field(&over_ride)?);

        self.send_request(msg.as_str())?;
        Ok(())
    }

    //#########################################################################
    //################## Orders
    //########################################################################

    /// Call this function to place an order. The order status will
    /// be returned by the Wrapper::order_status event.
    ///
    /// # Arguments
    /// * order_id - The order id. You must specify a unique value. When the
    ///              order status returns, it will be identified by this tag.
    ///              This tag is also used when canceling the order.
    /// * contract - This structure contains a description of the
    ///              contract which is being traded.
    /// * order - This structure contains the details of the order.
    ///
    /// Note: Each client MUST connect with a unique client_id.
    pub fn place_order(
        &mut self,
        order_id: i32,
        contract: &Contract,
        order: &Order,
    ) -> Result<(), IBKRApiLibError> {
        self.check_connected(NO_VALID_ID)?;

        if self.server_version() < MIN_SERVER_VER_DELTA_NEUTRAL {
            if contract.delta_neutral_contract.is_some() {
                let err = IBKRApiLibError::ApiError(TwsApiReportableError::new(
                    NO_VALID_ID,
                    TwsError::UpdateTws.code().to_string(),
                    format!(
                        "{}{}",
                        TwsError::UpdateTws.message(),
                        " It does not support delta-neutral orders."
                    ),
                ));

                return Err(err);
            }
        }

        if self.server_version() < MIN_SERVER_VER_SCALE_ORDERS2
            && order.scale_subs_level_size != UNSET_INTEGER
        {
            let err = IBKRApiLibError::ApiError(TwsApiReportableError::new(
                order_id,
                TwsError::UpdateTws.code().to_string(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    " It does not support Subsequent Level Size for Scale orders."
                ),
            ));

            return Err(err);
        }

        if self.server_version() < MIN_SERVER_VER_ALGO_ORDERS && !order.algo_strategy.is_empty() {
            let err = IBKRApiLibError::ApiError(TwsApiReportableError::new(
                order_id,
                TwsError::UpdateTws.code().to_string(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    " It does not support algo orders."
                ),
            ));

            return Err(err);
        }

        if self.server_version() < MIN_SERVER_VER_NOT_HELD && order.not_held {
            let err = IBKRApiLibError::ApiError(TwsApiReportableError::new(
                order_id,
                TwsError::UpdateTws.code().to_string(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    " It does not support notHeld parameter."
                ),
            ));

            return Err(err);
        }

        if self.server_version() < MIN_SERVER_VER_SEC_ID_TYPE
            && (!contract.sec_id_type.is_empty() || !contract.sec_id.is_empty())
        {
            let err = IBKRApiLibError::ApiError(TwsApiReportableError::new(
                order_id,
                TwsError::UpdateTws.code().to_string(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    " It does not support sec_id_type && secId parameters."
                ),
            ));

            return Err(err);
        }

        if self.server_version() < MIN_SERVER_VER_PLACE_ORDER_CONID && contract.con_id > 0 {
            let err = IBKRApiLibError::ApiError(TwsApiReportableError::new(
                order_id,
                TwsError::UpdateTws.code().to_string(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    " It does not support con_id parameter."
                ),
            ));

            return Err(err);
        }

        if self.server_version() < MIN_SERVER_VER_SSHORTX {
            if order.exempt_code != -1 {
                let err = IBKRApiLibError::ApiError(TwsApiReportableError::new(
                    order_id,
                    TwsError::UpdateTws.code().to_string(),
                    format!(
                        "{}{}",
                        TwsError::UpdateTws.message(),
                        " It does not support exempt_code parameter."
                    ),
                ));

                return Err(err);
            }
            if contract.combo_legs.len() > 0
                && contract.combo_legs.iter().any(|x| x.exempt_code != -1)
            {
                let err = IBKRApiLibError::ApiError(TwsApiReportableError::new(
                    order_id,
                    TwsError::UpdateTws.code().to_string(),
                    format!(
                        "{}{}",
                        TwsError::UpdateTws.message(),
                        " It does not support exempt_code parameter."
                    ),
                ));

                return Err(err);
            }
        }
        if self.server_version() < MIN_SERVER_VER_HEDGE_ORDERS && !order.hedge_type.is_empty() {
            let err = IBKRApiLibError::ApiError(TwsApiReportableError::new(
                order_id,
                TwsError::UpdateTws.code().to_string(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    " It does not support hedge orders."
                ),
            ));

            return Err(err);
        }

        if self.server_version() < MIN_SERVER_VER_OPT_OUT_SMART_ROUTING
            && order.opt_out_smart_routing
        {
            let err = IBKRApiLibError::ApiError(TwsApiReportableError::new(
                order_id,
                TwsError::UpdateTws.code().to_string(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    " It does not support optOutSmartRouting parameter."
                ),
            ));

            return Err(err);
        }

        if self.server_version() < MIN_SERVER_VER_DELTA_NEUTRAL_CONID
            && (order.delta_neutral_con_id > 0
                || !order.delta_neutral_settling_firm.is_empty()
                || !order.delta_neutral_clearing_account.is_empty()
                || !order.delta_neutral_clearing_intent.is_empty())
        {
            let err = IBKRApiLibError::ApiError(TwsApiReportableError::new(
                order_id,
                TwsError::UpdateTws.code().to_string(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    " It does not support deltaNeutral parameters: con_id, SettlingFirm, ClearingAccount, ClearingIntent."
                ),
            ));

            return Err(err);
        }

        if self.server_version() < MIN_SERVER_VER_DELTA_NEUTRAL_OPEN_CLOSE
            && (!order.delta_neutral_open_close.is_empty()
                || order.delta_neutral_short_sale
                || order.delta_neutral_short_sale_slot > 0
                || !order.delta_neutral_designated_location.is_empty())
        {
            let err = IBKRApiLibError::ApiError(TwsApiReportableError::new(
                order_id,
                TwsError::UpdateTws.code().to_string(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    " It does not support deltaNeutral parameters: open_close, ShortSale, short_sale_slot, designated_location."
                ),
            ));

            return Err(err);
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
            let err = IBKRApiLibError::ApiError(TwsApiReportableError::new(
                order_id,
                TwsError::UpdateTws.code().to_string(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    " It does not support Scale order parameters: PriceAdjustValue, \
                PriceAdjustInterval, ProfitOffset, AutoReset, InitPosition, InitFillQty && RandomPercent"
                ),
            ));

            return Err(err);
        }

        if self.server_version() < MIN_SERVER_VER_ORDER_COMBO_LEGS_PRICE
            && contract.sec_type == "BAG"
            && order.order_combo_legs.len() > 0
            && order
                .order_combo_legs
                .iter()
                .any(|x| x.price != UNSET_DOUBLE)
        {
            let err = IBKRApiLibError::ApiError(TwsApiReportableError::new(
                order_id,
                TwsError::UpdateTws.code().to_string(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    " It does not support per-leg prices for order combo legs."
                ),
            ));

            return Err(err);
        }

        if self.server_version() < MIN_SERVER_VER_TRAILING_PERCENT
            && order.trailing_percent != UNSET_DOUBLE
        {
            let err = IBKRApiLibError::ApiError(TwsApiReportableError::new(
                order_id,
                TwsError::UpdateTws.code().to_string(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    " It does not support trailing percent parameter."
                ),
            ));

            return Err(err);
        }

        if self.server_version() < MIN_SERVER_VER_TRADING_CLASS
            && !contract.trading_class.is_empty()
        {
            let err = IBKRApiLibError::ApiError(TwsApiReportableError::new(
                order_id,
                TwsError::UpdateTws.code().to_string(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    " It does not support trading_class parameter in placeOrder."
                ),
            ));

            return Err(err);
        }

        if self.server_version() < MIN_SERVER_VER_SCALE_TABLE
            && (!order.scale_table.is_empty()
                || !order.active_start_time.is_empty()
                || !order.active_stop_time.is_empty())
        {
            let err = IBKRApiLibError::ApiError(TwsApiReportableError::new(
                order_id,
                TwsError::UpdateTws.code().to_string(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    " It does not support scaleTable, activeStartTime && activeStopTime parameters."
                ),
            ));

            return Err(err);
        }

        if self.server_version() < MIN_SERVER_VER_ALGO_ID && order.algo_id != "" {
            let err = IBKRApiLibError::ApiError(TwsApiReportableError::new(
                order_id,
                TwsError::UpdateTws.code().to_string(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    " It does not support algoId parameter."
                ),
            ));

            return Err(err);
        }

        if self.server_version() < MIN_SERVER_VER_ORDER_SOLICITED && order.solicited {
            let err = IBKRApiLibError::ApiError(TwsApiReportableError::new(
                order_id,
                TwsError::UpdateTws.code().to_string(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    " It does not support order solicited parameter."
                ),
            ));

            return Err(err);
        }

        if self.server_version() < MIN_SERVER_VER_MODELS_SUPPORT && !order.model_code.is_empty() {
            let err = IBKRApiLibError::ApiError(TwsApiReportableError::new(
                order_id,
                TwsError::UpdateTws.code().to_string(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    " It does not support model code parameter."
                ),
            ));

            return Err(err);
        }

        if self.server_version() < MIN_SERVER_VER_EXT_OPERATOR && !order.ext_operator.is_empty() {
            let err = IBKRApiLibError::ApiError(TwsApiReportableError::new(
                order_id,
                TwsError::UpdateTws.code().to_string(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    " It does not support ext operator parameter."
                ),
            ));

            return Err(err);
        }

        if self.server_version() < MIN_SERVER_VER_SOFT_DOLLAR_TIER
            && (!order.soft_dollar_tier.name.is_empty() || !order.soft_dollar_tier.val.is_empty())
        {
            let err = IBKRApiLibError::ApiError(TwsApiReportableError::new(
                order_id,
                TwsError::UpdateTws.code().to_string(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    " It does not support soft dollar tier."
                ),
            ));

            return Err(err);
        }

        if self.server_version() < MIN_SERVER_VER_CASH_QTY && order.cash_qty != 0.0 {
            let err = IBKRApiLibError::ApiError(TwsApiReportableError::new(
                order_id,
                TwsError::UpdateTws.code().to_string(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    " It does not support cash quantity parameter."
                ),
            ));

            return Err(err);
        }

        if self.server_version() < MIN_SERVER_VER_DECISION_MAKER
            && (!order.mifid2decision_maker.is_empty() || !order.mifid2decision_algo.is_empty())
        {
            let err = IBKRApiLibError::ApiError(TwsApiReportableError::new(
                order_id,
                TwsError::UpdateTws.code().to_string(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    " It does not support MIFID II decision maker parameters."
                ),
            ));

            return Err(err);
        }

        if self.server_version() < MIN_SERVER_VER_MIFID_EXECUTION
            && (!order.mifid2execution_trader.is_empty() || !order.mifid2execution_algo.is_empty())
        {
            let err = IBKRApiLibError::ApiError(TwsApiReportableError::new(
                order_id,
                TwsError::UpdateTws.code().to_string(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    " It does not support MIFID II execution parameters."
                ),
            ));

            return Err(err);
        }

        if self.server_version() < MIN_SERVER_VER_AUTO_PRICE_FOR_HEDGE
            && order.dont_use_auto_price_for_hedge
        {
            let err = IBKRApiLibError::ApiError(TwsApiReportableError::new(
                order_id,
                TwsError::UpdateTws.code().to_string(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    " It does not support dontUseAutoPriceForHedge parameter."
                ),
            ));

            return Err(err);
        }

        if self.server_version() < MIN_SERVER_VER_ORDER_CONTAINER && order.is_oms_container {
            let err = IBKRApiLibError::ApiError(TwsApiReportableError::new(
                order_id,
                TwsError::UpdateTws.code().to_string(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    " It does not support oms container parameter."
                ),
            ));

            return Err(err);
        }

        if self.server_version() < MIN_SERVER_VER_PRICE_MGMT_ALGO && order.use_price_mgmt_algo {
            let err = IBKRApiLibError::ApiError(TwsApiReportableError::new(
                order_id,
                TwsError::UpdateTws.code().to_string(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    " It does not support Use price management algo requests."
                ),
            ));

            return Err(err);
        }

        if self.server_version() < MIN_SERVER_VER_DURATION && order.duration != UNSET_INTEGER {
            let err = IBKRApiLibError::ApiError(TwsApiReportableError::new(
                order_id,
                TwsError::UpdateTws.code().to_string(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    " It does not does not support duration attribute"
                ),
            ));
            return Err(err);
        }

        if self.server_version() < MIN_SERVER_VER_POST_TO_ATS && order.post_to_ats != UNSET_INTEGER
        {
            let err = IBKRApiLibError::ApiError(TwsApiReportableError::new(
                order_id,
                TwsError::UpdateTws.code().to_string(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    "   It does not support postToAts attribute"
                ),
            ));
            return Err(err);
        }

        let version: i32 = if self.server_version() < MIN_SERVER_VER_NOT_HELD {
            27
        } else {
            45
        };

        //send place order msg
        let mut msg = "".to_string();

        let message_id = OutgoingMessageIds::PlaceOrder as i32;

        msg.push_str(&make_field(&message_id)?);

        if self.server_version() < MIN_SERVER_VER_ORDER_CONTAINER {
            msg.push_str(&make_field(&version)?);
        }

        msg.push_str(&make_field(&order_id)?);

        // send contract fields
        if self.server_version() >= MIN_SERVER_VER_PLACE_ORDER_CONID {
            msg.push_str(&make_field(&contract.con_id)?);
        }
        msg.push_str(&make_field(&contract.symbol)?);
        msg.push_str(&make_field(&contract.sec_type)?);
        msg.push_str(&make_field(&contract.last_trade_date_or_contract_month)?);
        msg.push_str(&make_field(&contract.strike)?);
        msg.push_str(&make_field(&contract.right)?);
        msg.push_str(&make_field(&contract.multiplier)?); // srv v15 && above
        msg.push_str(&make_field(&contract.exchange)?);
        msg.push_str(&make_field(&contract.primary_exchange)?); // srv v14 && above
        msg.push_str(&make_field(&contract.currency)?);
        msg.push_str(&make_field(&contract.local_symbol)?); // srv v2 && above

        if self.server_version() >= MIN_SERVER_VER_TRADING_CLASS {
            msg.push_str(&make_field(&contract.trading_class)?);
        }

        if self.server_version() >= MIN_SERVER_VER_SEC_ID_TYPE {
            msg.push_str(&make_field(&contract.sec_id_type)?);
            msg.push_str(&make_field(&contract.sec_id)?);
        }

        // send main order fields
        msg.push_str(&make_field(&order.action)?);

        if self.server_version() >= MIN_SERVER_VER_FRACTIONAL_POSITIONS {
            msg.push_str(&make_field(&order.total_quantity)?);
        } else {
            msg.push_str(&make_field(&(order.total_quantity as i32))?);
        }

        msg.push_str(&make_field(&order.order_type)?);

        if self.server_version() < MIN_SERVER_VER_ORDER_COMBO_LEGS_PRICE {
            msg.push_str(&make_field(if order.lmt_price != UNSET_DOUBLE {
                &order.lmt_price
            } else {
                &0
            })?);
        } else {
            msg.push_str(&make_field_handle_empty(&order.lmt_price)?);
        }

        if self.server_version() < MIN_SERVER_VER_TRAILING_PERCENT {
            msg.push_str(&make_field(if order.aux_price != UNSET_DOUBLE {
                &order.aux_price
            } else {
                &0
            })?);
        } else {
            msg.push_str(&make_field_handle_empty(&order.aux_price)?);
        }

        // send extended order fields
        msg.push_str(&make_field(&order.tif)?);
        msg.push_str(&make_field(&order.oca_group)?);
        msg.push_str(&make_field(&order.account)?);
        msg.push_str(&make_field(&order.open_close)?);
        msg.push_str(&make_field(&(order.origin as i32))?);
        msg.push_str(&make_field(&order.order_ref)?);
        msg.push_str(&make_field(&order.transmit)?);
        msg.push_str(&make_field(&order.parent_id)?); // srv v4 && above
        msg.push_str(&make_field(&order.block_order)?); // srv v5 && above
        msg.push_str(&make_field(&order.sweep_to_fill)?); // srv v5 && above
        msg.push_str(&make_field(&order.display_size)?); // srv v5 && above
        msg.push_str(&make_field(&order.trigger_method)?); // srv v5 && above
        msg.push_str(&make_field(&order.outside_rth)?); // srv v5 && above
        msg.push_str(&make_field(&order.hidden)?); // srv v7 && above

        // Send combo legs for BAG requests (srv v8 && above)
        if contract.sec_type == "BAG" {
            let combo_legs_count = contract.combo_legs.len();
            msg.push_str(&make_field(&combo_legs_count)?);
            if combo_legs_count > 0 {
                for combo_leg in &contract.combo_legs {
                    msg.push_str(&make_field(&combo_leg.con_id)?);
                    msg.push_str(&make_field(&combo_leg.ratio)?);
                    msg.push_str(&make_field(&combo_leg.action)?);
                    msg.push_str(&make_field(&combo_leg.exchange)?);
                    msg.push_str(&make_field(&(combo_leg.open_close as i32))?);
                    msg.push_str(&make_field(&combo_leg.short_sale_slot)?); //srv v35 && above
                    msg.push_str(&make_field(&combo_leg.designated_location)?); // srv v35 && above
                    if self.server_version() >= MIN_SERVER_VER_SSHORTX_OLD {
                        msg.push_str(&make_field(&combo_leg.exempt_code)?);
                    }
                }
            }
        }

        // Send order combo legs for BAG requests
        if self.server_version() >= MIN_SERVER_VER_ORDER_COMBO_LEGS_PRICE
            && contract.sec_type == "BAG"
        {
            let order_combo_legs_count = order.order_combo_legs.len();

            msg.push_str(&make_field(&order_combo_legs_count)?);
            if order_combo_legs_count > 0 {
                for order_combo_leg in &order.order_combo_legs {
                    msg.push_str(&make_field_handle_empty(&order_combo_leg.price)?);
                }
            }
        }

        if self.server_version() >= MIN_SERVER_VER_SMART_COMBO_ROUTING_PARAMS
            && contract.sec_type == "BAG"
        {
            let smart_combo_routing_params_count = order.smart_combo_routing_params.len();
            msg.push_str(&make_field(&smart_combo_routing_params_count)?);
            if smart_combo_routing_params_count > 0 {
                for tag_value in &order.smart_combo_routing_params {
                    msg.push_str(&make_field(&tag_value.tag)?);
                    msg.push_str(&make_field(&tag_value.value)?);
                }
            }
        }

        //    ######################################################################
        //    # Send the shares allocation.
        //    #
        //    # This specifies the number of order shares allocated to each Financial
        //    # Advisor managed account. The format of the allocation string is as
        //    # follows:
        //    # <account_code1>/<number_shares1>,<account_code2>/<number_shares2>,...N
        //    # E.g.
        //    #      To allocate 20 shares of a 100 share order to account 'U101' && the
        //    #      residual 80 to account 'U203' enter the following share allocation string:
        //    #      U101/20,U203/80
        //    #####################################################################

        // send deprecated sharesAllocation field
        msg.push_str(&make_field(&"")?); // srv v9 && above

        msg.push_str(&make_field(&order.discretionary_amt)?); // srv v10 && above
        msg.push_str(&make_field(&order.good_after_time)?); // srv v11 && above
        msg.push_str(&make_field(&order.good_till_date)?); // srv v12 && above

        msg.push_str(&make_field(&order.fa_group)?); // srv v13 && above
        msg.push_str(&make_field(&order.fa_method)?); // srv v13 && above
        msg.push_str(&make_field(&order.fa_percentage)?); // srv v13 && above
        msg.push_str(&make_field(&order.fa_profile)?); // srv v13 && above

        if self.server_version() >= MIN_SERVER_VER_MODELS_SUPPORT {
            msg.push_str(&make_field(&order.model_code)?);
        }

        // institutional short saleslot data (srv v18 && above)
        msg.push_str(&make_field(&order.short_sale_slot)?); // 0 for retail, 1 || 2 for institutions
        msg.push_str(&make_field(&order.designated_location)?); // populate only when shortSaleSlot = 2.

        if self.server_version() >= MIN_SERVER_VER_SSHORTX_OLD {
            msg.push_str(&make_field(&order.exempt_code)?);
        }

        // not needed anymore
        //bool isVolOrder = (order.orderType.CompareNoCase("VOL").as_ref() == 0)

        // srv v19 && above fields
        msg.push_str(&make_field(&order.oca_type)?);
        //if( self.server_version() < 38) {
        // will never happen
        //      send( /* order.rthOnly */ false);
        //}
        msg.push_str(&make_field(&order.rule80a)?);
        msg.push_str(&make_field(&order.settling_firm)?);
        msg.push_str(&make_field(&order.all_or_none)?);
        msg.push_str(&make_field_handle_empty(&order.min_qty)?);
        msg.push_str(&make_field_handle_empty(&order.percent_offset)?);
        msg.push_str(&make_field(&order.e_trade_only)?);
        msg.push_str(&make_field(&order.firm_quote_only)?);
        msg.push_str(&make_field_handle_empty(&order.nbbo_price_cap)?);
        msg.push_str(&make_field(&(order.auction_strategy as i32))?); // AUCTION_MATCH, AUCTION_IMPROVEMENT, AUCTION_TRANSPARENT
        msg.push_str(&make_field_handle_empty(&order.starting_price)?);
        msg.push_str(&make_field_handle_empty(&order.stock_ref_price)?);
        msg.push_str(&make_field_handle_empty(&order.delta)?);
        msg.push_str(&make_field_handle_empty(&order.stock_range_lower)?);
        msg.push_str(&make_field_handle_empty(&order.stock_range_upper)?);

        msg.push_str(&make_field(&order.override_percentage_constraints)?); //srv v22 && above

        // volatility orders (srv v26 && above)
        msg.push_str(&make_field_handle_empty(&order.volatility)?);
        msg.push_str(&make_field_handle_empty(&order.volatility_type)?);
        msg.push_str(&make_field(&order.delta_neutral_order_type)?); // srv v28 && above
        msg.push_str(&make_field_handle_empty(&order.delta_neutral_aux_price)?); // srv v28 && above

        if self.server_version() >= MIN_SERVER_VER_DELTA_NEUTRAL_CONID
            && !order.delta_neutral_order_type.is_empty()
        {
            msg.push_str(&make_field(&order.delta_neutral_con_id)?);
            msg.push_str(&make_field(&order.delta_neutral_settling_firm)?);
            msg.push_str(&make_field(&order.delta_neutral_clearing_account)?);
            msg.push_str(&make_field(&order.delta_neutral_clearing_intent)?);
        }

        if self.server_version() >= MIN_SERVER_VER_DELTA_NEUTRAL_OPEN_CLOSE
            && order.delta_neutral_order_type != ""
        {
            msg.push_str(&make_field(&order.delta_neutral_open_close)?);
            msg.push_str(&make_field(&order.delta_neutral_short_sale)?);
            msg.push_str(&make_field(&order.delta_neutral_short_sale_slot)?);
            msg.push_str(&make_field(&order.delta_neutral_designated_location)?);
        }

        msg.push_str(&make_field(&order.continuous_update)?);
        msg.push_str(&make_field_handle_empty(&order.reference_price_type)?);
        msg.push_str(&make_field_handle_empty(&order.trail_stop_price)?); // srv v30 && above

        if self.server_version() >= MIN_SERVER_VER_TRAILING_PERCENT {
            msg.push_str(&make_field_handle_empty(&order.trailing_percent)?);
        }

        // SCALE orders
        if self.server_version() >= MIN_SERVER_VER_SCALE_ORDERS2 {
            msg.push_str(&make_field_handle_empty(&order.scale_init_level_size)?);
            msg.push_str(&make_field_handle_empty(&order.scale_subs_level_size)?);
        } else {
            // srv v35 && above)
            msg.push_str(&make_field(&"")?); // for not supported scaleNumComponents
            msg.push_str(&make_field_handle_empty(&order.scale_init_level_size)?);
            // for scaleComponentSize
        }

        msg.push_str(&make_field_handle_empty(&order.scale_price_increment)?);

        if self.server_version() >= MIN_SERVER_VER_SCALE_ORDERS3
            && order.scale_price_increment != UNSET_DOUBLE
            && order.scale_price_increment > 0.0
        {
            msg.push_str(&make_field_handle_empty(&order.scale_price_adjust_value)?);
            msg.push_str(&make_field_handle_empty(
                &order.scale_price_adjust_interval,
            )?);
            msg.push_str(&make_field_handle_empty(&order.scale_profit_offset)?);
            msg.push_str(&make_field(&order.scale_auto_reset)?);
            msg.push_str(&make_field_handle_empty(&order.scale_init_position)?);
            msg.push_str(&make_field_handle_empty(&order.scale_init_fill_qty)?);
            msg.push_str(&make_field(&order.scale_random_percent)?);
        }

        if self.server_version() >= MIN_SERVER_VER_SCALE_TABLE {
            msg.push_str(&make_field(&order.scale_table)?);
            msg.push_str(&make_field(&order.active_start_time)?);
            msg.push_str(&make_field(&order.active_stop_time)?);
        }

        // HEDGE orders
        if self.server_version() >= MIN_SERVER_VER_HEDGE_ORDERS {
            msg.push_str(&make_field(&order.hedge_type)?);

            if !order.hedge_type.is_empty() {
                msg.push_str(&make_field(&order.hedge_param)?);
            }
        }

        if self.server_version() >= MIN_SERVER_VER_OPT_OUT_SMART_ROUTING {
            msg.push_str(&make_field(&order.opt_out_smart_routing)?);
        }

        if self.server_version() >= MIN_SERVER_VER_PTA_ORDERS {
            msg.push_str(&make_field(&order.clearing_account)?);
            msg.push_str(&make_field(&order.clearing_intent)?);
        }

        if self.server_version() >= MIN_SERVER_VER_NOT_HELD {
            msg.push_str(&make_field(&order.not_held)?);
        }

        if self.server_version() >= MIN_SERVER_VER_DELTA_NEUTRAL {
            if contract.delta_neutral_contract.is_some() {
                msg.push_str(&make_field(&true)?);
                msg.push_str(&make_field(
                    &contract.delta_neutral_contract.as_ref().unwrap().con_id,
                )?);
                msg.push_str(&make_field(
                    &contract.delta_neutral_contract.as_ref().unwrap().delta,
                )?);
                msg.push_str(&make_field(
                    &contract.delta_neutral_contract.as_ref().unwrap().price,
                )?);
            } else {
                msg.push_str(&make_field(&false)?);
            }
        }

        if self.server_version() >= MIN_SERVER_VER_ALGO_ORDERS {
            msg.push_str(&make_field(&order.algo_strategy)?);
            if !order.algo_strategy.is_empty() {
                let algo_params_count = order.algo_params.len();
                msg.push_str(&make_field(&algo_params_count)?);
                if algo_params_count > 0 {
                    for algo_param in &order.algo_params {
                        msg.push_str(&make_field(&algo_param.tag)?);
                        msg.push_str(&make_field(&algo_param.value)?);
                    }
                }
            }
        }

        if self.server_version() >= MIN_SERVER_VER_ALGO_ID {
            msg.push_str(&make_field(&order.algo_id)?);
        }

        msg.push_str(&make_field(&order.what_if)?); // srv v36 && above

        // send miscOptions parameter
        if self.server_version() >= MIN_SERVER_VER_LINKING {
            let misc_options_str = order
                .order_misc_options
                .iter()
                .map(|x| format!("{}={};", x.tag, x.value))
                .collect::<String>();
            msg.push_str(&make_field(&misc_options_str)?);
        }

        if self.server_version() >= MIN_SERVER_VER_ORDER_SOLICITED {
            msg.push_str(&make_field(&order.solicited)?);
        }

        if self.server_version() >= MIN_SERVER_VER_RANDOMIZE_SIZE_AND_PRICE {
            msg.push_str(&make_field(&order.randomize_size)?);
            msg.push_str(&make_field(&order.randomize_price)?);
        }

        if self.server_version() >= MIN_SERVER_VER_PEGGED_TO_BENCHMARK {
            if order.order_type == "PEG BENCH" {
                msg.push_str(&make_field(&order.reference_contract_id)?);
                msg.push_str(&make_field(&order.is_pegged_change_amount_decrease)?);
                msg.push_str(&make_field(&order.pegged_change_amount)?);
                msg.push_str(&make_field(&order.reference_change_amount)?);
                msg.push_str(&make_field(&order.reference_exchange_id)?);
            }

            msg.push_str(&make_field(&order.conditions.len())?);

            if order.conditions.len() > 0 {
                for cond in &order.conditions {
                    msg.push_str(&make_field(&(cond.get_type() as i32))?);
                    let vals = cond.make_fields()?;
                    let vals_string = vals.iter().map(|val| val.clone()).collect::<String>();
                    msg.push_str(vals_string.as_ref());
                }

                msg.push_str(&make_field(&order.conditions_ignore_rth)?);
                msg.push_str(&make_field(&order.conditions_cancel_order)?);
            }

            msg.push_str(&make_field(&order.adjusted_order_type)?);
            msg.push_str(&make_field(&order.trigger_price)?);
            msg.push_str(&make_field(&order.lmt_price_offset)?);
            msg.push_str(&make_field(&order.adjusted_stop_price)?);
            msg.push_str(&make_field(&order.adjusted_stop_limit_price)?);
            msg.push_str(&make_field(&order.adjusted_trailing_amount)?);
            msg.push_str(&make_field(&order.adjustable_trailing_unit)?);
        }

        if self.server_version() >= MIN_SERVER_VER_EXT_OPERATOR {
            msg.push_str(&make_field(&order.ext_operator)?);
        }

        if self.server_version() >= MIN_SERVER_VER_SOFT_DOLLAR_TIER {
            msg.push_str(&make_field(&order.soft_dollar_tier.name)?);
            msg.push_str(&make_field(&order.soft_dollar_tier.val)?);
        }

        if self.server_version() >= MIN_SERVER_VER_CASH_QTY {
            msg.push_str(&make_field(&order.cash_qty)?);
        }

        if self.server_version() >= MIN_SERVER_VER_DECISION_MAKER {
            msg.push_str(&make_field(&order.mifid2decision_maker)?);
            msg.push_str(&make_field(&order.mifid2decision_algo)?);
        }

        if self.server_version() >= MIN_SERVER_VER_MIFID_EXECUTION {
            msg.push_str(&make_field(&order.mifid2execution_trader)?);
            msg.push_str(&make_field(&order.mifid2execution_algo)?);
        }

        if self.server_version() >= MIN_SERVER_VER_AUTO_PRICE_FOR_HEDGE {
            msg.push_str(&make_field(&order.dont_use_auto_price_for_hedge)?);
        }

        if self.server_version() >= MIN_SERVER_VER_ORDER_CONTAINER {
            msg.push_str(&make_field(&order.is_oms_container)?);
        }

        if self.server_version() >= MIN_SERVER_VER_D_PEG_ORDERS {
            msg.push_str(&make_field(&order.discretionary_up_to_limit_price)?);
        }

        if self.server_version() >= MIN_SERVER_VER_PRICE_MGMT_ALGO {
            msg.push_str(&make_field_handle_empty(&order.use_price_mgmt_algo)?);
        }

        if self.server_version() >= MIN_SERVER_VER_DURATION {
            msg.push_str(&make_field(&order.duration)?);
        }

        if self.server_version() >= MIN_SERVER_VER_POST_TO_ATS {
            msg.push_str(&make_field(&order.post_to_ats)?);
        }

        self.send_request(msg.as_str())?;
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    /// Call this function to cancel an order.
    /// # Arguments
    /// * order_id - The order ID that was specified previously when placing the order
    pub fn cancel_order(&mut self, order_id: i32) -> Result<(), IBKRApiLibError> {
        self.check_connected(NO_VALID_ID)?;

        let version = 2;

        let mut msg = "".to_string();

        let message_id = OutgoingMessageIds::CancelOrder as i32;

        msg.push_str(&make_field(&message_id)?);
        msg.push_str(&make_field(&version)?);
        msg.push_str(&make_field(&order_id)?);

        self.send_request(msg.as_str())?;
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    /// Call this function to request the open orders that were
    /// placed from this client. Each open order will be fed back through the
    /// open_order() and order_status() functions on the Wrapper.
    ///
    /// Note:  The client with a client_id of 0 will also receive the TWS-owned
    ///        open orders. These orders will be associated with the client and a new
    ///        order_id will be generated. This association will persist over multiple
    ///        API and TWS sessions
    pub fn req_open_orders(&mut self) -> Result<(), IBKRApiLibError> {
        self.check_connected(NO_VALID_ID)?;

        let version = 1;

        let mut msg = "".to_string();

        let message_id = OutgoingMessageIds::ReqOpenOrders as i32;

        msg.push_str(&make_field(&message_id)?);
        msg.push_str(&make_field(&version)?);

        self.send_request(msg.as_str())?;
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    /// Call this function to request that newly created TWS orders
    /// be implicitly associated with the client.When a new TWS order is
    /// created, the order will be associated with the client, and fed back
    /// through the openOrder() and orderStatus() functions on the EWrapper.
    ///
    /// Note: This request can only be made from a client with client_id of 0.
    ///
    /// # Arguments
    /// * b_auto_bind - If set to TRUE, newly created TWS orders will be implicitly
    ///                 associated with the client.If set to FALSE, no association will be
    ///                 made.
    pub fn req_auto_open_orders(&mut self, b_auto_bind: bool) -> Result<(), IBKRApiLibError> {
        self.check_connected(NO_VALID_ID)?;

        let version = 1;

        let mut msg = "".to_string();

        let message_id = OutgoingMessageIds::ReqAutoOpenOrders as i32;

        msg.push_str(&make_field(&message_id)?);
        msg.push_str(&make_field(&version)?);
        msg.push_str(&make_field(&b_auto_bind)?); // TRUE = subscribe, FALSE = unsubscribe

        self.send_request(msg.as_str())?;

        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    /// Call this function to request the open orders placed from all
    /// clients and also from TWS. Each open order will be fed back through the
    /// open_order() and order_status() functions on the EWrapper.
    /// Note:  No association is made between the returned orders and the
    /// requesting client.
    pub fn req_all_open_orders(&mut self) -> Result<(), IBKRApiLibError> {
        self.check_connected(NO_VALID_ID)?;

        let version = 1;

        let mut msg = "".to_string();

        let message_id = OutgoingMessageIds::ReqAllOpenOrders as i32;

        msg.push_str(&make_field(&message_id)?);
        msg.push_str(&make_field(&version)?);

        self.send_request(msg.as_str())?;

        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    /// Use this function to cancel all open orders globally. It
    /// cancels both API and TWS open orders.
    /// If the order was created in TWS, it also gets canceled. If the order
    /// was initiated in the API client, it also gets canceled.
    pub fn req_global_cancel(&mut self) -> Result<(), IBKRApiLibError> {
        self.check_connected(NO_VALID_ID)?;

        let version = 1;

        let mut msg = "".to_string();

        let message_id = OutgoingMessageIds::ReqGlobalCancel as i32;

        msg.push_str(&make_field(&message_id)?);
        msg.push_str(&make_field(&version)?);

        self.send_request(msg.as_str())?;

        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    /// Call this function to request from TWS the next valid ID that
    /// can be used when placing an order.  After calling this function, the
    /// next_valid_id() Wrapper event will be triggered, and the id returned is the next
    /// valid ID. That ID will reflect any autobinding that has occurred (which
    /// generates new IDs and increments the next valid ID therein).
    ///
    /// # Arguments
    /// * num_ids - deprecated
    pub fn req_ids(&mut self, num_ids: i32) -> Result<(), IBKRApiLibError> {
        self.check_connected(NO_VALID_ID)?;
        info!("req_ids is connected...");
        let version = 1;

        let mut msg = "".to_string();

        let message_id = OutgoingMessageIds::ReqIds as i32;

        msg.push_str(&make_field(&message_id)?);
        msg.push_str(&make_field(&version)?);
        msg.push_str(&make_field(&num_ids)?);
        info!("req_ids... sending request...");
        self.send_request(msg.as_str())?;
        Ok(())
    }

    //#########################################################################
    //################## Account and Portfolio
    //#########################################################################
    /// Call this function to start getting account values, portfolio,
    /// and last update time information via Wrapper.update_account_value());
    /// Wrapper.update_portfolio() and Wrapper.update_account_time().
    ///
    ///
    /// # Arguments
    /// * subscribe - If set to TRUE, the client will start receiving account
    ///               and Portfoliolio updates. If set to FALSE, the client will stop
    ///               receiving this information.
    /// * acct_code - The account code for which to receive account and portfolio updates.
    pub fn req_account_updates(
        &mut self,
        subscribe: bool,
        acct_code: &str,
    ) -> Result<(), IBKRApiLibError> {
        info!("subscribe: {}, acct_code: {}", subscribe, acct_code);
        debug!(
            "req_account_updates: subscribe: {}, acct_code: {}",
            subscribe, acct_code
        );

        self.check_connected(NO_VALID_ID)?;

        let version = 2;

        let mut msg = "".to_string();

        let message_id = OutgoingMessageIds::ReqAcctData as i32;

        msg.push_str(&make_field(&message_id)?);
        msg.push_str(&make_field(&version)?);
        msg.push_str(&make_field(&subscribe)?); // TRUE = subscribe, FALSE = unsubscribe
        msg.push_str(&make_field(&String::from(acct_code))?); // srv v9 and above, the account code.This will only be used for FA clients

        self.send_request(msg.as_str())?;

        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    /// Requests a specific account's summary.
    /// This method will subscribe to the account summary as presented in the TWS' Account Summary tab.
    /// The data is returned at Wrapper::account_summary
    /// https://www.interactivebrokers.com/en/software/tws/accountwindowtop.htm.
    /// Note:   This request is designed for an FA managed account but can be
    ///         used for any multi-account structure.
    ///
    /// # Arguments
    /// * req_id - The ID of the data request. Ensures that responses are matched
    ///            to requests If several requests are in process.
    /// * group_name - Set to All to return account summary data for all
    ///                accounts, or set to a specific Advisor Account Group name that has
    ///                already been created in TWS Global Configuration.
    /// * tags- A comma-separated list of account tags.  See the AccountSummaryTags enum for valid values
    pub fn req_account_summary(
        &mut self,
        req_id: i32,
        group_name: &str,
        tags: &str,
    ) -> Result<(), IBKRApiLibError> {
        self.check_connected(NO_VALID_ID)?;

        let version = 2;

        let message_id: i32 = OutgoingMessageIds::ReqAccountSummary as i32;
        let mut msg = "".to_string();

        msg.push_str(&make_field(&message_id)?);
        msg.push_str(&make_field(&version)?);
        msg.push_str(&make_field(&req_id)?);
        msg.push_str(&make_field(&String::from(group_name))?);
        msg.push_str(&make_field(&String::from(tags))?);

        self.send_request(msg.as_str())?;
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    /// Cancels the request for Account Window Summary tab data.
    ///
    /// # Arguments
    /// * req_id - The ID of the data request being canceled.
    pub fn cancel_account_summary(&mut self, req_id: i32) -> Result<(), IBKRApiLibError> {
        self.check_connected(req_id)?;
        let version = 1;

        let message_id: i32 = OutgoingMessageIds::CancelAccountSummary as i32;
        let mut msg = "".to_string();
        msg.push_str(&make_field(&message_id)?);
        msg.push_str(&make_field(&version)?);
        msg.push_str(&make_field(&req_id)?);

        self.send_request(msg.as_str())?;

        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    /// Requests real-time position data for all accounts.
    pub fn req_positions(&mut self) -> Result<(), IBKRApiLibError> {
        self.check_connected(NO_VALID_ID)?;

        if self.server_version() < MIN_SERVER_VER_POSITIONS {
            let err = IBKRApiLibError::ApiError(TwsApiReportableError::new(
                NO_VALID_ID,
                TwsError::UpdateTws.code().to_string(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    " It does not support positions request."
                ),
            ));

            return Err(err);
        }

        let version = 1;

        let message_id: i32 = OutgoingMessageIds::ReqPositions as i32;
        let mut msg = "".to_string();
        msg.push_str(&make_field(&message_id)?);
        msg.push_str(&make_field(&version)?);

        self.send_request(msg.as_str())?;

        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    /// Cancels real-time position updates.
    pub fn cancel_positions(&mut self) -> Result<(), IBKRApiLibError> {
        self.check_connected(NO_VALID_ID)?;

        if self.server_version() < MIN_SERVER_VER_POSITIONS {
            let err = IBKRApiLibError::ApiError(TwsApiReportableError::new(
                NO_VALID_ID,
                TwsError::UpdateTws.code().to_string(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    " It does not support positions request."
                ),
            ));

            return Err(err);
        }

        let version = 1;

        let message_id: i32 = OutgoingMessageIds::CancelPositions as i32;
        let mut msg = "".to_string();
        msg.push_str(&make_field(&message_id)?);
        msg.push_str(&make_field(&version)?);
        self.send_request(msg.as_str())?;

        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    /// Requests position subscription for account and/or model Initially all positions are returned,
    /// and then updates are returned for any position changes in real time.
    ///
    /// # Arguments
    /// * req_id - Request's identifier
    /// * account - If an account Id is provided, only the account's positions belonging to the
    ///             specified model will be delivered
    /// * modelCode	- The code of the model's positions we are interested in.
    pub fn req_positions_multi(
        &mut self,
        req_id: i32,
        account: &str,
        model_code: &str,
    ) -> Result<(), IBKRApiLibError> {
        self.check_connected(req_id)?;

        if self.server_version() < MIN_SERVER_VER_POSITIONS {
            let err = IBKRApiLibError::ApiError(TwsApiReportableError::new(
                req_id,
                TwsError::UpdateTws.code().to_string(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    " It does not support positions multi request."
                ),
            ));

            return Err(err);
        }

        let version = 1;
        let mut_req_id = req_id;
        let mut_account = account;
        let mut_model_code = model_code;
        let message_id: i32 = OutgoingMessageIds::ReqPositionsMulti as i32;
        let mut msg = "".to_string();
        msg.push_str(&make_field(&message_id)?);
        msg.push_str(&make_field(&version)?);
        msg.push_str(&make_field(&mut_req_id)?);
        msg.push_str(&make_field(&String::from(mut_account))?);
        msg.push_str(&make_field(&String::from(mut_model_code))?);

        self.send_request(msg.as_str())?;

        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    /// Cancels positions request for account and/or model.
    ///
    /// # Arguments
    /// * req_id - The id of the original request
    pub fn cancel_positions_multi(&mut self, req_id: i32) -> Result<(), IBKRApiLibError> {
        self.check_connected(req_id)?;

        if self.server_version() < MIN_SERVER_VER_POSITIONS {
            let err = IBKRApiLibError::ApiError(TwsApiReportableError::new(
                req_id,
                TwsError::UpdateTws.code().to_string(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    " It does not support positions multi request."
                ),
            ));

            return Err(err);
        }

        let version = 1;
        let mut_req_id = req_id;
        let message_id: i32 = OutgoingMessageIds::CancelPositionsMulti as i32;
        let mut msg = "".to_string();
        msg.push_str(&make_field(&message_id)?);
        msg.push_str(&make_field(&version)?);
        msg.push_str(&make_field(&mut_req_id)?);

        self.send_request(msg.as_str())?;
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    /// Requests account updates for account and/or model.
    ///
    /// # Arguments
    /// * req_id - identifier to tag the request
    /// * account - account values can be requested for a particular account
    /// * model_code - values can also be requested for a model
    /// * ledger_and_nvl - returns light-weight request; only currency positions as opposed to
    ///                    account values and currency positions
    pub fn req_account_updates_multi(
        &mut self,
        req_id: i32,
        account: &str,
        model_code: &str,
        ledger_and_nlv: bool,
    ) -> Result<(), IBKRApiLibError> {
        self.check_connected(req_id)?;

        if self.server_version() < MIN_SERVER_VER_MODELS_SUPPORT {
            let err = IBKRApiLibError::ApiError(TwsApiReportableError::new(
                req_id,
                TwsError::UpdateTws.code().to_string(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    " It does not support account updates multi request."
                ),
            ));

            return Err(err);
        }

        let version = 1;
        let mut_req_id = req_id;
        let mut_account = account;
        let mut_model_code = model_code;
        let mut_ledger_and_nlv = ledger_and_nlv;

        let message_id: i32 = OutgoingMessageIds::ReqAccountUpdatesMulti as i32;
        let mut msg = "".to_string();
        msg.push_str(&make_field(&message_id)?);
        msg.push_str(&make_field(&version)?);
        msg.push_str(&make_field(&mut_req_id)?);
        msg.push_str(&make_field(&String::from(mut_account))?);
        msg.push_str(&make_field(&String::from(mut_model_code))?);
        msg.push_str(&make_field(&mut_ledger_and_nlv)?);

        self.send_request(msg.as_str())?;

        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    /// Cancels account update request for account and/or model.
    ///
    /// # Arguments
    /// * req_id - The id of the original request
    pub fn cancel_account_updates_multi(&mut self, req_id: i32) -> Result<(), IBKRApiLibError> {
        self.check_connected(req_id)?;

        if self.server_version() < MIN_SERVER_VER_MODELS_SUPPORT {
            let err = IBKRApiLibError::ApiError(TwsApiReportableError::new(
                req_id,
                TwsError::UpdateTws.code().to_string(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    " It does not support account updates multi request."
                ),
            ));

            return Err(err);
        }

        let version = 1;
        let mut_req_id = req_id;
        let message_id: i32 = OutgoingMessageIds::CancelAccountUpdatesMulti as i32;
        let mut msg = "".to_string();
        msg.push_str(&make_field(&message_id)?);
        msg.push_str(&make_field(&version)?);
        msg.push_str(&make_field(&mut_req_id)?);

        self.send_request(msg.as_str())
    }

    //#########################################################################
    //################## Daily PnL
    //#########################################################################
    /// Requests profit and loss for account and/or model.
    ///
    /// # Arguments
    /// * req_id - identifier to tag the request
    /// * account - account values can be requested for a particular account
    /// * model_code - values can also be requested for a model
    pub fn req_pnl(
        &mut self,
        req_id: i32,
        account: &str,
        model_code: &str,
    ) -> Result<(), IBKRApiLibError> {
        self.check_connected(req_id)?;

        if self.server_version() < MIN_SERVER_VER_PNL {
            let err = IBKRApiLibError::ApiError(TwsApiReportableError::new(
                req_id,
                TwsError::UpdateTws.code().to_string(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    " It does not support PnL request."
                ),
            ));

            return Err(err);
        }

        let message_id: i32 = OutgoingMessageIds::ReqPnl as i32;
        let mut msg = "".to_string();
        msg.push_str(&make_field(&message_id)?);
        msg.push_str(&make_field(&req_id)?);
        msg.push_str(&make_field(&String::from(account))?);
        msg.push_str(&make_field(&String::from(model_code))?);

        self.send_request(msg.as_str())
    }

    //----------------------------------------------------------------------------------------------
    /// Cancels profit and loss update request for account and/or model.
    ///
    /// # Arguments
    /// * req_id - The id of the original request
    pub fn cancel_pnl(&mut self, req_id: i32) -> Result<(), IBKRApiLibError> {
        self.check_connected(req_id)?;

        if self.server_version() < MIN_SERVER_VER_PNL {
            let err = IBKRApiLibError::ApiError(TwsApiReportableError::new(
                req_id,
                TwsError::UpdateTws.code().to_string(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    " It does not support PnL request."
                ),
            ));

            return Err(err);
        }

        let message_id: i32 = OutgoingMessageIds::CancelPnl as i32;
        let mut msg = "".to_string();
        msg.push_str(&make_field(&message_id)?);
        msg.push_str(&make_field(&req_id)?);

        self.send_request(msg.as_str())
    }

    //----------------------------------------------------------------------------------------------
    /// Requests profit and loss for account and/or model for a specific contract.
    ///
    /// # Arguments
    /// * req_id - identifier to tag the request
    /// * account - account values can be requested for a particular account
    /// * model_code - values can also be requested for a model
    /// * con_id - contract id of the specific contact of interest
    pub fn req_pnl_single(
        &mut self,
        req_id: i32,
        account: &str,
        model_code: &str,
        con_id: i32,
    ) -> Result<(), IBKRApiLibError> {
        self.check_connected(req_id)?;

        if self.server_version() < MIN_SERVER_VER_PNL {
            let err = IBKRApiLibError::ApiError(TwsApiReportableError::new(
                req_id,
                TwsError::UpdateTws.code().to_string(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    " It does not support PnL request."
                ),
            ));

            return Err(err);
        }

        let message_id: i32 = OutgoingMessageIds::ReqPnlSingle as i32;
        let mut msg = "".to_string();
        msg.push_str(&make_field(&message_id)?);
        msg.push_str(&make_field(&req_id)?);
        msg.push_str(&make_field(&String::from(account))?);
        msg.push_str(&make_field(&String::from(model_code))?);
        msg.push_str(&make_field(&con_id)?);

        self.send_request(msg.as_str())
    }

    //----------------------------------------------------------------------------------------------
    /// Cancels profit and loss update request for account and/or model.
    ///
    /// # Arguments
    /// * req_id - The id of the original request
    pub fn cancel_pnl_single(&mut self, req_id: i32) -> Result<(), IBKRApiLibError> {
        self.check_connected(req_id)?;

        if self.server_version() < MIN_SERVER_VER_PNL {
            let err = IBKRApiLibError::ApiError(TwsApiReportableError::new(
                req_id,
                TwsError::UpdateTws.code().to_string(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    " It does not support PnL request."
                ),
            ));

            return Err(err);
        }

        let message_id: i32 = OutgoingMessageIds::CancelPnlSingle as i32;
        let mut msg = "".to_string();
        msg.push_str(&make_field(&message_id)?);
        msg.push_str(&make_field(&req_id)?);

        self.send_request(msg.as_str())
    }

    //#########################################################################
    //################## Executions
    //#########################################################################

    /// When this function is called, the execution reports that meet the
    /// filter criteria are downloaded to the client via the execDetails()
    /// function. To view executions beyond the past 24 hours, open the
    /// Trade Log in TWS and, while the Trade Log is displayed, request
    /// the executions again from the API.
    ///
    /// # Arguments
    /// * req_id - The ID of the data request. Ensures that responses are
    ///            matched to requests if several requests are in process.
    /// * exec_filter - This object contains attributes that
    ///                 describe the filter criteria used to determine which execution
    ///                 reports are returned.
    ///
    /// NOTE: Time format must be 'yyyymmdd-hh:mm:ss' Eg: '20030702-14:55'
    pub fn req_executions(
        &mut self,
        req_id: i32,
        exec_filter: &ExecutionFilter,
    ) -> Result<(), IBKRApiLibError> {
        self.check_connected(req_id)?;

        let version = 3;
        let message_id: i32 = OutgoingMessageIds::ReqExecutions as i32;
        let mut msg = "".to_string();
        msg.push_str(&make_field(&message_id)?);
        msg.push_str(&make_field(&version)?);
        if self.server_version() >= MIN_SERVER_VER_EXECUTION_DATA_CHAIN {
            msg.push_str(&make_field(&req_id)?);
        }
        msg.push_str(&make_field(&exec_filter.client_id)?);
        msg.push_str(&make_field(&exec_filter.acct_code)?);
        msg.push_str(&make_field(&exec_filter.time)?);
        msg.push_str(&make_field(&exec_filter.symbol)?);
        msg.push_str(&make_field(&exec_filter.sec_type)?);
        msg.push_str(&make_field(&exec_filter.exchange)?);
        msg.push_str(&make_field(&exec_filter.side)?);

        self.send_request(msg.as_str())
    }

    //#########################################################################
    //################## Contract Details
    //#########################################################################
    /// Call this function to download all details for a particular
    /// underlying. The contract details will be received via the contractDetails()
    /// function on the EWrapper.
    ///
    ///    
    /// # Arguments
    /// * req_id - The ID of the data request. Ensures that responses are
    ///            matched to requests if several requests are in process.
    /// * contract - The summary description of the contract being looked up.
    pub fn req_contract_details(
        &mut self,
        req_id: i32,
        contract: &Contract,
    ) -> Result<(), IBKRApiLibError> {
        self.check_connected(req_id)?;

        if self.server_version() < MIN_SERVER_VER_SEC_ID_TYPE {
            if contract.sec_id_type != "" || contract.sec_id != "" {
                let err = IBKRApiLibError::ApiError(TwsApiReportableError::new(
                    req_id,
                    TwsError::UpdateTws.code().to_string(),
                    format!(
                        "{}{}",
                        TwsError::UpdateTws.message(),
                        " It does not support sec_id_type and secId parameters."
                    ),
                ));

                return Err(err);
            }
        }

        if self.server_version() < MIN_SERVER_VER_TRADING_CLASS {
            if contract.trading_class != "" {
                let err = IBKRApiLibError::ApiError(TwsApiReportableError::new(
                    req_id,
                    TwsError::UpdateTws.code().to_string(),
                    format!(
                        "{}{}",
                        TwsError::UpdateTws.message(),
                        " It does not support trading_class parameter in req_contract_details."
                    ),
                ));

                return Err(err);
            }
        }

        if self.server_version() < MIN_SERVER_VER_LINKING {
            if contract.primary_exchange != "" {
                let err = IBKRApiLibError::ApiError(TwsApiReportableError::new(
                    req_id,
                    TwsError::UpdateTws.code().to_string(),
                    format!(
                        "{}{}",
                        TwsError::UpdateTws.message(),
                        " It does not support primary_exchange parameter in req_contract_details."
                    ),
                ));

                return Err(err);
            }
        }

        let version = 8;

        let message_id: i32 = OutgoingMessageIds::ReqContractData as i32;
        let mut msg = "".to_string();
        msg.push_str(&make_field(&message_id)?);
        msg.push_str(&make_field(&version)?);

        if self.server_version() >= MIN_SERVER_VER_CONTRACT_DATA_CHAIN {
            msg.push_str(&make_field(&req_id)?);
        }

        // send contract fields
        msg.push_str(&make_field(&contract.con_id)?); // srv v37 and above
        msg.push_str(&make_field(&contract.symbol)?);

        msg.push_str(&make_field(&contract.sec_type)?);
        msg.push_str(&make_field(&contract.last_trade_date_or_contract_month)?);
        msg.push_str(&make_field(&contract.strike)?);
        msg.push_str(&make_field(&contract.right)?);
        msg.push_str(&make_field(&contract.multiplier)?); // srv v15 and above

        if self.server_version() >= MIN_SERVER_VER_PRIMARYEXCH {
            msg.push_str(&make_field(&contract.exchange)?);
            msg.push_str(&make_field(&contract.primary_exchange)?);
        } else if self.server_version() >= MIN_SERVER_VER_LINKING {
            if contract.primary_exchange != ""
                && (contract.exchange == "BEST" || contract.exchange == "SMART")
            {
                msg.push_str(&make_field(&format!(
                    "{}:{}",
                    &contract.exchange, &contract.primary_exchange
                ))?);
            }
        } else {
            msg.push_str(&make_field(&contract.exchange)?);
        }

        msg.push_str(&make_field(&contract.currency)?);
        msg.push_str(&make_field(&contract.local_symbol)?);

        if self.server_version() >= MIN_SERVER_VER_TRADING_CLASS {
            msg.push_str(&make_field(&contract.trading_class)?);
            msg.push_str(&make_field(&contract.include_expired)?); // srv v31 and above
        }

        if self.server_version() >= MIN_SERVER_VER_SEC_ID_TYPE {
            msg.push_str(&make_field(&contract.sec_id_type)?);
            msg.push_str(&make_field(&contract.sec_id)?);
        }

        self.send_request(msg.as_str())
    }

    //#########################################################################
    //################## Market Depth
    //#########################################################################
    /// Requests venues for which market data is returned to update_mkt_depth_l2 (those with market makers)
    pub fn req_mkt_depth_exchanges(&mut self) -> Result<(), IBKRApiLibError> {
        self.check_connected(NO_VALID_ID)?;

        if self.server_version() < MIN_SERVER_VER_REQ_MKT_DEPTH_EXCHANGES {
            let err = IBKRApiLibError::ApiError(TwsApiReportableError::new(
                NO_VALID_ID,
                TwsError::UpdateTws.code().to_string(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    " It does not support market depth exchanges request."
                ),
            ));

            return Err(err);
        }

        let message_id: i32 = OutgoingMessageIds::ReqMktDepthExchanges as i32;
        let mut msg = "".to_string();
        msg.push_str(&make_field(&message_id)?);

        self.send_request(msg.as_str())
    }

    //----------------------------------------------------------------------------------------------
    /// Call this function to request market depth for a specific
    /// contract. The market depth will be returned by the update_mkt_depth() and
    /// update_mkt_depth_l2() events.
    ///
    /// Requests the contract's market depth (order book). Note this request must be
    /// direct-routed to an exchange and not smart-routed. The number of simultaneous
    /// market depth requests allowed in an account is calculated based on a formula
    /// that looks at an accounts equity, commissions, and quote booster packs.
    ///
    /// # Arguments
    /// * req_id - The request id. Must be a unique value. When the market
    ///            depth data returns, it will be identified by this tag. This is
    ///            also used when canceling the market depth
    /// * contract - This structure contains a description of the contract
    ///              for which market depth data is being requested.
    /// * num_rows - Specifies the number of rows of market depth rows to display.
    /// * is_smart_depth - specifies SMART depth request  NOTE: ALWAYS SET TO FALSE!!!!!
    ///                    THERE SEEMS TO BE A BUG ON IB's SIDE AND THEY WILL STOP STREAMING
    ///                    DATA IF THIS IS SET TO TRUE
    /// * mkt_depth_options - For internal use only. Use default value XYZ.
    pub fn req_mkt_depth(
        &mut self,
        req_id: i32,
        contract: &Contract,
        num_rows: i32,
        is_smart_depth: bool,
        mkt_depth_options: Vec<TagValue>,
    ) -> Result<(), IBKRApiLibError> {
        self.check_connected(NO_VALID_ID)?;

        if self.server_version() < MIN_SERVER_VER_TRADING_CLASS {
            if &contract.trading_class != "" || *&contract.con_id > 0 {
                let err = IBKRApiLibError::ApiError(TwsApiReportableError::new(
                    req_id,
                    TwsError::UpdateTws.code().to_string(),
                    format!(
                        "{}{}",
                        TwsError::UpdateTws.message(),
                        " It does not support con_id and trading_class parameters in req_mkt_depth."
                    ),
                ));

                return Err(err);
            }
        }

        if self.server_version() < MIN_SERVER_VER_SMART_DEPTH && is_smart_depth {
            let err = IBKRApiLibError::ApiError(TwsApiReportableError::new(
                req_id,
                TwsError::UpdateTws.code().to_string(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    " It does not support SMART depth request."
                ),
            ));

            return Err(err);
        }

        if self.server_version() < MIN_SERVER_VER_MKT_DEPTH_PRIM_EXCHANGE
            && contract.primary_exchange != ""
        {
            let err = IBKRApiLibError::ApiError(TwsApiReportableError::new(
                req_id,
                TwsError::UpdateTws.code().to_string(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    " It does not support primary_exchange parameter in req_mkt_depth."
                ),
            ));

            return Err(err);
        }

        let version = 5;

        // send req mkt depth msg

        let message_id: i32 = OutgoingMessageIds::ReqMktDepth as i32;
        let mut msg = "".to_string();
        msg.push_str(&make_field(&message_id)?);
        msg.push_str(&make_field(&version)?);
        msg.push_str(&make_field(&req_id)?);

        // send contract fields
        if self.server_version() >= MIN_SERVER_VER_TRADING_CLASS {
            msg.push_str(&make_field(&contract.con_id)?);
        }
        msg.push_str(&make_field(&contract.symbol)?);
        msg.push_str(&make_field(&contract.sec_type)?);
        msg.push_str(&make_field(&contract.last_trade_date_or_contract_month)?);
        msg.push_str(&make_field(&contract.strike)?);
        msg.push_str(&make_field(&contract.right)?);
        msg.push_str(&make_field(&contract.multiplier)?); // srv v15 and above
        msg.push_str(&make_field(&contract.exchange)?);

        if self.server_version() >= MIN_SERVER_VER_MKT_DEPTH_PRIM_EXCHANGE {
            msg.push_str(&make_field(&contract.primary_exchange)?);
        }
        msg.push_str(&make_field(&contract.currency)?);
        msg.push_str(&make_field(&contract.local_symbol)?);

        if self.server_version() >= MIN_SERVER_VER_TRADING_CLASS {
            msg.push_str(&make_field(&contract.trading_class)?);
        }
        msg.push_str(&make_field(&num_rows)?); // srv v19 and above

        if self.server_version() >= MIN_SERVER_VER_SMART_DEPTH {
            msg.push_str(&make_field(&is_smart_depth)?);
        }
        // send mkt_depth_options parameter
        if self.server_version() >= MIN_SERVER_VER_LINKING {
            // current doc says this part if for "internal use only" -> won't support it
            if mkt_depth_options.len() > 0 {
                let err = IBKRApiLibError::ApiError(TwsApiReportableError::new(
                    req_id,
                    TwsError::Unsupported.code().to_string(),
                    format!(
                        "{}{}",
                        TwsError::Unsupported.message(),
                        " mkt_depth_options."
                    ),
                ));

                return Err(err);
            }
            let mkt_data_options_str = "";
            msg.push_str(&make_field(&mkt_data_options_str)?);
        }
        self.send_request(msg.as_str())
    }

    //----------------------------------------------------------------------------------------------
    /// After calling this function, market depth data for the specified id
    /// will stop flowing.
    ///
    /// # Arguments
    /// * req_id - The ID that was specified in the call to req_mkt_depth().
    //  * is_smart_depth - specifies SMART depth request
    //
    pub fn cancel_mkt_depth(
        &mut self,
        req_id: i32,
        is_smart_depth: bool,
    ) -> Result<(), IBKRApiLibError> {
        self.check_connected(NO_VALID_ID)?;

        if self.server_version() < MIN_SERVER_VER_SMART_DEPTH && is_smart_depth {
            let err = IBKRApiLibError::ApiError(TwsApiReportableError::new(
                req_id,
                TwsError::UpdateTws.code().to_string(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    " It does not support SMART depth cancel."
                ),
            ));

            return Err(err);
        }

        let version = 1;

        let message_id: i32 = OutgoingMessageIds::CancelMktDepth as i32;
        let mut msg = "".to_string();
        msg.push_str(&make_field(&message_id)?);
        msg.push_str(&make_field(&version)?);
        msg.push_str(&make_field(&req_id)?);

        if self.server_version() >= MIN_SERVER_VER_SMART_DEPTH {
            msg.push_str(&make_field(&is_smart_depth)?);
        }

        self.send_request(msg.as_str())
    }

    //#########################################################################
    //################## News Bulletins
    //#########################################################################
    /// Call this function to start receiving news bulletins. Each bulletin
    /// will be returned by the update_news_bulletin() event.
    //
    /// # Arguments
    /// * all_msgs - If set to TRUE, returns all the existing bulletins for
    //               the current day and any new ones. If set to FALSE, will only
    //               return new bulletins.
    pub fn req_news_bulletins(&mut self, all_msgs: bool) -> Result<(), IBKRApiLibError> {
        self.check_connected(NO_VALID_ID)?;

        let version = 1;

        let message_id: i32 = OutgoingMessageIds::ReqNewsBulletins as i32;
        let mut msg = "".to_string();
        msg.push_str(&make_field(&message_id)?);
        msg.push_str(&make_field(&version)?);
        msg.push_str(&make_field(&all_msgs)?);

        self.send_request(msg.as_str())?;
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    ///Call this function to stop receiving news bulletins.
    pub fn cancel_news_bulletins(&mut self) -> Result<(), IBKRApiLibError> {
        self.check_connected(NO_VALID_ID)?;

        let version = 1;

        let message_id: i32 = OutgoingMessageIds::CancelNewsBulletins as i32;
        let mut msg = "".to_string();
        msg.push_str(&make_field(&message_id)?);
        msg.push_str(&make_field(&version)?);
        self.send_request(msg.as_str())?;
        Ok(())
    }

    //#########################################################################
    //################## Financial Advisors
    //#########################################################################
    /// Call this function to request the list of managed accounts. The list
    /// will be returned by the managed_accounts() function on the Wrapper.
    ///
    /// Note:  This request can only be made when connected to a FA managed account.
    pub fn req_managed_accts(&mut self) -> Result<(), IBKRApiLibError> {
        self.check_connected(NO_VALID_ID)?;

        let version = 1;
        let message_id: i32 = OutgoingMessageIds::ReqManagedAccts as i32;
        let mut msg = "".to_string();
        msg.push_str(&make_field(&message_id)?);
        msg.push_str(&make_field(&version)?);
        self.send_request(msg.as_str())
    }

    //----------------------------------------------------------------------------------------------
    /// Call this function to request FA configuration information from TWS.
    /// The data returns in an XML string via a "receiveFA" ActiveX event.
    ///
    /// # Arguments
    /// * fa_data - See the FaDataType enum. Specifies the type of Financial Advisor configuration data beingingg requested. Valid values include:
    ///     * 1 = GROUPS
    ///     * 2 = PROFILE
    ///     * 3 = ACCOUNT ALIASES
    pub fn request_fa(&mut self, fa_data: FaDataType) -> Result<(), IBKRApiLibError> {
        self.check_connected(NO_VALID_ID)?;

        let version = 1;
        let message_id: i32 = OutgoingMessageIds::ReqFa as i32;
        let mut msg = "".to_string();
        msg.push_str(&make_field(&message_id)?);
        msg.push_str(&make_field(&version)?);
        msg.push_str(&make_field(&(fa_data as i32))?);

        self.send_request(msg.as_str())?;
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    /// Call this function to modify FA configuration information from the
    /// API. Note that this can also be done manually in TWS itself.
    /// * req_id - The id of the request.
    /// * fa_data - See the FaDataType enum. Specifies the type of Financial Advisor
    ///             configuration data beingingg requested. Valid values include:
    ///     * 1 = GROUPS
    ///     * 2 = PROFILE
    ///     * 3 = ACCOUNT ALIASES
    /// *cxml - The XML string containing the new FA configuration
    ///         information.
    pub fn replace_fa(
        &mut self,
        req_id: i32,
        fa_data: FaDataType,
        cxml: &str,
    ) -> Result<(), IBKRApiLibError> {
        self.check_connected(NO_VALID_ID)?;

        let version = 1;
        let message_id: i32 = OutgoingMessageIds::ReplaceFa as i32;
        let mut msg = "".to_string();
        msg.push_str(&make_field(&message_id)?);

        msg.push_str(&make_field(&version)?);
        msg.push_str(&make_field(&fa_data)?);
        msg.push_str(&make_field(&String::from(cxml))?);

        if self.server_version() >= MIN_SERVER_VER_REPLACE_FA_END {
            msg.push_str(&make_field(&req_id)?);
        }

        self.send_request(msg.as_str())
    }

    //#########################################################################
    //################## Historical Data
    //#########################################################################
    /// Requests contracts' historical data. When requesting historical data, a
    /// finishing time and date is required along with a duration string. The
    /// resulting bars will be returned in EWrapper.historicalData()
    ///
    /// # Arguments
    /// * req_id - The id of the request. Must be a unique value. When the
    ///            market data returns, it whatToShowill be identified by this tag. This is also
    ///            used when canceling the market data.
    /// * contract - This object contains a description of the contract for which
    ///              market data is being requested.
    /// * end_date_time - Defines a query end date and time at any point during the past 6 mos.
    ///                   Valid values include any date/time within the past six months in the format:
    ///                   yyyymmdd HH:mm:ss ttt where "ttt" is the optional time zone.
    /// * duration_str - Set the query duration up to one week, using a time unit
    ///                  of seconds, days or weeks. Valid values include any integer followed by a space
    ///                  and then S (seconds)); D (days) or W (week). If no unit is specified, seconds is used.
    /// * bar_size_setting - See the BarSize enum for valid values. Specifies the size of the bars that will be returned (within IB/TWS listimits).
    ///                      Valid values include:
    ///     * 1 sec
    ///     * 5 secs
    ///     * 15 secs
    ///     * 30 secs
    ///     * 1 min
    ///     * 2 mins
    ///     * 3 mins
    ///     * 5 mins
    ///     * 15 mins
    ///     * 30 mins
    ///     * 1 hour
    ///     * 1 day
    /// * what_to_show - See the WhatToShow enum for valid values.  Determines the nature of data beinging extracted. Valid values include:
    ///
    ///     * TRADES
    ///     * MIDPOINT
    ///     * BID
    ///     * ASK
    ///     * BID_ASK
    ///     * HISTORICAL_VOLATILITY
    ///     * OPTION_IMPLIED_VOLATILITY
    /// * use_rth - Determines whether to return all data available during the requested time span,
    ///             or only data that falls within regular trading hours. Valid values include:
    ///
    ///     * 0 - all data is returned even where the market in question was outside of its
    ///                       regular trading hours.
    ///     * 1 - only data within the regular trading hours is returned, even if the
    ///                       requested time span falls partially or completely outside of the RTH.
    /// * format_date - Determines the date format applied to returned bars. validd values include:
    ///
    ///     * 1 - dates applying to bars returned in the format: yyyymmdd{space}{space}hh:mm:dd
    ///     * 2 - dates are returned as a long integer specifying the number of seconds since 1/1/1970 GMT.
    /// *chart_options: - For internal use only. Use default value XYZ.
    pub fn req_historical_data(
        &mut self,
        req_id: i32,
        contract: &Contract,
        end_date_time: &str,
        duration_str: &str,
        bar_size_setting: &str,
        what_to_show: &str,
        use_rth: i32,
        format_date: i32,
        keep_up_to_date: bool,
        chart_options: Vec<TagValue>,
    ) -> Result<(), IBKRApiLibError> {
        self.check_connected(NO_VALID_ID)?;

        if self.server_version() < MIN_SERVER_VER_TRADING_CLASS {
            if &contract.trading_class != "" || contract.con_id > 0 {
                let err = IBKRApiLibError::ApiError(TwsApiReportableError::new(
                    req_id,
                    TwsError::UpdateTws.code().to_string(),
                    format!(
                        "{}{}",
                        TwsError::UpdateTws.message(),
                        " It does not support con_id and trading_class parameters in req_historical_data."
                    ),
                ));

                return Err(err);
            }
        }

        let version = 6;

        // send req mkt data msg
        let message_id: i32 = OutgoingMessageIds::ReqHistoricalData as i32;
        let mut msg = "".to_string();
        msg.push_str(&make_field(&message_id)?);

        if self.server_version() < MIN_SERVER_VER_SYNT_REALTIME_BARS {
            msg.push_str(&make_field(&version)?);
        }

        msg.push_str(&make_field(&req_id)?);

        // Send contract fields
        if self.server_version() >= MIN_SERVER_VER_TRADING_CLASS {
            msg.push_str(&make_field(&contract.con_id)?);
            msg.push_str(&make_field(&contract.symbol)?);
            msg.push_str(&make_field(&contract.sec_type)?);
            msg.push_str(&make_field(&contract.last_trade_date_or_contract_month)?);
            msg.push_str(&make_field(&contract.strike)?);
            msg.push_str(&make_field(&contract.right)?);
            msg.push_str(&make_field(&contract.multiplier)?);
            msg.push_str(&make_field(&contract.exchange)?);
            msg.push_str(&make_field(&contract.primary_exchange)?);
            msg.push_str(&make_field(&contract.currency)?);
            msg.push_str(&make_field(&contract.local_symbol)?);
        }
        if self.server_version() >= MIN_SERVER_VER_TRADING_CLASS {
            msg.push_str(&make_field(&contract.trading_class)?);
        }
        msg.push_str(&make_field(&contract.include_expired)?); // srv v31 and above

        msg.push_str(&make_field(&String::from(end_date_time))?); // srv v20 and above
        msg.push_str(&make_field(&String::from(bar_size_setting))?); // srv v20 and above
        msg.push_str(&make_field(&String::from(duration_str))?);
        msg.push_str(&make_field(&use_rth)?);
        msg.push_str(&make_field(&String::from(what_to_show))?);
        msg.push_str(&make_field(&format_date)?); // srv v16 and above

        // Send combo legs for BAG requests
        if contract.sec_type == "BAG" {
            msg.push_str(&make_field(&contract.combo_legs.len())?);
            for combo_leg in &contract.combo_legs {
                msg.push_str(&make_field(&combo_leg.con_id)?);
                msg.push_str(&make_field(&combo_leg.ratio)?);
                msg.push_str(&make_field(&combo_leg.action)?);
                msg.push_str(&make_field(&combo_leg.exchange)?);
            }
        }
        if self.server_version() >= MIN_SERVER_VER_SYNT_REALTIME_BARS {
            msg.push_str(&make_field(&keep_up_to_date)?);
        }
        // Send chart_options parameter
        if self.server_version() >= MIN_SERVER_VER_LINKING {
            let chart_options_str = chart_options
                .iter()
                .map(|x| format!("{}={};", x.tag, x.value))
                .collect::<String>();
            msg.push_str(&make_field(&chart_options_str)?);
        }

        self.send_request(msg.as_str())?;
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    /// Used if an internet disconnect has occurred or the results of a query
    /// are otherwise delayed and the client is no longer interested in receiving
    /// the data.
    ///
    /// # Arguments
    /// * req_id - the id of the original request
    pub fn cancel_historical_data(&mut self, req_id: i32) -> Result<(), IBKRApiLibError> {
        self.check_connected(NO_VALID_ID)?;

        let version = 1;

        let message_id: i32 = OutgoingMessageIds::CancelHistoricalData as i32;
        let mut msg = "".to_string();
        msg.push_str(&make_field(&message_id)?);

        msg.push_str(&make_field(&version)?);
        msg.push_str(&make_field(&req_id)?);

        self.send_request(msg.as_str())?;

        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    /// Returns the timestamp of earliest available historical data for a contract and data type.
    ///
    /// # Arguments
    /// * req_id	- an identifier for the request
    /// * contract	- contract object for which head timestamp is being requested
    /// * what_to_show	- type of data for head timestamp - "BID", "ASK", "TRADES", etc
    /// * use_rth	- use regular trading hours only, 1 for yes or 0 for no
    /// * format_date	set to 1 to obtain the bars' time as yyyyMMdd HH:mm:ss, set to 2 to obtain it like system time format in seconds
    ///
    /// Note that formatData parameter affects intraday bars only
    /// 1-day bars always return with date in YYYYMMDD format
    pub fn req_head_time_stamp(
        &mut self,
        req_id: i32,
        contract: &Contract,
        what_to_show: &str,
        use_rth: i32,
        format_date: i32,
    ) -> Result<(), IBKRApiLibError> {
        self.check_connected(NO_VALID_ID)?;

        if self.server_version() < MIN_SERVER_VER_REQ_HEAD_TIMESTAMP {
            let err = IBKRApiLibError::ApiError(TwsApiReportableError::new(
                req_id,
                TwsError::UpdateTws.code().to_string(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    " It does not support head time stamp requests."
                ),
            ));

            return Err(err);
        }

        let message_id: i32 = OutgoingMessageIds::ReqHeadTimestamp as i32;
        let mut msg = "".to_string();
        msg.push_str(&make_field(&message_id)?);

        msg.push_str(&make_field(&req_id)?);
        msg.push_str(&make_field(&contract.con_id)?);
        msg.push_str(&make_field(&contract.symbol)?);
        msg.push_str(&make_field(&contract.sec_type)?);
        msg.push_str(&make_field(&contract.last_trade_date_or_contract_month)?);
        msg.push_str(&make_field(&contract.strike)?);
        msg.push_str(&make_field(&contract.right)?);
        msg.push_str(&make_field(&contract.multiplier)?);
        msg.push_str(&make_field(&contract.exchange)?);
        msg.push_str(&make_field(&contract.primary_exchange)?);
        msg.push_str(&make_field(&contract.currency)?);
        msg.push_str(&make_field(&contract.local_symbol)?);
        msg.push_str(&make_field(&contract.trading_class)?);
        msg.push_str(&make_field(&contract.include_expired)?);
        msg.push_str(&make_field(&use_rth)?);
        msg.push_str(&make_field(&String::from(what_to_show))?);
        msg.push_str(&make_field(&format_date)?);

        self.send_request(msg.as_str())?;
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    /// Cancel the request
    ///
    /// # Arguments
    /// * req_id - the id of the original request
    pub fn cancel_head_time_stamp(&mut self, req_id: i32) -> Result<(), IBKRApiLibError> {
        self.check_connected(NO_VALID_ID)?;

        if self.server_version() < MIN_SERVER_VER_CANCEL_HEADTIMESTAMP {
            let err = IBKRApiLibError::ApiError(TwsApiReportableError::new(
                req_id,
                TwsError::UpdateTws.code().to_string(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    " It does not support head time stamp requests."
                ),
            ));

            return Err(err);
        }

        let message_id: i32 = OutgoingMessageIds::CancelHeadTimestamp as i32;
        let mut msg = "".to_string();
        msg.push_str(&make_field(&message_id)?);

        msg.push_str(&make_field(&req_id)?);

        self.send_request(msg.as_str())?;
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    /// Returns data histogram of specified contract
    ///
    /// # Arguments
    /// * ticker_id - an identifier for the request
    /// * contract - Contract object for which histogram is being requested
    /// * use_rth - use regular trading hours only, 1 for yes or 0 for no
    /// * time_period - period of which data is being requested, e.g. "3 days"
    pub fn req_histogram_data(
        &mut self,
        ticker_id: i32,
        contract: &Contract,
        use_rth: bool,
        time_period: &str,
    ) -> Result<(), IBKRApiLibError> {
        self.check_connected(NO_VALID_ID)?;

        if self.server_version() < MIN_SERVER_VER_REQ_HISTOGRAM {
            let err = IBKRApiLibError::ApiError(TwsApiReportableError::new(
                NO_VALID_ID,
                TwsError::UpdateTws.code().to_string(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    " It does not support histogram requests."
                ),
            ));

            return Err(err);
        }

        let message_id: i32 = OutgoingMessageIds::ReqHistogramData as i32;
        let mut msg = "".to_string();
        msg.push_str(&make_field(&message_id)?);

        msg.push_str(&make_field(&ticker_id)?);
        msg.push_str(&make_field(&contract.con_id)?);
        msg.push_str(&make_field(&contract.symbol)?);
        msg.push_str(&make_field(&contract.sec_type)?);
        msg.push_str(&make_field(&contract.last_trade_date_or_contract_month)?);
        msg.push_str(&make_field(&contract.strike)?);
        msg.push_str(&make_field(&contract.right)?);
        msg.push_str(&make_field(&contract.multiplier)?);
        msg.push_str(&make_field(&contract.exchange)?);
        msg.push_str(&make_field(&contract.primary_exchange)?);
        msg.push_str(&make_field(&contract.currency)?);
        msg.push_str(&make_field(&contract.local_symbol)?);
        msg.push_str(&make_field(&contract.trading_class)?);
        msg.push_str(&make_field(&contract.include_expired)?);
        msg.push_str(&make_field(&use_rth)?);
        msg.push_str(&make_field(&String::from(time_period))?);

        self.send_request(msg.as_str())?;
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    /// Cancel the request
    ///
    /// # Arguments
    /// * req_id - the id of the original request
    pub fn cancel_histogram_data(&mut self, ticker_id: i32) -> Result<(), IBKRApiLibError> {
        self.check_connected(NO_VALID_ID)?;

        if self.server_version() < MIN_SERVER_VER_REQ_HISTOGRAM {
            let err = IBKRApiLibError::ApiError(TwsApiReportableError::new(
                NO_VALID_ID,
                TwsError::UpdateTws.code().to_string(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    " It does not support histogram requests."
                ),
            ));

            return Err(err);
        }

        let message_id: i32 = OutgoingMessageIds::CancelHistogramData as i32;
        let mut msg = "".to_string();
        msg.push_str(&make_field(&message_id)?);
        msg.push_str(&make_field(&ticker_id)?);

        self.send_request(msg.as_str())?;
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    /// Requests historical Time&Sales data for an instrument.
    ///
    /// # Arguments
    /// * req_id - id of the request
    /// * contract - Contract object that is subject of query
    /// * start_date_time,i.e.	"20170701 12:01:00". Uses TWS timezone specified at login.
    /// * end_date_time,i.e.	"20170701 13:01:00". In TWS timezone. Exactly one of start time and end time has to be defined.
    /// * number_of_ticks - Number of distinct data points. Max currently 1000 per request.
    /// * what_to_show - (Bid_Ask, Midpoint, Trades) Type of data requested.
    /// * use_rth - Data from regular trading hours (1), or all available hours (0)
    /// * ignore_size - A filter only used when the source price is Bid_Ask
    /// * misc_options - should be defined as null, reserved for internal use
    pub fn req_historical_ticks(
        &mut self,
        req_id: i32,
        contract: &Contract,
        start_date_time: &str,
        end_date_time: &str,
        number_of_ticks: i32,
        what_to_show: &str,
        use_rth: i32,
        ignore_size: bool,
        misc_options: Vec<TagValue>,
    ) -> Result<(), IBKRApiLibError> {
        self.check_connected(NO_VALID_ID)?;

        if self.server_version() < MIN_SERVER_VER_HISTORICAL_TICKS {
            let err = IBKRApiLibError::ApiError(TwsApiReportableError::new(
                NO_VALID_ID,
                TwsError::UpdateTws.code().to_string(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    " It does not support historical ticks requests."
                ),
            ));

            return Err(err);
        }

        let message_id: i32 = OutgoingMessageIds::ReqHistoricalTicks as i32;
        let mut msg = "".to_string();
        msg.push_str(&make_field(&message_id)?);

        msg.push_str(&make_field(&req_id)?);
        msg.push_str(&make_field(&contract.con_id)?);
        msg.push_str(&make_field(&contract.symbol)?);
        msg.push_str(&make_field(&contract.sec_type)?);
        msg.push_str(&make_field(&contract.last_trade_date_or_contract_month)?);
        msg.push_str(&make_field(&contract.strike)?);
        msg.push_str(&make_field(&contract.right)?);
        msg.push_str(&make_field(&contract.multiplier)?);
        msg.push_str(&make_field(&contract.exchange)?);
        msg.push_str(&make_field(&contract.primary_exchange)?);
        msg.push_str(&make_field(&contract.currency)?);
        msg.push_str(&make_field(&contract.local_symbol)?);
        msg.push_str(&make_field(&contract.trading_class)?);
        msg.push_str(&make_field(&contract.include_expired)?);
        msg.push_str(&make_field(&String::from(start_date_time))?);
        msg.push_str(&make_field(&String::from(end_date_time))?);
        msg.push_str(&make_field(&number_of_ticks)?);
        msg.push_str(&make_field(&String::from(what_to_show))?);
        msg.push_str(&make_field(&use_rth)?);
        msg.push_str(&make_field(&ignore_size)?);

        let misc_options_string = misc_options
            .iter()
            .map(|x| format!("{}={};", x.tag, x.value))
            .collect::<String>();

        msg.push_str(&make_field(&misc_options_string)?);

        self.send_request(msg.as_str())?;
        Ok(())
    }

    //#########################################################################
    //################## Market Scanners
    //#########################################################################
    /// Requests an XML list of scanner parameters valid in TWS.
    /// Not all parameters are valid from API scanner.
    pub fn req_scanner_parameters(&mut self) -> Result<(), IBKRApiLibError> {
        /*Requests an XML string that describes all possible scanner queries*/

        self.check_connected(NO_VALID_ID)?;

        let version = 1;
        let message_id: i32 = OutgoingMessageIds::ReqScannerParameters as i32;
        let mut msg = "".to_string();
        msg.push_str(&make_field(&message_id)?);
        msg.push_str(&make_field(&version)?);

        self.send_request(msg.as_str())?;
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    /// Starts a subscription to market scan results based on the provided parameters.
    ///
    /// # Arguments
    /// * req_id - The ticker ID. Must be a unique value.
    /// * subscription - This structure contains possible parameters used to filter results.
    /// * scanner_subscription_options -  For internal use only. Use default value XYZ
    pub fn req_scanner_subscription(
        &mut self,
        req_id: i32,
        subscription: ScannerSubscription,
        scanner_subscription_options: Vec<TagValue>,
        scanner_subscription_filter_options: Vec<TagValue>,
    ) -> Result<(), IBKRApiLibError> {
        self.check_connected(req_id)?;

        error!("Server version: {}", self.server_version());
        if self.server_version() < MIN_SERVER_VER_SCANNER_GENERIC_OPTS {
            let err = IBKRApiLibError::ApiError(TwsApiReportableError::new(
                req_id,
                TwsError::UpdateTws.code().to_string(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    " It does not support API scanner subscription generic filter options"
                ),
            ));

            return Err(err);
        }

        let version = 4;

        let message_id: i32 = OutgoingMessageIds::ReqScannerSubscription as i32;
        let mut msg = "".to_string();
        msg.push_str(&make_field(&message_id)?);

        if self.server_version() < MIN_SERVER_VER_SCANNER_GENERIC_OPTS {
            msg.push_str(&make_field(&version)?);
        }
        msg.push_str(&make_field(&req_id)?);
        msg.push_str(&make_field_handle_empty(&subscription.number_of_rows)?);
        msg.push_str(&make_field(&subscription.instrument)?);
        msg.push_str(&make_field(&subscription.location_code)?);
        msg.push_str(&make_field(&subscription.scan_code)?);
        msg.push_str(&make_field_handle_empty(&subscription.above_price)?);
        msg.push_str(&make_field_handle_empty(&subscription.below_price)?);
        msg.push_str(&make_field_handle_empty(&subscription.above_volume)?);
        msg.push_str(&make_field_handle_empty(&subscription.market_cap_above)?);
        msg.push_str(&make_field_handle_empty(&subscription.market_cap_below)?);
        msg.push_str(&make_field(&subscription.moody_rating_above)?);
        msg.push_str(&make_field(&subscription.moody_rating_below)?);
        msg.push_str(&make_field(&subscription.sp_rating_above)?);
        msg.push_str(&make_field(&subscription.sp_rating_below)?);
        msg.push_str(&make_field(&subscription.maturity_date_above)?);
        msg.push_str(&make_field(&subscription.maturity_date_below)?);
        msg.push_str(&make_field_handle_empty(&subscription.coupon_rate_above)?);
        msg.push_str(&make_field_handle_empty(&subscription.coupon_rate_below)?);
        msg.push_str(&make_field(&subscription.exclude_convertible)?);
        msg.push_str(&make_field_handle_empty(
            &subscription.average_option_volume_above,
        )?); // srv v25 and above
        msg.push_str(&make_field(&subscription.scanner_setting_pairs)?); // srv v25 and above
        msg.push_str(&make_field(&subscription.stock_type_filter)?); // srv v27 and above

        // Send scanner_subscription_filter_options parameter
        if self.server_version() >= MIN_SERVER_VER_SCANNER_GENERIC_OPTS {
            error!("!!!!!!!! making scanner options");
            let scanner_subscription_filter = scanner_subscription_filter_options
                .iter()
                .map(|x| format!("{}={};", x.tag, x.value))
                .collect::<String>();

            msg.push_str(&make_field(&scanner_subscription_filter)?);
        }
        // Send scanner_subscription_options parameter
        if self.server_version() >= MIN_SERVER_VER_LINKING {
            let scanner_subscription_options = scanner_subscription_options
                .iter()
                .map(|x| format!("{}={};", x.tag, x.value))
                .collect::<String>();
            msg.push_str(&make_field(&scanner_subscription_options)?);
        }
        error!("req_scanner_subscription");
        error!("{}", msg);
        self.send_request(msg.as_str())?;
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    /// Cancel the request
    ///
    /// # Arguments
    /// * req_id - the id of the original request
    pub fn cancel_scanner_subscription(&mut self, req_id: i32) -> Result<(), IBKRApiLibError> {
        /*reqId:i32 - The ticker ID. Must be a unique value*/

        self.check_connected(NO_VALID_ID)?;

        let version = 1;

        let message_id: i32 = OutgoingMessageIds::CancelScannerSubscription as i32;
        let mut msg = "".to_string();
        msg.push_str(&make_field(&message_id)?);

        msg.push_str(&make_field(&version)?);
        msg.push_str(&make_field(&req_id)?);

        self.send_request(msg.as_str())?;
        Ok(())
    }

    //#########################################################################
    //################## Real Time Bars
    //#########################################################################
    /// Call the req_real_time_bars() function to start receiving real time bar
    /// results through the realtimeBar() EWrapper function.
    ///
    /// # Arguments
    /// * req_id - The Id for the request. Must be a unique value. When the
    ///            data is received, it will be identified by this Id. This is also
    ///            used when canceling the request.
    /// * contract - This object contains a description of the contract
    ///              for which real time bars are being requested
    /// * bar_size - Currently only 5 second bars are supported, if any other
    ///              value is used, an exception will be thrown.
    /// * what_to_show - Determines the nature of the data extracted. Valid
    ///                  values include:
    ///                  * TRADES
    ///                  * BID
    ///                  * ASK
    ///                  * MIDPOINT
    /// * use_rth:bool - Regular Trading Hours only. Valid values include:
    ///                  * 0 = all data available during the time span requested is returned,
    ///                        including time intervals when the market in question was
    ///                        outside of regular trading hours.
    ///                  * 1 = only data within the regular trading hours for the product
    ///                        requested is returned, even if the time time span falls
    ///                        partially or completely outside.
    /// * real_time_bars_options: - For internal use only. Use pub fnault value XYZ
    pub fn req_real_time_bars(
        &mut self,
        req_id: i32,
        contract: &Contract,
        bar_size: i32,
        what_to_show: &str,
        use_rth: bool,
        real_time_bars_options: Vec<TagValue>,
    ) -> Result<(), IBKRApiLibError> {
        self.check_connected(NO_VALID_ID)?;

        if self.server_version() < MIN_SERVER_VER_TRADING_CLASS {
            if !contract.trading_class.is_empty() {
                let err = IBKRApiLibError::ApiError(TwsApiReportableError::new(
                    NO_VALID_ID,
                    TwsError::UpdateTws.code().to_string(),
                    format!(
                        "{}{}",
                        TwsError::UpdateTws.message(),
                        " It does not support con_id and trading_class parameter in req_real_time_bars."
                    ),
                ));

                return Err(err);
            }
        }

        let version = 3;

        let message_id: i32 = OutgoingMessageIds::ReqRealTimeBars as i32;
        let mut msg = "".to_string();
        msg.push_str(&make_field(&message_id)?);

        msg.push_str(&make_field(&version)?);
        msg.push_str(&make_field(&req_id)?);

        // Send contract fields
        if self.server_version() >= MIN_SERVER_VER_TRADING_CLASS {
            msg.push_str(&make_field(&contract.con_id)?);
        }
        msg.push_str(&make_field(&contract.symbol)?);
        msg.push_str(&make_field(&contract.sec_type)?);
        msg.push_str(&make_field(&contract.last_trade_date_or_contract_month)?);
        msg.push_str(&make_field(&contract.strike)?);
        msg.push_str(&make_field(&contract.right)?);
        msg.push_str(&make_field(&contract.multiplier)?);
        msg.push_str(&make_field(&contract.exchange)?);
        msg.push_str(&make_field(&contract.primary_exchange)?);
        msg.push_str(&make_field(&contract.currency)?);
        msg.push_str(&make_field(&contract.local_symbol)?);
        if self.server_version() >= MIN_SERVER_VER_TRADING_CLASS {
            msg.push_str(&make_field(&contract.trading_class)?);
        }
        msg.push_str(&make_field(&bar_size)?);
        msg.push_str(&make_field(&String::from(what_to_show))?);
        msg.push_str(&make_field(&use_rth)?);

        // Send real_time_bars_options parameter
        if self.server_version() >= MIN_SERVER_VER_LINKING {
            let real_time_bars_options_str = real_time_bars_options
                .iter()
                .map(|x| format!("{}={};", x.tag, x.value))
                .collect::<String>();

            msg.push_str(&make_field(&real_time_bars_options_str)?);
        }

        self.send_request(msg.as_str())?;
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    /// Call this to stop receiving real time bars.
    ///
    /// # Arguments
    /// * req_id - The Id that was specified in the call to req_real_time_bars().
    pub fn cancel_real_time_bars(&mut self, req_id: i32) -> Result<(), IBKRApiLibError> {
        self.check_connected(NO_VALID_ID)?;

        let version = 1;

        // Send req mkt data msg
        let message_id: i32 = OutgoingMessageIds::CancelRealTimeBars as i32;
        let mut msg = "".to_string();
        msg.push_str(&make_field(&message_id)?);

        msg.push_str(&make_field(&version)?);
        msg.push_str(&make_field(&req_id)?);

        self.send_request(msg.as_str())?;
        Ok(())
    }

    //#########################################################################
    //################## Fundamental Data
    //#########################################################################
    /// Call this function to receive fundamental data for
    /// stocks. The appropriate market data subscription must be set up in
    /// Account Management before you can receive this data.
    /// Fundamental data will be returned at EWrapper.fundamentalData().
    ///
    /// req_fundamental_data() can handle conid specified in the Contract object,
    /// but not trading_class or multiplier. This is because req_fundamental_data()
    /// is used only for stocks and stocks do not have a multiplier and
    /// trading class.
    ///
    /// # Arguments
    /// * req_id - The ID of the data request. Ensures that responses are
    ///            matched to requests if several requests are in process.
    /// * contract - This structure contains a description of the
    ///              contract for which fundamental data is being requested.
    /// * report_type - One of the following XML reports:
    ///     * ReportSnapshot (company overview)
    ///     * ReportsFinSummary (financial summary)
    ///     * ReportRatios (financial ratios)
    ///     * ReportsFinStatements (financial statements)
    ///     * RESC (analyst estimates)
    ///     * CalendarReport (company calendar)
    pub fn req_fundamental_data(
        &mut self,
        req_id: i32,
        contract: &Contract,
        report_type: &str,
        fundamental_data_options: Vec<TagValue>,
    ) -> Result<(), IBKRApiLibError> {
        self.check_connected(NO_VALID_ID)?;

        let version = 2;

        if self.server_version() < MIN_SERVER_VER_FUNDAMENTAL_DATA {
            let err = IBKRApiLibError::ApiError(TwsApiReportableError::new(
                req_id,
                TwsError::UpdateTws.code().to_string(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    " It does not support fundamental data request."
                ),
            ));

            return Err(err);
        }

        if self.server_version() < MIN_SERVER_VER_TRADING_CLASS {
            let err = IBKRApiLibError::ApiError(TwsApiReportableError::new(
                req_id,
                TwsError::UpdateTws.code().to_string(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    " It does not support con_id parameter in fundamental data request."
                ),
            ));

            return Err(err);
        }

        let message_id: i32 = OutgoingMessageIds::ReqFundamentalData as i32;
        let mut msg = "".to_string();
        msg.push_str(&make_field(&message_id)?);

        msg.push_str(&make_field(&version)?);
        msg.push_str(&make_field(&req_id)?);

        // Send contract fields
        if self.server_version() >= MIN_SERVER_VER_TRADING_CLASS {
            msg.push_str(&make_field(&contract.con_id)?);
        }
        msg.push_str(&make_field(&contract.symbol)?);
        msg.push_str(&make_field(&contract.sec_type)?);
        msg.push_str(&make_field(&contract.exchange)?);
        msg.push_str(&make_field(&contract.primary_exchange)?);
        msg.push_str(&make_field(&contract.currency)?);
        msg.push_str(&make_field(&contract.local_symbol)?);
        msg.push_str(&make_field(&String::from(report_type))?);

        if self.server_version() >= MIN_SERVER_VER_LINKING {
            let tags_value_count = fundamental_data_options.len();
            let fund_data_opt_str = fundamental_data_options
                .iter()
                .map(|x| format!("{}={};", x.tag, x.value))
                .collect::<String>();

            msg.push_str(&make_field(&tags_value_count)?);
            msg.push_str(&make_field(&fund_data_opt_str)?);
        }

        self.send_request(msg.as_str())?;
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    /// Call this function to stop receiving fundamental data.
    ///
    /// # Arguments
    /// * req_id - The ID of the data request
    pub fn cancel_fundamental_data(&mut self, req_id: i32) -> Result<(), IBKRApiLibError> {
        self.check_connected(NO_VALID_ID)?;

        if self.server_version() < MIN_SERVER_VER_FUNDAMENTAL_DATA {
            let err = IBKRApiLibError::ApiError(TwsApiReportableError::new(
                req_id,
                TwsError::UpdateTws.code().to_string(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    " It does not support fundamental data request."
                ),
            ));

            return Err(err);
        }

        let version = 1;

        let message_id: i32 = OutgoingMessageIds::CancelFundamentalData as i32;
        let mut msg = "".to_string();
        msg.push_str(&make_field(&message_id)?);

        msg.push_str(&make_field(&version)?);
        msg.push_str(&make_field(&req_id)?);

        self.send_request(msg.as_str())?;
        Ok(())
    }

    //########################################################################
    //################## News
    //#########################################################################
    /// Requests all open orders places by this specific API client (identified by the API client id).
    /// For client ID 0, this will bind previous manual TWS orders.
    pub fn req_news_providers(&mut self) -> Result<(), IBKRApiLibError> {
        self.check_connected(NO_VALID_ID)?;

        if self.server_version() < MIN_SERVER_VER_REQ_NEWS_PROVIDERS {
            let err = IBKRApiLibError::ApiError(TwsApiReportableError::new(
                NO_VALID_ID,
                TwsError::UpdateTws.code().to_string(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    " It does not support news providers request."
                ),
            ));

            return Err(err);
        }

        let message_id: i32 = OutgoingMessageIds::ReqNewsProviders as i32;
        let mut msg = "".to_string();
        msg.push_str(&make_field(&message_id)?);

        self.send_request(msg.as_str())?;
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    /// Requests news article body given articleId.
    ///
    /// # Arguments
    ///
    /// * req_id - id of the request
    /// * provider_code - short code indicating news provider, e.g. FLY
    /// * article_id - id of the specific article
    /// * news_article_options - reserved for internal use. Should be defined as null.
    pub fn req_news_article(
        &mut self,
        req_id: i32,
        provider_code: &str,
        article_id: &str,
        news_article_options: Vec<TagValue>,
    ) -> Result<(), IBKRApiLibError> {
        self.check_connected(NO_VALID_ID)?;

        if self.server_version() < MIN_SERVER_VER_REQ_NEWS_ARTICLE {
            let err = IBKRApiLibError::ApiError(TwsApiReportableError::new(
                req_id,
                TwsError::UpdateTws.code().to_string(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    " It does not support news article request."
                ),
            ));

            return Err(err);
        }

        let message_id: i32 = OutgoingMessageIds::ReqNewsArticle as i32;
        let mut msg = "".to_string();
        msg.push_str(&make_field(&message_id)?);

        msg.push_str(&make_field(&req_id)?);
        msg.push_str(&make_field(&String::from(provider_code))?);
        msg.push_str(&make_field(&String::from(article_id))?);

        // Send news_article_options parameter
        if self.server_version() >= MIN_SERVER_VER_NEWS_QUERY_ORIGINS {
            let news_article_options_str = news_article_options
                .iter()
                .map(|x| format!("{}={};", x.tag, x.value))
                .collect::<String>();
            msg.push_str(&make_field(&news_article_options_str)?);
        }

        self.send_request(msg.as_str())?;
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    /// Requests historical news headlines.
    ///
    /// # Arguments
    ///
    /// * req_id - id of the request
    /// * con_id - contract id
    /// * provider_codes - a '+'-separated list of provider codes
    /// * start_date_time	- marks the (exclusive) start of the date range. The format is yyyy-MM-dd HH:mm:ss.0
    /// * end_date_time	- marks the (inclusive) end of the date range. The format is yyyy-MM-dd HH:mm:ss.0
    /// * total_results	- the maximum number of headlines to fetch (1 - 300)
    /// * historical_news_options	reserved for internal use. Should be defined as null.
    pub fn req_historical_news(
        &mut self,
        req_id: i32,
        con_id: i32,
        provider_codes: &str,
        start_date_time: &str,
        end_date_time: &str,
        total_results: i32,
        historical_news_options: Vec<TagValue>,
    ) -> Result<(), IBKRApiLibError> {
        self.check_connected(NO_VALID_ID)?;

        if self.server_version() < MIN_SERVER_VER_REQ_HISTORICAL_NEWS {
            let err = IBKRApiLibError::ApiError(TwsApiReportableError::new(
                req_id,
                TwsError::UpdateTws.code().to_string(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    " It does not support historical news request."
                ),
            ));

            return Err(err);
        }

        let message_id: i32 = OutgoingMessageIds::ReqHistoricalNews as i32;
        let mut msg = "".to_string();
        msg.push_str(&make_field(&message_id)?);

        msg.push_str(&make_field(&req_id)?);
        msg.push_str(&make_field(&con_id)?);
        msg.push_str(&make_field(&String::from(provider_codes))?);
        msg.push_str(&make_field(&String::from(start_date_time))?);
        msg.push_str(&make_field(&String::from(end_date_time))?);
        msg.push_str(&make_field(&total_results)?);

        // Send historical_news_options parameter
        if self.server_version() >= MIN_SERVER_VER_NEWS_QUERY_ORIGINS {
            let historical_news_options_str = historical_news_options
                .iter()
                .map(|x| format!("{}={};", x.tag, x.value))
                .collect::<String>();
            msg.push_str(&make_field(&historical_news_options_str)?);
        }

        self.send_request(msg.as_str())?;
        Ok(())
    }

    //#########################################################################
    //################## Display Groups
    //#########################################################################
    /// Replaces Financial Advisor's settings A Financial Advisor can define three different configurations:
    /// 1. Groups - offer traders a way to create a group of accounts and apply a single allocation method to all accounts in the group.
    /// 2. Profiles - let you allocate shares on an account-by-account basis using a predefined calculation value.
    /// 3. Account Aliases - let you easily identify the accounts by meaningful names rather than account numbers.
    ///                     More information at <https://www.interactivebrokers.com/en/?f=%2Fen%2Fsoftware%2Fpdfhighlights%2FPDF-AdvisorAllocations.php%3Fib_entity%3Dllc>
    ///
    /// # Arguments
    /// * req_id - The unique number that will be associated with the response
    pub fn query_display_groups(&mut self, req_id: i32) -> Result<(), IBKRApiLibError> {
        self.check_connected(NO_VALID_ID)?;

        if self.server_version() < MIN_SERVER_VER_LINKING {
            let err = IBKRApiLibError::ApiError(TwsApiReportableError::new(
                req_id,
                TwsError::UpdateTws.code().to_string(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    " It does not support query_display_groups request."
                ),
            ));

            return Err(err);
        }

        let version = 1;

        let message_id: i32 = OutgoingMessageIds::QueryDisplayGroups as i32;
        let mut msg = "".to_string();
        msg.push_str(&make_field(&message_id)?);
        msg.push_str(&make_field(&version)?);
        msg.push_str(&make_field(&req_id)?);

        self.send_request(msg.as_str())?;
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    /// Integrates API client and TWS window grouping.
    ///
    /// # Arguments
    /// * req_id - The unique number that will be associated with the response
    /// * group_id - is the display group for integration
    pub fn subscribe_to_group_events(
        &mut self,
        req_id: i32,
        group_id: i32,
    ) -> Result<(), IBKRApiLibError> {
        self.check_connected(NO_VALID_ID)?;

        if self.server_version() < MIN_SERVER_VER_LINKING {
            let err = IBKRApiLibError::ApiError(TwsApiReportableError::new(
                req_id,
                TwsError::UpdateTws.code().to_string(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    " It does not support subscribe_to_group_events request."
                ),
            ));

            return Err(err);
        }

        let version = 1;

        let message_id: i32 = OutgoingMessageIds::SubscribeToGroupEvents as i32;
        let mut msg = "".to_string();
        msg.push_str(&make_field(&message_id)?);

        msg.push_str(&make_field(&version)?);
        msg.push_str(&make_field(&req_id)?);
        msg.push_str(&make_field(&group_id)?);

        self.send_request(msg.as_str())?;
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    /// Updates the contract displayed in a TWS Window Group.
    ///
    /// # Arguments
    /// * req_id - The requestId specified in subscribe_to_group_events().
    /// * contract_info	is an encoded value designating a unique IB contract. Possible values include:
    /// 1. none - empty selection
    /// 2. contract_id - any non-combination contract. Examples 8314 for IBM SMART; 8314 for IBM ARCA
    /// 3. combo - if any combo is selected Note: This request from the API does not get a TWS response unless an error occurs.

    pub fn update_display_group(
        &mut self,
        req_id: i32,
        contract_info: &str,
    ) -> Result<(), IBKRApiLibError> {
        self.check_connected(NO_VALID_ID)?;

        if self.server_version() < MIN_SERVER_VER_LINKING {
            let err = IBKRApiLibError::ApiError(TwsApiReportableError::new(
                req_id,
                TwsError::UpdateTws.code().to_string(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    " It does not support update_display_group request."
                ),
            ));

            return Err(err);
        }

        let version = 1;

        let message_id: i32 = OutgoingMessageIds::UpdateDisplayGroup as i32;
        let mut msg = "".to_string();
        msg.push_str(&make_field(&message_id)?);

        msg.push_str(&make_field(&version)?);
        msg.push_str(&make_field(&req_id)?);
        msg.push_str(&make_field(&String::from(contract_info))?);

        self.send_request(msg.as_str())?;
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    /// Cancel subscription
    ///
    /// # Arguments
    /// * req_id - The request Id specified in subscribe_to_group_events()
    pub fn unsubscribe_from_group_events(&mut self, req_id: i32) -> Result<(), IBKRApiLibError> {
        self.check_connected(NO_VALID_ID)?;

        if self.server_version() < MIN_SERVER_VER_LINKING {
            let err = IBKRApiLibError::ApiError(TwsApiReportableError::new(
                req_id,
                TwsError::UpdateTws.code().to_string(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    " It does not support unsubscribe_from_group_events request."
                ),
            ));

            return Err(err);
        }

        let version = 1;

        let message_id: i32 = OutgoingMessageIds::UnsubscribeFromGroupEvents as i32;
        let mut msg = "".to_string();
        msg.push_str(&make_field(&message_id)?);

        msg.push_str(&make_field(&version)?);
        msg.push_str(&make_field(&req_id)?);

        self.send_request(msg.as_str())?;
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    /// For IB's internal purpose. Allows to provide means of verification between the TWS and third party programs.
    pub fn verify_request(
        &mut self,
        api_name: &str,
        api_version: &str,
    ) -> Result<(), IBKRApiLibError> {
        self.check_connected(NO_VALID_ID)?;

        if self.server_version() < MIN_SERVER_VER_LINKING {
            let err = IBKRApiLibError::ApiError(TwsApiReportableError::new(
                NO_VALID_ID,
                TwsError::UpdateTws.code().to_string(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    " It does not support verification request."
                ),
            ));

            return Err(err);
        }

        if !self.extra_auth {
            let err = IBKRApiLibError::ApiError(TwsApiReportableError::new(
                NO_VALID_ID,
                TwsError::BadMessage.code().to_string(),
                format!(
                    "{}{}",
                    TwsError::BadMessage.message(),
                    " Intent to authenticate needs to be expressed during initial connect request."
                ),
            ));

            return Err(err);
        }

        let version = 1;

        let message_id: i32 = OutgoingMessageIds::VerifyRequest as i32;
        let mut msg = "".to_string();
        msg.push_str(&make_field(&message_id)?);

        msg.push_str(&make_field(&version)?);
        msg.push_str(&make_field(&String::from(api_name))?);
        msg.push_str(&make_field(&String::from(api_version))?);

        self.send_request(msg.as_str())?;
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    /// For IB's internal purpose. Allows to provide means of verification between the TWS and third party programs.
    pub fn verify_message(&mut self, api_data: &'static str) -> Result<(), IBKRApiLibError> {
        self.check_connected(NO_VALID_ID)?;

        if self.server_version() < MIN_SERVER_VER_LINKING {
            let err = IBKRApiLibError::ApiError(TwsApiReportableError::new(
                NO_VALID_ID,
                TwsError::UpdateTws.code().to_string(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    " It does not support verification request."
                ),
            ));

            return Err(err);
        }

        let version = 1;

        let message_id: i32 = OutgoingMessageIds::VerifyMessage as i32;
        let mut msg = "".to_string();
        msg.push_str(&make_field(&message_id)?);

        msg.push_str(&make_field(&version)?);
        msg.push_str(&make_field(&api_data)?);

        self.send_request(msg.as_str())?;
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    /// For IB's internal purpose. Allows to provide means of verification between the TWS and third party programs.
    pub fn verify_and_auth_request(
        &mut self,
        api_name: &str,
        api_version: &str,
        opaque_isv_key: &str,
    ) -> Result<(), IBKRApiLibError> {
        self.check_connected(NO_VALID_ID)?;

        if self.server_version() < MIN_SERVER_VER_LINKING {
            let err = IBKRApiLibError::ApiError(TwsApiReportableError::new(
                NO_VALID_ID,
                TwsError::UpdateTws.code().to_string(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    " It does not support verification request."
                ),
            ));

            return Err(err);
        }

        if !self.extra_auth {
            let err = IBKRApiLibError::ApiError(TwsApiReportableError::new(
                NO_VALID_ID,
                TwsError::BadMessage.code().to_string(),
                format!(
                    "{}{}",
                    TwsError::BadMessage.message(),
                    " Intent to authenticate needs to be expressed during initial connect request."
                ),
            ));

            return Err(err);
        }

        let version = 1;

        let message_id: i32 = OutgoingMessageIds::VerifyAndAuthRequest as i32;
        let mut msg = "".to_string();
        msg.push_str(&make_field(&message_id)?);

        msg.push_str(&make_field(&version)?);
        msg.push_str(&make_field(&String::from(api_name))?);
        msg.push_str(&make_field(&String::from(api_version))?);
        msg.push_str(&make_field(&String::from(opaque_isv_key))?);

        self.send_request(msg.as_str())?;
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    /// For IB's internal purpose. Allows to provide means of verification between the TWS and third party programs
    pub fn verify_and_auth_message(
        &mut self,
        api_data: &str,
        xyz_response: &str,
    ) -> Result<(), IBKRApiLibError> {
        self.check_connected(NO_VALID_ID)?;

        if self.server_version() < MIN_SERVER_VER_LINKING {
            let err = IBKRApiLibError::ApiError(TwsApiReportableError::new(
                NO_VALID_ID,
                TwsError::UpdateTws.code().to_string(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    " It does not support verification request."
                ),
            ));

            return Err(err);
        }

        let version = 1;

        let message_id: i32 = OutgoingMessageIds::VerifyAndAuthMessage as i32;
        let mut msg = "".to_string();
        msg.push_str(&make_field(&message_id)?);

        msg.push_str(&make_field(&version)?);
        msg.push_str(&make_field(&String::from(api_data))?);
        msg.push_str(&make_field(&String::from(xyz_response))?);

        self.send_request(msg.as_str())?;
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    /// Requests security definition option parameters for viewing a contract's option chain.
    ///
    /// # Arguments
    /// * req_id - the ID chosen for the request
    /// * underlying_symbol
    /// * fut_fop_exchange - The exchange on which the returned options are trading. Can be set to the empty string "" for all exchanges.
    /// * underlying_sec_type - The type of the underlying security, i.e. STK
    /// * underlying_con_id - the contract ID of the underlying security
    pub fn req_sec_def_opt_params(
        &mut self,
        req_id: i32,
        underlying_symbol: &str,
        fut_fop_exchange: &str,
        underlying_sec_type: &str,
        underlying_con_id: i32,
    ) -> Result<(), IBKRApiLibError> {
        self.check_connected(NO_VALID_ID)?;

        if self.server_version() < MIN_SERVER_VER_SEC_DEF_OPT_PARAMS_REQ {
            let err = IBKRApiLibError::ApiError(TwsApiReportableError::new(
                NO_VALID_ID,
                TwsError::UpdateTws.code().to_string(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    " It does not support security pub fninition option request."
                ),
            ));

            return Err(err);
        }

        let message_id: i32 = OutgoingMessageIds::ReqSecDefOptParams as i32;
        let mut msg = "".to_string();
        msg.push_str(&make_field(&message_id)?);

        msg.push_str(&make_field(&req_id)?);
        msg.push_str(&make_field(&String::from(underlying_symbol))?);
        msg.push_str(&make_field(&String::from(fut_fop_exchange))?);
        msg.push_str(&make_field(&String::from(underlying_sec_type))?);
        msg.push_str(&make_field(&underlying_con_id)?);

        self.send_request(msg.as_str())?;
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    /// Requests pre-defined Soft Dollar Tiers. This is only supported for registered professional
    /// advisors and hedge and mutual funds who have configured Soft Dollar Tiers in Account Management.
    /// Refer to: <https://www.interactivebrokers.com/en/software/am/am/manageaccount/requestsoftdollars.htm?Highlight=soft%20dollar%20tier>.
    ///
    /// # Arguments
    /// * req_id - the identifier for this request
    pub fn req_soft_dollar_tiers(&mut self, req_id: i32) -> Result<(), IBKRApiLibError> {
        self.check_connected(NO_VALID_ID)?;

        let message_id: i32 = OutgoingMessageIds::ReqSoftDollarTiers as i32;
        let mut msg = "".to_string();
        msg.push_str(&make_field(&message_id)?);

        msg.push_str(&make_field(&req_id)?);

        self.send_request(msg.as_str())?;
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    /// Requests family codes for an account, for instance if it is a FA, IBroker, or associated account.
    pub fn req_family_codes(&mut self) -> Result<(), IBKRApiLibError> {
        self.check_connected(NO_VALID_ID)?;

        if self.server_version() < MIN_SERVER_VER_REQ_FAMILY_CODES {
            let err = IBKRApiLibError::ApiError(TwsApiReportableError::new(
                NO_VALID_ID,
                TwsError::UpdateTws.code().to_string(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    " It does not support family codes request."
                ),
            ));

            return Err(err);
        }

        let message_id: i32 = OutgoingMessageIds::ReqFamilyCodes as i32;
        let mut msg = "".to_string();
        msg.push_str(&make_field(&message_id)?);

        self.send_request(msg.as_str())?;
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    /// Requests matching stock symbols.
    ///
    /// # Arguments
    /// * req_id - the identifier for this request
    /// * pattern - either start of ticker symbol or (for larger strings) company name
    pub fn req_matching_symbols(
        &mut self,
        req_id: i32,
        pattern: &str,
    ) -> Result<(), IBKRApiLibError> {
        self.check_connected(NO_VALID_ID)?;

        if self.server_version() < MIN_SERVER_VER_REQ_MATCHING_SYMBOLS {
            let err = IBKRApiLibError::ApiError(TwsApiReportableError::new(
                NO_VALID_ID,
                TwsError::UpdateTws.code().to_string(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    " It does not support matching symbols request."
                ),
            ));

            return Err(err);
        }

        let message_id: i32 = OutgoingMessageIds::ReqMatchingSymbols as i32;
        let mut msg = "".to_string();
        msg.push_str(&make_field(&message_id)?);

        msg.push_str(&make_field(&req_id)?);
        msg.push_str(&make_field(&String::from(pattern))?);

        self.send_request(msg.as_str())?;
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    /// Requests completed orders.
    ///
    /// # Arguments
    /// * api_only - If api_only parameter is true, then only completed orders placed from API are requested.
    ///              Each completed order will be fed back through the completed_order() function on the Wrapper
    pub fn req_completed_orders(&mut self, api_only: bool) -> Result<(), IBKRApiLibError> {
        self.check_connected(NO_VALID_ID)?;

        let message_id: i32 = OutgoingMessageIds::ReqCompletedOrders as i32;
        let mut msg = "".to_string();
        msg.push_str(&make_field(&message_id)?);

        msg.push_str(&make_field(&api_only)?);

        self.send_request(msg.as_str())?;
        Ok(())
    }

    /// Request WshMetadata.
    pub fn req_wsh_metadata(&mut self, req_id: i32) -> Result<(), IBKRApiLibError> {
        self.check_connected(NO_VALID_ID)?;

        if self.server_version() < MIN_SERVER_VER_WSHE_CALENDAR {
            let err = IBKRApiLibError::ApiError(TwsApiReportableError::new(
                NO_VALID_ID,
                TwsError::UpdateTws.code().to_string(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    " It does not support WSHE Calendar API."
                ),
            ));
            return Err(err);
        }

        let mut msg = "".to_string();
        let message_id = OutgoingMessageIds::ReqWshMetadata as i32;
        msg.push_str(&make_field(&message_id)?);
        msg.push_str(&make_field(&req_id)?);

        self.send_request(msg.as_str())?;

        Ok(())
    }

    pub fn cancel_wsh_metadata(&mut self, req_id: i32) -> Result<(), IBKRApiLibError> {
        self.check_connected(NO_VALID_ID)?;

        if self.server_version() < MIN_SERVER_VER_WSHE_CALENDAR {
            let err = IBKRApiLibError::ApiError(TwsApiReportableError::new(
                NO_VALID_ID,
                TwsError::UpdateTws.code().to_string(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    "  It does not support WSHE Calendar API."
                ),
            ));
            return Err(err);
        }

        let mut msg = "".to_string();
        let message_id = OutgoingMessageIds::CancelWshMetadata as i32;
        msg.push_str(&make_field(&message_id)?);
        msg.push_str(&make_field(&req_id)?);

        self.send_request(msg.as_str())?;
        Ok(())
    }

    pub fn req_wsh_event_data(
        &mut self,
        req_id: i32,
        wsh_event_data: WshEventData,
    ) -> Result<(), IBKRApiLibError> {
        self.check_connected(NO_VALID_ID)?;

        if self.server_version() < MIN_SERVER_VER_WSHE_CALENDAR {
            let err = IBKRApiLibError::ApiError(TwsApiReportableError::new(
                NO_VALID_ID,
                TwsError::UpdateTws.code().to_string(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    " It does not support WSHE Calendar API."
                ),
            ));
            return Err(err);
        }

        let mut msg = "".to_string();
        let message_id = OutgoingMessageIds::ReqWshEventData as i32;
        msg.push_str(&make_field(&message_id)?);
        msg.push_str(&make_field(&req_id)?);
        msg.push_str(&make_field(&wsh_event_data.con_id)?);

        self.send_request(msg.as_str())?;

        Ok(())
    }

    pub fn cancel_wsh_event_data(&mut self, req_id: i32) -> Result<(), IBKRApiLibError> {
        self.check_connected(NO_VALID_ID)?;

        if self.server_version() < MIN_SERVER_VER_WSHE_CALENDAR {
            let err = IBKRApiLibError::ApiError(TwsApiReportableError::new(
                NO_VALID_ID,
                TwsError::UpdateTws.code().to_string(),
                format!(
                    "{}{}",
                    TwsError::UpdateTws.message(),
                    " It does not support WSHE Calendar API."
                ),
            ));
            return Err(err);
        }

        let mut msg = "".to_string();
        let message_id = OutgoingMessageIds::CancelWshEventData as i32;
        msg.push_str(&make_field(&message_id)?);
        msg.push_str(&make_field(&req_id)?);
        Ok(())
    }

    //------------------------------------------------------------------------------------------------
    /// check if client is connected to TWS
    fn check_connected(&mut self, req_id: i32) -> Result<(), IBKRApiLibError> {
        match self.is_connected() {
            false => {
                let err = IBKRApiLibError::ApiError(TwsApiReportableError::new(
                    req_id,
                    TwsError::NotConnected.code().to_string(),
                    TwsError::NotConnected.message().to_string(),
                ));
                Err(err)
            }
            true => Ok(()),
        }
    }
}
