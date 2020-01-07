use std::borrow::{Borrow, BorrowMut};
use std::io::Read;
use std::io::Stdin;
use std::net::{TcpListener, ToSocketAddrs};
use std::ops::Deref;
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::time::Duration;

use twsapi::core::client::EClient;
use twsapi::core::errors::IBKRApiLibError;
use twsapi::core::wrapper::Wrapper;
use twsapi::examples::defaults::DefaultWrapper;

fn main() -> Result<(), IBKRApiLibError> {
    log4rs::init_file("log_config.yml", Default::default()).unwrap();

    let wrapper = DefaultWrapper::new();
    let app = Arc::new(Mutex::new(EClient::new(wrapper)));
    app.lock().unwrap().wrapper.lock().unwrap().client = Option::from(app.clone());
    app.lock()
        .unwrap()
        .wrapper
        .lock()
        .unwrap()
        .deref()
        .next_valid_id(3);
    app.lock()
        .unwrap()
        .connect("127.0.0.1".to_string(), 7497, 0);
    //let fut = app.run();
    //app.req_account_updates(true, "");

    // app.req_current_time();

    app.lock()
        .unwrap()
        .req_account_summary(2, "All", "NetLiquidation");

    app.lock().unwrap().req_current_time();

    app.lock().unwrap().cancel_account_summary(2);

    thread::sleep(Duration::new(2, 0));

    app.lock()
        .unwrap()
        .req_account_summary(3, "All", "NetLiquidation");
    app.lock().unwrap().req_current_time();

    thread::sleep(Duration::new(2, 0));

    app.lock().unwrap().cancel_account_summary(2);

    thread::sleep(Duration::new(2, 0));

    app.lock()
        .unwrap()
        .req_account_summary(3, "All", "NetLiquidation");
    app.lock().unwrap().req_current_time();

    thread::sleep(Duration::new(2, 0));

    app.lock().unwrap().cancel_account_summary(2);

    thread::sleep(Duration::new(2, 0));

    app.lock()
        .unwrap()
        .req_account_summary(3, "All", "NetLiquidation");
    app.lock().unwrap().req_current_time();

    thread::sleep(Duration::new(2, 0));

    app.lock().unwrap().cancel_account_summary(2);

    app.lock()
        .unwrap()
        .req_account_summary(4, "All", "NetLiquidation");
    app.lock().unwrap().req_current_time();

    app.lock().unwrap().disconnect();

    //    thread::sleep(Duration::new(2, 0));
    //    app.lock().unwrap().req_current_time();
    //    thread::sleep(Duration::new(2, 0));

    thread::sleep(Duration::new(18600, 0));
    Ok(())
}
