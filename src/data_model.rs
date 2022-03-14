use serde::{Deserialize, Serialize};
use lazy_static::lazy_static;
use std::sync::{RwLock, Arc};
use chrono::prelude::*;
use std::collections::{BTreeMap};
use rust_decimal::prelude::*;
use gloo_storage::{LocalStorage, Storage};

pub(crate) use crate::data_model_reports::*;
pub(crate) use crate::data_model_orders::*;
pub(crate) use crate::auth_utils::{get_active_user, get_active_user_async};
use crate::gql_utils::{make_gql_request, GraphQlReq};

static ALWAYS_CONFIG_GQL:&'static str =
r#"
{
  config {
    isLocked
    lastModifiedTime
  }
}"#;

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
    users {
      id
      name
      group
    }
  }
}"#;

// Internal Schma version for stored config data.  This gives me a way
//   to force update reload of config even if last_modified_time hasn't changed
static LOCAL_STORE_SCHEMA_VER: u32 = 020501;

lazy_static! {
    static ref NEIGHBORHOODS: RwLock<Option<Arc<Vec<Neighborhood>>>> = RwLock::new(None);
    static ref PRODUCTS: RwLock<Option<Arc<BTreeMap<String, ProductInfo>>>> = RwLock::new(None);
    static ref DELIVERIES: RwLock<Option<Arc<BTreeMap<u32, DeliveryInfo>>>> = RwLock::new(None);
    static ref FRCONFIG: RwLock<Option<Arc<FrConfig>>> = RwLock::new(None);
    // map<uid,(name, group)>
    static ref USER_MAP: RwLock<Arc<BTreeMap<String,UserInfo>>> = RwLock::new(Arc::new(BTreeMap::new()));
}

////////////////////////////////////////////////////////////////////////////
//
#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct UserInfo {
    pub(crate) name: String,
    pub(crate) group: String,
}

////////////////////////////////////////////////////////////////////////////
//
pub(crate) struct FrConfig {
    pub(crate) kind: String,
    pub(crate) description: String,
    pub(crate) last_modified_time: String,
    pub(crate) is_locked: bool,
}

////////////////////////////////////////////////////////////////////////////
//
pub(crate) struct DeliveryInfo {
    pub(crate) delivery_date: DateTime<Utc>,
    pub(crate) new_order_cutoff_date: DateTime<Utc>,
}
impl DeliveryInfo {
    pub(crate) fn get_delivery_date_str(&self) -> String {
        self.delivery_date.format("%Y-%m-%d").to_string()
    }
}

////////////////////////////////////////////////////////////////////////////
//
#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct Neighborhood {
    pub(crate) name: String,
    #[serde(alias = "distributionPoint")]
    pub(crate) distribution_point: String,
}

////////////////////////////////////////////////////////////////////////////
//
#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct ProductPriceBreak {
    pub(crate) gt: u32,
    #[serde(alias = "unitPrice")]
    pub(crate) unit_price: String,
}

////////////////////////////////////////////////////////////////////////////
//
pub(crate) struct ProductInfo {
    pub(crate) label: String,
    pub(crate) min_units: u32,
    pub(crate) unit_price: String,
    pub(crate) price_breaks: Vec<ProductPriceBreak>,
}

////////////////////////////////////////////////////////////////////////////
//
#[derive(Serialize, Deserialize, Debug, Clone)]
struct ConfigApi {
    local_store_schema_ver: Option<u32>,
    config: FrConfigApi,
}

////////////////////////////////////////////////////////////////////////////
//
#[derive(Serialize, Deserialize, Debug, Clone)]
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
    users: Vec<UsersConfigApi>,
}

////////////////////////////////////////////////////////////////////////////
//
#[derive(Serialize, Deserialize, Debug, Clone)]
struct UsersConfigApi {
    id: String,
    name: String,
    group: String,
}

////////////////////////////////////////////////////////////////////////////
//
#[derive(Serialize, Deserialize, Debug, Clone)]
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

////////////////////////////////////////////////////////////////////////////
//
#[derive(Serialize, Deserialize, Debug, Clone)]
struct MulchDeliveryConfigApi {
    id: u32,
    #[serde(alias = "date")]
    delivery_date: String,
    #[serde(alias = "newOrderCutoffDate")]
    new_order_cutoff_date: String,
}

////////////////////////////////////////////////////////////////////////////
//
fn process_config_data(config: FrConfigApi) {
    *FRCONFIG.write().unwrap() = Some(Arc::new(FrConfig {
        kind: config.kind,
        description: config.description,
        last_modified_time: config.last_modified_time,
        is_locked: config.is_locked,
    }));
    *NEIGHBORHOODS.write().unwrap() = Some(Arc::new(config.neighborhoods));

    let mut deliveries = BTreeMap::new();
    for delivery in config.mulch_delivery_configs {
        let delivery_date = NaiveDate::parse_from_str(&delivery.delivery_date, "%m/%d/%Y").unwrap();
        let cutoff_date = NaiveDate::parse_from_str(&delivery.new_order_cutoff_date, "%m/%d/%Y").unwrap();
        deliveries.insert(delivery.id, DeliveryInfo{
            delivery_date: Utc.ymd(delivery_date.year(), delivery_date.month(), delivery_date.day()).and_hms(0, 0, 0),
            new_order_cutoff_date: Utc.ymd(cutoff_date.year(), cutoff_date.month(), cutoff_date.day()).and_hms(0, 0, 0),
        });
    }
    *DELIVERIES.write().unwrap() = Some(Arc::new(deliveries));

    let mut products = BTreeMap::new();
    for product in config.products {
        products.insert(product.id, ProductInfo{
            label: product.label,
            min_units: product.min_units,
            unit_price: product.unit_price,
            price_breaks: product.price_breaks,
        });
    }
    *PRODUCTS.write().unwrap() = Some(Arc::new(products));

    {
        let mut new_map: BTreeMap<String, UserInfo> =
            config.users.into_iter().map(|v| (v.id.clone(), UserInfo{name: v.name.clone(), group: v.group.clone()})).collect::<_>();
        if let Ok(mut arc_umap) = USER_MAP.write() {
            Arc::get_mut(&mut *arc_umap).unwrap().append(&mut new_map);
            Arc::get_mut(&mut *arc_umap).unwrap().insert("fradmin".to_string(), UserInfo{name:"Super User".to_string(), group: "Bear".to_string()});
        }
    }

    log::info!("Fundraising Config retrieved");
}

////////////////////////////////////////////////////////////////////////////
//
pub(crate) async fn load_config() {
    log::info!("Getting Fundraising Config: Loading From LocalStorage");
    let rslt = LocalStorage::get("FrConfig");
    if rslt.is_ok() {
        let stored_config: ConfigApi = rslt.unwrap();

        let req = GraphQlReq::new(ALWAYS_CONFIG_GQL.to_string());
        if let Ok(val) = make_gql_request::<serde_json::Value>(&req).await {
            let is_last_mod_time_check_passed =
                val["config"]["lastModifiedTime"].as_str().unwrap() == stored_config.config.last_modified_time;
            let is_ver_schema_check_passed =
                stored_config.local_store_schema_ver.unwrap_or(0) == LOCAL_STORE_SCHEMA_VER;
            if is_last_mod_time_check_passed && is_ver_schema_check_passed {
                log::info!("Using stored config data");
                process_config_data(stored_config.config);
                return;
            } else {
                log::info!("Config lastModifiedTime doesn't match forcing cache refresh");
            }
        } else {
            log::error!("Error reading lastModifiedTime config from network");
        }
    } else {
        log::info!("No stored config loading from network...");
    }

    let req = GraphQlReq::new(CONFIG_GQL.to_string());
    let rslt = make_gql_request::<ConfigApi>(&req).await;
    if let Err(err) = rslt {
        gloo_dialogs::alert(&format!("Failed to retrieve config: {:#?}", err));
        return;
    }

    let config = {
        let mut frconfig = rslt.unwrap();
        frconfig.local_store_schema_ver = Some(LOCAL_STORE_SCHEMA_VER);
        if let Err(err) = LocalStorage::set("FrConfig", frconfig.clone()) {
            log::error!("Failed to cache config to storage: {:#?}", err);
        } else {
            log::info!("Config cached to storage");
        }
        frconfig.config
    };

    process_config_data(config);
    //log::info!("```` Config: \n {:#?}", config);

}

////////////////////////////////////////////////////////////////////////////
//
pub(crate) fn get_users() -> Arc<BTreeMap<String, UserInfo>> {
    USER_MAP.read().unwrap().clone()
}

////////////////////////////////////////////////////////////////////////////
//
pub(crate) fn get_deliveries() -> Arc<BTreeMap<u32,DeliveryInfo>> {
    DELIVERIES.read().unwrap().as_ref().unwrap().clone()
}

////////////////////////////////////////////////////////////////////////////
//
pub(crate) fn get_delivery_date(delivery_id: &u32) -> String {
    get_deliveries().get(delivery_id).unwrap()
        .delivery_date.format("%Y-%m-%d").to_string()
}

////////////////////////////////////////////////////////////////////////////
//
pub(crate) fn get_neighborhoods() -> Arc<Vec<Neighborhood>>
{
    NEIGHBORHOODS.read().unwrap().as_ref().unwrap().clone()
}

////////////////////////////////////////////////////////////////////////////
//
pub(crate) fn get_products() -> Arc<BTreeMap<String, ProductInfo>>
{
    PRODUCTS.read().unwrap().as_ref().unwrap().clone()
}

////////////////////////////////////////////////////////////////////////////
//
pub(crate) fn get_fr_config() -> Arc<FrConfig> {
    FRCONFIG.read().unwrap().as_ref().unwrap().clone()
}

////////////////////////////////////////////////////////////////////////////
//
pub(crate) fn are_sales_still_allowed() -> bool {
    let deliveries = get_deliveries();
    let now = Utc::now();
    let mut are_any_still_active = false;
    for delivery_info in deliveries.values() {
        if delivery_info.new_order_cutoff_date >= now {
            are_any_still_active = true;
        }
    }
    are_any_still_active
}

////////////////////////////////////////////////////////////////////////////
//
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

////////////////////////////////////////////////////////////////////////////
//
pub(crate) fn is_purchase_valid(product_id: &str, num_sold: u32) -> bool {
    match get_products().get(product_id) {
        None=>false,
        Some(product)=>product.min_units <= num_sold,
    }
}

////////////////////////////////////////////////////////////////////////////
//
// pub(crate) fn get_active_sellers() -> Vec<String> {
//    //TOOD: Need to add GraphQL to get list of active sellers
//    vec![get_active_user().get_id()]
//}


////////////////////////////////////////////////////////////////////////////
//
pub(crate) fn is_fundraiser_locked() -> bool {
    get_fr_config().is_locked
}





////////////////////////////////////////////////////////////////////////////
//
static CREATE_ISSUE_GQL:&'static str =
r#"
mutation {
  createIssue(input: {
    id: "***USERID***",
    title: "***TITLE***",
    body: "***BODY***"
  })
}"#;

////////////////////////////////////////////////////////////////////////////
//
pub(crate) async fn report_new_issue(reporting_id: &str, title: &str, body: &str)
    -> std::result::Result<() ,Box<dyn std::error::Error>>
{
    let query = CREATE_ISSUE_GQL
        .replace("***USERID***", &reporting_id)
        .replace("***TITLE***", &title)
        .replace("***BODY***", &body);

    let req = GraphQlReq::new(query);
    make_gql_request::<serde_json::Value>(&req).await.map(|_| ())
}




////////////////////////////////////////////////////////////////////////////
//
static GET_TIMECARDS_GRAPHQL: &'static str = r"
{
  mulchTimecards(***GET_TIMECARDS_PARAMS***){
    id,
    deliveryId
    timeIn
    timeOut
    timeTotal
  }
}
";

////////////////////////////////////////////////////////////////////////////
//
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub(crate) struct TimeCard{
    #[serde(rename = "id")]
    pub(crate) uid: String,
    #[serde(rename = "deliveryId")]
    pub(crate) delivery_id: u32,
    #[serde(rename = "timeIn")]
    pub(crate) time_in: String,
    #[serde(rename = "timeOut")]
    pub(crate) time_out: String,
    #[serde(rename = "timeTotal")]
    pub(crate) time_total: String,
}

////////////////////////////////////////////////////////////////////////////
//
pub(crate) async fn get_timecards_data(delivery_id: Option<u32>, _uid: Option<String>)
    -> std::result::Result<Vec<(String, String, Option<TimeCard>)> ,Box<dyn std::error::Error>>
{
    let query = if let Some(delivery_id) = delivery_id {
        GET_TIMECARDS_GRAPHQL.replace("***GET_TIMECARDS_PARAMS***", &format!("deliveryId: {}", delivery_id))
    } else {
        GET_TIMECARDS_GRAPHQL.replace("(***GET_TIMECARDS_PARAMS***)", "")
    };
    log::info!("Running Query: {}", &query);

    let mut timecard_map = {
        #[derive(Serialize, Deserialize, Debug)]
        struct GqlResp {
            #[serde(alias = "mulchTimecards")]
            mulch_timecards: Vec<TimeCard>,
        }

        let req = GraphQlReq::new(query);
        let resp = make_gql_request::<GqlResp>(&req).await?;
        let timecard_map: BTreeMap<_, _> = resp.mulch_timecards.into_iter().map(|v|(v.uid.clone(), v)).collect();
        timecard_map
    };

    let timecard_data = (*get_users()).clone()
        .into_iter()
        .filter(|(_,user_info)| "Bear"!=user_info.group && "Bogus"!=user_info.group)
        .map(|(uid,user_info)| {
            let tc = timecard_map.remove(&uid);
            (uid, user_info.name, tc)
        })
        .collect::<Vec<(String, String, Option<TimeCard>)>>();
    //timecard_data.sort_by_key(|k| k.1.clone());  //Shouldn't need this since btree is sorted
    Ok(timecard_data)
}

////////////////////////////////////////////////////////////////////////////
//
static SET_TIMECARDS_GRAPHQL: &'static str = r"
mutation {
  setMulchTimecards(timecards: [
***SET_TIMECARDS_PARAMS***
  ])
}";

////////////////////////////////////////////////////////////////////////////
//
pub(crate) async fn save_timecards_data(timecards: Vec<TimeCard>)
    -> std::result::Result<() ,Box<dyn std::error::Error>>
{
    if timecards.len() == 0 { return Ok(()); }

    let timecards_param = timecards.iter().map(|v|{
        format!("\t\t{{\n{}\n{}\n{}\n{}\n{}\n\t\t}}",
            format!("\t\t\tid: \"{}\",", &v.uid),
            format!("\t\t\tdeliveryId: {},", v.delivery_id),
            format!("\t\t\ttimeIn: \"{}\",", &v.time_in),
            format!("\t\t\ttimeOut: \"{}\",", &v.time_out),
            format!("\t\t\ttimeTotal: \"{}\"", &v.time_total))
    }).collect::<Vec<String>>().join(",\n");

    let query = SET_TIMECARDS_GRAPHQL.replace("***SET_TIMECARDS_PARAMS***", &timecards_param);
    log::info!("Running Query: {}", &query);

    let req = GraphQlReq::new(query);
    let _ = make_gql_request::<serde_json::Value>(&req).await?;
    Ok(())
}
// static TEST_ADMIN_API_GQL:&'static str =
// r#"
// {
//   testApi(param1: "***USERID***")
// }"#;
//
// pub(crate) async fn call_admin_test_api()
//     -> std::result::Result<() ,Box<dyn std::error::Error>>
// {
//     let active_user = get_active_user();
//     let user_id = active_user.get_id();
//     let query = TEST_ADMIN_API_GQL
//         .replace("***USERID***", &user_id);
//
//     let req = GraphQlReq::new(query);
//     make_gql_request::<serde_json::Value>(&req).await.map(|_| ())
// }

