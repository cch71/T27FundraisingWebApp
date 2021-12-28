use yew::prelude::*;
use yew_router::prelude::*;

mod order_utils;
use order_utils::*;

mod currency_utils;

mod pages;
use pages::{
    home::Home,
    reports::Reports,
    order_form::OrderForm,
    order_donations::OrderDonations,
    order_products::OrderProducts,
};

//AWS API URL
//invokeUrl: 'https://j0azby8rm6.execute-api.us-east-1.amazonaws.com/prod'

/////////////////////////////////////////////////
// Auth Comp Stuff
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
#[wasm_bindgen(module = "/src/js/auth.js")]
extern "C" {
    #[wasm_bindgen(catch)]
    async fn loginUser() -> Result<(), JsValue>;

    #[wasm_bindgen(catch)]
    async fn logoutUser() -> Result<(), JsValue>;

    #[wasm_bindgen(catch)]
    async fn isAuthenticated() -> Result<JsValue, JsValue>;

    #[wasm_bindgen(catch)]
    async fn getUserInfo() -> Result<JsValue, JsValue>;
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct UserInfo {
    pub email: String,
    pub token: String,
}

impl UserInfo {
    pub fn user_id(&self)->String {
        // TODO: Do this at creation time
        let v: Vec<&str> = self.email.split('@').collect();
        v[0].to_owned()
    }
}

pub async fn get_user_info() -> Option<UserInfo> {
    match getUserInfo().await {
        Ok(user_info) => {
            log::info!("User Info: {:#?}", user_info);
            let user_info: UserInfo = serde_wasm_bindgen::from_value(user_info).unwrap();
            Some(user_info)
        },
        Err(err) => {
            log::error!("User Info Err: {:#?}", err);
            gloo_dialogs::alert(&format!("Failed to get User Info: {:#?}", err));
            None
        },
    }
}

pub async fn is_authenticated() -> bool {
    match isAuthenticated().await {
        Ok(is_auth) => {
            log::info!("Is Authenticated: {:#?}", &is_auth);
            let is_auth: bool = serde_wasm_bindgen::from_value(is_auth).unwrap();
            return is_auth;
        },
        Err(err) => log::error!("User Info Err: {:#?}", err),
    };
    false
}

pub async fn login() {
    if let Err(err) = loginUser().await {
        log::error!("Authentication Err: {:#?}", err);
    } else {
        log::info!("Authenticated");
    }
}

pub async fn logout() {
    if let Err(err) = logoutUser().await {
        log::error!("Error logging out Err: {:#?}", err);
    } else {
        log::info!("Logged out");
    }
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
// Route Logic
#[derive(Clone, Routable, PartialEq)]
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

fn switch(routes: &AppRoutes) -> Html {
    match routes {
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
    Authenticated(UserInfo),
    UpdateRoute,
    Logout,
}
type Msg = AppMsg;

struct Model {
    user_info: Option<UserInfo>,
    is_loading: bool,
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        ctx.link().send_future(async move {
            if is_authenticated().await {
                log::info!("Authenticated");
                match get_user_info().await {
                    Some(user_info)=> Msg::Authenticated(user_info),
                    None=>Msg::UpdateRoute,
                }
            } else {
                Msg::NotAuthenticated
            }
        });

        Self {
            user_info: None,
            is_loading: true,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Authenticated(user_info) => {
                self.user_info = Some(user_info);
                self.is_loading = false;
                //let history = ctx.link().history().unwrap();
                //history.replace(AppRoutes::Home);
                true
            },
            Msg::NotAuthenticated => {
                log::info!("Not authenticated need to get signed in");
                ctx.link().send_future(async move {
                    login().await;
                    Msg::UpdateRoute
                });
                false
            },
            Msg::Logout=>{
                log::info!("User has asked to logout");
                ctx.link().send_future(async move {
                    logout().await;
                    Msg::UpdateRoute
                });
                false
            },
            Msg::UpdateRoute=>false,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        // let history = ctx.link().history().unwrap();
        // log::info!("!!!! Location: {:#?}", &history.location());

        if self.is_loading {
            html! {
                <div id="notReadyView" class="col-xs-1 d-flex justify-content-center" >
                    <div class="spinner-border" role="status">
                        <span class="visually-hidden">{ "Loading..."}</span>
                    </div>
                </div>
            }
        } else {
            // let user_info_ctx = use_state(|| self.user_info.clone());
            // let is_not_authenticated = self.user_id.is_none();
            // let on_auth_complete = ctx.link().callback(|user_info: UserInfo| Msg::Authenticated(user_info));

            html! {
                <BrowserRouter>
                    { self.app_nav(ctx) }
                    <main class="flex-shrink-0">
                        <Switch<AppRoutes> render={Switch::render(switch)} />
                    </main>
                    <footer class="footer mt-auto py-3 bg-light">
                        <AddNewOrderButton userid={self.user_info.as_ref().unwrap().user_id()}/>
                    </footer>
                </BrowserRouter>
            }
        }
    }
}

impl Model {
 fn app_nav(&self, ctx: &Context<Self>) -> Html {

    let on_logout_click = ctx.link().callback(|_| Msg::Logout);

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
                            { "Home" }
                        </Link<AppRoutes>>
                    </li>
                    <li class="nav-item">
                        <Link<AppRoutes> classes="nav-link" to={AppRoutes::OrderForm} >
                            { "Order" }
                        </Link<AppRoutes>>
                    </li>
                    <li class="nav-item">
                        <Link<AppRoutes> classes="nav-link" to={AppRoutes::Reports} >
                            { "Reports" }
                        </Link<AppRoutes>>
                    </li>
                </ul>
                <span class="navbar-nav nav-item dropdown">
                    <a class="nav-link dropdown-toggle" href="#" id="navbarDropdown"
                       data-bs-toggle="dropdown" aria-expanded="false" role="button">
                        { self.user_info.as_ref().map_or_else(||"".to_string(), |v|v.user_id()) }
                    </a>
                    <div class="dropdown-menu dropdown-menu-end" aria-labelledby="navbarDropdown">
                        <a class="dropdown-item" href="#" data-bs-toggle="modal">
                          { "Report Issue" }
                        </a>
                        <a class="dropdown-item" onclick={on_logout_click} href="#" data-bs-toggle="modal">
                          { "Logout" }
                        </a>
                    </div>
                </span>
            </div>
        </nav>
    }
}
}


fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::start_app::<Model>();
}
