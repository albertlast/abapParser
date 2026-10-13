//! AMDP methods of the corpus: ABAP requires every database table, view and
//! AMDP method used in the SQLScript body to be listed after USING, so the
//! tables and calls found by the HANA SQL parser must all be listed there,
//! and every listed one should be found.
//!
//! Run with `cargo test --release -p abap-analysis --test amdp_corpus -- --nocapture`.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use abap_model as m;

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
fn amdp_bodies_use_what_using_lists() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/corpus");
    let mut files = Vec::new();
    abap_files(&root, &mut files);
    let (mut methods, mut listed, mut found) = (0, 0, 0);
    let mut problems: Vec<String> = Vec::new();
    for f in &files {
        let source = std::fs::read_to_string(f).unwrap();
        if !source.to_ascii_uppercase().contains("BY DATABASE") {
            continue;
        }
        let symbols = abap_analysis::symbols(&source);
        let accesses = abap_analysis::db_access(&source);
        let calls = abap_analysis::calls(&source);
        for class in &symbols.classes {
            let Some(implementation) = &class.implementation else {
                continue;
            };
            for method in &implementation.methods {
                let Some(amdp) = &method.amdp else { continue };
                methods += 1;
                let in_method = |b: &Option<m::BlockRef>| {
                    b.as_ref().is_some_and(|b| {
                        b.name.eq_ignore_ascii_case(&method.name)
                            && b.class
                                .as_deref()
                                .is_some_and(|c| c.eq_ignore_ascii_case(&class.name))
                    })
                };
                let using: BTreeSet<String> =
                    amdp.using.iter().map(|u| u.to_ascii_uppercase()).collect();
                let mut used = BTreeSet::new();
                let mine: Vec<&m::DbAccess> =
                    accesses.iter().filter(|a| in_method(&a.block)).collect();
                for a in mine {
                    for t in &a.tables {
                        if matches!(t.role, m::DbTableRole::InternalTable | m::DbTableRole::Cte) {
                            continue;
                        }
                        used.insert(t.name.to_ascii_uppercase());
                    }
                }
                for c in calls
                    .iter()
                    .filter(|c| in_method(&c.from) && c.text.contains("=>"))
                {
                    used.insert(
                        format!("{}=>{}", c.class.as_deref().unwrap_or_default(), c.name)
                            .to_ascii_uppercase(),
                    );
                }
                let file = f.file_name().unwrap().to_string_lossy();
                for u in used.difference(&using) {
                    problems.push(format!("{file} {}: uses {u}, not in USING", method.name));
                }
                for u in using.difference(&used) {
                    problems.push(format!("{file} {}: USING {u} not found", method.name));
                }
                listed += using.len();
                found += using.intersection(&used).count();
            }
        }
    }
    for p in &problems {
        println!("    {p}");
    }
    println!("{methods} AMDP methods, {found} of {listed} USING entries found in the bodies");
    assert!(problems.is_empty(), "{} differences", problems.len());
}
