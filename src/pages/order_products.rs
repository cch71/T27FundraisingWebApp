use yew::prelude::*;
use yew_router::prelude::*;
use web_sys::{InputEvent, MouseEvent, FocusEvent};
use crate::AppRoutes;
use crate::currency_utils::*;
use crate::data_model::*;
use chrono::prelude::*;

/////////////////////////////////////////////////
/////////////////////////////////////////////////

#[derive(Properties, PartialEq)]
pub struct ProductItemProps {
    pub id: String,
    pub label: String,
    pub numordered: String,
}

#[function_component(ProductItem)]
pub fn product_item(props: &ProductItemProps) -> Html
{
    html! {
        <div class="row mb-2 col-sm-12">
            <label for={props.id.clone()}>{props.label.clone()}</label>
            <input type="number" min="0" step="any" class="form-control" id={props.id.clone()}
                   value={props.numordered.clone()} autocomplete="off"
                   placeholder={"0"}/>
        </div>
    }
}

/////////////////////////////////////////////////
/////////////////////////////////////////////////

#[function_component(OrderProducts)]
pub fn order_products() -> Html
{
    let order = get_active_order().unwrap();
    let is_order_readonly = order.is_readonly();
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


    html! {
        <div class="col-xs-1 justify-content-center">
            <div class="card">
                <div class="card-body">
                    <h5 class="card-title">{format!("{} Order", get_fr_config().description)}</h5>
                    <form onsubmit={on_form_submission} id="productForm">
			<div class="row mb-3 col-sm-12">
				<label for="formSelectDeliveryDate">{"Select Delivery Date"}</label>
				<select class="custom-select" id="formSelectDeliveryDate">
                                {
                                    get_deliveries().iter().map(|delivery| {
                                        let is_selected = &delivery.id == order.delivery_id.as_ref().unwrap_or(&"".to_string());
                                        if delivery.new_order_cutoff_date > Utc::now() {
                                            html!{
                                                <option key={delivery.id.clone()} selected={is_selected}>
                                                    {delivery.delivery_date.format("%Y-%m-%d").to_string()}
                                                </option>
                                            }
                                        } else {
                                            html!{}
                                        }
                                    }).collect::<Html>()
                                }
                                    <option value="none" selected=true disabled=true hidden=true>{"Select delivery date"}</option>
				</select>
			</div>
                        {
                            get_products().iter().map(|product| {
                                html!{
                                    <ProductItem
                                        id={format!("formProduct-{}",product.id)}
                                        label={product.label.clone()}
                                        numordered={order.get_num_sold(&product.id).map_or("".to_string(), |v| v.to_string())}
                                    />}
                            }).collect::<Html>()
                        }
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
