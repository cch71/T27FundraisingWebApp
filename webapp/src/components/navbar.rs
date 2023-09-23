use data_model::AppRoutes;
use web_sys::MouseEvent;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Properties, PartialEq)]
pub(crate) struct AppNavProps {
    pub(crate) userid: String,
    pub(crate) username: String,
    pub(crate) isadmin: bool,
    pub(crate) onlogoff: Callback<MouseEvent>,
    pub(crate) onreportissue: Callback<MouseEvent>,
    pub(crate) isactiveorder: bool,
}

#[function_component(AppNav)]
pub(crate) fn app_nav(props: &AppNavProps) -> Html {
    let _ = use_navigator().unwrap(); // This forces re-render on path changes
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
                    if props.isactiveorder {
                        <li class="nav-item">
                            <Link<AppRoutes> classes="nav-link" to={AppRoutes::OrderForm} >
                                {"Order"}
                            </Link<AppRoutes>>
                        </li>
                    }
                    <li class="nav-item">
                        // <Link<AppRoutes> classes="nav-link" to={AppRoutes::Reports} >
                        //     {"Reports"}
                        // </Link<AppRoutes>>
                    </li>
                </ul>
                <span class="navbar-nav nav-item dropdown">
                    <a class="nav-link dropdown-toggle" href="#" id="navbarDropdown"
                       data-bs-toggle="dropdown" aria-expanded="false" role="button">
                        {userlabel}
                    </a>
                    <div class="dropdown-menu dropdown-menu-end" aria-labelledby="navbarDropdown">
                        if props.isadmin {
                            <Link<AppRoutes> classes="dropdown-item" to={AppRoutes::Timecards} >
                                {"Timecards"}
                            </Link<AppRoutes>>
                            // <Link<AppRoutes> classes="dropdown-item" to={AppRoutes::FundraiserCloseout} >
                            //     {"Closeout Fundraiser"}
                            // </Link<AppRoutes>>
                            // <Link<AppRoutes> classes="dropdown-item" to={AppRoutes::FrConfig} >
                            //     {"Configure Fundraiser"}
                            // </Link<AppRoutes>>
                        }
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
