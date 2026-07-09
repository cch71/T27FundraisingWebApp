use tracing::error;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = bootstrap)]
    #[derive(Clone, Debug)]
    pub type Modal;

    #[wasm_bindgen(constructor, js_namespace = bootstrap)]
    pub fn new(e: web_sys::Element) -> Modal;

    #[wasm_bindgen(method, js_namespace = bootstrap)]
    pub fn show(this: &Modal);

    #[wasm_bindgen(method, js_namespace = bootstrap)]
    pub fn hide(this: &Modal);

    #[wasm_bindgen(method, js_namespace = bootstrap)]
    pub fn toggle(this: &Modal);

}

pub fn get_modal_by_id(id: &str) -> Option<Modal> {
    gloo::utils::document()
        .get_element_by_id(id)
        .map(Modal::new)
}

#[wasm_bindgen(module = "/src/js/bootstrap_helpers.js")]
extern "C" {
    #[wasm_bindgen(catch)]
    fn modalOp(id: &str, op: &str) -> Result<(), JsValue>;
}

pub fn modal_op(id: &str, op: &str) {
    // The JS helper uses the jQuery `$` global (from the DataTables bundle); if
    // that bundle failed to load, catch the exception instead of letting it
    // unwind uncaught through the wasm boundary mid-event-handler.
    if let Err(err) = modalOp(id, op) {
        error!("modalOp({id}, {op}) failed: {err:?}");
    }
}
