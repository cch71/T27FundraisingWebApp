use data_model::*;
use js::bootstrap;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::JsCast;
use web_sys::{Element, HtmlButtonElement, HtmlElement, HtmlInputElement, MouseEvent};
use yew::prelude::*;

#[derive(PartialEq, Clone, Debug, Default)]
struct SelectedDeliveryInfo {
    delivery_id_str: String,
    delivery_date_str: String,
    cutoff_date_str: String,
}

thread_local! {
    static SELECTED_DELIVERY: Rc<RefCell<Option<UseStateHandle<SelectedDeliveryInfo>>>> = Rc::new(RefCell::new(None));
}

/////////////////////////////////////////////////
/////////////////////////////////////////////////
type DeliveryDlgAddOrUpdateCb = (u32, String, String);
/////////////////////////////////////////////////
//
#[derive(Properties, PartialEq, Clone, Debug)]
struct DeliveryAddEditDlgProps {
    onaddorupdate: Callback<DeliveryDlgAddOrUpdateCb>,
}

#[function_component(DeliveryAddEditDlg)]
fn delivery_add_or_edit_dlg(props: &DeliveryAddEditDlgProps) -> Html {
    //Tuple of Devlivery ID, Delivery Data, Cutoff Date
    let delivery_info = use_state_eq(SelectedDeliveryInfo::default);
    // let delivery_date_str = use_state_eq(|| "".to_string());
    // let cutoff_date_str = use_state_eq(|| "".to_string());

    {
        // This is just to initialize it with the state value so we can trigger it later
        let delivery_info = delivery_info.clone();
        SELECTED_DELIVERY.with(|selected_delivery_rc| {
            *selected_delivery_rc.borrow_mut() = Some(delivery_info);
        });
    }

    let on_add_update = {
        let delivery_info = delivery_info.clone();
        let onaddorupdate = props.onaddorupdate.clone();
        move |_evt: MouseEvent| {
            let document = gloo::utils::document();
            let delivery_date = document
                .get_element_by_id("formDeliveryDate")
                .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
                .unwrap()
                .value();
            let order_cutoff_date = document
                .get_element_by_id("formOrderCutoffDate")
                .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
                .unwrap()
                .value();
            onaddorupdate.emit((
                delivery_info.delivery_id_str.parse::<u32>().unwrap(),
                delivery_date,
                order_cutoff_date,
            ));
        }
    };

    // log::info!("Cutoff Date String: {}", &*cutoff_date_str);
    html! {
        <div class="modal fade" id="deliveryAddOrEditDlg"
             tabIndex="-1" role="dialog" aria-labelledby="deliveryAddOrEditDlgTitle" aria-hidden="true">
            <div class="modal-dialog modal-dialog-centered" role="document">
                <div class="modal-content">
                    <div class="modal-header">
                        <h5 class="modal-title" id="deliveryAddOrEditLongTitle">
                           {"Add/Edit Delivery Date"}
                        </h5>
                    </div>
                    <div class="modal-body">
                        <div class="container-sm">
                            <div class="row">
                                <div class="form-floating col-md">
                                    <div>{format!("Delivery ID: {}", &delivery_info.delivery_id_str)}</div>
                                </div>
                            </div>
                            <div class="row mb-2">
                                <div class="col-md">
                                    <div class="form-floating">
                                        <input class="form-control" type="date" autocomplete="fr-order-cutoff-date" id="formOrderCutoffDate"
                                            required=true
                                            value={delivery_info.cutoff_date_str.clone()} />
                                        <label for="formOrderCutoffDate">{"New Order Cutoff Date"}</label>
                                    </div>
                                </div>
                            </div>
                            <div class="row">
                                <div class="col-md">
                                    <div class="form-floating">
                                        <input class="form-control" type="date" autocomplete="fr-new-delivery-date" id="formDeliveryDate"
                                            required=true
                                            value={delivery_info.delivery_date_str.clone()} />
                                            <label for="formDeliveryDate">{"Delivery Date"}</label>
                                    </div>
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
#[derive(Properties, PartialEq, Clone, Debug)]
struct DeliveryLiProps {
    deliveryid: u32,
    deliverydate: String,
    newordercutoff: String,
    onedit: Callback<MouseEvent>,
    ondelete: Callback<MouseEvent>,
}

/////////////////////////////////////////////////
#[function_component(DeliveryLi)]
fn delivery_item(props: &DeliveryLiProps) -> Html {
    html! {
        <li class="list-group-item d-flex justify-content-between">
            <div>
                <div class="d-flex justify-content-between">
                    <div class="mb-1">{format!("Delivery Date: {}", &props.deliverydate)}</div>
                    <small class="text-muted mx-2">{props.deliveryid.to_string()}</small>
                </div>
                <small class="text-muted">{format!("New Order Cutoff: {}", &props.newordercutoff)}</small>
            </div>
            <div class="float-end">
                <button class="btn btn-outline-danger mx-1 float-end order-del-btn"
                    data-deliveryid={props.deliveryid.to_string()} onclick={props.ondelete.clone()}>
                    <i class="bi bi-trash" fill="currentColor"></i>
                </button>
                <button class="btn btn-outline-info float-end order-edt-btn"
                    data-deliveryid={props.deliveryid.to_string()} onclick={props.onedit.clone()}>
                    <i class="bi bi-pencil" fill="currentColor"></i>
                </button>
            </div>
        </li>
    }
}

/////////////////////////////////////////////////
fn get_delivery_id(evt: MouseEvent) -> u32 {
    let btn_elm = evt
        .target()
        .and_then(|t| t.dyn_into::<Element>().ok())
        .and_then(|t| {
            // log::info!("Node Name: {}", t.node_name());
            if t.node_name() == "I" {
                t.parent_element()
            } else {
                Some(t)
            }
        })
        .unwrap();
    let elm = btn_elm.dyn_into::<HtmlElement>().ok().unwrap();

    elm.dataset()
        .get("deliveryid")
        .unwrap()
        .parse::<u32>()
        .unwrap()
}

/////////////////////////////////////////////////
fn disable_save_button(document: &web_sys::Document, value: bool, with_spinner: bool) {
    if let Some(btn) = document
        .get_element_by_id("btnSaveDeliveries")
        .and_then(|t| t.dyn_into::<HtmlButtonElement>().ok())
    {
        btn.set_disabled(value);
        let spinner_display = if with_spinner { "inline-block" } else { "none" };
        let _ = document
            .get_element_by_id("saveDeliveryConfigSpinner")
            .and_then(|t| t.dyn_into::<HtmlElement>().ok())
            .unwrap()
            .style()
            .set_property("display", spinner_display);
    }
}

/////////////////////////////////////////////////
#[function_component(DeliveryUl)]
pub(crate) fn delivery_list() -> Html {
    let deliveries = use_state(|| (*get_deliveries()).clone());
    let is_dirty = use_state_eq(|| false);

    let on_add_or_update_dlg_submit = {
        let is_dirty = is_dirty.clone();
        let deliveries = deliveries.clone();
        move |vals: DeliveryDlgAddOrUpdateCb| {
            let (delivery_id, delivery_date, cutoff_date) = vals.to_owned();
            log::info!(
                "Add/Updating Delivery {} - {} - {}",
                delivery_id,
                delivery_date,
                cutoff_date
            );
            let delivery_info = DeliveryInfo::new_from_admin(delivery_date, cutoff_date);
            let mut delivery_map = (*deliveries).clone();
            delivery_map.insert(delivery_id, delivery_info);
            deliveries.set(delivery_map);
            is_dirty.set(true);
        }
    };

    let on_delete = {
        let deliveries = deliveries.clone();
        let is_dirty = is_dirty.clone();
        move |evt: MouseEvent| {
            let delivery_id = get_delivery_id(evt);
            let mut delivery_map = (*deliveries).clone();
            log::info!("Deleting ID: {}", delivery_id);
            delivery_map.remove(&delivery_id);
            deliveries.set(delivery_map);
            is_dirty.set(true);
        }
    };

    let on_add_delivery = {
        let deliveries = deliveries.clone();
        move |_evt: MouseEvent| {
            // Since we are adding we don't have a selected delivery id
            SELECTED_DELIVERY.with(|selected_delivery_rc| {
                let selected_delivery = selected_delivery_rc.borrow().as_ref().unwrap().clone();
                let delivery_id_str = (deliveries.len() + 1).to_string();
                selected_delivery.set(SelectedDeliveryInfo {
                    delivery_id_str,
                    ..Default::default()
                });
            });

            let dlg = bootstrap::get_modal_by_id("deliveryAddOrEditDlg").unwrap();
            dlg.toggle();
        }
    };

    let on_edit = {
        let deliveries = deliveries.clone();
        move |evt: MouseEvent| {
            let delivery_id = get_delivery_id(evt);
            log::info!("Editing ID: {}", delivery_id);
            SELECTED_DELIVERY.with(|selected_delivery_rc| {
                let di = deliveries.get(&delivery_id).unwrap();
                let delivery_date_str = di.get_delivery_date_str();
                let cutoff_date_str = di.get_new_order_cutoff_date_str();
                let selected_delivery = selected_delivery_rc.borrow().as_ref().unwrap().clone();
                let delivery_id_str = delivery_id.to_string();
                selected_delivery.set(SelectedDeliveryInfo {
                    delivery_id_str,
                    delivery_date_str,
                    cutoff_date_str,
                });
            });
            let dlg = bootstrap::get_modal_by_id("deliveryAddOrEditDlg").unwrap();
            dlg.toggle();
        }
    };

    let on_save_deliveries = {
        let deliveries = deliveries.clone();
        let is_dirty = is_dirty.clone();
        move |_evt: MouseEvent| {
            let document = gloo::utils::document();
            disable_save_button(&document, true, true);
            let deliveries = deliveries.clone();
            let is_dirty = is_dirty.clone();
            wasm_bindgen_futures::spawn_local(async move {
                // log::info!("Saving Deliveries {:#?}", &deliveries);
                if let Err(err) = set_deliveries((*deliveries).clone()).await {
                    gloo::dialogs::alert(&format!("Failed saving delivery config:\n{:#?}", err));
                }
                disable_save_button(&document, false, false);
                is_dirty.set(false);
            });
        }
    };

    html! {
        <div>
            <div class="card">
                <div class="card-body">
                    <h5 class="card-title">
                        {"Mulch Delivery Dates"}
                        <button class="btn btn-outline-info float-end order-edt-btn" onclick={on_add_delivery}>
                            <i class="bi bi-plus-square" fill="currentColor"></i>
                        </button>
                        if *is_dirty {
                            <button class="btn btn-primary" onclick={on_save_deliveries} id="btnSaveDeliveries">
                                <span class="spinner-border spinner-border-sm me-1" role="status"
                                aria-hidden="true" id="saveDeliveryConfigSpinner" style="display: none;" />
                                {"Save"}
                            </button>
                        }
                    </h5>


                    <ul class="list-group">
                    {
                        (*deliveries).iter().map(|(id,delivery_info)| {
                            html!{<DeliveryLi deliveryid={id}
                                deliverydate={delivery_info.get_delivery_date_str()}
                                newordercutoff={delivery_info.get_new_order_cutoff_date_str()}
                                ondelete={on_delete.clone()}
                                onedit={on_edit.clone()} />}
                        }).collect::<Html>()
                    }
                    </ul>
                </div>
            </div>
            <DeliveryAddEditDlg onaddorupdate={on_add_or_update_dlg_submit}/>
        </div>
    }
}
