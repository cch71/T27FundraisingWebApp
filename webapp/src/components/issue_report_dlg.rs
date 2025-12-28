use data_model::*;
use js::bootstrap;
use wasm_bindgen::JsCast;
use web_sys::{HtmlButtonElement, HtmlInputElement, HtmlTextAreaElement, MouseEvent};
use yew::prelude::*;

/////////////////////////////////////////////////
/////////////////////////////////////////////////
pub(crate) fn show_report_issue_dlg(do_show: bool) {
    let document = gloo::utils::document();

    document
        .get_element_by_id("formSummary")
        .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
        .unwrap()
        .set_value("");

    document
        .get_element_by_id("formDescription")
        .and_then(|t| t.dyn_into::<HtmlTextAreaElement>().ok())
        .unwrap()
        .set_value("");

    if do_show {
        bootstrap::modal_op("xmitIssueDlg", "show");
    } else {
        bootstrap::modal_op("xmitIssueDlg", "hide");
    }
}

#[component(ReportIssueDlg)]
pub(crate) fn report_issue() -> Html {
    let spinner_state = use_state_eq(|| "d-none");
    let on_submit_issue = {
        let spinner_state = spinner_state.clone();

        Callback::from(move |evt: MouseEvent| {
            evt.prevent_default();
            evt.stop_propagation();
            let document = gloo::utils::document();
            let btn_elm = evt
                .target()
                .and_then(|t| t.dyn_into::<HtmlButtonElement>().ok())
                .unwrap();
            let summary_elm = document
                .get_element_by_id("formSummary")
                .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
                .unwrap();
            let desc_elm = document
                .get_element_by_id("formDescription")
                .and_then(|t| t.dyn_into::<HtmlTextAreaElement>().ok())
                .unwrap();

            spinner_state.set("d-inline-block");
            btn_elm.set_disabled(true);

            let desc = desc_elm.value();
            let summary = summary_elm.value();
            if desc.is_empty() || summary.is_empty() {
                if desc.is_empty() {
                    let _ = desc_elm.class_list().add_1("is-invalid");
                } else {
                    let _ = desc_elm.class_list().remove_1("is-invalid");
                }
                if summary.is_empty() {
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
                match rslt {
                    Err(err) => {
                        gloo::dialogs::alert(&format!("Failed to submit report: {err:#?}"));
                    }
                    _ => {
                        show_report_issue_dlg(false);
                    }
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
