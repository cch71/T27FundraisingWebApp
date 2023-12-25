use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "/src/js/google_charts.js")]
extern "C" {
    #[wasm_bindgen(catch)]
    fn drawGoogleChart(params: JsValue) -> Result<(), JsValue>;
}

pub fn draw_google_chart(params: &std::collections::HashMap<String, f32>) {
    // log::info!("Google Chart Params: {:#?}", params);
    drawGoogleChart(serde_wasm_bindgen::to_value(params).unwrap()).unwrap()
}
