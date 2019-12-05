use crate::client::decoder::Decoder;
use crate::client::wrapper::Wrapper;
use crate::connection::Connection;
use std::collections::vec_deque::VecDeque;
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
}
