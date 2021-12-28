use yew::prelude::*;
use yew_router::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{InputEvent, MouseEvent, FocusEvent, HtmlInputElement};
use crate::AppRoutes;
use crate::currency_utils::*;
use crate::order_utils::*;

#[function_component(OrderDonations)]
pub fn order_donations() -> Html
{
    let order = get_active_order().unwrap();
    let is_order_readonly = order.is_readonly();
    let history = use_history().unwrap();

    let on_form_submission = {
        let history = history.clone();
        let mut updated_order = order.clone();
        Callback::once(move |evt: FocusEvent| {
            evt.prevent_default();
            evt.stop_propagation();
            log::info!("on_form_submission");
            let document = web_sys::window().unwrap().document().unwrap();
            let donation_amt = document.get_element_by_id("formDonationAmount")
                .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
                .unwrap()
                .value();
            if 0==donation_amt.len() {
                updated_order.amount_for_donations_collected = None;
            } else {
                updated_order.amount_for_donations_collected = Some(donation_amt);
            }
            //  const amountDue = currency((document.getElementById('formDonationAmount') as HTMLInputElement).value);
            //  if (amountDue) {
            //      currentOrder['donation'] = amountDue;
            //  } else {
            //      delete currentOrder['donation'];
            //  }
            //  navigate('/order_step_1/');
            update_active_order(updated_order).unwrap();
            history.push(AppRoutes::OrderForm);
        })
    };

    let on_cancel_item = {
        let history = history.clone();
        Callback::from(move |evt: MouseEvent| {
            evt.prevent_default();
            evt.stop_propagation();
            log::info!("on_cancel_item");
            history.push(AppRoutes::OrderForm);
        })
    };

    let does_submit_get_enabled = Callback::from(move |evt: InputEvent| {
        evt.prevent_default();
        evt.stop_propagation();
        log::info!("does_submit_get_enabled");
       //  const origAmt = (document.getElementById('formDonationAmount') as HTMLInputElement).value;
       //  const [amt, isChanged] = moneyFloor(origAmt);
       //  if (isChanged) {
       //      (document.getElementById('formDonationAmount') as HTMLInputElement).value = amt.toString();
       //  }
       //  if (event.currentTarget.value) {
       //      (document.getElementById('formDonationSubmit') as HTMLButtonElement).disabled = false;
       //  } else {
       //      (document.getElementById('formDonationSubmit') as HTMLButtonElement).disabled = true;
       //  }
    });

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
                                       value={order.amount_for_donations_collected.unwrap_or("".to_string())}
                                       placeholder="0.00"
                                       oninput={does_submit_get_enabled} onkeypress={on_currency_field_key_press}/>
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
