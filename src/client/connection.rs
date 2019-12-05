use crate::client::wrapper::Wrapper;
use bytebuffer::ByteBuffer;
use log;
use log4rs;
use std::io::{Error, ErrorKind};
use std::io::{Read, Write};
use std::net::{Shutdown, TcpStream};
use std::str;

//--------------------------------------------------------------------------------------------------
pub struct Connection<'a, T> {
    pub host: &'a str,
    pub port: u16,
    socket: Option<TcpStream>,
    wrapper: T,
}

//--------------------------------------------------------------------------------------------------
impl<'a, T> Connection<'a, T>
where
    T: Wrapper,
{
    pub fn new(host: &'a str, port: u16, wrapper: T) -> Self {
        Connection {
            host,
            port,
            socket: None,
            wrapper: wrapper,
        }
    }

    //----------------------------------------------------------------------------------------------
    pub fn connect(&mut self) -> Result<(), Error> {
        let address = format!("{}:{}", self.host, self.port);

        let socket: Result<TcpStream, Error> = TcpStream::connect(address);
        match socket {
            Ok(_) => {
                info!("Connected to the server!");
                self.socket = Some(socket.unwrap());
                Ok(())
            }
            Err(e) => {
                info!("Couldn't connect to server...");
                Err(e)
            }
        }
    }

    //----------------------------------------------------------------------------------------------
    pub fn disconnect(&mut self) {
        match self.socket {
            Some(_) => {
                debug!("disconnecting");
                self.socket.as_ref().unwrap().shutdown(Shutdown::Both);
                self.socket = None;
                debug!("disconnected");
            }
            _ => {
                self.wrapper.connection_closed();
            }
        }
    }

    //----------------------------------------------------------------------------------------------
    pub fn is_connected(&self) -> bool {
        match self.socket {
            Some(_) => true,
            _ => false,
        }
    }

    //----------------------------------------------------------------------------------------------
    pub fn send_msg(&self, msg: &str) -> Result<usize, Error> {
        if !self.is_connected() {
            debug!("send_msg attempted while not connected");
            Err(Error::new(
                ErrorKind::NotConnected,
                "send_msg attempted while not connected.",
            ))
        } else {
            let n_sent = self.socket.as_ref().unwrap().write(msg.as_bytes()).unwrap();

            debug!("send_msg: sent: {}", n_sent);
            Ok(n_sent)
        }
    }

    //----------------------------------------------------------------------------------------------
    fn _recv_all_msg(&self) -> Vec<u8> {
        let mut cont = true;
        let mut allbuf = ByteBuffer::new();

        while cont && self.socket.is_some() {
            let mut buf: [u8; 4096] = [0; 4096];
            self.socket.as_ref().unwrap().read(&mut buf);
            allbuf.write_bytes(&buf);
            //allbuff!("len %d raw:%s|", buf.len(), buf);

            if buf.len() < 4096 {
                cont = false;
            }
        }
        allbuf.to_bytes()
    }

    //----------------------------------------------------------------------------------------------
    pub fn recv_msg(&mut self) -> Vec<u8> {
        if !self.is_connected() {
            debug!("recvMsg attempted while not connected, releasing lock");
            ByteBuffer::from_bytes(b"").to_bytes()
        } else {
            let buf = self._recv_all_msg();
            // receiving 0 bytes outside a timeout means the connection is either
            // closed or broken
            if buf.len() == 0 {
                debug!("socket either closed or broken, disconnecting");
                self.disconnect();

                debug!("socket timeout from recvMsg");
                ByteBuffer::from_bytes(b"").to_bytes()
            } else {
                buf
            }
        }
    }
}
