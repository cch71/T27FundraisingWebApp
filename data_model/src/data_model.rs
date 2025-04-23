use super::{
    get_active_user,
    gql_utils::{GraphQlReq, make_gql_request},
};
use chrono::prelude::*;
use gloo::storage::{LocalStorage, Storage};
use rust_decimal::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::{Arc, LazyLock, RwLock};
use std::time::Duration;
use yew::prelude::*;

static ALWAYS_CONFIG_GQL: &str = r#"
{
  config {
    isLocked
    lastModifiedTime
  }
}"#;

static CONFIG_GQL: &str = r#"
{
  config {
    description
    kind
    isLocked
    lastModifiedTime
    neighborhoods {
      name
      city
      zipcode
      isVisible
      distributionPoint
    }
    mulchDeliveryConfigs {
      id
      timezone
      date
      dateAsEpoch
      newOrderCutoffDate
      newOrderCutoffDateAsEpoch
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

// Internal Schema version for stored config data.  This gives me a way
//   to force update reload of config even if last_modified_time hasn't changed
static LOCAL_STORE_SCHEMA_VER: u32 = 20501;

pub type UserMapType = BTreeMap<String, UserInfo>;
type ProductMapType = BTreeMap<String, ProductInfo>;
type DeliveryMapType = BTreeMap<u32, DeliveryInfo>;

static NEIGHBORHOODS: LazyLock<RwLock<Option<Arc<Vec<Neighborhood>>>>> =
    LazyLock::new(|| RwLock::new(None));
static PRODUCTS: LazyLock<RwLock<Option<Arc<ProductMapType>>>> =
    LazyLock::new(|| RwLock::new(None));
static DELIVERIES: LazyLock<RwLock<Option<Arc<DeliveryMapType>>>> =
    LazyLock::new(|| RwLock::new(None));
static FRCONFIG: LazyLock<RwLock<Option<Arc<FrConfig>>>> = LazyLock::new(|| RwLock::new(None));
// map<uid,(name, group)>
static USER_MAP: LazyLock<RwLock<Arc<UserMapType>>> =
    LazyLock::new(|| RwLock::new(Arc::new(BTreeMap::new())));
static FR_CLOSURE_DATA: LazyLock<RwLock<Arc<BTreeMap<String, FrClosureMapData>>>> =
    LazyLock::new(|| RwLock::new(Arc::new(BTreeMap::new())));

////////////////////////////////////////////////////////////////////////////
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct UserInfo {
    pub name: String,
    pub group: String,
}

////////////////////////////////////////////////////////////////////////////
pub struct FrConfig {
    pub kind: String,
    pub description: String,
    // pub last_modified_time: String,
    pub is_locked: bool,
    pub is_finalized: bool,
}

////////////////////////////////////////////////////////////////////////////
#[derive(Debug, Clone)]
pub struct DeliveryInfo {
    delivery_date_api_str: String,
    new_order_cutoff_date_api_str: String,
    delivery_date_str: String,
    new_order_cutoff_date_str: String,
    new_order_cutoff_date: Option<DateTime<Utc>>,
}
impl DeliveryInfo {
    pub fn new(
        delivery_date_raw_api_str: String,
        new_order_cutoff_raw_api_str: String,
        new_order_cutoff_epoch: u32,
    ) -> DeliveryInfo {
        let formatted_delivery_date_str = {
            let nd = NaiveDate::parse_from_str(&delivery_date_raw_api_str, "%m/%d/%Y").unwrap();
            // Utc.with_ymd_and_hms(nd.year(), nd.month(), nd.day(), 0, 0, 0)
            //    .unwrap()
            nd.format("%Y-%m-%d").to_string()
        };
        let formatted_cutoff_date_str = {
            let nd = NaiveDate::parse_from_str(&new_order_cutoff_raw_api_str, "%m/%d/%Y").unwrap();
            // let Utc.with_ymd_and_hms(nd.year(), nd.month(), nd.day(), 0, 0, 0)
            //     .unwrap()
            nd.format("%Y-%m-%d").to_string()
        };

        let cutoff_date = Utc.timestamp_opt(new_order_cutoff_epoch as i64, 0).unwrap();

        DeliveryInfo {
            delivery_date_api_str: delivery_date_raw_api_str,
            new_order_cutoff_date_api_str: new_order_cutoff_raw_api_str,
            delivery_date_str: formatted_delivery_date_str,
            new_order_cutoff_date_str: formatted_cutoff_date_str,
            new_order_cutoff_date: Some(cutoff_date),
        }
    }

    pub fn new_from_admin(
        delivery_date_raw_str: String,
        order_cutoff_raw_str: String,
    ) -> DeliveryInfo {
        let formatted_delivery_date_api_str = {
            let nd = NaiveDate::parse_from_str(&delivery_date_raw_str, "%Y-%m-%d").unwrap();
            // Utc.with_ymd_and_hms(nd.year(), nd.month(), nd.day(), 0, 0, 0).unwrap()
            nd.format("%m/%d/%Y").to_string()
        };
        let formatted_new_order_cutoff_date_api_str = {
            let nd = NaiveDate::parse_from_str(&order_cutoff_raw_str, "%Y-%m-%d").unwrap();
            // Utc.with_ymd_and_hms(nd.year(), nd.month(), nd.day(), 0, 0, 0).unwrap()
            nd.format("%m/%d/%Y").to_string()
        };

        DeliveryInfo {
            delivery_date_api_str: formatted_delivery_date_api_str,
            new_order_cutoff_date_api_str: formatted_new_order_cutoff_date_api_str,
            delivery_date_str: delivery_date_raw_str,
            new_order_cutoff_date_str: order_cutoff_raw_str,
            new_order_cutoff_date: None,
        }
    }

    pub fn get_delivery_date_str(&self) -> String {
        // self.delivery_date.format("%Y-%m-%d").to_string()
        self.delivery_date_str.clone()
    }

    pub fn get_new_order_cutoff_date_str(&self) -> String {
        // self.new_order_cutoff_date.format("%Y-%m-%d").to_string()
        self.new_order_cutoff_date_str.clone()
    }

    pub fn get_api_delivery_date_str(&self) -> String {
        // self.delivery_date.format("%m/%d/%Y").to_string()
        self.delivery_date_api_str.clone()
    }

    pub fn get_api_new_order_cutoff_date_str(&self) -> String {
        // self.new_order_cutoff_date.format("%m/%d/%Y").to_string()
        self.new_order_cutoff_date_api_str.clone()
    }

    pub fn can_take_orders(&self) -> bool {
        self.new_order_cutoff_date.is_none_or(|v| v.ge(&Utc::now()))
    }
}

////////////////////////////////////////////////////////////////////////////
pub fn are_sales_still_allowed() -> bool {
    let deliveries = get_deliveries();
    let mut are_any_still_active = false;
    for delivery_info in deliveries.values() {
        if delivery_info.can_take_orders() {
            are_any_still_active = true;
        }
    }
    are_any_still_active
}

////////////////////////////////////////////////////////////////////////////
#[derive(Serialize, Deserialize, Properties, Debug, Clone, PartialEq)]
pub struct Neighborhood {
    pub name: String,
    pub zipcode: Option<u32>,
    pub city: Option<String>,
    #[serde(alias = "isVisible")]
    pub is_visible: bool,
    #[serde(alias = "distributionPoint")]
    pub distribution_point: String,
}

////////////////////////////////////////////////////////////////////////////
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProductPriceBreak {
    pub gt: u32,
    #[serde(alias = "unitPrice")]
    pub unit_price: String,
}

////////////////////////////////////////////////////////////////////////////
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProductInfo {
    pub label: String,
    pub min_units: u32,
    pub unit_price: String,
    pub price_breaks: Vec<ProductPriceBreak>,
}

////////////////////////////////////////////////////////////////////////////
#[derive(Serialize, Deserialize, Debug, Clone)]
struct ConfigApi {
    local_store_schema_ver: Option<u32>,
    config: FrConfigApi,
}

////////////////////////////////////////////////////////////////////////////
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
#[derive(Serialize, Deserialize, Debug, Clone)]
struct UsersConfigApi {
    id: String,
    name: String,
    group: String,
}

////////////////////////////////////////////////////////////////////////////
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
#[derive(Serialize, Deserialize, Debug, Clone)]
struct MulchDeliveryConfigApi {
    id: u32,
    #[serde(rename = "timezone")]
    timezone: String,
    #[serde(rename = "date")]
    delivery_date: String,
    #[serde(rename = "dateAsEpoch")]
    delivery_date_as_epoch: u32,
    #[serde(rename = "newOrderCutoffDate")]
    new_order_cutoff_date: String,
    #[serde(rename = "newOrderCutoffDateAsEpoch")]
    new_order_cutoff_date_as_epoch: u32,
}

////////////////////////////////////////////////////////////////////////////
fn process_config_data(config: FrConfigApi) {
    let is_config_finalized = match config.finalization_data.as_ref() {
        Some(finalized_data) => !finalized_data.money_pool_for_scout_delivery.is_empty(),
        None => false,
    };

    *FRCONFIG.write().unwrap() = Some(Arc::new(FrConfig {
        kind: config.kind,
        description: config.description,
        // last_modified_time: config.last_modified_time,
        is_locked: config.is_locked,
        is_finalized: is_config_finalized,
    }));
    *NEIGHBORHOODS.write().unwrap() = Some(Arc::new(config.neighborhoods));

    let mut deliveries = BTreeMap::new();
    for delivery in config.mulch_delivery_configs {
        let delivery_info = DeliveryInfo::new(
            delivery.delivery_date.clone(),
            delivery.new_order_cutoff_date.clone(),
            delivery.new_order_cutoff_date_as_epoch,
        );
        deliveries.insert(delivery.id, delivery_info);
    }
    *DELIVERIES.write().unwrap() = Some(Arc::new(deliveries));

    let mut products = BTreeMap::new();
    for product in config.products {
        products.insert(
            product.id,
            ProductInfo {
                label: product.label,
                min_units: product.min_units,
                unit_price: product.unit_price,
                price_breaks: product.price_breaks,
            },
        );
    }
    *PRODUCTS.write().unwrap() = Some(Arc::new(products));

    {
        let mut new_map: BTreeMap<String, UserInfo> = config
            .users
            .into_iter()
            .map(|v| {
                (
                    v.id.clone(),
                    UserInfo {
                        name: v.name.clone(),
                        group: v.group.clone(),
                    },
                )
            })
            .collect::<_>();
        if let Ok(mut arc_user_map) = USER_MAP.write() {
            Arc::get_mut(&mut *arc_user_map)
                .unwrap()
                .append(&mut new_map);
            Arc::get_mut(&mut *arc_user_map).unwrap().insert(
                "fradmin".to_string(),
                UserInfo {
                    name: "Super User".to_string(),
                    group: "Admins".to_string(),
                },
            );
        }
    }
    log::info!("Fundraising Config retrieved");
}

////////////////////////////////////////////////////////////////////////////
pub async fn load_config() {
    log::info!("Getting Fundraising Config: Loading From LocalStorage");
    let rslt = LocalStorage::get("FrConfig");
    if rslt.is_ok() {
        let stored_config: ConfigApi = rslt.unwrap();

        let req = GraphQlReq::new(ALWAYS_CONFIG_GQL);
        match make_gql_request::<serde_json::Value>(&req).await {
            Ok(val) => {
                let is_last_mod_time_check_passed =
                    val["config"]["lastModifiedTime"].as_str().unwrap()
                        == stored_config.config.last_modified_time;
                let is_ver_schema_check_passed =
                    stored_config.local_store_schema_ver.unwrap_or(0) == LOCAL_STORE_SCHEMA_VER;
                if is_last_mod_time_check_passed && is_ver_schema_check_passed {
                    log::info!("Using stored config data");
                    process_config_data(stored_config.config);
                    return;
                } else {
                    log::info!("Config lastModifiedTime doesn't match forcing cache refresh");
                }
            }
            _ => {
                log::error!("Error reading lastModifiedTime config from network");
            }
        }
    } else {
        log::info!("No stored config loading from network...");
    }

    let req = GraphQlReq::new(CONFIG_GQL);
    let rslt = make_gql_request::<ConfigApi>(&req).await;
    if let Err(err) = rslt {
        gloo::dialogs::alert(&format!("Failed to retrieve config: {:#?}", err));
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
pub fn get_users() -> Arc<BTreeMap<String, UserInfo>> {
    USER_MAP.read().unwrap().clone()
}

////////////////////////////////////////////////////////////////////////////
pub fn get_username_from_id(uid: &str) -> Option<String> {
    USER_MAP.read().unwrap().get(uid).map(|v| v.name.clone())
}

////////////////////////////////////////////////////////////////////////////
static SET_PRODUCTS_GQL: &str = r#"
mutation {
  updateConfig(config: {
    products: [
      {
          id: "bags",
          label: "Bags of Mulch",
          unitPrice: "***MULCH_UNIT_PRICE***",
          minUnits: ***MULCH_MIN_UNITS***,
          priceBreaks: [
            ***MULCH_PRICE_BREAKS***
          ]
      },{
          id: "spreading",
          label: "Bags to Spread",
          unitPrice: "***SPREADING_UNIT_PRICE***"
      }
    ]
  })
}"#;

////////////////////////////////////////////////////////////////////////////
pub async fn set_products(
    products: BTreeMap<String, ProductInfo>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mulch_info = products.get("bags").unwrap();
    let spreading_info = products.get("spreading").unwrap();

    let mulch_price_breaks_str = mulch_info
        .price_breaks
        .iter()
        .map(|v| {
            format!(
                "\t\t{{\n{}\n{}\n\t\t}}",
                format_args!("\t\t\tgt: {},", v.gt),
                format_args!("\t\t\tunitPrice: \"{}\",", v.unit_price)
            )
        })
        .collect::<Vec<String>>()
        .join(",");

    let query = SET_PRODUCTS_GQL
        .replace("***SPREADING_UNIT_PRICE***", &spreading_info.unit_price)
        .replace("***MULCH_UNIT_PRICE***", &mulch_info.unit_price)
        .replace("***MULCH_MIN_UNITS***", &mulch_info.min_units.to_string())
        .replace("***MULCH_PRICE_BREAKS***", &mulch_price_breaks_str);

    log::info!("Set Product Mutation:\n{}", &query);
    let req = GraphQlReq::new(query);
    make_gql_request::<serde_json::Value>(&req).await.map(|_| {
        *PRODUCTS.write().unwrap() = Some(Arc::new(products));
    })
}

////////////////////////////////////////////////////////////////////////////
static SET_DELIVERIES_GQL: &str = r#"
mutation {
  updateConfig(config: {
    mulchDeliveryConfigs: [
        ***DELIVERIES_PARAMS***
    ]
  })
}"#;

////////////////////////////////////////////////////////////////////////////
pub async fn set_deliveries(
    deliveries: BTreeMap<u32, DeliveryInfo>,
) -> Result<(), Box<dyn std::error::Error>> {
    let deliveries_str = deliveries
        .iter()
        .map(|(k, v)| {
            format!(
                "\t\t{{\n{}\n{}\n{}\n{}\n\t\t}}",
                format_args!("\t\t\tid: {},", k),
                "\t\t\ttimezone: \"America/Chicago\",", //Hardcoded for now
                format_args!("\t\t\tdate: \"{}\",", v.get_api_delivery_date_str()),
                format_args!(
                    "\t\t\tnewOrderCutoffDate: \"{}\"",
                    v.get_api_new_order_cutoff_date_str()
                )
            )
        })
        .collect::<Vec<String>>()
        .join(",");

    let query = SET_DELIVERIES_GQL.replace("***DELIVERIES_PARAMS***", &deliveries_str);

    // log::info!("Set Delivery Mutation:\n{}", &query);
    let req = GraphQlReq::new(query);
    make_gql_request::<serde_json::Value>(&req).await.map(|_| {
        *DELIVERIES.write().unwrap() = Some(Arc::new(deliveries));
    })
}

////////////////////////////////////////////////////////////////////////////
pub fn get_deliveries() -> Arc<BTreeMap<u32, DeliveryInfo>> {
    DELIVERIES.read().unwrap().as_ref().unwrap().clone()
}

////////////////////////////////////////////////////////////////////////////
pub fn get_delivery_date(delivery_id: &u32) -> String {
    get_deliveries()
        .get(delivery_id)
        .unwrap()
        .get_delivery_date_str()
}

////////////////////////////////////////////////////////////////////////////
static UPDATE_NEIGHBORHOODS_GQL: &str = r#"
mutation {
  addOrUpdateNeighborhoods( neighborhoods: [
    ***HOOD_PARAMS***
  ])
}"#;

////////////////////////////////////////////////////////////////////////////
pub async fn update_neighborhoods(
    hoods: Vec<Neighborhood>,
) -> Result<(), Box<dyn std::error::Error>> {
    let neighborhoods_str = hoods
        .iter()
        .map(|v| {
            format!(
                "\t\t{{\n{},\n{},{}{}\n{}\n\t\t}}",
                format_args!("\t\t\tname: \"{}\"", v.name),
                format_args!("\t\t\tdistributionPoint: \"{}\"", v.distribution_point),
                "***CITY***",
                "***ZIP***",
                format_args!("\t\t\tisVisible: {}", v.is_visible)
            )
            .replace(
                "***CITY***",
                &v.city
                    .as_ref()
                    .map(|v| format!("\t\t\tcity: \"{}\",", v))
                    .unwrap_or("".to_string()),
            )
            .replace(
                "***ZIP***",
                &v.zipcode
                    .as_ref()
                    .map(|v| format!("\t\t\tzipcode: {},", v))
                    .unwrap_or("".to_string()),
            )
        })
        .collect::<Vec<String>>()
        .join(",");

    let query = UPDATE_NEIGHBORHOODS_GQL.replace("***HOOD_PARAMS***", &neighborhoods_str);

    // log::info!("Set Delivery Mutation:\n{}", &query);
    let req = GraphQlReq::new(query);
    make_gql_request::<serde_json::Value>(&req)
        .await
        .map(|_| ())?;

    // I don't know if there is any better way. Making DB Query costs money
    // Trying to merge in place would also take multiple passes through the neighborhood list
    // so converting it into a map and then replacing list with values

    // Map neighborhood names to neighborhood and add ability to mark dirty
    let mut merged_hoods = (*get_neighborhoods())
        .iter()
        .map(|ni| (ni.name.clone(), ni.clone()))
        .collect::<BTreeMap<String, Neighborhood>>();

    for hood in hoods {
        merged_hoods.insert(hood.name.clone(), hood);
    }

    *NEIGHBORHOODS.write().unwrap() = Some(Arc::new(
        merged_hoods.into_values().collect::<Vec<Neighborhood>>(),
    ));
    Ok(())
}

////////////////////////////////////////////////////////////////////////////
pub fn get_neighborhoods() -> Arc<Vec<Neighborhood>> {
    NEIGHBORHOODS.read().unwrap().as_ref().unwrap().clone()
}

////////////////////////////////////////////////////////////////////////////
pub fn get_neighborhood<T: AsRef<str>>(hood: T) -> Option<Neighborhood> {
    NEIGHBORHOODS
        .read()
        .unwrap()
        .as_ref()
        .and_then(|v| v.iter().find(|&v| v.name == hood.as_ref()).cloned())
}

////////////////////////////////////////////////////////////////////////////
pub fn get_city_and_zip_from_neighborhood<T: AsRef<str>>(hood: T) -> Option<(String, u32)> {
    get_neighborhood(hood).and_then(|v| {
        if v.city.is_some() && v.zipcode.is_some() {
            Some((v.city.unwrap(), v.zipcode.unwrap()))
        } else {
            None
        }
    })
}

////////////////////////////////////////////////////////////////////////////
pub fn get_products() -> Arc<BTreeMap<String, ProductInfo>> {
    PRODUCTS.read().unwrap().as_ref().unwrap().clone()
}

////////////////////////////////////////////////////////////////////////////
pub fn get_fr_config() -> Arc<FrConfig> {
    FRCONFIG.read().unwrap().as_ref().unwrap().clone()
}

////////////////////////////////////////////////////////////////////////////
pub fn get_purchase_cost_for(product_id: &str, num_sold: u32) -> String {
    if 0 == num_sold {
        return "0.00".to_string();
    }
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
pub fn is_purchase_valid(product_id: &str, num_sold: u32) -> bool {
    match get_products().get(product_id) {
        None => false,
        Some(product) => product.min_units <= num_sold,
    }
}

////////////////////////////////////////////////////////////////////////////
pub fn is_fundraiser_locked() -> bool {
    get_fr_config().is_locked
}

////////////////////////////////////////////////////////////////////////////
pub fn is_fundraiser_finalized() -> bool {
    get_fr_config().is_finalized
}

////////////////////////////////////////////////////////////////////////////
pub fn is_fundraiser_editable() -> bool {
    let is_fr_readonly = is_fundraiser_locked() || is_fundraiser_finalized();
    !is_fr_readonly || (get_active_user().get_id() == "fradmin")
}

////////////////////////////////////////////////////////////////////////////
static CREATE_ISSUE_GQL: &str = r#"
mutation {
  createIssue(input: {
    id: "***USERID***",
    title: "***TITLE***",
    body: "***BODY***"
  })
}"#;

////////////////////////////////////////////////////////////////////////////
pub async fn report_new_issue(
    reporting_id: &str,
    title: &str,
    body: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let query = CREATE_ISSUE_GQL
        .replace("***USERID***", reporting_id)
        .replace("***TITLE***", title)
        .replace("***BODY***", body);

    let req = GraphQlReq::new(query);
    make_gql_request::<serde_json::Value>(&req)
        .await
        .map(|_| ())
}

////////////////////////////////////////////////////////////////////////////
static GET_TIMECARDS_GRAPHQL: &str = r"
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
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct TimeCard {
    #[serde(rename = "id")]
    pub uid: String,
    #[serde(rename = "deliveryId")]
    pub delivery_id: u32,
    #[serde(rename = "timeIn")]
    pub time_in: String,
    #[serde(rename = "timeOut")]
    pub time_out: String,
    #[serde(rename = "timeTotal")]
    pub time_total: String,
}

////////////////////////////////////////////////////////////////////////////
pub async fn get_timecards_data(
    delivery_id: Option<u32>,
    _uid: Option<String>,
) -> Result<Vec<(String, String, Option<TimeCard>)>, Box<dyn std::error::Error>> {
    let query = if let Some(delivery_id) = delivery_id {
        GET_TIMECARDS_GRAPHQL.replace(
            "***GET_TIMECARDS_PARAMS***",
            &format!("deliveryId: {}", delivery_id),
        )
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
        let timecard_map: BTreeMap<_, _> = resp
            .mulch_timecards
            .into_iter()
            .map(|v| (v.uid.clone(), v))
            .collect();
        timecard_map
    };

    let timecard_data = (*get_users())
        .clone()
        .into_iter()
        .filter(|(_, user_info)| "Bear" != user_info.group && "Bogus" != user_info.group)
        .map(|(uid, user_info)| {
            let tc = timecard_map.remove(&uid);
            (uid, user_info.name, tc)
        })
        .collect::<Vec<(String, String, Option<TimeCard>)>>();
    //timecard_data.sort_by_key(|k| k.1.clone());  //Shouldn't need this since btree is sorted
    Ok(timecard_data)
}

////////////////////////////////////////////////////////////////////////////
static SET_TIMECARDS_GRAPHQL: &str = r"
mutation {
  setMulchTimecards(timecards: [
***SET_TIMECARDS_PARAMS***
  ])
}";

////////////////////////////////////////////////////////////////////////////
pub async fn save_timecards_data(
    timecards: Vec<TimeCard>,
) -> Result<(), Box<dyn std::error::Error>> {
    if timecards.is_empty() {
        return Ok(());
    }

    let timecards_param = timecards
        .iter()
        .map(|v| {
            format!(
                "\t\t{{\n{}\n{}\n{}\n{}\n{}\n\t\t}}",
                format_args!("\t\t\tid: \"{}\",", &v.uid),
                format_args!("\t\t\tdeliveryId: {},", v.delivery_id),
                format_args!("\t\t\ttimeIn: \"{}\",", &v.time_in),
                format_args!("\t\t\ttimeOut: \"{}\",", &v.time_out),
                format_args!("\t\t\ttimeTotal: \"{}\"", &v.time_total)
            )
        })
        .collect::<Vec<String>>()
        .join(",\n");

    let query = SET_TIMECARDS_GRAPHQL.replace("***SET_TIMECARDS_PARAMS***", &timecards_param);
    log::info!("Running Query: {}", &query);

    let req = GraphQlReq::new(query);
    let _ = make_gql_request::<serde_json::Value>(&req).await?;
    Ok(())
}

////////////////////////////////////////////////////////////////////////////
static GET_FR_CLOSURE_DATA_GRAPHQL: &str = r"
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
#[derive(Default, Debug, Clone, PartialEq)]
pub struct FrClosureMapData {
    pub delivery_time_total: Duration,
    pub num_bags_sold: u64,
    pub amount_from_bags_sales: Decimal,
    pub num_bags_to_spread_sold: u64,
    pub amount_from_bags_to_spread_sales: Decimal,
    pub amount_from_donations: Decimal,
    pub amount_total_collected: Decimal,
    pub num_bags_spread: Decimal,
}

/////////////////////////////////////////////////
pub fn time_val_str_to_duration(time_val_str: &str) -> Option<Duration> {
    let mut time_val_str = time_val_str
        .split(":")
        .map(|v| v.to_string())
        .collect::<Vec<String>>();
    if time_val_str.len() == 3 {
        //If vector is server time
        time_val_str.pop();
    }

    if time_val_str.len() == 2 {
        return time_val_str[0]
            .parse::<u64>()
            .ok()
            .map(|v1| Duration::from_secs(v1 * 60 * 60))
            .map(|v1| {
                time_val_str[1]
                    .parse::<u64>()
                    .ok()
                    .map(|v2| Duration::from_secs(v2 * 60))
                    .map(|v2| v1.checked_add(v2))?
            })?;
    }
    None
}

/// Calculates the number of spread bags divided up among the people that spread.
pub fn get_calculated_bags_spread_per_user(spreaders: &[String], num_bags: usize)->Decimal {
    if num_bags == 0 {
        log::error!("num_bags must be greater than 0");
        Decimal::ZERO
    } else if spreaders.len() == 1 {
        Decimal::from(num_bags)
    } else {
        Decimal::from(num_bags)
            .checked_div(spreaders.len().into())
            .unwrap()
    }
}

pub type FrClosureStaticData = Arc<BTreeMap<String, FrClosureMapData>>;
////////////////////////////////////////////////////////////////////////////
pub async fn get_fundraiser_closure_static_data()
-> Result<FrClosureStaticData, Box<dyn std::error::Error>> {
    if let Ok(closure_data) = FR_CLOSURE_DATA.read() {
        if !closure_data.is_empty() {
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
    pub struct PurchasedItemsClosureData {
        #[serde(alias = "productId")]
        pub product_id: String,
        #[serde(alias = "numSold")]
        pub num_sold: u64,
        #[serde(alias = "amountCharged")]
        pub amount_charged: String,
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
        cd.delivery_time_total = cd.delivery_time_total.checked_add(add_dur).unwrap();
    }

    fn add_order_data(cd: &mut FrClosureMapData, new_data: &FrClosureMapData) {
        cd.amount_from_donations = cd
            .amount_from_donations
            .checked_add(new_data.amount_from_donations)
            .unwrap();
        cd.amount_total_collected = cd
            .amount_total_collected
            .checked_add(new_data.amount_total_collected)
            .unwrap();
        cd.num_bags_sold += new_data.num_bags_sold;
        cd.amount_from_bags_sales = cd
            .amount_from_bags_sales
            .checked_add(new_data.amount_from_bags_sales)
            .unwrap();
        cd.num_bags_to_spread_sold += new_data.num_bags_to_spread_sold;
        cd.amount_from_bags_to_spread_sales = cd
            .amount_from_bags_to_spread_sales
            .checked_add(new_data.amount_from_bags_to_spread_sales)
            .unwrap();
    }

    fn register_spreaders(
        closure_data: &mut BTreeMap<String, FrClosureMapData>,
        mut spreaders: Vec<String>,
        num_bags: u64,
    ) {
        //Due to bug there can be empty spreaders
        spreaders.retain(|v| !v.is_empty());

        if spreaders.is_empty() {
            return;
        }

        if num_bags == 0 {
            log::error!("We have spreaders but no bags to spread");
            return;
        }
        
        let num_bags_to_record_as_spread_per_user: Decimal = get_calculated_bags_spread_per_user(
            &spreaders,num_bags as usize );

        for uid in spreaders {
            if !closure_data.contains_key(&uid) {
                closure_data.insert(uid.clone(), FrClosureMapData::default());
            }

            let datum: &mut FrClosureMapData = closure_data.get_mut(&uid).unwrap();
            datum.num_bags_spread = datum
                .num_bags_spread
                .checked_add(num_bags_to_record_as_spread_per_user)
                .unwrap();
            let datum: &mut FrClosureMapData = closure_data.get_mut("TROOP_TOTALS").unwrap();
            datum.num_bags_spread = datum
                .num_bags_spread
                .checked_add(num_bags_to_record_as_spread_per_user)
                .unwrap();
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
        add_tc(closure_data.get_mut(&tc.uid).unwrap(), dur);
        add_tc(closure_data.get_mut("TROOP_TOTALS").unwrap(), dur);
    }

    for order in resp.orders {
        //log::info!("{} Spread order: {:#?}",&order.uid, &order);
        // convert values and total and assign to user
        if !closure_data.contains_key(&order.uid) {
            closure_data.insert(order.uid.clone(), FrClosureMapData::default());
        }

        let new_data = {
            let mut new_data = FrClosureMapData {
                amount_from_donations: order
                    .amount_from_donations
                    .map_or(Decimal::ZERO, |v| Decimal::from_str(v.as_str()).unwrap()),
                amount_total_collected: order
                    .amount_total_collected
                    .map_or(Decimal::ZERO, |v| Decimal::from_str(v.as_str()).unwrap()),
                ..Default::default()
            };

            for purchase in order.purchases {
                if "bags" == purchase.product_id.as_str() && purchase.num_sold != 0 {
                    new_data.num_bags_sold = purchase.num_sold;
                    // Issue #108 hack replace ","->""
                    new_data.amount_from_bags_sales =
                        Decimal::from_str(&purchase.amount_charged.replace(",", "")).unwrap();
                } else if "spreading" == purchase.product_id.as_str() && purchase.num_sold != 0 {
                    new_data.num_bags_to_spread_sold = purchase.num_sold;
                    new_data.amount_from_bags_to_spread_sales =
                        Decimal::from_str(&purchase.amount_charged).unwrap();
                }
            }

            new_data
        };

        add_order_data(closure_data.get_mut(&order.uid).unwrap(), &new_data);
        add_order_data(closure_data.get_mut("TROOP_TOTALS").unwrap(), &new_data);

        register_spreaders(
            &mut closure_data,
            order.spreaders,
            new_data.num_bags_to_spread_sold,
        );
    }

    if let Ok(mut arc_map) = FR_CLOSURE_DATA.write() {
        Arc::get_mut(&mut *arc_map)
            .unwrap()
            .append(&mut closure_data);
    }

    Ok(FR_CLOSURE_DATA.read().unwrap().clone())
}

////////////////////////////////////////////////////////////////////////////
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FrClosureDynamicData {
    pub bank_deposited: Option<String>,
    pub mulch_cost: Option<String>,
}

////////////////////////////////////////////////////////////////////////////
pub fn get_fundraiser_closure_dynamic_data() -> Option<FrClosureDynamicData> {
    log::info!("Getting Fundraiser Closure Data From LocalStorage");
    LocalStorage::get("FrClosureDynamicData").ok()
}

////////////////////////////////////////////////////////////////////////////
pub fn set_fundraiser_closure_dynamic_data(data: FrClosureDynamicData) {
    log::info!("Getting Fundraiser Closure Data From LocalStorage");
    let _ = LocalStorage::set("FrClosureDynamicData", data);
}

////////////////////////////////////////////////////////
#[derive(Serialize, Default, Debug, PartialEq, Clone)]
pub struct FrCloseoutDynamicVars {
    pub bank_deposited: Decimal,
    pub mulch_cost: Decimal,
    pub per_bag_cost: Decimal,
    pub profits_from_bags: Decimal,
    pub mulch_sales_gross: Decimal,
    pub money_pool_for_troop: Decimal,
    pub money_pool_for_scouts_sub_pools: Decimal,
    pub money_pool_for_scout_sales: Decimal,
    pub per_bag_avg_earnings: Decimal,
    pub money_pool_for_scout_delivery: Decimal,
    pub delivery_earnings_per_minute: Decimal,
}

impl FrCloseoutDynamicVars {
    pub fn new() -> Self {
        FrCloseoutDynamicVars::default()
    }
}

////////////////////////////////////////////////////////
#[derive(Serialize, Default, Debug, PartialEq, Clone)]
pub struct FrCloseoutAllocationVals {
    pub name: String,
    pub uid: String,
    pub bags_sold: u64,
    pub bags_spread: Decimal,
    pub delivery_minutes: Decimal,
    pub total_donations: Decimal,
    pub allocation_from_bags_sold: Decimal,
    pub allocation_from_bags_spread: Decimal,
    pub allocation_from_delivery: Decimal,
    pub allocation_total: Decimal,
}

////////////////////////////////////////////////////////////////////////////
static SET_FR_CLOSEOUT_CONFIG_DATA_GRAPHQL: &str = r#"
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
static SET_FR_CLOSEOUT_ALLOC_DATA_GRAPHQL: &str = r#"
mutation {
  setFundraiserCloseoutAllocations(
    allocations: [
        ***ALLOCATIONS***
    ]
  )
}
"#;

////////////////////////////////////////////////////////////////////////////
static SET_FR_CLOSEOUT_ALLOC_DATUM_GRAPHQL: &str = r#"
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
pub async fn set_fr_closeout_data(
    dynamic_vars: &FrCloseoutDynamicVars,
    allocation_list: &[FrCloseoutAllocationVals],
) {
    // Set Config closeout data
    let query = SET_FR_CLOSEOUT_CONFIG_DATA_GRAPHQL
        .replace(
            "bankDeposited: \"0.0000\"",
            &format!(
                "bankDeposited: \"{}\"",
                dynamic_vars.bank_deposited.round_dp(4)
            ),
        )
        .replace(
            "mulchCost: \"0.0000\"",
            &format!("mulchCost: \"{}\"", dynamic_vars.mulch_cost.round_dp(4)),
        )
        .replace(
            "perBagCost: \"0.0000\"",
            &format!("perBagCost: \"{}\"", dynamic_vars.per_bag_cost.round_dp(4)),
        )
        .replace(
            "profitsFromBags: \"0.0000\"",
            &format!(
                "profitsFromBags: \"{}\"",
                dynamic_vars.profits_from_bags.round_dp(4)
            ),
        )
        .replace(
            "mulchSalesGross: \"0.0000\"",
            &format!(
                "mulchSalesGross: \"{}\"",
                dynamic_vars.mulch_sales_gross.round_dp(4)
            ),
        )
        .replace(
            "moneyPoolForTroop: \"0.0000\"",
            &format!(
                "moneyPoolForTroop: \"{}\"",
                dynamic_vars.money_pool_for_troop.round_dp(4)
            ),
        )
        .replace(
            "moneyPoolForScoutsSubPools: \"0.0000\"",
            &format!(
                "moneyPoolForScoutsSubPools: \"{}\"",
                dynamic_vars.money_pool_for_scouts_sub_pools.round_dp(4)
            ),
        )
        .replace(
            "moneyPoolForScoutsSales: \"0.0000\"",
            &format!(
                "moneyPoolForScoutsSales: \"{}\"",
                dynamic_vars.money_pool_for_scout_sales.round_dp(4)
            ),
        )
        .replace(
            "perBagAvgEarnings: \"0.0000\"",
            &format!(
                "perBagAvgEarnings: \"{}\"",
                dynamic_vars.per_bag_avg_earnings.round_dp(4)
            ),
        )
        .replace(
            "moneyPoolForScoutsDelivery: \"0.0000\"",
            &format!(
                "moneyPoolForScoutsDelivery: \"{}\"",
                dynamic_vars.money_pool_for_scout_delivery.round_dp(4)
            ),
        )
        .replace(
            "deliveryEarningsPerMinute: \"0.0000\"",
            &format!(
                "deliveryEarningsPerMinute: \"{}\"",
                dynamic_vars.delivery_earnings_per_minute.round_dp(4)
            ),
        );

    let req = GraphQlReq::new(query);
    let _ = make_gql_request::<serde_json::Value>(&req).await.unwrap();

    let query = SET_FR_CLOSEOUT_ALLOC_DATA_GRAPHQL.replace(
        "***ALLOCATIONS***",
        allocation_list
            .iter()
            .map(|v| {
                let bags_sold_str = if 0 != v.bags_sold {
                    format!("bagsSold: {},\n", v.bags_sold)
                } else {
                    "".to_string()
                };
                let bags_spread_str = if Decimal::ZERO != v.bags_spread {
                    format!("bagsSpread: \"{}\",\n", v.bags_spread.round_dp(4))
                } else {
                    "".to_string()
                };
                let delivery_minutes_str = if Decimal::ZERO != v.delivery_minutes {
                    format!("deliveryMinutes: \"{}\",\n", v.delivery_minutes.round_dp(4))
                } else {
                    "".to_string()
                };
                let total_donations_str = if Decimal::ZERO != v.total_donations {
                    format!("totalDonations: \"{}\",\n", v.total_donations.round_dp(4))
                } else {
                    "".to_string()
                };
                let allocation_from_bags_sold_str = if Decimal::ZERO != v.allocation_from_bags_sold
                {
                    format!(
                        "allocationsFromBagsSold: \"{}\",\n",
                        v.allocation_from_bags_sold.round_dp(4)
                    )
                } else {
                    "".to_string()
                };
                let allocation_from_bags_spread_str =
                    if Decimal::ZERO != v.allocation_from_bags_spread {
                        format!(
                            "allocationsFromBagsSpread: \"{}\",\n",
                            v.allocation_from_bags_spread.round_dp(4)
                        )
                    } else {
                        "".to_string()
                    };
                let allocation_from_delivery_str = if Decimal::ZERO != v.allocation_from_delivery {
                    format!(
                        "allocationsFromDelivery: \"{}\",\n",
                        v.allocation_from_delivery.round_dp(4)
                    )
                } else {
                    "".to_string()
                };
                SET_FR_CLOSEOUT_ALLOC_DATUM_GRAPHQL
                    .replace("uid:", &format!("uid: \"{}\"", v.uid))
                    .replace(
                        "allocationsTotal:",
                        &format!("allocationsTotal: \"{}\"", v.allocation_total.round_dp(4)),
                    )
                    .replace("bagsSold:,\n", &bags_sold_str)
                    .replace("bagsSpread:,\n", &bags_spread_str)
                    .replace("deliveryMinutes:,\n", &delivery_minutes_str)
                    .replace("totalDonations:,\n", &total_donations_str)
                    .replace(
                        "allocationsFromBagsSold:,\n",
                        &allocation_from_bags_sold_str,
                    )
                    .replace(
                        "allocationsFromBagsSpread:,\n",
                        &allocation_from_bags_spread_str,
                    )
                    .replace("allocationsFromDelivery:,\n", &allocation_from_delivery_str)
            })
            .collect::<Vec<String>>()
            .join(",\n")
            .as_str(),
    );

    log::info!("Allocation Mutation:\n{}", &query);
    let req = GraphQlReq::new(query);
    let _ = make_gql_request::<serde_json::Value>(&req).await.unwrap();
}

////////////////////////////////////////////////////////////////////////////
static GET_ADDR_API_GQL: &str = r#"
{
  getAddress(***LAT_LNG_PARAMS***) {
    zipcode
    city
    houseNumber
    street
  }
}"#;

////////////////////////////////////////////////////////////////////////////
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct AddressInfo {
    #[serde(rename = "houseNumber")]
    pub house_number: Option<String>,
    pub street: Option<String>,
    pub city: Option<String>,
    pub zipcode: Option<u32>,
}

////////////////////////////////////////////////////////////////////////////
pub async fn get_address_from_lat_lng(
    lat: f64,
    lng: f64,
) -> Result<AddressInfo, Box<dyn std::error::Error>> {
    let query = GET_ADDR_API_GQL.replace(
        "***LAT_LNG_PARAMS***",
        &format!("lat: {}, lng: {}", lat, lng),
    );

    #[derive(Deserialize)]
    struct RespAddressInfo {
        #[serde(rename = "getAddress")]
        address_info: AddressInfo,
    }

    // log::info!("Get Addr Query:\n{}", &query);
    let req = GraphQlReq::new(query);
    make_gql_request::<RespAddressInfo>(&req)
        .await
        .map(|v| v.address_info)
}

////////////////////////////////////////////////////////////////////////////
static RESET_FUNDRAISER_API_GQL: &str = r#"
mutation {
  resetFundraisingData(doResetUsers: true, doResetOrders: true)
}"#;

////////////////////////////////////////////////////////////////////////////
pub async fn reset_fundraiser() -> Result<(), Box<dyn std::error::Error>> {
    let req = GraphQlReq::new(RESET_FUNDRAISER_API_GQL);
    make_gql_request::<serde_json::Value>(&req)
        .await
        .map(|_| ())?;
    load_config().await;
    Ok(())
}

////////////////////////////////////////////////////////////////////////////
static GET_USERS_FOR_CONFIG_API_GQL: &str = r#"
{
  users {
    id
    group
    firstName
    lastName
  }
}"#;
////////////////////////////////////////////////////////////////////////////
#[derive(Serialize, Deserialize, Properties, Debug, Clone, PartialEq)]
pub struct UserAdminConfig {
    pub id: String,
    #[serde(rename = "firstName")]
    pub first_name: String,
    #[serde(rename = "lastName")]
    pub last_name: String,
    pub group: String,
}

////////////////////////////////////////////////////////////////////////////
pub async fn get_users_for_admin_config()
-> Result<BTreeMap<String, UserAdminConfig>, Box<dyn std::error::Error>> {
    #[derive(Deserialize)]
    struct RespUserInfo {
        users: Vec<UserAdminConfig>,
    }

    // log::info!("Get Addr Query:\n{}", &query);
    let req = GraphQlReq::new(GET_USERS_FOR_CONFIG_API_GQL);
    make_gql_request::<RespUserInfo>(&req).await.map(|v| {
        v.users
            .into_iter()
            .map(|v| (v.id.clone(), v))
            .collect::<BTreeMap<String, UserAdminConfig>>()
    })
}

////////////////////////////////////////////////////////////////////////////
static ADD_OR_UPDATE_USERS_FOR_CONFIG_API_GQL: &str = r#"
mutation {
  addOrUpdateUsers(users: [
     ***USERS_PARAMS***
  ])
}"#;
////////////////////////////////////////////////////////////////////////////
pub async fn add_or_update_users_for_admin_config(
    users: Vec<UserAdminConfig>,
) -> Result<(), Box<dyn std::error::Error>> {
    log::info!("Adding or Updating Users: {:#?}", &users);

    let users_str = users
        .iter()
        .map(|v| {
            format!(
                "\t\t{{\n{}\n{}\n{}\n{}\n\t\t}}",
                format_args!("\t\t\tid: \"{}\"", v.id),
                format_args!("\t\t\tfirstName: \"{}\"", v.first_name),
                format_args!("\t\t\tlastName: \"{}\"", v.last_name),
                format_args!("\t\t\tgroup: \"{}\"", v.group)
            )
        })
        .collect::<Vec<String>>()
        .join(",");

    let query = ADD_OR_UPDATE_USERS_FOR_CONFIG_API_GQL.replace("***USERS_PARAMS***", &users_str);

    let req = GraphQlReq::new(query);
    make_gql_request::<serde_json::Value>(&req)
        .await
        .map(|_| ())?;

    // I don't know if there is any better way. Making DB Query costs money
    // Trying to merge in place would also take multiple passes through the neighborhood list
    // so converting it into a map and then replacing list with values
    let mut new_map: UserMapType = users
        .into_iter()
        .map(|v| {
            let ui = UserInfo {
                name: format!("{} {}", v.first_name, v.last_name),
                group: v.group,
            };
            (v.id, ui)
        })
        .collect();
    if let Ok(mut arc_user_map) = USER_MAP.write() {
        Arc::get_mut(&mut *arc_user_map)
            .unwrap()
            .append(&mut new_map);
    }

    Ok(())
}
