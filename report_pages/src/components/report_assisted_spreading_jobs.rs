use crate::components::report_loading_spinny::*;
use data_model::*;
use js::datatable::*;
use yew::prelude::*;

/////////////////////////////////////////////////
/////////////////////////////////////////////////
#[derive(Properties, PartialEq, Clone, Debug)]
pub(crate) struct SpreadingAssistJobsReportViewProps {
    pub(crate) spreader: String,
}
#[function_component(SpreadingAssistJobsReportView)]
pub(crate) fn report_spreading_assist_jobs(props: &SpreadingAssistJobsReportViewProps) -> Html {
    let report_state = use_state(|| ReportViewState::IsLoading);
    let datatable: std::rc::Rc<std::cell::RefCell<Option<DataTable>>> = use_mut_ref(|| None);
    let current_view_spreader = use_mut_ref(|| props.spreader.clone());

    if (*current_view_spreader.borrow()).ne(&props.spreader) {
        log::info!(
            "Current Seller doesn't match original seller: {}:{}",
            *current_view_spreader.borrow(),
            &props.spreader
        );
        *current_view_spreader.borrow_mut() = props.spreader.clone();
        report_state.set(ReportViewState::IsLoading);
    } else {
        log::info!("Current Spreader: {}", &props.spreader);
    }

    {
        let report_state = report_state.clone();
        let spreader = props.spreader.to_string();
        use_effect(move || {
            match &*report_state {
                ReportViewState::IsLoading => {
                    wasm_bindgen_futures::spawn_local(async move {
                        log::info!(
                            "Downloading Spreading Jobs Report View Data for {}",
                            &spreader
                        );
                        let seller = if spreader.eq(ALL_USERS_TAG) {
                            None
                        } else {
                            Some(spreader)
                        };
                        let resp = get_spreading_assist_jobs_report_data(seller.as_ref())
                            .await
                            .unwrap();
                        log::info!("Report Data has been downloaded");
                        report_state.set(ReportViewState::ReportHtmlGenerated(resp));
                    });
                }
                ReportViewState::ReportHtmlGenerated(_) => {
                    // log::info!("Setting DataTable");
                    *datatable.borrow_mut() = get_datatable(&serde_json::json!({
                        "reportType": "spreadingAssistJobs",
                        "id": ".data-table-report table",
                        "showOrderOwner": true,
                        "isMulchOrder": true,
                    }));
                }
            };

            || {}
        });
    }

    match &*report_state {
        ReportViewState::IsLoading => html! { <ReportLoadingSpinny/> },
        ReportViewState::ReportHtmlGenerated(orders) => {
            let header_footer = html! {
                <tr>
                    <th>{"OrderId"}</th>
                    <th>{"Address"}</th>
                    <th>{"Neighborhood"}</th>
                    <th>{"Order Owner"}</th>
                </tr>
            };
            html! {
                <div class="data-table-report">
                    <table class="display responsive nowrap collapsed" role="grid" cellspacing="0" width="100%">
                        <thead>
                            {header_footer.clone()}
                        </thead>
                        <tbody>
                        {
                            orders.iter().map(|v|{
                                let address = format!("{} {}",
                                    v["customer"]["addr1"].as_str().unwrap(),
                                    v["customer"]["addr2"].as_str().unwrap_or("")).trim().to_string();
                                let uid = v["ownerId"].as_str().unwrap();
                                html!{
                                    <tr>
                                        <td>{v["orderId"].as_str().unwrap()}</td>
                                        <td>{&address}</td>
                                        <td>{v["customer"]["neighborhood"].as_str().unwrap_or("")}</td>
                                        <td>{get_username_from_id(uid).map_or(uid.to_string(), |v|format!("{v}[{uid}]"))}</td>
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
        }
    }
}
