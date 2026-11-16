//! The grammar of SQLScript: it must accept the body of every AMDP method
//! of the corpus and of the classes fetched from an SAP system into
//! `tools/.cache/sap/amdp` (SAP's code, not committed; skipped when absent),
//! which all activate on SAP HANA; nor may the rules of USING report one.
//! Our own probe bodies must be decided as the syntax check of an SAP
//! system decided them (`tests/corpus/sap-verdicts/sqlscript.json`).
//!
//! Run with `cargo test --release -p abap-syntax --test sqlscript_grammar -- --nocapture`.

use std::path::{Path, PathBuf};

use abap_syntax::sqlscript::{amdp_body, syntax_error};

fn files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            files(&p, out);
        } else if p.to_string_lossy().ends_with(".abap") {
            out.push(p);
        }
    }
}

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// The AMDP bodies in `dir` and those the grammar rejects, with where and
/// the line there.
fn rejected(dir: &Path) -> (usize, Vec<String>) {
    let mut paths = Vec::new();
    files(dir, &mut paths);
    let mut bodies = 0;
    let mut wrong = Vec::new();
    for path in &paths {
        let source = std::fs::read_to_string(path).unwrap();
        if !source.to_ascii_uppercase().contains("BY DATABASE") {
            continue;
        }
        let file = abap_syntax::parse(&source);
        for i in 0..file.statements.len() {
            let Some(range) = amdp_body(&file, i) else {
                continue;
            };
            bodies += 1;
            // USING of the method, quoted names in the body
            let rules = std::iter::once(&file.statements[i]).chain(
                file.statements[i + 1..]
                    .iter()
                    .take_while(|s| s.kind == abap_syntax::StatementKind::NativeSQL),
            );
            for s in rules {
                if let Some(e) = abap_syntax::check::syntax_error(&file, s) {
                    if !e.message.contains("SQLScript grammar") {
                        wrong.push(format!(
                            "{}: {}: {}",
                            path.strip_prefix(root()).unwrap_or(path).display(),
                            e.message,
                            file.statement_text(s).chars().take(100).collect::<String>()
                        ));
                    }
                }
            }
            let body = &file.source()[range.clone()];
            if let Some(e) = syntax_error(body) {
                let before = file.source()[..range.start].matches('\n').count() as u32;
                let line = body.lines().nth(e.start.row as usize - 1).unwrap_or("");
                wrong.push(format!(
                    "{}:{}:{} at `{}`: {}",
                    path.strip_prefix(root()).unwrap_or(path).display(),
                    before + e.start.row,
                    e.start.col,
                    e.text,
                    line.trim()
                ));
            }
        }
    }
    (bodies, wrong)
}

#[test]
fn corpus_bodies() {
    let (n, wrong) = rejected(&root().join("tests/corpus"));
    println!("corpus AMDP bodies: {n}, rejected: {}", wrong.len());
    for w in &wrong {
        println!("  {w}");
    }
    assert!(n >= 30, "only {n} AMDP bodies");
    assert!(wrong.is_empty());
}

#[test]
fn sap_system_bodies() {
    let dir = root().join("tools/.cache/sap/amdp");
    if !dir.exists() {
        println!("skipped: no {}", dir.display());
        return;
    }
    let (n, wrong) = rejected(&dir);
    println!("SAP AMDP bodies: {n}, rejected: {}", wrong.len());
    for w in wrong.iter().take(60) {
        println!("  {w}");
    }
    assert!(wrong.is_empty());
}

/// The probe class of `tests/corpus/sap-verdicts/sqlscript.json`: each case
/// is the body of its method `run`, with the USING given.
const PROBE_CLASS: &str = "CLASS zcl_abapparser_amdp DEFINITION PUBLIC FINAL CREATE PUBLIC.
  PUBLIC SECTION.
    INTERFACES if_amdp_marker_hdb.
    TYPES: tt_scarr TYPE STANDARD TABLE OF scarr WITH EMPTY KEY,
           tt_spfli TYPE STANDARD TABLE OF spfli WITH EMPTY KEY,
           BEGIN OF ty_sum,
             carrid TYPE s_carr_id,
             total  TYPE p LENGTH 15 DECIMALS 2,
           END OF ty_sum,
           tt_sum TYPE STANDARD TABLE OF ty_sum WITH EMPTY KEY.
    CLASS-METHODS helper IMPORTING VALUE(iv_carrid) TYPE s_carr_id EXPORTING VALUE(et_carr) TYPE tt_scarr.
    CLASS-METHODS run IMPORTING VALUE(iv_carrid) TYPE s_carr_id VALUE(iv_n) TYPE i
                      EXPORTING VALUE(et_carr) TYPE tt_scarr VALUE(et_spfli) TYPE tt_spfli
                                VALUE(et_sum) TYPE tt_sum VALUE(ev_count) TYPE i
                      CHANGING  VALUE(ct_carr) TYPE tt_scarr.
ENDCLASS.
CLASS zcl_abapparser_amdp IMPLEMENTATION.
  METHOD helper BY DATABASE PROCEDURE FOR HDB LANGUAGE SQLSCRIPT OPTIONS READ-ONLY USING scarr.
    et_carr = SELECT * FROM scarr WHERE carrid = :iv_carrid;
  ENDMETHOD.
  METHOD run BY DATABASE PROCEDURE FOR HDB LANGUAGE SQLSCRIPT";

#[derive(serde::Deserialize)]
struct Verdict {
    name: String,
    using: String,
    body: String,
    /// What the syntax check of the SAP ABAP Cloud Developer Trial 2025
    /// (8.16, SAP HANA 2.0 SPS 08) reported on the class, 2026-11-15:
    /// `ok`; `syntax` and `semantic` for a syntax error and another error
    /// of SAP HANA; `unclosed` for a body closed unexpectedly; `quoted` for
    /// a quoted name not in upper case; `unneeded` for an object after USING
    /// not used; `unknown method` for an AMDP method not after USING; and,
    /// not decided without the dictionary or the system, `unknown` for any
    /// other name not after USING, `transactional` for COMMIT and `client`
    /// for a client-specific CDS entity.
    sap: String,
}

/// Cases decided otherwise than by SAP. A built-in function with a named
/// argument is a syntax error of SAP HANA, which is not reported, as
/// functions of the system and CONVERT_CURRENCY take them.
/// `CARDINALITY()` is rejected by SAP HANA for the number of its arguments,
/// here for the empty list.
const KNOWN: [&str; 2] = [
    "probe: named arguments builtin",
    "round 1: base 30: delete :arr @93",
];

/// Our own bodies on the flight tables, changed by one token (dropped,
/// doubled, swapped, replaced), probes of the grammar and of USING, with the
/// verdict of SAP's syntax check: every error SAP reports that needs neither
/// dictionary nor system is reported, nothing it accepts or rejects for
/// semantic reasons is.
#[test]
fn sap_verdicts() {
    let path = root().join("tests/corpus/sap-verdicts/sqlscript.json");
    let cases: Vec<Verdict> =
        serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let mut count = std::collections::BTreeMap::<(&str, bool), usize>::new();
    let mut wrong = Vec::new();
    for c in &cases {
        let using = if c.using.is_empty() {
            String::new()
        } else {
            format!(" {}", c.using)
        };
        let source = format!("{PROBE_CLASS}{using}.\n{}\n  ENDMETHOD.\nENDCLASS.", c.body);
        let file = abap_syntax::parse(&source);
        let errors: Vec<String> = file
            .statements
            .iter()
            .filter_map(|s| {
                abap_syntax::check::syntax_error(&file, s)
                    .map(|e| format!("{}: {}", file.statement_text(s), e.message))
            })
            .collect();
        let reported = !errors.is_empty();
        *count.entry((c.sap.as_str(), reported)).or_default() += 1;
        let expected = match c.sap.as_str() {
            "ok" | "semantic" => Some(false),
            "syntax" | "unclosed" | "quoted" | "unneeded" | "unknown method" => Some(true),
            "unknown" | "transactional" | "client" => None,
            other => panic!("verdict {other}"),
        };
        let known = KNOWN.contains(&c.name.as_str());
        if expected.is_some_and(|e| e != reported) != known {
            wrong.push(format!(
                "{} (SAP: {}, here: {}){}",
                c.name,
                c.sap,
                if reported {
                    errors.join("; ")
                } else {
                    "none".into()
                },
                if known { ", listed in KNOWN" } else { "" }
            ));
        }
    }
    println!("SAP verdicts: {} cases", cases.len());
    for ((sap, reported), n) in &count {
        println!("  SAP {sap:>14}, reported here: {reported:<5} {n}");
    }
    assert!(cases.len() >= 1_300, "only {} cases", cases.len());
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}
