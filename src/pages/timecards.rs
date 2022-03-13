use yew::prelude::*;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{Event, InputEvent, MouseEvent, Element, HtmlElement, HtmlButtonElement, HtmlInputElement, HtmlSelectElement};

use crate::data_model::*;


/////////////////////////////////////////////////
///
fn get_delivery_id() -> Option<u32> {
    let document = gloo_utils::document();
    let value = document.get_element_by_id("timeSheetSelectDeliveryDate")
        .and_then(|t| t.dyn_into::<HtmlSelectElement>().ok())
        .unwrap()
        .value();
    log::info!("Delivery Date Selection Val: {}", &value);
    if 0==value.len() || "none" == value {
        None
    } else {
        value.parse::<u32>().ok()
    }
}

/////////////////////////////////////////////////
/////////////////////////////////////////////////
#[function_component(Timecards)]
pub fn timecards_page() -> Html {

    let timecards_data_ready: yew::UseStateHandle<Option<Vec<(String, String, Option<TimeCard>)>>> = use_state_eq(|| None);
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
        Callback::from(move |evt: Event| {
            evt.prevent_default();
            evt.stop_propagation();
            log::info!("on_delivery_selection_change");
            if let Some(delivery_id) = get_delivery_id() {
                is_delivery_date_selected.set(true);
                log::info!("Downloading timecard data for: {delivery_id}");
                let timecards_data_ready = timecards_data_ready.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    match get_timecards_data(Some(delivery_id), None).await {
                        Ok(resp)=>{
                            log::info!("Timecards data ready");
                            timecards_data_ready.set(Some(resp));
                        },
                        Err(err)=>{
                            let err_str = format!("Failed to get retrieve timecard data: {:#?}", err);
                            log::error!("{}",&err_str);
                            gloo_dialogs::alert(err_str.as_str());
                        },
                    };
                });
            }
        })
    };

    let on_time_change = {
        Callback::from(move |evt: InputEvent| {
            use std::time::{ Duration };

            evt.prevent_default();
            evt.stop_propagation();
            log::info!("on_time_change");

            let row_elm = evt.target()
                .and_then(|t| t.dyn_into::<Element>().ok())
                .and_then(|t|t.parent_element())
                .and_then(|t|t.parent_element())
                .and_then(|t|t.parent_element())
                .unwrap();

            fn read_time_val(elm: Result<Option<Element>, JsValue>)->Option<Duration> {
                if let Some(time_val_str) = elm
                    .ok()
                    .flatten()
                    .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
                    .and_then(|t| {
                        // log::info!("IEVal: {}", t.value());
                        Some(t.value().split(":").map(|v|v.to_string()).collect::<Vec<String>>())
                    })
                {
                    // log::info!("Read in: {}", time_val_str.join(" ,"));
                    if time_val_str.len() == 2 {
                        return time_val_str[0]
                            .parse::<u64>().ok()
                            .and_then(|v1|Some(Duration::from_secs(v1*60*60)))
                            .and_then(|v1| {
                                time_val_str[1]
                                    .parse::<u64>().ok()
                                    .and_then(|v2|Some(Duration::from_secs(v2*60)))
                                    .and_then(|v2| v1.checked_add(v2))
                            });
                    }
                }
                None
            }

            let time_in_val = read_time_val(row_elm.query_selector(".time-in"));
            // log::info!("TI Val: {}", time_in_val.unwrap_or(Duration::from_secs(0)).as_secs());
            let time_out_val = read_time_val(row_elm.query_selector(".time-out"));
            // log::info!("TO Val: {}", time_out_val.unwrap_or(Duration::from_secs(0)).as_secs());


            let time_calc_elm = row_elm.query_selector(".time-calc").ok()
                .flatten()
                .and_then(|t| t.dyn_into::<HtmlElement>().ok())
                .unwrap();

            let btn_elm = row_elm.query_selector(".btn").ok()
                .flatten()
                .and_then(|t| t.dyn_into::<HtmlElement>().ok())
                .unwrap();

            if time_in_val.is_none() || time_out_val.is_none() {
                let _ = btn_elm.class_list().add_1("invisible");
                time_calc_elm.set_inner_text("00:00");
                return;
            }

            let time_in = time_in_val.unwrap();
            let time_out = time_out_val.unwrap();


            let mark_invalid = ||{
                let _ = time_calc_elm.class_list().add_1("is-invalid");
                let _ = btn_elm.class_list().add_1("invisible");
                time_calc_elm.set_inner_text("00:00");
            };

            match time_out.checked_sub(time_in) {
                Some(time_total)=>{
                    let time_total_secs = time_total.as_secs();
                    if 0==time_total_secs {
                        mark_invalid();
                        return;
                    }

                    let _ = time_calc_elm.class_list().remove_1("is-invalid");
                    let new_hours:u64 = (time_total_secs as f64 / (60.0*60.0)).floor() as u64;
                    let new_mins:u64 = ((time_total_secs as f64 % (60.0*60.0)) / 60.0).floor() as u64;
                    let new_time_total_str = format!("{:02}:{:02}", new_hours, new_mins);
                    if 0 == new_hours && 0 == new_mins {
                        let _ = btn_elm.class_list().add_1("invisible");
                        return;
                    }
                    let _ = btn_elm.class_list().remove_1("invisible");
                    time_calc_elm.set_inner_text(&new_time_total_str);
                },
                None=>mark_invalid(),
            };
        })
    };


    let on_save_entry = {
        Callback::from(move |evt: MouseEvent| {
            evt.prevent_default();
            evt.stop_propagation();
            log::info!("on_save_entry");
            let btn_elm = evt.target()
                .and_then(|t| t.dyn_into::<Element>().ok())
                .and_then(|t|
                    match t.node_name().as_str() {
                        "I"=>t.parent_element(),
                        _=>Some(t),
                    }
                )
                .and_then(|t| t.dyn_into::<HtmlButtonElement>().ok())
                .unwrap();

            let spinny_elm = btn_elm.query_selector(".spinner-border")
                .ok().flatten()
                .and_then(|t| t.dyn_into::<HtmlElement>().ok())
                .unwrap();

            let _ = spinny_elm.class_list().remove_1("d-none");
            btn_elm.set_disabled(true);

            let _ = spinny_elm.class_list().add_1("d-none");
            btn_elm.set_disabled(false);
        })
    };

    html! {
        <div class="col-xs-1 d-flex justify-content-center">
            <div class="card" style="width: 100%;">

                <div class="card-body" id="cardReportBody">
                    <h6 class="card-title">{"Delivery Timesheet"}</h6>
                    <div class="row mb-2">
                        <span>
                            {"Choose Delivery Date"}
                            <select class="ms-1" id="timeSheetSelectDeliveryDate" onchange={on_delivery_selection_change.clone()}>
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
                    if !(*is_delivery_date_selected) {
                        <div>{"Select a delivery date"}</div>
                    } else if let Some(timecards_data) = &*timecards_data_ready {
                        <ul class="list-group" id="timeSheet"> {
                            timecards_data.into_iter().map(|(uid, user_name, tc)| {
                                let time_in_id = format!("timeInId-{}", &uid);
                                let time_out_id = format!("timeOutId-{}", &uid);
                                let time_calc_id = format!("timeCalcId-{}", &uid);
                                html!{
                                    <li class="list-group-item">
                                        <div class="row" data-uid={uid.clone()} data-uname={user_name.clone()}>
                                            <div class="col">
                                                {user_name.clone()}
                                            </div>
                                            <div class="col">
                                                <div class="form-floating">
                                                    <input data-clocklet="format: HH:mm;" oninput={on_time_change.clone()}
                                                           class="form-control time-in" id={time_in_id.clone()}
                                                           value={tc.as_ref().map_or("".to_string(), |v|v.time_in.clone())}
                                                    />
                                                    <label for={time_in_id}>{"Time In"}</label>
                                                </div>
                                            </div>
                                            <div class="col">
                                                <div class="form-floating">
                                                    <input data-clocklet="format: HH:mm;" oninput={on_time_change.clone()}
                                                           class="form-control time-out" id={time_out_id.clone()}
                                                           value={tc.as_ref().map_or("".to_string(), |v|v.time_out.clone())}
                                                    />
                                                    <label for={time_out_id}>{"Time Out"}</label>
                                                </div>
                                            </div>
                                            <div class="col">
                                                <div class="form-floating">
                                                    <div id={time_calc_id.clone()} class="form-control time-calc">
                                                        {tc.as_ref().map_or("00:00".to_string(), |v|v.time_total.clone())}
                                                    </div>
                                                    <label for={time_calc_id}>{"Total Time"}</label>
                                                </div>
                                            </div>
                                            <div class="col">
                                                <button type="button" class="btn btn-primary invisible" onclick={on_save_entry.clone()}>
                                                    <span class="spinner-border spinner-border-sm me-1 d-none" role="status" aria-hidden="true" />
                                                    {"Save"}
                                                </button>
                                            </div>
                                        </div>
                                    </li>
                                }
                            }).collect::<Html>()
                        }
                        </ul>
                    } else {
                        <div class="spinner-border" role="status">
                            <span class="visually-hidden">{"Loading Timecard data..."}</span>
                        </div>
                    }
                </div>
            </div>
        </div>
    }
}

