use data_model::*;
use std::collections::HashMap;
use tracing::info;
use wasm_bindgen::JsCast;
use web_sys::{HtmlButtonElement, HtmlInputElement, InputEvent, MouseEvent, SubmitEvent};
use yew::prelude::*;
use yew_router::prelude::*;

/////////////////////////////////////////////////
fn disable_submit_button(value: bool) {
    if let Some(btn) = web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .get_element_by_id("formAddProductsSubmit")
        .and_then(|t| t.dyn_into::<HtmlButtonElement>().ok())
    {
        btn.set_disabled(value);
    }
}

/////////////////////////////////////////////////
fn get_product_items(document: &web_sys::Document) -> HashMap<String, PurchasedItem> {
    let mut product_map = HashMap::new();
    if let Ok(product_nodes) = document.query_selector_all("input[data-productid]") {
        if 0 == product_nodes.length() {
            return product_map;
        }
        for idx in 0..product_nodes.length() {
            if let Some(element) = product_nodes
                .item(idx)
                .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
            {
                let value = element.value();
                if let Ok(num_sold) = value.parse::<u32>() {
                    let product_id = element.dataset().get("productid").unwrap();
                    if 0 == num_sold {
                        info!("Purchase Item (Removing): {}: {}", &product_id, num_sold);
                    } else {
                        info!("Purchase Item: {}: {}", &product_id, num_sold);
                        let amount_charged = get_purchase_cost_for(&product_id, num_sold);
                        product_map
                            .insert(product_id, PurchasedItem::new(num_sold, amount_charged));
                    }
                }
            }
        }
    }
    product_map
}

/////////////////////////////////////////////////
fn are_product_items_valid(document: &web_sys::Document) -> bool {
    if let Ok(product_nodes) = document.query_selector_all("input[data-productid]") {
        if 0 == product_nodes.length() {
            return false;
        }
        let mut is_all_0 = true;
        for idx in 0..product_nodes.length() {
            if let Some(element) = product_nodes
                .item(idx)
                .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
            {
                let value = element.value();
                if value.is_empty() {
                    continue;
                }
                let num_to_purchase = match value.parse::<u32>() {
                    Ok(value) => {
                        if 0 == value {
                            continue;
                        }
                        value
                    }
                    Err(_) => return false,
                };
                is_all_0 = false;
                let product_id = match element.dataset().get("productid") {
                    Some(product_id) => product_id,
                    None => return false,
                };
                if !is_purchase_valid(&product_id, num_to_purchase) {
                    return false;
                }
            }
        }
        if is_all_0 {
            return false;
        }
    }
    true
}

/////////////////////////////////////////////////
/////////////////////////////////////////////////

#[derive(Properties, PartialEq)]
pub struct ProductItemProps {
    pub id: String,
    pub productid: String,
    pub label: String,
    pub numordered: String,
    pub oninput: Callback<InputEvent>,
    pub minunits: u32,
}

#[component(ProductItem)]
pub fn product_item(props: &ProductItemProps) -> Html {
    html! {
        <div class="row mb-2 col-sm-12">
            <label for={props.id.clone()}>
                {props.label.clone()}
                 if props.minunits > 0 {
                    <small class="form-text text-muted ps-1">{format!("*minimum purchase: {}", props.minunits)}</small>
                 }
            </label>
            <input type="number" min="0" step="any" class="form-control" id={props.id.clone()}
                   value={props.numordered.clone()}
                   autocomplete="off"
                   data-productid={props.productid.clone()}
                   oninput={props.oninput.clone()}
                   placeholder={"0"}/>
        </div>
    }
}

/////////////////////////////////////////////////
/////////////////////////////////////////////////

#[component(OrderProducts)]
pub fn order_products() -> Html {
    let history = use_navigator().unwrap();

    if !is_active_order() {
        history.push(&AppRoutes::Home);
    }
    let order = get_active_order().unwrap();
    let is_order_readonly = order.is_readonly();

    let on_form_submission = {
        let history = history.clone();
        move |evt: SubmitEvent| {
            evt.prevent_default();
            evt.stop_propagation();
            info!("on_form_submission");
            let mut updated_order = get_active_order().unwrap();

            let document = gloo::utils::document();
            let purchases = get_product_items(&document);
            updated_order.set_purchases(purchases);
            update_active_order(updated_order).unwrap();

            history.push(&AppRoutes::OrderForm);
        }
    };

    let on_cancel_item = {
        let history = history.clone();
        Callback::from(move |evt: MouseEvent| {
            evt.prevent_default();
            evt.stop_propagation();
            //info!("on_cancel_item");
            history.push(&AppRoutes::OrderForm);
        })
    };

    let on_product_order_change = {
        Callback::from(move |evt: InputEvent| {
            evt.prevent_default();
            evt.stop_propagation();

            do_form_validation();
        })
    };

    fn do_form_validation() {
        info!("do_form_validation");
        let document = gloo::utils::document();

        let is_disabled = !are_product_items_valid(&document);

        // If it gets to here the we passed all the tests so enable
        disable_submit_button(is_disabled);
    }

    {
        use_effect(move || {
            let document = gloo::utils::document();
            disable_submit_button(!are_product_items_valid(&document));
            || {}
        });
    }

    html! {
        <div class="col-xs-1 justify-content-center">
            <div class="card">
                <div class="card-body">
                    <h5 class="card-title">{format!("{} Order", get_fr_config().description)}</h5>
                    <form onsubmit={on_form_submission} id="productForm">
                        {
                            get_products().iter().map(|(product_id, product)| {
                                html!{
                                    <ProductItem
                                        id={format!("formProduct-{}",product_id)}
                                        productid={product_id.clone()}
                                        label={product.label.clone()}
                                        numordered={order.get_num_sold(product_id).map_or("".to_string(), |v| v.to_string())}
                                        oninput={on_product_order_change.clone()}
                                        minunits={product.min_units}
                                    />}
                            }).collect::<Html>()
                        }
                        <button type="button" class="btn btn-primary my-2" onclick={on_cancel_item}>
                            {"Cancel"}
                        </button>
                        if !is_order_readonly {
                            <button type="submit" class="btn btn-primary my-2 float-end" id="formAddProductsSubmit">
                                {"Submit"}
                            </button>
                        }
                    </form>
                </div>
            </div>
        </div>
    }
}
