use web_sys::js_sys::{encode_uri, encode_uri_component};
use crate::components::report_loading_spinny::*;
use data_model::*;
use js::datatable::*;
use yew::prelude::*;
use ToString;
use qrcode_generator::QrCodeEcc;

/////////////////////////////////////////////////
/////////////////////////////////////////////////
#[function_component(DeliveriesReportView)]
pub(crate) fn report_deliveries_view() -> Html {
    let report_state = use_state(|| ReportViewState::IsLoading);
    let datatable: std::rc::Rc<std::cell::RefCell<Option<DataTable>>> = use_mut_ref(|| None);

    {
        let report_state = report_state.clone();
        use_effect(move || {
            match &*report_state {
                ReportViewState::IsLoading => {
                    wasm_bindgen_futures::spawn_local(async move {
                        log::info!("Downloading Deliveries Report View Data");
                        let mut resp = get_deliveries_report_data().await.unwrap();
                        log::info!("Report Data has been downloaded");
                        resp.sort_by(|a, b| {
                            let a_delivery_id = a["deliveryId"].as_u64().unwrap_or(0);
                            let b_delivery_id = b["deliveryId"].as_u64().unwrap_or(0);
                            a_delivery_id.partial_cmp(&b_delivery_id).unwrap()
                        });
                        report_state.set(ReportViewState::ReportHtmlGenerated(resp));
                    });
                }
                ReportViewState::ReportHtmlGenerated(_) => {
                    log::info!("Setting DataTable");
                    *datatable.borrow_mut() = get_datatable(&serde_json::json!({
                        "reportType": "deliveries",
                        "id": ".data-table-report table",
                        "showOrderOwner": true,
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
                    <th>{"OrderId"}</th>
                    <th>{"Delivery Date"}</th>
                    <th>{"Name"}</th>
                    <th>{"Neighborhood"}</th>
                    <th>{"Address"}</th>
                    <th>{"Map"}</th>
                    <th>{"Map QrCode"}</th>
                    <th>{"Bags"}</th>
                    <th>{"Phone"}</th>
                    <th>{"Location"}</th>
                    <th>{"Notes"}</th>
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
                                let num_bags_sold: u64 = v["purchases"].as_array().unwrap_or(&Vec::new())
                                    .iter()
                                    .find(|&v| v["productId"].as_str().unwrap() == "bags")
                                    .map_or(0, |v| v["numSold"].as_u64().unwrap());

                                if num_bags_sold == 0 { return html!{}; }

                                let address = format!("{} {},{},{}",
                                    v["customer"]["addr1"].as_str().unwrap(),
                                    v["customer"]["addr2"].as_str().unwrap_or(""),
                                    v["customer"]["city"].as_str().unwrap_or(""),
                                    v["customer"]["zipcode"].as_u64().map_or("".to_string(), |v| v.to_string()));

                                let google_map_url: String = encode_uri(
                                    &format!("https://www.google.com/maps/search/?api=1&query={}",
                                    address)).into();

                                let google_map_url_qrcode = qrcode_generator::to_svg_to_string(
                                    &google_map_url,
                                    QrCodeEcc::Low, 100, None::<&str>).map_or(
                                        html!{},
                                        |v| {
                                            let src = format!("data:image/svg+xml;utf8,{}",
                                                encode_uri_component(&v));
                                            html!{
                                                <img src={src} />
                                            }
                                        }
                                   );

                                let (delivery_date, delivery_id) = match v["deliveryId"].as_u64() {
                                    Some(delivery_id) => (get_delivery_date(&(delivery_id as u32)), delivery_id.to_string()),
                                    None => return html!{}, // Donation order
                                };
                                let neighborhood = v["customer"]["neighborhood"].as_str().unwrap();
                                let dist_point = get_neighborhood(neighborhood)
                                    .map_or("".to_string(), |v|v.distribution_point.clone());
                                let uid = v["ownerId"].as_str().unwrap();
                                html!{
                                    <tr>
                                        <td>{v["orderId"].as_str().unwrap()}</td>
                                        <td data-deliveryid={delivery_id}>{delivery_date}</td>
                                        <td>{v["customer"]["name"].as_str().unwrap()}</td>
                                        <td>{neighborhood}</td>
                                        <td>{address}</td>
                                        <td><a href={google_map_url} target="_blank">{"map"}</a></td>
                                        <td>{google_map_url_qrcode}</td>
                                        <td>{num_bags_sold.to_string()}</td>
                                        <td>{v["customer"]["phone"].as_str().unwrap()}</td>
                                        <td>{dist_point}</td>
                                        <td>{v["specialInstructions"].as_str().unwrap_or("").to_string()}</td>
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
