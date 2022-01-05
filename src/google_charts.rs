use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "/src/js/google_charts.js")]
extern "C" {
    #[wasm_bindgen(catch)]
    fn drawGoogleChart(params: &JsValue) -> Result<(), JsValue>;
}

pub(crate) fn draw_google_chart(params: &serde_json::Value) {
    drawGoogleChart(&JsValue::from_serde(params).unwrap()).unwrap()
}
