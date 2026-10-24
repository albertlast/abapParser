//! The logic of procedures: statements as a tree along the control
//! structures, the conditions every statement runs under, a control flow
//! graph, and the data flow from writes to reads (reaching definitions).
//!
//! Calls are followed: what a called procedure reads and writes outside of
//! itself (attributes, global data, database tables) is added to the calling
//! statement before the data flow of the caller is computed. For one file
//! the procedures of that file are followed, through a [`Workspace`] also
//! those of other objects.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::sync::{Arc, OnceLock};

use abap_model as m;
use abap_source::SourceRef;
use abap_syntax::expr::{Access, Cond, CondKind, DataRef, Expr, ExprKind, Head, Link};
use abap_syntax::operands::operands;
use abap_syntax::structure;
use abap_syntax::system::{self, SystemField};
use abap_syntax::{decl, join_tokens, File, Statement, StatementKind as K, Token, TokenKind};

use crate::args::{call_args, Actual, At, CallArgs};
use crate::calls::{span_of, type_info, CallExtractor, SymbolsResolver};
use crate::context::statement_blocks;
use crate::db::{db_access_with, literal_constants};
use crate::workspace::{Object, Workspace};

/// The logic of the procedures of an ABAP source.
pub fn logic(source: &str) -> m::FileLogic {
    logic_of(&abap_syntax::parse(source))
}

/// Like [`logic`], for a parsed file.
pub fn logic_of(file: &File) -> m::FileLogic {
    let symbols = Arc::new(crate::symbols_of(file));
    let resolver = SymbolsResolver {
        symbols: &symbols,
        fallback: None,
    };
    let calls = CallExtractor {
        file,
        symbols: &symbols,
        resolver: &resolver,
        program: crate::calls::main_program(&symbols),
    }
    .calls();
    let attributes = |class: &str| class_attributes(&symbols, None, class);
    let procs = build_unit(
        file,
        None,
        &symbols,
        &globals_of(&symbols),
        &attributes,
        calls,
    );
    let mut engine = Engine::new(None, 0);
    engine.objects.push(ObjectLogic {
        reference: None,
        symbols: symbols.clone(),
        procs,
        depth: 0,
    });
    m::FileLogic {
        schema_version: m::SCHEMA_VERSION,
        object: None,
        procedures: engine.finish(0),
        missing: Vec::new(),
        skipped: Vec::new(),
        errors: Vec::new(),
        notes: Vec::new(),
    }
}

impl Workspace {
    /// The logic of all sources of an object: class parts, includes,
    /// function modules. Data declared at program level in any of them, e.g.
    /// in the TOP include, has the scope `global`. Calls into other objects
    /// are followed up to `depth` hops, calls within the object always.
    pub fn logic(&self, root: &SourceRef, depth: u32) -> m::FileLogic {
        // the engine loads callees one by one: load them before, several at once
        self.add_roots(std::slice::from_ref(root));
        self.preload(std::slice::from_ref(root), depth);
        let mut engine = Engine::new(Some(self), depth);
        let procedures = match engine.load(root, 0) {
            Some(o) => engine.finish(o),
            None => Vec::new(),
        };
        m::FileLogic {
            schema_version: m::SCHEMA_VERSION,
            object: Some(root.object().to_string()),
            procedures,
            missing: engine.missing.into_iter().collect(),
            skipped: self.skipped(),
            errors: self.errors(),
            notes: self.notes(),
        }
    }
}

// ----- following calls -------------------------------------------------------

/// Calls through an interface are followed into at most this many
/// implementations; with more, which one runs says too little.
pub(crate) const MAX_IMPLEMENTATIONS: usize = 10;

/// Subclasses looked at for redefinitions of a called method; with more,
/// a class is too general to tell which redefinition runs.
const MAX_SUBCLASSES: usize = 50;

/// What a written value depends on in the procedure writing it, in its names.
#[derive(Clone, Default)]
struct Deps {
    /// data outside of the procedure
    reads: Vec<(String, m::DataScope)>,
    /// parameters, so the arguments of a call
    formals: Vec<String>,
    tables: Vec<String>,
}

/// A parameter a procedure writes.
#[derive(Clone)]
struct ParamWrite {
    formal: String,
    deps: Deps,
    /// written through the reference it holds: the paths behind it, like
    /// `->*` or `->mv_x`
    through: Vec<String>,
    /// data outside of the procedure a reference parameter is bound to
    refs: Vec<(String, m::DataScope)>,
}

/// What a procedure reads and writes outside of itself, as its callers see it.
#[derive(Clone, Default)]
struct Summary {
    parameters: Vec<m::ProcedureParameter>,
    reads: Vec<(String, m::DataScope)>,
    writes: Vec<(String, m::DataScope, Deps)>,
    params: Vec<ParamWrite>,
    /// tables read, and tables written with what the rows depend on
    tables: Vec<(m::TableAccess, Option<Deps>)>,
}

/// A field symbol or reference bound by a step to data: (step, alias,
/// target, scope of the target).
type Binding = (u32, String, String, m::DataScope);

/// A call to follow.
struct PendingCall {
    step: u32,
    call: m::Call,
    args: Option<CallArgs>,
    /// data deciding which procedure runs: the operand of a dynamic call
    deciding: Vec<String>,
}

/// A procedure with the calls of its steps, before and after the data flow.
struct Proc {
    procedure: m::Procedure,
    /// calls with the step they are in and their arguments, until followed
    calls: Vec<PendingCall>,
    /// references bound through calls, e.g. `lr = get_ref( )`
    bindings: Vec<Binding>,
    summary: Option<Summary>,
    /// being analyzed: a call back to it is recursion
    running: bool,
}

struct ObjectLogic {
    reference: Option<SourceRef>,
    symbols: Arc<m::FileSymbols>,
    procs: Vec<Proc>,
    /// hops from the root object
    depth: u32,
}

struct Engine<'w> {
    workspace: Option<&'w Workspace>,
    max_depth: u32,
    objects: Vec<ObjectLogic>,
    loaded: HashMap<SourceRef, Option<usize>>,
    missing: BTreeSet<String>,
}

/// Whose attributes a called method works on.
enum Receiver {
    /// the caller's own object or class
    Own,
    /// a static method of another class, lower case
    Class(String),
    /// a method of another object, the reference as written in lower case
    Object(String),
    /// e.g. a new object or a method chain
    Unknown,
}

impl<'w> Engine<'w> {
    fn new(workspace: Option<&'w Workspace>, max_depth: u32) -> Self {
        Engine {
            workspace,
            max_depth,
            objects: Vec::new(),
            loaded: HashMap::new(),
            missing: BTreeSet::new(),
        }
    }

    /// The procedures of an object, analyzed.
    fn finish(&mut self, o: usize) -> Vec<m::Procedure> {
        for k in 0..self.objects[o].procs.len() {
            self.summary(o, k);
        }
        std::mem::take(&mut self.objects[o].procs)
            .into_iter()
            .map(|p| p.procedure)
            .collect()
    }

    /// An object loaded through the workspace, `depth` hops from the root.
    fn load(&mut self, r: &SourceRef, depth: u32) -> Option<usize> {
        let r = r.object();
        if let Some(o) = self.loaded.get(&r) {
            return *o;
        }
        let ws = self.workspace?;
        if depth > self.max_depth {
            return None;
        }
        let Some(object) = ws.object(&r) else {
            // not followed, beyond the limit or cancelled: in `skipped`;
            // failed: in `errors`
            if ws.skip_reason(&r).is_none() && !ws.failed(&r) {
                self.missing.insert(r.to_string());
            }
            self.loaded.insert(r, None);
            return None;
        };
        self.missing
            .extend(object.missing_includes.iter().map(|i| i.to_string()));
        let (symbols, procs) = build_object(ws, &object);
        self.objects.push(ObjectLogic {
            reference: Some(r.clone()),
            symbols,
            procs,
            depth,
        });
        let o = self.objects.len() - 1;
        self.loaded.insert(r, Some(o));
        Some(o)
    }

    fn find(&self, o: usize, kind: &str, class: Option<&str>, name: &str) -> Option<usize> {
        self.objects[o].procs.iter().position(|p| {
            let b = &p.procedure.block;
            b.kind == kind
                && b.name.eq_ignore_ascii_case(name)
                && class.is_none_or(|c| {
                    b.class
                        .as_deref()
                        .is_some_and(|x| x.eq_ignore_ascii_case(c))
                })
        })
    }

    /// Whether `program` is the object `o` itself.
    fn is_program(&self, o: usize, program: &str) -> bool {
        let object = &self.objects[o];
        object
            .reference
            .as_ref()
            .is_some_and(|r| r.name.eq_ignore_ascii_case(program))
            || object
                .symbols
                .program
                .as_ref()
                .is_some_and(|p| p.name.eq_ignore_ascii_case(program))
    }

    /// The object defining a class or interface: `o` itself or a loaded one.
    fn holder(&mut self, o: usize, class: &str, depth: u32) -> Option<usize> {
        if type_info(&self.objects[o].symbols, class).is_some() {
            return Some(o);
        }
        let r = self.workspace?.type_object(class)?.reference.clone();
        self.load(&r, depth)
    }

    /// A method of a class or of its superclasses, aliases resolved.
    fn method_in(
        &mut self,
        o: usize,
        class: &str,
        name: &str,
        depth: u32,
    ) -> Option<(usize, usize)> {
        let mut class = class.to_string();
        for _ in 0..10 {
            let holder = self.holder(o, &class, depth)?;
            if let Some(k) = self.find(holder, "method", Some(&class), name) {
                return Some((holder, k));
            }
            let info = type_info(&self.objects[holder].symbols, &class)?;
            let alias = info
                .aliases
                .iter()
                .find(|a| a.name.eq_ignore_ascii_case(name))
                .map(|a| a.component.clone());
            let superclass = info.superclass.map(str::to_string);
            if let Some(k) = alias.and_then(|a| self.find(holder, "method", Some(&class), &a)) {
                return Some((holder, k));
            }
            class = superclass?;
        }
        None
    }

    /// Classes implementing an interface: in the object `o` and, through the
    /// workspace, in the sources.
    fn implementations(&self, o: usize, interface: &str) -> Vec<String> {
        let mut out: Vec<String> = self.objects[o]
            .symbols
            .classes
            .iter()
            .filter(|c| {
                c.interfaces
                    .iter()
                    .any(|i| i.name.eq_ignore_ascii_case(interface))
            })
            .map(|c| c.name.clone())
            .collect();
        if let Some(ws) = self.workspace {
            for c in ws.implementations(interface) {
                if !out.iter().any(|x| x.eq_ignore_ascii_case(&c)) {
                    out.push(c);
                }
            }
        }
        out
    }

    /// Classes inheriting directly from a class: in the object `o` and,
    /// through the workspace, in the sources.
    fn subclasses(&self, o: usize, class: &str) -> Vec<String> {
        let mut out: Vec<String> = self.objects[o]
            .symbols
            .classes
            .iter()
            .filter(|c| {
                c.superclass
                    .as_deref()
                    .is_some_and(|s| s.eq_ignore_ascii_case(class))
            })
            .map(|c| c.name.clone())
            .collect();
        if let Some(ws) = self.workspace {
            for c in ws.subclasses(class) {
                if !out.iter().any(|x| x.eq_ignore_ascii_case(&c)) {
                    out.push(c);
                }
            }
        }
        out
    }

    /// Redefinitions of a method in the subclasses of a class, at any
    /// depth; none when the class has too many subclasses to tell.
    fn redefinitions(
        &mut self,
        o: usize,
        class: &str,
        name: &str,
        depth: u32,
    ) -> Vec<(usize, usize)> {
        let mut out = Vec::new();
        let mut queue = vec![class.to_string()];
        let mut seen: Vec<String> = Vec::new();
        while let Some(cls) = queue.pop() {
            for sub in self.subclasses(o, &cls) {
                if seen.iter().any(|x| x.eq_ignore_ascii_case(&sub)) {
                    continue;
                }
                if seen.len() >= MAX_SUBCLASSES {
                    return Vec::new();
                }
                seen.push(sub.clone());
                if let Some(h) = self.holder(o, &sub, depth) {
                    let redefines = type_info(&self.objects[h].symbols, &sub).is_some_and(|i| {
                        i.methods
                            .iter()
                            .any(|m| m.is_redefinition && m.name.eq_ignore_ascii_case(name))
                    });
                    if redefines {
                        if let Some(k) = self.find(h, "method", Some(&sub), name) {
                            out.push((h, k));
                        }
                    }
                }
                queue.push(sub);
            }
        }
        if out.len() > MAX_IMPLEMENTATIONS {
            return Vec::new();
        }
        out
    }

    /// The procedures a call may run that are in the sources: one, or for a
    /// method of an interface its implementations; for an instance method
    /// also its redefinitions in subclasses.
    fn callees(&mut self, o: usize, c: &m::Call) -> Vec<(usize, usize)> {
        if c.dynamic {
            return Vec::new();
        }
        let depth = self.objects[o].depth + 1;
        let one = match c.kind {
            m::CallKind::Form => {
                let target = match c.program.as_deref() {
                    Some(p) if !self.is_program(o, p) => self.load(&SourceRef::program(p), depth),
                    _ => Some(o),
                };
                target.and_then(|t| self.find(t, "form", None, &c.name).map(|k| (t, k)))
            }
            m::CallKind::Submit => {
                let Some(t) = self.load(&SourceRef::program(&c.name), depth) else {
                    return Vec::new();
                };
                return self.objects[t]
                    .procs
                    .iter()
                    .enumerate()
                    .filter(|(_, p)| matches!(p.procedure.block.kind.as_str(), "event" | "program"))
                    .map(|(k, _)| (t, k))
                    .collect();
            }
            // lock modules are generated; what they do is in the lock store
            m::CallKind::Function if crate::stores::lock_module(&c.name).is_some() => None,
            m::CallKind::Function => match self.find(o, "function_module", None, &c.name) {
                Some(k) => Some((o, k)),
                None => self
                    .load(&SourceRef::function_module(&c.name), depth)
                    .and_then(|t| {
                        self.find(t, "function_module", None, &c.name)
                            .map(|k| (t, k))
                    }),
            },
            m::CallKind::Method | m::CallKind::Constructor => {
                let Some(class) = c.class.clone() else {
                    return Vec::new();
                };
                let name = if c.kind == m::CallKind::Constructor {
                    "constructor"
                } else {
                    c.name.as_str()
                };
                // the object may be of a subclass redefining the method,
                // unless the call names the implementation: super->m( ),
                // static methods, constructors
                let dispatched = c.kind == m::CallKind::Method
                    && !c.is_static
                    && !c
                        .receiver
                        .as_deref()
                        .is_some_and(|r| r.eq_ignore_ascii_case("super"));
                if let Some(found) = self.method_in(o, &class, name, depth) {
                    let mut out = vec![found];
                    if dispatched {
                        out.extend(self.redefinitions(o, &class, name, depth));
                    }
                    return out;
                }
                // a method of an interface: any implementation may run
                let interface = self.holder(o, &class, depth).is_some_and(|h| {
                    self.objects[h]
                        .symbols
                        .interfaces
                        .iter()
                        .any(|i| i.name.eq_ignore_ascii_case(&class))
                });
                if !interface || name.contains('~') {
                    return Vec::new();
                }
                // CALL BADI runs the active implementations of the BAdI
                let classes = match (&c.badi, self.workspace) {
                    (Some(badi), Some(ws)) => ws
                        .badi(badi)
                        .map(|b| crate::workspace::badi_classes(&b))
                        .unwrap_or_default(),
                    _ => self.implementations(o, &class),
                };
                if classes.len() > MAX_IMPLEMENTATIONS {
                    return Vec::new();
                }
                let method = format!("{class}~{name}");
                let mut out = Vec::new();
                for cls in &classes {
                    if let Some(found) = self.method_in(o, cls, &method, depth) {
                        out.push(found);
                        out.extend(self.redefinitions(o, cls, &method, depth));
                    }
                }
                out.dedup();
                return out;
            }
            _ => None,
        };
        one.into_iter().collect()
    }

    /// Analyzes a procedure after the procedures it calls.
    fn summary(&mut self, o: usize, k: usize) -> Summary {
        let p = &mut self.objects[o].procs[k];
        if let Some(s) = &p.summary {
            return s.clone();
        }
        if p.running {
            return Summary::default();
        }
        p.running = true;
        let calls = std::mem::take(&mut p.calls);
        for c in &calls {
            for (o2, k2) in self.callees(o, &c.call) {
                if (o2, k2) != (o, k) {
                    self.apply(o, k, c, o2, k2);
                }
            }
        }
        let p = &mut self.objects[o].procs[k];
        let parameters: HashSet<String> = p
            .procedure
            .parameters
            .iter()
            .map(|x| x.name.clone())
            .collect();
        let bound = aliases(
            &mut p.procedure.body,
            &p.procedure.flow,
            &p.bindings,
            &parameters,
        );
        data_flow(&mut p.procedure.body, &p.procedure.flow);
        guard_subrc(&mut p.procedure.body);
        p.procedure.effects = effects(&p.procedure.body);
        let s = summarize(&p.procedure, &bound);
        p.summary = Some(s.clone());
        p.running = false;
        s
    }

    /// A program run by SUBMIT: its own data ends with it, the tables it
    /// reads and writes are what the caller sees. What written rows depend
    /// on in its selection parameters comes from the values passed with
    /// `WITH`.
    fn apply_submit(&mut self, o: usize, k: usize, pending: &PendingCall, o2: usize, k2: usize) {
        let s = self.summary(o2, k2);
        let via = m::BlockRef {
            kind: "program".to_string(),
            name: pending.call.name.clone(),
            class: None,
        };
        let caller = &mut self.objects[o].procs[k].procedure;
        let Some(st) = find_step_mut(&mut caller.body, pending.step) else {
            return;
        };
        let passed = submit_parameters(&pending.call.text);
        let own_reads: Vec<String> = st
            .reads
            .iter()
            .filter(|r| r.via.is_none())
            .map(|r| r.name.clone())
            .collect();
        // the caller's data passed to a selection parameter of the program
        let values = |parameter: &str| -> Vec<String> {
            passed
                .iter()
                .filter(|(p, _)| p.is_empty() || p.eq_ignore_ascii_case(parameter))
                .flat_map(|(_, operands)| operands.iter())
                .filter(|o| own_reads.contains(o))
                .cloned()
                .collect()
        };
        for (t, deps) in &s.tables {
            let depends = deps.as_ref().map(|d| {
                let mut reads: Vec<String> = Vec::new();
                for (name, scope) in &d.reads {
                    if *scope != m::DataScope::Global {
                        continue;
                    }
                    for v in values(name) {
                        if !reads.contains(&v) {
                            reads.push(v);
                        }
                    }
                }
                m::Dependency {
                    reads,
                    arguments: false,
                    tables: d.tables.clone(),
                }
            });
            let t = m::TableAccess {
                via: Some(via.clone()),
                depends,
                ..t.clone()
            };
            if !st.tables.contains(&t) {
                st.tables.push(t);
            }
        }
    }

    /// Adds what the procedure `k2` of object `o2` reads and writes to the
    /// step of a call to it, and what the data passed to its parameters
    /// gets.
    fn apply(&mut self, o: usize, k: usize, pending: &PendingCall, o2: usize, k2: usize) {
        if pending.call.kind == m::CallKind::Submit {
            return self.apply_submit(o, k, pending, o2, k2);
        }
        let (step, call, args) = (pending.step, &pending.call, pending.args.as_ref());
        let s = self.summary(o2, k2);
        let via = self.objects[o2].procs[k2].procedure.block.clone();
        let other = (o2 != o)
            .then(|| self.objects[o2].reference.as_ref())
            .flatten()
            .map(|r| r.name.to_ascii_lowercase());
        let Proc {
            procedure: caller,
            bindings,
            ..
        } = &mut self.objects[o].procs[k];
        let receiver = receiver(call, caller.block.class.as_deref(), o2 != o);
        let Some(st) = find_step_mut(&mut caller.body, step) else {
            return;
        };
        let at = m::Position {
            row: call.span.start.row,
            col: call.span.start.col,
        };
        let map =
            |name: &str, scope: m::DataScope| map_name(name, scope, &receiver, other.as_deref());
        let bound: Vec<(String, &Actual)> = args.map(|a| a.bind(&s.parameters)).unwrap_or_default();
        // the caller's own reads: arguments and receiver
        let own_reads: Vec<(String, At)> = st
            .reads
            .iter()
            .filter(|r| r.via.is_none())
            .map(|r| (r.name.clone(), position_of(r)))
            .collect();
        let reads_for = |formals: &[String]| -> Vec<String> {
            let mut out: Vec<String> = Vec::new();
            for (name, p) in &own_reads {
                let passed = bound
                    .iter()
                    .any(|(f, a)| formals.contains(f) && a.contains(*p));
                let receiving = args.is_some_and(|a| a.in_receiver(*p));
                let deciding = pending.deciding.contains(name);
                if (passed || receiving || deciding) && !out.contains(name) {
                    out.push(name.clone());
                }
            }
            out
        };
        let dependency = |d: &Deps| {
            let mut reads: Vec<String> = d.reads.iter().filter_map(|(n, sc)| map(n, *sc)).collect();
            for n in reads_for(&d.formals) {
                if !reads.contains(&n) {
                    reads.push(n);
                }
            }
            m::Dependency {
                reads,
                arguments: args.is_none() && !d.formals.is_empty(),
                tables: d.tables.clone(),
            }
        };
        let access = |name: String, scope, access, depends| m::DataAccess {
            path: name.clone(),
            name,
            access,
            scope,
            whole: false,
            declared: false,
            implicit: false,
            data_type: None,
            sources: Vec::new(),
            from_entry: false,
            via: Some(via.clone()),
            through: None,
            depends,
            position: at,
        };
        for (name, scope) in &s.reads {
            if let Some(name) = map(name, *scope) {
                if !st
                    .reads
                    .iter()
                    .any(|a| a.name == name && a.via.as_ref() == Some(&via))
                {
                    st.reads
                        .push(access(name, *scope, m::AccessKind::Read, None));
                }
            }
        }
        for (name, scope, deps) in &s.writes {
            if let Some(name) = map(name, *scope) {
                if !st
                    .writes
                    .iter()
                    .any(|a| a.name == name && a.via.as_ref() == Some(&via))
                {
                    let d = Some(dependency(deps));
                    st.writes
                        .push(access(name, *scope, m::AccessKind::Write, d));
                }
            }
        }
        for (t, deps) in &s.tables {
            let t = m::TableAccess {
                via: Some(via.clone()),
                depends: deps.as_ref().map(dependency),
                ..t.clone()
            };
            if !st.tables.contains(&t) {
                st.tables.push(t);
            }
        }

        // the data passed to the parameters it writes
        let Some(args) = args else { return };
        let returning = s
            .parameters
            .iter()
            .find(|p| p.direction == m::Direction::Returning)
            .map(|p| p.name.as_str());
        let mut added = Vec::new();
        for pw in &s.params {
            let mut dep = dependency(&pw.deps);
            let actuals: Vec<&Actual> = bound
                .iter()
                .filter(|(f, _)| *f == pw.formal)
                .map(|(_, a)| *a)
                .collect();
            // `x = m( )`: the result goes to what the statement writes
            // outside of the call, and the other operands count too
            let receiving = returning == Some(pw.formal.as_str()) && actuals.is_empty();
            if receiving {
                for (name, p) in &own_reads {
                    if !args.in_range(*p) && !dep.reads.contains(name) {
                        dep.reads.push(name.clone());
                    }
                }
            }
            let mut written = false;
            for w in st.writes.iter_mut().filter(|w| w.via.is_none()) {
                let p = position_of(w);
                let target =
                    actuals.iter().any(|a| a.contains(p)) || (receiving && !args.in_range(p));
                if !target {
                    continue;
                }
                written = true;
                w.depends = Some(match w.depends.take() {
                    Some(old) => merge(old, &dep),
                    None => dep.clone(),
                });
                for (t, sc) in &pw.refs {
                    if let Some(t) = map(t, *sc) {
                        bindings.push((step, w.name.clone(), t, *sc));
                    }
                }
            }
            if written {
                continue;
            }
            // passed by reference without being written by the statement:
            // a reference written through, a FORM parameter
            let direction = s
                .parameters
                .iter()
                .find(|p| p.name == pw.formal)
                .map(|p| p.direction);
            let by_reference = matches!(
                direction,
                Some(m::Direction::Using | m::Direction::Changing | m::Direction::Tables)
            );
            if pw.through.is_empty() && !by_reference {
                continue;
            }
            for a in &actuals {
                let first = st
                    .reads
                    .iter()
                    .filter(|r| r.via.is_none() && a.contains(position_of(r)))
                    .min_by(by_position);
                let Some(r) = first else { continue };
                // what is written: the data passed, or behind the reference
                let mut targets: Vec<(String, String, m::DataScope)> = Vec::new();
                if pw.through.is_empty() {
                    targets.push((r.name.clone(), r.path.clone(), r.scope));
                }
                for suffix in &pw.through {
                    let path = format!("{}{suffix}", r.name);
                    match suffix.strip_prefix("->").filter(|x| !x.starts_with('*')) {
                        // an attribute of the object passed
                        Some(attr) => {
                            let attr = attr.split('-').next().unwrap_or(attr);
                            targets.push((
                                format!("{}->{attr}", r.name),
                                path,
                                m::DataScope::Attribute,
                            ));
                        }
                        None => targets.push((r.name.clone(), path, r.scope)),
                    }
                }
                for (name, path, scope) in targets {
                    added.push(m::DataAccess {
                        name,
                        path,
                        access: m::AccessKind::Change,
                        scope,
                        whole: false,
                        declared: false,
                        implicit: false,
                        data_type: None,
                        sources: Vec::new(),
                        from_entry: false,
                        via: Some(via.clone()),
                        through: None,
                        depends: Some(dep.clone()),
                        position: r.position,
                    });
                }
            }
        }
        st.writes.extend(added);
    }
}

fn position_of(a: &m::DataAccess) -> At {
    (a.position.row, a.position.col)
}

fn merge(mut a: m::Dependency, b: &m::Dependency) -> m::Dependency {
    for r in &b.reads {
        if !a.reads.contains(r) {
            a.reads.push(r.clone());
        }
    }
    for t in &b.tables {
        if !a.tables.contains(t) {
            a.tables.push(t.clone());
        }
    }
    a.arguments |= b.arguments;
    a
}

fn receiver(c: &m::Call, caller_class: Option<&str>, other_object: bool) -> Receiver {
    if !matches!(c.kind, m::CallKind::Method | m::CallKind::Constructor) {
        // forms and function modules of another program have no access to
        // the attributes of the caller
        return if other_object {
            Receiver::Unknown
        } else {
            Receiver::Own
        };
    }
    let r = c.receiver.as_deref().map(|r| r.trim().to_ascii_lowercase());
    match r.as_deref() {
        None if c.kind == m::CallKind::Constructor => Receiver::Unknown,
        None | Some("me") | Some("super") => Receiver::Own,
        Some(r) if c.is_static => {
            if caller_class.is_some_and(|k| k.eq_ignore_ascii_case(r)) {
                Receiver::Own
            } else {
                Receiver::Class(r.to_string())
            }
        }
        Some(r) => {
            let r = r.strip_prefix("me->").unwrap_or(r);
            let simple = !r.is_empty()
                && r.chars()
                    .all(|ch| ch.is_ascii_alphanumeric() || "_-/<>=~".contains(ch));
            if simple {
                Receiver::Object(r.to_string())
            } else {
                Receiver::Unknown
            }
        }
    }
}

/// The name in the caller of data a called procedure accesses.
fn map_name(
    name: &str,
    scope: m::DataScope,
    receiver: &Receiver,
    other_object: Option<&str>,
) -> Option<String> {
    match scope {
        m::DataScope::Global => Some(match other_object {
            Some(object) if !name.starts_with('(') => format!("({object}){name}"),
            _ => name.to_string(),
        }),
        m::DataScope::Other => Some(name.to_string()),
        m::DataScope::Attribute if name.contains("=>") => Some(name.to_string()),
        m::DataScope::Attribute => match receiver {
            Receiver::Own => Some(name.to_string()),
            Receiver::Class(c) => Some(format!("{c}=>{name}")),
            Receiver::Object(r) => Some(format!("{r}->{name}")),
            Receiver::Unknown => None,
        },
        _ => None,
    }
}

/// What callers see of a procedure: data outside of it that it reads from
/// before the call or writes, with what the written values depend on, the
/// parameters it writes, and the database tables it accesses.
/// The selection parameters a SUBMIT fills and the operands of the values,
/// lower case: `WITH p_id = lv_id` gives (`p_id`, [`lv_id`]); `WITH
/// SELECTION-TABLE lt_sel` fills any parameter, the name is empty then.
fn submit_parameters(text: &str) -> Vec<(String, Vec<String>)> {
    const ENDS: &[&str] = &[
        "WITH",
        "AND",
        "VIA",
        "USING",
        "TO",
        "EXPORTING",
        "LINE-SIZE",
        "LINE-COUNT",
        "KEEP",
        "USER",
        "SPOOL",
        "IMMEDIATELY",
        "WITHOUT",
        "LIST",
        "NEW",
        "DATASET",
    ];
    let words: Vec<String> = text
        .replace(['.', ',', '(', ')'], " ")
        .split_whitespace()
        .map(|w| w.trim_start_matches('@').to_ascii_lowercase())
        .collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < words.len() {
        if !words[i].eq_ignore_ascii_case("with") {
            i += 1;
            continue;
        }
        let Some(first) = words.get(i + 1) else { break };
        let (name, mut j) = match first.as_str() {
            "selection-table" | "free" => (String::new(), i + 2),
            _ => (first.clone(), i + 2),
        };
        let mut operands = Vec::new();
        while j < words.len() {
            let w = &words[j];
            // BETWEEN a AND b: the AND belongs to the clause
            let between = w == "and" && words[..j].iter().rev().take(3).any(|x| x == "between");
            if ENDS.iter().any(|e| e.eq_ignore_ascii_case(w)) && !between {
                break;
            }
            operands.push(w.clone());
            j += 1;
        }
        out.push((name, operands));
        i = j;
    }
    out
}

fn summarize(p: &m::Procedure, bound: &Aliases) -> Summary {
    let own: HashSet<&str> = p
        .locals
        .iter()
        .map(String::as_str)
        .chain(p.parameters.iter().map(|x| x.name.as_str()))
        .collect();
    let visible = |name: &str, scope: m::DataScope| match scope {
        m::DataScope::Global => true,
        m::DataScope::Other => name.contains("=>"),
        // `lo->x` or `ls-ref->x` with a local or parameter `lo`, `ls` means
        // nothing to callers
        m::DataScope::Attribute => {
            let root = name.split(['-', '=', '~']).next().unwrap_or_default();
            name.contains("=>") || !own.contains(root)
        }
        _ => false,
    };
    let own_parameters: HashSet<&str> = p.parameters.iter().map(|x| x.name.as_str()).collect();
    let mut s = Summary {
        parameters: p.parameters.clone(),
        ..Summary::default()
    };
    let mut scopes: HashMap<&str, m::DataScope> = HashMap::new();
    // paths written behind reference parameters: `->*`, `->mv_x`
    let mut through: HashMap<String, Vec<String>> = HashMap::new();
    walk(&p.body, &mut |st| {
        for r in &st.reads {
            scopes.entry(r.name.as_str()).or_insert(r.scope);
            if r.from_entry
                && visible(&r.name, r.scope)
                && !s.reads.iter().any(|(n, _)| *n == r.name)
            {
                s.reads.push((r.name.clone(), r.scope));
            }
        }
        for w in &st.writes {
            let Some(root) = w.through.as_deref().filter(|r| own_parameters.contains(r)) else {
                continue;
            };
            let suffix = if w.name == root {
                let path = w.path.to_ascii_lowercase();
                path.strip_prefix(root).unwrap_or("->*").to_string()
            } else {
                match w.name.strip_prefix(root) {
                    Some(rest) => rest.to_string(),
                    None => continue,
                }
            };
            let paths = through.entry(root.to_string()).or_default();
            if !paths.contains(&suffix) {
                paths.push(suffix);
            }
        }
        for t in &st.tables {
            if t.access == m::AccessKind::Read {
                let t = m::TableAccess {
                    via: None,
                    depends: None,
                    ..t.clone()
                };
                if !s.tables.iter().any(|(x, _)| *x == t) {
                    s.tables.push((t, None));
                }
            }
        }
    });
    // the effects tell what the written values depend on
    for e in &p.effects {
        let mut deps = Deps {
            tables: e.tables.clone(),
            ..Deps::default()
        };
        for i in &e.inputs {
            match scopes.get(i.as_str()).copied() {
                Some(m::DataScope::Parameter) => deps.formals.push(i.clone()),
                Some(scope) if visible(i, scope) => deps.reads.push((i.clone(), scope)),
                _ => {}
            }
        }
        match e.scope {
            m::DataScope::Database | m::DataScope::External => {
                let mut operations: Vec<(m::DbOperation, Option<m::Store>)> = Vec::new();
                walk(&p.body, &mut |st| {
                    for t in &st.tables {
                        if t.name == e.name
                            && t.access == m::AccessKind::Write
                            && !operations.contains(&(t.operation, t.store))
                        {
                            operations.push((t.operation, t.store));
                        }
                    }
                });
                for (operation, store) in operations {
                    let t = m::TableAccess {
                        name: e.name.clone(),
                        access: m::AccessKind::Write,
                        operation,
                        store,
                        via: None,
                        depends: None,
                    };
                    s.tables.push((t, Some(deps.clone())));
                }
            }
            _ if e
                .name
                .split_once("->")
                .is_some_and(|(r, _)| own_parameters.contains(r)) =>
            {
                let formal = e.name.split("->").next().unwrap_or_default();
                add_param(&mut s.params, formal, deps, &through, Vec::new());
            }
            m::DataScope::Parameter => {
                let refs = bound
                    .get(&e.name)
                    .map(|(_, targets)| {
                        targets
                            .iter()
                            .filter(|(t, sc)| visible(t, *sc))
                            .cloned()
                            .collect()
                    })
                    .unwrap_or_default();
                add_param(&mut s.params, &e.name, deps, &through, refs);
            }
            scope if visible(&e.name, scope) => s.writes.push((e.name.clone(), scope, deps)),
            _ => {}
        }
    }
    s
}

/// Adds a written parameter, merged with what is known of it already.
fn add_param(
    params: &mut Vec<ParamWrite>,
    formal: &str,
    deps: Deps,
    through: &HashMap<String, Vec<String>>,
    refs: Vec<(String, m::DataScope)>,
) {
    let paths = through.get(formal).cloned().unwrap_or_default();
    match params.iter_mut().find(|p| p.formal == formal) {
        Some(p) => {
            for r in deps.reads {
                if !p.deps.reads.contains(&r) {
                    p.deps.reads.push(r);
                }
            }
            for f in deps.formals {
                if !p.deps.formals.contains(&f) {
                    p.deps.formals.push(f);
                }
            }
            for t in deps.tables {
                if !p.deps.tables.contains(&t) {
                    p.deps.tables.push(t);
                }
            }
            for r in refs {
                if !p.refs.contains(&r) {
                    p.refs.push(r);
                }
            }
        }
        None => params.push(ParamWrite {
            formal: formal.to_string(),
            deps,
            through: paths,
            refs,
        }),
    }
}

fn find_step_mut(steps: &mut [m::Step], id: u32) -> Option<&mut m::Step> {
    for s in steps {
        if s.id == id {
            return Some(s);
        }
        if let Some(found) = find_step_mut(&mut s.children, id) {
            return Some(found);
        }
    }
    None
}

// ----- building --------------------------------------------------------------

/// Data declared at program level, lower case.
fn globals_of(symbols: &m::FileSymbols) -> HashSet<String> {
    use m::DeclarationKind as D;
    symbols
        .declarations
        .iter()
        .filter(|d| !matches!(d.kind, D::Type | D::Include | D::EnumValue))
        .map(|d| d.name.to_ascii_lowercase())
        .collect()
}

/// Attributes of a class and its superclasses, lower case; superclasses in
/// other objects are looked up through the workspace.
fn class_attributes(
    symbols: &m::FileSymbols,
    workspace: Option<&Workspace>,
    class: &str,
) -> HashSet<String> {
    let mut out = HashSet::new();
    let mut class = class.to_string();
    for _ in 0..10 {
        let object;
        let holder = if type_info(symbols, &class).is_some() {
            symbols
        } else {
            match workspace.and_then(|w| w.type_object(&class)) {
                Some(o) => {
                    object = o;
                    &object.symbols
                }
                None => break,
            }
        };
        let Some(info) = type_info(holder, &class) else {
            break;
        };
        out.extend(info.attributes.iter().map(|a| a.name.to_ascii_lowercase()));
        match info.superclass {
            Some(s) => class = s.to_string(),
            None => break,
        }
    }
    out
}

fn build_object(ws: &Workspace, object: &Object) -> (Arc<m::FileSymbols>, Vec<Proc>) {
    let symbols = Arc::new(object.symbols.clone());
    let globals = globals_of(&symbols);
    let calls = ws.object_calls(object);
    let attributes = |class: &str| class_attributes(&symbols, Some(ws), class);
    let mut procs = Vec::new();
    for u in &object.units {
        let unit = u.reference.to_string();
        let unit_calls = calls
            .iter()
            .filter(|c| c.unit.as_deref() == Some(unit.as_str()))
            .cloned()
            .collect();
        procs.extend(build_unit(
            &u.file,
            Some(&unit),
            &symbols,
            &globals,
            &attributes,
            unit_calls,
        ));
    }
    (symbols, procs)
}

/// The procedures of one source with their steps, conditions and control
/// flow, the database tables of the steps and the calls to follow.
fn build_unit(
    file: &File,
    unit: Option<&str>,
    symbols: &m::FileSymbols,
    globals: &HashSet<String>,
    attributes: &dyn Fn(&str) -> HashSet<String>,
    calls: Vec<m::Call>,
) -> Vec<Proc> {
    // consecutive statements of the same processing block
    let mut groups: Vec<(m::BlockRef, Vec<usize>)> = Vec::new();
    for (i, b) in statement_blocks(file).into_iter().enumerate() {
        let Some(b) = b else { continue };
        match groups.last_mut() {
            Some((r, v)) if *r == b => v.push(i),
            _ => groups.push((b, vec![i])),
        }
    }
    let mut procs: Vec<Proc> = groups
        .into_iter()
        .filter_map(|(block, statements)| {
            procedure(file, symbols, globals, attributes, block, &statements)
        })
        .map(|mut procedure| {
            procedure.unit = unit.map(str::to_string);
            Proc {
                procedure,
                calls: Vec::new(),
                bindings: Vec::new(),
                summary: None,
                running: false,
            }
        })
        .collect();
    for a in db_access_with(file, symbols) {
        let Some((k, id)) = locate(&procs, &a.span.start) else {
            continue;
        };
        if let Some(s) = find_step_mut(&mut procs[k].procedure.body, id) {
            for t in table_accesses(&a) {
                if !s.tables.contains(&t) {
                    s.tables.push(t);
                }
            }
        }
    }
    // data kept outside the program: memory, parameters, files, locks;
    // infotype tables of HR
    let constants = literal_constants(symbols);
    let infotypes = crate::hr::infotypes(symbols);
    for st in &file.statements {
        let mut found = crate::stores::store_accesses(file, st, &constants);
        found.extend(crate::hr::infotype_reads(file, st, &constants, &infotypes));
        if found.is_empty() {
            continue;
        }
        let start = file.lexed.tokens[st.tokens[0]].start;
        let at = m::Position {
            row: start.row,
            col: start.col,
        };
        let Some((k, id)) = locate(&procs, &at) else {
            continue;
        };
        if let Some(s) = find_step_mut(&mut procs[k].procedure.body, id) {
            for t in found {
                if !s.tables.contains(&t) {
                    s.tables.push(t);
                }
            }
        }
    }
    // GET node: the logical database fills the node and, at GET PERNR or
    // GET OBJEC, the tables of the infotypes
    for st in &file.statements {
        let Some(node) = crate::hr::get_node(file, st) else {
            continue;
        };
        let at = span_of(file, st).start;
        let Some((k, id)) = locate(&procs, &at) else {
            continue;
        };
        let Some(s) = find_step_mut(&mut procs[k].procedure.body, id) else {
            continue;
        };
        // each with the table it comes from
        let mut filled = vec![(node.clone(), Vec::new())];
        filled.extend(
            crate::hr::filled(&node, &infotypes)
                .into_iter()
                .map(|(t, db)| (t.table.clone(), vec![db])),
        );
        for (name, tables) in filled {
            let scope = if globals.contains(&name) {
                m::DataScope::Global
            } else {
                m::DataScope::Other
            };
            s.writes.push(m::DataAccess {
                scope,
                depends: Some(m::Dependency {
                    reads: Vec::new(),
                    arguments: false,
                    tables,
                }),
                ..implicit(&name, at, true)
            });
        }
    }
    // calls with their arguments; dynamic ones with the names they can have
    let mut seen: HashMap<(u32, String), usize> = HashMap::new();
    for c in calls {
        let Some((k, id)) = locate(&procs, &c.span.start) else {
            continue;
        };
        let (resolved, deciding) = if c.dynamic {
            resolve_dynamic(&procs[k].procedure, id, &c, &constants)
        } else {
            (vec![c], Vec::new())
        };
        if let Some(whole) = resolved
            .iter()
            .filter_map(|c| call_sets_subrc(c.kind))
            .max()
        {
            if let Some(s) = find_step_mut(&mut procs[k].procedure.body, id) {
                match s
                    .writes
                    .iter_mut()
                    .find(|w| w.implicit && w.name == "sy-subrc")
                {
                    Some(w) => w.whole |= whole,
                    None => s.writes.push(implicit("sy-subrc", s.span.start, whole)),
                }
            }
        }
        for c in resolved {
            let n = seen
                .entry((id, format!("{:?} {}", c.kind, c.name.to_ascii_lowercase())))
                .or_insert(0);
            let args = file
                .statements
                .get(id as usize)
                .and_then(|s| call_args(file, s, &c, *n));
            *n += 1;
            procs[k].calls.push(PendingCall {
                step: id,
                call: c,
                args,
                deciding: deciding.clone(),
            });
        }
    }
    procs
}

/// The procedure and step a position is in: the last step starting before.
fn locate(procs: &[Proc], pos: &m::Position) -> Option<(usize, u32)> {
    let key = |p: &m::Position| (p.row, p.col);
    let k = procs.iter().position(|p| {
        let span = &p.procedure.span;
        key(&span.start) <= key(pos) && key(pos) <= key(&span.end)
    })?;
    let mut best: Option<((u32, u32), u32)> = None;
    walk(&procs[k].procedure.body, &mut |s| {
        let start = key(&s.span.start);
        if start <= key(pos) && best.is_none_or(|(b, _)| start >= b) {
            best = Some((start, s.id));
        }
    });
    best.map(|(_, id)| (k, id))
}

/// The tables of a database access: the changed table is written, the
/// others are read.
fn table_accesses(a: &m::DbAccess) -> Vec<m::TableAccess> {
    use m::DbOperation as Op;
    let changes = matches!(
        a.operation,
        Op::Insert | Op::Update | Op::Modify | Op::Delete | Op::Merge
    );
    let mut out: Vec<m::TableAccess> = Vec::new();
    for t in &a.tables {
        if matches!(t.role, m::DbTableRole::InternalTable | m::DbTableRole::Cte) {
            continue;
        }
        let name = match &t.resolved {
            Some(r) => r.to_ascii_uppercase(),
            None if t.dynamic => format!("({})", t.name),
            None => t.name.to_ascii_uppercase(),
        };
        let access = if changes && t.role == m::DbTableRole::Main {
            m::AccessKind::Write
        } else {
            m::AccessKind::Read
        };
        let t = m::TableAccess {
            name,
            access,
            operation: a.operation,
            store: None,
            via: None,
            depends: None,
        };
        if !out.contains(&t) {
            out.push(t);
        }
    }
    out
}

/// A dynamic call is followed into at most this many names.
const MAX_DYNAMIC: usize = 10;

/// The calls a dynamic call can be: the names its operand has on the way to
/// the call, from literals assigned to it and from constants; with the
/// variables deciding which one runs.
fn resolve_dynamic(
    p: &m::Procedure,
    step: u32,
    c: &m::Call,
    constants: &HashMap<String, String>,
) -> (Vec<m::Call>, Vec<String>) {
    let deciding = std::cell::RefCell::new(Vec::<String>::new());
    // `x` of `(x)` at the end of a name, or the name itself
    let inner = |text: &str| -> String {
        let t = text.trim();
        match (t.rfind('('), t.ends_with(')')) {
            (Some(open), true) => t[open + 1..t.len() - 1].trim().to_string(),
            _ => t.to_string(),
        }
    };
    let values = |operand: &str| -> Vec<String> {
        if let Some(v) = literal_value(operand) {
            return vec![v];
        }
        let var = operand.to_ascii_lowercase();
        if !constants.contains_key(&var) {
            deciding.borrow_mut().push(var.clone());
        }
        let mut defs: HashMap<u32, Vec<String>> = HashMap::new();
        let mut steps: HashMap<u32, &m::Step> = HashMap::new();
        walk(&p.body, &mut |s| {
            steps.insert(s.id, s);
            if s.writes.iter().any(|w| w.name == var) {
                defs.insert(s.id, vec![var.clone()]);
            }
        });
        let uses = HashMap::from([(step, vec![var.clone()])]);
        let reach = reaching(&p.body, &p.flow, &defs, &uses);
        let mut out: Vec<String> = Vec::new();
        let Some((sources, entry)) = reach.get(&(step, var.clone())) else {
            return out;
        };
        let assigned = sources.iter().filter_map(|s| {
            steps
                .get(s)
                .and_then(|s| assigned_value(s, &var, constants))
        });
        let constant = entry.then(|| constants.get(&var).cloned()).flatten();
        for v in assigned.chain(constant) {
            if !out.contains(&v) {
                out.push(v);
            }
        }
        out
    };
    let mut out: Vec<m::Call> = Vec::new();
    let mut add = |name: String, class: Option<String>| {
        if out.len() < MAX_DYNAMIC {
            let mut x = c.clone();
            x.dynamic = false;
            x.name = name;
            x.class = class;
            out.push(x);
        }
    };
    match c.kind {
        m::CallKind::Function => {
            for v in values(&c.name) {
                add(v.to_ascii_uppercase(), None);
            }
        }
        m::CallKind::Form if c.program.as_deref().is_none_or(|p| !p.starts_with('(')) => {
            for v in values(&inner(&c.name)) {
                add(v.to_ascii_lowercase(), None);
            }
        }
        m::CallKind::Constructor => {
            if let Some(class) = c.class.as_deref().filter(|k| k.starts_with('(')) {
                for v in values(&inner(class)) {
                    add("constructor".to_string(), Some(v.to_ascii_lowercase()));
                }
            }
        }
        m::CallKind::Method => {
            let classes = match c.class.as_deref() {
                Some(k) if k.starts_with('(') => values(&inner(k)),
                Some(k) => vec![k.to_string()],
                None if c.receiver.is_none() => p.block.class.clone().into_iter().collect(),
                None => Vec::new(),
            };
            let method = c.name.rsplit('>').next().unwrap_or(&c.name);
            let methods = if method.trim().starts_with('(') {
                values(&inner(method))
            } else {
                vec![method.to_string()]
            };
            for k in &classes {
                for name in &methods {
                    add(name.to_ascii_lowercase(), Some(k.to_ascii_lowercase()));
                }
            }
        }
        _ => {}
    }
    (out, deciding.into_inner())
}

/// The literal or constant a step assigns to `var`: `var = 'X'`,
/// `DATA var TYPE t VALUE 'X'`.
fn assigned_value(s: &m::Step, var: &str, constants: &HashMap<String, String>) -> Option<String> {
    let text = s.text.trim().trim_end_matches(['.', ',']).trim();
    let upper = text.to_ascii_uppercase();
    let rhs = match kind_of(s) {
        K::Move => {
            let (lhs, rhs) = text.split_once(" = ")?;
            let lhs = lhs.trim().to_ascii_lowercase();
            if lhs != var && !lhs.ends_with(&format!("({var})")) {
                return None;
            }
            rhs
        }
        K::Data | K::Static | K::Constant => &text[upper.find(" VALUE ")? + 7..],
        _ => return None,
    };
    let rhs = rhs.trim();
    literal_value(rhs).or_else(|| constants.get(&rhs.to_ascii_lowercase()).cloned())
}

// ----- statement kinds -------------------------------------------------------

/// Statements that are no steps: the frame of the procedure, comments, macros.
fn is_frame(kind: K) -> bool {
    matches!(
        kind,
        K::Comment
            | K::Empty
            | K::MacroCall
            | K::MacroContent
            | K::Define
            | K::EndOfDefinition
            | K::MethodImplementation
            | K::EndMethod
            | K::Form
            | K::EndForm
            | K::FunctionModule
            | K::EndFunction
            | K::Module
            | K::EndModule
            | K::StartOfSelection
            | K::EndOfSelection
            | K::Initialization
            | K::TopOfPage
            | K::EndOfPage
            | K::LoadOfProgram
            | K::AtSelectionScreen
            | K::AtLineSelection
            | K::AtUserCommand
            | K::AtPF
    )
}

/// Declarations and program frame statements, which do nothing at runtime.
fn is_declarative(kind: K) -> bool {
    matches!(
        kind,
        K::Data
            | K::DataBegin
            | K::DataEnd
            | K::ClassData
            | K::ClassDataBegin
            | K::ClassDataEnd
            | K::Static
            | K::StaticBegin
            | K::StaticEnd
            | K::Type
            | K::TypeBegin
            | K::TypeEnd
            | K::TypeEnumBegin
            | K::TypeEnumEnd
            | K::TypeEnum
            | K::TypeMesh
            | K::TypeMeshBegin
            | K::TypeMeshEnd
            | K::Constant
            | K::ConstantBegin
            | K::ConstantEnd
            | K::FieldSymbol
            | K::Ranges
            | K::Tables
            | K::Nodes
            | K::Infotypes
            | K::Parameter
            | K::SelectOption
            | K::SelectionScreen
            | K::FieldGroup
            | K::Controls
            | K::Local
            | K::Contexts
            | K::Report
            | K::Program
            | K::FunctionPool
            | K::ClassPool
            | K::InterfacePool
            | K::TypePool
            | K::TypePools
            | K::Include
            | K::IncludeType
            | K::ClassDefinition
            | K::ClassImplementation
            | K::ClassDeferred
            | K::ClassDefinitionLoad
            | K::ClassLocalFriends
            | K::EndClass
            | K::Interface
            | K::InterfaceDeferred
            | K::InterfaceLoad
            | K::EndInterface
            | K::InterfaceDef
            | K::Public
            | K::Protected
            | K::Private
            | K::Package
            | K::MethodDef
            | K::Events
            | K::Aliases
            | K::FormDefinition
    ) || is_frame(kind)
}

/// Statements opening a block of steps.
fn opens(kind: K) -> bool {
    structure::closer(kind).is_some()
        && !is_declarative(kind)
        && !matches!(kind, K::ExecSQL | K::Chain)
}

fn closers() -> &'static HashSet<K> {
    static SET: OnceLock<HashSet<K>> = OnceLock::new();
    SET.get_or_init(|| {
        K::ALL
            .iter()
            .filter_map(|k| structure::closer(*k))
            .collect()
    })
}

/// Whether the end statement `end` closes a block opened by `open`.
fn closes(end: K, open: K) -> bool {
    match end {
        K::EndIf => matches!(open, K::If | K::ElseIf | K::Else),
        K::EndTry => matches!(open, K::Try | K::Catch | K::Cleanup),
        _ => structure::closer(open) == Some(end),
    }
}

fn is_loop(kind: K) -> bool {
    matches!(
        kind,
        K::Loop
            | K::Do
            | K::While
            | K::SelectLoop
            | K::WithLoop
            | K::LoopAtScreen
            | K::LoopExtract
            | K::DynproLoop
            | K::Provide
    )
}

/// Steps after which the statements of the same block do not run.
fn is_jump(kind: K) -> bool {
    matches!(
        kind,
        K::Return | K::Exit | K::Continue | K::Raise | K::Leave | K::Stop | K::Reject
    )
}

fn kind_of(step: &m::Step) -> K {
    step.kind.parse().unwrap_or(K::Unknown)
}

// ----- procedures ------------------------------------------------------------

/// What the steps are built from.
struct Ctx<'a> {
    file: &'a File,
    locals: HashSet<String>,
    parameters: HashSet<String>,
    attributes: HashSet<String>,
    globals: &'a HashSet<String>,
    /// data declared in event blocks, dialog modules and at program level,
    /// which is global data of the program
    more_globals: HashSet<String>,
}

/// A procedure with its steps, guards and control flow; the data flow and
/// the effects follow once its calls are followed.
fn procedure(
    file: &File,
    symbols: &m::FileSymbols,
    globals: &HashSet<String>,
    attributes: &dyn Fn(&str) -> HashSet<String>,
    block: m::BlockRef,
    statements: &[usize],
) -> Option<m::Procedure> {
    let kinds = |i: &usize| file.statements[*i].kind;
    if block.kind == "program" && statements.iter().all(|i| is_declarative(kinds(i))) {
        return None;
    }
    let first = *statements.first()?;
    let last = *statements.last()?;
    let span = m::Span {
        start: span_of(file, &file.statements[first]).start,
        end: span_of(file, &file.statements[last]).end,
    };

    // methods, forms and function modules have local data; what event
    // blocks and dialog modules declare is global
    let routine = matches!(block.kind.as_str(), "method" | "form" | "function_module");
    let parameters = parameters_of(symbols, &block);
    let mut ctx = Ctx {
        file,
        locals: HashSet::new(),
        parameters: parameters.iter().map(|p| p.name.clone()).collect(),
        attributes: block.class.as_deref().map(attributes).unwrap_or_default(),
        globals,
        more_globals: HashSet::new(),
    };
    for i in statements {
        let s = &file.statements[*i];
        if matches!(
            s.kind,
            K::Data | K::Static | K::Constant | K::FieldSymbol | K::Ranges | K::DataBegin
        ) {
            if let Some(v) = decl::variable(file, s) {
                let name = v.name.to_ascii_lowercase();
                if routine {
                    ctx.locals.insert(name);
                } else {
                    ctx.more_globals.insert(name);
                }
            }
        }
    }

    let mut body = tree(&ctx, statements);
    // inline declarations too
    let mut declared = BTreeSet::new();
    walk(&body, &mut |s| {
        for w in &s.writes {
            if w.declared {
                declared.insert(w.name.clone());
            }
        }
    });
    if routine {
        ctx.locals.extend(declared);
    } else {
        ctx.more_globals.extend(declared);
    }
    walk_mut(&mut body, &mut |s| {
        for a in s.reads.iter_mut().chain(s.writes.iter_mut()) {
            a.scope = ctx.scope(&a.name, a.scope);
        }
    });
    case_conditions(&mut body);
    guards(&mut body, &[]);
    let flow = flow(&body);

    let mut locals: Vec<String> = ctx.locals.into_iter().collect();
    locals.sort();
    Some(m::Procedure {
        block,
        unit: None,
        span,
        parameters,
        locals,
        body,
        flow,
        effects: Vec::new(),
    })
}

impl Ctx<'_> {
    fn scope(&self, name: &str, found: m::DataScope) -> m::DataScope {
        if found == m::DataScope::System {
            m::DataScope::System
        } else if self.locals.contains(name) {
            m::DataScope::Local
        } else if self.parameters.contains(name) {
            m::DataScope::Parameter
        } else if found == m::DataScope::Attribute || self.attributes.contains(name) {
            m::DataScope::Attribute
        } else if self.globals.contains(name) || self.more_globals.contains(name) {
            m::DataScope::Global
        } else {
            m::DataScope::Other
        }
    }
}

fn parameters_of(symbols: &m::FileSymbols, block: &m::BlockRef) -> Vec<m::ProcedureParameter> {
    let eq = |a: &str, b: &str| a.eq_ignore_ascii_case(b);
    let params: Option<&Vec<m::Parameter>> = match block.kind.as_str() {
        "method" => {
            let class = block.class.as_deref().unwrap_or_default();
            match block.name.split_once('~') {
                Some((intf, method)) => symbols
                    .interfaces
                    .iter()
                    .find(|i| eq(&i.name, intf))
                    .and_then(|i| i.methods.iter().find(|m| eq(&m.name, method)))
                    .map(|m| &m.parameters),
                None => symbols
                    .classes
                    .iter()
                    .find(|c| eq(&c.name, class))
                    .and_then(|c| c.methods.iter().find(|m| eq(&m.name, &block.name)))
                    .map(|m| &m.parameters),
            }
        }
        "form" => symbols
            .forms
            .iter()
            .find(|f| eq(&f.name, &block.name))
            .map(|f| &f.parameters),
        "function_module" => symbols
            .function_modules
            .iter()
            .find(|f| eq(&f.name, &block.name))
            .map(|f| &f.parameters),
        _ => None,
    };
    params
        .into_iter()
        .flatten()
        .map(|p| m::ProcedureParameter {
            name: p.name.trim_start_matches('!').to_ascii_lowercase(),
            direction: p.direction,
        })
        .collect()
}

fn walk<'a>(steps: &'a [m::Step], f: &mut impl FnMut(&'a m::Step)) {
    for s in steps {
        f(s);
        walk(&s.children, f);
    }
}

fn walk_mut(steps: &mut [m::Step], f: &mut impl FnMut(&mut m::Step)) {
    for s in steps {
        f(s);
        walk_mut(&mut s.children, f);
    }
}

// ----- steps -----------------------------------------------------------------

/// The statement tokens, for the texts of expressions.
fn tokens<'a>(file: &'a File, s: &Statement) -> Vec<&'a Token> {
    s.tokens.iter().map(|t| &file.tokens()[*t]).collect()
}

fn text(toks: &[&Token], first: usize, last: usize) -> String {
    let last = last.min(toks.len().saturating_sub(1));
    if first > last {
        return String::new();
    }
    join_tokens(toks[first..=last].iter().copied())
}

fn step(ctx: &Ctx, i: usize) -> m::Step {
    let file = ctx.file;
    let s = &file.statements[i];
    let toks = tokens(file, s);
    let ops = operands(file, s);

    let mut reads = Vec::new();
    let mut writes = Vec::new();
    for r in ops.refs() {
        let a = access(&toks, &r);
        match r.access {
            Access::Read => reads.push(a),
            Access::Write => writes.push(a),
            Access::Change => {
                reads.push(a.clone());
                writes.push(a);
            }
        }
    }
    if let Some((name, used)) = abap_syntax::sap_macro(file, s) {
        macro_accesses(file, s, name, used, &mut reads, &mut writes);
    }
    let conditions: Vec<m::Condition> = ops.conds().map(|c| condition(&toks, c)).collect();
    for (field, whole) in sets_fields(file, s, &conditions) {
        writes.push(implicit(field, start_of(&toks), whole));
    }
    // the start value of `DATA x TYPE i VALUE 1`
    if matches!(s.kind, K::Data | K::Constant) && toks.iter().any(|t| t.is("VALUE")) {
        if let Some(v) = decl::variable(file, s) {
            let t = &file.tokens()[v.token];
            writes.push(m::DataAccess {
                name: v.name.to_ascii_lowercase(),
                path: v.name.clone(),
                access: m::AccessKind::Write,
                scope: m::DataScope::Local,
                whole: true,
                declared: true,
                implicit: false,
                data_type: None,
                sources: Vec::new(),
                from_entry: false,
                via: None,
                through: None,
                depends: None,
                position: m::Position {
                    row: t.start.row,
                    col: t.start.col,
                },
            });
        }
    }

    let condition = match s.kind {
        K::Catch => {
            let classes = decl::text_after(file, s, 1);
            let classes = classes
                .split(" INTO ")
                .next()
                .unwrap_or_default()
                .trim()
                .to_string();
            Some(m::Condition {
                kind: m::ConditionKind::Catch,
                text: classes,
                children: Vec::new(),
                left: None,
                operator: None,
                right: None,
                high: None,
                values: Vec::new(),
                negated: false,
            })
        }
        _ => ops.conds().next().map(|c| condition(&toks, c)),
    };

    m::Step {
        id: i as u32,
        kind: s.kind.as_str().to_string(),
        text: file.statement_text(s),
        span: span_of(file, s),
        condition,
        guards: Vec::new(),
        reads,
        writes,
        tables: Vec::new(),
        children: Vec::new(),
    }
}

fn access(toks: &[&Token], r: &DataRef) -> m::DataAccess {
    let path = text(toks, r.start, r.last);
    let field = system_field(&path);
    let known = field.as_ref().and_then(|(_, f)| *f);
    let field = field.map(|(name, _)| name);
    let name = if r.attribute {
        let full = text(toks, r.start, r.pos).to_ascii_lowercase();
        full.strip_prefix("me->")
            .map(str::to_string)
            .unwrap_or(full)
    } else if let Some(f) = &field {
        f.clone()
    } else {
        toks[r.pos].text.to_ascii_lowercase()
    };
    let t = toks[r.pos];
    // `sy-subrc = 4` replaces the system field
    let whole = r.whole
        || field
            .as_deref()
            .is_some_and(|f| f.eq_ignore_ascii_case(&path));
    m::DataAccess {
        name: name.trim_start_matches('!').to_string(),
        path,
        access: match r.access {
            Access::Read => m::AccessKind::Read,
            Access::Write => m::AccessKind::Write,
            Access::Change => m::AccessKind::Change,
        },
        scope: if r.attribute {
            m::DataScope::Attribute
        } else if known.is_some() {
            m::DataScope::System
        } else {
            m::DataScope::Other
        },
        whole: r.access == Access::Write && whole,
        declared: r.declared,
        implicit: false,
        data_type: known.map(data_type),
        sources: Vec::new(),
        from_entry: false,
        via: None,
        through: None,
        depends: None,
        position: m::Position {
            row: t.start.row,
            col: t.start.col,
        },
    }
}

/// A system field accessed as `sy-subrc` or `syst-subrc`: its name
/// `sy-subrc`, lower case, and the field of SYST, unless no field has the
/// name. Each is its own data object, not a component of `sy`.
fn system_field(path: &str) -> Option<(String, Option<&'static SystemField>)> {
    let lower = path.to_ascii_lowercase();
    let rest = lower
        .strip_prefix("sy-")
        .or_else(|| lower.strip_prefix("syst-"))?;
    let field: String = rest
        .chars()
        .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
        .collect();
    (!field.is_empty()).then(|| (format!("sy-{field}"), system::system_field(&field)))
}

fn data_type(f: &SystemField) -> m::DataType {
    m::DataType {
        kind: f.kind.as_str().to_string(),
        length: (f.length > 0).then_some(f.length),
        decimals: (f.decimals > 0).then_some(f.decimals),
        data_element: Some(f.data_element.to_string()),
    }
}

fn start_of(toks: &[&Token]) -> m::Position {
    m::Position {
        row: toks[0].start.row,
        col: toks[0].start.col,
    }
}

/// Data a statement writes without naming it, like `sy-subrc`; `whole`
/// when it always does.
fn implicit(name: &str, at: m::Position, whole: bool) -> m::DataAccess {
    let field = system_field(name).and_then(|(_, f)| f);
    m::DataAccess {
        name: name.to_string(),
        path: name.to_string(),
        access: m::AccessKind::Write,
        scope: match field {
            Some(_) => m::DataScope::System,
            None => m::DataScope::Other,
        },
        whole,
        declared: false,
        implicit: true,
        data_type: field.map(data_type),
        sources: Vec::new(),
        from_entry: false,
        via: None,
        through: None,
        depends: None,
        position: at,
    }
}

/// Whether a statement sets `sy-subrc`: `Some(true)` when it always does,
/// as SAP documents for each statement, `Some(false)` when it may. Calls
/// of methods and forms are added with the calls of the step, see
/// [`call_sets_subrc`].
fn sets_subrc(file: &File, s: &Statement) -> Option<bool> {
    match s.kind {
        K::ReadTable
        | K::Loop
        | K::InsertInternal
        | K::ModifyInternal
        | K::DeleteInternal
        | K::Select
        | K::SelectLoop
        | K::With
        | K::WithLoop
        | K::InsertDatabase
        | K::UpdateDatabase
        | K::ModifyDatabase
        | K::DeleteDatabase
        | K::FetchNextCursor
        | K::CallFunction
        | K::CreateObject
        | K::AuthorityCheck
        | K::Find
        | K::Replace
        | K::Search
        | K::Split
        | K::Concatenate
        | K::Overlay
        | K::Import
        | K::DeleteCluster
        | K::GetParameter
        | K::OpenDataset
        | K::ReadDataset
        | K::DeleteDataset
        | K::CallSelectionScreen
        | K::ReadLine
        | K::ReadReport
        | K::InsertReport
        | K::GenerateSubroutine
        | K::Convert
        | K::GetBit
        | K::SetBit
        | K::Commit
        // ENDEXEC sets it, for the block
        | K::ExecSQL => Some(true),
        // only a dynamic assignment or one of a table expression
        K::Assign => {
            let toks = tokens(file, s);
            let to = toks.iter().position(|t| t.is("TO")).unwrap_or(toks.len());
            toks[..to]
                .iter()
                .any(|t| {
                    matches!(t.kind, TokenKind::ParenLeft | TokenKind::BracketLeft)
                        || t.is("COMPONENT")
                })
                .then_some(true)
        }
        // with USING the result of the batch input
        K::CallTransaction => Some(false),
        _ => None,
    }
}

/// The system fields a statement sets, as SAP documents for each, and
/// whether it always does: `sy-subrc` (see [`sets_subrc`]), `sy-tabix`,
/// `sy-index` in loops, `sy-dbcnt`, `sy-fdpos`, `sy-tfill` and `sy-tleng`,
/// the message fields. Loops set theirs in each pass, which is what their
/// statements read; after the loop the value is again the one from before,
/// which this does not tell.
fn sets_fields(
    file: &File,
    s: &Statement,
    conditions: &[m::Condition],
) -> Vec<(&'static str, bool)> {
    let mut out: Vec<(&'static str, bool)> = sets_subrc(file, s)
        .map(|whole| ("sy-subrc", whole))
        .into_iter()
        .collect();
    let describe_table = || tokens(file, s).get(1).is_some_and(|t| t.is("TABLE"));
    out.extend_from_slice(match s.kind {
        K::ReadTable | K::Append | K::Collect | K::Loop => &[("sy-tabix", true)],
        K::Do | K::While => &[("sy-index", true)],
        K::Select
        | K::SelectLoop
        | K::With
        | K::WithLoop
        | K::InsertDatabase
        | K::UpdateDatabase
        | K::ModifyDatabase
        | K::DeleteDatabase
        | K::FetchNextCursor
        | K::ExecSQL => &[("sy-dbcnt", true)],
        // only when it finds the pattern
        K::Search => &[("sy-fdpos", false)],
        K::Describe if describe_table() => &[("sy-tfill", true), ("sy-tleng", true)],
        K::Message => &[
            ("sy-msgid", true),
            ("sy-msgty", true),
            ("sy-msgno", true),
            ("sy-msgv1", true),
            ("sy-msgv2", true),
            ("sy-msgv3", true),
            ("sy-msgv4", true),
        ],
        _ => &[],
    });
    // CO, CN, CA, NA, CS, NS, CP and NP set sy-fdpos, true or not
    fn compares_strings(c: &m::Condition) -> bool {
        c.operator
            .as_deref()
            .is_some_and(|o| matches!(o, "CO" | "CN" | "CA" | "NA" | "CS" | "NS" | "CP" | "NP"))
            || c.children.iter().any(compares_strings)
    }
    if conditions.iter().any(compares_strings) && !out.iter().any(|(f, _)| *f == "sy-fdpos") {
        out.push(("sy-fdpos", true));
    }
    out
}

/// Whether a call sets `sy-subrc`: every method call sets it to 0 when the
/// method is called, the method may change it then; a form may set it.
fn call_sets_subrc(kind: m::CallKind) -> Option<bool> {
    match kind {
        m::CallKind::Method | m::CallKind::Constructor => Some(true),
        m::CallKind::Form => Some(false),
        _ => None,
    }
}

/// What a macro of SAP reads and writes, see [`crate::hr::macro_access`].
fn macro_accesses(
    file: &File,
    s: &Statement,
    name: &str,
    used: usize,
    reads: &mut Vec<m::DataAccess>,
    writes: &mut Vec<m::DataAccess>,
) {
    use crate::hr;
    let Some(effect) = hr::macro_access(name) else {
        return;
    };
    let args = hr::arguments(file, s, used);
    let access = |i: usize, kind: m::AccessKind, whole: bool| {
        let arg = args.get(i).filter(|a| hr::is_data(a))?;
        let path = hr::join(arg);
        let field = system_field(&path);
        let known = field.as_ref().and_then(|(_, f)| *f);
        Some(m::DataAccess {
            name: field
                .map(|(name, _)| name)
                .unwrap_or_else(|| arg[0].text.to_ascii_lowercase()),
            path,
            access: kind,
            scope: match known {
                Some(_) => m::DataScope::System,
                None => m::DataScope::Other,
            },
            whole,
            declared: false,
            implicit: false,
            data_type: known.map(data_type),
            sources: Vec::new(),
            from_entry: false,
            via: None,
            through: None,
            depends: None,
            position: m::Position {
                row: arg[0].start.row,
                col: arg[0].start.col,
            },
        })
    };
    reads.extend(
        effect
            .reads
            .iter()
            .filter_map(|i| access(*i, m::AccessKind::Read, false)),
    );
    writes.extend(
        effect
            .writes
            .iter()
            .filter_map(|(i, whole)| access(*i, m::AccessKind::Write, *whole)),
    );
    let at = start_of(&tokens(file, s));
    for set in effect.sets {
        // like `pnp-sw-found`, a field of the structure `pnp`
        let root = set.split('-').next().unwrap_or(set);
        writes.push(m::DataAccess {
            path: set.to_string(),
            ..implicit(root, at, false)
        });
    }
}

fn operand(toks: &[&Token], e: &Expr) -> m::Operand {
    let text = text(toks, e.first, e.last);
    let mut value = None;
    let kind = match &e.kind {
        ExprKind::Literal => {
            value = literal_value(&text);
            m::OperandKind::Literal
        }
        ExprKind::Template(_) => m::OperandKind::Template,
        ExprKind::Chain(c) => {
            let call = matches!(c.links.last(), Some(Link::Method { .. }))
                || (c.links.is_empty() && matches!(c.head, Head::Call { .. }));
            if call {
                m::OperandKind::Call
            } else {
                m::OperandKind::Data
            }
        }
        ExprKind::Constructor(_) => m::OperandKind::Constructor,
        ExprKind::Unary { operand, .. } if matches!(operand.kind, ExprKind::Literal) => {
            value = Some(text.replace(' ', ""));
            m::OperandKind::Literal
        }
        ExprKind::Binary { .. } | ExprKind::Unary { .. } | ExprKind::Paren(_) => {
            m::OperandKind::Arithmetic
        }
        ExprKind::Component => m::OperandKind::Component,
    };
    m::Operand { kind, text, value }
}

fn literal_value(text: &str) -> Option<String> {
    let quoted = |q: char| {
        let inner = text.strip_prefix(q)?;
        let end = inner.rfind(q)?;
        Some(inner[..end].replace(&format!("{q}{q}"), &q.to_string()))
    };
    if text.starts_with('\'') {
        quoted('\'')
    } else if text.starts_with('`') {
        quoted('`')
    } else if text.starts_with('|') && text.ends_with('|') && text.len() >= 2 {
        Some(text[1..text.len() - 1].to_string())
    } else if text.bytes().all(|b| b.is_ascii_digit()) {
        Some(text.to_string())
    } else {
        None
    }
}

fn condition(toks: &[&Token], c: &Cond) -> m::Condition {
    let mut out = m::Condition {
        kind: m::ConditionKind::Predicate,
        text: text(toks, c.first, c.last),
        children: Vec::new(),
        left: None,
        operator: None,
        right: None,
        high: None,
        values: Vec::new(),
        negated: false,
    };
    match &c.kind {
        CondKind::And(all) | CondKind::Or(all) | CondKind::Equiv(all) => {
            out.kind = match c.kind {
                CondKind::And(_) => m::ConditionKind::And,
                CondKind::Or(_) => m::ConditionKind::Or,
                _ => m::ConditionKind::Equiv,
            };
            out.children = all.iter().map(|c| condition(toks, c)).collect();
        }
        CondKind::Not(inner) => {
            out.kind = m::ConditionKind::Not;
            out.children = vec![condition(toks, inner)];
        }
        CondKind::Paren(inner) => return condition(toks, inner),
        CondKind::Compare { left, op, right } => {
            out.kind = m::ConditionKind::Compare;
            out.left = Some(operand(toks, left));
            out.operator = Some(op.clone());
            out.right = Some(operand(toks, right));
        }
        CondKind::Is {
            operand: o,
            negated,
            predicate,
            class,
        } => {
            out.kind = m::ConditionKind::Is;
            out.left = Some(operand(toks, o));
            out.operator = Some(predicate.clone());
            out.negated = *negated;
            out.right = class.map(|p| m::Operand {
                kind: m::OperandKind::Data,
                text: text(toks, p, p),
                value: None,
            });
        }
        CondKind::In {
            operand: o,
            negated,
            values,
        } => {
            out.kind = m::ConditionKind::In;
            out.left = Some(operand(toks, o));
            out.operator = Some("IN".into());
            out.negated = *negated;
            out.values = values.iter().map(|v| operand(toks, v)).collect();
        }
        CondKind::Between {
            operand: o,
            negated,
            low,
            high,
        } => {
            out.kind = m::ConditionKind::Between;
            out.left = Some(operand(toks, o));
            out.operator = Some("BETWEEN".into());
            out.negated = *negated;
            out.right = Some(operand(toks, low));
            out.high = Some(operand(toks, high));
        }
        CondKind::Predicate(o) => {
            out.left = Some(operand(toks, o));
        }
    }
    out
}

/// The steps of a procedure nested along its control structures.
fn tree(ctx: &Ctx, statements: &[usize]) -> Vec<m::Step> {
    let file = ctx.file;
    let mut root: Vec<m::Step> = Vec::new();
    let mut stack: Vec<(K, m::Step)> = Vec::new();

    fn attach(stack: &mut [(K, m::Step)], root: &mut Vec<m::Step>, step: m::Step) {
        match stack.last_mut() {
            Some((_, parent)) => parent.children.push(step),
            None => root.push(step),
        }
    }
    // closes the blocks from `pos` upwards
    fn close(stack: &mut Vec<(K, m::Step)>, root: &mut Vec<m::Step>, pos: usize) {
        while stack.len() > pos {
            let (_, step) = stack.pop().unwrap();
            attach(stack, root, step);
        }
    }

    for &i in statements {
        let kind = file.statements[i].kind;
        // the macros of SAP have no definition here: their call is the step
        if is_frame(kind) && abap_syntax::sap_macro(file, &file.statements[i]).is_none() {
            continue;
        }
        if closers().contains(&kind) {
            if let Some(pos) = stack.iter().rposition(|(k, _)| closes(kind, *k)) {
                close(&mut stack, &mut root, pos);
            }
            continue;
        }
        let step = step(ctx, i);
        match kind {
            K::ElseIf | K::Else | K::Catch | K::Cleanup => {
                let opener = |k: &K| match kind {
                    K::ElseIf | K::Else => matches!(k, K::If | K::ElseIf),
                    _ => matches!(k, K::Try | K::Catch | K::Cleanup),
                };
                if let Some(pos) = stack.iter().rposition(|(k, _)| opener(k)) {
                    close(&mut stack, &mut root, pos);
                }
                stack.push((kind, step));
            }
            K::When | K::WhenOthers | K::WhenType => {
                if let Some(pos) = stack
                    .iter()
                    .rposition(|(k, _)| matches!(k, K::Case | K::CaseType))
                {
                    close(&mut stack, &mut root, pos + 1);
                }
                stack.push((kind, step));
            }
            _ if opens(kind) => stack.push((kind, step)),
            _ => attach(&mut stack, &mut root, step),
        }
    }
    close(&mut stack, &mut root, 0);
    root
}

/// The conditions of WHEN: the CASE operand compared to the WHEN values.
fn case_conditions(steps: &mut [m::Step]) {
    for s in steps.iter_mut() {
        if kind_of(s) == K::Case {
            let operand = s
                .text
                .trim_end_matches(['.', ','])
                .split_once(' ')
                .map(|x| x.1);
            let operand = operand.unwrap_or_default().trim().to_string();
            for w in s.children.iter_mut().filter(|w| kind_of(w) == K::When) {
                let values = w.text.trim_end_matches(['.', ',']);
                let values = values.split_once(' ').map(|x| x.1).unwrap_or_default();
                let compares: Vec<m::Condition> = split_or(values)
                    .into_iter()
                    .map(|v| m::Condition {
                        kind: m::ConditionKind::Compare,
                        text: format!("{operand} = {v}"),
                        children: Vec::new(),
                        left: Some(m::Operand {
                            kind: m::OperandKind::Data,
                            text: operand.clone(),
                            value: None,
                        }),
                        operator: Some("=".into()),
                        right: Some(m::Operand {
                            kind: if literal_value(&v).is_some() {
                                m::OperandKind::Literal
                            } else {
                                m::OperandKind::Data
                            },
                            value: literal_value(&v),
                            text: v,
                        }),
                        high: None,
                        values: Vec::new(),
                        negated: false,
                    })
                    .collect();
                w.condition = match compares.len() {
                    0 => None,
                    1 => compares.into_iter().next(),
                    _ => Some(m::Condition {
                        kind: m::ConditionKind::Or,
                        text: compares
                            .iter()
                            .map(|c| c.text.as_str())
                            .collect::<Vec<_>>()
                            .join(" OR "),
                        children: compares,
                        left: None,
                        operator: None,
                        right: None,
                        high: None,
                        values: Vec::new(),
                        negated: false,
                    }),
                };
            }
        }
        case_conditions(&mut s.children);
    }
}

/// `a OR 'b c' OR d` split at the OR outside of literals.
fn split_or(values: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut current = String::new();
    let mut quote: Option<char> = None;
    let words: Vec<&str> = values.split(' ').collect();
    for w in words {
        if quote.is_none() && w.eq_ignore_ascii_case("OR") {
            out.push(std::mem::take(&mut current));
            continue;
        }
        for ch in w.chars() {
            match quote {
                Some(q) if ch == q => quote = None,
                None if ch == '\'' || ch == '`' => quote = Some(ch),
                _ => {}
            }
        }
        if !current.is_empty() {
            current.push(' ');
        }
        current.push_str(w);
    }
    out.push(current);
    out.retain(|v| !v.is_empty());
    out
}

// ----- field symbols and references ------------------------------------------

fn by_position(a: &&m::DataAccess, b: &&m::DataAccess) -> std::cmp::Ordering {
    (a.position.row, a.position.col).cmp(&(b.position.row, b.position.col))
}

/// The field symbol or data reference a step binds, and the data it binds
/// it to: `LOOP AT itab ASSIGNING <fs>`, `READ TABLE itab REFERENCE INTO lr`,
/// `ASSIGN x TO <fs>`, `GET REFERENCE OF x INTO lr`, `lr = REF #( x )`.
fn binding(s: &m::Step) -> Option<(&m::DataAccess, &m::DataAccess)> {
    let upper = s.text.to_ascii_uppercase();
    let own_reads = || {
        s.reads
            .iter()
            .filter(|r| r.via.is_none() && r.through.is_none())
    };
    let first_read = || own_reads().min_by(by_position);
    let assigning = upper.contains(" ASSIGNING ");
    match kind_of(s) {
        K::Loop | K::ReadTable | K::InsertInternal | K::Append | K::ModifyInternal | K::Collect
            if assigning || upper.contains(" REFERENCE INTO ") =>
        {
            let alias = s
                .writes
                .iter()
                .filter(|w| {
                    w.via.is_none()
                        && if assigning {
                            w.name.starts_with('<')
                        } else {
                            w.access == m::AccessKind::Write
                        }
                })
                .max_by(by_position)?;
            // the table: changed by INSERT and APPEND, read by READ TABLE
            let table = own_reads()
                .filter(|r| r.access == m::AccessKind::Change)
                .min_by(by_position)
                .or_else(first_read)?;
            Some((alias, table))
        }
        K::Assign if !upper.starts_with("ASSIGN (") => {
            let alias = s.writes.iter().find(|w| w.name.starts_with('<'))?;
            let target = match upper.find(" OF STRUCTURE ") {
                Some(i) => {
                    let word = upper[i + 14..].split_whitespace().next()?;
                    let root = word.split('-').next().unwrap_or(word);
                    own_reads().find(|r| r.name.eq_ignore_ascii_case(root))
                }
                None => first_read(),
            }?;
            Some((alias, target))
        }
        K::GetReference => Some((s.writes.first()?, first_read()?)),
        K::Move
            if upper
                .split_once('=')
                .is_some_and(|(_, r)| r.trim_start().starts_with("REF ")) =>
        {
            Some((s.writes.first()?, first_read()?))
        }
        _ => None,
    }
}

/// Field symbols and references with their scope.
type AliasScopes = HashMap<String, m::DataScope>;

/// Field symbols and references with what they may point to anywhere in a
/// procedure, for its callers.
type Aliases = HashMap<String, (m::DataScope, Vec<(String, m::DataScope)>)>;

/// Writes and reads of a name: where definitions of it reach.
type Reach = HashMap<(u32, String), (Vec<u32>, bool)>;

/// The field symbol or reference an access goes through, and whether it
/// dereferences a reference (`lr->x`, `lr->*`).
fn alias_of(a: &m::DataAccess, aliases: &AliasScopes) -> Option<(String, bool)> {
    if a.name.starts_with('<') && aliases.contains_key(&a.name) {
        return Some((a.name.clone(), false));
    }
    deref_root(a)
        .filter(|root| aliases.contains_key(root))
        .map(|root| (root, true))
}

/// `lr` of `lr->x` or `lr->*`.
fn deref_root(a: &m::DataAccess) -> Option<String> {
    if let Some((root, _)) = a.name.split_once("->") {
        return Some(root.to_string());
    }
    a.path
        .to_ascii_lowercase()
        .strip_prefix(a.name.as_str())
        .is_some_and(|rest| rest.starts_with("->"))
        .then(|| a.name.clone())
}

/// Reaching definitions of some names only: `defs` are the steps
/// defining a name, `uses` the steps asking where it comes from.
fn reaching(
    body: &[m::Step],
    flow: &[m::FlowEdge],
    defs: &HashMap<u32, Vec<String>>,
    uses: &HashMap<u32, Vec<String>>,
) -> Reach {
    let plain = |name: &String, access: m::AccessKind| m::DataAccess {
        name: name.clone(),
        path: name.clone(),
        access,
        scope: m::DataScope::Local,
        whole: access == m::AccessKind::Write,
        declared: false,
        implicit: false,
        data_type: None,
        sources: Vec::new(),
        from_entry: false,
        via: None,
        through: None,
        depends: None,
        position: m::Position { row: 0, col: 0 },
    };
    fn skeleton(
        steps: &[m::Step],
        f: &dyn Fn(u32) -> (Vec<m::DataAccess>, Vec<m::DataAccess>),
    ) -> Vec<m::Step> {
        steps
            .iter()
            .map(|s| {
                let (reads, writes) = f(s.id);
                m::Step {
                    id: s.id,
                    kind: s.kind.clone(),
                    text: String::new(),
                    span: s.span,
                    condition: None,
                    guards: Vec::new(),
                    reads,
                    writes,
                    tables: Vec::new(),
                    children: skeleton(&s.children, f),
                }
            })
            .collect()
    }
    let accesses = |id: u32| {
        let reads = uses
            .get(&id)
            .into_iter()
            .flatten()
            .map(|n| plain(n, m::AccessKind::Read))
            .collect();
        let writes = defs
            .get(&id)
            .into_iter()
            .flatten()
            .map(|n| plain(n, m::AccessKind::Write))
            .collect();
        (reads, writes)
    };
    let mut sk = skeleton(body, &accesses);
    data_flow(&mut sk, flow);
    let mut out = Reach::new();
    walk(&sk, &mut |s| {
        for r in &s.reads {
            out.insert((s.id, r.name.clone()), (r.sources.clone(), r.from_entry));
        }
    });
    out
}

/// Bindings of the steps: (alias, target, scope of the target).
type Bindings = HashMap<u32, Vec<(String, String, m::DataScope)>>;

/// What an alias points to at a step: the targets of the bindings reaching it.
fn resolve_at(
    alias: &str,
    step: u32,
    reach: &Reach,
    bindings: &Bindings,
    scopes: &AliasScopes,
    depth: u32,
) -> Vec<(String, m::DataScope)> {
    let Some((sources, _)) = reach.get(&(step, alias.to_string())) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for b in sources {
        for (a, t, ts) in bindings.get(b).into_iter().flatten() {
            if a != alias {
                continue;
            }
            let more = if scopes.contains_key(t) {
                if depth < 5 {
                    resolve_at(t, *b, reach, bindings, scopes, depth + 1)
                } else {
                    Vec::new()
                }
            } else {
                vec![(t.clone(), *ts)]
            };
            for x in more {
                if !out.contains(&x) {
                    out.push(x);
                }
            }
        }
    }
    out
}

/// Writes and reads through field symbols and data references become
/// accesses of the data they point to, as bound on the way to the step.
/// Binding a table to a field symbol does not change it. A write through a
/// reference parameter is a write of the parameter. Returns what each alias
/// may point to, for the callers.
fn aliases(
    body: &mut [m::Step],
    flow: &[m::FlowEdge],
    extra: &[Binding],
    parameters: &HashSet<String>,
) -> Aliases {
    let mut bindings: Bindings = HashMap::new();
    let mut scopes: AliasScopes = HashMap::new();
    walk(body, &mut |s| {
        if let Some((alias, target)) = binding(s) {
            bindings.entry(s.id).or_default().push((
                alias.name.clone(),
                target.name.clone(),
                target.scope,
            ));
            scopes.entry(alias.name.clone()).or_insert(alias.scope);
        }
        for (step, alias, target, scope) in extra.iter().filter(|b| b.0 == s.id) {
            let own = s.writes.iter().find(|w| w.name == *alias).map(|w| w.scope);
            scopes
                .entry(alias.clone())
                .or_insert(own.unwrap_or(m::DataScope::Local));
            bindings
                .entry(*step)
                .or_default()
                .push((alias.clone(), target.clone(), *scope));
        }
    });

    let reach = if scopes.is_empty() {
        Reach::new()
    } else {
        let defs: HashMap<u32, Vec<String>> = bindings
            .iter()
            .map(|(s, b)| (*s, b.iter().map(|x| x.0.clone()).collect()))
            .collect();
        let mut uses: HashMap<u32, Vec<String>> = HashMap::new();
        walk(body, &mut |s| {
            let u = uses.entry(s.id).or_default();
            for a in s.reads.iter().chain(&s.writes) {
                if let Some((alias, _)) = alias_of(a, &scopes) {
                    if !u.contains(&alias) {
                        u.push(alias);
                    }
                }
            }
            for (_, t, _) in bindings.get(&s.id).into_iter().flatten() {
                if scopes.contains_key(t) && !u.contains(t) {
                    u.push(t.clone());
                }
            }
        });
        reaching(body, flow, &defs, &uses)
    };

    let mut union: Aliases = HashMap::new();
    walk_mut(body, &mut |s| {
        let bound: Vec<(String, String, m::DataScope)> =
            bindings.get(&s.id).cloned().unwrap_or_default();
        for (a, t, ts) in &bound {
            // for the callers: what it points to after this step
            let targets = if scopes.contains_key(t) {
                resolve_at(t, s.id, &reach, &bindings, &scopes, 1)
            } else {
                vec![(t.clone(), *ts)]
            };
            let e = union.entry(a.clone()).or_insert_with(|| {
                (
                    scopes.get(a).copied().unwrap_or(m::DataScope::Local),
                    Vec::new(),
                )
            });
            for x in targets {
                if !e.1.contains(&x) {
                    e.1.push(x);
                }
            }
        }
        if let Some((_, table, _)) = bound.first() {
            if matches!(kind_of(s), K::Loop | K::ReadTable) {
                s.writes
                    .retain(|w| w.name != *table || w.access != m::AccessKind::Change);
                for r in s.reads.iter_mut().filter(|r| r.name == *table) {
                    r.access = m::AccessKind::Read;
                }
            }
        }
        let id = s.id;
        let through =
            |a: &m::DataAccess, alias: &str, (name, scope): (String, m::DataScope)| m::DataAccess {
                name,
                scope,
                path: a.path.clone(),
                access: a.access,
                whole: false,
                declared: false,
                implicit: false,
                data_type: None,
                sources: Vec::new(),
                from_entry: false,
                via: a.via.clone(),
                through: Some(alias.to_string()),
                depends: a.depends.clone(),
                position: a.position,
            };
        let root = |a: &m::DataAccess, alias: &str| m::DataAccess {
            name: alias.to_string(),
            scope: scopes.get(alias).copied().unwrap_or(m::DataScope::Local),
            path: alias.to_string(),
            access: m::AccessKind::Read,
            whole: false,
            declared: false,
            implicit: false,
            data_type: None,
            sources: Vec::new(),
            from_entry: false,
            via: None,
            through: None,
            depends: None,
            position: a.position,
        };
        let targets = |alias: &str| resolve_at(alias, id, &reach, &bindings, &scopes, 0);
        // through a reference parameter: `ir->*` is the parameter, `io->x`
        // the attribute of the object passed
        let parameter = |a: &m::DataAccess| {
            deref_root(a)
                .filter(|r| parameters.contains(r))
                .map(|r| m::DataAccess {
                    scope: if a.name == r {
                        m::DataScope::Parameter
                    } else {
                        a.scope
                    },
                    whole: false,
                    through: Some(r),
                    ..a.clone()
                })
        };
        let mut writes = Vec::new();
        let mut reads = Vec::new();
        for w in std::mem::take(&mut s.writes) {
            let Some((alias, deref)) = alias_of(&w, &scopes) else {
                if let Some(p) = parameter(&w) {
                    writes.push(p);
                } else if w.path.contains("->") && !w.name.contains("->") {
                    // `lr->* = x` with an unknown target does not replace `lr`
                    writes.push(m::DataAccess { whole: false, ..w });
                } else {
                    writes.push(w);
                }
                continue;
            };
            if !deref && bound.iter().any(|(a, _, _)| *a == alias) {
                // the binding itself
                writes.push(w);
                continue;
            }
            for t in targets(&alias) {
                writes.push(through(&w, &alias, t));
            }
            if deref {
                reads.push(root(&w, &alias));
            } else {
                writes.push(w);
            }
        }
        for r in std::mem::take(&mut s.reads) {
            let Some((alias, deref)) = alias_of(&r, &scopes) else {
                reads.push(parameter(&r).unwrap_or(r));
                continue;
            };
            for t in targets(&alias) {
                reads.push(through(&r, &alias, t));
            }
            reads.push(if deref { root(&r, &alias) } else { r });
        }
        dedup_accesses(&mut writes);
        dedup_accesses(&mut reads);
        s.writes = writes;
        s.reads = reads;
    });
    union
}

fn dedup_accesses(list: &mut Vec<m::DataAccess>) {
    let mut seen: Vec<(String, Option<String>, m::AccessKind)> = Vec::new();
    list.retain(|a| {
        let key = (a.name.clone(), a.through.clone(), a.access);
        if seen.contains(&key) {
            false
        } else {
            seen.push(key);
            true
        }
    });
}

// ----- guards ----------------------------------------------------------------

fn guard_of(step: &m::Step, negated: bool) -> Option<m::Guard> {
    step.condition.as_ref().map(|c| m::Guard {
        statement: step.id,
        negated,
        text: c.text.clone(),
        subrc: Vec::new(),
    })
}

/// Fills `subrc` of the guards checking `sy-subrc`: the statements whose
/// value the condition reads, after the data flow. A WHEN reads it through
/// its CASE.
fn guard_subrc(body: &mut [m::Step]) {
    let mut set_by: HashMap<u32, Vec<u32>> = HashMap::new();
    let mut case_of: HashMap<u32, u32> = HashMap::new();
    walk(body, &mut |s| {
        let sources: BTreeSet<u32> = s
            .reads
            .iter()
            .filter(|r| r.name == "sy-subrc")
            .flat_map(|r| r.sources.iter().copied())
            .collect();
        if !sources.is_empty() {
            set_by.insert(s.id, sources.into_iter().collect());
        }
        if matches!(kind_of(s), K::Case) {
            for w in &s.children {
                case_of.insert(w.id, s.id);
            }
        }
    });
    if set_by.is_empty() {
        return;
    }
    walk_mut(body, &mut |s| {
        for g in &mut s.guards {
            let checked = set_by
                .get(&g.statement)
                .or_else(|| case_of.get(&g.statement).and_then(|c| set_by.get(c)));
            if let Some(ids) = checked {
                g.subrc = ids.clone();
            }
        }
    });
}

/// Whether the steps end with a jump out of their block.
fn exits(steps: &[m::Step]) -> bool {
    steps.last().is_some_and(|s| is_jump(kind_of(s)))
}

/// Fills the guards of the steps, given the guards of their block.
fn guards(steps: &mut [m::Step], outer: &[m::Guard]) {
    // guards from CHECK and early exits for the following steps
    let mut pending: Vec<m::Guard> = Vec::new();
    let mut i = 0;
    while i < steps.len() {
        let base: Vec<m::Guard> = outer.iter().chain(&pending).cloned().collect();
        let kind = kind_of(&steps[i]);
        match kind {
            K::If => {
                let mut end = i + 1;
                while end < steps.len() && matches!(kind_of(&steps[end]), K::ElseIf | K::Else) {
                    end += 1;
                }
                let mut prior: Vec<m::Guard> = Vec::new();
                let mut all_exit = true;
                let mut has_else = false;
                for b in &mut steps[i..end] {
                    b.guards = base.iter().chain(&prior).cloned().collect();
                    let mut inner = b.guards.clone();
                    inner.extend(guard_of(b, false));
                    guards(&mut b.children, &inner);
                    match guard_of(b, true) {
                        Some(g) => prior.push(g),
                        None => has_else = true,
                    }
                    all_exit &= exits(&b.children);
                }
                if all_exit && !has_else {
                    pending.extend(prior);
                }
                i = end;
                continue;
            }
            K::Case | K::CaseType => {
                steps[i].guards = base.clone();
                let mut prior: Vec<m::Guard> = Vec::new();
                for w in &mut steps[i].children {
                    w.guards = base.iter().chain(&prior).cloned().collect();
                    let mut inner = w.guards.clone();
                    inner.extend(guard_of(w, false));
                    guards(&mut w.children, &inner);
                    prior.extend(guard_of(w, true));
                }
            }
            _ => {
                steps[i].guards = base.clone();
                let mut inner = base;
                if matches!(kind, K::While | K::Loop | K::Catch | K::At | K::OnChange) {
                    inner.extend(guard_of(&steps[i], false));
                }
                guards(&mut steps[i].children, &inner);
                if kind == K::Check {
                    pending.extend(guard_of(&steps[i], false));
                }
            }
        }
        i += 1;
    }
}

// ----- control flow ----------------------------------------------------------

/// A step, or `None` for the entry or exit of the procedure.
type Node = Option<u32>;

struct Flow {
    edges: Vec<m::FlowEdge>,
}

#[derive(Clone, Default)]
struct Scope {
    /// where CONTINUE and a failing CHECK go
    loop_head: Option<u32>,
    /// where EXIT goes
    loop_exit: Option<Node>,
    /// CATCH steps of the enclosing TRY blocks
    catches: Vec<u32>,
}

impl Flow {
    fn edge(&mut self, from: Node, to: Node, label: Option<&str>) {
        let e = m::FlowEdge {
            from,
            to,
            label: label.map(str::to_string),
        };
        if !self.edges.contains(&e) {
            self.edges.push(e);
        }
    }

    /// Edges of a sequence of steps continuing at `follow`; returns where
    /// the sequence starts.
    fn sequence(&mut self, steps: &[m::Step], follow: Node, scope: &Scope) -> Node {
        // units: IF with its ELSEIFs and ELSE, TRY with its CATCHes
        let mut units: Vec<&[m::Step]> = Vec::new();
        let mut i = 0;
        while i < steps.len() {
            let kind = kind_of(&steps[i]);
            let mut end = i + 1;
            let follows = |k: K| match kind {
                K::If => matches!(k, K::ElseIf | K::Else),
                K::Try => matches!(k, K::Catch | K::Cleanup),
                _ => false,
            };
            while end < steps.len() && follows(kind_of(&steps[end])) {
                end += 1;
            }
            units.push(&steps[i..end]);
            i = end;
        }
        let mut next = follow;
        for unit in units.into_iter().rev() {
            next = self.unit(unit, next, scope);
        }
        next
    }

    fn unit(&mut self, unit: &[m::Step], follow: Node, scope: &Scope) -> Node {
        let s = &unit[0];
        let id = Some(s.id);
        let kind = kind_of(s);
        for c in &scope.catches {
            if !matches!(kind, K::Try) {
                self.edge(id, Some(*c), Some("exception"));
            }
        }
        match kind {
            K::If => {
                let mut on_false = follow;
                for b in unit.iter().rev() {
                    let body = self.sequence(&b.children, follow, scope);
                    if kind_of(b) == K::Else {
                        self.edge(Some(b.id), body, None);
                    } else {
                        self.edge(Some(b.id), body, Some("true"));
                        self.edge(Some(b.id), on_false, Some("false"));
                    }
                    on_false = Some(b.id);
                }
            }
            K::Try => {
                let mut inner = scope.clone();
                for c in &unit[1..] {
                    if kind_of(c) == K::Catch {
                        inner.catches.push(c.id);
                        let body = self.sequence(&c.children, follow, scope);
                        self.edge(Some(c.id), body, None);
                    } else {
                        let body = self.sequence(&c.children, None, scope);
                        self.edge(Some(c.id), body, None);
                    }
                }
                let body = self.sequence(&s.children, follow, &inner);
                self.edge(id, body, None);
                for c in &unit[1..] {
                    self.edge(id, Some(c.id), Some("exception"));
                }
            }
            K::Case | K::CaseType => {
                let mut others = false;
                for w in &s.children {
                    others |= kind_of(w) == K::WhenOthers;
                    let body = self.sequence(&w.children, follow, scope);
                    self.edge(id, Some(w.id), Some("when"));
                    self.edge(Some(w.id), body, None);
                }
                if !others {
                    self.edge(id, follow, Some("other"));
                }
            }
            k if is_loop(k) => {
                let inner = Scope {
                    loop_head: Some(s.id),
                    loop_exit: Some(follow),
                    catches: scope.catches.clone(),
                };
                let body = self.sequence(&s.children, id, &inner);
                self.edge(id, body, Some("iterate"));
                self.edge(id, follow, Some("done"));
            }
            K::Return | K::Leave | K::Stop | K::Reject => self.edge(id, None, None),
            K::Exit => self.edge(id, scope.loop_exit.unwrap_or(None), None),
            K::Continue => self.edge(id, scope.loop_head, None),
            K::Check => {
                self.edge(id, follow, Some("true"));
                self.edge(id, scope.loop_head, Some("false"));
            }
            K::Raise => {
                if scope.catches.is_empty() {
                    self.edge(id, None, Some("exception"));
                }
            }
            _ if !s.children.is_empty() => {
                // AT NEW, ON CHANGE OF, TEST-SEAM ...: the body may run
                let body = self.sequence(&s.children, follow, scope);
                self.edge(id, body, Some("true"));
                self.edge(id, follow, Some("false"));
            }
            _ => self.edge(id, follow, None),
        }
        id
    }
}

fn flow(body: &[m::Step]) -> Vec<m::FlowEdge> {
    let mut f = Flow { edges: Vec::new() };
    let entry = f.sequence(body, None, &Scope::default());
    f.edge(None, entry, None);
    f.edges
}

// ----- data flow -------------------------------------------------------------

struct Bits(Vec<u64>);

impl Bits {
    fn new(n: usize) -> Self {
        Bits(vec![0; n.div_ceil(64)])
    }
    fn set(&mut self, i: usize) {
        self.0[i / 64] |= 1 << (i % 64);
    }
    fn get(&self, i: usize) -> bool {
        self.0[i / 64] & (1 << (i % 64)) != 0
    }
}

/// Reaching definitions: fills `sources` and `from_entry` of the reads.
fn data_flow(body: &mut [m::Step], edges: &[m::FlowEdge]) {
    // definitions: (step, name); u32::MAX is the entry
    const ENTRY: u32 = u32::MAX;
    let mut defs: Vec<(u32, String)> = Vec::new();
    let mut names: BTreeSet<String> = BTreeSet::new();
    let mut steps: Vec<u32> = Vec::new();
    walk(body, &mut |s| {
        steps.push(s.id);
        for a in s.reads.iter().chain(&s.writes) {
            names.insert(a.name.clone());
        }
        let mut written: Vec<&str> = s.writes.iter().map(|w| w.name.as_str()).collect();
        written.sort();
        written.dedup();
        for w in written {
            defs.push((s.id, w.to_string()));
        }
    });
    for n in &names {
        defs.push((ENTRY, n.clone()));
    }
    let n = defs.len();
    let mut defs_of: HashMap<&str, Vec<usize>> = HashMap::new();
    for (i, (_, name)) in defs.iter().enumerate() {
        defs_of.entry(name.as_str()).or_default().push(i);
    }
    let index: HashMap<u32, usize> = steps.iter().enumerate().map(|(i, s)| (*s, i)).collect();

    // gen and kill per step
    let mut gen: Vec<Bits> = steps.iter().map(|_| Bits::new(n)).collect();
    let mut kill: Vec<Bits> = steps.iter().map(|_| Bits::new(n)).collect();
    walk(body, &mut |s| {
        let k = index[&s.id];
        for w in &s.writes {
            let mine = defs_of[w.name.as_str()]
                .iter()
                .copied()
                .find(|d| defs[*d].0 == s.id);
            if let Some(d) = mine {
                gen[k].set(d);
            }
            if w.whole {
                for d in &defs_of[w.name.as_str()] {
                    if defs[*d].0 != s.id {
                        kill[k].set(*d);
                    }
                }
            }
        }
    });

    let mut entry = Bits::new(n);
    for (i, (s, _)) in defs.iter().enumerate() {
        if *s == ENTRY {
            entry.set(i);
        }
    }
    let mut preds: Vec<Vec<Option<usize>>> = vec![Vec::new(); steps.len()];
    for e in edges {
        let Some(to) = e.to.and_then(|t| index.get(&t)) else {
            continue;
        };
        let from = match e.from {
            None => None,
            Some(f) => match index.get(&f) {
                Some(i) => Some(*i),
                None => continue,
            },
        };
        preds[*to].push(from);
    }

    let words = n.div_ceil(64);
    let mut ins: Vec<Bits> = steps.iter().map(|_| Bits::new(n)).collect();
    let mut outs: Vec<Bits> = steps.iter().map(|_| Bits::new(n)).collect();
    let mut changed = true;
    while changed {
        changed = false;
        for k in 0..steps.len() {
            let mut input = vec![0u64; words];
            for p in &preds[k] {
                let src = match p {
                    None => &entry.0,
                    Some(p) => &outs[*p].0,
                };
                for (a, b) in input.iter_mut().zip(src) {
                    *a |= b;
                }
            }
            let output: Vec<u64> = input
                .iter()
                .zip(&gen[k].0)
                .zip(&kill[k].0)
                .map(|((i, g), k)| g | (i & !k))
                .collect();
            if output != outs[k].0 {
                outs[k].0 = output;
                changed = true;
            }
            ins[k].0 = input;
        }
    }

    walk_mut(body, &mut |s| {
        let k = index[&s.id];
        for r in &mut s.reads {
            let mut sources = BTreeSet::new();
            for d in defs_of.get(r.name.as_str()).into_iter().flatten() {
                if ins[k].get(*d) {
                    match defs[*d].0 {
                        ENTRY => r.from_entry = true,
                        step => {
                            sources.insert(step);
                        }
                    }
                }
            }
            r.sources = sources.into_iter().collect();
        }
    });
}

/// Non-local data and database tables written, and the inputs and tables
/// read that the written values depend on.
fn effects(body: &[m::Step]) -> Vec<m::Effect> {
    let mut by_id: HashMap<u32, &m::Step> = HashMap::new();
    // WHEN steps depend on the operand of their CASE
    let mut case_of: HashMap<u32, u32> = HashMap::new();
    walk(body, &mut |s| {
        by_id.insert(s.id, s);
        if matches!(kind_of(s), K::Case | K::CaseType) {
            for w in &s.children {
                case_of.insert(w.id, s.id);
            }
        }
    });
    // what the value of `var` written by a step depends on; without a
    // variable, e.g. for a condition, everything the step reads
    let inputs_of = |start: u32,
                     var: &str,
                     inputs: &mut BTreeSet<String>,
                     tables: &mut BTreeSet<String>| {
        let mut seen: HashSet<(u32, String)> = HashSet::new();
        let mut todo: Vec<(u32, String)> = vec![(start, var.to_string())];
        while let Some((id, var)) = todo.pop() {
            if !seen.insert((id, var.clone())) {
                continue;
            }
            let Some(s) = by_id.get(&id) else { continue };
            // written only through calls: what it depends on there
            let through_calls: Vec<&m::Dependency> = s
                .writes
                .iter()
                .filter(|w| !var.is_empty() && w.name == var)
                .map(|w| w.depends.as_ref())
                .chain(
                    s.tables
                        .iter()
                        .filter(|t| {
                            !var.is_empty() && t.name == var && t.access == m::AccessKind::Write
                        })
                        .map(|t| t.depends.as_ref()),
                )
                .collect::<Option<Vec<_>>>()
                .unwrap_or_default();
            let narrow = !through_calls.is_empty();
            let arguments = through_calls.iter().any(|d| d.arguments);
            for d in &through_calls {
                tables.extend(d.tables.iter().cloned());
            }
            for r in &s.reads {
                // the reference `lo` of `lo->x` decides which object it is
                let relevant = !narrow
                    || (arguments && r.via.is_none())
                    || through_calls.iter().any(|d| d.reads.contains(&r.name))
                    || var
                        .strip_prefix(r.name.as_str())
                        .is_some_and(|rest| rest.starts_with("->"));
                if !relevant {
                    continue;
                }
                // system fields come from the runtime, not from callers
                if r.from_entry && !matches!(r.scope, m::DataScope::Local | m::DataScope::System) {
                    inputs.insert(r.name.clone());
                }
                todo.extend(r.sources.iter().map(|src| (*src, r.name.clone())));
            }
            if !narrow {
                for t in &s.tables {
                    if t.access == m::AccessKind::Read {
                        tables.insert(t.name.clone());
                    }
                }
            }
            // the conditions deciding whether the step runs
            todo.extend(s.guards.iter().map(|g| (g.statement, String::new())));
            todo.extend(case_of.get(&id).map(|c| (*c, String::new())));
        }
    };

    let mut out: BTreeMap<String, m::Effect> = BTreeMap::new();
    let mut add = |name: &str, scope: m::DataScope, id: u32, via: Option<&m::BlockRef>| {
        let e = out.entry(name.to_string()).or_insert_with(|| m::Effect {
            name: name.to_string(),
            scope,
            statements: Vec::new(),
            inputs: Vec::new(),
            tables: Vec::new(),
            via: Vec::new(),
        });
        if !e.statements.contains(&id) {
            e.statements.push(id);
        }
        if let Some(v) = via {
            if !e.via.contains(v) {
                e.via.push(v.clone());
            }
        }
    };
    walk(body, &mut |s| {
        for w in &s.writes {
            // `sy-subrc` set by a statement, or `pnp-sw-found` by a macro of
            // SAP, are for the checks that follow
            let undeclared =
                w.implicit && matches!(w.scope, m::DataScope::Other | m::DataScope::System);
            if w.scope != m::DataScope::Local && !w.declared && !undeclared {
                add(&w.name, w.scope, s.id, w.via.as_ref());
            }
        }
        for t in &s.tables {
            if t.access == m::AccessKind::Write {
                let scope = match t.store {
                    Some(_) => m::DataScope::External,
                    None => m::DataScope::Database,
                };
                add(&t.name, scope, s.id, t.via.as_ref());
            }
        }
    });
    for e in out.values_mut() {
        let mut inputs = BTreeSet::new();
        let mut tables = BTreeSet::new();
        for s in &e.statements {
            inputs_of(*s, &e.name, &mut inputs, &mut tables);
        }
        e.inputs = inputs.into_iter().collect();
        e.tables = tables.into_iter().collect();
    }
    out.into_values().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const CODE: &str = r#"
CLASS lcl DEFINITION.
  PUBLIC SECTION.
    DATA mv_total TYPE i.
    METHODS calc IMPORTING iv_a TYPE i iv_mode TYPE c EXPORTING ev_result TYPE i.
ENDCLASS.
CLASS lcl IMPLEMENTATION.
  METHOD calc.
    DATA lv_x TYPE i.
    IF iv_a IS INITIAL.
      RETURN.
    ENDIF.
    lv_x = iv_a * 2.
    CASE iv_mode.
      WHEN 'A' OR 'B'.
        ev_result = lv_x.
      WHEN OTHERS.
        ev_result = 0.
    ENDCASE.
    LOOP AT mt_items INTO DATA(ls_item) WHERE active = abap_true.
      mv_total = mv_total + ls_item-amount.
    ENDLOOP.
  ENDMETHOD.
ENDCLASS.
"#;

    fn find<'a>(steps: &'a [m::Step], text: &str) -> &'a m::Step {
        fn go<'a>(steps: &'a [m::Step], text: &str) -> Option<&'a m::Step> {
            steps.iter().find_map(|s| {
                if s.text.starts_with(text) {
                    Some(s)
                } else {
                    go(&s.children, text)
                }
            })
        }
        go(steps, text).unwrap_or_else(|| panic!("no step {text}"))
    }

    #[test]
    fn procedure_logic() {
        let l = logic(CODE);
        assert_eq!(l.procedures.len(), 1);
        let p = &l.procedures[0];
        assert_eq!(p.block.name, "calc");
        assert_eq!(p.parameters.len(), 3);
        assert_eq!(p.locals, ["ls_item", "lv_x"]);

        // early exit: later steps run only when iv_a is not initial
        let x = find(&p.body, "lv_x = iv_a * 2");
        assert_eq!(x.guards.len(), 1);
        assert!(x.guards[0].negated);
        assert_eq!(x.guards[0].text, "iv_a IS INITIAL");
        assert_eq!(x.reads[0].scope, m::DataScope::Parameter);
        assert!(x.reads[0].from_entry);

        // WHEN conditions and the data flow from lv_x
        let when = find(&p.body, "WHEN 'A'");
        assert_eq!(
            when.condition.as_ref().unwrap().text,
            "iv_mode = 'A' OR iv_mode = 'B'"
        );
        let r = find(&p.body, "ev_result = lv_x");
        assert_eq!(r.guards.len(), 2);
        assert_eq!(r.reads[0].sources, [x.id]);
        assert!(!r.reads[0].from_entry);
        let others = find(&p.body, "ev_result = 0");
        assert!(others
            .guards
            .iter()
            .any(|g| g.negated && g.text.starts_with("iv_mode")));

        // the loop: attribute read from entry and from the previous pass
        let total = find(&p.body, "mv_total = mv_total");
        assert_eq!(total.writes[0].scope, m::DataScope::Attribute);
        let read = &total.reads[0];
        assert!(read.from_entry);
        assert_eq!(read.sources, [total.id]);
        assert_eq!(total.guards.last().unwrap().text, "active = abap_true");

        let effects: Vec<(&str, Vec<&str>)> = p
            .effects
            .iter()
            .map(|e| {
                (
                    e.name.as_str(),
                    e.inputs.iter().map(String::as_str).collect(),
                )
            })
            .collect();
        assert_eq!(
            effects,
            [
                ("ev_result", vec!["iv_a", "iv_mode"]),
                (
                    "mv_total",
                    vec!["abap_true", "iv_a", "mt_items", "mv_total"]
                ),
            ]
        );
    }

    #[test]
    fn control_flow() {
        let l = logic(CODE);
        let p = &l.procedures[0];
        let has = |from: Option<u32>, label: &str| {
            p.flow
                .iter()
                .any(|e| e.from == from && e.label.as_deref() == Some(label))
        };
        let loop_step = find(&p.body, "LOOP AT");
        assert!(has(Some(loop_step.id), "iterate"));
        assert!(has(Some(loop_step.id), "done"));
        let ret = find(&p.body, "RETURN");
        assert!(p
            .flow
            .iter()
            .any(|e| e.from == Some(ret.id) && e.to.is_none()));
        assert!(p.flow.iter().any(|e| e.from.is_none()));
    }

    #[test]
    fn check_and_else_guards() {
        let l = logic(
            "FORM f USING p TYPE i.\n  CHECK p > 0.\n  IF p = 1.\n    x = 1.\n  ELSEIF p = 2.\n    x = 2.\n  ELSE.\n    x = 3.\n  ENDIF.\nENDFORM.",
        );
        let p = &l.procedures[0];
        assert_eq!(p.parameters[0].name, "p");
        let third = find(&p.body, "x = 3");
        let texts: Vec<(bool, &str)> = third
            .guards
            .iter()
            .map(|g| (g.negated, g.text.as_str()))
            .collect();
        assert_eq!(texts, [(false, "p > 0"), (true, "p = 1"), (true, "p = 2")]);
        assert_eq!(p.effects[0].name, "x");
        assert_eq!(p.effects[0].inputs, ["p"]);
    }

    fn effect<'a>(p: &'a m::Procedure, name: &str) -> &'a m::Effect {
        p.effects
            .iter()
            .find(|e| e.name == name)
            .unwrap_or_else(|| panic!("no effect {name} in {:#?}", p.effects))
    }

    fn procedure<'a>(l: &'a m::FileLogic, name: &str) -> &'a m::Procedure {
        l.procedures
            .iter()
            .find(|p| p.block.name.eq_ignore_ascii_case(name))
            .unwrap_or_else(|| panic!("no procedure {name}"))
    }

    const ORDERS: &str = r#"
CLASS lcl DEFINITION.
  PUBLIC SECTION.
    METHODS total IMPORTING iv_vbeln TYPE vbeln EXPORTING ev_total TYPE netwr.
    METHODS write_log IMPORTING iv_text TYPE string.
    METHODS run.
  PRIVATE SECTION.
    DATA mv_count TYPE i.
    DATA mo_log TYPE REF TO lcl.
ENDCLASS.
CLASS lcl IMPLEMENTATION.
  METHOD total.
    SELECT SUM( netwr ) FROM vbap INTO @DATA(lv_sum) WHERE vbeln = @iv_vbeln.
    ev_total = lv_sum.
  ENDMETHOD.
  METHOD write_log.
    DATA ls_log TYPE zlog.
    ls_log-text = iv_text.
    INSERT zlog FROM @ls_log.
    mv_count = mv_count + 1.
  ENDMETHOD.
  METHOD run.
    total( EXPORTING iv_vbeln = '1' IMPORTING ev_total = DATA(lv_total) ).
    IF lv_total > 100.
      write_log( |big| ).
    ENDIF.
    mo_log->write_log( `x` ).
  ENDMETHOD.
ENDCLASS.
"#;

    #[test]
    fn database_tables_in_effects() {
        let l = logic(ORDERS);
        let total = procedure(&l, "total");
        let select = find(&total.body, "SELECT SUM");
        assert_eq!(
            select.tables,
            [m::TableAccess {
                name: "VBAP".into(),
                access: m::AccessKind::Read,
                operation: m::DbOperation::Select,
                store: None,
                via: None,
                depends: None,
            }]
        );
        let e = effect(total, "ev_total");
        assert_eq!(e.inputs, ["iv_vbeln"]);
        assert_eq!(e.tables, ["VBAP"]);

        let log = procedure(&l, "write_log");
        let e = effect(log, "ZLOG");
        assert_eq!(e.scope, m::DataScope::Database);
        assert_eq!(e.inputs, ["iv_text"]);
        assert!(e.via.is_empty());
    }

    #[test]
    fn calls_are_followed() {
        let l = logic(ORDERS);
        let run = procedure(&l, "run");
        // the table read by total decides whether log runs
        let zlog = effect(run, "ZLOG");
        assert_eq!(zlog.scope, m::DataScope::Database);
        assert_eq!(zlog.tables, ["VBAP"]);
        assert_eq!(zlog.via.len(), 1);
        assert_eq!(zlog.via[0].name, "write_log");
        let count = effect(run, "mv_count");
        assert_eq!(count.scope, m::DataScope::Attribute);
        assert_eq!(count.inputs, ["mv_count"]);
        assert_eq!(count.tables, ["VBAP"]);
        // the attribute of another object
        let other = effect(run, "mo_log->mv_count");
        assert_eq!(other.scope, m::DataScope::Attribute);
        assert!(other.inputs.contains(&"mo_log".to_string()));
        let call = find(&run.body, "write_log( |big| )");
        assert!(call
            .tables
            .iter()
            .any(|t| t.name == "ZLOG" && t.access == m::AccessKind::Write));
        let written = call.writes.iter().find(|w| !w.implicit).unwrap();
        assert_eq!(
            written.via.as_ref().map(|b| b.name.as_str()),
            Some("write_log")
        );
    }

    #[test]
    fn recursion_ends() {
        let l = logic(
            "FORM a USING p TYPE i.\n  gv = p.\n  PERFORM b USING p.\nENDFORM.\nFORM b USING p TYPE i.\n  PERFORM a USING p.\n  DELETE FROM zt WHERE id = p.\nENDFORM.",
        );
        let a = procedure(&l, "a");
        let names: Vec<&str> = a.effects.iter().map(|e| e.name.as_str()).collect();
        assert_eq!(names, ["ZT", "gv"]);
    }

    #[test]
    fn program_globals() {
        let l = logic(
            "REPORT z.\nDATA gv_total TYPE i.\nPARAMETERS p_n TYPE i.\nSTART-OF-SELECTION.\n  DATA lv_x TYPE i.\n  lv_x = p_n.\n  PERFORM add USING lv_x.\nFORM add USING p TYPE i.\n  gv_total = gv_total + p.\nENDFORM.",
        );
        let add = procedure(&l, "add");
        let e = effect(add, "gv_total");
        assert_eq!(e.scope, m::DataScope::Global);
        assert_eq!(e.inputs, ["gv_total", "p"]);
        let start = procedure(&l, "START-OF-SELECTION");
        // data of event blocks is global
        assert_eq!(effect(start, "lv_x").scope, m::DataScope::Global);
        let e = effect(start, "gv_total");
        assert_eq!(e.via[0].kind, "form");
        assert_eq!(e.inputs, ["gv_total", "p_n"]);
    }

    #[test]
    fn workspace_logic_across_includes_and_objects() {
        use abap_source::{MemoryProvider, SourceRef};
        let mut p = MemoryProvider::new();
        p.insert(
            SourceRef::program("ZREPORT"),
            "REPORT zreport.\nINCLUDE zreport_top.\nINCLUDE zreport_f01.\nSTART-OF-SELECTION.\n  PERFORM count.\n  zcl_counter=>inc( ).",
        );
        p.insert(SourceRef::program("ZREPORT_TOP"), "DATA gv_count TYPE i.");
        p.insert(
            SourceRef::program("ZREPORT_F01"),
            "FORM count.\n  gv_count = gv_count + 1.\n  CALL FUNCTION 'Z_SAVE' EXPORTING iv_n = gv_count.\nENDFORM.",
        );
        p.insert(
            SourceRef::function_module("Z_SAVE"),
            "FUNCTION z_save.\n*\"  IMPORTING\n*\"     VALUE(IV_N) TYPE  I\n  UPDATE ztotals SET n = @iv_n WHERE id = 1.\nENDFUNCTION.",
        );
        p.insert(
            SourceRef::class("ZCL_COUNTER"),
            "CLASS zcl_counter DEFINITION PUBLIC.\n  PUBLIC SECTION.\n    CLASS-METHODS inc.\n    CLASS-DATA gv_calls TYPE i.\nENDCLASS.\nCLASS zcl_counter IMPLEMENTATION.\n  METHOD inc.\n    gv_calls = gv_calls + 1.\n  ENDMETHOD.\nENDCLASS.",
        );
        let ws = Workspace::new(Box::new(p));
        let l = ws.logic(&SourceRef::program("ZREPORT"), 1);
        assert_eq!(l.object.as_deref(), Some("PROG:ZREPORT"));
        assert!(l.missing.is_empty(), "{:?}", l.missing);

        // the global of the TOP include, used in another include
        let count = procedure(&l, "count");
        assert_eq!(count.unit.as_deref(), Some("PROG:ZREPORT_F01"));
        assert_eq!(effect(count, "gv_count").scope, m::DataScope::Global);
        let save = effect(count, "ZTOTALS");
        assert_eq!(save.via[0].kind, "function_module");
        assert_eq!(save.inputs, ["gv_count"]);

        let start = procedure(&l, "START-OF-SELECTION");
        let names: Vec<&str> = start.effects.iter().map(|e| e.name.as_str()).collect();
        assert_eq!(names, ["ZTOTALS", "gv_count", "zcl_counter=>gv_calls"]);
        assert_eq!(effect(start, "ZTOTALS").via[0].name, "count");

        // without following other objects
        let l = ws.logic(&SourceRef::program("ZREPORT"), 0);
        let start = procedure(&l, "START-OF-SELECTION");
        let names: Vec<&str> = start.effects.iter().map(|e| e.name.as_str()).collect();
        assert_eq!(names, ["gv_count"]);
    }

    #[test]
    fn writes_through_field_symbols_and_references() {
        let l = logic(
            "CLASS lcl DEFINITION.
  PUBLIC SECTION.
    METHODS run.
    METHODS read_only.
  PRIVATE SECTION.
    DATA mt_items TYPE ty_items.
    DATA ms_head TYPE ty_head.
    DATA mv_x TYPE i.
ENDCLASS.
CLASS lcl IMPLEMENTATION.
  METHOD run.
    LOOP AT mt_items ASSIGNING FIELD-SYMBOL(<ls_item>).
      <ls_item>-amount = 0.
    ENDLOOP.
    ASSIGN COMPONENT 'TOTAL' OF STRUCTURE ms_head TO FIELD-SYMBOL(<lv_total>).
    <lv_total> = 1.
    GET REFERENCE OF mv_x INTO DATA(lr_x).
    lr_x->* = 5.
  ENDMETHOD.
  METHOD read_only.
    LOOP AT mt_items ASSIGNING FIELD-SYMBOL(<ls_item>).
      DATA(lv_sum) = <ls_item>-amount.
    ENDLOOP.
  ENDMETHOD.
ENDCLASS.",
        );
        let run = procedure(&l, "run");
        let names: Vec<&str> = run.effects.iter().map(|e| e.name.as_str()).collect();
        assert_eq!(names, ["ms_head", "mt_items", "mv_x"]);
        let write = find(&run.body, "<ls_item>-amount = 0");
        let w = write.writes.iter().find(|w| w.name == "mt_items").unwrap();
        assert_eq!(w.through.as_deref(), Some("<ls_item>"));
        assert_eq!(w.path, "<ls_item>-amount");
        // a write through a reference does not replace the reference
        let deref = find(&run.body, "lr_x->* = 5");
        assert!(deref.writes.iter().all(|w| w.name != "lr_x"));
        assert!(deref.reads.iter().any(|r| r.name == "lr_x"));
        // binding a field symbol to a table does not change it
        assert!(procedure(&l, "read_only").effects.is_empty());
    }

    #[test]
    fn interface_calls_reach_implementations() {
        let l = logic(
            "INTERFACE lif_store.
  METHODS save.
ENDINTERFACE.
CLASS lcl_db DEFINITION.
  PUBLIC SECTION.
    INTERFACES lif_store.
ENDCLASS.
CLASS lcl_db IMPLEMENTATION.
  METHOD lif_store~save.
    INSERT zlog FROM @( VALUE #( id = 1 ) ).
  ENDMETHOD.
ENDCLASS.
CLASS lcl_file DEFINITION.
  PUBLIC SECTION.
    INTERFACES lif_store.
    ALIASES save FOR lif_store~save.
  PRIVATE SECTION.
    DATA mv_written TYPE abap_bool.
ENDCLASS.
CLASS lcl_file IMPLEMENTATION.
  METHOD lif_store~save.
    mv_written = abap_true.
  ENDMETHOD.
ENDCLASS.
CLASS lcl_app DEFINITION.
  PUBLIC SECTION.
    METHODS run IMPORTING io_store TYPE REF TO lif_store io_file TYPE REF TO lcl_file.
ENDCLASS.
CLASS lcl_app IMPLEMENTATION.
  METHOD run.
    io_store->save( ).
    io_file->save( ).
  ENDMETHOD.
ENDCLASS.",
        );
        let run = procedure(&l, "run");
        let call = find(&run.body, "io_store->save");
        assert!(call.tables.iter().any(|t| t.name == "ZLOG"
            && t.via.as_ref().and_then(|b| b.class.as_deref()) == Some("lcl_db")));
        let names: Vec<&str> = run.effects.iter().map(|e| e.name.as_str()).collect();
        assert_eq!(
            names,
            ["ZLOG", "io_file->mv_written", "io_store->mv_written"]
        );
        // the alias of lcl_file leads to its implementation of the interface
        assert_eq!(
            effect(run, "io_file->mv_written").via[0].name,
            "lif_store~save"
        );
    }

    #[test]
    fn data_kept_outside_the_program() {
        let l = logic(
            "REPORT z.
FORM put.
  SELECT * FROM zsrc INTO TABLE @DATA(lt_rows).
  EXPORT rows = lt_rows TO MEMORY ID 'ZROWS'.
  SET PARAMETER ID 'ZID' FIELD lt_rows[ 1 ]-id.
ENDFORM.
FORM take.
  DATA lt_rows TYPE STANDARD TABLE OF zsrc.
  IMPORT rows = lt_rows FROM MEMORY ID 'ZROWS'.
  INSERT ztgt FROM TABLE @lt_rows.
  TRANSFER lt_rows[ 1 ]-id TO '/tmp/ids.txt'.
ENDFORM.",
        );
        let put = procedure(&l, "put");
        let e = effect(put, "memory:ZROWS");
        assert_eq!(e.scope, m::DataScope::External);
        assert_eq!(e.tables, ["ZSRC"]);
        assert_eq!(effect(put, "parameter:ZID").tables, ["ZSRC"]);
        let take = procedure(&l, "take");
        assert_eq!(effect(take, "ZTGT").tables, ["memory:ZROWS"]);
        assert_eq!(effect(take, "file:/tmp/ids.txt").tables, ["memory:ZROWS"]);
        assert_eq!(
            crate::text::logic(&l)
                .lines()
                .find(|l| l.contains("writes ZTGT")),
            Some("  writes ZTGT (database table) at 10; from tables memory:ZROWS")
        );
    }

    #[test]
    fn guards_know_which_statement_set_sy_subrc() {
        let l = logic(
            "FORM f USING it_tab TYPE ty_tab.
  READ TABLE it_tab INTO DATA(ls) INDEX 1.
  IF sy-subrc = 0.
    CALL FUNCTION 'Z_X' EXCEPTIONS failed = 1.
    IF syst-subrc NE 0.
      gv_err = abap_true.
    ENDIF.
  ENDIF.
  PERFORM other.
  CASE sy-subrc.
    WHEN 4.
      gv_four = abap_true.
  ENDCASE.
ENDFORM.",
        );
        let f = procedure(&l, "f");
        let err = find(&f.body, "gv_err = abap_true.");
        let set: Vec<(&str, &[u32])> = err
            .guards
            .iter()
            .map(|g| (g.text.as_str(), g.subrc.as_slice()))
            .collect();
        assert_eq!(
            set,
            [("sy-subrc = 0", &[1][..]), ("syst-subrc NE 0", &[3][..])]
        );
        // a form may set it or leave it: from READ TABLE when the IF is
        // not taken, from the call when it is
        let four = find(&f.body, "gv_four = abap_true.");
        assert_eq!(four.guards[0].subrc, [1, 3, 8]);
        // what READ TABLE found decides the write
        assert_eq!(effect(f, "gv_err").inputs, ["abap_true", "it_tab"]);
        // sy-subrc itself is no effect
        assert!(f.effects.iter().all(|e| !e.name.starts_with("sy")));
        assert!(crate::text::logic(&l)
            .contains("when sy-subrc = 0 [sy-subrc of 2] AND syst-subrc NE 0 [sy-subrc of 4]"));
    }

    #[test]
    fn system_fields_have_types_and_the_statements_setting_them() {
        let l = logic(
            "FORM f USING it_tab TYPE ty_tab.
  gv_date = sy-datum.
  READ TABLE it_tab INTO DATA(ls) WITH KEY id = 1.
  gv_line = sy-tabix.
  SELECT * FROM zt INTO TABLE @DATA(lt).
  gv_count = sy-dbcnt.
  DO 3 TIMES.
    gv_round = sy-index.
  ENDDO.
  IF gv_text CA ' '.
    gv_pos = sy-fdpos.
  ENDIF.
  MESSAGE e001(zm) INTO DATA(lv_msg).
  gv_class = sy-msgid.
ENDFORM.",
        );
        let f = procedure(&l, "f");
        let read = |text: &str| {
            find(&f.body, text)
                .reads
                .iter()
                .find(|r| r.name.starts_with("sy-"))
                .unwrap()
                .clone()
        };
        // set by the runtime, with its type
        let datum = read("gv_date = sy-datum.");
        assert_eq!(datum.scope, m::DataScope::System);
        assert!(datum.from_entry);
        let t = datum.data_type.unwrap();
        assert_eq!(
            (t.kind.as_str(), t.data_element.as_deref()),
            ("d", Some("SYST_DATUM"))
        );
        // from the runtime: no input of the procedure
        assert!(effect(f, "gv_date").inputs.is_empty());
        // set by statements: from them
        assert_eq!(read("gv_line = sy-tabix.").sources, [2]);
        assert_eq!(effect(f, "gv_line").inputs, ["it_tab"]);
        assert_eq!(read("gv_count = sy-dbcnt.").sources, [4]);
        assert_eq!(effect(f, "gv_count").tables, ["ZT"]);
        let index = read("gv_round = sy-index.");
        assert_eq!(
            (index.sources.as_slice(), index.from_entry),
            (&[6][..], false)
        );
        assert_eq!(read("gv_pos = sy-fdpos.").sources, [9]);
        let msgid = read("gv_class = sy-msgid.");
        assert_eq!(msgid.sources, [12]);
        assert_eq!(msgid.data_type.unwrap().length, Some(20));
        // no system field is an effect
        assert!(f.effects.iter().all(|e| !e.name.starts_with("sy-")));
    }

    #[test]
    fn data_clusters_in_tables() {
        let l = logic(
            "REPORT z.
FORM put.
  SELECT * FROM zsrc INTO TABLE @DATA(lt_rows).
  EXPORT rows = lt_rows TO DATABASE indx(zr) ID 'K1'.
ENDFORM.
FORM take.
  DATA lt_rows TYPE STANDARD TABLE OF zsrc.
  IMPORT rows = lt_rows FROM DATABASE indx(zr) ID 'K1'.
  INSERT ztgt FROM TABLE @lt_rows.
ENDFORM.",
        );
        assert_eq!(effect(procedure(&l, "put"), "INDX").tables, ["ZSRC"]);
        assert_eq!(effect(procedure(&l, "take"), "ZTGT").tables, ["INDX"]);
        let db = crate::db_access("IMPORT rows = lt_rows FROM DATABASE indx(zr) ID 'K1'.");
        assert_eq!(db[0].cluster.as_ref().unwrap().area, "zr");
    }

    #[test]
    fn lock_modules_set_and_release_locks() {
        let l = logic(
            "REPORT z.
FORM f.
  CALL FUNCTION 'ENQUEUE_EZ_ORDER' EXPORTING vbeln = lv_vbeln EXCEPTIONS foreign_lock = 1.
  CHECK sy-subrc = 0.
  UPDATE zorder SET status = 'X' WHERE vbeln = lv_vbeln.
  CALL FUNCTION 'DEQUEUE_EZ_ORDER' EXPORTING vbeln = lv_vbeln.
ENDFORM.
FUNCTION enqueue_ez_order.
  INSERT zlocks FROM ls_lock.
ENDFUNCTION.",
        );
        let f = procedure(&l, "f");
        let lock = effect(f, "lock:EZ_ORDER");
        assert_eq!(lock.scope, m::DataScope::External);
        assert_eq!(lock.statements, [2, 5]);
        // the generated module is not followed: no table write from it
        assert!(f.effects.iter().all(|e| e.name != "ZLOCKS"));
        let step = find(
            &f.body,
            "CALL FUNCTION 'DEQUEUE_EZ_ORDER' EXPORTING vbeln = lv_vbeln.",
        );
        assert_eq!(step.tables[0].store, Some(m::Store::Lock));
        assert_eq!(step.tables[0].operation, m::DbOperation::Delete);
    }

    #[test]
    fn hr_infotypes() {
        let l = logic(
            "REPORT zhr.
TABLES pernr.
INFOTYPES: 0001, 0002, 0105 MODE N.
DATA gv_name TYPE string.
DATA gt_0185 TYPE TABLE OF p0185.
GET pernr.
  rp-provide-from-last p0002 space pn-begda pn-endda.
  IF pnp-sw-found = '1'.
    gv_name = p0002-nachn.
  ENDIF.
  rp-read-infotype pernr-pernr 0105 p0105 pn-begda pn-endda.
  CALL FUNCTION 'HR_READ_INFOTYPE'
    EXPORTING
      pernr     = pernr-pernr
      infty     = '0185'
    TABLES
      infty_tab = gt_0185.",
        );
        let get = &l.procedures[0];
        assert_eq!(get.block.name, "GET pernr");
        // the logical database fills each table from its own
        assert_eq!(effect(get, "p0001").tables, ["PA0001"]);
        assert_eq!(effect(get, "p0001").scope, m::DataScope::Global);
        assert_eq!(effect(get, "gv_name").tables, ["PA0002"]);
        assert_eq!(effect(get, "p0105").tables, ["PA0105"]);
        assert_eq!(effect(get, "gt_0185").tables, ["PA0185"]);
        // MODE N: read by the program, not at GET
        assert!(get.body[0].tables.iter().all(|t| t.name != "PA0105"));
        // pnp-sw-found decides, but is no effect
        let name = find(&get.body, "gv_name = p0002-nachn.");
        let found = find(&get.body, "IF pnp-sw-found = '1'.");
        assert_eq!(found.reads[0].sources, [name.id - 2]);
        assert!(get.effects.iter().all(|e| e.name != "pnp"));
    }

    #[test]
    fn submit_with_clauses() {
        let p = submit_parameters(
            "SUBMIT zpost WITH p_id = @lv_id WITH s_date BETWEEN lv_from AND lv_to \
             WITH SELECTION-TABLE lt_sel VIA SELECTION-SCREEN AND RETURN.",
        );
        let got: Vec<(&str, Vec<&str>)> = p
            .iter()
            .map(|(n, o)| (n.as_str(), o.iter().map(String::as_str).collect()))
            .collect();
        assert_eq!(
            got,
            [
                ("p_id", vec!["=", "lv_id"]),
                ("s_date", vec!["between", "lv_from", "and", "lv_to"]),
                ("", vec!["lt_sel"]),
            ]
        );
    }

    #[test]
    fn calls_reach_redefinitions() {
        let l = logic(
            "CLASS lcl_base DEFINITION.
  PUBLIC SECTION.
    METHODS save.
    CLASS-METHODS create RETURNING VALUE(ro) TYPE REF TO lcl_base.
  PROTECTED SECTION.
    DATA mv_saved TYPE abap_bool.
ENDCLASS.
CLASS lcl_base IMPLEMENTATION.
  METHOD save.
    mv_saved = abap_true.
  ENDMETHOD.
  METHOD create.
  ENDMETHOD.
ENDCLASS.
CLASS lcl_db DEFINITION INHERITING FROM lcl_base.
  PUBLIC SECTION.
    METHODS save REDEFINITION.
ENDCLASS.
CLASS lcl_db IMPLEMENTATION.
  METHOD save.
    super->save( ).
    INSERT zlog FROM @( VALUE #( id = 1 ) ).
  ENDMETHOD.
ENDCLASS.
CLASS lcl_db_audit DEFINITION INHERITING FROM lcl_db.
  PUBLIC SECTION.
    METHODS save REDEFINITION.
ENDCLASS.
CLASS lcl_db_audit IMPLEMENTATION.
  METHOD save.
    INSERT zaudit FROM @( VALUE #( id = 1 ) ).
  ENDMETHOD.
ENDCLASS.
CLASS lcl_app DEFINITION.
  PUBLIC SECTION.
    METHODS run IMPORTING io TYPE REF TO lcl_base.
ENDCLASS.
CLASS lcl_app IMPLEMENTATION.
  METHOD run.
    io->save( ).
  ENDMETHOD.
ENDCLASS.",
        );
        // the object may be of any subclass: the redefinitions may run
        let run = procedure(&l, "run");
        let names: Vec<&str> = run.effects.iter().map(|e| e.name.as_str()).collect();
        assert_eq!(names, ["ZAUDIT", "ZLOG", "io->mv_saved"]);
        let via: Vec<&str> = effect(run, "ZAUDIT")
            .via
            .iter()
            .filter_map(|b| b.class.as_deref())
            .collect();
        assert_eq!(via, ["lcl_db_audit"]);
        // super->save( ) runs the implementation of the superclass only
        let db = l
            .procedures
            .iter()
            .find(|p| p.block.class.as_deref() == Some("lcl_db"))
            .unwrap();
        let names: Vec<&str> = db.effects.iter().map(|e| e.name.as_str()).collect();
        assert_eq!(names, ["ZLOG", "mv_saved"]);
    }

    #[test]
    fn values_written_through_calls_keep_their_own_dependencies() {
        let l = logic(
            "CLASS lcl DEFINITION.
  PUBLIC SECTION.
    METHODS load IMPORTING iv_id TYPE i.
    METHODS run.
  PRIVATE SECTION.
    DATA mv_net TYPE p.
    DATA mv_name TYPE string.
    DATA mv_mode TYPE c.
ENDCLASS.
CLASS lcl IMPLEMENTATION.
  METHOD load.
    SELECT SINGLE netwr FROM vbap WHERE vbeln = @iv_id INTO @mv_net.
    SELECT SINGLE name1 FROM kna1 WHERE kunnr = @mv_mode INTO @mv_name.
  ENDMETHOD.
  METHOD run.
    load( 42 ).
  ENDMETHOD.
ENDCLASS.",
        );
        let run = procedure(&l, "run");
        let net = effect(run, "mv_net");
        assert_eq!(net.tables, ["VBAP"]);
        assert!(net.inputs.is_empty(), "{:?}", net.inputs);
        let name = effect(run, "mv_name");
        assert_eq!(name.tables, ["KNA1"]);
        assert_eq!(name.inputs, ["mv_mode"]);
        let w = find(&run.body, "load( 42 )")
            .writes
            .iter()
            .find(|w| w.name == "mv_net")
            .unwrap();
        // the argument is a literal: nothing of the caller
        let d = w.depends.as_ref().unwrap();
        assert!(!d.arguments && d.reads.is_empty());
        assert_eq!(d.tables, ["VBAP"]);
    }

    fn inputs<'a>(p: &'a m::Procedure, name: &str) -> Vec<&'a str> {
        effect(p, name).inputs.iter().map(String::as_str).collect()
    }

    #[test]
    fn parameters_are_matched_to_arguments() {
        let l = logic(
            "CLASS lcl DEFINITION.
  PUBLIC SECTION.
    METHODS calc IMPORTING ia TYPE i ib TYPE i EXPORTING ea TYPE i RETURNING VALUE(rb) TYPE i.
    METHODS run IMPORTING iv_x TYPE i iv_y TYPE i iv_z TYPE i.
  PRIVATE SECTION.
    DATA mv_a TYPE i.
    DATA mv_b TYPE i.
ENDCLASS.
CLASS lcl IMPLEMENTATION.
  METHOD calc.
    ea = ia * 2.
    rb = ib.
  ENDMETHOD.
  METHOD run.
    calc( EXPORTING ia = iv_x ib = iv_y IMPORTING ea = mv_a ).
    mv_b = calc( ia = iv_z ib = iv_y ) + 1.
  ENDMETHOD.
ENDCLASS.",
        );
        let run = procedure(&l, "run");
        assert_eq!(inputs(run, "mv_a"), ["iv_x"]);
        assert_eq!(inputs(run, "mv_b"), ["iv_y"]);

        let l = logic(
            "REPORT z.
DATA: gv_a TYPE i, gv_b TYPE i, gv_out TYPE i.
START-OF-SELECTION.
  PERFORM f USING gv_a gv_b CHANGING gv_out.
FORM f USING p1 TYPE i p2 TYPE i CHANGING c TYPE i.
  c = p2.
ENDFORM.",
        );
        let start = procedure(&l, "START-OF-SELECTION");
        assert_eq!(inputs(start, "gv_out"), ["gv_b"]);
    }

    #[test]
    fn field_symbols_follow_the_control_flow() {
        let l = logic(
            "FORM f.
  ASSIGN gv_a TO FIELD-SYMBOL(<x>).
  <x> = 1.
  ASSIGN gv_b TO <x>.
  <x> = 2.
ENDFORM.",
        );
        let f = procedure(&l, "f");
        let written = |text: &str| -> Vec<String> {
            find(&f.body, text)
                .writes
                .iter()
                .filter(|w| w.through.is_some())
                .map(|w| w.name.clone())
                .collect()
        };
        assert_eq!(written("<x> = 1"), ["gv_a"]);
        assert_eq!(written("<x> = 2"), ["gv_b"]);
    }

    #[test]
    fn references_across_calls() {
        let l = logic(
            "CLASS lcl DEFINITION.
  PUBLIC SECTION.
    METHODS get_ref RETURNING VALUE(rr) TYPE REF TO i.
    METHODS set_via IMPORTING ir TYPE REF TO i.
    METHODS touch IMPORTING io TYPE REF TO lcl.
    METHODS run.
    DATA mv_a TYPE i.
    DATA mv_c TYPE i.
    DATA mo_other TYPE REF TO lcl.
ENDCLASS.
CLASS lcl IMPLEMENTATION.
  METHOD get_ref.
    rr = REF #( mv_a ).
  ENDMETHOD.
  METHOD set_via.
    ir->* = 5.
  ENDMETHOD.
  METHOD touch.
    io->mv_a = 1.
  ENDMETHOD.
  METHOD run.
    DATA(lr) = get_ref( ).
    lr->* = 3.
    GET REFERENCE OF mv_c INTO DATA(lr_c).
    set_via( lr_c ).
    touch( mo_other ).
  ENDMETHOD.
ENDCLASS.",
        );
        let run = procedure(&l, "run");
        // the reference returned points to mv_a
        let w = find(&run.body, "lr->* = 3");
        assert!(w
            .writes
            .iter()
            .any(|x| x.name == "mv_a" && x.through.as_deref() == Some("lr")));
        // written through the reference passed
        let set = find(&run.body, "set_via( lr_c )");
        assert!(
            set.writes.iter().any(|x| x.name == "mv_c"),
            "{:?}",
            set.writes
        );
        // an attribute of the object passed
        let names: Vec<&str> = run.effects.iter().map(|e| e.name.as_str()).collect();
        assert_eq!(names, ["mo_other->mv_a", "mv_a", "mv_c"]);
        // in the called method, the parameter is what is written
        let set_via = procedure(&l, "set_via");
        assert_eq!(set_via.effects[0].name, "ir");
        assert_eq!(set_via.effects[0].scope, m::DataScope::Parameter);
    }

    #[test]
    fn dynamic_calls_with_known_names() {
        let l = logic(
            "CLASS lcl DEFINITION.
  PUBLIC SECTION.
    METHODS run IMPORTING iv_mode TYPE c.
    METHODS set_a.
    METHODS set_b.
    METHODS set_c.
    CONSTANTS c_meth TYPE string VALUE 'SET_C'.
  PRIVATE SECTION.
    DATA: mv_a TYPE i, mv_b TYPE i, mv_c TYPE i.
ENDCLASS.
CLASS lcl IMPLEMENTATION.
  METHOD run.
    DATA lv_meth TYPE string.
    IF iv_mode = 'X'.
      lv_meth = 'SET_A'.
    ELSE.
      lv_meth = 'SET_B'.
    ENDIF.
    CALL METHOD me->(lv_meth).
    CALL METHOD (c_meth).
  ENDMETHOD.
  METHOD set_a.
    mv_a = 1.
  ENDMETHOD.
  METHOD set_b.
    mv_b = 1.
  ENDMETHOD.
  METHOD set_c.
    mv_c = 1.
  ENDMETHOD.
ENDCLASS.",
        );
        let run = procedure(&l, "run");
        let names: Vec<&str> = run.effects.iter().map(|e| e.name.as_str()).collect();
        assert_eq!(names, ["mv_a", "mv_b", "mv_c"]);
        // which method runs depends on the mode
        assert_eq!(inputs(run, "mv_a"), ["iv_mode"]);

        let l = logic(
            "FUNCTION z_main.
  DATA lv_fm TYPE string VALUE 'Z_ONE'.
  CALL FUNCTION lv_fm.
  lv_fm = 'Z_TWO'.
  CALL FUNCTION lv_fm.
ENDFUNCTION.
FUNCTION z_one.
  DELETE FROM zone WHERE id = 1.
ENDFUNCTION.
FUNCTION z_two.
  DELETE FROM ztwo WHERE id = 1.
ENDFUNCTION.",
        );
        let main = procedure(&l, "z_main");
        let tables = |id: usize| -> Vec<String> {
            let calls: Vec<&m::Step> = main
                .body
                .iter()
                .filter(|s| s.kind == "CallFunction")
                .collect();
            calls[id].tables.iter().map(|t| t.name.clone()).collect()
        };
        assert_eq!(tables(0), ["ZONE"]);
        assert_eq!(tables(1), ["ZTWO"]);
    }
}
