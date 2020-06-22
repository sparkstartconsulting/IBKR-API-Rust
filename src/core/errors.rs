use std::num::{ParseFloatError, ParseIntError};
use std::sync::mpsc::{RecvError, RecvTimeoutError};
use std::{error, fmt, io};

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

#[derive(Clone, Debug)]
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
        match *self {
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
        match *self {
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

impl error::Error for TwsError {}

impl fmt::Display for TwsError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "Code: {}, Message: {}", self.code(), self.message())
    }
}

#[derive(Debug)]
pub enum IBKRApiLibError {
    Io(io::Error),
    ParseFloat(ParseFloatError),
    ParseInt(ParseIntError),
    RecvError(RecvError),
    RecvTimeoutError(RecvTimeoutError),
    ApiError(TwsApiReportableError),
}

impl fmt::Display for IBKRApiLibError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            // Both underlying errors already impl `Display`, so we defer to
            // their implementations.
            IBKRApiLibError::Io(ref err) => write!(f, "IO error: {}", err),
            IBKRApiLibError::ParseFloat(ref err) => write!(f, "Parse error: {}", err),
            IBKRApiLibError::ParseInt(ref err) => write!(f, "Parse error: {}", err),
            IBKRApiLibError::RecvError(ref err) => write!(f, "Recieve error: {}", err),
            IBKRApiLibError::RecvTimeoutError(ref err) => write!(f, "Reader Send error {}", err),
            IBKRApiLibError::ApiError(ref err) => write!(f, "TWS Error: {}", err),
        }
    }
}

impl error::Error for IBKRApiLibError {
    fn cause(&self) -> Option<&dyn error::Error> {
        match self {
            // N.B. Both of these implicitly cast `err` from their concrete
            // types (either `&io::Error` or `&num::ParseIntError`)
            // to a trait object `&Error`. This works because both error types
            // implement `Error`.
            IBKRApiLibError::Io(ref err) => Some(err),
            IBKRApiLibError::ParseFloat(ref err) => Some(err),
            IBKRApiLibError::ParseInt(ref err) => Some(err),
            IBKRApiLibError::RecvError(ref err) => Some(err),
            IBKRApiLibError::RecvTimeoutError(ref err) => Some(err),
            IBKRApiLibError::ApiError(ref err) => Some(err),
        }
    }
}

impl From<io::Error> for IBKRApiLibError {
    fn from(err: io::Error) -> IBKRApiLibError {
        IBKRApiLibError::Io(err)
    }
}

impl From<ParseIntError> for IBKRApiLibError {
    fn from(err: ParseIntError) -> IBKRApiLibError {
        IBKRApiLibError::ParseInt(err)
    }
}

impl From<ParseFloatError> for IBKRApiLibError {
    fn from(err: ParseFloatError) -> IBKRApiLibError {
        IBKRApiLibError::ParseFloat(err)
    }
}

impl From<RecvError> for IBKRApiLibError {
    fn from(err: RecvError) -> IBKRApiLibError {
        IBKRApiLibError::RecvError(err)
    }
}

impl From<RecvTimeoutError> for IBKRApiLibError {
    fn from(err: RecvTimeoutError) -> IBKRApiLibError {
        IBKRApiLibError::RecvTimeoutError(err)
    }
}

impl From<TwsApiReportableError> for IBKRApiLibError {
    fn from(err: TwsApiReportableError) -> IBKRApiLibError {
        IBKRApiLibError::ApiError(err)
    }
}

#[derive(Clone)]
pub struct TwsApiReportableError {
    pub req_id: i32,
    pub code: String,
    pub description: String,
}

impl TwsApiReportableError {
    pub fn new(req_id: i32, code: String, description: String) -> Self {
        Self {
            req_id: req_id,
            code: code,
            description: description,
        }
    }
}

impl fmt::Display for TwsApiReportableError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "TWS Error: req_id = {}. code = {}. description = {}",
            self.req_id, self.code, self.description
        )
    }
}

impl fmt::Debug for TwsApiReportableError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "TWS Error: req_id = {}. code = {}. description = {}",
            self.req_id, self.code, self.description
        )
    }
}

impl error::Error for TwsApiReportableError {}
