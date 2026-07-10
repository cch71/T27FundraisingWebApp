use crate::pages::{CloseoutFundraiser, FrConfigEditor};
use std::cell::RefCell;
use wasm_bindgen::prelude::*;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Clone, Routable, PartialEq, Debug)]
pub enum AdminRoutes {
    #[at("/frcloseout")]
    FundraiserCloseout,
    #[at("/frcconfig")]
    FrConfigEditor,
    // Every other path belongs to the shell; render nothing while it unmounts us
    #[not_found]
    #[at("/404")]
    NotFound,
}

fn route_switch(route: AdminRoutes) -> Html {
    match route {
        AdminRoutes::FundraiserCloseout => html! {<CloseoutFundraiser/>},
        AdminRoutes::FrConfigEditor => html! {<FrConfigEditor/>},
        AdminRoutes::NotFound => html! {},
    }
}

#[component(AdminModuleApp)]
fn admin_module_app() -> Html {
    html! {
        <BrowserRouter>
            <Switch<AdminRoutes> render={route_switch} />
        </BrowserRouter>
    }
}

thread_local! {
    static APP_HANDLE: RefCell<Option<AppHandle<AdminModuleApp>>> = const { RefCell::new(None) };
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
    let handle = yew::Renderer::<AdminModuleApp>::with_root(root).render();
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
