
use yew::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{InputEvent, MouseEvent, HtmlInputElement};
use rust_decimal::prelude::*;

use crate::data_model::*;
use crate::currency_utils::decimal_to_money_string;

////////////////////////////////////////////////////////
///
#[derive(Default, Debug, PartialEq, Clone)]
struct DynamicVars {
    bank_deposited: Decimal,
    mulch_cost: Decimal,
    per_bag_cost: Decimal,
    profits_from_bags: Decimal,
    mulch_sales_gross: Decimal,
    money_pool_for_troop: Decimal,
    money_pool_for_scouts_sub_pools: Decimal,
    money_pool_for_scout_sales: Decimal,
    per_bag_avg_earnings: Decimal,
    money_pool_for_scout_delivery: Decimal,
    delivery_earnings_per_minute: Decimal,
}

impl DynamicVars {
    fn new()->Self {
        DynamicVars::default()
    }

}

////////////////////////////////////////////////////////
///
#[derive(Default, Debug, PartialEq, Clone)]
struct ScoutVals {
    name: String,
    uid: String,
    bags_sold: u64,
    bags_spread: u64,
    delivery_minutes: Decimal,
    total_donations: Decimal,
    allocation_from_bags_sold: Decimal,
    allocation_from_bags_spread: Decimal,
    allocation_from_delivery: Decimal,
    allocation_total: Decimal,
}

////////////////////////////////////////////////////////
///
fn calculate_new_dvars(mut dvars: DynamicVars, svar_map: FrClosureStaticData)->Option<DynamicVars> {
    //use Decimal::dec;
    let svars = svar_map.get("TROOP_TOTALS").unwrap();

    log::info!("BD: {}, MS: {}, SP: {} DN: {}",
        &dvars.bank_deposited,
        &dvars.mulch_cost,
        &svars.amount_from_bags_to_spread_sales,
        &svars.amount_from_donations);
    dvars.mulch_sales_gross = dvars.bank_deposited
        .checked_sub(svars.amount_from_bags_to_spread_sales)
        .and_then(|v| v.checked_sub(dvars.mulch_cost))
        .and_then(|v| v.checked_sub(svars.amount_from_donations))
        .unwrap();
    dvars.money_pool_for_troop = dvars.mulch_sales_gross.checked_mul(Decimal::from_f32(0.20).unwrap()).unwrap();
    dvars.money_pool_for_scouts_sub_pools = dvars.mulch_sales_gross.checked_mul(Decimal::from_f32(0.80).unwrap()).unwrap();
    //Distribute profits between selling/delivery buckets
    dvars.money_pool_for_scout_sales = dvars.money_pool_for_scouts_sub_pools.checked_div(Decimal::from_f32(2.0).unwrap()).unwrap();
    dvars.money_pool_for_scout_delivery = dvars.money_pool_for_scout_sales;
    dvars.per_bag_avg_earnings = dvars.money_pool_for_scout_sales.checked_div(svars.num_bags_sold.into()).unwrap();
    dvars.per_bag_cost = dvars.mulch_cost.checked_div(svars.num_bags_sold.into()).unwrap();
    dvars.profits_from_bags = svars.amount_from_bags_sales.checked_sub(dvars.mulch_cost).unwrap();
    let delivery_time_in_minutes = Decimal::from_f64(svars.delivery_time_total.as_secs_f64()/60.0).unwrap();
    dvars.delivery_earnings_per_minute = dvars.money_pool_for_scout_delivery.checked_div(delivery_time_in_minutes).unwrap();
    Some(dvars)
}
////////////////////////////////////////////////////////
///
fn calculate_per_scout_report(dvars:&DynamicVars, svar_map: FrClosureStaticData) -> Vec<ScoutVals> {

    // These totals are calculated from what is in the scout report as given to the scouts
    let mut total_calc_donations = Decimal::ZERO;
    let mut calc_allocations_from_bags_sold = Decimal::ZERO;
    let mut calc_allocations_from_bags_spread = Decimal::ZERO;
    let mut calc_allocations_from_delivery = Decimal::ZERO;
    let mut calc_delivery_minutes = Decimal::ZERO;
    let mut calc_num_bags_spread: u64 = 0;
    let mut calc_allocations_total = Decimal::ZERO;

    let uid_2_name_map = get_users();
    let spreading_price = get_products().get("spreading").map(|v| Decimal::from_str(&v.unit_price).ok()).flatten().unwrap();

    let mut scout_vals = svar_map.iter().filter(|(uid, _)| uid.as_str() != "TROOP_TOTALS").map(|(uid, data)| {
        total_calc_donations = total_calc_donations.checked_add(data.amount_from_donations).unwrap();
        let allocations_from_bags_sold = dvars.per_bag_cost.checked_mul(data.num_bags_sold.into())
            .and_then(|cost_for_this_scouts_bags_sold|data.amount_from_bags_sales.checked_sub(cost_for_this_scouts_bags_sold))
            .and_then(|profit_from_these_bags|profit_from_these_bags.checked_div(dvars.profits_from_bags))
            .and_then(|these_bags_percentage_of_the_overall_sales| {
                //log::info!("%{} of: {}", these_bags_percentage_of_the_overall_sales.round_dp(2), dvars.money_pool_for_scout_sales);
                these_bags_percentage_of_the_overall_sales.checked_mul(dvars.money_pool_for_scout_sales)
            })
            .unwrap();
        calc_allocations_from_bags_sold = calc_allocations_from_bags_sold.checked_add(allocations_from_bags_sold).unwrap();

        let allocation_from_bags_spread= spreading_price.checked_mul(data.num_bags_spread.into()).unwrap();
        calc_allocations_from_bags_spread = calc_allocations_from_bags_spread.checked_add(allocation_from_bags_spread).unwrap();
        calc_num_bags_spread += data.num_bags_spread;

        let delivery_minutes = Decimal::from_f64(data.delivery_time_total.as_secs_f64()/60.0).unwrap();
        let allocations_from_delivery = dvars.delivery_earnings_per_minute
            .checked_mul(delivery_minutes)
            .unwrap();
        calc_allocations_from_delivery = calc_allocations_from_delivery.checked_add(allocations_from_delivery).unwrap();
        calc_delivery_minutes = calc_delivery_minutes.checked_add(delivery_minutes).unwrap();

        let total_allocations = allocations_from_delivery
            .checked_add(allocations_from_bags_sold)
            .and_then(|v|v.checked_add(allocation_from_bags_spread))
            .and_then(|v|v.checked_add(data.amount_from_donations))
            .unwrap();
        calc_allocations_total = calc_allocations_total.checked_add(total_allocations).unwrap();


        ScoutVals {
            name: uid_2_name_map.get(uid.as_str()).map_or("".to_string(), |v|v.name.clone()),
            uid: uid.clone(),
            bags_sold: data.num_bags_sold,
            bags_spread: data.num_bags_spread,
            delivery_minutes: delivery_minutes,
            total_donations: data.amount_from_donations,
            allocation_from_bags_sold: allocations_from_bags_sold,
            allocation_from_bags_spread: allocation_from_bags_spread,
            allocation_from_delivery: allocations_from_delivery,
            allocation_total: total_allocations,
        }
    }).collect::<Vec<ScoutVals>>();

    let svars = svar_map.get("TROOP_TOTALS").unwrap();
    // First Record is special Troop Totals
    scout_vals.insert(0, ScoutVals{
        name: "Scout Alloc Totals".to_string(),
        uid: "".to_string(),
        bags_sold: svars.num_bags_sold,
        bags_spread: calc_num_bags_spread,
        delivery_minutes: calc_delivery_minutes,
        total_donations: total_calc_donations,
        allocation_from_bags_sold: calc_allocations_from_bags_sold,
        allocation_from_bags_spread: calc_allocations_from_bags_spread,
        allocation_from_delivery: calc_allocations_from_delivery,
        allocation_total: calc_allocations_total,
    });
    scout_vals
}

/////////////////////////////////////////////////
/////////////////////////////////////////////////
#[derive(Properties, PartialEq)]
struct AllocationReportRowProps {
    scoutvals: ScoutVals,
    class: Classes,
}
#[function_component(AllocationReportRow)]
fn allocation_report_row(props: &AllocationReportRowProps) -> Html {
    html! {
        <tr class={ classes!(props.class.clone())}>
            <td>{&props.scoutvals.name}</td>
            <td>{&props.scoutvals.uid}</td>
            <td>{&props.scoutvals.bags_sold}</td>
            <td>{&props.scoutvals.bags_spread}</td>
            <td>{props.scoutvals.delivery_minutes.round_dp(2).to_string()}</td>
            <td>{decimal_to_money_string(&props.scoutvals.total_donations)}</td>
            <td>{decimal_to_money_string(&props.scoutvals.allocation_from_bags_sold)}</td>
            <td>{decimal_to_money_string(&props.scoutvals.allocation_from_bags_spread)}</td>
            <td>{decimal_to_money_string(&props.scoutvals.allocation_from_delivery)}</td>
            <td>{decimal_to_money_string(&props.scoutvals.allocation_total)}</td>
        </tr>
    }
}

/////////////////////////////////////////////////
/////////////////////////////////////////////////
#[derive(Properties, PartialEq)]
struct AllocationReportProps {
    reportlist: Vec<ScoutVals>,
}
#[function_component(AllocationReport)]
fn allocation_report(props: &AllocationReportProps) -> Html {

    let on_download_report = {
        Callback::from(move |evt: MouseEvent| {
            evt.prevent_default();
            evt.stop_propagation();
            log::info!("on_download_report");
        })
    };

    let on_release_funds_form_submission = {
        Callback::from(move |evt: FocusEvent| {
            evt.prevent_default();
            evt.stop_propagation();
            log::info!("on_release_funds_form_submission");

        })
    };

    html! {
        <div class="col-md-9">
            <div class="card">
                <h5 class="card-header justify-content-center text-center">
                    {"Allocation Report"}
                    <button type="button" class="btn reports-view-setting-btn ms-3"
                            onclick={on_download_report} data-bs-toggle="tooltip"
                            // data-reportfields={JSON.stringify(perScoutReportDataFields)}
                            // data-reportheaders={JSON.stringify(perScoutReportDataHeaders)}
                            title="Download Report">
                        <i class="bi bi-cloud-download" fill="currentColor"></i>
                    </button>
                </h5>
                <div class="card-body">
                    <form onsubmit={on_release_funds_form_submission}>
                        <div class="table-responsive-xxl" id="fundsReleaseTables">
                            <table class="table table-striped">
                                <thead>
                                    <tr>
                                        <th scope="col">{"Name"}</th>
                                        <th scope="col">{"Id"}</th>
                                        <th scope="col">{"# Bags Sold"}</th>
                                        <th scope="col">{"# Bags Spread"}</th>
                                        <th scope="col">{"# Delivery Minutes"}</th>
                                        <th scope="col">{"$ Donations"}</th>
                                        <th scope="col">{"$ Allocations from Bags Sold"}</th>
                                        <th scope="col">{"$ Allocations from Spreading"}</th>
                                        <th scope="col">{"$ Allocations from Delivery"}</th>
                                        <th scope="col">{"$ Total Allocations"}</th>
                                    </tr>
                                    <AllocationReportRow
                                        scoutvals={props.reportlist[0].clone()}
                                        class={classes!("alloc-first-row")}/>
                                </thead>
                                <tbody>
                                {
                                    props.reportlist.iter().skip(1).map(|scout| {
                                        html!{
                                            <AllocationReportRow
                                                scoutvals={scout.clone()}
                                                class={classes!("alloc-scout-row")}
                                            />
                                        }
                                    }).collect::<Html>()
                                }
                                </tbody>
                            </table>
                        </div>
                        <button type="submit" class="btn btn-primary my-2 float-end"
                                id="releaseFundsBtn"
                                data-bs-toggle="tooltip"
                                title="Release Report to Scouts">
                            <span class="spinner-border spinner-border-sm me-1" role="status"
                                  aria-hidden="true" id="formReleaseFundsSpinner" style="display: none" />
                            {"Save and Release Funds"}
                        </button>
                    </form>
                </div>
            </div>
        </div>
    }
}

/////////////////////////////////////////////////
/////////////////////////////////////////////////
#[derive(Properties, PartialEq)]
struct SalesTableProps {
    svarsmap: FrClosureStaticData,
}
#[function_component(SalesTable)]
fn sales_table(props: &SalesTableProps) -> Html {
    let svars = props.svarsmap.get("TROOP_TOTALS").unwrap();
    html! {
        <table class="table table-striped caption-top" id="fundsSalesTable">
            <caption>{"Sales"}</caption>
            <thead>
                <tr>
                    <th scope="col"></th>
                    <th scope="col">{"Num Sold"}</th>
                    <th scope="col">{"Sales"}</th>
                </tr>
            </thead>
            <tbody>
                <tr>
                    <td>{"Bags of Mulch"}</td>
                    <td>{svars.num_bags_sold}</td>
                    <td>{decimal_to_money_string(&svars.amount_from_bags_sales)}</td>
                </tr>
                <tr>
                    <td>{"Spreading Jobs"}</td>
                    <td>{svars.num_bags_to_spread_sold}</td>
                    <td>{decimal_to_money_string(&svars.amount_from_bags_to_spread_sales)}</td>
                </tr>
                <tr>
                    <td>{"Donations"}</td>
                    <td></td>
                    <td>{decimal_to_money_string(&svars.amount_from_donations)}</td>
                </tr>
            </tbody>
            <tfoot>
                <tr>
                    <td>{"Total Collected"}</td>
                    <td></td>
                    <td>{decimal_to_money_string(&svars.amount_total_collected)}</td>
                </tr>
            </tfoot>
        </table>
    }
}

/////////////////////////////////////////////////
/////////////////////////////////////////////////
#[derive(Properties, PartialEq)]
struct AllocationsTableProps {
    dvars: DynamicVars,
    svarsmap: FrClosureStaticData,
}
#[function_component(AllocationsTable)]
fn allocations_table(props: &AllocationsTableProps) -> Html {
    let svars = props.svarsmap.get("TROOP_TOTALS").unwrap();
    html! {
        <>
        <table class="table table-striped table-responsive caption-top" id="fundsAllocationsTable">
            <caption>{"Allocations"}</caption>
            <tbody>
                <tr>
                    <td>{"Gross Profits"}</td>
                    <td>{decimal_to_money_string(&props.dvars.mulch_sales_gross)}</td>
                </tr>
                <tr>
                    <td>{"Min Allocations to Troop (est)"}</td>
                    <td>{decimal_to_money_string(&props.dvars.money_pool_for_troop)}</td>
                </tr>
                <tr>
                    <td>{"Max Allocations to Scouts (est)"}</td>
                    <td>{decimal_to_money_string(&props.dvars.money_pool_for_scouts_sub_pools)}</td>
                </tr>
                <tr>
                </tr>
            </tbody>
            <tfoot>
            </tfoot>
        </table>

        <table class="table table-striped caption-top mb-0" id="fundsScoutAllocationsTable">
            <caption>{"Scout Allocations"}</caption>
            <tbody>
                <tr>
                    <td>{"For Mulch Bag Sales (est)"}</td>
                    <td>{decimal_to_money_string(&props.dvars.money_pool_for_scout_sales)}</td>
                </tr>
                <tr>
                    <td>{"Avg Allocation per Bag"}</td>
                    <td>{decimal_to_money_string(&props.dvars.per_bag_avg_earnings)}</td>
                </tr>
                <tr>
                    <td>{"For Delivery (est)"}</td>
                    <td>{decimal_to_money_string(&props.dvars.money_pool_for_scout_delivery)}</td>
                </tr>
                <tr>
                    <td>{"Total Delivery Minutes"}</td>
                    <td>{
                        Decimal::from_f64(svars.delivery_time_total.as_secs_f64()/60.0)
                            .unwrap()
                            .round_dp(2)
                            .to_string()
                    }</td>
                </tr>
                <tr>
                    <td>{"Allocation Per Delivery Minute"}</td>
                    <td>{decimal_to_money_string(&props.dvars.delivery_earnings_per_minute)}</td>
                </tr>
            </tbody>
        </table>
        </>
    }
}

/////////////////////////////////////////////////
/////////////////////////////////////////////////
#[derive(Properties, PartialEq)]
struct CurrencyWidgetProps {
    id: String,
    value: Decimal,
    label: String,
    oninput: Callback<InputEvent>,
}
#[function_component(CurrencyWidget)]
fn currency_widget(props: &CurrencyWidgetProps) -> Html {
    // function formatCurrency(evt) {
    //     evt.currentTarget.value = USD(evt.currentTarget.value).format();
    // }
    html! {
        <div class="form-floating">
            <input type="text" min="0" step="any" class="form-control"
                   //pattern={r"^\$\d{1,3}(,\d{3})*(\.\d+)?$"}
                   id={props.id.clone()}
                   value={props.value.to_string()}
                   placeholder="$0.00"
                   // onblur={formatcurrency}
                   oninput={props.oninput.clone()}
            />
            <label for={props.id.clone()}>{props.label.clone()}</label>
        </div>
    }
}

/////////////////////////////////////////////////
/////////////////////////////////////////////////
#[derive(Properties, PartialEq)]
struct AllocationsFormProps {
    dvars: DynamicVars,
    svarsmap: FrClosureStaticData,
    oninput: Callback<InputEvent>,
}
#[function_component(AllocationsForm)]
fn allocations_form(props: &AllocationsFormProps) -> Html {
    html! {
        <form>
            <div class="row mb-2">
                <CurrencyWidget id="formBankDeposited"
                                value={props.dvars.bank_deposited.clone()}
                                label="Amount Deposited in Bank"
                                oninput={props.oninput.clone()}
                />
            </div>
            <div class="row mb-2">
                <CurrencyWidget id="formMulchCost"
                                value={props.dvars.mulch_cost.clone()}
                                label="Amount Paid for Mulch"
                                oninput={props.oninput.clone()}
                />
            </div>

            <div class="table-responsive" id="fundsReleaseTables">
                <SalesTable svarsmap={props.svarsmap.clone()} />
                if Decimal::ZERO != props.dvars.bank_deposited && Decimal::ZERO != props.dvars.mulch_cost {
                    <AllocationsTable dvars={props.dvars.clone()} svarsmap={props.svarsmap.clone()} />
                }
            </div>
        </form>
    }
}

/////////////////////////////////////////////////
/////////////////////////////////////////////////
#[function_component(StaticDataLoadingSpinny)]
fn static_data_loading_spinny() -> Html {
    html! {
        <div class="justify-content-center text-center">
            <h2>{"Loading Static Closeout Report Data..."}</h2>
            <span role="status" class="spinner-border ms-1"/>
        </div>
    }
}

/////////////////////////////////////////////////
/////////////////////////////////////////////////
macro_rules! get_new_input_val_maybe{
// using a ty token type for macthing datatypes passed to maccro
    ($a:expr,$b:ident,$c:expr)=>{
        if $a.$b != $c {
            let mut new_dvars = $a.clone();
            new_dvars.$b = $c;
            Some(new_dvars)
        } else {
            None
        }
    }
}

#[function_component(CloseoutFundraiser)]
pub fn closeout_fundraiser_page() -> Html {

    let dvars = use_state_eq(|| DynamicVars::new());
    let scout_report_list: yew::UseStateHandle<Vec<ScoutVals>> = use_state_eq(|| Vec::new());
    let fr_closure_static_data: yew::UseStateHandle<Option<FrClosureStaticData>> = use_state_eq(|| None);

    let on_download_summary = {
        Callback::from(move |evt: MouseEvent| {
            evt.prevent_default();
            evt.stop_propagation();
            log::info!("on_download_summary");

        })
    };

    let on_allocation_form_inputs_change = {
        let dvars = dvars.clone();
        let fr_closure_static_data = fr_closure_static_data.clone();

        Callback::from(move |evt: InputEvent| {
            evt.prevent_default();
            evt.stop_propagation();
            log::info!("on_allocation_form_inputs_change");

            let input_elm = evt.target()
                .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
                .unwrap();

            let new_val = Decimal::from_str(input_elm.value().as_str()).unwrap();

            let new_dvars_opt = match input_elm.id().as_str() {
                "formBankDeposited" => {
                    get_new_input_val_maybe!(*dvars, bank_deposited, new_val)
                },
                "formMulchCost" => {
                    get_new_input_val_maybe!(*dvars, mulch_cost, new_val)
                },
                _ => {
                    log::error!("Invalid input elememnt");
                    None
                },
            };
            if let Some(new_dvars) = new_dvars_opt {
                match calculate_new_dvars(new_dvars, (*fr_closure_static_data).as_ref().unwrap().clone()) {
                    Some(new_dvars)=>{
                        dvars.set(new_dvars);
                    },
                    None=>{},
                };
            }

        })
    };

    {
        let fr_closure_static_data = fr_closure_static_data.clone();
        let dvars = dvars.clone();
        let scout_report_list = scout_report_list.clone();
        use_effect(move || {
            wasm_bindgen_futures::spawn_local(async move {
                log::info!("Downloading Static Fr Closure Data");
                let resp = get_fundraiser_closure_static_data().await.unwrap();
                log::info!("Data has been downloaded");
                fr_closure_static_data.set(Some(resp));
                if dvars.bank_deposited > Decimal::ZERO && dvars.mulch_cost > Decimal::ZERO {
                    scout_report_list.set(calculate_per_scout_report(
                            &*dvars, (*fr_closure_static_data).as_ref().unwrap().clone()));
                }
            });

            ||{}
        });
    }

    if (*fr_closure_static_data).is_none() {
        html! { <StaticDataLoadingSpinny/> }
    } else {
        html!{
            <>
                <div class="col-xs-1 d-flex justify-content-center">
                    <h4>{"Funds Release Page"}</h4>
                </div>
                <div class="releaseFundsCards">
                    <div class="row">

                        <div class="col">

                            <div class="card" style="maxWidth: 30rem">
                                <h5 class="card-header justify-content-center text-center">
                                    {"Allocation Calculations"}
                                    <button type="button" class="btn reports-view-setting-btn ms-3"
                                            onclick={on_download_summary.clone()} data-bs-toggle="tooltip"
                                            title="Download Summary">
                                        <i class="bi bi-cloud-download" fill="currentColor"></i>
                                    </button>
                                </h5>
                                <div class="card-body">
                                    <AllocationsForm
                                        oninput={on_allocation_form_inputs_change.clone()}
                                        dvars={(*dvars).clone()}
                                        svarsmap={(*fr_closure_static_data).as_ref().unwrap().clone()}
                                    />
                                </div>
                            </div> // End of Card
                        </div>
                        if (*scout_report_list).len() > 0 {
                            <AllocationReport reportlist={(*scout_report_list).clone()}/>
                        }
                    </div>
                </div>
            </>
        }
    }
}
