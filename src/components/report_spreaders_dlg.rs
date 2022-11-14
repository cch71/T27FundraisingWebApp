use yew::prelude::*;
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::BTreeMap;
use wasm_bindgen::{JsValue, JsCast};
use web_sys::{ MouseEvent, Element, HtmlElement, HtmlInputElement};
use crate::bootstrap;
use crate::data_model::*;
use crate::datatable::*;

/////////////////////////////////////////////////
/////////////////////////////////////////////////
thread_local! {
    static META: Rc<RefCell<Option<DlgMeta>>> = Rc::new(RefCell::new(None));
    static DLG_STATE: Rc<RefCell<Option<UseStateHandle<SelectionState>>>> = Rc::new(RefCell::new(None));
}

struct DlgMeta {
    datatable: JsValue,
    dlg: bootstrap::Modal,
    tr_node: web_sys::Node,
    order_id: String,
    selected_users: BTreeMap<String, String>,
    dataset_elm: HtmlElement,
}

/////////////////////////////////////////////////
///
pub(crate) fn on_edit_spreading_from_rpt( evt: MouseEvent, datatable: std::rc::Rc<std::cell::RefCell<Option<DataTable>>>)
{
    evt.prevent_default();
    evt.stop_propagation();
    let btn_elm = evt.target()
        .and_then(|t| t.dyn_into::<Element>().ok())
        .and_then(|t| {
            // log::info!("Node Name: {}", t.node_name());
            if t.node_name() == "I" {
                t.parent_element()
            } else {
                Some(t)
            }
        }).unwrap();

    let tr_node = btn_elm.parent_node() .and_then(|t| t.parent_node()) .unwrap();
    let elm = btn_elm.dyn_into::<HtmlElement>().ok().unwrap();

    let order_id = elm.dataset().get("orderid").unwrap();
    let users = get_users();
    let spreaders:BTreeMap<String, String> = elm.dataset().get("spreaders")
        .unwrap_or("".to_string())
        .split(",").into_iter()
        .map(|v|{
            (v.to_string(), users.get(v).map_or(v.to_string(),|ui|ui.name.to_string()))
        }).collect::<_>();
    log::info!("on_edit_spreading: {}", order_id);

    let dlg = bootstrap::get_modal_by_id("spreadingDlg").unwrap();

    META.with(|f|{
        *f.borrow_mut() = Some(DlgMeta{
            datatable: (*datatable.borrow().as_ref().unwrap()).clone(),
            dlg: dlg.clone().dyn_into::<bootstrap::Modal>().unwrap(),
            tr_node: tr_node,
            order_id: order_id,
            selected_users: spreaders.into_iter().filter(|(_id, name)| 0!=name.len()).collect(),
            dataset_elm: elm,
        });
    });

    dlg.toggle();

    DLG_STATE.with(|v|{
        let dlg_state = v.borrow().as_ref().unwrap().clone();
        dlg_state.set(SelectionState::Choosing);
    });

}

#[derive(PartialEq,Copy,Clone)]
enum SelectionState {
    Choosing,
    Reviewing,
    Submitting,
}
use std::fmt;
impl fmt::Display for SelectionState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            SelectionState::Choosing => write!(f, "Choosing"),
            SelectionState::Reviewing => write!(f, "Reviewing"),
            SelectionState::Submitting => write!(f, "Submitting"),
        }
    }
}

#[function_component(ChooseSpreadersDlg)]
pub(crate) fn choose_spreaders_dlg() -> Html
{
    let dlg_state = use_state(|| SelectionState::Choosing);
    {
        let dlg_state = dlg_state.clone();
        DLG_STATE.with(|v|{
            *v.borrow_mut() = Some(dlg_state);
        });
    }
    log::info!("Rendering: {}", *dlg_state);

    let on_cancel = {
        let dlg_state = dlg_state.clone();
        Callback::from(move |_evt: MouseEvent|{
            META.with(|metarc|{
                *metarc.borrow_mut() = None;
            });
            dlg_state.set(SelectionState::Choosing);
        })
    };

    let on_submit = {
        let dlg_state = dlg_state.clone();
        Callback::from(move |_evt: MouseEvent|{
            // evt.prevent_default();
            // evt.stop_propagation();

            dlg_state.set(SelectionState::Submitting);

            META.with(|metarc|{
                let dlg_state = dlg_state.clone();
                let metarc=metarc.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    if let Some(meta) = &*metarc.borrow() {
                        let spreaders:Vec<String> = meta.selected_users.keys().cloned().collect::<_>();
                        if let Err(err) = set_spreaders(&meta.order_id, &spreaders).await {
                            gloo_dialogs::alert(&format!("Failed to submit spreaders: {:#?}", err));
                        } else {
                            let spreaders = spreaders.join(",");
                            let _ = meta.dataset_elm.dataset().set("spreaders", &spreaders);
                            if let Err(err) = set_spreaders_with_tr(&meta.datatable, &meta.tr_node, &spreaders) {
                                gloo_dialogs::alert(&format!("Order was set in the cloud db but not the local table: {:#?}", err));
                            }
                        }

                        meta.dlg.toggle();
                        dlg_state.set(SelectionState::Choosing);
                    }
                    *metarc.borrow_mut() = None;
                });
            });
        })
    };

    let on_select = {
        let dlg_state = dlg_state.clone();
        Callback::from(move |_evt: MouseEvent|{
            dlg_state.set(SelectionState::Choosing);
        })
    };

    let on_review = {
        let dlg_state = dlg_state.clone();
        Callback::from(move |_evt: MouseEvent|{
            dlg_state.set(SelectionState::Reviewing);
        })
    };

    let handle_selected_user = {
        Callback::from(move |evt: Event|{
            let target_elm = evt.target()
                .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
                .unwrap();
            // let parentNode = evt.target().as_ref().parent_node()
            //    .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
            let uid = target_elm.dataset().get("uid").unwrap();
            META.with(|metarc|{
                if let Some(meta) = metarc.borrow_mut().as_mut() {
                    if target_elm.checked() {
                        let uname = target_elm.dataset().get("uname").unwrap();
                        log::info!("Selecting: {}:{}", uid, uname);
                        let _ = meta.selected_users.insert(uid, uname);
                        let _ = target_elm.parent_element().unwrap().class_list().add_1("active");
                    } else {
                        log::info!("Unselecting: {}", uid);
                        let _ = meta.selected_users.remove(&uid);
                        let _ = target_elm.parent_element().unwrap().class_list().remove_1("active");
                    }
                }
            });
        })
    };

    let (selecting_btn_classes, reviewing_btn_classes, save_btn_classes) = match *dlg_state {
        SelectionState::Choosing=>("btn-check active", "btn-check", "btn-check make-disabled"),
        SelectionState::Reviewing=>("btn-check", "btn-check active", "btn-check"),
        SelectionState::Submitting=>("btn-check make-disabled", "btn-check make-disabled", "btn-check active make-disabled"),
    };

    let mut selected_users = BTreeMap::new();
    META.with(|metarc|{
        if let Some(meta) = &*metarc.borrow() {
            selected_users = meta.selected_users.clone();
        }
    });

    html!{
        <div class="modal fade" id="spreadingDlg"
             tabIndex="-1" role="dialog" aria-labelledby="spreadingDlgTitle" aria-hidden="true">
            <div class="modal-dialog modal-dialog-centered" role="document">
                <div class="modal-content">
                    <div class="modal-header">
                        <h5 class="modal-title" id="spreadingDlgLongTitle">
                           {"Spreading Completion"}
                        </h5>
                    </div>
                    <div class="modal-body">
                        <div class="container-sm">
                            <div class="row">
                                if SelectionState::Choosing == *dlg_state {
                                    <div class="col-sm" id="spreadSelectTab">
                                        <label for="spreadingDlgSpreaderSelection">
                                            {"Select Spreaders"}
                                        </label>
                                        <div class="btn-group-vertical overflow-auto" role="group"
                                             id="spreadingDlgSpreaderSelection" aria-label="Select Spreaders">
                                             {
                                                 get_users()
                                                     .iter()
                                                     .filter(|(_,user_info)| "Bear"!=user_info.group && "Bogus"!=user_info.group)
                                                     .map(|(userid, user_info)|{
                                                         let name = &user_info.name;
                                                         let (li_classes, is_checked, lbl_classes) = if selected_users.contains_key(userid) {
                                                             ("btn-check active", true, "btn btn-outline-primary active")
                                                         } else {
                                                             ("btn-check", false, "btn btn-outline-primary")
                                                         };
                                                         //log::info!("Reviewing: {}:{} is_checked: {}", userid, name, is_checked);
                                                         html! {
                                                             <label class={lbl_classes}>
                                                                {name.clone()}
                                                                <input type="checkbox" class={li_classes} onchange={handle_selected_user.clone()}
                                                                    data-uid={userid.clone()} data-uname={name.clone()} autocomplete="off"
                                                                    checked={is_checked}/>
                                                             </label>
                                                         }
                                                     }).collect::<Html>()
                                             }
                                        </div>
                                    </div>
                                } else if SelectionState::Reviewing == *dlg_state || SelectionState::Submitting == *dlg_state  {
                                    <div class="col-sm" id="spreadReviewTab">
                                        if selected_users.is_empty() {
                                            <div class="alert alert-danger">
                                                <h6>
                                                    {"No spreaders were selected. Submitting this will mark this order as not spread yet."}
                                                </h6>
                                            </div>
                                        } else {
                                            <label for={"spreadingDlgSpreaderSelectionReview"}>
                                                {"Review Spreaders"}
                                            </label>
                                            <ul class="list-group" id="spreadingDlgSpreaderSelectionReview">
                                            {
                                                selected_users.values().into_iter().map(|name| {
                                                    html!{
                                                        <li class="list-group-item">
                                                            {name}
                                                        </li>
                                                    }
                                                 }).collect::<Html>()
                                            }
                                            </ul>
                                        }
                                    </div>
                                }
                            </div>
                        </div>
                    </div>
                    <div class="modal-footer">
                        <button type="button" class="btn btn-secondary" data-bs-dismiss="modal" onclick={on_cancel}>{"Cancel"}</button>
                        <div class="btn-group" role="group" aria-label="Spreader Selection Group">
                            <input type="radio" class={selecting_btn_classes} name="btnradio" id="spreadersSelectBtn"
                                   autocomplete="off" defaultChecked="true" onclick={on_select.clone()}/>
                            <label class="btn btn-outline-primary" for="spreadersSelectBtn">
                                {"1. Select"}
                            </label>
                            <input type="radio" class={reviewing_btn_classes} name="btnradio" id="spreadersReviewBtn"
                                autocomplete="off" onclick={on_review.clone()}/>
                            <label class="btn btn-outline-primary" for="spreadersReviewBtn" >
                                {"2. Review"}
                            </label>
                            <input type="radio" class={save_btn_classes} onclick={on_submit} name="btnradio"
                                id="spreadersSaveBtn" autocomplete="off"/>
                            <label class="btn btn-outline-primary" for="spreadersSaveBtn" >
                                {"3. Submit"}
                                if SelectionState::Submitting == *dlg_state {
                                    <span class={"spinner-border spinner-border-sm me-1"}
                                        role="status" aria-hidden="true" id="spreadingSubmitBtnSpinny"/>
                                }
                            </label>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}
