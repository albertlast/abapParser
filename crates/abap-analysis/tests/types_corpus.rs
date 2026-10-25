//! Share of inline declarations with a type over the ABAP cheat sheets:
//! file by file, and as classes of a workspace over the folder, where the
//! types of other classes are found too. Fails when a share falls below its
//! baseline; raise the baseline when it rises.
//!
//! Run with `cargo test --release -p abap-analysis --test types_corpus -- --nocapture`.

use std::path::Path;

use abap_analysis::abap_source::{FsProvider, ObjectKind};
use abap_analysis::Workspace;
use abap_model as m;

/// Percent of typed inline declarations, at least.
const FILES: f64 = 62.5;
const WORKSPACE: f64 = 66.3;

fn count(steps: &[m::Step], typed: &mut usize, all: &mut usize) {
    for s in steps {
        for w in s.writes.iter().filter(|w| w.declared) {
            *all += 1;
            *typed += usize::from(w.data_type.is_some());
        }
        count(&s.children, typed, all);
    }
}

fn share(typed: usize, all: usize) -> f64 {
    100.0 * typed as f64 / all.max(1) as f64
}

#[test]
fn cheat_sheet_types() {
    let dir =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/corpus/abap-cheat-sheets/src");
    if !dir.exists() {
        println!("skipped, submodule missing");
        return;
    }
    let (mut typed, mut all) = (0, 0);
    for entry in std::fs::read_dir(&dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_some_and(|e| e == "abap") {
            let l = abap_analysis::logic(&std::fs::read_to_string(&path).unwrap());
            for p in &l.procedures {
                count(&p.body, &mut typed, &mut all);
            }
        }
    }
    let files = share(typed, all);
    println!("file by file: {typed} of {all} typed, {files:.1}%");

    let provider = FsProvider::new(vec![dir]).unwrap();
    let classes: Vec<_> = provider
        .references()
        .filter(|r| r.kind == ObjectKind::Clas && r.part.is_none())
        .cloned()
        .collect();
    let workspace = Workspace::new(Box::new(provider));
    let (mut typed, mut all) = (0, 0);
    for r in &classes {
        let l = workspace.logic(r, 0);
        for p in &l.procedures {
            count(&p.body, &mut typed, &mut all);
        }
    }
    let ws = share(typed, all);
    println!("workspace: {typed} of {all} typed, {ws:.1}%");

    assert!(files >= FILES, "file by file {files:.1}% < {FILES}%");
    assert!(ws >= WORKSPACE, "workspace {ws:.1}% < {WORKSPACE}%");
}
