use crate::pages::timecards::Timecards;
use std::cell::RefCell;
use wasm_bindgen::prelude::*;
use yew::prelude::*;

thread_local! {
    static APP_HANDLE: RefCell<Option<AppHandle<Timecards>>> = const { RefCell::new(None) };
}

/// Called by the shell's module loader to render this module into `root_id`
#[wasm_bindgen]
pub async fn mount(root_id: String) -> Result<(), JsValue> {
    data_model::init_page_module()
        .await
        .map_err(|err| JsValue::from_str(&err))?;

    unmount();
    let root = gloo::utils::document()
        .get_element_by_id(&root_id)
        .ok_or_else(|| JsValue::from_str(&format!("Module root element '{root_id}' not found")))?;
    let handle = yew::Renderer::<Timecards>::with_root(root).render();
    APP_HANDLE.with(|h| h.borrow_mut().replace(handle));
    Ok(())
}

/// Called by the shell's module loader before another module takes the stage
#[wasm_bindgen]
pub fn unmount() {
    if let Some(handle) = APP_HANDLE.with(|h| h.borrow_mut().take()) {
        handle.destroy();
    }
}
