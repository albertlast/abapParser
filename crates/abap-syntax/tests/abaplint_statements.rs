//! Table-driven test over the statement cases extracted from abaplint's test
//! suite (tests/corpus/abaplint/statements, see tools/extract-abaplint-tests.mjs).
//!
//! Run with `cargo test -p abap-syntax --test abaplint_statements -- --nocapture`
//! to see the per-kind report.

mod common;

use std::collections::BTreeMap;

use abap_syntax::StatementKind;
use serde::Deserialize;

/// Minimum share of "match" cases classified like abaplint; raise when improving.
const MIN_MATCH_PERCENT: f64 = 99.8;

#[derive(Deserialize)]
struct Case {
    abap: String,
    expect: String,
    #[serde(rename = "type")]
    kind: Option<String>,
}

#[test]
fn abaplint_statement_cases() {
    let dir = common::repo_root().join("tests/corpus/abaplint/statements");
    let files = common::json_files(&dir);
    assert!(!files.is_empty(), "no cases found in {}", dir.display());

    let mut total = 0;
    let mut passed = 0;
    let mut unknown_total = 0;
    let mut unknown_passed = 0;
    let mut failures: BTreeMap<String, Vec<String>> = BTreeMap::new();

    for f in files {
        let cases: Vec<Case> = serde_json::from_str(&std::fs::read_to_string(&f).unwrap()).unwrap();
        for case in cases {
            let file = abap_syntax::parse(&case.abap);
            let statements: Vec<_> = file
                .statements
                .iter()
                .filter(|s| s.kind != StatementKind::Comment)
                .collect();
            let got = match statements.as_slice() {
                [one] => one.kind.as_str().to_string(),
                _ => format!("{} statements", statements.len()),
            };
            if case.expect == "unknown" {
                unknown_total += 1;
                if got == "Unknown" {
                    unknown_passed += 1;
                }
                continue;
            }
            let expected = case.kind.unwrap_or_default();
            total += 1;
            if got == expected {
                passed += 1;
            } else {
                failures
                    .entry(expected)
                    .or_default()
                    .push(format!("got {got}: {}", case.abap.replace('\n', " ")));
            }
        }
    }

    let mut by_count: Vec<_> = failures.iter().collect();
    by_count.sort_by_key(|(_, v)| std::cmp::Reverse(v.len()));
    for (kind, cases) in by_count.iter().take(40) {
        println!("{kind}: {} failing", cases.len());
        for c in cases.iter().take(3) {
            println!("    {c}");
        }
    }
    let rate = common::percent(passed, total);
    println!(
        "\nmatch cases: {passed}/{total} ({rate:.1}%), expected-unknown cases rejected: {unknown_passed}/{unknown_total}"
    );
    assert!(
        rate >= MIN_MATCH_PERCENT,
        "classification rate {rate:.1}% below baseline {MIN_MATCH_PERCENT}%"
    );
}
