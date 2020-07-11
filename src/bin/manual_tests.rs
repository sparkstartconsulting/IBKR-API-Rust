//! Binary for manually testing crate

use log::*;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use twsapi::core::client::EClient;
use twsapi::core::errors::*;
use twsapi::examples::test_helpers::TestWrapper;

/// Example of using client and wrapper.
/// Requuires a running instance of TWS or IB Gateway connected to the port in main.
/// Upon connecting, TWS will send the next valid order ID which will cause the wrapper callback method
/// next_valid_id to be called, which will start sending tests requests to TWS (see the
/// start_requests function inn TestWrapper which is called by next_valid_id
//==================================================================================================
pub fn main() -> Result<(), IBKRApiLibError> {
    log4rs::init_file("./ibkr-api-rust/log_config.yml", Default::default()).unwrap();

    let wrapper = Arc::new(Mutex::new(TestWrapper::new()));
    let app = Arc::new(Mutex::new(EClient::new(wrapper.clone())));

    info!("getting connection...");
    wrapper.lock().unwrap().client = Option::from(app.clone());

    // Upon connecting, TWS will send the next valid order ID which will cause the wrapper callback method
    // next_valid_id to be called, which will start sending tests requests to TWS (see the
    // start_requests function inn TestWrapper which is called by next_valid_id
    // app.lock().unwrap().connect("127.0.0.1", 7497, 0);
    app.lock().unwrap().connect("127.0.0.1", 4002, 0)?;

    thread::sleep(Duration::new(18600, 0));

    Ok(())
}
