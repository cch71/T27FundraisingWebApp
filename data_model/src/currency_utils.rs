use std::cmp::Ordering;

use rust_decimal::prelude::*;
use rusty_money::{iso, Formatter, Money, Params, Position, Round};

pub fn to_money_str<T>(input: Option<T>) -> String
where
    T: Into<String>,
{
    input.map_or_else(
        || "".to_string(),
        |v| Money::from_str(&v.into(), iso::USD).unwrap().to_string(),
    )
}
pub fn str_to_money_str(input: &str) -> String {
    Money::from_str(input, iso::USD).unwrap().to_string()
}

pub fn to_money_str_no_symbol(input: Option<&String>) -> String {
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
        },
    )
}

pub fn from_cloud_to_money_str(input: Option<String>) -> Option<String> {
    input.map(|v| {
        let mut money = Money::from_str(&v, iso::USD).unwrap();
        money = money.round(2, Round::HalfEven);
        money.amount().to_string()
    })
}

pub fn parse_money_str_as_decimal(input: &str) -> Option<Decimal> {
    if input.is_empty() {
        return Some(Decimal::ZERO);
    }
    Some(
        Money::from_str(input, iso::USD)
            .unwrap()
            .amount()
            .to_owned(),
    )
}

pub fn on_money_input_filter(input: Option<&String>) -> String {
    if input.is_none() {
        return "".to_string();
    }

    let input = input.unwrap();
    if input.starts_with(".") {
        //Special case money doesn't handle
        let mut value = input.to_string();
        value.truncate(3);
        return format!("0{}", value);
    }

    let parts: Vec<&str> = input.split(".").collect();

    let major = parts[0].parse::<i32>().unwrap_or(0);

    match parts.len().cmp(&1) {
        Ordering::Equal => {
            //don't have to wory about fractions
            major.to_string()
        }
        Ordering::Greater => {
            let mut fract_str = parts[1].to_string();
            fract_str.truncate(2);
            format!("{}.{}", major, fract_str)
        }
        Ordering::Less => "".to_string(),
    }
}

pub fn decimal_to_money_string(dec_amount: &Decimal) -> String {
    // log::info!("Decimal To Money: {}", dec_amount.round_dp(4).to_string());
    to_money_str(Some(dec_amount.round_dp(4).to_string()))
}
