//! Types related to order and execution conditions
use std::fmt::{Debug, Display, Error, Formatter};
use std::slice::Iter;

use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

use serde::{Deserialize, Serialize};

use crate::core::decoder::{decode_bool, decode_f64, decode_i32, decode_string};
use crate::core::errors::IBKRApiLibError;
use crate::core::messages::make_field;

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
        write!(f, "{} = {}", self, *self as i32)
    }
}

//==================================================================================================
#[derive(Serialize, Deserialize, Clone)]
pub enum OrderConditionEnum {
    Price(PriceCondition),
    Time(TimeCondition),
    Margin(MarginCondition),
    Execution(ExecutionCondition),
    Volume(VolumeCondition),
    PercentChange(PercentChangeCondition),
}

impl Condition for OrderConditionEnum {
    fn decode(&mut self, fields_iter: &mut Iter<String>) -> Result<(), IBKRApiLibError> {
        match self {
            OrderConditionEnum::Execution(s) => s.decode(fields_iter),
            OrderConditionEnum::Price(p) => p.decode(fields_iter),
            OrderConditionEnum::Margin(m) => m.decode(fields_iter),
            OrderConditionEnum::Time(t) => t.decode(fields_iter),
            OrderConditionEnum::Volume(v) => v.decode(fields_iter),
            OrderConditionEnum::PercentChange(pch) => pch.decode(fields_iter),
        }
    }

    //----------------------------------------------------------------------------------------------
    fn make_fields(&self) -> Result<Vec<String>, IBKRApiLibError> {
        match self {
            OrderConditionEnum::Execution(s) => s.make_fields(),
            OrderConditionEnum::Price(p) => p.make_fields(),
            OrderConditionEnum::Margin(m) => m.make_fields(),
            OrderConditionEnum::Time(t) => t.make_fields(),
            OrderConditionEnum::Volume(v) => v.make_fields(),
            OrderConditionEnum::PercentChange(pch) => pch.make_fields(),
        }
    }

    //----------------------------------------------------------------------------------------------
    fn value_to_string(&self) -> String {
        match self {
            OrderConditionEnum::Execution(s) => s.value_to_string(),
            OrderConditionEnum::Price(p) => p.value_to_string(),
            OrderConditionEnum::Margin(m) => m.value_to_string(),
            OrderConditionEnum::Time(t) => t.value_to_string(),
            OrderConditionEnum::Volume(v) => v.value_to_string(),
            OrderConditionEnum::PercentChange(pch) => pch.value_to_string(),
        }
    }

    //----------------------------------------------------------------------------------------------
    fn set_value_from_string(&mut self, text: String) {
        match self {
            OrderConditionEnum::Execution(s) => s.set_value_from_string(text),
            OrderConditionEnum::Price(p) => p.set_value_from_string(text),
            OrderConditionEnum::Margin(m) => m.set_value_from_string(text),
            OrderConditionEnum::Time(t) => t.set_value_from_string(text),
            OrderConditionEnum::Volume(v) => v.set_value_from_string(text),
            OrderConditionEnum::PercentChange(pch) => pch.set_value_from_string(text),
        }
    }

    //----------------------------------------------------------------------------------------------
    fn get_type(&self) -> ConditionType {
        match self {
            OrderConditionEnum::Execution(s) => s.get_type(),
            OrderConditionEnum::Price(p) => p.get_type(),
            OrderConditionEnum::Margin(m) => m.get_type(),
            OrderConditionEnum::Time(t) => t.get_type(),
            OrderConditionEnum::Volume(v) => v.get_type(),
            OrderConditionEnum::PercentChange(pch) => pch.get_type(),
        }
    }
}

impl Display for OrderConditionEnum {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}", self.value_to_string())
    }
}

impl Debug for OrderConditionEnum {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            OrderConditionEnum::Execution(s) => {
                write!(f, "Execution = {}", s.value_to_string())
            }
            OrderConditionEnum::Price(p) => write!(f, "Price = {}", p.value_to_string()),
            OrderConditionEnum::Margin(m) => write!(f, "Margin = {}",  m.value_to_string()),
            OrderConditionEnum::Time(t) => write!(f, "Time = {}",  t.value_to_string()),
            OrderConditionEnum::Volume(v) => write!(f, "Volume = {}", v.value_to_string()),
            OrderConditionEnum::PercentChange(pch) => {
                write!(f, "Percentage Change = {}", pch.value_to_string())
            }
        }
    }
}

//==================================================================================================
pub trait Condition: Display + Debug + Serialize {
    fn decode(&mut self, fields_iter: &mut Iter<String>) -> Result<(), IBKRApiLibError>;
    fn make_fields(&self) -> Result<Vec<String>, IBKRApiLibError>;
    fn value_to_string(&self) -> String;
    fn set_value_from_string(&mut self, text: String);
    fn get_type(&self) -> ConditionType;
}

//==================================================================================================
#[derive(Serialize, Deserialize, Clone, Debug, Copy, Default)]
pub struct OrderCondition {
    pub cond_type: ConditionType,
    pub is_conjunction_connection: bool,
}

impl OrderCondition {
    pub fn new(cond_type: ConditionType) -> Self {
        OrderCondition {
            cond_type,
            is_conjunction_connection: false,
        }
    }

    //----------------------------------------------------------------------------------------------
    pub fn get_type(&self) -> ConditionType {
        self.cond_type
    }

    //----------------------------------------------------------------------------------------------
    pub fn and(&mut self) -> &OrderCondition {
        self.is_conjunction_connection = true;
        self
    }

    //----------------------------------------------------------------------------------------------
    pub fn or(&mut self) -> &OrderCondition {
        self.is_conjunction_connection = false;
        self
    }

    //----------------------------------------------------------------------------------------------
    pub fn decode(&mut self, fields_iter: &mut Iter<String>) -> Result<(), IBKRApiLibError> {
        let connector = decode_string(fields_iter)?;
        self.is_conjunction_connection = connector == "a";
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    pub fn make_fields(&self) -> Result<Vec<String>, IBKRApiLibError> {
        let mut flds = vec![];
        let val = if self.is_conjunction_connection {
            "a"
        } else {
            "o"
        };
        flds.push(make_field(&val.to_string())?);
        Ok(flds)
    }
}
//==================================================================================================
#[derive(Serialize, Deserialize, Clone, Default)]
pub struct ExecutionCondition {
    pub sec_type: String,
    pub exchange: String,
    pub symbol: String,
    pub order_condition: OrderCondition,
}

impl ExecutionCondition {
    pub fn new(sec_type: String, exchange: String, symbol: String) -> Self {
        ExecutionCondition {
            sec_type,
            exchange,
            symbol,
            order_condition: OrderCondition::new(ConditionType::Execution),
        }
    }
}

impl Condition for ExecutionCondition {
    //----------------------------------------------------------------------------------------------
    fn decode(&mut self, fields_iter: &mut Iter<String>) -> Result<(), IBKRApiLibError> {
        self.order_condition.decode(fields_iter)?;
        self.sec_type = decode_string(fields_iter)?;
        self.exchange = decode_string(fields_iter)?;
        self.symbol = decode_string(fields_iter)?;
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    fn make_fields(&self) -> Result<Vec<String>, IBKRApiLibError> {
        let mut flds = self.order_condition.make_fields()?;
        flds.push(make_field(&self.sec_type)?);
        flds.push(make_field(&self.exchange)?);
        flds.push(make_field(&self.symbol)?);
        Ok(flds)
    }

    //----------------------------------------------------------------------------------------------
    fn value_to_string(&self) -> String {
        format!(
            "sec_type: {}, exchange: {}, symbol: {}",
            self.sec_type, self.exchange, self.symbol
        )
    }

    //----------------------------------------------------------------------------------------------
    fn set_value_from_string(&mut self, _text: String) {
        unimplemented!()
    }

    //----------------------------------------------------------------------------------------------
    fn get_type(&self) -> ConditionType {
        self.order_condition.cond_type
    }
}

impl Display for ExecutionCondition {
    //----------------------------------------------------------------------------------------------
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}", self.value_to_string())
    }
}

impl Debug for ExecutionCondition {
    //----------------------------------------------------------------------------------------------
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}", self.value_to_string())
    }
}

impl From<OrderConditionEnum> for ExecutionCondition {
    //----------------------------------------------------------------------------------------------
    fn from(_: OrderConditionEnum) -> Self {
        ExecutionCondition::default()
    }
}

//==================================================================================================
#[derive(Serialize, Deserialize, Clone, Debug, Copy, Default)]
pub struct OperatorCondition {
    pub order_condition: OrderCondition,
    pub is_more: bool,
}

impl OperatorCondition {
    pub fn new(cond_type: ConditionType, is_more: bool) -> Self {
        OperatorCondition {
            order_condition: OrderCondition::new(cond_type),
            is_more,
        }
    }

    //----------------------------------------------------------------------------------------------
    pub fn value_to_string(&self) -> String {
        unimplemented!();
    }

    //----------------------------------------------------------------------------------------------
    pub fn set_value_from_string(&self, _text: &str) {
        unimplemented!();
    }

    //----------------------------------------------------------------------------------------------
    pub fn decode(&mut self, fields_iter: &mut Iter<String>) -> Result<(), IBKRApiLibError> {
        self.order_condition.decode(fields_iter)?;
        self.is_more = decode_bool(fields_iter)?;
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    fn make_fields(&self) -> Result<Vec<String>, IBKRApiLibError> {
        let mut flds = self.order_condition.make_fields()?;
        flds.push(make_field(&self.is_more)?);
        Ok(flds)
    }
}

//==================================================================================================
#[derive(Serialize, Deserialize, Clone, Copy, Default)]
pub struct MarginCondition {
    pub operator_condition: OperatorCondition,
    pub percent: f64,
}

impl MarginCondition {
    //----------------------------------------------------------------------------------------------
    pub fn new(is_more: bool, percent: f64) -> Self {
        MarginCondition {
            operator_condition: OperatorCondition::new(ConditionType::Margin, is_more),
            percent,
        }
    }
}

impl Condition for MarginCondition {
    //----------------------------------------------------------------------------------------------
    fn decode(&mut self, fields_iter: &mut Iter<String>) -> Result<(), IBKRApiLibError> {
        self.operator_condition.decode(fields_iter)?;
        self.percent = decode_f64(fields_iter).unwrap();
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    fn make_fields(&self) -> Result<Vec<String>, IBKRApiLibError> {
        let mut flds = self.operator_condition.make_fields()?;
        flds.push(make_field(&self.percent)?);
        Ok(flds)
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
    //----------------------------------------------------------------------------------------------
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}", self.value_to_string())
    }
}

impl Debug for MarginCondition {
    //----------------------------------------------------------------------------------------------
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "The margin cushion percent {}", self.value_to_string())
    }
}

impl From<OrderConditionEnum> for MarginCondition {
    //----------------------------------------------------------------------------------------------
    fn from(_: OrderConditionEnum) -> Self {
        MarginCondition::default()
    }
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct ContractCondition {
    pub operator_condition: OperatorCondition,
    pub con_id: i32,
    pub exchange: String,
}

impl ContractCondition {
    //----------------------------------------------------------------------------------------------
    pub fn new(cond_type: ConditionType, con_id: i32, exchange: &str, is_more: bool) -> Self {
        ContractCondition {
            operator_condition: OperatorCondition::new(cond_type, is_more),
            con_id,
            exchange: exchange.to_string(),
        }
    }
}

impl Condition for ContractCondition {
    //----------------------------------------------------------------------------------------------
    fn decode(&mut self, fields_iter: &mut Iter<String>) -> Result<(), IBKRApiLibError> {
        self.operator_condition.decode(fields_iter)?;
        self.con_id = decode_i32(fields_iter)?;
        self.exchange = decode_string(fields_iter)?;
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    fn make_fields(&self) -> Result<Vec<String>, IBKRApiLibError> {
        let mut flds = self.operator_condition.make_fields()?;
        flds.push(make_field(&self.con_id)?);
        flds.push(make_field(&self.exchange)?);
        Ok(flds)
    }

    //----------------------------------------------------------------------------------------------
    fn value_to_string(&self) -> String {
        format!("contract id: {}, Exchange: {}", self.con_id, self.exchange)
    }

    //----------------------------------------------------------------------------------------------
    fn set_value_from_string(&mut self, _text: String) {
        unimplemented!()
    }

    //----------------------------------------------------------------------------------------------
    fn get_type(&self) -> ConditionType {
        self.operator_condition.order_condition.cond_type
    }
}

impl Display for ContractCondition {
    //----------------------------------------------------------------------------------------------
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}", self.value_to_string())
    }
}

impl Debug for ContractCondition {
    //----------------------------------------------------------------------------------------------
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}", self.value_to_string())
    }
}

impl From<OrderConditionEnum> for ContractCondition {
    //----------------------------------------------------------------------------------------------
    fn from(_: OrderConditionEnum) -> Self {
        ContractCondition::default()
    }
}
//==================================================================================================
#[derive(Serialize, Deserialize, Clone, Default)]
pub struct TimeCondition {
    pub operator_condition: OperatorCondition,
    pub time: String,
}

impl TimeCondition {
    //----------------------------------------------------------------------------------------------
    pub fn new(is_more: bool, time: String) -> Self {
        TimeCondition {
            operator_condition: OperatorCondition::new(ConditionType::Time, is_more),
            time,
        }
    }
}

impl Condition for TimeCondition {
    //----------------------------------------------------------------------------------------------
    fn decode(&mut self, fields_iter: &mut Iter<String>) -> Result<(), IBKRApiLibError> {
        self.operator_condition.decode(fields_iter)?;
        self.time = decode_string(fields_iter).unwrap();
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    fn make_fields(&self) -> Result<Vec<String>, IBKRApiLibError> {
        let mut flds = self.operator_condition.make_fields()?;
        flds.push(make_field(&self.time)?);
        Ok(flds)
    }

    //----------------------------------------------------------------------------------------------
    fn value_to_string(&self) -> String {
        self.time.clone()
    }

    //----------------------------------------------------------------------------------------------
    fn set_value_from_string(&mut self, text: String) {
        self.time = text.to_string()
    }

    //----------------------------------------------------------------------------------------------
    fn get_type(&self) -> ConditionType {
        self.operator_condition.order_condition.cond_type
    }
}

impl Display for TimeCondition {
    //----------------------------------------------------------------------------------------------
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}", self.value_to_string())
    }
}

//==================================================================================================
impl Debug for TimeCondition {
    //----------------------------------------------------------------------------------------------
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "time_condition = {}", self.value_to_string())
    }
}

//==================================================================================================
impl From<OrderConditionEnum> for TimeCondition {
    //----------------------------------------------------------------------------------------------
    fn from(_: OrderConditionEnum) -> Self {
        TimeCondition::default()
    }
}

//==================================================================================================
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct PriceCondition {
    pub contract_condition: ContractCondition,
    pub price: f64,
    pub trigger_method: TriggerMethod,
}

impl PriceCondition {
    //----------------------------------------------------------------------------------------------
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
            price,
            trigger_method,
        }
    }
}

impl Condition for PriceCondition {
    //----------------------------------------------------------------------------------------------
    fn decode(&mut self, fields_iter: &mut Iter<String>) -> Result<(), IBKRApiLibError> {
        self.price = decode_f64(fields_iter)?;
        self.contract_condition.decode(fields_iter)?;
        self.trigger_method = FromPrimitive::from_i32(decode_i32(fields_iter)?).unwrap();
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    fn make_fields(&self) -> Result<Vec<String>, IBKRApiLibError> {
        let mut flds = self
            .contract_condition
            .operator_condition
            .order_condition
            .make_fields()?;
        flds.push(make_field(
            &(self.contract_condition.operator_condition.is_more as i32),
        )?);
        flds.push(make_field(&(self.price))?);
        flds.push(make_field(&(self.contract_condition.con_id))?);
        flds.push(make_field(&(self.contract_condition.exchange))?);
        flds.push(make_field(&(self.trigger_method as i32))?);

        Ok(flds)
    }

    //----------------------------------------------------------------------------------------------
    fn value_to_string(&self) -> String {
        self.price.to_string()
    }

    //----------------------------------------------------------------------------------------------
    fn set_value_from_string(&mut self, text: String) {
        self.price = text.parse().unwrap();
    }

    //----------------------------------------------------------------------------------------------
    fn get_type(&self) -> ConditionType {
        self.contract_condition
            .operator_condition
            .order_condition
            .cond_type
    }
}

impl Display for PriceCondition {
    //----------------------------------------------------------------------------------------------
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(
            f,
            "{} price of {} ",
            self.trigger_method.to_string(),
            self.value_to_string()
        )
    }
}

impl From<OrderConditionEnum> for PriceCondition {
    //----------------------------------------------------------------------------------------------
    fn from(_: OrderConditionEnum) -> Self {
        PriceCondition::default()
    }
}

//==================================================================================================
#[derive(Serialize, Deserialize, Clone, Default)]
pub struct PercentChangeCondition {
    pub contract_condition: ContractCondition,
    pub change_percent: f64,
}

impl PercentChangeCondition {
    //----------------------------------------------------------------------------------------------
    pub fn new(con_id: i32, exchange: String, is_more: bool, change_percent: f64) -> Self {
        PercentChangeCondition {
            contract_condition: ContractCondition::new(
                ConditionType::PercentChange,
                con_id,
                exchange.as_ref(),
                is_more,
            ),
            change_percent,
        }
    }
}

impl Condition for PercentChangeCondition {
    //----------------------------------------------------------------------------------------------
    fn decode(&mut self, fields_iter: &mut Iter<String>) -> Result<(), IBKRApiLibError> {
        self.change_percent = decode_f64(fields_iter)?;
        self.contract_condition.decode(fields_iter)?;
        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    fn make_fields(&self) -> Result<Vec<String>, IBKRApiLibError> {
        let mut flds = self
            .contract_condition
            .operator_condition
            .order_condition
            .make_fields()?;
        flds.push(make_field(
            &(self.contract_condition.operator_condition.is_more as i32),
        )?);
        flds.push(make_field(&(self.change_percent))?);
        flds.push(make_field(&(self.contract_condition.con_id))?);
        flds.push(make_field(&(self.contract_condition.exchange))?);

        Ok(flds)
    }

    //----------------------------------------------------------------------------------------------
    fn value_to_string(&self) -> String {
        self.change_percent.to_string()
    }

    //----------------------------------------------------------------------------------------------
    fn set_value_from_string(&mut self, text: String) {
        self.change_percent = text.parse().unwrap();
    }

    //----------------------------------------------------------------------------------------------
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
    //----------------------------------------------------------------------------------------------
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "percent change of {}", self.value_to_string())
    }
}

impl From<OrderConditionEnum> for PercentChangeCondition {
    //----------------------------------------------------------------------------------------------
    fn from(_: OrderConditionEnum) -> Self {
        PercentChangeCondition::default()
    }
}

//==================================================================================================
#[derive(Serialize, Deserialize, Clone, Default)]
pub struct VolumeCondition {
    pub contract_condition: ContractCondition,
    pub volume: i32,
}

impl VolumeCondition {
    //----------------------------------------------------------------------------------------------
    pub fn new(con_id: i32, exchange: &str, is_more: bool, volume: i32) -> Self {
        VolumeCondition {
            contract_condition: ContractCondition::new(
                ConditionType::Volume,
                con_id,
                exchange,
                is_more,
            ),
            volume,
        }
    }
}

impl Condition for VolumeCondition {
    //----------------------------------------------------------------------------------------------
    fn decode(&mut self, fields_iter: &mut Iter<String>) -> Result<(), IBKRApiLibError> {
        self.contract_condition.decode(fields_iter)?;
        self.volume = decode_i32(fields_iter)?;

        Ok(())
    }

    //----------------------------------------------------------------------------------------------
    fn make_fields(&self) -> Result<Vec<String>, IBKRApiLibError> {
        let mut flds = self
            .contract_condition
            .operator_condition
            .order_condition
            .make_fields()?;
        flds.push(make_field(
            &(self.contract_condition.operator_condition.is_more as i32),
        )?);
        flds.push(make_field(&(self.volume))?);
        flds.push(make_field(&(self.contract_condition.con_id))?);
        flds.push(make_field(&(self.contract_condition.exchange))?);
        Ok(flds)
    }

    //----------------------------------------------------------------------------------------------
    fn value_to_string(&self) -> String {
        self.volume.to_string()
    }

    //----------------------------------------------------------------------------------------------
    fn set_value_from_string(&mut self, text: String) {
        self.volume = text.parse().unwrap();
    }

    //----------------------------------------------------------------------------------------------
    fn get_type(&self) -> ConditionType {
        self.contract_condition
            .operator_condition
            .order_condition
            .cond_type
    }
}

impl Display for VolumeCondition {
    //----------------------------------------------------------------------------------------------
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}", self.value_to_string())
    }
}

impl Debug for VolumeCondition {
    //----------------------------------------------------------------------------------------------
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "volume of {}", self.value_to_string())
    }
}

impl From<OrderConditionEnum> for VolumeCondition {
    //----------------------------------------------------------------------------------------------
    fn from(_: OrderConditionEnum) -> Self {
        VolumeCondition::default()
    }
}

//----------------------------------------------------------------------------------------------
pub fn create_condition<'a>(cond_type: ConditionType) -> OrderConditionEnum {
    match cond_type {
        ConditionType::Execution => OrderConditionEnum::Execution(ExecutionCondition::default()),
        ConditionType::Margin => OrderConditionEnum::Margin(MarginCondition::default()),
        ConditionType::PercentChange => {
            OrderConditionEnum::PercentChange(PercentChangeCondition::default())
        }
        ConditionType::Price => OrderConditionEnum::Price(PriceCondition::default()),
        ConditionType::Time => OrderConditionEnum::Time(TimeCondition::default()),
        ConditionType::Volume => OrderConditionEnum::Volume(VolumeCondition::default()),
    }
}
