use data_model::*;
use rust_decimal::prelude::*;
use rusty_money::{Money, iso};
use wasm_bindgen::JsCast;
use web_sys::{
    Element, Event, HtmlButtonElement, HtmlElement, HtmlInputElement, HtmlSelectElement,
    InputEvent, MouseEvent, SubmitEvent,
};
use yew::prelude::*;
use yew_router::prelude::*;

/////////////////////////////////////////////////
fn set_html_input_value(id: &str, document: &web_sys::Document, value: &str) {
    document
        .get_element_by_id(id)
        .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
        .unwrap()
        .set_value(value);
}

/////////////////////////////////////////////////
fn get_cash_amount_collected(document: &web_sys::Document) -> Option<String> {
    get_html_input_value("formCashPaid", document)
}

/////////////////////////////////////////////////
fn get_check_amount_collected(document: &web_sys::Document) -> Option<String> {
    get_html_input_value("formCheckPaid", document)
}

/////////////////////////////////////////////////
fn set_cash_amount_collected(document: &web_sys::Document, value: &str) {
    set_html_input_value("formCashPaid", document, value)
}

/////////////////////////////////////////////////
fn set_check_amount_collected(document: &web_sys::Document, value: &str) {
    set_html_input_value("formCheckPaid", document, value)
}

/////////////////////////////////////////////////
fn disable_submit_button(document: &web_sys::Document, value: bool, with_spinner: bool) {
    if let Some(btn) = document
        .get_element_by_id("formOrderSubmit")
        .and_then(|t| t.dyn_into::<HtmlButtonElement>().ok())
    {
        btn.set_disabled(value);
        let spinner_display = if with_spinner { "inline-block" } else { "none" };
        let _ = document
            .get_element_by_id("formOrderSubmitSpinner")
            .and_then(|t| t.dyn_into::<HtmlElement>().ok())
            .unwrap()
            .style()
            .set_property("display", spinner_display);
    }
}

/////////////////////////////////////////////////
fn disable_cancel_button(document: &web_sys::Document, value: bool) {
    document
        .get_element_by_id("formOrderCancel")
        .and_then(|t| t.dyn_into::<HtmlButtonElement>().ok())
        .unwrap()
        .set_disabled(value);
}

/////////////////////////////////////////////////
fn update_order_amount_due_element(order: &MulchOrder, document: &web_sys::Document) {
    let total_to_collect = order.get_total_to_collect();
    document
        .get_element_by_id("orderAmountDue")
        .and_then(|t| t.dyn_into::<HtmlElement>().ok())
        .unwrap()
        .set_inner_text(&Money::from_decimal(total_to_collect, iso::USD).to_string());
}

/////////////////////////////////////////////////
fn update_city_and_zip<T: AsRef<str>>(city: T, zipcode: T, document: &web_sys::Document) {
    set_html_input_value("formZipcode", document, zipcode.as_ref());
    set_html_input_value("formCity", document, city.as_ref());
}

/////////////////////////////////////////////////
fn update_addr1(addr: String, document: &web_sys::Document) {
    set_html_input_value("formAddr1", document, addr.as_str());
}

/////////////////////////////////////////////////
fn validate_order_form(document: &web_sys::Document) -> bool {
    save_to_active_order();
    let order = get_active_order().unwrap();
    let mut is_valid = true;

    let check_num_field_element = document
        .get_element_by_id("formCheckNumbers")
        .and_then(|t| t.dyn_into::<Element>().ok())
        .unwrap();
    let totals_form_row_element = document
        .get_element_by_id("totalsFormRow")
        .and_then(|t| t.dyn_into::<Element>().ok())
        .unwrap();
    match order.is_payment_valid() {
        true => {
            let _ = totals_form_row_element.class_list().remove_1("is-invalid");
            let _ = check_num_field_element.class_list().remove_1("is-invalid");
        }
        false => {
            let _ = totals_form_row_element.class_list().add_1("is-invalid");
            if !order.is_check_numbers_valid() {
                let _ = check_num_field_element.class_list().add_1("is-invalid");
            }
            is_valid = false;
        }
    };

    let form_node_list = document
        .query_selector("#newOrEditOrderForm")
        .ok()
        .flatten()
        .and_then(|t| t.dyn_into::<Element>().ok())
        .unwrap()
        .query_selector_all("[required]")
        .unwrap();
    for idx in 0..form_node_list.length() {
        log::info!("Going through Form List");
        if let Some(element) = form_node_list
            .item(idx)
            .and_then(|t| t.dyn_into::<Element>().ok())
        {
            //log::info!("Validationg ID: {}", element.id());
            let is_form_element_valid = {
                match element.clone().dyn_into::<HtmlInputElement>() {
                    Ok(form_element) => form_element.check_validity(),
                    _ => match element.clone().dyn_into::<HtmlSelectElement>() {
                        Ok(form_element) => {
                            form_element.check_validity() && form_element.value() != ""
                        }
                        _ => false,
                    },
                }
            };
            if is_form_element_valid {
                let _ = element.class_list().remove_1("is-invalid");
            } else {
                let _ = element.class_list().add_1("is-invalid");
                is_valid = false;
            }
        }
    }

    let product_list_element = document
        .get_element_by_id("productList")
        .and_then(|t| t.dyn_into::<Element>().ok())
        .unwrap();
    if order.are_purchases_valid() {
        let _ = product_list_element.class_list().remove_1("is-invalid");
    } else {
        let _ = product_list_element.class_list().add_1("is-invalid");
        is_valid = false;
    }

    is_valid
}

/////////////////////////////////////////////////
/////////////////////////////////////////////////
#[function_component(RequiredSmall)]
fn required_small() -> Html {
    html! {
        <small class="form-text text-muted ps-1">{"*required"}</small>
    }
}

/////////////////////////////////////////////////
/////////////////////////////////////////////////
#[derive(Properties, PartialEq, Clone, Debug)]
pub struct OrderCostItemProps {
    pub label: AttrValue,
    pub isreadonly: bool,
    pub amount: AttrValue,
    #[prop_or_default]
    pub deliveryid: u32,
    pub ondelete: Callback<MouseEvent>,
}

#[function_component(OrderCostItem)]
pub fn order_cost_item(props: &OrderCostItemProps) -> Html {
    let history = use_navigator().unwrap();

    let on_add_edit_view = {
        let props_label = props.label.clone();
        let history = history.clone();
        Callback::from(move |evt: MouseEvent| {
            evt.prevent_default();
            evt.stop_propagation();
            log::info!("On Add/Edit/View Called");
            if props_label == "Donation" {
                history.push(&AppRoutes::OrderDonations);
            } else {
                history.push(&AppRoutes::OrderProducts);
            }
        })
    };

    // If it is readonly and there isn't anything
    if props.amount.is_empty() && props.isreadonly {
        return html! {};
    }
    // If it isn't read only and we can add
    if props.amount.is_empty() && !props.isreadonly {
        return html! {
            <li class="list-group-item">
                {format!("Add {}", props.label)}
                <button class="btn btn-outline-info float-end order-edt-btn" onclick={on_add_edit_view}>
                    <i class="bi bi-plus-square" fill="currentColor"></i>
                </button>
            </li>
        };
    }

    let amount_label = format!("Amount: {}", to_money_str(Some(props.amount.as_str())));
    let (delivery_id, delivery_label) = if 0 == props.deliveryid {
        ("".to_string(), html! {})
    } else {
        (
            props.deliveryid.to_string(),
            html! {<><br/>{format!("To be delivered on: {}", get_delivery_date(&props.deliveryid))}</>},
        )
    };

    // if the order already exists...
    html! {
         //With Edit/Delete Button
        <li class="list-group-item">
            {amount_label}{delivery_label}
            if props.isreadonly {
                <button class="btn btn-outline-info float-end order-edt-btn"
                     data-deliveryid={delivery_id} onclick={on_add_edit_view}>
                    <i class="bi bi-eye" fill="currentColor"></i>
                </button>
            } else {
                <button class="btn btn-outline-danger mx-1 float-end order-del-btn"
                    data-deliveryid={delivery_id.clone()} onclick={props.ondelete.clone()}>
                    <i class="bi bi-trash" fill="currentColor"></i>
                </button>
                <button class="btn btn-outline-info float-end order-edt-btn"
                    data-deliveryid={delivery_id} onclick={on_add_edit_view}>
                    <i class="bi bi-pencil" fill="currentColor"></i>
                </button>
            }
        </li>
    }
}

/////////////////////////////////////////////////
/////////////////////////////////////////////////

#[function_component(HoodSelector)]
pub fn hood_selector() -> Html {
    let history = use_navigator().unwrap();
    if !is_active_order() {
        history.push(&AppRoutes::Home);
    }

    let order = use_state_eq(|| get_active_order().unwrap());
    // log::info!("Loading Order: {:#?}", &*order);

    let on_hood_warning = use_state_eq(|| "display: none;".to_owned());
    let on_hood_change = {
        let on_hood_warning = on_hood_warning.clone();
        Callback::from(move |evt: Event| {
            let hood_value = evt
                .target()
                .and_then(|t| t.dyn_into::<HtmlSelectElement>().ok());
            if let Some(v) = hood_value {
                let val = v.value();
                if val.starts_with("Out of Area") {
                    log::info!("Is Out Of Area");
                    on_hood_warning.set("display: block;".to_owned());
                    update_city_and_zip("", "", &gloo::utils::document());
                } else {
                    log::info!("Is Not Out Of Area");
                    on_hood_warning.set("display: none;".to_owned());
                    let (city, zipcode) = get_city_and_zip_from_neighborhood(&val).unwrap();
                    log::info!("Using Neighborhood City: {}, Zipcode: {}", &city, zipcode);
                    update_city_and_zip(&city, &zipcode.to_string(), &gloo::utils::document());
                }
            };
        })
    };

    let mut did_find_selected_hood = false;

    html! {
        <div class="form-floating col-md-4">
            <select class="form-control" id="formNeighborhood" onchange={on_hood_change} required=true>
                {
                    get_neighborhoods().iter().filter(|hood_info| hood_info.is_visible).map(|hood_info| {
                        let is_selected = {
                            match order.customer.neighborhood.as_ref() { Some(neighborhood) => {
                                &hood_info.name == neighborhood
                            } _ => {
                                false
                            }}
                        };
                        if !did_find_selected_hood && is_selected { did_find_selected_hood=true; }
                        html!{<option value={hood_info.name.clone()} selected={is_selected}>{hood_info.name.clone()}</option>}
                    }).collect::<Html>()
                }
                if !did_find_selected_hood {
                    if order.customer.neighborhood.is_none() {
                        <option value="" selected=true disabled=true hidden=true>
                            {"Select Neighborhood"}
                        </option>
                    } else {
                        <option value={order.customer.neighborhood.as_ref().unwrap().clone()}
                                selected=true disabled=true hidden=true>
                            {format!("{} (DNE. Report Issue)", order.customer.neighborhood.as_ref().unwrap())}
                        </option>
                    }
                }
            </select>
            <label for="formNeighborhood">
                {"Neighborhood"}<RequiredSmall/>
            </label>
            <small id="outOfHoodDisclaimer" style={(*on_hood_warning).clone()}>
                <i class="bi bi-exclamation-triangle-fill pe-1"></i>
                {"You are responsible for delivery of all out of area orders"}
                <i class="bi bi-exclamation-triangle-fill ps-1"></i>
            </small>
        </div>
    }
}

/////////////////////////////////////////////////
/////////////////////////////////////////////////

#[function_component(OrderFormFields)]
pub fn order_form_fields() -> Html {
    let history = use_navigator().unwrap();
    if !is_active_order() {
        history.push(&AppRoutes::Home);
        return html! {<div/>};
    }

    let is_admin = get_active_user().is_admin();
    let order = use_state_eq(|| get_active_order().unwrap());
    let is_order_readonly = order.is_readonly();
    // log::info!("Loading Order: {:#?}", &*order);

    let amount_due_str = use_state_eq(|| "$0.00".to_owned());
    let amount_collected_str = match order.amount_total_collected.as_ref() {
        None => "$0.00".to_string(),
        Some(v) => to_money_str(Some(v)),
    };

    let on_payment_input = {
        Callback::from(move |evt: InputEvent| {
            log::info!("On Payment Due");
            evt.prevent_default();
            evt.stop_propagation();

            let document = gloo::utils::document();
            let mut cash_amt_collected = get_cash_amount_collected(&document);
            if cash_amt_collected.is_some() {
                let new_amt = on_money_input_filter(cash_amt_collected.as_ref());
                if &new_amt != cash_amt_collected.as_ref().unwrap() {
                    cash_amt_collected = Some(new_amt);
                    set_cash_amount_collected(&document, cash_amt_collected.as_ref().unwrap());
                }
            }

            let mut check_amt_collected = get_check_amount_collected(&document);
            if check_amt_collected.is_some() {
                let new_amt = on_money_input_filter(check_amt_collected.as_ref());
                if &new_amt != check_amt_collected.as_ref().unwrap() {
                    check_amt_collected = Some(new_amt);
                    set_check_amount_collected(&document, check_amt_collected.as_ref().unwrap());
                }
            }

            let mut total_collected = Decimal::ZERO;
            if let Some(payment) = cash_amt_collected {
                total_collected = total_collected
                    .checked_add(Decimal::from_str(&payment).unwrap())
                    .unwrap();
            }
            if let Some(payment) = check_amt_collected {
                total_collected = total_collected
                    .checked_add(Decimal::from_str(&payment).unwrap())
                    .unwrap();
            }
            document
                .get_element_by_id("orderAmountPaid")
                .and_then(|t| t.dyn_into::<HtmlElement>().ok())
                .unwrap()
                .set_inner_text(&to_money_str(Some(&total_collected.to_string())));

            let collect_later_element = document
                .get_element_by_id("formCollectLater")
                .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
                .unwrap();
            if total_collected > Decimal::ZERO {
                if collect_later_element.checked() {
                    collect_later_element.set_checked(false);
                }
                collect_later_element.set_disabled(true);
            } else {
                collect_later_element.set_disabled(false);
            }
        })
    };

    let on_form_submission = {
        let history = history.clone();
        let on_form_submitted = move |_was_submitted_ok: bool| {
            let was_from_db = is_active_order_from_db();
            reset_active_order();
            if was_from_db {
                history.push(&AppRoutes::Reports);
            } else {
                history.push(&AppRoutes::Home);
            }
        };
        Callback::from(move |evt: SubmitEvent| {
            let on_form_submitted = on_form_submitted.clone();
            evt.prevent_default();
            evt.stop_propagation();
            log::info!("on_form_submission");

            let document = gloo::utils::document();

            disable_submit_button(&document, true, true);
            disable_cancel_button(&document, true);

            if !validate_order_form(&document) {
                log::info!("Form isn't valid refusing submission");
                disable_submit_button(&document, false, false);
                disable_cancel_button(&document, false);
            } else {
                // Send request
                wasm_bindgen_futures::spawn_local(async move {
                    log::info!("Submitting Order");
                    let rslt = submit_active_order().await;
                    disable_submit_button(&document, false, false);
                    disable_cancel_button(&document, false);
                    match rslt {
                        Err(err) => {
                            gloo::dialogs::alert(&format!("Failed to submit order: {:#?}", err));
                        }
                        _ => {
                            on_form_submitted(true);
                        }
                    }
                });
            }
        })
    };

    let on_cancel_order = {
        let history = history.clone();
        Callback::from(move |evt: MouseEvent| {
            evt.prevent_default();
            evt.stop_propagation();
            log::info!("on_cancel_order");
            let was_from_db = is_active_order_from_db();
            reset_active_order();
            if was_from_db {
                history.push(&AppRoutes::Reports);
            } else {
                history.push(&AppRoutes::Home);
            }
        })
    };

    let on_donations_delete = {
        let order = order.clone();
        Callback::from(move |evt: MouseEvent| {
            evt.prevent_default();
            evt.stop_propagation();
            let mut updated_order = get_active_order().unwrap();
            updated_order.clear_donations();
            update_active_order(updated_order.clone()).unwrap();
            order.set(updated_order);
        })
    };

    let on_geolocate = {
        // let order = order.clone();
        Callback::from(move |evt: MouseEvent| {
            evt.prevent_default();
            evt.stop_propagation();

            let document = gloo::utils::document();
            let elm = document
                .get_element_by_id("btnGeolocate")
                .and_then(|t| t.dyn_into::<Element>().ok())
                .unwrap();
            let _ = elm.class_list().add_1("btn-outline-danger");
            let _ = elm.class_list().remove_1("btn-outline-info");
            wasm_bindgen_futures::spawn_local(async move {
                log::info!("Making Geolocate call");
                if let Some(pos) = js::geolocate::get_current_position().await {
                    log::info!(
                        "Geo callback: {}",
                        serde_json::to_string_pretty(&pos).unwrap()
                    );
                    if let Ok(addr) =
                        get_address_from_lat_lng(pos.coords.latitude, pos.coords.longitude).await
                    {
                        log::info!(
                            "Geo address: {}",
                            serde_json::to_string_pretty(&addr).unwrap()
                        );
                        if addr.house_number.is_some() && addr.street.is_some() {
                            update_addr1(
                                format!("{} {}", addr.house_number.unwrap(), addr.street.unwrap()),
                                &document,
                            );
                        }
                    }
                }
                let _ = elm.class_list().add_1("btn-outline-info");
                let _ = elm.class_list().remove_1("btn-outline-danger");
            });
        })
    };

    let on_purchases_delete = {
        let order = order.clone();
        Callback::from(move |evt: MouseEvent| {
            evt.prevent_default();
            evt.stop_propagation();
            let mut updated_order = get_active_order().unwrap();
            updated_order.clear_purchases();
            update_active_order(updated_order.clone()).unwrap();
            order.set(updated_order);
        })
    };

    {
        let order = order.clone();
        use_effect(move || {
            let document = gloo::utils::document();
            update_order_amount_due_element(&order, &document);
            || {}
        });
    }

    let mut did_find_selected_order_owner = false;
    let amt_cash_paid = from_cloud_to_money_str(order.amount_cash_collected.clone());
    let amt_checks_paid = from_cloud_to_money_str(order.amount_checks_collected.clone());

    html! {
        <form class="needs-validation" id="newOrEditOrderForm" novalidate=true onsubmit={on_form_submission}>

            if is_admin {
                 <div class="row mb-2 g-2">
                     <div class="form-floating col-md-4">
                         <select class="form-control" id="formOrderOwner" >
                         {
                             get_users().keys().into_iter().map(|user_id| {
                                 let is_selected = user_id == &order.order_owner_id;
                                 if !did_find_selected_order_owner && is_selected { did_find_selected_order_owner=true; }
                                 html!{
                                     <option value={user_id.to_string()} selected={is_selected}>
                                         {get_username_from_id(user_id).unwrap_or(user_id.to_string())}
                                     </option>}
                             }).collect::<Html>()
                         }
                         if !did_find_selected_order_owner {
                             if order.order_owner_id.is_empty() {
                                 <option value="none" selected=true disabled=true hidden=true>{
                                     "Select Order Owner (DNE. Report Issue)"}
                                 </option>
                             } else {
                                 <option value={order.order_owner_id.clone()}
                                         selected=true disabled=true hidden=true>
                                     {format!("{} (DNE. Report Issue)", &order.order_owner_id)}
                                 </option>
                             }
                         }
                         </select>
                         <label for="formOrderOwner">{"Order Owner"}</label>
                     </div>

                     <div class="col-md-2">
                        <div class="form-check form-switch">
                            <input class="form-check-input order-vrfy-switch"
                                id="formIsVerified"
                                type="checkbox"
                                checked={order.is_verified.unwrap_or(false)}
                            />
                        </div>
                        <label for="formIsVerified">{"Verified"}</label>
                     </div>
                 </div>
            } else {
                <input
                    id="formIsVerified"
                    type="hidden"
                    checked={order.is_verified.unwrap_or(false)}
                />
            }

            <div class="row mb-2 g-2">
                <div class="form-floating col-md-6">
                    <input class="form-control" type="text" autocomplete="fr-new-cust-info" id="formCustomerName"
                           placeholder="Name" required=true
                           value={order.customer.name.clone()} />
                    <label for="formCustomerName">
                        {"Customer Name"}<RequiredSmall/>
                    </label>
                </div>
                <div class="form-floating col-md-2" id="formPhoneFloatDiv">
                    <input class="form-control" type="tel" autocomplete="fr-new-cust-info" id="formPhone"
                           pattern=r#"[0-9]{3}[\-\|.]?[0-9]{3}[\-\|.]?[0-9]{4}"#
                           placeholder="Phone" required=true
                           value={order.customer.phone.clone()} />
                    <label for="formPhone">
                        {"Phone"}
                        <small class="form-text text-muted ps-1">{"(xxx-xxx-xxxx)"}</small>
                        <RequiredSmall/>
                    </label>
                </div>
                <div class="form-floating col-md-4">
                    <input class="form-control" type="text" autocomplete="fr-new-cust-info" id="formEmail"
                           placeholder="Email"
                           value={order.customer.email.clone()}/>
                    <label for="formEmail">{"Email"}</label>
                </div>
            </div>

            <div class="row mb-2 g-2">
                <div class="input-group col">
                    <div class="input-group-prepend">
                        <button class="btn btn-outline-info order-edt-btn"
                            id="btnGeolocate"
                            onclick={on_geolocate}>
                            <i class="bi bi-geo" fill="currentColor"></i>
                        </button>
                    </div>
                    <div class="form-floating">
                        <input class="form-control" type="text" autocomplete="fr-new-cust-info" id="formAddr1"
                               placeholder="House Number/Street" required=true
                               value={order.customer.addr1.clone()} />
                        <label for="formAddr1">
                            {"House Number/Street"}<RequiredSmall/>
                        </label>
                    </div>
                </div>
                <div class="form-floating col">
                    <input class="form-control" type="text" autocomplete="fr-new-cust-info" id="formAddr2"
                           placeholder="Suite, etc..."
                           value={order.customer.addr2.clone()}/>
                    <label for="formAddr2">{"Suite, etc..."}</label>
                </div>
            </div>

            <div class="row mb-2 g-2">
                <HoodSelector/>
                <div class="form-floating col-md-3" id="formZipcodeFloatDiv">
                    <input class="form-control" type="number" autocomplete="fr-new-cust-info" id="formZipcode"
                           pattern="[0-9]{5}"
                           placeholder="Zipcode"
                           value={order.customer.zipcode.map(|v|v.to_string())}/>
                    <label for="formZipcode">
                        {"Zipcode"}
                    </label>
                </div>
                <div class="form-floating col-md-5" id="formCityFloatDiv">
                    <input class="form-control" type="text" autocomplete="fr-new-cust-info" id="formCity"
                           placeholder="City"
                           value={order.customer.city.clone()}/>
                    <label for="formCity">
                        {"City"}
                    </label>
                </div>
            </div>

            <div class="row mb-2 g-2">
                <div class="form-floating col-md-12">
                    <textarea class="form-control" id="formSpecialInstructions" rows="2"
                              value={order.special_instructions.clone()}>
                    </textarea>
                    <label for="formSpecialInstructions">{"Special Delivery Instructions"}</label>
                </div>
            </div>

            <div class="row mb-2 g-2">
                <div class="form-floating col-md-12">
                    <textarea class="form-control" id="formOrderComments" rows="2"
                              value={order.comments.clone()}>
                    </textarea>
                    <label for="formOrderComments">{"Order Comments"}</label>
                </div>
            </div>

            <div class="row mb-2 my-2 g-2" style="display: contents; border: 0;" >
                <div class="form-control" id="productList">
                    <ul class="list-group">
                        <OrderCostItem label="Donation"
                            isreadonly={is_order_readonly}
                            ondelete={on_donations_delete}
                            amount={from_cloud_to_money_str(order.amount_from_donations.clone()).unwrap_or("".to_string())}/>
                        <OrderCostItem label="Product Order"
                            deliveryid={order.delivery_id.unwrap_or(0)}
                            ondelete={on_purchases_delete}
                            isreadonly={is_order_readonly}
                            amount={from_cloud_to_money_str(order.amount_from_purchases.clone()).unwrap_or("".to_string())}/>
                    </ul>
                </div>
                <div class="invalid-feedback">
                {"*Either a donation or a product order is required"}
                </div>
            </div>

            <div class="mb-2 my-2 g-2 form-control" style="display: flex;" id="totalsFormRow">
                <div class="row">
                    <div class="col-md-2 d-none">
                        <label class="form-check-label" for="formCollectLater">{"Collect Later"}</label>
                        <div class="form-check form-switch">
                            <input class="form-check-input" type="checkbox" id="formCollectLater"
                                   checked={order.will_collect_money_later.unwrap_or(false)}  />
                        </div>
                    </div>
                    <div class="col-md-3">
                        <label for="formCashPaid">{"Total Cash Amount"}</label>
                        <div class="input-group">
                            <div class="input-group-prepend">
                                <span class="input-group-text">{"$"}</span>
                            </div>
                            <input class="form-control" type="number" min="0" step="any"
                                   autocomplete="fr-new-cust-info"
                                   id="formCashPaid" placeholder="0.00"
                                   oninput={on_payment_input.clone()}
                                   value={amt_cash_paid}/>
                        </div>
                    </div>
                    <div class="col-md-3">
                        <label for="formCheckPaid">{"Total Check Amount"}</label>
                        <div class="input-group">
                            <div class="input-group-prepend">
                                <span class="input-group-text">{"$"}</span>
                            </div>
                            <input class="form-control" type="number" min="0" step="any"
                                   autocomplete="fr-new-cust-info"
                                   id="formCheckPaid" placeholder="0.00"
                                   oninput={on_payment_input.clone()}
                                   value={amt_checks_paid}/>
                        </div>
                    </div>
                    <div class="col-md-4">
                        <label for="formCheckNumbers">{"Enter Check Numbers"}</label>
                        <input class="form-control" autocomplete="fr-new-cust-info"
                               id="formCheckNumbers" placeholder="Enter Check #s"
                               value={order.check_numbers.as_ref().map_or("".to_string(), |v|v.clone())}/>
                    </div>

                    <div class="row mb-2 my-2 g-2">
                        <span class="col-md-6">
                            {"Total Due:"}<div id="orderAmountDue" style="display: inline;" >{(*amount_due_str).clone()}</div>
                        </span>
                        <span class="col-md-6 g-2" aria-describedby="orderAmountPaidHelp">
                             {"Total Paid:"}
                            <div id="orderAmountPaid" style="display: inline;">
                                {amount_collected_str}
                            </div>
                        </span>
                    </div>
                </div>
            </div>

            <div class="invalid-feedback">
                {"*Must match total due or the check amount field is populated but there are no check numbers"}
            </div>

            <div class="pt-4">
                <button type="button" class="btn btn-primary" id="formOrderCancel" onclick={on_cancel_order}>
                    {"Cancel"}
                </button>
                if !is_order_readonly {
                  <button type="submit" class="btn btn-primary float-end" id="formOrderSubmit">
                      <span class="spinner-border spinner-border-sm me-1" role="status"
                      aria-hidden="true" id="formOrderSubmitSpinner" style="display: none;" />
                      {"Submit"}
                  </button>
                }
            </div>

        </form>
    }
}

/////////////////////////////////////////////////
/////////////////////////////////////////////////

#[function_component(OrderForm)]
pub fn order_form() -> Html {
    html! {
        <div class="col-xs-1 justify-content-center">
            <div class="card">
                <div class="card-body">
                    <h5 class="card-title">{"Customer Information"}</h5>
                    <OrderFormFields/>
                </div>
            </div>
        </div>
    }
}
