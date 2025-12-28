use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, LazyLock, RwLock};
use tracing::{error, info};
use wasm_bindgen::prelude::*;

static ACTIVE_USER: LazyLock<RwLock<Option<Arc<AuthenticatedUserInfo>>>> =
    LazyLock::new(|| RwLock::new(None));

/////////////////////////////////////////////////
// Auth Comp Stuff
#[wasm_bindgen(module = "/src/js/auth.js")]
extern "C" {
    #[wasm_bindgen(catch)]
    async fn loginUser() -> Result<(), JsValue>;

    #[wasm_bindgen(catch)]
    async fn logoutUser() -> Result<(), JsValue>;

    #[wasm_bindgen(catch)]
    async fn isAuthenticated() -> Result<JsValue, JsValue>;

    #[wasm_bindgen(catch)]
    async fn getUserInfo() -> Result<JsValue, JsValue>;
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct AuthenticatedUserInfo {
    pub email: String,
    pub token: String,
    name: Option<String>,
    id: Option<String>,
    pub roles: Vec<String>,
}

impl AuthenticatedUserInfo {
    pub fn get_id(&self) -> String {
        // TODO: Do this at creation time
        self.id.as_ref().map_or_else(
            || {
                let v: Vec<&str> = self.email.split('@').collect();
                v[0].to_string()
            },
            |v| v.clone(),
        )
    }

    pub fn get_name(&self) -> String {
        self.name.as_ref().unwrap_or(&self.get_id()).clone()
    }

    pub fn is_admin(&self) -> bool {
        self.roles.contains(&"FrAdmins".to_string())
    }
}

fn parse_active_user(user_info: JsValue) -> anyhow::Result<Arc<AuthenticatedUserInfo>> {
    match serde_wasm_bindgen::from_value::<AuthenticatedUserInfo>(user_info) {
        Ok(user_info) => Ok(Arc::new(user_info)),
        Err(err) => Err(anyhow!("{:?}", err)),
    }
}

pub async fn get_active_user_async() -> anyhow::Result<Arc<AuthenticatedUserInfo>> {
    match getUserInfo().await {
        Ok(user_info) => {
            // log::info!("User Info: {:#?}", user_info);
            parse_active_user(user_info).and_then(|user_info| {
                ACTIVE_USER
                    .write()
                    .map(|mut active_user| {
                        active_user.replace(user_info.clone());
                        user_info
                    })
                    .map_err(|err| anyhow!("Failed to replace user info: {err}"))
            })
        }
        Err(err) => Err(anyhow!("Get User Info Err: {:#?}", err)),
    }
}

pub fn get_active_user() -> Arc<AuthenticatedUserInfo> {
    ACTIVE_USER.read().unwrap().as_ref().unwrap().clone()
}

pub async fn is_authenticated() -> bool {
    match isAuthenticated().await {
        Ok(is_auth) => {
            info!("Is Authenticated: {:#?}", &is_auth);
            let is_auth: bool = serde_wasm_bindgen::from_value(is_auth).unwrap();
            return is_auth;
        }
        Err(err) => error!("User Info Err: {err:#?}"),
    };
    false
}

pub async fn login() {
    match loginUser().await {
        Err(err) => {
            error!("Error logging in Err: {err:#?}");
        }
        _ => {
            info!("Logged In");
        }
    }
}

pub async fn logout() {
    match logoutUser().await {
        Err(err) => {
            error!("Error logging out Err: {err:#?}");
        }
        _ => {
            info!("Logged out");
        }
    }
}
