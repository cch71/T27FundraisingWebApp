use std::str::FromStr;
use wasm_bindgen::JsCast;
use web_sys::{Event, HtmlElement, HtmlSelectElement, MouseEvent};
use yew::prelude::*;

use data_model::*;
use js::bootstrap;

use crate::components::delete_report_order_dlg::DeleteOrderDlg;
use crate::components::report_deliveries::DeliveriesReportView;
use crate::components::report_distribution_points::DistributionPointsReportView;
use crate::components::report_full::FullReportView;
use crate::components::report_money_collection::MoneyCollectionReportView;
use crate::components::report_quick::QuickReportView;
use crate::components::report_sell_map::SellMapReportView;
use crate::components::report_spreaders_dlg::ChooseSpreadersDlg;
use crate::components::report_spreading_jobs::SpreadingJobsReportView;
use crate::components::report_spreading_jobs_unfinished::SpreadingJobsUnfinishedReportView;
use crate::components::report_verify::OrderVerificationView;

/////////////////////////////////////////////////
/////////////////////////////////////////////////
#[derive(Properties, PartialEq, Clone, Debug)]
struct ReportsSettingsDlgProps {
    id: String,
    onsave: Callback<MouseEvent>,
    currentview: ReportViews,
}
#[function_component(ReportsSettingsDlg)]
fn reports_settings_dlg(props: &ReportsSettingsDlgProps) -> Html {
    let tag = props.id.clone();
    let active_user_id = get_active_user().get_id();

    let mut did_find_selected_view = false;
    let mut did_find_selected_seller = false;

    let on_view_selection_change = {
        Callback::from(move |evt: Event| {
            if let Some(v) = evt
                .target()
                .and_then(|t| t.dyn_into::<HtmlSelectElement>().ok())
            {
                let selected_view = ReportViews::from_str(&v.value()).unwrap();
                let do_show_seller = do_show_current_seller(&selected_view);
                log::info!("Do Show Seller Selection: {}", do_show_seller);

                if get_active_user().is_admin() {
                    if do_show_seller {
                        let _ = gloo::utils::document()
                            .get_element_by_id("reportViewSettingsDlgUserSelectionCol")
                            .and_then(|t| t.dyn_into::<HtmlElement>().ok())
                            .unwrap()
                            .class_list()
                            .remove_1("d-none");
                    } else {
                        let _ = gloo::utils::document()
                            .get_element_by_id("reportViewSettingsDlgUserSelectionCol")
                            .and_then(|t| t.dyn_into::<HtmlElement>().ok())
                            .unwrap()
                            .class_list()
                            .add_1("d-none");
                    }
                }
            };
        })
    };

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
                                        <select class="form-select" id={format!("{}ViewSelection", &tag)} onchange={on_view_selection_change}>
                                        {
                                            get_allowed_report_views().iter().map(|v|{
                                                let is_selected = &props.currentview == v;
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
                                                get_users().iter().map(|(uid,user_info)|{
                                                    let is_selected = &active_user_id == uid;
                                                    if !did_find_selected_seller && is_selected { did_find_selected_seller=true; }
                                                    html! {
                                                        <option value={uid.clone()} selected={is_selected}>
                                                           {user_info.name.clone()}
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

#[function_component(Reports)]
pub fn reports_page() -> Html {
    let current_settings = use_state_eq(load_report_settings);

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
            let report_view = gloo::utils::document()
                .get_element_by_id("reportViewSettingsDlgViewSelection")
                .and_then(|t| t.dyn_into::<HtmlSelectElement>().ok())
                .unwrap()
                .value();

            let seller_id: String = if get_active_user().is_admin() {
                gloo::utils::document()
                    .get_element_by_id("reportViewSettingsDlgUserSelection")
                    .and_then(|t| t.dyn_into::<HtmlSelectElement>().ok())
                    .unwrap()
                    .value()
            } else {
                get_active_user().get_id()
            };

            let updated_settings = ReportViewSettings {
                current_view: ReportViews::from_str(&report_view).unwrap(),
                seller_id_filter: seller_id,
            };
            if let Err(err) = save_report_settings(&updated_settings) {
                log::error!("Failed saving report settings: {:#?}", err);
            }

            log::info!(
                "on_save_settings.  report view: {} seller: {}",
                &updated_settings.current_view,
                &updated_settings.seller_id_filter
            );

            current_settings.set(updated_settings);
        })
    };

    log::info!(
        "Report View Rendering.  report view: {} seller: {}",
        &current_settings.current_view,
        &current_settings.seller_id_filter
    );

    let do_show_current_seller = do_show_current_seller(&current_settings.current_view);

    html! {
        <div>
            <DeleteOrderDlg />
            <ReportsSettingsDlg id="reportViewSettingsDlg"
                onsave={on_save_settings} currentview={current_settings.current_view.clone()}/>
            <ChooseSpreadersDlg />
            <div class="col-xs-1 d-flex justify-content-center">
                <div class="card" style="width: 100%;">

                    <div class="card-body" id="cardReportBody">
                        <h6 class="card-title ps-2" id="orderCardTitle">
                            <ul class="list-group list-group-horizontal-sm">
                                <li class="list-group-item me-3">
                                    <label class="text-muted pe-2">{"Report View:"}</label>
                                    <div class="d-inline" id="reportViewLabel">
                                        {current_settings.current_view.to_string()}
                                    </div>
                                </li>
                                if get_active_user().is_admin() && do_show_current_seller {
                                    <li class="list-group-item" id="orderOwnerLabel">
                                        <label class="text-muted pe-2">{"Showing Orders for:"}</label>
                                        <div class="d-inline" id="reportViewOrderOwner">
                                            {current_settings.seller_id_filter.clone()}
                                        </div>
                                    </li>
                                }
                            </ul>
                            <div id="reportViewSettings" class="float-end">
                                <button type="button" class="btn reports-view-setting-btn" onclick={on_view_settings}
                                        data-bs-toggle="tooltip" data-bs-placement="left" title="Change Report View">
                                    <i class="bi bi-gear" fill="currentColor"></i>
                                </button>
                            </div>
                        </h6>

                        <div class="visually-hidden" id="orderLoadingSpinner">
                            <h2>{"Loading Report Data..."}</h2>
                            <span role="status" class="spinner-border ms-1"/>
                        </div>
                    </div>

                </div>
            </div>

            {
                match current_settings.current_view {
                    ReportViews::Quick=>html!{<QuickReportView seller={current_settings.seller_id_filter.clone()}/>},
                    ReportViews::Full=>html!{<FullReportView seller={current_settings.seller_id_filter.clone()}/>},
                    ReportViews::MoneyCollection=>html!{<MoneyCollectionReportView seller={current_settings.seller_id_filter.clone()}/>},
                    ReportViews::SpreadingJobs=>html!{<SpreadingJobsReportView seller={current_settings.seller_id_filter.clone()}/>},
                    ReportViews::UnfinishedSpreadingJobs=>html!{<SpreadingJobsUnfinishedReportView />},
                    ReportViews::OrderVerification=>html!{<OrderVerificationView seller={current_settings.seller_id_filter.clone()}/>},
                    ReportViews::Deliveries=>html!{<DeliveriesReportView />},
                    ReportViews::DistributionPoints=>html!{<DistributionPointsReportView />},
                    ReportViews::SellMap=>html!{<SellMapReportView />},
                    _=>html!{<h6>{"Not Yet Implemented"}</h6>},
                }
            }

        </div>
    }
}
