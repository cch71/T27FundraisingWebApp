use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "/src/js/module_loader.js")]
extern "C" {
    /// Fetches (first use only) and mounts the named page module's wasm
    /// binary into the element with id `root_id`.
    #[wasm_bindgen(js_name = loadModule, catch)]
    pub async fn load_module(name: &str, root_id: &str) -> Result<(), JsValue>;

    /// Unmounts the named page module if it is mounted. The module's wasm
    /// instance is kept so remounting is instant.
    #[wasm_bindgen(js_name = unloadModule)]
    pub fn unload_module(name: &str);
}
