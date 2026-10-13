//! Differential test of the SQL parser against abaplint: tables, INTO
//! targets, number of joins and FOR ALL ENTRIES, for the abaplint statement
//! cases and for every database statement of the corpus.
//!
//! Needs the golden data of tools/oracle.mjs (see crates/abap-syntax/tests/oracle_diff.rs):
//!   node tools/oracle.mjs --sql-cases tests/corpus/abaplint/statements tests/corpus/golden/sql_cases.json
//! Run with `cargo test -p abap-sql --test sql_diff -- --nocapture` for the report.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use abap_model as m;
use serde::Deserialize;

/// Minimum agreement per category, in percent; raise when improving.
/// Known, intended differences: joins nested in parentheses count each join
/// (abaplint counts the outer one), `PROVIDED BY schema` is no table.
const MIN_PERCENT: f64 = 99.5;

#[derive(Deserialize)]
struct SqlInfo {
    tables: Vec<String>,
    targets: Vec<String>,
    joins: usize,
    #[serde(rename = "forAllEntries")]
    for_all_entries: Option<String>,
}

#[derive(Deserialize)]
struct Case {
    abap: String,
    sql: SqlInfo,
}

#[derive(Deserialize)]
struct Golden {
    file: String,
    statements: Vec<GStatement>,
}

#[derive(Deserialize)]
struct GStatement {
    tokens: Vec<(u32, u32)>,
    sql: Option<SqlInfo>,
}

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
}

fn json_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            json_files(&p, out);
        } else if p.extension().is_some_and(|x| x == "json") {
            out.push(p);
        }
    }
}

fn norm(s: &str) -> String {
    s.chars()
        .filter(|c| !c.is_whitespace())
        .collect::<String>()
        .to_ascii_lowercase()
}

fn sorted(mut v: Vec<String>) -> Vec<String> {
    v.sort();
    v
}

#[derive(Default)]
struct Report {
    categories: BTreeMap<&'static str, (usize, usize, Vec<String>)>,
}

impl Report {
    fn check<T: PartialEq + std::fmt::Debug>(
        &mut self,
        category: &'static str,
        at: &str,
        ours: T,
        theirs: T,
    ) {
        let e = self.categories.entry(category).or_default();
        e.0 += 1;
        if ours == theirs {
            e.1 += 1;
        } else {
            e.2.push(format!(
                "{at}\n        ours {ours:?}\n        abaplint {theirs:?}"
            ));
        }
    }

    fn compare(&mut self, prefix: &'static str, at: &str, a: &m::DbAccess, g: &SqlInfo) {
        let ours_tables = a
            .tables
            .iter()
            .filter(|t| !matches!(t.role, m::DbTableRole::Cte | m::DbTableRole::InternalTable))
            .map(|t| {
                norm(&if t.dynamic {
                    format!("({})", t.name)
                } else {
                    t.name.clone()
                })
            })
            .collect();
        let theirs_tables = g.tables.iter().map(|t| norm(t)).collect();
        let cat = |c: &'static str| -> &'static str {
            match (prefix, c) {
                ("cases", "tables") => "cases: tables",
                ("cases", "targets") => "cases: targets",
                ("cases", "joins") => "cases: joins",
                ("cases", "fae") => "cases: for all entries",
                (_, "tables") => "corpus: tables",
                (_, "targets") => "corpus: targets",
                (_, "joins") => "corpus: joins",
                _ => "corpus: for all entries",
            }
        };
        self.check(
            cat("tables"),
            at,
            sorted(ours_tables),
            sorted(theirs_tables),
        );
        // abaplint counts the cursor of OPEN CURSOR as a target
        let ours_targets = a
            .target
            .iter()
            .flat_map(|t| t.targets.iter().map(|x| norm(x)))
            .chain(a.cursor.iter().map(|c| norm(c)))
            .collect();
        self.check(
            cat("targets"),
            at,
            sorted(ours_targets),
            sorted(g.targets.iter().map(|t| norm(t)).collect()),
        );
        self.check(cat("joins"), at, a.joins.len(), g.joins);
        self.check(
            cat("fae"),
            at,
            a.for_all_entries.as_deref().map(norm),
            g.for_all_entries.as_deref().map(norm),
        );
    }
}

#[test]
fn sql_diff() {
    let root = root();
    let cases_file = root.join("tests/corpus/golden/sql_cases.json");
    if !cases_file.exists() {
        println!("skipped: no golden data");
        return;
    }
    let mut r = Report::default();

    let cases: Vec<Case> =
        serde_json::from_str(&std::fs::read_to_string(&cases_file).unwrap()).unwrap();
    for c in &cases {
        let file = abap_syntax::parse(&c.abap);
        let Some(s) = file
            .statements
            .iter()
            .find(|s| abap_sql::is_db_statement(s.kind))
        else {
            r.check(
                "cases: recognized as database statement",
                &c.abap,
                false,
                true,
            );
            continue;
        };
        r.check(
            "cases: recognized as database statement",
            &c.abap,
            true,
            true,
        );
        let a = abap_sql::parse_statement(&file, s).unwrap();
        r.compare("cases", &c.abap.replace('\n', " "), &a, &c.sql);
    }

    let mut files = Vec::new();
    json_files(&root.join("tests/corpus/golden/tests"), &mut files);
    files.sort();
    for f in &files {
        let golden: Golden = serde_json::from_str(&std::fs::read_to_string(f).unwrap()).unwrap();
        if golden.statements.iter().all(|s| s.sql.is_none()) {
            continue;
        }
        let source = std::fs::read_to_string(root.join(&golden.file)).unwrap();
        let file = abap_syntax::parse(&source);
        let tokens = file.tokens();
        // statements of a macro expansion share the position of the call,
        // so several database statements can end at one position: matched in order
        let mut by_last: BTreeMap<(u32, u32), Vec<&abap_syntax::Statement>> = BTreeMap::new();
        for s in &file.statements {
            if abap_sql::is_db_statement(s.kind) {
                let t = &tokens[*s.tokens.last().unwrap()];
                by_last
                    .entry((t.start.row, t.start.col))
                    .or_default()
                    .push(s);
            }
        }
        for g in &golden.statements {
            let Some(info) = &g.sql else { continue };
            let Some(last) = g.tokens.last() else {
                continue;
            };
            let at = format!("{}:{}", golden.file, last.0);
            let ours = by_last
                .get_mut(last)
                .and_then(|q| (!q.is_empty()).then(|| q.remove(0)));
            r.check(
                "corpus: recognized as database statement",
                &at,
                ours.is_some(),
                true,
            );
            if let Some(s) = ours {
                let a = abap_sql::parse_statement(&file, s).unwrap();
                r.compare("corpus", &at, &a, info);
            }
        }
    }

    let mut failed = Vec::new();
    for (category, (compared, equal, mismatches)) in &r.categories {
        let percent = 100.0 * *equal as f64 / (*compared).max(1) as f64;
        println!("{category}: {equal}/{compared} ({percent:.2}%)");
        for mm in mismatches
            .iter()
            .take(if std::env::var("SQL_DIFF_ALL").is_ok() {
                1000
            } else {
                5
            })
        {
            println!("    {mm}");
        }
        if percent < MIN_PERCENT {
            failed.push(format!("{category} {percent:.2}%"));
        }
    }
    assert!(failed.is_empty(), "below baseline: {failed:?}");
}
