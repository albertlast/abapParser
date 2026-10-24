//! Compact text forms of the analyses, for language models and people:
//! definitions written like ABAP, one line per call, effect or access,
//! rows where to find them. Much shorter than the JSON, which stays the
//! complete form.

use std::fmt::Write;

use abap_model as m;

/// Classes, interfaces, routines and global declarations, written like
/// their ABAP definitions; `" 12` gives the row, `" 12-40` a range.
pub fn symbols(s: &m::FileSymbols) -> String {
    let mut out = String::new();
    if let Some(p) = &s.program {
        line(
            &mut out,
            0,
            &format!("{} {}.", p.kind.to_uppercase(), p.name),
            Some(&p.span),
        );
    }
    for i in &s.includes {
        line(&mut out, 0, &format!("INCLUDE {}.", i.name), Some(&i.span));
    }
    for d in &s.declarations {
        line(&mut out, 0, &declaration(d), Some(&d.span));
    }
    for i in &s.interfaces {
        interface(&mut out, i);
    }
    for c in &s.classes {
        class(&mut out, c);
    }
    for o in &s.orphan_implementations {
        let _ = writeln!(out, "CLASS {} IMPLEMENTATION.", o.class);
        implemented(&mut out, &o.implementation);
    }
    for f in &s.forms {
        routine(&mut out, "FORM", f);
    }
    for f in &s.function_modules {
        routine(&mut out, "FUNCTION", f);
    }
    for mo in &s.modules {
        let kind = mo.kind.as_deref().map(|k| format!(" {}", k.to_uppercase()));
        line(
            &mut out,
            0,
            &format!("MODULE {}{}.", mo.name, kind.unwrap_or_default()),
            Some(&mo.span),
        );
    }
    for e in &s.event_blocks {
        line(&mut out, 0, &format!("{}.", e.event), Some(&e.span));
    }
    for mac in &s.macros {
        line(
            &mut out,
            0,
            &format!("DEFINE {}.", mac.name),
            Some(&mac.span),
        );
    }
    out
}

fn class(out: &mut String, c: &m::Class) {
    let mut head = format!("CLASS {} DEFINITION", c.name);
    if c.is_global {
        head.push_str(" PUBLIC");
    }
    if let Some(s) = &c.superclass {
        head.push_str(&format!(" INHERITING FROM {s}"));
    }
    if c.is_abstract {
        head.push_str(" ABSTRACT");
    }
    if c.is_final {
        head.push_str(" FINAL");
    }
    if let Some(cr) = &c.create {
        head.push_str(&format!(" CREATE {}", cr.to_uppercase()));
    }
    if c.is_for_testing {
        head.push_str(" FOR TESTING");
    }
    if let Some(b) = &c.behavior_of {
        head.push_str(&format!(" FOR BEHAVIOR OF {b}"));
    }
    if !c.friends.is_empty() {
        head.push_str(&format!(" FRIENDS {}", c.friends.join(" ")));
    }
    head.push('.');
    line(out, 0, &head, Some(&c.span));
    for v in [
        m::Visibility::Public,
        m::Visibility::Protected,
        m::Visibility::Private,
    ] {
        let mut section = String::new();
        members(
            &mut section,
            4,
            v,
            &c.interfaces,
            &c.aliases,
            &c.types,
            &c.attributes,
            &c.events,
            &c.methods,
        );
        if !section.is_empty() {
            let _ = writeln!(out, "  {}.", section_name(v));
            out.push_str(&section);
        }
    }
    let _ = writeln!(out, "ENDCLASS.");
    if let Some(imp) = &c.implementation {
        implemented(out, imp);
    }
}

fn interface(out: &mut String, i: &m::Interface) {
    let head = format!(
        "INTERFACE {}{}.",
        i.name,
        if i.is_global { " PUBLIC" } else { "" }
    );
    line(out, 0, &head, Some(&i.span));
    members(
        out,
        2,
        m::Visibility::Public,
        &i.interfaces,
        &i.aliases,
        &i.types,
        &i.attributes,
        &i.events,
        &i.methods,
    );
    let _ = writeln!(out, "ENDINTERFACE.");
}

#[allow(clippy::too_many_arguments)]
fn members(
    out: &mut String,
    indent: usize,
    v: m::Visibility,
    interfaces: &[m::ImplementedInterface],
    aliases: &[m::Alias],
    types: &[m::Declaration],
    attributes: &[m::Declaration],
    events: &[m::Event],
    methods: &[m::Method],
) {
    for i in interfaces.iter().filter(|i| i.visibility == v) {
        let mut text = format!("INTERFACES {}", i.name);
        if i.partially_implemented {
            text.push_str(" PARTIALLY IMPLEMENTED");
        }
        if i.all_abstract {
            text.push_str(" ALL METHODS ABSTRACT");
        } else if !i.abstract_methods.is_empty() {
            text.push_str(&format!(
                " ABSTRACT METHODS {}",
                i.abstract_methods.join(" ")
            ));
        }
        if i.all_final {
            text.push_str(" ALL METHODS FINAL");
        } else if !i.final_methods.is_empty() {
            text.push_str(&format!(" FINAL METHODS {}", i.final_methods.join(" ")));
        }
        text.push('.');
        line(out, indent, &text, None);
    }
    for a in aliases.iter().filter(|a| a.visibility == v) {
        line(
            out,
            indent,
            &format!("ALIASES {} FOR {}.", a.name, a.component),
            Some(&a.span),
        );
    }
    for d in types
        .iter()
        .chain(attributes)
        .filter(|d| d.visibility.unwrap_or(m::Visibility::Public) == v)
    {
        line(out, indent, &declaration(d), Some(&d.span));
    }
    for e in events.iter().filter(|e| e.visibility == v) {
        let keyword = if e.is_static {
            "CLASS-EVENTS"
        } else {
            "EVENTS"
        };
        let text = format!("{keyword} {}{}.", e.name, parameters(&e.parameters));
        line(out, indent, &text, Some(&e.span));
    }
    for me in methods.iter().filter(|me| me.visibility == v) {
        line(out, indent, &method(me), Some(&me.span));
    }
}

fn method(me: &m::Method) -> String {
    let mut text = format!(
        "{} {}",
        if me.is_static {
            "CLASS-METHODS"
        } else {
            "METHODS"
        },
        me.name
    );
    if me.is_abstract {
        text.push_str(" ABSTRACT");
    }
    if me.is_final {
        text.push_str(" FINAL");
    }
    if let Some(d) = &me.default {
        text.push_str(&format!(" DEFAULT {}", d.to_uppercase()));
    }
    if me.is_for_testing {
        text.push_str(" FOR TESTING");
    }
    if let Some(h) = &me.event_handler {
        text.push_str(&format!(" FOR EVENT {} OF {}", h.event, h.class));
    }
    if let Some(r) = &me.rap_handler {
        text.push_str(&format!(" FOR {r}"));
    }
    if me.is_redefinition {
        text.push_str(" REDEFINITION");
    }
    text.push_str(&parameters(&me.parameters));
    if !me.raising.is_empty() {
        text.push_str(&format!(" RAISING {}", me.raising.join(" ")));
    }
    if !me.exceptions.is_empty() {
        text.push_str(&format!(" EXCEPTIONS {}", me.exceptions.join(" ")));
    }
    text.push('.');
    text
}

fn routine(out: &mut String, keyword: &str, r: &m::Routine) {
    let mut text = format!("{keyword} {}{}", r.name, parameters(&r.parameters));
    if !r.raising.is_empty() {
        text.push_str(&format!(" RAISING {}", r.raising.join(" ")));
    }
    if !r.exceptions.is_empty() {
        text.push_str(&format!(" EXCEPTIONS {}", r.exceptions.join(" ")));
    }
    text.push('.');
    line(out, 0, &text, Some(&r.span));
}

/// Methods implemented, with their rows, for reading their code.
fn implemented(out: &mut String, imp: &m::ClassImplementation) {
    if imp.methods.is_empty() {
        return;
    }
    let methods: Vec<String> = imp
        .methods
        .iter()
        .map(|me| {
            let amdp = if me.is_amdp { " (AMDP)" } else { "" };
            format!("{}{amdp} {}", me.name, rows(&me.span))
        })
        .collect();
    let _ = writeln!(out, "\" implemented: {}", methods.join(", "));
}

/// Parameters grouped by direction in ABAP order.
fn parameters(ps: &[m::Parameter]) -> String {
    let mut out = String::new();
    for (direction, keyword) in [
        (m::Direction::Importing, "IMPORTING"),
        (m::Direction::Using, "USING"),
        (m::Direction::Exporting, "EXPORTING"),
        (m::Direction::Tables, "TABLES"),
        (m::Direction::Changing, "CHANGING"),
        (m::Direction::Returning, "RETURNING"),
    ] {
        let of: Vec<String> = ps
            .iter()
            .filter(|p| p.direction == direction)
            .map(parameter)
            .collect();
        if !of.is_empty() {
            out.push_str(&format!(" {keyword} {}", of.join(" ")));
        }
    }
    out
}

fn parameter(p: &m::Parameter) -> String {
    let mut text = if p.by_value {
        format!("VALUE({})", p.name)
    } else {
        p.name.clone()
    };
    if let Some(t) = &p.typing {
        text.push(' ');
        text.push_str(&squeeze(&t.text));
    }
    // a default makes it optional, both together are no ABAP
    if p.optional && p.default.is_none() {
        text.push_str(" OPTIONAL");
    }
    if let Some(d) = &p.default {
        text.push_str(&format!(" DEFAULT {d}"));
    }
    if p.preferred {
        text.push_str(" PREFERRED PARAMETER");
    }
    text
}

fn declaration(d: &m::Declaration) -> String {
    let keyword = match d.kind {
        m::DeclarationKind::Data => "DATA",
        m::DeclarationKind::ClassData => "CLASS-DATA",
        m::DeclarationKind::Static => "STATICS",
        m::DeclarationKind::Constant => "CONSTANTS",
        m::DeclarationKind::Type => "TYPES",
        m::DeclarationKind::FieldSymbol => "FIELD-SYMBOLS",
        m::DeclarationKind::Parameter => "PARAMETERS",
        m::DeclarationKind::SelectOption => "SELECT-OPTIONS",
        m::DeclarationKind::Range => "RANGES",
        m::DeclarationKind::Tables => "TABLES",
        m::DeclarationKind::Node => "NODES",
        m::DeclarationKind::Infotype => "INFOTYPES",
        m::DeclarationKind::Include => "INCLUDE",
        m::DeclarationKind::EnumValue => "VALUE",
    };
    // the clause as written names the infotype, not the table
    if d.kind == m::DeclarationKind::Infotype {
        let clause = d
            .typing
            .as_ref()
            .map(|t| squeeze(&t.text))
            .unwrap_or_default();
        return format!("{keyword} {clause}.");
    }
    format!("{keyword} {}.", declared(d))
}

/// Name and typing of a declaration; structures with their components.
fn declared(d: &m::Declaration) -> String {
    if d.is_structure || d.is_enum || !d.components.is_empty() {
        let begin = if d.is_enum {
            "BEGIN OF ENUM"
        } else {
            "BEGIN OF"
        };
        let components: Vec<String> = d.components.iter().map(declared).collect();
        let mut text = format!("{begin} {}", d.name);
        if !components.is_empty() {
            text.push_str(&format!(", {}", components.join(", ")));
        }
        text.push_str(&format!(", END OF {}", d.name));
        return text;
    }
    let mut text = d.name.clone();
    if let Some(t) = &d.typing {
        text.push(' ');
        text.push_str(&squeeze(&t.text));
    }
    if let Some(v) = &d.value {
        text.push_str(&format!(" VALUE {v}"));
    }
    if d.read_only && d.kind != m::DeclarationKind::Constant {
        text.push_str(" READ-ONLY");
    }
    text
}

fn section_name(v: m::Visibility) -> &'static str {
    match v {
        m::Visibility::Public => "PUBLIC SECTION",
        m::Visibility::Protected => "PROTECTED SECTION",
        m::Visibility::Private => "PRIVATE SECTION",
    }
}

fn line(out: &mut String, indent: usize, text: &str, span: Option<&m::Span>) {
    let _ = write!(out, "{:indent$}{text}", "", indent = indent);
    if let Some(s) = span {
        let _ = write!(out, "  \" {}", rows(s));
    }
    out.push('\n');
}

fn rows(s: &m::Span) -> String {
    if s.start.row == s.end.row {
        s.start.row.to_string()
    } else {
        format!("{}-{}", s.start.row, s.end.row)
    }
}

/// Runs of whitespace as one space, for statements spread over lines.
fn squeeze(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// A CDS definition: its sources and associations, then each element with
/// the fields it is made of.
pub fn cds(d: &m::CdsDefinition) -> String {
    let mut out = String::new();
    let mut head = format!("{} {}", d.kind.to_uppercase(), d.name);
    if let Some(v) = &d.sql_view {
        head.push_str(&format!(" (SQL view {v})"));
    }
    if let Some(l) = &d.label {
        head.push_str(&format!(" '{l}'"));
    }
    line(&mut out, 0, &head, Some(&d.span));
    if !d.parameters.is_empty() {
        let p: Vec<String> = d
            .parameters
            .iter()
            .map(|p| format!("{} : {}", p.name, p.typing))
            .collect();
        let _ = writeln!(out, "  parameters: {}", p.join(", "));
    }
    for s in &d.sources {
        let _ = write!(out, "  {} {}", s.role, s.name);
        if let Some(a) = &s.alias {
            let _ = write!(out, " as {a}");
        }
        if let Some(on) = &s.on {
            let _ = write!(out, " on {on}");
        }
        out.push('\n');
    }
    for a in &d.associations {
        let _ = write!(
            out,
            "  {} {}{} to {}",
            a.kind,
            a.name,
            a.cardinality.as_deref().unwrap_or(""),
            a.target
        );
        if let Some(on) = &a.on {
            let _ = write!(out, " on {on}");
        }
        out.push('\n');
    }
    if let Some(w) = &d.where_text {
        let _ = writeln!(out, "  where {w}");
    }
    if let Some(m) = &d.implemented_by {
        let _ = writeln!(out, "  implemented by {m}");
    }
    for e in &d.elements {
        let key = if e.key { "key " } else { "" };
        let from = if e.association {
            "association".to_string()
        } else if e.fields.is_empty() {
            e.text.clone()
        } else if e.fields.len() == 1 && !e.text.contains([' ', '(']) {
            // a field as it is: where it comes from says it all
            e.fields[0].clone()
        } else {
            format!(
                "{} from {}",
                shorten(&e.text, MAX_STATEMENT),
                e.fields.join(", ")
            )
        };
        let _ = writeln!(out, "    {key}{} = {from}", e.name);
    }
    out
}

/// A processing block: `class->method`, `form name`, `function_module name`.
pub fn block(b: &m::BlockRef) -> String {
    match &b.class {
        Some(c) => format!("{c}->{}", b.name),
        None => format!("{} {}", b.kind, b.name),
    }
}

/// One line per call, grouped by source: row, calling block, kind and
/// target, options, and whether it is dynamic or unresolved.
pub fn call_graph(graph: &m::CallGraph) -> String {
    let mut out = format!(
        "{} calls in {} sources; rows per source\n",
        graph.calls.len(),
        graph.units.len()
    );
    let mut unit = None;
    for c in &graph.calls {
        if c.unit != unit {
            unit = c.unit.clone();
            let _ = writeln!(out, "\n{}", unit.as_deref().unwrap_or("?"));
        }
        out.push_str(&call(c));
    }
    if !graph.missing.is_empty() {
        let _ = writeln!(out, "\nmissing: {}", graph.missing.join(", "));
    }
    if !graph.skipped.is_empty() {
        let _ = writeln!(out, "\nnot loaded: {}", graph.skipped.join(", "));
    }
    if !graph.notes.is_empty() {
        let _ = writeln!(out, "\nnotes: {}", graph.notes.join("; "));
    }
    if !graph.errors.is_empty() {
        let _ = writeln!(out, "\nerrors: {}", errors(&graph.errors));
    }
    out
}

/// The reason of an unresolved call, as in the JSON.
fn unresolved(why: m::Unresolved) -> &'static str {
    use m::Unresolved as U;
    match why {
        U::Dynamic => "unresolved: dynamic",
        U::UnknownType => "unresolved: unknown type",
        U::Missing => "unresolved: missing",
        U::LookupFailed => "unresolved: lookup failed",
        U::HierarchyIncomplete => "unresolved: hierarchy incomplete",
        U::NotFollowed => "unresolved: not followed",
        U::Limit => "unresolved: limit",
        U::Cancelled => "unresolved: cancelled",
        U::NotLoaded => "unresolved: not loaded",
    }
}

fn errors(errors: &[m::LoadError]) -> String {
    errors
        .iter()
        .map(|e| e.to_string())
        .collect::<Vec<_>>()
        .join("; ")
}

/// Calls of one file, one line each.
pub fn calls(calls: &[m::Call]) -> String {
    calls.iter().map(call).collect()
}

fn call(c: &m::Call) -> String {
    let from = c.from.as_ref().map(block).unwrap_or_default();
    let target = match (&c.class, c.kind) {
        (Some(class), _) => format!("{class}{}{}", if c.is_static { "=>" } else { "->" }, c.name),
        (None, m::CallKind::Form) => match &c.program {
            Some(p) => format!("{}({p})", c.name),
            None => c.name.clone(),
        },
        _ => c.name.clone(),
    };
    let mut flags = Vec::new();
    if c.dynamic {
        flags.push("dynamic");
    }
    // why, unless the flag says it already
    match c.unresolved {
        _ if c.resolved => {}
        Some(why) if why != m::Unresolved::Dynamic => {
            flags.push(unresolved(why));
        }
        _ => flags.push("unresolved"),
    }
    // CALL BADI with the BAdI, its interface is the class
    let options: Vec<String> = c
        .options
        .iter()
        .map(|o| match &c.badi {
            Some(b) if o == "BADI" => format!("BADI {b}"),
            _ => o.clone(),
        })
        .collect();
    format!(
        "  {:>5} {from}: {} {target}{}{}\n",
        c.span.start.row,
        format!("{:?}", c.kind).to_ascii_lowercase(),
        if options.is_empty() {
            String::new()
        } else {
            format!(" [{}]", options.join(", "))
        },
        if flags.is_empty() {
            String::new()
        } else {
            format!(" ({})", flags.join(", "))
        },
    )
}

/// Database accesses, one line each: row, block, operation, tables and
/// the statement.
pub fn db_access(accesses: &[m::DbAccess]) -> String {
    let mut out = String::new();
    for a in accesses {
        let from = a.block.as_ref().map(block).unwrap_or_default();
        let mut tables: Vec<String> = a.tables.iter().map(|t| t.name.clone()).collect();
        // a cluster with its area: indx(mo)
        if let (Some(c), Some(t)) = (&a.cluster, tables.first_mut()) {
            let _ = write!(t, "({}) cluster", c.area);
        }
        let _ = writeln!(
            out,
            "  {:>5} {from}: {} {} | {}",
            a.span.start.row,
            format!("{:?}", a.operation).to_ascii_lowercase(),
            tables.join(" "),
            squeeze(&a.text)
        );
    }
    out
}

/// What each procedure does: its parameters, the tables it reads, and per
/// effect the statements writing it with their rows and the conditions
/// they run under, the inputs the values depend on, and the called
/// procedures writing it.
pub fn logic(logic: &m::FileLogic) -> String {
    let mut out = String::new();
    for p in &logic.procedures {
        procedure(&mut out, p);
    }
    if !logic.missing.is_empty() {
        let _ = writeln!(out, "missing: {}", logic.missing.join(", "));
    }
    if !logic.skipped.is_empty() {
        let _ = writeln!(out, "not loaded: {}", logic.skipped.join(", "));
    }
    if !logic.notes.is_empty() {
        let _ = writeln!(out, "notes: {}", logic.notes.join("; "));
    }
    if !logic.errors.is_empty() {
        let _ = writeln!(out, "errors: {}", errors(&logic.errors));
    }
    out
}

fn procedure(out: &mut String, p: &m::Procedure) {
    let mut steps = Vec::new();
    collect_steps(&p.body, &mut steps);
    let _ = write!(out, "{} {}", block(&p.block), rows(&p.span));
    if let Some(u) = &p.unit {
        let _ = write!(out, " in {u}");
    }
    out.push('\n');
    if !p.parameters.is_empty() {
        let params: Vec<String> = p
            .parameters
            .iter()
            .map(|x| format!("{} {}", direction(x.direction), x.name))
            .collect();
        let _ = writeln!(out, "  parameters: {}", params.join(", "));
    }
    let mut reads: Vec<String> = Vec::new();
    for s in &steps {
        for t in s.tables.iter().filter(|t| t.access == m::AccessKind::Read) {
            let name = match &t.via {
                Some(v) => format!("{} (via {})", t.name, block(v)),
                None => t.name.clone(),
            };
            if !reads.contains(&name) {
                reads.push(name);
            }
        }
    }
    if !reads.is_empty() {
        let _ = writeln!(out, "  reads tables: {}", reads.join(", "));
    }
    if p.effects.is_empty() {
        let _ = writeln!(out, "  no effects");
        return;
    }
    // one statement often writes several effects: each is listed once,
    // the effects name their rows
    let mut writing: Vec<&m::Step> = Vec::new();
    for e in &p.effects {
        let mut at: Vec<String> = Vec::new();
        for id in &e.statements {
            if let Some(s) = steps.iter().find(|s| s.id == *id) {
                let row = s.span.start.row.to_string();
                if !at.contains(&row) {
                    at.push(row);
                }
                if !writing.iter().any(|w| w.id == s.id) {
                    writing.push(s);
                }
            }
        }
        let _ = write!(out, "  writes {} ({})", e.name, scope(e.scope));
        if !at.is_empty() {
            let _ = write!(out, " at {}", at.join(", "));
        }
        if !e.via.is_empty() {
            let via: Vec<String> = e.via.iter().map(block).collect();
            let _ = write!(out, " via {}", via.join(", "));
        }
        if !e.inputs.is_empty() {
            let _ = write!(out, "; from {}", e.inputs.join(", "));
        }
        if !e.tables.is_empty() {
            let _ = write!(out, "; from tables {}", e.tables.join(", "));
        }
        out.push('\n');
    }
    writing.sort_by_key(|s| (s.span.start.row, s.span.start.col));
    let _ = writeln!(out, "  statements:");
    for s in writing {
        let _ = write!(
            out,
            "    {:>5} {}",
            s.span.start.row,
            shorten(&squeeze(&s.text), MAX_STATEMENT)
        );
        let guards: Vec<String> = s
            .guards
            .iter()
            .map(|g| {
                let mut text = if g.negated {
                    format!("NOT ( {} )", squeeze(&g.text))
                } else {
                    squeeze(&g.text)
                };
                // the rows of the statements setting the sy-subrc checked
                let mut set: Vec<String> = Vec::new();
                for id in &g.subrc {
                    if let Some(s) = steps.iter().find(|s| s.id == *id) {
                        let row = s.span.start.row.to_string();
                        if !set.contains(&row) {
                            set.push(row);
                        }
                    }
                }
                if !set.is_empty() {
                    let _ = write!(text, " [sy-subrc of {}]", set.join(", "));
                }
                text
            })
            .collect();
        if !guards.is_empty() {
            let _ = write!(out, "  when {}", guards.join(" AND "));
        }
        out.push('\n');
    }
}

/// Statements longer than this are shortened in the logic text; the source
/// has them in full.
const MAX_STATEMENT: usize = 160;

fn shorten(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        return text.to_string();
    }
    let cut: String = text.chars().take(max).collect();
    format!("{cut} ...")
}

fn collect_steps<'a>(steps: &'a [m::Step], out: &mut Vec<&'a m::Step>) {
    for s in steps {
        out.push(s);
        collect_steps(&s.children, out);
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

fn scope(s: m::DataScope) -> &'static str {
    match s {
        m::DataScope::Local => "local",
        m::DataScope::Parameter => "parameter",
        m::DataScope::Attribute => "attribute",
        m::DataScope::Global => "global",
        m::DataScope::Database => "database table",
        m::DataScope::External => "external",
        m::DataScope::System => "system",
        m::DataScope::Other => "other",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SRC: &str = "CLASS zcl_orders DEFINITION PUBLIC FINAL CREATE PRIVATE.
  PUBLIC SECTION.
    INTERFACES zif_saver.
    TYPES: BEGIN OF ty_key,
             id TYPE i,
           END OF ty_key.
    CONSTANTS c_open TYPE c LENGTH 1 VALUE 'O'.
    CLASS-METHODS create RETURNING VALUE(ro) TYPE REF TO zcl_orders.
    METHODS save
      IMPORTING iv_id     TYPE i
                iv_status TYPE c DEFAULT c_open
      EXPORTING ev_count  TYPE i
      RAISING   zcx_error.
  PRIVATE SECTION.
    DATA mv_count TYPE i.
ENDCLASS.
CLASS zcl_orders IMPLEMENTATION.
  METHOD create.
    ro = NEW #( ).
  ENDMETHOD.
  METHOD save.
    SELECT SINGLE status FROM zorders INTO @DATA(lv_old) WHERE id = @iv_id.
    IF iv_id > 0.
      UPDATE zorders SET status = @iv_status WHERE id = @iv_id.
      ev_count = sy-dbcnt.
    ENDIF.
  ENDMETHOD.
ENDCLASS.";

    #[test]
    fn symbols_as_definitions() {
        let text = symbols(&crate::symbols(SRC));
        assert_eq!(
            text,
            "CLASS zcl_orders DEFINITION PUBLIC FINAL CREATE PRIVATE.  \" 1-16
  PUBLIC SECTION.
    INTERFACES zif_saver.
    TYPES BEGIN OF ty_key, id TYPE i, END OF ty_key.  \" 4-6
    CONSTANTS c_open TYPE c LENGTH 1 VALUE 'O'.  \" 7
    CLASS-METHODS create RETURNING VALUE(ro) TYPE REF TO zcl_orders.  \" 8
    METHODS save IMPORTING iv_id TYPE i iv_status TYPE c DEFAULT c_open EXPORTING ev_count TYPE i RAISING zcx_error.  \" 9-13
  PRIVATE SECTION.
    DATA mv_count TYPE i.  \" 15
ENDCLASS.
\" implemented: create 18-20, save 21-27
"
        );
    }

    #[test]
    fn logic_as_effects() {
        let text = logic(&crate::logic(SRC));
        assert!(
            text.ends_with(
                "zcl_orders->save 21-27
  parameters: importing iv_id, importing iv_status, exporting ev_count
  reads tables: ZORDERS
  writes ZORDERS (database table) at 24; from iv_id, iv_status
  writes ev_count (parameter) at 25; from iv_id, iv_status
  statements:
       24 UPDATE zorders SET status = @iv_status WHERE id = @iv_id.  when iv_id > 0
       25 ev_count = sy-dbcnt.  when iv_id > 0
"
            ),
            "{text}"
        );
        assert_eq!(shorten("abcdef", 4), "abcd ...");
        assert_eq!(shorten("abcd", 4), "abcd");
    }

    #[test]
    fn calls_and_db_access_one_line_each() {
        let c = calls(&crate::calls(
            "FORM f.\n  CALL FUNCTION 'Z_POST' DESTINATION 'NONE'.\n  PERFORM g IN PROGRAM zprog.\nENDFORM.",
        ));
        assert_eq!(
            c,
            "      2 form f: function Z_POST [DESTINATION 'NONE'] (unresolved)\n      3 form f: form g(ZPROG) (unresolved)\n"
        );
        let d = db_access(&crate::db_access(SRC));
        assert!(
            d.contains("     24 zcl_orders->save: update zorders | UPDATE zorders SET status = @iv_status WHERE id = @iv_id."),
            "{d}"
        );
    }
}
