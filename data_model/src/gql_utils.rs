use super::get_active_user;
use gloo::net::http::Request;
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

static GQLURL: LazyLock<String> = LazyLock::new(|| crate::CLOUD_API_URL.to_string() + "/graphql");

#[derive(Serialize, Deserialize, Debug)]
pub(super) struct GraphQlReq {
    pub query: String,
}
impl GraphQlReq {
    pub fn new<T: AsRef<str>>(query: T) -> Self {
        Self {
            query: query.as_ref().to_string(),
        }
    }
}

pub(super) async fn make_gql_request<T>(
    req: &GraphQlReq,
) -> Result<T, Box<dyn std::error::Error>>
where
    T: serde::de::DeserializeOwned,
{
    #[derive(Serialize, Deserialize, Debug)]
    struct DataWrapper<T> {
        data: Option<T>,
        errors: Option<Vec<serde_json::Value>>,
    }

    // log::info!("Bearer Token: {}", get_active_user().token);
    let raw_resp: serde_json::Value = Request::post(&GQLURL)
        .header("Content-Type", "application/json")
        .header(
            "Authorization",
            &format!("Bearer {}", &get_active_user().token),
        )
        .body(serde_json::to_string(req).unwrap())?
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
            "GQL Resp: {}",
            serde_json::to_string_pretty(&raw_resp).unwrap()
        );
    }

    if !raw_resp["message"].is_null() {
        let err_str =
            serde_json::to_string(&raw_resp).unwrap_or("Failed to stringify json resp".to_string());
        return Err(Box::new(std::io::Error::other(
            format!("GQL request returned raw error:\n {err_str}").as_str(),
        )));
    }

    let resp: DataWrapper<T> = serde_json::from_value(raw_resp)?;
    match resp.errors {
        Some(errs) => {
            let err_str =
                serde_json::to_string(&errs).unwrap_or("Failed to parse error resp".to_string());
            Err(Box::new(std::io::Error::other(
                format!("GQL request returned error:\n {err_str}").as_str(),
            )))
        }
        _ => Ok(resp.data.unwrap()),
    }
}
