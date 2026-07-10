use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "/src/js/nav.js")]
extern "C" {
    /// Navigates the SPA to `path`, waking the shell router and any
    /// mounted module router. Use for all cross-module navigation;
    /// yew-router `Link`/`Navigator` only notify the router instance
    /// inside the same wasm binary.
    #[wasm_bindgen(js_name = navigateTo)]
    pub fn navigate_to(path: &str);
}
