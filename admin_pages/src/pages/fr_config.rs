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
                <ul class="nav nav-tabs" role="tablist">
                    // col-xs-1 d-flex justify-content-center
                    <li class="nav-item" role="presentation">
                        <button 
                            class="nav-link active" 
                            id="home-tab"
                            data-bs-toggle="tab"
                            data-bs-target="#deliveries-tab-pane"
                            type="button" role="tab"
                            aria-controls="deliveries-tab-pane"
                            aria-selected="true">
                                {"Deliveries"}
                        </button>
                    </li>
                    <li class="nav-item" role="presentation">
                        <button 
                            class="nav-link" 
                            id="products-tab"
                            data-bs-toggle="tab"
                            data-bs-target="#products-tab-pane"
                            type="button" role="tab"
                            aria-controls="products-tab-pane"
                            aria-selected="false"
                            disabled={*are_orders_created}>
                                {"Products"}
                        </button>
                    </li>
                    <li class="nav-item" role="presentation">
                        <button 
                            class="nav-link" 
                            id="neighborhoods-tab"
                            data-bs-toggle="tab"
                            data-bs-target="#neighborhoods-tab-pane"
                            type="button" role="tab"
                            aria-controls="neighborhoods-tab-pane"
                            aria-selected="false">
                                {"Neighborhoods"}
                        </button>
                    </li>
                    <li class="nav-item" role="presentation">
                        <button 
                            class="nav-link" 
                            id="users-tab"
                            data-bs-toggle="tab"
                            data-bs-target="#users-tab-pane"
                            type="button" role="tab"
                            aria-controls="users-tab-pane"
                            aria-selected="false">
                                {"Users"}
                        </button>
                    </li>
                    <li class="nav-item" role="presentation">
                        <button 
                            class="nav-link" 
                            id="reset-tab"
                            data-bs-toggle="tab"
                            data-bs-target="#reset-tab-pane"
                            type="button" role="tab"
                            aria-controls="reset-tab-pane"
                            aria-selected="false">
                                {"Reset"}
                        </button>
                    </li>
                </ul>
                    <div class="tab-content" id="myTabContent">
                        <div class="tab-pane fade show active" id="deliveries-tab-pane" role="tabpanel" aria-labelledby="deliveries-tab" tabindex="0">
                            <div class="row mt-2">
                                <div class="col-xs-1 d-flex justify-content-center">
                                    <DeliveryUl/>
                                </div>
                            </div>
                        </div>
                        <div class="tab-pane fade" id="products-tab-pane" role="tabpanel" aria-labelledby="products-tab" tabindex="0">
                            <div class="row mt-2">
                                <div class="col-xs-1 d-flex justify-content-center">
                                    if !(*are_orders_created) {
                                        <MulchCost/>
                                    }
                                </div>
                            </div>
                        </div>
                        <div class="tab-pane fade" id="neighborhoods-tab-pane" role="tabpanel" aria-labelledby="neighborhoods-tab" tabindex="0">
                            <div class="row mt-2">
                                <div class="col-xs-1 d-flex justify-content-center">
                                    <NeighborhoodUl/>
                                </div>
                            </div>
                        </div>
                        <div class="tab-pane fade" id="users-tab-pane" role="tabpanel" aria-labelledby="users-tab" tabindex="0">
                            <div class="row mt-2">
                                <div class="col-xs-1 d-flex justify-content-center">
                                    <UsersUl/>
                                </div>
                            </div>
                        </div>
                        <div class="tab-pane fade" id="reset-tab-pane" role="tabpanel" aria-labelledby="reset-tab" tabindex="0">
                            <div class="row mt-2">
                                <div class="col-xs-1 d-flex justify-content-center">
                                    <ResetOrders/>
                                </div>
                            </div>
                        </div>
                    </div>
            </div>
        </div>
    }
}
