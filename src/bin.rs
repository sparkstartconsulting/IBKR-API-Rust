extern crate bytebuffer;
#[macro_use]
extern crate log;
extern crate log4rs;
extern crate num;
extern crate twsapi;
#[macro_use]
extern crate num_derive;
use client::connection;
use std::io::Read;

mod client;

fn main() {
    //log4rs::init_file("log_config.yml", Default::default()).unwrap();
    info!("starting");
    let x = b"&".to_vec();
    let a: u8 = '&' as u8; //x.as_slice();
    println!("{}", a);
    let z = [a];
    let ab = std::str::from_utf8(&z).unwrap();
    let b = ab.chars().nth(0).unwrap();
    println!("{}", b);
}
