//! Native part of the Node.js package. Results cross the boundary as JSON
//! strings, index.js parses them, so the shape is exactly the Rust model.
//!
//! Every function has an `…Async` variant that runs on the libuv thread
//! pool and returns a promise, so loading sources from an SAP system or
//! analyzing large files does not block the event loop.

use napi::bindgen_prelude::{AsyncTask, Env, Error, Result, Task};
use napi_derive::napi;

type Job = Box<dyn FnOnce() -> std::result::Result<String, String> + Send>;

/// Work for the thread pool returning JSON, or an error message.
pub struct JsonTask(Option<Job>);

impl JsonTask {
    fn new(
        f: impl FnOnce() -> std::result::Result<String, String> + Send + 'static,
    ) -> AsyncTask<Self> {
        AsyncTask::new(JsonTask(Some(Box::new(f))))
    }
}

impl Task for JsonTask {
    type Output = String;
    type JsValue = String;

    fn compute(&mut self) -> Result<String> {
        let job = self
            .0
            .take()
            .ok_or_else(|| Error::from_reason("task ran twice"))?;
        job().map_err(Error::from_reason)
    }

    fn resolve(&mut self, _env: Env, output: String) -> Result<String> {
        Ok(output)
    }
}

#[napi]
pub fn symbols_json(source: String) -> String {
    abap_analysis::json::symbols(&source)
}

#[napi(ts_return_type = "Promise<string>")]
pub fn symbols_json_async(source: String) -> AsyncTask<JsonTask> {
    JsonTask::new(move || Ok(abap_analysis::json::symbols(&source)))
}

#[napi]
pub fn db_access_json(source: String) -> String {
    abap_analysis::json::db_access(&source)
}

#[napi(ts_return_type = "Promise<string>")]
pub fn db_access_json_async(source: String) -> AsyncTask<JsonTask> {
    JsonTask::new(move || Ok(abap_analysis::json::db_access(&source)))
}

#[napi]
pub fn calls_json(source: String) -> String {
    abap_analysis::json::calls(&source)
}

#[napi(ts_return_type = "Promise<string>")]
pub fn calls_json_async(source: String) -> AsyncTask<JsonTask> {
    JsonTask::new(move || Ok(abap_analysis::json::calls(&source)))
}

/// Blocking: loads sources from folders or the SAP system.
#[napi]
pub fn call_graph_json(roots: Vec<String>, options: String, depth: u32) -> Result<String> {
    abap_analysis::json::call_graph(&roots, &options, depth).map_err(Error::from_reason)
}

#[napi(ts_return_type = "Promise<string>")]
pub fn call_graph_json_async(
    roots: Vec<String>,
    options: String,
    depth: u32,
) -> AsyncTask<JsonTask> {
    JsonTask::new(move || abap_analysis::json::call_graph(&roots, &options, depth))
}

#[napi]
pub fn fetch_source(reference: String, options: String) -> Result<Option<String>> {
    abap_analysis::json::fetch(&reference, &options).map_err(Error::from_reason)
}

/// The source as a JSON string or `null`.
#[napi(ts_return_type = "Promise<string>")]
pub fn fetch_source_json_async(reference: String, options: String) -> AsyncTask<JsonTask> {
    JsonTask::new(move || {
        let text = abap_analysis::json::fetch(&reference, &options)?;
        Ok(serde_json::to_string(&text).expect("text serializes"))
    })
}

#[napi]
pub fn logic_json(source: String) -> String {
    abap_analysis::json::logic(&source)
}

#[napi(ts_return_type = "Promise<string>")]
pub fn logic_json_async(source: String) -> AsyncTask<JsonTask> {
    JsonTask::new(move || Ok(abap_analysis::json::logic(&source)))
}

/// Blocking: loads sources from folders or the SAP system.
#[napi]
pub fn logic_object_json(reference: String, options: String, depth: u32) -> Result<String> {
    abap_analysis::json::logic_object(&reference, &options, depth).map_err(Error::from_reason)
}

#[napi(ts_return_type = "Promise<string>")]
pub fn logic_object_json_async(
    reference: String,
    options: String,
    depth: u32,
) -> AsyncTask<JsonTask> {
    JsonTask::new(move || abap_analysis::json::logic_object(&reference, &options, depth))
}

#[napi]
pub fn statements_json(source: String) -> String {
    abap_analysis::json::statements(&source)
}

#[napi(ts_return_type = "Promise<string>")]
pub fn statements_json_async(source: String) -> AsyncTask<JsonTask> {
    JsonTask::new(move || Ok(abap_analysis::json::statements(&source)))
}

#[napi]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
