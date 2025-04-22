#[cfg(target_os = "linux")]
use calamine::{Ods, RangeDeserializerBuilder, Reader, Xlsx, open_workbook_from_rs};
#[cfg(target_os = "linux")]
use std::io::Cursor;

use data_model::*;
use gloo::file::File;
use js::bootstrap;
use serde::Deserialize;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;
use wasm_bindgen::JsCast;
use web_sys::{Element, FileList, HtmlButtonElement, HtmlElement, HtmlInputElement, MouseEvent};
use yew::prelude::*;

#[derive(PartialEq, Clone, Default, Debug)]
struct SelectedUserType {
    uid: String,
    name: String,
    group: String,
}

thread_local! {
    static SELECTED_USER: Rc<RefCell<Option<UseStateHandle<SelectedUserType>>>> = Rc::new(RefCell::new(None));
}

/////////////////////////////////////////////////
/////////////////////////////////////////////////

/////////////////////////////////////////////////
#[derive(Properties, PartialEq, Clone, Debug)]
struct UploadUsersDlgProps {
    onadd: Callback<Vec<UserAdminConfig>>,
    knownusers: BTreeMap<String, UserAdminConfig>,
}

#[derive(Debug, Deserialize, PartialEq)]
struct UserFileRec {
    last_name: String,
    first_name: String,
    group: String,
    uid: Option<String>,
}

/////////////////////////////////////////////////
fn get_or_gen_imported_user_id(record: &UserFileRec) -> String {
    match record.uid.as_ref() {
        Some(id) => id.to_ascii_lowercase(),
        None =>
        // Generate the new user id
        {
            record
                .uid
                .clone()
                .unwrap_or_else(|| {
                    format!(
                        "{}{}",
                        record.first_name.trim().chars().next().unwrap(),
                        record.last_name
                    )
                })
                .chars()
                .filter(|c| !c.is_whitespace())
                .collect::<String>()
                .to_ascii_lowercase()
        }
    }
}

/////////////////////////////////////////////////
fn process_user_file_rec(
    record: UserFileRec,
    potential_new_users: &mut BTreeMap<String, UserFileRec>,
) {
    let mut new_id: String = get_or_gen_imported_user_id(&record);
    log::info!("Rec: {:?} -> Id: {}", record, new_id);

    // Make sure there aren't dups in the uploaded list and create a unique id if it isn't
    if let Some(found_user) = potential_new_users.get(&new_id) {
        if found_user.first_name == record.first_name.trim()
            && found_user.last_name == record.last_name.trim()
        {
            //Duplicate in list
            return;
        } else {
            let mut idx = 1;
            loop {
                let new_id_candidate = format!("{}{}", new_id, idx);
                if potential_new_users.contains_key(&new_id_candidate) {
                    idx += 1;
                    continue;
                }
                new_id = new_id_candidate;
                break;
            }
        }
    }
    potential_new_users.insert(new_id, record);
}

/////////////////////////////////////////////////
fn process_csv_records(
    data: Vec<u8>,
    potential_new_users: &mut BTreeMap<String, UserFileRec>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut rdr = csv::Reader::from_reader(&data[..]);
    for result in rdr.deserialize() {
        let record: UserFileRec = result?;
        process_user_file_rec(record, potential_new_users);
    }
    Ok(())
}

#[cfg(target_os = "linux")]
/////////////////////////////////////////////////
fn process_spreadsheet_records<T>(
    mut wb: T,
    potential_new_users: &mut BTreeMap<String, UserFileRec>,
) -> Result<(), Box<dyn std::error::Error>>
where
    T: Reader<Cursor<Vec<u8>>>,
{
    let range = wb
        .worksheet_range_at(0)
        .unwrap()
        .map_err(|_| calamine::Error::Msg("Cannot find Sheet1"))?;

    let iter_records = RangeDeserializerBuilder::new()
        .has_headers(true)
        .from_range(&range)?;

    for result in iter_records {
        let record: UserFileRec = result?;
        process_user_file_rec(record, potential_new_users);
    }

    Ok(())
}

/////////////////////////////////////////////////
#[cfg(target_os = "linux")]
fn process_xlsx_records(
    data: Vec<u8>,
    potential_new_users: &mut BTreeMap<String, UserFileRec>,
) -> Result<(), Box<dyn std::error::Error>> {
    let c = Cursor::new(data);
    let wb: Xlsx<_> = open_workbook_from_rs(c)?;

    process_spreadsheet_records(wb, potential_new_users)
}

/////////////////////////////////////////////////
#[cfg(target_os = "linux")]
fn process_ods_records(
    data: Vec<u8>,
    potential_new_users: &mut BTreeMap<String, UserFileRec>,
) -> Result<(), Box<dyn std::error::Error>> {
    let c = Cursor::new(data);
    let wb: Ods<_> = open_workbook_from_rs(c)?;

    process_spreadsheet_records(wb, potential_new_users)
}

/////////////////////////////////////////////////
fn process_uploaded_file(
    filename: String,
    mimetype: String,
    data: Vec<u8>,
    potential_new_users: &mut BTreeMap<String, UserFileRec>,
) -> Result<(), Box<dyn std::error::Error>> {
    // 2ndBatch.xlsx type: application/vnd.openxmlformats-officedocument.spreadsheetml.sheet
    // 2ndBatch.ods type: application/vnd.oasis.opendocument.spreadsheet
    
    match true {
        _ if filename.ends_with(".csv") || mimetype.eq("text/csv") => {
            process_csv_records(data, potential_new_users)
        },
        #[cfg(target_os = "linux")]
        _ if filename.to_ascii_lowercase().ends_with(".xlsx")  || mimetype.eq("application/vnd.openxmlformats-officedocument.spreadsheetml.sheet") => {
            process_xlsx_records(data, potential_new_users)
        }
        #[cfg(target_os = "linux")]
        _ if filename.to_ascii_lowercase().ends_with(".ods") || mimetype.eq("application/vnd.oasis.opendocument.spreadsheet") => {
            process_ods_records(data, potential_new_users)
        }
        _ => panic!()
        
    }
}

#[function_component(UploadUsersDlg)]
fn upload_users_dlg(props: &UploadUsersDlgProps) -> Html {
    let users = use_state_eq(Vec::<UserAdminConfig>::new);
    let dup_users = use_state_eq(Vec::<UserFileRec>::new);
    let is_working = use_state_eq(|| false);

    let on_cancel = {
        move |_evt: MouseEvent| {
            let document = gloo::utils::document();
            document
                .get_element_by_id("fileupload")
                .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
                .unwrap()
                .set_value("");
        }
    };

    let on_submit = {
        let onadd = props.onadd.clone();
        let users = users.clone();
        let dup_users = dup_users.clone();
        move |_evt: MouseEvent| {
            onadd.emit((*users).clone());
            users.set(Vec::<UserAdminConfig>::new());
            dup_users.set(Vec::<UserFileRec>::new());
            let document = gloo::utils::document();
            document
                .get_element_by_id("fileupload")
                .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
                .unwrap()
                .set_value("");
        }
    };

    let on_file_input_change = {
        let is_working = is_working.clone();
        let users = users.clone();
        let dup_users = dup_users.clone();
        let found_users = props.knownusers.clone();
        move |evt: Event| {
            log::info!("On Change Triggered");
            is_working.set(true);
            users.set(Vec::<UserAdminConfig>::new());
            dup_users.set(Vec::<UserFileRec>::new());
            let input: HtmlInputElement = evt.target_unchecked_into();
            let files: Option<FileList> = input.files();
            let mut results = Vec::new();

            match files {
                Some(files) => {
                    let files = js_sys::try_iter(&files)
                        .unwrap()
                        .unwrap()
                        .map(|v| web_sys::File::from(v.unwrap()))
                        .map(File::from);
                    results.extend(files);
                    log::info!("Found some files: {:#?}", &results);
                }
                _ => {
                    log::info!("No files so returning");
                    return;
                }
            }

            let is_working = is_working.clone();
            let users = users.clone();
            let dup_users = dup_users.clone();
            let found_users = found_users.clone();
            wasm_bindgen_futures::spawn_local(async move {
                // First read in a list of all the potential new users
                let mut potential_new_users: BTreeMap<String, UserFileRec> = BTreeMap::new();
                for file in results.into_iter() {
                    let file_name = file.name();
                    let file_type = file.raw_mime_type();
                    log::info!("Loading: {} type: {}", file_name, file_type);
                    let data = match gloo::file::futures::read_as_bytes(&file).await {
                        Ok(raw_data) => raw_data,
                        Err(err) => {
                            gloo::dialogs::alert(&format!(
                                "Failed to read selected file: {:#?}",
                                err
                            ));
                            input.set_value("");
                            return;
                        }
                    };

                    if let Err(err) =
                        process_uploaded_file(file_name, file_type, data, &mut potential_new_users)
                    {
                        gloo::dialogs::alert(&format!(
                            "Error in users file make sure proper headers are in place:\n{:#?}",
                            err
                        ));
                        input.set_value("");
                        potential_new_users.clear();
                        break;
                    }
                }

                // Then get a list of all the existing users
                // let found_users = match get_users_for_admin_config().await {
                //     Ok(found_users) => found_users,
                //     Err(err) => {
                //         gloo::dialogs::alert(&format!("Failed to get users from server: {:#?}", err));
                //         return;
                //     }
                // };

                // Sort out the duplicates from the new users
                let mut new_users = Vec::new();
                let mut found_dup_users = Vec::new();

                for (k, v) in potential_new_users.into_iter() {
                    if !found_users.contains_key(&k) {
                        new_users.push(UserAdminConfig {
                            id: k,
                            first_name: v.first_name.trim().to_string(),
                            last_name: v.last_name.trim().to_string(),
                            group: v.group.trim().to_string(),
                        });
                        continue;
                    }

                    let found_user = found_users.get(&k).unwrap();
                    if found_user.first_name == v.first_name.trim()
                        && found_user.last_name == v.last_name.trim()
                    {
                        found_dup_users.push(v);
                    } else {
                        let mut idx = 1;
                        loop {
                            let new_id = format!("{}{}", k, idx);
                            if found_users.contains_key(&new_id) {
                                idx += 1;
                                continue;
                            }
                            new_users.push(UserAdminConfig {
                                id: new_id,
                                first_name: v.first_name.trim().to_string(),
                                last_name: v.last_name.trim().to_string(),
                                group: v.group.trim().to_string(),
                            });
                            break;
                        }
                    }
                }

                users.set(new_users);
                dup_users.set(found_dup_users);
                is_working.set(false);
            });
        }
    };

    html! {
        <div class="modal fade" id="uploadUsersDlg"
             tabIndex="-1" role="dialog" aria-labelledby="uploadUsersDlgTitle" aria-hidden="true">
            <div class="modal-dialog modal-dialog-centered" role="document">
                <div class="modal-content">
                    <div class="modal-header">
                        <h5 class="modal-title" id="addUsersLongTitle">
                           {"Upload Users"}
                        </h5>
                    </div>
                    <div class="modal-body">
                        <div class="container-sm">
                            <div class="row">
                                {"Uploads a .csv/.xlsx/.ods formatted file."}
                                <br/>
                                <br/>
                                {"Files should have a header row with:"}
                                <br/>
                                {"last_name,first_name,group,uid"}
                                <br/>
                                <br/>
                                {"CSV file example formatting:"}
                                <br/>
                                {"Jim,Johnson,Apache,"}
                                <br/>
                                {"James,Kirk,Apache,jkirk"}
                                <br/>
                                <br/>
                                {"The UserID field is optional and if a duplicate is already found then this field will not be honoured"}
                                {" and a generated one will be used."}
                            </div>
                            <div class="row mt-2">
                                <input
                                    id="fileupload"
                                    type="file"
                                    accept=".csv,.ods,.xlsx"
                                    multiple={false}
                                    onchange={on_file_input_change}
                                />
                            </div>
                            <div class="row">
                            if *is_working {
                                <span class="spinner-border spinner-border-sm me-1" role="status"
                                aria-hidden="true" style="display: block;" />
                            } else {
                                if !dup_users.is_empty() {
                                    <h6>{"Duplicate Users"}</h6>
                                    <ul class="list-group">
                                    {
                                        (*dup_users).iter().map(|ui| {
                                            html!{
                                                <li class="list-group-item d-flex justify-content-between">
                                                    {format!("{} {}", ui.first_name, ui.last_name)}
                                                </li>
                                            }
                                        }).collect::<Html>()
                                    }
                                    </ul>
                                }

                                if users.is_empty() {
                                    <h6>{"New Users"}</h6>
                                    <ul class="list-group">
                                    {
                                        (*users).iter().map(|ui| {
                                            html!{
                                                <li class="list-group-item d-flex justify-content-between">
                                                    {format!("Name: {} {} | Id: {}", ui.first_name, ui.last_name, ui.id)}
                                                </li>
                                            }
                                        }).collect::<Html>()
                                    }
                                    </ul>
                                }
                            }
                            </div>
                        </div>
                    </div>
                    <div class="modal-footer">
                        <button type="button" class="btn btn-secondary" data-bs-dismiss="modal" onclick={on_cancel}>{"Cancel"}</button>
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
fn edit_user_dlg(props: &EditUserDlgProps) -> Html {
    let selected_user = use_state_eq(SelectedUserType::default);
    {
        let selected_user = selected_user.clone();
        SELECTED_USER.with(|rc| {
            *rc.borrow_mut() = Some(selected_user);
        });
    }

    let on_update = {
        let onupdate = props.onupdate.clone();
        let selected_user = selected_user.clone();
        move |_evt: MouseEvent| {
            let SelectedUserType { uid, name, .. } = &*selected_user;
            let document = gloo::utils::document();
            let group = document
                .get_element_by_id("formEditUserGroup")
                .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
                .unwrap()
                .value();
            onupdate.emit((uid.clone(), name.clone(), group));
        }
    };

    let SelectedUserType { uid, name, group } = &*selected_user;

    html! {
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
                            <div class="row mb-2">
                                <div class="col-md">
                                    <div class="form-floating">
                                        <input class="form-control" type="text" id="formEditUserUid"
                                            readonly=true
                                            value={uid.clone()} />
                                            <label for="formEditUserUid">{"UserID"}</label>
                                    </div>
                                </div>
                            </div>
                            <div class="row mb-2">
                                <div class="col-md">
                                    <div class="form-floating">
                                        <input class="form-control" type="text" id="formEditUserName"
                                            readonly=true
                                            value={name.clone()} />
                                            <label for="formEditUserName">{"Name"}</label>
                                    </div>
                                </div>
                            </div>
                            <div class="row">
                                <div class="col-md">
                                    <div class="form-floating col-md">
                                        <input class="form-control" type="text" autocomplete="fr-new-distpt" id="formEditUserGroup"
                                            required=true
                                            value={group.clone()} />
                                        <label for="formEditUserGroup">{"Group"}</label>
                                    </div>
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
fn user_item(props: &UserLiProps) -> Html {
    html! {
        <li class="list-group-item d-flex justify-content-between">
            <div>
                <div class="mb-1">{props.uid.clone()}</div>
                <small class="text-muted">{format!("Name: {}", &props.name)}</small>
                <small class="text-muted mx-2">{format!("Group: {}", &props.group)}</small>
            </div>
            if props.uid!="fradmin" {
                <div class="float-end">
                    <button class="btn btn-outline-info float-end order-edt-btn"
                        data-uid={props.uid.clone()} onclick={props.onedit.clone()}>
                        <i class="bi bi-pencil" fill="currentColor"></i>
                    </button>
                </div>
            }
        </li>
    }
}

/////////////////////////////////////////////////
fn get_selected_user(evt: MouseEvent) -> String {
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

    elm.dataset().get("uid").unwrap()
}

/////////////////////////////////////////////////
fn disable_save_button(document: &web_sys::Document, value: bool) {
    if let Some(btn) = document
        .get_element_by_id("btnSaveUsersConfigUpdates")
        .and_then(|t| t.dyn_into::<HtmlButtonElement>().ok())
    {
        btn.set_disabled(value);
        let spinner_display = if value { "inline-block" } else { "none" };
        let _ = document
            .get_element_by_id("saveUsersConfigSpinner")
            .and_then(|t| t.dyn_into::<HtmlElement>().ok())
            .unwrap()
            .style()
            .set_property("display", spinner_display);
    }
}

#[function_component(UsersUl)]
pub(crate) fn user_list() -> Html {
    let users = use_state_eq(BTreeMap::<String, UserAdminConfig>::new);
    let is_dirty = use_state_eq(|| false);
    let dirty_entries = use_mut_ref(std::collections::HashSet::<String>::new);

    let on_upload_users_dlg_submit = {
        let is_dirty = is_dirty.clone();
        let users = users.clone();
        let dirty_entries = dirty_entries.clone();
        move |new_users: Vec<UserAdminConfig>| {
            log::info!("Adding Users...");
            let mut users_map = (*users).clone();
            for user_info in new_users {
                dirty_entries.borrow_mut().insert(user_info.id.clone());
                users_map.insert(user_info.id.clone(), user_info);
            }
            users.set(users_map);
            is_dirty.set(true);
        }
    };

    let on_edit_user_group_dlg_submit = {
        let is_dirty = is_dirty.clone();
        let users = users.clone();
        let dirty_entries = dirty_entries.clone();
        move |vals: EditUserDlgCb| {
            let (uid, name, group) = vals.to_owned();
            log::info!("Done Editing User: {}, \"{}\", \"{}\"", &uid, &name, &group);
            let mut users_map = (*users).clone();
            let mut user_info = users_map.get(&uid).unwrap().clone();
            if user_info.group.ne(&group) {
                user_info.group = group;
                users_map.insert(uid.clone(), user_info);
                users.set(users_map);
                dirty_entries.borrow_mut().insert(uid);
                is_dirty.set(true);
            }
        }
    };

    let on_upload_users = {
        move |_evt: MouseEvent| {
            bootstrap::modal_op("uploadUsersDlg", "toggle");
        }
    };

    let on_edit_users_group = {
        let users = users.clone();
        move |evt: MouseEvent| {
            let uid = get_selected_user(evt);
            let user_info = (*users).get(&uid).unwrap();
            let name = format!("{} {}", user_info.first_name, user_info.last_name);
            log::info!("Editing User: {} {} {}", &uid, &name, &user_info.group);

            SELECTED_USER.with(|rc| {
                rc.borrow().as_ref().unwrap().set(SelectedUserType {
                    uid: uid.clone(),
                    name,
                    group: user_info.group.clone(),
                });
            });
            bootstrap::modal_op("editUserDlg", "toggle");
        }
    };

    let on_save_users = {
        let users = users.clone();
        let dirty_entries = dirty_entries.clone();
        let is_dirty = is_dirty.clone();
        move |_evt: MouseEvent| {
            log::info!("Saving Users to cloud");
            let document = gloo::utils::document();
            disable_save_button(&document, true);

            let mut updated_users = Vec::new();
            for uid in dirty_entries.borrow().iter() {
                updated_users.push((*users).get(uid).unwrap().clone());
            }

            let is_dirty = is_dirty.clone();
            let dirty_entries = dirty_entries.clone();
            wasm_bindgen_futures::spawn_local(async move {
                //log::info!("Saving users to cloud: {:#?}", updated_users);
                match add_or_update_users_for_admin_config(updated_users).await {
                    Err(err) => {
                        gloo::dialogs::alert(&format!("Failed adding/updating users:\n{:#?}", err));
                    }
                    _ => {
                        dirty_entries.borrow_mut().clear();
                        is_dirty.set(false);
                    }
                }
                disable_save_button(&document, false);
            });
        }
    };

    {
        let users = users.clone();
        use_effect(move || {
            if users.is_empty() {
                wasm_bindgen_futures::spawn_local(async move {
                    log::info!("Getting user list from cloud");
                    match get_users_for_admin_config().await {
                        Ok(user_map) => users.set(user_map),
                        Err(err) => gloo::dialogs::alert(&format!(
                            "Failed to get user list from server: {:#?}",
                            err
                        )),
                    };
                });
            }
            || {}
        });
    }
    html! {
        <div>
            <div class="card">
                <div class="card-body">
                    <h5 class="card-title">
                        {"Users"}
                        <button class="btn btn-outline-info float-end order-edt-btn" onclick={on_upload_users}>
                            <i class="bi bi-cloud-upload" fill="currentColor"></i>
                        </button>
                        if *is_dirty {
                            <button class="btn btn-primary" onclick={on_save_users} id="btnSaveUsersConfigUpdates" >
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
                                    name={format!("{} {}",user_info.first_name, user_info.last_name)}
                                    group={user_info.group.clone()}
                                    onedit={on_edit_users_group.clone()} />}
                        }).collect::<Html>()
                    }
                    </ul>
                </div>
            </div>
            <EditUserDlg onupdate={on_edit_user_group_dlg_submit}/>
            <UploadUsersDlg onadd={on_upload_users_dlg_submit} knownusers={(*users).clone()}/>
        </div>
    }
}
