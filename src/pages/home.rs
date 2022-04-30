use yew::prelude::*;
// use web_sys::{
//     MouseEvent, HtmlButtonElement,
// };

use crate::data_model::*;
use crate::currency_utils::*;
use crate::google_charts::*;

// #[function_component(AdminTestButton)]
// pub fn admin_test_button() -> Html
// {
//     let on_press_red_button = {
//         Callback::from(move |_| {
//             wasm_bindgen_futures::spawn_local(async move {
//                 log::info!("Calling Admin Test API");
//                 let rslt = call_admin_test_api().await;
//                 if let Err(err) = rslt {
//                     gloo_dialogs::alert(&format!("Bad: {:#?}", err));
//                 } else {
//                     gloo_dialogs::alert(":)");
//                 }
//                 log::info!("Done Calling Admin Test API");
//             });
//         })
//     };
//
//     html! {
//         <div>
//             <label>{"Admin Test Button"}</label>
//             <button type="button"
//                     class="btn btn-outline-primary"
//                     onclick={on_press_red_button}>
//             </button>
//         </div>
//     }
// }

/////////////////////////////////////////////////
/////////////////////////////////////////////////

fn gen_summary_html(full_summary: &SummaryReport)->Html {
    let mut summary_html = Vec::new();
    let summary = &full_summary.seller_summary;
    if summary.total_num_bags_sold != 0 {
        summary_html.push(html!{
            <li class="list-group-item border-0 py-1">
                {format!("Num bags sold: {}", summary.total_num_bags_sold)}
            </li>
        });
    }
    if summary.total_num_bags_to_spread_sold != 0 {
        summary_html.push(html!{
            <li class="list-group-item border-0 py-1">
                {format!("Num bags to spread: {}", summary.total_num_bags_to_spread_sold)}
            </li>
        });
    }
    if summary.amount_total_collected_for_donations != "0" {
        summary_html.push(html!{
            <li class="list-group-item border-0 py-1">
                {format!("Donations collected: {}", str_to_money_str(&summary.amount_total_collected_for_donations))}
            </li>
        });
    }
    if summary.amount_total_collected_for_bags != "0" {
        summary_html.push(html!{
            <li class="list-group-item border-0 py-1">
                {format!("Your bag sales: {}", str_to_money_str(&summary.amount_total_collected_for_bags))}
            </li>
        });
    }
    if summary.amount_total_collected_for_bags_to_spread != "0" {
        summary_html.push(html!{
            <li class="list-group-item border-0 py-1">
                {format!("Your bags to spread sales: {}", str_to_money_str(&summary.amount_total_collected_for_bags_to_spread))}
            </li>
        });
    }
    if summary.amount_total_collected != "0" {
        summary_html.push(html!{
            <li class="list-group-item border-0 py-1">
                {format!("Your total sales: {}", str_to_money_str(&summary.amount_total_collected))}
            </li>
        });
    }

    let troop_summary = &full_summary.troop_summary;
    if troop_summary.amount_total_collected != "0" {
        summary_html.push(html!{
            <li class="list-group-item border-0 py-1">
                {format!("Troop has sold: {}", str_to_money_str(&troop_summary.amount_total_collected))}
            </li>
        });
    }

    summary_html.into_iter().collect::<Html>()
}

fn gen_top_sellers_html(top_sellers: &Vec<TopSeller>)->Html {
    let mut ranking = 0;
    top_sellers.into_iter().map(|seller| {
        ranking = ranking + 1;
        html!{
            <tr>
                <td class="py-1">{ranking.to_string()}</td>
                <td class="py-1">{seller.name.clone()}</td>
                <td class="py-1">{str_to_money_str(&seller.amount_total_collected)}</td>
            </tr>
        }
    }).collect::<Html>()
}

#[function_component(Home)]
pub fn home_page() -> Html
{
    let fr_config = get_fr_config();
    let active_user = get_active_user();
    //let summary_values: Rc<RefCell<Option<SummaryReport>>> = use_state_eq(|| None);
    let summary_values: UseStateHandle<Option<SummaryReport>> = use_state_eq(|| None);

    {
        let summary_values = summary_values.clone();
        use_effect(move || {
            if let Some(summary) = (*summary_values).as_ref() {
                draw_google_chart(&serde_json::json!({
                    "groupRankings": summary.troop_summary.group_summary.iter().map(|v|{
                        (v.group_id.clone(), v.amount_total_collected.parse::<f32>().unwrap_or(0.0))
                    }).collect::<Vec<(String, f32)>>(),
                }));
            }

            || {}
        });
    }


    let fundraiser_sales_finished_msg = if are_sales_still_allowed() {
        html!{}
    } else {
        html! {
            <div style="color: red;">
                <b>{"(The order phase has concluded. Contact the fundrasier admin for new orders/changes)"}</b>
            </div>
        }
    };

    {
        let summary_values = summary_values.clone();
        wasm_bindgen_futures::spawn_local(async move {
            let id = get_active_user().get_id();
            match get_summary_report_data(&id, 10).await{
                Err(err) => gloo_dialogs::alert(&format!("Failed to retrieve summary data to local storage: {:#?}", err)),
                Ok(summary) => summary_values.set(Some(summary)),
            };
        });
    }

    let (summary_html, top_sellers_html) = {
        if let Some(summary) = (*summary_values).as_ref() {
            ( gen_summary_html(&summary), gen_top_sellers_html(&summary.troop_summary.top_sellers) )
        } else {
            (html!{}, html!{})
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
                                    <small muted=true>{"*updates may take up to 24 hours"}</small>
                                    <ul class="list-group list-group-flush sm-owner-summary" id="orderOwnerSummaryList">
                                        {summary_html}
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
                                            {top_sellers_html}
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
