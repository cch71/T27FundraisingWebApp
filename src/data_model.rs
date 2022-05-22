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
    finalizationData {
      bankDeposited
      mulchCost
      perBagCost
      profitsFromBags
      mulchSalesGross
      moneyPoolForTroop
      moneyPoolForScoutsSubPools
      moneyPoolForScoutsSales
      moneyPoolForScoutsDelivery
      perBagAvgEarnings
      deliveryEarningsPerMinute
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
    static ref FR_CLOSURE_DATA: RwLock<Arc<BTreeMap<String,FrClosureMapData>>> = RwLock::new(Arc::new(BTreeMap::new()));
    static ref IS_FR_FINALIZED: RwLock<bool> = RwLock::new(false);
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
    #[serde(alias = "finalizationData")]
    finalization_data: Option<FinalizationDataConfigApi>,
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
struct FinalizationDataConfigApi {
    #[serde(alias = "bankDeposited")]
    bank_deposited: String,
    #[serde(alias = "mulchCost")]
    mulch_cost: String,
    #[serde(alias = "perBagCost")]
    per_bag_cost: String,
    #[serde(alias = "profitsFromBags")]
    profits_from_bags: String,
    #[serde(alias = "mulchSalesGross")]
    mulch_sales_gross: String,
    #[serde(alias = "moneyPoolForTroop")]
    money_pool_for_troop: String,
    #[serde(alias = "moneyPoolForScoutsSubPools")]
    money_pool_for_scouts_sub_pools: String,
    #[serde(alias = "moneyPoolForScoutsSales")]
    money_pool_for_scout_sales: String,
    #[serde(alias = "perBagAvgEarnings")]
    per_bag_avg_earnings: String,
    #[serde(alias = "moneyPoolForScoutsDelivery")]
    money_pool_for_scout_delivery: String,
    #[serde(alias = "deliveryEarningsPerMinute")]
    delivery_earnings_per_minute: String,
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
    if let Some(finalization_data) = config.finalization_data {
        // If we have the value AND it isn't set to 0 then we are finalized. If one is set they all should be
        if Decimal::from_str(finalization_data.bank_deposited.as_str()).map_or(false, |v| v > Decimal::ZERO) {
            *IS_FR_FINALIZED.write().unwrap() = true;
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
pub(crate) fn is_fundraiser_finalized()->bool {
    *IS_FR_FINALIZED.read().unwrap()
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
pub(crate) struct FrClosureMapData {
    pub(crate) delivery_time_total: Duration,
    pub(crate) num_bags_sold: u64,
    pub(crate) amount_from_bags_sales: Decimal,
    pub(crate) num_bags_to_spread_sold: u64,
    pub(crate) amount_from_bags_to_spread_sales: Decimal,
    pub(crate) amount_from_donations: Decimal,
    pub(crate) amount_total_collected: Decimal,
    pub(crate) num_bags_spread: Decimal,
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

pub(crate) type FrClosureStaticData = Arc<BTreeMap<String, FrClosureMapData>>;
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
        #[serde(rename = "mulchTimecards")]
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

    fn add_tc(cd: &mut FrClosureMapData, add_dur: Duration) {
        cd.delivery_time_total =
            cd.delivery_time_total.checked_add(add_dur).unwrap();
    }

    fn add_order_data(cd: &mut FrClosureMapData, new_data: &FrClosureMapData) {
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

    fn register_spreaders(closure_data: &mut BTreeMap<String, FrClosureMapData>, mut spreaders: Vec<String>, num_bags: u64) {
        //Due to bug there can be empty spreaders
        spreaders.retain(|v| v.len()>0 );

        if spreaders.len() == 0 {
            return;
        }

        if num_bags == 0 {
            log::error!("We have spreaders but no bags to spread");
            return;
        }

        let num_bags_to_record_as_spread_per_user: Decimal = {
            if spreaders.len() == 1 {
                Decimal::from(num_bags)
            } else {
                Decimal::from(num_bags).checked_div(spreaders.len().into()).unwrap()
            }
        };

        for uid in spreaders {
            if !closure_data.contains_key(&uid) {
                closure_data.insert(uid.clone(), FrClosureMapData::default());
            }

            let mut datum:&mut FrClosureMapData = closure_data.get_mut(&uid).unwrap();
            datum.num_bags_spread = datum.num_bags_spread.checked_add(num_bags_to_record_as_spread_per_user).unwrap();
            let mut datum:&mut FrClosureMapData = closure_data.get_mut("TROOP_TOTALS").unwrap();
            datum.num_bags_spread = datum.num_bags_spread.checked_add(num_bags_to_record_as_spread_per_user).unwrap();
        }
    }

    let mut closure_data = BTreeMap::new();
    closure_data.insert("TROOP_TOTALS".to_string(), FrClosureMapData::default());

    // convert time and total and assign to user
    for tc in resp.time_cards {
        if !closure_data.contains_key(&tc.uid) {
            closure_data.insert(tc.uid.clone(), FrClosureMapData::default());
        }
        let dur = time_val_str_to_duration(tc.time_total.as_str()).unwrap();
        add_tc(closure_data.get_mut(&tc.uid).unwrap(), dur.clone());
        add_tc(closure_data.get_mut("TROOP_TOTALS").unwrap(), dur);
    }

    for order in resp.orders {
        //log::info!("{} Spread order: {:#?}",&order.uid, &order);
        // convert values and total and assign to user
        if !closure_data.contains_key(&order.uid) {
            closure_data.insert(order.uid.clone(), FrClosureMapData::default());
        }

        let new_data = {
            let mut new_data = FrClosureMapData::default();

            new_data.amount_from_donations = order.amount_from_donations
                .map_or(Decimal::ZERO,|v| Decimal::from_str(v.as_str()).unwrap());
            new_data.amount_total_collected = order.amount_total_collected
                .map_or(Decimal::ZERO,|v| Decimal::from_str(v.as_str()).unwrap());
            for purchase in order.purchases {
                if "bags" == purchase.product_id.as_str() && purchase.num_sold != 0 {
                    new_data.num_bags_sold = purchase.num_sold;
                    // Issue #108 hack replace ","->""
                    new_data.amount_from_bags_sales =
                        Decimal::from_str(& purchase.amount_charged.replace(",","")).unwrap();
                } else if "spreading" == purchase.product_id.as_str() && purchase.num_sold !=0 {
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

////////////////////////////////////////////////////////////////////////////
//
#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct FrClosureDynamicData {
    pub(crate) bank_deposited: Option<String>,
    pub(crate) mulch_cost: Option<String>,
}

////////////////////////////////////////////////////////////////////////////
//
pub(crate) fn get_fundraiser_closure_dynamic_data()
    -> Option<FrClosureDynamicData>
{
    log::info!("Getting Fundraiser Closure Data From LocalStorage");
    LocalStorage::get("FrClosureDynamicData").ok()
}

////////////////////////////////////////////////////////////////////////////
//
pub(crate) fn set_fundraiser_closure_dynamic_data(data: FrClosureDynamicData) {
    log::info!("Getting Fundraiser Closure Data From LocalStorage");
    let _ = LocalStorage::set("FrClosureDynamicData", data);
}


////////////////////////////////////////////////////////
///
#[derive(Serialize, Default, Debug, PartialEq, Clone)]
pub(crate) struct FrCloseoutDynamicVars {
    pub(crate) bank_deposited: Decimal,
    pub(crate) mulch_cost: Decimal,
    pub(crate) per_bag_cost: Decimal,
    pub(crate) profits_from_bags: Decimal,
    pub(crate) mulch_sales_gross: Decimal,
    pub(crate) money_pool_for_troop: Decimal,
    pub(crate) money_pool_for_scouts_sub_pools: Decimal,
    pub(crate) money_pool_for_scout_sales: Decimal,
    pub(crate) per_bag_avg_earnings: Decimal,
    pub(crate) money_pool_for_scout_delivery: Decimal,
    pub(crate) delivery_earnings_per_minute: Decimal,
}

impl FrCloseoutDynamicVars {
    pub(crate) fn new()->Self {
        FrCloseoutDynamicVars::default()
    }

}

////////////////////////////////////////////////////////
///
#[derive(Serialize, Default, Debug, PartialEq, Clone)]
pub(crate) struct FrCloseoutAllocationVals {
    pub(crate) name: String,
    pub(crate) uid: String,
    pub(crate) bags_sold: u64,
    pub(crate) bags_spread: Decimal,
    pub(crate) delivery_minutes: Decimal,
    pub(crate) total_donations: Decimal,
    pub(crate) allocation_from_bags_sold: Decimal,
    pub(crate) allocation_from_bags_spread: Decimal,
    pub(crate) allocation_from_delivery: Decimal,
    pub(crate) allocation_total: Decimal,
}

////////////////////////////////////////////////////////////////////////////
//
static SET_FR_CLOSEOUT_CONFIG_DATA_GRAPHQL: &'static str = r#"
mutation {
  updateConfig(config: {
    finalizationData: {
      bankDeposited: "0.0000",
      mulchCost: "0.0000",
      perBagCost: "0.0000",
      profitsFromBags: "0.0000",
      mulchSalesGross: "0.0000",
      moneyPoolForTroop: "0.0000",
      moneyPoolForScoutsSubPools: "0.0000",
      moneyPoolForScoutsSales: "0.0000",
      moneyPoolForScoutsDelivery: "0.0000",
      perBagAvgEarnings: "0.0000",
      deliveryEarningsPerMinute: "0.0000"
    }
  })
}
"#;

////////////////////////////////////////////////////////////////////////////
//
static SET_FR_CLOSEOUT_ALLOC_DATA_GRAPHQL: &'static str = r#"
mutation {
  setFundraiserCloseoutAllocations(
    allocations: [
        ***ALLOCATIONS***
    ]
  )
}
"#;

////////////////////////////////////////////////////////////////////////////
//
static SET_FR_CLOSEOUT_ALLOC_DATUM_GRAPHQL: &'static str = r#"
        {
            uid:,
            bagsSold:,
            bagsSpread:,
            deliveryMinutes:,
            totalDonations:,
            allocationsFromBagsSold:,
            allocationsFromBagsSpread:,
            allocationsFromDelivery:,
            allocationsTotal:
        }
"#;
////////////////////////////////////////////////////////////////////////////
//
pub(crate) async fn set_fr_closeout_data(dvars: &FrCloseoutDynamicVars, allocation_list: &Vec<FrCloseoutAllocationVals>) {

    // Set Config closeout data
    let query = SET_FR_CLOSEOUT_CONFIG_DATA_GRAPHQL
        .replace("bankDeposited: \"0.0000\"", &format!("bankDeposited: \"{}\"", dvars.bank_deposited.round_dp(4).to_string()))
        .replace("mulchCost: \"0.0000\"", &format!("mulchCost: \"{}\"", dvars.mulch_cost.round_dp(4).to_string()))
        .replace("perBagCost: \"0.0000\"", &format!("perBagCost: \"{}\"", dvars.per_bag_cost.round_dp(4).to_string()))
        .replace("profitsFromBags: \"0.0000\"", &format!("profitsFromBags: \"{}\"", dvars.profits_from_bags.round_dp(4).to_string()))
        .replace("mulchSalesGross: \"0.0000\"", &format!("mulchSalesGross: \"{}\"", dvars.mulch_sales_gross.round_dp(4).to_string()))
        .replace("moneyPoolForTroop: \"0.0000\"", &format!("moneyPoolForTroop: \"{}\"", dvars.money_pool_for_troop.round_dp(4).to_string()))
        .replace("moneyPoolForScoutsSubPools: \"0.0000\"", &format!("moneyPoolForScoutsSubPools: \"{}\"", dvars.money_pool_for_scouts_sub_pools.round_dp(4).to_string()))
        .replace("moneyPoolForScoutsSales: \"0.0000\"", &format!("moneyPoolForScoutsSales: \"{}\"", dvars.money_pool_for_scout_sales.round_dp(4).to_string()))
        .replace("perBagAvgEarnings: \"0.0000\"", &format!("perBagAvgEarnings: \"{}\"", dvars.per_bag_avg_earnings.round_dp(4).to_string()))
        .replace("moneyPoolForScoutsDelivery: \"0.0000\"", &format!("moneyPoolForScoutsDelivery: \"{}\"", dvars.money_pool_for_scout_delivery.round_dp(4).to_string()))
        .replace("deliveryEarningsPerMinute: \"0.0000\"", &format!("deliveryEarningsPerMinute: \"{}\"", dvars.delivery_earnings_per_minute.round_dp(4).to_string()));

    let req = GraphQlReq::new(query);
    let _ = make_gql_request::<serde_json::Value>(&req).await.unwrap();

    let query = SET_FR_CLOSEOUT_ALLOC_DATA_GRAPHQL.replace(
        "***ALLOCATIONS***" ,
        allocation_list
        .iter()
        .map(|v|{
            let bags_sold_str = if 0!=v.bags_sold { format!("bagsSold: {},\n", v.bags_sold) } else {"".to_string()};
            let bags_spread_str = if Decimal::ZERO != v.bags_spread {
                format!("bagsSpread: \"{}\",\n", v.bags_spread.round_dp(4).to_string())
            } else {
                "".to_string()
            };
            let delivery_minutes_str = if Decimal::ZERO != v.delivery_minutes {
                format!("deliveryMinutes: \"{}\",\n", v.delivery_minutes.round_dp(4).to_string())
            } else {
                "".to_string()
            };
            let total_donations_str = if Decimal::ZERO != v.total_donations {
                format!("totalDonations: \"{}\",\n", v.total_donations.round_dp(4).to_string())
            } else {
                "".to_string()
            };
            let allocation_from_bags_sold_str = if Decimal::ZERO != v.allocation_from_bags_sold {
                format!("allocationsFromBagsSold: \"{}\",\n", v.allocation_from_bags_sold.round_dp(4).to_string())
            } else {
                "".to_string()
            };
            let allocation_from_bags_spread_str = if Decimal::ZERO != v.allocation_from_bags_spread {
                format!("allocationsFromBagsSpread: \"{}\",\n", v.allocation_from_bags_spread.round_dp(4).to_string())
            } else {
                "".to_string()
            };
            let allocation_from_delivery_str = if Decimal::ZERO != v.allocation_from_delivery {
                format!("allocationsFromDelivery: \"{}\",\n", v.allocation_from_delivery.round_dp(4).to_string())
            } else {
                "".to_string()
            };
            SET_FR_CLOSEOUT_ALLOC_DATUM_GRAPHQL
                .replace("uid:", &format!("uid: \"{}\"", v.uid))
                .replace("allocationsTotal:", &format!("allocationsTotal: \"{}\"", v.allocation_total.round_dp(4).to_string()))
                .replace("bagsSold:,\n", &bags_sold_str)
                .replace("bagsSpread:,\n", &bags_spread_str)
                .replace("deliveryMinutes:,\n", &delivery_minutes_str)
                .replace("totalDonations:,\n", &total_donations_str)
                .replace("allocationsFromBagsSold:,\n", &allocation_from_bags_sold_str)
                .replace("allocationsFromBagsSpread:,\n", &allocation_from_bags_spread_str)
                .replace("allocationsFromDelivery:,\n", &allocation_from_delivery_str)

        }).collect::<Vec<String>>()
        .join(",\n").as_str());

    log::info!("Allocation Mutation:\n{}", &query);
    let req = GraphQlReq::new(query);
    let _ = make_gql_request::<serde_json::Value>(&req).await.unwrap();
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

