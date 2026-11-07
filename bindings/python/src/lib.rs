//! Native part of the Python package. Results cross the boundary as JSON
//! strings, abap_parser/__init__.py parses them, so the shape is exactly the
//! Rust model.
//!
//! Functions loading sources take the `fetch`, `query` and `progress`
//! callables of the host, and a [`CancelToken`]; the native code runs
//! without the GIL and takes it to call them. abap_parser wraps `fetch` and
//! `query` to answer in JSON, `{"value": ...}` or `{"error": {...}}` (see
//! [`FnProvider::json`]).

use pyo3::prelude::*;

#[pymodule]
mod _native {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    use abap_analysis::abap_source::{FnProvider, HostFn};
    use abap_analysis::json::Hooks;
    use pyo3::prelude::*;
    use pyo3::types::PyDict;

    /// Stops an analysis loading sources: once cancelled, nothing more is
    /// loaded and the analysis finishes with what it has.
    #[pyclass(frozen)]
    struct CancelToken {
        flag: Arc<AtomicBool>,
    }

    #[pymethods]
    impl CancelToken {
        #[new]
        fn new() -> Self {
            CancelToken {
                flag: Arc::new(AtomicBool::new(false)),
            }
        }

        fn cancel(&self) {
            self.flag.store(true, Ordering::Relaxed);
        }

        #[getter]
        fn cancelled(&self) -> bool {
            self.flag.load(Ordering::Relaxed)
        }
    }

    /// A callable of the host answering in JSON.
    fn host_fn(f: Py<PyAny>) -> HostFn {
        Box::new(move |argument| {
            Python::attach(|py| {
                f.call1(py, (argument,))
                    .and_then(|v| v.bind(py).extract::<String>())
            })
            .map_err(|e| e.to_string())
        })
    }

    /// `fetch` and `query` as a source provider, `progress({"loaded": n,
    /// "queued": n, "object": "CLAS:ZCL_X", "kind": "CLAS", "outcome":
    /// "loaded"})`, and the flag of the cancel token.
    fn hooks(
        fetch: Option<Py<PyAny>>,
        query: Option<Py<PyAny>>,
        progress: Option<Py<PyAny>>,
        cancel: Option<&Bound<'_, CancelToken>>,
    ) -> Hooks {
        let host: Option<Box<dyn abap_analysis::abap_source::SourceProvider>> =
            if fetch.is_some() || query.is_some() {
                Some(Box::new(FnProvider::json(
                    fetch.map(host_fn),
                    query.map(host_fn),
                )))
            } else {
                None
            };
        Hooks {
            host,
            progress: progress.map(|progress| -> abap_analysis::ProgressFn {
                Box::new(move |p| {
                    Python::attach(|py| {
                        let d = PyDict::new(py);
                        let set = d
                            .set_item("loaded", p.loaded)
                            .and_then(|_| d.set_item("queued", p.queued))
                            .and_then(|_| d.set_item("object", &p.object))
                            .and_then(|_| d.set_item("kind", p.kind))
                            .and_then(|_| d.set_item("outcome", p.outcome.as_str()));
                        // an error of the callable does not stop the analysis
                        if let Err(e) = set.and_then(|_| progress.call1(py, (d,)).map(|_| ())) {
                            e.print(py);
                        }
                    })
                })
            }),
            cancel: cancel.map(|c| c.get().flag.clone()),
        }
    }

    #[pyfunction]
    fn symbols_json(py: Python<'_>, source: &str) -> String {
        py.detach(|| abap_analysis::json::symbols(source))
    }

    #[pyfunction]
    fn db_access_json(py: Python<'_>, source: &str) -> String {
        py.detach(|| abap_analysis::json::db_access(source))
    }

    #[pyfunction]
    fn calls_json(py: Python<'_>, source: &str) -> String {
        py.detach(|| abap_analysis::json::calls(source))
    }

    // the signature of the Python function
    #[allow(clippy::too_many_arguments)]
    #[pyfunction]
    #[pyo3(signature = (roots, options, depth, fetch=None, query=None, progress=None, cancel=None))]
    fn call_graph_json(
        py: Python<'_>,
        roots: Vec<String>,
        options: &str,
        depth: u32,
        fetch: Option<Py<PyAny>>,
        query: Option<Py<PyAny>>,
        progress: Option<Py<PyAny>>,
        cancel: Option<&Bound<'_, CancelToken>>,
    ) -> PyResult<String> {
        let hooks = hooks(fetch, query, progress, cancel);
        py.detach(|| abap_analysis::json::call_graph(&roots, options, depth, hooks))
            .map_err(pyo3::exceptions::PyRuntimeError::new_err)
    }

    #[pyfunction]
    #[pyo3(signature = (reference, options, fetch=None, query=None, progress=None, cancel=None))]
    fn fetch_source(
        py: Python<'_>,
        reference: &str,
        options: &str,
        fetch: Option<Py<PyAny>>,
        query: Option<Py<PyAny>>,
        progress: Option<Py<PyAny>>,
        cancel: Option<&Bound<'_, CancelToken>>,
    ) -> PyResult<Option<String>> {
        let hooks = hooks(fetch, query, progress, cancel);
        py.detach(|| abap_analysis::json::fetch(reference, options, hooks))
            .map_err(pyo3::exceptions::PyRuntimeError::new_err)
    }

    #[pyfunction]
    fn logic_json(py: Python<'_>, source: &str) -> String {
        py.detach(|| abap_analysis::json::logic(source))
    }

    // the signature of the Python function
    #[allow(clippy::too_many_arguments)]
    #[pyfunction]
    #[pyo3(signature = (reference, options, depth, fetch=None, query=None, progress=None, cancel=None))]
    fn logic_object_json(
        py: Python<'_>,
        reference: &str,
        options: &str,
        depth: u32,
        fetch: Option<Py<PyAny>>,
        query: Option<Py<PyAny>>,
        progress: Option<Py<PyAny>>,
        cancel: Option<&Bound<'_, CancelToken>>,
    ) -> PyResult<String> {
        let hooks = hooks(fetch, query, progress, cancel);
        py.detach(|| abap_analysis::json::logic_object(reference, options, depth, hooks))
            .map_err(pyo3::exceptions::PyRuntimeError::new_err)
    }

    #[pyfunction]
    fn statements_json(py: Python<'_>, source: &str) -> String {
        py.detach(|| abap_analysis::json::statements(source))
    }

    #[pyfunction]
    fn cds_json(py: Python<'_>, source: &str) -> String {
        py.detach(|| abap_analysis::json::cds(source))
    }

    #[pyfunction]
    fn text(py: Python<'_>, analysis: &str, source: &str) -> PyResult<String> {
        py.detach(|| abap_analysis::json::text(analysis, source))
            .map_err(pyo3::exceptions::PyValueError::new_err)
    }

    // the signature of the Python function
    #[allow(clippy::too_many_arguments)]
    #[pyfunction]
    #[pyo3(signature = (analysis, objects, options, depth, fetch=None, query=None, progress=None, cancel=None))]
    fn object_text(
        py: Python<'_>,
        analysis: &str,
        objects: Vec<String>,
        options: &str,
        depth: u32,
        fetch: Option<Py<PyAny>>,
        query: Option<Py<PyAny>>,
        progress: Option<Py<PyAny>>,
        cancel: Option<&Bound<'_, CancelToken>>,
    ) -> PyResult<String> {
        let hooks = hooks(fetch, query, progress, cancel);
        py.detach(|| abap_analysis::json::object_text(analysis, &objects, options, depth, hooks))
            .map_err(pyo3::exceptions::PyRuntimeError::new_err)
    }

    #[pyfunction]
    fn version() -> &'static str {
        env!("CARGO_PKG_VERSION")
    }
}
