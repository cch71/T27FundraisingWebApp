use web_sys::{KeyboardEvent};
use rusty_money::{Money, iso};


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

pub(crate) fn to_money_str(input: Option<&String>) -> String {
    input.map_or_else(
        || "".to_string(),
        |v| Money::from_str(v, iso::USD) .unwrap() .to_string()
    )
}


