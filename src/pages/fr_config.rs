use yew::prelude::*;
use crate::components::admin_config_deliveries::*;
use crate::components::admin_config_neighborhoods::*;

/////////////////////////////////////////////////
/////////////////////////////////////////////////

#[function_component(FrConfig)]
pub fn fr_config() -> Html
{
    html! {
        <div>
            <div class="row">
                <div class="col-xs-1 d-flex justify-content-center">
                    <h2>{"Fundraiser Configuration"}</h2>
                </div>
            </div>
            <div class="row">
                <div class="col-xs-1 d-flex justify-content-center">
                    <NeighborhoodUl/>
                    <DeliveryUl/>
                </div>
            </div>
        </div>
    }
}
