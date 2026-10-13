//! The CDS parser over the data definitions (`.ddls.asddls`) of the corpus:
//! every one gives its kind and name, and the elements of views come with
//! the fields they are made of.
//!
//! Run with `cargo test --release -p abap-sql --test cds_corpus -- --nocapture`.

use std::path::{Path, PathBuf};

fn files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            files(&p, out);
        } else if p.to_string_lossy().ends_with(".ddls.asddls") {
            out.push(p);
        }
    }
}

#[test]
fn corpus_cds_definitions() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/corpus");
    let mut paths = Vec::new();
    files(&root, &mut paths);
    let (mut views, mut elements, mut with_fields, mut sources, mut associations) = (0, 0, 0, 0, 0);
    for path in &paths {
        let text = std::fs::read_to_string(path).unwrap();
        let d = abap_sql::cds::parse(&text);
        let file = path.file_name().unwrap().to_string_lossy();
        assert!(!d.kind.is_empty() && !d.name.is_empty(), "{file}: {d:?}");
        let stem = file.split('.').next().unwrap();
        // an extension names the view it extends, the file the extension
        assert!(
            d.kind.starts_with("extend") || d.name.eq_ignore_ascii_case(&stem.replace('#', "/")),
            "{file}: {}",
            d.name
        );
        if d.kind.contains("view") && !d.kind.starts_with("extend") {
            views += 1;
            assert!(!d.sources.is_empty(), "{file}: no source");
            elements += d.elements.len();
            with_fields += d
                .elements
                .iter()
                .filter(|e| !e.fields.is_empty() || e.association)
                .count();
        }
        sources += d.sources.len();
        associations += d.associations.len();
    }
    println!(
        "{} definitions, {views} views: {sources} sources, {associations} associations, \
         {with_fields} of {elements} view elements with their fields",
        paths.len()
    );
}
