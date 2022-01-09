use yew::prelude::*;
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::BTreeMap;
use wasm_bindgen::JsCast;
use web_sys::{MouseEvent, HtmlInputElement};
use crate::bootstrap;
use crate::data_model::*;

/////////////////////////////////////////////////
/////////////////////////////////////////////////
thread_local! {
    static DLG: Rc<RefCell<Option<bootstrap::Modal>>> = Rc::new(RefCell::new(None));
    static ORDER_ID: Rc<RefCell<Option<String>>> = Rc::new(RefCell::new(None));
}

pub(crate) fn show_report_spreader_chooser_dlg(do_show: bool, order_id: Option<String>) {
    DLG.with(|v|{
        if (*v).borrow().is_none() {
            *v.borrow_mut() = Some(bootstrap::get_modal_by_id("spreadingDlg").unwrap());
        };

        if do_show {
            v.borrow().as_ref().unwrap().show();
        } else {
            v.borrow().as_ref().unwrap().hide();
            *v.borrow_mut() = None;
        }
    });

    ORDER_ID.with(|v| {
        *v.borrow_mut() = order_id;
    });
}

#[function_component(ChooseSpreadersDlg)]
pub(crate) fn choose_spreaders_dlg() -> Html
{
    #[derive(PartialEq)]
    enum SelectionState {
        Choosing,
        Reviewing,
        Submitting,
    }

    let selected_users = use_mut_ref(|| BTreeMap::new());
    let state = use_state(|| SelectionState::Choosing);
    let spinner_state = use_state_eq(||"d-none");

    let on_cancel = {
        let state = state.clone();
        let selected_users = selected_users.clone();
        Callback::from(move |_evt: MouseEvent|{
            ORDER_ID.with(|v| {
                *v.borrow_mut() = None;
            });
            // The order here is important and it makes usre noone is selected
            selected_users.borrow_mut().clear();
            state.set(SelectionState::Choosing);
        })
    };

    let on_submit = {
        let state = state.clone();
        let selected_users = selected_users.clone();
        let spinner_state = spinner_state.clone();
        Callback::from(move |evt: MouseEvent|{
            evt.prevent_default();
            evt.stop_propagation();

            state.set(SelectionState::Submitting);
            spinner_state.set("d-inline-block");

            let mut order_id = "".to_string();
            ORDER_ID.with(|v| {
                order_id = (*v.borrow()).as_ref().unwrap().to_string();
            });
            let selected_users = selected_users.clone();
            let state = state.clone();
            let spinner_state = spinner_state.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let rslt = set_spreaders(&order_id, selected_users.borrow().keys().cloned().collect::<_>()).await;
                spinner_state.set("d-none");
                // The order here is important and it makes usre noone is selected
                selected_users.borrow_mut().clear();
                state.set(SelectionState::Choosing);
                if let Err(err) = rslt {
                    gloo_dialogs::alert(&format!("Failed to submit spreaders: {:#?}", err));
                } else {
                    show_report_spreader_chooser_dlg(false, None);
                }
            });
        })
    };

    let on_select = {
        let state = state.clone();
        Callback::from(move |_evt: MouseEvent|{
            state.set(SelectionState::Choosing);
        })
    };

    let on_review = {
        let state = state.clone();
        Callback::from(move |_evt: MouseEvent|{
            state.set(SelectionState::Reviewing);
        })
    };

    let handle_selected_user = {
        let selected_users = selected_users.clone();
        Callback::from(move |evt: Event|{
            let target_elm = evt.target()
                .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
                .unwrap();
            // let parentNode = evt.target().as_ref().parent_node()
            //    .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
            let uid = target_elm.dataset().get("uid").unwrap();
            if target_elm.checked() {
                let uname = target_elm.dataset().get("uname").unwrap();
                log::info!("Selecting: {}:{}", uid, uname);
                let _ = selected_users.borrow_mut().insert(uid, uname);
                let _ = target_elm.parent_element().unwrap().class_list().add_1("active");
            } else {
                log::info!("Unselecting: {}", uid);
                let _ = selected_users.borrow_mut().remove(&uid);
                let _ = target_elm.parent_element().unwrap().class_list().remove_1("active");
            }
        })
    };

    let (selecting_btn_classes, reviewing_btn_classes, save_btn_classes) = match *state {
        SelectionState::Choosing=>("btn-check active", "btn-check", "btn-check make-disabled"),
        SelectionState::Reviewing=>("btn-check", "btn-check active", "btn-check"),
        SelectionState::Submitting=>("btn-check make-disabled", "btn-check make-disabled", "btn-check active make-disabled"),
    };

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
                                if SelectionState::Choosing == *state {
                                    <div class="col-sm" id="spreadSelectTab">
                                        <label for="spreadingDlgSpreaderSelection">
                                            {"Select Spreaders"}
                                        </label>
                                        <div class="btn-group-vertical overflow-auto" role="group"
                                             id="spreadingDlgSpreaderSelection" aria-label="Select Spreaders">
                                             {
                                                 get_users().iter().map(|(userid, name)|{
                                                     let (li_classes, is_checked, lbl_classes) = if selected_users.borrow().contains_key(userid) {
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
                                } else if SelectionState::Reviewing == *state || SelectionState::Submitting == *state  {
                                    <div class="col-sm" id="spreadReviewTab">
                                        if selected_users.borrow().is_empty() {
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
                                                selected_users.borrow().values().into_iter().map(|name| {
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
                                <span class={format!("spinner-border spinner-border-sm me-1 {}",*spinner_state)}
                                    role="status" aria-hidden="true" id="spreadingSubmitBtnSpinny"/>
                            </label>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}
