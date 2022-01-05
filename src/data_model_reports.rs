
use serde::{Deserialize, Serialize};
use chrono::prelude::*;
use gloo_storage::{LocalStorage, Storage};
use crate::gql_utils::{make_gql_request, GraphQlReq};

static QUICK_RPT_GRAPHQL: &'static str = r"
{
  mulchOrders(***ORDER_OWNER_PARAM***) {
    orderId
    ownerId
    deliveryId
    spreaders
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
        QUICK_RPT_GRAPHQL.replace("***ORDER_OWNER_PARAM***", "")
    };

    #[derive(Serialize, Deserialize, Debug)]
    struct GqlResp {
        #[serde(alias = "mulchOrders")]
        mulch_orders: Vec<serde_json::Value>,
    }

    let req = GraphQlReq::new(query);
    let rslts = make_gql_request::<GqlResp>(&req).await?;
    Ok(rslts.mulch_orders)
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
