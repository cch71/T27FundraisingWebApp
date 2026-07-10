mod currency_utils;
mod data_model;
mod data_model_orders;
mod data_model_summary;
pub mod gql_utils;
mod module_init;

pub use currency_utils::*;
pub use data_model::*;
pub use data_model_orders::*;
pub use data_model_summary::*;
pub use module_init::*;
pub use js::auth_utils::{get_active_user, get_active_user_async};

// Needed for HTML functions
use wasm_bindgen::JsCast;
use web_sys::{HtmlInputElement, HtmlTextAreaElement};

/////////////////////////////////////////////////
pub const CLOUD_API_URL: &str = "https://j0azby8rm6.execute-api.us-east-1.amazonaws.com/prod";

/////////////////////////////////////////////////
pub const NUM_TOP_SELLERS_TO_GET: u8 = 25;

/////////////////////////////////////////////////
pub fn get_element<T>(id: &str, document: &web_sys::Document) -> T
where
    T: JsCast,
{
    document
        .get_element_by_id(id)
        .and_then(|t| t.dyn_into::<T>().ok())
        .unwrap()
}

/////////////////////////////////////////////////
pub fn get_html_input_value(id: &str, document: &web_sys::Document) -> Option<String> {
    let value = get_element::<HtmlInputElement>(id, document).value();
    if value.is_empty() { None } else { Some(value) }
}

/////////////////////////////////////////////////
pub fn get_html_checked_input_value(id: &str, document: &web_sys::Document) -> bool {
    document
        .get_element_by_id(id)
        .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
        .unwrap()
        .checked()
}

/////////////////////////////////////////////////
pub fn get_html_textarea_value(id: &str, document: &web_sys::Document) -> Option<String> {
    let value = document
        .get_element_by_id(id)
        .and_then(|t| t.dyn_into::<HtmlTextAreaElement>().ok())
        .unwrap()
        .value();
    if value.is_empty() { None } else { Some(value) }
}
