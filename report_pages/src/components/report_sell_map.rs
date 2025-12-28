use crate::components::report_loading_spinny::*;
use data_model::*;
use js::leaflet::*;
use serde::{Deserialize, Serialize};
use tracing::info;
use yew::prelude::*;

#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq)]
pub struct Point(pub f64, pub f64);

const SJV: Point = Point(30.5461096, -97.6723646);
const SELL_MAP_ID: &str = "sellMap";

/////////////////////////////////////////////////
/////////////////////////////////////////////////
#[component(SellMapReportView)]
pub(crate) fn report_sell_view() -> Html {
    let report_state = use_state(|| ReportViewState::IsLoading);
    let sell_map: std::rc::Rc<std::cell::RefCell<Option<Map>>> = use_mut_ref(|| None);

    {
        let report_state = report_state.clone();
        let sell_map = sell_map.clone();
        use_effect(move || {
            match &*report_state {
                ReportViewState::IsLoading => {
                    wasm_bindgen_futures::spawn_local(async move {
                        info!("Downloading Geo Location data");
                        let resp = get_sales_geojson().await.unwrap();
                        info!("Report Data has been downloaded");
                        report_state.set(ReportViewState::ReportHtmlGenerated(vec![resp]));
                    });
                }
                ReportViewState::ReportHtmlGenerated(json_list) => {
                    info!("Handling ReportHtmlGenerated");
                    if sell_map.borrow().is_none() {
                        *sell_map.borrow_mut() = create_sell_map(&serde_json::json!({
                            "id": SELL_MAP_ID,
                            "geoJson": json_list[0],
                            "centerPt": SJV,
                        }));
                    }
                }
            };

            || {}
        });
    }

    match &*report_state {
        ReportViewState::IsLoading => html! { <ReportLoadingSpinny/> },
        ReportViewState::ReportHtmlGenerated(_) => {
            html! {
                <div class="sale-map-container">
                    <div id={SELL_MAP_ID} />
                </div>
            }
        }
    }
}
