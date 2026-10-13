//! Differential test of the symbols against abaplint's file information.
//!
//! Needs the golden data of tools/oracle.mjs, see crates/abap-syntax/tests/oracle_diff.rs.
//! Run with `cargo test -p abap-analysis --test symbols_diff -- --nocapture` for the report.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use abap_model as m;
use serde::Deserialize;

/// Minimum agreement per category, in percent; raise when improving.
const MIN_PERCENT: f64 = 99.9;

#[derive(Deserialize)]
struct Golden {
    file: String,
    statements: Vec<GStatement>,
    info: Info,
}

#[derive(Deserialize)]
struct GStatement {
    #[serde(rename = "type")]
    kind: String,
}

#[derive(Deserialize)]
struct Info {
    interfaces: Vec<GInterface>,
    classes: Vec<GClass>,
    implementations: Vec<GImplementation>,
    forms: Vec<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GInterface {
    name: String,
    is_global: bool,
    interfaces: Vec<String>,
    methods: Vec<GMethod>,
    aliases: Vec<GAlias>,
    attributes: Vec<GAttribute>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GClass {
    #[serde(flatten)]
    base: GInterface,
    super_class_name: Option<String>,
    is_abstract: bool,
    is_final: bool,
    is_for_testing: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GMethod {
    name: String,
    visibility: String,
    is_redefinition: bool,
    is_event_handler: bool,
    is_for_testing: bool,
    is_abstract: bool,
    parameters: Vec<GParameter>,
}

#[derive(Deserialize)]
struct GParameter {
    name: String,
    direction: String,
}

#[derive(Deserialize)]
struct GAlias {
    name: String,
    component: String,
    visibility: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GAttribute {
    name: String,
    level: String,
    read_only: bool,
    visibility: String,
}

#[derive(Deserialize)]
struct GImplementation {
    name: String,
    methods: Vec<String>,
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
}

fn json_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            json_files(&p, out);
        } else if p.extension().is_some_and(|x| x == "json") {
            out.push(p);
        }
    }
}

fn lc(s: &str) -> String {
    s.to_ascii_lowercase()
}

fn visibility(v: m::Visibility) -> &'static str {
    match v {
        m::Visibility::Public => "public",
        m::Visibility::Protected => "protected",
        m::Visibility::Private => "private",
    }
}

fn direction(d: m::Direction) -> &'static str {
    match d {
        m::Direction::Importing => "importing",
        m::Direction::Exporting => "exporting",
        m::Direction::Changing => "changing",
        m::Direction::Returning => "returning",
        m::Direction::Using => "using",
        m::Direction::Tables => "tables",
    }
}

#[derive(Default)]
struct Report {
    /// category -> (compared, equal, mismatches)
    categories: BTreeMap<&'static str, (usize, usize, Vec<String>)>,
}

impl Report {
    fn check<T: PartialEq + std::fmt::Debug>(
        &mut self,
        category: &'static str,
        at: &str,
        ours: T,
        abaplint: T,
    ) {
        let entry = self.categories.entry(category).or_default();
        entry.0 += 1;
        if ours == abaplint {
            entry.1 += 1;
        } else {
            entry
                .2
                .push(format!("{at}: ours {ours:?}, abaplint {abaplint:?}"));
        }
    }
}

/// What both sides know about a class or interface.
struct Members<'a> {
    interfaces: Vec<String>,
    methods: Vec<&'a m::Method>,
    aliases: Vec<&'a m::Alias>,
    attributes: Vec<&'a m::Declaration>,
}

fn compare_members(r: &mut Report, at: &str, ours: Members, g: &GInterface) {
    let mut ours_intf: Vec<_> = ours.interfaces.iter().map(|n| lc(n)).collect();
    let mut g_intf: Vec<_> = g.interfaces.iter().map(|n| lc(n)).collect();
    ours_intf.sort();
    g_intf.sort();
    r.check("implemented interfaces", at, ours_intf, g_intf);

    let ours_methods: BTreeMap<String, &m::Method> =
        ours.methods.iter().map(|m| (lc(&m.name), *m)).collect();
    let g_methods: BTreeMap<String, &GMethod> =
        g.methods.iter().map(|m| (lc(&m.name), m)).collect();
    r.check(
        "method names",
        at,
        ours_methods.keys().collect::<Vec<_>>(),
        g_methods.keys().collect::<Vec<_>>(),
    );
    for (name, gm) in &g_methods {
        let Some(om) = ours_methods.get(name) else {
            continue;
        };
        let at = format!("{at} {name}");
        if om.rap_handler.is_some() || om.event_handler.is_some() {
            // abaplint lists no parameters for RAP and event handler methods
            continue;
        }
        r.check(
            "method flags",
            &at,
            (
                visibility(om.visibility),
                om.is_redefinition,
                om.event_handler.is_some(),
                om.is_for_testing,
                om.is_abstract,
            ),
            (
                gm.visibility.as_str(),
                gm.is_redefinition,
                gm.is_event_handler,
                gm.is_for_testing,
                gm.is_abstract,
            ),
        );
        r.check(
            "method parameters",
            &at,
            om.parameters
                .iter()
                .map(|p| (lc(&p.name), direction(p.direction).to_string()))
                .collect::<Vec<_>>(),
            gm.parameters
                .iter()
                .map(|p| (lc(&p.name), p.direction.clone()))
                .collect::<Vec<_>>(),
        );
    }

    r.check(
        "aliases",
        at,
        ours.aliases
            .iter()
            .map(|a| {
                (
                    lc(&a.name),
                    lc(&a.component),
                    visibility(a.visibility).to_string(),
                )
            })
            .collect::<Vec<_>>(),
        g.aliases
            .iter()
            .map(|a| (lc(&a.name), lc(&a.component), a.visibility.clone()))
            .collect::<Vec<_>>(),
    );

    let mut ours_attr: Vec<_> = ours
        .attributes
        .iter()
        // abaplint leaves BEGIN OF ... END OF structures out of the attributes
        .filter(|a| !a.is_structure)
        .map(|a| {
            let level = match a.kind {
                m::DeclarationKind::ClassData => "static",
                m::DeclarationKind::Constant => "constant",
                _ => "instance",
            };
            (
                lc(&a.name),
                level.to_string(),
                a.read_only,
                a.visibility.map(visibility).unwrap_or("").to_string(),
            )
        })
        .collect();
    let mut g_attr: Vec<_> = g
        .attributes
        .iter()
        .map(|a| {
            (
                lc(&a.name),
                a.level.clone(),
                a.read_only,
                a.visibility.clone(),
            )
        })
        .collect();
    ours_attr.sort();
    g_attr.sort();
    r.check("attributes", at, ours_attr, g_attr);
}

#[test]
fn symbols_diff() {
    let root = repo_root();
    let mut files = Vec::new();
    json_files(&root.join("tests/corpus/golden/tests"), &mut files);
    files.sort();
    if files.is_empty() {
        println!("skipped: no golden data");
        return;
    }

    let mut r = Report::default();
    let mut skipped = 0;
    for f in &files {
        let golden: Golden = serde_json::from_str(&std::fs::read_to_string(f).unwrap()).unwrap();
        // abaplint drops symbols around statements it cannot parse
        if golden.statements.iter().any(|s| s.kind == "Unknown") {
            skipped += 1;
            continue;
        }
        let source = std::fs::read_to_string(root.join(&golden.file)).unwrap();
        let s = abap_analysis::symbols(&source);
        let file = golden.file.rsplit('/').next().unwrap().to_string();

        let mut ours_classes: Vec<_> = s.classes.iter().map(|c| lc(&c.name)).collect();
        let mut g_classes: Vec<_> = golden
            .info
            .classes
            .iter()
            .map(|c| lc(&c.base.name))
            .collect();
        ours_classes.sort();
        g_classes.sort();
        r.check("class names", &file, ours_classes, g_classes);

        for gc in &golden.info.classes {
            let Some(c) = s
                .classes
                .iter()
                .find(|c| c.name.eq_ignore_ascii_case(&gc.base.name))
            else {
                continue;
            };
            let at = format!("{file} {}", lc(&c.name));
            r.check(
                "class flags",
                &at,
                (
                    c.superclass.as_deref().map(lc),
                    c.is_global,
                    c.is_final,
                    c.is_abstract,
                    c.is_for_testing,
                ),
                (
                    gc.super_class_name.as_deref().map(lc),
                    gc.base.is_global,
                    gc.is_final,
                    gc.is_abstract,
                    gc.is_for_testing,
                ),
            );
            let members = Members {
                interfaces: c.interfaces.iter().map(|i| i.name.clone()).collect(),
                methods: c.methods.iter().collect(),
                aliases: c.aliases.iter().collect(),
                attributes: c.attributes.iter().collect(),
            };
            compare_members(&mut r, &at, members, &gc.base);
        }

        let mut ours_intf: Vec<_> = s.interfaces.iter().map(|c| lc(&c.name)).collect();
        let mut g_intf: Vec<_> = golden.info.interfaces.iter().map(|c| lc(&c.name)).collect();
        ours_intf.sort();
        g_intf.sort();
        r.check("interface names", &file, ours_intf, g_intf);
        for gi in &golden.info.interfaces {
            let Some(i) = s
                .interfaces
                .iter()
                .find(|i| i.name.eq_ignore_ascii_case(&gi.name))
            else {
                continue;
            };
            let at = format!("{file} {}", lc(&i.name));
            r.check("interface flags", &at, i.is_global, gi.is_global);
            let members = Members {
                interfaces: i.interfaces.iter().map(|i| i.name.clone()).collect(),
                methods: i.methods.iter().collect(),
                aliases: i.aliases.iter().collect(),
                attributes: i.attributes.iter().collect(),
            };
            compare_members(&mut r, &at, members, gi);
        }

        r.check(
            "forms",
            &file,
            s.forms.iter().map(|f| lc(&f.name)).collect::<Vec<_>>(),
            golden.info.forms.iter().map(|f| lc(f)).collect::<Vec<_>>(),
        );

        let mut ours_impl: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for c in &s.classes {
            if let Some(imp) = &c.implementation {
                ours_impl.insert(
                    lc(&c.name),
                    imp.methods.iter().map(|m| lc(&m.name)).collect(),
                );
            }
        }
        for o in &s.orphan_implementations {
            ours_impl.insert(
                lc(&o.class),
                o.implementation
                    .methods
                    .iter()
                    .map(|m| lc(&m.name))
                    .collect(),
            );
        }
        let g_impl: BTreeMap<String, Vec<String>> = golden
            .info
            .implementations
            .iter()
            .map(|i| (lc(&i.name), i.methods.iter().map(|m| lc(m)).collect()))
            .collect();
        r.check("implementations", &file, ours_impl, g_impl);
    }

    println!(
        "compared {} files, skipped {skipped} with statements abaplint cannot parse",
        files.len() - skipped
    );
    let mut failed = Vec::new();
    for (category, (compared, equal, mismatches)) in &r.categories {
        let percent = 100.0 * *equal as f64 / (*compared).max(1) as f64;
        println!("{category}: {equal}/{compared} ({percent:.2}%)");
        for mm in mismatches.iter().take(4) {
            println!("    {mm}");
        }
        if percent < MIN_PERCENT {
            failed.push(format!("{category} {percent:.2}%"));
        }
    }
    assert!(failed.is_empty(), "below baseline: {failed:?}");
}
