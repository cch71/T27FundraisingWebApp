use tracing::error;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "/src/js/google_charts.js")]
extern "C" {
    #[wasm_bindgen(catch)]
    fn drawGoogleChart(params: JsValue) -> Result<(), JsValue>;
}

pub fn draw_google_chart(params: &std::collections::HashMap<String, f32>) {
    // log::info!("Google Chart Params: {:#?}", params);
    // The JS side references the third-party `google` global, which is absent
    // offline or if the loader script is blocked. Log instead of panicking (a
    // wasm panic would abort the whole app) — the `catch` binding is pointless
    // if we then unwrap it.
    let params = match serde_wasm_bindgen::to_value(params) {
        Ok(params) => params,
        Err(err) => {
            error!("Failed to serialize chart params: {err:?}");
            return;
        }
    };
    if let Err(err) = drawGoogleChart(params) {
        error!("Failed to draw google chart: {err:?}");
    }
}
