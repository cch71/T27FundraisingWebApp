use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = bootstrap)]
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
    gloo_utils::document().get_element_by_id(id).and_then(|v| Some(Modal::new(v)))
}
