//! Lib for sending requests to and processing responses from Interactive Broker's Trader Workstation or IB Gateway
//!
//! For usage of this library, please see the example implementation in [src/examples/test_helpers/manual_tests.rs](https://github.com/sparkstartconsulting/IBKR-API-Rust/blob/fix_docs_add_tests/src/examples/test_helpers/manual_tests.rs)
//!
//! The main structs and traits that clients will use are [**EClient**](https://github.com/sparkstartconsulting/IBKR-API-Rust/blob/fix_docs_add_tests/src/core/client.rs) , a struct that is responsible for
//! connecting to TWS or IB Gateway and sending requests,  and [**Wrapper**](https://github.com/sparkstartconsulting/IBKR-API-Rust/blob/fix_docs_add_tests/src/core/wrapper.rs), a trait that clients will implement that declares callback functions
//! that get called when the application receives messages from the server.
//!
//! In the example below, TWS will send the next valid order ID when the sample application connects.  This will cause the ***Wrapper*** callback method
//! ***next_valid_id*** to be called, which will start sending test requests to TWS (see the
//! ***start_requests*** method in ***TestWrapper*** which is called by ***next_valid_id***).
//!
//! ```no_run        
//! use twsapi::core::errors::IBKRApiLibError;
//! use twsapi::core::client::EClient;
//! use twsapi::core::streamer::{Streamer, TcpStreamer};
//! use std::time::Duration;
//! use twsapi::examples::test_helpers::TestWrapper;
//! use std::sync::{Arc, Mutex};
//! use std::thread;
//!
//! fn main() -> Result<(), IBKRApiLibError> {
//!
//!     let wrapper = Arc::new(Mutex::new(TestWrapper::<TcpStreamer>::new()));
//!     let app = Arc::new(Mutex::new(EClient::new(wrapper.clone())));
//!
//!     println!("getting connection...");
//!
//!     wrapper.lock().expect("Wrapper mutex was poisoned").client = Option::from(app.clone());
//!
//!    //use port 7497 for TWS or 4002 for IB Gateway, depending on the port you have set
//!    app.lock()
//!       .expect("EClient mutex was poisoned")
//!       .connect("127.0.0.1", 4002, 0)?;
//!
//!    //4002
//!    thread::sleep(Duration::new(18600, 0));
//!
//!    Ok(())
//! }
//! ```     
pub mod core;
pub mod examples;
