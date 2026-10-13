//! Logic of whole objects over every corpus of tests/corpus/corpora.txt,
//! through the folder provider: includes, class parts and called objects
//! loaded through the workspace. Reports how many effects come through
//! calls, how many write database tables and how much data is global.
//!
//! Run with `cargo test --release -p abap-analysis --test logic_corpus -- --nocapture`.

use std::path::Path;

use abap_analysis::abap_source::{FsProvider, ObjectKind};
use abap_analysis::Workspace;
use abap_model as m;

fn walk<'a>(steps: &'a [m::Step], f: &mut impl FnMut(&'a m::Step)) {
    for s in steps {
        f(s);
        walk(&s.children, f);
    }
}

#[test]
fn corpus_object_logic() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let list = std::fs::read_to_string(root.join("tests/corpus/corpora.txt")).unwrap();
    let (mut procedures, mut effects, mut via, mut database, mut globals, mut tables) =
        (0, 0, 0, 0, 0, 0);
    let (mut inputs, mut through) = (0, 0);
    let mut slowest = (std::time::Duration::ZERO, String::new());
    let started = std::time::Instant::now();
    for dir in list
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
    {
        let path = root.join(dir);
        if !path.exists() {
            println!("{dir}: skipped, submodule missing");
            continue;
        }
        let provider = FsProvider::new(vec![path]).unwrap();
        let roots: Vec<_> = provider
            .references()
            .filter(|r| {
                matches!(
                    r.kind,
                    ObjectKind::Clas | ObjectKind::Prog | ObjectKind::Fugr
                ) && r.part.is_none()
            })
            .cloned()
            .collect();
        let workspace = Workspace::new(Box::new(provider));
        for r in &roots {
            let t = std::time::Instant::now();
            let l = workspace.logic(r, 1);
            let elapsed = t.elapsed();
            if elapsed > slowest.0 {
                slowest = (elapsed, r.to_string());
            }
            assert!(l.errors.is_empty(), "{r}: {:?}", l.errors);
            procedures += l.procedures.len();
            for p in &l.procedures {
                effects += p.effects.len();
                inputs += p.effects.iter().map(|e| e.inputs.len()).sum::<usize>();
                via += p.effects.iter().filter(|e| !e.via.is_empty()).count();
                database += p
                    .effects
                    .iter()
                    .filter(|e| e.scope == m::DataScope::Database)
                    .count();
                walk(&p.body, &mut |s| {
                    globals += s
                        .reads
                        .iter()
                        .chain(&s.writes)
                        .filter(|a| a.scope == m::DataScope::Global)
                        .count();
                    tables += s.tables.len();
                    through += s.writes.iter().filter(|w| w.through.is_some()).count();
                });
            }
        }
        println!("{dir}: {} objects", roots.len());
    }
    println!(
        "all: {procedures} procedures, {effects} effects, {via} through calls, \
         {database} on database tables, {:.2} inputs per effect; {tables} table \
         accesses, {globals} global data accesses, {through} writes through field \
         symbols and references; {:.2?}, slowest {:.2?} {}",
        inputs as f64 / effects.max(1) as f64,
        started.elapsed(),
        slowest.0,
        slowest.1
    );
    assert!(procedures > 0);
}
