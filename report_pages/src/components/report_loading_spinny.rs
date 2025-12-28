use yew::prelude::*;

/////////////////////////////////////////////////
/////////////////////////////////////////////////
#[component(ReportLoadingSpinny)]
pub(crate) fn report_loading_spinny() -> Html {
    html! {
        <div class="justify-content-center text-center">
            <h2>{"Loading Report Data..."}</h2>
            <span class="loader"></span>
        </div>
    }
}
