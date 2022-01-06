use yew::prelude::*;
use yew_router::prelude::*;
use wasm_bindgen::{JsValue, JsCast};
use web_sys::{ MouseEvent, InputEvent, Element, HtmlElement, HtmlSelectElement, HtmlInputElement, HtmlButtonElement};
use std::str::FromStr;
use std::cell::RefCell;
use std::rc::Rc;

use crate::data_model::*;
use crate::bootstrap;
use crate::datatable::*;
use crate::currency_utils::*;
use crate::AppRoutes;

static ALL_USERS_TAG: &'static str = "doShowAllUsers";

thread_local! {
    static CHOPPING_BLOCK: Rc<RefCell<Option<OrderToDelete>>> = Rc::new(RefCell::new(None));
}

struct OrderToDelete {
    datatable: JsValue,
    delete_dlg: bootstrap::Modal,
    tr_node: web_sys::Node,
    order_id: String,
}

enum ReportViewState {
    IsLoading,
    ReportHtmlGenerated(Vec<serde_json::Value>),
}

/////////////////////////////////////////////////
///
fn on_delete_order_from_rpt( evt: MouseEvent, datatable: std::rc::Rc<std::cell::RefCell<Option<DataTable>>>)
{
    evt.prevent_default();
    evt.stop_propagation();
    let btn_elm = evt.target()
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

    // for idx in 0..btn_elm.attributes().length() {
    //     if let Some(attr) = btn_elm.attributes().get_with_index(idx) {
    //         log::info!("{}: {}: {}", idx, attr.name(), attr.value());
    //     }
    // }
    let tr_node = btn_elm.parent_node()
        .and_then(|t| t.parent_node())
        .unwrap();
    let order_id = btn_elm.dyn_into::<HtmlElement>()
        .ok()
        .and_then(|t| t.dataset().get("orderid"))
        .unwrap();
    log::info!("on_delete_order: {}", order_id);

    let dlg = bootstrap::get_modal_by_id("deleteOrderDlg").unwrap();

    CHOPPING_BLOCK.with(|f|{
        *f.borrow_mut() = Some(OrderToDelete{
            datatable: (*datatable.borrow().as_ref().unwrap()).clone(),
            delete_dlg: dlg.clone().dyn_into::<bootstrap::Modal>().unwrap(),
            tr_node: tr_node,
            order_id: order_id,
        });
    });

    dlg.show();
}

/////////////////////////////////////////////////
///
fn on_view_or_edit_from_rpt( evt: MouseEvent, history: AnyHistory)
{
    evt.prevent_default();
    evt.stop_propagation();
    let btn_elm = evt.target()
        .and_then(|t| t.dyn_into::<Element>().ok())
        .and_then(|t| {
            if t.node_name() == "I" {
                t.parent_element()
            } else {
                Some(t)
            }
        })
    .unwrap();
    let order_id = btn_elm.dyn_into::<HtmlElement>()
        .ok()
        .and_then(|t| t.dataset().get("orderid"))
        .unwrap();
    wasm_bindgen_futures::spawn_local(async move {
        log::info!("on_view_or_edit_order: {}", order_id);
        if let Err(err) = load_active_order_from_db(&order_id).await {
            gloo_dialogs::alert(&format!("Failed to load order: {}: Err: {:#?}", order_id, err));
        }
        history.push(AppRoutes::OrderForm);
    });
}

/////////////////////////////////////////////////
///
fn on_edit_spreading_from_rpt( evt: MouseEvent)
{
    evt.prevent_default();
    evt.stop_propagation();
    let btn_elm = evt.target()
        .and_then(|t| t.dyn_into::<Element>().ok())
        .and_then(|t| t.parent_element())
        .and_then(|t| t.dyn_into::<HtmlElement>().ok())
        .unwrap();
    log::info!("on_edit_spreading: {}", btn_elm.dataset().get("orderid").unwrap());
}

/////////////////////////////////////////////////
/////////////////////////////////////////////////
#[function_component(DeleteOrderDlg)]
fn delete_order_confirmation_dlg() -> Html {

    let on_confirm_input = {
        Callback::from(move |evt: InputEvent|{
            let value = evt.target()
                .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
                .unwrap()
                .value();
            if "delete" == &value {
                gloo_utils::document().get_element_by_id("deleteDlgBtn")
                    .and_then(|t| t.dyn_into::<HtmlButtonElement>().ok())
                    .unwrap().set_disabled(false);
            } else {
                gloo_utils::document().get_element_by_id("deleteDlgBtn")
                    .and_then(|t| t.dyn_into::<HtmlButtonElement>().ok())
                    .unwrap().set_disabled(true);
            }
        })
    };

    let on_submit = {
        Callback::from(move |evt: MouseEvent|{
            evt.target()
                .and_then(|t| t.dyn_into::<HtmlButtonElement>().ok())
                .unwrap().set_disabled(true);
                //.and_then(|t| t.set_disabled(true));
            CHOPPING_BLOCK.with(|f|{
                let f=f.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    if let Some(to_delete) = &*f.borrow() {
                        if let Err(err) = delete_order(&to_delete.order_id).await {
                            gloo_dialogs::alert(&format!("Failed to delete order in the cloud: {:#?}", err));
                        } else {
                            if let Err(err) = remove_row_with_tr(&to_delete.datatable, &to_delete.tr_node) {
                                gloo_dialogs::alert(&format!("Order was deleted from the cloud but not the local table: {:#?}", err));
                            }
                        }

                        to_delete.delete_dlg.hide();
                        gloo_utils::document().get_element_by_id("confirmDeleteOrderInput")
                            .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
                            .unwrap()
                            .set_value("");
                    }
                    *f.borrow_mut() = None;

                    evt.target()
                        .and_then(|t| t.dyn_into::<HtmlButtonElement>().ok())
                        .unwrap().set_disabled(false);
                });
            });
        })
    };

    html! {
        <div class="modal fade" id="deleteOrderDlg"
             tabIndex="-1" role="dialog" aria-labelledby="deleteOrderDlgTitle" aria-hidden="true">
            <div class="modal-dialog modal-dialog-centered" role="document">
                <div class="modal-content">
                    <div class="modal-header">
                        <h5 class="modal-title" id="deleteOrderDlgLongTitle">
                            {"Confirm Order Deletion"}
                        </h5>
                    </div>
                    <div class="modal-body">
                        <input type="text" class="form-control" id="confirmDeleteOrderInput"
                               placeholder="type delete to confirm" autocomplete="fr-new-cust-info"
                               oninput={on_confirm_input.clone()} aria-describedby="confirmDeleteOrderHelp" />
                        <small id="confirmDeleteOrderHelp" class="form-text text-muted">
                            {"Enter \"delete\" to confirm order deletion."}
                        </small>

                    </div>
                    <div class="modal-footer">
                        <button type="button" class="btn btn-secondary" data-bs-dismiss="modal">
                            {"Cancel"}
                        </button>
                        <button type="button" disabled=true class="btn btn-primary" onclick={on_submit} id="deleteDlgBtn">
                            {"Delete Order"}
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
pub struct ReportActionButtonsProps {
    pub showspreading: bool,
    pub isreadonly: bool,
    pub orderid: String,
    pub ondeleteorder: Callback<MouseEvent>,
    pub onvieworder: Callback<MouseEvent>,
    pub oneditorder: Callback<MouseEvent>,
    pub oneditspreading: Callback<MouseEvent>,
}

#[function_component(ReportActionButtons)]
pub fn report_action_buttons(props: &ReportActionButtonsProps) -> Html {
    html!{
        <>
        if props.showspreading && false /* TOOD: Enable Later */ {
            <button type="button" class="btn btn-outline-info me-1 order-spread-btn"
                onclick={props.oneditspreading.clone()} data-orderid={props.orderid.clone()}
                data-bs-toggle="tooltip" title="Select Spreaders" data-bs-placement="left">
                 <i class="bi bi-layout-wtf" fill="currentColor" />
            </button>
        }

        if props.isreadonly {
            <button type="button" class="btn btn-outline-info me-1 order-view-btn"
                onclick={props.onvieworder.clone()} data-orderid={props.orderid.clone()}
                data-bs-toggle="tooltip" title="View Order" data-bs-placement="left">
                 <i class="bi bi-eye" fill="currentColor" />
            </button>
        } else {
            <button type="button" class="btn btn-outline-info me-1 order-edt-btn"
                onclick={props.oneditorder.clone()} data-orderid={props.orderid.clone()}
                data-bs-toggle="tooltip" title="Edit Order" data-bs-placement="left">
                 <i class="bi bi-pencil" fill="currentColor" />
            </button>
            <button type="button" class="btn btn-outline-danger order-del-btn"
                onclick={props.ondeleteorder.clone()} data-orderid={props.orderid.clone()}
                data-bs-toggle="tooltip" title="Delete Order" data-bs-placement="left">
                <i class="bi bi-trash" fill="currentColor" />
            </button>
        }

        </>
    }
}

/////////////////////////////////////////////////
/////////////////////////////////////////////////
#[function_component(ReportLoadingSpinny)]
fn report_loading_spinny() -> Html {
    html! {
        <div class="justify-content-center text-center">
            <h2>{"Loading Report Data..."}</h2>
            <span role="status" class="spinner-border ms-1"/>
        </div>
    }
}

/////////////////////////////////////////////////
/////////////////////////////////////////////////
#[derive(Properties, PartialEq, Clone, Debug)]
pub struct QuickReportViewProps {
    pub seller: String,
}
#[function_component(QuickReportView)]
pub fn report_quick_view(props: &QuickReportViewProps) -> Html {
    let report_state = use_state(||ReportViewState::IsLoading);
    let history = use_history().unwrap();
    let is_fr_locked = is_fundraiser_locked();
    let datatable: std::rc::Rc<std::cell::RefCell<Option<DataTable>>> = use_mut_ref(|| None);

    let on_delete_order = {
        let datatable = datatable.clone();
        Callback::from(move |evt: MouseEvent| {
            on_delete_order_from_rpt(evt, datatable.clone());
        })
    };
    let on_view_or_edit_order = {
        let history = history.clone();
        Callback::once(move |evt: MouseEvent| {
            on_view_or_edit_from_rpt(evt, history.clone());
        })
    };

    let on_edit_spreading = {
        Callback::from(move |evt: MouseEvent| {
            on_edit_spreading_from_rpt(evt);
        })
    };

    {
        let report_state = report_state.clone();
        let seller = props.seller.to_string();
        use_effect(move || {
            match &*report_state {
                ReportViewState::IsLoading=>{
                    wasm_bindgen_futures::spawn_local(async move {
                        log::info!("Downloading Quick Report View Data");
                        let seller = if &seller == ALL_USERS_TAG {
                            None
                        } else {
                            Some(seller)
                        };
                        let resp = get_quick_report_data(seller.as_ref()).await.unwrap();
                        log::info!("Report Data has been downloaded");
                        report_state.set(ReportViewState::ReportHtmlGenerated(resp));
                    });
                },
                ReportViewState::ReportHtmlGenerated(_) => {
                    log::info!("Setting DataTable");
                    *datatable.borrow_mut() = get_datatable(&serde_json::json!({
                        "reportType": "quick",
                        "id": ".data-table-report table",
                        "showOrderOwner": &seller != &get_active_user().get_id(),
                        "isMulchOrder": true
                    }));

                },
            };

            ||{}
        });
    }

    match &*report_state {
        ReportViewState::IsLoading => html! { <ReportLoadingSpinny/> },
        ReportViewState::ReportHtmlGenerated(orders) => {
            let header_footer = html! {
                <tr>
                    <th>{"OrderId"}</th>
                    <th>{"Name"}</th>
                    <th>{"Delivery Date"}</th>
                    <th>{"Spreaders"}</th>
                    <th>{"Spreading"}</th>
                    <th>{"Order Owner"}</th>
                    <th>{"Actions"}</th>
                </tr>
            };
            html!{
                <div class="data-table-report">
                    <table class="display responsive nowrap collapsed" role="grid" style="width: 100%;">
                        <thead>
                            {header_footer.clone()}
                        </thead>
                        <tbody>
                        {
                            orders.into_iter().map(|v|{
                                let mut spreading = "".to_string();
                                for purchase in v["purchases"].as_array().unwrap_or(&Vec::new()) {
                                    if purchase["productId"].as_str().unwrap() == "spreading" {
                                        spreading = purchase["numSold"].as_u64().unwrap().to_string();
                                        break;
                                    }
                                }
                                let enable_spreading_button = spreading.len()!=0 && !is_fr_locked;
                                let (delivery_date, delivery_id) = match v["deliveryId"].as_u64() {
                                    Some(delivery_id) => (get_delivery_date(&(delivery_id as u32)), delivery_id.to_string()),
                                    None => ("Donation".to_string(), "Donation".to_string()),
                                };
                                let is_readonly = is_order_readonly(delivery_id.parse::<u32>().ok());
                                html!{
                                    <tr>
                                        <td>{v["orderId"].as_str().unwrap()}</td>
                                        <td>{v["customer"]["name"].as_str().unwrap()}</td>
                                        <td data-deliveryid={delivery_id}>{delivery_date}</td>
                                        <td>{v["spreaders"].as_str().unwrap_or("")}</td>
                                        <td>{&spreading}</td>
                                        <td>{v["ownerId"].as_str().unwrap()}</td>
                                        <td>
                                            <ReportActionButtons
                                                orderid={v["orderId"].as_str().unwrap().to_string()}
                                                showspreading={enable_spreading_button}
                                                isreadonly={is_readonly}
                                                ondeleteorder={on_delete_order.clone()}
                                                onvieworder={on_view_or_edit_order.clone()}
                                                oneditorder={on_view_or_edit_order.clone()}
                                                oneditspreading={on_edit_spreading.clone()}
                                            />
                                        </td>
                                    </tr>
                                }
                            }).collect::<Html>()
                        }
                        </tbody>
                        <tfoot>
                            {header_footer}
                        </tfoot>
                    </table>
                </div>
            }
        },
    }
}

/////////////////////////////////////////////////
/////////////////////////////////////////////////
#[derive(Properties, PartialEq, Clone, Debug)]
pub struct FullReportViewProps {
    pub seller: String,
}
#[function_component(FullReportView)]
pub fn report_full_view(props: &FullReportViewProps) -> Html {
    let report_state = use_state(||ReportViewState::IsLoading);
    let history = use_history().unwrap();
    let is_fr_locked = is_fundraiser_locked();
    let datatable: std::rc::Rc<std::cell::RefCell<Option<DataTable>>> = use_mut_ref(|| None);

    let on_delete_order = {
        let datatable = datatable.clone();
        Callback::from(move |evt: MouseEvent| {
            on_delete_order_from_rpt(evt, datatable.clone());
        })
    };
    let on_view_or_edit_order = {
        let history = history.clone();
        Callback::once(move |evt: MouseEvent| {
            on_view_or_edit_from_rpt(evt, history.clone());
        })
    };

    let on_edit_spreading = {
        Callback::from(move |evt: MouseEvent| {
            on_edit_spreading_from_rpt(evt);
        })
    };

    {
        let report_state = report_state.clone();
        let seller = props.seller.to_string();
        use_effect(move || {
            match &*report_state {
                ReportViewState::IsLoading=>{
                    wasm_bindgen_futures::spawn_local(async move {
                        log::info!("Downloading Full Report View Data");
                        let seller = if &seller == ALL_USERS_TAG {
                            None
                        } else {
                            Some(seller)
                        };
                        let resp = get_full_report_data(seller.as_ref()).await.unwrap();
                        log::info!("Report Data has been downloaded");
                        report_state.set(ReportViewState::ReportHtmlGenerated(resp));
                    });
                },
                ReportViewState::ReportHtmlGenerated(_) => {
                    log::info!("Setting DataTable");
                    *datatable.borrow_mut() = get_datatable(&serde_json::json!({
                        "reportType": "full",
                        "id": ".data-table-report table",
                        "showOrderOwner": &seller != &get_active_user().get_id(),
                        "isMulchOrder": true
                    }));

                },
            };

            ||{}
        });
    }

    match &*report_state {
        ReportViewState::IsLoading => html! { <ReportLoadingSpinny/> },
        ReportViewState::ReportHtmlGenerated(orders) => {
            let header_footer = html! {
                <tr>
                    <th>{"OrderId"}</th>
                    <th>{"Name"}</th>
                    <th>{"Phone"}</th>
                    <th>{"Email"}</th>
                    <th>{"Address 1"}</th>
                    <th>{"Address 2"}</th>
                    <th>{"Neighborhood"}</th>
                    <th>{"Delivery Date"}</th>
                    <th>{"Spreaders"}</th>
                    <th>{"Spreading"}</th>
                    <th>{"Bags"}</th>
                    <th>{"Special Instructions"}</th>
                    <th>{"Verified"}</th>
                    <th>{"Money Collected"}</th>
                    <th>{"Donations"}</th>
                    <th>{"Cash"}</th>
                    <th>{"Check"}</th>
                    <th>{"Check Numbers"}</th>
                    <th>{"Total Amount"}</th>
                    <th>{"Order Owner"}</th>
                    <th>{"Actions"}</th>
                </tr>
            };
            html!{
                <div class="data-table-report">
                    <table class="display responsive nowrap collapsed" role="grid" style="width: 100%;">
                        <thead>
                            {header_footer.clone()}
                        </thead>
                        <tbody>
                        {
                            orders.into_iter().map(|v|{
                                let mut spreading = "".to_string();
                                let mut bags = "".to_string();
                                for purchase in v["purchases"].as_array().unwrap_or(&Vec::new()) {
                                    if purchase["productId"].as_str().unwrap() == "spreading" {
                                        spreading = purchase["numSold"].as_u64().unwrap().to_string();
                                        break;
                                    } else if purchase["productId"].as_str().unwrap() == "bags" {
                                        bags = purchase["numSold"].as_u64().unwrap().to_string();
                                        break;
                                    }
                                }
                                let enable_spreading_button = spreading.len()!=0 && !is_fr_locked;
                                let (delivery_date, delivery_id) = match v["deliveryId"].as_u64() {
                                    Some(delivery_id) => (get_delivery_date(&(delivery_id as u32)), delivery_id.to_string()),
                                    None => ("Donation".to_string(), "Donation".to_string()),
                                };
                                let is_readonly = is_order_readonly(delivery_id.parse::<u32>().ok());
                                html!{
                                    <tr>
                                        <td>{v["orderId"].as_str().unwrap()}</td>
                                        <td>{v["customer"]["name"].as_str().unwrap()}</td>
                                        <td>{v["customer"]["phone"].as_str().unwrap()}</td>
                                        <td>{v["customer"]["email"].as_str().unwrap_or("")}</td>
                                        <td>{v["customer"]["addr1"].as_str().unwrap()}</td>
                                        <td>{v["customer"]["addr2"].as_str().unwrap_or("")}</td>
                                        <td>{v["customer"]["neighborhood"].as_str().unwrap()}</td>
                                        <td data-deliveryid={delivery_id}>{delivery_date}</td>
                                        <td>{v["spreaders"].as_str().unwrap_or("")}</td>
                                        <td>{&spreading}</td>
                                        <td>{&bags}</td>
                                        <td>{v["specialInstructions"].as_str().unwrap_or("")}</td>
                                        <td>{v["isVerified"].as_bool().unwrap_or(false).to_string()}</td>
                                        <td>{v["willCollectMoneyLater"].as_bool().unwrap_or(false).to_string()}</td>
                                        <td>{to_money_str(v["amountFromDonations"].as_str())}</td>
                                        <td>{to_money_str(v["amountFromCashCollected"].as_str())}</td>
                                        <td>{to_money_str(v["amountFromChecksCollected"].as_str())}</td>
                                        <td>{v["checkNumbers"].as_str().unwrap_or("")}</td>
                                        <td>{to_money_str(v["amountTotalCollected"].as_str())}</td>
                                        <td>{v["ownerId"].as_str().unwrap()}</td>
                                        <td>
                                            <ReportActionButtons
                                                orderid={v["orderId"].as_str().unwrap().to_string()}
                                                showspreading={enable_spreading_button}
                                                isreadonly={is_readonly}
                                                ondeleteorder={on_delete_order.clone()}
                                                onvieworder={on_view_or_edit_order.clone()}
                                                oneditorder={on_view_or_edit_order.clone()}
                                                oneditspreading={on_edit_spreading.clone()}
                                            />
                                        </td>
                                    </tr>
                                }
                            }).collect::<Html>()
                        }
                        </tbody>
                        <tfoot>
                            {header_footer}
                        </tfoot>
                    </table>
                </div>
            }
        },
    }
}

/////////////////////////////////////////////////
/////////////////////////////////////////////////
#[derive(Properties, PartialEq, Clone, Debug)]
pub struct ReportsSettingsDlgProps {
    pub id: String,
    pub onsave: Callback<MouseEvent>,
    pub currentview: String,

}
#[function_component(ReportsSettingsDlg)]
pub fn reports_settings_dlg(props: &ReportsSettingsDlgProps) -> Html {

    let tag = props.id.clone();
    let active_user_id = get_active_user().get_id();

    let mut did_find_selected_view = false;
    let mut did_find_selected_seller = false;

    html! {
        <div class="modal fade" id={tag.to_string()} tabIndex="-1" aria-labelledby={format!("{}Title", &tag)} aria-hidden="true">
            <div class="modal-dialog modal-dialog-centered">
                <div class="modal-content">
                    <div class="modal-header">
                        <h5 class="modal-title" id={format!("{}LongTitle", &tag)}>
                            {"Switch report view settings"}
                        </h5>
                        //<button type="button" class="close" data-bs-dismiss="modal" aria-label="Close"></button>
                    </div>
                    <div class="modal-body">
                        <div class="container-sm">
                            <div class="row">
                                <div class="col-sm">
                                    <div class="form-floating">
                                        <select class="form-select" id={format!("{}ViewSelection", &tag)}>
                                        {
                                            get_allowed_report_views().iter().map(|v|{
                                                let is_selected = &ReportViews::from_str(&props.currentview).unwrap() == v;
                                                if !did_find_selected_view && is_selected { did_find_selected_view=true; }
                                                html! {
                                                    <option value={v.to_string()} selected={is_selected}>
                                                       {v.to_string()}
                                                    </option>
                                                }
                                            }).collect::<Html>()
                                        }
                                        if !did_find_selected_view {
                                            <option value="none" selected=true disabled=true hidden=true>
                                                {"Select Report View (DNE. Report Issue)"}
                                            </option>
                                        }
                                        </select>
                                        <label for={format!("{}ViewSelection", &tag)}>
                                            {"Select Report View"}
                                        </label>
                                    </div>
                                </div>
                                if get_active_user().is_admin() {
                                    <div class="col-sm" id={format!("{}UserSelectionCol", &tag)}>
                                        <div class="form-floating">
                                            <select class="form-select" id={format!("{}UserSelection", &tag)}>
                                            {
                                                get_active_sellers().iter().map(|v|{
                                                    let is_selected = &active_user_id == v;
                                                    if !did_find_selected_seller && is_selected { did_find_selected_seller=true; }
                                                    html! {
                                                        <option value={v.clone()} selected={is_selected}>
                                                           {v.clone()}
                                                        </option>
                                                    }
                                                }).collect::<Html>()
                                            }
                                                <option value={ALL_USERS_TAG}>{"Show All Users"}</option>

                                            if !did_find_selected_seller {
                                                <option value="none" selected=true disabled=true hidden=true>
                                                   {"Select Seller (DNE. Report Issue)"}
                                                </option>
                                            }
                                            </select>
                                            <label for={format!("{}UserSelection", &tag)}>
                                                {"Select Active Sellers"}
                                            </label>
                                        </div>
                                    </div>
                                }
                            </div>
                        </div>
                    </div>
                    <div class="modal-footer">
                        <button type="button" class="btn btn-secondary" data-bs-dismiss="modal">
                            {"Cancel"}
                        </button>
                        <button type="button" class="btn btn-primary" data-bs-dismiss="modal" onclick={props.onsave.clone()}>
                            {"Save"}
                        </button>
                    </div>
                </div>
            </div>
        </div>
    }
}

/////////////////////////////////////////////////
/////////////////////////////////////////////////

#[derive(PartialEq, Debug)]
struct ReportViewSettings {
    current_view: ReportViews,
    seller_id_filter: String,
}

#[function_component(Reports)]
pub fn reports_page() -> Html {
    let current_settings = use_state_eq(|| ReportViewSettings{
        current_view: ReportViews::Quick,
        seller_id_filter: get_active_user().get_id(),
    });


    // let on_download_report = {
    //     Callback::from(move |evt: MouseEvent| {
    //         evt.prevent_default();
    //         evt.stop_propagation();
    //         log::info!("on_download_report");

    //     })
    // };

    let on_view_settings = {
        Callback::from(move |evt: MouseEvent| {
            evt.prevent_default();
            evt.stop_propagation();
            log::info!("on_view_settings");

            let dlg = bootstrap::get_modal_by_id("reportViewSettingsDlg").unwrap();
            dlg.show();

        })
    };


    let on_save_settings = {
        let current_settings = current_settings.clone();
        Callback::from(move |_evt: MouseEvent| {
            let report_view = gloo_utils::document().get_element_by_id("reportViewSettingsDlgViewSelection")
                .and_then(|t| t.dyn_into::<HtmlSelectElement>().ok())
                .unwrap()
                .value();

            let seller_id: String = if get_active_user().is_admin() {
                gloo_utils::document().get_element_by_id("reportViewSettingsDlgUserSelection")
                    .and_then(|t| t.dyn_into::<HtmlSelectElement>().ok())
                    .unwrap()
                    .value()
            } else {
                get_active_user().get_id()
            };

            let updated_settings = ReportViewSettings{
                current_view: ReportViews::from_str(&report_view).unwrap(),
                seller_id_filter: seller_id,
            };

            log::info!("on_save_settings.  report view: {} seller: {}",
                &updated_settings.current_view, &updated_settings.seller_id_filter);

            current_settings.set(updated_settings);
        })
    };

    log::info!("Report View Rendering.  report view: {} seller: {}",
        &current_settings.current_view, &current_settings.seller_id_filter);

    html! {
        <div>
            <div class="col-xs-1 d-flex justify-content-center">
                <div class="card" style="width: 100%;">

                    <div class="card-body" id="cardReportBody">
                        <h6 class="card-title ps-2" id="orderCardTitle">
                            <ul class="list-group list-group-horizontal-sm">
                                <li class="list-group-item me-3">
                                    <label class="text-muted pe-2">{"Report View:"}</label>
                                    <div class="d-inline" id="reportViewLabel">
                                        {(*current_settings).current_view.to_string()}
                                    </div>
                                </li>
                                <li class="list-group-item" id="orderOwnerLabel">
                                    <label class="text-muted pe-2">{"Showing Orders for:"}</label>
                                    <div class="d-inline" id="reportViewOrderOwner">
                                        {(*current_settings).seller_id_filter.clone()}
                                    </div>
                                </li>
                            </ul>
                            <div id="reportViewSettings" class="float-end">
                                // <button type="button" class="btn reports-view-setting-btn" onclick={on_download_report}
                                //         data-bs-toggle="tooltip" title="Download Current Report">
                                //     <i class="bi bi-cloud-download" fill="currentColor"></i>
                                // </button>
                                <button type="button" class="btn reports-view-setting-btn" onclick={on_view_settings}
                                        data-bs-toggle="tooltip" data-bs-placement="left" title="Change Report View">
                                    <i class="bi bi-gear" fill="currentColor"></i>
                                </button>
                            </div>
                        </h6>

                        {
                            match (*current_settings).current_view {
                                ReportViews::Quick=>html!{<QuickReportView seller={(*current_settings).seller_id_filter.clone()}/>},
                                ReportViews::Full=>html!{<FullReportView seller={(*current_settings).seller_id_filter.clone()}/>},
                                _=>html!{<h6>{"Not Yet Implemented"}</h6>},
                            }
                        }

                        <div class="visually-hidden" id="orderLoadingSpinner">
                            <h2>{"Loading Report Data..."}</h2>
                            <span role="status" class="spinner-border ms-1"/>
                        </div>
                    </div>

                </div>
            </div>


            <DeleteOrderDlg />
            <ReportsSettingsDlg id="reportViewSettingsDlg"
                onsave={on_save_settings} currentview={(*current_settings).current_view.to_string()}/>
            // {spreadDlg}
            // {confirmDlg}
        </div>
    }
}

