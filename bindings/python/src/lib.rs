//! Native part of the Python package. Results cross the boundary as JSON
//! strings, abap_parser/__init__.py parses them, so the shape is exactly the
//! Rust model.
//!
//! Functions loading sources take a `fetch` and a `progress` callable of
//! the host, and a [`CancelToken`]; the native code runs without the GIL
//! and takes it to call them.

use pyo3::prelude::*;

#[pymodule]
mod _native {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    use abap_analysis::abap_source::{FnProvider, SourceError};
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

    /// `fetch(reference) -> str | None` as a source provider,
    /// `progress({"loaded": n, "object": "CLAS:ZCL_X"})`, and the flag of
    /// the cancel token.
    fn hooks(
        fetch: Option<Py<PyAny>>,
        progress: Option<Py<PyAny>>,
        cancel: Option<&Bound<'_, CancelToken>>,
    ) -> Hooks {
        Hooks {
            host: fetch.map(
                |fetch| -> Box<dyn abap_analysis::abap_source::SourceProvider> {
                    Box::new(FnProvider::new("fetch", move |r| {
                        Python::attach(|py| {
                            fetch
                                .call1(py, (r.to_string(),))
                                .and_then(|v| v.bind(py).extract::<Option<String>>())
                        })
                        .map_err(|e| SourceError::Other(format!("fetch {r}: {e}")))
                    }))
                },
            ),
            progress: progress.map(|progress| -> abap_analysis::ProgressFn {
                Box::new(move |p| {
                    Python::attach(|py| {
                        let d = PyDict::new(py);
                        let set = d
                            .set_item("loaded", p.loaded)
                            .and_then(|_| d.set_item("object", &p.object));
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

    #[pyfunction]
    #[pyo3(signature = (roots, options, depth, fetch=None, progress=None, cancel=None))]
    fn call_graph_json(
        py: Python<'_>,
        roots: Vec<String>,
        options: &str,
        depth: u32,
        fetch: Option<Py<PyAny>>,
        progress: Option<Py<PyAny>>,
        cancel: Option<&Bound<'_, CancelToken>>,
    ) -> PyResult<String> {
        let hooks = hooks(fetch, progress, cancel);
        py.detach(|| abap_analysis::json::call_graph(&roots, options, depth, hooks))
            .map_err(pyo3::exceptions::PyRuntimeError::new_err)
    }

    #[pyfunction]
    #[pyo3(signature = (reference, options, fetch=None, progress=None, cancel=None))]
    fn fetch_source(
        py: Python<'_>,
        reference: &str,
        options: &str,
        fetch: Option<Py<PyAny>>,
        progress: Option<Py<PyAny>>,
        cancel: Option<&Bound<'_, CancelToken>>,
    ) -> PyResult<Option<String>> {
        let hooks = hooks(fetch, progress, cancel);
        py.detach(|| abap_analysis::json::fetch(reference, options, hooks))
            .map_err(pyo3::exceptions::PyRuntimeError::new_err)
    }

    #[pyfunction]
    fn logic_json(py: Python<'_>, source: &str) -> String {
        py.detach(|| abap_analysis::json::logic(source))
    }

    #[pyfunction]
    #[pyo3(signature = (reference, options, depth, fetch=None, progress=None, cancel=None))]
    fn logic_object_json(
        py: Python<'_>,
        reference: &str,
        options: &str,
        depth: u32,
        fetch: Option<Py<PyAny>>,
        progress: Option<Py<PyAny>>,
        cancel: Option<&Bound<'_, CancelToken>>,
    ) -> PyResult<String> {
        let hooks = hooks(fetch, progress, cancel);
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
    #[pyo3(signature = (analysis, objects, options, depth, fetch=None, progress=None, cancel=None))]
    fn object_text(
        py: Python<'_>,
        analysis: &str,
        objects: Vec<String>,
        options: &str,
        depth: u32,
        fetch: Option<Py<PyAny>>,
        progress: Option<Py<PyAny>>,
        cancel: Option<&Bound<'_, CancelToken>>,
    ) -> PyResult<String> {
        let hooks = hooks(fetch, progress, cancel);
        py.detach(|| abap_analysis::json::object_text(analysis, &objects, options, depth, hooks))
            .map_err(pyo3::exceptions::PyRuntimeError::new_err)
    }

    #[pyfunction]
    fn version() -> &'static str {
        env!("CARGO_PKG_VERSION")
    }
}
