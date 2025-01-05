use crate::components::report_loading_spinny::*;
use data_model::*;
use js::datatable::*;
use yew::prelude::*;

/////////////////////////////////////////////////
/////////////////////////////////////////////////
#[function_component(SpreadingJobsUnfinishedReportView)]
pub(crate) fn report_quick_view() -> Html {
    let report_state = use_state(|| ReportViewState::IsLoading);
    let datatable: std::rc::Rc<std::cell::RefCell<Option<DataTable>>> = use_mut_ref(|| None);

    {
        let report_state = report_state.clone();
        use_effect(move || {
            match &*report_state {
                ReportViewState::IsLoading => {
                    wasm_bindgen_futures::spawn_local(async move {
                        log::info!("Downloading Unfinished Spreading Jobs Report View Data");
                        let resp = get_unfinished_spreading_jobs_report_data().await.unwrap();
                        log::info!("Report Data has been downloaded");
                        report_state.set(ReportViewState::ReportHtmlGenerated(resp));
                    });
                }
                ReportViewState::ReportHtmlGenerated(_) => {
                    // log::info!("Setting DataTable");
                    if datatable.borrow().is_none() {
                        *datatable.borrow_mut() = get_datatable(&serde_json::json!({
                            "reportType": "spreadingJobsUnfinished",
                            "id": ".data-table-report table",
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
            let header_footer = html! {
                <tr>
                    <th>{"Order Owner"}</th>
                    <th>{"Name"}</th>
                    <th>{"Delivery Date"}</th>
                    <th>{"Bags Left To Spread"}</th>
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
                            resp.iter().map(|v|{
                                let owner_id = v["ownerId"].as_str().unwrap();
                                let (delivery_date, delivery_id) = match v["deliveryId"].as_u64() {
                                    Some(delivery_id) => (get_delivery_date(&(delivery_id as u32)), delivery_id.to_string()),
                                    None => ("N/A".to_string(), "N/A".to_string()),
                                };
                                html!{
                                    <tr>
                                        <td>{owner_id.to_string()}</td>
                                        <td>{get_username_from_id(owner_id).unwrap_or("".to_string())}</td>
                                        <td data-deliveryid={delivery_id}>{delivery_date}</td>
                                        <td>{v["bagsLeft"].as_u64().unwrap_or(0).to_string()}</td>
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
