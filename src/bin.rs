extern crate ascii;
extern crate bigdecimal;
extern crate bytebuffer;
extern crate float_cmp;
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
use std::io::Stdin;
use std::net::{TcpListener, ToSocketAddrs};
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::time::Duration;

use client::connection;
use client::messages::make_field;

use crate::client::client::EClient;
use crate::client::defaults::DefaultWrapper;
use crate::client::wrapper::Wrapper;

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

    let listener = TcpListener::bind(("127.0.0.1", 7495)).unwrap();

    let wrapper = DefaultWrapper::new();
    let app = Arc::new(Mutex::new(EClient::new(wrapper)));
    let app2 = Option::from(app.clone());
    app.lock().unwrap().wrapper.lock().unwrap().client = app2;
    app.lock().unwrap().wrapper.lock().unwrap().next_valid_id(3);
    app.lock()
        .unwrap()
        .connect("127.0.0.1".to_string(), 7497, 0);
    //let fut = app.run();
    //app.req_account_updates(true, "");

    // app.req_current_time();
    {
        app.lock()
            .unwrap()
            .req_account_summary(2, "All", "NetLiquidation");

        //    let app2 = Arc::new(Mutex::new(app));
        //    let mut moved = app2.clone();
        //    thread::spawn(move || {
        //        moved.lock().unwrap().run();
        //    });
        //app2.lock().into_inner().unwrap().req_current_time();
    }
    {
        app.lock().unwrap().cancel_account_summary(2);
    }
    thread::sleep(Duration::new(2, 0));
    {
        app.lock()
            .unwrap()
            .req_account_summary(3, "All", "NetLiquidation");
    }
    thread::sleep(Duration::new(2, 0));
    {
        app.lock().unwrap().cancel_account_summary(2);
    }
    thread::sleep(Duration::new(2, 0));
    {
        app.lock()
            .unwrap()
            .req_account_summary(3, "All", "NetLiquidation");
    }
    thread::sleep(Duration::new(2, 0));
    {
        app.lock().unwrap().cancel_account_summary(2);
    }
    thread::sleep(Duration::new(2, 0));
    {
        app.lock()
            .unwrap()
            .req_account_summary(3, "All", "NetLiquidation");
    }
    thread::sleep(Duration::new(2, 0));
    {
        app.lock().unwrap().cancel_account_summary(2);
    }
    {
        app.lock()
            .unwrap()
            .req_account_summary(4, "All", "NetLiquidation");
    }

    {
        app.lock().unwrap().disconnect();
    }
    //    thread::sleep(Duration::new(2, 0));
    //    app.lock().unwrap().req_current_time();
    //    thread::sleep(Duration::new(2, 0));

    thread::sleep(Duration::new(18600, 0));
}
