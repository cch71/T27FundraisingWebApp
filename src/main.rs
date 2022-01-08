
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
use std::rc::Rc;
use std::cell::RefCell;
use wasm_bindgen::JsCast;
use web_sys::{window, MouseEvent, HtmlSelectElement, HtmlButtonElement, HtmlInputElement, HtmlTextAreaElement};
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
thread_local! {
    static REPORT_ISSUE_DLG: Rc<RefCell<Option<bootstrap::Modal>>> = Rc::new(RefCell::new(None));
}

fn show_report_issue_dlg(do_show: bool) {
    let document = gloo_utils::document();

    document.get_element_by_id("formSummary")
        .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
        .unwrap()
        .set_value("");

    document.get_element_by_id("formDescription")
        .and_then(|t| t.dyn_into::<HtmlTextAreaElement>().ok())
        .unwrap()
        .set_value("");

    REPORT_ISSUE_DLG.with(|v|{
        if (*v).borrow().is_none() {
            *v.borrow_mut() = Some(bootstrap::get_modal_by_id("xmitIssueDlg").unwrap());
        };

        if do_show {
            v.borrow().as_ref().unwrap().show();
        } else {
            v.borrow().as_ref().unwrap().hide();
            *v.borrow_mut() = None;
        }
    });
}

#[function_component(ReportIssueDlg)]
pub fn report_issue() -> Html
{
    let spinner_state = use_state_eq(||"d-none");
    let on_submit_issue = {
        let spinner_state = spinner_state.clone();

        Callback::from(move |evt: MouseEvent|{
            evt.prevent_default();
            evt.stop_propagation();
            let document = gloo_utils::document();
            let btn_elm = evt.target()
                .and_then(|t| t.dyn_into::<HtmlButtonElement>().ok())
                .unwrap();
            let summary_elm = document.get_element_by_id("formSummary")
                .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
                .unwrap();
            let desc_elm = document.get_element_by_id("formDescription")
                .and_then(|t| t.dyn_into::<HtmlTextAreaElement>().ok())
                .unwrap();

            spinner_state.set("d-inline-block");
            btn_elm.set_disabled(true);

            let desc = desc_elm.value();
            let summary = summary_elm.value();
            if 0==desc.len() || 0==summary.len() {
                if 0==desc.len() {
                    let _ = desc_elm.class_list().add_1("is-invalid");
                } else {
                    let _ = desc_elm.class_list().remove_1("is-invalid");
                }
                if 0==summary.len() {
                    let _ = summary_elm.class_list().add_1("is-invalid");
                } else {
                    let _ = summary_elm.class_list().remove_1("is-invalid");
                }
                spinner_state.set("d-none");
                return;
            }

            let reporting_user = get_active_user().get_id();
            let spinner_state = spinner_state.clone();
            wasm_bindgen_futures::spawn_local(async move {
                    let rslt = report_new_issue(&reporting_user, &summary, &desc).await;
                spinner_state.set("d-none");
                btn_elm.set_disabled(false);
                if let Err(err) = rslt {
                    gloo_dialogs::alert(&format!("Failed to submit report: {:#?}", err));
                } else {
                    show_report_issue_dlg(false);
                }
            });
        })
    };

    html! {
        <div class="modal fade" id="xmitIssueDlg"
             tabIndex="-1" role="dialog" aria-labelledby="xmitIssueDlgTitle" aria-hidden="true">
            <div class="modal-dialog modal-dialog-centered" role="document">
                <div class="modal-content">
                    <div class="modal-header">
                        <h5 class="modal-title" id="xmitIssueDlgLongTitle">
                            {"Report a issue with Fundraiser App"}
                        </h5>
                    </div>
                    <div class="modal-body">
                        <form class="needs-validation" id="formXmitIssue" novalidate=true>
                            <div class="row mb-2 g-2">
                                <div class="form-floating">
                                    <input class="form-control" type="text" autocomplete="fr-new-issue" id="formSummary"
                                           placeholder="Summary" required=true maxlength="255"/>
                                    <label for="formSummary">
                                        {"Summary (255 Max Chars)"}
                                    </label>
                                </div>
                            </div>
                            <div class="row mb-2 g-2">
                                <div class="form-floating">
                                    <textarea class="form-control" rows="10" required=true id="formDescription"/>
                                    <label for="formDescription">{"Description of problem"}</label>
                                </div>
                            </div>
                        </form>
                    </div>
                    <div class="modal-footer">
                        <button type="button" class="btn btn-secondary" data-bs-dismiss="modal">{"Cancel"}</button>
                        <button type="button" class="btn btn-primary" onclick={on_submit_issue}>
                            <span class={format!("spinner-border spinner-border-sm me-1 {}",*spinner_state)} role="status"
                                  aria-hidden="true" id="formXmitIssueSpinner" />
                                  {"Submit"}
                        </button>
                    </div>
                </div>
            </div>
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
    pub username: String,
    pub onlogoff: Callback<MouseEvent>,
    pub onreportissue: Callback<MouseEvent>,
}

#[function_component(AppNav)]
pub fn app_nav(props: &AppNavProps) -> Html
{
    let _ = use_history().unwrap(); // This forces re-render on path changes
    //log::info!("~~~~~~~ Re Rendered ~~~~~~~~~~~~~~");
    let userlabel = if props.username != props.userid {
        format!("{} ({})", props.username, props.userid)
    } else {
        props.userid.clone()
    };

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
                        {userlabel}
                    </a>
                    <div class="dropdown-menu dropdown-menu-end" aria-labelledby="navbarDropdown">
                        <a class="dropdown-item" onclick={props.onreportissue.clone()} href="#" data-bs-toggle="modal">
                            {"Report Issue"}
                        </a>
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

            html! {
                <BrowserRouter>
                    <AppNav userid={user_id.clone()} username={user_name} onlogoff={on_logoff} onreportissue={on_reportissue}/>
                    <main class="flex-shrink-0">
                        <Switch<AppRoutes> render={Switch::render(switch)} />
                        <ReportIssueDlg/>
                    </main>
                    <AppFooter>
                        <AddNewOrderButton userid={user_id}/>
                    </AppFooter>
                </BrowserRouter>
            }
        }
    }
}


fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::start_app::<Model>();
}
