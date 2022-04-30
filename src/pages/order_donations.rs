use yew::prelude::*;
use yew_router::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{InputEvent, MouseEvent, FocusEvent, HtmlInputElement, HtmlButtonElement};
use crate::AppRoutes;
use crate::currency_utils::*;
use crate::data_model::*;

/////////////////////////////////////////////////
///
fn get_donation_amount() -> Option<String> {
    let document = gloo_utils::document();
    let donation_amt = document.get_element_by_id("formDonationAmount")
        .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
        .unwrap()
        .value();
    if 0==donation_amt.len() {
        None
    } else {
        Some(donation_amt)
    }
}

/////////////////////////////////////////////////
///
fn set_donation_amount(value: &str) {
    let document = gloo_utils::document();
    document.get_element_by_id("formDonationAmount")
        .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
        .unwrap()
        .set_value(value);
}

/////////////////////////////////////////////////
///
fn disable_submit_button(value: bool) {
    if let Some(btn) = web_sys::window().unwrap()
        .document().unwrap()
        .get_element_by_id("formDonationSubmit")
        .and_then(|t| t.dyn_into::<HtmlButtonElement>().ok())
    {
        btn.set_disabled(value);
    }
}

/////////////////////////////////////////////////
/////////////////////////////////////////////////
#[function_component(OrderDonations)]
pub fn order_donations() -> Html
{
    let history = use_history().unwrap();
    if !is_active_order() {
            history.push(AppRoutes::Home);
    }
    let order = get_active_order().unwrap();
    let is_order_readonly = order.is_readonly();

    let on_form_submission = {
        let history = history.clone();
        let mut updated_order = order.clone();
        Callback::once(move |evt: FocusEvent| {
            evt.prevent_default();
            evt.stop_propagation();
            // log::info!("on_form_submission");
            let mut donation_amt = get_donation_amount();
            if donation_amt.is_some() {
                donation_amt = Some(to_money_str_no_symbol(donation_amt.as_ref()));
                let dec_donation_amt = parse_money_str_as_decimal(donation_amt.as_ref().unwrap()).unwrap();
                updated_order.set_donations(dec_donation_amt.to_string());
            } else {
                updated_order.clear_donations();
            }
            update_active_order(updated_order).unwrap();
            history.push(AppRoutes::OrderForm);
        })
    };

    let on_cancel_item = {
        let history = history.clone();
        Callback::from(move |evt: MouseEvent| {
            evt.prevent_default();
            evt.stop_propagation();
            //log::info!("on_cancel_item");
            history.push(AppRoutes::OrderForm);
        })
    };

    let do_form_validation = {
        let order = order.clone();
        Callback::from(move |evt: InputEvent| {
            evt.prevent_default();
            evt.stop_propagation();
            log::info!("do_form_validation");

            let mut donation_amt = get_donation_amount();

            if donation_amt.is_some() {
                let new_donation_amt = on_money_input_filter(donation_amt.as_ref());
                log::info!("New Donation: {} Old Donation: {}", new_donation_amt, donation_amt.as_ref().unwrap());
                if &new_donation_amt != donation_amt.as_ref().unwrap() {

                    donation_amt = Some(new_donation_amt);
                    set_donation_amount(donation_amt.as_ref().unwrap());
                }
            }

            disable_submit_button(order.amount_from_donations == donation_amt);
        })
    };

    {
        let order = order.clone();
        use_effect(move || {
            let donation_amt = get_donation_amount();
            disable_submit_button(order.amount_from_donations == donation_amt);
            ||{}
        });
    }

    html! {
        <div class="col-xs-1 justify-content-center">
            <div class="card">
                <div class="card-body">
                    <h5 class="card-title">{"Donations"}</h5>
                    <form onsubmit={on_form_submission}>
                        <div class="row col-sm-12">
                            <label for="formDonationAmount">{"Donation"}</label>
                            <div class="input-group mb-3">
                                <div class="input-group-prepend">
                                    <span class="input-group-text">{"$"}</span>
                                </div>
                                <input type="number" min="0" step="any" class="form-control" id="formDonationAmount"
                                       value={order.amount_from_donations.unwrap_or("".to_string())}
                                       placeholder="0.00"
                                       oninput={do_form_validation} />
                            </div>
                        </div>

                        <button type="button" class="btn btn-primary my-2" onclick={on_cancel_item}>
                            {"Cancel"}
                        </button>
                        if !is_order_readonly {
                            <button type="submit" class="btn btn-primary my-2 float-end" id="formDonationSubmit">
                                {"Submit"}
                            </button>
                        }
                    </form>
                </div>
            </div>
        </div>
    }
}
