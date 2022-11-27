
use yew::prelude::*;
//use yew_router::prelude::*;
use web_sys::{
    Event, InputEvent, MouseEvent, SubmitEvent,
    Element, HtmlElement, HtmlSelectElement, HtmlInputElement, HtmlButtonElement,
};
use crate::data_model::*;
// use crate::AppRoutes;
// use crate::{get_html_input_value, save_to_active_order};
// use wasm_bindgen::JsCast;
// use crate::currency_utils::*;
// use rust_decimal::prelude::*;
// use rusty_money::{Money, iso};



/////////////////////////////////////////////////
/////////////////////////////////////////////////
#[derive(Properties, PartialEq, Clone, Debug)]
struct NeighborhoodLiProps {
    pub name: String,
    pub distpt: String,
}

#[function_component(NeighborhoodLi)]
fn neighborhood_item(props: &NeighborhoodLiProps) -> Html
{

    html! {
        <li class="list-group-item">
            <div class="mb-1">{props.name.clone()}</div>
            <small class="text-muted">{format!("Distribution Point: {}", &props.distpt)}</small>
        </li>
    }
}

#[function_component(NeighborhoodUl)]
fn neighborhood_list() -> Html
{

    html! {
        <div class="card">
            <div class="card-body">
                <h5 class="card-title">{"Neighborhoods"}</h5>
                <ul class="list-group">
                {
                    get_neighborhoods().iter().map(|hood_info| {
                        html!{<NeighborhoodLi name={hood_info.name.clone()} distpt={hood_info.distribution_point.clone()} />}
                    }).collect::<Html>()
                }
                </ul>
            </div>
        </div>

    }
}

/////////////////////////////////////////////////
/////////////////////////////////////////////////
#[derive(Properties, PartialEq, Clone, Debug)]
struct DeliveryLiProps {
    pub deliveryid: u32,
    pub deliverydate: String,
    pub newordercutoff: String,
}

#[function_component(DeliveryLi)]
fn delivery_item(props: &DeliveryLiProps) -> Html
{

    html! {
        <li class="list-group-item">
            <div class="d-flex w-100 justify-content-between">
                <div class="mb-1">{format!("Delivery Date: {}", &props.deliverydate)}</div>
                <small class="text-muted mx-2">{props.deliveryid.to_string()}</small>
            </div>
            <small class="text-muted">{format!("New Order Cutoff: {}", &props.newordercutoff)}</small>
        </li>
    }
}


#[function_component(DeliveryUl)]
fn delivery_list() -> Html
{

    html! {
        <div class="card">
            <div class="card-body">
                <h5 class="card-title">{"Mulch Delivery Dates"}</h5>
                <ul class="list-group">
                {
                    get_deliveries().iter().map(|(id,delivery_info)| {
                        html!{<DeliveryLi deliveryid={id}
                            deliverydate={delivery_info.get_delivery_date_str()}
                            newordercutoff={delivery_info.get_new_order_cutoff_date_str()} />}
                    }).collect::<Html>()
                }
                </ul>
            </div>
        </div>
    }
}

/////////////////////////////////////////////////
/////////////////////////////////////////////////

#[function_component(FrConfig)]
pub fn fr_config() -> Html
{
    html! {
        <div class="col-xs-1 d-flex justify-content-center">
            <NeighborhoodUl/>
            <DeliveryUl/>
        </div>
    }
}
