use yew::prelude::*;
use crate::components::admin_config_deliveries::*;
use crate::components::admin_config_neighborhoods::*;

use web_sys::{
   MouseEvent, Element, HtmlElement, HtmlInputElement, InputEvent,
};
use crate::data_model::*;

/////////////////////////////////////////////////
/////////////////////////////////////////////////
#[derive(Properties, PartialEq, Clone, Debug)]
struct MulchPriceBreakLiProps {
    gt: u32,
    unitprice: String,
    onedit: Callback<MouseEvent>,
    ondelete: Callback<MouseEvent>,
}

#[function_component(MulchPriceBreakLi)]
fn mulch_pricebreak_item(props: &MulchPriceBreakLiProps) -> Html
{
    html! {
        <li class="list-group-item d-flex justify-content-between">
            <div>
                <div class="mb-1">{format!("Unit Price: {}", &props.unitprice)}</div>
                <small class="text-muted">{format!("Greater Than: {}", props.gt)}</small>
            </div>
            <div class="float-end">
                <button class="btn btn-outline-danger mx-1 float-end order-del-btn"
                    data-gt={props.gt.to_string()} onclick={props.ondelete.clone()}>
                    <i class="bi bi-trash" fill="currentColor"></i>
                </button>
                <button class="btn btn-outline-info float-end order-edt-btn"
                    data-gt={props.gt.to_string()} onclick={props.onedit.clone()}>
                    <i class="bi bi-pencil" fill="currentColor"></i>
                </button>
            </div>
        </li>
    }
}

#[function_component(MulchCost)]
fn set_mulch_cost() -> Html
{
    let is_dirty = use_state_eq(|| false);


    let on_save_mulch_cost = {
        let is_dirty = is_dirty.clone();
        move |_evt: MouseEvent| {
            log::info!("Saving Spreading Cost");
            is_dirty.set(false);
        }
    };

    let on_mulch_change = {
        let is_dirty = is_dirty.clone();
        move |_evt: InputEvent| {
            log::info!("Spreading Changed");
            is_dirty.set(true);
        }
    };

    let on_delete = {
        let is_dirty = is_dirty.clone();
        move | evt: MouseEvent | {
            log::info!("Deleting Pricebreak: ");
            is_dirty.set(true);
        }
    };

    let on_add_pricebreak = {
        let is_dirty = is_dirty.clone();
        move | _evt: MouseEvent | {
            log::info!("Adding Pricebreak: ");
            is_dirty.set(true);
        }
    };

    let on_edit = {
        let is_dirty = is_dirty.clone();
        move | evt: MouseEvent | {
            log::info!("Editing Pricebreak: ");
            is_dirty.set(true);
        }
    };

    let products = get_products();
    let mulch_product_info = products.get("bags").unwrap().clone();

    html! {
        <div class="card">
            <div class="card-body">
                <h5 class="card-title">
                    {"Set Mulch Cost"}
                    if *is_dirty {
                        <button class="btn btn-primary" onclick={on_save_mulch_cost}>
                            <span class="spinner-border spinner-border-sm me-1" role="status"
                            aria-hidden="true" id="saveSpreadingConfigSpinner" style="display: none;" />
                            {"Save"}
                        </button>
                    }
                </h5>
               <div class="row">
                   <div class="form-floating col-md">
                       <input class="form-control" type="number" autocomplete="fr-new-mulch-cost" id="formMulchCost"
                           required=true
                           oninput={on_mulch_change.clone()}
                           value={mulch_product_info.unit_price.clone()} />

                           <label for="formMulchCost">{"Base Mulch Cost Per Bag"}</label>
                   </div>
               </div>
               <div class="row">
                   <div class="form-floating col-md">
                       <input class="form-control" type="number" autocomplete="fr-new-mulch-units" id="formMulchMinUnits"
                           required=true
                           oninput={on_mulch_change.clone()}
                           value={mulch_product_info.min_units.to_string()} />
                           <label for="formMulchMinUnits">{"Min Bags"}</label>
                   </div>
               </div>
               <div class="row">
                    {"Price Breaks"}
                    <button class="btn btn-outline-info float-end order-edt-btn" onclick={on_add_pricebreak}>
                        <i class="bi bi-plus-square" fill="currentColor"></i>
                    </button>
                    <ul class="list-group">
                    {
                        mulch_product_info.price_breaks.iter().map(|v|{
                            html!{<MulchPriceBreakLi
                                    gt={v.gt}
                                    unitprice={v.unit_price.clone()}
                                    ondelete={on_delete.clone()}
                                    onedit={on_edit.clone()} />}
                        }).collect::<Html>()
                    }
                    </ul>
               </div>
            </div>
        </div>
    }
}

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
                    <MulchCost/>
                    <SpreadingCost/>
                    <DeliveryUl/>
                </div>
            </div>
            <div class="row">
                <div class="col-xs-1 d-flex justify-content-center">
                    <NeighborhoodUl/>
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
