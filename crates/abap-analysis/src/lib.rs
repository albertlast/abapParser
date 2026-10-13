//! Analyses on parsed ABAP files.

mod args;
mod calls;
mod context;
mod db;
mod ddic;
mod enhance;
mod logic;
mod stores;
mod symbols;
mod workspace;

pub use abap_source;
pub use calls::{calls, SymbolsResolver, TypeResolver};
pub use context::statement_blocks;
pub use db::{db_access, db_access_of};
pub use logic::{logic, logic_of};
pub use symbols::{symbols, symbols_of};

/// A CDS data definition: its sources, joins, associations, parameters and
/// the fields each element is made of.
pub fn cds(source: &str) -> m::CdsDefinition {
    abap_sql::cds::parse(source)
}
pub mod text;
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

    pub fn cds(source: &str) -> String {
        serde_json::to_string(&super::cds(source)).expect("cds serializes")
    }

    /// Compact text of one source, see [`crate::text`]: `symbols`, `calls`,
    /// `db` or `logic`.
    pub fn text(analysis: &str, source: &str) -> Result<String, String> {
        use crate::text;
        Ok(match analysis {
            "symbols" => text::symbols(&super::symbols(source)),
            "calls" => text::calls(&super::calls(source)),
            "db" => text::db_access(&super::db_access(source)),
            "logic" => text::logic(&super::logic(source)),
            "cds" => text::cds(&super::cds(source)),
            other => {
                return Err(format!(
                    "unknown analysis {other:?}, expected symbols, calls, db, logic or cds"
                ))
            }
        })
    }

    /// Compact text of objects like `CLAS:ZCL_X`, sources as described by a
    /// JSON [`abap_source::ProviderOptions`] object: `symbols` of all
    /// sources of the first object, its `logic` with calls followed `depth`
    /// hops, or the call `graph` from all of them.
    pub fn object_text(
        analysis: &str,
        objects: &[String],
        options: &str,
        depth: u32,
    ) -> Result<String, String> {
        use crate::text;
        let objects = objects
            .iter()
            .map(|r| r.parse::<abap_source::SourceRef>())
            .collect::<Result<Vec<_>, _>>()?;
        let first = objects.first().ok_or("no object given")?;
        let workspace = workspace(options)?;
        match analysis {
            "symbols" if first.kind == abap_source::ObjectKind::Ddls => workspace
                .unit(first)
                .map(|u| text::cds(&super::cds(u.file.source())))
                .ok_or_else(|| format!("{first} not found")),
            "symbols" => workspace
                .object(first)
                .map(|o| text::symbols(&o.symbols))
                .ok_or_else(|| format!("{first} not found")),
            "logic" => Ok(text::logic(&workspace.logic(first, depth))),
            "graph" => Ok(text::call_graph(&workspace.call_graph(&objects, depth))),
            other => Err(format!(
                "unknown analysis {other:?}, expected symbols, logic or graph"
            )),
        }
    }
}
