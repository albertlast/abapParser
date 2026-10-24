//! Native part of the Node.js package. Results cross the boundary as JSON
//! strings, index.js parses them, so the shape is exactly the Rust model.
//!
//! Every function has an `…Async` variant that runs on the libuv thread
//! pool and returns a promise, so loading sources from an SAP system or
//! analyzing large files does not block the event loop.
//!
//! The async variants loading sources take a `fetch` function of the host:
//! the worker threads call it through a threadsafe function and wait for
//! the promise it returns, while the event loop runs it. They also take a
//! `progress` function, called without waiting, and a [`CancelToken`].

use std::future::Future;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};
use std::task::{Context, Poll, Wake, Waker};

use abap_analysis::abap_source::{FnProvider, SourceError, SourceRef};
use abap_analysis::json::Hooks;
use napi::bindgen_prelude::{AsyncTask, Env, Error, Function, Promise, Result, Task};
use napi::threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode};
use napi::Status;
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

/// `(reference) => Promise<string | null>`, index.js makes every fetch
/// function of the host return a promise.
type FetchJs<'a> = Function<'a, String, Promise<Option<String>>>;
type FetchTsfn = ThreadsafeFunction<String, Promise<Option<String>>, String, Status, false>;

/// `(json) => void`, called with `{"loaded": 3, "object": "CLAS:ZCL_X"}`.
type ProgressJs<'a> = Function<'a, String, ()>;
type ProgressTsfn = ThreadsafeFunction<String, (), String, Status, false>;

/// Stops an analysis loading sources: once cancelled, nothing more is
/// loaded and the analysis finishes with what it has. index.js cancels it
/// when the `signal` of the options aborts.
#[napi]
pub struct CancelToken {
    flag: Arc<AtomicBool>,
}

#[napi]
impl CancelToken {
    #[napi(constructor)]
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        CancelToken {
            flag: Arc::new(AtomicBool::new(false)),
        }
    }

    #[napi]
    pub fn cancel(&self) {
        self.flag.store(true, Ordering::Relaxed);
    }
}

/// The functions of the host for an analysis: built in the JavaScript
/// thread, called from the worker threads.
fn hooks(
    fetch: Option<FetchJs>,
    progress: Option<ProgressJs>,
    cancel: Option<&CancelToken>,
) -> Result<Hooks> {
    let mut hooks = Hooks::default();
    if let Some(f) = fetch {
        let tsfn: Arc<FetchTsfn> = Arc::new(
            f.build_threadsafe_function()
                .callee_handled::<false>()
                .build()?,
        );
        hooks.host = Some(Box::new(FnProvider::new("fetch", move |r| {
            call_fetch(&tsfn, r)
        })));
    }
    if let Some(f) = progress {
        let tsfn: ProgressTsfn = f
            .build_threadsafe_function()
            .callee_handled::<false>()
            .build()?;
        hooks.progress = Some(Box::new(move |p| {
            let json = serde_json::json!({"loaded": p.loaded, "object": p.object});
            tsfn.call(json.to_string(), ThreadsafeFunctionCallMode::NonBlocking);
        }));
    }
    hooks.cancel = cancel.map(|c| c.flag.clone());
    Ok(hooks)
}

fn call_fetch(
    tsfn: &FetchTsfn,
    r: &SourceRef,
) -> abap_analysis::abap_source::Result<Option<String>> {
    let failed = |e: String| SourceError::Other(format!("fetch {r}: {e}"));
    let (tx, rx) = mpsc::channel();
    let status = tsfn.call_with_return_value(
        r.to_string(),
        ThreadsafeFunctionCallMode::Blocking,
        move |promise, _env| {
            let _ = tx.send(promise);
            Ok(())
        },
    );
    if status != Status::Ok {
        return Err(failed(format!("not called ({status})")));
    }
    let promise = rx
        .recv()
        .map_err(|_| failed("no answer".to_string()))?
        .map_err(|e| failed(e.reason.clone()))?;
    block_on(promise).map_err(|e| failed(e.reason.clone()))
}

/// Waits in this thread for a future, like the promise of the fetch
/// function, which the JavaScript thread settles.
fn block_on<F: Future>(future: F) -> F::Output {
    struct Unpark(std::thread::Thread);
    impl Wake for Unpark {
        fn wake(self: Arc<Self>) {
            self.0.unpark();
        }
    }
    let waker = Waker::from(Arc::new(Unpark(std::thread::current())));
    let mut cx = Context::from_waker(&waker);
    let mut future = std::pin::pin!(future);
    loop {
        if let Poll::Ready(out) = future.as_mut().poll(&mut cx) {
            return out;
        }
        std::thread::park();
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
    abap_analysis::json::call_graph(&roots, &options, depth, Hooks::default())
        .map_err(Error::from_reason)
}

#[napi(ts_return_type = "Promise<string>")]
pub fn call_graph_json_async(
    roots: Vec<String>,
    options: String,
    depth: u32,
    fetch: Option<FetchJs>,
    progress: Option<ProgressJs>,
    cancel: Option<&CancelToken>,
) -> Result<AsyncTask<JsonTask>> {
    let hooks = hooks(fetch, progress, cancel)?;
    Ok(JsonTask::new(move || {
        abap_analysis::json::call_graph(&roots, &options, depth, hooks)
    }))
}

#[napi]
pub fn fetch_source(reference: String, options: String) -> Result<Option<String>> {
    abap_analysis::json::fetch(&reference, &options, Hooks::default()).map_err(Error::from_reason)
}

/// The source as a JSON string or `null`.
#[napi(ts_return_type = "Promise<string>")]
pub fn fetch_source_json_async(
    reference: String,
    options: String,
    fetch: Option<FetchJs>,
    progress: Option<ProgressJs>,
    cancel: Option<&CancelToken>,
) -> Result<AsyncTask<JsonTask>> {
    let hooks = hooks(fetch, progress, cancel)?;
    Ok(JsonTask::new(move || {
        let text = abap_analysis::json::fetch(&reference, &options, hooks)?;
        Ok(serde_json::to_string(&text).expect("text serializes"))
    }))
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
    abap_analysis::json::logic_object(&reference, &options, depth, Hooks::default())
        .map_err(Error::from_reason)
}

#[napi(ts_return_type = "Promise<string>")]
pub fn logic_object_json_async(
    reference: String,
    options: String,
    depth: u32,
    fetch: Option<FetchJs>,
    progress: Option<ProgressJs>,
    cancel: Option<&CancelToken>,
) -> Result<AsyncTask<JsonTask>> {
    let hooks = hooks(fetch, progress, cancel)?;
    Ok(JsonTask::new(move || {
        abap_analysis::json::logic_object(&reference, &options, depth, hooks)
    }))
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
pub fn cds_json(source: String) -> String {
    abap_analysis::json::cds(&source)
}

#[napi(ts_return_type = "Promise<string>")]
pub fn cds_json_async(source: String) -> AsyncTask<JsonTask> {
    JsonTask::new(move || Ok(abap_analysis::json::cds(&source)))
}

/// Compact text of one source: `symbols`, `calls`, `db`, `logic` or `cds`.
#[napi]
pub fn text(analysis: String, source: String) -> Result<String> {
    abap_analysis::json::text(&analysis, &source).map_err(Error::from_reason)
}

#[napi(ts_return_type = "Promise<string>")]
pub fn text_async(analysis: String, source: String) -> AsyncTask<JsonTask> {
    JsonTask::new(move || abap_analysis::json::text(&analysis, &source))
}

/// Blocking: compact text of objects, `symbols`, `logic` or `graph`.
#[napi]
pub fn object_text(
    analysis: String,
    objects: Vec<String>,
    options: String,
    depth: u32,
) -> Result<String> {
    abap_analysis::json::object_text(&analysis, &objects, &options, depth, Hooks::default())
        .map_err(Error::from_reason)
}

#[napi(ts_return_type = "Promise<string>")]
pub fn object_text_async(
    analysis: String,
    objects: Vec<String>,
    options: String,
    depth: u32,
    fetch: Option<FetchJs>,
    progress: Option<ProgressJs>,
    cancel: Option<&CancelToken>,
) -> Result<AsyncTask<JsonTask>> {
    let hooks = hooks(fetch, progress, cancel)?;
    Ok(JsonTask::new(move || {
        abap_analysis::json::object_text(&analysis, &objects, &options, depth, hooks)
    }))
}

#[napi]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
