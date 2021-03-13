extern crate console_error_panic_hook;

use serde::Serialize;
use sourcemap::{Error, SourceMap};
use std::{
    cell::RefCell,
    collections::HashMap,
    panic::{self},
};
use wasm_bindgen::prelude::*;

enum LookupStatus {
    SourceMapNotFound,
    LookupFailed,
    LookupSuccess(LookupResult),
}
#[wasm_bindgen]
#[derive(Serialize)]
pub struct LookupResult {
    #[allow(dead_code)]
    line: u32,
    #[allow(dead_code)]
    column: u32,
    #[allow(dead_code)]
    source: String,
}

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

thread_local! {
    static COUNTER: RefCell<u32> = RefCell::new(0);
    static PARSED_MAPS: RefCell<HashMap<u32, SourceMap>> = RefCell::new(HashMap::new());
}

#[wasm_bindgen]
pub fn parse_source_map(source_map: &[u8]) -> Result<f64, JsValue> {
    panic::set_hook(Box::new(console_error_panic_hook::hook));

    let index = COUNTER.with(|counter| {
        *counter.borrow_mut() += 1;
        *counter.borrow()
    });

    let parse_error: Option<Error> = PARSED_MAPS.with(|parsed_maps| {
        let parsed_map = SourceMap::from_reader(source_map);
        match parsed_map {
            Ok(parsed_map) => {
                parsed_maps.borrow_mut().insert(index, parsed_map);
                None
            }
            Err(err) => Some(err),
        }
    });

    match parse_error {
        Some(e) => Err(JsValue::from(String::from(format!(
            "Failed to parse source map - Reason: {}",
            e.to_string()
        )))),
        None => Ok(index as f64),
    }
}

#[wasm_bindgen]
pub fn lookup_original_position(handle: u32, line: u32, column: u32) -> Result<JsValue, JsValue> {
    panic::set_hook(Box::new(console_error_panic_hook::hook));

    let lookup_status: LookupStatus = PARSED_MAPS.with(|parsed_maps| {
        let parsed_map = parsed_maps.borrow();
        let parsed_map = parsed_map.get(&handle);

        if let Some(parsed_map) = parsed_map {
            let token = parsed_map.lookup_token(line, column);

            if let Some(token) = token {
                LookupStatus::LookupSuccess(LookupResult {
                    line: token.get_src_line(),
                    column: token.get_src_col(),
                    source: match token.get_source() {
                        Some(v) => v.to_string(),
                        None => String::from(""),
                    },
                })
            } else {
                LookupStatus::LookupFailed
            }
        } else {
            LookupStatus::SourceMapNotFound
        }
    });

    match lookup_status {
        LookupStatus::LookupSuccess(lookup_result) => {
            Ok(serde_wasm_bindgen::to_value(&lookup_result).unwrap())
        }
        LookupStatus::SourceMapNotFound => Err(JsValue::from(String::from(format!(
            "Source map was not found, did you dispose it?"
        )))),
        LookupStatus::LookupFailed => Err(JsValue::from(String::from(format!(
            "Failed to lookup original position for given line and column"
        )))),
    }
}

#[wasm_bindgen]
pub fn dispose(handle: u32) -> usize {
    panic::set_hook(Box::new(console_error_panic_hook::hook));

    let maps_left = PARSED_MAPS.with(|parsed_maps| {
        parsed_maps.borrow_mut().remove(&handle);
        parsed_maps.borrow().len()
    });

    maps_left
}