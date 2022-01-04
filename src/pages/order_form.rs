//use yew::{function_component, html, Properties};
use yew::prelude::*;
use yew_router::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{
    Event, InputEvent, MouseEvent,
    Element, HtmlElement, HtmlSelectElement, HtmlInputElement, HtmlButtonElement,
};
use crate::AppRoutes;
use crate::{get_html_input_value, save_to_active_order};
use crate::currency_utils::*;
use crate::data_model::*;
use rust_decimal::prelude::*;
use rusty_money::{Money, iso};


/////////////////////////////////////////////////
///
fn set_html_input_value(id: &str, document: &web_sys::Document, value: &str) {
    document.get_element_by_id(id)
        .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
        .unwrap()
        .set_value(value);
}

/////////////////////////////////////////////////
///
fn get_cash_amount_collected(document: &web_sys::Document) -> Option<String> {
    get_html_input_value("formCashPaid", document)
}

/////////////////////////////////////////////////
///
fn get_check_amount_collected(document: &web_sys::Document) -> Option<String> {
    get_html_input_value("formCheckPaid", document)
}

/////////////////////////////////////////////////
///
fn set_cash_amount_collected(document: &web_sys::Document, value: &str) {
    set_html_input_value("formCashPaid", document, value)
}

/////////////////////////////////////////////////
///
fn set_check_amount_collected(document: &web_sys::Document, value: &str) {
    set_html_input_value("formCheckPaid", document, value)
}

/////////////////////////////////////////////////
///
fn disable_submit_button(document: &web_sys::Document, value: bool, with_spinner: bool) {
    document.get_element_by_id("formOrderSubmit")
        .and_then(|t| t.dyn_into::<HtmlButtonElement>().ok())
        .unwrap()
        .set_disabled(value);
    let spinner_display = if with_spinner { "inline-block" } else { "none" };
    let _ = document.get_element_by_id("formOrderSubmitSpinner")
        .and_then(|t| t.dyn_into::<HtmlElement>().ok())
        .unwrap()
        .style()
        .set_property("display", spinner_display);
}

/////////////////////////////////////////////////
///
fn disable_cancel_button(document: &web_sys::Document, value: bool) {
    document.get_element_by_id("formOrderCancel")
        .and_then(|t| t.dyn_into::<HtmlButtonElement>().ok())
        .unwrap()
        .set_disabled(value);
}

/////////////////////////////////////////////////
///
fn update_order_amount_due_element(order: &MulchOrder, document: &web_sys::Document) {
    let total_to_collect = order.get_total_to_collect();
    document.get_element_by_id("orderAmountDue")
        .and_then(|t| t.dyn_into::<HtmlElement>().ok())
        .unwrap()
        .set_inner_text(&Money::from_decimal(total_to_collect, iso::USD).to_string());
}

/////////////////////////////////////////////////
///
fn validate_order_form(document: &web_sys::Document) -> bool {
    save_to_active_order();
    let order = get_active_order().unwrap();
    let mut is_valid = true;

    let check_num_field_element = document.get_element_by_id("formCheckNumbers")
        .and_then(|t| t.dyn_into::<Element>().ok())
        .unwrap();
    let totals_form_row_element = document.get_element_by_id("totalsFormRow")
        .and_then(|t| t.dyn_into::<Element>().ok())
        .unwrap();
    match order.is_payment_valid() {
        true=>{
            let _ = totals_form_row_element.class_list().remove_1("is-invalid");
            let _ = check_num_field_element.class_list().remove_1("is-invalid");
        },
        false=>{
            let _ = totals_form_row_element.class_list().add_1("is-invalid");
            if !order.is_check_numbers_valid() {
                let _ = check_num_field_element.class_list().add_1("is-invalid");
            }
            is_valid = false;
        }
    };

    let form_node_list = document.query_selector("#newOrEditOrderForm")
        .ok()
        .flatten()
        .and_then(|t| t.dyn_into::<Element>().ok())
        .unwrap()
        .query_selector_all("[required]")
        .unwrap();
    for idx in 0..form_node_list.length() {
        log::info!("Going through Form List");
        if let Some(element) = form_node_list.item(idx).and_then(|t| t.dyn_into::<Element>().ok()) {
            let is_form_element_valid = {
                if let Some(form_element) = element.clone().dyn_into::<HtmlInputElement>().ok() {
                    form_element.check_validity()
                } else if let Some(form_element) = element.clone().dyn_into::<HtmlSelectElement>().ok() {
                    form_element.check_validity()
                } else {
                    false
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

    let product_list_element = document.get_element_by_id("productList")
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
fn required_small() -> Html
{
    html! {
        <small class="form-text text-muted ps-1">{"*required"}</small>
    }
}

/////////////////////////////////////////////////
/////////////////////////////////////////////////
#[derive(Properties, PartialEq, Clone, Debug)]
pub struct OrderCostItemProps {
    pub label: String,
    pub isreadonly: bool,
    pub amount: Option<String>,
    pub deliveryid: Option<u32>,
    pub ondelete: Callback<MouseEvent>,

}

#[function_component(OrderCostItem)]
pub fn order_cost_item(props: &OrderCostItemProps) -> Html
{
    let history = use_history().unwrap();

    let on_add_edit_view = {
        let props_label = props.label.clone();
        let history = history.clone();
        Callback::from(move |evt: MouseEvent| {
            evt.prevent_default();
            evt.stop_propagation();
            log::info!("On Add/Edit/View Called");
            if props_label=="Donation" {
                history.push(AppRoutes::OrderDonations);
            } else {
                history.push(AppRoutes::OrderProducts);
            }
        })
    };

    // If it is readonly and there isn't anything
    if props.amount.is_none() && props.isreadonly  {
        return html!{};
    }
    // If it isn't read only and we can add
    if props.amount.is_none() && !props.isreadonly  {
        return html! {
            <li class="list-group-item">
                {format!("Add {}", props.label)}
                <button class="btn btn-outline-info float-end order-edt-btn" onclick={on_add_edit_view}>
                    <i class="bi bi-plus-square" fill="currentColor"></i>
                </button>
            </li>
        };
    }

    let amount_label = format!("Amount: {}", to_money_str(props.amount.as_ref()));
    let mut delivery_label = html! {};
    let delivery_id = props.deliveryid.as_ref().map_or_else(
        || "".to_string(),
        |v| {
            delivery_label = html! {<><br/>{format!("To be delivered on: {}", get_delivery_date(v))}</>};
            v.to_string()
        });


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

#[function_component(OrderFormFields)]
pub fn order_form_fields() -> Html
{
    let history = use_history().unwrap();
    if !is_active_order() {
            history.push(AppRoutes::Home);
    }

    let is_admin = false;
    let user_ids = vec!["ablash", "craigh", "fradmin"];
    let order = use_state_eq(||get_active_order().unwrap());
    let is_order_readonly = order.is_readonly();
    // log::info!("Loading Order: {:#?}", &*order);

    let on_hood_warning = use_state_eq(|| "display: none;".to_owned());
    let on_hood_change = {
        let on_hood_warning = on_hood_warning.clone();
        Callback::from(move |evt: Event| {
            let hood_value = evt.target().and_then(|t| t.dyn_into::<HtmlSelectElement>().ok());
            hood_value.map(|v| {
                if v.value().starts_with("Out of Area") {
                    log::info!("Is Out Of Area");
                    on_hood_warning.set("display: block;".to_owned());
                } else {
                    log::info!("Is Not Out Of Area");
                    on_hood_warning.set("display: none;".to_owned());
                }
            });
        })
    };

    let amount_due_str = use_state_eq(|| "$0.00".to_owned());
    let amount_collected_str = match (*order).amount_total_collected.as_ref() {
        None=>"$0.00".to_string(),
        Some(v)=>to_money_str(Some(v)),
    };

    let on_payment_input = {
        Callback::from(move |evt: InputEvent| {
            log::info!("On Payment Due");
            evt.prevent_default();
            evt.stop_propagation();

            let document = gloo_utils::document();
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
                total_collected = total_collected.checked_add(Decimal::from_str(&payment).unwrap()).unwrap();
            }
            if let Some(payment) = check_amt_collected {
                total_collected = total_collected.checked_add(Decimal::from_str(&payment).unwrap()).unwrap();
            }
            document.get_element_by_id("orderAmountPaid")
                .and_then(|t| t.dyn_into::<HtmlElement>().ok())
                .unwrap()
                .set_inner_text(&to_money_str(Some(&total_collected.to_string())));

            let collect_later_element = document.get_element_by_id("formCollectLater")
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
        let on_form_submitted = Callback::once(move |_was_submitted_ok: bool| {
            let was_from_db =  is_active_order_from_db();
            reset_active_order();
            if was_from_db {
                history.push(AppRoutes::Reports);
            } else {
                history.push(AppRoutes::Home);
            }
        });
        Callback::from(move |evt: FocusEvent| {
            let on_form_submitted = on_form_submitted.clone();
            evt.prevent_default();
            evt.stop_propagation();
            log::info!("on_form_submission");

            let document = gloo_utils::document();

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
                    if let Err(err) = rslt {
                        gloo_dialogs::alert(&format!("Failed to submit order: {:#?}", err));
                    } else {
                        on_form_submitted.emit(true);
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
            let was_from_db =  is_active_order_from_db();
            reset_active_order();
            if was_from_db {
                history.push(AppRoutes::Reports);
            } else {
                history.push(AppRoutes::Home);
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
            let document = gloo_utils::document();
            update_order_amount_due_element(&order, &document);
            // disable_submit_button(
            //     !are_product_items_valid(&document) || get_delivery_id(&document).is_none()
            // );
            ||{}
        });
    }

    html! {
        <form class="needs-validation" id="newOrEditOrderForm" novalidate=true onsubmit={on_form_submission}>

            <div class="row mb-2 g-2" style={ if !is_admin { "display: none;"} else {"display:block;"} } >
                <div class="form-floating col-md-4">
                    <select class="form-control" id="formOrderOwner" >
                    {
                        user_ids.into_iter().map(|user_id| {
                            let is_selected = user_id == order.order_owner_id;
                            html!{<option key={user_id} selected={is_selected}>{user_id}</option>}
                        }).collect::<Html>()
                    }
                    </select>
                    <label for="formOrderOwner">{"Order Owner"}</label>
                </div>
            </div>

            <div class="row mb-2 g-2">
                <div class="form-floating col-md">
                    <input class="form-control" type="text" autocomplete="fr-new-cust-info" id="formCustomerName"
                           placeholder="Name" required=true
                           value={order.customer.name.clone()} />
                    <label for="formCustomerName">
                        {"Customer Name"}<RequiredSmall/>
                    </label>
                </div>
            </div>

            <div class="row mb-2 g-2">
                <div class="form-floating col-md-6">
                    <input class="form-control" type="text" autocomplete="fr-new-cust-info" id="formAddr1"
                           placeholder="Address 1" required=true
                           value={order.customer.addr1.clone()} />
                    <label for="formAddr1">
                    {"Address 1"}<RequiredSmall/>
                    </label>
                </div>
                <div class="form-floating col-md-6">
                    <input class="form-control" type="text" autocomplete="fr-new-cust-info" id="formAddr2"
                           placeholder="Address 2"
                           value={order.customer.addr2.clone()}/>
                    <label for="formAddr2">{"Address 2"}</label>
                </div>
            </div>

            <div class="row mb-2 g-2">
                <div class="form-floating col-md-4">
                    <select class="form-control" id="formNeighborhood" onchange={on_hood_change} required=true>
                        {
                            get_neighborhoods().iter().map(|hood_info| {
                                let is_selected = hood_info.name == order.customer.neighborhood;
                                html!{<option key={hood_info.name.clone()} selected={is_selected}>{hood_info.name.clone()}</option>}
                            }).collect::<Html>()
                        }
                        <option value="none" selected=true disabled=true hidden=true>{"Select Neighborhood"}</option>
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
                <div class="form-floating col-md-4" id="formPhoneFloatDiv">
                    <input class="form-control" type="tel" autocomplete="fr-new-cust-info" id="formPhone"
                           pattern="[0-9]{3}[-|.]{0,1}[0-9]{3}[-|.]{0,1}[0-9]{4}"
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
                <div class="form-floating col-md-12">
                    <textarea class="form-control" id="formSpecialInstructions" rows="2"
                              value={order.special_instructions.clone()}>
                    </textarea>
                    <label for="formSpecialInstructions">{"Special Delivery Instructions"}</label>
                </div>
            </div>

            <div class="row mb-2 my-2 g-2" style="display: contents; border: 0;" >
                <div class="form-control" id="productList">
                    <ul class="list-group">
                        <OrderCostItem label="Donation"
                            isreadonly={is_order_readonly}
                            ondelete={on_donations_delete}
                            amount={order.amount_from_donations.clone()}/>
                        <OrderCostItem label="Product Order"
                            deliveryid={order.delivery_id}
                            ondelete={on_purchases_delete}
                            isreadonly={is_order_readonly}
                            amount={order.amount_from_purchases.clone()}/>
                    </ul>
                </div>
                <div class="invalid-feedback">
                {"*Either a donation or a product order is required"}
                </div>
            </div>

            <div class="mb-2 my-2 g-2 form-control" style="display: flex;" id="totalsFormRow">
                <div class="row">
                    <div class="col-md-2">
                        <label class="form-check-label" for="formCollectLater">{"Collect Later"}</label>
                        <div class="form-check form-switch">
                            <input class="form-check-input" type="checkbox" id="formCollectLater"
                                   checked={order.will_collect_money_later}  />
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
                                   value={order.amount_cash_collected.as_ref().unwrap_or(&"".to_string()).clone()}/>
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
                                   value={order.amount_checks_collected.as_ref().unwrap_or(&"".to_string()).clone()}/>
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
pub fn order_form() -> Html
{
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


