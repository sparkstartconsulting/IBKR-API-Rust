extern crate ascii;
extern crate bytebuffer;
extern crate from_ascii;
#[macro_use]
extern crate log;
extern crate log4rs;
extern crate num;
#[macro_use]
extern crate num_derive;
extern crate serde;
extern crate twsapi;

use std::borrow::{Borrow, BorrowMut};
use std::io::Read;
use std::thread;
use std::time::Duration;

use client::connection;
use client::messages::make_field;

use crate::client::client::EClient;
use crate::client::defaults::DefaultWrapper;

mod client;

fn main() {
    log4rs::init_file("log_config.yml", Default::default()).unwrap();
    debug!("starting");
    error!("test logging");
    info!("test logging");
    debug!("test logging");
    //let x = b"&".to_vec();
    //let a: u8 = '&' as u8; //x.as_slice();
    //debug!("{}", a);
    //let z = [a];
    //let ab = std::str::from_utf8(&z).unwrap();
    //let b = ab.chars().nth(0).unwrap();
    //debug!("{}", b);

    //debug!("{}", make_field(&mut true));
    //debug!("{}", make_field(&mut false));
    //debug!("{}", make_field(&mut "Hello!!".to_string()));
    //debug!("{}", make_field(&mut 47));
    //debug!("{}", make_field(&mut 100.3));
    let mut wrapper = DefaultWrapper::new();
    let mut app = EClient::new(&mut wrapper);
    {
        app.connect("127.0.0.1".to_string(), 7497, 0);
    }
    {
        app.req_account_updates(true, "");
    }

    app.run();

    thread::sleep(Duration::new(60, 0));
}
