//use yew::{function_component, html, Properties};
use yew::prelude::*;
use yew_router::prelude::*;
use std::fmt::Display;
use rusty_money::{Money, iso};
use crate::order_utils::*;
use wasm_bindgen::JsCast;
use web_sys::{Event, InputEvent, KeyboardEvent, MouseEvent, HtmlSelectElement};
use crate::AppRoutes;
use crate::currency_utils::*;

fn to_money_str(input: Option<&String>) -> String {
    input.map_or_else(
        || "".to_string(),
        |v| Money::from_str(v, iso::USD) .unwrap() .to_string()
    )
}

fn recalculate_total_paid(_evt: InputEvent) {
    log::info!("recalced total paid");
}

fn on_check_nums_key_press(_evt: KeyboardEvent) {
    log::info!("On on_check_nums_key_press");
}

#[function_component(RequiredSmall)]
pub fn required_small() -> Html
{
    html! {
        <small class="form-text text-muted ps-1">{"*required"}</small>
    }
}
#[derive(Properties, PartialEq)]
pub struct OrderCostItemProps {
    pub label: String,
    pub isreadonly: bool,
    pub amount: Option<String>,
    pub deliveryid: Option<u64>,

}
#[function_component(OrderCostItem)]
pub fn order_cost_item(props: &OrderCostItemProps) -> Html
{
    let history = use_history().unwrap();
    let on_add_edit = Callback::from(move |evt: MouseEvent| {
        evt.prevent_default();
        evt.stop_propagation();
        log::info!("OnAdd/Edit Called");
    });

    let on_del = Callback::from(move |evt: MouseEvent| {
        evt.prevent_default();
        evt.stop_propagation();
        log::info!("OnDel Called");
    });

    let on_view = {
        let props_label = props.label.clone();
        let history = history.clone();
        Callback::from(move |evt: MouseEvent| {
            evt.prevent_default();
            evt.stop_propagation();
            log::info!("OnView Called");
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
                <button class="btn btn-outline-info float-end order-edt-btn" onclick={on_add_edit}>
                    {"+"}
                </button>
            </li>
        };
    }

    let delivery_id = props.deliveryid.map_or_else(
        || "".to_string(),
        |v| format!("{}", v));

    // if the order already exists...
    html! {
         //With Edit/Delete Button
        <li class="list-group-item">
            {to_money_str(props.amount.as_ref())}
            if props.isreadonly {
                <button class="btn btn-outline-info float-end order-edt-btn"
                     data-deliveryid={delivery_id} onclick={on_view}>
                    <i class="bi bi-eye" fill="currentColor"></i>
                </button>
            } else {
                <button class="btn btn-outline-danger mx-1 float-end order-del-btn"
                    data-deliveryid={delivery_id.clone()} onclick={on_del}>
                    <i class="bi bi-trash" fill="currentColor"></i>
                </button>
                <button class="btn btn-outline-info float-end order-edt-btn"
                    data-deliveryid={delivery_id} onclick={on_add_edit}>
                    <i class="bi bi-pencil" fill="currentColor"></i>
                </button>
            }
        </li>
    }
}

#[function_component(OrderFormFields)]
pub fn order_form_fields() -> Html
{
    let is_admin = false;
    let order = MulchOrder{
        order_owner_id: "ablash".to_string(),
        customer: CustomerInfo {
            name: "John Stamose".to_string(),
            addr1: "202 lovers lane".to_string(),
            neighborhood: "Bear Valley".to_string(),
            phone: "455-234-4234".to_string(),
            ..Default::default()
        },
        amount_for_donations_collected: Some("200.24".to_string()),
        amount_total_collected: "200.24".to_string(),
        ..Default::default()
    };
    let user_ids = vec!["ablash", "craigh", "fradmin"];
    let is_order_readonly = true;

    let hoods = vec!["Bear Valley", "Out of Area", "Other..."];
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
    let amount_paid_str = use_state_eq(|| "$0.00".to_owned());


    html! {
        <form class="needs-validation" id="newOrEditOrderForm" novalidate=true >

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
                    <input class="form-control" type="text" autocomplete="fr-new-cust-info" id="formFirstName"
                           placeholder="First Name" required=true
                           value={order.customer.name} />
                    <label for="formCustomerName">
                        {"Customer Name"}<RequiredSmall/>
                    </label>
                </div>
            </div>

            <div class="row mb-2 g-2">
                <div class="form-floating col-md-6">
                    <input class="form-control" type="text" autocomplete="fr-new-cust-info" id="formAddr1"
                           placeholder="Address 1" required=true
                           value={order.customer.addr1} />
                    <label for="formAddr1">
                    {"Address 1"}<RequiredSmall/>
                    </label>
                </div>
                <div class="form-floating col-md-6">
                    <input class="form-control" type="text" autocomplete="fr-new-cust-info" id="formAddr2"
                           placeholder="Address 2"
                           value={order.customer.addr2}/>
                    <label for="formAddr2">{"Address 2"}</label>
                </div>
            </div>

            <div class="row mb-2 g-2">
                <div class="form-floating col-md-4">
                    <select class="form-control" id="formNeighborhood" onchange={on_hood_change}>
                        {
                            hoods.into_iter().map(|hood| {
                                let is_selected = hood == order.customer.neighborhood;
                                html!{<option key={hood} selected={is_selected}>{hood}</option>}
                            }).collect::<Html>()
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
                <div class="form-floating col-md-4" id="formPhoneFloatDiv">
                    <input class="form-control" type="tel" autocomplete="fr-new-cust-info" id="formPhone"
                           pattern="[0-9]{3}[-|.]{0,1}[0-9]{3}[-|.]{0,1}[0-9]{4}"
                           placeholder="Phone" required=true
                           value={order.customer.phone} />
                    <label for="formPhone">
                        {"Phone"}
                        <small class="form-text text-muted ps-1">{"(xxx-xxx-xxxx)"}</small>
                        <RequiredSmall/>
                    </label>
                </div>
                <div class="form-floating col-md-4">
                    <input class="form-control" type="text" autocomplete="fr-new-cust-info" id="formEmail"
                           placeholder="Email"
                           value={order.customer.email}/>
                    <label for="formEmail">{"Email"}</label>
                </div>
            </div>

            <div class="row mb-2 g-2">
                <div class="form-floating col-md-12">
                    <textarea class="form-control" id="formSpecialInstructions" rows="2"
                              value={order.special_instructions}>
                    </textarea>
                    <label for="formSpecialInstructions">{"Special Delivery Instructions"}</label>
                </div>
            </div>

            <div class="row mb-2 my-2 g-2" style="display: contents; border: 0;" >
                <div class="form-control" id="productList">
                    <ul class="list-group">
                        <OrderCostItem label="Donation" isreadonly={is_order_readonly} amount={order.amount_for_donations_collected.clone()}/>
                        <OrderCostItem label="Product Order"
                            deliveryid={order.delivery_id.clone()}
                            isreadonly={is_order_readonly}
                            amount={order.amount_for_purchases_collected.clone()}/>
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
                                   oninput={recalculate_total_paid} onkeypress={on_currency_field_key_press}
                                   value={to_money_str(order.amount_cash_collected.as_ref())}/>
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
                                   oninput={recalculate_total_paid} onkeypress={on_currency_field_key_press}
                                   value={to_money_str(order.amount_checks_collected.as_ref())}/>
                        </div>
                    </div>
                    <div class="col-md-4">
                        <label for="formCheckNumbers">{"Enter Check Numbers"}</label>
                        <input class="form-control" autocomplete="fr-new-cust-info"
                               id="formCheckNumbers" placeholder="Enter Check #s"
                               onkeypress={on_check_nums_key_press}
                               value={order.check_numbers.join(", ")}/>
                    </div>

                    <div class="row mb-2 my-2 g-2">
                        <span class="col-md-6">
                            {"Total Due:"}<div id="orderAmountDue" style="display: inline;" >{(*amount_due_str).clone()}</div>
                        </span>
                        <span class="col-md-6 g-2" aria-describedby="orderAmountPaidHelp">
                             {"Total Paid:"}<div id="orderAmountPaid" style="display: inline;">{(*amount_paid_str).clone()}</div>
                        </span>
                    </div>
                </div>
            </div>

            <div class="invalid-feedback">
                {"*Must match total due or the check amount field is populated but there are no check numbers"}
            </div>

            <div class="pt-4">
                <button type="button" class="btn btn-primary" id="formOrderCancel">
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


