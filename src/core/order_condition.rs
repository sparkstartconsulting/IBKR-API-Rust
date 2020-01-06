use std::borrow::Borrow;
use std::fmt::{Debug, Display};
use std::slice::Iter;

use num_traits::FromPrimitive;
use serde::export::fmt::Error;
use serde::export::Formatter;
use serde::{Deserialize, Serialize};

use crate::core::decoder::{decode_bool, decode_i32, decode_string};
use crate::core::errors::IBKRApiLibError;
use crate::core::messages::{make_field, make_message};
use crate::core::order_condition::ConditionType::{
    Execution, Margin, PercentChange, Price, Time, Volume,
};

//==================================================================================================
#[repr(i32)]
#[derive(Serialize, Deserialize, Clone, FromPrimitive, Copy)]
pub enum ConditionType {
    Price = 1,
    Time = 3,
    Margin = 4,
    Execution = 5,
    Volume = 6,
    PercentChange = 7,
}

impl Default for ConditionType {
    fn default() -> Self {
        ConditionType::Price
    }
}

impl Display for ConditionType {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match *self {
            ConditionType::Price => write!(f, "Price"),
            ConditionType::Time => write!(f, "Time"),
            ConditionType::Margin => write!(f, "Margin"),
            ConditionType::Execution => write!(f, "Execution"),
            ConditionType::Volume => write!(f, "Volume"),
            ConditionType::PercentChange => write!(f, "PercentChange"),
        }
    }
}

impl Debug for ConditionType {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{} = {}", self.to_string(), *self as i32)
    }
}

//==================================================================================================
#[repr(i32)]
#[derive(Serialize, Deserialize, Clone, FromPrimitive, Copy)]
pub enum TriggerMethod {
    Default = 0,
    DoubleBidAsk = 1,
    Last = 2,
    DoubleLast = 3,
    BidAsk = 4,
    NA1 = 5,
    NA2 = 6,
    LastBidAsk = 7,
    MidPoint = 8,
}

impl Default for TriggerMethod {
    fn default() -> Self {
        TriggerMethod::Default
    }
}

impl Display for TriggerMethod {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match *self {
            TriggerMethod::Default => write!(f, "Default"),
            TriggerMethod::DoubleBidAsk => write!(f, "DoubleBidAsk"),
            TriggerMethod::Last => write!(f, "Last"),
            TriggerMethod::DoubleLast => write!(f, "DoubleLast"),
            TriggerMethod::BidAsk => write!(f, "BidAsk"),
            TriggerMethod::NA1 => write!(f, "NA1"),
            TriggerMethod::NA2 => write!(f, "NA2"),
            TriggerMethod::LastBidAsk => write!(f, "LastBidAsk"),
            TriggerMethod::MidPoint => write!(f, "MidPoint"),
        }
    }
}

impl Debug for TriggerMethod {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{} = {}", self.to_string(), *self as i32)
    }
}

//==================================================================================================
#[derive(Serialize, Deserialize, Clone)]
pub enum OrderConditionEnum {
    Price {
        price_condition: PriceCondition,
    },
    Time {
        time_condition: TimeCondition,
    },
    Margin {
        margin_condition: MarginCondition,
    },
    Execution {
        execution_condition: ExecutionCondition,
    },
    Volume {
        volume_condition: VolumeCondition,
    },
    PercentChange {
        percent_change_condition: PercentChangeCondition,
    },
}

impl OrderConditionEnum {
    pub fn get_condition(&self) -> Box<dyn Condition> {
        match (self) {
            OrderConditionEnum::Price { price_condition } => Box::new(price_condition.clone()),
            OrderConditionEnum::Time { time_condition } => Box::new(time_condition.clone()),
            OrderConditionEnum::Margin { margin_condition } => Box::new(margin_condition.clone()),
            OrderConditionEnum::Execution {
                execution_condition,
            } => Box::new(execution_condition.clone()),
            OrderConditionEnum::Volume { volume_condition } => Box::new(volume_condition.clone()),
            OrderConditionEnum::PercentChange {
                percent_change_condition,
            } => Box::new(percent_change_condition.clone()),
        }
    }
}

impl Display for OrderConditionEnum {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}", self.get_condition().value_to_string())
    }
}

impl Debug for OrderConditionEnum {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match (self) {
            OrderConditionEnum::Execution {
                execution_condition,
            } => write!(
                f,
                "{} = {}",
                "Execution",
                execution_condition.value_to_string()
            ),
            OrderConditionEnum::Price { price_condition } => {
                write!(f, "{} = {}", "Price", price_condition.value_to_string())
            }
            OrderConditionEnum::Margin { margin_condition } => {
                write!(f, "{} = {}", "Margin", margin_condition.value_to_string())
            }
            OrderConditionEnum::Time { time_condition } => {
                write!(f, "{} = {}", "Time", time_condition.value_to_string())
            }
            OrderConditionEnum::Volume { volume_condition } => {
                write!(f, "{} = {}", "Volume", volume_condition.value_to_string())
            }
            OrderConditionEnum::PercentChange {
                percent_change_condition,
            } => write!(
                f,
                "{} = {}",
                "Percentage Change",
                percent_change_condition.value_to_string()
            ),
        }
    }
}

pub trait Condition: Display + Debug {
    fn decode(&mut self, fields_iter: &mut Iter<String>) -> Result<(), IBKRApiLibError>;
    fn make_fields(&self) -> Vec<String>;
    fn value_to_string(&self) -> String;
    fn set_value_from_string(&mut self, text: String);
    fn get_type(&self) -> ConditionType;
}

//==================================================================================================
#[derive(Serialize, Deserialize, Clone, Debug, Copy, Default)]
pub struct OrderCondition {
    pub cond_type: ConditionType,
    is_conjunction_connection: bool,
}

impl OrderCondition {
    pub fn new(cond_type: ConditionType) -> Self {
        OrderCondition {
            cond_type: cond_type,
            is_conjunction_connection: false,
        }
    }

    pub fn get_type(&self) -> ConditionType {
        self.cond_type
    }

    pub fn and(&mut self) -> &OrderCondition {
        self.is_conjunction_connection = true;
        self
    }

    pub fn or(&mut self) -> &OrderCondition {
        self.is_conjunction_connection = false;
        self
    }

    pub fn decode(&mut self, fields_iter: &mut Iter<String>) -> Result<(), IBKRApiLibError> {
        let connector = decode_string(fields_iter)?;
        self.is_conjunction_connection = connector == "a";
        Ok(())
    }

    pub fn make_fields(&self) -> Vec<String> {
        let mut flds = vec![];
        let val = if self.is_conjunction_connection {
            "a"
        } else {
            "o"
        };
        flds.push(make_field(&val.to_string()));
        flds
    }
}

//pub fn  __str__(self):
//return "<AND>" if self.is_conjunction_connection else "<OR>"
#[derive(Serialize, Deserialize, Clone, Default)]
pub struct ExecutionCondition {
    sec_type: String,
    exchange: String,
    symbol: String,
    order_condition: OrderCondition,
}

impl ExecutionCondition {
    pub fn new(sec_type: String, exchange: String, symbol: String) -> Self {
        ExecutionCondition {
            sec_type: sec_type,
            exchange: exchange,
            symbol: symbol,
            order_condition: OrderCondition::new(ConditionType::Execution),
        }
    }
}

impl Condition for ExecutionCondition {
    fn decode(&mut self, fields_iter: &mut Iter<String>) -> Result<(), IBKRApiLibError> {
        self.order_condition.decode(fields_iter);
        self.sec_type = decode_string(fields_iter)?;
        self.exchange = decode_string(fields_iter)?;
        self.symbol = decode_string(fields_iter)?;
        Ok(())
    }

    fn make_fields(&self) -> Vec<String> {
        let mut flds = self.order_condition.make_fields();
        flds.push(make_field(&self.sec_type));
        flds.push(make_field(&self.exchange));
        flds.push(make_field(&self.symbol));
        flds
    }

    fn value_to_string(&self) -> String {
        format!(
            "sec_type: {}, exchange: {}, symbol: {}",
            self.sec_type, self.exchange, self.symbol
        )
    }

    fn set_value_from_string(&mut self, text: String) {
        unimplemented!()
    }

    fn get_type(&self) -> ConditionType {
        self.order_condition.cond_type
    }
}

impl Display for ExecutionCondition {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}", self.value_to_string())
    }
}

impl Debug for ExecutionCondition {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}", self.value_to_string())
    }
}

//pub fn  __str__(self):
//return "trade occurs for " + self.symbol + " symbol on " + \
//self.exchange + " exchange for " + self.secType + " security type"
#[derive(Serialize, Deserialize, Clone, Debug, Copy, Default)]
pub struct OperatorCondition {
    order_condition: OrderCondition,
    is_moore: bool,
}

impl OperatorCondition {
    pub fn new(cond_type: ConditionType, is_more: bool) -> Self {
        OperatorCondition {
            order_condition: OrderCondition::new(cond_type),
            is_moore: is_more,
        }
    }

    pub fn value_to_string(&self) -> String {
        unimplemented!();
    }

    pub fn set_value_from_string(&self, text: &str) {
        unimplemented!();
    }

    pub fn decode(&mut self, fields_iter: &mut Iter<String>) -> Result<(), IBKRApiLibError> {
        self.order_condition.decode(fields_iter)?;
        self.is_moore = decode_bool(fields_iter)?;
        let text = decode_string(fields_iter)?;
        self.set_value_from_string(text.as_ref());
        Ok(())
    }

    pub fn make_fields(&self) -> Vec<String> {
        let mut flds = self.order_condition.make_fields();
        flds.push(make_field(&self.is_moore));
        flds.push(make_field(&self.value_to_string()));
        flds
    }

    //pub fn  __str__(self):
    //sb = ">= " if self.isMore else "<= "
    //return " %s %s" % (sb, self.value_to_string())
    //}
}

#[derive(Serialize, Deserialize, Clone, Copy, Default)]
pub struct MarginCondition {
    operator_condition: OperatorCondition,
    percent: f64,
}

impl MarginCondition {
    pub fn new(is_more: bool, percent: f64) -> Self {
        MarginCondition {
            operator_condition: OperatorCondition::new(ConditionType::Margin, is_more),
            percent: percent,
        }
    }
}

impl Condition for MarginCondition {
    fn decode(&mut self, fields_iter: &mut Iter<String>) -> Result<(), IBKRApiLibError> {
        self.operator_condition.decode(fields_iter)
    }

    fn make_fields(&self) -> Vec<String> {
        let flds = self.operator_condition.make_fields();
        flds
    }

    fn value_to_string(&self) -> String {
        self.percent.to_string()
    }

    fn set_value_from_string(&mut self, text: String) {
        self.percent = text.parse().unwrap();
    }

    fn get_type(&self) -> ConditionType {
        self.operator_condition.order_condition.cond_type
    }
}

impl Display for MarginCondition {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}", self.value_to_string())
    }
}

impl Debug for MarginCondition {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "The margin cushion percent {}", self.value_to_string())
    }
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct ContractCondition {
    operator_condition: OperatorCondition,
    con_id: i32,
    exchange: String,
}

impl ContractCondition {
    pub fn new(cond_type: ConditionType, con_id: i32, exchange: &str, is_more: bool) -> Self {
        ContractCondition {
            operator_condition: OperatorCondition::new(cond_type, is_more),
            con_id: con_id,
            exchange: exchange.to_string(),
        }
    }
}

impl Condition for ContractCondition {
    fn decode(&mut self, fields_iter: &mut Iter<String>) -> Result<(), IBKRApiLibError> {
        self.operator_condition.decode(fields_iter)?;
        self.con_id = decode_i32(fields_iter)?;
        self.exchange = decode_string(fields_iter)?;
        Ok(())
    }

    fn make_fields(&self) -> Vec<String> {
        let mut flds = self.operator_condition.make_fields();
        flds.push(make_field(&self.con_id));
        flds.push(make_field(&self.exchange));
        flds
    }

    fn value_to_string(&self) -> String {
        format!("contract id: {}, Exchange: {}", self.con_id, self.exchange)
    }

    fn set_value_from_string(&mut self, text: String) {
        unimplemented!()
    }

    fn get_type(&self) -> ConditionType {
        self.operator_condition.order_condition.cond_type
    }
}

impl Display for ContractCondition {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}", self.value_to_string())
    }
}

impl Debug for ContractCondition {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}", self.value_to_string())
    }
}
//pub fn  __str__(self):
//return "%s on %s is %s " % (self.conId, self.exchange,
//OperatorCondition.__str__(self))

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct TimeCondition {
    operator_condition: OperatorCondition,
    time: String,
}

impl TimeCondition {
    pub fn new(is_more: bool, time: String) -> Self {
        TimeCondition {
            operator_condition: OperatorCondition::new(ConditionType::Time, is_more),
            time: time,
        }
    }
}

impl Condition for TimeCondition {
    fn decode(&mut self, fields_iter: &mut Iter<String>) -> Result<(), IBKRApiLibError> {
        self.operator_condition.decode(fields_iter)
    }

    fn make_fields(&self) -> Vec<String> {
        let flds = self.operator_condition.make_fields();
        return flds;
    }

    fn value_to_string(&self) -> String {
        self.time.clone()
    }

    fn set_value_from_string(&mut self, text: String) {
        self.time = text.to_string()
    }

    fn get_type(&self) -> ConditionType {
        self.operator_condition.order_condition.cond_type
    }
}

impl Display for TimeCondition {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}", self.value_to_string())
    }
}

impl Debug for TimeCondition {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "TimeCondition = {}", self.value_to_string())
    }
}

//pub fn  __str__(self):
//return "time is %s " % (OperatorCondition.__str__(self))

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct PriceCondition {
    contract_condition: ContractCondition,
    price: f64,
    trigger_method: TriggerMethod,
}

impl PriceCondition {
    pub fn new(
        trigger_method: TriggerMethod,
        con_id: i32,
        exchange: &str,
        is_more: bool,
        price: f64,
    ) -> Self {
        PriceCondition {
            contract_condition: ContractCondition::new(
                ConditionType::Price,
                con_id,
                exchange,
                is_more,
            ),
            price: price,
            trigger_method: trigger_method,
        }
    }
}

impl Condition for PriceCondition {
    fn decode(&mut self, fields_iter: &mut Iter<String>) -> Result<(), IBKRApiLibError> {
        self.contract_condition.decode(fields_iter)?;
        self.trigger_method = FromPrimitive::from_i32(decode_i32(fields_iter)?).unwrap();
        Ok(())
    }

    fn make_fields(&self) -> Vec<String> {
        let mut flds = self.contract_condition.make_fields();
        flds.push(make_field(&(self.trigger_method as i32)));
        flds
    }

    fn value_to_string(&self) -> String {
        self.price.to_string()
    }

    fn set_value_from_string(&mut self, text: String) {
        self.price = text.parse().unwrap();
    }

    fn get_type(&self) -> ConditionType {
        self.contract_condition
            .operator_condition
            .order_condition
            .cond_type
    }
}

impl Display for PriceCondition {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(
            f,
            "{} price of {} ",
            self.trigger_method.to_string(),
            self.value_to_string()
        )
    }
}
//pub fn  __str__(self):
//return "%s price of %s " % (
//price_condition.TriggerMethodEnum.to_str(self.triggerMethod),
//ContractCondition.__str__(self))

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct PercentChangeCondition {
    contract_condition: ContractCondition,
    change_percent: f64,
}

impl PercentChangeCondition {
    pub fn new(con_id: i32, exchange: String, is_more: bool, change_percent: f64) -> Self {
        PercentChangeCondition {
            contract_condition: ContractCondition::new(
                ConditionType::PercentChange,
                con_id,
                exchange.as_ref(),
                is_more,
            ),
            change_percent: change_percent,
        }
    }
}

impl Condition for PercentChangeCondition {
    fn decode(&mut self, fields_iter: &mut Iter<String>) -> Result<(), IBKRApiLibError> {
        self.contract_condition.decode(fields_iter)
    }

    fn make_fields(&self) -> Vec<String> {
        let flds = self.contract_condition.make_fields();
        flds
    }

    fn value_to_string(&self) -> String {
        self.change_percent.to_string()
    }

    fn set_value_from_string(&mut self, text: String) {
        self.change_percent = text.parse().unwrap();
    }

    fn get_type(&self) -> ConditionType {
        self.contract_condition
            .operator_condition
            .order_condition
            .cond_type
    }
}

impl Display for PercentChangeCondition {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}", self.value_to_string())
    }
}

impl Debug for PercentChangeCondition {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "percent change of {}", self.value_to_string())
    }
}
//pub fn  __str__(self):
//return "percent change of %s " % (
//ContractCondition.__str__(self))

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct VolumeCondition {
    contract_condition: ContractCondition,
    volume: i32,
}

impl VolumeCondition {
    pub fn new(con_id: i32, exchange: &str, is_more: bool, volume: i32) -> Self {
        VolumeCondition {
            contract_condition: ContractCondition::new(
                ConditionType::Volume,
                con_id,
                exchange,
                is_more,
            ),
            volume: volume,
        }
    }
}

impl Condition for VolumeCondition {
    fn decode(&mut self, fields_iter: &mut Iter<String>) -> Result<(), IBKRApiLibError> {
        self.contract_condition.decode(fields_iter)
    }

    fn make_fields(&self) -> Vec<String> {
        let flds = self.contract_condition.make_fields();
        flds
    }

    fn value_to_string(&self) -> String {
        self.volume.to_string()
    }

    fn set_value_from_string(&mut self, text: String) {
        self.volume = text.parse().unwrap();
    }

    fn get_type(&self) -> ConditionType {
        self.contract_condition
            .operator_condition
            .order_condition
            .cond_type
    }
}

impl Display for VolumeCondition {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}", self.value_to_string())
    }
}

impl Debug for VolumeCondition {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "volume of {}", self.value_to_string())
    }
}
//pub fn  __str__(self):
//return "volume of %s " % (
//ContractCondition.__str__(self))

pub fn create(cond_type: ConditionType) -> OrderConditionEnum {
    match (cond_type) {
        ConditionType::Execution => OrderConditionEnum::Execution {
            execution_condition: Default::default(),
        },
        ConditionType::Margin => OrderConditionEnum::Margin {
            margin_condition: Default::default(),
        },
        ConditionType::PercentChange => OrderConditionEnum::PercentChange {
            percent_change_condition: Default::default(),
        },
        ConditionType::Price => OrderConditionEnum::PercentChange {
            percent_change_condition: Default::default(),
        },
        ConditionType::Time => OrderConditionEnum::Time {
            time_condition: Default::default(),
        },
        ConditionType::Volume => OrderConditionEnum::Volume {
            volume_condition: Default::default(),
        },
    }
}
