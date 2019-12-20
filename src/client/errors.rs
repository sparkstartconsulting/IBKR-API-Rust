pub enum Error {
    AlreadyConnected(i32, &'static str),
    ConnectFail(i32, &'static str),
    UpdateTws(i32, &'static str),
    NotConnected(i32, &'static str),
    UnknownId(i32, &'static str),
    Unsupported(i32, &'static str),
    BadLength(i32, &'static str),
    BadMessage(i32, &'static str),
    SocketException(i32, &'static str),
    FailCreateSock(i32, &'static str),
    SslFail(i32, &'static str),
}

impl Error {
    fn get_error_info(&self) -> Error {
        match (*self) {
            Error::AlreadyConnected(code, message) => Error::AlreadyConnected(501, "Already connected."),
            Error::ConnectFail(code, message) => Error::ConnectFail(502,
                                                                    "Couldn't connect to TWS. Confirm that \"Enable ActiveX and Socket EClients\"
                                                            is enabled and connection port is the same as \"Socket Port\" on the
                                                            TWS \"Edit->Global Configuration...->API->Settings\" menu. Live Trading ports:
                                                            TWS: 7496; IB Gateway: 4001. Simulated Trading ports for new installations
                                                            of version 954.1 or newer:  TWS: 7497; IB Gateway: 4002"),
            Error::UpdateTws(code, message) => Error::UpdateTws(503, "The TWS is out of date and must be upgraded."),
            Error::NotConnected(code, message) => Error::NotConnected(504, "Not connected."),
            Error::UnknownId(code, message) => Error::UnknownId(505, "Fatal Error: Unknown message id."),
            Error::Unsupported(code, message) => Error::Unsupported(506, "Unsupported version"),
            Error::BadLength(code, message) => Error::BadLength(507, "Bad message length."),
            Error::BadMessage(code, message) => Error::BadMessage(508, "Bad message."),
            Error::SocketException(code, message) => Error::SocketException(509, "Exception caught while reading socket."),
            Error::FailCreateSock(code, message) => Error::FailCreateSock(520, "Failed to create socket."),
            Error::SslFail(code, message) => Error::SslFail(530, "SSL specific error."),
        }
    }
}
