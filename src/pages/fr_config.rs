use yew::prelude::*;
use crate::components::admin_config_deliveries::*;
use crate::components::admin_config_neighborhoods::*;

use web_sys::{
   MouseEvent, Element, HtmlElement, HtmlInputElement, InputEvent,
};
use crate::data_model::*;

/////////////////////////////////////////////////
/////////////////////////////////////////////////
#[function_component(SpreadingCost)]
fn set_spreading_cost() -> Html
{
    let is_dirty = use_state_eq(|| false);

    let spreading_cost = get_purchase_cost_for("spreading", 1);

    let on_save_spreading_cost = {
        let is_dirty = is_dirty.clone();
        move |_evt: MouseEvent| {
            log::info!("Saving Spreading Cost");
            is_dirty.set(false);
        }
    };

    let on_spreading_change = {
        let is_dirty = is_dirty.clone();
        move |_evt: InputEvent| {
            log::info!("Spreading Changed");
            is_dirty.set(true);
        }
    };

    html! {
        <div class="card">
            <div class="card-body">
                <h5 class="card-title">
                    {"Set Spreading Cost"}
                    if *is_dirty {
                        <button class="btn btn-primary" onclick={on_save_spreading_cost}>
                            <span class="spinner-border spinner-border-sm me-1" role="status"
                            aria-hidden="true" id="saveSpreadingConfigSpinner" style="display: none;" />
                            {"Save"}
                        </button>
                    }
                </h5>
               <div class="row">
                   <div class="form-floating col-md">
                       <input class="form-control" type="number" autocomplete="fr-new-spreading" id="formSpreading"
                           required=true
                           oninput={on_spreading_change}
                           value={spreading_cost} />

                           <label for="formSpreading">{"Spreading Cost Per Bag"}</label>
                   </div>
               </div>
            </div>
        </div>
    }
}

/////////////////////////////////////////////////
/////////////////////////////////////////////////
#[function_component(ResetOrders)]
fn reset_orders_database() -> Html
{
    let on_reset_db = {
        move |_evt: MouseEvent| {
            let do_reset_resp = gloo::dialogs::prompt(
                "This will remove all order data from the system.\nIT IS DESTRUCTIVE!!!\nAre You Sure?\nType \"delete order data\" to delete",
                None);
            log::info!("Resetting Order Database: {}", do_reset_resp.map_or(false, |v| v=="delete order data"));
        }
    };

    html! {
        <div class="card">
            <div class="card-body">
                <h5 class="card-title">
                    {"Clear Order Data"}
                </h5>
               <div class="row">
                  <button class="btn btn-primary" onclick={on_reset_db}>
                      <span class="spinner-border spinner-border-sm me-1" role="status"
                      aria-hidden="true" id="resetOrderTableSpinner" style="display: none;" />
                      {"Save"}
                  </button>
               </div>
            </div>
        </div>
    }
}

/////////////////////////////////////////////////
/////////////////////////////////////////////////

#[function_component(FrConfig)]
pub fn fr_config() -> Html
{
    html! {
        <div>
            <div class="row">
                <div class="col-xs-1 d-flex justify-content-center">
                    <h2>{"Fundraiser Configuration"}</h2>
                </div>
            </div>
            <div class="row">
                <div class="col-xs-1 d-flex justify-content-center">
                    <SpreadingCost/>
                    <DeliveryUl/>
                    <ResetOrders/>
                </div>
            </div>
            <div class="row">
                <div class="col-xs-1 d-flex justify-content-center">
                    <NeighborhoodUl/>
                </div>
            </div>
        </div>
    }
}
