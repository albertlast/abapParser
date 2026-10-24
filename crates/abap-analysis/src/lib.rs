//! Analyses on parsed ABAP files.

mod args;
mod calls;
mod context;
mod db;
mod ddic;
mod enhance;
mod hr;
mod logic;
mod stores;
mod symbols;
mod types;
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
pub use workspace::{merge_symbols, Follow, Object, Progress, ProgressFn, Unit, Workspace};

use abap_model as m;

/// The statements of `source` with kind, text and span, the release each
/// needs and syntax errors found.
pub fn statements(source: &str) -> Vec<m::StatementInfo> {
    let file = abap_syntax::parse(source);
    file.statements
        .iter()
        .map(|s| statement_info(&file, s))
        .collect()
}

fn statement_info(file: &abap_syntax::File, s: &abap_syntax::Statement) -> m::StatementInfo {
    let tokens = file.tokens();
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
        release: abap_syntax::release::statement_release(file, s).map(release_need),
        error: abap_syntax::check::syntax_error(file, s).map(|e| e.message.to_string()),
    }
}

fn release_need(n: abap_syntax::release::Need) -> m::ReleaseNeed {
    m::ReleaseNeed {
        release: n.release.name().to_string(),
        id: n.release.id().to_string(),
        construct: n.construct.to_string(),
    }
}

/// The release `source` needs, the statements that need a newer one than
/// `target` (7.00 when `None`) and the statements with a syntax error.
pub fn release(source: &str, target: Option<abap_syntax::release::Release>) -> m::ReleaseReport {
    use abap_syntax::release::{file_release, statement_release, Release};
    let file = abap_syntax::parse(source);
    let target_release = target.unwrap_or(Release::V700);
    let mut statements = Vec::new();
    let mut errors = Vec::new();
    for s in &file.statements {
        let newer = statement_release(&file, s).is_some_and(|n| n.release > target_release);
        let error = abap_syntax::check::syntax_error(&file, s).is_some();
        if newer || error {
            let info = statement_info(&file, s);
            if newer {
                statements.push(info.clone());
            }
            if error {
                errors.push(info);
            }
        }
    }
    m::ReleaseReport {
        schema_version: m::SCHEMA_VERSION,
        release: file_release(&file).map(|(_, n)| release_need(n)),
        target: target
            .filter(|t| *t != Release::V700)
            .map(|t| t.name().to_string()),
        statements,
        errors,
    }
}

/// JSON entry points shared by the language bindings. Those loading
/// sources take the options as JSON: the [`abap_source::ProviderOptions`],
/// and under `follow` the [`crate::Follow`] patterns and limit; and the
/// [`Hooks`] of the caller.
pub mod json {
    use std::sync::atomic::AtomicBool;
    use std::sync::Arc;

    /// What the caller adds to an analysis loading sources.
    #[derive(Default)]
    pub struct Hooks {
        /// a provider of the caller, like a [`abap_source::FnProvider`]
        /// calling back into Node.js or Python, asked after the folders and
        /// before the SAP system
        pub host: Option<Box<dyn abap_source::SourceProvider>>,
        /// called after each object loaded or looked for
        pub progress: Option<crate::ProgressFn>,
        /// once set, nothing more is loaded
        pub cancel: Option<Arc<AtomicBool>>,
    }

    pub fn symbols(source: &str) -> String {
        serde_json::to_string(&super::symbols(source)).expect("symbols serialize")
    }

    /// Call graph from objects like `CLAS:ZCL_X`, sources as described by a
    /// JSON [`abap_source::ProviderOptions`] object.
    pub fn call_graph(
        roots: &[String],
        options: &str,
        depth: u32,
        hooks: Hooks,
    ) -> Result<String, String> {
        let workspace = workspace(options, hooks)?;
        let roots = roots
            .iter()
            .map(|r| r.parse::<abap_source::SourceRef>())
            .collect::<Result<Vec<_>, _>>()?;
        let graph = workspace.call_graph(&roots, depth);
        Ok(serde_json::to_string(&graph).expect("graph serializes"))
    }

    /// Source text of one object, `None` when no provider has it.
    pub fn fetch(reference: &str, options: &str, hooks: Hooks) -> Result<Option<String>, String> {
        let r: abap_source::SourceRef = reference.parse()?;
        let (options, _) = split(options)?;
        let provider = options.build_with(hooks.host).map_err(|e| e.to_string())?;
        provider
            .fetch(&r)
            .map(|s| s.map(|s| s.text))
            .map_err(|e| e.to_string())
    }

    /// The source options and the follow options of a JSON object.
    fn split(options: &str) -> Result<(abap_source::ProviderOptions, crate::Follow), String> {
        let mut value: serde_json::Value =
            serde_json::from_str(options).map_err(|e| format!("invalid options: {e}"))?;
        let follow = match value.as_object_mut().and_then(|o| o.remove("follow")) {
            Some(f) => serde_json::from_value(f).map_err(|e| format!("invalid follow: {e}"))?,
            None => crate::Follow::default(),
        };
        let sources = serde_json::from_value(value).map_err(|e| format!("invalid options: {e}"))?;
        Ok((sources, follow))
    }

    fn workspace(options: &str, hooks: Hooks) -> Result<crate::Workspace, String> {
        let (sources, follow) = split(options)?;
        let provider = sources.build_with(hooks.host).map_err(|e| e.to_string())?;
        let mut workspace = crate::Workspace::new(provider).with_follow(follow);
        if let Some(p) = hooks.progress {
            workspace = workspace.with_progress(p);
        }
        if let Some(c) = hooks.cancel {
            workspace = workspace.with_cancel(c);
        }
        Ok(workspace)
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
    pub fn logic_object(
        reference: &str,
        options: &str,
        depth: u32,
        hooks: Hooks,
    ) -> Result<String, String> {
        let r: abap_source::SourceRef = reference.parse()?;
        let logic = workspace(options, hooks)?.logic(&r, depth);
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
        hooks: Hooks,
    ) -> Result<String, String> {
        use crate::text;
        let objects = objects
            .iter()
            .map(|r| r.parse::<abap_source::SourceRef>())
            .collect::<Result<Vec<_>, _>>()?;
        let first = objects.first().ok_or("no object given")?;
        let workspace = workspace(options, hooks)?;
        match analysis {
            "symbols" if first.kind == abap_source::ObjectKind::Ddls => workspace
                .unit(first)
                .map(|u| text::cds(&super::cds(u.file.source())))
                .ok_or_else(|| format!("{first} not found")),
            "symbols" => workspace
                .root(first)
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
