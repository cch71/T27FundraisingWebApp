use std::collections::HashMap;
use super::{
    gql_utils::{GraphQlReq, make_gql_request},
    {get_active_user, get_fr_config, get_neighborhood},
};
use chrono::prelude::*;
use gloo::storage::{LocalStorage, SessionStorage, Storage};
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

// Exposing this const out to keep consistent tag name
pub static ALL_USERS_TAG: &str = "doShowAllUsers";
// URL to api that returns GeoJSON locations
static GEOJSONURL: LazyLock<String> =
    LazyLock::new(|| crate::CLOUD_API_URL.to_owned() + "/salelocs");

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub enum ReportViews {
    // Reports available to sellers
    Quick,
    Full,
    SpreadingJobs,
    SpreadingAssistJobs,
    AllocationSummary,
    SellMap,
    MoneyCollection,

    // Admin Only Reports
    UnfinishedSpreadingJobs,
    OrderVerification,
    DistributionPoints,
    Deliveries,
}

impl std::fmt::Display for ReportViews {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            ReportViews::Quick => write!(f, "Default"),
            ReportViews::Full => write!(f, "Full"),
            ReportViews::SpreadingJobs => write!(f, "Spreading Jobs"),
            ReportViews::UnfinishedSpreadingJobs => write!(f, "Unfinished Spreading Jobs"),
            ReportViews::SpreadingAssistJobs => write!(f, "Assisted Spreading Jobs"),
            ReportViews::OrderVerification => write!(f, "Order Verification"),
            ReportViews::DistributionPoints => write!(f, "Distribution Point"),
            ReportViews::Deliveries => write!(f, "Deliveries"),
            ReportViews::SellMap => write!(f, "Sales Map"),
            ReportViews::AllocationSummary => write!(f, "Allocation Summary"),
            ReportViews::MoneyCollection => write!(f, "Money Collection"),
        }
    }
}

impl std::str::FromStr for ReportViews {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Default" => Ok(ReportViews::Quick),
            "Full" => Ok(ReportViews::Full),
            "Spreading Jobs" => Ok(ReportViews::SpreadingJobs),
            "Unfinished Spreading Jobs" => Ok(ReportViews::UnfinishedSpreadingJobs),
            "Assisted Spreading Jobs" => Ok(ReportViews::SpreadingAssistJobs),
            "Order Verification" => Ok(ReportViews::OrderVerification),
            "Distribution Point" => Ok(ReportViews::DistributionPoints),
            "Deliveries" => Ok(ReportViews::Deliveries),
            "Sales Map" => Ok(ReportViews::SellMap),
            "Allocation Summary" => Ok(ReportViews::AllocationSummary),
            "Money Collection" => Ok(ReportViews::MoneyCollection),
            _ => Err(format!("'{}' is not a valid value for ReportViews", s)),
        }
    }
}

pub fn get_allowed_report_views() -> Vec<ReportViews> {
    let mut reports = vec![
        ReportViews::Quick,
        ReportViews::Full,
        ReportViews::SellMap,
        ReportViews::MoneyCollection,
    ];

    if get_fr_config().kind == "mulch" {
        reports.push(ReportViews::SpreadingJobs);
        reports.push(ReportViews::SpreadingAssistJobs);

        if get_active_user().is_admin() {
            reports.push(ReportViews::UnfinishedSpreadingJobs);
            reports.push(ReportViews::OrderVerification);
            reports.push(ReportViews::DistributionPoints);
            reports.push(ReportViews::Deliveries);
        }
    }

    // if allocation_summary availalble add allocation summary {
    //      reports.push(ReportViews::AllocationSummary);
    // }

    reports
}

/// Parses the purchases and creates a map
pub fn get_purchase_to_map(v: &serde_json::Value)->HashMap<String, u64> {
    let mut purchases = HashMap::new();

    for purchase in v["purchases"].as_array().unwrap_or(&Vec::new()) {
        let product_id = purchase["productId"].as_str();
        match  product_id {
          Some("spreading") => {
              purchases.insert("spreading".to_string(), purchase["numSold"].as_u64().unwrap_or_default());
          }, 
            Some("bags") => {
                purchases.insert("bags".to_string(), purchase["numSold"].as_u64().unwrap_or_default());
            },
            _ => log::error!("Unknown product id: {:?}", product_id),
        };
    }
    purchases
}

/////////////////////////////////////////////////////////////////////////////////
/// This will determine if the switch user option is available in the report
/// settings dialog
pub fn do_show_current_seller(current_view: &ReportViews) -> bool {
    matches!(
        *current_view,
        ReportViews::Quick
            | ReportViews::Full
            | ReportViews::SpreadingJobs
            | ReportViews::SpreadingAssistJobs
            | ReportViews::MoneyCollection
            | ReportViews::OrderVerification
    )
}

/////////////////////////////////////////////////////////////////////////////////
async fn make_report_query(
    query: String,
) -> Result<Vec<serde_json::Value>, Box<dyn std::error::Error>> {
    #[derive(Serialize, Deserialize, Debug)]
    struct GqlResp {
        #[serde(alias = "mulchOrders")]
        mulch_orders: Vec<serde_json::Value>,
    }

    let req = GraphQlReq::new(query);
    let rslts = make_gql_request::<GqlResp>(&req).await?;
    Ok(rslts.mulch_orders)
}

/////////////////////////////////////////////////////////////////////////////////
pub enum ReportViewState {
    IsLoading,
    ReportHtmlGenerated(Vec<serde_json::Value>),
}

/////////////////////////////////////////////////////////////////////////////////
#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct ReportViewSettings {
    pub current_view: ReportViews,
    pub seller_id_filter: String,
}

/////////////////////////////////////////////////////////////////////////////////
pub fn save_report_settings(
    settings: &ReportViewSettings,
) -> Result<(), Box<dyn std::error::Error>> {
    SessionStorage::set("ReportViewSettings", settings)?;
    Ok(())
}

/////////////////////////////////////////////////////////////////////////////////
pub fn delete_report_settings() {
    SessionStorage::delete("ReportViewSettings");
}

/////////////////////////////////////////////////////////////////////////////////
pub fn load_report_settings() -> ReportViewSettings {
    SessionStorage::get("ReportViewSettings").unwrap_or(ReportViewSettings {
        current_view: ReportViews::Quick,
        seller_id_filter: get_active_user().get_id(),
    })
}

//////////////////////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////////
pub async fn get_sales_geojson()
-> Result<Vec<serde_json::Value>, Box<dyn std::error::Error>> {
    use gloo::net::http::Request;
    // log::info!("Running Query: {}", &query);

    let mut raw_resp: serde_json::Value = Request::get(&GEOJSONURL)
        .header("Content-Type", "application/json")
        .header(
            "Authorization",
            &format!("Bearer {}", &get_active_user().token),
        )
        .send()
        .await?
        .json()
        .await?;
    let host_str = gloo::utils::window()
        .location()
        .host()
        .unwrap_or("".to_string());
    // log::info!("Hostname: {host_str}");
    if host_str.starts_with("localhost") {
        log::info!(
            "GeoJSON Resp: {}",
            serde_json::to_string_pretty(&raw_resp).unwrap()
        );
    }

    if !raw_resp["message"].is_null() {
        let err_str =
            serde_json::to_string(&raw_resp).unwrap_or("Failed to stringify json resp".to_string());
        use std::io::{Error, ErrorKind};
        return Err(Box::new(Error::new(
            ErrorKind::Other,
            format!("GeoJSON request returned raw error:\n {}", err_str).as_str(),
        )));
    }

    // make_report_query(query).await
    Ok(raw_resp["features"]
        .take()
        .as_array_mut()
        .unwrap()
        .drain(..)
        .collect())
}

//////////////////////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////////
static QUICK_RPT_GRAPHQL: &str = r"
{
  mulchOrders(***ORDER_OWNER_PARAM***) {
    orderId
    ownerId
    deliveryId
    spreaders
    isVerified
    customer {
        name
    }
    purchases {
        productId
        numSold
        amountCharged
    }
  }
}
";

/////////////////////////////////////////////////////////////////////////////////
pub async fn get_quick_report_data(
    order_owner_id: Option<&String>,
) -> Result<Vec<serde_json::Value>, Box<dyn std::error::Error>> {
    let query = if let Some(order_owner_id) = order_owner_id {
        QUICK_RPT_GRAPHQL.replace(
            "***ORDER_OWNER_PARAM***",
            &format!("ownerId: \"{}\"", order_owner_id),
        )
    } else {
        QUICK_RPT_GRAPHQL.replace("(***ORDER_OWNER_PARAM***)", "")
    };
    log::info!("Running Query: {}", &query);
    make_report_query(query).await
}

//////////////////////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////////
static FULL_RPT_GRAPHQL: &str = r"
{
  mulchOrders(***ORDER_OWNER_PARAM***) {
    orderId
    ownerId
    amountFromDonations
    amountFromCashCollected
    amountFromChecksCollected
    checkNumbers
    amountTotalCollected
    isVerified
    customer {
        name
        addr1
        addr2
        phone
        email
        city
        zipcode
        neighborhood
    }
    specialInstructions
    purchases {
        productId
        numSold
        amountCharged
    }
    deliveryId
    spreaders
  }
}
";

pub async fn get_full_report_data(
    order_owner_id: Option<&String>,
) -> Result<Vec<serde_json::Value>, Box<dyn std::error::Error>> {
    let query = if let Some(order_owner_id) = order_owner_id {
        FULL_RPT_GRAPHQL.replace(
            "***ORDER_OWNER_PARAM***",
            &format!("ownerId: \"{}\"", order_owner_id),
        )
    } else {
        FULL_RPT_GRAPHQL.replace("(***ORDER_OWNER_PARAM***)", "")
    };

    make_report_query(query).await
}

//////////////////////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////////
static MONEY_COLLECTION_RPT_GRAPHQL: &str = r"
{
  mulchOrders(***ORDER_OWNER_PARAM***) {
    ownerId
    deliveryId
    amountTotalFromCashCollected
    amountTotalFromChecksCollected
    amountTotalCollected
  }
}
";

/////////////////////////////////////////////////////////////////////////////////
pub async fn get_money_collection_report_data(
    order_owner_id: Option<&String>,
) -> Result<Vec<serde_json::Value>, Box<dyn std::error::Error>> {
    let query = if let Some(order_owner_id) = order_owner_id {
        MONEY_COLLECTION_RPT_GRAPHQL.replace(
            "***ORDER_OWNER_PARAM***",
            &format!("ownerId: \"{}\"", order_owner_id),
        )
    } else {
        MONEY_COLLECTION_RPT_GRAPHQL.replace("(***ORDER_OWNER_PARAM***)", "")
    };
    log::info!("Running Query: {}", &query);
    make_report_query(query).await
}

//////////////////////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////////
static DISTRIBUTION_POINTS_RPT_GRAPHQL: &str = r#"
{
  mulchOrders {
    customer {
        neighborhood
    }
    purchases {
        productId
        numSold
    }
    deliveryId
  }
}
"#;

/////////////////////////////////////////////////////////////////////////////////
pub async fn get_distribution_points_report_data()
-> Result<Vec<serde_json::Value>, Box<dyn std::error::Error>> {
    use std::collections::{BTreeMap, BTreeSet};
    let mut delivery_id_map: BTreeMap<u64, BTreeMap<String, u64>> = BTreeMap::new();
    make_report_query(DISTRIBUTION_POINTS_RPT_GRAPHQL.to_string())
        .await
        .map(|orders| {
            orders
                .into_iter()
                .filter(|v| v["deliveryId"].as_u64().is_some())
                .for_each(|v| {
                    let delivery_id = v["deliveryId"].as_u64().unwrap();
                    delivery_id_map.entry(delivery_id).or_default();
                    let dist_point_map = delivery_id_map.get_mut(&delivery_id).unwrap();
                    let neighborhood = v["customer"]["neighborhood"].as_str().unwrap();
                    let dist_point = get_neighborhood(neighborhood)
                        .map_or("".to_string(), |v| v.distribution_point.clone());
                    let purchases = get_purchase_to_map(&v);
                    let num_bags_sold = *purchases.get("bags").unwrap_or(&0);
                    match dist_point_map.get_mut(&dist_point) {
                        Some(num_bags_for_point) => {
                            *num_bags_for_point += num_bags_sold;
                        }
                        None => {
                            dist_point_map.insert(dist_point.to_string(), num_bags_sold);
                        }
                    };
                    match dist_point_map.get_mut("TotalBagSummary") {
                        Some(num_bags_for_point) => {
                            *num_bags_for_point += num_bags_sold;
                        }
                        None => {
                            dist_point_map.insert("TotalBagSummary".to_string(), num_bags_sold);
                        }
                    };
                });
        })?;

    let mut dist_points_set = BTreeSet::new();
    delivery_id_map.values().for_each(|v| {
        v.keys().filter(|v| *v != "TotalBagSummary").for_each(|v| {
            dist_points_set.insert(v.to_string());
        })
    });

    // This is really not very efficient to convert to vec serde vals just to avoid
    //  adding another enum but this is least impact for a report that is rarely ran
    //  so right now this should be acceptable
    Ok(vec![serde_json::json!({
        "deliveryIdMap": delivery_id_map,
        "distPoints": dist_points_set,
    })])
}

//////////////////////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////////
static DELIVERIES_RPT_GRAPHQL: &str = r#"
{
  mulchOrders {
    orderId
    ownerId
    customer {
        name
        addr1
        addr2
        city
        zipcode
        phone
        neighborhood
    }
    specialInstructions
    purchases {
        productId
        numSold
    }
    deliveryId
  }
}
"#;

/////////////////////////////////////////////////////////////////////////////////
pub async fn get_deliveries_report_data()
-> Result<Vec<serde_json::Value>, Box<dyn std::error::Error>> {
    make_report_query(DELIVERIES_RPT_GRAPHQL.to_string())
        .await
        .map(|orders| {
            orders
                .into_iter()
                .filter(|v| v["deliveryId"].as_u64().is_some())
                .collect::<Vec<_>>()
        })
}

//////////////////////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////////
static SPREADING_JOBS_RPT_GRAPHQL: &str = r"
{
  mulchOrders(doGetSpreadOrdersOnly: true***ORDER_OWNER_PARAM***) {
    orderId
    ownerId
    isVerified
    customer {
        name
        phone
        addr1
        addr2
        neighborhood
    }
    specialInstructions
    purchases {
        productId
        numSold
        amountCharged
    }
    deliveryId
    spreaders
  }
}
";

/////////////////////////////////////////////////////////////////////////////////
pub async fn get_spreading_jobs_report_data(
    order_owner_id: Option<&String>,
) -> Result<Vec<serde_json::Value>, Box<dyn std::error::Error>> {
    let query = if let Some(order_owner_id) = order_owner_id {
        SPREADING_JOBS_RPT_GRAPHQL.replace(
            "***ORDER_OWNER_PARAM***",
            &format!(", ownerId: \"{}\"", order_owner_id),
        )
    } else {
        SPREADING_JOBS_RPT_GRAPHQL.replace("***ORDER_OWNER_PARAM***", "")
    };

    make_report_query(query).await
}


//////////////////////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////////
static SPREADING_ASSIST_JOBS_RPT_GRAPHQL: &str = r"
{
  mulchOrders(doGetSpreadOrdersOnly: true***EXTRA_PARAMS***) {
    orderId
    ownerId
    customer {
        name
        addr1
        addr2
        neighborhood
    }
    purchases {
        productId
        numSold
    }
    spreaders
  }
}
";

/////////////////////////////////////////////////////////////////////////////////
pub async fn get_spreading_assist_jobs_report_data(
    order_owner_id: Option<&String>,
) -> Result<Vec<serde_json::Value>, Box<dyn std::error::Error>> {
    let query = if let Some(order_owner_id) = order_owner_id {
        SPREADING_ASSIST_JOBS_RPT_GRAPHQL.replace(
            "***EXTRA_PARAMS***",
            &format!(", excludeOwnerId: \"{0}\", spreaderId: \"{0}\"", order_owner_id),
        )
    } else {
        return Ok(Vec::new());
    };

    make_report_query(query).await
}


//////////////////////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////////
static UNFINISHED_SPREADING_JOBS_RPT_GRAPHQL: &str = r"
{
  mulchOrders(doGetSpreadOrdersOnly: true) {
    ownerId
    deliveryId
    purchases {
        productId
        numSold
    }
    spreaders
  }
}
";

/////////////////////////////////////////////////////////////////////////////////
pub async fn get_unfinished_spreading_jobs_report_data()
-> Result<Vec<serde_json::Value>, Box<dyn std::error::Error>> {
    use std::collections::BTreeMap;
    let mut unfinished_job_map: BTreeMap<(String, u64), u64> = BTreeMap::new();
    make_report_query(UNFINISHED_SPREADING_JOBS_RPT_GRAPHQL.to_string())
        .await
        .map(|orders| {
            orders.into_iter().for_each(|v| {
                let num_spreaders = v["spreaders"].as_array().unwrap_or(&Vec::new()).len();
                if num_spreaders != 0 {
                    // There is an assumption that if spreaders have been set then all bags have been spread
                    return;
                }
                let purchases = get_purchase_to_map(&v);
                let num_spreading_bags_sold = *purchases.get("spreading").unwrap_or(&0);
                if num_spreading_bags_sold == 0 {
                    log::warn!("Query qualifier should have meant this doesn't happen!");
                    return;
                }
                let uid = v["ownerId"].as_str().unwrap().to_string();
                let delivery_id = v["deliveryId"].as_u64().unwrap();
                let key = (uid, delivery_id);
                match unfinished_job_map.get_mut(&key) {
                    Some(uid_unfinished_jobs) => *uid_unfinished_jobs += num_spreading_bags_sold,
                    None => {
                        unfinished_job_map.insert(key, num_spreading_bags_sold);
                    }
                };
            });
        })?;
    Ok(unfinished_job_map
        .into_iter()
        .map(
            |((uid, did), v)| serde_json::json!({"ownerId": uid, "deliveryId": did, "bagsLeft": v}),
        )
        .collect::<Vec<serde_json::Value>>())
}

//////////////////////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////////
#[derive(Serialize, Deserialize, Debug)]
struct SummaryReportStorage {
    summary_report: SummaryReport,
    seller_id: String,
    num_top_sellers: u8,
    timestamp: i64,
}

/////////////////////////////////////////////////////////////////////////////////
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SummaryReport {
    #[serde(alias = "troop")]
    pub troop_summary: TroopSummary,
    #[serde(alias = "orderOwner")]
    pub seller_summary: SellerSummary,
}

/////////////////////////////////////////////////////////////////////////////////
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct TroopSummary {
    #[serde(alias = "totalAmountCollected")]
    pub amount_total_collected: String,

    #[serde(alias = "topSellers")]
    pub top_sellers: Vec<TopSeller>,

    #[serde(alias = "groupSummary")]
    pub group_summary: Vec<GroupSummary>,
}

/////////////////////////////////////////////////////////////////////////////////
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct TopSeller {
    #[serde(alias = "name")]
    pub name: String,

    #[serde(alias = "totalAmountCollected")]
    pub amount_total_collected: String,
}

/////////////////////////////////////////////////////////////////////////////////
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct GroupSummary {
    #[serde(alias = "groupId")]
    pub group_id: String,

    #[serde(alias = "totalAmountCollected")]
    pub amount_total_collected: String,
}

/////////////////////////////////////////////////////////////////////////////////
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SellerSummary {
    #[serde(alias = "totalDeliveryMinutes")]
    pub total_delivery_minutes: u32,

    #[serde(alias = "totalAssistedSpreadingBags")]
    pub total_assisted_spreading_bags: String,

    #[serde(alias = "totalNumBagsSold")]
    pub total_num_bags_sold: u32,

    #[serde(alias = "totalNumBagsSoldToSpread")]
    pub total_num_bags_to_spread_sold: u32,

    #[serde(alias = "totalAmountCollectedForDonations")]
    pub amount_total_collected_for_donations: String,

    #[serde(alias = "totalAmountCollectedForBags")]
    pub amount_total_collected_for_bags: String,

    #[serde(alias = "totalAmountCollectedForBagsToSpread")]
    pub amount_total_collected_for_bags_to_spread: String,

    #[serde(alias = "totalAmountCollected")]
    pub amount_total_collected: String,

    #[serde(alias = "allocationsFromDelivery")]
    pub allocations_from_deliveries: String,

    #[serde(alias = "allocationsFromBagsSold")]
    pub allocations_from_bags_sold: String,

    #[serde(alias = "allocationsFromBagsSpread")]
    pub allocations_from_bags_spread: String,

    #[serde(alias = "allocationsTotal")]
    pub allocations_total: String,
}

/////////////////////////////////////////////////////////////////////////////////
static SUMMARY_RPT_GRAPHQL: &str = r"
{
  summary {
    orderOwner(***ORDER_OWNER_PARAM***) {
      totalDeliveryMinutes
      totalAssistedSpreadingBags
      totalNumBagsSold
      totalNumBagsSoldToSpread
      totalAmountCollectedForDonations
      totalAmountCollectedForBags
      totalAmountCollectedForBagsToSpread
      totalAmountCollected
      allocationsFromDelivery
      allocationsFromBagsSold
      allocationsFromBagsSpread
      allocationsTotal
    }
    troop(***TOP_SELLERS_PARAM***) {
      totalAmountCollected
      topSellers {
        totalAmountCollected
        name
      }
      groupSummary {
        groupId
        totalAmountCollected
      }
    }
  }
}
";

/////////////////////////////////////////////////////////////////////////////////
pub async fn get_summary_report_data(
    seller_id: &str,
    top_sellers: u8,
) -> Result<SummaryReport, Box<dyn std::error::Error>> {
    let rslt = LocalStorage::get("SummaryData");
    if rslt.is_ok() {
        let data: SummaryReportStorage = rslt.unwrap();
        let now_ts = Utc::now().timestamp() - 86400;
        if now_ts <= data.timestamp
            && seller_id == data.seller_id
            && top_sellers == data.num_top_sellers
        {
            log::info!("Summary data retrieved from cache");
            return Ok(data.summary_report);
        }
    }

    let query = SUMMARY_RPT_GRAPHQL
        .replace(
            "***ORDER_OWNER_PARAM***",
            &format!("ownerId: \"{}\"", seller_id),
        )
        .replace(
            "***TOP_SELLERS_PARAM***",
            &format!("numTopSellers: {}", top_sellers),
        );

    #[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
    struct SummaryReportRslt {
        #[serde(alias = "summary")]
        summary: SummaryReport,
    }

    let req = GraphQlReq::new(query);
    let rslt = make_gql_request::<SummaryReportRslt>(&req).await?;

    LocalStorage::set(
        "SummaryData",
        SummaryReportStorage {
            summary_report: rslt.summary.clone(),
            seller_id: seller_id.to_string(),
            num_top_sellers: top_sellers,
            timestamp: Utc::now().timestamp(),
        },
    )?;
    Ok(rslt.summary)
}

//////////////////////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////////
static ORDER_VERIFICATION_GRAPHQL: &str = r"
{
  mulchOrders(***ORDER_OWNER_PARAM***) {
    orderId
    ownerId
    amountFromDonations
    amountFromCashCollected
    amountFromChecksCollected
    checkNumbers
    amountTotalCollected
    isVerified
    customer {
        name
    }
    deliveryId
  }
}
";

/////////////////////////////////////////////////////////////////////////////////
pub async fn get_order_verification_report_data(
    order_owner_id: Option<&String>,
) -> Result<Vec<serde_json::Value>, Box<dyn std::error::Error>> {
    let query = if let Some(order_owner_id) = order_owner_id {
        ORDER_VERIFICATION_GRAPHQL.replace(
            "***ORDER_OWNER_PARAM***",
            &format!("ownerId: \"{}\"", order_owner_id),
        )
    } else {
        ORDER_VERIFICATION_GRAPHQL.replace("(***ORDER_OWNER_PARAM***)", "")
    };
    log::info!("Running Query: {}", &query);
    make_report_query(query).await
}
