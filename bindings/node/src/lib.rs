//! Native part of the Node.js package. Results cross the boundary as JSON
//! strings, index.js parses them, so the shape is exactly the Rust model.

use napi_derive::napi;

#[napi]
pub fn symbols_json(source: String) -> String {
    abap_analysis::json::symbols(&source)
}

#[napi]
pub fn db_access_json(source: String) -> String {
    abap_analysis::json::db_access(&source)
}

#[napi]
pub fn calls_json(source: String) -> String {
    abap_analysis::json::calls(&source)
}

/// Blocking: loads sources from folders or the SAP system.
#[napi]
pub fn call_graph_json(roots: Vec<String>, options: String, depth: u32) -> napi::Result<String> {
    abap_analysis::json::call_graph(&roots, &options, depth).map_err(napi::Error::from_reason)
}

#[napi]
pub fn fetch_source(reference: String, options: String) -> napi::Result<Option<String>> {
    abap_analysis::json::fetch(&reference, &options).map_err(napi::Error::from_reason)
}

#[napi]
pub fn logic_json(source: String) -> String {
    abap_analysis::json::logic(&source)
}

#[napi]
pub fn statements_json(source: String) -> String {
    abap_analysis::json::statements(&source)
}

#[napi]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
