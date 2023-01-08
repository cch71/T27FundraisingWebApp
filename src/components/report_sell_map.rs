use yew::prelude::*;
use serde::{Deserialize, Serialize};
use crate::data_model::*;
use crate::leaflet::*;
use crate::components::report_loading_spinny::*;

#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq)]
pub struct Point(pub f64, pub f64);

const SJV: Point = Point(30.5461096, -97.6723646);
const SELLMAPID: &str = "sellMap";

/////////////////////////////////////////////////
/////////////////////////////////////////////////
#[function_component(SellMapReportView)]
pub(crate) fn report_sell_view() -> Html {
    let report_state = use_state(||ReportViewState::IsLoading);
    let sell_map: std::rc::Rc<std::cell::RefCell<Option<Map>>> = use_mut_ref(|| None);

    {
        let report_state = report_state.clone();
        let sell_map = sell_map.clone();
        use_effect(move || {
            match &*report_state {
                ReportViewState::IsLoading=>{
                    wasm_bindgen_futures::spawn_local(async move {
                        log::info!("Downloading Geo Location data");
                        let resp = get_sales_geojson().await.unwrap();
                        log::info!("Report Data has been downloaded");
                        report_state.set(ReportViewState::ReportHtmlGenerated(resp));
                    });
                },
                ReportViewState::ReportHtmlGenerated(geojson) => {
                    log::info!("Handling ReportHtmlGenerated");
                    if sell_map.borrow().is_none() {
                        *sell_map.borrow_mut() = create_sell_map(&serde_json::json!({
                            "id": SELLMAPID,
                            "geoJson": geojson,
                            "centerPt": SJV,
                        }));
                    }
                },
            };

            ||{}
        });
    }

    match &*report_state {
        ReportViewState::IsLoading => html! { <ReportLoadingSpinny/> },
        ReportViewState::ReportHtmlGenerated(_) => {
            html!{
                <div class="container">
                    <div id={SELLMAPID} />
                </div>
            }
        },
    }
}

