use yew::prelude::*;
// use web_sys::{
//     MouseEvent, HtmlButtonElement,
// };
use crate::data_model::*;
//use crate::currency_utils::*;

/////////////////////////////////////////////////
/////////////////////////////////////////////////

#[function_component(Home)]
pub fn home_page() -> Html
{
    let fr_config = get_fr_config();
    let active_user = get_active_user();

    let fundraiser_sales_finished_msg = if are_sales_still_allowed() {
        html!{}
    } else {
        html! {
            <div style="color: red;">
                <b>{"(The order phase has concluded. Contact the fundrasier admin for new orders/changes)"}</b>
            </div>
        }
    };

    html! {
        <div>
            <div class="justify-content-center text-center">
                <h6>{format!("{} Fundraiser", &fr_config.description)} {fundraiser_sales_finished_msg}</h6>
                <div class="col-xs-1 d-flex justify-content-center">
                    <div class="row">

                        <div class="col-lg-4">
                            <div class="card" id="orderOwnerSummaryCard">
                                <div class="card-header">
                                {format!("Summary for: {}", active_user.get_name())}
                                </div>
                                <div class="card-body text-start">
                                    <small muted=true>{"*updates may take up to 15 minutes"}</small>
                                    <ul class="list-group list-group-flush sm-owner-summary" id="orderOwnerSummaryList">
                                        //{summaryStats}
                                    </ul>
                                </div>
                            </div>
                        </div>

                        <div class="col-lg-4">
                            <div class="card" id="topSellersCard">
                                <div class="card-header">{"Top Sellers:"}</div>
                                <div class="card-body text-start">
                                    <table class="table table-sm table-borderless table-responsive" id="topSellersTable">
                                        <tbody>
                                            //{topSellers}
                                        </tbody>
                                    </table>
                                </div>
                            </div>
                        </div>

                        <div class="col-lg-4">
                            <div class="card" id="patrolStandingsChartCard">
                                <div class="card-header">{"Sales by Patrol:"}</div>
                                <div class="card-body">
                                    <div id="patrolStandingsChart"/>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}
