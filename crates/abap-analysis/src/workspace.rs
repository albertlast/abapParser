//! Objects loaded through a source provider: include expansion, symbols
//! across the parts of an object, type resolution across objects and the
//! call graph.

use std::collections::{BTreeSet, HashMap, VecDeque};
use std::sync::{Arc, Mutex};

use abap_model as m;
use abap_source::{ClassPart, ObjectKind, SourceProvider, SourceRef};
use abap_syntax::{File, StatementKind as K};

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
}

pub struct Workspace {
    provider: Box<dyn SourceProvider>,
    units: Mutex<HashMap<SourceRef, Option<Arc<Unit>>>>,
    objects: Mutex<HashMap<SourceRef, Option<Arc<Object>>>>,
    implementations: Mutex<HashMap<String, Vec<String>>>,
    structures: Mutex<HashMap<String, Option<Arc<ddic::Structure>>>>,
    data_elements: Mutex<HashMap<String, Option<String>>>,
    errors: Mutex<Vec<String>>,
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
            units: Mutex::new(HashMap::new()),
            objects: Mutex::new(HashMap::new()),
            implementations: Mutex::new(HashMap::new()),
            structures: Mutex::new(HashMap::new()),
            data_elements: Mutex::new(HashMap::new()),
            errors: Mutex::new(Vec::new()),
        }
    }

    /// Provider errors so far, e.g. failed authorization.
    pub fn errors(&self) -> Vec<String> {
        self.errors.lock().unwrap().clone()
    }

    /// One source, parsed; `None` if no provider has it or fetching failed.
    pub fn unit(&self, r: &SourceRef) -> Option<Arc<Unit>> {
        if let Some(u) = self.units.lock().unwrap().get(r) {
            return u.clone();
        }
        let unit = match self.provider.fetch(r) {
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
                self.errors.lock().unwrap().push(format!("{r}: {e}"));
                None
            }
        };
        self.units.lock().unwrap().insert(r.clone(), unit.clone());
        unit
    }

    /// The text of a dictionary type, `None` if no provider has it.
    fn fetch_text(&self, r: &SourceRef) -> Option<String> {
        match self.provider.fetch(r) {
            Ok(s) => s.map(|s| s.text),
            Err(e) => {
                self.errors.lock().unwrap().push(format!("{r}: {e}"));
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
        if let Some(o) = self.objects.lock().unwrap().get(&r) {
            return o.clone();
        }
        let object = self.load_object(&r).map(Arc::new);
        self.objects.lock().unwrap().insert(r, object.clone());
        object
    }

    fn load_object(&self, r: &SourceRef) -> Option<Object> {
        let main = self.unit(r)?;
        let mut units = vec![main];
        let mut missing_includes = Vec::new();
        match r.kind {
            ObjectKind::Clas => {
                for part in ClassPart::ALL {
                    if let Some(u) = self.unit(&r.clone().with_part(part)) {
                        units.push(u);
                    }
                }
            }
            ObjectKind::Fugr => {
                if let Ok(fms) = self.provider.function_modules(&r.name) {
                    for fm in fms {
                        if let Some(u) = self.unit(&SourceRef::function_module(&fm)) {
                            units.push(u);
                        }
                    }
                }
            }
            _ => {}
        }
        // INCLUDE statements, recursively
        let mut queue: VecDeque<Arc<Unit>> = units.iter().cloned().collect();
        let mut seen: BTreeSet<SourceRef> = units.iter().map(|u| u.reference.clone()).collect();
        while let Some(u) = queue.pop_front() {
            for name in includes(&u.file) {
                let inc = SourceRef::program(&name);
                if seen.len() >= MAX_INCLUDES || !seen.insert(inc.clone()) {
                    continue;
                }
                match self.unit(&inc) {
                    Some(iu) => {
                        units.push(iu.clone());
                        queue.push_back(iu);
                    }
                    None => missing_includes.push(inc),
                }
            }
        }
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
        })
    }

    /// Global classes implementing an interface, as far as the provider can
    /// list them, upper case.
    pub fn implementations(&self, interface: &str) -> Vec<String> {
        let key = interface.to_ascii_uppercase();
        if let Some(c) = self.implementations.lock().unwrap().get(&key) {
            return c.clone();
        }
        let candidates = match self.provider.implementations(&key) {
            Ok(c) => c,
            Err(e) => {
                self.errors.lock().unwrap().push(format!("{key}: {e}"));
                Vec::new()
            }
        };
        // a where-used list also names classes only using the interface:
        // keep those whose definition implements it; long lists are kept
        // unchecked, they are too many to follow anyway
        let classes = if candidates.len() > MAX_CHECKED_IMPLEMENTATIONS {
            candidates
        } else {
            candidates
                .into_iter()
                .filter(|c| {
                    self.object(&SourceRef::class(c)).is_some_and(|o| {
                        type_info(&o.symbols, c).is_some_and(|i| {
                            i.interfaces.iter().any(|x| x.eq_ignore_ascii_case(&key))
                        })
                    })
                })
                .collect()
        };
        self.implementations
            .lock()
            .unwrap()
            .insert(key, classes.clone());
        classes
    }

    /// The object of a class or interface name, whichever exists.
    pub fn type_object(&self, name: &str) -> Option<Arc<Object>> {
        self.object(&SourceRef::class(name))
            .or_else(|| self.object(&SourceRef::interface(name)))
    }

    /// Calls of an object, types resolved through the workspace.
    pub fn object_calls(&self, object: &Object) -> Vec<m::Call> {
        let resolver = WorkspaceResolver { workspace: self };
        let local = SymbolsResolver {
            symbols: &object.symbols,
            fallback: Some(&resolver),
        };
        let mut out = Vec::new();
        for u in &object.units {
            let calls = CallExtractor {
                file: &u.file,
                symbols: &object.symbols,
                resolver: &local,
            }
            .calls();
            for mut c in calls {
                c.unit = Some(u.reference.to_string());
                out.push(c);
            }
        }
        out
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
            errors: Vec::new(),
        };
        let mut queue: VecDeque<(SourceRef, u32)> = roots.iter().map(|r| (r.object(), 0)).collect();
        let mut seen: BTreeSet<SourceRef> = queue.iter().map(|(r, _)| r.clone()).collect();
        let mut missing: BTreeSet<String> = BTreeSet::new();
        while let Some((r, d)) = queue.pop_front() {
            let Some(object) = self.object(&r) else {
                missing.insert(r.to_string());
                continue;
            };
            for u in &object.units {
                graph.units.push(m::GraphUnit {
                    reference: u.reference.to_string(),
                    origin: u.origin.clone(),
                    depth: d,
                });
            }
            missing.extend(object.missing_includes.iter().map(|i| i.to_string()));
            if d >= depth {
                continue;
            }
            for mut c in self.object_calls(&object) {
                for target in self.targets(&c) {
                    if seen.insert(target.clone()) {
                        queue.push_back((target, d + 1));
                    }
                }
                c.resolved = self.resolved(&c, &object);
                if !c.resolved && matches!(c.kind, m::CallKind::Method | m::CallKind::Constructor) {
                    if let Some(class) = c.class.as_deref() {
                        c.hierarchy_incomplete = !self.hierarchy_known(class, &object, 0);
                    }
                }
                graph.calls.push(c);
            }
        }
        graph.missing = missing.into_iter().collect();
        graph.errors = self.errors();
        graph
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
            m::CallKind::Method | m::CallKind::Constructor => c
                .class
                .as_deref()
                .and_then(|cls| self.type_object(cls))
                .map(|o| vec![o.reference.clone()])
                .unwrap_or_default(),
            _ => Vec::new(),
        }
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
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use abap_source::MemoryProvider;

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
