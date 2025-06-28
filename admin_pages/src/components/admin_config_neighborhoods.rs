use data_model::*;
use js::bootstrap;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::JsCast;
use web_sys::{Element, HtmlButtonElement, HtmlElement, MouseEvent};
use yew::prelude::*;

thread_local! {
    static SELECTED_NEIGHBORHOOD: Rc<RefCell<Option<UseStateHandle<Neighborhood>>>> = Rc::new(RefCell::new(None));
}

/////////////////////////////////////////////////
/////////////////////////////////////////////////
/////////////////////////////////////////////////
#[derive(Properties, PartialEq, Clone, Debug)]
struct NeighborhoodAddEditDlgProps {
    onaddorupdate: Callback<Neighborhood>,
}

#[function_component(NeighborhoodAddEditDlg)]
fn neighborhood_add_or_edit_dlg(props: &NeighborhoodAddEditDlgProps) -> Html {
    let neighborhood = use_state_eq(|| Neighborhood {
        name: "".to_string(),
        zipcode: None,
        city: None,
        is_visible: false,
        distribution_point: "".to_string(),
    });
    {
        // This adds the use_state handler so it can be accessed externally
        let neighborhood = neighborhood.clone();
        SELECTED_NEIGHBORHOOD.with(|rc| {
            *rc.borrow_mut() = Some(neighborhood);
        });
    }

    let on_form_submission = {
        let onaddorupdate = props.onaddorupdate.clone();

        Callback::from(move |evt: SubmitEvent| {
            evt.prevent_default();
            evt.stop_propagation();

            log::info!("on_form_submission");

            let mut is_valid = true;
            let document = gloo::utils::document();

            let name = match get_html_input_value("frmDlgNeighborhood", &document) {
                Some(name) => {
                    let _ = get_element::<Element>("frmDlgNeighborhood", &document)
                        .class_list()
                        .remove_1("is-invalid");
                    name
                }
                None => {
                    is_valid = false;
                    let _ = get_element::<Element>("frmDlgNeighborhood", &document)
                        .class_list()
                        .add_1("is-invalid");
                    "".to_string()
                }
            };

            let distribution_point = match get_html_input_value("frmDlgHoodDistPt", &document) {
                Some(distribution_point) => {
                    let _ = get_element::<Element>("frmDlgHoodDistPt", &document)
                        .class_list()
                        .remove_1("is-invalid");
                    distribution_point
                }
                None => {
                    is_valid = false;
                    let _ = get_element::<Element>("frmDlgHoodDistPt", &document)
                        .class_list()
                        .add_1("is-invalid");
                    "".to_string()
                }
            };

            let city = match get_html_input_value("frmDlgHoodCity", &document) {
                Some(city) => {
                    let _ = get_element::<Element>("frmDlgHoodCity", &document)
                        .class_list()
                        .remove_1("is-invalid");
                    Some(city)
                }
                None => {
                    if !name.to_lowercase().starts_with("out of area") {
                        is_valid = false;
                        let _ = get_element::<Element>("frmDlgHoodCity", &document)
                            .class_list()
                            .add_1("is-invalid");
                    }
                    None
                }
            };

            let zipcode = match get_html_input_value("frmDlgHoodZip", &document)
                .and_then(|v| v.parse::<u32>().ok())
            {
                Some(zipcode) => {
                    let _ = get_element::<Element>("frmDlgHoodZip", &document)
                        .class_list()
                        .remove_1("is-invalid");
                    Some(zipcode)
                }
                None => {
                    if !name.to_lowercase().starts_with("out of area") {
                        is_valid = false;
                        let _ = get_element::<Element>("frmDlgHoodZip", &document)
                            .class_list()
                            .add_1("is-invalid");
                    }
                    None
                }
            };

            if is_valid {
                let hood = Neighborhood {
                    name,
                    distribution_point,
                    city,
                    zipcode,
                    is_visible: get_html_checked_input_value("frmDlgHoodIsVisible", &document),
                };

                onaddorupdate.emit(hood);
            }
        })
    };

    html! {
        <div class="modal fade" id="neighborhoodAddOrEditDlg"
             tabIndex="-1" role="dialog" aria-labelledby="neighborhoodAddOrEditDlgTitle" aria-hidden="true">
            <div class="modal-dialog modal-dialog-centered" role="document">
                <div class="modal-content">
                    <div class="modal-header">
                        <h5 class="modal-title" id="neighborhoodAddOrEditLongTitle">
                           {"Add/Edit Neighborhood"}
                        </h5>
                    </div>
                    <div class="modal-body">
                        <div class="container-sm">
                            <form class="needs-validation" novalidate=true onsubmit={on_form_submission}>
                                <div class="row mb-1">
                                    <div class="col-md">
                                        <div class="form-check form-switch col-md">
                                            <input class="form-check-input" type="checkbox" id="frmDlgHoodIsVisible"
                                                required=true
                                                checked={neighborhood.is_visible} />
                                            <label class="form-check-label" for="frmDlgHoodIsVisible">{"Is Visible"}</label>
                                        </div>
                                    </div>
                                </div>
                                <div class="row mb-1">
                                    <div class="col-md">
                                        <div class="form-floating col-md">
                                            <input class="form-control" type="text" autocomplete="fr-new-neighborhood" id="frmDlgNeighborhood"
                                                required=true
                                                readonly={!neighborhood.name.is_empty()}
                                                disabled={!neighborhood.name.is_empty()}
                                                value={neighborhood.name.clone()} />
                                                <label for="frmDlgNeighborhood">{"Neighborhood"}</label>
                                            <div class="invalid-feedback">
                                                {"* Neighborhood name is required"}
                                            </div>
                                        </div>
                                    </div>
                                </div>
                                <div class="row mb-1">
                                    <div class="col-md">
                                        <div class="form-floating col-md">
                                            <input class="form-control" type="edit" autocomplete="fr-new-distpt" id="frmDlgHoodDistPt"
                                                required=true
                                                value={neighborhood.distribution_point.clone()} />
                                            <label for="frmDlgHoodDistPt">{"Distribution Point"}</label>
                                            <div class="invalid-feedback">
                                                {"* Distribution point is required"}
                                            </div>
                                        </div>
                                    </div>
                                </div>
                                <div class="row mb-1">
                                    <div class="col-md">
                                        <div class="form-floating col-md">
                                            <input class="form-control" type="edit" autocomplete="fr-new-city" id="frmDlgHoodCity"
                                                required=true
                                                value={neighborhood.city.clone().unwrap_or("".to_string())} />
                                            <label for="frmDlgHoodCity">{"City"}</label>
                                            <div class="invalid-feedback">
                                                {"* City is required for non 'Out of Area' entries."}
                                            </div>
                                        </div>
                                    </div>
                                </div>
                                <div class="row mb-1">
                                    <div class="col-md">
                                        <div class="form-floating col-md">
                                            <input class="form-control" type="number" autocomplete="fr-new-zipcode" id="frmDlgHoodZip"
                                                pattern="[0-9]{5}"
                                                required=true
                                                value={neighborhood.zipcode.map(|v|v.to_string()).unwrap_or("".to_string())} />
                                            <label for="frmDlgHoodZip">{"Zipcode"}</label>
                                            <div class="invalid-feedback">
                                                {"* Zipcode is required for non 'Out of Area' entries."}
                                            </div>
                                        </div>
                                    </div>
                                </div>
                                <div class="row">
                                    <div class="col-md">
                                        <button type="button" class="btn btn-secondary" data-bs-dismiss="modal">{"Cancel"}</button>
                                        <button type="submit" class="btn btn-primary float-end">
                                            {"Submit"}
                                        </button>
                                    </div>
                                </div>
                            </form>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}

/////////////////////////////////////////////////
/////////////////////////////////////////////////
#[derive(Properties, PartialEq, Clone, Debug)]
struct NeighborhoodLiProps {
    hood: Neighborhood,
    onedit: Callback<MouseEvent>,
}

#[function_component(NeighborhoodLi)]
fn neighborhood_item(props: &NeighborhoodLiProps) -> Html {
    let mut liclass = "list-group-item d-flex justify-content-between".to_string();
    if !props.hood.is_visible {
        liclass = format!("{liclass} list-group-item-dark");
    }
    html! {
        <li class={liclass}>
            <div class="container">
                <div class="row mb-1">{props.hood.name.clone()}</div>
                <small class="text-muted row">{format!("Distribution Point: {}", &props.hood.distribution_point)}</small>
                if props.hood.city.is_some() {
                    <small class="text-muted row">{format!("City: {}", &props.hood.city.as_ref().unwrap())}</small>
                }
                if props.hood.zipcode.is_some() {
                    <small class="text-muted row">{format!("Zip: {}", &props.hood.zipcode.as_ref().unwrap())}</small>
                }
                <small class="text-muted row">{format!("isVisible: {}", &props.hood.is_visible)}</small>
            </div>
            <div class="float-end">
                <button class="btn btn-outline-info float-end order-edt-btn"
                    data-neighborhood={props.hood.name.clone()} onclick={props.onedit.clone()}>
                    <i class="bi bi-pencil" fill="currentColor"></i>
                </button>
            </div>
        </li>
    }
}

/////////////////////////////////////////////////
fn get_selected_neighborhood(evt: MouseEvent) -> String {
    let btn_elm = evt
        .target()
        .and_then(|t| t.dyn_into::<Element>().ok())
        .and_then(|t| {
            // log::info!("Node Name: {}", t.node_name());
            if t.node_name() == "I" {
                t.parent_element()
            } else {
                Some(t)
            }
        })
        .unwrap();
    let elm = btn_elm.dyn_into::<HtmlElement>().ok().unwrap();

    elm.dataset().get("neighborhood").unwrap()
}

/////////////////////////////////////////////////
fn disable_save_button(document: &web_sys::Document, value: bool) {
    if let Some(btn) = document
        .get_element_by_id("btnSaveUpdatedHoods")
        .and_then(|t| t.dyn_into::<HtmlButtonElement>().ok())
    {
        btn.set_disabled(value);
        let spinner_display = if value { "inline-block" } else { "none" };
        let _ = document
            .get_element_by_id("saveNeighborhoodConfigSpinner")
            .and_then(|t| t.dyn_into::<HtmlElement>().ok())
            .unwrap()
            .style()
            .set_property("display", spinner_display);
    }
}

#[function_component(NeighborhoodUl)]
pub(crate) fn neighborhood_list() -> Html {
    use std::collections::BTreeMap;
    // Map neighborhood names to neighborhood and add ability to mark dirty
    let neighborhoods = use_state(|| {
        (*get_neighborhoods())
            .iter()
            .map(|hi| (hi.name.clone(), (false, hi.clone())))
            .collect::<BTreeMap<String, (bool, Neighborhood)>>()
    });
    let is_dirty = use_state_eq(|| false);

    let on_add_or_update_dlg_submit = {
        let is_dirty = is_dirty.clone();
        let neighborhoods = neighborhoods.clone();

        move |hood: Neighborhood| {
            log::info!("Add/Updating Neighborhood: {:#?}", &hood);
            log::info!("Closing Dlg");

            bootstrap::modal_op("neighborhoodAddOrEditDlg", "hide");

            let mut neighborhood_map = (*neighborhoods).clone();
            neighborhood_map.insert(hood.name.clone(), (true, hood));
            neighborhoods.set(neighborhood_map);
            is_dirty.set(true);
        }
    };

    let on_add_neighborhood = {
        move |_evt: MouseEvent| {
            // Since we are adding we don't have a selected index
            SELECTED_NEIGHBORHOOD.with(|rc| {
                let selected_neighborhood = rc.borrow().as_ref().unwrap().clone();
                selected_neighborhood.set(Neighborhood {
                    name: "".to_string(),
                    zipcode: None,
                    city: None,
                    is_visible: false,
                    distribution_point: "".to_string(),
                });
            });

            bootstrap::modal_op("neighborhoodAddOrEditDlg", "show");
        }
    };

    let on_edit = {
        let neighborhoods = neighborhoods.clone();
        move |evt: MouseEvent| {
            let hood_name_str = get_selected_neighborhood(evt);
            let (_, hood) = (*neighborhoods).get(&hood_name_str).unwrap();
            log::info!("Editing Neighborhood: {:#?}", &hood);

            SELECTED_NEIGHBORHOOD.with(|rc| {
                let selected_neighborhood = rc.borrow().as_ref().unwrap().clone();
                selected_neighborhood.set(hood.clone());
            });

            bootstrap::modal_op("neighborhoodAddOrEditDlg", "show");
        }
    };

    let on_save_neighborhoods = {
        let neighborhoods = neighborhoods.clone();
        let is_dirty = is_dirty.clone();
        move |_evt: MouseEvent| {
            let document = gloo::utils::document();
            disable_save_button(&document, true);
            let updated_hoods = neighborhoods
                .values()
                .filter_map(|(is_dirty, hood)| if *is_dirty { Some(hood.clone()) } else { None })
                .collect();

            let is_dirty = is_dirty.clone();
            wasm_bindgen_futures::spawn_local(async move {
                log::info!("Saving Neighborhoods: {updated_hoods:#?}");
                if let Err(err) = update_neighborhoods(updated_hoods).await {
                    gloo::dialogs::alert(&format!("Failed updating neighborhoods:\n{err:#?}"));
                }
                disable_save_button(&document, false);
                is_dirty.set(false);
            });
        }
    };

    html! {
        <div>
            <div class="card">
                <div class="card-body">
                    <h5 class="card-title">
                        {"Neighborhoods"}
                        <button class="btn btn-outline-info float-end order-edt-btn" onclick={on_add_neighborhood}>
                            <i class="bi bi-plus-square" fill="currentColor"></i>
                        </button>
                        if *is_dirty {
                            <button class="btn btn-primary" onclick={on_save_neighborhoods} id="btnSaveUpdatedHoods">
                                <span class="spinner-border spinner-border-sm me-1" role="status"
                                aria-hidden="true" id="saveNeighborhoodConfigSpinner" style="display: none;" />
                                {"Save"}
                            </button>
                        }
                    </h5>
                    <ul class="list-group">
                    {
                        neighborhoods.values().into_iter().map(|(_, hood)| {
                            html!{<NeighborhoodLi hood={hood.clone()} onedit={on_edit.clone()} />}
                        }).collect::<Html>()
                    }
                    </ul>
                </div>
            </div>
            <NeighborhoodAddEditDlg onaddorupdate={on_add_or_update_dlg_submit}/>
        </div>
    }
}
