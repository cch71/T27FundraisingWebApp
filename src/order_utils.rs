use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::{RwLock};
use rust_decimal::prelude::*;
use regex::Regex;

// use crate::data_model::{get_active_user};
use crate::gql_utils::{make_gql_request, GraphQlReq};

lazy_static! {
    static ref ACTIVE_ORDER: RwLock<Option<ActiveOrderState>> = RwLock::new(None);
    static ref CHECKNUM_RE_DELIMETERS: Regex = Regex::new(r"[ ,;.]+").unwrap();
}

#[derive(Default, Clone, PartialEq, Debug)]
pub(crate) struct ActiveOrderState {
    order: MulchOrder,
    is_new_order: bool,
    is_dirty: bool,
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
    pub(crate) is_verified: Option<bool>,
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
            let check_nums_str = self.check_numbers.as_ref().unwrap();
            let check_nums: Vec<&str> = CHECKNUM_RE_DELIMETERS.split(check_nums_str).collect();
            for check_num in check_nums {
                if check_num.trim().parse::<u32>().is_err() {
                    log::info!("Check Num: {} in: {} is invalid", check_num, check_nums_str);
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
    let new_active_order_state = ActiveOrderState {
        order: MulchOrder::new(owner_id),
        is_new_order: true,
        is_dirty: true,
    };

    *ACTIVE_ORDER.write().unwrap() = Some(new_active_order_state);
}

pub(crate) fn is_active_order() -> bool {
    ACTIVE_ORDER.read().unwrap().is_some()
}

pub(crate) fn is_active_order_from_db() -> bool {
    ACTIVE_ORDER.read().unwrap().as_ref().map_or(false, |v| !v.is_new_order)
}

pub(crate) fn reset_active_order() {
    *ACTIVE_ORDER.write().unwrap() = None;
}

pub(crate) fn get_active_order() -> Option<MulchOrder> {
    match &*ACTIVE_ORDER.read().unwrap() {
        Some(order_state)=>Some(order_state.order.clone()),
        None=>None,
    }
}

pub(crate) fn update_active_order(order: MulchOrder)
    -> std::result::Result<(),Box<dyn std::error::Error>>
{
    let mut order_state_opt = ACTIVE_ORDER.write()?;
    let mut order_state = order_state_opt.as_mut().unwrap();
    if !order_state.is_dirty && order_state.order != order {
        order_state.is_dirty = true;
    }
    order_state.order = order;
    Ok(())
}

pub(crate) async fn submit_active_order()
    -> std::result::Result<(),Box<dyn std::error::Error>>
{
    let order_state_opt = ACTIVE_ORDER.write()?;
    let order_state = order_state_opt.as_ref().unwrap();
    if !order_state.is_dirty {
        log::info!("Order doesn't need updating so not submitting");
        return Ok(());
    }

    let order = &order_state.order;

    let mut query = String::with_capacity(1024*32);
    query.push_str("mutation {\n");
    if order_state.is_new_order {
        query.push_str("\t createMulchOrder(order: {\n");
    } else {
        query.push_str("\t updateMulchOrder(order: {\n");
    }

    query.push_str(&format!("\t\t orderId: \"{}\"\n", order.order_id.trim()));
    query.push_str(&format!("\t\t ownerId: \"{}\"\n", order.order_owner_id.trim()));

    if let Some(value) = order.special_instructions.as_ref() {
        query.push_str(&format!("\t\t specialInstructions: \"{}\"\n", value.trim()));
    }

    if let Some(value) = order.is_verified.as_ref() {
        query.push_str(&format!("\t\t isVerified: \"{}\"\n", value));
    }

    if let Some(value) = order.amount_total_collected.as_ref() {
        query.push_str(&format!("\t\t amountTotalCollected: \"{}\"\n", value.trim()));
    } else {
        if !order.will_collect_money_later {
            log::error!("Total collected is zero. will collect later should be true");
        }
        query.push_str("\t\t willCollectMoneyLater: true\n");
    }

    if let Some(value) = order.amount_from_donations.as_ref() {
        query.push_str(&format!("\t\t amountFromDonations: \"{}\"\n", value.trim()));
    }

    if let Some(value) = order.amount_from_purchases.as_ref() {
        query.push_str(&format!("\t\t amountFromPurchases: \"{}\"\n", value.trim()));

        let mut purchases = Vec::new();
        for (product_id, info) in order.purchases.as_ref().unwrap() {
            let mut purchase_str = String::new();
            purchase_str.push_str("\t\t\t {\n");
            purchase_str.push_str(&format!("\t\t\t\t productId: \"{}\"\n", product_id.trim()));
            purchase_str.push_str(&format!("\t\t\t\t numSold: {}\n", info.num_sold));
            purchase_str.push_str(&format!("\t\t\t\t amountCharged: \"{}\"\n", info.amount_charged.trim()));
            purchase_str.push_str("\t\t\t }\n");
            purchases.push(purchase_str);
        }

        query.push_str("\t\t purchases: [\n");
        query.push_str(&purchases.join(","));
        query.push_str("\t\t ]\n");

        query.push_str(&format!("\t\t deliveryId: {}\n", order.delivery_id.as_ref().unwrap()));
    }

    if let Some(value) = order.amount_cash_collected.as_ref() {
        query.push_str(&format!("\t\t amountFromCashCollected: \"{}\"\n", value.trim()));
    }

    if let Some(value) = order.amount_checks_collected.as_ref() {
        query.push_str(&format!("\t\t amountFromChecksCollected: \"{}\"\n", value.trim()));
        query.push_str(&format!("\t\t checkNumbers: \"{}\"\n", order.check_numbers.as_ref().unwrap().trim()));
    }


    query.push_str("\t\t customer: {\n");
    query.push_str(&format!("\t\t\t name: \"{}\"\n", order.customer.name.trim()));
    query.push_str(&format!("\t\t\t addr1: \"{}\"\n", order.customer.addr1.trim()));
    if let Some(value) = order.customer.addr2.as_ref() {
        query.push_str(&format!("\t\t addr2: \"{}\"\n", value.trim()));
    }
    query.push_str(&format!("\t\t\t phone: \"{}\"\n", order.customer.phone.trim()));
    if let Some(value) = order.customer.email.as_ref() {
        query.push_str(&format!("\t\t email: \"{}\"\n", value.trim()));
    }
    query.push_str(&format!("\t\t\t neighborhood: \"{}\"\n", order.customer.neighborhood.trim()));
    query.push_str("\t\t }\n");

    query.push_str("\t})\n");
    query.push_str("}");

    log::info!("Submitting Request:\n{}", &query);

    //Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "TODO Issue")))
    let req = GraphQlReq::new(query);
    make_gql_request::<serde_json::Value>(&req).await.map(|_| ())
}
