use js::nav::navigate_to;
use web_sys::MouseEvent;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub(crate) struct AppNavProps {
    pub(crate) userid: String,
    pub(crate) username: String,
    pub(crate) isadmin: bool,
    pub(crate) onlogoff: Callback<MouseEvent>,
    pub(crate) onreportissue: Callback<MouseEvent>,
    pub(crate) isactiveorder: bool,
}

/// Navigation links go through `navigate_to` rather than yew-router `Link`s:
/// it wakes both the shell router and the router inside whichever page
/// module is currently mounted (a `Link` only notifies the shell's).
fn nav_click(path: &'static str) -> Callback<MouseEvent> {
    Callback::from(move |evt: MouseEvent| {
        evt.prevent_default();
        navigate_to(path);
    })
}

#[component(AppNav)]
pub(crate) fn app_nav(props: &AppNavProps) -> Html {
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
                        <a class="nav-link" href="/" onclick={nav_click("/")}>
                            {"Home"}
                        </a>
                    </li>
                    if props.isactiveorder {
                        <li class="nav-item">
                            <a class="nav-link" href="/order" onclick={nav_click("/order")}>
                                {"Order"}
                            </a>
                        </li>
                    }
                    <li class="nav-item">
                        <a class="nav-link" href="/reports" onclick={nav_click("/reports")}>
                            {"Reports"}
                        </a>
                    </li>
                </ul>
                <span class="navbar-nav nav-item dropdown">
                    <a class="nav-link dropdown-toggle" href="#" id="navbarDropdown"
                       data-bs-toggle="dropdown" aria-expanded="false" role="button">
                        {userlabel}
                    </a>
                    <div class="dropdown-menu dropdown-menu-end" aria-labelledby="navbarDropdown">
                        if props.isadmin {
                            <a class="dropdown-item" href="/timecards" onclick={nav_click("/timecards")}>
                                {"Timecards"}
                            </a>
                            <a class="dropdown-item" href="/frcloseout" onclick={nav_click("/frcloseout")}>
                                {"Closeout Fundraiser"}
                            </a>
                            <a class="dropdown-item" href="/frcconfig" onclick={nav_click("/frcconfig")}>
                                {"Configure Fundraiser"}
                            </a>
                        }
                        <a class="dropdown-item" onclick={props.onreportissue.clone()} href="#" data-bs-toggle="modal">
                            {"Report Issue"}
                        </a>
                        <a
                            class="dropdown-item"
                            href="https://cch71.github.io/T27FundraisingWebAppManual/docs/overview/"
                            target="_blank">
                                {"Help Manual"}
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
