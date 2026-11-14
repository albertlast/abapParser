//! The syntax checks: they must report no statement of the corpus, which is
//! code that compiles, apart from errors an SAP system confirms, and none of
//! the cases abaplint accepts, apart from the ones an SAP system rejects;
//! they should report the cases abaplint rejects.
//!
//! Run with `cargo test --release -p abap-syntax --test syntax_errors -- --nocapture`
//! to see the report.

mod common;

use abap_syntax::check::syntax_error;
use abap_syntax::StatementKind;
use serde::Deserialize;

/// Baseline; raise when improving.
const MIN_REJECTED: usize = 96;

/// Syntax errors the corpus really has, each confirmed by the syntax check
/// of an SAP system: the end of the file's path and the start of the
/// statement's text, tokens joined by blanks.
const REAL_ERRORS: &[(&str, &str)] = &[];

/// Cases abaplint accepts and the syntax check of an SAP system (8.16)
/// rejects, with its message.
const SAP_REJECTS: &[&str] = &[
    // A component name was expected after "-".
    "IF foo- = 2.",
    // no name ends with a dash: Field "CHAR-" is unknown.
    "SELECT SINGLE foo FROM bar WHERE column IN (sdf, char-).",
    // The elements in the "SELECT LIST" list must be separated using commas
    // (strict mode, from the host variables with @).
    "SELECT f1 SUM( f2 ) AS s1 SUM( f3 ) AS s2 FROM ztab INTO CORRESPONDING FIELDS OF TABLE @lt GROUP BY f1 HAVING SUM( f2 ) <> 0 OR SUM( f3 ) <> 0.",
    "SELECT f1 SUM( ABS( f2 ) ) AS s1 MAX( f3 ) AS s2 INTO CORRESPONDING FIELDS OF TABLE @lt FROM ztab GROUP BY f1 HAVING SUM( ABS( f2 ) ) <> MAX( f3 ).",
    "SELECT f1 SUM( f2 ) AS s FROM ztab AS t1 GROUP BY f1 HAVING COUNT( * ) > ( SELECT COUNT( * ) FROM ztab2 WHERE f3 = SUM( t1~f2 ) ) INTO TABLE @lt.",
];

#[test]
fn no_error_in_the_corpus() {
    let root = common::repo_root();
    let mut checked = 0;
    let mut real = 0;
    let mut wrong = Vec::new();
    for path in common::corpus_files() {
        let source = String::from_utf8_lossy(&std::fs::read(&path).unwrap()).into_owned();
        let parsed = abap_syntax::parse_with_macros(&source, &common::sibling_macros(&path));
        let file = path
            .strip_prefix(&root)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");
        for s in &parsed.statements {
            checked += 1;
            let Some(e) = syntax_error(&parsed, s) else {
                continue;
            };
            let text = abap_syntax::join_tokens(parsed.statement_tokens(s));
            if REAL_ERRORS
                .iter()
                .any(|(f, t)| file.ends_with(f) && text.starts_with(t))
            {
                real += 1;
                continue;
            }
            let at = &parsed.tokens()[s.tokens[e.token]];
            wrong.push(format!(
                "{file}:{} {} at `{}`: {}",
                at.start.row,
                e.message,
                at.text,
                text.chars().take(160).collect::<String>()
            ));
        }
    }
    for w in &wrong {
        println!("  {w}");
    }
    println!(
        "
corpus statements: {checked}, real errors: {real}, reported otherwise: {}",
        wrong.len()
    );
    assert!(
        wrong.is_empty(),
        "{} corpus statements reported",
        wrong.len()
    );
}

#[derive(Deserialize)]
struct Case {
    abap: String,
    expect: String,
    lang: Option<String>,
    release: Option<String>,
}

#[test]
fn abaplint_cases() {
    let dir = common::repo_root().join("tests/corpus/abaplint/statements");
    let mut rejected = 0;
    let mut missed = Vec::new();
    let mut wrong = Vec::new();
    for f in common::json_files(&dir) {
        let cases: Vec<Case> = serde_json::from_str(&std::fs::read_to_string(&f).unwrap()).unwrap();
        for case in cases {
            // the language versions of ABAP Cloud restrict what is valid
            if case
                .lang
                .as_deref()
                .is_some_and(|l| l != "None" && l != "Normal")
            {
                continue;
            }
            let file = abap_syntax::parse(&case.abap);
            let Some(s) = file
                .statements
                .iter()
                .find(|s| s.kind != StatementKind::Comment)
            else {
                continue;
            };
            let error = syntax_error(&file, s);
            match case.expect.as_str() {
                "match" => {
                    let abap = case.abap.replace('\n', " ");
                    if let Some(e) = error.filter(|_| !SAP_REJECTS.contains(&abap.as_str())) {
                        wrong.push(format!("{}: {abap}", e.message));
                    }
                }
                // rejected at every release, not only at an older one
                _ if case.release.is_none() => {
                    if s.kind == StatementKind::Unknown || error.is_some() {
                        rejected += 1;
                    } else {
                        missed.push(case.abap.replace('\n', " "));
                    }
                }
                _ => {}
            }
        }
    }
    println!("not rejected:");
    for m in &missed {
        println!("  {m}");
    }
    println!("valid cases reported:");
    for w in &wrong {
        println!("  {w}");
    }
    println!(
        "\ninvalid cases rejected: {rejected}/{}, valid cases reported: {}",
        rejected + missed.len(),
        wrong.len()
    );
    assert!(wrong.is_empty(), "{} valid cases reported", wrong.len());
    assert!(rejected >= MIN_REJECTED, "{rejected} below {MIN_REJECTED}");
}
