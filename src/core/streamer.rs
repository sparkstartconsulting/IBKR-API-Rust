use bytebuffer::ByteBuffer;
use std::net::{SocketAddr, TcpStream};
use std::{
    io::{self, Read, Write},
    net::Shutdown,
};

//----------------------------------------------------------------------------------------------
pub trait Streamer: Read + Write + Send + Sync {
    fn shutdown(&mut self, how: Shutdown) -> io::Result<()>;
    fn connect(&mut self, addr: &SocketAddr);
}
//----------------------------------------------------------------------------------------------
#[derive(Debug)]
pub struct TcpStreamer {
    pub(crate) stream: TcpStream,
}

impl TcpStreamer {
    pub fn new(stream: TcpStream) -> Self {
        Self { stream }
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

//----------------------------------------------------------------------------------------------
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

impl Default for TestStreamer {
    fn default() -> Self {
        Self::new()
    }
}

impl Streamer for TestStreamer {
    fn shutdown(&mut self, _how: Shutdown) -> io::Result<()> {
        Ok(())
    }

    fn connect(&mut self, _addr: &SocketAddr) {}
}

impl Read for TestStreamer {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.stream.read(buf)
    }

    fn read_to_end(&mut self, allbuf: &mut Vec<u8>) -> io::Result<usize> {
        let mut cont = true;
        const NUM_BYTES: usize = 4096;

        while cont {
            let mut buf: [u8; NUM_BYTES] = [0; NUM_BYTES];

            let bytes_read = self
                .stream
                .read(&mut buf)
                .expect("Couldnt read from reader...");
            allbuf.extend_from_slice(&buf[0..bytes_read]);

            if bytes_read < NUM_BYTES {
                cont = false;
            }
        }
        Ok(allbuf.len())
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
