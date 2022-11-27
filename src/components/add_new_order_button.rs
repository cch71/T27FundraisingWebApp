use yew::prelude::*;
use yew_router::prelude::*;
use crate::data_model::*;
use crate::AppRoutes;

#[derive(Properties, PartialEq)]
pub struct AddNewOrderButtonProps {
    pub userid: String,
}

#[function_component(AddNewOrderButton)]
pub fn add_new_order_button(props: &AddNewOrderButtonProps) -> Html
{
    let history = use_navigator().unwrap();
    let on_add_new_order = {
        let history = history.clone();
        let userid = props.userid.clone();
        move |_| {
            log::info!("Starting process to add new order");
            create_new_active_order(&userid);
            history.push(&AppRoutes::OrderForm);
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

