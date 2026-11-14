//! The grammar of CDS data definitions: it must accept every data definition
//! of the corpus, which all activate, every one of abaplint's CDS parser
//! test cases it accepts (`tests/corpus/abaplint/cds.json`), and the data
//! definitions fetched from an SAP system into `tools/.cache/sap` (SAP's
//! code, not committed; skipped when absent).
//!
//! Run with `cargo test --release -p abap-syntax --test cds_grammar -- --nocapture`.

use std::path::{Path, PathBuf};

use abap_syntax::cds::syntax_error;
use serde::Deserialize;

fn files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            files(&p, out);
        } else {
            let name = p.to_string_lossy().to_ascii_lowercase();
            if name.ends_with(".asddls") || name.ends_with(".ddls.abap") || name.ends_with(".ddls")
            {
                out.push(p);
            }
        }
    }
}

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Every source in `dir` that the grammar rejects, with where.
fn rejected(dir: &Path) -> (usize, Vec<String>) {
    let mut paths = Vec::new();
    files(dir, &mut paths);
    let mut wrong = Vec::new();
    for path in &paths {
        let text = std::fs::read_to_string(path).unwrap();
        if let Some(e) = syntax_error(&text) {
            wrong.push(format!(
                "{}:{}:{} at `{}`",
                path.strip_prefix(root()).unwrap_or(path).display(),
                e.start.row,
                e.start.col,
                e.text
            ));
        }
    }
    (paths.len(), wrong)
}

#[test]
fn corpus_definitions() {
    let (n, wrong) = rejected(&root().join("tests/corpus"));
    println!("corpus data definitions: {n}, rejected: {}", wrong.len());
    assert!(n >= 47, "only {n} data definitions");
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

#[test]
fn sap_system_definitions() {
    let dir = root().join("tools/.cache/sap");
    let (n, wrong) = rejected(&dir);
    println!(
        "data definitions of the SAP system: {n}, rejected: {}",
        wrong.len()
    );
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

#[derive(Deserialize)]
struct Case {
    cds: String,
    valid: bool,
}

/// abaplint's cases that the syntax check of the SAP ABAP Cloud Developer
/// Trial 2025 (8.16) rejects, by a line of each, with SAP's message for the
/// same construct on T100 (2026-11-15).
const SAP_REJECTS: [&str; 8] = [
    // a reserved word as a name: "Syntax Errors", "Unexpected keyword "count""
    "Group         : /moo/de_group;",
    "/ Count as sometype preserving type",
    // an annotation name has letters, digits and underscores: "Annotation
    // could not be parsed"
    "@Semantics.businessDate.from/to",
    // a typed literal in a DDIC-based view, not as the operand of CAST:
    // "Typed literals are not allowed in DDLS type V (views)"
    "abap.int4'1' as CountField",
    "where name = abap.sstr'ABC'",
    "{ abap.sstr'ABCD' as label }",
    // parameters of a table function with =>: "Unexpected word "=>""
    "date_start => :p_from",
    // "Field _/ABC/X does not start with a letter"
    "x as _/mynamespace/alias",
];

#[test]
fn abaplint_cases() {
    let path = root().join("tests/corpus/abaplint/cds.json");
    let cases: Vec<Case> = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let mut wrong = Vec::new();
    for c in cases.iter().filter(|c| c.valid) {
        if SAP_REJECTS.iter().any(|r| c.cds.contains(r)) {
            assert!(syntax_error(&c.cds).is_some(), "SAP rejects it: {}", c.cds);
            continue;
        }
        if let Some(e) = syntax_error(&c.cds) {
            let line = c.cds.lines().nth(e.start.row as usize - 1).unwrap_or("");
            wrong.push(format!(
                "at `{}` in line {}: {}",
                e.text,
                e.start.row,
                line.trim()
            ));
        }
    }
    println!(
        "abaplint's valid CDS cases: {}, rejected: {}",
        cases.iter().filter(|c| c.valid).count(),
        wrong.len()
    );
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}
