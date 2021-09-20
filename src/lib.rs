//! Lib for sending requests to and processing responses from Interactive
//! Broker's Trader Workstation (TWS) or IB Gateway.
//!
//! For usage of this library, please see the example implementation in
//! [`manual_tests.rs`](https://github.com/sparkstartconsulting/IBKR-API-Rust/blob/master/src/examples/test_helpers/manual_tests.rs)
//!
//! The main structs and traits that clients will use are
//! * [`EClient`](core::client::EClient); connecting to TWS or IB Gateway and
//! sending requests.
//! * [`Wrapper`](core::wrapper::Wrapper); a trait that clients will implement
//!   that declares callback functions that get called when
//! the application receives messages from the server.
//!
//! In the example below, TWS will send the next valid order ID when the sample
//! application connects.  This will cause
//! [`Wrapper::next_valid_id()`](core::wrapper::Wrapper::next_valid_id) method
//! to be called, which will start sending test requests to
//! TWS.
//!
//! See the
//! [`TestWrapper::start_requests()`](https://github.com/sparkstartconsulting/IBKR-API-Rust/blob/master/src/examples/test_helpers.rs#L3207-L3214)
//! method which is called by
//! [`Wrapper::next_valid_id()`](core::wrapper::Wrapper::next_valid_id).
//!
//! ```no_run
//! use std::sync::{Arc, Mutex};
//! use std::thread;
//! use std::time::Duration;
//! use twsapi::core::client::EClient;
//! use twsapi::core::errors::IBKRApiLibError;
//! use twsapi::core::streamer::{Streamer, TcpStreamer};
//! use twsapi::examples::test_helpers::TestWrapper;
//!
//! fn main() -> Result<(), IBKRApiLibError> {
//!     let wrapper = Arc::new(Mutex::new(TestWrapper::<TcpStreamer>::new()));
//!     let app = Arc::new(Mutex::new(EClient::new(wrapper.clone())));
//!
//!     println!("getting connection...");
//!
//!     wrapper.lock().expect("Wrapper mutex was poisoned").client = Option::from(app.clone());
//!
//!     //use port 7497 for TWS or 4002 for IB Gateway, depending on the port you have set
//!     app.lock()
//!         .expect("EClient mutex was poisoned")
//!         .connect("127.0.0.1", 4002, 0)?;
//!
//!     //4002
//!     thread::sleep(Duration::new(18600, 0));
//!
//!     Ok(())
//! }
//! ```
pub mod core;
pub mod examples;
mod tests;
