pub(crate) use crate::components::delete_report_order_dlg::*;
pub(crate) use crate::components::report_spreaders_dlg::*;
use data_model::*;
use tracing::info;
use wasm_bindgen::JsCast;
use web_sys::{Element, HtmlElement, MouseEvent};
use yew::prelude::*;
use yew_router::prelude::*;

/////////////////////////////////////////////////
pub(crate) fn on_view_or_edit_from_rpt(evt: MouseEvent, history: Navigator) {
    evt.prevent_default();
    evt.stop_propagation();
    let btn_elm = evt
        .target()
        .and_then(|t| t.dyn_into::<Element>().ok())
        .and_then(|t| {
            if t.node_name() == "I" {
                t.parent_element()
            } else {
                Some(t)
            }
        })
        .unwrap();
    let order_id = btn_elm
        .dyn_into::<HtmlElement>()
        .ok()
        .and_then(|t| t.dataset().get("orderid"))
        .unwrap();
    wasm_bindgen_futures::spawn_local(async move {
        info!("on_view_or_edit_order: {order_id}");
        if let Err(err) = load_active_order_from_db(&order_id).await {
            gloo::dialogs::alert(&format!("Failed to load order: {order_id}: Err: {err:#?}"));
        }
        history.push(&AppRoutes::OrderForm);
    });
}

/////////////////////////////////////////////////
/////////////////////////////////////////////////
#[derive(Properties, PartialEq, Clone, Debug)]
pub(crate) struct ReportActionButtonsProps {
    pub(crate) showspreading: bool,
    pub(crate) isreadonly: bool,
    pub(crate) orderid: String,
    pub(crate) ondeleteorder: Callback<MouseEvent>,
    pub(crate) onvieworder: Callback<MouseEvent>,
    pub(crate) oneditorder: Callback<MouseEvent>,
    #[prop_or_default]
    pub(crate) oneditspreading: Callback<MouseEvent>,
    #[prop_or_default]
    pub(crate) spreaders: AttrValue,
}

#[component(ReportActionButtons)]
pub(crate) fn report_action_buttons(props: &ReportActionButtonsProps) -> Html {
    html! {
        <>
        if props.showspreading {
            <button type="button" class="btn btn-outline-info me-1 order-spread-btn"
                onclick={props.oneditspreading.clone()}
                data-orderid={props.orderid.clone()}
                data-spreaders={props.spreaders.as_str().to_string()}
                data-bs-toggle="tooltip" title="Select Spreaders" data-bs-placement="left">
                 <i class="bi bi-layout-wtf" fill="currentColor" />
            </button>
        }

        if props.isreadonly {
            <button type="button" class="btn btn-outline-info me-1 order-view-btn"
                onclick={props.onvieworder.clone()} data-orderid={props.orderid.clone()}
                data-bs-toggle="tooltip" title="View Order" data-bs-placement="left">
                 <i class="bi bi-eye" fill="currentColor" />
            </button>
        } else {
            <button type="button" class="btn btn-outline-info me-1 order-edt-btn"
                onclick={props.oneditorder.clone()} data-orderid={props.orderid.clone()}
                data-bs-toggle="tooltip" title="Edit Order" data-bs-placement="left">
                 <i class="bi bi-pencil" fill="currentColor" />
            </button>
            <button type="button" class="btn btn-outline-danger order-del-btn"
                onclick={props.ondeleteorder.clone()} data-orderid={props.orderid.clone()}
                data-bs-toggle="tooltip" title="Delete Order" data-bs-placement="left">
                <i class="bi bi-trash" fill="currentColor" />
            </button>
        }

        </>
    }
}
