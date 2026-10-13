//! Analyses on parsed ABAP files.

mod calls;
mod context;
mod db;
mod logic;
mod symbols;
mod workspace;

pub use abap_source;
pub use calls::{calls, SymbolsResolver, TypeResolver};
pub use context::statement_blocks;
pub use db::{db_access, db_access_of};
pub use logic::{logic, logic_of};
pub use symbols::{symbols, symbols_of};
pub use workspace::{merge_symbols, Object, Unit, Workspace};

use abap_model as m;

/// The statements of `source` with kind, text and span.
pub fn statements(source: &str) -> Vec<m::StatementInfo> {
    let file = abap_syntax::parse(source);
    let tokens = file.tokens();
    file.statements
        .iter()
        .map(|s| {
            let first = &tokens[s.tokens[0]];
            let last = tokens[*s.tokens.last().unwrap()].end();
            m::StatementInfo {
                kind: s.kind.as_str().to_string(),
                text: file.statement_text(s),
                span: m::Span {
                    start: m::Position {
                        row: first.start.row,
                        col: first.start.col,
                    },
                    end: m::Position {
                        row: last.row,
                        col: last.col,
                    },
                },
            }
        })
        .collect()
}

/// JSON entry points shared by the language bindings.
pub mod json {
    pub fn symbols(source: &str) -> String {
        serde_json::to_string(&super::symbols(source)).expect("symbols serialize")
    }

    /// Call graph from objects like `CLAS:ZCL_X`, sources as described by a
    /// JSON [`abap_source::ProviderOptions`] object.
    pub fn call_graph(roots: &[String], options: &str, depth: u32) -> Result<String, String> {
        let workspace = workspace(options)?;
        let roots = roots
            .iter()
            .map(|r| r.parse::<abap_source::SourceRef>())
            .collect::<Result<Vec<_>, _>>()?;
        let graph = workspace.call_graph(&roots, depth);
        Ok(serde_json::to_string(&graph).expect("graph serializes"))
    }

    /// Source text of one object, `None` when no provider has it.
    pub fn fetch(reference: &str, options: &str) -> Result<Option<String>, String> {
        let r: abap_source::SourceRef = reference.parse()?;
        let provider = provider(options)?;
        provider
            .fetch(&r)
            .map(|s| s.map(|s| s.text))
            .map_err(|e| e.to_string())
    }

    fn provider(options: &str) -> Result<Box<dyn abap_source::SourceProvider>, String> {
        let options: abap_source::ProviderOptions =
            serde_json::from_str(options).map_err(|e| format!("invalid options: {e}"))?;
        options.build().map_err(|e| e.to_string())
    }

    fn workspace(options: &str) -> Result<crate::Workspace, String> {
        Ok(crate::Workspace::new(provider(options)?))
    }

    pub fn calls(source: &str) -> String {
        serde_json::to_string(&super::calls(source)).expect("calls serialize")
    }

    pub fn db_access(source: &str) -> String {
        serde_json::to_string(&super::db_access(source)).expect("db access serializes")
    }

    pub fn logic(source: &str) -> String {
        serde_json::to_string(&super::logic(source)).expect("logic serializes")
    }

    /// Logic of all sources of an object like `PROG:ZREPORT`, calls into
    /// other objects followed `depth` hops; sources as described by a JSON
    /// [`abap_source::ProviderOptions`] object.
    pub fn logic_object(reference: &str, options: &str, depth: u32) -> Result<String, String> {
        let r: abap_source::SourceRef = reference.parse()?;
        let logic = workspace(options)?.logic(&r, depth);
        Ok(serde_json::to_string(&logic).expect("logic serializes"))
    }

    pub fn statements(source: &str) -> String {
        serde_json::to_string(&super::statements(source)).expect("statements serialize")
    }
}
