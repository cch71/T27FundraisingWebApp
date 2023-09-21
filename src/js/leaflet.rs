use wasm_bindgen::prelude::*;
use serde::{Serialize};
use serde_wasm_bindgen::{Serializer};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen]
    pub type Map;
}

#[wasm_bindgen(module = "/src/js/leaflet.js")]
extern "C" {
    #[wasm_bindgen(catch)]
    fn createSellMap(params: &JsValue) -> Result<Map, JsValue>;
}

pub(crate) fn create_sell_map(params: &serde_json::Value) -> Option<Map> {
    //log::info!("Get Data Table: {:#?}", &params);
    let serializer = Serializer::new().serialize_maps_as_objects(true);
    let serialized_params = (*params).serialize(&serializer).unwrap();
    createSellMap(&serialized_params).ok()
}

