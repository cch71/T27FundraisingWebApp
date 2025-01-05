use crate::currency_utils::*;
use regex::Regex;
use rust_decimal::prelude::*;
use rusty_money::{iso, Money};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{LazyLock, RwLock};

use super::{
    get_active_user,
    gql_utils::{make_gql_request, GraphQlReq},
};

static ACTIVE_ORDER: LazyLock<RwLock<Option<ActiveOrderState>>> =
    LazyLock::new(|| RwLock::new(None));
static CHECKNUM_RE_DELIMETERS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[ ,;.]+").unwrap());

#[derive(Default, Clone, PartialEq, Debug)]
pub struct ActiveOrderState {
    order: MulchOrder,
    is_new_order: bool,
    is_dirty: bool,
}

#[derive(Default, Clone, PartialEq, Debug)]
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
    pub delivery_id: Option<u32>,
    pub year_ordered: Option<String>,
}

#[derive(Default, Clone, PartialEq, Debug)]
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

        if let Some(_delivery_id) = self.delivery_id {
            // now > order.delivery_id.cutoff_date return true
        }
        false
    }

    pub fn clear_donations(&mut self) {
        self.amount_from_donations = None;
    }

    pub fn set_donations(&mut self, donation_amt: String) {
        log::info!("!!!! Setting Donations to: {}", donation_amt);
        self.amount_from_donations = Some(donation_amt);
    }

    pub fn clear_purchases(&mut self) {
        self.delivery_id = None;
        self.amount_from_purchases = None;
        self.purchases = None;
    }

    pub fn set_purchases(&mut self, delivery_id: u32, purchases: HashMap<String, PurchasedItem>) {
        let mut total_purchase_amt = Decimal::ZERO;
        for purchase in purchases.values() {
            total_purchase_amt = total_purchase_amt
                .checked_add(Decimal::from_str(&purchase.amount_charged).unwrap())
                .unwrap();
        }

        self.delivery_id = Some(delivery_id);
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
        } else {
            !(self.amount_checks_collected.is_none() && self.check_numbers.is_some())
        }
    }

    pub fn are_purchases_valid(&self) -> bool {
        let is_product_purchase_valid = self.delivery_id.is_some()
            && self.amount_from_purchases.is_some()
            && self.purchases.is_some();
        let is_donations_valid = self.amount_from_donations.is_some();
        let is_total_valid = self.amount_total_collected.as_ref().map_or(true, |v| {
            Decimal::from_str(v).map_or(false, |v| v != Decimal::ZERO && v.is_sign_positive())
        });

        is_total_valid && (is_product_purchase_valid || is_donations_valid)
    }
}

pub fn is_order_from_report_data_readonly(jorder: &serde_json::Value) -> bool {
    /* if is_system_locked() { return true } */

    if get_active_user().is_admin() {
        return false;
    }

    if jorder["isVerified"].as_bool().unwrap_or(false) {
        return true;
    }

    if let Some(_delivery_id) = jorder["deliveryId"].as_u64() {
        // now > order.delivery_id.cutoff_date return true
    }
    false
}

pub fn create_new_active_order(owner_id: &str) {
    let new_active_order_state = ActiveOrderState {
        order: MulchOrder::new(owner_id),
        is_new_order: true,
        is_dirty: true,
    };

    *ACTIVE_ORDER.write().unwrap() = Some(new_active_order_state);
}

pub fn is_active_order() -> bool {
    ACTIVE_ORDER.read().unwrap().is_some()
}

pub fn is_active_order_from_db() -> bool {
    ACTIVE_ORDER
        .read()
        .unwrap()
        .as_ref()
        .map_or(false, |v| !v.is_new_order)
}

pub fn reset_active_order() {
    *ACTIVE_ORDER.write().unwrap() = None;
}

pub fn get_active_order() -> Option<MulchOrder> {
    ACTIVE_ORDER
        .read()
        .unwrap()
        .as_ref()
        .map(|v| v.order.clone())
}

pub fn update_active_order(
    order: MulchOrder,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let mut order_state_opt = ACTIVE_ORDER.write()?;
    let order_state = order_state_opt.as_mut().unwrap();
    if !order_state.is_dirty && order_state.order != order {
        order_state.is_dirty = true;
    }
    order_state.order = order;
    Ok(())
}

fn gen_submit_active_order_req_str() -> std::result::Result<String, Box<dyn std::error::Error>> {
    let order_state_opt = ACTIVE_ORDER.write()?;
    let order_state = order_state_opt.as_ref().unwrap();
    if !order_state.is_dirty {
        log::info!("Order doesn't need updating so not submitting");
        return Ok("".to_string());
    }

    let order = &order_state.order;

    let mut query = String::with_capacity(1024 * 32);
    query.push_str("mutation {\n");
    if order_state.is_new_order {
        query.push_str("\t createMulchOrder(order: {\n");
    } else {
        query.push_str("\t updateMulchOrder(order: {\n");
    }

    query.push_str(&format!("\t\t orderId: \"{}\"\n", order.order_id.trim()));
    query.push_str(&format!(
        "\t\t ownerId: \"{}\"\n",
        order.order_owner_id.trim()
    ));

    if let Some(value) = order.comments.as_ref() {
        query.push_str(&format!("\t\t comments: \"{}\"\n", value.trim()));
    }

    if let Some(value) = order.special_instructions.as_ref() {
        query.push_str(&format!("\t\t specialInstructions: \"{}\"\n", value.trim()));
    }

    if let Some(value) = order.is_verified.as_ref() {
        query.push_str(&format!("\t\t isVerified: {}\n", value));
    }

    if let Some(value) = order.amount_total_collected.as_ref() {
        query.push_str(&format!(
            "\t\t amountTotalCollected: \"{}\"\n",
            value.trim()
        ));
    } else {
        if !order.will_collect_money_later.unwrap_or(false) {
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
            purchase_str.push_str(&format!(
                "\t\t\t\t amountCharged: \"{}\"\n",
                info.amount_charged.trim()
            ));
            purchase_str.push_str("\t\t\t }\n");
            purchases.push(purchase_str);
        }

        query.push_str("\t\t purchases: [\n");
        query.push_str(&purchases.join(","));
        query.push_str("\t\t ]\n");

        query.push_str(&format!(
            "\t\t deliveryId: {}\n",
            order.delivery_id.as_ref().unwrap()
        ));
    }

    if let Some(value) = order.amount_cash_collected.as_ref() {
        query.push_str(&format!(
            "\t\t amountFromCashCollected: \"{}\"\n",
            value.trim()
        ));
    }

    if let Some(value) = order.amount_checks_collected.as_ref() {
        query.push_str(&format!(
            "\t\t amountFromChecksCollected: \"{}\"\n",
            value.trim()
        ));
        query.push_str(&format!(
            "\t\t checkNumbers: \"{}\"\n",
            order.check_numbers.as_ref().unwrap().trim()
        ));
    }

    query.push_str("\t\t customer: {\n");
    query.push_str(&format!(
        "\t\t\t name: \"{}\"\n",
        order.customer.name.trim()
    ));
    query.push_str(&format!(
        "\t\t\t addr1: \"{}\"\n",
        order.customer.addr1.trim()
    ));
    if let Some(value) = order.customer.addr2.as_ref() {
        query.push_str(&format!("\t\t\t addr2: \"{}\"\n", value.trim()));
    }
    if let Some(value) = order.customer.city.as_ref() {
        query.push_str(&format!("\t\t\t city: \"{}\"\n", value.trim()));
    }
    if let Some(value) = order.customer.zipcode.as_ref() {
        query.push_str(&format!("\t\t\t zipcode: {}\n", value));
    }
    query.push_str(&format!(
        "\t\t\t phone: \"{}\"\n",
        order.customer.phone.trim()
    ));
    if let Some(value) = order.customer.email.as_ref() {
        query.push_str(&format!("\t\t email: \"{}\"\n", value.trim()));
    }
    query.push_str(&format!(
        "\t\t\t neighborhood: \"{}\"\n",
        order
            .customer
            .neighborhood
            .as_ref()
            .unwrap_or(&"".to_string())
            .trim()
    ));
    query.push_str("\t\t }\n");

    query.push_str("\t})\n");
    query.push('}');
    Ok(query)
}

pub async fn submit_active_order() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let query = gen_submit_active_order_req_str()?;

    if query.is_empty() {
        // If a query wasn't generated then we don't need to submit it
        return Ok(());
    }

    log::info!("Submitting Request:\n{}", &query);

    //Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "TODO Issue")))
    let req = GraphQlReq::new(query);
    make_gql_request::<serde_json::Value>(&req)
        .await
        .map(|_| ())
}

static DELETE_ORDER_GQL: &str = r"
mutation {
  deleteMulchOrder(***ORDER_ID_PARAM***)
}
";

pub async fn delete_order(order_id: &str) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let query = DELETE_ORDER_GQL.replace(
        "***ORDER_ID_PARAM***",
        &format!("orderId: \"{}\"", order_id),
    );

    let req = GraphQlReq::new(query);
    log::info!("Delete GraphQL: {}", &req.query);
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

pub async fn load_active_order_from_db(
    order_id: &str,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
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
        pub delivery_id: Option<u32>,
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

    let query = LOAD_ORDER_GQL.replace(
        "***ORDER_ID_PARAM***",
        &format!("orderId: \"{}\"", order_id),
    );

    let req = GraphQlReq::new(query);
    log::info!("Load GraphQL: {}", &req.query);
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

    *ACTIVE_ORDER.write().unwrap() = Some(new_active_order_state);
    Ok(())
}

// use wasm_bindgen::prelude::*;
// #[wasm_bindgen]
// pub fn sleep(ms: i32) -> js_sys::Promise {
//     js_sys::Promise::new(&mut |resolve, _| {
//         web_sys::window()
//             .unwrap()
//             .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, ms)
//             .unwrap();
//     })
// }

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
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    log::info!(
        "Setting Spreaders for orderid: {}:{:#?}",
        order_id,
        &spreaders
    );
    let spreaders = spreaders
        .iter()
        .map(|v| format!("\"{}\"", v))
        .collect::<Vec<String>>()
        .join(",");
    let query = SET_SPREADERS_GQL
        .replace(
            "***ORDER_ID_PARAM***",
            &format!("orderId: \"{}\"", order_id),
        )
        .replace("***SPREADERS_PARAM***", &spreaders);

    let req = GraphQlReq::new(query);
    log::info!("Setting Spreaders GraphQL: {}", &req.query);
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

pub async fn have_orders_been_created() -> std::result::Result<bool, Box<dyn std::error::Error>> {
    // Fails safe
    let req = GraphQlReq::new(TROOP_ORDER_AMOUNT_COLLECTED_GQL);
    make_gql_request::<serde_json::Value>(&req).await.map(|v| {
        v["summary"]["troop"]["totalAmountCollected"]
            .as_str()
            .map_or_else(|| true, |i| i != "0")
    })
}
