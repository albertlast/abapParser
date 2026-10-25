//! Speed of each stage over the test corpus, to compare versions on the
//! same input and machine: every `.abap` file of `tests/corpus/corpora.txt`
//! through parse, symbols, calls, database access and logic, one after the
//! other on one thread; then per corpus the call graph and the logic of each
//! object through the folder provider, which loads objects several at once.
//! Each stage takes the best of the runs.
//!
//! Run with `cargo run --release -p abap-analysis --example bench -- [root] [runs]`,
//! `root` the repository with the corpus (default `.`), `runs` 3 by default.
//! `node tools/bench.mjs` runs it for several versions and compares them.
//!
//! Only API that every release since 0.3.0 has is used, so the file can be
//! copied into an older checkout and built there.

use std::hint::black_box;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use abap_analysis::abap_source::{FsProvider, ObjectKind, SourceRef};
use abap_analysis::Workspace;

fn abap_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            abap_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "abap") {
            out.push(path);
        }
    }
}

/// The shortest of `runs` runs.
fn best(runs: usize, mut f: impl FnMut()) -> Duration {
    (0..runs.max(1))
        .map(|_| {
            let started = Instant::now();
            f();
            started.elapsed()
        })
        .min()
        .unwrap_or_default()
}

/// The objects a corpus test starts from: classes, programs, function groups.
fn roots(provider: &FsProvider) -> Vec<SourceRef> {
    let mut roots: Vec<SourceRef> = provider
        .references()
        .filter(|r| {
            matches!(
                r.kind,
                ObjectKind::Clas | ObjectKind::Prog | ObjectKind::Fugr
            ) && r.part.is_none()
        })
        .cloned()
        .collect();
    roots.sort_by_key(|r| r.to_string());
    roots
}

fn main() {
    let mut args = std::env::args().skip(1);
    let root = PathBuf::from(args.next().unwrap_or_else(|| ".".to_string()));
    let runs: usize = args.next().and_then(|r| r.parse().ok()).unwrap_or(3);
    let list = std::fs::read_to_string(root.join("tests/corpus/corpora.txt"))
        .expect("tests/corpus/corpora.txt below the root");
    // one corpus per line, several folders of one corpus on a line
    let corpora: Vec<Vec<PathBuf>> = list
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(|l| l.split_whitespace().map(|d| root.join(d)).collect())
        .filter(|c: &Vec<PathBuf>| c.iter().all(|d| d.exists()))
        .collect();
    let mut paths = Vec::new();
    for corpus in &corpora {
        for dir in corpus {
            abap_files(dir, &mut paths);
        }
    }
    let texts: Vec<String> = paths
        .iter()
        .map(|p| String::from_utf8_lossy(&std::fs::read(p).unwrap()).into_owned())
        .collect();
    let bytes: usize = texts.iter().map(String::len).sum();
    let lines: usize = texts.iter().map(|t| t.lines().count()).sum();
    println!(
        "input\t{} corpora\t{} files\t{lines} lines\t{:.1} MB\t{runs} runs",
        corpora.len(),
        texts.len(),
        bytes as f64 / 1e6
    );
    let report = |stage: &str, d: Duration| {
        println!(
            "{stage}\t{:.0} ms\t{:.0}k lines/s",
            d.as_secs_f64() * 1000.0,
            lines as f64 / d.as_secs_f64() / 1000.0
        );
    };

    report(
        "parse",
        best(runs, || {
            texts.iter().for_each(|t| {
                black_box(abap_syntax::parse(t));
            })
        }),
    );
    report(
        "symbols",
        best(runs, || {
            texts.iter().for_each(|t| {
                black_box(abap_analysis::symbols(t));
            })
        }),
    );
    report(
        "calls",
        best(runs, || {
            texts.iter().for_each(|t| {
                black_box(abap_analysis::calls(t));
            })
        }),
    );
    report(
        "db_access",
        best(runs, || {
            texts.iter().for_each(|t| {
                black_box(abap_analysis::db_access(t));
            })
        }),
    );
    report(
        "logic",
        best(runs, || {
            texts.iter().for_each(|t| {
                black_box(abap_analysis::logic(t));
            })
        }),
    );
    report(
        "call_graph",
        best(runs, || {
            for corpus in &corpora {
                let provider = FsProvider::new(corpus.clone()).unwrap();
                let roots = roots(&provider);
                let workspace = Workspace::new(Box::new(provider));
                black_box(workspace.call_graph(&roots, 1));
            }
        }),
    );
    report(
        "logic_objects",
        best(runs, || {
            for corpus in &corpora {
                let provider = FsProvider::new(corpus.clone()).unwrap();
                let roots = roots(&provider);
                let workspace = Workspace::new(Box::new(provider));
                for r in &roots {
                    black_box(workspace.logic(r, 0));
                }
            }
        }),
    );
}
