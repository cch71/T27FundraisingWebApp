use yew::prelude::*;

use data_model::*;
use js::google_charts::*;

/////////////////////////////////////////////////
/////////////////////////////////////////////////

fn gen_summary_html(full_summary: &SummaryReport) -> Html {
    let mut summary_html = Vec::new();
    let summary = &full_summary.seller_summary;
    if summary.total_num_bags_sold != 0 {
        summary_html.push(html! {
            <tr>
                <td class="py-1">{"Num bags sold:"}</td>
                <td class="py-1">{summary.total_num_bags_sold.to_string()}</td>
            </tr>
        });
    }
    if summary.total_num_bags_to_spread_sold != 0 {
        summary_html.push(html! {
            <tr>
                <td class="py-1">{"Num bags to spread:"}</td>
                <td class="py-1">{summary.total_num_bags_to_spread_sold.to_string()}</td>
            </tr>
        });
    }
    if summary.amount_total_collected_for_donations != "0" {
        summary_html.push(html!{
            <tr>
                <td class="py-1">{"Donations collected:"}</td>
                <td class="py-1">{str_to_money_str(&summary.amount_total_collected_for_donations)}</td>
            </tr>
        });
    }
    if summary.amount_total_collected_for_bags != "0" {
        summary_html.push(html! {
            <tr>
                <td class="py-1">{"Your bag sales:"}</td>
                <td class="py-1">{str_to_money_str(&summary.amount_total_collected_for_bags)}</td>
            </tr>
        });
    }
    if summary.amount_total_collected_for_bags_to_spread != "0" {
        summary_html.push(html!{
            <tr>
                <td class="py-1">{"Your bags to spread sales:"}</td>
                <td class="py-1">{str_to_money_str(&summary.amount_total_collected_for_bags_to_spread)}</td>
            </tr>
        });
    }
    if summary.amount_total_collected != "0" {
        summary_html.push(html! {
            <tr>
                <td class="py-1">{"Your total sales:"}</td>
                <td class="py-1">{str_to_money_str(&summary.amount_total_collected)}</td>
            </tr>
        });
    }

    // Allocation Information
    if is_fundraiser_finalized() {
        if summary.allocations_from_deliveries != "0" {
            summary_html.push(html! {
                <tr>
                    <td class="py-1">{"Alloc from deliveries:"}</td>
                    <td class="py-1">{str_to_money_str(&summary.allocations_from_deliveries)}</td>
                </tr>
            });
        }

        if summary.allocations_from_bags_sold != "0" {
            summary_html.push(html! {
                <tr>
                    <td class="py-1">{"Alloc from bags sold:"}</td>
                    <td class="py-1">{str_to_money_str(&summary.allocations_from_bags_sold)}</td>
                </tr>
            });
        }

        if summary.allocations_from_bags_spread != "0" {
            summary_html.push(html! {
                <tr>
                    <td class="py-1">{"Alloc from bags spread:"}</td>
                    <td class="py-1">{str_to_money_str(&summary.allocations_from_bags_spread)}</td>
                </tr>
            });
        }

        if summary.allocations_total != "0" {
            summary_html.push(html! {
                <tr>
                    <td class="py-1">{"Alloc total:"}</td>
                    <td class="py-1">{str_to_money_str(&summary.allocations_total)}</td>
                </tr>
            });
        }
    }

    let troop_summary = &full_summary.troop_summary;
    if troop_summary.amount_total_collected != "0" {
        summary_html.push(html! {
            <tr>
                <td class="py-1">{"Troop has sold:"}</td>
                <td class="py-1">{str_to_money_str(&troop_summary.amount_total_collected)}</td>
            </tr>
        });
    }

    summary_html.into_iter().collect::<Html>()
}

fn gen_top_sellers_html(top_sellers: &[TopSeller]) -> Html {
    let mut ranking = 0;
    top_sellers
        .iter()
        .map(|seller| {
            ranking += 1;
            html! {
                <tr>
                    <td class="py-1">{ranking.to_string()}</td>
                    <td class="py-1">{seller.name.clone()}</td>
                    <td class="py-1">{str_to_money_str(&seller.amount_total_collected)}</td>
                </tr>
            }
        })
        .collect::<Html>()
}

#[function_component(Home)]
pub fn home_page() -> Html {
    let fr_config = get_fr_config();
    let active_user = get_active_user();
    //let summary_values: Rc<RefCell<Option<SummaryReport>>> = use_state_eq(|| None);
    let summary_values: UseStateHandle<Option<SummaryReport>> = use_state_eq(|| None);

    {
        let summary_values = summary_values.clone();
        use_effect(move || {
            if let Some(summary) = (*summary_values).as_ref() {
                use std::collections::HashMap;
                let patrol_summary_map: HashMap<String, f32> = summary
                    .troop_summary
                    .group_summary
                    .iter()
                    .map(|v| {
                        (
                            v.group_id.clone(),
                            v.amount_total_collected.parse::<f32>().unwrap_or(0.0),
                        )
                    })
                    .collect();
                //log::info!("Google Chart Params: {:#?}", &m);
                draw_google_chart(&patrol_summary_map);
            }

            || {}
        });
    }

    let fundraiser_sales_finished_msg = if are_sales_still_allowed() {
        html! {}
    } else if !is_fundraiser_finalized() {
        html! {
            <div style="color: red;">
                <b>{"(The order phase has concluded. Contact the fundraiser admin for new orders/changes)"}</b>
            </div>
        }
    } else {
        // Fundraiser is finished and allocations have been distributed
        html! {
            <div style="color: red;">
                <b>{"(The fundraiser is now closed and funds have been released)"}</b>
            </div>
        }
    };

    {
        let summary_values = summary_values.clone();
        wasm_bindgen_futures::spawn_local(async move {
            let id = get_active_user().get_id();
            match get_summary_report_data(&id, NUM_TOP_SELLERS_TO_GET).await {
                Err(err) => gloo::dialogs::alert(&format!(
                    "Failed to retrieve summary data to local storage: {:#?}",
                    err
                )),
                Ok(summary) => summary_values.set(Some(summary)),
            };
        });
    }

    let (summary_html, top_sellers_html) = {
        match (*summary_values).as_ref() {
            Some(summary) => (
                gen_summary_html(summary),
                gen_top_sellers_html(&summary.troop_summary.top_sellers),
            ),
            _ => (html! {}, html! {}),
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
                                    <table class="table table-sm table-borderless table-responsive" id="orderOwnerSummaryTable">
                                        <tbody>
                                            {summary_html}
                                        </tbody>
                                    </table>
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
