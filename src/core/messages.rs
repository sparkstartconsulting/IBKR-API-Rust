//! Functions for processing messages
use std::any::Any;
use std::convert::TryInto;
use std::io::Write;
use std::string::String;

use std::vec::Vec;

use ascii;
use ascii::AsAsciiStr;

use log::*;
use num_derive::FromPrimitive;

use crate::core::common::{UNSET_DOUBLE, UNSET_INTEGER};
use crate::core::errors::IBKRApiLibError;

//==================================================================================================
trait EClientMsgSink {
    fn server_version(version: i32, time: &str);
    fn redirect(host: &str);
}

//==================================================================================================
/// FA msg data types
pub enum FAMessageDataTypes {
    Groups = 1,
    Profiles = 2,
    Aliases = 3,
}

//==================================================================================================
/// incoming msg id's
#[derive(FromPrimitive)]
#[repr(i32)]
pub enum IncomingMessageIds {
    TickPrice = 1,
    TickSize = 2,
    OrderStatus = 3,
    ErrMsg = 4,
    OpenOrder = 5,
    AcctValue = 6,
    PortfolioValue = 7,
    AcctUpdateTime = 8,
    NextValidId = 9,
    ContractData = 10,
    ExecutionData = 11,
    MarketDepth = 12,
    MarketDepthL2 = 13,
    NewsBulletins = 14,
    ManagedAccts = 15,
    ReceiveFa = 16,
    HistoricalData = 17,
    BondContractData = 18,
    ScannerParameters = 19,
    ScannerData = 20,
    TickOptionComputation = 21,
    TickGeneric = 45,
    TickString = 46,
    TickEfp = 47,
    CurrentTime = 49,
    RealTimeBars = 50,
    FundamentalData = 51,
    ContractDataEnd = 52,
    OpenOrderEnd = 53,
    AcctDownloadEnd = 54,
    ExecutionDataEnd = 55,
    DeltaNeutralValidation = 56,
    TickSnapshotEnd = 57,
    MarketDataType = 58,
    CommissionReport = 59,
    PositionData = 61,
    PositionEnd = 62,
    AccountSummary = 63,
    AccountSummaryEnd = 64,
    VerifyMessageApi = 65,
    VerifyCompleted = 66,
    DisplayGroupList = 67,
    DisplayGroupUpdated = 68,
    VerifyAndAuthMessageApi = 69,
    VerifyAndAuthCompleted = 70,
    PositionMulti = 71,
    PositionMultiEnd = 72,
    AccountUpdateMulti = 73,
    AccountUpdateMultiEnd = 74,
    SecurityDefinitionOptionParameter = 75,
    SecurityDefinitionOptionParameterEnd = 76,
    SoftDollarTiers = 77,
    FamilyCodes = 78,
    SymbolSamples = 79,
    MktDepthExchanges = 80,
    TickReqParams = 81,
    SmartComponents = 82,
    NewsArticle = 83,
    TickNews = 84,
    NewsProviders = 85,
    HistoricalNews = 86,
    HistoricalNewsEnd = 87,
    HeadTimestamp = 88,
    HistogramData = 89,
    HistoricalDataUpdate = 90,
    RerouteMktDataReq = 91,
    RerouteMktDepthReq = 92,
    MarketRule = 93,
    Pnl = 94,
    PnlSingle = 95,
    HistoricalTicks = 96,
    HistoricalTicksBidAsk = 97,
    HistoricalTicksLast = 98,
    TickByTick = 99,
    OrderBound = 100,
    CompletedOrder = 101,
    CompletedOrdersEnd = 102,
}

//==================================================================================================
/// Outgoing msg id's
#[derive(FromPrimitive)]
#[repr(i32)]
pub enum OutgoingMessageIds {
    ReqMktData = 1,
    CancelMktData = 2,
    PlaceOrder = 3,
    CancelOrder = 4,
    ReqOpenOrders = 5,
    ReqAcctData = 6,
    ReqExecutions = 7,
    ReqIds = 8,
    ReqContractData = 9,
    ReqMktDepth = 10,
    CancelMktDepth = 11,
    ReqNewsBulletins = 12,
    CancelNewsBulletins = 13,
    SetServerLoglevel = 14,
    ReqAutoOpenOrders = 15,
    ReqAllOpenOrders = 16,
    ReqManagedAccts = 17,
    ReqFa = 18,
    ReplaceFa = 19,
    ReqHistoricalData = 20,
    ExerciseOptions = 21,
    ReqScannerSubscription = 22,
    CancelScannerSubscription = 23,
    ReqScannerParameters = 24,
    CancelHistoricalData = 25,
    ReqCurrentTime = 49,
    ReqRealTimeBars = 50,
    CancelRealTimeBars = 51,
    ReqFundamentalData = 52,
    CancelFundamentalData = 53,
    ReqCalcImpliedVolat = 54,
    ReqCalcOptionPrice = 55,
    CancelCalcImpliedVolat = 56,
    CancelCalcOptionPrice = 57,
    ReqGlobalCancel = 58,
    ReqMarketDataType = 59,
    ReqPositions = 61,
    ReqAccountSummary = 62,
    CancelAccountSummary = 63,
    CancelPositions = 64,
    VerifyRequest = 65,
    VerifyMessage = 66,
    QueryDisplayGroups = 67,
    SubscribeToGroupEvents = 68,
    UpdateDisplayGroup = 69,
    UnsubscribeFromGroupEvents = 70,
    StartApi = 71,
    VerifyAndAuthRequest = 72,
    VerifyAndAuthMessage = 73,
    ReqPositionsMulti = 74,
    CancelPositionsMulti = 75,
    ReqAccountUpdatesMulti = 76,
    CancelAccountUpdatesMulti = 77,
    ReqSecDefOptParams = 78,
    ReqSoftDollarTiers = 79,
    ReqFamilyCodes = 80,
    ReqMatchingSymbols = 81,
    ReqMktDepthExchanges = 82,
    ReqSmartComponents = 83,
    ReqNewsArticle = 84,
    ReqNewsProviders = 85,
    ReqHistoricalNews = 86,
    ReqHeadTimestamp = 87,
    ReqHistogramData = 88,
    CancelHistogramData = 89,
    CancelHeadTimestamp = 90,
    ReqMarketRule = 91,
    ReqPnl = 92,
    CancelPnl = 93,
    ReqPnlSingle = 94,
    CancelPnlSingle = 95,
    ReqHistoricalTicks = 96,
    ReqTickByTickData = 97,
    CancelTickByTickData = 98,
    ReqCompletedOrders = 99,
}

//==================================================================================================
pub fn make_message(msg: &str) -> Result<Vec<u8>, IBKRApiLibError> {
    //let mut buffer = ByteBuffer::new();
    let mut buffer: Vec<u8> = Vec::new();

    buffer.extend_from_slice(&i32::to_be_bytes(msg.len() as i32));

    buffer.write_all(msg.as_ascii_str().unwrap().as_bytes())?;
    let tmp = buffer.clone();
    //debug!("Message after create: {:?}", buffer);

    let (_size, _msg, _buf) = read_msg(tmp.as_slice())?;
    //debug!("Message read: size:{}, msg:{}, bytes: {:?}", size, msg, buf);

    Ok(tmp)
}

//==================================================================================================
pub fn read_msg(buf: &[u8]) -> Result<(usize, String, Vec<u8>), IBKRApiLibError> {
    // first the size prefix and then the corresponding msg payload ""

    if buf.len() < 4 {
        debug!("read_msg:  buffer too small!! {:?}", buf.len());
        return Ok((0, String::new(), buf.to_vec()));
    }

    let size = i32::from_be_bytes(buf[0..4].try_into().unwrap()) as usize;
    //debug!("read_msg: Message size: {:?}", size);

    if buf.len() - 4 >= size {
        let text = String::from_utf8(buf[4..4 + size].to_vec()).unwrap();
        //debug!("read_msg: text in read message: {:?}", text);
        Ok((size, text, buf[4 + size..].to_vec()))
    } else {
        Ok((size, String::new(), buf.to_vec()))
    }
}

//==================================================================================================
pub fn read_fields(buf: &str) -> Vec<String> {
    //msg payload is made of fields terminated/separated by NULL chars
    let a = '\u{0}';
    let mut fields: Vec<&str> = buf.split(a).collect::<Vec<&str>>();
    //debug!("fields.len() in read_fields: {}", fields.len());
    //last one is empty
    fields.remove(fields.len() - 1);

    fields
        .iter()
        .map(|x| String::from(*x))
        .collect::<Vec<String>>()
}

//==================================================================================================
/// OH BOY THIS IS UGGLY
pub fn make_field(val: &dyn Any) -> Result<String, IBKRApiLibError> {
    // debug!("CALLING make_field!!");
    // adds the NULL string terminator
    let mut field = "\0".to_string();
    // bool type is encoded as int
    if let Some(boolval) = val.downcast_ref::<bool>() {
        field = format!("{}\0", *boolval as i32);
    } else if let Some(stringval) = val.downcast_ref::<usize>() {
        field = format!("{}\0", *stringval as i32);
    } else if let Some(stringval) = val.downcast_ref::<f64>() {
        if UNSET_DOUBLE == *stringval {
            field = format!("{}\0", "");
        } else {
            field = format!("{}\0", *stringval as f64);
        }
    } else if let Some(stringval) = val.downcast_ref::<i32>() {
        if UNSET_INTEGER == *stringval {
            field = format!("{}\0", "");
        } else {
            field = format!("{}\0", *stringval as i32);
        }
    } else if let Some(stringval) = val.downcast_ref::<String>() {
        field = format!("{}\0", stringval);
    } else if let Some(stringval) = val.downcast_ref::<&str>() {
        field = format!("{}\0", stringval);
    }

    Ok(field)
}

//==================================================================================================
pub fn make_field_handle_empty(val: &dyn Any) -> Result<String, IBKRApiLibError> {
    if let Some(stringval) = val.downcast_ref::<f64>() {
        if UNSET_DOUBLE == *stringval {
            return make_field(&"");
        }
    } else if let Some(stringval) = val.downcast_ref::<i32>() {
        if UNSET_INTEGER == *stringval {
            return make_field(&"");
        }
    }

    make_field(val)
}
