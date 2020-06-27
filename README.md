# IBKR-API-Rust
Port of Interactive Broker's trading API written in Rust (API_Version=9.76.01)

Please see the latest IB Tws Api documentation here: http://interactivebrokers.github.io/tws-api/introduction.html.
The documentation has information regarding configuring Trader WorkStation and IB Gateway to enable API access.

For usage of this library, please see the example implementation in src/bin/manual_tests.rs

The main structs and traits that clients will use are <b>EClient</b> , a struct that is responsible for 
connecting to TWS or IB Gateway and sending requests,  and <b>Wrapper</b>, a trait that clients will implement that declares callback functions 
that get called when the application receives messages from the server.

<b>Example of using EClient and Wrapper:</b>

Upon connecting, TWS will send the next valid order ID which will cause the wrapper callback method
next_valid_id to be called, which will start sending tests requests to TWS (see the
<i>start_requests</i> function in <i>TestWrapper</i> which is called by <i>next_valid_id</i>.

    // TestWrapper implements the Wrapper trait and handles messages sent from TWS
    let wrapper = Arc::new(Mutex::new(TestWrapper::new()));
    
    //EClient sends requests to TWS
    let app = Arc::new(Mutex::new(EClient::new(wrapper.clone())));

    info!("getting connection...");
    wrapper.lock().unwrap().client = Option::from(app.clone());

    // Upon connecting, TWS will send the next valid order ID which will cause the wrapper callback method
    // next_valid_id to be called, which will start sending tests requests to TWS (see the
    // start_requests function in TestWrapper which is called by next_valid_id
    app.lock().unwrap().connect("127.0.0.1", 7497, 0);

    thread::sleep(Duration::new(18600, 0));

    Ok(())
    
    
### TODO:
* Expand documentation
* Write automated tests
* Write an async function in TestWrapper that checks when next_valid_id has been populated by the callback

If you find a bug or would like to suggest changes, please contact me at brett.miller@sparkstart.com or submit a pull 
request.

# DISCLAIMER:

THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
