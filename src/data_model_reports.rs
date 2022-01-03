
use serde::{Deserialize, Serialize};
use yew::prelude::*;

use crate::gql_utils::{make_gql_request, GraphQlReq};


// {
//    "data":{
//       "mulchOrders":[
//          {
//             "amountTotalCollected":"955.0000",
//             "customer":{
//                "name":"Moona Luna Hamilton"
//             },
//             "deliveryId":2,
//             "orderId":"5e0c9b2a-e281-4977-ad67-59f7e066f4c0",
//             "purchases":[
//                {
//                   "amountCharged":"10.00",
//                   "numSold":5,
//                   "productId":"spreading"
//                },
//                {
//                   "amountCharged":"945.00",
//                   "numSold":252,
//                   "productId":"bags"
//                }
//             ],
//             "spreaders":null
//          },
//          {
//             "amountTotalCollected":"20.0000",
//             "customer":{
//                "name":"Aidan Hamilton"
//             },
//             "deliveryId":null,
//             "orderId":"9e484974-5a6d-4f7a-9780-4a83726e1269",
//             "purchases":[
//
//             ],
//             "spreaders":null
//          },
//          {
//             "amountTotalCollected":null,
//             "customer":{
//                "name":"Shannon Hamilton"
//             },
//             "deliveryId":2,
//             "orderId":"deccf9e5-18d5-4bb8-ae43-9ab0b5961d9d",
//             "purchases":[
//                {
//                   "amountCharged":"48.00",
//                   "numSold":24,
//                   "productId":"spreading"
//                }
//             ],
//             "spreaders":null
//          },
//          {
//             "amountTotalCollected":"96.0000",
//             "customer":{
//                "name":"Ariana Hamilton"
//             },
//             "deliveryId":1,
//             "orderId":"e407a220-33cd-4eed-b36b-d41368aca871",
//             "purchases":[
//                {
//                   "amountCharged":"96.00",
//                   "numSold":24,
//                   "productId":"bags"
//                }
//             ],
//             "spreaders":null
//          }
//       ]
//    }
// }


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
