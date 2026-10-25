//! The syntax checks: they must report none of the statements abaplint
//! accepts, on the corpus and in abaplint's test cases, and they should
//! report the cases abaplint rejects.
//!
//! The corpus part needs golden data from the oracle, otherwise it is
//! skipped: `node tools/oracle.mjs tests/corpus/golden --list tests/corpus/corpora.txt`.
//!
//! Run with `cargo test --release -p abap-syntax --test syntax_errors -- --nocapture`
//! to see the report.

mod common;

use std::collections::HashMap;

use abap_syntax::check::syntax_error;
use abap_syntax::StatementKind;
use serde::Deserialize;

/// Baseline; raise when improving.
const MIN_REJECTED: usize = 77;

#[derive(Deserialize)]
struct Golden {
    file: String,
    statements: Vec<GoldenStatement>,
}

#[derive(Deserialize)]
struct GoldenStatement {
    #[serde(rename = "type")]
    kind: String,
    tokens: Vec<(u32, u32)>,
}

#[test]
fn no_error_in_what_abaplint_accepts() {
    let root = common::repo_root();
    let files = common::json_files(&root.join("tests/corpus/golden/tests"));
    if files.is_empty() {
        println!("skipped: no golden data, see the comment at the top of this file");
        return;
    }
    let mut checked = 0;
    let mut wrong = Vec::new();
    for f in &files {
        let golden: Golden = serde_json::from_str(&std::fs::read_to_string(f).unwrap()).unwrap();
        let accepted: HashMap<Vec<(u32, u32)>, String> = golden
            .statements
            .into_iter()
            .filter(|s| s.kind != "Unknown")
            .map(|s| (s.tokens, s.kind))
            .collect();
        let path = root.join(&golden.file);
        let source = std::fs::read_to_string(&path).unwrap();
        let parsed = abap_syntax::parse_with_macros(&source, &common::sibling_macros(&path));
        let tokens = parsed.tokens();
        for s in &parsed.statements {
            let key: Vec<(u32, u32)> = s
                .tokens
                .iter()
                .map(|i| (tokens[*i].start.row, tokens[*i].start.col))
                .collect();
            if !accepted.contains_key(&key) {
                continue;
            }
            checked += 1;
            if let Some(e) = syntax_error(&parsed, s) {
                wrong.push(format!(
                    "{}:{} {}: {}",
                    golden.file,
                    key[0].0,
                    e.message,
                    abap_syntax::join_tokens(parsed.statement_tokens(s))
                        .chars()
                        .take(140)
                        .collect::<String>()
                ));
            }
        }
    }
    for w in wrong.iter().take(40) {
        println!("  {w}");
    }
    println!(
        "\nstatements abaplint accepts: {checked}, reported as errors: {}",
        wrong.len()
    );
    assert!(
        wrong.is_empty(),
        "{} accepted statements reported",
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
                    if let Some(e) = error {
                        wrong.push(format!("{}: {}", e.message, case.abap.replace('\n', " ")));
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
