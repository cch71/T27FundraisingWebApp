use serde::{Deserialize, Serialize};
pub(crate) use crate::order_utils::*;
pub(crate) use crate::auth_utils::{get_active_user, get_active_user_async, UserInfo};
use lazy_static::lazy_static;
use std::sync::{RwLock, Arc};
use chrono::prelude::*;
use std::collections::{HashMap};
use rust_decimal::prelude::*;

static CONFIG_GQL:&'static str =
r#"
{
  config {
    description
    kind
    isLocked
    lastModifiedTime
    neighborhoods {
      name
      distributionPoint
    }
    mulchDeliveryConfigs {
      id
      date
      newOrderCutoffDate
    }
    products {
      id
      label
      unitPrice
      minUnits
      priceBreaks {
        gt
        unitPrice
      }
    }
  }
}"#;

#[derive(Serialize, Deserialize, Debug)]
struct GqlReq {
    query: String,
}

lazy_static! {
    static ref NEIGHBORHOODS: RwLock<Option<Arc<Vec<Neighborhood>>>> = RwLock::new(None);
    static ref PRODUCTS: RwLock<Option<Arc<HashMap<String, ProductInfo>>>> = RwLock::new(None);
    static ref DELIVERIES: RwLock<Option<Arc<HashMap<u32, DeliveryInfo>>>> = RwLock::new(None);
    static ref FRCONFIG: RwLock<Option<Arc<FrConfig>>> = RwLock::new(None);
}

pub(crate) struct FrConfig {
    pub(crate) kind: String,
    pub(crate) description: String,
    pub(crate) last_modified_time: String,
    pub(crate) is_locked: bool,
}

pub(crate) struct DeliveryInfo {
    pub(crate) delivery_date: DateTime<Utc>,
    pub(crate) new_order_cutoff_date: DateTime<Utc>,
}
impl DeliveryInfo {
    pub(crate) fn get_delivery_date_str(&self) -> String {
        self.delivery_date.format("%Y-%m-%d").to_string()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct Neighborhood {
    pub(crate) name: String,
    #[serde(alias = "distributionPoint")]
    pub(crate) distribution_point: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct ProductPriceBreak {
    pub(crate) gt: u32,
    #[serde(alias = "unitPrice")]
    pub(crate) unit_price: String,
}

pub(crate) struct ProductInfo {
    pub(crate) label: String,
    pub(crate) min_units: u32,
    pub(crate) unit_price: String,
    pub(crate) price_breaks: Vec<ProductPriceBreak>,
}


fn get_cloud_api_url() -> &'static str {
    //AWS API URL
    //invokeUrl: 'https://j0azby8rm6.execute-api.us-east-1.amazonaws.com/prod'
    "https://j0azby8rm6.execute-api.us-east-1.amazonaws.com/prod"
}

pub(crate) async fn load_config() {

    #[derive(Serialize, Deserialize, Debug)]
    struct DataWrapper<T> {
        data: T,
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct ConfigWrapper {
        config: FrConfigApi,
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct FrConfigApi {
        kind: String,
        description: String,
        #[serde(alias = "lastModifiedTime")]
        last_modified_time: String,
        #[serde(alias = "isLocked")]
        is_locked: bool,
        neighborhoods: Vec<Neighborhood>,
        products: Vec<ProductsApi>,
        #[serde(alias = "mulchDeliveryConfigs")]
        mulch_delivery_configs: Vec<MulchDeliveryConfigApi>,
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct ProductsApi {
        id: String,
        label: String,
        #[serde(alias = "minUnits")]
        min_units: u32,
        #[serde(alias = "unitPrice")]
        unit_price: String,
        #[serde(alias = "priceBreaks")]
        price_breaks: Vec<ProductPriceBreak>,
    }
    #[derive(Serialize, Deserialize, Debug)]
    struct MulchDeliveryConfigApi {
        id: u32,
        #[serde(alias = "date")]
        delivery_date: String,
        #[serde(alias = "newOrderCutoffDate")]
        new_order_cutoff_date: String,
    }

    let req_url = format!("{}/graphql", get_cloud_api_url());
    // wasm_bindgen_futures::spawn_local(async move {
    // });
    log::info!("Getting Fundraising Config");
    let req = GqlReq {
        query: CONFIG_GQL.to_string(),
    };

    use reqwasm::http::Request;
    let mut resp: DataWrapper<ConfigWrapper> = Request::post(&req_url)
        .header("Content-Type", "application/json")
        .header("Authorization", &format!("Bearer {}", &get_active_user().token))
        .body(serde_json::to_string(&req).unwrap())
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    let mut config = resp.data.config;
    log::info!("```` Config: \n {:#?}", config);

    *FRCONFIG.write().unwrap() = Some(Arc::new(FrConfig {
        kind: config.kind,
        description: config.description,
        last_modified_time: config.last_modified_time,
        is_locked: config.is_locked,
    }));
    *NEIGHBORHOODS.write().unwrap() = Some(Arc::new(config.neighborhoods));

    let mut deliveries = HashMap::new();
    for delivery in config.mulch_delivery_configs {
        let delivery_date = NaiveDate::parse_from_str(&delivery.delivery_date, "%m/%d/%Y").unwrap();
        let cutoff_date = NaiveDate::parse_from_str(&delivery.new_order_cutoff_date, "%m/%d/%Y").unwrap();
        deliveries.insert(delivery.id, DeliveryInfo{
            delivery_date: Utc.ymd(delivery_date.year(), delivery_date.month(), delivery_date.day()).and_hms(0, 0, 0),
            new_order_cutoff_date: Utc.ymd(cutoff_date.year(), cutoff_date.month(), cutoff_date.day()).and_hms(0, 0, 0),
        });
    }
    *DELIVERIES.write().unwrap() = Some(Arc::new(deliveries));

    let mut products = HashMap::new();
    for product in config.products {
        products.insert(product.id, ProductInfo{
            label: product.label,
            min_units: product.min_units,
            unit_price: product.unit_price,
            price_breaks: product.price_breaks,
        });
    }
    *PRODUCTS.write().unwrap() = Some(Arc::new(products));
}

pub(crate) fn get_deliveries() -> Arc<HashMap<u32,DeliveryInfo>> {
    DELIVERIES.read().unwrap().as_ref().unwrap().clone()
}

pub(crate) fn get_delivery_date(delivery_id: &u32) -> String {
    get_deliveries().get(delivery_id).unwrap()
        .delivery_date.format("%Y-%m-%d").to_string()
}

pub(crate) fn get_neighborhoods() -> Arc<Vec<Neighborhood>>
{
    NEIGHBORHOODS.read().unwrap().as_ref().unwrap().clone()
}

pub(crate) fn get_products() -> Arc<HashMap<String, ProductInfo>>
{
    PRODUCTS.read().unwrap().as_ref().unwrap().clone()
}

pub(crate) fn get_fr_config() -> Arc<FrConfig> {
    FRCONFIG.read().unwrap().as_ref().unwrap().clone()
}

pub(crate) fn are_sales_still_allowed() -> bool {
    true
}

pub(crate) fn get_purchase_cost_for(product_id: &str, num_sold: u32) -> String {
    if 0==num_sold { return "0.00".to_string(); }
    let products = get_products();
    let product_info = products.get(product_id).unwrap();

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
    match get_products().get(product_id) {
        None=>false,
        Some(product)=>product.min_units <= num_sold,
    }
}

