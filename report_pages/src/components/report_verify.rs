use crate::components::action_report_buttons::{
    on_delete_order_from_rpt, on_view_or_edit_from_rpt, ReportActionButtons,
};
use crate::components::report_loading_spinny::*;
use data_model::*;
use js::datatable::*;
use web_sys::MouseEvent;
use yew::prelude::*;
use yew_router::prelude::*;

/////////////////////////////////////////////////
/////////////////////////////////////////////////
#[derive(Properties, PartialEq, Clone, Debug)]
pub(crate) struct OrderVerificationViewProps {
    pub(crate) seller: String,
}
#[function_component(OrderVerificationView)]
pub(crate) fn order_verification_view(props: &OrderVerificationViewProps) -> Html {
    let report_state = use_state(|| ReportViewState::IsLoading);
    let history = use_navigator().unwrap();
    // let is_fr_locked = is_fundraiser_locked();
    let datatable: std::rc::Rc<std::cell::RefCell<Option<DataTable>>> = use_mut_ref(|| None);
    let current_view_seller = use_mut_ref(|| props.seller.clone());

    if &*current_view_seller.borrow() != &props.seller {
        log::info!(
            "Current Seller doesn't match original seller: {}:{}",
            *current_view_seller.borrow(),
            &props.seller
        );
        *current_view_seller.borrow_mut() = props.seller.clone();
        report_state.set(ReportViewState::IsLoading);
    } else {
        log::info!("Current Seller: {}", &props.seller);
    }

    let on_delete_order = {
        let datatable = datatable.clone();
        Callback::from(move |evt: MouseEvent| {
            on_delete_order_from_rpt(evt, datatable.clone());
        })
    };
    let on_view_or_edit_order = {
        let history = history.clone();
        move |evt: MouseEvent| {
            on_view_or_edit_from_rpt(evt, history.clone());
        }
    };

    {
        let report_state = report_state.clone();
        let seller = props.seller.to_string();
        use_effect(move || {
            match &*report_state {
                ReportViewState::IsLoading => {
                    wasm_bindgen_futures::spawn_local(async move {
                        log::info!("Downloading Verification Report View Data for {}", &seller);
                        let seller = if &seller == ALL_USERS_TAG {
                            None
                        } else {
                            Some(seller)
                        };
                        let resp = get_order_verfification_report_data(seller.as_ref())
                            .await
                            .unwrap();
                        log::info!("Report Data has been downloaded");
                        report_state.set(ReportViewState::ReportHtmlGenerated(resp));
                    });
                }
                ReportViewState::ReportHtmlGenerated(_) => {
                    // log::info!("Setting DataTable");
                    if datatable.borrow().is_none() {
                        *datatable.borrow_mut() = get_datatable(&serde_json::json!({
                            "reportType": "verification",
                            "id": ".data-table-report table",
                            "showOrderOwner": true,
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
        ReportViewState::ReportHtmlGenerated(orders) => {
            let header_footer = html! {
                <tr>
                    <th>{"OrderId"}</th>
                    <th>{"Name"}</th>
                    <th>{"Delivery Date"}</th>
                    <th>{"Donations"}</th>
                    <th>{"Cash"}</th>
                    <th>{"Check"}</th>
                    <th>{"Check Numbers"}</th>
                    <th>{"Total Amount"}</th>
                    <th>{"Order Owner"}</th>
                    <th>{"Verified"}</th>
                    <th>{"Actions"}</th>
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
                            orders.into_iter().map(|v|{
                                let (delivery_date, delivery_id) = match v["deliveryId"].as_u64() {
                                    Some(delivery_id) => (get_delivery_date(&(delivery_id as u32)), delivery_id.to_string()),
                                    None => ("Donation".to_string(), "Donation".to_string()),
                                };
                                let is_readonly = is_order_from_report_data_readonly(&v);
                                let uid = v["ownerId"].as_str().unwrap();
                                html!{
                                    <tr>
                                        <td>{v["orderId"].as_str().unwrap()}</td>
                                        <td>{v["customer"]["name"].as_str().unwrap()}</td>
                                        <td data-deliveryid={delivery_id}>{delivery_date}</td>
                                        <td>{to_money_str(v["amountFromDonations"].as_str())}</td>
                                        <td>{to_money_str(v["amountFromCashCollected"].as_str())}</td>
                                        <td>{to_money_str(v["amountFromChecksCollected"].as_str())}</td>
                                        <td>{v["checkNumbers"].as_str().unwrap_or("")}</td>
                                        <td>{to_money_str(v["amountTotalCollected"].as_str())}</td>
                                        <td>{get_username_from_id(uid).map_or(uid.to_string(), |v|format!("{v}[{uid}]"))}</td>
                                        <td>{v["isVerified"].as_bool().unwrap_or(false).to_string()}</td>
                                        <td>
                                            <ReportActionButtons
                                                orderid={v["orderId"].as_str().unwrap().to_string()}
                                                showspreading={false}
                                                isreadonly={is_readonly}
                                                ondeleteorder={on_delete_order.clone()}
                                                onvieworder={on_view_or_edit_order.clone()}
                                                oneditorder={on_view_or_edit_order.clone()}
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
        }
    }
}
