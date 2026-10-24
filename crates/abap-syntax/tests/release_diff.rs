//! The release each statement needs, compared with abaplint on whole files
//! and with abaplint's release-tagged statement cases.
//!
//! The corpus comparison needs golden data from the oracle, otherwise it is
//! skipped:
//!   node tools/oracle.mjs tests/corpus/golden --list tests/corpus/corpora.txt
//!   node tools/oracle.mjs --releases tests/corpus/golden/releases --list tests/corpus/corpora.txt
//!
//! Run with `cargo test --release -p abap-syntax --test release_diff -- --nocapture`
//! to see the report.

mod common;

use std::collections::{BTreeMap, HashMap, HashSet};

use abap_syntax::check::syntax_error;
use abap_syntax::release::{statement_needs, statement_release, Release};
use abap_syntax::StatementKind;
use serde::Deserialize;

/// Baselines; raise when improving.
const MIN_CORPUS_PERCENT: f64 = 99.99;
const MIN_CASES_PERCENT: f64 = 97.9;

#[derive(Deserialize)]
struct Golden {
    file: String,
    statements: Vec<GoldenStatement>,
}

#[derive(Deserialize)]
struct GoldenStatement {
    #[serde(rename = "type", default)]
    kind: Option<String>,
    tokens: Vec<(u32, u32)>,
    #[serde(default)]
    release: Option<String>,
}

type Diffs = BTreeMap<(String, String), Vec<String>>;

/// Constructs that abaplint's statement parser also reads as older syntax:
/// `DATA(x)` and `FINAL(x)` as a length access of a field, `xsdbool( )` and
/// `boolc( )` as a method call. abaplint gives the older release for them;
/// the construct needs the newer one.
const READ_AS_OLDER: [&str; 5] = [
    "inline declaration",
    "FINAL",
    "XSDBOOL",
    "BOOLC",
    "dynamic component",
];

/// Built-in predicate functions like `contains( )` came with the string
/// functions of 7.02; abaplint dates them, as conditions of their own, like
/// predicate method calls with 7.40 SP08.
const PREDICATE_FUNCTION: &str = "predicate function";

/// abaplint's cloud releases that equal an on-premise one.
const ON_PREMISE: [&str; 10] = [
    "v762", "v765", "v769", "v773", "v777", "v781", "v785", "v789", "v793", "v916",
];

fn report(title: &str, diffs: &Diffs) {
    let mut by_count: Vec<_> = diffs.iter().collect();
    by_count.sort_by_key(|(_, v)| std::cmp::Reverse(v.len()));
    println!("{title}:");
    for ((ours, theirs), cases) in by_count.iter().take(40) {
        println!("  ours {ours}, abaplint {theirs}: {}", cases.len());
        for c in cases.iter().take(3) {
            println!("      {c}");
        }
    }
}

#[test]
fn releases_of_the_corpus() {
    let root = common::repo_root();
    let files = common::json_files(&root.join("tests/corpus/golden/releases"));
    if files.is_empty() {
        println!("skipped: no release golden data, see the comment at the top of this file");
        return;
    }
    let mut total = 0;
    let mut same = 0;
    let mut older = 0;
    let mut predicates = 0;
    let mut diffs: Diffs = BTreeMap::new();
    for f in &files {
        let golden: Golden = serde_json::from_str(&std::fs::read_to_string(f).unwrap()).unwrap();
        // the statements abaplint has, to compare only those both split alike
        let all = root
            .join("tests/corpus/golden")
            .join(format!("{}.json", golden.file));
        let Ok(text) = std::fs::read_to_string(&all) else {
            continue;
        };
        let known: Golden = serde_json::from_str(&text).unwrap();
        let known: HashSet<Vec<(u32, u32)>> = known
            .statements
            .into_iter()
            .filter(|s| s.kind.as_deref() != Some("Unknown"))
            .map(|s| s.tokens)
            .collect();
        let theirs: HashMap<Vec<(u32, u32)>, Release> = golden
            .statements
            .into_iter()
            .filter_map(|s| {
                Some((
                    s.tokens,
                    s.release.as_deref().and_then(Release::of_abaplint)?,
                ))
            })
            .collect();

        let path = root.join(&golden.file);
        let source = std::fs::read_to_string(&path).unwrap();
        let parsed = abap_syntax::parse_with_macros(&source, &common::sibling_macros(&path));
        let tokens = parsed.tokens();
        for s in &parsed.statements {
            if matches!(
                s.kind,
                StatementKind::Comment | StatementKind::Unknown | StatementKind::NativeSQL
            ) || s.tokens.iter().any(|i| tokens[*i].expanded)
            {
                continue;
            }
            let key: Vec<(u32, u32)> = s
                .tokens
                .iter()
                .map(|i| (tokens[*i].start.row, tokens[*i].start.col))
                .collect();
            if !known.contains(&key) {
                continue;
            }
            total += 1;
            let needs = statement_needs(&parsed, s);
            let ours = statement_release(&parsed, s);
            let ours_release = ours.map_or(Release::V700, |n| n.release);
            let their_release = theirs.get(&key).copied().unwrap_or(Release::V700);
            if ours_release == their_release {
                same += 1;
            } else if ours_release > their_release
                && ours.is_some_and(|n| READ_AS_OLDER.contains(&n.construct))
            {
                same += 1;
                older += 1;
            } else if their_release == Release::V740Sp08
                && ours_release < their_release
                && needs.iter().any(|n| n.construct == PREDICATE_FUNCTION)
            {
                same += 1;
                predicates += 1;
            } else {
                let text = abap_syntax::join_tokens(parsed.statement_tokens(s));
                let construct = ours.map_or("-", |n| n.construct);
                diffs
                    .entry((
                        ours_release.id().to_string(),
                        their_release.id().to_string(),
                    ))
                    .or_default()
                    .push(format!(
                        "{}:{} [{construct}] {}",
                        golden.file,
                        key[0].0,
                        text.chars().take(140).collect::<String>()
                    ));
            }
        }
    }
    report("differences", &diffs);
    let rate = common::percent(same, total);
    println!(
                "\nstatements with the release abaplint gives: {same}/{total} ({rate:.2}%), {older} of them newer syntax abaplint also reads as older, {predicates} predicate functions abaplint dates with 7.40 SP08"
    );
    assert!(
        rate >= MIN_CORPUS_PERCENT,
        "{rate:.2}% below {MIN_CORPUS_PERCENT}%"
    );
}

#[derive(Deserialize)]
struct Case {
    abap: String,
    expect: String,
    #[serde(rename = "minRelease")]
    min_release: Option<String>,
    release: Option<String>,
    lang: Option<String>,
}

/// abaplint's cases: valid from a release (and not before), valid at a
/// release, or failing at a release.
#[test]
fn release_tagged_statement_cases() {
    let dir = common::repo_root().join("tests/corpus/abaplint/statements");
    let mut total = 0;
    let mut ok = 0;
    let mut diffs: Diffs = BTreeMap::new();
    for f in common::json_files(&dir) {
        let cases: Vec<Case> = serde_json::from_str(&std::fs::read_to_string(&f).unwrap()).unwrap();
        for case in cases {
            // the language versions of ABAP Cloud are not releases
            if case
                .lang
                .as_deref()
                .is_some_and(|l| l != "None" && l != "Normal")
            {
                continue;
            }
            let (tag, check): (&str, fn(Release, Release) -> bool) =
                match (&case.min_release, &case.release, case.expect.as_str()) {
                    (Some(r), _, "match") => (r, |ours, r| ours == r),
                    (None, Some(r), "match") => (r, |ours, r| ours <= r),
                    // a cloud release between two on-premise ones says
                    // nothing about either
                    (None, Some(r), "unknown")
                        if Release::ALL.iter().any(|x| x.id() == r)
                            || ON_PREMISE.contains(&r.as_str()) =>
                    {
                        (r, |ours, r| ours > r)
                    }
                    _ => continue,
                };
            let Some(release) = Release::of_abaplint(tag) else {
                continue;
            };
            let file = abap_syntax::parse(&case.abap);
            let Some(s) = file
                .statements
                .iter()
                .find(|s| s.kind != StatementKind::Comment)
            else {
                continue;
            };
            total += 1;
            let ours = statement_release(&file, s).map_or(Release::V700, |n| n.release);
            // a statement invalid at every release fails at this one too
            let invalid = case.expect == "unknown" && syntax_error(&file, s).is_some();
            if invalid || check(ours, release) {
                ok += 1;
            } else {
                let rule = match (&case.min_release, case.expect.as_str()) {
                    (Some(_), _) => "from",
                    (None, "match") => "at most",
                    _ => "above",
                };
                diffs
                    .entry((ours.id().to_string(), format!("{rule} {}", release.id())))
                    .or_default()
                    .push(case.abap.replace('\n', " "));
            }
        }
    }
    report("release-tagged cases not met", &diffs);
    let rate = common::percent(ok, total);
    println!("\nrelease-tagged cases met: {ok}/{total} ({rate:.1}%)");
    assert!(
        rate >= MIN_CASES_PERCENT,
        "{rate:.1}% below {MIN_CASES_PERCENT}%"
    );
}
