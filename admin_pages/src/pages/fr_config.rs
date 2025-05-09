use crate::components::admin_config_deliveries::*;
use crate::components::admin_config_neighborhoods::*;
use crate::components::admin_config_product_costs::*;
use crate::components::admin_config_users::*;
use yew::prelude::*;

use data_model::*;
use wasm_bindgen::JsCast;
use web_sys::{HtmlButtonElement, HtmlElement, MouseEvent};

/////////////////////////////////////////////////
/////////////////////////////////////////////////

/////////////////////////////////////////////////
fn disable_reset_button(document: &web_sys::Document, value: bool) {
    if let Some(btn) = document
        .get_element_by_id("btnResetFrData")
        .and_then(|t| t.dyn_into::<HtmlButtonElement>().ok())
    {
        btn.set_disabled(value);
        let spinner_display = if value { "inline-block" } else { "none" };
        let _ = document
            .get_element_by_id("resetUserAndOrderDataSpinner")
            .and_then(|t| t.dyn_into::<HtmlElement>().ok())
            .unwrap()
            .style()
            .set_property("display", spinner_display);
    }
}

#[function_component(ResetOrders)]
fn reset_orders_database() -> Html {
    let on_reset_db = {
        move |_evt: MouseEvent| {
            let document = gloo::utils::document();
            disable_reset_button(&document, true);

            wasm_bindgen_futures::spawn_local(async move {
                let verify_phrase = "delete order and user data";
                let msg = format!(
                    "This will remove all order data from the system.\nIT IS DESTRUCTIVE!!!\nAre You Sure?\nType \"{}\" to delete",
                    verify_phrase
                );
                let do_reset =
                    gloo::dialogs::prompt(&msg, None).is_some_and(|v| v == verify_phrase);

                log::info!("Resetting Order Database: {}", do_reset);
                if do_reset {
                    log::info!("Resetting User and Order Data!!!!!!!!...");
                    if let Err(err) = reset_fundraiser().await {
                        gloo::dialogs::alert(&format!(
                            "Failed to reset fundraiser data: {:#?}",
                            err
                        ));
                    }
                }

                disable_reset_button(&document, false);
                let _ = gloo::utils::window().location().reload();
            });
        }
    };

    html! {
        <div class="card">
            <div class="card-body">
                <h5 class="card-title">
                    {"Clear Order And User Data"}
                </h5>
               <div class="row">
                  <button class="btn btn-danger" onclick={on_reset_db} id="btnResetFrData">
                      <span class="spinner-border spinner-border-sm me-1" role="status"
                      aria-hidden="true" id="resetUserAndOrderDataSpinner" style="display: none;" />
                      {"Reset!!"}
                  </button>
               </div>
            </div>
        </div>
    }
}

/////////////////////////////////////////////////
/////////////////////////////////////////////////

#[function_component(FrConfigEditor)]
pub fn fr_config() -> Html {
    let are_orders_created = use_state_eq(|| true);

    {
        let are_orders_created = are_orders_created.clone();
        wasm_bindgen_futures::spawn_local(async move {
            are_orders_created.set(have_orders_been_created().await.unwrap_or(true));
        });
    }

    html! {
        <div>
            <div class="row">
                <div class="col-xs-1 d-flex justify-content-center">
                    <h2>{"Fundraiser Configuration"}</h2>
                </div>
            </div>
            <div class="row">
                <div class="col-xs-1 d-flex justify-content-center">
                    if !(*are_orders_created) {
                        <MulchCost/>
                    }
                    <DeliveryUl/>
                </div>
            </div>
            <div class="row">
                <div class="col-xs-1 d-flex justify-content-center">
                    <NeighborhoodUl/>
                    <UsersUl/>
                </div>
            </div>
            <div class="row">
                <div class="col-xs-1 d-flex justify-content-center">
                    <ResetOrders/>
                </div>
            </div>
        </div>
    }
}
