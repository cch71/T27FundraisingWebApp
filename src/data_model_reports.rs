
use serde::{Deserialize, Serialize};
use chrono::prelude::*;
use gloo_storage::{LocalStorage, SessionStorage, Storage};
use crate::gql_utils::{make_gql_request, GraphQlReq};
use crate::auth_utils::{get_active_user};
use crate::data_model::{get_fr_config};

pub(crate) static ALL_USERS_TAG: &'static str = "doShowAllUsers";

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub(crate) enum ReportViews {
    // Reports available to sellers
    Quick,
    Full,
    SpreadingJobs,
    AllocationSummary,

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
            "Allocation Summary" => Ok(ReportViews::AllocationSummary),
            _ => Err(format!("'{}' is not a valid value for ReportViews", s)),
        }
    }
}

pub(crate) fn get_allowed_report_views() -> Vec<ReportViews> {
    let mut reports = vec![
        ReportViews::Quick,
        ReportViews::Full,
    ];

    if get_fr_config().kind == "mulch" {
        reports.push(ReportViews::SpreadingJobs);

        if get_active_user().is_admin() {
            //         reports.push(ReportViews::UnfinishedSpreadingJobs);
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
                            *dist_point_map.get_mut("TotalBagSummary").unwrap() += num_bags_sold;
                        },
                        None=>{
                            dist_point_map.insert(dist_point.to_string(), num_bags_sold);
                            dist_point_map.insert("TotalBagSummary".to_string(), num_bags_sold);
                        },
                    }
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

#[derive(Serialize, Deserialize, Debug)]
struct SummaryReportStorage {
    summary_report: SummaryReport,
    seller_id: String,
    num_top_sellers: u32,
    timestamp: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub(crate) struct SummaryReport {
    #[serde(alias = "troopSummary")]
    pub(crate) troop_summary: TroopSummary,
    #[serde(alias = "summaryByOwnerId")]
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

    //TODO: Add Allocations in fo rthis
}

static SUMMARY_RPT_GRAPHQL: &'static str = r"
{
  summaryByOwnerId(***ORDER_OWNER_PARAM***) {
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
  troopSummary(***TOP_SELLERS_PARAM***) {
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

    let req = GraphQlReq::new(query);
    let rslts = make_gql_request::<SummaryReport>(&req).await?;

    LocalStorage::set("SummaryData", SummaryReportStorage{
        summary_report: rslts.clone(),
        seller_id: seller_id.to_string(),
        num_top_sellers: num_top_sellers,
        timestamp: Utc::now().timestamp(),
    })?;
    Ok(rslts)
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
