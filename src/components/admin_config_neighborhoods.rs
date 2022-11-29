use yew::prelude::*;
use web_sys::{
   MouseEvent, Element, HtmlElement, HtmlInputElement,
};
use crate::data_model::*;
use crate::bootstrap;
use std::rc::Rc;
use std::cell::RefCell;
use wasm_bindgen::JsCast;


thread_local! {
    static SELECTED_NEIGHBORHOOD: Rc<RefCell<Option<UseStateHandle<Option<(String, String)>>>>> = Rc::new(RefCell::new(None));
}

/////////////////////////////////////////////////
/////////////////////////////////////////////////
type NeighborhoodDlgAddOrUpdateCb = (String, String);
/////////////////////////////////////////////////
//
#[derive(Properties, PartialEq, Clone, Debug)]
struct NeighborhoodAddEditDlgProps {
    onaddorupdate: Callback<NeighborhoodDlgAddOrUpdateCb>,
}

#[function_component(NeighborhoodAddEditDlg)]
fn neighborhood_add_or_edit_dlg(props: &NeighborhoodAddEditDlgProps) -> Html
{
    let neighborhood = use_state_eq(|| None);
    {
        let neighborhood = neighborhood.clone();
        SELECTED_NEIGHBORHOOD.with(|rc|{
            *rc.borrow_mut() = Some(neighborhood);
        });
    }

    let on_add_update = {
        let onaddorupdate = props.onaddorupdate.clone();
        move |_evt: MouseEvent| {
            let document = gloo::utils::document();
            let neighborhood = document.get_element_by_id("formNeighborhood")
                .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
                .unwrap()
                .value();
            let distpt = document.get_element_by_id("formDistPt")
                .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
                .unwrap()
                .value();
            onaddorupdate.emit((neighborhood, distpt));
        }
    };

    let (is_new, hood, distpt) = (*neighborhood)
        .as_ref()
        .map_or_else(
            ||(true, "".to_string(), "".to_string()),
            |(hood, distpt)| (false, hood.clone(), distpt.clone()));

    // log::info!("Cutoff Date String: {}", &*cutoff_date_str);
    html!{
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
                            <div class="row">
                                <div class="form-floating col-md">
                                    <input class="form-control" type="text" autocomplete="fr-new-neighborhood" id="formNeighborhood"
                                        required=true
                                        readonly={!is_new}
                                        value={hood} />
                                        <label for="formNeighborhood">{"Neighborhood"}</label>
                                </div>
                            </div>
                            <div class="row">
                                <div class="form-floating col-md">
                                    <input class="form-control" type="edit" autocomplete="fr-new-distpt" id="formDistPt"
                                        required=true
                                        value={distpt} />
                                    <label for="formDistPt">{"Distribution Point"}</label>
                                </div>
                            </div>
                        </div>
                    </div>
                    <div class="modal-footer">
                        <button type="button" class="btn btn-secondary" data-bs-dismiss="modal">{"Cancel"}</button>
                        <button type="button" class="btn btn-primary float-end" data-bs-dismiss="modal" onclick={on_add_update}>
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
#[derive(Properties, PartialEq, Clone, Debug)]
struct NeighborhoodLiProps {
    name: String,
    distpt: String,
    onedit: Callback<MouseEvent>,
    ondelete: Callback<MouseEvent>,
}

#[function_component(NeighborhoodLi)]
fn neighborhood_item(props: &NeighborhoodLiProps) -> Html
{

    html! {
        <li class="list-group-item d-flex justify-content-between">
            <div>
                <div class="mb-1">{props.name.clone()}</div>
                <small class="text-muted">{format!("Distribution Point: {}", &props.distpt)}</small>
            </div>
            <div class="float-end">
                <button class="btn btn-outline-danger mx-1 float-end order-del-btn"
                    data-neighborhood={props.name.clone()} onclick={props.ondelete.clone()}>
                    <i class="bi bi-trash" fill="currentColor"></i>
                </button>
                <button class="btn btn-outline-info float-end order-edt-btn"
                    data-neighborhood={props.name.clone()} onclick={props.onedit.clone()}>
                    <i class="bi bi-pencil" fill="currentColor"></i>
                </button>
            </div>
        </li>
    }
}

/////////////////////////////////////////////////
//
fn get_selected_neighborhood(evt: MouseEvent) -> String {
    let btn_elm = evt.target()
        .and_then(|t| t.dyn_into::<Element>().ok())
        .and_then(|t| {
            // log::info!("Node Name: {}", t.node_name());
            if t.node_name() == "I" {
                t.parent_element()
            } else {
                Some(t)
            }
        }).unwrap();
    let elm = btn_elm.dyn_into::<HtmlElement>().ok().unwrap();

    elm.dataset().get("neighborhood").unwrap()
}

#[function_component(NeighborhoodUl)]
pub(crate) fn neighborhood_list() -> Html
{
    use std::collections::BTreeMap;
    let neighborhoods = use_state(|| (*get_neighborhoods())
        .iter()
        .map(|hi| (hi.name.clone(),hi.distribution_point.clone()))
        .collect::<BTreeMap<String, String>>());
    let is_dirty = use_state_eq(|| false);

    let on_add_or_update_dlg_submit = {
        let is_dirty = is_dirty.clone();
        let neighborhoods = neighborhoods.clone();
        move | vals: NeighborhoodDlgAddOrUpdateCb | {
            let  (neighborhood, distpt) = vals.to_owned();
            log::info!("Add/Updating Neighborhood: {}: {}", &neighborhood, &distpt);
            let mut neighborhood_map = (*neighborhoods).clone();
            neighborhood_map.insert(neighborhood, distpt);
            neighborhoods.set(neighborhood_map);
            is_dirty.set(true);
        }
    };

    let on_delete = {
        let neighborhoods = neighborhoods.clone();
        let is_dirty = is_dirty.clone();
        move | evt: MouseEvent | {
            let neighborhood = get_selected_neighborhood(evt);
            let mut neighborhood_map = (*neighborhoods).clone();
            log::info!("Neighborhood ID: {}", &neighborhood);
            neighborhood_map.remove(&neighborhood);
            neighborhoods.set(neighborhood_map);
            is_dirty.set(true);
        }
    };

    let on_add_neighborhood = {
        move | _evt: MouseEvent | {
            // Since we are adding we don't have a selected index
            SELECTED_NEIGHBORHOOD.with(|rc|{
                let selected_neighborhood = rc.borrow().as_ref().unwrap().clone();
                selected_neighborhood.set(None);
            });

            let dlg = bootstrap::get_modal_by_id("neighborhoodAddOrEditDlg").unwrap();
            dlg.toggle();
        }
    };

    let on_edit = {
        let neighborhoods = neighborhoods.clone();
        move | evt: MouseEvent | {
            let neighborhood = get_selected_neighborhood(evt);
            let distpt = (*neighborhoods).get(&neighborhood).unwrap();
            log::info!("Editing Neighborhood: {} distpt:{}", &neighborhood, distpt);

            SELECTED_NEIGHBORHOOD.with(|rc|{
                let selected_neighborhood = rc.borrow().as_ref().unwrap().clone();
                selected_neighborhood.set(Some((neighborhood, distpt.to_string())));
            });
            let dlg = bootstrap::get_modal_by_id("neighborhoodAddOrEditDlg").unwrap();
            dlg.toggle();
        }
    };

    let on_save_neighborhoods = {
        let neighborhoods = neighborhoods.clone();
        let is_dirty = is_dirty.clone();
        move | _evt: MouseEvent | {
            log::info!("Saving Neighborhoods");
            is_dirty.set(false);
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
                            <button class="btn btn-primary" onclick={on_save_neighborhoods}>
                                <span class="spinner-border spinner-border-sm me-1" role="status"
                                aria-hidden="true" id="saveNeighborhoodConfigSpinner" style="display: none;" />
                                {"Save"}
                            </button>
                        }
                    </h5>
                    <ul class="list-group">
                    {
                        neighborhoods.iter().map(|(name, distribution_point)| {
                            html!{<NeighborhoodLi
                                    name={name.clone()}
                                    distpt={distribution_point.clone()}
                                    ondelete={on_delete.clone()}
                                    onedit={on_edit.clone()} />}
                        }).collect::<Html>()
                    }
                    </ul>
                </div>
            </div>
            <NeighborhoodAddEditDlg onaddorupdate={on_add_or_update_dlg_submit}/>
        </div>
    }
}
