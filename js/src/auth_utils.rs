use once_cell::sync::Lazy as LazyLock;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
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

pub async fn get_active_user_async() -> Option<Arc<AuthenticatedUserInfo>> {
    match getUserInfo().await {
        Ok(user_info) => {
            //log::info!("User Info: {:#?}", user_info);
            let user_info: AuthenticatedUserInfo =
                serde_wasm_bindgen::from_value(user_info).unwrap();
            let user_info = Arc::new(user_info);
            *ACTIVE_USER.write().unwrap() = Some(user_info.clone());
            Some(user_info)
        }
        Err(err) => {
            log::error!("User Info Err: {:#?}", err);
            gloo::dialogs::alert(&format!("Failed to get User Info: {:#?}", err));
            None
        }
    }
}

pub fn get_active_user() -> Arc<AuthenticatedUserInfo> {
    ACTIVE_USER.read().unwrap().as_ref().unwrap().clone()
}

pub async fn is_authenticated() -> bool {
    match isAuthenticated().await {
        Ok(is_auth) => {
            log::info!("Is Authenticated: {:#?}", &is_auth);
            let is_auth: bool = serde_wasm_bindgen::from_value(is_auth).unwrap();
            return is_auth;
        }
        Err(err) => log::error!("User Info Err: {:#?}", err),
    };
    false
}

pub async fn login() {
    if let Err(err) = loginUser().await {
        log::error!("Error logging in Err: {:#?}", err);
    } else {
        log::info!("Logged In");
    }
}

pub async fn logout() {
    if let Err(err) = logoutUser().await {
        log::error!("Error logging out Err: {:#?}", err);
    } else {
        log::info!("Logged out");
    }
}
