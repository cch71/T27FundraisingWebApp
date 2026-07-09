use std::cmp::Ordering;

use rust_decimal::prelude::*;
use rusty_money::{Formatter, Money, Params, Position, Round, iso};

pub fn to_money_str<T>(input: Option<T>) -> String
where
    T: Into<String>,
{
    input.map_or_else(
        || "".to_string(),
        |v| {
            let v = v.into();
            // Fall back to the raw value rather than panicking on unparseable
            // input (a wasm panic aborts the whole app).
            Money::from_str(&v, iso::USD).map_or(v, |m| m.to_string())
        },
    )
}
pub fn str_to_money_str(input: &str) -> String {
    Money::from_str(input, iso::USD).map_or_else(|_| input.to_string(), |m| m.to_string())
}

pub fn to_money_str_no_symbol(input: Option<&String>) -> String {
    input.map_or_else(
        || "".to_string(),
        |v| match Money::from_str(v, iso::USD) {
            Ok(money) => {
                let money = money.round(2, Round::HalfEven);
                let params = Params {
                    positions: &[Position::Amount],
                    ..Default::default()
                };
                Formatter::money(&money, params)
            }
            Err(_) => v.clone(),
        },
    )
}

pub fn from_cloud_to_money_str(input: Option<String>) -> Option<String> {
    input.map(|v| match Money::from_str(&v, iso::USD) {
        Ok(money) => money.round(2, Round::HalfEven).amount().to_string(),
        Err(_) => v,
    })
}

pub fn parse_money_str_as_decimal(input: &str) -> Option<Decimal> {
    if input.is_empty() {
        return Some(Decimal::ZERO);
    }
    // Returns None on unparseable input so callers can handle it instead of
    // panicking.
    Money::from_str(input, iso::USD)
        .ok()
        .map(|m| m.amount().to_owned())
}

pub fn on_money_input_filter(input: Option<&String>) -> String {
    if input.is_none() {
        return "".to_string();
    }

    let input = input.unwrap();

    // Keep only digits per segment. This prevents non-numeric characters
    // (e.g. a pasted "1.5e3" or scientific notation) from surviving into a
    // value that would later panic Decimal/Money parsing, and avoids silently
    // turning an out-of-range integer into 0.
    let digits_only = |s: &str| -> String { s.chars().filter(|c| c.is_ascii_digit()).collect() };

    if let Some(fract) = input.strip_prefix('.') {
        //Special case money doesn't handle
        let mut fract_str = digits_only(fract);
        fract_str.truncate(2);
        return format!("0.{fract_str}");
    }

    let parts: Vec<&str> = input.split(".").collect();

    let major = digits_only(parts[0]);
    let major = if major.is_empty() { "0".to_string() } else { major };

    match parts.len().cmp(&1) {
        Ordering::Equal => {
            //don't have to worry about fractions
            major
        }
        Ordering::Greater => {
            let mut fract_str = digits_only(parts[1]);
            fract_str.truncate(2);
            format!("{major}.{fract_str}")
        }
        Ordering::Less => "".to_string(),
    }
}

pub fn decimal_to_money_string(dec_amount: &Decimal) -> String {
    // log::info!("Decimal To Money: {}", dec_amount.round_dp(4).to_string());
    to_money_str(Some(dec_amount.round_dp(4).to_string()))
}
