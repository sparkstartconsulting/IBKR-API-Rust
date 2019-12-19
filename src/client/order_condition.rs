use std::fmt::Display;

use serde::export::fmt::Error;
use serde::export::Formatter;

pub enum ConditionType {
    Price = 1,
    Time = 3,
    Margin = 4,
    Execution = 5,
    Volume = 6,
    PercentChange = 7,
}

pub struct OrderCondition {
    cond_type: ConditionType,
    is_conjunction_connection: bool,
}

impl OrderCondition {
    pub fn new(cond_type: ConditionType, is_conjunction_connection: bool) -> Self {
        OrderCondition {
            cond_type,
            is_conjunction_connection,
        }
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
