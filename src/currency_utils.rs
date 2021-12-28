use web_sys::{KeyboardEvent};


pub(crate) fn on_currency_field_key_press(_evt: KeyboardEvent) {
    log::info!("On currency field key pres");

    // const charCode = (evt.which) ? evt.which : event.keyCode;
    // if (45 === charCode) {
    //     evt.preventDefault();
    //     evt.stopPropagation();
    //     return false;
    // }
    // if (charCode != 46 && charCode > 31 && (charCode < 48 || charCode > 57)) {
    //     evt.preventDefault();
    //     evt.stopPropagation();
    //     return false;
    // }
    // return true;
}

