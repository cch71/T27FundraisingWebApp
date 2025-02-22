use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen]
    pub type DataTable;

    #[wasm_bindgen(constructor)]
    pub fn new_0(id: &str) -> DataTable;

    #[wasm_bindgen(constructor)]
    pub fn new_1(id: &str, init: &JsValue) -> DataTable;
}

#[wasm_bindgen(module = "/src/js/datatable.js")]
extern "C" {
    #[wasm_bindgen(catch)]
    fn getDataTable(params: &JsValue) -> Result<DataTable, JsValue>;

    #[wasm_bindgen(catch)]
    fn removeRowWithTr(dt: &JsValue, tr: &web_sys::Node) -> Result<(), JsValue>;

    #[wasm_bindgen(catch)]
    fn setSpreadersWithTr(dt: &JsValue, tr: &web_sys::Node, spreaders: &str)
    -> Result<(), JsValue>;
}

pub fn get_datatable(params: &serde_json::Value) -> Option<DataTable> {
    //log::info!("Get Data Table: {:#?}", &params);
    getDataTable(&serde_wasm_bindgen::to_value(params).unwrap()).ok()
}

pub fn remove_row_with_tr(dt: &JsValue, tr: &web_sys::Node) -> Result<(), JsValue> {
    removeRowWithTr(dt, tr)
}

pub fn set_spreaders_with_tr(
    dt: &JsValue,
    tr: &web_sys::Node,
    spreaders: &str,
) -> Result<(), JsValue> {
    setSpreadersWithTr(dt, tr, spreaders)
}
