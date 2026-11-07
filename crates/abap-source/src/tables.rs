//! Lookups of an SAP system as queries on single tables: BAdIs, classic
//! BAdIs, class relations (where-used of interfaces, subclasses) and the
//! enhancement index. The ADT provider runs them through the data preview,
//! a [`crate::FnProvider`] through the `query` function of its host.

use serde_json::Value;

use crate::{Badi, Hook, Result};

/// Rows of a query, the values in the order of its columns.
pub type Rows = Vec<Vec<String>>;

/// A query on one table: `columns` of the rows where each column of
/// `conditions` has one of its values. Names are upper case.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Query {
    pub table: String,
    pub columns: Vec<String>,
    /// a column and the values it may have; all of them must hold
    pub conditions: Vec<(String, Vec<String>)>,
}

impl Query {
    pub fn new(table: &str, columns: &[&str]) -> Self {
        Query {
            table: table.to_ascii_uppercase(),
            columns: columns.iter().map(|c| c.to_ascii_uppercase()).collect(),
            conditions: Vec::new(),
        }
    }

    /// Rows whose `column` has one of `values`.
    pub fn filter<S: Into<String>>(
        mut self,
        column: &str,
        values: impl IntoIterator<Item = S>,
    ) -> Self {
        self.conditions.push((
            column.to_ascii_uppercase(),
            values.into_iter().map(Into::into).collect(),
        ));
        self
    }

    /// The query in ABAP SQL, names lower case, lists one value a line: the
    /// data preview takes lines up to 255 characters.
    pub fn sql(&self) -> String {
        let quote = |v: &String| format!("'{}'", v.replace('\'', "''"));
        let columns = self
            .columns
            .iter()
            .map(|c| c.to_ascii_lowercase())
            .collect::<Vec<_>>()
            .join(", ");
        let mut sql = format!("SELECT {columns} FROM {}", self.table.to_ascii_lowercase());
        for (i, (column, values)) in self.conditions.iter().enumerate() {
            sql.push_str(if i == 0 { " WHERE " } else { " AND " });
            let column = column.to_ascii_lowercase();
            match values.as_slice() {
                [one] => sql.push_str(&format!("{column} = {}", quote(one))),
                _ => {
                    let list = values.iter().map(quote).collect::<Vec<_>>().join(",\n");
                    sql.push_str(&format!("{column} IN ( {list} )"));
                }
            }
        }
        sql
    }

    /// What the `query` function of a host gets: `{table, columns, where,
    /// sql}`, `where` like `{"BADI_NAME": ["ZBADI"]}`.
    pub fn to_json(&self) -> Value {
        let conditions: serde_json::Map<String, Value> = self
            .conditions
            .iter()
            .map(|(c, v)| (c.clone(), Value::from(v.clone())))
            .collect();
        serde_json::json!({
            "table": self.table,
            "columns": self.columns,
            "where": conditions,
            "sql": self.sql(),
        })
    }

    /// The rows of an answer of a host: `null` when it cannot answer, else
    /// a list of rows, each a list of values in the order of the columns or
    /// an object with the columns as keys (any case). Numbers and booleans
    /// become text, `null` an empty value.
    pub fn rows_of(&self, answer: &Value) -> std::result::Result<Option<Rows>, String> {
        let rows = match answer {
            Value::Null => return Ok(None),
            Value::Array(rows) => rows,
            other => {
                return Err(format!(
                    "expected a list of rows or null, got {}",
                    kind(other)
                ))
            }
        };
        let text = |v: Option<&Value>| match v {
            None | Some(Value::Null) => Ok(String::new()),
            Some(Value::String(s)) => Ok(s.trim().to_string()),
            Some(Value::Number(n)) => Ok(n.to_string()),
            Some(Value::Bool(b)) => Ok(if *b { "X" } else { "" }.to_string()),
            Some(other) => Err(format!("expected a value, got {}", kind(other))),
        };
        let mut out = Vec::new();
        for row in rows {
            out.push(match row {
                Value::Array(values) => (0..self.columns.len())
                    .map(|i| text(values.get(i)))
                    .collect::<std::result::Result<Vec<_>, _>>()?,
                Value::Object(fields) => self
                    .columns
                    .iter()
                    .map(|c| {
                        text(
                            fields
                                .iter()
                                .find(|(k, _)| k.eq_ignore_ascii_case(c))
                                .map(|(_, v)| v),
                        )
                    })
                    .collect::<std::result::Result<Vec<_>, _>>()?,
                other => return Err(format!("expected a row, got {}", kind(other))),
            });
        }
        Ok(Some(out))
    }
}

fn kind(v: &Value) -> &'static str {
    match v {
        Value::Null => "null",
        Value::Bool(_) => "a boolean",
        Value::Number(_) => "a number",
        Value::String(_) => "a string",
        Value::Array(_) => "a list",
        Value::Object(_) => "an object",
    }
}

/// Runs a query for a lookup, like `BAdI definitions`; `None` when it
/// cannot be answered, which the runner notes.
pub(crate) type Select<'a> = &'a dyn Fn(&str, &Query) -> Result<Option<Rows>>;

/// Whether a name may go into SQL: only what ABAP names consist of.
pub(crate) fn is_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '/')
}

/// Whether a program name may go into SQL: ABAP names, class pools and
/// their includes.
pub(crate) fn is_program(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '/' | '='))
}

/// The values of the first column, upper case, sorted, without empty ones.
pub(crate) fn first_column(rows: Rows) -> Vec<String> {
    let mut out: Vec<String> = rows
        .into_iter()
        .filter_map(|r| non_empty(r.first()))
        .collect();
    out.sort();
    out.dedup();
    out
}

pub(crate) fn non_empty(value: Option<&String>) -> Option<String> {
    value
        .map(|v| v.trim().to_ascii_uppercase())
        .filter(|v| !v.is_empty())
}

/// Relation types of the class relations, SEOMETAREL-RELTYPE.
#[derive(Clone, Copy)]
pub(crate) enum Relation {
    Implements = 1,
    Inherits = 2,
}

/// Classes with a relation to `name` in the class relations (SEOMETAREL);
/// `None` when the query cannot be answered.
pub(crate) fn related_classes(
    select: Select,
    name: &str,
    relation: Relation,
) -> Result<Option<Vec<String>>> {
    let key = name.to_ascii_uppercase();
    if !is_name(&key) {
        return Ok(Some(Vec::new()));
    }
    let query = Query::new("SEOMETAREL", &["CLSNAME"])
        .filter("REFCLSNAME", [key])
        .filter("RELTYPE", [(relation as u8).to_string()])
        .filter("VERSION", ["1"]);
    Ok(select("class relations", &query)?.map(first_column))
}

/// A BAdI of an enhancement spot: interface and fallback class from its
/// definition, the classes of its active implementations.
pub(crate) fn spot_badi(select: Select, name: &str) -> Result<Option<Badi>> {
    if !is_name(name) {
        return Ok(None);
    }
    let query =
        Query::new("BADI_MAIN", &["INTERFACE_NAME", "DEFAULT_CLASS"]).filter("BADI_NAME", [name]);
    let definition = select("BAdI definitions", &query)?.unwrap_or_default();
    let Some(row) = definition.first() else {
        return Ok(None);
    };
    Ok(Some(Badi {
        name: name.to_string(),
        interface: non_empty(row.first()),
        implementations: badi_implementations(select, name)?,
        fallback: non_empty(row.get(1)),
        classic: false,
    }))
}

/// Classes of the active implementations of a BAdI of an enhancement
/// spot: the runtime table only has those.
pub(crate) fn badi_implementations(select: Select, name: &str) -> Result<Vec<String>> {
    let query = Query::new("BADI_IMPL", &["CLASS_NAME"]).filter("BADI_NAME", [name]);
    Ok(select("BAdI implementations", &query)?
        .map(first_column)
        .unwrap_or_default())
}

/// The definition of a classic BAdI by its interface, as rows of exit
/// name, BAdI class and the BAdI it was migrated to.
pub(crate) fn exit_definition(select: Select, interface: &str) -> Result<Option<Rows>> {
    let query = Query::new("SXS_INTER", &["EXIT_NAME"]).filter("INTER_NAME", [interface]);
    let Some(exits) = select("classic BAdI definitions", &query)? else {
        return Ok(None);
    };
    let Some(exit) = first_column(exits).into_iter().find(|e| is_name(e)) else {
        return Ok(Some(Vec::new()));
    };
    let query = Query::new("SXS_ATTR", &["EXIT_NAME", "DEF_CLNAME", "MIG_BADI_NAME"])
        .filter("EXIT_NAME", [exit]);
    select("classic BAdI definitions", &query)
}

/// The classes of the active implementations of a classic BAdI, as rows.
pub(crate) fn exit_implementations(select: Select, interface: &str) -> Result<Option<Rows>> {
    let query =
        Query::new("SXC_CLASS", &["IMP_NAME", "IMP_CLASS"]).filter("INTER_NAME", [interface]);
    let Some(classes) = select("classic BAdI implementations", &query)? else {
        return Ok(None);
    };
    let classes: Vec<(String, String)> = classes
        .iter()
        .filter_map(|row| Some((non_empty(row.first())?, non_empty(row.get(1))?)))
        .filter(|(imp, _)| is_name(imp))
        .collect();
    if classes.is_empty() {
        return Ok(Some(Vec::new()));
    }
    let query = Query::new("SXC_ATTR", &["IMP_NAME"])
        .filter("IMP_NAME", classes.iter().map(|(imp, _)| imp.clone()))
        .filter("ACTIVE", ["X"]);
    let Some(active) = select("classic BAdI implementations", &query)? else {
        return Ok(None);
    };
    let active = first_column(active);
    Ok(Some(
        classes
            .into_iter()
            .filter(|(imp, _)| active.contains(imp))
            .map(|(_, class)| vec![class])
            .collect(),
    ))
}

/// A classic BAdI from its definition rows (exit name, BAdI class,
/// migrated BAdI) and the classes of its implementations; those of the
/// BAdI it was migrated to count too.
pub(crate) fn exit_badi(
    interface: &str,
    definition: Rows,
    mut implementations: Vec<String>,
    migrated: impl FnOnce(&str) -> Result<Vec<String>>,
) -> Result<Option<Badi>> {
    let Some(row) = definition.first() else {
        return Ok(None);
    };
    if let Some(m) = non_empty(row.get(2)).filter(|m| is_name(m)) {
        implementations.extend(migrated(&m)?);
        implementations.sort();
        implementations.dedup();
    }
    Ok(Some(Badi {
        name: non_empty(row.first()).unwrap_or_default(),
        interface: Some(interface.to_string()),
        implementations,
        fallback: non_empty(row.get(1)),
        classic: true,
    }))
}

/// A classic BAdI by its interface, through single-table queries.
pub(crate) fn classic_badi(select: Select, interface: &str) -> Result<Option<Badi>> {
    if !is_name(interface) {
        return Ok(None);
    }
    let Some(definition) = exit_definition(select, interface)? else {
        return Ok(None);
    };
    if definition.is_empty() {
        return Ok(None);
    }
    let implementations = exit_implementations(select, interface)?
        .map(first_column)
        .unwrap_or_default();
    exit_badi(interface, definition, implementations, |m| {
        badi_implementations(select, m)
    })
}

/// The query of the enhancement index for some programs. FULL_NAME is a
/// long string and not allowed in a condition on SAP_BASIS 750, so it is
/// only read.
pub(crate) fn enhancement_query(programs: &[String]) -> Query {
    Query::new(
        "ENHINCINX",
        &["ENHNAME", "ID", "ENHMODE", "FULL_NAME", "PROGRAMNAME"],
    )
    .filter("VERSION", ["A"])
    .filter("PROGRAMNAME", programs.iter().cloned())
}

/// The main program of a function group, `SAPL<group>` or `/NS/SAPL<group>`.
pub(crate) fn group_program(group: &str) -> String {
    match group.strip_prefix('/').and_then(|g| g.split_once('/')) {
        Some((ns, name)) => format!("/{ns}/SAPL{name}").to_ascii_uppercase(),
        None => format!("SAPL{group}").to_ascii_uppercase(),
    }
}

/// The code of an enhancement implementation, where it is from and
/// whether it is switched off.
#[derive(Clone)]
pub(crate) struct HookSource {
    pub text: String,
    pub origin: String,
    pub switched_off: bool,
}

/// The block `ENHANCEMENT <id> ...` to `ENDENHANCEMENT.` of the code of an
/// enhancement implementation.
pub(crate) fn hook_block(source: &str, id: &str) -> Option<String> {
    let mut out: Option<Vec<&str>> = None;
    for line in source.lines() {
        let words: Vec<String> = line
            .split(|c: char| c.is_whitespace() || c == '.')
            .filter(|w| !w.is_empty())
            .map(|w| w.to_ascii_uppercase())
            .collect();
        match &mut out {
            None => {
                if words.first().is_some_and(|w| w == "ENHANCEMENT")
                    && words.get(1).map(String::as_str) == Some(id)
                {
                    out = Some(vec![line]);
                }
            }
            Some(lines) => {
                lines.push(line);
                if words.first().is_some_and(|w| w == "ENDENHANCEMENT") {
                    return Some(lines.join("\n"));
                }
            }
        }
    }
    None
}

/// The hooks of rows of the enhancement index (name, id, mode, position,
/// program) in the programs `named`, or with a position starting with one
/// of `prefixes` (`\FU:<module>\`), with the code `source` gives for each
/// enhancement implementation. One it does not give does not stop the
/// others.
pub(crate) fn hooks_of(
    rows: Rows,
    named: &[String],
    prefixes: &[String],
    source: impl Fn(&str) -> Result<Option<HookSource>>,
) -> Result<Vec<Hook>> {
    let mut out: Vec<Hook> = Vec::new();
    for row in rows {
        let [name, id, mode, position, program] = &row[..] else {
            continue;
        };
        let position = position.trim();
        let upper = position.to_ascii_uppercase();
        let wanted = named.iter().any(|n| n.eq_ignore_ascii_case(program.trim()))
            || prefixes.iter().any(|p| upper.starts_with(p.as_str()));
        let name = name.trim().to_ascii_uppercase();
        // enhancements of a function module's parameters are no code
        if !wanted
            || !is_name(&name)
            || position.matches(r"\SE:").count() > 1
            || out
                .iter()
                .any(|h| h.enhancement == name && h.position == position)
        {
            continue;
        }
        let source = match source(&name) {
            Ok(Some(source)) => source,
            Ok(None) | Err(crate::SourceError::Http { .. }) => continue,
            Err(e) => return Err(e),
        };
        // switched off, only the static parts stay, like declarations
        if source.switched_off && mode.trim() == "D" {
            continue;
        }
        if let Some(code) = hook_block(&source.text, id.trim()) {
            out.push(Hook {
                enhancement: name,
                position: position.to_string(),
                code,
                origin: source.origin,
            });
        }
    }
    Ok(out)
}

/// Hooks of the programs and function modules through single-table
/// queries: the programs of the function modules from TFDIR, the hooks
/// from the enhancement index.
pub(crate) fn enhancements(
    select: Select,
    programs: &[String],
    function_modules: &[String],
    source: impl Fn(&str) -> Result<Option<HookSource>>,
) -> Result<Vec<Hook>> {
    let mut wanted: Vec<String> = programs
        .iter()
        .filter(|p| is_program(p))
        .map(|p| p.to_ascii_uppercase())
        .collect();
    let named = wanted.clone();
    let modules: Vec<String> = function_modules
        .iter()
        .filter(|m| is_name(m))
        .map(|m| m.to_ascii_uppercase())
        .collect();
    let mut prefixes: Vec<String> = Vec::new();
    if !modules.is_empty() {
        let query = Query::new("TFDIR", &["FUNCNAME", "PNAME"]).filter("FUNCNAME", modules);
        for row in select("enhancements", &query)?.unwrap_or_default() {
            if let (Some(module), Some(program)) = (non_empty(row.first()), non_empty(row.get(1))) {
                wanted.push(program);
                prefixes.push(format!("\\FU:{module}\\"));
            }
        }
    }
    wanted.sort();
    wanted.dedup();
    let mut rows = Vec::new();
    for chunk in wanted.chunks(50) {
        match select("enhancements", &enhancement_query(chunk))? {
            Some(r) => rows.extend(r),
            None => return Ok(Vec::new()),
        }
    }
    hooks_of(rows, &named, &prefixes, source)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn queries_in_sql_and_json() {
        let q = Query::new("seometarel", &["clsname"])
            .filter("refclsname", ["ZIF_X"])
            .filter("reltype", ["1"]);
        assert_eq!(
            q.sql(),
            "SELECT clsname FROM seometarel WHERE refclsname = 'ZIF_X' AND reltype = '1'"
        );
        assert_eq!(
            q.to_json(),
            serde_json::json!({
                "table": "SEOMETAREL",
                "columns": ["CLSNAME"],
                "where": {"REFCLSNAME": ["ZIF_X"], "RELTYPE": ["1"]},
                "sql": q.sql(),
            })
        );
        let q = enhancement_query(&["SAPLZGRP".to_string(), "ZPROG".to_string()]);
        let condition = q.sql().split("WHERE").nth(1).unwrap().to_string();
        assert!(!condition.to_ascii_lowercase().contains("full_name"));
        assert!(q.sql().lines().all(|l| l.len() < 255));
        assert!(q.sql().contains("programname IN ( 'SAPLZGRP',\n'ZPROG' )"));
        assert_eq!(
            Query::new("T", &["A"]).filter("B", ["it's"]).sql(),
            "SELECT a FROM t WHERE b = 'it''s'"
        );
        assert_eq!(group_program("ZGRP"), "SAPLZGRP");
        assert_eq!(group_program("/abc/grp"), "/ABC/SAPLGRP");
    }

    #[test]
    fn answers_of_a_host() {
        let q = Query::new("SXS_ATTR", &["EXIT_NAME", "DEF_CLNAME", "MIG_BADI_NAME"]);
        let rows = |v: Value| q.rows_of(&v);
        assert_eq!(rows(Value::Null), Ok(None));
        assert_eq!(
            rows(serde_json::json!([["ZEX", "ZCL_DEF"], [" ZEX2 ", null, 3]])),
            Ok(Some(vec![
                vec!["ZEX".to_string(), "ZCL_DEF".to_string(), String::new()],
                vec!["ZEX2".to_string(), String::new(), "3".to_string()],
            ]))
        );
        assert_eq!(
            rows(serde_json::json!([{"exit_name": "ZEX", "MIG_BADI_NAME": "ZB"}])),
            Ok(Some(vec![vec![
                "ZEX".to_string(),
                String::new(),
                "ZB".to_string()
            ]]))
        );
        assert!(rows(serde_json::json!("ZEX")).is_err());
        assert!(rows(serde_json::json!([["ZEX", ["x"]]])).is_err());
    }

    #[test]
    fn blocks_of_enhancement_code() {
        let source = "ENHANCEMENT 1  .\r\n  x = 1.\r\nENDENHANCEMENT.\r\nENHANCEMENT 12  .\r\n  y = 2.\r\nENDENHANCEMENT.";
        assert_eq!(
            hook_block(source, "12").as_deref(),
            Some("ENHANCEMENT 12  .\n  y = 2.\nENDENHANCEMENT.")
        );
        assert_eq!(
            hook_block(source, "1").as_deref(),
            Some("ENHANCEMENT 1  .\n  x = 1.\nENDENHANCEMENT.")
        );
        assert_eq!(hook_block(source, "2"), None);
    }
}
