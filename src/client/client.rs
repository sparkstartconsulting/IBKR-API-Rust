use std::collections::vec_deque::VecDeque;

use crate::client::decoder::Decoder;
use crate::client::messages::make_message;
use crate::client::reader::Reader;
use crate::client::wrapper::Wrapper;
use crate::connection::Connection;
use std::io::Write;
use std::net::TcpStream;
use std::sync::atomic::AtomicBool;

enum ConnStatus {
    DISCONNECTED,
    CONNECTING,
    CONNECTED,
    REDIRECT,
}

pub struct EClient<'a, T> {
    msg_queue: VecDeque<&'a str>,
    wrapper: T,
    decoder: Option<Decoder<Box<dyn Wrapper>>>,
    done: bool,
    n_keyb_int_hard: i32,
    stream: TcpStream,
    host: &'a str,
    port: i32,
    extra_auth: bool,
    client_id: i32,
    server_version: i32,
    conn_time: &'a str,
    conn_state: ConnStatus,
    opt_capab: &'a str,
    asynchronous: bool,
    //reader: Reader<T>,
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

impl<'a, T> EClient<'a, T> {
    fn send_sequest(&mut self, request: &str) {
        let bytes = make_message(request);
        self.stream.write(bytes.to_bytes().as_slice());
    }
}
