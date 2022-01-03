use yew::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{ MouseEvent, HtmlSelectElement, HtmlButtonElement};

use crate::data_model::*;
use crate::bootstrap;
use crate::datatable::*;
use std::str::FromStr;

static ALL_USERS_TAG: &'static str = "doShowAllUsers";

enum ReportViewState {
    IsLoading,
    ReportHtmlGenerated(Vec<serde_json::Value>),
}

/////////////////////////////////////////////////
/////////////////////////////////////////////////
#[derive(Properties, PartialEq, Clone, Debug)]
pub struct ReportActionButtonsProps {
    pub spreading: String,
    pub isreadonly: bool
}
#[function_component(ReportActionButtons)]
pub fn report_action_buttons(_props: &ReportActionButtonsProps) -> Html {
    html!{}
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

    {
        let report_state = report_state.clone();
        let seller = props.seller.to_string();
        use_effect(move || {
            match &*report_state {
                ReportViewState::IsLoading=>{
                    wasm_bindgen_futures::spawn_local(async move {
                        log::info!("Downloading Quick Report View Data");
                        // Load Data from network
                        // Generate Html
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
                    let _ = get_quick_view_report_datatable(".data-table-report table");
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
                            orders.iter().map(|v|{
                                let mut spreading = "".to_string();
                                for purchase in v["purchases"].as_array().unwrap_or(&Vec::new()) {
                                    if purchase["productId"].as_str().unwrap() == "spreading" {
                                        spreading = purchase["numSold"].as_u64().unwrap().to_string();
                                        break;
                                    }
                                }
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
                                            <ReportActionButtons spreading={spreading} isreadonly={is_readonly}/>
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
                                                html! {
                                                    <option value={v.to_string()} selected={is_selected}>
                                                       {v.to_string()}
                                                    </option>
                                                }
                                            }).collect::<Html>()
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
                                                    html! {
                                                        <option value={v.clone()} selected={is_selected}>
                                                           {v.clone()}
                                                        </option>
                                                    }
                                                }).collect::<Html>()
                                            }
                                                <option value={ALL_USERS_TAG} selected=true>{"Show All Users"}</option>
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


    let on_download_report = {
        Callback::from(move |evt: MouseEvent| {
            evt.prevent_default();
            evt.stop_propagation();
            log::info!("on_download_report");

        })
    };

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

            let seller_id = gloo_utils::document().get_element_by_id("reportViewSettingsDlgUserSelection")
                .and_then(|t| t.dyn_into::<HtmlSelectElement>().ok())
                .unwrap()
                .value();

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
                                <button type="button" class="btn reports-view-setting-btn" onclick={on_download_report}
                                        data-bs-toggle="tooltip" title="Download Current Report">
                                    <i class="bi bi-cloud-download" fill="currentColor"></i>
                                </button>
                                <button type="button" class="btn reports-view-setting-btn" onclick={on_view_settings}
                                        data-bs-toggle="tooltip" data-bs-placement="left" title="Change Report View">
                                    <i class="bi bi-gear" fill="currentColor"></i>
                                </button>
                            </div>
                        </h6>

                        {
                            match (*current_settings).current_view {
                                ReportViews::Quick=>html!{<QuickReportView seller={(*current_settings).seller_id_filter.clone()}/>},
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


            // {deleteDlg}
            <ReportsSettingsDlg id="reportViewSettingsDlg"
                onsave={on_save_settings} currentview={(*current_settings).current_view.to_string()}/>
            // {spreadDlg}
            // {confirmDlg}
        </div>
    }
}

