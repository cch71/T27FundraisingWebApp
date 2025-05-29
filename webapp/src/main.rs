use yew::prelude::*;
use yew_router::prelude::*;

use data_model::{AppRoutes, NUM_TOP_SELLERS_TO_GET, are_sales_still_allowed, get_active_user, get_active_user_async, get_summary_report_data, is_active_order, load_config, save_to_active_order, clear_local_storage, clear_session_storage};
use js::auth_utils::{is_authenticated, login, logout};

mod components;
use components::{
    issue_report_dlg::{ReportIssueDlg, show_report_issue_dlg},
    navbar::AppNav,
};

mod pages;
use pages::home::Home;

use admin_pages::pages::{CloseoutFundraiser, FrConfigEditor};
use order_pages::{
    components::AddNewOrderButton,
    pages::{OrderDonations, OrderForm, OrderProducts},
};
use report_pages::pages::Reports;
use timecard_pages::Timecards;

/////////////////////////////////////////////////
/////////////////////////////////////////////////

#[derive(Properties, PartialEq)]
pub struct AppFooterProps {
    #[prop_or_default]
    pub children: Children,
}

#[function_component(AppFooter)]
pub fn app_footer(props: &AppFooterProps) -> Html {
    let cur_win_loc = gloo::utils::window().location().pathname().unwrap();

    html! {
        <footer class="footer mt-auto py-3">
            if !cur_win_loc.starts_with("/order") {//TODO this kill every child
                {for props.children.iter()}
            }
        </footer>
    }
}

/////////////////////////////////////////////////
/////////////////////////////////////////////////

/////////////////////////////////////////////////
/////////////////////////////////////////////////

#[function_component]
fn App() -> Html {
    let is_loading = use_state_eq(|| true);
    let is_order_active = use_state_eq(is_active_order);

    let route_switch = {
        let is_order_active = is_order_active.clone();
        move |route: AppRoutes| -> Html {
            // This is kind of a hack to save order form before we switch away
            let document = web_sys::window().unwrap().document().unwrap();
            if document.get_element_by_id("newOrEditOrderForm").is_some() {
                if is_active_order() {
                    save_to_active_order();
                    is_order_active.set(true);
                } else {
                    is_order_active.set(false);
                }
            }

            match route {
                AppRoutes::Home => html! {<Home/>},
                AppRoutes::OrderForm => html! {<OrderForm/>},
                AppRoutes::OrderProducts => html! {<OrderProducts/>},
                AppRoutes::OrderDonations => html! {<OrderDonations/>},
                AppRoutes::Reports => html! {<Reports/>},
                AppRoutes::Timecards => html! {<Timecards/>},
                AppRoutes::FundraiserCloseout => html! {<CloseoutFundraiser/>},
                AppRoutes::FrConfigEditor => html! {<FrConfigEditor/>},
                AppRoutes::NotFound => html! { <h1>{ "404" }</h1> },
            }
        }
    };

    let on_report_issue = {
        move |_: MouseEvent| {
            log::info!("Bringing up Report Issue Dlg");
            show_report_issue_dlg(true);
        }
    };

    let on_logoff = {
        move |_: MouseEvent| {
            wasm_bindgen_futures::spawn_local(async move {
                log::info!("User has asked to logout");
                // We need to clear local storage so a logon/logoff can force a reload
                clear_local_storage();
                // We need to delete report settings in case admin is on view user can't support
                clear_session_storage(); 
                logout().await;
            });
        }
    };

    {
        let is_loading = is_loading.clone();
        use_effect(move || {
            if *is_loading {
                wasm_bindgen_futures::spawn_local(async move {
                    if is_authenticated().await {
                        match get_active_user_async().await {
                            Ok(user_info) => {
                                // We are authenticated so get initial config stuff before we bring up ui
                                load_config().await;
                                // Preload summary_report data TODO: this is goofy
                                let _ = get_summary_report_data(
                                    &user_info.get_id(),
                                    NUM_TOP_SELLERS_TO_GET,
                                ).await;
                                log::info!("Showing UI");
                                is_loading.set(false);
                            },
                            Err(err) => {
                                log::error!("Get Active User Err: {:#?}", err);
                                gloo::dialogs::alert(&format!("Failed to get User Info: {:#?}", err));
                            }
                        };
                    } else {
                        log::info!("Not authenticated need to get signed in");
                        login().await;
                    }
                });
            }
            || ()
        });
    }

    if *is_loading {
        html! {
            <div id="notReadyView" class="justify-content-center text-center" >
                <h2>{"Loading..."}</h2>
                <span class="loader"></span>
            </div>
        }
    } else {
        let active_user = get_active_user();
        let user_id = active_user.get_id();
        let user_name = active_user.get_name();
        let is_admin = active_user.is_admin();

        html! {
            <BrowserRouter>
                <AppNav
                    userid={user_id.clone()}
                    username={user_name}
                    isadmin={is_admin}
                    onlogoff={on_logoff}
                    isactiveorder={*is_order_active}
                    onreportissue={on_report_issue}/>
                <main class="flex-shrink-0">
                    <Switch<AppRoutes> render={route_switch} />
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

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    log::info!("RelVer: {}", option_env!("GITHUB_REF").unwrap_or("?"));
    yew::Renderer::<App>::new().render();
}
