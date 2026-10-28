//! The system tables the ADT provider reads through the data preview, filled
//! from the folders the mock serves (table types from their `.ttyp.xml`), and the part of ABAP SQL it sends:
//! `SELECT` columns `FROM` a table, at most one `INNER JOIN`, a `WHERE` of
//! `=`, `IN` and `LIKE` conditions joined by `AND`.

use std::collections::HashMap;

use crate::{class_pool, FsProvider, ObjectKind, SourceProvider};

struct Table {
    columns: &'static [&'static str],
    /// long strings (type STRG): ABAP SQL does not allow them in `WHERE`
    long: &'static [&'static str],
    rows: Vec<Vec<String>>,
}

impl Table {
    fn new(columns: &'static [&'static str]) -> Self {
        Table {
            columns,
            long: &[],
            rows: Vec::new(),
        }
    }

    fn column(&self, name: &str) -> Option<usize> {
        self.columns.iter().position(|c| *c == name)
    }
}

pub(super) struct Tables(HashMap<&'static str, Table>);

/// The answer to a query: column names and rows, or why it is rejected.
pub(super) type Answer = std::result::Result<(Vec<String>, Vec<Vec<String>>), String>;

impl Tables {
    /// The class relations, BAdIs and enhancements of the served folders;
    /// `groups` maps function modules to their group.
    pub(super) fn of(provider: &FsProvider, groups: &HashMap<String, String>) -> Self {
        let mut relations = Table::new(&["CLSNAME", "REFCLSNAME", "RELTYPE", "VERSION"]);
        let r = provider.relations();
        for (kind, map) in [("1", &r.implementers), ("2", &r.subclasses)] {
            for (target, classes) in map {
                for class in classes {
                    relations.rows.push(row(&[class, target, kind, "1"]));
                }
            }
        }

        let mut badi_main = Table::new(&["BADI_NAME", "INTERFACE_NAME", "DEFAULT_CLASS"]);
        let mut badi_impl = Table::new(&["BADI_NAME", "CLASS_NAME"]);
        let mut exits = Table::new(&["EXIT_NAME", "INTER_NAME"]);
        let mut exit_attrs = Table::new(&["EXIT_NAME", "DEF_CLNAME", "MIG_BADI_NAME"]);
        let mut exit_classes = Table::new(&["IMP_NAME", "INTER_NAME", "IMP_CLASS"]);
        let mut exit_impls = Table::new(&["IMP_NAME", "ACTIVE"]);
        for b in provider.badis().all() {
            let interface = b.interface.clone().unwrap_or_default();
            let fallback = b.fallback.clone().unwrap_or_default();
            if !b.classic {
                badi_main.rows.push(row(&[&b.name, &interface, &fallback]));
                for class in &b.implementations {
                    badi_impl.rows.push(row(&[&b.name, class]));
                }
                continue;
            }
            if !b.name.is_empty() {
                exits.rows.push(row(&[&b.name, &interface]));
                exit_attrs.rows.push(row(&[&b.name, &fallback, ""]));
            }
            // only active implementations are known; each is named after
            // its class
            for class in &b.implementations {
                exit_classes.rows.push(row(&[class, &interface, class]));
                exit_impls.rows.push(row(&[class, "X"]));
            }
        }

        let mut hooks = Table {
            long: &["FULL_NAME"],
            ..Table::new(&[
                "ENHNAME",
                "ID",
                "ENHMODE",
                "FULL_NAME",
                "PROGRAMNAME",
                "VERSION",
            ])
        };
        for h in provider.hooks() {
            let id = h
                .code
                .split_whitespace()
                .nth(1)
                .map(|w| w.trim_end_matches('.'))
                .unwrap_or_default();
            let Some(program) = hook_program(&h.position, groups) else {
                continue;
            };
            hooks
                .rows
                .push(row(&[&h.enhancement, id, "D", &h.position, &program, "A"]));
        }

        let columns = ["ROWTYPE", "ROWKIND", "DATATYPE", "LENG", "DECIMALS"];
        let mut table_types = Table::new(&[
            "TYPENAME", "ROWTYPE", "ROWKIND", "DATATYPE", "LENG", "DECIMALS", "AS4LOCAL",
        ]);
        for r in provider.references().filter(|r| r.kind == ObjectKind::Ttyp) {
            let Ok(Some(source)) = provider.fetch(r) else {
                continue;
            };
            let mut values = vec![r.name.clone()];
            values.extend(columns.iter().map(|c| tag(&source.text, c)));
            values.push("A".to_string());
            table_types.rows.push(values);
        }

        let mut tables = HashMap::new();
        for (name, table) in [
            ("SEOMETAREL", relations),
            ("BADI_MAIN", badi_main),
            ("BADI_IMPL", badi_impl),
            ("SXS_INTER", exits),
            ("SXS_ATTR", exit_attrs),
            ("SXC_CLASS", exit_classes),
            ("SXC_ATTR", exit_impls),
            ("ENHINCINX", hooks),
            ("DD40L", table_types),
        ] {
            tables.insert(name, table);
        }
        Tables(tables)
    }

    /// Runs a query, at most `max` rows. With `only`, the query must read
    /// that one table, as the table preview of a dictionary object does.
    pub(super) fn select(&self, sql: &str, only: Option<&str>, max: usize) -> Answer {
        if sql.lines().any(|l| l.len() > 255) {
            return Err("A line of the statement is longer than 255 characters".into());
        }
        let query = Query::parse(&tokens(sql))?;
        if let Some(entity) = only {
            if query.sources.len() > 1 || !query.sources[0].0.eq_ignore_ascii_case(entity) {
                return Err(format!("Only a SELECT on {entity} is allowed"));
            }
        }
        let mut sources: Vec<(&str, &Table)> = Vec::new();
        for (name, alias) in &query.sources {
            let table = self.0.get(name.as_str()).ok_or_else(|| {
                format!("\"{name}\" is not defined in the ABAP Dictionary as a table, projection view, or database view")
            })?;
            sources.push((alias, table));
        }
        let resolve = |name: &str| -> std::result::Result<(usize, usize), String> {
            let (alias, column) = match name.split_once('~') {
                Some((a, c)) => (Some(a), c),
                None => (None, name),
            };
            sources
                .iter()
                .enumerate()
                .filter(|(_, (a, _))| alias.is_none_or(|x| x == *a))
                .find_map(|(i, (_, t))| t.column(column).map(|c| (i, c)))
                .ok_or_else(|| format!("The column \"{name}\" is unknown"))
        };
        for c in &query.conditions {
            let (s, column) = resolve(&c.column)?;
            let table = sources[s].1;
            let name = table.columns[column];
            if table.long.contains(&name) {
                return Err(format!(
                    "The field {name} is a long string or a text and cannot be used in the WHERE condition"
                ));
            }
        }

        // the rows of the FROM part: one table, or the pairs of the join
        let mut combined: Vec<Vec<&Vec<String>>> =
            sources[0].1.rows.iter().map(|r| vec![r]).collect();
        if let Some((left, right)) = &query.join {
            let (l, r) = (resolve(left)?, resolve(right)?);
            let mut joined = Vec::new();
            for first in &combined {
                for second in &sources[1].1.rows {
                    let candidate = vec![first[0], second];
                    if candidate[l.0][l.1] == candidate[r.0][r.1] {
                        joined.push(candidate);
                    }
                }
            }
            combined = joined;
        }
        let mut conditions = Vec::new();
        for c in &query.conditions {
            conditions.push((resolve(&c.column)?, &c.test));
        }
        combined.retain(|r| {
            conditions
                .iter()
                .all(|((s, c), test)| test.matches(&r[*s][*c]))
        });

        let mut picked = Vec::new();
        for column in &query.columns {
            if column == "*" {
                for (s, (_, t)) in sources.iter().enumerate() {
                    picked.extend((0..t.columns.len()).map(|c| (s, c)));
                }
            } else {
                picked.push(resolve(column)?);
            }
        }
        let names = picked
            .iter()
            .map(|(s, c)| sources[*s].1.columns[*c].to_string())
            .collect();
        let mut rows: Vec<Vec<String>> = combined
            .iter()
            .map(|r| picked.iter().map(|(s, c)| r[*s][*c].clone()).collect())
            .collect();
        rows.sort();
        rows.truncate(max);
        Ok((names, rows))
    }
}

/// The text of an abapGit XML element, empty when it is missing.
fn tag(xml: &str, name: &str) -> String {
    xml.split(&format!("<{name}>"))
        .nth(1)
        .and_then(|v| v.split(&format!("</{name}>")).next())
        .unwrap_or_default()
        .trim()
        .to_string()
}

fn row(values: &[&str]) -> Vec<String> {
    values.iter().map(|v| v.to_string()).collect()
}

/// The program an enhancement position is in, as ENHINCINX names it: a
/// function module's hooks are under the main program of its group, a
/// method's under the class pool.
fn hook_program(position: &str, groups: &HashMap<String, String>) -> Option<String> {
    let first = position.strip_prefix('\\')?.split('\\').next()?;
    let (kind, name) = first.split_once(':')?;
    let name = name.to_ascii_uppercase();
    match kind {
        "PR" | "IC" => Some(name),
        "TY" => Some(class_pool(&name)),
        "FU" => {
            let group = groups.get(&name)?;
            Some(
                match group.strip_prefix('/').and_then(|g| g.split_once('/')) {
                    Some((ns, local)) => format!("/{ns}/SAPL{local}"),
                    None => format!("SAPL{group}"),
                },
            )
        }
        _ => None,
    }
}

/// Words upper case; literals keep their quote in front and their case.
fn tokens(sql: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut chars = sql.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            c if c.is_whitespace() => {}
            '\'' => {
                let mut literal = String::from("'");
                while let Some(c) = chars.next() {
                    if c == '\'' {
                        if chars.peek() == Some(&'\'') {
                            chars.next();
                        } else {
                            break;
                        }
                    }
                    literal.push(c);
                }
                out.push(literal);
            }
            '(' | ')' | ',' | '=' => out.push(c.to_string()),
            _ => {
                let mut word = String::from(c);
                while let Some(&n) = chars.peek() {
                    if n.is_whitespace() || "(),='".contains(n) {
                        break;
                    }
                    word.push(n);
                    chars.next();
                }
                out.push(word.to_ascii_uppercase());
            }
        }
    }
    out
}

struct Query {
    columns: Vec<String>,
    /// table and alias, the alias being the table when there is none
    sources: Vec<(String, String)>,
    /// the columns compared by the ON condition of the join
    join: Option<(String, String)>,
    conditions: Vec<Condition>,
}

struct Condition {
    column: String,
    test: Test,
}

enum Test {
    Equals(String),
    In(Vec<String>),
    Like(String),
}

impl Test {
    fn matches(&self, value: &str) -> bool {
        match self {
            Test::Equals(v) => value == v,
            Test::In(list) => list.iter().any(|v| v == value),
            Test::Like(pattern) => like(pattern, value),
        }
    }
}

/// ABAP SQL `LIKE` with `%` for any characters and `_` for one.
fn like(pattern: &str, value: &str) -> bool {
    let p: Vec<char> = pattern.chars().collect();
    let v: Vec<char> = value.chars().collect();
    fn at(p: &[char], v: &[char]) -> bool {
        match p.first() {
            None => v.is_empty(),
            Some('%') => (0..=v.len()).any(|i| at(&p[1..], &v[i..])),
            Some('_') => !v.is_empty() && at(&p[1..], &v[1..]),
            Some(c) => v.first() == Some(c) && at(&p[1..], &v[1..]),
        }
    }
    at(&p, &v)
}

impl Query {
    fn parse(tokens: &[String]) -> std::result::Result<Self, String> {
        let unsupported = |what: &str| format!("mock data preview: {what} is not supported");
        let mut words = tokens.iter().peekable();
        if words.next().map(String::as_str) != Some("SELECT") {
            return Err("Only SELECT statements are allowed".into());
        }
        let mut columns = Vec::new();
        loop {
            match words.next().map(String::as_str) {
                Some("FROM") => break,
                Some(",") => {}
                Some(c) => columns.push(c.to_string()),
                None => return Err("FROM is missing".into()),
            }
        }
        let mut sources = vec![source(&mut words).ok_or("the table is missing")?];
        let mut join = None;
        let mut conditions = Vec::new();
        while let Some(word) = words.next() {
            match word.as_str() {
                "INNER" if words.next().map(String::as_str) == Some("JOIN") => {
                    if sources.len() > 1 {
                        return Err(unsupported("more than one join"));
                    }
                    sources.push(source(&mut words).ok_or("the joined table is missing")?);
                    let on = [words.next(), words.next(), words.next(), words.next()];
                    let [Some(o), Some(left), Some(eq), Some(right)] = on else {
                        return Err("the ON condition is missing".into());
                    };
                    if o != "ON" || eq != "=" {
                        return Err(unsupported("this ON condition"));
                    }
                    join = Some((left.clone(), right.clone()));
                }
                "WHERE" | "AND" => {
                    let column = words.next().ok_or("a condition is missing")?.clone();
                    let test = match words.next().map(String::as_str) {
                        Some("=") => Test::Equals(literal(words.next())?),
                        Some("LIKE") => Test::Like(literal(words.next())?),
                        Some("IN") => {
                            if words.next().map(String::as_str) != Some("(") {
                                return Err(unsupported("IN without a list"));
                            }
                            let mut list = Vec::new();
                            loop {
                                match words.next().map(String::as_str) {
                                    Some(")") => break,
                                    Some(",") => {}
                                    Some(v) => list.push(literal(Some(&v.to_string()))?),
                                    None => return Err("the IN list is not closed".into()),
                                }
                            }
                            Test::In(list)
                        }
                        _ => return Err(unsupported("this condition")),
                    };
                    conditions.push(Condition { column, test });
                }
                other => return Err(unsupported(&format!("\"{other}\""))),
            }
        }
        Ok(Query {
            columns,
            sources,
            join,
            conditions,
        })
    }
}

type Words<'a> = std::iter::Peekable<std::slice::Iter<'a, String>>;

/// A table with its alias, the table itself when it has none.
fn source(words: &mut Words) -> Option<(String, String)> {
    let name = words.next()?.clone();
    let alias = if words.peek().map(|s| s.as_str()) == Some("AS") {
        words.next();
        words.next()?.clone()
    } else {
        name.clone()
    };
    Some((name, alias))
}

fn literal(token: Option<&String>) -> std::result::Result<String, String> {
    token
        .and_then(|t| t.strip_prefix('\''))
        .map(str::to_string)
        .ok_or_else(|| "a literal is expected".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tables() -> Tables {
        let mut relations = Table::new(&["CLSNAME", "REFCLSNAME", "RELTYPE", "VERSION"]);
        relations.rows = vec![
            row(&["ZCL_A", "ZIF_X", "1", "1"]),
            row(&["ZCL_B", "ZIF_X", "1", "1"]),
            row(&["ZCL_C", "ZCL_A", "2", "1"]),
        ];
        let mut exits = Table::new(&["EXIT_NAME", "INTER_NAME"]);
        exits.rows = vec![row(&["ZEXIT", "ZIF_EX"])];
        let mut attrs = Table::new(&["EXIT_NAME", "DEF_CLNAME", "MIG_BADI_NAME"]);
        attrs.rows = vec![row(&["ZEXIT", "ZCL_DEFAULT", ""])];
        let mut hooks = Table {
            long: &["FULL_NAME"],
            ..Table::new(&["ENHNAME", "FULL_NAME", "PROGRAMNAME"])
        };
        hooks.rows = vec![row(&["ZENH", "\\FU:ZFM\\SE:BEGIN\\EI", "SAPLZGRP"])];
        Tables(HashMap::from([
            ("SEOMETAREL", relations),
            ("SXS_INTER", exits),
            ("SXS_ATTR", attrs),
            ("ENHINCINX", hooks),
        ]))
    }

    #[test]
    fn queries_of_the_adt_provider() {
        let t = tables();
        let (columns, rows) = t
            .select(
                "SELECT clsname FROM seometarel WHERE refclsname = 'ZIF_X' AND reltype = '1' AND version = '1'",
                None,
                100,
            )
            .unwrap();
        assert_eq!(columns, ["CLSNAME"]);
        assert_eq!(rows, [["ZCL_A"], ["ZCL_B"]]);
        let (_, rows) = t
            .select(
                "SELECT x~exit_name, a~def_clname, a~mig_badi_name FROM sxs_inter AS x \
                 INNER JOIN sxs_attr AS a ON a~exit_name = x~exit_name \
                 WHERE x~inter_name = 'ZIF_EX'",
                None,
                100,
            )
            .unwrap();
        assert_eq!(rows, [["ZEXIT", "ZCL_DEFAULT", ""]]);
        let (_, rows) = t
            .select(
                "SELECT enhname, full_name FROM enhincinx WHERE programname IN ( 'SAPLZGRP',\n'ZPROG' )",
                None,
                100,
            )
            .unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(
            t.select("SELECT clsname FROM seometarel", None, 1)
                .unwrap()
                .1
                .len(),
            1
        );
    }

    #[test]
    fn what_basis_750_rejects() {
        let t = tables();
        let long = t
            .select(
                "SELECT enhname FROM enhincinx WHERE full_name LIKE '\\FU:ZFM\\%'",
                None,
                100,
            )
            .unwrap_err();
        assert!(long.contains("FULL_NAME is a long string"), "{long}");
        let line = format!(
            "SELECT clsname FROM seometarel WHERE refclsname IN ( {} )",
            "'ZIF_X', ".repeat(30)
        );
        assert!(t.select(&line, None, 100).unwrap_err().contains("255"));
        // the table preview reads the one table it is opened on
        assert!(t
            .select("SELECT clsname FROM seometarel", Some("SXS_INTER"), 100)
            .is_err());
        assert!(t
            .select("WHERE refclsname = 'ZIF_X'", Some("SEOMETAREL"), 100)
            .unwrap_err()
            .contains("Only SELECT"));
        assert!(t.select("SELECT x FROM nope", None, 100).is_err());
        assert!(like("\\FU:ZFM\\%", "\\FU:ZFM\\SE:BEGIN\\EI"));
        assert!(!like("Z_", "ZAB"));
    }

    #[test]
    fn positions_name_their_program() {
        let groups = HashMap::from([
            ("ZFM".to_string(), "ZGRP".to_string()),
            ("/ABC/FM".to_string(), "/ABC/GRP".to_string()),
        ]);
        let program = |p: &str| hook_program(p, &groups);
        assert_eq!(
            program("\\PR:ZPROG\\EX:ZPOINT\\EI").as_deref(),
            Some("ZPROG")
        );
        assert_eq!(
            program("\\FU:ZFM\\SE:BEGIN\\EI").as_deref(),
            Some("SAPLZGRP")
        );
        assert_eq!(
            program("\\FU:/ABC/FM\\SE:END\\EI").as_deref(),
            Some("/ABC/SAPLGRP")
        );
        assert_eq!(
            program("\\TY:ZCL_A\\ME:RUN\\SE:BEGIN\\EI").as_deref(),
            Some("ZCL_A=========================CP")
        );
        assert_eq!(program("\\FU:UNKNOWN\\SE:BEGIN\\EI"), None);
    }
}
