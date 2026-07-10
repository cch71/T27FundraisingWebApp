use super::{
    get_active_user,
    gql_utils::{GraphQlReq, make_gql_request},
    is_valid_delivery_id,
};
use crate::currency_utils::*;
use gloo::storage::{SessionStorage, Storage};
use regex::Regex;
use rust_decimal::prelude::*;
use rusty_money::{Money, iso};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::LazyLock;
use tracing::{error, info};

// The active order lives in sessionStorage rather than wasm memory so it is
// shared between the shell and the dynamically loaded page modules (each is
// a separate wasm instance): reports stages an order for editing here and
// the order module picks it up.
const ACTIVE_ORDER_STORAGE_KEY: &str = "ActiveOrderState";

static CHECKNUM_RE_DELIMETERS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[ ,;.]+").unwrap());

#[derive(Default, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct ActiveOrderState {
    pub order: MulchOrder,
    pub is_new_order: bool,
    pub is_dirty: bool,
}

#[derive(Default, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct MulchOrder {
    pub order_id: String,
    pub order_owner_id: String,
    pub last_modified_time: String,
    pub comments: Option<String>,
    pub special_instructions: Option<String>,
    pub amount_from_donations: Option<String>,
    pub amount_from_purchases: Option<String>,
    pub amount_cash_collected: Option<String>,
    pub amount_checks_collected: Option<String>,
    pub amount_total_collected: Option<String>,
    pub check_numbers: Option<String>,
    pub will_collect_money_later: Option<bool>,
    pub is_verified: Option<bool>,
    pub customer: CustomerInfo,
    pub purchases: Option<HashMap<String, PurchasedItem>>,
    pub delivery_id: u32,
    pub year_ordered: Option<String>,
}

#[derive(Default, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct PurchasedItem {
    pub num_sold: u32,
    pub amount_charged: String,
}
impl PurchasedItem {
    pub fn new(num_sold: u32, amount_charged: String) -> Self {
        Self {
            num_sold,
            amount_charged,
        }
    }
}

#[derive(Default, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct CustomerInfo {
    pub name: String,
    pub addr1: String,
    pub addr2: Option<String>,
    pub city: Option<String>,
    pub zipcode: Option<u32>,
    pub phone: String,
    pub email: Option<String>,
    pub neighborhood: Option<String>,
}

impl MulchOrder {
    fn new(owner_id: &str) -> Self {
        Self {
            order_owner_id: owner_id.to_owned(),
            order_id: uuid::Uuid::new_v4().to_string(),
            ..Default::default()
        }
    }

    pub fn is_readonly(&self) -> bool {
        /* if is_system_locked() { return true } */

        if get_active_user().is_admin() {
            return false;
        }

        if self.is_verified.unwrap_or(false) {
            return true;
        }

        false
    }

    pub fn is_delivery_id_valid(&self) -> bool {
        is_valid_delivery_id(self.delivery_id)
    }

    pub fn set_delivery_id(&mut self, delivery_id: u32) {
        self.delivery_id = delivery_id;
    }

    pub fn clear_donations(&mut self) {
        self.amount_from_donations = None;
    }

    pub fn set_donations(&mut self, donation_amt: String) {
        self.amount_from_donations = Some(donation_amt);
    }

    pub fn clear_purchases(&mut self) {
        self.amount_from_purchases = None;
        self.purchases = None;
    }

    pub fn set_purchases(&mut self, purchases: HashMap<String, PurchasedItem>) {
        let mut total_purchase_amt = Decimal::ZERO;
        for purchase in purchases.values() {
            total_purchase_amt = total_purchase_amt
                .checked_add(Decimal::from_str(&purchase.amount_charged).unwrap())
                .unwrap();
        }

        self.purchases = Some(purchases);
        self.amount_from_purchases = Some(total_purchase_amt.to_string());
    }

    pub fn get_num_sold(&self, product_id: &str) -> Option<u32> {
        match self.purchases.as_ref() {
            Some(purchases) => purchases.get(product_id).map(|v| v.num_sold),
            None => None,
        }
    }

    pub fn get_total_to_collect(&self) -> Decimal {
        let mut total = Decimal::ZERO;
        if let Some(amt) = self.amount_from_donations.as_ref() {
            total = total
                .checked_add(*Money::from_str(amt, iso::USD).unwrap().amount())
                .unwrap();
        }
        if let Some(amt) = self.amount_from_purchases.as_ref() {
            total = total
                .checked_add(*Money::from_str(amt, iso::USD).unwrap().amount())
                .unwrap();
        }
        total
    }

    pub fn get_total_collected(&self) -> Decimal {
        let mut total = Decimal::ZERO;
        if let Some(amt) = self.amount_cash_collected.as_ref() {
            total = total
                .checked_add(*Money::from_str(amt, iso::USD).unwrap().amount())
                .unwrap();
        }
        if let Some(amt) = self.amount_checks_collected.as_ref() {
            total = total
                .checked_add(*Money::from_str(amt, iso::USD).unwrap().amount())
                .unwrap();
        }
        total
    }

    pub fn is_payment_valid(&self) -> bool {
        self.is_check_numbers_valid()
            && ((self.get_total_to_collect() != Decimal::ZERO
                && (self.get_total_to_collect() == self.get_total_collected()))
                || self.will_collect_money_later.unwrap_or(false))
    }

    pub fn is_check_numbers_valid(&self) -> bool {
        // Collected checks is something all the checks in check nums str needs to be valid
        if self.amount_checks_collected.is_some() {
            self.check_numbers.as_ref().is_some_and(|check_nums_str| {
                let check_nums: Vec<&str> = CHECKNUM_RE_DELIMETERS
                    .split(check_nums_str)
                    // Ignore empty segments produced by a leading/trailing or
                    // doubled delimiter (e.g. "1234, 5678,") so a valid entry
                    // isn't rejected.
                    .filter(|s| !s.trim().is_empty())
                    .collect();
                !check_nums.is_empty()
                    && check_nums.iter().all(|&check_num| {
                        check_num
                            .trim()
                            .parse::<u32>()
                            .inspect_err(|_| {
                                info!("Check Num: {check_num} in: {check_nums_str} is invalid");
                            })
                            .is_ok()
                    })
            })
        } else {
            self.check_numbers.is_none()
        }
    }

    pub fn are_purchases_valid(&self) -> bool {
        let is_product_purchase_valid =
            self.amount_from_purchases.is_some() && self.purchases.is_some();
        let is_donations_valid = self.amount_from_donations.is_some();
        let is_total_valid = self.amount_total_collected.as_ref().is_none_or(|v| {
            Decimal::from_str(v)
                .ok()
                .is_none_or(|v| v != Decimal::ZERO && v.is_sign_positive())
        });

        is_total_valid && (is_product_purchase_valid || is_donations_valid)
    }
}

pub fn is_order_from_report_data_readonly(j_order: &serde_json::Value) -> bool {
    /* if is_system_locked() { return true } */

    if get_active_user().is_admin() {
        return false;
    }

    if j_order["isVerified"].as_bool().unwrap_or(false) {
        return true;
    }

    false
}

pub fn get_active_order_state() -> Option<ActiveOrderState> {
    SessionStorage::get(ACTIVE_ORDER_STORAGE_KEY).ok()
}

fn set_active_order_state(state: &ActiveOrderState) {
    if let Err(err) = SessionStorage::set(ACTIVE_ORDER_STORAGE_KEY, state) {
        error!("Failed saving active order to session storage: {err:#?}");
    }
}

pub fn create_new_active_order(owner_id: &str) {
    set_active_order_state(&ActiveOrderState {
        order: MulchOrder::new(owner_id),
        is_new_order: true,
        is_dirty: true,
    });
}

pub fn is_active_order() -> bool {
    get_active_order_state().is_some()
}

pub fn is_active_order_from_db() -> bool {
    get_active_order_state().is_none_or(|v| !v.is_new_order)
}

pub fn reset_active_order() {
    SessionStorage::delete(ACTIVE_ORDER_STORAGE_KEY);
}

pub fn get_active_order() -> Option<MulchOrder> {
    get_active_order_state().map(|v| v.order)
}

pub fn update_active_order(order: MulchOrder) -> Result<(), Box<dyn std::error::Error>> {
    let mut order_state = get_active_order_state().ok_or("No active order to update")?;
    if !order_state.is_dirty && order_state.order != order {
        order_state.is_dirty = true;
    }
    order_state.order = order;
    set_active_order_state(&order_state);
    Ok(())
}

static DELETE_ORDER_GQL: &str = r"
mutation {
  deleteMulchOrder(***ORDER_ID_PARAM***)
}
";

pub async fn delete_order(order_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    let query =
        DELETE_ORDER_GQL.replace("***ORDER_ID_PARAM***", &format!("orderId: \"{order_id}\""));

    let req = GraphQlReq::new(query);
    info!("Delete GraphQL: {}", &req.query);
    make_gql_request::<serde_json::Value>(&req)
        .await
        .map(|_| ())
}

static LOAD_ORDER_GQL: &str = r"
{
  mulchOrder(***ORDER_ID_PARAM***) {
    orderId
    ownerId
    amountFromPurchases
    amountFromDonations
    amountFromCashCollected
    amountFromChecksCollected
    checkNumbers
    amountTotalCollected
    willCollectMoneyLater
    isVerified
    customer {
        name
        addr1
        addr2
        city
        zipcode
        phone
        email
        neighborhood
    }
    comments
    specialInstructions
    deliveryId
    purchases {
        productId
        numSold
        amountCharged
    }
  }
}
";

pub async fn load_active_order_from_db(order_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    #[derive(Deserialize, Debug)]
    struct RespWrapper {
        #[serde(alias = "mulchOrder")]
        mulch_order: MulchOrderApi,
    }

    #[derive(Deserialize, Debug)]
    pub struct MulchOrderApi {
        #[serde(alias = "orderId")]
        pub order_id: String,
        #[serde(alias = "ownerId")]
        pub order_owner_id: String,
        pub comments: Option<String>,
        #[serde(alias = "specialInstructions")]
        pub special_instructions: Option<String>,
        #[serde(alias = "amountFromDonations")]
        pub amount_from_donations: Option<String>,
        #[serde(alias = "amountFromPurchases")]
        pub amount_from_purchases: Option<String>,
        #[serde(alias = "amountFromCashCollected")]
        pub amount_cash_collected: Option<String>,
        #[serde(alias = "amountFromChecksCollected")]
        pub amount_checks_collected: Option<String>,
        #[serde(alias = "amountTotalCollected")]
        pub amount_total_collected: Option<String>,
        #[serde(alias = "checkNumbers")]
        pub check_numbers: Option<String>,
        #[serde(alias = "willCollectMoneyLater")]
        pub will_collect_money_later: Option<bool>,
        #[serde(alias = "isVerified")]
        pub is_verified: Option<bool>,
        pub customer: CustomerInfo,
        pub purchases: Option<Vec<PurchasedItemApi>>,
        #[serde(alias = "deliveryId")]
        pub delivery_id: u32,
    }

    #[derive(Deserialize, Debug)]
    pub struct PurchasedItemApi {
        #[serde(alias = "productId")]
        pub product_id: String,
        #[serde(alias = "numSold")]
        pub num_sold: u32,
        #[serde(alias = "amountCharged")]
        pub amount_charged: String,
    }

    let query = LOAD_ORDER_GQL.replace("***ORDER_ID_PARAM***", &format!("orderId: \"{order_id}\""));

    let req = GraphQlReq::new(query);
    info!("Load GraphQL: {}", &req.query);
    let resp = make_gql_request::<RespWrapper>(&req).await?;
    let order = resp.mulch_order;

    let new_active_order_state = ActiveOrderState {
        order: MulchOrder {
            order_id: order.order_id,
            order_owner_id: order.order_owner_id,
            comments: order.comments,
            special_instructions: order.special_instructions,
            amount_from_donations: order.amount_from_donations,
            amount_from_purchases: order.amount_from_purchases,
            amount_cash_collected: order.amount_cash_collected,
            amount_checks_collected: order.amount_checks_collected,
            check_numbers: order.check_numbers,
            amount_total_collected: from_cloud_to_money_str(order.amount_total_collected),
            will_collect_money_later: order.will_collect_money_later,
            is_verified: order.is_verified,
            customer: order.customer,
            delivery_id: order.delivery_id,
            purchases: order.purchases.map(|v| {
                v.into_iter()
                    .map(|i| {
                        (
                            i.product_id,
                            PurchasedItem {
                                num_sold: i.num_sold,
                                amount_charged: to_money_str_no_symbol(Some(&i.amount_charged)),
                            },
                        )
                    })
                    .collect()
            }),
            ..Default::default()
        },
        is_new_order: false,
        is_dirty: false,
    };

    set_active_order_state(&new_active_order_state);
    Ok(())
}

static SET_SPREADERS_GQL: &str = r"
mutation {
  setSpreaders(
    ***ORDER_ID_PARAM***,
    spreaders: [***SPREADERS_PARAM***]
  )
}
";

pub async fn set_spreaders(
    order_id: &str,
    spreaders: &Vec<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    info!(
        "Setting Spreaders for order id: {}:{:#?}",
        order_id, &spreaders
    );
    let spreaders = spreaders
        .iter()
        .map(|v| format!("\"{v}\""))
        .collect::<Vec<String>>()
        .join(",");
    let query = SET_SPREADERS_GQL
        .replace("***ORDER_ID_PARAM***", &format!("orderId: \"{order_id}\""))
        .replace("***SPREADERS_PARAM***", &spreaders);

    let req = GraphQlReq::new(query);
    info!("Setting Spreaders GraphQL: {}", &req.query);
    make_gql_request::<serde_json::Value>(&req)
        .await
        .map(|_| ())
}

static TROOP_ORDER_AMOUNT_COLLECTED_GQL: &str = r"
{
  summary {
    troop(numTopSellers: 1) {
      totalAmountCollected
    }
  }
}
";

pub async fn have_orders_been_created() -> Result<bool, Box<dyn std::error::Error>> {
    // Fails safe
    let req = GraphQlReq::new(TROOP_ORDER_AMOUNT_COLLECTED_GQL);
    make_gql_request::<serde_json::Value>(&req).await.map(|v| {
        v["summary"]["troop"]["totalAmountCollected"]
            .as_str()
            .map_or_else(|| true, |i| i != "0")
    })
}
