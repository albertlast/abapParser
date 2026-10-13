//! Native part of the Python package. Results cross the boundary as JSON
//! strings, abap_parser/__init__.py parses them, so the shape is exactly the
//! Rust model.

use pyo3::prelude::*;

#[pymodule]
mod _native {
    use pyo3::prelude::*;

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
    fn call_graph_json(
        py: Python<'_>,
        roots: Vec<String>,
        options: &str,
        depth: u32,
    ) -> PyResult<String> {
        py.detach(|| abap_analysis::json::call_graph(&roots, options, depth))
            .map_err(pyo3::exceptions::PyRuntimeError::new_err)
    }

    #[pyfunction]
    fn fetch_source(py: Python<'_>, reference: &str, options: &str) -> PyResult<Option<String>> {
        py.detach(|| abap_analysis::json::fetch(reference, options))
            .map_err(pyo3::exceptions::PyRuntimeError::new_err)
    }

    #[pyfunction]
    fn logic_json(py: Python<'_>, source: &str) -> String {
        py.detach(|| abap_analysis::json::logic(source))
    }

    #[pyfunction]
    fn statements_json(py: Python<'_>, source: &str) -> String {
        py.detach(|| abap_analysis::json::statements(source))
    }

    #[pyfunction]
    fn version() -> &'static str {
        env!("CARGO_PKG_VERSION")
    }
}
