
mod bootstrap;
mod datatable;
mod auth_utils;
mod data_model_orders;
mod data_model_reports;
mod data_model;
mod currency_utils;
mod gql_utils;

use yew::prelude::*;
use yew_router::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{window, MouseEvent, HtmlSelectElement, HtmlInputElement, HtmlTextAreaElement};
use rust_decimal::prelude::*;

use auth_utils::{login, logout, is_authenticated};
use data_model::*;


mod pages;
use pages::{
    home::Home,
    reports::Reports,
    order_form::OrderForm,
    order_donations::OrderDonations,
    order_products::OrderProducts,
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

    let document = web_sys::window().unwrap().document().unwrap();
    let mut order = get_active_order().unwrap();

    order.order_owner_id = document.get_element_by_id("formOrderOwner")
        .and_then(|t| t.dyn_into::<HtmlSelectElement>().ok())
        .unwrap()
        .value();
    order.customer.name = get_html_input_value("formCustomerName", &document).unwrap_or("".to_string());
    order.customer.phone = get_html_input_value("formPhone", &document).unwrap_or("".to_string());
    order.customer.email = get_html_input_value("formEmail", &document);
    order.customer.addr1 = get_html_input_value("formAddr1", &document).unwrap_or("".to_string());
    order.customer.addr2 = get_html_input_value("formAddr2", &document);
    order.customer.neighborhood = document.get_element_by_id("formNeighborhood")
        .and_then(|t| t.dyn_into::<HtmlSelectElement>().ok())
        .unwrap()
        .value();
    order.special_instructions = get_html_textarea_value("formSpecialInstructions", &document);
    order.amount_cash_collected = get_html_input_value("formCashPaid", &document);
    order.amount_checks_collected = get_html_input_value("formCheckPaid", &document);
    order.check_numbers = get_html_input_value("formCheckNumbers", &document);
    order.will_collect_money_later = document.get_element_by_id("formCollectLater")
        .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
        .unwrap()
        .checked();
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
pub struct AddNewOrderButtonProps {
    pub userid: String,
}

#[function_component(AddNewOrderButton)]
pub fn add_new_order_button(props: &AddNewOrderButtonProps) -> Html
{
    let history = use_history().unwrap();
    let on_add_new_order = {
        let history = history.clone();
        let userid = props.userid.clone();
        Callback::from(move |_| {
            log::info!("Starting process to add new order");
            create_new_active_order(&userid);
            history.push(AppRoutes::OrderForm);
        })
    };

    html! {
        <div class="add-order-widget float-end me-3 my-1">
            <label>{"Add New Order"}</label>
            <button type="button"
                    class="btn btn-outline-primary add-order-btn"
                    onclick={on_add_new_order}>
                <i class="bi bi-plus-square-fill" fill="currentColor"></i>
            </button>
        </div>
    }
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
    let cur_win_loc = window().unwrap().location().pathname().unwrap();
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

#[derive(Properties, PartialEq)]
pub struct AppNavProps {
    pub userid: String,
    pub onlogoff: Callback<MouseEvent>,
}

#[function_component(AppNav)]
pub fn app_nav(props: &AppNavProps) -> Html
{
    let _ = use_history().unwrap(); // This forces re-render on path changes
    //log::info!("~~~~~~~ Re Rendered ~~~~~~~~~~~~~~");

    html! {

        <nav class="navbar sticky-top navbar-expand-sm navbar-light bg-light" id="primaryNavBar">
            <a class="navbar-brand" href="#">
                <span>
                    <img class="navbar-logo ms-2" src="t27patch.png" alt="Logo" />
                </span>
            </a>

            <button class="navbar-toggler" type="button" data-bs-toggle="collapse" data-bs-target="#navbarNav"
                    aria-controls="navbarNav" aria-expanded="false" aria-label="Toggle navigation">
                <span class="navbar-toggler-icon"></span>
            </button>

            <div class="collapse navbar-collapse" id="navbarNav">
                <ul class="navbar-nav me-auto">
                    <li class="nav-item">
                        <Link<AppRoutes> classes="nav-link" to={AppRoutes::Home} >
                            {"Home"}
                        </Link<AppRoutes>>
                    </li>
                    if is_active_order() {
                        <li class="nav-item">
                            <Link<AppRoutes> classes="nav-link" to={AppRoutes::OrderForm} >
                                {"Order"}
                            </Link<AppRoutes>>
                        </li>
                    }
                    <li class="nav-item">
                        <Link<AppRoutes> classes="nav-link" to={AppRoutes::Reports} >
                            {"Reports"}
                        </Link<AppRoutes>>
                    </li>
                </ul>
                <span class="navbar-nav nav-item dropdown">
                    <a class="nav-link dropdown-toggle" href="#" id="navbarDropdown"
                       data-bs-toggle="dropdown" aria-expanded="false" role="button">
                        {&props.userid}
                    </a>
                    <div class="dropdown-menu dropdown-menu-end" aria-labelledby="navbarDropdown">
                        // <a class="dropdown-item" href="#" data-bs-toggle="modal">
                        //     {"Report Issue"}
                        // </a>
                        <a class="dropdown-item" onclick={props.onlogoff.clone()} href="#" data-bs-toggle="modal">
                            {"Logout"}
                        </a>
                    </div>
                </span>
            </div>
        </nav>
    }
}

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
                    Some(_)=> {
                        // We are authenticated so get initial config stuff before we bring up ui
                        load_config().await;
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
                    logout().await;
                    Msg::NoOp
                });
                false
            },
            Msg::NoOp=>false,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let on_logoff = ctx.link().callback(|_| Msg::Logout);

        if self.is_loading {
            html! {
                <div id="notReadyView" class="col-xs-1 d-flex justify-content-center" >
                    <div class="spinner-border" role="status">
                        <span class="visually-hidden">{ "Loading..."}</span>
                    </div>
                </div>
            }
        } else {
            let user_id = get_active_user().get_id();

            html! {
                <BrowserRouter>
                    <AppNav userid={user_id.clone()} onlogoff={on_logoff} />
                    <main class="flex-shrink-0">
                        <Switch<AppRoutes> render={Switch::render(switch)} />
                    </main>
                    <AppFooter>
                        <AddNewOrderButton userid={user_id}/>
                    </AppFooter>
                </BrowserRouter>
            }
        }
    }
}

// impl Model {
// }


fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::start_app::<Model>();
}
