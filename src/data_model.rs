pub(crate) use crate::order_utils::*;
use lazy_static::lazy_static;
use std::sync::Arc;
use chrono::prelude::*;
use std::collections::{HashMap};
use rust_decimal::prelude::*;

lazy_static! {
    static ref NEIGHBORHOODS: Arc<Vec<Neighborhood>> ={
        let mut hoods = Vec::new();
        hoods.push(Neighborhood{
            name: "Bear Valley".to_string(),
            distribution_point: "Walsh".to_string(),
        });
        hoods.push(Neighborhood{
            name: "Out of Area".to_string(),
            distribution_point: "Walsh".to_string(),
        });
        hoods.push(Neighborhood{
            name: "Other...".to_string(),
            distribution_point: "Walsh".to_string(),
        });
        Arc::new(hoods)
    };

    static ref PRODUCTS: Arc<HashMap<String, ProductInfo>> = {
        let mut products = HashMap::new();
        let mut bags_pb = Vec::new();
        bags_pb.push(ProductPriceBreak{
            gt: 15,
            unit_price: "4.00".to_string(),
        });
        bags_pb.push(ProductPriceBreak{
            gt: 35,
            unit_price: "3.85".to_string(),
        });

        products.insert("bags".to_string(), ProductInfo{
            label: "Bags of Mulch".to_string(),
            unit_price: "4.15".to_string(),
            min_units: 5,
            price_breaks: bags_pb,
        });
        products.insert("spreading".to_string(), ProductInfo{
            label: "Bags to Spread".to_string(),
            unit_price: "2.00".to_string(),
            min_units: 0,
            price_breaks: Vec::new(),
        });
        Arc::new(products)
    };

    static ref DELIVERIES: Arc<HashMap<String, DeliveryInfo>> = {
        let mut deliveries = HashMap::new();
        deliveries.insert("1".to_string(), DeliveryInfo{
           delivery_date: Utc.ymd(2022, 3, 13).and_hms(0, 0, 0),
           new_order_cutoff_date: Utc.ymd(2022, 2, 10).and_hms(0, 0, 0),
        });
        deliveries.insert("2".to_string(), DeliveryInfo{
           delivery_date: Utc.ymd(2022, 4, 13).and_hms(0, 0, 0),
           new_order_cutoff_date: Utc.ymd(2022, 3, 27).and_hms(0, 0, 0),
        });
        Arc::new(deliveries)
    };

    static ref FRCONFIG: Arc<FrConfig> = {
        Arc::new(FrConfig{
            kind: "mulch".to_string(),
            description: "Mulch".to_string(),
        })
    };
}

pub(crate) struct FrConfig {
    pub(crate) kind: String,
    pub(crate) description: String,
}

pub(crate) struct DeliveryInfo {
    pub(crate) delivery_date: DateTime<Utc>,
    pub(crate) new_order_cutoff_date: DateTime<Utc>,
}

pub(crate) struct Neighborhood {
    pub(crate) name: String,
    pub(crate) distribution_point: String,
}

pub(crate) struct ProductPriceBreak {
    pub(crate) gt: u32,
    pub(crate) unit_price: String,
}

pub(crate) struct ProductInfo {
    pub(crate) label: String,
    pub(crate) min_units: u32,
    pub(crate) unit_price: String,
    pub(crate) price_breaks: Vec<ProductPriceBreak>,
}

pub(crate) fn get_deliveries() -> Arc<HashMap<String,DeliveryInfo>> {
    DELIVERIES.clone()
}

pub(crate) fn get_delivery_date(delivery_id: &str) -> String {
    DELIVERIES.get(delivery_id).unwrap()
        .delivery_date.format("%Y-%m-%d").to_string()
}

pub(crate) fn get_neighborhoods() -> Arc<Vec<Neighborhood>>
{
    NEIGHBORHOODS.clone()
}

pub(crate) fn get_products() -> Arc<HashMap<String, ProductInfo>>
{
    PRODUCTS.clone()
}

pub(crate) fn get_fr_config() -> Arc<FrConfig> {
    FRCONFIG.clone()
}

pub(crate) fn get_purchase_cost_for(product_id: &str, num_sold: u32) -> String {
    if 0==num_sold { return "0.00".to_string(); }
    let product_info = PRODUCTS.get(product_id).unwrap();

    let mut price_per_unit = Decimal::from_str(&product_info.unit_price).unwrap();
    for price_break in &product_info.price_breaks {
        if price_break.gt < num_sold {
            price_per_unit = Decimal::from_str(&price_break.unit_price).unwrap();
        } else {
            break; //no point in continuing on since price_breaks should be ordered
        }
    }
    let amount = price_per_unit.checked_mul(num_sold.into()).unwrap();
    amount.to_string()
}

pub(crate) fn is_purchase_valid(product_id: &str, num_sold: u32) -> bool {
    match PRODUCTS.get(product_id) {
        None=>false,
        Some(product)=>product.min_units <= num_sold,
    }
}
