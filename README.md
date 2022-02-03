# IBKR-API-Rust

## This branch is currently in progress to upgrade to version 10.12.01.  Do not use until it is merged to master!!! test

Port of Interactive Broker's trading API written in Rust (API_Version=10.12.01)

Please see the latest IB Tws Api documentation here: <http://interactivebrokers.github.io/tws-api/introduction.html>.

The documentation has information regarding configuring Trader WorkStation and IB Gateway to enable API access.

For usage of this library, please see the example implementation in [src/examples/test_helpers/manual_tests.rs](src/bin/manual_tests.rs)

The main structs and traits that clients will use are [**EClient**](src/core/client.rs) , a struct that is responsible for
connecting to TWS or IB Gateway and sending requests,  and [**Wrapper**](src/core/wrapper.rs), a trait that clients will implement that declares callback functions
that get called when the application receives messages from TWS/IB Gateway.

## Example

In the example below, TWS will send the next valid order ID when the sample application connects. This will cause the ***Wrapper*** callback method
***next_valid_id*** to be called, which will start sending test requests to TWS (see the
***start_requests*** method in ***TestWrapper*** which is called by ***next_valid_id***).

```rust, no_run
use twsapi::core::errors::IBKRApiLibError;
use twsapi::core::client::EClient;
use std::time::Duration;
use twsapi::examples::test_helpers::TestWrapper;
use std::sync::{Arc, Mutex};
use std::thread;

fn main() -> Result<(), IBKRApiLibError> {
    let wrapper = Arc::new(Mutex::new(TestWrapper::new()));
    let app = Arc::new(Mutex::new(EClient::new(wrapper.clone())));

    println!("getting connection...");

    wrapper.lock().expect("Wrapper mutex was poisoned").client = Option::from(app.clone());

    app.lock()
        .expect("EClient mutex was poisoned")
        .connect("127.0.0.1", 4002, 0)?;

    thread::sleep(Duration::new(18600, 0));

    Ok(())
}
```

## TODO

- [X] Expand documentation - Done
- [ ] Write automated tests - in progress
- [ ] Write an async function in TestWrapper that checks when next_valid_id has been populated by the callback
- [ ] Publish to crates.io

If you find a bug or would like to suggest changes, please contact me at brett.miller@sparkstart.com or submit a pull
request.

## DISCLAIMER

THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
