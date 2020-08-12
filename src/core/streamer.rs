use bytebuffer::ByteBuffer;
use std::net::{Ipv4Addr, SocketAddr, TcpStream};
use std::{
    io::{self, Read, Write},
    net::{Shutdown, ToSocketAddrs},
};

pub trait Streamer: Read + Write + Send + Sync {
    fn shutdown(&mut self, how: Shutdown) -> io::Result<()>;
    fn connect(&mut self, addr: &SocketAddr);
}
#[derive(Debug)]
pub struct TcpStreamer {
    pub(crate) stream: TcpStream,
}

impl TcpStreamer {
    pub fn new(stream: TcpStream) -> Self {
        Self { stream: stream }
    }
}

impl Streamer for TcpStreamer {
    fn shutdown(&mut self, how: Shutdown) -> io::Result<()> {
        self.stream.shutdown(how)
    }

    fn connect(&mut self, addr: &SocketAddr) {
        self.stream = TcpStream::connect(addr).expect("Cannot connect!!");
    }
}

impl Clone for TcpStreamer {
    fn clone(&self) -> Self {
        TcpStreamer::new(self.stream.try_clone().unwrap())
    }
}

impl Read for TcpStreamer {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.stream.read(buf)
    }
}

impl Write for TcpStreamer {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.stream.write(buf)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.stream.flush()
    }
}

pub struct TestStreamer {
    stream: ByteBuffer,
}

impl TestStreamer {
    pub fn new() -> Self {
        TestStreamer {
            stream: ByteBuffer::new(),
        }
    }
}

impl Streamer for TestStreamer {
    fn shutdown(&mut self, how: Shutdown) -> io::Result<()> {
        Ok(())
    }

    fn connect(&mut self, addr: &SocketAddr) {}
}

impl Read for TestStreamer {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.stream.read(buf)
    }
}

impl Write for TestStreamer {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.stream.write(buf)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.stream.flush()
    }
}
