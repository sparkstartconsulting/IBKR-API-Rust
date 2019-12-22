use std::any::Any;
use std::borrow::{Borrow, Cow};
use std::collections::vec_deque::VecDeque;
use std::convert::TryFrom;
use std::io::Write;
use std::net::TcpStream;
use std::net::{Shutdown, SocketAddr};
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::{channel, Receiver};
use std::thread;

use byteorder::{BigEndian, ByteOrder};
use encoding::all::ASCII;
use encoding::types::RawEncoder;
use encoding::{ByteWriter, DecoderTrap, EncoderTrap, Encoding};
use from_ascii::{FromAscii, FromAsciiRadix};
use num_traits::FromPrimitive;

use crate::client::common::*;
use crate::client::decoder::Decoder;
use crate::client::messages::read_msg;
use crate::client::messages::{make_message, read_fields, OutgoingMessageIds};
use crate::client::reader::Reader;
use crate::client::server_versions::*;
use crate::client::wrapper::Wrapper;
use crate::connection::Connection;
use crate::make_field;

enum ConnStatus {
    DISCONNECTED,
    CONNECTING,
    CONNECTED,
    REDIRECT,
}

pub struct EClient<'a, T: Wrapper> {
    msg_queue: Option<Receiver<String>>,
    wrapper: &'a T,
    decoder: Decoder<'a, T>,
    done: bool,
    n_keyb_int_hard: i32,
    stream: Option<TcpStream>,
    host: String,
    port: u32,
    extra_auth: bool,
    client_id: i32,
    server_version: i32,
    conn_time: String,
    conn_state: ConnStatus,
    opt_capab: String,
    asynchronous: bool,
}

impl<'a, T: Wrapper> EClient<'a, T>
where
    T: Wrapper,
{
    pub fn new(wrapper: &'a T) -> Self {
        EClient {
            msg_queue: None,
            wrapper,
            decoder: Decoder::new(wrapper, 0),
            done: false,
            n_keyb_int_hard: 0,
            stream: None,
            host: "".to_string(),
            port: 0,
            extra_auth: false,
            client_id: 0,
            server_version: 0,
            conn_time: "".to_string(),
            conn_state: ConnStatus::DISCONNECTED,
            opt_capab: "".to_string(),
            asynchronous: false,
        }
    }
    fn send_request(&self, request: &str) {
        info!("Sending request to server...");
        let bytes = make_message(request);
        self.send_bytes(bytes.as_slice());
    }

    fn send_bytes(&self, bytes: &[u8]) {
        debug!("Message before send {:?}", bytes);
        self.stream.as_ref().unwrap().write(bytes);
    }

    pub fn connect(&mut self, host: String, port: u32, client_id: i32) {
        self.host = host;
        self.port = port;
        self.client_id = client_id;
        debug!("Connecting");
        let thestream = TcpStream::connect(format!("{}:{}", self.host.to_string(), port)).unwrap();
        debug!("Connected");
        self.stream = Option::from(thestream.try_clone().unwrap());

        let reader_stream = thestream.try_clone().unwrap();

        let (tx, rx) = channel::<String>();
        let mut reader = Reader::new(thestream, tx);

        self.msg_queue = Option::from(rx);

        let v_100_prefix = "API\0";
        let v_100_version = format!("v{}..{}", MIN_CLIENT_VER, MAX_CLIENT_VER);

        let msg = make_message(v_100_version.as_str());
        debug!("v_100_version.as_str(): {}", v_100_version.as_str());
        //logger.debug("msg %s", msg)
        //let encoded = ASCII.encode(v_100_prefix, EncoderTrap::NcrEscape).unwrap();
        let mut bytearray: Vec<u8> = Vec::new();
        bytearray.extend_from_slice(v_100_prefix.as_bytes());
        bytearray.extend_from_slice(msg.as_slice());
        //let msg2 = format!("{:?}", String::from_utf8(bytearray).unwrap());
        debug!(
            "sending initial request: {:?}",
            String::from_utf8(bytearray.as_slice().to_vec()).unwrap()
        );

        self.send_bytes(bytearray.as_slice());
        let mut fields: Vec<String> = Vec::new();

        //sometimes I get news before the server version, thus the loop

        while fields.len() != 2 {
            if fields.len() > 0 {
                self.decoder.interpret(fields.as_slice());
            }

            let mut buf = reader.recv_packet();
            debug!("got initial packet: {}", buf.len());

            if buf.len() > 0 {
                let (size, msg, remaining_messages) = read_msg(buf.as_slice());

                fields.clear();
                fields.extend_from_slice(read_fields(msg.as_ref()).as_slice());
                debug!("fields.len(): {}", fields.len());
            } else {
                fields.clear();
            }
        }

        self.server_version = i32::from_ascii(fields.get(0).unwrap().as_bytes()).unwrap();
        debug!("Server version: {} ", self.server_version);
        self.conn_time = fields.get(1).unwrap().to_string();
        debug!("Connection time: {} ", self.conn_time);
        self.decoder.server_version = self.server_version;
        thread::spawn(move || {
            reader.run();
        });

        self.start_api();
    }

    pub fn is_connected(&mut self) -> bool {
        true
    }

    //#########################################################################
    //################## Account and Portfolio
    //########################################################################

    pub fn req_account_updates(&mut self, subscribe: bool, acct_code: &'static str) {
        /*Call this function to start getting account values, portfolio,
        and last update time information via EWrapper.updateAccountValue(),
        EWrapperi.updatePortfolio() and Wrapper.updateAccountTime().

        subscribe:bool - If set to TRUE, the client will start receiving account
            and Portfoliolio updates. If set to FALSE, the client will stop
            receiving this information.
        acctCode:str -The account code for which to receive account and
            portfolio updates.*/

        info!("subscribe: {}, acct_code: {}", subscribe, acct_code);
        debug!(
            "req_account_updates: subscribe: {}, acct_code: {}",
            subscribe, acct_code
        );

        if !self.is_connected() {
            //self.wrapper.error(NO_VALID_ID, NOT_CONNECTED.code(), NOT_CONNECTED.msg());
            return;
        }

        let mut version = 2;
        let mut _subscribe = subscribe;
        let mut _acct_code = acct_code;
        let mut msg = "".to_string();

        let mut message_id = OutgoingMessageIds::ReqAcctData as i32;
        let x = message_id.to_be_bytes();
        msg.push_str(&make_field(&mut message_id));
        msg.push_str(&make_field(&mut version));
        msg.push_str(&make_field(&mut _subscribe)); // TRUE = subscribe, FALSE = unsubscribe.
                                                    // msg.push_str(&make_field(&mut _acct_code)); // srv v9 and above, the account code.This will only be used for FA clients

        self.send_request(msg.as_str());
    }
    pub fn req_account_summary(
        &mut self,
        req_id: i32,
        group_name: &'static str,
        tags: &'static str,
    ) {
        /* Call this method to request and keep up to date the data that appears
        on the TWS Account Window Summary tab. The data is returned by
        accountSummary().

        Note:   This request is designed for an FA managed account but can be
        used for any multi-account structure.

        req_id:int - The ID of the data request. Ensures that responses are matched
            to requests If several requests are in process.
        group_name:str - Set to All to returnrn account summary data for all
            accounts, or set to a specific Advisor Account Group name that has
            already been created in TWS Global Configuration.
        tags:str - A comma-separated list of account tags.  Available tags are:
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

        //self.logRequest(current_fn_name(), vars())

        if !self.is_connected() {
            //self.wrapper.error(NO_VALID_ID, NOT_CONNECTED.code(), NOT_CONNECTED.msg());
            return;
        }

        let mut version = 2;
        let mut _req_id = req_id;
        let mut _group_name = group_name;
        let mut _tags = tags;
        let mut message_id: i32 = OutgoingMessageIds::ReqAccountSummary as i32;
        let mut msg = "".to_string();
        msg.push_str(&make_field(&mut message_id));
        msg.push_str(&make_field(&mut version));
        msg.push_str(&make_field(&mut _req_id));
        msg.push_str(&make_field(&mut _group_name));
        msg.push_str(&make_field(&mut _tags));

        self.send_request(msg.as_str())
    }

    pub fn req_current_time(&mut self) {
        let mut version = 2;

        let mut message_id: i32 = OutgoingMessageIds::ReqCurrentTime as i32;
        let mut msg = "".to_string();
        msg.push_str(&make_field(&mut message_id));
        msg.push_str(&make_field(&mut version));

        debug!(
            "#########################    Requesting current time: {}",
            msg.as_str()
        );
        self.send_request(msg.as_str())
    }
    pub fn disconnect(&mut self) {
        self.stream.as_mut().unwrap().shutdown(Shutdown::Both);
    }

    pub fn cancel_account_summary(&mut self, req_id: i32) {
        /*Cancels the request for Account Window Summary tab data.

        req_id:int - The ID of the data request being canceled.*/

        //self.logRequest(current_fn_name(), vars())

        if !self.is_connected() {
            //self.wrapper.error(NO_VALID_ID, NOT_CONNECTED.code(), NOT_CONNECTED.msg())
            return;
        }

        let mut version = 2;
        let mut _req_id = req_id;
        let mut message_id = OutgoingMessageIds::ReqAcctData as i32;
        let mut msg = "".to_string();
        msg.push_str(&make_field(&mut message_id));
        msg.push_str(&make_field(&mut version));
        msg.push_str(&make_field(&mut _req_id));

        //self.send_request(msg.as_str())
    }

    pub fn run(&mut self) {
        //This is the function that has the message loop.
        info!("Starting run...");
        let queue = self.msg_queue.as_mut().unwrap();
        while !self.done && true {
            info!("################Client trying to receive...");
            let text = queue.recv().unwrap();
            info!("Client got message...");
            info!("{}", text.as_str());
            if text.len() > MAX_MSG_LEN as usize {
                //self.wrapper.error(
                //NO_VALID_ID,
                //BAD_LENGTH.code(),
                //format!("{}:{}:{}"(BAD_LENGTH.msg(), len(text), &text)),
                //);
                self.disconnect();
                break;
            } else {
                let fields = read_fields((&text).as_ref());
                //debug("fields {}", fields)
                self.decoder.interpret(fields.as_slice());
            }
        }
    }

    fn start_api(&mut self) {
        //Initiates the message exchange between the client application and
        //the TWS/IB Gateway. """

        //self.logRequest(current_fn_name(), vars())

        if !self.is_connected() {
            // self.wrapper.error(NO_VALID_ID, NOT_CONNECTED.code(),
            //                   NOT_CONNECTED.msg());
            return;
        }

        let version = 2;
        let mut opt_capab = "".to_string();
        if self.server_version >= MIN_SERVER_VER_OPTIONAL_CAPABILITIES as i32 {
            opt_capab = make_field(&mut self.opt_capab);
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
}
