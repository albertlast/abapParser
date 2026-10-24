//! Objects loaded through a source provider: include expansion, symbols
//! across the parts of an object, type resolution across objects and the
//! call graph.

use std::cell::Cell;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

use abap_model as m;
use abap_source::{Badi, ClassPart, ObjectKind, SourceError, SourceProvider, SourceRef};
use abap_syntax::{File, StatementKind as K};
use serde::Deserialize;

use crate::calls::{
    method_defined, type_info, CallExtractor, SymbolsResolver, TypeResolver, ValueType,
};
use crate::ddic;

/// One parsed source.
pub struct Unit {
    pub reference: SourceRef,
    pub origin: String,
    pub file: File,
    pub symbols: m::FileSymbols,
}

/// All sources of one object with their combined symbols.
pub struct Object {
    pub reference: SourceRef,
    pub units: Vec<Arc<Unit>>,
    pub symbols: m::FileSymbols,
    /// INCLUDEs no provider had
    pub missing_includes: Vec<SourceRef>,
    /// enhancement implementations whose code was put into the sources, with
    /// where the code came from
    pub enhancements: Vec<(String, String)>,
}

/// Which objects calls are followed into, and how many are loaded. Roots
/// are always loaded.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Follow {
    /// name patterns of the objects to load, like `Z*`, or `/ABC/*` for a
    /// custom namespace; `*` for any characters, in any case; all objects
    /// when empty
    pub include: Vec<String>,
    /// name patterns of objects not to load, also when `include` matches
    pub exclude: Vec<String>,
    /// at most this many objects are loaded or looked for, the roots
    /// included
    pub limit: Option<usize>,
}

impl Follow {
    /// Whether an object of this name is loaded.
    pub fn allows(&self, name: &str) -> bool {
        (self.include.is_empty() || self.include.iter().any(|p| matches(p, name)))
            && !self.exclude.iter().any(|p| matches(p, name))
    }
}

/// Whether `name` matches `pattern`, `*` standing for any characters; case
/// does not matter.
fn matches(pattern: &str, name: &str) -> bool {
    let pattern = pattern.trim().to_ascii_uppercase();
    let name = name.to_ascii_uppercase();
    let parts: Vec<&str> = pattern.split('*').collect();
    if parts.len() == 1 {
        return pattern == name;
    }
    let (first, last) = (parts[0], parts[parts.len() - 1]);
    if !name.starts_with(first) || name.len() < first.len() + last.len() {
        return false;
    }
    let mut rest = &name[first.len()..];
    for part in &parts[1..parts.len() - 1] {
        match rest.find(part) {
            Some(i) => rest = &rest[i + part.len()..],
            None => return false,
        }
    }
    rest.ends_with(last)
}

/// How far loading is: objects loaded or looked for so far, and the last.
#[derive(Debug, Clone)]
pub struct Progress {
    pub loaded: usize,
    /// like `CLAS:ZCL_X`
    pub object: String,
}

/// Called after each object loaded, from the loading threads.
pub type ProgressFn = Box<dyn Fn(&Progress) + Send + Sync>;

pub struct Workspace {
    provider: Box<dyn SourceProvider>,
    follow: Follow,
    progress: Option<ProgressFn>,
    cancel: Option<Arc<AtomicBool>>,
    /// objects always loaded: those an analysis starts from
    roots: Mutex<HashSet<SourceRef>>,
    /// objects not loaded, and why: not followed, limit, cancelled
    skipped: Mutex<BTreeMap<SourceRef, m::Unresolved>>,
    /// objects loaded or looked for
    loads: AtomicUsize,
    /// sources whose loading failed
    failed: Mutex<HashSet<SourceRef>>,
    units: Mutex<HashMap<SourceRef, Slot<Unit>>>,
    objects: Mutex<HashMap<SourceRef, Slot<Object>>>,
    /// calls of each object, worked out once
    calls: Mutex<HashMap<SourceRef, Slot<Vec<m::Call>>>>,
    implementations: Mutex<HashMap<String, Vec<String>>>,
    subclasses: Mutex<HashMap<String, Vec<String>>>,
    structures: Mutex<HashMap<String, Option<Arc<ddic::Structure>>>>,
    data_elements: Mutex<HashMap<String, Option<String>>>,
    errors: Mutex<Vec<m::LoadError>>,
}

/// Requests to an SAP system mostly wait on the network: this many loads
/// run at once.
const PARALLEL_LOADS: usize = 8;

/// Loads once: threads asking for the same source wait for one load.
type Slot<T> = Arc<OnceLock<Option<Arc<T>>>>;

fn slot<T>(cache: &Mutex<HashMap<SourceRef, Slot<T>>>, r: &SourceRef) -> Slot<T> {
    cache.lock().unwrap().entry(r.clone()).or_default().clone()
}

thread_local! {
    /// Set in the threads of [`parallel_map`]: work started there runs in
    /// that thread, so no more than [`PARALLEL_LOADS`] requests are open.
    static IN_PARALLEL: Cell<bool> = const { Cell::new(false) };
}

/// `f` of each item in up to [`PARALLEL_LOADS`] threads, results in order.
fn parallel_map<T: Sync, R: Send>(items: &[T], f: impl Fn(&T) -> R + Sync) -> Vec<R> {
    if items.len() < 2 || IN_PARALLEL.with(Cell::get) {
        return items.iter().map(f).collect();
    }
    let next = AtomicUsize::new(0);
    let results: Mutex<Vec<Option<R>>> = Mutex::new(items.iter().map(|_| None).collect());
    std::thread::scope(|scope| {
        for _ in 0..PARALLEL_LOADS.min(items.len()) {
            scope.spawn(|| {
                IN_PARALLEL.with(|p| p.set(true));
                loop {
                    let i = next.fetch_add(1, Ordering::Relaxed);
                    let Some(item) = items.get(i) else {
                        break;
                    };
                    let r = f(item);
                    results.lock().unwrap()[i] = Some(r);
                }
            });
        }
    });
    results
        .into_inner()
        .unwrap()
        .into_iter()
        .map(|r| r.expect("every item is done"))
        .collect()
}

/// Classes and interfaces the code of an object names, upper case: after
/// `REF TO`, `INTERFACES`, `INHERITING FROM`, `NEW`, `CAST`, and before `=>`;
/// without its own classes.
fn named_types(object: &Object) -> Vec<String> {
    use abap_syntax::TokenKind as T;
    let own = |n: &str| {
        object
            .symbols
            .classes
            .iter()
            .any(|c| c.name.eq_ignore_ascii_case(n))
            || object
                .symbols
                .interfaces
                .iter()
                .any(|i| i.name.eq_ignore_ascii_case(n))
    };
    let mut out = Vec::new();
    for u in &object.units {
        let t: Vec<&abap_syntax::Token> = u
            .file
            .tokens()
            .iter()
            .filter(|t| t.kind != T::Comment)
            .collect();
        for i in 0..t.len() {
            let after = |k: usize, words: &[&str]| {
                k >= words.len()
                    && words
                        .iter()
                        .enumerate()
                        .all(|(n, w)| t[k - words.len() + n].is(w))
            };
            let named = (after(i, &["REF", "TO"])
                || after(i, &["INTERFACES"])
                || after(i, &["INHERITING", "FROM"])
                || after(i, &["NEW"])
                || after(i, &["CAST"]))
                || t.get(i + 1).is_some_and(|n| n.kind == T::StaticArrow);
            let x = t[i];
            if named && x.kind == T::Identifier && x.text != "#" && !own(&x.text) {
                out.push(x.text.to_ascii_uppercase());
            }
        }
    }
    out
}

/// The program name SAP gives a source: the main program `SAPL<group>` of a
/// function group, the class pool and its includes for a class.
fn program_name(r: &SourceRef) -> String {
    match (r.kind, r.part) {
        (ObjectKind::Fugr, _) => main_program_of_group(&r.name),
        (ObjectKind::Clas, None) => abap_source::class_pool(&r.name),
        (ObjectKind::Clas, Some(part)) => {
            let pool = abap_source::class_pool(&r.name);
            let suffix = match part {
                ClassPart::Definitions => "CCDEF",
                ClassPart::Implementations => "CCIMP",
                ClassPart::Macros => "CCMAC",
                ClassPart::TestClasses => "CCAU",
            };
            format!("{}{suffix}", &pool[..pool.len() - 2])
        }
        _ => r.name.clone(),
    }
}

/// `SAPL<group>`, after the namespace of the group: `/ABC/SAPLGRP`.
pub(crate) fn main_program_of_group(group: &str) -> String {
    match group.strip_prefix('/').and_then(|g| g.split_once('/')) {
        Some((ns, name)) => format!("/{ns}/SAPL{name}"),
        None => format!("SAPL{group}"),
    }
}

/// Classes a BAdI runs: those of its active implementations, else its
/// fallback class.
pub(crate) fn badi_classes(b: &Badi) -> Vec<String> {
    if b.implementations.is_empty() {
        b.fallback.iter().cloned().collect()
    } else {
        b.implementations.clone()
    }
}

/// Implementations of an interface named by a provider are checked against
/// their sources up to this many.
const MAX_CHECKED_IMPLEMENTATIONS: usize = 50;

/// Includes may include each other in cycles; expansion stops here.
const MAX_INCLUDES: usize = 500;

/// The implicit superclass of all classes, `TYPE REF TO object`; it has no
/// source, and an SAP system answers a request for it with an error.
const ROOT_CLASS: &str = "OBJECT";

/// Built-in and generic types, never looked up in the dictionary.
fn is_builtin_type(name: &str) -> bool {
    const BUILTIN: &[&str] = &[
        "ANY",
        "B",
        "C",
        "CLIKE",
        "CSEQUENCE",
        "D",
        "DATA",
        "DECFLOAT",
        "DECFLOAT16",
        "DECFLOAT34",
        "F",
        "I",
        "INT8",
        "N",
        "NUMERIC",
        "OBJECT",
        "P",
        "S",
        "SIMPLE",
        "STRING",
        "T",
        "UTCLONG",
        "X",
        "XSEQUENCE",
        "XSTRING",
        "ABAP_BOOL",
        "TABLE",
        "INDEX",
        "STANDARD",
    ];
    BUILTIN.contains(&name) || name.contains("=>") || name.contains('~')
}

fn is_root_class(r: &SourceRef) -> bool {
    r.kind == ObjectKind::Clas && r.name.eq_ignore_ascii_case(ROOT_CLASS)
}

impl Workspace {
    pub fn new(provider: Box<dyn SourceProvider>) -> Self {
        Workspace {
            provider,
            follow: Follow::default(),
            progress: None,
            cancel: None,
            roots: Mutex::new(HashSet::new()),
            skipped: Mutex::new(BTreeMap::new()),
            loads: AtomicUsize::new(0),
            failed: Mutex::new(HashSet::new()),
            units: Mutex::new(HashMap::new()),
            objects: Mutex::new(HashMap::new()),
            calls: Mutex::new(HashMap::new()),
            implementations: Mutex::new(HashMap::new()),
            subclasses: Mutex::new(HashMap::new()),
            structures: Mutex::new(HashMap::new()),
            data_elements: Mutex::new(HashMap::new()),
            errors: Mutex::new(Vec::new()),
        }
    }

    /// Loads only the objects `follow` allows, the roots always.
    pub fn with_follow(mut self, follow: Follow) -> Self {
        self.follow = follow;
        self
    }

    /// Calls `progress` after each object loaded or looked for.
    pub fn with_progress(mut self, progress: ProgressFn) -> Self {
        self.progress = Some(progress);
        self
    }

    /// Loads nothing more once `cancel` is set; analyses then finish with
    /// what they have.
    pub fn with_cancel(mut self, cancel: Arc<AtomicBool>) -> Self {
        self.cancel = Some(cancel);
        self
    }

    /// Objects an analysis starts from: loaded whatever `follow` says.
    pub fn add_roots(&self, roots: &[SourceRef]) {
        let mut set = self.roots.lock().unwrap();
        set.extend(roots.iter().map(SourceRef::object));
    }

    /// An object asked for by name, loaded whatever `follow` says.
    pub fn root(&self, r: &SourceRef) -> Option<Arc<Object>> {
        self.add_roots(std::slice::from_ref(r));
        self.object(r)
    }

    fn cancelled(&self) -> bool {
        self.cancel
            .as_ref()
            .is_some_and(|c| c.load(Ordering::Relaxed))
    }

    fn skip(&self, r: &SourceRef, why: m::Unresolved) {
        self.skipped.lock().unwrap().entry(r.clone()).or_insert(why);
    }

    /// Why an object was not loaded, when `follow`, the limit or
    /// cancelling kept it out.
    pub fn skip_reason(&self, r: &SourceRef) -> Option<m::Unresolved> {
        self.skipped.lock().unwrap().get(&r.object()).copied()
    }

    /// Objects not loaded because of `follow`, the limit or cancelling.
    pub fn skipped(&self) -> Vec<String> {
        self.skipped
            .lock()
            .unwrap()
            .keys()
            .map(|r| r.to_string())
            .collect()
    }

    /// Whether loading this source failed with an error.
    pub(crate) fn failed(&self, r: &SourceRef) -> bool {
        self.failed.lock().unwrap().contains(r)
    }

    /// Provider errors so far, e.g. failed authorization.
    pub fn errors(&self) -> Vec<m::LoadError> {
        self.errors.lock().unwrap().clone()
    }

    fn record(&self, object: String, e: &SourceError) {
        let (code, status) = match e {
            SourceError::Io(_) => (m::ErrorCode::Io, None),
            SourceError::Http { status, .. } => (m::ErrorCode::Http, Some(*status)),
            SourceError::Auth { .. } => (m::ErrorCode::Auth, None),
            SourceError::Network(_) => (m::ErrorCode::Network, None),
            SourceError::Other(_) => (m::ErrorCode::Other, None),
        };
        self.errors.lock().unwrap().push(m::LoadError {
            code,
            object,
            cause: e.to_string(),
            retryable: e.retryable(),
            status,
        });
    }

    /// Lookups the system could not do, once each, like
    /// `enhancements: not available (...)`, and whether the limit or
    /// cancelling stopped loading. Not errors.
    pub fn notes(&self) -> Vec<String> {
        let mut notes = self.provider.notes();
        let skipped = self.skipped.lock().unwrap();
        if let (Some(limit), true) = (
            self.follow.limit,
            skipped.values().any(|w| *w == m::Unresolved::Limit),
        ) {
            notes.push(format!(
                "limit: {limit} objects loaded, more were not loaded"
            ));
        }
        if skipped.values().any(|w| *w == m::Unresolved::Cancelled) {
            notes.push("cancelled: objects after it were not loaded".to_string());
        }
        notes.sort();
        notes.dedup();
        notes
    }

    /// One source, parsed; `None` if no provider has it or fetching failed.
    pub fn unit(&self, r: &SourceRef) -> Option<Arc<Unit>> {
        let slot = slot(&self.units, r);
        slot.get_or_init(|| self.load_unit(r)).clone()
    }

    fn load_unit(&self, r: &SourceRef) -> Option<Arc<Unit>> {
        match self.provider.fetch(r) {
            Ok(Some(source)) => {
                let file = abap_syntax::parse(&source.text);
                let symbols = crate::symbols_of(&file);
                Some(Arc::new(Unit {
                    reference: r.clone(),
                    origin: source.origin,
                    file,
                    symbols,
                }))
            }
            Ok(None) => None,
            Err(e) => {
                self.record(r.to_string(), &e);
                self.failed.lock().unwrap().insert(r.clone());
                None
            }
        }
    }

    /// The text of a dictionary type, `None` if no provider has it.
    fn fetch_text(&self, r: &SourceRef) -> Option<String> {
        match self.provider.fetch(r) {
            Ok(s) => s.map(|s| s.text),
            Err(e) => {
                self.record(r.to_string(), &e);
                None
            }
        }
    }

    /// A dictionary structure or table.
    pub fn structure(&self, name: &str) -> Option<Arc<ddic::Structure>> {
        let key = name.to_ascii_uppercase();
        if let Some(s) = self.structures.lock().unwrap().get(&key) {
            return s.clone();
        }
        let s = if is_builtin_type(&key) {
            None
        } else {
            self.fetch_text(&SourceRef::table(&key))
                .and_then(|text| ddic::structure(&text))
                .map(Arc::new)
        };
        self.structures.lock().unwrap().insert(key, s.clone());
        s
    }

    /// The class or interface a dictionary data element refers to, when it
    /// is typed as a reference.
    pub fn data_element_reference(&self, name: &str) -> Option<String> {
        let key = name.to_ascii_uppercase();
        if let Some(r) = self.data_elements.lock().unwrap().get(&key) {
            return r.clone();
        }
        let r = if is_builtin_type(&key) {
            None
        } else {
            self.fetch_text(&SourceRef::data_element(&key))
                .and_then(|text| ddic::data_element_reference(&text))
        };
        self.data_elements.lock().unwrap().insert(key, r.clone());
        r
    }

    /// All sources of an object: class parts, includes of programs, function
    /// modules of function groups. `None` if the object itself is missing.
    pub fn object(&self, r: &SourceRef) -> Option<Arc<Object>> {
        let r = r.object();
        if is_root_class(&r) {
            return None;
        }
        let root = self.roots.lock().unwrap().contains(&r);
        if !root {
            if self.cancelled() {
                self.skip(&r, m::Unresolved::Cancelled);
                return None;
            }
            if !self.follow.allows(&r.name) {
                self.skip(&r, m::Unresolved::NotFollowed);
                return None;
            }
        }
        // another thread loading it is waited for
        let slot = slot(&self.objects, &r);
        slot.get_or_init(|| {
            let loaded = self.loads.fetch_add(1, Ordering::SeqCst) + 1;
            if !root && self.follow.limit.is_some_and(|l| loaded > l) {
                self.skip(&r, m::Unresolved::Limit);
                return None;
            }
            let object = self.load_object(&r).map(Arc::new);
            if let Some(progress) = &self.progress {
                progress(&Progress {
                    loaded,
                    object: r.to_string(),
                });
            }
            object
        })
        .clone()
    }

    /// Loads the objects calls lead to, up to `depth` hops from the roots,
    /// several at once: an SAP system answers each request on its own.
    /// Analyses after it find them loaded.
    pub fn preload(&self, roots: &[SourceRef], depth: u32) {
        self.add_roots(roots);
        let mut level: Vec<SourceRef> = roots.iter().map(SourceRef::object).collect();
        let mut seen: BTreeSet<SourceRef> = level.iter().cloned().collect();
        for d in 0..=depth {
            let objects: Vec<Arc<Object>> = parallel_map(&level, |r| self.object(r))
                .into_iter()
                .flatten()
                .collect();
            self.load_named_types(&objects);
            let calls: Vec<m::Call> = parallel_map(&objects, |o| self.object_calls(o))
                .into_iter()
                .flatten()
                .collect();
            // the calls of the last level are not followed
            if d == depth {
                break;
            }
            level = parallel_map(&calls, |c| self.logic_targets(c))
                .into_iter()
                .flatten()
                .filter(|t| seen.insert(t.clone()))
                .collect();
        }
    }

    /// Working out the calls of an object loads the classes they are of,
    /// one by one; with fewer objects than threads, the classes their code
    /// names are loaded before, several at once.
    fn load_named_types(&self, objects: &[Arc<Object>]) {
        if objects.len() >= PARALLEL_LOADS {
            return;
        }
        let mut names: Vec<String> = objects.iter().flat_map(|o| named_types(o)).collect();
        names.sort();
        names.dedup();
        parallel_map(&names, |n| self.type_object(n));
    }

    fn load_object(&self, r: &SourceRef) -> Option<Object> {
        let main = self.unit(r)?;
        let mut units = vec![main];
        let mut missing_includes = Vec::new();
        match r.kind {
            ObjectKind::Clas => {
                let parts: Vec<SourceRef> = ClassPart::ALL
                    .iter()
                    .map(|p| r.clone().with_part(*p))
                    .collect();
                units.extend(parallel_map(&parts, |p| self.unit(p)).into_iter().flatten());
            }
            ObjectKind::Fugr => {
                if let Ok(fms) = self.provider.function_modules(&r.name) {
                    let fms: Vec<SourceRef> =
                        fms.iter().map(|f| SourceRef::function_module(f)).collect();
                    units.extend(parallel_map(&fms, |f| self.unit(f)).into_iter().flatten());
                }
            }
            _ => {}
        }
        // INCLUDE statements, recursively
        let mut seen: BTreeSet<SourceRef> = units.iter().map(|u| u.reference.clone()).collect();
        let all = units.clone();
        self.add_includes(all, &mut units, &mut seen, &mut missing_includes);
        let (enhancements, enhanced) = self.enhance(r, &mut units);
        // enhancement code may include more
        let enhanced: Vec<Arc<Unit>> = enhanced.iter().map(|i| units[*i].clone()).collect();
        self.add_includes(enhanced, &mut units, &mut seen, &mut missing_includes);
        // macros of one part (class macros include, TOP include) are usable in
        // the others: parse again where a call of such a macro was not expanded
        let macros: Vec<abap_syntax::MacroDef> =
            units.iter().flat_map(|u| u.file.macros()).collect();
        if !macros.is_empty() {
            for u in units.iter_mut() {
                let calls_other = u.file.statements.iter().any(|s| {
                    s.kind == K::Unknown
                        && u.file
                            .tokens()
                            .get(s.tokens[0])
                            .is_some_and(|t| macros.iter().any(|m| m.name == t.upper()))
                });
                if calls_other {
                    let source = &u.file.lexed.source;
                    let file = abap_syntax::parse_with_macros(source, &macros);
                    let symbols = crate::symbols_of(&file);
                    *u = Arc::new(Unit {
                        reference: u.reference.clone(),
                        origin: u.origin.clone(),
                        file,
                        symbols,
                    });
                }
            }
        }
        let symbols = merge_symbols(units.iter().map(|u| &u.symbols));
        Some(Object {
            reference: r.clone(),
            units,
            symbols,
            missing_includes,
            enhancements,
        })
    }

    /// Adds the units the INCLUDE statements of `level` name, recursively;
    /// the includes of one level are loaded at once.
    fn add_includes(
        &self,
        mut level: Vec<Arc<Unit>>,
        units: &mut Vec<Arc<Unit>>,
        seen: &mut BTreeSet<SourceRef>,
        missing: &mut Vec<SourceRef>,
    ) {
        while !level.is_empty() {
            let mut names = Vec::new();
            for u in &level {
                for name in includes(&u.file) {
                    let inc = SourceRef::program(&name);
                    if seen.len() < MAX_INCLUDES && seen.insert(inc.clone()) {
                        names.push(inc);
                    }
                }
            }
            let loaded = parallel_map(&names, |inc| self.unit(inc));
            level = Vec::new();
            for (inc, u) in names.into_iter().zip(loaded) {
                match u {
                    Some(u) => {
                        units.push(u.clone());
                        level.push(u);
                    }
                    None => missing.push(inc),
                }
            }
        }
    }

    /// Puts the code of active enhancement implementations into the units
    /// where it runs; returns the implementations with their origin, and
    /// the units changed.
    fn enhance(
        &self,
        r: &SourceRef,
        units: &mut [Arc<Unit>],
    ) -> (Vec<(String, String)>, Vec<usize>) {
        if !matches!(
            r.kind,
            ObjectKind::Prog | ObjectKind::Fugr | ObjectKind::Func | ObjectKind::Clas
        ) {
            return (Vec::new(), Vec::new());
        }
        let names: Vec<String> = units.iter().map(|u| program_name(&u.reference)).collect();
        let mut programs = Vec::new();
        let mut functions = Vec::new();
        for (u, name) in units.iter().zip(&names) {
            if u.reference.kind == ObjectKind::Func {
                functions.push(name.clone());
            } else {
                programs.push(name.clone());
            }
        }
        let hooks = match self.provider.enhancements(&programs, &functions) {
            Ok(h) => h,
            Err(e) => {
                self.record(format!("enhancements:{r}"), &e);
                return (Vec::new(), Vec::new());
            }
        };
        if hooks.is_empty() {
            return (Vec::new(), Vec::new());
        }
        let files: Vec<(&str, &File)> = names
            .iter()
            .zip(units.iter())
            .map(|(n, u)| (n.as_str(), &u.file))
            .collect();
        let (texts, applied) = crate::enhance::apply(&files, &hooks);
        let mut out: Vec<(String, String)> = Vec::new();
        for h in applied {
            if !out.iter().any(|(n, _)| *n == h.enhancement) {
                out.push((h.enhancement.clone(), h.origin.clone()));
            }
        }
        let changed: Vec<usize> = texts.iter().map(|(i, _)| *i).collect();
        for (i, text) in texts {
            let file = abap_syntax::parse(&text);
            let symbols = crate::symbols_of(&file);
            units[i] = Arc::new(Unit {
                reference: units[i].reference.clone(),
                origin: units[i].origin.clone(),
                file,
                symbols,
            });
        }
        (out, changed)
    }

    /// A BAdI of an enhancement spot by its name, as far as the provider
    /// knows it.
    pub fn badi(&self, name: &str) -> Option<Badi> {
        let key = name.to_ascii_uppercase();
        self.provider.badi(&key).unwrap_or_else(|e| {
            self.record(format!("badi:{key}"), &e);
            None
        })
    }

    /// A classic BAdI by its interface, as far as the provider knows it.
    pub fn classic_badi(&self, interface: &str) -> Option<Badi> {
        let key = interface.to_ascii_uppercase();
        self.provider.classic_badi(&key).unwrap_or_else(|e| {
            self.record(format!("classic_badi:{key}"), &e);
            None
        })
    }

    /// Global classes implementing an interface, as far as the provider can
    /// list them, upper case. For the interface of a classic BAdI the
    /// classes of its active implementations, else its default class.
    pub fn implementations(&self, interface: &str) -> Vec<String> {
        let key = interface.to_ascii_uppercase();
        if let Some(c) = self.implementations.lock().unwrap().get(&key) {
            return c.clone();
        }
        if let Some(b) = self.classic_badi(&key) {
            let classes = badi_classes(&b);
            self.implementations
                .lock()
                .unwrap()
                .insert(key, classes.clone());
            return classes;
        }
        let candidates = match self.provider.implementations(&key) {
            Ok(c) => c,
            Err(e) => {
                self.record(format!("implementations:{key}"), &e);
                Vec::new()
            }
        };
        // a where-used list also names classes only using the interface:
        // keep those whose definition implements it; long lists are kept
        // unchecked, they are too many to follow anyway
        let classes = if candidates.len() > MAX_CHECKED_IMPLEMENTATIONS {
            candidates
        } else {
            let implements = parallel_map(&candidates, |c| {
                self.object(&SourceRef::class(c)).is_some_and(|o| {
                    type_info(&o.symbols, c)
                        .is_some_and(|i| i.interfaces.iter().any(|x| x.eq_ignore_ascii_case(&key)))
                })
            });
            candidates
                .into_iter()
                .zip(implements)
                .filter_map(|(c, yes)| yes.then_some(c))
                .collect()
        };
        self.implementations
            .lock()
            .unwrap()
            .insert(key, classes.clone());
        classes
    }

    /// Global classes inheriting directly from a class, as far as the
    /// provider can list them, upper case.
    pub fn subclasses(&self, class: &str) -> Vec<String> {
        let key = class.to_ascii_uppercase();
        if let Some(c) = self.subclasses.lock().unwrap().get(&key) {
            return c.clone();
        }
        let classes = match self.provider.subclasses(&key) {
            Ok(c) => c,
            Err(e) => {
                self.record(format!("subclasses:{key}"), &e);
                Vec::new()
            }
        };
        self.subclasses.lock().unwrap().insert(key, classes.clone());
        classes
    }

    /// The object of a class or interface name, whichever exists.
    pub fn type_object(&self, name: &str) -> Option<Arc<Object>> {
        let class = SourceRef::class(name);
        self.object(&class).or_else(|| {
            // a name not followed is skipped once, as a class
            (self.skip_reason(&class).is_none())
                .then(|| self.object(&SourceRef::interface(name)))
                .flatten()
        })
    }

    /// Calls of an object, types resolved through the workspace.
    pub fn object_calls(&self, object: &Object) -> Vec<m::Call> {
        let slot = slot(&self.calls, &object.reference);
        let calls = slot.get_or_init(|| Some(Arc::new(self.find_calls(object))));
        calls.as_deref().cloned().unwrap_or_default()
    }

    fn find_calls(&self, object: &Object) -> Vec<m::Call> {
        let resolver = WorkspaceResolver { workspace: self };
        let local = SymbolsResolver {
            symbols: &object.symbols,
            fallback: Some(&resolver),
        };
        // a program without REPORT is an include: the object is the program
        let program =
            crate::calls::main_program(&object.symbols).or_else(|| match object.reference.kind {
                ObjectKind::Prog => Some(object.reference.name.clone()),
                ObjectKind::Fugr => Some(main_program_of_group(&object.reference.name)),
                _ => None,
            });
        let mut out = Vec::new();
        for u in &object.units {
            let calls = CallExtractor {
                file: &u.file,
                symbols: &object.symbols,
                resolver: &local,
                program: program.clone(),
            }
            .calls();
            for mut c in calls {
                c.unit = Some(u.reference.to_string());
                self.badi_call(&mut c);
                out.push(c);
            }
        }
        out
    }

    /// `CALL BADI` names the BAdI where a class goes: its interface has the
    /// method.
    fn badi_call(&self, c: &mut m::Call) {
        if c.kind != m::CallKind::Method || !c.options.iter().any(|o| o == "BADI") {
            return;
        }
        let Some(name) = c.class.clone() else {
            return;
        };
        if let Some(interface) = self.badi(&name).and_then(|b| b.interface) {
            c.badi = Some(name.to_ascii_uppercase());
            c.class = Some(interface);
        }
    }

    /// Calls reachable from the roots: roots are analyzed, called objects are
    /// loaded and analyzed up to `depth` hops; objects one hop further are
    /// only loaded to resolve the calls into them.
    pub fn call_graph(&self, roots: &[SourceRef], depth: u32) -> m::CallGraph {
        let mut graph = m::CallGraph {
            schema_version: m::SCHEMA_VERSION,
            roots: roots.iter().map(|r| r.to_string()).collect(),
            units: Vec::new(),
            calls: Vec::new(),
            missing: Vec::new(),
            skipped: Vec::new(),
            errors: Vec::new(),
            notes: Vec::new(),
        };
        self.add_roots(roots);
        let mut level: Vec<SourceRef> = Vec::new();
        let mut seen: BTreeSet<SourceRef> = BTreeSet::new();
        for r in roots {
            if seen.insert(r.object()) {
                level.push(r.object());
            }
        }
        let mut missing: BTreeSet<String> = BTreeSet::new();
        let mut d = 0;
        // one level after the other; within a level, objects, their calls and
        // what the calls need are loaded several at once
        while !level.is_empty() {
            let objects = parallel_map(&level, |r| self.object(r));
            if d < depth {
                let loaded: Vec<Arc<Object>> = objects.iter().flatten().cloned().collect();
                self.load_named_types(&loaded);
            }
            let calls: Vec<Vec<m::Call>> = parallel_map(&objects, |o| match o {
                Some(o) if d < depth => self.object_calls(o),
                _ => Vec::new(),
            });
            let each: Vec<(usize, usize)> = calls
                .iter()
                .enumerate()
                .flat_map(|(i, c)| (0..c.len()).map(move |k| (i, k)))
                .collect();
            let facts = parallel_map(&each, |(i, k)| {
                let (Some(object), c) = (&objects[*i], &calls[*i][*k]) else {
                    return (Vec::new(), false, false, None);
                };
                let resolved = self.resolved(c, object);
                let incomplete = !resolved
                    && matches!(c.kind, m::CallKind::Method | m::CallKind::Constructor)
                    && c.class
                        .as_deref()
                        .is_some_and(|class| !self.hierarchy_known(class, object, 0));
                let why = (!resolved).then(|| self.unresolved(c, incomplete));
                (self.targets(c), resolved, incomplete, why)
            });
            let mut facts = facts.into_iter();
            let mut next = Vec::new();
            for ((r, object), calls) in level.iter().zip(&objects).zip(calls) {
                let Some(object) = object else {
                    // not loaded on purpose, or failed: in `skipped`, `errors`
                    if self.skip_reason(r).is_none() && !self.failed(r) {
                        missing.insert(r.to_string());
                    }
                    continue;
                };
                for u in &object.units {
                    graph.units.push(m::GraphUnit {
                        reference: u.reference.to_string(),
                        origin: u.origin.clone(),
                        depth: d,
                    });
                }
                // code of enhancement implementations in the sources
                for (name, origin) in &object.enhancements {
                    graph.units.push(m::GraphUnit {
                        reference: format!("ENHO:{name}"),
                        origin: origin.clone(),
                        depth: d,
                    });
                }
                missing.extend(object.missing_includes.iter().map(|i| i.to_string()));
                for mut c in calls {
                    let (targets, resolved, incomplete, why) = facts.next().unwrap_or_default();
                    for target in targets {
                        if seen.insert(target.clone()) {
                            next.push(target);
                        }
                    }
                    c.resolved = resolved;
                    c.hierarchy_incomplete = incomplete;
                    c.unresolved = why;
                    graph.calls.push(c);
                }
            }
            level = next;
            d += 1;
        }
        graph.missing = missing.into_iter().collect();
        graph.skipped = self.skipped();
        graph.errors = self.errors();
        graph.notes = self.notes();
        graph
    }

    /// Objects a call leads to in the logic, which follows calls of an
    /// interface method into the implementations too.
    fn logic_targets(&self, c: &m::Call) -> Vec<SourceRef> {
        let mut out = self.targets(c);
        let dispatched = c.kind == m::CallKind::Method && !c.is_static && c.badi.is_none();
        let interface = out.first().is_some_and(|r| r.kind == ObjectKind::Intf);
        if dispatched && interface {
            if let Some(class) = &c.class {
                let classes = self.implementations(class);
                if classes.len() <= crate::logic::MAX_IMPLEMENTATIONS {
                    out.extend(classes.iter().map(|c| SourceRef::class(c)));
                }
            }
        }
        out
    }

    /// Objects a call leads to.
    fn targets(&self, c: &m::Call) -> Vec<SourceRef> {
        if c.dynamic {
            return Vec::new();
        }
        match c.kind {
            m::CallKind::Function => vec![SourceRef::function_module(&c.name)],
            m::CallKind::Submit => vec![SourceRef::program(&c.name)],
            m::CallKind::Form => c.program.iter().map(|p| SourceRef::program(p)).collect(),
            m::CallKind::Method | m::CallKind::Constructor => {
                let mut out: Vec<SourceRef> = c
                    .class
                    .as_deref()
                    .and_then(|cls| self.type_object(cls))
                    .map(|o| vec![o.reference.clone()])
                    .unwrap_or_default();
                // a BAdI runs the classes of its active implementations
                let badi = match (&c.badi, c.class.as_deref()) {
                    (Some(name), _) => self.badi(name),
                    (None, Some(cls))
                        if out.first().is_some_and(|r| r.kind == ObjectKind::Intf) =>
                    {
                        self.classic_badi(cls)
                    }
                    _ => None,
                };
                if let Some(b) = badi {
                    out.extend(badi_classes(&b).iter().map(|cls| SourceRef::class(cls)));
                }
                out
            }
            _ => Vec::new(),
        }
    }

    /// Why the target of an unresolved call was not found.
    fn unresolved(&self, c: &m::Call, incomplete: bool) -> m::Unresolved {
        use m::Unresolved as U;
        if c.dynamic {
            return U::Dynamic;
        }
        let targets: Vec<SourceRef> = match c.kind {
            m::CallKind::Function => vec![SourceRef::function_module(&c.name)],
            m::CallKind::Submit | m::CallKind::Include => vec![SourceRef::program(&c.name)],
            m::CallKind::Form => c.program.iter().map(|p| SourceRef::program(p)).collect(),
            m::CallKind::Method | m::CallKind::Constructor => match &c.class {
                Some(class) => vec![SourceRef::class(class), SourceRef::interface(class)],
                None => return U::UnknownType,
            },
            m::CallKind::Transaction | m::CallKind::Transformation => return U::NotLoaded,
        };
        if let Some(why) = targets.iter().find_map(|t| self.skip_reason(t)) {
            return why;
        }
        if targets.iter().any(|t| self.failed(t)) {
            return U::LookupFailed;
        }
        if incomplete {
            return U::HierarchyIncomplete;
        }
        U::Missing
    }

    /// Whether the target of a call exists in the loaded sources.
    fn resolved(&self, c: &m::Call, from: &Object) -> bool {
        if c.dynamic {
            return false;
        }
        match c.kind {
            m::CallKind::Function => self.object(&SourceRef::function_module(&c.name)).is_some(),
            m::CallKind::Submit => self.object(&SourceRef::program(&c.name)).is_some(),
            m::CallKind::Form => {
                let symbols = match &c.program {
                    Some(p) => match self.object(&SourceRef::program(p)) {
                        Some(o) => o,
                        None => return false,
                    },
                    None => {
                        return from
                            .symbols
                            .forms
                            .iter()
                            .any(|f| f.name.eq_ignore_ascii_case(&c.name))
                    }
                };
                symbols
                    .symbols
                    .forms
                    .iter()
                    .any(|f| f.name.eq_ignore_ascii_case(&c.name))
            }
            m::CallKind::Include => self.object(&SourceRef::program(&c.name)).is_some(),
            m::CallKind::Method | m::CallKind::Constructor => {
                let Some(class) = c.class.as_deref() else {
                    return false;
                };
                // super->constructor( ) also exists when only implicit
                let constructor = c.kind == m::CallKind::Constructor
                    || c.name.eq_ignore_ascii_case("constructor")
                    || c.name.eq_ignore_ascii_case("class_constructor");
                // local classes are in the calling object itself
                method_defined(&from.symbols, class, &c.name, constructor)
                    || self.method_exists(class, &c.name, constructor, 0)
            }
            _ => false,
        }
    }

    /// Whether a class or interface and all its superclasses and interfaces
    /// are in the sources; local classes are looked up in `from`.
    fn hierarchy_known(&self, class: &str, from: &Object, level: u32) -> bool {
        if level > 10 || class.eq_ignore_ascii_case(ROOT_CLASS) {
            return true;
        }
        let object;
        let symbols = if type_info(&from.symbols, class).is_some() {
            &from.symbols
        } else {
            match self.type_object(class) {
                Some(o) => {
                    object = o;
                    &object.symbols
                }
                None => return false,
            }
        };
        let Some(info) = type_info(symbols, class) else {
            return false;
        };
        let parents: Vec<String> = info
            .interfaces
            .iter()
            .map(|i| i.to_string())
            .chain(info.superclass.map(str::to_string))
            .collect();
        parents
            .iter()
            .all(|p| self.hierarchy_known(p, from, level + 1))
    }

    fn method_exists(&self, class: &str, method: &str, constructor: bool, level: u32) -> bool {
        let Some(object) = self.type_object(class) else {
            return false;
        };
        let s = &object.symbols;
        if method_defined(s, class, method, constructor) {
            return true;
        }
        if level > 10 {
            return false;
        }
        let Some(info) = type_info(s, class) else {
            return false;
        };
        // interfaces of other objects, then the superclass
        let interfaces: Vec<String> = info.interfaces.iter().map(|i| i.to_string()).collect();
        let superclass = info.superclass.map(str::to_string);
        let (intf, meth) = match method.split_once('~') {
            Some((i, mth)) => (Some(i.to_string()), mth.to_string()),
            None => (None, method.to_string()),
        };
        interfaces
            .iter()
            .filter(|i| intf.as_deref().is_none_or(|x| x.eq_ignore_ascii_case(i)))
            .any(|i| self.method_exists(i, &meth, false, level + 1))
            || superclass.is_some_and(|sc| self.method_exists(&sc, method, constructor, level + 1))
    }
}

/// Types of classes in other objects, loaded on demand.
struct WorkspaceResolver<'a> {
    workspace: &'a Workspace,
}

impl WorkspaceResolver<'_> {
    fn with<T>(&self, class: &str, f: impl FnOnce(&SymbolsResolver) -> Option<T>) -> Option<T> {
        let object = self.workspace.type_object(class)?;
        let resolver = SymbolsResolver {
            symbols: &object.symbols,
            fallback: Some(self),
        };
        f(&resolver)
    }
}

impl WorkspaceResolver<'_> {
    /// A component of a dictionary structure, also in its includes.
    fn dictionary_component(
        &self,
        structure: &str,
        component: &str,
        level: u32,
    ) -> Option<ValueType> {
        if level > 10 {
            return None;
        }
        let s = self.workspace.structure(structure)?;
        if let Some(ty) = s.component(component) {
            return ty.clone();
        }
        s.includes
            .iter()
            .find_map(|i| self.dictionary_component(i, component, level + 1))
    }
}

impl TypeResolver for WorkspaceResolver<'_> {
    fn returning_type(&self, class: &str, method: &str) -> Option<ValueType> {
        self.with(class, |r| {
            type_info(r.symbols, class)?;
            r.returning_type(class, method)
        })
    }

    fn attribute_type(&self, class: &str, attribute: &str) -> Option<ValueType> {
        if self.workspace.type_object(class).is_none() {
            // a reference to a structure: lr->component
            return self.component_type(class, attribute);
        }
        self.with(class, |r| {
            type_info(r.symbols, class)?;
            r.attribute_type(class, attribute)
        })
    }

    fn superclass(&self, class: &str) -> Option<String> {
        self.with(class, |r| {
            type_info(r.symbols, class)?.superclass.map(str::to_string)
        })
    }

    fn method_parameters(&self, class: &str, method: &str) -> Option<Vec<(String, ValueType)>> {
        self.with(class, |r| {
            type_info(r.symbols, class)?;
            r.method_parameters(class, method)
        })
    }

    fn component_type(&self, structure: &str, component: &str) -> Option<ValueType> {
        // a type of a class or interface: zcl_x=>ty_s
        if let Some((class, ty)) = structure.split_once("=>") {
            return self.with(class, |r| {
                type_info(r.symbols, class)?;
                r.component_type(ty, component)
            });
        }
        self.dictionary_component(structure, component, 0)
    }

    fn data_element_reference(&self, name: &str) -> Option<String> {
        if let Some((class, ty)) = name.split_once("=>") {
            return self.with(class, |r| {
                type_info(r.symbols, class)?;
                r.data_element_reference(ty)
            });
        }
        self.workspace.data_element_reference(name)
    }
}

/// Names of the INCLUDE statements of a file.
fn includes(file: &File) -> Vec<String> {
    file.statements
        .iter()
        .filter(|s| s.kind == K::Include)
        .filter_map(|s| abap_syntax::decl::first_name(file, s, 1).map(|n| n.0))
        .collect()
}

/// Symbols of several sources of one object; class implementations found
/// in another part are attached to their definition.
pub fn merge_symbols<'a>(parts: impl Iterator<Item = &'a m::FileSymbols>) -> m::FileSymbols {
    let mut out = m::FileSymbols {
        schema_version: m::SCHEMA_VERSION,
        program: None,
        classes: Vec::new(),
        interfaces: Vec::new(),
        forms: Vec::new(),
        function_modules: Vec::new(),
        modules: Vec::new(),
        event_blocks: Vec::new(),
        macros: Vec::new(),
        includes: Vec::new(),
        declarations: Vec::new(),
        orphan_implementations: Vec::new(),
    };
    for p in parts {
        out.program = out.program.or_else(|| p.program.clone());
        out.classes.extend(p.classes.iter().cloned());
        out.interfaces.extend(p.interfaces.iter().cloned());
        out.forms.extend(p.forms.iter().cloned());
        out.function_modules
            .extend(p.function_modules.iter().cloned());
        out.modules.extend(p.modules.iter().cloned());
        out.event_blocks.extend(p.event_blocks.iter().cloned());
        out.macros.extend(p.macros.iter().cloned());
        out.includes.extend(p.includes.iter().cloned());
        out.declarations.extend(p.declarations.iter().cloned());
        out.orphan_implementations
            .extend(p.orphan_implementations.iter().cloned());
    }
    let orphans = std::mem::take(&mut out.orphan_implementations);
    for o in orphans {
        match out
            .classes
            .iter_mut()
            .find(|c| c.name.eq_ignore_ascii_case(&o.class) && c.implementation.is_none())
        {
            Some(c) => c.implementation = Some(o.implementation),
            None => out.orphan_implementations.push(o),
        }
    }
    // types declared in one part, like the TOP include, type the others
    crate::symbols::resolve_types(&mut out);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use abap_source::{Hook, MemoryProvider};

    fn workspace() -> Workspace {
        let mut p = MemoryProvider::new();
        p.insert(
            SourceRef::program("ZREPORT"),
            "REPORT zreport.\nINCLUDE zreport_top.\nINCLUDE zreport_missing.\nSTART-OF-SELECTION.\n  go_app = zcl_app=>create( ).\n  go_app->run( ).\n  PERFORM log.\nFORM log.\n  CALL FUNCTION 'Z_LOG'.\nENDFORM.",
        );
        p.insert(
            SourceRef::program("ZREPORT_TOP"),
            "DATA go_app TYPE REF TO zcl_app.",
        );
        p.insert(
            SourceRef::class("ZCL_APP"),
            "CLASS zcl_app DEFINITION PUBLIC INHERITING FROM zcl_base.\n  PUBLIC SECTION.\n    CLASS-METHODS create RETURNING VALUE(ro) TYPE REF TO zcl_app.\n    METHODS run.\nENDCLASS.\n\
CLASS zcl_app IMPLEMENTATION.\n  METHOD create.\n    ro = NEW #( ).\n  ENDMETHOD.\n  METHOD run.\n    get_repo( )->save( ).\n    lcl_helper=>help( ).\n  ENDMETHOD.\nENDCLASS.",
        );
        p.insert(
            SourceRef::class("ZCL_APP").with_part(ClassPart::Definitions),
            "CLASS lcl_helper DEFINITION.\n  PUBLIC SECTION.\n    CLASS-METHODS help.\nENDCLASS.",
        );
        p.insert(
            SourceRef::class("ZCL_APP").with_part(ClassPart::Implementations),
            "CLASS lcl_helper IMPLEMENTATION.\n  METHOD help.\n    SUBMIZ x.\n  ENDMETHOD.\nENDCLASS.",
        );
        p.insert(
            SourceRef::class("ZCL_BASE"),
            "CLASS zcl_base DEFINITION PUBLIC.\n  PUBLIC SECTION.\n    METHODS get_repo RETURNING VALUE(ro) TYPE REF TO zif_repo.\nENDCLASS.\nCLASS zcl_base IMPLEMENTATION.\n  METHOD get_repo.\n  ENDMETHOD.\nENDCLASS.",
        );
        p.insert(
            SourceRef::interface("ZIF_REPO"),
            "INTERFACE zif_repo PUBLIC.\n  METHODS save.\nENDINTERFACE.",
        );
        Workspace::new(Box::new(p))
    }

    #[test]
    fn macros_of_other_parts() {
        let mut p = MemoryProvider::new();
        p.insert(
            SourceRef::class("ZCL_M"),
            "CLASS zcl_m DEFINITION PUBLIC.
  PUBLIC SECTION.
    METHODS run.
ENDCLASS.
CLASS zcl_m IMPLEMENTATION.
  METHOD run.
    log 'x'.
  ENDMETHOD.
ENDCLASS.",
        );
        p.insert(
            SourceRef::class("ZCL_M").with_part(ClassPart::Macros),
            "DEFINE log.
  CALL FUNCTION 'Z_LOG' EXPORTING text = &1.
END-OF-DEFINITION.",
        );
        let ws = Workspace::new(Box::new(p));
        let object = ws.object(&SourceRef::class("ZCL_M")).unwrap();
        let calls = ws.object_calls(&object);
        assert!(calls
            .iter()
            .any(|c| c.kind == m::CallKind::Function && c.name == "Z_LOG"));
    }

    /// Fails like an SAP system for the root class.
    struct NoRootClass(MemoryProvider);

    impl SourceProvider for NoRootClass {
        fn fetch(&self, r: &SourceRef) -> abap_source::Result<Option<abap_source::Source>> {
            assert!(!is_root_class(r), "{r} requested");
            self.0.fetch(r)
        }

        fn describe(&self) -> String {
            self.0.describe()
        }
    }

    #[test]
    fn root_class_is_not_requested() {
        let mut p = MemoryProvider::new();
        p.insert(
            SourceRef::class("ZCL_ANY"),
            "CLASS zcl_any DEFINITION PUBLIC INHERITING FROM object.
  PUBLIC SECTION.
    METHODS run IMPORTING io TYPE REF TO object.
ENDCLASS.
CLASS zcl_any IMPLEMENTATION.
  METHOD run.
    io->to_string( ).
    CALL METHOD io->('RUN').
  ENDMETHOD.
ENDCLASS.",
        );
        let ws = Workspace::new(Box::new(NoRootClass(p)));
        let g = ws.call_graph(&[SourceRef::class("ZCL_ANY")], 2);
        assert!(g.missing.is_empty(), "{:?}", g.missing);
        assert!(g.errors.is_empty(), "{:?}", g.errors);
        ws.logic(&SourceRef::class("ZCL_ANY"), 2);
    }

    #[test]
    fn redefinitions_in_other_objects() {
        let mut p = MemoryProvider::new();
        p.insert(
            SourceRef::class("ZCL_BASE"),
            "CLASS zcl_base DEFINITION PUBLIC.
  PUBLIC SECTION.
    METHODS save.
ENDCLASS.
CLASS zcl_base IMPLEMENTATION.
  METHOD save.
  ENDMETHOD.
ENDCLASS.",
        );
        p.insert(
            SourceRef::class("ZCL_DB"),
            "CLASS zcl_db DEFINITION PUBLIC INHERITING FROM zcl_base.
  PUBLIC SECTION.
    METHODS save REDEFINITION.
ENDCLASS.
CLASS zcl_db IMPLEMENTATION.
  METHOD save.
    DELETE FROM zorders WHERE id = 1.
  ENDMETHOD.
ENDCLASS.",
        );
        p.insert(
            SourceRef::class("ZCL_OTHER"),
            "CLASS zcl_other DEFINITION PUBLIC INHERITING FROM zcl_base.
ENDCLASS.
CLASS zcl_other IMPLEMENTATION.
ENDCLASS.",
        );
        p.insert(
            SourceRef::program("ZREPORT"),
            "REPORT zreport.
DATA go TYPE REF TO zcl_base.
START-OF-SELECTION.
  go->save( ).",
        );
        let ws = Workspace::new(Box::new(p));
        assert_eq!(ws.subclasses("zcl_base"), ["ZCL_DB", "ZCL_OTHER"]);
        let l = ws.logic(&SourceRef::program("ZREPORT"), 1);
        let effects: Vec<(&str, Option<&str>)> = l.procedures[0]
            .effects
            .iter()
            .map(|e| (e.name.as_str(), e.via[0].class.as_deref()))
            .collect();
        assert_eq!(effects, [("ZORDERS", Some("zcl_db"))]);
        // not followed into other objects: only the class the call names
        let l = ws.logic(&SourceRef::program("ZREPORT"), 0);
        assert!(l.procedures[0].effects.is_empty());
    }

    #[test]
    fn submitted_programs() {
        let mut p = MemoryProvider::new();
        p.insert(
            SourceRef::program("ZCALLER"),
            "REPORT zcaller.
DATA gv_id TYPE i.
DATA gv_count TYPE i.
START-OF-SELECTION.
  SELECT SINGLE id FROM zheads INTO @gv_id.
  SUBMIT zpost WITH p_id = gv_id WITH p_mode = 'X' AND RETURN.",
        );
        p.insert(
            SourceRef::program("ZPOST"),
            "REPORT zpost.
PARAMETERS p_id TYPE i.
PARAMETERS p_mode TYPE c.
DATA gv_total TYPE i.
INITIALIZATION.
  gv_total = 0.
START-OF-SELECTION.
  SELECT SINGLE amount FROM zitems INTO @gv_total WHERE id = @p_id.
  PERFORM post.
FORM post.
  UPDATE ztotals SET total = @gv_total WHERE id = @p_id.
ENDFORM.",
        );
        let ws = Workspace::new(Box::new(p));
        let l = ws.logic(&SourceRef::program("ZCALLER"), 1);
        let start = l
            .procedures
            .iter()
            .find(|p| p.unit.as_deref() == Some("PROG:ZCALLER"))
            .unwrap();
        // the tables of the submitted program, its own data not; ZTOTALS
        // depends on ZHEADS through gv_id passed to p_id
        let effects: Vec<String> = start
            .effects
            .iter()
            .map(|e| {
                format!(
                    "{} via {} from {} tables {}",
                    e.name,
                    e.via
                        .iter()
                        .map(|v| v.name.as_str())
                        .collect::<Vec<_>>()
                        .join(","),
                    e.inputs.join(","),
                    e.tables.join(",")
                )
            })
            .collect();
        assert_eq!(
            effects,
            [
                "ZTOTALS via ZPOST from  tables ZHEADS,ZITEMS",
                "gv_id via  from  tables ZHEADS"
            ],
            "{:#?}",
            start.effects
        );
        // not followed with depth 0
        let l = ws.logic(&SourceRef::program("ZCALLER"), 0);
        assert!(l
            .procedures
            .iter()
            .all(|p| p.effects.iter().all(|e| e.name != "ZTOTALS")));
    }

    /// A class implementing an interface method by writing a table.
    fn implementation(class: &str, interface: &str, method: &str, table: &str) -> String {
        format!(
            "CLASS {class} DEFINITION PUBLIC.
  PUBLIC SECTION.
    INTERFACES {interface}.
ENDCLASS.
CLASS {class} IMPLEMENTATION.
  METHOD {interface}~{method}.
    DELETE FROM {table} WHERE id = 1.
  ENDMETHOD.
ENDCLASS."
        )
    }

    fn hook(position: &str, code: &str) -> Hook {
        Hook {
            enhancement: "ZENH_ORDERS".into(),
            position: position.into(),
            code: code.into(),
            origin: "memory:ZENH_ORDERS".into(),
        }
    }

    /// A provider whose system cannot answer the enhancement lookup.
    struct WithoutEnhancements(MemoryProvider);

    impl SourceProvider for WithoutEnhancements {
        fn fetch(&self, r: &SourceRef) -> abap_source::Result<Option<abap_source::Source>> {
            self.0.fetch(r)
        }

        fn notes(&self) -> Vec<String> {
            vec!["enhancements: not available (no enhancements/enhoxhh)".to_string()]
        }

        fn describe(&self) -> String {
            "test".to_string()
        }
    }

    #[test]
    fn name_patterns() {
        assert!(matches("Z*", "zcl_orders"));
        assert!(matches("/ABC/*", "/ABC/CL_RATES"));
        assert!(matches("*_ENQUEUE", "/ABC/LOCK_ENQUEUE"));
        assert!(matches("Z*_TEST*", "ZCL_X_TEST_HELPER"));
        assert!(!matches("Z*", "CL_ABAP_TYPEDESCR"));
        assert!(matches("BAPI_USER_GET_DETAIL", "bapi_user_get_detail"));
        assert!(!matches("Z*X", "ZX_Y"));
        let f = Follow {
            include: vec!["Z*".into(), "/ABC/*".into()],
            exclude: vec!["ZTEST*".into()],
            limit: None,
        };
        assert!(f.allows("ZCL_X") && f.allows("/ABC/CL_Y"));
        assert!(!f.allows("ZTEST_X") && !f.allows("CL_GUI_ALV_GRID"));
        assert!(Follow::default().allows("ANYTHING"));
    }

    /// Sources of a report calling custom and standard code.
    fn report() -> MemoryProvider {
        let mut p = MemoryProvider::new();
        p.insert(
            SourceRef::program("ZMAIN"),
            "REPORT zmain.
START-OF-SELECTION.
  CALL FUNCTION 'Z_ONE'.
  CALL FUNCTION 'Z_TWO'.
  CALL FUNCTION 'BAPI_USER_GET_DETAIL'.
  CALL FUNCTION 'Z_GONE'.
  CALL FUNCTION lv_name.
  lo_unknown->run( ).
  CALL TRANSACTION 'VA01'.",
        );
        for (name, body) in [
            ("Z_ONE", "  DELETE FROM zone WHERE id = 1."),
            ("Z_TWO", "  DELETE FROM ztwo WHERE id = 1."),
            (
                "BAPI_USER_GET_DETAIL",
                "  SELECT * FROM usr02 INTO TABLE @DATA(lt).",
            ),
        ] {
            p.insert(
                SourceRef::function_module(name),
                format!("FUNCTION {name}.\n{body}\nENDFUNCTION."),
            );
        }
        p
    }

    fn reasons(g: &m::CallGraph) -> Vec<String> {
        g.calls
            .iter()
            .map(|c| format!("{} {:?}", c.name, c.unresolved))
            .collect()
    }

    #[test]
    fn calls_say_why_they_are_unresolved() {
        let ws = Workspace::new(Box::new(report()));
        let g = ws.call_graph(&[SourceRef::program("ZMAIN")], 1);
        assert_eq!(
            reasons(&g),
            [
                "Z_ONE None",
                "Z_TWO None",
                "BAPI_USER_GET_DETAIL None",
                "Z_GONE Some(Missing)",
                "lv_name Some(Dynamic)",
                "run Some(UnknownType)",
                "VA01 Some(NotLoaded)",
            ]
        );
    }

    #[test]
    fn follow_patterns_and_limit() {
        let follow = Follow {
            include: vec!["Z*".into()],
            exclude: vec![],
            limit: None,
        };
        let ws = Workspace::new(Box::new(report())).with_follow(follow.clone());
        let g = ws.call_graph(&[SourceRef::program("ZMAIN")], 1);
        assert!(reasons(&g).contains(&"BAPI_USER_GET_DETAIL Some(NotFollowed)".to_string()));
        assert_eq!(g.skipped, ["FUNC:BAPI_USER_GET_DETAIL"]);
        // not followed is not missing
        assert_eq!(g.missing, ["FUNC:Z_GONE"]);
        // the root is loaded even when the patterns leave it out
        let ws = Workspace::new(Box::new(report())).with_follow(Follow {
            include: vec!["Y*".into()],
            ..follow.clone()
        });
        let g = ws.call_graph(&[SourceRef::program("ZMAIN")], 1);
        assert_eq!(g.units[0].reference, "PROG:ZMAIN");
        // the root and one more object
        let ws = Workspace::new(Box::new(report())).with_follow(Follow {
            limit: Some(2),
            ..Follow::default()
        });
        let g = ws.call_graph(&[SourceRef::program("ZMAIN")], 1);
        let limited: Vec<&str> = g
            .calls
            .iter()
            .filter(|c| c.unresolved == Some(m::Unresolved::Limit))
            .map(|c| c.name.as_str())
            .collect();
        assert_eq!(limited.len(), 3, "{limited:?}");
        assert!(g.notes.iter().any(|n| n.starts_with("limit: 2 objects")));
        // logic too: the effects of the objects followed only
        let ws = Workspace::new(Box::new(report())).with_follow(follow);
        let l = ws.logic(&SourceRef::program("ZMAIN"), 1);
        let effects: Vec<&str> = l.procedures[0]
            .effects
            .iter()
            .map(|e| e.name.as_str())
            .collect();
        assert_eq!(effects, ["ZONE", "ZTWO"]);
        assert_eq!(l.skipped, ["FUNC:BAPI_USER_GET_DETAIL"]);
        assert_eq!(l.missing, ["FUNC:Z_GONE"]);
    }

    #[test]
    fn progress_and_cancel() {
        let seen = Arc::new(Mutex::new(Vec::new()));
        let cancel = Arc::new(AtomicBool::new(false));
        let (log, stop) = (seen.clone(), cancel.clone());
        let ws = Workspace::new(Box::new(report()))
            .with_progress(Box::new(move |p| {
                log.lock().unwrap().push(p.object.clone());
                // cancelled after the first object called
                if p.object == "FUNC:Z_ONE" {
                    stop.store(true, Ordering::Relaxed);
                }
            }))
            .with_cancel(cancel);
        // one by one, so the order is known
        IN_PARALLEL.with(|p| p.set(true));
        let g = ws.call_graph(&[SourceRef::program("ZMAIN")], 1);
        IN_PARALLEL.with(|p| p.set(false));
        assert_eq!(*seen.lock().unwrap(), ["PROG:ZMAIN", "FUNC:Z_ONE"]);
        assert!(reasons(&g).contains(&"Z_TWO Some(Cancelled)".to_string()));
        assert!(g.notes.iter().any(|n| n.starts_with("cancelled")));
    }

    /// A provider failing for one object.
    struct Failing(MemoryProvider);

    impl SourceProvider for Failing {
        fn fetch(&self, r: &SourceRef) -> abap_source::Result<Option<abap_source::Source>> {
            match r.name.as_str() {
                "Z_TWO" => Err(SourceError::Http {
                    status: 503,
                    url: "https://sap/sap/bc/adt/functions/groups/z/fmodules/z_two".into(),
                }),
                "Z_ONE" => Err(SourceError::Auth {
                    url: "https://sap/sap/bc/adt/functions/groups/z/fmodules/z_one".into(),
                }),
                _ => self.0.fetch(r),
            }
        }

        fn describe(&self) -> String {
            "test".to_string()
        }
    }

    #[test]
    fn errors_are_structured() {
        let ws = Workspace::new(Box::new(Failing(report())));
        let g = ws.call_graph(&[SourceRef::program("ZMAIN")], 1);
        let mut errors = g.errors.clone();
        errors.sort_by(|a, b| a.object.cmp(&b.object));
        let summary: Vec<(&str, m::ErrorCode, bool, Option<u16>)> = errors
            .iter()
            .map(|e| (e.object.as_str(), e.code, e.retryable, e.status))
            .collect();
        assert_eq!(
            summary,
            [
                ("FUNC:Z_ONE", m::ErrorCode::Auth, false, None),
                ("FUNC:Z_TWO", m::ErrorCode::Http, true, Some(503)),
            ]
        );
        assert!(errors[1].cause.starts_with("HTTP 503 for https://sap/"));
        assert!(reasons(&g).contains(&"Z_TWO Some(LookupFailed)".to_string()));
        // a failed object is not missing
        assert_eq!(g.missing, ["FUNC:Z_GONE"]);
    }

    #[test]
    fn notes_of_the_provider_are_in_the_results() {
        let mut p = MemoryProvider::new();
        p.insert(
            SourceRef::program("ZREPORT"),
            "REPORT zreport.
WRITE 1.",
        );
        let ws = Workspace::new(Box::new(WithoutEnhancements(p)));
        let graph = ws.call_graph(&[SourceRef::program("ZREPORT")], 1);
        assert_eq!(
            graph.notes,
            ["enhancements: not available (no enhancements/enhoxhh)"]
        );
        assert!(graph.errors.is_empty());
        let logic = ws.logic(&SourceRef::program("ZREPORT"), 0);
        assert_eq!(logic.notes, graph.notes);
        assert!(crate::text::call_graph(&graph).contains("notes: enhancements: not available"));
    }

    #[test]
    fn parallel_loads_keep_their_order() {
        let items: Vec<usize> = (0..100).collect();
        let open = AtomicUsize::new(0);
        let most = AtomicUsize::new(0);
        let out = parallel_map(&items, |i| {
            let now = open.fetch_add(1, Ordering::SeqCst) + 1;
            most.fetch_max(now, Ordering::SeqCst);
            // work started in a thread stays in it
            let inner = parallel_map(&[1, 2, 3], |x| x * 2);
            std::thread::sleep(std::time::Duration::from_millis(1));
            open.fetch_sub(1, Ordering::SeqCst);
            i * 10 + inner.len()
        });
        assert_eq!(out, (0..100).map(|i| i * 10 + 3).collect::<Vec<_>>());
        assert!(most.load(Ordering::SeqCst) <= PARALLEL_LOADS);
        assert!(most.load(Ordering::SeqCst) > 1);
    }

    #[test]
    fn objects_load_once_across_threads() {
        let mut p = MemoryProvider::new();
        p.insert(SourceRef::program("ZREPORT"), "REPORT zreport.");
        let ws = Workspace::new(Box::new(p));
        let refs: Vec<SourceRef> = (0..20).map(|_| SourceRef::program("ZREPORT")).collect();
        let objects = parallel_map(&refs, |r| ws.object(r).unwrap());
        assert!(objects.iter().all(|o| Arc::ptr_eq(o, &objects[0])));
    }

    #[test]
    fn enhancements_run_where_they_are() {
        let mut p = MemoryProvider::new();
        p.insert(
            SourceRef::program("ZORDERS"),
            "REPORT zorders.
INCLUDE zorders_f01.
START-OF-SELECTION.
  PERFORM save USING 1.",
        );
        p.insert(
            SourceRef::program("ZORDERS_F01"),
            "FORM save USING iv_id TYPE i.
  ENHANCEMENT-POINT ep_save SPOTS zes_orders.
  UPDATE zorders SET status = 'S' WHERE id = iv_id.
ENDFORM.",
        );
        p.insert_hook(hook(
            r"\PR:ZORDERS\EX:EP_SAVE\EI",
            "ENHANCEMENT 1 ZENH_ORDERS.    \"active version
  DELETE FROM zorder_log WHERE id = iv_id.
  CALL FUNCTION 'Z_NOTIFY'.
ENDENHANCEMENT.",
        ));
        p.insert_hook(hook(
            r"\PR:ZORDERS\FO:SAVE\SE:END\EI",
            "ENHANCEMENT 2 ZENH_ORDERS.
  INSERT zorder_audit FROM @( VALUE #( id = iv_id ) ).
ENDENHANCEMENT.",
        ));
        // of another program
        p.insert_hook(hook(
            r"\PR:ZOTHER\FO:SAVE\SE:END\EI",
            "ENHANCEMENT 3 ZENH_ORDERS.
  DELETE FROM zother.
ENDENHANCEMENT.",
        ));
        let ws = Workspace::new(Box::new(p));

        let graph = ws.call_graph(&[SourceRef::program("ZORDERS")], 1);
        let notify = graph.calls.iter().find(|c| c.name == "Z_NOTIFY").unwrap();
        // on the row of the enhancement point
        assert_eq!(notify.span.start.row, 2);
        assert_eq!(notify.unit.as_deref(), Some("PROG:ZORDERS_F01"));
        assert!(graph
            .units
            .iter()
            .any(|u| u.reference == "ENHO:ZENH_ORDERS" && u.origin == "memory:ZENH_ORDERS"));

        let l = ws.logic(&SourceRef::program("ZORDERS"), 1);
        let save = l
            .procedures
            .iter()
            .find(|p| p.block.name.eq_ignore_ascii_case("save"))
            .unwrap();
        let effects: Vec<&str> = save
            .effects
            .iter()
            .filter(|e| e.scope == m::DataScope::Database)
            .map(|e| e.name.as_str())
            .collect();
        assert_eq!(
            effects,
            ["ZORDERS", "ZORDER_AUDIT", "ZORDER_LOG"],
            "{:#?}",
            save.effects
        );
    }

    #[test]
    fn badis_and_customer_exits() {
        let mut p = MemoryProvider::new();
        // a BAdI of an enhancement spot: one active implementation, one
        // class implementing the interface without being active
        p.insert(
            SourceRef::interface("ZIF_BADI_CHECK"),
            "INTERFACE zif_badi_check PUBLIC.\n  METHODS check.\nENDINTERFACE.",
        );
        p.insert(
            SourceRef::class("ZCL_CHECK_ACTIVE"),
            implementation("zcl_check_active", "zif_badi_check", "check", "zactive"),
        );
        p.insert(
            SourceRef::class("ZCL_CHECK_INACTIVE"),
            implementation("zcl_check_inactive", "zif_badi_check", "check", "zinactive"),
        );
        p.insert_badi(Badi {
            name: "ZBADI_CHECK".into(),
            interface: Some("ZIF_BADI_CHECK".into()),
            implementations: vec!["ZCL_CHECK_ACTIVE".into()],
            fallback: None,
            classic: false,
        });
        // a classic BAdI: the generated adapter class implements the
        // interface too, but runs no code of its own
        p.insert(
            SourceRef::interface("ZIF_EX_OLD"),
            "INTERFACE zif_ex_old PUBLIC.\n  METHODS run.\nENDINTERFACE.",
        );
        p.insert(
            SourceRef::class("ZCL_IM_OLD"),
            implementation("zcl_im_old", "zif_ex_old", "run", "zold"),
        );
        p.insert(
            SourceRef::class("ZCL_EX_OLD"),
            implementation("zcl_ex_old", "zif_ex_old", "run", "zadapter"),
        );
        p.insert_badi(Badi {
            name: "ZOLD".into(),
            interface: Some("ZIF_EX_OLD".into()),
            implementations: vec!["ZCL_IM_OLD".into()],
            fallback: None,
            classic: true,
        });
        // a customer exit of the program, in its function group
        p.insert(
            SourceRef::function_module("EXIT_ZREPORT_001"),
            "FUNCTION exit_zreport_001.\n  DELETE FROM zexit WHERE id = 1.\nENDFUNCTION.",
        );
        p.insert(
            SourceRef::program("ZREPORT"),
            "REPORT zreport.
DATA lo_badi TYPE REF TO zbadi_check.
DATA lo_old TYPE REF TO zif_ex_old.
START-OF-SELECTION.
  GET BADI lo_badi.
  CALL BADI lo_badi->check.
  CALL METHOD cl_exithandler=>get_instance CHANGING instance = lo_old.
  lo_old->run( ).
  CALL CUSTOMER-FUNCTION '001'.",
        );
        let ws = Workspace::new(Box::new(p));
        assert_eq!(ws.implementations("zif_ex_old"), ["ZCL_IM_OLD"]);

        let graph = ws.call_graph(&[SourceRef::program("ZREPORT")], 1);
        let badi = graph.calls.iter().find(|c| c.name == "check").unwrap();
        assert_eq!(badi.badi.as_deref(), Some("ZBADI_CHECK"));
        assert_eq!(badi.class.as_deref(), Some("ZIF_BADI_CHECK"));
        assert!(badi.resolved);
        let exit = graph
            .calls
            .iter()
            .find(|c| c.kind == m::CallKind::Function)
            .unwrap();
        assert_eq!(exit.name, "EXIT_ZREPORT_001");
        assert!(exit.resolved);
        let units: Vec<&str> = graph.units.iter().map(|u| u.reference.as_str()).collect();
        for loaded in [
            "CLAS:ZCL_CHECK_ACTIVE",
            "CLAS:ZCL_IM_OLD",
            "FUNC:EXIT_ZREPORT_001",
        ] {
            assert!(units.contains(&loaded), "{units:?}");
        }
        for not_run in ["CLAS:ZCL_CHECK_INACTIVE", "CLAS:ZCL_EX_OLD"] {
            assert!(!units.contains(&not_run), "{units:?}");
        }

        // only the active implementations run
        let l = ws.logic(&SourceRef::program("ZREPORT"), 1);
        let mut tables: Vec<&str> = l.procedures[0]
            .effects
            .iter()
            .filter(|e| e.scope == m::DataScope::Database)
            .map(|e| e.name.as_str())
            .collect();
        tables.sort();
        assert_eq!(tables, ["ZACTIVE", "ZEXIT", "ZOLD"]);
    }

    #[test]
    fn calls_through_dictionary_types() {
        let mut p = MemoryProvider::new();
        p.insert(
            SourceRef::interface("ZIF_ID"),
            "INTERFACE zif_id PUBLIC.\n  METHODS get_name.\nENDINTERFACE.",
        );
        // DDL as from an SAP system, with an include
        p.insert(
            SourceRef::table("ZS_NODE"),
            "define structure zs_node {\n  include zs_key;\n  idref : zid_ref;\n}",
        );
        p.insert(
            SourceRef::table("ZS_KEY"),
            "define structure zs_key {\n  owner : reference to zif_id;\n}",
        );
        // abapGit data element typed as a reference
        p.insert(
            SourceRef::data_element("ZID_REF"),
            "<DD04V><ROLLNAME>ZID_REF</ROLLNAME><DOMNAME>ZIF_ID</DOMNAME><DATATYPE>REF</DATATYPE><REFKIND>R</REFKIND><REFTYPE>I</REFTYPE></DD04V>",
        );
        p.insert(
            SourceRef::class("ZCL_USE"),
            "CLASS zcl_use DEFINITION PUBLIC.
  PUBLIC SECTION.
    TYPES: BEGIN OF ty_local,
             id TYPE REF TO zif_id,
           END OF ty_local.
    METHODS run IMPORTING io_id TYPE zid_ref ir_node TYPE REF TO zs_node.
ENDCLASS.
CLASS zcl_use IMPLEMENTATION.
  METHOD run.
    DATA ls_node TYPE zs_node.
    DATA ls_local TYPE ty_local.
    io_id->get_name( ).
    ir_node->idref->get_name( ).
    ls_node-idref->get_name( ).
    ls_node-owner->get_name( ).
    ls_local-id->get_name( ).
    DATA(lo_id) = ls_node-idref.
    lo_id->get_name( ).
  ENDMETHOD.
ENDCLASS.",
        );
        let ws = Workspace::new(Box::new(p));
        let g = ws.call_graph(&[SourceRef::class("ZCL_USE")], 1);
        let got: Vec<String> = g
            .calls
            .iter()
            .map(|c| {
                format!(
                    "{:?} {}",
                    c.class.as_deref().map(str::to_uppercase),
                    c.resolved
                )
            })
            .collect();
        assert_eq!(got, vec!["Some(\"ZIF_ID\") true"; 6], "{:#?}", g.calls);
        assert!(g.errors.is_empty(), "{:?}", g.errors);
    }

    #[test]
    fn includes_and_parts() {
        let ws = workspace();
        let prog = ws.object(&SourceRef::program("ZREPORT")).unwrap();
        assert_eq!(prog.units.len(), 2);
        assert_eq!(
            prog.missing_includes,
            [SourceRef::program("ZREPORT_MISSING")]
        );
        assert_eq!(prog.symbols.declarations[0].name, "go_app");

        let class = ws.object(&SourceRef::class("ZCL_APP")).unwrap();
        assert_eq!(class.units.len(), 3);
        let helper = class
            .symbols
            .classes
            .iter()
            .find(|c| c.name == "lcl_helper")
            .unwrap();
        assert!(helper.implementation.is_some());
    }

    #[test]
    fn call_graph_across_objects() {
        let ws = workspace();
        let g = ws.call_graph(&[SourceRef::program("ZREPORT")], 2);
        let calls: Vec<String> = g
            .calls
            .iter()
            .map(|c| {
                format!(
                    "{} {:?} {}{}{}",
                    c.unit.as_deref().unwrap_or(""),
                    c.kind,
                    c.class
                        .as_deref()
                        .map(|x| format!("{x}."))
                        .unwrap_or_default(),
                    c.name,
                    if c.resolved { " resolved" } else { "" }
                )
            })
            .collect();
        assert_eq!(
            calls,
            [
                "PROG:ZREPORT Include ZREPORT_TOP resolved",
                "PROG:ZREPORT Include ZREPORT_MISSING",
                "PROG:ZREPORT Method zcl_app.create resolved",
                "PROG:ZREPORT Method zcl_app.run resolved",
                "PROG:ZREPORT Form log resolved",
                "PROG:ZREPORT Function Z_LOG",
                "CLAS:ZCL_APP Constructor zcl_app.constructor resolved",
                "CLAS:ZCL_APP Method zcl_app.get_repo resolved",
                "CLAS:ZCL_APP Method zif_repo.save resolved",
                "CLAS:ZCL_APP Method lcl_helper.help resolved",
            ]
        );
        assert_eq!(g.missing, ["FUNC:Z_LOG", "PROG:ZREPORT_MISSING"]);
        // zcl_base was loaded as superclass for the type of get_repo
        assert!(g
            .units
            .iter()
            .any(|u| u.reference == "CLAS:ZCL_APP/implementations"));
    }
}
