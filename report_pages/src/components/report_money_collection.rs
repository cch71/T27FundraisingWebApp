use crate::components::report_loading_spinny::*;
use data_model::*;
use js::datatable::*;
use tracing::info;
use yew::prelude::*;

/////////////////////////////////////////////////
/////////////////////////////////////////////////
#[derive(Properties, PartialEq, Clone, Debug)]
pub(crate) struct MoneyCollectionReportViewProps {
    pub(crate) seller: String,
}
#[function_component(MoneyCollectionReportView)]
pub(crate) fn report_money_collection_view(props: &MoneyCollectionReportViewProps) -> Html {
    let report_state = use_state(|| ReportViewState::IsLoading);
    let datatable: std::rc::Rc<std::cell::RefCell<Option<DataTable>>> = use_mut_ref(|| None);
    let current_view_seller = use_mut_ref(|| props.seller.clone());

    if (*current_view_seller.borrow()).ne(&props.seller) {
        info!(
            "Current Seller doesn't match original seller: {}:{}",
            *current_view_seller.borrow(),
            &props.seller
        );
        *current_view_seller.borrow_mut() = props.seller.clone();
        report_state.set(ReportViewState::IsLoading);
    } else {
        info!("Current Seller: {}", &props.seller);
    }

    {
        let report_state = report_state.clone();
        let seller = props.seller.to_string();
        use_effect(move || {
            match &*report_state {
                ReportViewState::IsLoading => {
                    wasm_bindgen_futures::spawn_local(async move {
                        info!(
                            "Downloading Money Collection Report View Data for {}",
                            &seller
                        );
                        let seller = if seller.eq(ALL_USERS_TAG) {
                            None
                        } else {
                            Some(seller)
                        };
                        let resp = get_money_collection_report_data(seller.as_ref())
                            .await
                            .unwrap();
                        info!("Report Data has been downloaded");
                        report_state.set(ReportViewState::ReportHtmlGenerated(resp));
                    });
                }
                ReportViewState::ReportHtmlGenerated(_) => {
                    info!("Setting DataTable");
                    *datatable.borrow_mut() = get_datatable(&serde_json::json!({
                        "reportType": "moneyCollection",
                        "id": ".data-table-report table",
                        "showOrderOwner": seller.ne(&get_active_user().get_id()),
                        "isMulchOrder": true
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
                    <th>{"Order Owner"}</th>
                    <th>{"Delivery Date"}</th>
                    <th>{"Total From Checks"}</th>
                    <th>{"Total From Cash"}</th>
                    <th>{"Total"}</th>
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
                                let (delivery_date, delivery_id) = match v["deliveryId"].as_u64() {
                                    Some(delivery_id) => (get_delivery_date(&(delivery_id as u32)), delivery_id.to_string()),
                                    None => ("Donation".to_string(), "Donation".to_string()),
                                };
                                let uid = v["ownerId"].as_str().unwrap();
                                html!{
                                    <tr>
                                        <td>{get_username_from_id(uid).map_or(uid.to_string(), |v|format!("{v}[{uid}]"))}</td>
                                        <td data-deliveryid={delivery_id}>{delivery_date}</td>
                                        <td>{to_money_str(v["amountTotalFromChecksCollected"].as_str())}</td>
                                        <td>{to_money_str(v["amountTotalFromCashCollected"].as_str())}</td>
                                        <td>{to_money_str(v["amountTotalCollected"].as_str())}</td>
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
