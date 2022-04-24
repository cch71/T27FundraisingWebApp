use serde::{Deserialize, Serialize};
use lazy_static::lazy_static;
use std::sync::{RwLock, Arc};
use chrono::prelude::*;
use std::collections::{BTreeMap};
use rust_decimal::prelude::*;
use gloo_storage::{LocalStorage, Storage};
use std::time::{ Duration };

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
    static ref FR_CLOSURE_DATA: RwLock<Arc<BTreeMap<String,ClosureData>>> = RwLock::new(Arc::new(BTreeMap::new()));
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
pub(crate) fn get_username_from_id(uid: &str) -> Option<String> {
    USER_MAP.read().unwrap().get(uid).and_then(|v|Some(v.name.clone()))
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
pub(crate) fn get_neighborhood(hood: &str) -> Option<Neighborhood>
{
    NEIGHBORHOODS.read()
        .unwrap()
        .as_ref()
        .and_then(|v|v.iter().find(|&v|v.name == hood).cloned())
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


////////////////////////////////////////////////////////////////////////////
//
static GET_FR_CLOSURE_DATA_GRAPHQL: &'static str = r"
{
  mulchTimecards{
    id,
    timeTotal
  }
  mulchOrders {
    ownerId
    purchases {
        productId
        numSold
        amountCharged
    }
    amountFromDonations
    amountTotalCollected
    spreaders
  }
}
";

////////////////////////////////////////////////////////////////////////////
//
#[derive(Default, Debug, Clone, PartialEq)]
pub(crate) struct ClosureData {
    pub(crate) delivery_time_total: Duration,
    pub(crate) num_bags_sold: u64,
    pub(crate) amount_from_bags_sales: Decimal,
    pub(crate) num_bags_to_spread_sold: u64,
    pub(crate) amount_from_bags_to_spread_sales: Decimal,
    pub(crate) amount_from_donations: Decimal,
    pub(crate) amount_total_collected: Decimal,
    pub(crate) num_bags_spread: u64,
}

/////////////////////////////////////////////////
///
pub(crate) fn duration_to_time_val_str(dur: &Duration) -> String {
    let new_hours:u64 = (dur.as_secs() as f64 / (60.0*60.0)).floor() as u64;
    let new_mins:u64 = ((dur.as_secs() as f64 % (60.0*60.0)) / 60.0).floor() as u64;
    format!("{:02}:{:02}", new_hours, new_mins)
}

/////////////////////////////////////////////////
///
pub(crate) fn time_val_str_to_duration(time_val_str: &str) -> Option<Duration> {
    let mut time_val_str = time_val_str.split(":").map(|v|v.to_string()).collect::<Vec<String>>();
    if time_val_str.len() == 3 { //If vector is server time
        time_val_str.pop();
    }

    if time_val_str.len() == 2 {
        return time_val_str[0]
            .parse::<u64>().ok()
            .and_then(|v1|Some(Duration::from_secs(v1*60*60)))
            .and_then(|v1| {
                time_val_str[1]
                    .parse::<u64>().ok()
                    .and_then(|v2|Some(Duration::from_secs(v2*60)))
                    .and_then(|v2| v1.checked_add(v2))
            });
    }
    None
}

pub(crate) type FrClosureStaticData = Arc<BTreeMap<String, ClosureData>>;
////////////////////////////////////////////////////////////////////////////
//
pub(crate) async fn get_fundraiser_closure_static_data()
    -> std::result::Result<FrClosureStaticData ,Box<dyn std::error::Error>>
{
    if let Ok(closure_data) = FR_CLOSURE_DATA.read() {
        if closure_data.len() > 0 {
            return Ok(closure_data.clone());
        }
    }

    #[derive(Deserialize, Debug)]
    struct TimecardClosureData {
        #[serde(rename = "id")]
        uid: String,
        #[serde(rename = "timeTotal")]
        time_total: String,
    }
    #[derive(Deserialize, Debug)]
    pub(crate) struct PurchasedItemsClosureData {
        #[serde(alias = "productId")]
        pub(crate) product_id: String,
        #[serde(alias = "numSold")]
        pub(crate) num_sold: u64,
        #[serde(alias = "amountCharged")]
        pub(crate) amount_charged: String,
    }
    #[derive(Deserialize, Debug)]
    struct OrdersClosureData {
        #[serde(rename = "ownerId")]
        uid: String,
        #[serde(rename = "amountFromDonations")]
        amount_from_donations: Option<String>,
        #[serde(rename = "amountTotalCollected")]
        amount_total_collected: Option<String>,
        purchases: Vec<PurchasedItemsClosureData>,
        spreaders: Vec<String>,
    }
    #[derive(Deserialize, Debug)]
    struct RespClosureData {
        #[serde(rename = "mulchTimeCards")]
        time_cards: Vec<TimecardClosureData>,
        #[serde(rename = "mulchOrders")]
        orders: Vec<OrdersClosureData>,
    }

    let resp = {
        let query = GET_FR_CLOSURE_DATA_GRAPHQL.to_string();
        log::info!("Running Query: {}", &query);
        let req = GraphQlReq::new(query);
        make_gql_request::<RespClosureData>(&req).await?
    };

    fn add_tc(cd: &mut ClosureData, add_dur: Duration) {
        cd.delivery_time_total =
            cd.delivery_time_total.checked_add(add_dur).unwrap();
    }

    fn add_order_data(cd: &mut ClosureData, new_data: &ClosureData) {
        cd.amount_from_donations =
            cd.amount_from_donations.checked_add(new_data.amount_from_donations).unwrap();
        cd.amount_total_collected =
            cd.amount_total_collected.checked_add(new_data.amount_total_collected).unwrap();
        cd.num_bags_sold += new_data.num_bags_sold;
        cd.amount_from_bags_sales =
            cd.amount_from_bags_sales.checked_add(new_data.amount_from_bags_sales).unwrap();
        cd.num_bags_to_spread_sold += new_data.num_bags_to_spread_sold;
        cd.amount_from_bags_to_spread_sales =
            cd.amount_from_bags_to_spread_sales.checked_add(new_data.amount_from_bags_to_spread_sales).unwrap();
    }

    fn register_spreaders(closure_data: &mut BTreeMap<String, ClosureData>, mut spreaders: Vec<String>, num_bags: u64) {
        //Due to bug there can be empty spreaders
        spreaders.retain(|v| v.len()>0 );

        if spreaders.len() == 0 {
            return;
        }

        if num_bags == 0 {
            log::error!("We have spreaders but no bags to spread");
            return;
        }

        let num_bags_to_record_as_spread_per_user: u64 = {
            if spreaders.len() == 1 {
                num_bags
            } else {
                (((num_bags as f64)/(spreaders.len() as f64)).floor()) as u64
            }
        };

        for uid in spreaders {
            if !closure_data.contains_key(&uid) {
                closure_data.insert(uid.clone(), ClosureData::default());
            }

            closure_data.get_mut(&uid).unwrap().num_bags_spread += num_bags_to_record_as_spread_per_user;
            closure_data.get_mut("TROOP_TOTALS").unwrap().num_bags_spread += num_bags_to_record_as_spread_per_user;
        }
    }

    let mut closure_data = BTreeMap::new();
    closure_data.insert("TROOP_TOTALS".to_string(), ClosureData::default());

    // convert time and total and assign to user
    for tc in resp.time_cards {
        if !closure_data.contains_key(&tc.uid) {
            closure_data.insert(tc.uid.clone(), ClosureData::default());
        }
        let dur = time_val_str_to_duration(tc.time_total.as_str()).unwrap();
        add_tc(closure_data.get_mut(&tc.uid).unwrap(), dur.clone());
        add_tc(closure_data.get_mut("TROOP_TOTALS").unwrap(), dur);
    }

    for order in resp.orders {
        // convert values and total and assign to user
        if !closure_data.contains_key(&order.uid) {
            closure_data.insert(order.uid.clone(), ClosureData::default());
        }

        let new_data = {
            let mut new_data = ClosureData::default();

            new_data.amount_from_donations = order.amount_from_donations
                .map_or(Decimal::ZERO,|v| Decimal::from_str(v.as_str()).unwrap());
            new_data.amount_total_collected = order.amount_total_collected
                .map_or(Decimal::ZERO,|v| Decimal::from_str(v.as_str()).unwrap());
            for purchase in order.purchases {
                if "bags" == purchase.product_id.as_str() {
                    new_data.num_bags_sold = purchase.num_sold;
                    new_data.amount_from_bags_sales =
                        Decimal::from_str(& purchase.amount_charged).unwrap();
                } else if "spreading" == purchase.product_id.as_str() {
                    new_data.num_bags_to_spread_sold = purchase.num_sold;
                    new_data.amount_from_bags_to_spread_sales =
                        Decimal::from_str(& purchase.amount_charged).unwrap();
                }
            }

            new_data
        };

        add_order_data(closure_data.get_mut(&order.uid).unwrap(), &new_data);
        add_order_data(closure_data.get_mut("TROOP_TOTALS").unwrap(), &new_data);

        register_spreaders(&mut closure_data, order.spreaders, new_data.num_bags_to_spread_sold);
    }

    if let Ok(mut arc_map) = FR_CLOSURE_DATA.write() {
        Arc::get_mut(&mut *arc_map).unwrap().append(&mut closure_data);
    }

    Ok(FR_CLOSURE_DATA.read().unwrap().clone())
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

