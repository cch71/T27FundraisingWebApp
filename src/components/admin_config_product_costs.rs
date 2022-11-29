use yew::prelude::*;
use web_sys::{
   MouseEvent, Element, HtmlElement, HtmlInputElement, InputEvent,
};
use crate::data_model::*;
use crate::bootstrap;
use std::rc::Rc;
use std::cell::RefCell;
use wasm_bindgen::JsCast;

thread_local! {
    static SELECTED_PRICE_BREAK: Rc<RefCell<Option<UseStateHandle<Option<(u32, String)>>>>> = Rc::new(RefCell::new(None));
}

/////////////////////////////////////////////////
/////////////////////////////////////////////////
type PriceBreakAddUpdateDlgCb = (u32, String);
/////////////////////////////////////////////////
//
#[derive(Properties, PartialEq, Clone, Debug)]
struct PriceBreakAddEditDlgProps {
    onaddorupdate: Callback<PriceBreakAddUpdateDlgCb>,
}

#[function_component(PriceBreakAddEditDlg)]
fn pricebreak_add_or_edit_dlg(props: &PriceBreakAddEditDlgProps) -> Html
{
    let price_break = use_state_eq(|| None);
    {
        let price_break = price_break.clone();
        SELECTED_PRICE_BREAK.with(|rc|{
            *rc.borrow_mut() = Some(price_break);
        });
    }

    let on_add_update = {
        let onaddorupdate = props.onaddorupdate.clone();
        move |_evt: MouseEvent| {
            let document = gloo::utils::document();
            let gt = document.get_element_by_id("formGt")
                .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
                .unwrap()
                .value()
                .parse::<u32>()
                .unwrap();
            let unit_price = document.get_element_by_id("formUnitPrice")
                .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
                .unwrap()
                .value();
            onaddorupdate.emit((gt, unit_price));
        }
    };

    let (is_new, gt, unit_price) = (*price_break)
        .as_ref()
        .map_or_else(
            ||(true, "".to_string(), "".to_string()),
            |(gt, unit_price)| (false, gt.to_string(), unit_price.clone()));

    html!{
        <div class="modal fade" id="pricebreakAddOrEditDlg"
             tabIndex="-1" role="dialog" aria-labelledby="pricebreakAddOrEditDlgTitle" aria-hidden="true">
            <div class="modal-dialog modal-dialog-centered" role="document">
                <div class="modal-content">
                    <div class="modal-header">
                        <h5 class="modal-title" id="pricebreakAddOrEditLongTitle">
                           {"Add/Edit Price Break"}
                        </h5>
                    </div>
                    <div class="modal-body">
                        <div class="container-sm">
                            <div class="row">
                                <div class="form-floating col-md">
                                    <input class="form-control" type="number" autocomplete="fr-new-gt" id="formGt"
                                        required=true
                                        readonly={!is_new}
                                        value={gt} />
                                        <label for="formGt">{"Greater Than"}</label>
                                </div>
                            </div>
                            <div class="row">
                                <div class="form-floating col-md">
                                    <input class="form-control" type="number" autocomplete="fr-new-unitprice" id="formUnitPrice"
                                        required=true
                                        value={unit_price} />
                                    <label for="formUnitPrice">{"Unit Price"}</label>
                                </div>
                            </div>
                        </div>
                    </div>
                    <div class="modal-footer">
                        <button type="button" class="btn btn-secondary" data-bs-dismiss="modal">{"Cancel"}</button>
                        <button type="button" class="btn btn-primary float-end" data-bs-dismiss="modal" onclick={on_add_update}>
                            {"Submit"}
                        </button>
                    </div>
                </div>
            </div>
        </div>
    }
}

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

/////////////////////////////////////////////////
//
fn get_selected_pricebreak(evt: MouseEvent) -> u32 {
    let btn_elm = evt.target()
        .and_then(|t| t.dyn_into::<Element>().ok())
        .and_then(|t| {
            if t.node_name() == "I" {
                t.parent_element()
            } else {
                Some(t)
            }
        }).unwrap();
    let elm = btn_elm.dyn_into::<HtmlElement>().ok().unwrap();

    elm.dataset().get("gt").unwrap().parse::<u32>().unwrap()
}

#[function_component(MulchCost)]
pub(crate) fn set_mulch_cost() -> Html
{
    use std::collections::BTreeMap;
    let product = get_products();
    let mulch_product_info = product.get("bags").unwrap().clone();

    let price_breaks = use_state(|| mulch_product_info
        .price_breaks
        .iter()
        .map(|v| (v.gt,v.unit_price.clone()))
        .collect::<BTreeMap<u32, String>>());
    let is_dirty = use_state_eq(|| false);

    let on_add_or_update_dlg_submit = {
        let is_dirty = is_dirty.clone();
        let price_breaks = price_breaks.clone();

        move | vals: PriceBreakAddUpdateDlgCb | {
            let  (gt, unit_price) = vals.to_owned();
            log::info!("Add/Updating Price Break {} - {}", gt, unit_price);
            let mut price_breaks_map = (*price_breaks).clone();
            price_breaks_map.insert(gt, unit_price);
            price_breaks.set(price_breaks_map);
            is_dirty.set(true);
        }
    };

    let on_delete = {
        let is_dirty = is_dirty.clone();
        let price_breaks = price_breaks.clone();
        move | evt: MouseEvent | {
            let gt = get_selected_pricebreak(evt);
            log::info!("Deleting Price Break {}", gt);
            let mut price_breaks_map = (*price_breaks).clone();
            price_breaks_map.remove(&gt);
            price_breaks.set(price_breaks_map);
            is_dirty.set(true);
        }
    };

    let on_add_pricebreak = {
        let is_dirty = is_dirty.clone();
        move | _evt: MouseEvent | {
            log::info!("Adding Pricebreak...");
            // Since we are adding we don't have a selected index
            SELECTED_PRICE_BREAK.with(|rc|{
                let selected_price_break = rc.borrow().as_ref().unwrap().clone();
                selected_price_break.set(None);
            });

            let dlg = bootstrap::get_modal_by_id("pricebreakAddOrEditDlg").unwrap();
            dlg.toggle();
        }
    };

    let on_edit = {
        let is_dirty = is_dirty.clone();
        let price_breaks = price_breaks.clone();
        move | evt: MouseEvent | {
            let gt = get_selected_pricebreak(evt);
            let unit_price = (*price_breaks).get(&gt).unwrap();
            log::info!("Editing Pricebreak: {}", &gt);
            SELECTED_PRICE_BREAK.with(|rc|{
                let selected_price_break = rc.borrow().as_ref().unwrap().clone();
                selected_price_break.set(Some((gt, unit_price.clone())));
            });
            let dlg = bootstrap::get_modal_by_id("pricebreakAddOrEditDlg").unwrap();
            dlg.toggle();
        }
    };

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

    html! {
        <div>
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
                            (*price_breaks).iter().map(|(gt, unit_price)|{
                                html!{<MulchPriceBreakLi
                                        gt={gt}
                                        unitprice={unit_price.clone()}
                                        ondelete={on_delete.clone()}
                                        onedit={on_edit.clone()} />}
                            }).collect::<Html>()
                        }
                        </ul>
                   </div>
                </div>
            </div>
            <PriceBreakAddEditDlg onaddorupdate={on_add_or_update_dlg_submit}/>
        </div>
    }
}

/////////////////////////////////////////////////
/////////////////////////////////////////////////
#[function_component(SpreadingCost)]
pub(crate) fn set_spreading_cost() -> Html
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

