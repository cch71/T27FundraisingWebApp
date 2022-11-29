use yew::prelude::*;
use crate::components::admin_config_deliveries::*;
use crate::components::admin_config_neighborhoods::*;
use crate::components::admin_config_product_costs::*;
use crate::components::admin_config_users::*;

use web_sys::{
   MouseEvent,
};
//use crate::data_model::*;

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
                    <MulchCost/>
                    <SpreadingCost/>
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
