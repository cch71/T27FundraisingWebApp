use web_sys::{KeyboardEvent};
use rusty_money::{Money, Formatter, Params, Position, Round, iso};
//use rust_decimal::prelude::*;


// pub(crate) fn on_currency_field_key_press(_evt: KeyboardEvent) {
//    log::info!("On currency field key pres");

    // const charCode = (evt.which) ? evt.which : event.keyCode;
    // if (45 === charCode) {
    //     evt.preventDefault();
    //     evt.stopPropagation();
    //     return false;
    // }
    // if (charCode != 46 && charCode > 31 && (charCode < 48 || charCode > 57)) {
    //     evt.preventDefault();
    //     evt.stopPropagation();
    //     return false;
    // }
    // return true;
//}

pub(crate) fn to_money_str(input: Option<&String>) -> String {
    input.map_or_else(
        || "".to_string(),
        |v| Money::from_str(v, iso::USD).unwrap().to_string()
    )
}

pub(crate) fn to_money_str_no_symbol(input: Option<&String>) -> String {
    input.map_or_else(
        || "".to_string(),
        |v| {
            let mut money = Money::from_str(v, iso::USD).unwrap();
            money = money.round(2, Round::HalfEven);
            let params = Params {
                positions: vec![Position::Amount],
                ..Default::default()
            };
            Formatter::money(&money, params)
        }
    )
}

pub(crate) fn on_money_input_filter(input: Option<&String>) -> String {
    if input.is_none() {
        return "".to_string();
    }

    let input = input.unwrap();
    if input.starts_with(".") { //Special case money doesn't handle
        let mut value = input.to_string();
        value.truncate(3);
        return format!("0{}", value);
    }

    let parts:Vec<&str> = input.split(".").collect();

    let major = parts[0].parse::<i32>().unwrap_or(0);
    if parts.len() == 1 { //don't have to wory about fractions
        major.to_string()
    } else if parts.len() > 1 {
        let mut fract_str = parts[1].to_string();
        fract_str.truncate(2);
        format!("{}.{}", major, fract_str)
    } else {
        "".to_string()
    }
}


