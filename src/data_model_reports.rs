
use serde::{Deserialize, Serialize};
use chrono::prelude::*;
use gloo::storage::{LocalStorage, SessionStorage, Storage};
use crate::gql_utils::{make_gql_request, GraphQlReq};
use crate::auth_utils::{get_active_user};
use crate::data_model::{get_fr_config};

pub(crate) static ALL_USERS_TAG: &'static str = "doShowAllUsers";

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub(crate) enum ReportViews {
    // Reports available to sellers
    Quick,
    Full,
    SpreadingJobs,
    AllocationSummary,
    SellMap,

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
           ReportViews::OrderVerification => write!(f, "Order Verfication"),
           ReportViews::DistributionPoints => write!(f, "Distribution Point"),
           ReportViews::Deliveries => write!(f, "Deliveries"),
           ReportViews::SellMap => write!(f, "Sales Map"),
           ReportViews::AllocationSummary => write!(f, "Allocation Summary"),
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
            "Order Verfication" => Ok(ReportViews::OrderVerification),
            "Distribution Point" => Ok(ReportViews::DistributionPoints),
            "Deliveries" => Ok(ReportViews::Deliveries),
            "Sales Map" => Ok(ReportViews::SellMap),
            "Allocation Summary" => Ok(ReportViews::AllocationSummary),
            _ => Err(format!("'{}' is not a valid value for ReportViews", s)),
        }
    }
}

pub(crate) fn get_allowed_report_views() -> Vec<ReportViews> {
    let mut reports = vec![
        ReportViews::Quick,
        ReportViews::Full,
        ReportViews::SellMap,
    ];

    if get_fr_config().kind == "mulch" {
        reports.push(ReportViews::SpreadingJobs);

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

pub(crate) fn do_show_current_seller(current_view: &ReportViews) -> bool {
    match *current_view {
        ReportViews::Quick=>true,
        ReportViews::Full=>true,
        ReportViews::SpreadingJobs=>true,
        _=>false,
    }
}

async fn make_report_query(query: String)
    -> std::result::Result<Vec<serde_json::Value> ,Box<dyn std::error::Error>>
{
    #[derive(Serialize, Deserialize, Debug)]
    struct GqlResp {
        #[serde(alias = "mulchOrders")]
        mulch_orders: Vec<serde_json::Value>,
    }

    let req = GraphQlReq::new(query);
    let rslts = make_gql_request::<GqlResp>(&req).await?;
    Ok(rslts.mulch_orders)
}

static QUICK_RPT_GRAPHQL: &'static str = r"
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

pub(crate) async fn get_quick_report_data(order_owner_id: Option<&String>)
    -> std::result::Result<Vec<serde_json::Value> ,Box<dyn std::error::Error>>
{

    let query = if let Some(order_owner_id) = order_owner_id {
        QUICK_RPT_GRAPHQL.replace("***ORDER_OWNER_PARAM***", &format!("ownerId: \"{}\"", order_owner_id))
    } else {
        QUICK_RPT_GRAPHQL.replace("(***ORDER_OWNER_PARAM***)", "")
    };
    log::info!("Running Query: {}", &query);
    make_report_query(query).await
}

static FULL_RPT_GRAPHQL: &'static str = r"
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

pub(crate) async fn get_full_report_data(order_owner_id: Option<&String>)
    -> std::result::Result<Vec<serde_json::Value> ,Box<dyn std::error::Error>>
{
    let query = if let Some(order_owner_id) = order_owner_id {
        FULL_RPT_GRAPHQL.replace("***ORDER_OWNER_PARAM***", &format!("ownerId: \"{}\"", order_owner_id))
    } else {
        FULL_RPT_GRAPHQL.replace("(***ORDER_OWNER_PARAM***)", "")
    };

    make_report_query(query).await
}

static DISTRIBUTION_POINTS_RPT_GRAPHQL: &'static str = r#"
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

pub(crate) async fn get_distribution_points_report_data()
    -> std::result::Result<Vec<serde_json::Value> ,Box<dyn std::error::Error>>
{
    use std::collections::{BTreeMap, BTreeSet};
    let mut delivery_id_map: BTreeMap<u64,BTreeMap<String, u64>> = BTreeMap::new();
    make_report_query(DISTRIBUTION_POINTS_RPT_GRAPHQL.to_string()).await
        .and_then(|orders| {
            orders.into_iter()
                .filter(|v|v["deliveryId"].as_u64().is_some())
                .for_each(|v|{
                    let delivery_id = v["deliveryId"].as_u64().unwrap();
                    if !delivery_id_map.contains_key(&delivery_id) {
                        delivery_id_map.insert(delivery_id, BTreeMap::new());
                    }
                    let dist_point_map = delivery_id_map.get_mut(&delivery_id).unwrap();
                    let neighborhood = v["customer"]["neighborhood"].as_str().unwrap();
                    let dist_point = crate::get_neighborhood(neighborhood)
                        .map_or("".to_string(), |v|v.distribution_point.clone());
                    let num_bags_sold: u64 = v["purchases"].as_array().unwrap_or(&Vec::new())
                        .iter()
                        .find(|&v| v["productId"].as_str().unwrap() == "bags")
                        .map_or(0, |v| v["numSold"].as_u64().unwrap());
                    match dist_point_map.get_mut(&dist_point) {
                        Some(num_bags_for_point)=>{
                            *num_bags_for_point += num_bags_sold;
                        },
                        None=>{
                            dist_point_map.insert(dist_point.to_string(), num_bags_sold);
                        },
                    };
                    match dist_point_map.get_mut("TotalBagSummary") {
                        Some(num_bags_for_point)=>{
                            *num_bags_for_point += num_bags_sold;
                        },
                        None=>{
                            dist_point_map.insert("TotalBagSummary".to_string(), num_bags_sold);
                        },
                    };
                });
            Ok(())
        })?;

    let mut dist_points_set = BTreeSet::new();
    delivery_id_map.values().for_each(|v| {
        v.keys()
            .filter(|v| *v!="TotalBagSummary" )
            .for_each(|v| {dist_points_set.insert(v.to_string());})
    });

    // This is really not very efficient to convert to vec serde vals just to avoid
    //  adding another enum but this is least impact for a report that is rarely ran
    //  so right now this should be acceptable
    Ok(vec![serde_json::json!({
        "deliveryIdMap": delivery_id_map,
        "distPoints": dist_points_set,
    })])
}

static DELIVERIES_RPT_GRAPHQL: &'static str = r#"
{
  mulchOrders {
    orderId
    ownerId
    customer {
        name
        addr1
        addr2
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

pub(crate) async fn get_deliveries_report_data()
    -> std::result::Result<Vec<serde_json::Value> ,Box<dyn std::error::Error>>
{
    make_report_query(DELIVERIES_RPT_GRAPHQL.to_string()).await
        .and_then(|orders|Ok(orders.into_iter()
            .filter(|v|v["deliveryId"].as_u64().is_some()).collect::<Vec<_>>()))
}

static SPREADING_JOBS_RPT_GRAPHQL: &'static str = r"
{
  mulchOrders(***ORDER_OWNER_PARAM***) {
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

pub(crate) async fn get_spreading_jobs_report_data(order_owner_id: Option<&String>)
    -> std::result::Result<Vec<serde_json::Value> ,Box<dyn std::error::Error>>
{
    let query = if let Some(order_owner_id) = order_owner_id {
        SPREADING_JOBS_RPT_GRAPHQL.replace("***ORDER_OWNER_PARAM***", &format!("ownerId: \"{}\"", order_owner_id))
    } else {
        SPREADING_JOBS_RPT_GRAPHQL.replace("(***ORDER_OWNER_PARAM***)", "")
    };

    make_report_query(query).await
}

static UNFINISHED_SPREADING_JOBS_RPT_GRAPHQL: &'static str = r"
{
  mulchOrders {
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

pub(crate) async fn get_unfinished_spreading_jobs_report_data()
    -> std::result::Result<Vec<serde_json::Value> ,Box<dyn std::error::Error>>
{
    use std::collections::BTreeMap;
    let mut unfinished_job_map: BTreeMap<String, u64> = BTreeMap::new();
    make_report_query(UNFINISHED_SPREADING_JOBS_RPT_GRAPHQL.to_string()).await
        .and_then(|orders| {
            orders.into_iter()
                .for_each(|v|{
                    let num_spreaders = v["spreaders"].as_array().unwrap_or(&Vec::new()).len();
                    if num_spreaders != 0 { return; }
                    let num_spreading_bags_sold: u64 = v["purchases"].as_array().unwrap_or(&Vec::new())
                        .iter()
                        .find(|&v| v["productId"].as_str().unwrap() == "spreading")
                        .map_or(0, |v| v["numSold"].as_u64().unwrap());
                    if num_spreading_bags_sold == 0 { return; }
                    let uid = v["ownerId"].as_str().unwrap();
                    if uid == "alatham" {
                        log::info!("{}: NumSpreaders: {}   Info: {}", uid, num_spreaders, serde_json::to_string(&v).unwrap());
                    }
                    match unfinished_job_map.get_mut(uid) {
                        Some(uid_unfinished_jobs) => *uid_unfinished_jobs += num_spreading_bags_sold,
                        None=>{ unfinished_job_map.insert(uid.to_string(), num_spreading_bags_sold); },
                    };
                });
            Ok(())
        })?;
    Ok(unfinished_job_map
        .into_iter()
        .map(|(k,v)|serde_json::json!({"ownerId": k, "bagsLeft": v}))
        .collect::<Vec<serde_json::Value>>())
}

#[derive(Serialize, Deserialize, Debug)]
struct SummaryReportStorage {
    summary_report: SummaryReport,
    seller_id: String,
    num_top_sellers: u32,
    timestamp: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub(crate) struct SummaryReport {
    #[serde(alias = "troop")]
    pub(crate) troop_summary: TroopSummary,
    #[serde(alias = "orderOwner")]
    pub(crate) seller_summary: SellerSummary,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub(crate) struct TroopSummary {
    #[serde(alias = "totalAmountCollected")]
    pub(crate) amount_total_collected: String,

    #[serde(alias = "topSellers")]
    pub(crate) top_sellers: Vec<TopSeller>,

    #[serde(alias = "groupSummary")]
    pub(crate) group_summary: Vec<GroupSummary>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub(crate) struct TopSeller {
    #[serde(alias = "name")]
    pub(crate) name: String,

    #[serde(alias = "totalAmountCollected")]
    pub(crate) amount_total_collected: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub(crate) struct GroupSummary {
    #[serde(alias = "groupId")]
    pub(crate) group_id: String,

    #[serde(alias = "totalAmountCollected")]
    pub(crate) amount_total_collected: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub(crate) struct SellerSummary {
    #[serde(alias = "totalDeliveryMinutes")]
    pub(crate) total_delivery_minutes: u32,

    #[serde(alias = "totalNumBagsSold")]
    pub(crate) total_num_bags_sold: u32,

    #[serde(alias = "totalNumBagsSoldToSpread")]
    pub(crate) total_num_bags_to_spread_sold: u32,

    #[serde(alias = "totalAmountCollectedForDonations")]
    pub(crate) amount_total_collected_for_donations: String,

    #[serde(alias = "totalAmountCollectedForBags")]
    pub(crate) amount_total_collected_for_bags: String,

    #[serde(alias = "totalAmountCollectedForBagsToSpread")]
    pub(crate) amount_total_collected_for_bags_to_spread: String,

    #[serde(alias = "totalAmountCollected")]
    pub(crate) amount_total_collected: String,

    #[serde(alias = "allocationsFromDelivery")]
    pub(crate) allocations_from_deliveries: String,

    #[serde(alias = "allocationsFromBagsSold")]
    pub(crate) allocations_from_bags_sold: String,

    #[serde(alias = "allocationsFromBagsSpread")]
    pub(crate) allocations_from_bags_spread: String,

    #[serde(alias = "allocationsTotal")]
    pub(crate) allocations_total: String,
}

static SUMMARY_RPT_GRAPHQL: &'static str = r"
{
  summary {
    orderOwner(***ORDER_OWNER_PARAM***) {
      totalDeliveryMinutes
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

pub(crate) async fn get_summary_report_data(seller_id: &str, num_top_sellers: u32)
    -> std::result::Result<SummaryReport, Box<dyn std::error::Error>>
{
    let rslt = LocalStorage::get("SummaryData");
    if rslt.is_ok() {
        let data: SummaryReportStorage = rslt.unwrap();
        let now_ts = Utc::now().timestamp() - 86400;
        if now_ts <= data.timestamp && seller_id == &data.seller_id && num_top_sellers == data.num_top_sellers {
            log::info!("Summary data retrieved from cache");
            return Ok(data.summary_report);
        }
    }

    let query = SUMMARY_RPT_GRAPHQL
        .replace("***ORDER_OWNER_PARAM***", &format!("ownerId: \"{}\"", seller_id))
        .replace("***TOP_SELLERS_PARAM***", &format!("numTopSellers: {}", num_top_sellers));

    #[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
    struct SummaryReportRslt {
        #[serde(alias = "summary")]
        summary: SummaryReport,
    }

    let req = GraphQlReq::new(query);
    let rslt = make_gql_request::<SummaryReportRslt>(&req).await?;

    LocalStorage::set("SummaryData", SummaryReportStorage{
        summary_report: rslt.summary.clone(),
        seller_id: seller_id.to_string(),
        num_top_sellers: num_top_sellers,
        timestamp: Utc::now().timestamp(),
    })?;
    Ok(rslt.summary)
}

pub(crate) enum ReportViewState {
    IsLoading,
    ReportHtmlGenerated(Vec<serde_json::Value>),
}


static ORDER_VERIFICATION_GRAPHQL: &'static str = r"
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

pub(crate) async fn get_order_verfification_report_data(order_owner_id: Option<&String>)
    -> std::result::Result<Vec<serde_json::Value> ,Box<dyn std::error::Error>>
{

    let query = if let Some(order_owner_id) = order_owner_id {
        ORDER_VERIFICATION_GRAPHQL.replace("***ORDER_OWNER_PARAM***", &format!("ownerId: \"{}\"", order_owner_id))
    } else {
        ORDER_VERIFICATION_GRAPHQL.replace("(***ORDER_OWNER_PARAM***)", "")
    };
    log::info!("Running Query: {}", &query);
    make_report_query(query).await
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub(crate) struct ReportViewSettings {
    pub(crate) current_view: ReportViews,
    pub(crate) seller_id_filter: String,
}

pub(crate) fn save_report_settings(settings: &ReportViewSettings)
    -> std::result::Result<(), Box<dyn std::error::Error>>
{
    SessionStorage::set("ReportViewSettings", settings)?;
    Ok(())
}

pub(crate) fn delete_report_settings()
{
    SessionStorage::delete("ReportViewSettings");
}

pub(crate) fn load_report_settings() -> ReportViewSettings
{
    SessionStorage::get("ReportViewSettings").unwrap_or(
        ReportViewSettings{
            current_view: ReportViews::Quick,
            seller_id_filter: get_active_user().get_id(),
        }
    )
}

// const GEOJSONSTR: &str = r#"[]"#;
const GEOJSONSTR: &str = r#"
[{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.727027,30.539475]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.69232,30.49891]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.72692,30.532796]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.721862,30.535106]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.728062,30.543635]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.72584,30.517968]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.606452,30.518165]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.608994,30.51297]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.726752,30.542845]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.725178,30.53873]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.730078,30.534422]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.605195,30.51267]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.606485,30.523725]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.70529,30.48852]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.705467,30.534115]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.620037,30.502741]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.709093,30.54377]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.724662,30.535785]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.59397,30.50392]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.65038,30.49349]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.690247,30.499045]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.754325,30.561622]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.620896,30.500976]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.701385,30.484085]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.69083,30.501695]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.669963,30.553863]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.606027,30.497455]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.786069,30.606183]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.604622,30.512315]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.72807,30.54086]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.612891,30.515195]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.724056,30.539806]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.664265,30.58114]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.700672,30.543275]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.741819,30.544193]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.656685,30.539717]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.621509,30.502502]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.691007,30.499385]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.754061,30.558041]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.590585,30.514905]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.6071,30.52161]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.617909,30.505495]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.788247,30.601235]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.787006,30.599965]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.727713,30.54485]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.747732,30.556515]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.734346,30.536811]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.603732,30.519785]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.748671,30.510506]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.703622,30.54443]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.70944,30.54457]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.788608,30.60227]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.61769,30.51332]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.621653,30.506131]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.72786,30.543415]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.747475,30.555739]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.620037,30.502741]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.610109,30.561616]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.753048,30.558449]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.618937,30.51311]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.605235,30.50751]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.623981,30.513729]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.747665,30.55632]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.603615,30.52124]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.621537,30.553285]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.752341,30.559027]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.607235,30.51248]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.732726,30.532503]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.689238,30.498395]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.721647,30.534384]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.65038,30.49349]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.69243,30.506755]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.70139,30.49916]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.608147,30.566255]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.787662,30.59962]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.73035,30.52829]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.65038,30.49349]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.59327,30.49867]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.701782,30.4998]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.72695,30.5395]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.72346,30.539115]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.65873,30.52332]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.607175,30.5188]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.727865,30.535426]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.62299,30.55868]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.666079,30.55467]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.704146,30.48596]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.723784,30.519341]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.753356,30.557555]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.65038,30.49349]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.70958,30.54126]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.599794,30.508754]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.672145,30.558059]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.590672,30.514565]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.606422,30.52441]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.725045,30.538612]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.615871,30.560098]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.73106,30.55173]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.704845,30.48593]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.60258,30.52424]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.606385,30.52214]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.602412,30.507545]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.595325,30.516055]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.60689,30.49555]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.66503,30.578175]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.592322,30.508675]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.70771,30.53801]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.725389,30.537696]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.731175,30.54268]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.610792,30.511605]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.712227,30.531785]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.584115,30.55404]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.73299,30.530525]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.669565,30.54286]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.705,30.48723]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.72164,30.534604]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.75334,30.558092]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.70988,30.54359]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.608702,30.50976]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.723529,30.537961]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.616877,30.505837]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.692945,30.500835]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.666568,30.57447]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.71255,30.53311]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.593295,30.51402]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.62265,30.56247]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.72688,30.543125]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.732525,30.529435]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.65038,30.49349]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.704637,30.487225]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.696267,30.504615]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.596742,30.514025]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.728858,30.5265]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.74263,30.5358]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.603952,30.51345]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.60387,30.522395]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.6165,30.4998]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.733596,30.545702]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.74264,30.54579]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.593215,30.5143]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.622019,30.50209]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.710762,30.533184]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.72751,30.54523]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.729575,30.52514]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.59299,30.51573]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.619343,30.504888]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.625857,30.506535]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.604065,30.51174]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.703717,30.48555]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.742699,30.501252]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.726036,30.53998]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.601887,30.563775]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.70219,30.54268]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.699575,30.54159]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.612642,30.518155]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.627767,30.514835]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.617783,30.501504]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.616341,30.499495]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.704363,30.48564]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.723529,30.537961]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.725199,30.538719]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.61065,30.51411]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.701391,30.54783]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.747485,30.554415]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.694145,30.50579]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.601665,30.51656]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.592552,30.517525]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.733064,30.536165]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.723301,30.538951]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.703185,30.483415]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.7276,30.54378]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.604292,30.51359]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.702117,30.486255]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.618061,30.556362]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.617579,30.506677]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.601665,30.50732]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.593841,30.514615]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.609037,30.515355]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.751967,30.562835]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.725217,30.538706]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.59289,30.512965]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.629116,30.507595]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.623145,30.505012]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.592366,30.512745]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.728812,30.534733]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.60638,30.524765]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.721033,30.533752]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.751866,30.551595]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.78465,30.60292]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.715285,30.52838]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.601665,30.51656]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.604869,30.510925]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.606422,30.52441]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.728052,30.534215]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.703352,30.484145]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.751117,30.56178]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.738727,30.563289]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.752684,30.562601]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.729072,30.54974]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.78788,30.59831]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.705187,30.54519]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.731801,30.550486]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.728212,30.52778]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.65038,30.49349]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.752505,30.562659]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.734924,30.546345]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.604315,30.52166]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.605594,30.507235]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.59821,30.50962]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.607322,30.51439]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.619241,30.498697]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.614557,30.510125]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.753194,30.558271]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.665269,30.575354]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.704262,30.483885]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.608154,30.508305]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.78959,30.603495]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.605555,30.501615]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.599095,30.50659]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.604492,30.512595]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.712975,30.529195]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.610392,30.51326]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.597425,30.5094]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.618868,30.554047]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.607949,30.523945]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.731265,30.550945]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.601197,30.506675]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.60734,30.51339]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.610207,30.515165]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.604247,30.50966]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.616839,30.554284]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.591345,30.51544]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.599885,30.505055]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.724425,30.540299]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.593052,30.517235]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.623177,30.506205]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.725257,30.53868]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.702495,30.49124]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.701637,30.493545]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.599794,30.508754]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.612432,30.517365]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.696492,30.501655]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.597512,30.509925]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.72575,30.53913]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.730329,30.542225]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.6165,30.4998]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.701857,30.493365]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.733476,30.548074]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.661282,30.592775]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.715892,30.525085]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.730175,30.538663]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.7871,30.60525]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.754007,30.557985]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.724105,30.539864]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.601677,30.507145]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.723958,30.539689]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.693825,30.50049]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.60387,30.522395]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.751161,30.562005]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.723548,30.538956]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.601095,30.50229]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.704335,30.54011]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.595097,30.515425]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.618579,30.555672]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.602045,30.50589]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.717417,30.529505]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.704675,30.488535]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.604315,30.52166]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.601267,30.506505]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.619188,30.506859]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.732346,30.547962]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.65038,30.49349]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.747141,30.555888]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.72265,30.519755]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.752362,30.559002]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.664505,30.581095]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.78573,30.605725]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.601152,30.507]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.60925,30.51161]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.748671,30.510506]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.602962,30.507505]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.604347,30.514205]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.622987,30.500475]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.745452,30.555484]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.752617,30.55869]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.618004,30.505331]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.61798,30.505494]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.741251,30.54719]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.692022,30.500965]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.702049,30.485405]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.596035,30.51402]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.649139,30.538117]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.609517,30.564775]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.618677,30.500623]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.723354,30.539006]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.726922,30.538975]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.678806,30.508592]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.68671,30.48881]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.607215,30.51999]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.723185,30.52697]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.624076,30.513203]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.618885,30.5603]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.70862,30.541075]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.608172,30.513025]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.70867,30.543995]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.706572,30.534325]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.629102,30.506995]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.609094,30.556651]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.724022,30.539477]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.747358,30.555777]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.608085,30.49545]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.74879,30.55517]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.617546,30.500689]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.730241,30.529585]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.719849,30.535429]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.621866,30.501495]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.730995,30.518712]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.625756,30.548153]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.709056,30.51789]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.68671,30.48881]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.674695,30.49532]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.620902,30.508415]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.60455,30.50743]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.696492,30.501655]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.73202,30.536795]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.70711,30.543685]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.742132,30.54565]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.735668,30.537776]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.72775,30.54329]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.66313,30.56253]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.725902,30.528805]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.603182,30.528515]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.721865,30.535138]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.7322,30.52912]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.60476,30.510435]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.747382,30.555665]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.707922,30.53966]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.741806,30.54416]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.72508,30.538577]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.730645,30.528025]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.719395,30.535271]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.737307,30.540778]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.621777,30.506137]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.730105,30.52854]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.608154,30.508305]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.735801,30.537793]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.719148,30.536855]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.725035,30.528575]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.601162,30.50683]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.752587,30.559255]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.723248,30.538897]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.59835,30.504855]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.70167,30.54269]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.742433,30.545273]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.70609,30.541765]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.7267,30.54273]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.604897,30.50907]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.72342,30.538835]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.621286,30.502228]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.60772,30.511816]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.707985,30.50143]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.727965,30.543515]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.72575,30.53913]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.618101,30.501956]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.60326,30.508535]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.754184,30.55846]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.781465,30.600615]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.740779,30.559578]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.60779,30.51753]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.70325,30.4916]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.593901,30.515005]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.702822,30.489925]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.73617,30.53788]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.626062,30.507835]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.607137,30.524915]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.70288,30.484145]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.60501,30.57374]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.731175,30.550675]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.604415,30.52198]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.714457,30.530745]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.621452,30.505706]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.721685,30.533597]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.70277,30.48661]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.720582,30.533355]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.753822,30.557842]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.721638,30.534664]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.730131,30.54977]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.72868,30.545125]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.719256,30.53503]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.62995,30.511985]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.60323,30.507105]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.601235,30.51339]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.754114,30.560944]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.72714,30.54229]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.670335,30.551245]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.60851,30.52276]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.741819,30.544193]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.7276,30.54378]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.722114,30.519488]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.741921,30.54333]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.694995,30.49658]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.604375,30.50817]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.741015,30.54553]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.70304,30.49206]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.610222,30.512415]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.608994,30.51297]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.72299,30.534363]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.619648,30.556846]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.665126,30.546991]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.60511,30.51159]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.735725,30.52359]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.730981,30.551322]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.70454,30.54452]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.619145,30.503217]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.596995,30.514845]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.61065,30.51411]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.613175,30.517065]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.727082,30.542505]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.743911,30.555628]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.727047,30.54337]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.619186,30.502937]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.702435,30.485]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.666079,30.55467]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.724662,30.527455]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.604805,30.51015]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.723182,30.53889]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.701892,30.495255]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.617637,30.500687]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.703357,30.54235]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.607772,30.566775]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.617175,30.55317]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.602487,30.512425]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.72701,30.54196]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.60552,30.50142]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.623203,30.510551]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.722015,30.53501]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.728943,30.535389]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.610697,30.51348]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.590247,30.510365]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.606484,30.51994]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.605042,30.52181]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.724674,30.54037]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.724203,30.539981]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.753229,30.562246]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.700474,30.542835]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.761232,30.52045]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.724246,30.539749]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.748915,30.5546]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.602037,30.51449]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.616973,30.505806]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.703015,30.497415]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.624834,30.536444]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.665126,30.546991]},"properties":null},{"type":"Feature","geometry":{"type":"Point","coordinates":[-97.610937,30.519695]},"properties":null}]
"#;
pub(crate) async fn get_sales_geojson()
    -> std::result::Result<Vec<serde_json::Value> ,Box<dyn std::error::Error>>
{
    // log::info!("Running Query: {}", &query);
    // make_report_query(query).await

    let resp:Vec<serde_json::Value> = serde_json::from_str(GEOJSONSTR)
        .unwrap_or_else(|_|Vec::<serde_json::Value>::new());
    Ok(resp)

}
