//! Native part of the Python package. Results cross the boundary as JSON
//! strings, abap_parser/__init__.py parses them, so the shape is exactly the
//! Rust model.
//!
//! Functions loading sources take a `fetch` callable of the host; the
//! native code runs without the GIL and takes it to call it.

use pyo3::prelude::*;

#[pymodule]
mod _native {
    use abap_analysis::abap_source::{FnProvider, SourceError};
    use abap_analysis::json::Host;
    use pyo3::prelude::*;

    /// `fetch(reference) -> str | None` as a source provider.
    fn host(fetch: Option<Py<PyAny>>) -> Host {
        let fetch = fetch?;
        Some(Box::new(FnProvider::new("fetch", move |r| {
            Python::attach(|py| {
                fetch
                    .call1(py, (r.to_string(),))
                    .and_then(|v| v.bind(py).extract::<Option<String>>())
            })
            .map_err(|e| SourceError::Other(format!("fetch {r}: {e}")))
        })))
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
    #[pyo3(signature = (roots, options, depth, fetch=None))]
    fn call_graph_json(
        py: Python<'_>,
        roots: Vec<String>,
        options: &str,
        depth: u32,
        fetch: Option<Py<PyAny>>,
    ) -> PyResult<String> {
        let host = host(fetch);
        py.detach(|| abap_analysis::json::call_graph(&roots, options, depth, host))
            .map_err(pyo3::exceptions::PyRuntimeError::new_err)
    }

    #[pyfunction]
    #[pyo3(signature = (reference, options, fetch=None))]
    fn fetch_source(
        py: Python<'_>,
        reference: &str,
        options: &str,
        fetch: Option<Py<PyAny>>,
    ) -> PyResult<Option<String>> {
        let host = host(fetch);
        py.detach(|| abap_analysis::json::fetch(reference, options, host))
            .map_err(pyo3::exceptions::PyRuntimeError::new_err)
    }

    #[pyfunction]
    fn logic_json(py: Python<'_>, source: &str) -> String {
        py.detach(|| abap_analysis::json::logic(source))
    }

    #[pyfunction]
    #[pyo3(signature = (reference, options, depth, fetch=None))]
    fn logic_object_json(
        py: Python<'_>,
        reference: &str,
        options: &str,
        depth: u32,
        fetch: Option<Py<PyAny>>,
    ) -> PyResult<String> {
        let host = host(fetch);
        py.detach(|| abap_analysis::json::logic_object(reference, options, depth, host))
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

    #[pyfunction]
    #[pyo3(signature = (analysis, objects, options, depth, fetch=None))]
    fn object_text(
        py: Python<'_>,
        analysis: &str,
        objects: Vec<String>,
        options: &str,
        depth: u32,
        fetch: Option<Py<PyAny>>,
    ) -> PyResult<String> {
        let host = host(fetch);
        py.detach(|| abap_analysis::json::object_text(analysis, &objects, options, depth, host))
            .map_err(pyo3::exceptions::PyRuntimeError::new_err)
    }

    #[pyfunction]
    fn version() -> &'static str {
        env!("CARGO_PKG_VERSION")
    }
}
