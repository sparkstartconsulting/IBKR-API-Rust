extern crate bytebuffer;
#[macro_use]
extern crate log;
extern crate log4rs;
extern crate num;
extern crate twsapi;
#[macro_use]
extern crate num_derive;
use crate::client::client::EClient;
use crate::client::defaults::DefaultWrapper;
use client::connection;
use client::messages::make_field;
use std::borrow::{Borrow, BorrowMut};
use std::io::Read;
extern crate serde;
extern crate ascii;

mod client;

fn main() {
    //log4rs::init_file("log_config.yml", Default::default()).unwrap();
    println!("starting");
    //let x = b"&".to_vec();
    //let a: u8 = '&' as u8; //x.as_slice();
    //println!("{}", a);
    //let z = [a];
    //let ab = std::str::from_utf8(&z).unwrap();
    //let b = ab.chars().nth(0).unwrap();
    //println!("{}", b);

    //println!("{}", make_field(&mut true));
    //println!("{}", make_field(&mut false));
    //println!("{}", make_field(&mut "Hello!!".to_string()));
    //println!("{}", make_field(&mut 47));
    //println!("{}", make_field(&mut 100.3));
    let mut wrapper = DefaultWrapper::new();
    let mut app = EClient::new(&mut wrapper);
    {
        app.connect("127.0.0.1".to_string(), 7497, 0);
    }
    {
        app.req_account_updates(true, "DU248225");
    }
}
