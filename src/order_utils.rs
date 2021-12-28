use rusty_money::{Money, iso};

#[derive(Default)]
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
    pub purchases: Option<MulchPurchases>,
    pub delivery_id: Option<u64>,
    pub year_ordered: Option<String>
}

#[derive(Default)]
pub struct MulchPurchases {
    pub bags_sold: Option<i64>,
    pub bogs_to_spread: Option<i64>,
    pub amount_charged_for_bags: Option<String>,
    pub amount_charged_for_spreading: Option<String>,
}

#[derive(Default)]
pub struct CustomerInfo {
    pub name: String,
    pub addr1: String,
    pub addr2: Option<String>,
    pub phone: String,
    pub email: Option<String>,
    pub neighborhood: String,
}

// impl Default for MulchOrder
