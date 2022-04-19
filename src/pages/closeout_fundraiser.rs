
use yew::prelude::*;
use web_sys::{Event, InputEvent, MouseEvent, Element, HtmlElement, HtmlButtonElement, HtmlInputElement, HtmlSelectElement};
use rust_decimal::prelude::*;

use crate::data_model::*;
use std::time::{ Duration };

#[derive(Default, Debug, PartialEq)]
struct DynamicVars {
    bank_deposited: Decimal,
    mulch_cost: Decimal,
    bags_sold: u64,
    bags_total_sales: Decimal,
    bags_spread: u64,
    spreading_total_sales: Decimal,
    total_donated: Decimal,
    total_collected: Decimal,
    mulch_sales_gross: Decimal,
    troop_percentage: f32,
    scout_percentage: f32,
    scout_selling_percentage: f32,
    per_bag_avg_earnings: Decimal,
    scout_delivery_percentage: f32,
    delivery_minutes: u64,
    delivery_earnings: Decimal,
}
impl DynamicVars {
    fn new()->Self {
        DynamicVars::default()
    }

}

#[derive(Default, Debug, PartialEq)]
struct ScoutVals {
    name: String,
    uid: String,
    bags_sold: u64,
    bags_spread: u64,
    delivery_minutes: String,
    total_donations: Decimal,
    allocation_from_bags_sold: Decimal,
    allocation_from_bags_spread: Decimal,
    allocations_from_delivery: Decimal,
    allocations_total: Decimal,
}

/////////////////////////////////////////////////
/////////////////////////////////////////////////
#[function_component(CloseoutFundraiser)]
pub fn closeout_fundraiser_page() -> Html {

    let dvars = use_state_eq(|| DynamicVars::new());
    let scout_report_list: yew::UseStateHandle<Vec<ScoutVals>> = use_state_eq(|| Vec::new());

    let on_download_summary = {
        Callback::from(move |evt: MouseEvent| {
            evt.prevent_default();
            evt.stop_propagation();
            log::info!("on_download_summary");

        })
    };

    let on_download_report = {
        Callback::from(move |evt: MouseEvent| {
            evt.prevent_default();
            evt.stop_propagation();
            log::info!("on_download_report");

        })
    };

    let on_allocation_form_submission = {
        Callback::from(move |evt: FocusEvent| {
            evt.prevent_default();
            evt.stop_propagation();
            log::info!("on_allocation_form_submission");

        })
    };

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
                                <form onsubmit={on_allocation_form_submission}>
                                    <div class="row mb-2">
                                        <CurrencyWidget id="formBankDeposited"
                                                        defaultValue={dvars.bank_deposited}
                                                        label="Amount Deposited in Bank"
                                                        oninput={on_allocation_form_inputs_change}
                                        />
                                    </div>
                                    <div class="row mb-2">
                                        <CurrencyWidget id="formMulchCost"
                                                        defaultValue={dvars.mulch_cost}
                                                        label="Amount Paid for Mulch"
                                                        oninput={on_allocation_form_inputs_change}
                                        />
                                    </div>

                                    <div class="table-responsive" id="fundsReleaseTables">
                                        <table class="table table-striped caption-top">
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
                                                    <td>{dvars.bags_sold}</td>
                                                    <td>{dvars.bags_total_sales}</td>
                                                </tr>
                                                <tr>
                                                    <td>{"Spreading Jobs"}</td>
                                                    <td>{dvars.bags_spread}</td>
                                                    <td>{dvars.spreading_total_sales}</td>
                                                </tr>
                                                <tr>
                                                    <td>{"Donations"}</td>
                                                    <td></td>
                                                    <td>{dvars.total_donated}</td>
                                                </tr>
                                            </tbody>
                                            <tfoot>
                                                <tr>
                                                    <td>{"Total Collected"}</td>
                                                    <td></td>
                                                    <td>{dvars.total_collected}</td>
                                                </tr>
                                            </tfoot>
                                        </table>

                                        <table class="table table-striped table-responsive caption-top">
                                            <caption>{"Allocations"}</caption>
                                            <tbody>
                                                <tr>
                                                    <td>{"Gross Profits"}</td>
                                                    <td>{dvars.mulch_sales_gross}</td>
                                                </tr>
                                                <tr>
                                                    <td>{"Min Allocations to Troop (est)"}</td>
                                                    <td>{dvars.troop_percentage}</td>
                                                </tr>
                                                <tr>
                                                    <td>{"Max Allocations to Scouts (est)"}</td>
                                                    <td>{dvars.scout_percentage}</td>
                                                </tr>
                                                <tr>
                                                    <td colSpan="4">
                                                        <table class="table table-striped caption-top mb-0">
                                                            <caption>{"Scout Allocations"}</caption>
                                                            <tbody>
                                                                <tr>
                                                                    <td>{"For Mulch Bag Sales (est)"}</td>
                                                                    <td>{dvars.scout_selling_percentage}</td>
                                                                </tr>
                                                                <tr>
                                                                    <td>{"Avg Allocation per Bag"}</td>
                                                                    <td>{dvars.per_bag_avg_earnings}</td>
                                                                </tr>
                                                                <tr>
                                                                    <td>{"For Delivery (est)"}</td>
                                                                    <td>{dvars.scout_delivery_percentage}</td>
                                                                </tr>
                                                                <tr>
                                                                    <td>{"Total Delivery Minutes"}</td>
                                                                    <td>{dvars.delivery_minutes}</td>
                                                                </tr>
                                                                <tr>
                                                                    <td>{"Allocation Per Delivery Minute"}</td>
                                                                    <td>{dvars.delivery_earnings}</td>
                                                                </tr>
                                                            </tbody>
                                                        </table>
                                                    </td>
                                                </tr>
                                            </tbody>
                                            <tfoot>
                                            </tfoot>
                                        </table>
                                    </div>

                                    <button type="submit" class="btn btn-primary my-2 float-end"
                                            id="generateReportsBtn"
                                            data-bs-toggle="tooltip"
                                            title="Generate Data">
                                            {"Generate Data"}
                                    </button>
                                </form>
                            </div>
                        </div> // End of Card
                    </div>
                    {
                        if !scout_report_list.empty() {
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
                                                            <th scope="col">{"# Bags to Spread Sold"}</th>
                                                            <th scope="col">{"# Delivery Minutes"}</th>
                                                            <th scope="col">{"$ Donations"}</th>
                                                            <th scope="col">{"$ Allocations from Bags Sold"}</th>
                                                            <th scope="col">{"$ Allocations from Spreading"}</th>
                                                            <th scope="col">{"$ Allocations from Delivery"}</th>
                                                            <th scope="col">{"$ Total Allocations"}</th>
                                                        </tr>
                                                        // <tr style="backgroundColor: DarkSeaGreen">
                                                        //     <td>{"Scout Alloc Totals"}</td>
                                                        //     <td>{""}</td>
                                                        //     <td>{scout.bags_sold}</td>
                                                        //     <td>{scout.bags_spread}</td>
                                                        //     <td>{scout.delivery_minutes}</td>
                                                        //     <td>{scout.total_donations}</td>
                                                        //     <td>{scout.allocation_from_bags_sold}</td>
                                                        //     <td>{scout.allocation_from_bags_spread}</td>
                                                        //     <td>{scout.allocations_from_delivery}</td>
                                                        //     <td>{scout.allocations_total}</td>
                                                        // </tr>
                                                    </thead>
                                                    <tbody>
                                                    {
                                                        for scout in &scout_report_list {
                                                            <td>{scout.name}</td>
                                                            <td>{scout.uid}</td>
                                                            <td>{scout.bags_sold}</td>
                                                            <td>{scout.bags_spread}</td>
                                                            <td>{scout.delivery_minutes}</td>
                                                            <td>{scout.total_donations}</td>
                                                            <td>{scout.allocation_from_bags_sold}</td>
                                                            <td>{scout.allocation_from_bags_spread}</td>
                                                            <td>{scout.allocations_from_delivery}</td>
                                                            <td>{scout.allocations_total}</td>
                                                        }
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
                    }
                </div>
            </div>
        </>
    }
}
