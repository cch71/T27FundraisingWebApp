use serde::{Deserialize, Serialize};
use reqwasm::http::Request;
use lazy_static::lazy_static;

use crate::auth_utils::{get_active_user};

lazy_static! {
    static ref GQLURL: String = format!("{}/graphql", get_cloud_api_url());
}


#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct GraphQlReq {
    pub(crate) query: String,
}
impl GraphQlReq {
    pub(crate) fn new(query: String) -> Self {
        return Self{
            query: query,
        }
    }
}

fn get_cloud_api_url() -> &'static str {
    //AWS API URL
    //invokeUrl: 'https://j0azby8rm6.execute-api.us-east-1.amazonaws.com/prod'
    "https://j0azby8rm6.execute-api.us-east-1.amazonaws.com/prod"
}

pub(crate) async fn make_gql_request<T>(req: &GraphQlReq)
    -> std::result::Result<T,Box<dyn std::error::Error>>
    where T: serde::de::DeserializeOwned
{
    #[derive(Serialize, Deserialize, Debug)]
    struct DataWrapper<T> {
        data: Option<T>,
        errors: Option<Vec<serde_json::Value>>,
    }

    // log::info!("Bearer Token: {}", get_active_user().token);
    let raw_resp: serde_json::Value = Request::post(&*GQLURL)
        .header("Content-Type", "application/json")
        .header("Authorization", &format!("Bearer {}", &get_active_user().token))
        .body(serde_json::to_string(req).unwrap())
        .send()
        .await?
        .json()
        .await?;

    // log::info!("GQL Resp: {}", serde_json::to_string_pretty(&raw_resp).unwrap());
    let resp: DataWrapper<T> = serde_json::from_value(raw_resp).unwrap();
    if let Some(errs) = resp.errors {
        let err_str = serde_json::to_string(&errs).unwrap_or("Failed to parse error resp".to_string());
        use std::io::{Error, ErrorKind};
        Err(Box::new(
                Error::new(
                    ErrorKind::Other,
                    format!("GQL request returned and error:\n {}", err_str).as_str())))
    } else {
        Ok(resp.data.unwrap())
    }
}

