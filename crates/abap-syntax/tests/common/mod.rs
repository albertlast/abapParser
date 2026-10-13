#![allow(dead_code)]

use std::path::{Path, PathBuf};

pub fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
}

pub fn json_files(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir) else {
        return out;
    };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            out.extend(json_files(&p));
        } else if p.extension().is_some_and(|x| x == "json") {
            out.push(p);
        }
    }
    out.sort();
    out
}

pub fn percent(part: usize, total: usize) -> f64 {
    if total == 0 {
        100.0
    } else {
        100.0 * part as f64 / total as f64
    }
}

/// Macros defined in the other files of the same abapGit object, e.g. the
/// macros include of a class or the TOP include of a function group.
pub fn sibling_macros(path: &std::path::Path) -> Vec<abap_syntax::MacroDef> {
    let name = path
        .file_name()
        .unwrap()
        .to_string_lossy()
        .to_ascii_lowercase();
    let object: Vec<&str> = name.split('.').take(2).collect();
    let mut out = Vec::new();
    let Ok(entries) = std::fs::read_dir(path.parent().unwrap()) else {
        return out;
    };
    for e in entries.flatten() {
        let other = e.file_name().to_string_lossy().to_ascii_lowercase();
        let same_object = other.split('.').take(2).collect::<Vec<_>>() == object;
        if other == name || !same_object || !other.ends_with(".abap") {
            continue;
        }
        if let Ok(text) = std::fs::read_to_string(e.path()) {
            if text.to_ascii_uppercase().contains("DEFINE ") {
                out.extend(abap_syntax::parse(&text).macros());
            }
        }
    }
    out
}
