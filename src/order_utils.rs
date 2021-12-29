use lazy_static::lazy_static;
use crate::auth_utils::{is_admin};

use std::collections::HashMap;
use std::sync::{RwLock};

lazy_static! {
    static ref ACTIVE_ORDER: RwLock<Option<MulchOrder>> = RwLock::new(None);
}

#[derive(Default, Clone, PartialEq)]
pub struct MulchOrder {
    pub order_id: String,
    pub order_owner_id: String,
    pub last_modified_time: String,
    pub special_instructions: Option<String>,
    pub amount_for_donations_collected: Option<String>,
    pub amount_for_purchases_collected: Option<String>,
    pub amount_cash_collected: Option<String>,
    pub amount_checks_collected: Option<String>,
    pub amount_total_collected: String,
    pub check_numbers: Vec<String>,
    pub will_collect_money_later: bool,
    pub is_verified: bool,
    pub customer: CustomerInfo,
    pub purchases: Option<HashMap<String, PurchasedItem>>,
    pub delivery_id: Option<String>,
    pub year_ordered: Option<String>
}

#[derive(Default, Clone, PartialEq)]
pub struct PurchasedItem {
    pub num_sold: u32,
    pub amount_charged: String,
}

#[derive(Default, Clone, PartialEq)]
pub struct CustomerInfo {
    pub name: String,
    pub addr1: String,
    pub addr2: Option<String>,
    pub phone: String,
    pub email: Option<String>,
    pub neighborhood: String,
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
            amount_for_donations_collected: Some("200.24".to_string()),
            amount_total_collected: "200.24".to_string(),
            ..Default::default()
        }
    }

    pub fn is_readonly(&self) -> bool {
        /* if is_system_locked() { return true } */
        // return !is_admin() /* && now > order.delivery_id.cutoff_date */;
        false
    }

    pub fn clear_donations(&mut self) {
        self.amount_for_donations_collected = None;
    }

    pub fn clear_purchases(&mut self) {
        self.amount_for_purchases_collected = None;
        self.purchases = None;
    }

    pub fn get_num_sold(&self, product_id: &str) -> Option<u32> {
        match self.purchases.as_ref() {
            Some(purchases) => purchases.get(product_id).map_or(None, |v| Some(v.num_sold)),
            None=>None,
        }
    }
}

pub(crate) fn create_new_active_order(owner_id: &str) {
    let new_order = MulchOrder::new(owner_id);
    *ACTIVE_ORDER.write().unwrap() = Some(new_order.clone());
}

pub(crate) fn is_active_order() -> bool {
    ACTIVE_ORDER.read().unwrap().is_some()
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
