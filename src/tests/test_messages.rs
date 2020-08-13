#[cfg(test)]
mod tests {

    use crate::core::common::{TickByTickType, UNSET_DOUBLE, UNSET_INTEGER};
    use crate::core::errors::IBKRApiLibError;
    use crate::core::messages::{
        make_field, make_field_handle_empty, make_message, read_fields, read_msg,
        OutgoingMessageIds,
    };
    use crate::examples::contract_samples;
    #[test]
    fn test_make_field() -> Result<(), IBKRApiLibError> {
        assert_eq!("1\u{0}", make_field(&true)?);
        assert_eq!("\u{0}", make_field(&UNSET_DOUBLE)?);
        assert_eq!("\u{0}", make_field(&UNSET_INTEGER)?);
        assert_eq!("100\u{0}", make_field(&100)?);
        assert_eq!("2.5\u{0}", make_field(&2.5)?);
        assert_eq!("hello\u{0}", make_field(&"hello")?);
        assert_eq!("hello\u{0}", make_field(&"hello".to_string())?);
        assert_eq!("\u{0}", make_field(&"".to_string())?);
        assert_eq!("\u{0}", make_field(&Option::<String>::None)?);
        Ok(())
    }

    #[test]
    fn test_make_field_handle_empty() -> Result<(), IBKRApiLibError> {
        assert_eq!("1\u{0}", make_field_handle_empty(&true)?);
        assert_eq!("\u{0}", make_field_handle_empty(&UNSET_DOUBLE)?);
        assert_eq!("\u{0}", make_field_handle_empty(&UNSET_INTEGER)?);
        assert_eq!("100\u{0}", make_field_handle_empty(&100)?);
        assert_eq!("2.5\u{0}", make_field_handle_empty(&2.5)?);
        assert_eq!("hello\u{0}", make_field_handle_empty(&"hello")?);
        assert_eq!("hello\u{0}", make_field_handle_empty(&"hello".to_string())?);
        Ok(())
    }

    #[test]
    fn test_read_fields() {
        let fields = "here\u{0}are\u{0}some\u{0}fields\u{0}1\u{0}2.5\u{0}1000\u{0}";
        let result_fields = vec!["here", "are", "some", "fields", "1", "2.5", "1000"];
        assert_eq!(result_fields, read_fields(fields));
    }

    #[test]
    fn test_make_msg() -> Result<(), IBKRApiLibError> {
        let mut msg = "".to_string();
        let contract = contract_samples::usstock();
        let message_id = OutgoingMessageIds::ReqTickByTickData as i32;

        msg.push_str(&make_field(&message_id)?);
        msg.push_str(&make_field(&1009)?);
        msg.push_str(&make_field(&contract.con_id)?);
        msg.push_str(&make_field(&contract.symbol)?);
        msg.push_str(&make_field(&contract.sec_type)?);
        msg.push_str(&make_field(&contract.last_trade_date_or_contract_month)?);
        msg.push_str(&make_field(&contract.strike)?);
        msg.push_str(&make_field(&contract.right)?);
        msg.push_str(&make_field(&contract.multiplier)?);
        msg.push_str(&make_field(&contract.exchange)?);
        msg.push_str(&make_field(&contract.primary_exchange)?);
        msg.push_str(&make_field(&contract.currency)?);
        msg.push_str(&make_field(&contract.local_symbol)?);
        msg.push_str(&make_field(&contract.trading_class)?);
        msg.push_str(&make_field(&(TickByTickType::AllLast.to_string()))?);

        msg.push_str(&make_field(&0)?);
        msg.push_str(&make_field(&false)?);

        let actual = make_message(msg.as_str())?;

        let expected: Vec<u8> = vec![
            0, 0, 0, 50, 57, 55, 0, 49, 48, 48, 57, 0, 48, 0, 65, 77, 90, 78, 0, 83, 84, 75, 0, 0,
            48, 0, 0, 0, 73, 83, 76, 65, 78, 68, 0, 0, 85, 83, 68, 0, 0, 0, 65, 108, 108, 76, 97,
            115, 116, 0, 48, 0, 48, 0,
        ];

        assert_eq!(expected, actual);

        Ok(())
    }

    #[test]
    fn test_read_msg() -> Result<(), IBKRApiLibError> {
        let msg_bytes: Vec<u8> = vec![
            0, 0, 0, 50, 57, 55, 0, 49, 48, 48, 57, 0, 48, 0, 65, 77, 90, 78, 0, 83, 84, 75, 0, 0,
            48, 0, 0, 0, 73, 83, 76, 65, 78, 68, 0, 0, 85, 83, 68, 0, 0, 0, 65, 108, 108, 76, 97,
            115, 116, 0, 48, 0, 48, 0,
        ];
        let expected = (50, "97\u{0}1009\u{0}0\u{0}AMZN\u{0}STK\u{0}\u{0}0\u{0}\u{0}\u{0}ISLAND\u{0}\u{0}USD\u{0}\u{0}\u{0}AllLast\u{0}0\u{0}0\u{0}".to_owned(), Vec::<u8>::new());
        let actual = read_msg(&msg_bytes)?;
        assert_eq!(expected, actual);

        Ok(())
    }
}
