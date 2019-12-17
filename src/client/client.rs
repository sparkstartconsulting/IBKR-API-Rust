use std::collections::vec_deque::VecDeque;

use crate::client::decoder::Decoder;
use crate::client::messages::{make_message, read_fields};
use crate::client::reader::Reader;
use crate::client::server_versions::*;
use crate::client::wrapper::Wrapper;
use crate::connection::Connection;

use crate::client::messages::read_msg;
use encoding::all::ASCII;
use encoding::types::RawEncoder;
use encoding::{ByteWriter, DecoderTrap, EncoderTrap, Encoding};
use std::borrow::{Borrow, Cow};
use std::convert::TryFrom;
use std::io::Write;
use std::net::SocketAddr;
use std::net::TcpStream;
use std::sync::atomic::AtomicBool;

enum ConnStatus {
    DISCONNECTED,
    CONNECTING,
    CONNECTED,
    REDIRECT,
}

pub struct EClient<'a, T: Wrapper> {
    msg_queue: VecDeque<&'a str>,
    wrapper: T,
    decoder: Decoder<'a, T>,
    done: bool,
    n_keyb_int_hard: i32,
    stream: TcpStream,
    host: &'a str,
    port: u32,
    extra_auth: bool,
    client_id: i32,
    server_version: i32,
    conn_time: String,
    conn_state: ConnStatus,
    opt_capab: &'a str,
    asynchronous: bool,
    reader: Reader,
    /*
    self.setConnState(EClient.DISCONNECTED)
            self.done = False
            self.n_keyb_int_hard = 0
            self.conn = None
            self.host = None
            self.port = None
            self.extra_auth = False
            self.client_id = None
            self.serverVersion_ = None
            self.conn_time = None
            self.conn_state = None
            self.opt_capab = ""
            self.asynchronous = False
            self.reader = None
            self.decode = None
            self.setConnState(EClient.DISCONNECTED)
            */
}

impl<'a, T: Wrapper> EClient<'a, T> {
    fn send_sequest(&mut self, request: &str) {
        let bytes = make_message(request);
        self.stream.write(bytes.to_bytes().as_slice());
    }

    fn connect(&'a mut self, host: &'a str, port: u32, client_id: i32) {
        self.host = host;
        self.port = port;
        self.client_id = client_id;

        self.stream = TcpStream::connect(format!("{}:{}", host, port)).unwrap();
        let v_100_prefix = "API\0";
        let v_100_version = format!("v{}..{}", MIN_CLIENT_VER, MAX_CLIENT_VER);

        let msg = make_message(v_100_version.as_str());
        //logger.debug("msg %s", msg)
        let encoded = ASCII.encode(v_100_prefix, EncoderTrap::NcrEscape).unwrap();
        let msg2 = format!(
            "{}{}",
            ASCII.decode(&encoded, DecoderTrap::Strict).unwrap(),
            msg.to_string()
        );

        self.send_sequest(msg2.as_str());

        self.decoder = Decoder::new(&mut self.wrapper, self.server_version);

        let mut fields: Vec<String> = Vec::new();

        //sometimes I get news before the server version, thus the loop
        while fields.len() != 2 {
            self.decoder.interpret(fields.as_slice());
            let buf = self.reader.recv_packet();
            //logger.debug("ANSWER %s", buf)
            if buf.len() > 0 {
                let (size, msg, remaining_messages) = read_msg(buf.as_slice());
                //logger.debug("size:%d msg:%s rest:%s|", size, msg, rest)

                fields.clear();
                fields.extend_from_slice(read_fields(msg.as_str()).as_slice());
            } else {
                fields.clear();
            }
        }
        self.server_version = fields.get(0).unwrap().parse::<i32>().unwrap();

        self.conn_time = fields.get(1).unwrap().to_string();

        //self.decoder.
        //logger.debug("fields %s", fields)
    }
}
