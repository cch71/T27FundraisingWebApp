use data_model::create_new_active_order;
use js::nav::navigate_to;
use tracing::info;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub(crate) struct AddNewOrderButtonProps {
    pub(crate) userid: String,
}

#[component(AddNewOrderButton)]
pub(crate) fn add_new_order_button(props: &AddNewOrderButtonProps) -> Html {
    let on_add_new_order = {
        let userid = props.userid.clone();
        move |_| {
            info!("Starting process to add new order");
            create_new_active_order(&userid);
            navigate_to("/order");
        }
    };

    html! {
        <div class="add-order-widget float-end me-3 my-1">
            <label>{"Add New Order"}</label>
            <button type="button"
                    class="btn btn-outline-primary add-order-btn"
                    onclick={on_add_new_order}>
                <i class="bi bi-plus-square-fill" fill="currentColor"></i>
            </button>
        </div>
    }
}
