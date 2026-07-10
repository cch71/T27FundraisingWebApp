use data_model::*;
use rust_decimal::prelude::*;
use tracing::info;
use wasm_bindgen::JsCast;
use web_sys::HtmlSelectElement;

/////////////////////////////////////////////////
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
        get_html_input_value("formZipcode", &document).and_then(|v| v.trim().parse::<u32>().ok());
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

    info!("Saving Order: {:#?}", &order);
    update_active_order(order).unwrap();
}

/////////////////////////////////////////////////
/// Saves the order form to the active order if the form is currently in the
/// DOM. Called both when switching between order pages and when the shell
/// unmounts this module, so in-progress edits survive navigation.
pub fn save_order_form_if_present() {
    let document = gloo::utils::document();
    if document.get_element_by_id("newOrEditOrderForm").is_some() && is_active_order() {
        save_to_active_order();
    }
}
