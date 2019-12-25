use std::fmt;

const BITS: (i32, &str) = (501, "message");

const ALREADY_CONNECTED: (i32, &str) = (501, "Already connected.");
const CONNECT_FAIL: (i32, &str) = (502, "Couldn't connect to TWS. Confirm that \"Enable ActiveX and Socket EClients\"
                                            is enabled and connection port is the same as \"Socket Port\" on the
                                            TWS \"Edit->Global Configuration...->API->Settings\" menu. Live Trading ports:
                                            TWS: 7496; IB Gateway: 4001. Simulated Trading ports for new installations
                                            of version 954.1 or newer:  TWS: 7497; IB Gateway: 4002");
const UPDATE_TWS: (i32, &str) = (503, "The TWS is out of date and must be upgraded.");
const NOT_CONNECTED: (i32, &str) = (504, "Not connected.");
const UNKNOWN_ID: (i32, &str) = (505, "Fatal TwsError: Unknown message id.");
const UNSUPPORTED: (i32, &str) = (506, "UNSUPPORTED version");
const BAD_LENGTH: (i32, &str) = (507, "Bad message length.");
const BAD_MESSAGE: (i32, &str) = (508, "Bad message.");
const SOCKET_EXCEPTION: (i32, &str) = (509, "Exception caught while reading socket.");
const FAIL_CREATE_SOCK: (i32, &str) = (520, "Failed to create socket.");
const SSL_FAIL: (i32, &str) = (530, "SSL specific TwsError.");

pub enum TwsError {
    AlreadyConnected,
    ConnectFail,
    UpdateTws,
    NotConnected,
    UnknownId,
    Unsupported,
    BadLength,
    BadMessage,
    SocketException,
    FailCreateSock,
    SslFail,
}

impl TwsError {
    pub fn code(&self) -> i32 {
        match (*self) {
            TwsError::AlreadyConnected => ALREADY_CONNECTED.0,
            TwsError::ConnectFail => CONNECT_FAIL.0,
            TwsError::UpdateTws => UPDATE_TWS.0,
            TwsError::NotConnected => NOT_CONNECTED.0,
            TwsError::UnknownId => UNKNOWN_ID.0,
            TwsError::Unsupported => UNSUPPORTED.0,
            TwsError::BadLength => BAD_LENGTH.0,
            TwsError::BadMessage => BAD_MESSAGE.0,
            TwsError::SocketException => SOCKET_EXCEPTION.0,
            TwsError::FailCreateSock => FAIL_CREATE_SOCK.0,
            TwsError::SslFail => SSL_FAIL.0,
        }
    }
    pub fn message(&self) -> &'static str {
        match (*self) {
            TwsError::AlreadyConnected => ALREADY_CONNECTED.1,
            TwsError::ConnectFail => CONNECT_FAIL.1,
            TwsError::UpdateTws => UPDATE_TWS.1,
            TwsError::NotConnected => NOT_CONNECTED.1,
            TwsError::UnknownId => UNKNOWN_ID.1,
            TwsError::Unsupported => UNSUPPORTED.1,
            TwsError::BadLength => BAD_LENGTH.1,
            TwsError::BadMessage => BAD_MESSAGE.1,
            TwsError::SocketException => SOCKET_EXCEPTION.1,
            TwsError::FailCreateSock => FAIL_CREATE_SOCK.1,
            TwsError::SslFail => SSL_FAIL.1,
        }
    }
}

impl fmt::Display for TwsError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "Code: {}, Message: {}", self.code(), self.message())
    }
}
