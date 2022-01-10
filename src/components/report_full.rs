use yew::prelude::*;
use yew_router::prelude::*;
use web_sys::{ MouseEvent};
use crate::datatable::*;
use crate::data_model::*;
use crate::currency_utils::*;
use crate::components::action_report_buttons::{
    on_delete_order_from_rpt,
    on_view_or_edit_from_rpt,
    on_edit_spreading_from_rpt,
    ReportActionButtons,
};
use crate::components::report_loading_spinny::*;


/////////////////////////////////////////////////
/////////////////////////////////////////////////
#[derive(Properties, PartialEq, Clone, Debug)]
pub(crate) struct FullReportViewProps {
    pub(crate) seller: String,
}
#[function_component(FullReportView)]
pub(crate) fn report_full_view(props: &FullReportViewProps) -> Html {
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
        let datatable = datatable.clone();
        Callback::from(move |evt: MouseEvent| {
            on_edit_spreading_from_rpt(evt, datatable.clone());
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
                                let spreaders: Vec<String> = serde_json::from_value(v["spreaders"].clone()).unwrap_or(Vec::new());
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
                                        <td>{spreaders.join(",")}</td>
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

