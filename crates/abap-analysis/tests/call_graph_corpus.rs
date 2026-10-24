//! Call graph over every corpus of tests/corpus/corpora.txt, each through the
//! folder provider: how many method calls get a receiver type, and how many
//! calls into classes of the same corpus resolve to an existing method.
//!
//! Run with `cargo test --release -p abap-analysis --test call_graph_corpus -- --nocapture`.

use std::collections::BTreeSet;
use std::path::Path;

use abap_analysis::abap_source::{FsProvider, ObjectKind};
use abap_analysis::Workspace;
use abap_model as m;

/// Baselines over all corpora, in percent; raise when improving. The HR
/// corpora call many SAP standard classes that are not in the sources.
const MIN_TYPED: f64 = 96.9;
const MIN_RESOLVED_IN_CORPUS: f64 = 99.9;

fn pct(a: usize, b: usize) -> f64 {
    100.0 * a as f64 / b.max(1) as f64
}

#[test]
fn corpus_call_graphs() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let list = std::fs::read_to_string(root.join("tests/corpus/corpora.txt")).unwrap();
    let (mut methods_total, mut typed_total, mut own_total, mut resolved_total) = (0, 0, 0, 0);

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
        // classes and interfaces of this corpus
        let types: BTreeSet<String> = provider
            .references()
            .filter(|r| matches!(r.kind, ObjectKind::Clas | ObjectKind::Intf))
            .map(|r| r.name.clone())
            .collect();
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
        let started = std::time::Instant::now();
        let graph = workspace.call_graph(&roots, 1);
        let elapsed = started.elapsed();
        assert!(graph.errors.is_empty(), "{dir}: {:?}", graph.errors);

        let methods: Vec<&m::Call> = graph
            .calls
            .iter()
            .filter(|c| c.kind == m::CallKind::Method && !c.dynamic)
            .collect();
        let typed = methods.iter().filter(|c| c.class.is_some()).count();
        // calls into the corpus whose class hierarchy is complete; methods of
        // SAP standard superclasses cannot be checked
        let own: Vec<&&m::Call> = methods
            .iter()
            .filter(|c| {
                c.class
                    .as_deref()
                    .is_some_and(|x| types.contains(&x.to_ascii_uppercase()))
            })
            .filter(|c| !c.hierarchy_incomplete)
            .collect();
        let resolved = own.iter().filter(|c| c.resolved).count();
        println!(
            "{dir}: {} objects, {} calls in {elapsed:.2?}; method calls {}, typed {:.1}%, into the corpus {} resolved {:.1}%",
            roots.len(),
            graph.calls.len(),
            methods.len(),
            pct(typed, methods.len()),
            own.len(),
            pct(resolved, own.len()),
        );
        for c in own.iter().filter(|c| !c.resolved).take(3) {
            println!(
                "    unresolved {}.{} | {}",
                c.class.as_deref().unwrap_or(""),
                c.name,
                c.text
            );
        }
        methods_total += methods.len();
        typed_total += typed;
        own_total += own.len();
        resolved_total += resolved;
    }

    println!(
        "all: method calls {methods_total}, typed {:.1}%, into the corpus {own_total} resolved {:.1}%",
        pct(typed_total, methods_total),
        pct(resolved_total, own_total)
    );
    assert!(pct(typed_total, methods_total) >= MIN_TYPED);
    assert!(pct(resolved_total, own_total) >= MIN_RESOLVED_IN_CORPUS);
}
