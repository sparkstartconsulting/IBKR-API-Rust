/*! Lib for sending requests to and processing responses from Interactive Broker's Trader Workstation or IB Gateway

For usage of this library, please see the example implementation in src/bin/manual_tests.rs

The main structs and traits that clients will use are **EClient** , a struct that is responsible for 
connecting to TWS or IB Gateway and sending requests,  and **Wrapper**, a trait that clients will implement that declares callback functions 
that get called when the application receives messages from the server.

# Example

Upon connecting, TWS will send the next valid order ID which will cause the ***Wrapper*** callback method
next_valid_id to be called, which will start sending tests requests to TWS (see the
***start_requests*** function in ***TestWrapper*** which is called by ***next_valid_id***.

    // TestWrapper implements the Wrapper trait and handles messages sent from TWS
    let wrapper = Arc::new(Mutex::new(TestWrapper::new()));
    
    //EClient sends requests to TWS
    let app = Arc::new(Mutex::new(EClient::new(wrapper.clone())));

    wrapper.lock().unwrap().client = Option::from(app.clone());

    // Upon connecting, TWS will send the next valid order ID which will cause the wrapper callback method
    // next_valid_id to be called, which will start sending tests requests to TWS (see the
    // start_requests function in TestWrapper which is called by next_valid_id
    info!("getting connection...");
    app.lock().unwrap().connect("127.0.0.1", 7497, 0);

    thread::sleep(Duration::new(18600, 0));

    Ok(())
*/
pub mod core;
pub mod examples;
