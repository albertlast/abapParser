//! The grammar of SQLScript: it must accept the body of every AMDP method
//! of the corpus and of the classes fetched from an SAP system into
//! `tools/.cache/sap/amdp` (SAP's code, not committed; skipped when absent),
//! which all activate on SAP HANA; nor may the rules of USING report one.
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
