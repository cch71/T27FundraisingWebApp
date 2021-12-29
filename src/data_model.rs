pub(crate) use crate::order_utils::*;
use lazy_static::lazy_static;
use std::sync::Arc;
use chrono::prelude::*;

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

    static ref PRODUCTS: Arc<Vec<ProductInfo>> = {
        let mut products = Vec::new();
        let mut bags_pb = Vec::new();
        bags_pb.push(ProductPriceBreak{
            gt: 15,
            unit_price: "4.00".to_string(),
        });
        bags_pb.push(ProductPriceBreak{
            gt: 35,
            unit_price: "3.85".to_string(),
        });

        products.push(ProductInfo{
            id: "bags".to_string(),
            label: "Bags of Mulch".to_string(),
            unit_price: "4.15".to_string(),
            min_units: 0,
            price_breaks: bags_pb,
        });
        products.push(ProductInfo{
            id: "spreading".to_string(),
            label: "Bags to Spread".to_string(),
            unit_price: "2.00".to_string(),
            min_units: 5,
            price_breaks: Vec::new(),
        });
        Arc::new(products)
    };

    static ref DELIVERIES: Arc<Vec<DeliveryInfo>> = {
        let mut deliveries = Vec::new();
        deliveries.push(DeliveryInfo{
           id: "1".to_string(),
           delivery_date: Utc.ymd(2022, 3, 13).and_hms(0, 0, 0),
           new_order_cutoff_date: Utc.ymd(2022, 2, 10).and_hms(0, 0, 0),
        });
        deliveries.push(DeliveryInfo{
           id: "2".to_string(),
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
    pub(crate) id: String,
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
    pub(crate) id: String,
    pub(crate) label: String,
    pub(crate) min_units: u32,
    pub(crate) unit_price: String,
    pub(crate) price_breaks: Vec<ProductPriceBreak>,
}

pub(crate) fn get_deliveries() -> Arc<Vec<DeliveryInfo>> {
    DELIVERIES.clone()
}

pub(crate) fn get_neighborhoods() -> Arc<Vec<Neighborhood>>
{
    NEIGHBORHOODS.clone()
}

pub(crate) fn get_products() -> Arc<Vec<ProductInfo>>
{
    PRODUCTS.clone()
}

pub(crate) fn get_fr_config() -> Arc<FrConfig> {
    FRCONFIG.clone()
}
