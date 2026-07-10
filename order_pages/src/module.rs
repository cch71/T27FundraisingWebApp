use crate::order_utils::save_order_form_if_present;
use crate::pages::{OrderDonations, OrderForm, OrderProducts};
use std::cell::RefCell;
use wasm_bindgen::prelude::*;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Clone, Routable, PartialEq, Debug)]
pub enum OrderRoutes {
    #[at("/order")]
    OrderForm,
    #[at("/orderproducts")]
    OrderProducts,
    #[at("/orderdonations")]
    OrderDonations,
    // Every other path belongs to the shell; render nothing while it unmounts us
    #[not_found]
    #[at("/404")]
    NotFound,
}

fn route_switch(route: OrderRoutes) -> Html {
    // Save the in-progress order form before switching between order pages
    save_order_form_if_present();

    match route {
        OrderRoutes::OrderForm => html! {<OrderForm/>},
        OrderRoutes::OrderProducts => html! {<OrderProducts/>},
        OrderRoutes::OrderDonations => html! {<OrderDonations/>},
        OrderRoutes::NotFound => html! {},
    }
}

#[component(OrderModuleApp)]
fn order_module_app() -> Html {
    html! {
        <BrowserRouter>
            <Switch<OrderRoutes> render={route_switch} />
        </BrowserRouter>
    }
}

thread_local! {
    static APP_HANDLE: RefCell<Option<AppHandle<OrderModuleApp>>> = const { RefCell::new(None) };
    static SAVE_ON_NAV: RefCell<Option<gloo::events::EventListener>> = const { RefCell::new(None) };
}

/// Called by the shell's module loader to render this module into `root_id`
#[wasm_bindgen]
pub async fn mount(root_id: String) -> Result<(), JsValue> {
    data_model::init_page_module()
        .await
        .map_err(|err| JsValue::from_str(&err))?;

    // Cross-module navigation dispatches a synchronous popstate event (see
    // js/src/js/nav.js) before yew tears down the DOM, so this listener is
    // the last reliable chance to persist in-progress order form edits when
    // the user navigates away from the order pages.
    SAVE_ON_NAV.with(|l| {
        if l.borrow().is_none() {
            let listener = gloo::events::EventListener::new(
                &gloo::utils::window(),
                "popstate",
                |_| save_order_form_if_present(),
            );
            l.borrow_mut().replace(listener);
        }
    });

    unmount();
    let root = gloo::utils::document()
        .get_element_by_id(&root_id)
        .ok_or_else(|| JsValue::from_str(&format!("Module root element '{root_id}' not found")))?;
    let handle = yew::Renderer::<OrderModuleApp>::with_root(root).render();
    APP_HANDLE.with(|h| h.borrow_mut().replace(handle));
    Ok(())
}

/// Called by the shell's module loader before another module takes the stage
#[wasm_bindgen]
pub fn unmount() {
    // Persist any in-progress order edits before the form leaves the DOM
    save_order_form_if_present();
    if let Some(handle) = APP_HANDLE.with(|h| h.borrow_mut().take()) {
        handle.destroy();
    }
}
