mod currency_utils;
mod data_model;
mod data_model_orders;
mod data_model_reports;
mod gql_utils;

pub use currency_utils::*;
pub use data_model::*;
pub use data_model_orders::*;
pub use data_model_reports::*;
pub use js::auth_utils::{get_active_user, get_active_user_async};

// Needed for HTML functions
use rust_decimal::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{HtmlInputElement, HtmlSelectElement, HtmlTextAreaElement};

// Needed for AppRoutes
use yew_router::prelude::*;

/////////////////////////////////////////////////
///
pub const CLOUD_API_URL: &'static str =
    "https://j0azby8rm6.execute-api.us-east-1.amazonaws.com/prod";

/////////////////////////////////////////////////
// Route Logic
#[derive(Clone, Routable, PartialEq, Debug)]
pub enum AppRoutes {
    #[at("/")]
    Home,
    #[at("/order")]
    OrderForm,
    #[at("/orderproducts")]
    OrderProducts,
    #[at("/orderdonations")]
    OrderDonations,
    #[at("/reports")]
    Reports,
    #[at("/timecards")]
    Timecards,
    #[at("/frcloseout")]
    FundraiserCloseout,
    #[at("/frcconfig")]
    FrConfig,
    #[not_found]
    #[at("/404")]
    NotFound,
}

/////////////////////////////////////////////////
///
pub fn save_to_active_order() {
    if !is_active_order() {
        return;
    }

    let document = gloo::utils::document();
    let mut order = get_active_order().unwrap();

    if let Some(order_owner_element) = document
        .get_element_by_id("formOrderOwner")
        .and_then(|t| t.dyn_into::<HtmlSelectElement>().ok())
    {
        // If it isn't there then it is because we aren't in admin mode
        order.order_owner_id = order_owner_element.value();
    }

    order.customer.name =
        get_html_input_value("formCustomerName", &document).unwrap_or("".to_string());
    order.customer.phone = get_html_input_value("formPhone", &document).unwrap_or("".to_string());
    order.customer.email = get_html_input_value("formEmail", &document);
    order.customer.addr1 = get_html_input_value("formAddr1", &document).unwrap_or("".to_string());
    order.customer.addr2 = get_html_input_value("formAddr2", &document);
    order.customer.city = get_html_input_value("formCity", &document);
    order.customer.zipcode =
        get_html_input_value("formZipcode", &document).map(|v| v.parse::<u32>().unwrap());
    order.customer.neighborhood = Some(
        document
            .get_element_by_id("formNeighborhood")
            .and_then(|t| t.dyn_into::<HtmlSelectElement>().ok())
            .unwrap()
            .value(),
    );
    order.comments = get_html_textarea_value("formOrderComments", &document);
    order.special_instructions = get_html_textarea_value("formSpecialInstructions", &document);
    order.amount_cash_collected = get_html_input_value("formCashPaid", &document);
    order.amount_checks_collected = get_html_input_value("formCheckPaid", &document);
    order.check_numbers = get_html_input_value("formCheckNumbers", &document);
    order.will_collect_money_later =
        Some(get_html_checked_input_value("formCollectLater", &document));
    order.is_verified = Some(get_html_checked_input_value("formIsVerified", &document));
    // This must come after setting checks/cash collected
    let total_collected = order.get_total_collected();
    if total_collected == Decimal::ZERO {
        order.amount_total_collected = None;
    } else {
        order.amount_total_collected = Some(total_collected.to_string());
    }

    log::info!("Saving Order: {:#?}", &order);
    update_active_order(order).unwrap();
}

/////////////////////////////////////////////////
///
pub fn get_html_input_value(id: &str, document: &web_sys::Document) -> Option<String> {
    let value = document
        .get_element_by_id(id)
        .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
        .unwrap()
        .value();
    if 0 == value.len() {
        None
    } else {
        Some(value)
    }
}

/////////////////////////////////////////////////
///
pub fn get_html_checked_input_value(id: &str, document: &web_sys::Document) -> bool {
    document
        .get_element_by_id(id)
        .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
        .unwrap()
        .checked()
}

/////////////////////////////////////////////////
///
pub fn get_html_textarea_value(id: &str, document: &web_sys::Document) -> Option<String> {
    let value = document
        .get_element_by_id(id)
        .and_then(|t| t.dyn_into::<HtmlTextAreaElement>().ok())
        .unwrap()
        .value();
    if 0 == value.len() {
        None
    } else {
        Some(value)
    }
}
