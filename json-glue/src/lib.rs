use std::fmt::Display;

use serde::Serialize;
use serde_wasm_bindgen::*;
use wasm_bindgen::prelude::*;

static SERIALIZER: Serializer = Serializer::new()
    .serialize_large_number_types_as_bigints(true)
    .serialize_maps_as_objects(true)
    .serialize_bytes_as_arrays(true);

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn error(s: &str);
}

pub fn print_error<T: Display>(err: T) -> JsValue {
    let s = format!("Error: {err}");
    error(&s);
    JsValue::null()
}

#[wasm_bindgen(js_name = fromJsonString)]
pub fn from_json(json_str: &str) -> Result<JsValue, JsValue> {
    let value: serde_json::Value = serde_json::from_str(json_str).map_err(print_error)?;

    value.serialize(&SERIALIZER).map_err(print_error)
}

#[wasm_bindgen(js_name = toJsonString)]
pub fn to_json(obj: JsValue) -> Result<JsValue, JsValue> {
    let v: serde_json::Value = from_value(obj).map_err(print_error)?;
    let s = serde_json::to_string(&v).map_err(print_error)?;
    to_value(&s).map_err(print_error)
}
