use yew::prelude::*;
use wasm_bindgen::{JsCast};
use web_sys::{ MouseEvent, HtmlSelectElement};
use std::str::FromStr;

use crate::data_model::*;


/////////////////////////////////////////////////
///
fn get_delivery_id() -> Option<String> {
    let document = gloo_utils::document();
    let value = document.get_element_by_id("timeSheetSelectDeliveryDate")
        .and_then(|t| t.dyn_into::<HtmlSelectElement>().ok())
        .unwrap()
        .value();
    log::info!("Delivery Date Selection Val: {}", &value);
    if 0==value.len() || "none" == value {
        None
    } else {
        Some(value)
    }
}

/////////////////////////////////////////////////
/////////////////////////////////////////////////
#[function_component(Timecards)]
pub fn timecards_page() -> Html {

    let timecards_data_ready: Option<Vec<serde_json::Value>> = use_state_eq(|| None);
    let is_delivery_date_selected = use_state_eq(|| false);

    let on_export_timecards = {
        Callback::from(move |evt: MouseEvent| {
            evt.prevent_default();
            evt.stop_propagation();
            log::info!("on_export_timecards");

        })
    };

    let on_delivery_selection_change = {
        let timecards_data_ready = timecards_data_ready.clone();
        let is_delivery_date_selected = is_delivery_date_selected.clone();
        Callback::from(move |evt: MouseEvent| {
            evt.prevent_default();
            evt.stop_propagation();
            log::info!("on_delivery_selection_change");
            if let Some(delivery_id_str) = get_delivery_id() {
                is_delivery_date_selected.set(true);
                log::info!("Downloading timecard data for: {delivery_id_str}");
                let delivery_id = delivery_id_str.parse::<u32>().unwrap();
                wasm_bindgen_futures::spawn_local(async move {
                    let resp = get_timecards_data(delivery_id).await.unwrap();
                    log::info!("Timecards data ready");
                    timecards_data_ready.set(Some(resp));
                });
            }
        })
    };

    // let on_view_settings = {
    //     Callback::from(move |evt: MouseEvent| {
    //         evt.prevent_default();
    //         evt.stop_propagation();
    //         log::info!("on_view_settings");

    //         let dlg = bootstrap::get_modal_by_id("reportViewSettingsDlg").unwrap();
    //         dlg.show();

    //     })
    // };


    // let on_save_entry = {
    //     let current_settings = current_settings.clone();
    //     Callback::from(move |_evt: MouseEvent| {
    //         let report_view = gloo_utils::document().get_element_by_id("reportViewSettingsDlgViewSelection")
    //             .and_then(|t| t.dyn_into::<HtmlSelectElement>().ok())
    //             .unwrap()
    //             .value();

    //         let seller_id: String = if get_active_user().is_admin() {
    //             gloo_utils::document().get_element_by_id("reportViewSettingsDlgUserSelection")
    //                 .and_then(|t| t.dyn_into::<HtmlSelectElement>().ok())
    //                 .unwrap()
    //                 .value()
    //         } else {
    //             get_active_user().get_id()
    //         };

    //         let updated_settings = ReportViewSettings{
    //             current_view: ReportViews::from_str(&report_view).unwrap(),
    //             seller_id_filter: seller_id,
    //         };
    //         if let Err(err) = save_report_settings(&updated_settings) {
    //             log::error!("Failed saving report settings: {:#?}", err);
    //         }

    //         log::info!("on_save_settings.  report view: {} seller: {}",
    //             &updated_settings.current_view, &updated_settings.seller_id_filter);

    //         current_settings.set(updated_settings);
    //     })
    // };

    html! {
        <div>
            <div class="col-xs-1 d-flex justify-content-center">
                <div class="card" style="width: 100%;">

                    <div class="card-body" id="cardReportBody">
                        <h6 class="card-title ps-2" id="orderCardTitle">

                    <h5 class="card-title">{"Delivery Timesheet"}</h5>
                    <div class="row mb-2">
                        <span>{"Choose Delivery Date"}
                            <select class="ms-1" id="timeSheetSelectDeliveryDate" onchange={on_delivery_selection_change}>
                                {
                                    get_deliveries().iter().map(|(delivery_id, delivery)| {
                                        html!{
                                            <option value={delivery_id.to_string()}>
                                                {delivery.get_delivery_date_str()}
                                            </option>
                                        }
                                    }).collect::<Html>()
                                }
                                <option value="none" selected=true disabled=true hidden=true>{"Select delivery date"}</option>
                            </select>
                            <button type="button" class="btn reports-view-setting-btn invisible ms-3"
                                    onclick={on_export_timecards} data-bs-toggle="tooltip"
                                    title="Download Timecards">
                                <i class="bi bi-cloud-download" fill="currentColor"></i>
                            </button>
                        </span>
                    </div>
                    if !is_delivery_date_selected {
                        <div>{"Select a delivery date"}</div>
                    } else if let Some(timecards) = timecards_data_ready {
                        <ul class="list-group" id="timeSheet">
                            timecardData.into_iter().map(|v| {
                                let time_in_id = format!("timeInId-{}", &uid);
                                let time_out_id = format!("timeOutId-{}", &uid);
                                let time_calc_id = format!("timeCalcId-{}", &uid);
                                html!{
                                    <li class="list-group-item">
                                        <div class="row" data-uid={uid.clone()} data-uname={user_name}>
                                            <div class="col">
                                                {userName}
                                            </div>
                                            <div class="col">
                                                <div class="form-floating">
                                                    <input data-clocklet="format: HH:mm;" oninput={on_time_change}
                                                           class="form-control time-in" id={time_in_id} />
                                                    <label for={time_in_id}>{"Time In"}</label>
                                                </div>
                                            </div>
                                            <div class="col">
                                                <div class="form-floating">
                                                    <input data-clocklet="format: HH:mm;" oninput={on_time_change}
                                                           class="form-control time-out" id={time_out_id} />
                                                    <label for={time_out_id}>{"Time Out"}</label>
                                                </div>
                                            </div>
                                            <div class="col">
                                                <div class="form-floating">
                                                    <div id={time_calc_id} class="form-control time-calc">{"00:00"}</div>
                                                    <label for={time_calc_id}>{"Total Time"}</label>
                                                </div>
                                            </div>
                                            <div class="col">
                                                <button type="button" class="btn btn-primary invisible" onclick={on_save_entry}>
                                                    <span class="spinner-border spinner-border-sm me-1" role="status"
                                                          aria-hidden="true" style={{display: "none"}} />
                                                          {"Save"}
                                                </button>
                                            </div>
                                        </div>
                                    </li>
                                }
                            });
                        </ul>
                    } else {
                        <div class="col-xs-1 d-flex justify-content-center" >
                            <div class="spinner-border" role="status">
                                <span class="visually-hidden">{"Loading Timecard data..."}</span>
                            </div>
                        </div>
                    }
                </div>
            </div>
        </div>
    }
}

