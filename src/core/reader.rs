//! Reads and processes messages from the TCP socket
use std::io::Read;
use std::net::Shutdown;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;
use std::sync::Arc;

use log::*;

use super::streamer::Streamer;
use crate::core::errors::IBKRApiLibError;
use crate::core::messages::read_msg;

//==================================================================================================
pub struct Reader {
    stream: Box<dyn Streamer + 'static>,
    messages: Sender<String>,
    disconnect_requested: Arc<AtomicBool>,
    is_connected: bool,
}

impl Reader {
    pub fn new(
        stream: Box<impl Streamer + 'static>,
        messages: Sender<String>,
        disconnect_requested: Arc<AtomicBool>,
    ) -> Self {
        Reader {
            stream,
            messages,
            disconnect_requested,
            is_connected: true,
        }
    }

    //----------------------------------------------------------------------------------------------
    pub fn recv_packet(&mut self) -> Result<Vec<u8>, IBKRApiLibError> {
        //debug!("_recv_all_msg");
        let buf = self._recv_all_msg()?;
        // receiving 0 bytes outside a timeout means the connection is either
        // closed or broken
        if buf.is_empty() && !self.disconnect_requested.load(Ordering::Acquire) {
            info!("socket either closed or broken, disconnecting");
            self.stream.shutdown(Shutdown::Both)?;
            self.is_connected = false;
        }
        Ok(buf)
    }

    //----------------------------------------------------------------------------------------------
    fn _recv_all_msg(&mut self) -> Result<Vec<u8>, IBKRApiLibError> {
        let mut cont = true;
        let mut allbuf: Vec<u8> = Vec::new();
        const NUM_BYTES: usize = 4096;

        while cont {
            let mut buf: [u8; NUM_BYTES] = [0; NUM_BYTES];

            let bytes_read = self
                .stream
                .read(&mut buf)
                .expect("Couldnt read from reader...");
            allbuf.extend_from_slice(&buf[0..bytes_read]);
            //logger.debug("len %d raw:%s|", len(buf), buf)

            if bytes_read < NUM_BYTES {
                cont = false;
            }
        }
        Ok(allbuf)
    }

    //----------------------------------------------------------------------------------------------
    fn process_reader_msgs(&mut self) -> Result<(), IBKRApiLibError> {
        // grab a packet of messages from the socket
        let mut message_packet = self.recv_packet()?;
        //debug!(" recvd size {}", message_packet.len());

        // Read messages from the packet until there are no more.
        // When this loop ends, break into the outer loop and grab another packet.
        // Repeat until the connection is closed
        //
        let _msg = String::new();
        while !message_packet.is_empty() {
            // Read a message from the packet then add it to the message queue below.
            let (_size, msg, remaining_messages) = read_msg(message_packet.as_slice())?;

            // clear the Vec that holds the bytes from the packet
            // and reload with the bytes that haven't been read.
            // The variable remaining_messages only holds the unread messagesleft in the packet
            message_packet.clear();
            message_packet.extend_from_slice(remaining_messages.as_slice());

            if msg.as_str() != "" {
                self.messages.send(msg).expect("READER CANNOT SEND MESSAGE");
            } else {
                //Break to the outer loop in run and get another packet of messages.

                debug!("more incoming packet(s) are needed ");
                break;
            }
        }
        Ok(())
    }
    //----------------------------------------------------------------------------------------------
    pub fn run(&mut self) {
        debug!("starting reader loop");
        loop {
            if self.disconnect_requested.load(Ordering::Acquire) || !self.is_connected {
                return;
            }
            let result = self.process_reader_msgs();
            if result.is_ok() {
                continue;
            }
            error!("{:?}", result);
        }
    }
}
