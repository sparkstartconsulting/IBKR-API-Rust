use std::io::{Error, ErrorKind};

use crate::client::messages::EMessage;

trait Closeable {
    fn close() -> Result<(), Error>;
}

trait ETransport: Closeable {
    fn send(msg: EMessage) -> Result<(), Error>;
}
