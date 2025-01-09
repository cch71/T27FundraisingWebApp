use data_model::*;
use wasm_bindgen::JsCast;
use web_sys::{HtmlSelectElement, InputEvent};
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct DeliveryDateSelectorProps {
    pub on_delivery_change: Callback<Option<u32>>,
}

#[function_component(DeliveryDateSelector)]
pub fn delivery_date_selector(props: &DeliveryDateSelectorProps) -> Html {
    let order = get_active_order().unwrap();

    let on_input = {
        let on_delivery_change = props.on_delivery_change.clone();
        Callback::from(move |evt: InputEvent| {
            evt.prevent_default();
            evt.stop_propagation();
            let value = evt
                .target()
                .and_then(|t| t.dyn_into::<HtmlSelectElement>().ok())
                .unwrap()
                .value();
            log::info!("Delivery Date Selection Val: {}", &value);
            if value.is_empty() || "none" == value {
                on_delivery_change.emit(None);
            } else {
                let delivery_id = value.parse::<u32>().unwrap();
                on_delivery_change.emit(Some(delivery_id));
            }
        })
    };

    let mut found_selected_delivery = false;
    let is_admin = get_active_user().is_admin();

    html! {
        <div class="delivery-selector-widget">
            <label for="formSelectDeliveryDate">{"Select Delivery Date"}</label>
            <select
                class="custom-select"
                id="formSelectDeliveryDate"
                required=true
                oninput={on_input}>
                {
                    get_deliveries().iter().map(|(delivery_id, delivery)| {
                        let is_selected = delivery_id == order.delivery_id.as_ref().unwrap_or(&0);
                        if is_selected && !found_selected_delivery {
                            found_selected_delivery = true;
                            html!{
                                <option value={delivery_id.to_string()} selected={is_selected}>
                                    {delivery.get_delivery_date_str()}
                                </option>
                            }
                        } else if is_admin || delivery.can_take_orders() {
                            html!{
                                <option value={delivery_id.to_string()} selected={is_selected}>
                                    {delivery.get_delivery_date_str()}
                                </option>
                            }
                        } else {
                            html!{}
                        }
                    }).collect::<Html>()
                }
                if !found_selected_delivery {
                    <option value="none" selected=true disabled=true hidden=true>{"Select delivery date"}</option>
                }
            </select>
        </div>
    }
}
