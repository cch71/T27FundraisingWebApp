
mod bootstrap;
mod datatable;
mod google_charts;
mod auth_utils;
mod data_model_orders;
mod data_model_reports;
mod data_model;
mod currency_utils;
mod gql_utils;

use yew::prelude::*;
use yew_router::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{HtmlSelectElement, HtmlInputElement, HtmlTextAreaElement};
use rust_decimal::prelude::*;

use auth_utils::{login, logout, is_authenticated};
use data_model::*;

mod components;
use components::{
    issue_report_dlg::{ReportIssueDlg, show_report_issue_dlg},
    navbar::{AppNav},
    add_new_order_button::{AddNewOrderButton},
};

mod pages;
use pages::{
    home::Home,
    reports::Reports,
    order_form::OrderForm,
    order_donations::OrderDonations,
    order_products::OrderProducts,
    timecards::Timecards,
    closeout_fundraiser::CloseoutFundraiser,
};

/////////////////////////////////////////////////
///
pub(crate) fn get_html_input_value(id: &str, document: &web_sys::Document) -> Option<String> {
    let value = document.get_element_by_id(id)
        .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
        .unwrap()
        .value();
    if 0==value.len() {
        None
    } else {
        Some(value)
    }
}

/////////////////////////////////////////////////
///
pub(crate) fn get_html_textarea_value(id: &str, document: &web_sys::Document) -> Option<String> {
    let value = document.get_element_by_id(id)
        .and_then(|t| t.dyn_into::<HtmlTextAreaElement>().ok())
        .unwrap()
        .value();
    if 0==value.len() {
        None
    } else {
        Some(value)
    }
}

/////////////////////////////////////////////////
///
pub(crate) fn save_to_active_order() {
    if !is_active_order() { return; }

    let document = gloo_utils::document();
    let mut order = get_active_order().unwrap();

    if let Some(order_owner_element) = document.get_element_by_id("formOrderOwner")
        .and_then(|t| t.dyn_into::<HtmlSelectElement>().ok())
    {
        // If it isn't there then it is because we aren't in admin mode
        order.order_owner_id = order_owner_element.value();
    }

    order.customer.name = get_html_input_value("formCustomerName", &document).unwrap_or("".to_string());
    order.customer.phone = get_html_input_value("formPhone", &document).unwrap_or("".to_string());
    order.customer.email = get_html_input_value("formEmail", &document);
    order.customer.addr1 = get_html_input_value("formAddr1", &document).unwrap_or("".to_string());
    order.customer.addr2 = get_html_input_value("formAddr2", &document);
    order.customer.neighborhood = Some(document.get_element_by_id("formNeighborhood")
        .and_then(|t| t.dyn_into::<HtmlSelectElement>().ok())
        .unwrap()
        .value());
    order.special_instructions = get_html_textarea_value("formSpecialInstructions", &document);
    order.amount_cash_collected = get_html_input_value("formCashPaid", &document);
    order.amount_checks_collected = get_html_input_value("formCheckPaid", &document);
    order.check_numbers = get_html_input_value("formCheckNumbers", &document);
    order.will_collect_money_later = Some(document.get_element_by_id("formCollectLater")
        .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
        .unwrap()
        .checked());
    order.is_verified = Some(document.get_element_by_id("formIsVerified")
        .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
        .unwrap()
        .checked());
    // This must come after setting checks/cash collected
    let total_collected = order.get_total_collected();
    if total_collected == Decimal::ZERO {
        order.amount_total_collected = None;
    } else {
        order.amount_total_collected = Some(total_collected.to_string());
    }

    log::info!("Saving Order: {:#?}", &order);
    update_active_order(order).unwrap();

}

/////////////////////////////////////////////////
/////////////////////////////////////////////////

#[derive(Properties, PartialEq)]
pub struct AppFooterProps {
    #[prop_or_default]
    pub children: Children,
}

#[function_component(AppFooter)]
pub fn app_footer(props: &AppFooterProps) -> Html
{
    let cur_win_loc = gloo_utils::window().location().pathname().unwrap();
    // log::info!("!!!!! WinLoc: {}", cur_win_loc);

    html! {
        <footer class="footer mt-auto py-3 bg-light">
            if !cur_win_loc.starts_with("/order") {//TODO this kill every child
                {for props.children.iter()}
            }
        </footer>
    }
}

/////////////////////////////////////////////////
/////////////////////////////////////////////////


/////////////////////////////////////////////////
// Route Logic
#[derive(Clone, Routable, PartialEq, Debug)]
pub enum AppRoutes {
    #[at("/")]
    Home,
    #[at("/order")]
    OrderForm,
    #[at("/orderproducts")]
    OrderProducts,
    #[at("/orderdonations")]
    OrderDonations,
    #[at("/reports")]
    Reports,
    #[at("/timecards")]
    Timecards,
    #[at("/frcloseout")]
    FundraiserCloseout,
    #[not_found]
    #[at("/404")]
    NotFound,
}

fn switch(route: &AppRoutes) -> Html {
    // This is kindof a hack to save order form before we switch away
    let document = web_sys::window().unwrap().document().unwrap();
    if document.get_element_by_id("newOrEditOrderForm").is_some() {
        save_to_active_order();
    }
    //log::info!("````````` switcthing ``````````, {:?}  {}", route, is_some);

    match route {
        AppRoutes::Home => html!{<Home/>},
        AppRoutes::OrderForm => html!{<OrderForm/>},
        //TODO: should these be in a seperate routing table?
        AppRoutes::OrderProducts => html!{<OrderProducts/>},
        AppRoutes::OrderDonations => html!{<OrderDonations/>},
        AppRoutes::Reports => html!{<Reports/>},
        AppRoutes::Timecards => html!{<Timecards/>},
        AppRoutes::FundraiserCloseout => html!{<CloseoutFundraiser/>},

        AppRoutes::NotFound => html! { <h1>{ "404" }</h1> },
    }
}

/////////////////////////////////////////////////
/////////////////////////////////////////////////

pub enum AppMsg {
    NotAuthenticated,
    Authenticated,
    NoOp,
    Logout,
    ReportIssue,
}
type Msg = AppMsg;

struct Model {
    is_loading: bool,
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        ctx.link().send_future(async move {
            if is_authenticated().await {
                match get_active_user_async().await {
                    Some(user_info)=> {
                        // We are authenticated so get initial config stuff before we bring up ui
                        load_config().await;
                        // Preload summary_report data TODO: this is goofy
                        let _ = get_summary_report_data(&user_info.get_id(), 10).await;
                        log::info!("Showing UI");
                        Msg::Authenticated
                    },
                    None=>Msg::NoOp,
                }
            } else {
                Msg::NotAuthenticated
            }
        });

        Self {
            is_loading: true,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Authenticated => {
                self.is_loading = false;
                true
            },
            Msg::NotAuthenticated => {
                log::info!("Not authenticated need to get signed in");
                ctx.link().send_future(async move {
                    login().await;
                    Msg::NoOp
                });
                false
            },
            Msg::Logout=>{
                log::info!("User has asked to logout");
                ctx.link().send_future(async move {
                    delete_report_settings(); // We needs to delete report settings in case admin is on view user can't support
                    logout().await;
                    Msg::NoOp
                });
                false
            },
            Msg::ReportIssue=>{
                log::info!("Bringing up Report Issue Dlg");
                show_report_issue_dlg(true);
                // I could trigger this to bring up dlg by returning true but shutting it down would be harder
                false
            },
            Msg::NoOp=>false,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let on_logoff = ctx.link().callback(|_| Msg::Logout);
        let on_reportissue = ctx.link().callback(|_| Msg::ReportIssue);

        if self.is_loading {
            html! {
                <div id="notReadyView" class="col-xs-1 d-flex justify-content-center" >
                    <div class="spinner-border" role="status">
                        <span class="visually-hidden">{ "Loading..."}</span>
                    </div>
                </div>
            }
        } else {
            let active_user = get_active_user();
            let user_id = active_user.get_id();
            let user_name = active_user.get_name();
            let is_admin = active_user.is_admin();

            html! {
                <BrowserRouter>
                    <AppNav userid={user_id.clone()} username={user_name} isadmin={is_admin} onlogoff={on_logoff} onreportissue={on_reportissue}/>
                    <main class="flex-shrink-0">
                        <Switch<AppRoutes> render={Switch::render(switch)} />
                        <ReportIssueDlg/>
                    </main>
                    <AppFooter>
                        if are_sales_still_allowed() || is_admin {
                            <AddNewOrderButton userid={user_id}/>
                        }
                    </AppFooter>
                </BrowserRouter>
            }
        }
    }
}


fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    log::info!("RelVer: {}", std::option_env!("AWS_COMMIT_ID").unwrap_or("?"));
    yew::start_app::<Model>();
}
