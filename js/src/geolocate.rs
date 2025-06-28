use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "/src/js/geolocate.js")]
extern "C" {
    #[wasm_bindgen(catch)]
    async fn getExactCurrentPosition() -> Result<JsValue, JsValue>;
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct GeolocationCoordinates {
    pub latitude: f64,
    pub longitude: f64,
    pub accuracy: f64,
    pub altitude: Option<f64>,
    #[serde(rename = "altitudeAccuracy")]
    pub altitude_accuracy: Option<f64>,
    pub heading: Option<f64>,
    pub speed: Option<f64>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct GeolocationPosition {
    pub coords: GeolocationCoordinates,
    pub timestamp: i64,
}

pub async fn get_current_position() -> Option<GeolocationPosition> {
    match getExactCurrentPosition().await {
        Ok(pos) => {
            let pos: GeolocationPosition = serde_wasm_bindgen::from_value(pos).unwrap();
            // log::info!("Position: {:#?}", &pos);
            Some(pos)
        }
        Err(err) => {
            log::error!("Geolocation Err: {err:#?}");
            gloo::dialogs::alert(&format!("Failed to get Geolocation Info: {err:#?}"));
            None
        }
    }
}
