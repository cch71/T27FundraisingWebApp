use std::str::FromStr;
use wasm_bindgen::JsCast;
use web_sys::{Event, HtmlSelectElement};
use yew::prelude::*;

use data_model::*;

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
#[derive(Properties, PartialEq, Clone, Debug)]
struct ReportViewSettingsSelectionProp {
    onchange: Callback<ReportViewSettings>,
    current: ReportViewSettings,
    showseller: bool,
}
#[function_component(ReportViewSettingsSelection)]
fn reports_selections(props: &ReportViewSettingsSelectionProp) -> Html {
    let on_view_selection_change = {
        let props = props.clone();
        Callback::from(move |evt: Event| {
            if let Some(v) = evt
                .target()
                .and_then(|t| t.dyn_into::<HtmlSelectElement>().ok())
            {
                let selected_view = ReportViews::from_str(&v.value()).unwrap();
                props.onchange.emit(ReportViewSettings {
                    current_view: selected_view,
                    seller_id_filter: props.current.seller_id_filter.clone(),
                })
            };
        })
    };

    let on_userid_selection_change = {
        let props = props.clone();
        Callback::from(move |evt: Event| {
            if let Some(v) = evt
                .target()
                .and_then(|t| t.dyn_into::<HtmlSelectElement>().ok())
            {
                let selected_userid = v.value();
                props.onchange.emit(ReportViewSettings {
                    current_view: props.current.current_view.clone(),
                    seller_id_filter: selected_userid,
                })
            };
        })
    };

    html! {

        <ul class="list-group list-group-horizontal-sm">
            <li class="list-group-item me-3">
                <label class="text-muted pe-2">{"Report View:"}</label>
                <div class="d-inline">
                   <select class="form-select" onchange={on_view_selection_change}>
                   {
                       get_allowed_report_views().iter().map(|v|{
                           let is_selected = &props.current.current_view == v;
                           html! {
                               <option value={v.to_string()} selected={is_selected}>
                                  {v.to_string()}
                               </option>
                           }
                       }).collect::<Html>()
                   }
                   </select>
                </div>
            </li>
            if props.showseller {
                <li class="list-group-item">
                    <label class="text-muted pe-2">{"Showing Orders for:"}</label>
                    <div class="d-inline">
                       <select class="form-select" onchange={on_userid_selection_change}>
                       {
                           get_users().iter().map(|(uid,user_info)|{
                               let is_selected = &props.current.seller_id_filter == uid;
                               html! {
                                   <option value={uid.clone()} selected={is_selected}>
                                      {user_info.name.clone()}
                                   </option>
                               }
                           }).collect::<Html>()
                       }
                           <option value={ALL_USERS_TAG}>{"Show All Users"}</option>
                       </select>
                        // {
                        //     if ALL_USERS_TAG == current_settings.seller_id_filter {
                        //         Some("All Users".to_string())
                        //     } else {
                        //         get_username_from_id(&current_settings.seller_id_filter)
                        //     }
                        // }
                    </div>
                </li>
            }
        </ul>
    }
}

/////////////////////////////////////////////////
/////////////////////////////////////////////////
#[function_component(Reports)]
pub fn reports_page() -> Html {
    let current_settings = use_state_eq(load_report_settings);

    let on_report_selection_change = {
        let current_settings = current_settings.clone();
        Callback::from(move |updated_settings: ReportViewSettings| {
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

    let do_show_current_seller = get_active_user().is_admin()
        && do_show_current_seller(&current_settings.current_view)
        && do_show_current_seller(&current_settings.current_view);

    html! {
        <div>
            <DeleteOrderDlg />
            <ChooseSpreadersDlg />
            <div class="col-xs-1 d-flex justify-content-center">
                <div class="card" style="width: 100%;">

                    <div class="card-body" id="cardReportBody">
                        <h6 class="card-title ps-2" id="orderCardTitle">
                            <ReportViewSettingsSelection
                                showseller={do_show_current_seller}
                                onchange={on_report_selection_change}
                                current={(*current_settings).clone()}/>
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
