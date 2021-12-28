


/////////////////////////////////////////////////
// Auth Comp Stuff
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct UserInfo {
    pub email: String,
    pub token: String,
}

impl UserInfo {
    pub fn user_id(&self)->String {
        // TODO: Do this at creation time
        let v: Vec<&str> = self.email.split('@').collect();
        v[0].to_owned()
    }
}

pub async fn get_user_info() -> Option<UserInfo> {
    match getUserInfo().await {
        Ok(user_info) => {
            log::info!("User Info: {:#?}", user_info);
            let user_info: UserInfo = serde_wasm_bindgen::from_value(user_info).unwrap();
            Some(user_info)
        },
        Err(err) => {
            log::error!("User Info Err: {:#?}", err);
            gloo_dialogs::alert(&format!("Failed to get User Info: {:#?}", err));
            None
        },
    }
}

pub async fn is_authenticated() -> bool {
    match isAuthenticated().await {
        Ok(is_auth) => {
            log::info!("Is Authenticated: {:#?}", &is_auth);
            let is_auth: bool = serde_wasm_bindgen::from_value(is_auth).unwrap();
            return is_auth;
        },
        Err(err) => log::error!("User Info Err: {:#?}", err),
    };
    false
}

pub async fn login() {
    if let Err(err) = loginUser().await {
        log::error!("Authentication Err: {:#?}", err);
    } else {
        log::info!("Authenticated");
    }
}

pub async fn logout() {
    if let Err(err) = logoutUser().await {
        log::error!("Error logging out Err: {:#?}", err);
    } else {
        log::info!("Logged out");
    }
}
