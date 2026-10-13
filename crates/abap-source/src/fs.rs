//! Source files on disk: abapGit layout (`zcl_x.clas.abap`,
//! `zcl_x.clas.locals_imp.abap`, `zgrp.fugr.z_fm.abap`, `#ns#name` for
//! namespaces), or plain `name.abap` files taken as programs.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use crate::{ClassPart, ObjectKind, Result, Source, SourceProvider, SourceRef};

pub struct FsProvider {
    roots: Vec<PathBuf>,
    index: HashMap<SourceRef, PathBuf>,
    /// function group (upper case) -> function modules
    groups: HashMap<String, Vec<String>>,
    /// interface -> global classes implementing it, upper case; read from
    /// the class sources on first use
    implementers: OnceLock<HashMap<String, Vec<String>>>,
}

impl FsProvider {
    /// Indexes all `.abap` files below the given folders.
    pub fn new(roots: Vec<PathBuf>) -> Result<Self> {
        let mut p = FsProvider {
            roots: roots.clone(),
            index: HashMap::new(),
            groups: HashMap::new(),
            implementers: OnceLock::new(),
        };
        for root in &roots {
            p.scan(root)?;
        }
        Ok(p)
    }

    pub fn len(&self) -> usize {
        self.index.len()
    }

    pub fn is_empty(&self) -> bool {
        self.index.is_empty()
    }

    /// All indexed objects.
    pub fn references(&self) -> impl Iterator<Item = &SourceRef> {
        self.index.keys()
    }

    fn scan(&mut self, dir: &Path) -> Result<()> {
        if dir.is_file() {
            self.add_file(dir);
            return Ok(());
        }
        for entry in std::fs::read_dir(dir)? {
            let path = entry?.path();
            let hidden = path
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with('.') || n == "node_modules" || n == "target");
            if path.is_dir() && !hidden {
                self.scan(&path)?;
            } else if path
                .extension()
                .is_some_and(|e| e.eq_ignore_ascii_case("abap"))
            {
                self.add_file(&path);
            }
        }
        Ok(())
    }

    fn add_file(&mut self, path: &Path) {
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            return;
        };
        if let Some((reference, group)) = parse_file_name(name) {
            if let Some(group) = group {
                self.groups
                    .entry(group)
                    .or_default()
                    .push(reference.name.clone());
            }
            // a later root does not override an earlier one
            self.index
                .entry(reference)
                .or_insert_with(|| path.to_path_buf());
        }
    }
}

/// Interfaces named by the INTERFACES statements of a source, upper case.
fn interfaces_of(text: &str) -> Vec<String> {
    let code: String = text
        .lines()
        .filter(|l| !l.starts_with('*'))
        .map(|l| l.split('"').next().unwrap_or_default())
        .collect::<Vec<_>>()
        .join(" ");
    let mut out = Vec::new();
    for statement in code.split('.') {
        let statement = statement.trim();
        let Some(rest) = statement
            .get(..10)
            .filter(|k| k.eq_ignore_ascii_case("INTERFACES"))
            .map(|_| &statement[10..])
        else {
            continue;
        };
        if !rest.starts_with(|c: char| c == ':' || c.is_whitespace()) {
            continue;
        }
        for part in rest.trim_start_matches([' ', ':']).split(',') {
            if let Some(name) = part.split_whitespace().next() {
                let name = name.trim_start_matches(':').to_ascii_uppercase();
                if !name.is_empty() && !out.contains(&name) {
                    out.push(name);
                }
            }
        }
    }
    out
}

/// abapGit encodes '/' of namespaces as '#'.
fn decode(name: &str) -> String {
    name.replace('#', "/").to_ascii_uppercase()
}

/// Strips a namespace prefix like /ABC/.
fn local_name(name: &str) -> &str {
    if let Some(rest) = name.strip_prefix('/') {
        if let Some(p) = rest.find('/') {
            return &rest[p + 1..];
        }
    }
    name
}

/// The object of a file name, and for function modules their group.
fn parse_file_name(file: &str) -> Option<(SourceRef, Option<String>)> {
    let lower = file.to_ascii_lowercase();
    let stem = lower.strip_suffix(".abap")?;
    let parts: Vec<&str> = stem.split('.').collect();
    let r = match parts.as_slice() {
        [name] => SourceRef::program(&decode(name)),
        [name, "prog"] => SourceRef::program(&decode(name)),
        [name, "clas"] => SourceRef::class(&decode(name)),
        [name, "clas", part] => {
            let part = match *part {
                "locals_def" => ClassPart::Definitions,
                "locals_imp" => ClassPart::Implementations,
                "macros" => ClassPart::Macros,
                "testclasses" => ClassPart::TestClasses,
                _ => return None,
            };
            SourceRef::class(&decode(name)).with_part(part)
        }
        [name, "intf"] => SourceRef::interface(&decode(name)),
        [group, "fugr", include] => {
            let group = decode(group);
            let include = decode(include);
            let local_group = local_name(&group);
            let local_include = local_name(&include);
            if local_include == format!("SAPL{local_group}") {
                SourceRef::function_group(&group)
            } else if local_include.starts_with(&format!("L{local_group}")) {
                SourceRef::program(&include)
            } else {
                return Some((SourceRef::function_module(&include), Some(group)));
            }
        }
        _ => return None,
    };
    Some((r, None))
}

impl SourceProvider for FsProvider {
    fn fetch(&self, reference: &SourceRef) -> Result<Option<Source>> {
        let mut path = self.index.get(reference);
        // the main program of a function group is also a program
        if path.is_none() && reference.kind == ObjectKind::Prog {
            let local = local_name(&reference.name);
            if let Some(group) = local.strip_prefix("SAPL") {
                let prefix = &reference.name[..reference.name.len() - local.len()];
                path = self
                    .index
                    .get(&SourceRef::function_group(&format!("{prefix}{group}")));
            }
        }
        let Some(path) = path else {
            return Ok(None);
        };
        let bytes = std::fs::read(path)?;
        let text = String::from_utf8_lossy(&bytes).into_owned();
        Ok(Some(Source {
            reference: reference.clone(),
            text,
            origin: path.display().to_string(),
        }))
    }

    fn function_modules(&self, group: &str) -> Result<Vec<String>> {
        let mut fms = self
            .groups
            .get(&group.to_ascii_uppercase())
            .cloned()
            .unwrap_or_default();
        fms.sort();
        Ok(fms)
    }

    fn implementations(&self, interface: &str) -> Result<Vec<String>> {
        let index = self.implementers.get_or_init(|| {
            let mut index: HashMap<String, Vec<String>> = HashMap::new();
            for (r, path) in &self.index {
                if r.kind != ObjectKind::Clas || r.part.is_some() {
                    continue;
                }
                let Ok(text) = std::fs::read_to_string(path) else {
                    continue;
                };
                for i in interfaces_of(&text) {
                    index.entry(i).or_default().push(r.name.clone());
                }
            }
            for classes in index.values_mut() {
                classes.sort();
            }
            index
        });
        Ok(index
            .get(&interface.to_ascii_uppercase())
            .cloned()
            .unwrap_or_default())
    }

    fn describe(&self) -> String {
        self.roots
            .iter()
            .map(|r| r.display().to_string())
            .collect::<Vec<_>>()
            .join(", ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn abapgit_file_names() {
        let p = |f: &str| parse_file_name(f).map(|(r, g)| (r.to_string(), g));
        assert_eq!(p("zreport.prog.abap"), Some(("PROG:ZREPORT".into(), None)));
        assert_eq!(p("zcl_a.clas.abap"), Some(("CLAS:ZCL_A".into(), None)));
        assert_eq!(
            p("zcl_a.clas.locals_imp.abap"),
            Some(("CLAS:ZCL_A/implementations".into(), None))
        );
        assert_eq!(
            p("#abc#cl_x.clas.testclasses.abap"),
            Some(("CLAS:/ABC/CL_X/testclasses".into(), None))
        );
        assert_eq!(p("zif_a.intf.abap"), Some(("INTF:ZIF_A".into(), None)));
        assert_eq!(
            p("zgrp.fugr.saplzgrp.abap"),
            Some(("FUGR:ZGRP".into(), None))
        );
        assert_eq!(
            p("zgrp.fugr.lzgrptop.abap"),
            Some(("PROG:LZGRPTOP".into(), None))
        );
        assert_eq!(
            p("zgrp.fugr.z_do_it.abap"),
            Some(("FUNC:Z_DO_IT".into(), Some("ZGRP".into())))
        );
        assert_eq!(
            p("#abc#grp.fugr.#abc#lgrptop.abap"),
            Some(("PROG:/ABC/LGRPTOP".into(), None))
        );
        assert_eq!(p("ZPLAIN.abap"), Some(("PROG:ZPLAIN".into(), None)));
        assert_eq!(p("zcl_a.clas.xml"), None);
    }

    #[test]
    fn corpus_folder() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/corpus/abapgit/src");
        if !root.exists() {
            return;
        }
        let p = FsProvider::new(vec![root]).unwrap();
        assert!(p.len() > 500);
        let s = p
            .fetch(&SourceRef::class("ZCL_ABAPGIT_APACK_MIGRATION"))
            .unwrap()
            .unwrap();
        assert!(s
            .text
            .to_ascii_uppercase()
            .contains("CLASS ZCL_ABAPGIT_APACK_MIGRATION"));
        let fms = p.function_modules("ZABAPGIT_PARALLEL").unwrap();
        assert!(fms.contains(&"Z_ABAPGIT_SERIALIZE_PARALLEL".to_string()));
        let main = p
            .fetch(&SourceRef::program("SAPLZABAPGIT_PARALLEL"))
            .unwrap();
        assert!(main.is_some());
        let objects = p.implementations("zif_abapgit_object").unwrap();
        assert!(objects.len() > 50, "{objects:?}");
        assert!(objects.contains(&"ZCL_ABAPGIT_OBJECT_CLAS".to_string()));
    }

    #[test]
    fn interfaces_statements() {
        let src = "CLASS zcl_x DEFINITION PUBLIC.
  PUBLIC SECTION.
    INTERFACES zif_a.
    \"INTERFACES zif_comment.
*   INTERFACES zif_star.
    INTERFACES: zif_b, /abc/if_c ALL METHODS ABSTRACT.
    INTERFACES zif_d DATA VALUES x = 1.
ENDCLASS.";
        assert_eq!(interfaces_of(src), ["ZIF_A", "ZIF_B", "/ABC/IF_C", "ZIF_D"]);
    }
}
