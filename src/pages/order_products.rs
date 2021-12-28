use yew::prelude::*;
use yew_router::prelude::*;
use crate::currency_utils::*;
use web_sys::{InputEvent, MouseEvent, FocusEvent};
use crate::AppRoutes;

#[function_component(OrderProducts)]
pub fn order_products() -> Html
{
    let donation_amount = "$20.00";
    let is_order_readonly = false;
    let history = use_history().unwrap();

    let on_form_submission = {
        let history = history.clone();
        Callback::from(move |evt: FocusEvent| {
            evt.prevent_default();
            evt.stop_propagation();
            log::info!("on_form_submission");

            history.push(AppRoutes::OrderForm);
        })
    };

    let on_cancel_item = {
        let history = history.clone();
        Callback::from(move |evt: MouseEvent| {
            evt.prevent_default();
            evt.stop_propagation();
            log::info!("on_cancel_item");
            history.push(AppRoutes::OrderForm);
        })
    };

    let does_submit_get_enabled = Callback::from(move |evt: InputEvent| {
        evt.prevent_default();
        evt.stop_propagation();
        log::info!("does_submit_get_enabled");
    });

    let form_title = format!("{} Order", "Mulch"/*{fundraiserConfig.description()*/);

    html! {
        <div class="col-xs-1 justify-content-center">
            <div class="card">
                <div class="card-body">
                    <h5 class="card-title">{form_title}</h5>
                    <form onsubmit={on_form_submission} id="productForm">
                        /*
			<div className="row mb-3 col-sm-12">
				<label htmlFor="formSelectDeliveryDate">Select Delivery Date</label>
				<select className="custom-select" id="formSelectDeliveryDate" defaultValue={currentDeliveryId}>
					{deliveryDateOpts}
				</select>
			</div>

			{productElms}
                        */
                        <button type="button" class="btn btn-primary my-2" onclick={on_cancel_item}>
                            {"Cancel"}
                        </button>
                        if !is_order_readonly {
                            <button type="submit" class="btn btn-primary my-2 float-end" id="formDonationSubmit">
                                {"Submit"}
                            </button>
                        }
                    </form>
                </div>
            </div>
        </div>
    }
}
