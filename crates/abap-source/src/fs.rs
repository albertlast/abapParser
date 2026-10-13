//! Source files on disk: abapGit layout (`zcl_x.clas.abap`,
//! `zcl_x.clas.locals_imp.abap`, `zgrp.fugr.z_fm.abap`, `#ns#name` for
//! namespaces), or plain `name.abap` files taken as programs; dictionary
//! types from `name.tabl.xml` and `name.dtel.xml`, CDS data definitions from
//! `name.ddls.asddls`; BAdIs from enhancement spots and implementations
//! (`.enhs.xml`, `.enho.xml`) and classic ones (`.sxsd.xml`, `.sxci.xml`);
//! code of enhancement implementations from `.enho.<hash>.abap`.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use crate::badi::BadiIndex;
use crate::{
    hook_matches, Badi, ClassPart, Hook, ObjectKind, Result, Source, SourceProvider, SourceRef,
};

pub struct FsProvider {
    roots: Vec<PathBuf>,
    index: HashMap<SourceRef, PathBuf>,
    /// function group (upper case) -> function modules
    groups: HashMap<String, Vec<String>>,
    /// interface -> global classes implementing it, and superclass ->
    /// global classes inheriting from it, upper case; read from the class
    /// sources on first use
    relations: OnceLock<Relations>,
    /// abapGit files of BAdI definitions and implementations with their
    /// object type, read on first use
    enhancements: Vec<(String, PathBuf)>,
    badis: OnceLock<BadiIndex>,
    /// code files of enhancement implementations with the implementation,
    /// read on first use
    hook_files: Vec<(String, PathBuf)>,
    hooks: OnceLock<Vec<Hook>>,
}

/// abapGit object types holding BAdIs.
const BADI_TYPES: [&str; 4] = ["enhs", "enho", "sxsd", "sxci"];

#[derive(Default)]
struct Relations {
    implementers: HashMap<String, Vec<String>>,
    subclasses: HashMap<String, Vec<String>>,
}

impl FsProvider {
    /// Indexes all `.abap` files and dictionary types below the given folders.
    pub fn new(roots: Vec<PathBuf>) -> Result<Self> {
        let mut p = FsProvider {
            roots: roots.clone(),
            index: HashMap::new(),
            groups: HashMap::new(),
            relations: OnceLock::new(),
            enhancements: Vec::new(),
            badis: OnceLock::new(),
            hook_files: Vec::new(),
            hooks: OnceLock::new(),
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
            } else if path.extension().is_some_and(|e| {
                ["abap", "xml", "asddls"]
                    .iter()
                    .any(|x| e.eq_ignore_ascii_case(x))
            }) {
                self.add_file(&path);
            }
        }
        Ok(())
    }

    fn add_file(&mut self, path: &Path) {
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            return;
        };
        let lower = name.to_ascii_lowercase();
        if let Some(kind) = BADI_TYPES
            .iter()
            .find(|k| lower.ends_with(&format!(".{k}.xml")))
        {
            self.enhancements
                .push((kind.to_string(), path.to_path_buf()));
            return;
        }
        // name.enho.<hash>.abap: one position of an enhancement implementation
        if let [name, "enho", _, "abap"] = lower.split('.').collect::<Vec<_>>().as_slice() {
            self.hook_files.push((decode(name), path.to_path_buf()));
            return;
        }
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

/// The code without comments, statements separated by '.'.
fn code_of(text: &str) -> String {
    text.lines()
        .filter(|l| !l.starts_with('*'))
        .map(|l| l.split('"').next().unwrap_or_default())
        .collect::<Vec<_>>()
        .join(" ")
}

/// The superclass in the definition of a class, upper case.
pub(crate) fn superclass_of(text: &str, class: &str) -> Option<String> {
    for statement in code_of(text).split('.') {
        let words: Vec<&str> = statement.split_whitespace().collect();
        let defines = words.len() > 2
            && words[0].eq_ignore_ascii_case("CLASS")
            && words[1].eq_ignore_ascii_case(class)
            && words[2].eq_ignore_ascii_case("DEFINITION");
        if !defines {
            continue;
        }
        let at = words.windows(2).position(|w| {
            w[0].eq_ignore_ascii_case("INHERITING") && w[1].eq_ignore_ascii_case("FROM")
        })?;
        return words.get(at + 2).map(|s| s.to_ascii_uppercase());
    }
    None
}

impl FsProvider {
    fn badis(&self) -> &BadiIndex {
        self.badis.get_or_init(|| {
            let mut index = BadiIndex::default();
            for (kind, path) in &self.enhancements {
                if let Ok(xml) = std::fs::read_to_string(path) {
                    index.add(kind, &xml);
                }
            }
            index
        })
    }

    /// The code files of enhancement implementations; abapGit writes the
    /// position into the first line, `"Name: \PR:ZREPORT\EX:ZPOINT\EI`.
    fn hooks(&self) -> &[Hook] {
        self.hooks.get_or_init(|| {
            let mut out = Vec::new();
            for (enhancement, path) in &self.hook_files {
                let Ok(text) = std::fs::read_to_string(path) else {
                    continue;
                };
                let Some(position) = text
                    .lines()
                    .next()
                    .and_then(|l| l.trim().strip_prefix("\"Name:"))
                else {
                    continue;
                };
                let code: Vec<&str> = text
                    .lines()
                    .skip_while(|l| {
                        !l.trim_start()
                            .to_ascii_uppercase()
                            .starts_with("ENHANCEMENT ")
                    })
                    .collect();
                out.push(Hook {
                    enhancement: enhancement.clone(),
                    position: position.trim().to_string(),
                    code: code.join("\n"),
                    origin: path.display().to_string(),
                });
            }
            out
        })
    }

    fn relations(&self) -> &Relations {
        self.relations.get_or_init(|| {
            let mut index = Relations::default();
            for (r, path) in &self.index {
                if r.kind != ObjectKind::Clas || r.part.is_some() {
                    continue;
                }
                let Ok(text) = std::fs::read_to_string(path) else {
                    continue;
                };
                for i in interfaces_of(&text) {
                    index
                        .implementers
                        .entry(i)
                        .or_default()
                        .push(r.name.clone());
                }
                if let Some(s) = superclass_of(&text, &r.name) {
                    index.subclasses.entry(s).or_default().push(r.name.clone());
                }
            }
            for classes in index
                .implementers
                .values_mut()
                .chain(index.subclasses.values_mut())
            {
                classes.sort();
            }
            index
        })
    }
}

/// Interfaces named by the INTERFACES statements of a source, upper case.
pub(crate) fn interfaces_of(text: &str) -> Vec<String> {
    let code = code_of(text);
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
    if let Some(name) = lower.strip_suffix(".ddls.asddls") {
        return Some((SourceRef::data_definition(&decode(name)), None));
    }
    if let Some(stem) = lower.strip_suffix(".xml") {
        return match stem.split('.').collect::<Vec<_>>().as_slice() {
            [name, "tabl"] => Some((SourceRef::table(&decode(name)), None)),
            [name, "dtel"] => Some((SourceRef::data_element(&decode(name)), None)),
            _ => None,
        };
    }
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
        Ok(self
            .relations()
            .implementers
            .get(&interface.to_ascii_uppercase())
            .cloned()
            .unwrap_or_default())
    }

    fn subclasses(&self, class: &str) -> Result<Vec<String>> {
        Ok(self
            .relations()
            .subclasses
            .get(&class.to_ascii_uppercase())
            .cloned()
            .unwrap_or_default())
    }

    fn enhancements(&self, programs: &[String], function_modules: &[String]) -> Result<Vec<Hook>> {
        Ok(self
            .hooks()
            .iter()
            .filter(|h| hook_matches(h, programs, function_modules))
            .cloned()
            .collect())
    }

    fn badi(&self, name: &str) -> Result<Option<Badi>> {
        Ok(self.badis().by_name(name))
    }

    fn classic_badi(&self, interface: &str) -> Result<Option<Badi>> {
        Ok(self.badis().classic(interface))
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
        assert_eq!(p("zs_x.tabl.xml"), Some(("TABL:ZS_X".into(), None)));
        assert_eq!(p("#abc#d.dtel.xml"), Some(("DTEL:/ABC/D".into(), None)));
        assert_eq!(p("zcl_a.clas.xml"), None);
        assert_eq!(p("zi_x.ddls.asddls"), Some(("DDLS:ZI_X".into(), None)));
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
    fn enhancement_code_files() {
        let dir = std::env::temp_dir().join(format!("abap-fs-hooks-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("zenh_orders.enho.1a2b3c4d.abap"),
            "\"Name: \\PR:ZORDERS\\FO:SAVE\\SE:END\\EI\nENHANCEMENT 0 ZENH_ORDERS.\n  x = 1.\nENDENHANCEMENT.\n",
        )
        .unwrap();
        std::fs::write(
            dir.join("zenh_orders.enho.5e6f7a8b.abap"),
            "\"Name: \\FU:Z_POST\\SE:BEGIN\\EI\nENHANCEMENT 0 ZENH_ORDERS.\nENDENHANCEMENT.\n",
        )
        .unwrap();
        std::fs::write(
            dir.join("#abc#enh.enho.11111111.abap"),
            "\"Name: \\TY:/ABC/CL_X\\ME:RUN\\SE:BEGIN\\EI\nENHANCEMENT 0 /ABC/ENH.\nENDENHANCEMENT.\n",
        )
        .unwrap();
        let p = FsProvider::new(vec![dir.clone()]).unwrap();
        // the code files are no objects of their own
        assert!(p.is_empty());
        let hooks = p.enhancements(&["ZORDERS".into()], &[]).unwrap();
        assert_eq!(hooks.len(), 1);
        assert_eq!(hooks[0].enhancement, "ZENH_ORDERS");
        assert_eq!(hooks[0].position, r"\PR:ZORDERS\FO:SAVE\SE:END\EI");
        assert_eq!(
            hooks[0].code,
            "ENHANCEMENT 0 ZENH_ORDERS.\n  x = 1.\nENDENHANCEMENT."
        );
        let hooks = p.enhancements(&[], &["Z_POST".into()]).unwrap();
        assert_eq!(hooks.len(), 1);
        // a class by its class pool
        let pool = crate::class_pool("/ABC/CL_X");
        assert_eq!(pool, "/ABC/CL_X=====================CP");
        let hooks = p.enhancements(&[pool], &[]).unwrap();
        assert_eq!(hooks[0].enhancement, "/ABC/ENH");
        let _ = std::fs::remove_dir_all(&dir);
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

    #[test]
    fn superclass_of_the_definition() {
        let src = "\"! docs
CLASS zcl_x DEFINITION
  PUBLIC
  INHERITING FROM /abc/cl_base
  FINAL.
  PUBLIC SECTION.
ENDCLASS.
CLASS lcl_local DEFINITION INHERITING FROM zcl_other.
ENDCLASS.";
        assert_eq!(superclass_of(src, "ZCL_X").as_deref(), Some("/ABC/CL_BASE"));
        assert_eq!(
            superclass_of(src, "lcl_local").as_deref(),
            Some("ZCL_OTHER")
        );
        assert_eq!(
            superclass_of("CLASS zcl_y DEFINITION PUBLIC.", "zcl_y"),
            None
        );
    }
}
