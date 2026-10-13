//! Runs every analysis over every corpus file: nothing may panic.

use std::path::{Path, PathBuf};

fn abap_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            abap_files(&p, out);
        } else if p.extension().is_some_and(|x| x == "abap") {
            out.push(p);
        }
    }
}

#[test]
fn analyses_do_not_panic() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/corpus");
    let mut files = Vec::new();
    abap_files(&root, &mut files);
    let (mut accesses, mut procedures, mut reads, mut traced) = (0, 0, 0, 0);
    let started = std::time::Instant::now();
    let mut slowest = (std::time::Duration::ZERO, PathBuf::new());
    for f in &files {
        let source = std::fs::read_to_string(f).unwrap();
        let result = std::panic::catch_unwind(|| {
            let _ = abap_analysis::symbols(&source);
            let _ = abap_analysis::statements(&source);
            let t = std::time::Instant::now();
            let logic = abap_analysis::logic(&source);
            (abap_analysis::db_access(&source).len(), logic, t.elapsed())
        });
        let Ok((n, logic, elapsed)) = result else {
            panic!("analysis panicked on {}", f.display());
        };
        accesses += n;
        if elapsed > slowest.0 {
            slowest = (elapsed, f.clone());
        }
        procedures += logic.procedures.len();
        let mut stack: Vec<&abap_model::Step> =
            logic.procedures.iter().flat_map(|p| &p.body).collect();
        while let Some(s) = stack.pop() {
            reads += s.reads.len();
            traced += s
                .reads
                .iter()
                .filter(|r| r.from_entry || !r.sources.is_empty())
                .count();
            stack.extend(&s.children);
        }
    }
    println!(
        "{} files, {accesses} database accesses, {procedures} procedures, \
         {reads} reads, {traced} with a source, in {:.2?}; slowest logic {:.2?} {}",
        files.len(),
        started.elapsed(),
        slowest.0,
        slowest.1.display()
    );
}
