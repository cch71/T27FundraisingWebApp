use data_model::{
    CLOUD_API_URL, get_active_user, get_fr_config, get_neighborhood,
    gql_utils::{GraphQlReq, make_gql_request},
};
use gloo::storage::{SessionStorage, Storage};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::LazyLock;
use tracing::{error, info, warn};

// Exposing this const out to keep consistent tag name
pub static ALL_USERS_TAG: &str = "doShowAllUsers";
// URL to api that returns GeoJSON locations
static GEOJSONURL: LazyLock<String> =
    LazyLock::new(|| CLOUD_API_URL.to_owned() + "/salelocs");

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
            _ => Err(format!("'{s}' is not a valid value for ReportViews")),
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

    // if allocation_summary available add allocation summary {
    //      reports.push(ReportViews::AllocationSummary);
    // }

    reports
}

/// Parses the purchases and creates a map
pub fn get_purchase_to_map(v: &serde_json::Value) -> HashMap<String, u64> {
    let mut purchases = HashMap::new();

    for purchase in v["purchases"].as_array().unwrap_or(&Vec::new()) {
        let product_id = purchase["productId"].as_str();
        match product_id {
            Some("spreading") => {
                purchases.insert(
                    "spreading".to_string(),
                    purchase["numSold"].as_u64().unwrap_or_default(),
                );
            }
            Some("bags") => {
                purchases.insert(
                    "bags".to_string(),
                    purchase["numSold"].as_u64().unwrap_or_default(),
                );
            }
            _ => error!("Unknown product id: {product_id:?}"),
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
    make_gql_request::<GqlResp>(&req)
        .await
        .map(|v| v.mulch_orders)
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
pub fn load_report_settings() -> ReportViewSettings {
    SessionStorage::get("ReportViewSettings").unwrap_or(ReportViewSettings {
        current_view: ReportViews::Quick,
        seller_id_filter: get_active_user().get_id(),
    })
}

//////////////////////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////////
pub async fn get_sales_geojson() -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    use gloo::net::http::Request;
    // info!("Running Query: {}", &query);

    let raw_resp: serde_json::Value = Request::get(&GEOJSONURL)
        .header("Content-Type", "application/json")
        .header(
            "Authorization",
            &format!("Bearer {}", &get_active_user().token),
        )
        .send()
        .await?
        .json()
        .await?;

    // match gloo::utils::window().location().host() {
    //     Ok(host) if host.contains("localhost") => {
    //         info!(
    //             "GeoJSON Resp: {}",
    //             serde_json::to_string_pretty(&raw_resp).unwrap()
    //         );
    //     }
    //     _ => {}
    // };

    if !raw_resp["message"].is_null() {
        let err_str =
            serde_json::to_string(&raw_resp).unwrap_or("Failed to stringify json resp".to_string());
        return Err(Box::new(std::io::Error::other(format!(
            "GeoJSON request returned raw error:\n {err_str}"
        ))));
    }

    // make_report_query(query).await
    Ok(raw_resp)
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
            &format!("ownerId: \"{order_owner_id}\""),
        )
    } else {
        QUICK_RPT_GRAPHQL.replace("(***ORDER_OWNER_PARAM***)", "")
    };
    info!("Running Query: {}", &query);
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
            &format!("ownerId: \"{order_owner_id}\""),
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
            &format!("ownerId: \"{order_owner_id}\""),
        )
    } else {
        MONEY_COLLECTION_RPT_GRAPHQL.replace("(***ORDER_OWNER_PARAM***)", "")
    };
    info!("Running Query: {}", &query);
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
                .map(|v| {
                    let purchases = get_purchase_to_map(&v);
                    let num_bags_sold = purchases.get("bags").copied().unwrap_or(0);
                    (v, num_bags_sold)
                })
                .filter(|(_v, num_bags_sold)| *num_bags_sold != 0)
                .for_each(|(v, num_bags_sold)| {
                    let delivery_id = v["deliveryId"].as_u64().unwrap_or(0);
                    delivery_id_map.entry(delivery_id).or_default();
                    let dist_point_map = delivery_id_map.get_mut(&delivery_id).unwrap();
                    let neighborhood = v["customer"]["neighborhood"].as_str().unwrap_or("");
                    let dist_point = get_neighborhood(neighborhood)
                        .map_or("".to_string(), |v| v.distribution_point.clone());
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
    //  adding another enum, but this is the lesser impact for a report that is rarely ran
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
            &format!(", ownerId: \"{order_owner_id}\""),
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
            &format!(", excludeOwnerId: \"{order_owner_id}\", spreaderId: \"{order_owner_id}\""),
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
                    warn!("Query qualifier should have meant this doesn't happen!");
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
            &format!("ownerId: \"{order_owner_id}\""),
        )
    } else {
        ORDER_VERIFICATION_GRAPHQL.replace("(***ORDER_OWNER_PARAM***)", "")
    };
    info!("Running Query: {}", &query);
    make_report_query(query).await
}
