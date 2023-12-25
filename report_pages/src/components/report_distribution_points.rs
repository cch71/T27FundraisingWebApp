use crate::components::report_loading_spinny::*;
use data_model::*;
use js::datatable::*;
use yew::prelude::*;

/////////////////////////////////////////////////
/////////////////////////////////////////////////
#[function_component(DistributionPointsReportView)]
pub(crate) fn report_distribution_points_view() -> Html {
    let report_state = use_state(|| ReportViewState::IsLoading);
    let datatable: std::rc::Rc<std::cell::RefCell<Option<DataTable>>> = use_mut_ref(|| None);

    {
        let report_state = report_state.clone();
        use_effect(move || {
            match &*report_state {
                ReportViewState::IsLoading => {
                    wasm_bindgen_futures::spawn_local(async move {
                        log::info!("Downloading Distribution Points Report View Data");
                        let resp = get_distribution_points_report_data().await.unwrap();
                        log::info!("Report Data has been downloaded");
                        report_state.set(ReportViewState::ReportHtmlGenerated(resp));
                    });
                }
                ReportViewState::ReportHtmlGenerated(resp) => {
                    log::info!("Setting DataTable");
                    if datatable.borrow().is_none() {
                        *datatable.borrow_mut() = get_datatable(&serde_json::json!({
                            "reportType": "distributionPoints",
                            "id": ".data-table-report table",
                            "distPoints": resp[0]["distPoints"],
                            "showOrderOwner": false,
                            "isMulchOrder": true
                        }));
                    }
                }
            };

            || {}
        });
    }

    match &*report_state {
        ReportViewState::IsLoading => html! { <ReportLoadingSpinny/> },
        ReportViewState::ReportHtmlGenerated(resp) => {
            use std::collections::BTreeMap;
            let dist_points: Vec<String> =
                serde_json::from_value(resp[0]["distPoints"].clone()).unwrap();
            let delivery_id_to_dist_points_map: BTreeMap<u64, BTreeMap<String, u64>> =
                serde_json::from_value(resp[0]["deliveryIdMap"].clone()).unwrap();

            let header_footer = html! {
                <tr>
                    <th>{"Delivery Date"}</th>
                    <th>{"Total Bags"}</th>
                    {
                        dist_points.iter().map(|v| html!{
                            <th>{v.clone()}</th>
                        }).collect::<Html>()
                    }
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
                            delivery_id_to_dist_points_map.iter().map(|(delivery_id, dist_pt_map)|{
                                let delivery_date = get_delivery_date(&(*delivery_id as u32));

                                html!{
                                    <tr>
                                        <td>{delivery_date}</td>
                                        <td>{dist_pt_map.get("TotalBagSummary").unwrap_or(&0).to_string()}</td>
                                        {
                                            dist_points.iter().map(|v| html!{
                                                <td>{dist_pt_map.get(v).unwrap_or(&0).to_string()}</td>
                                            }).collect::<Html>()
                                        }
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
