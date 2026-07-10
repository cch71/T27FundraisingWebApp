use super::gql_utils::{GraphQlReq, make_gql_request};
use chrono::prelude::*;
use gloo::storage::{LocalStorage, SessionStorage, Storage};
use serde::{Deserialize, Serialize};
use tracing::info;

/////////////////////////////////////////////////////////////////////////////////
/// Clears session scoped state (report settings, active order, auth session)
/// so a logon/logoff can force a clean reload.
pub fn clear_session_storage() {
    SessionStorage::clear();
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
    if let Ok(data) = LocalStorage::get::<SummaryReportStorage>("SummaryData") {
        let now_ts = Utc::now().timestamp() - 86400;
        if now_ts <= data.timestamp
            && seller_id == data.seller_id
            && top_sellers == data.num_top_sellers
        {
            info!("Summary data retrieved from cache");
            return Ok(data.summary_report);
        }
    }

    let query = SUMMARY_RPT_GRAPHQL
        .replace(
            "***ORDER_OWNER_PARAM***",
            &format!("ownerId: \"{seller_id}\""),
        )
        .replace(
            "***TOP_SELLERS_PARAM***",
            &format!("numTopSellers: {top_sellers}"),
        );
    info!("Running Query: {}", &query);
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
