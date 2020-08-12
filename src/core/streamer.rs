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

    fn read_to_end(&mut self, allbuf: &mut Vec<u8>) -> io::Result<usize> {
        let mut cont = true;

        while cont {
            let mut buf: [u8; 4096] = [0; 4096];
            //debug!("Getting bytes");
            //info!("Starting read");
            let bytes_read = self
                .stream
                .read(&mut buf)
                .expect("Couldnt read from reader..."); //read(&mut buf)?;
                                                        //debug!("got bytes: {}", bytes_read);
                                                        //info!("Finished read. Read {}", bytes_read);
            allbuf.extend_from_slice(&buf[0..bytes_read]);
            //logger.debug("len %d raw:%s|", len(buf), buf)

            if bytes_read < 4096 {
                //debug!("bytes_read: {}", bytes_read);
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
