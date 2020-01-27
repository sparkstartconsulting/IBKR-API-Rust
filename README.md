# IBKR-API-Rust
Port of Interactive Broker's trading API written in Rust (API_Version=9.76.01)

#####This is a derivative work of the Interactive Brokers API and falls under their non-commercial or commercial licenses.

Please see the latest IB Tws Api documentation here: http://http://interactivebrokers.github.io/tws-api/introduction.html

For usage, please see the example implementation in src/bin.manual_tests.rs

Example of using client and wrapper.
Upon connecting, TWS will send the next valid order ID which will cause the wrapper callback method
next_valid_id to be called, which will start sending tests requests to TWS (see the
start_requests function inn TestWrapper which is called by next_valid_id.

//=================================================================================
fn main() -> Result<(), IBKRApiLibError> {
    log4rs::init_file("log_config.yml", Default::default()).unwrap();

    let wrapper = Arc::new(Mutex::new(TestWrapper::new()));
    let app = Arc::new(Mutex::new(EClient::new(wrapper.clone())));

    info!("getting connection...");
    wrapper.lock().unwrap().client = Option::from(app.clone());

    // Upon connecting, TWS will send the next valid order ID 
    // which will cause the wrapper callback method
    // next_valid_id to be called, which will start sending tests requests to TWS (see the
    // start_requests function inn TestWrapper which is called by next_valid_id
    app.lock().unwrap().connect("127.0.0.1", 7497, 0);

    thread::sleep(Duration::new(18600, 0));

    Ok(())
}


####TODO:
* Expand documentation
* Write automated tests
* Write an async function in TestWrapper that checks when next_valid_id has been populated by the callback

#DISCLAIMER:

THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
