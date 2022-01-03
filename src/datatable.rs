use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen]
    pub type DataTable;

    #[wasm_bindgen(constructor)]
    pub fn new_0(id: &str) -> DataTable;
    //pub fn new(e: web_sys::Element) -> DataTable;

    #[wasm_bindgen(constructor)]
    pub fn new_1(id: &str, init: &JsValue) -> DataTable;
}

// pub fn get_datatable(id: &str, init: Option<serde_json::Value>) -> Option<DataTable> {
//     //gloo_utils::document().get_element_by_id(id).and_then(|v| Some(DataTable::new(v)))
//     if let Some(init) = init {
//         Some(DataTable::new_1(id, &JsValue::from_serde(&init).unwrap()))
//     } else {
//         Some(DataTable::new_0(id))
//     }
// }

#[wasm_bindgen(module = "/src/js/datatable.js")]
extern "C" {
    #[wasm_bindgen(catch)]
    fn getQuickViewReportDataTable(id: &str) -> Result<DataTable, JsValue>;
}

pub(crate) fn get_quick_view_report_datatable(id: &str) -> Option<DataTable> {
    getQuickViewReportDataTable(id).ok()
}
