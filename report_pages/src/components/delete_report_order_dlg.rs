use data_model::*;
use js::{bootstrap, datatable::*};
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{Element, HtmlButtonElement, HtmlElement, HtmlInputElement, InputEvent, MouseEvent};
use yew::prelude::*;

thread_local! {
    static CHOPPING_BLOCK: Rc<RefCell<Option<OrderToDelete>>> = Rc::new(RefCell::new(None));
}

#[derive(Clone)]
struct OrderToDelete {
    datatable: JsValue,
    delete_dlg: bootstrap::Modal,
    tr_node: web_sys::Node,
    order_id: String,
}

/////////////////////////////////////////////////
pub(crate) fn on_delete_order_from_rpt(
    evt: MouseEvent,
    datatable: std::rc::Rc<std::cell::RefCell<Option<DataTable>>>,
) {
    evt.prevent_default();
    evt.stop_propagation();
    let btn_elm = evt
        .target()
        .and_then(|t| t.dyn_into::<Element>().ok())
        .and_then(|t| {
            // log::info!("Node Name: {}", t.node_name());
            if t.node_name() == "I" {
                t.parent_element()
            } else {
                Some(t)
            }
        })
        .unwrap();

    // for idx in 0..btn_elm.attributes().length() {
    //     if let Some(attr) = btn_elm.attributes().get_with_index(idx) {
    //         log::info!("{}: {}: {}", idx, attr.name(), attr.value());
    //     }
    // }
    let table_row_node = btn_elm.parent_node().and_then(|t| t.parent_node()).unwrap();
    let order_id_str = btn_elm
        .dyn_into::<HtmlElement>()
        .ok()
        .and_then(|t| t.dataset().get("orderid"))
        .unwrap();
    log::info!("on_delete_order: {}", order_id_str);

    let dlg = bootstrap::get_modal_by_id("deleteOrderDlg").unwrap();

    CHOPPING_BLOCK.with(|f| {
        *f.borrow_mut() = Some(OrderToDelete {
            datatable: (*datatable.borrow().as_ref().unwrap()).clone(),
            delete_dlg: dlg.clone().dyn_into::<bootstrap::Modal>().unwrap(),
            tr_node: table_row_node,
            order_id: order_id_str,
        });
    });

    dlg.show();
}

/////////////////////////////////////////////////
/////////////////////////////////////////////////
#[function_component(DeleteOrderDlg)]
pub(crate) fn delete_order_confirmation_dlg() -> Html {
    let on_confirm_input = {
        Callback::from(move |evt: InputEvent| {
            let value = evt
                .target()
                .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
                .unwrap()
                .value();
            if "delete" == &value {
                gloo::utils::document()
                    .get_element_by_id("deleteDlgBtn")
                    .and_then(|t| t.dyn_into::<HtmlButtonElement>().ok())
                    .unwrap()
                    .set_disabled(false);
            } else {
                gloo::utils::document()
                    .get_element_by_id("deleteDlgBtn")
                    .and_then(|t| t.dyn_into::<HtmlButtonElement>().ok())
                    .unwrap()
                    .set_disabled(true);
            }
        })
    };

    let on_submit = {
        Callback::from(move |evt: MouseEvent| {
            evt.target()
                .and_then(|t| t.dyn_into::<HtmlButtonElement>().ok())
                .unwrap()
                .set_disabled(true);
            //.and_then(|t| t.set_disabled(true));
            CHOPPING_BLOCK.with(|f| {
                let f = f.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let maybe_to_delete_order = f.borrow().as_ref().map(|v| v.clone());
                    if let Some(to_delete) = maybe_to_delete_order {
                        if let Err(err) = delete_order(&to_delete.order_id).await {
                            gloo::dialogs::alert(&format!(
                                "Failed to delete order in the cloud: {:#?}",
                                err
                            ));
                        } else if let Err(err) = remove_row_with_tr(&to_delete.datatable, &to_delete.tr_node) {
                                gloo::dialogs::alert(&format!(
                                    "Order was deleted from the cloud but not the local table: {:#?}",
                                    err
                                ));
                        }
                        
                        to_delete.delete_dlg.hide();
                        gloo::utils::document()
                            .get_element_by_id("confirmDeleteOrderInput")
                            .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
                            .unwrap()
                            .set_value("");
                    }
                    *f.borrow_mut() = None;

                    evt.target()
                        .and_then(|t| t.dyn_into::<HtmlButtonElement>().ok())
                        .unwrap()
                        .set_disabled(false);
                });
            });
        })
    };

    html! {
        <div class="modal fade" id="deleteOrderDlg"
             tabIndex="-1" role="dialog" aria-labelledby="deleteOrderDlgTitle" aria-hidden="true">
            <div class="modal-dialog modal-dialog-centered" role="document">
                <div class="modal-content">
                    <div class="modal-header">
                        <h5 class="modal-title" id="deleteOrderDlgLongTitle">
                            {"Confirm Order Deletion"}
                        </h5>
                    </div>
                    <div class="modal-body">
                        <input type="text" class="form-control" id="confirmDeleteOrderInput"
                               placeholder="type delete to confirm" autocomplete="fr-new-cust-info"
                               oninput={on_confirm_input.clone()} aria-describedby="confirmDeleteOrderHelp" />
                        <small id="confirmDeleteOrderHelp" class="form-text text-muted">
                            {"Enter \"delete\" to confirm order deletion."}
                        </small>

                    </div>
                    <div class="modal-footer">
                        <button type="button" class="btn btn-secondary" data-bs-dismiss="modal">
                            {"Cancel"}
                        </button>
                        <button type="button" disabled=true class="btn btn-primary" onclick={on_submit} id="deleteDlgBtn">
                            {"Delete Order"}
                        </button>
                    </div>
                </div>
            </div>
        </div>
    }
}
