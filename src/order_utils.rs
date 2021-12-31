use lazy_static::lazy_static;
// use crate::data_model::{get_active_user};

use std::collections::HashMap;
use std::sync::{RwLock};
use rust_decimal::prelude::*;

lazy_static! {
    static ref ACTIVE_ORDER: RwLock<Option<MulchOrder>> = RwLock::new(None);
}

#[derive(Default, Clone, PartialEq, Debug)]
pub(crate) struct MulchOrder {
    pub(crate) order_id: String,
    pub(crate) order_owner_id: String,
    pub(crate) last_modified_time: String,
    pub(crate) special_instructions: Option<String>,
    pub(crate) amount_from_donations: Option<String>,
    pub(crate) amount_from_purchases: Option<String>,
    pub(crate) amount_cash_collected: Option<String>,
    pub(crate) amount_checks_collected: Option<String>,
    pub(crate) amount_total_collected: Option<String>,
    pub(crate) check_numbers: Option<String>,
    pub(crate) will_collect_money_later: bool,
    pub(crate) is_verified: bool,
    pub(crate) customer: CustomerInfo,
    pub(crate) purchases: Option<HashMap<String, PurchasedItem>>,
    pub(crate) delivery_id: Option<u32>,
    pub(crate) year_ordered: Option<String>
}

#[derive(Default, Clone, PartialEq, Debug)]
pub(crate) struct PurchasedItem {
    pub(crate) num_sold: u32,
    pub(crate) amount_charged: String,
}
impl PurchasedItem {
    pub(crate) fn new(num_sold: u32, amount_charged:String)->Self {
        Self{
            num_sold: num_sold,
            amount_charged: amount_charged,
        }
    }
}

#[derive(Default, Clone, PartialEq, Debug)]
pub(crate) struct CustomerInfo {
    pub(crate) name: String,
    pub(crate) addr1: String,
    pub(crate) addr2: Option<String>,
    pub(crate) phone: String,
    pub(crate) email: Option<String>,
    pub(crate) neighborhood: String,
}

impl MulchOrder {
    fn new(owner_id: &str)->Self {
        Self{
            order_owner_id: owner_id.to_owned(),
            order_id: uuid::Uuid::new_v4().to_string(),
            customer: CustomerInfo {
                name: "John Stamose".to_string(),
                addr1: "202 lovers lane".to_string(),
                neighborhood: "Bear Valley".to_string(),
                phone: "455-234-4234".to_string(),
                ..Default::default()
            },
            amount_from_donations: Some("200.24".to_string()),
            amount_cash_collected: Some("200.24".to_string()),
            amount_total_collected: Some("200.24".to_string()),
            ..Default::default()
        }
    }

    pub(crate) fn is_readonly(&self) -> bool {
        /* if is_system_locked() { return true } */
        // return !is_admin() /* && now > order.delivery_id.cutoff_date */;
        false
    }

    pub(crate) fn clear_donations(&mut self) {
        self.amount_from_donations = None;
    }

    pub(crate) fn set_donations(&mut self, donation_amt: String) {
        self.amount_from_donations = Some(donation_amt);
    }

    pub(crate) fn clear_purchases(&mut self) {
        self.delivery_id = None;
        self.amount_from_purchases = None;
        self.purchases = None;
    }

    pub(crate) fn set_purchases(&mut self, delivery_id: u32, purchases: HashMap<String, PurchasedItem>)
    {
        let mut total_purchase_amt = Decimal::ZERO;
        for purchase in purchases.values() {
            total_purchase_amt = total_purchase_amt.checked_add(
                Decimal::from_str(&purchase.amount_charged).unwrap()).unwrap();

        }

        self.delivery_id = Some(delivery_id);
        self.purchases = Some(purchases);
        self.amount_from_purchases = Some(total_purchase_amt.to_string());
    }

    pub(crate) fn get_num_sold(&self, product_id: &str) -> Option<u32> {
        match self.purchases.as_ref() {
            Some(purchases) => purchases.get(product_id).map_or(None, |v| Some(v.num_sold)),
            None=>None,
        }
    }

    pub(crate) fn get_total_to_collect(&self)->Decimal {
        let mut total = Decimal::ZERO;
        if let Some(amt) = self.amount_from_donations.as_ref() {
            total = total.checked_add(
                Decimal::from_str(amt).unwrap()).unwrap();
        }
        if let Some(amt) = self.amount_from_purchases.as_ref() {
            total = total.checked_add(
                Decimal::from_str(amt).unwrap()).unwrap();
        }
        total
    }

    pub(crate) fn get_total_collected(&self)->Decimal {
        let mut total = Decimal::ZERO;
        if let Some(amt) = self.amount_cash_collected.as_ref() {
            total = total.checked_add(
                Decimal::from_str(amt).unwrap()).unwrap();
        }
        if let Some(amt) = self.amount_checks_collected.as_ref() {
            total = total.checked_add(
                Decimal::from_str(amt).unwrap()).unwrap();
        }
        total
    }

    pub(crate) fn is_payment_valid(&self) -> bool {
        self.is_check_numbers_valid() &&
            ((self.get_total_to_collect() == self.get_total_collected()) || self.will_collect_money_later)
    }

    pub(crate) fn is_check_numbers_valid(&self) -> bool {
        if self.amount_checks_collected.is_some() && self.check_numbers.is_some() {
            let check_nums = self.check_numbers.as_ref().unwrap();
            for check_num in check_nums.split(&[',', ';' , ' '][..]).collect::<Vec<&str>>() {
                if check_num.trim().parse::<u32>().is_err() {
                    log::info!("Check Num is invalid: {}", self.check_numbers.as_ref().unwrap());
                    return false;
                }
            }
            true
        } else if self.amount_checks_collected.is_none() && self.check_numbers.is_some() {
            false
        } else if self.amount_checks_collected.is_some() && self.check_numbers.is_none() {
            false
        } else {
            true
        }
    }

    pub(crate) fn are_purchases_valid(&self) -> bool {

        let is_product_purchase_valid = self.delivery_id.is_some() && self.amount_from_purchases.is_some() && self.purchases.is_some();
        let is_donations_valid = self.amount_from_donations.is_some();
        let is_total_valid = self.amount_total_collected
            .as_ref()
            .map_or(true, |v| {
                Decimal::from_str(v).map_or(false, |v|v!=Decimal::ZERO && v.is_sign_positive())
            });

        is_total_valid && (is_product_purchase_valid || is_donations_valid)
    }

}

pub(crate) fn create_new_active_order(owner_id: &str) {
    let new_order = MulchOrder::new(owner_id);
    *ACTIVE_ORDER.write().unwrap() = Some(new_order.clone());
}

pub(crate) fn is_active_order() -> bool {
    ACTIVE_ORDER.read().unwrap().is_some()
}

pub(crate) fn is_active_order_from_db() -> bool {
    is_active_order() && false //TODO: false should be a check from db
}

pub(crate) fn reset_active_order() {
    *ACTIVE_ORDER.write().unwrap() = None;
}

pub(crate) fn get_active_order() -> Option<MulchOrder> {
    match &*ACTIVE_ORDER.read().unwrap() {
        Some(order)=>Some(order.clone()),
        None=>None,
    }
}

pub(crate) fn update_active_order(order: MulchOrder)
    -> std::result::Result<(),Box<dyn std::error::Error>>
{
    *ACTIVE_ORDER.write().unwrap() = Some(order);
    Ok(())
}
