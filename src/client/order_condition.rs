use std::fmt::Display;

use serde::export::fmt::Error;
use serde::export::Formatter;
use serde::{Deserialize, Serialize};

use crate::client::messages::{make_field, make_message};

//==================================================================================================
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ConditionType {
    Price = 1,
    Time = 3,
    Margin = 4,
    Execution = 5,
    Volume = 6,
    PercentChange = 7,
}

//==================================================================================================
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OrderCondition {
    pub cond_type: ConditionType,
    pub is_conjunction_connection: bool,
}

impl OrderCondition {
    pub fn new(cond_type: ConditionType, is_conjunction_connection: bool) -> Self {
        OrderCondition {
            cond_type,
            is_conjunction_connection,
        }
    }

    pub fn make_fields(&self) -> String {
        let mut fields = "".to_string();
        fields.push(
            make_field(&if self.is_conjunction_connection {
                "a".to_owned()
            } else {
                "o".to_owned()
            })
            .parse()
            .unwrap(),
        );
        return fields;
    }
}

/*

    def And( self ):
    self.is_conjunction_connection = True
    return self

    def Or( self ):
    self.is_conjunction_connection = False
    return self

    def decode( self,
    fields):
    connector = decode(str,
    fields)
    self.is_conjunction_connection = connector == "a"

    def make_fields(self ):
    flds = []
    flds.append(comm.make_field("a" if self.is_conjunction_connection else "o"))
    return flds
}
*/
impl Display for OrderCondition {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(
            f,
            "{}",
            if self.is_conjunction_connection {
                "<AND>"
            } else {
                "<OR>"
            }
        )
    }
}
//def __str__(self):
//return "<AND>" if self.is_conjunction_connection else "<OR>"
