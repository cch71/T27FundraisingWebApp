use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};

#[wasm_bindgen(module = "/src/js/geolocate.js")]
extern "C" {
    #[wasm_bindgen(catch)]
    async fn getExactCurrentPosition() -> Result<JsValue, JsValue>;
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub(crate) struct GeolocationCoordinates {
    pub(crate) latitude: f64,
    pub(crate) longitude: f64,
    pub(crate) accuracy: f64,
    pub(crate) altitude: Option<f64>,
    pub(crate) altitudeAccuracy: Option<f64>,
    pub(crate) heading: Option<f64>,
    pub(crate) speed: Option<f64>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub(crate) struct GeolocationPosition {
    pub(crate) coords: GeolocationCoordinates,
    pub(crate) timestamp: i64,
}

pub(crate) async fn get_current_position() -> Option<GeolocationPosition> {
    match getExactCurrentPosition().await {
        Ok(pos) => {
            let pos: GeolocationPosition = serde_wasm_bindgen::from_value(pos).unwrap();
            // log::info!("Position: {:#?}", &pos);
            Some(pos)
        },
        Err(err) => {
            log::error!("Geolocation Err: {:#?}", err);
            gloo::dialogs::alert(&format!("Failed to get Geolocation Info: {:#?}", err));
            None
        },
    }
}
