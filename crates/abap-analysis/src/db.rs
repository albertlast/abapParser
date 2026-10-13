//! Database access statements of a file.

use std::collections::HashMap;

use abap_model as m;
use abap_syntax::{File, StatementKind as K};

use abap_sql::hana::Dialect;

use crate::context::statement_blocks;

/// Parses `source` and lists its database accesses.
pub fn db_access(source: &str) -> Vec<m::DbAccess> {
    db_access_of(&abap_syntax::parse(source))
}

pub fn db_access_of(file: &File) -> Vec<m::DbAccess> {
    db_access_with(file, &crate::symbols_of(file))
}

/// Like [`db_access_of`], dynamic table names resolved with the constants
/// of `symbols`, e.g. those of all parts of an object.
pub(crate) fn db_access_with(file: &File, symbols: &m::FileSymbols) -> Vec<m::DbAccess> {
    let blocks = statement_blocks(file);
    let constants = literal_constants(symbols);
    let mut out = Vec::new();
    let mut i = 0;
    while i < file.statements.len() {
        let s = &file.statements[i];
        if abap_sql::is_db_statement(s.kind) {
            if let Some(mut a) = abap_sql::parse_statement(file, s) {
                a.block = blocks[i].clone();
                a.text = file.statement_text(s);
                a.span = span(file, i, i);
                resolve_dynamic(&mut a, &constants);
                out.push(a);
            }
        } else if let Some(end) = native_end(file, i) {
            // EXEC SQL ... ENDEXEC, or the body of an AMDP method: HANA SQL
            let (from, to) = native_range(file, i, end);
            let dialect = if s.kind == K::ExecSQL {
                Dialect::ExecSql
            } else {
                Dialect::SqlScript
            };
            for n in abap_sql::hana::parse(&file.source()[from..to], dialect).accesses {
                let mut a = n.access;
                a.statement = s.kind.as_str().to_string();
                a.block = blocks[i].clone();
                a.span = m::Span {
                    start: position_at(file.source(), from + n.range.start),
                    end: position_at(file.source(), from + n.range.end),
                };
                out.push(a);
            }
            i = end;
            continue;
        }
        i += 1;
    }
    out
}

/// Constants with a text literal value, by lower case name.
fn literal_constants(symbols: &m::FileSymbols) -> HashMap<String, String> {
    fn collect(out: &mut HashMap<String, String>, decls: &[m::Declaration]) {
        for d in decls {
            if d.kind != m::DeclarationKind::Constant {
                continue;
            }
            let value = d.value.as_deref().unwrap_or("");
            let literal = value
                .strip_prefix('\'')
                .and_then(|v| v.strip_suffix('\''))
                .or_else(|| value.strip_prefix('`').and_then(|v| v.strip_suffix('`')));
            if let Some(v) = literal {
                out.insert(d.name.to_ascii_lowercase(), v.trim().to_string());
            }
        }
    }
    let mut out = HashMap::new();
    collect(&mut out, &symbols.declarations);
    for c in &symbols.classes {
        collect(&mut out, &c.attributes);
        if let Some(imp) = &c.implementation {
            for mi in &imp.methods {
                collect(&mut out, &mi.locals);
            }
        }
    }
    for i in &symbols.interfaces {
        collect(&mut out, &i.attributes);
    }
    for r in symbols.forms.iter().chain(&symbols.function_modules) {
        collect(&mut out, &r.locals);
    }
    for o in &symbols.orphan_implementations {
        for mi in &o.implementation.methods {
            collect(&mut out, &mi.locals);
        }
    }
    out
}

/// `FROM (c_tabname)` with `CONSTANTS c_tabname ... VALUE 'ZTAB'`: fills
/// `resolved`, and the table of unqualified columns when it is the only table.
fn resolve_dynamic(a: &mut m::DbAccess, constants: &HashMap<String, String>) {
    for t in a.tables.iter_mut().filter(|t| t.dynamic) {
        // me->c_tab, zcl_x=>c_tab
        let name = t.name.rsplit(['>', '~']).next().unwrap_or(&t.name);
        t.resolved = constants.get(&name.to_ascii_lowercase()).cloned();
    }
    let main: Vec<_> = a
        .tables
        .iter()
        .filter(|t| matches!(t.role, m::DbTableRole::Main | m::DbTableRole::Join))
        .collect();
    let Some(only) = (main.len() == 1)
        .then(|| main[0].resolved.clone())
        .flatten()
    else {
        return;
    };
    let columns = a
        .fields
        .iter_mut()
        .flat_map(|f| f.columns.iter_mut())
        .chain(a.conditions.iter_mut().flat_map(|c| c.columns.iter_mut()))
        .chain(a.group_by.iter_mut())
        .chain(a.set.iter_mut().map(|s| &mut s.column));
    for c in columns {
        if c.table.is_none() && c.qualifier.is_none() {
            c.table = Some(only.clone());
        }
    }
}

/// Byte range of the native SQL between the statement `i` opening the
/// region and the statement `end` closing it.
pub(crate) fn native_range(file: &File, i: usize, end: usize) -> (usize, usize) {
    let tokens = file.tokens();
    let from = file.statements[i]
        .tokens
        .last()
        .map(|t| tokens[*t].span.end)
        .unwrap_or(0);
    let to = file
        .statements
        .get(end)
        .map(|e| tokens[e.tokens[0]].span.start)
        .unwrap_or(file.source().len());
    (from, to.max(from))
}

/// Row and column of a byte offset, columns counted in characters.
pub(crate) fn position_at(source: &str, offset: usize) -> m::Position {
    let before = &source[..offset.min(source.len())];
    let line_start = before.rfind('\n').map(|i| i + 1).unwrap_or(0);
    m::Position {
        row: before.matches('\n').count() as u32 + 1,
        col: before[line_start..].chars().count() as u32 + 1,
    }
}

/// End statement index of a native SQL region opened at `i`.
pub(crate) fn native_end(file: &File, i: usize) -> Option<usize> {
    let s = &file.statements[i];
    let opens = s.kind == K::ExecSQL
        || (s.kind == K::MethodImplementation
            && file
                .statements
                .get(i + 1)
                .is_some_and(|n| n.kind == K::NativeSQL));
    if !opens {
        return None;
    }
    let closer = if s.kind == K::ExecSQL {
        K::EndExec
    } else {
        K::EndMethod
    };
    Some(
        (i + 1..file.statements.len())
            .find(|k| file.statements[*k].kind == closer)
            .unwrap_or(file.statements.len()),
    )
}

fn span(file: &File, from: usize, to: usize) -> m::Span {
    let tokens = file.tokens();
    let first = &tokens[file.statements[from].tokens[0]];
    let last = tokens[*file.statements[to].tokens.last().unwrap()].end();
    m::Span {
        start: m::Position {
            row: first.start.row,
            col: first.start.col,
        },
        end: m::Position {
            row: last.row,
            col: last.col,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accesses_with_context() {
        let src = "REPORT z.\n\
START-OF-SELECTION.\n\
  SELECT * FROM t001 INTO TABLE @DATA(lt).\n\
CLASS lcl DEFINITION.\n\
  PUBLIC SECTION.\n\
    METHODS run.\n\
ENDCLASS.\n\
CLASS lcl IMPLEMENTATION.\n\
  METHOD run.\n\
    UPDATE zlog SET done = 'X' WHERE id = 1.\n\
    EXEC SQL.\n\
      SELECT a FROM ztab\n\
    ENDEXEC.\n\
  ENDMETHOD.\n\
ENDCLASS.\n\
FORM f.\n\
  DELETE FROM zlog WHERE id = 2.\n\
ENDFORM.";
        let acc = db_access(src);
        let summary: Vec<_> = acc
            .iter()
            .map(|a| {
                let b = a.block.as_ref().unwrap();
                (
                    a.operation,
                    a.tables[0].name.as_str(),
                    b.kind.as_str(),
                    b.name.as_str(),
                    b.class.as_deref(),
                )
            })
            .collect();
        assert_eq!(
            summary,
            [
                (
                    m::DbOperation::Select,
                    "t001",
                    "event",
                    "START-OF-SELECTION",
                    None
                ),
                (m::DbOperation::Update, "zlog", "method", "run", Some("lcl")),
                (m::DbOperation::Select, "ztab", "method", "run", Some("lcl")),
                (m::DbOperation::Delete, "zlog", "form", "f", None),
            ]
        );
        assert_eq!(acc[0].span.start, m::Position { row: 3, col: 1 });
        assert_eq!(acc[2].native_sql.as_deref(), Some("SELECT a FROM ztab"));
    }

    #[test]
    fn dynamic_table_from_constant() {
        let src = "CLASS lcl DEFINITION.
  PRIVATE SECTION.
    CONSTANTS c_tab TYPE tabname VALUE 'ZLOG'.
    METHODS run.
ENDCLASS.
CLASS lcl IMPLEMENTATION.
  METHOD run.
    DELETE FROM (c_tab) WHERE id = @lv_id.
  ENDMETHOD.
ENDCLASS.";
        let acc = db_access(src);
        assert_eq!(acc[0].tables[0].resolved.as_deref(), Some("ZLOG"));
        assert_eq!(
            acc[0].conditions[0].columns[0].table.as_deref(),
            Some("ZLOG")
        );
    }
}
