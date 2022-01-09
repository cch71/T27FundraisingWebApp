use yew::prelude::*;
use std::rc::Rc;
use std::cell::RefCell;
use wasm_bindgen::JsCast;
use web_sys::{MouseEvent, HtmlButtonElement, HtmlInputElement, HtmlTextAreaElement};
use crate::bootstrap;
use crate::data_model::*;

/////////////////////////////////////////////////
/////////////////////////////////////////////////
thread_local! {
    static REPORT_ISSUE_DLG: Rc<RefCell<Option<bootstrap::Modal>>> = Rc::new(RefCell::new(None));
}

pub(crate) fn show_report_issue_dlg(do_show: bool) {
    let document = gloo_utils::document();

    document.get_element_by_id("formSummary")
        .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
        .unwrap()
        .set_value("");

    document.get_element_by_id("formDescription")
        .and_then(|t| t.dyn_into::<HtmlTextAreaElement>().ok())
        .unwrap()
        .set_value("");

    REPORT_ISSUE_DLG.with(|v|{
        if (*v).borrow().is_none() {
            *v.borrow_mut() = Some(bootstrap::get_modal_by_id("xmitIssueDlg").unwrap());
        };

        if do_show {
            v.borrow().as_ref().unwrap().show();
        } else {
            v.borrow().as_ref().unwrap().hide();
            *v.borrow_mut() = None;
        }
    });
}

#[function_component(ReportIssueDlg)]
pub(crate) fn report_issue() -> Html
{
    let spinner_state = use_state_eq(||"d-none");
    let on_submit_issue = {
        let spinner_state = spinner_state.clone();

        Callback::from(move |evt: MouseEvent|{
            evt.prevent_default();
            evt.stop_propagation();
            let document = gloo_utils::document();
            let btn_elm = evt.target()
                .and_then(|t| t.dyn_into::<HtmlButtonElement>().ok())
                .unwrap();
            let summary_elm = document.get_element_by_id("formSummary")
                .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
                .unwrap();
            let desc_elm = document.get_element_by_id("formDescription")
                .and_then(|t| t.dyn_into::<HtmlTextAreaElement>().ok())
                .unwrap();

            spinner_state.set("d-inline-block");
            btn_elm.set_disabled(true);

            let desc = desc_elm.value();
            let summary = summary_elm.value();
            if 0==desc.len() || 0==summary.len() {
                if 0==desc.len() {
                    let _ = desc_elm.class_list().add_1("is-invalid");
                } else {
                    let _ = desc_elm.class_list().remove_1("is-invalid");
                }
                if 0==summary.len() {
                    let _ = summary_elm.class_list().add_1("is-invalid");
                } else {
                    let _ = summary_elm.class_list().remove_1("is-invalid");
                }
                spinner_state.set("d-none");
                return;
            }

            let reporting_user = get_active_user().get_id();
            let spinner_state = spinner_state.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let rslt = report_new_issue(&reporting_user, &summary, &desc).await;
                spinner_state.set("d-none");
                btn_elm.set_disabled(false);
                if let Err(err) = rslt {
                    gloo_dialogs::alert(&format!("Failed to submit report: {:#?}", err));
                } else {
                    show_report_issue_dlg(false);
                }
            });
        })
    };

    html! {
        <div class="modal fade" id="xmitIssueDlg"
             tabIndex="-1" role="dialog" aria-labelledby="xmitIssueDlgTitle" aria-hidden="true">
            <div class="modal-dialog modal-dialog-centered" role="document">
                <div class="modal-content">
                    <div class="modal-header">
                        <h5 class="modal-title" id="xmitIssueDlgLongTitle">
                            {"Report a issue with Fundraiser App"}
                        </h5>
                    </div>
                    <div class="modal-body">
                        <form class="needs-validation" id="formXmitIssue" novalidate=true>
                            <div class="row mb-2 g-2">
                                <div class="form-floating">
                                    <input class="form-control" type="text" autocomplete="fr-new-issue" id="formSummary"
                                           placeholder="Summary" required=true maxlength="255"/>
                                    <label for="formSummary">
                                        {"Summary (255 Max Chars)"}
                                    </label>
                                </div>
                            </div>
                            <div class="row mb-2 g-2">
                                <div class="form-floating">
                                    <textarea class="form-control" rows="10" required=true id="formDescription"/>
                                    <label for="formDescription">{"Description of problem"}</label>
                                </div>
                            </div>
                        </form>
                    </div>
                    <div class="modal-footer">
                        <button type="button" class="btn btn-secondary" data-bs-dismiss="modal">{"Cancel"}</button>
                        <button type="button" class="btn btn-primary" onclick={on_submit_issue}>
                            <span class={format!("spinner-border spinner-border-sm me-1 {}",*spinner_state)} role="status"
                                  aria-hidden="true" id="formXmitIssueSpinner" />
                                  {"Submit"}
                        </button>
                    </div>
                </div>
            </div>
        </div>
    }
}

