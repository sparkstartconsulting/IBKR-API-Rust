use crate::client::connection::Connection;
use crate::client::messages::read_msg;
use crate::client::messages::EMessage;
use log;
use log4rs;
use std::borrow::Borrow;
use std::convert::TryInto;
use std::io::BufReader;
use std::io::Read;
use std::net::{Shutdown, TcpStream};
use std::sync::atomic::AtomicBool;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};

pub struct Reader {
    stream: TcpStream,
    messages: Sender<String>,
}

impl Reader {
    pub fn new(stream: TcpStream, messages: Sender<String>) -> Self {
        Reader { stream, messages }
    }

    fn recv_packet(&mut self) -> Vec<u8> {
        let buf = self._recv_all_msg();

        // receiving 0 bytes outside a timeout means the connection is either
        // closed or broken
        if buf.len() == 0 {
            debug!("socket either closed or broken, disconnecting");
            self.stream.shutdown(Shutdown::Both).unwrap();

            //debug!("socket timeout from recvMsg %s", sys.exc_info())
        }
        buf
    }

    fn _recv_all_msg(&mut self) -> Vec<u8> {
        let mut cont = true;
        let mut allbuf: Vec<u8> = Vec::new();

        while cont {
            let mut buf: [u8; 4096] = [0; 4096];
            let bytes_read = self.stream.read(&mut buf).unwrap();
            allbuf.extend_from_slice(&buf);
            //logger.debug("len %d raw:%s|", len(buf), buf)

            if bytes_read < 4096 {
                cont = false;
            }
        }
        allbuf
    }

    pub fn run(&mut self) {
        loop {
            /// grab a packet of messages from the socket
            let mut message_packet = self.recv_packet();
            debug!("reader loop, recvd size {}", message_packet.len());

            /// Read messages from the packet until there are no more.
            /// When this loop ends, break into the outer loop and grab another packet.  
            /// Repeat until the connection is closed
            while message_packet.len() > 0 {
                /// Read a message from the packet then add it to the message queue below.
                let (size, msg, remaining_messages) = read_msg(message_packet.as_slice());

                /// clear the Vec that holds the bytes from the packet
                /// and reload with the bytes that haven't been read.
                /// The variable new_buf only holds the unread bytes (messages) left in the packet
                message_packet.clear();
                message_packet.extend_from_slice(remaining_messages.as_slice());

                debug!(
                    "size:{} msg.size:{} msg:|{}| buf:{:?}|",
                    size,
                    msg.len(),
                    msg,
                    String::from_utf8(message_packet.to_owned()).unwrap()
                );

                if msg != "" {
                    self.messages.send(msg).unwrap();
                } else {
                    ///Break to the outer loop and get another packet of messages.
                    debug!("more incoming packet(s) are needed ");
                    break;
                }
            }
        }
        debug!("EReader thread finished")
    }

    /*fn read(&self,buf: [u8], off: i32, len: i32) throws IOException {
        return m_dis.read(buf, off, len);
    }*/
}
