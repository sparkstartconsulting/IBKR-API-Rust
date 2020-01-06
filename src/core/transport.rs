use std::io::{Error, ErrorKind};

use crate::core::messages::EMessage;

trait Closeable {
    fn close() -> Result<(), Error>;
}

trait ETransport: Closeable {
    fn send(msg: EMessage) -> Result<(), Error>;
}
