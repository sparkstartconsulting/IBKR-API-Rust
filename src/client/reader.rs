use std::borrow::Borrow;
use std::convert::TryInto;
use std::io::BufReader;
use std::io::Read;
use std::net::{Shutdown, TcpStream};
use std::ops::Deref;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;

use log;
use log4rs;

use crate::client::connection::Connection;
use crate::client::messages::read_msg;
use crate::client::messages::EMessage;

pub struct Reader {
    stream: TcpStream,
    messages: Sender<String>,
}

impl Reader {
    pub fn new(stream: TcpStream, messages: Sender<String>) -> Self {
        Reader { stream, messages }
    }

    pub fn recv_packet(&mut self) -> Vec<u8> {
        debug!("_recv_all_msg");
        let buf = self._recv_all_msg();
        // receiving 0 bytes outside a timeout means the connection is either
        // closed or broken
        if buf.len() == 0 {
            debug!("socket either closed or broken, disconnecting");
            self.stream.shutdown(Shutdown::Both).unwrap();
        }
        buf
    }

    fn _recv_all_msg(&mut self) -> Vec<u8> {
        let mut cont = true;
        let mut allbuf: Vec<u8> = Vec::new();

        while cont {
            let mut buf: [u8; 4096] = [0; 4096];
            debug!("Getting bytes");
            let bytes_read = self.stream.read(&mut buf).unwrap();
            debug!("got bytes: {}", bytes_read);

            allbuf.extend_from_slice(&buf[0..bytes_read]);
            //logger.debug("len %d raw:%s|", len(buf), buf)

            if bytes_read < 4096 {
                debug!("bytes_read: {}", bytes_read);
                cont = false;
            }
        }
        allbuf
    }

    pub fn process_reader_msgs(&mut self) {
        /// grab a packet of messages from the socket
        let mut message_packet = self.recv_packet();
        debug!(" recvd size {}", message_packet.len());

        /// Read messages from the packet until there are no more.
        /// When this loop ends, break into the outer loop and grab another packet.  
        /// Repeat until the connection is closed
        ///
        let mut msg = String::new();
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
                message_packet.to_owned()
            );

            if msg.as_str() != "" {
                debug!("sending message to client... ");
                self.messages.send(msg).unwrap();
            } else {
                ///Break to the outer loop and get another packet of messages.

                debug!("more incoming packet(s) are needed ");
                break;
            }
        }
    }

    pub async fn run(&mut self) {
        loop {
            debug!("starting reader loop");

            self.process_reader_msgs();
        }
        //debug!("EReader thread finished")
    }
}
