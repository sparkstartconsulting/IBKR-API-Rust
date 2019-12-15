use std::io::Write;
use std::net::TcpStream;
use std::string::ToString;
use std::u8;

use bytebuffer::ByteBuffer;

const SEP: u8 = '\0' as u8;
const EMPTY_LENGTH_HEADER: [u8; 4] = [0; 4];

trait Sender<T> {
    fn send(&mut self, a: T);
}

pub struct Builder {
    server_version: i32,
    buffer: ByteBuffer,
}

impl Builder {
    pub fn new(server_version: i32) -> Self {
        Builder {
            server_version,
            buffer: ByteBuffer::new(),
        }
    }

    pub fn write_out(&self, mut stream: &mut dyn std::io::Write) {
        stream.write(self.buffer.to_bytes().as_slice());
    }

    pub fn reset(&mut self) {
        self.buffer.clear();
    }
}

impl Sender<i32> for Builder {
    fn send(&mut self, a: i32) {
        self.buffer.write_i32(a);
        self.buffer.write_u8(SEP);
    }
}

impl Sender<i64> for Builder {
    fn send(&mut self, a: i64) {
        self.buffer.write_i64(a);
        self.buffer.write_u8(SEP);
    }
}

impl Sender<f32> for Builder {
    fn send(&mut self, a: f32) {
        self.buffer.write_f32(a);
        self.buffer.write_u8(SEP);
    }
}

impl Sender<f64> for Builder {
    fn send(&mut self, a: f64) {
        self.buffer.write_f64(a);
        self.buffer.write_u8(SEP);
    }
}

impl Sender<bool> for Builder {
    fn send(&mut self, a: bool) {
        self.buffer.write_bit(a);
        self.buffer.write_u8(SEP);
    }
}

impl Sender<&str> for Builder {
    fn send(&mut self, a: &str) {
        self.buffer.write_string(a);
        self.buffer.write_u8(SEP);
    }
}

pub struct Decoder<T> {
    wrapper: T,
    server_version: i32,
}
