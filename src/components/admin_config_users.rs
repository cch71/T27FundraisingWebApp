use yew::prelude::*;
use gloo::file::callbacks::FileReader;
use gloo::file::File;
use web_sys::{
   MouseEvent, Element, HtmlElement, FileList, HtmlInputElement, InputEvent,
};
use crate::data_model::*;
use crate::bootstrap;
use std::rc::Rc;
use std::cell::RefCell;
use wasm_bindgen::JsCast;

thread_local! {
    static SELECTED_USER: Rc<RefCell<Option<UseStateHandle<(String, String, String)>>>> = Rc::new(RefCell::new(None));
}

/////////////////////////////////////////////////
/////////////////////////////////////////////////

/////////////////////////////////////////////////
//
#[derive(Properties, PartialEq, Clone, Debug)]
struct AddUsersDlgProps {
    onadd: Callback<UserMapType>,
}

struct FileDetails {
    name: String,
    file_type: String,
    data: Vec<u8>,
}

#[function_component(AddUsersDlg)]
fn add_users_dlg(props: &AddUsersDlgProps) -> Html
{
    let on_submit = {
        // let onaddorupdate = props.onaddorupdate.clone();
        // let selected_user = selected_user.clone();
        move |_evt: MouseEvent| {
            // let (uid, name, _) = (*selected_user).as_ref().unwrap();
            // let document = gloo::utils::document();
            // let group = document.get_element_by_id("formEditUserGroup")
            //     .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
            //     .unwrap()
            //     .value();
            // onadd.emit((uid, name, group));
        }
    };
    let on_file_input_change = {
        move | evt: Event | {
            log::info!("On Change Triggered");
            let input: HtmlInputElement = evt.target_unchecked_into();
            let files: Option<FileList> = input.files();
            // let mut downloaded_files = Vec::default();
            let mut results = Vec::new();

            if let Some(files) = files {
                let files = js_sys::try_iter(&files)
                    .unwrap()
                    .unwrap()
                    .map(|v| web_sys::File::from(v.unwrap()))
                    .map(File::from);
                results.extend(files);
                log::info!("Found some files: {:#?}", &results);
            }

            for file in results.into_iter() {
                let file_name = file.name();
                let file_type = file.raw_mime_type();
                log::info!("Loading: {} type: {}", file_name, file_type);
                wasm_bindgen_futures::spawn_local(async move {
                    let data = gloo::file::futures::read_as_bytes(&file).await.unwrap();
                    let mut rdr = csv::Reader::from_reader(&data[..]);
                    for result in rdr.records() {
                        let record = result.unwrap();
                        log::info!("{:?}", record);
                    }
                    // log::info!("Raw File: {}", fc);
                    // let fd = FileDetails {
                    //     data: res.expect("failed to read file"),
                    //     file_type: file_type,
                    //     name: file_name.clone(),
                    // };
                    //downloaded_files.push(fd);
                });
            }
        }
    };

    html!{
        <div class="modal fade" id="addUsersDlg"
             tabIndex="-1" role="dialog" aria-labelledby="addUsersDlgTitle" aria-hidden="true">
            <div class="modal-dialog modal-dialog-centered" role="document">
                <div class="modal-content">
                    <div class="modal-header">
                        <h5 class="modal-title" id="addUsersLongTitle">
                           {"Add Users"}
                        </h5>
                    </div>
                    <div class="modal-body">
                        <div class="container-sm">
                        <input
                            id="file-upload"
                            type="file"
                            accept=".csv"
                            multiple={false}
                            onchange={on_file_input_change}
                        />
                        </div>
                    </div>
                    <div class="modal-footer">
                        <button type="button" class="btn btn-secondary" data-bs-dismiss="modal">{"Cancel"}</button>
                        <button type="button" class="btn btn-primary float-end" data-bs-dismiss="modal" onclick={on_submit}>
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
type EditUserDlgCb = (String, String, String);
/////////////////////////////////////////////////
//
#[derive(Properties, PartialEq, Clone, Debug)]
struct EditUserDlgProps {
    onupdate: Callback<EditUserDlgCb>,
}

#[function_component(EditUserDlg)]
fn edit_user_dlg(props: &EditUserDlgProps) -> Html
{
    let selected_user = use_state_eq(|| ("".to_string(), "".to_string(), "".to_string()));
    {
        let selected_user = selected_user.clone();
        SELECTED_USER.with(|rc|{
            *rc.borrow_mut() = Some(selected_user);
        });
    }

    let on_update = {
        let onupdate = props.onupdate.clone();
        let selected_user = selected_user.clone();
        move |_evt: MouseEvent| {
            let (uid, name, _) = &*selected_user;
            let document = gloo::utils::document();
            let group = document.get_element_by_id("formEditUserGroup")
                .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
                .unwrap()
                .value();
            onupdate.emit((uid.clone(), name.clone(), group));
        }
    };

    let (uid, name, group) = &*selected_user;

    html!{
        <div class="modal fade" id="editUserDlg"
             tabIndex="-1" role="dialog" aria-labelledby="editUserDlgTitle" aria-hidden="true">
            <div class="modal-dialog modal-dialog-centered" role="document">
                <div class="modal-content">
                    <div class="modal-header">
                        <h5 class="modal-title" id="editUserLongTitle">
                           {"Edit User's Group"}
                        </h5>
                    </div>
                    <div class="modal-body">
                        <div class="container-sm">
                            <div class="row">
                                <div class="form-floating col-md">
                                    <input class="form-control" type="text" id="formEditUserUid"
                                        readonly=true
                                        value={uid.clone()} />
                                        <label for="formEditUserUid">{"UserID"}</label>
                                </div>
                            </div>
                            <div class="row">
                                <div class="form-floating col-md">
                                    <input class="form-control" type="text" id="formEditUserName"
                                        readonly=true
                                        value={name.clone()} />
                                        <label for="formEditUserName">{"Name"}</label>
                                </div>
                            </div>
                            <div class="row">
                                <div class="form-floating col-md">
                                    <input class="form-control" type="text" autocomplete="fr-new-distpt" id="formEditUserGroup"
                                        required=true
                                        value={group.clone()} />
                                    <label for="formEditUserGroup">{"Group"}</label>
                                </div>
                            </div>
                        </div>
                    </div>
                    <div class="modal-footer">
                        <button type="button" class="btn btn-secondary" data-bs-dismiss="modal">{"Cancel"}</button>
                        <button type="button" class="btn btn-primary float-end" data-bs-dismiss="modal" onclick={on_update}>
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
struct UserLiProps {
    uid: String,
    name: String,
    group: String,
    onedit: Callback<MouseEvent>,
}

#[function_component(UserLi)]
fn user_item(props: &UserLiProps) -> Html
{

    html! {
        <li class="list-group-item d-flex justify-content-between">
            <div>
                <div class="mb-1">{props.uid.clone()}</div>
                <small class="text-muted">{format!("Name: {}", &props.name)}</small>
                <small class="text-muted mx-2">{format!("Group: {}", &props.group)}</small>
            </div>
            <div class="float-end">
                <button class="btn btn-outline-info float-end order-edt-btn"
                    data-uid={props.uid.clone()} onclick={props.onedit.clone()}>
                    <i class="bi bi-pencil" fill="currentColor"></i>
                </button>
            </div>
        </li>
    }
}

/////////////////////////////////////////////////
//
fn get_selected_user(evt: MouseEvent) -> String {
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

    elm.dataset().get("uid").unwrap()
}

#[function_component(UsersUl)]
pub(crate) fn user_list() -> Html
{
    let users = use_state(|| (*get_users()).clone());
    let is_dirty = use_state_eq(|| false);

    let on_add_users_dlg_submit = {
        let is_dirty = is_dirty.clone();
        let users = users.clone();
        move | new_users: UserMapType | {
            log::info!("Adding Users...");
            // let mut users_map = (*users).clone();
            // users_map.insert(uid, UserInfo{
            //     name: name,
            //     group: group,
            // });
            // users.set(users_map);
            is_dirty.set(true);
        }
    };

    let on_edit_user_group_dlg_submit = {
        let is_dirty = is_dirty.clone();
        let users = users.clone();
        move | vals: EditUserDlgCb | {
            let  (uid, name, group) = vals.to_owned();
            log::info!("Updating User: {}, \"{}\", \"{}\"", &uid, &name, &group);
            let mut users_map = (*users).clone();
            users_map.insert(uid, UserInfo{
                name: name,
                group: group,
            });
            users.set(users_map);
            is_dirty.set(true);
        }
    };

    let on_add_users = {
        move | _evt: MouseEvent | {
            let dlg = bootstrap::get_modal_by_id("addUsersDlg").unwrap();
            dlg.toggle();
        }
    };

    let on_edit_users_group = {
        let users = users.clone();
        move | evt: MouseEvent | {
            let uid = get_selected_user(evt);
            let user_info = (*users).get(&uid).unwrap();
            log::info!("Editing User: {} {} {}", &uid, &user_info.name, &user_info.group);

            SELECTED_USER.with(|rc|{
                let selected_user = rc.borrow().as_ref().unwrap().clone();
                selected_user.set((uid.clone(), user_info.name.clone(), user_info.group.clone()));
            });
            let dlg = bootstrap::get_modal_by_id("editUserDlg").unwrap();
            dlg.toggle();
        }
    };

    let on_clear_users = {
        let users = users.clone();
        let is_dirty = is_dirty.clone();
        move | _evt: MouseEvent | {
            log::info!("Clearing Users...");
            users.set(UserMapType::new());
            is_dirty.set(true);
        }
    };

    let on_save_users = {
        let users = users.clone();
        let is_dirty = is_dirty.clone();
        move | _evt: MouseEvent | {
            log::info!("Saving Users");
            is_dirty.set(false);
        }
    };

    html! {
        <div>
            <div class="card">
                <div class="card-body">
                    <h5 class="card-title">
                        {"Users"}
                        <button class="btn btn-outline-info float-end order-edt-btn" onclick={on_add_users}>
                            <i class="bi bi-plus-square" fill="currentColor"></i>
                        </button>
                        if *is_dirty {
                            <button class="btn btn-primary" onclick={on_save_users}>
                                <span class="spinner-border spinner-border-sm me-1" role="status"
                                aria-hidden="true" id="saveUsersConfigSpinner" style="display: none;" />
                                {"Save"}
                            </button>
                        }
                    </h5>
                    <ul class="list-group">
                    {
                        users.iter().map(|(id, user_info)| {
                            html!{<UserLi
                                    uid={id.clone()}
                                    name={user_info.name.clone()}
                                    group={user_info.group.clone()}
                                    onedit={on_edit_users_group.clone()} />}
                        }).collect::<Html>()
                    }
                    </ul>
                    <button class="btn btn-primary" onclick={on_clear_users}>
                        {"Clear Users"}
                    </button>
                </div>
            </div>
            <EditUserDlg onupdate={on_edit_user_group_dlg_submit}/>
            <AddUsersDlg onadd={on_add_users_dlg_submit}/>
        </div>
    }
}
