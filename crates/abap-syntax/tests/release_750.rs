//! Code known to be ABAP 7.50: abap-wasm checks its `src/` with abaplint
//! against 7.50 in its CI. None of its statements may need a newer release,
//! have a syntax error, or be unknown.
//!
//! Run with `cargo test -p abap-syntax --test release_750`.

use std::path::{Path, PathBuf};

use abap_syntax::check::syntax_error;
use abap_syntax::release::{statement_release, Release};
use abap_syntax::StatementKind;

fn abap_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            abap_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "abap") {
            out.push(path);
        }
    }
}

#[test]
fn abap_wasm_is_7_50() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/corpus/abap-wasm/src");
    if !dir.exists() {
        println!("skipped, submodule missing");
        return;
    }
    let mut files = Vec::new();
    abap_files(&dir, &mut files);
    let mut wrong = Vec::new();
    let mut statements = 0;
    for path in &files {
        let file = abap_syntax::parse(&std::fs::read_to_string(path).unwrap());
        let name = path.strip_prefix(&dir).unwrap().display();
        for s in &file.statements {
            statements += 1;
            let row = file.tokens()[s.tokens[0]].start.row;
            if s.kind == StatementKind::Unknown {
                wrong.push(format!("{name}:{row}: unknown statement"));
            }
            if let Some(need) = statement_release(&file, s).filter(|n| n.release > Release::V750) {
                wrong.push(format!(
                    "{name}:{row}: needs {} for {}",
                    need.release.name(),
                    need.construct
                ));
            }
            if let Some(e) = syntax_error(&file, s) {
                wrong.push(format!("{name}:{row}: syntax error {e:?}"));
            }
        }
    }
    println!("{} files, {statements} statements", files.len());
    assert!(files.len() >= 300, "only {} files", files.len());
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}
