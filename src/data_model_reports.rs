
use serde::{Deserialize, Serialize};

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
