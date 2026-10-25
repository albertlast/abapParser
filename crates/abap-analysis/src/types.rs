//! Types of data objects, as far as the sources tell them: declared types
//! with local TYPES resolved, LIKE resolved through the data object it
//! names, system fields, and the types of expressions, for inline
//! declarations. Members of other classes, methods, attributes and types,
//! come from the source or, through a workspace, from their objects.
//! Dictionary types stay names: `matnr`, `mara-matnr`.

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

use abap_model as m;
use abap_syntax::system::SystemField;

/// Built-in types with a fixed length; `c`, `n`, `x` and `p` have one too.
const BUILTIN: [&str; 16] = [
    "i",
    "int8",
    "b",
    "s",
    "c",
    "n",
    "d",
    "t",
    "x",
    "p",
    "f",
    "string",
    "xstring",
    "decfloat16",
    "decfloat34",
    "utclong",
];

/// Lookups through LIKE and local types end here.
const MAX_DEPTH: u32 = 10;

fn new(kind: &str) -> m::DataType {
    m::DataType {
        kind: kind.to_string(),
        length: None,
        decimals: None,
        name: None,
        line: None,
        data_element: None,
    }
}

/// A built-in type, like `c` with length 10.
pub(crate) fn builtin(kind: &str, length: Option<u32>, decimals: Option<u32>) -> m::DataType {
    // the default length of c, n and x is 1, of p 8
    let length = length.or(match kind {
        "c" | "n" | "x" => Some(1),
        "p" => Some(8),
        _ => None,
    });
    m::DataType {
        length,
        decimals: decimals.filter(|d| *d > 0),
        ..new(kind)
    }
}

/// A type known only by its name.
pub(crate) fn named(name: &str) -> m::DataType {
    m::DataType {
        name: Some(name.to_ascii_lowercase()),
        ..new("named")
    }
}

/// A reference to a class, interface or type.
pub(crate) fn reference(name: &str) -> m::DataType {
    m::DataType {
        name: Some(name.to_ascii_lowercase()),
        ..new("ref")
    }
}

/// An internal table of the line type.
pub(crate) fn table(line: m::DataType) -> m::DataType {
    m::DataType {
        line: Some(Box::new(line)),
        ..new("table")
    }
}

pub(crate) fn range() -> m::DataType {
    new("range")
}

pub(crate) fn structure() -> m::DataType {
    new("structure")
}

/// The type of a system field.
pub(crate) fn system(f: &SystemField) -> m::DataType {
    m::DataType {
        length: (f.length > 0).then_some(f.length),
        decimals: (f.decimals > 0).then_some(f.decimals),
        data_element: Some(f.data_element.to_string()),
        ..new(f.kind.as_str())
    }
}

fn number(text: Option<&str>) -> Option<u32> {
    text.and_then(|t| t.trim().trim_matches('\'').parse().ok())
}

/// Classes, interfaces and dictionary types outside the source, like the
/// objects of a workspace.
pub(crate) trait Repository {
    /// The object defining a class or interface.
    fn object_of(&self, name: &str) -> Option<Arc<crate::Object>>;
    /// The line type of a dictionary table type.
    fn table_type_line(&self, name: &str) -> Option<m::DataType>;
    /// A type group, `TYPE-POOL name`.
    fn type_group(&self, name: &str) -> Option<Arc<crate::Object>>;
}

impl Repository for crate::Workspace {
    fn object_of(&self, name: &str) -> Option<Arc<crate::Object>> {
        self.type_object(name)
    }

    fn table_type_line(&self, name: &str) -> Option<m::DataType> {
        crate::Workspace::table_type_line(self, name)
    }

    fn type_group(&self, name: &str) -> Option<Arc<crate::Object>> {
        crate::Workspace::type_group(self, name)
    }
}

/// The type groups a type name may belong to: its types start with the name
/// of the type group, at most five letters, digits or underscores, after a
/// namespace like `/abc/`; so each prefix up to a `_` that short.
fn type_groups_of(name: &str) -> Vec<&str> {
    let start = match name.strip_prefix('/') {
        Some(rest) => match rest.find('/') {
            Some(p) => p + 2,
            None => return Vec::new(),
        },
        None => 0,
    };
    let local = &name[start..];
    if !local.starts_with(|c: char| c.is_ascii_alphabetic()) {
        return Vec::new();
    }
    local
        .match_indices('_')
        .map(|(i, _)| i)
        .take_while(|i| *i <= 5)
        .filter(|i| *i > 0)
        .map(|i| &name[..start + i])
        .collect()
}

/// What to look for in a class or interface.
#[derive(Clone, Copy)]
enum Member<'n> {
    /// a parameter of a method, its RETURNING parameter when `None`
    Parameter {
        method: &'n str,
        name: Option<&'n str>,
    },
    Attribute(&'n str),
    /// a type, and components of it
    Type(&'n str, &'n [&'n str]),
}

/// A type found, with the declaration it comes from, whose components a
/// path may name.
#[derive(Clone)]
struct Found<'a> {
    ty: m::DataType,
    decl: Option<&'a m::Declaration>,
}

/// The data and types one procedure sees, innermost first, and what its
/// inline declarations are.
pub(crate) struct Types<'a> {
    data: Vec<&'a [m::Declaration]>,
    parameters: Vec<&'a m::Parameter>,
    symbols: &'a m::FileSymbols,
    /// the class of a method, lower case
    class: Option<String>,
    /// where classes outside the source are looked up
    classes: Option<&'a dyn Repository>,
    /// the interface or superclass that declares the signature of the
    /// method, for `zif_x~meth` and redefinitions, with the method's name
    inherited: Option<(String, String)>,
    inline: RefCell<HashMap<String, m::DataType>>,
}

impl<'a> Types<'a> {
    /// The scopes of a processing block: its locals and parameters, the
    /// attributes of its class, the data of the program.
    pub(crate) fn of_block(symbols: &'a m::FileSymbols, block: &m::BlockRef) -> Self {
        let eq = |a: &str, b: &str| a.eq_ignore_ascii_case(b);
        let mut data: Vec<&'a [m::Declaration]> = Vec::new();
        let mut parameters: Vec<&'a m::Parameter> = Vec::new();
        let mut class = None;
        let mut inherited = None;
        match block.kind.as_str() {
            "method" => {
                let name = block.class.as_deref().unwrap_or_default();
                class = Some(name.to_ascii_lowercase());
                let c = symbols.classes.iter().find(|c| eq(&c.name, name));
                let implementation = c.and_then(|c| c.implementation.as_ref()).or_else(|| {
                    symbols
                        .orphan_implementations
                        .iter()
                        .find(|o| eq(&o.class, name))
                        .map(|o| &o.implementation)
                });
                if let Some(mi) =
                    implementation.and_then(|i| i.methods.iter().find(|m| eq(&m.name, &block.name)))
                {
                    data.push(&mi.locals);
                }
                if let Some(c) = c {
                    let method = block.name.rsplit('~').next().unwrap_or(&block.name);
                    let md = c.methods.iter().find(|m| eq(&m.name, method));
                    if let Some(md) = md {
                        parameters.extend(&md.parameters);
                    }
                    // the signature is the interface's or the superclass's
                    inherited = match (block.name.split_once('~'), md) {
                        (Some((interface, m)), _) => {
                            Some((interface.to_ascii_lowercase(), m.to_ascii_lowercase()))
                        }
                        (None, Some(md)) if md.is_redefinition => c
                            .superclass
                            .as_ref()
                            .map(|s| (s.to_ascii_lowercase(), method.to_ascii_lowercase())),
                        _ => None,
                    };
                    data.push(&c.attributes);
                    data.push(&c.types);
                }
            }
            "form" | "function_module" => {
                let routines = if block.kind == "form" {
                    &symbols.forms
                } else {
                    &symbols.function_modules
                };
                if let Some(r) = routines.iter().find(|r| eq(&r.name, &block.name)) {
                    data.push(&r.locals);
                    parameters.extend(&r.parameters);
                }
            }
            _ => {}
        }
        data.push(&symbols.declarations);
        Types {
            data,
            parameters,
            symbols,
            class,
            classes: None,
            inherited,
            inline: RefCell::new(HashMap::new()),
        }
    }

    /// Looks up classes outside the source through `classes`.
    pub(crate) fn with_classes(mut self, classes: Option<&'a dyn Repository>) -> Self {
        self.classes = classes;
        self
    }

    /// Only the data of the program, for its declarations.
    pub(crate) fn of_program(symbols: &'a m::FileSymbols) -> Self {
        Types {
            data: vec![&symbols.declarations],
            parameters: Vec::new(),
            symbols,
            class: None,
            classes: None,
            inherited: None,
            inline: RefCell::new(HashMap::new()),
        }
    }

    /// Declarations in `extra` first, e.g. the components of a structure.
    pub(crate) fn with(mut self, extra: &'a [m::Declaration]) -> Self {
        self.data.insert(0, extra);
        self
    }

    /// Records the type of an inline declaration.
    pub(crate) fn declare(&self, name: &str, ty: m::DataType) {
        self.inline
            .borrow_mut()
            .insert(name.to_ascii_lowercase(), ty);
    }

    fn find(&self, name: &str, types: bool) -> Option<&'a m::Declaration> {
        self.data.iter().find_map(|scope| {
            scope.iter().find(|d| {
                (d.kind == m::DeclarationKind::Type) == types && d.name.eq_ignore_ascii_case(name)
            })
        })
    }

    /// The type of the data declared by name around a row, components of
    /// structures too, like `comp TYPE i VALUE 1` in `BEGIN OF ... END OF`.
    pub(crate) fn declared_at(&self, name: &str, row: u32) -> Option<m::DataType> {
        fn at<'d>(ds: &'d [m::Declaration], name: &str, row: u32) -> Option<&'d m::Declaration> {
            ds.iter().find_map(|d| {
                // components of a chain start where the chain does
                let within = d.span.start.row <= row && row <= d.span.end.row;
                if within && d.name.eq_ignore_ascii_case(name) {
                    Some(d)
                } else {
                    at(&d.components, name, row)
                }
            })
        }
        let d = self.data.iter().find_map(|ds| at(ds, name, row))?;
        self.of_declaration(d)
    }

    /// The type a declaration gives.
    pub(crate) fn of_declaration(&self, d: &'a m::Declaration) -> Option<m::DataType> {
        self.declared(d, 0).map(|f| f.ty)
    }

    fn declared(&self, d: &'a m::Declaration, depth: u32) -> Option<Found<'a>> {
        if d.is_structure && !d.is_enum {
            return Some(Found {
                ty: m::DataType {
                    name: (d.kind == m::DeclarationKind::Type).then(|| d.name.to_ascii_lowercase()),
                    ..structure()
                },
                decl: Some(d),
            });
        }
        let ty = match &d.typing {
            Some(t) => self.typing(t, depth)?,
            None => match d.kind {
                // DATA x. is c LENGTH 1
                m::DeclarationKind::Data
                | m::DeclarationKind::ClassData
                | m::DeclarationKind::Static
                | m::DeclarationKind::Constant => Found {
                    ty: builtin("c", None, None),
                    decl: None,
                },
                // TABLES ztab, NODES node: a work area of the structure
                m::DeclarationKind::Tables | m::DeclarationKind::Node => Found {
                    ty: named(&d.name),
                    decl: None,
                },
                _ => return None,
            },
        };
        Some(ty)
    }

    /// A length or number of decimals as written: a number, or a constant
    /// like `abap_max_comp_name_ln`; `Some(None)` when its value is not
    /// known, `None` when nothing is written.
    fn size(&self, text: &Option<String>) -> Option<Option<u32>> {
        let text = text.as_deref()?;
        if let Some(n) = number(Some(text)) {
            return Some(Some(n));
        }
        let constant = self.find(text.trim(), false);
        Some(constant.and_then(|c| number(c.value.as_deref())))
    }

    fn typing(&self, t: &m::Typing, depth: u32) -> Option<Found<'a>> {
        if depth > MAX_DEPTH {
            return None;
        }
        let name = t.name.as_deref()?;
        let like = t.text.trim_start().to_ascii_uppercase().starts_with("LIKE");
        let plain = |ty| Found { ty, decl: None };
        Some(match t.kind {
            m::TypingKind::Type => {
                let (length, decimals) = (self.size(&t.length), self.size(&t.decimals));
                let mut found = self.type_named(name, length.flatten(), decimals.flatten(), depth);
                // LENGTH of a constant not known: no length, not the default
                if length == Some(None) {
                    found.ty.length = None;
                }
                found
            }
            m::TypingKind::Like => self.path(name, depth + 1)?,
            m::TypingKind::Structure => self.type_named(name, None, None, depth),
            m::TypingKind::TypeRefTo => plain(reference(name)),
            m::TypingKind::LikeRefTo => plain(m::DataType {
                line: self.path(name, depth + 1).map(|f| Box::new(f.ty)),
                ..new("ref")
            }),
            m::TypingKind::Table => {
                let line = if like {
                    self.path(name, depth + 1)
                } else {
                    Some(self.type_named(name, None, None, depth))
                };
                plain(table(line.map(|f| f.ty).unwrap_or_else(|| named(name))))
            }
            m::TypingKind::Range | m::TypingKind::For => {
                let line = if t.kind == m::TypingKind::For || like {
                    self.path(name, depth + 1).map(|f| f.ty)
                } else {
                    Some(self.type_named(name, None, None, depth).ty)
                };
                plain(m::DataType {
                    line: Some(Box::new(line.unwrap_or_else(|| named(name)))),
                    ..new("range")
                })
            }
            m::TypingKind::LineOf => {
                let of = if like {
                    self.path(name, depth + 1)?
                } else {
                    self.type_named(name, None, None, depth)
                };
                match of.ty.line {
                    Some(line) if of.ty.kind == "table" => self.line(*line),
                    _ => plain(m::DataType {
                        name: Some(format!("line of {}", name.to_ascii_lowercase())),
                        ..new("named")
                    }),
                }
            }
        })
    }

    /// A line type: a local structure type keeps its components.
    fn line(&self, line: m::DataType) -> Found<'a> {
        let decl = match (&line.kind[..], &line.name) {
            ("structure", Some(n)) => self.find(n, true),
            _ => None,
        };
        Found { ty: line, decl }
    }

    /// The type of a type name: built-in, local TYPES resolved, a system
    /// field like `sy-datum`, else a name.
    fn type_named(
        &self,
        name: &str,
        length: Option<u32>,
        decimals: Option<u32>,
        depth: u32,
    ) -> Found<'a> {
        let lower = name.to_ascii_lowercase();
        if BUILTIN.contains(&lower.as_str()) {
            return Found {
                ty: builtin(&lower, length, decimals),
                decl: None,
            };
        }
        if let Some(f) = system_field(&lower) {
            return Found {
                ty: system(f),
                decl: None,
            };
        }
        if let Some(d) = self.find(&lower, true) {
            if let Some(mut f) = self.declared(d, depth + 1) {
                f.ty.name.get_or_insert(lower);
                return f;
            }
        }
        // a type of a class or interface: zcl_x=>ty_rows
        if let Some((class, ty)) = lower.split_once("=>") {
            if let Some(mut t) = self.member(class, Member::Type(ty, &[]), depth + 1) {
                // qualified, so its components can be looked up
                if t.name.as_deref() == Some(ty) {
                    t.name = Some(lower.clone());
                }
                return Found { ty: t, decl: None };
            }
        }
        // a type of a type group: abap_compdescr_tab
        if let Some(mut t) = self.group_type(&lower, &[], depth + 1) {
            t.name.get_or_insert(lower);
            return Found { ty: t, decl: None };
        }
        Found {
            ty: named(&lower),
            decl: None,
        }
    }

    /// A type of a type group, or a component of it, typed in the scope of
    /// the type group.
    fn group_type(&self, name: &str, components: &[&str], depth: u32) -> Option<m::DataType> {
        if depth > MAX_DEPTH {
            return None;
        }
        let classes = self.classes?;
        let (group, d) = type_groups_of(name).into_iter().find_map(|g| {
            let group = classes.type_group(g)?;
            let at = group.symbols.declarations.iter().position(|d| {
                d.kind == m::DeclarationKind::Type && d.name.eq_ignore_ascii_case(name)
            })?;
            Some((group, at))
        })?;
        let symbols = &group.symbols;
        let d = &symbols.declarations[d];
        let scope = Types {
            data: vec![&symbols.declarations],
            parameters: Vec::new(),
            symbols,
            class: None,
            classes: self.classes,
            inherited: None,
            inline: RefCell::new(HashMap::new()),
        };
        let mut found = scope.declared(d, depth + 1)?;
        for c in components {
            found = scope.component(found, c, depth + 1)?;
        }
        Some(found.ty)
    }

    /// The type of a data object as written, components included, like
    /// `ls_order-items`, `me->mv_count`, `sy-datum`.
    pub(crate) fn of_path(&self, path: &str) -> Option<m::DataType> {
        self.path(path, 0).map(|f| f.ty)
    }

    fn path(&self, path: &str, depth: u32) -> Option<Found<'a>> {
        if depth > MAX_DEPTH {
            return None;
        }
        let lower = path.trim().to_ascii_lowercase();
        let lower = lower.strip_prefix("me->").unwrap_or(&lower);
        // offsets and lengths keep the type
        let lower = lower.split('+').next().unwrap_or(lower);
        // a table expression `lt[ id = 1 ]` is a line: `~` marks it
        let lower = lines_marked(lower)?;
        let lower = lower.as_str();
        if let Some(f) = system_field(lower) {
            return Some(Found {
                ty: system(f),
                decl: None,
            });
        }
        // the last -> or =>: an attribute of what is before it
        let arrow = [lower.rfind("->"), lower.rfind("=>")]
            .into_iter()
            .flatten()
            .max();
        if let Some(at) = arrow {
            let (before, after) = (&lower[..at], &lower[at + 2..]);
            let mut parts = after.split('-');
            let first = parts.next()?;
            let name = first.trim_end_matches('~');
            let found = if name == "*" {
                // lr->*: what the reference refers to
                let ty = self.path(before, depth + 1)?.ty;
                self.line(self.line_of(&ty)?)
            } else if lower[at..].starts_with("=>") {
                Found {
                    ty: self.member(before, Member::Attribute(name), depth + 1)?,
                    decl: None,
                }
            } else {
                let ty = self.path(before, depth + 1)?.ty;
                self.through_reference(&ty, name, depth)?
            };
            let found = self.lines(found, first)?;
            return self.components(found, parts, depth);
        }
        let mut parts = lower.split('-');
        let first = parts.next()?;
        let root = first.trim_end_matches('~').trim_end_matches("[]");
        let mut found = self.root(root, depth)?;
        found = self.lines(found, first)?;
        self.components(found, parts, depth)
    }

    /// `lr->name` of a reference: an attribute of a class, or a component
    /// of the data a data reference refers to, like one of
    /// `REFERENCE INTO DATA(lr)` or `TYPE REF TO ty_s`.
    fn through_reference(&self, ty: &m::DataType, name: &str, depth: u32) -> Option<Found<'a>> {
        if ty.kind != "ref" {
            return None;
        }
        if let Some(line) = self.line_of(ty) {
            return self.component(self.line(line), name, depth + 1);
        }
        let target = ty.name.as_deref()?;
        // generic: no components known
        if target == "data" || target == "object" {
            return None;
        }
        if self.find(target, true).is_some() {
            let data = self.type_named(target, None, None, depth + 1);
            return self.component(data, name, depth + 1);
        }
        Some(Found {
            ty: self.member(target, Member::Attribute(name), depth + 1)?,
            decl: None,
        })
    }

    /// `->name` of a value of type `ty`, see [`Self::through_reference`].
    pub(crate) fn through(&self, ty: &m::DataType, name: &str) -> Option<m::DataType> {
        self.through_reference(ty, &name.to_ascii_lowercase(), 0)
            .map(|f| f.ty)
    }

    /// Follows the components of a path, like `-items~-id`.
    fn components<'p>(
        &self,
        mut found: Found<'a>,
        parts: impl Iterator<Item = &'p str>,
        depth: u32,
    ) -> Option<Found<'a>> {
        for part in parts {
            let component = part.trim_end_matches('~');
            found = self.component(found, component, depth)?;
            found = self.lines(found, part)?;
        }
        Some(found)
    }

    fn component(&self, found: Found<'a>, component: &str, depth: u32) -> Option<Found<'a>> {
        Some(match found.decl {
            Some(d) if !d.components.is_empty() => {
                let c = d
                    .components
                    .iter()
                    .find(|c| c.name.eq_ignore_ascii_case(component))?;
                self.declared(c, depth + 1)?
            }
            // a type of another class: zcl_x=>ty_s-comp
            _ if found.ty.kind == "structure"
                && found.ty.name.as_deref().is_some_and(|n| n.contains("=>")) =>
            {
                let name = found.ty.name.as_deref()?;
                let (class, ty) = name.split_once("=>")?;
                Found {
                    ty: self.member(class, Member::Type(ty, &[component]), depth + 1)?,
                    decl: None,
                }
            }
            // a structure of a type group: abap_compdescr-name
            _ if found.ty.kind == "structure" && found.ty.name.is_some() => Found {
                ty: self.group_type(found.ty.name.as_deref()?, &[component], depth + 1)?,
                decl: None,
            },
            // a dictionary structure: its column, like mara-matnr
            _ if found.ty.kind == "named" => Found {
                ty: named(&format!("{}-{component}", found.ty.name.as_deref()?)),
                decl: None,
            },
            _ => return None,
        })
    }

    /// The type of a component of a structure type, like that of a method
    /// result: `get( )-id`.
    pub(crate) fn component_of(&self, ty: &m::DataType, component: &str) -> Option<m::DataType> {
        let found = self.line(ty.clone());
        self.component(found, &component.to_ascii_lowercase(), 0)
            .map(|f| f.ty)
    }

    /// The line type, once for each table expression marked after `part`.
    fn lines(&self, mut found: Found<'a>, part: &str) -> Option<Found<'a>> {
        for _ in 0..part.len() - part.trim_end_matches('~').len() {
            let line = self.line_of(&found.ty)?;
            found = self.line(line);
        }
        Some(found)
    }

    fn root(&self, name: &str, depth: u32) -> Option<Found<'a>> {
        if let Some(ty) = self.inline.borrow().get(name) {
            let decl = match (&ty.kind[..], &ty.name) {
                ("structure", Some(n)) => self.find(n, true),
                _ => None,
            };
            return Some(Found {
                ty: ty.clone(),
                decl,
            });
        }
        if let Some(p) = self
            .parameters
            .iter()
            .find(|p| p.name.trim_start_matches('!').eq_ignore_ascii_case(name))
        {
            return self.typing(p.typing.as_ref()?, depth);
        }
        if let Some(d) = self.find(name, false) {
            return self.declared(d, depth);
        }
        // a parameter of zif_x~meth, or of a redefined method
        if let Some((owner, method)) = &self.inherited {
            let parameter = Member::Parameter {
                method,
                name: Some(name),
            };
            if let Some(ty) = self.member(owner, parameter, depth + 1) {
                return Some(Found { ty, decl: None });
            }
        }
        // an attribute of a superclass
        let inherited = self
            .class
            .clone()
            .and_then(|class| self.member(&class, Member::Attribute(name), depth + 1));
        Some(Found {
            ty: inherited.or_else(|| constant(name))?,
            decl: None,
        })
    }

    /// The line type of a table or range, or what a reference refers to.
    pub(crate) fn line_of(&self, ty: &m::DataType) -> Option<m::DataType> {
        match ty.kind.as_str() {
            "table" | "range" | "ref" => ty.line.as_deref().cloned(),
            // a dictionary table type: string_table
            "named" => {
                let name = ty.name.as_deref()?;
                if name.contains(['-', '=', ' ']) {
                    return None;
                }
                self.classes?.table_type_line(name)
            }
            _ => None,
        }
    }

    /// The RETURNING type of a method; of the own class when `class` is
    /// `None`.
    pub(crate) fn returning(&self, class: Option<&str>, method: &str) -> Option<m::DataType> {
        let class = class.map(str::to_string).or_else(|| self.class.clone())?;
        self.member(
            &class,
            Member::Parameter {
                method: &method.to_ascii_lowercase(),
                name: None,
            },
            0,
        )
    }

    /// The type of a parameter of a method, like that of `ev_x` of
    /// `lo->get( IMPORTING ev_x = DATA(x) )`.
    pub(crate) fn parameter(
        &self,
        class: Option<&str>,
        method: &str,
        parameter: &str,
    ) -> Option<m::DataType> {
        let class = class.map(str::to_string).or_else(|| self.class.clone())?;
        self.member(
            &class,
            Member::Parameter {
                method: &method.to_ascii_lowercase(),
                name: Some(&parameter.to_ascii_lowercase()),
            },
            0,
        )
    }

    /// The type of an attribute of a class or interface.
    pub(crate) fn attribute(&self, class: &str, attribute: &str) -> Option<m::DataType> {
        self.member(class, Member::Attribute(&attribute.to_ascii_lowercase()), 0)
    }

    /// The class of the method being analyzed, lower case.
    pub(crate) fn own_class(&self) -> Option<&str> {
        self.class.as_deref()
    }

    /// A member of a class or interface: of the source, else of its object
    /// through `classes`.
    fn member(&self, class: &str, member: Member, depth: u32) -> Option<m::DataType> {
        if depth > MAX_DEPTH {
            return None;
        }
        let class = class.to_ascii_lowercase();
        if let Some(found) = member_in(self.symbols, &class, member, self.classes, depth) {
            return found;
        }
        let object = self.classes?.object_of(&class)?;
        member_in(&object.symbols, &class, member, self.classes, depth).flatten()
    }

    /// The type of a type name in an expression, like the type of
    /// `VALUE ty_rows( )`.
    pub(crate) fn type_of_name(&self, name: &str) -> m::DataType {
        self.type_named(name, None, None, 0).ty
    }
}

/// A member of a class or interface of `symbols`: `None` when it has no
/// such class or interface, else what is found there or, for members it
/// inherits, in its superclass and interfaces. Types are those of the scope
/// of the class.
fn member_in(
    symbols: &m::FileSymbols,
    class: &str,
    member: Member,
    classes: Option<&dyn Repository>,
    depth: u32,
) -> Option<Option<m::DataType>> {
    let eq = |a: &str, b: &str| a.eq_ignore_ascii_case(b);
    let c = symbols.classes.iter().find(|c| eq(&c.name, class));
    let i = symbols.interfaces.iter().find(|i| eq(&i.name, class));
    if c.is_none() && i.is_none() {
        return None;
    }
    let (attributes, types, methods, aliases, parents) = match (c, i) {
        (Some(c), _) => (
            &c.attributes,
            &c.types,
            &c.methods,
            &c.aliases,
            c.interfaces
                .iter()
                .map(|x| x.name.as_str())
                .chain(c.superclass.as_deref())
                .collect::<Vec<_>>(),
        ),
        (None, Some(i)) => (
            &i.attributes,
            &i.types,
            &i.methods,
            &i.aliases,
            i.interfaces.iter().map(|x| x.name.as_str()).collect(),
        ),
        (None, None) => return None,
    };
    // the scope the member's types are written in
    let scope = Types {
        data: vec![types, attributes, &symbols.declarations],
        parameters: Vec::new(),
        symbols,
        class: Some(class.to_string()),
        classes,
        inherited: None,
        inline: RefCell::new(HashMap::new()),
    };
    // zif_x~name, directly or through an alias
    let named = match member {
        Member::Parameter { method, .. } => method,
        Member::Attribute(a) => a,
        Member::Type(t, _) => t,
    };
    let component = aliases
        .iter()
        .find(|a| eq(&a.name, named))
        .map(|a| a.component.to_ascii_lowercase())
        .or_else(|| named.contains('~').then(|| named.to_string()));
    if let Some((interface, name)) = component.as_deref().and_then(|c| c.split_once('~')) {
        let inner = match member {
            Member::Parameter { name: p, .. } => Member::Parameter {
                method: name,
                name: p,
            },
            Member::Attribute(_) => Member::Attribute(name),
            Member::Type(_, comps) => Member::Type(name, comps),
        };
        return Some(scope.member(interface, inner, depth + 1));
    }
    let found = match member {
        Member::Parameter { method, name } => methods
            .iter()
            .find(|m| eq(&m.name, method))
            .and_then(|md| {
                md.parameters.iter().find(|p| match name {
                    Some(n) => eq(p.name.trim_start_matches('!'), n),
                    None => p.direction == m::Direction::Returning,
                })
            })
            .map(|p| {
                p.typing
                    .as_ref()
                    .and_then(|t| scope.typing(t, depth + 1).map(|f| f.ty))
            }),
        Member::Attribute(a) => attributes
            .iter()
            .find(|d| d.kind != m::DeclarationKind::Type && eq(&d.name, a))
            .map(|d| scope.of_declaration(d)),
        Member::Type(t, comps) => types.iter().find(|d| eq(&d.name, t)).map(|d| {
            let mut found = scope.declared(d, depth + 1)?;
            for comp in comps {
                found = scope.component(found, comp, depth + 1)?;
            }
            Some(found.ty)
        }),
    };
    if let Some(found) = found {
        return Some(found.map(|t| qualified(t, class, types)));
    }
    // inherited: from the superclass or an interface
    Some(
        parents
            .iter()
            .find_map(|p| scope.member(p, member, depth + 1)),
    )
}

/// A type found in a class, with the names of the class's types qualified,
/// `zcl_x=>ty_s`, so they are found from elsewhere too; line types included.
fn qualified(mut ty: m::DataType, class: &str, types: &[m::Declaration]) -> m::DataType {
    if let Some(n) = &ty.name {
        if types.iter().any(|d| d.name.eq_ignore_ascii_case(n)) {
            ty.name = Some(format!("{class}=>{n}"));
        }
    }
    ty.line = ty.line.map(|l| Box::new(qualified(*l, class, types)));
    ty
}

/// The path with each table expression `[ ... ]` replaced by `~`; `[]`,
/// the table body, stays.
fn lines_marked(path: &str) -> Option<String> {
    let mut out = String::new();
    let mut depth = 0;
    let mut chars = path.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '[' if depth == 0 && chars.peek() == Some(&']') => {
                chars.next();
                out.push_str("[]");
            }
            '[' => depth += 1,
            ']' => {
                depth -= 1;
                if depth < 0 {
                    return None;
                }
                if depth == 0 {
                    out.push('~');
                }
            }
            _ if depth == 0 => out.push(c),
            _ => {}
        }
    }
    (depth == 0).then_some(out)
}

fn system_field(name: &str) -> Option<&'static SystemField> {
    let lower = name.to_ascii_lowercase();
    (lower.starts_with("sy-") || lower.starts_with("syst-"))
        .then(|| abap_syntax::system::system_field(&lower))
        .flatten()
}

/// The constants every program knows: `space`, a `c` of length 1, and
/// `abap_true`, `abap_false` and `abap_undefined` of the type pool ABAP,
/// typed `abap_bool`.
pub(crate) fn constant(name: &str) -> Option<m::DataType> {
    match name.to_ascii_lowercase().as_str() {
        "space" => Some(builtin("c", Some(1), None)),
        "abap_true" | "abap_false" | "abap_undefined" => Some(m::DataType {
            name: Some("abap_bool".to_string()),
            ..builtin("c", Some(1), None)
        }),
        _ => None,
    }
}

/// The type of `abs`, `sign`, `ceil`, `floor`, `trunc` and `frac` of an
/// argument outside an arithmetic expression, as SAP documents it: the
/// numeric type of the argument; `d`, `t`, `x` and `xstring` give `i`,
/// `c`, `n` and `string` give `p`.
pub(crate) fn numeric_function(name: &str, arg: &m::DataType) -> Option<m::DataType> {
    let lower = name.to_ascii_lowercase();
    if !["abs", "sign", "ceil", "floor", "trunc", "frac"].contains(&lower.as_str()) {
        return None;
    }
    match arg.kind.as_str() {
        "i" | "int8" | "p" | "decfloat16" | "decfloat34" | "f" => Some(m::DataType {
            data_element: None,
            ..arg.clone()
        }),
        // b and s too: integers
        "b" | "s" | "d" | "t" | "x" | "xstring" => Some(builtin("i", None, None)),
        "c" | "n" | "string" => Some(new("p")),
        _ => None,
    }
}

/// The calculation type of an arithmetic expression from the types of its
/// operands, as SAP documents it, which is the type of an inline declaration
/// it is assigned to: `decfloat34` with a decimal floating point number,
/// else `f` with `f` or the operator `**`, else `p` (length 8, no decimals),
/// `int8` or `i`; `d`, `t`, `x` and `xstring` count as `i`, `c`, `n` and
/// `string` as `p`. `None` when an operand type is not numeric or unknown.
pub(crate) fn calculation_type<'t>(
    operands: impl IntoIterator<Item = &'t m::DataType>,
    power: bool,
) -> Option<m::DataType> {
    // ranks: i, int8, p, f, decfloat34
    let mut rank = if power { 3 } else { 0 };
    for t in operands {
        let r = match t.kind.as_str() {
            "i" | "b" | "s" | "d" | "t" | "x" | "xstring" => 0,
            "int8" => 1,
            "p" | "c" | "n" | "string" => 2,
            "f" => 3,
            "decfloat16" | "decfloat34" => 4,
            _ => return None,
        };
        rank = rank.max(r);
    }
    Some(match rank {
        0 => builtin("i", None, None),
        1 => builtin("int8", None, None),
        2 => builtin("p", Some(8), None),
        3 => builtin("f", None, None),
        _ => builtin("decfloat34", None, None),
    })
}

/// The type of the built-in numeric functions with named arguments or a
/// floating point argument, from the types of their arguments, as SAP
/// documents each: `ipow` as `abs` of its `base`, `nmax` and `nmin` the
/// calculation type of all arguments, the floating point functions (`sqrt`,
/// `exp`, `sin`, ...) `decfloat34` for a decimal floating point argument,
/// else `f`; `round` and `rescale` convert theirs to `decfloat34`.
pub(crate) fn numeric_function_of(
    name: &str,
    args: &[(Option<&str>, Option<m::DataType>)],
) -> Option<m::DataType> {
    let lower = name.to_ascii_lowercase();
    let arg = |p: &str| {
        args.iter()
            .find(|(n, _)| n.is_some_and(|n| n.eq_ignore_ascii_case(p)))
            .and_then(|(_, t)| t.as_ref())
    };
    match lower.as_str() {
        "ipow" => numeric_function("abs", arg("base")?),
        "nmax" | "nmin" => {
            let types: Option<Vec<&m::DataType>> = args.iter().map(|(_, t)| t.as_ref()).collect();
            calculation_type(types?, false)
        }
        "acos" | "asin" | "atan" | "cos" | "sin" | "tan" | "cosh" | "sinh" | "tanh" | "exp"
        | "log" | "log10" | "sqrt" => {
            let [(None, Some(t))] = args else {
                return None;
            };
            Some(if t.kind.starts_with("decfloat") {
                builtin("decfloat34", None, None)
            } else {
                builtin("f", None, None)
            })
        }
        "round" | "rescale" => Some(builtin("decfloat34", None, None)),
        _ => None,
    }
}

/// The result types of built-in functions, as SAP's documentation gives
/// them: description functions are numeric, processing functions for
/// strings give a string; `boolc` a string, `boolx` a byte string, `xsdbool`
/// a `c` of length 1 typed `XSDBOOLEAN`. Predicate functions like
/// `line_exists` give no value.
pub(crate) fn builtin_function(name: &str) -> Option<m::DataType> {
    let lower = name.to_ascii_lowercase();
    Some(match lower.as_str() {
        "lines" | "line_index" | "strlen" | "numofchar" | "charlen" | "dbmaxlen" | "char_off"
        | "xstrlen" | "find" | "find_end" | "find_any_of" | "find_any_not_of" | "count"
        | "count_any_of" | "count_any_not_of" | "distance" => builtin("i", None, None),
        "xsdbool" => m::DataType {
            name: Some("xsdboolean".to_string()),
            ..builtin("c", Some(1), None)
        },
        "boolc" | "condense" | "concat_lines_of" | "escape" | "insert" | "match" | "repeat"
        | "replace" | "reverse" | "segment" | "shift_left" | "shift_right" | "substring"
        | "substring_after" | "substring_before" | "substring_from" | "substring_to"
        | "to_upper" | "to_lower" | "to_mixed" | "from_mixed" | "translate" | "cmax" | "cmin" => {
            builtin("string", None, None)
        }
        "boolx" => builtin("xstring", None, None),
        "utclong_current" | "utclong_add" => builtin("utclong", None, None),
        "utclong_diff" => builtin("decfloat34", None, None),
        _ => return None,
    })
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    /// A type in short: `c(10)`, `table of structure ty_s`, `ref zcl_x`.
    pub(crate) fn short(t: &m::DataType) -> String {
        let mut out = t.kind.clone();
        if let Some(l) = t.length {
            out.push_str(&format!("({l}"));
            if let Some(d) = t.decimals {
                out.push_str(&format!(",{d}"));
            }
            out.push(')');
        }
        if let Some(n) = &t.name {
            out.push_str(&format!(" {n}"));
        }
        if let Some(line) = &t.line {
            out.push_str(&format!(" of {}", short(line)));
        }
        out
    }

    #[test]
    fn declarations_with_like_and_local_types() {
        let s = crate::symbols(
            "REPORT z.
TYPES: BEGIN OF ty_s, id TYPE i, name TYPE string, END OF ty_s.
TYPES ty_t TYPE STANDARD TABLE OF ty_s WITH EMPTY KEY.
TYPES ty_c10 TYPE c LENGTH 10.
DATA gv_a(10) TYPE c.
DATA gv_b TYPE p LENGTH 8 DECIMALS 2.
DATA gv_c.
DATA gv_d TYPE ty_c10.
DATA gt LIKE TABLE OF gv_b.
DATA gr TYPE RANGE OF matnr.
DATA gs TYPE LINE OF ty_t.
DATA gx TYPE REF TO zcl_x.
DATA gl LIKE LINE OF gt.
DATA gm TYPE mara.
DATA gn LIKE gs-name.
DATA gk LIKE gm-matnr.
DATA gy LIKE sy-uname.
CONSTANTS gc_len TYPE i VALUE 61.
DATA gz TYPE c LENGTH gc_len.
DATA gw TYPE c LENGTH zcl_x=>len.
SELECT-OPTIONS so FOR gv_a.",
        );
        let got: Vec<String> = s
            .declarations
            .iter()
            .map(|d| format!("{} {}", d.name, short(d.data_type.as_ref().unwrap())))
            .collect();
        assert_eq!(
            got,
            [
                "ty_s structure ty_s",
                "ty_t table of structure ty_s",
                "ty_c10 c(10)",
                "gv_a c(10)",
                "gv_b p(8,2)",
                "gv_c c(1)",
                "gv_d c(10) ty_c10",
                "gt table of p(8,2)",
                "gr range of named matnr",
                "gs structure ty_s",
                "gx ref zcl_x",
                "gl p(8,2)",
                "gm named mara",
                "gn string",
                "gk named mara-matnr",
                "gy c(12)",
                "gc_len i",
                "gz c(61)",
                "gw c",
                "so range of c(10)",
            ]
        );
        // components too
        let id = &s.declarations[0].components[0];
        assert_eq!(short(id.data_type.as_ref().unwrap()), "i");
    }

    #[test]
    fn inline_declarations() {
        let l = crate::logic(
            "CLASS lcl DEFINITION.
  PUBLIC SECTION.
    TYPES: BEGIN OF ty_row, id TYPE i, name TYPE string, END OF ty_row,
           ty_rows TYPE STANDARD TABLE OF ty_row WITH EMPTY KEY.
    DATA mt_rows TYPE ty_rows.
    METHODS run IMPORTING iv_id TYPE i.
    METHODS get RETURNING VALUE(rv) TYPE string.
    CLASS-METHODS create RETURNING VALUE(ro) TYPE REF TO lcl.
ENDCLASS.
CLASS lcl IMPLEMENTATION.
  METHOD run.
    DATA(lv_n) = lines( mt_rows ).
    DATA(lv_t) = 'abc'.
    DATA(lv_s) = |x{ iv_id }|.
    DATA(lo) = lcl=>create( ).
    DATA(lv_g) = lo->get( ).
    DATA(lt) = VALUE ty_rows( ).
    LOOP AT mt_rows INTO DATA(ls_row).
      DATA(lv_name) = ls_row-name.
    ENDLOOP.
    READ TABLE mt_rows ASSIGNING FIELD-SYMBOL(<ls>) INDEX 1.
    DATA(ls_first) = mt_rows[ 1 ].
    DATA(lv_first) = mt_rows[ id = 1 ]-name.
    DATA(lv_there) = xsdbool( line_exists( mt_rows[ id = 2 ] ) ).
    <ls>-id = lv_n.
    LOOP AT mt_rows REFERENCE INTO DATA(lr_row).
    ENDLOOP.
    LOOP AT mt_rows INTO DATA(ls_any) GROUP BY ls_any-id INTO DATA(lv_group).
    ENDLOOP.
    SELECT SINGLE matnr FROM mara INTO @DATA(lv_matnr) WHERE matnr = 'X'.
    SELECT * FROM mara INTO TABLE @DATA(lt_mara).
    SELECT COUNT(*) FROM mara INTO @DATA(lv_count).
    FIND 'a' IN lv_g MATCH COUNT DATA(lv_found).
    TRY.
      CATCH cx_sy_zerodivide INTO DATA(lx).
    ENDTRY.
    DATA(lt_ids) = VALUE ty_rows( FOR r IN mt_rows ( id = r-id ) ).
    DATA(lt_some) = FILTER #( mt_rows WHERE id = 1 ).
    DATA(lr_n) = REF #( lv_n ).
    DATA lo_any TYPE REF TO object.
    CASE TYPE OF lo_any.
      WHEN TYPE lcl INTO DATA(lo_obj).
    ENDCASE.
  ENDMETHOD.
ENDCLASS.",
        );
        let mut got: Vec<String> = Vec::new();
        fn walk(steps: &[m::Step], got: &mut Vec<String>) {
            for s in steps {
                for w in s.writes.iter().filter(|w| w.declared) {
                    let ty = w.data_type.as_ref().map(short).unwrap_or_default();
                    got.push(format!("{} {ty}", w.name));
                }
                walk(&s.children, got);
            }
        }
        walk(&l.procedures[0].body, &mut got);
        assert_eq!(
            got,
            [
                "lv_n i",
                "lv_t c(3)",
                "lv_s string",
                "lo ref lcl",
                "lv_g string",
                "lt table ty_rows of structure ty_row",
                "ls_row structure ty_row",
                "lv_name string",
                "<ls> structure ty_row",
                "ls_first structure ty_row",
                "lv_first string",
                "lv_there c(1) xsdboolean",
                "lr_row ref data of structure ty_row",
                "ls_any structure ty_row",
                "lv_group ",
                "lv_matnr named mara-matnr",
                "lt_mara table of named mara",
                "lv_count int8",
                "lv_found i",
                "lx ref cx_sy_zerodivide",
                "r structure ty_row",
                "lt_ids table ty_rows of structure ty_row",
                "lt_some table ty_rows of structure ty_row",
                "lr_n ref data of i",
                "lo_obj ref lcl",
            ]
        );
        // FOR variables: the line of the table, not the type of the result
        let for_step = l.procedures[0]
            .body
            .iter()
            .find(|s| s.text.starts_with("DATA(lt_ids)"))
            .unwrap();
        let row = for_step.reads.iter().find(|r| r.path == "r-id").unwrap();
        assert_eq!(row.data_type.as_ref().map(short).as_deref(), Some("i"));
        // a component through the field symbol
        let set = l.procedures[0]
            .body
            .iter()
            .find(|s| s.text == "<ls>-id = lv_n.")
            .unwrap();
        assert!(set
            .writes
            .iter()
            .all(|w| w.data_type.as_ref().map(short).as_deref() == Some("i")));
    }

    #[test]
    fn parameters_of_interface_and_redefined_methods() {
        let l = crate::logic(
            "INTERFACE lif_step.
  METHODS execute IMPORTING io_memory TYPE REF TO lcl_memory RETURNING VALUE(rv_count) TYPE i.
ENDINTERFACE.
CLASS lcl_memory DEFINITION.
  PUBLIC SECTION.
    DATA mv_size TYPE int8.
ENDCLASS.
CLASS lcl_base DEFINITION.
  PUBLIC SECTION.
    METHODS run IMPORTING iv_text TYPE string.
ENDCLASS.
CLASS lcl_step DEFINITION INHERITING FROM lcl_base.
  PUBLIC SECTION.
    INTERFACES lif_step.
    METHODS run REDEFINITION.
ENDCLASS.
CLASS lcl_memory IMPLEMENTATION.
ENDCLASS.
CLASS lcl_base IMPLEMENTATION.
  METHOD run.
  ENDMETHOD.
ENDCLASS.
CLASS lcl_step IMPLEMENTATION.
  METHOD lif_step~execute.
    DATA(lv_size) = io_memory->mv_size.
    DATA(lv_count) = rv_count.
  ENDMETHOD.
  METHOD run.
    DATA(lv_text) = iv_text.
  ENDMETHOD.
ENDCLASS.",
        );
        let mut got: Vec<String> = Vec::new();
        for p in &l.procedures {
            for s in &p.body {
                for w in s.writes.iter().filter(|w| w.declared) {
                    let ty = w.data_type.as_ref().map(short).unwrap_or_default();
                    got.push(format!("{} {ty}", w.name));
                }
            }
        }
        assert_eq!(got, ["lv_size int8", "lv_count i", "lv_text string"]);
    }

    #[test]
    fn type_group_names() {
        assert_eq!(type_groups_of("abap_compdescr_tab"), ["abap"]);
        assert_eq!(type_groups_of("ab_cd_ty"), ["ab", "ab_cd"]);
        assert!(type_groups_of("string_table").is_empty());
        assert!(type_groups_of("matnr").is_empty());
        assert_eq!(type_groups_of("/abc/x_ty"), ["/abc/x"]);
    }

    #[test]
    fn statement_forms_classes_and_constants() {
        let l = crate::logic(
            "CLASS lcl DEFINITION.
  PUBLIC SECTION.
    TYPES: BEGIN OF ty_row, id TYPE i, name TYPE string, END OF ty_row,
           ty_rows TYPE STANDARD TABLE OF ty_row WITH EMPTY KEY.
    DATA mt_rows TYPE ty_rows.
    METHODS run IMPORTING iv_id TYPE i.
    METHODS get RETURNING VALUE(rv) TYPE string.
    METHODS other EXPORTING ev TYPE p.
ENDCLASS.
CLASS lcl IMPLEMENTATION.
  METHOD run.
    DATA lo TYPE REF TO lcl.
    DATA lv_g TYPE string.
    ASSERT REDUCE i( INIT n = 0 FOR r IN mt_rows NEXT n = n + r-id ) > 0.
    SELECT id FROM @mt_rows AS t INTO TABLE @DATA(lt_ids).
    SELECT SINGLE * FROM mara INTO NEW @DATA(lr_mara).
    SELECT SINGLE matnr, mtart FROM mara INTO (@DATA(lv_matnr), @DATA(lv_mtart)).
    DATA(lv_flag) = abap_true.
    DATA(lv_space) = space.
    DATA(lv_abs) = abs( iv_id ).
    DATA(lv_ceil) = ceil( lv_g ).
    ASSIGN mt_rows[ 1 ] TO FIELD-SYMBOL(<r>).
    SPLIT lv_g AT ',' INTO TABLE DATA(lt_parts).
    FIND ALL OCCURRENCES OF 'a' IN lv_g RESULTS DATA(lt_res).
    APPEND VALUE #( id = 1 ) TO mt_rows REFERENCE INTO DATA(lr_new).
    lo->other( IMPORTING ev = DATA(lv_ev) ).
    DATA(lv_c) = COND #( WHEN iv_id = 1 THEN `a` ELSE `b` ).
    DATA(lv_cast) = CAST lcl( lo )->get( ).
    DATA(lt_attr) = lo->mt_rows.
    DATA(lv_ref_id) = lr_new->id.
    DATA lr_typed TYPE REF TO ty_row.
    DATA(lv_ref_name) = lr_typed->name.
    LOOP AT CAST lcl( lo )->mt_rows INTO DATA(ls_cast).
    ENDLOOP.
    DATA(lv_day) = ( 5 + CONV d( lv_g ) ) MOD 7.
    DATA(lv_div) = iv_id / 2.
    DATA(lv_mix) = iv_id * '1.5'.
    DATA(lv_pow) = iv_id ** 2.
    DATA(lv_neg) = -5.
    DATA(lv_ipow) = ipow( base = iv_id exp = 2 ).
    DATA(lv_max) = nmax( val1 = iv_id val2 = CONV decfloat34( '1.5' ) ).
    DATA(lv_sqrt) = sqrt( CONV decfloat34( '9' ) ).
    DATA(lv_sin) = sin( '30' ).
    DATA(lv_round) = round( val = iv_id dec = 2 ).
    GET BIT 1 OF `FF` INTO DATA(lv_bit).
    GET RUN TIME FIELD DATA(lv_runtime).
    GET TIME FIELD DATA(lv_time).
    GET TIME STAMP FIELD DATA(lv_stamp).
    DATA: BEGIN OF ls_s,
            comp1 TYPE i VALUE 1,
            comp2 TYPE string VALUE `hi`,
          END OF ls_s.
  ENDMETHOD.
ENDCLASS.",
        );
        let mut got: Vec<String> = Vec::new();
        for s in &l.procedures[0].body {
            for w in s.writes.iter().filter(|w| w.declared) {
                let ty = w.data_type.as_ref().map(short).unwrap_or_default();
                got.push(format!("{} {ty}", w.name));
            }
        }
        assert_eq!(
            got,
            [
                "n i",
                "r structure ty_row",
                "lt_ids table of i",
                "lr_mara ref data of named mara",
                "lv_matnr named mara-matnr",
                "lv_mtart named mara-mtart",
                "lv_flag c(1) abap_bool",
                "lv_space c(1)",
                "lv_abs i",
                "lv_ceil p",
                "<r> structure ty_row",
                "lt_parts table of string",
                "lt_res named match_result_tab",
                "lr_new ref data of structure ty_row",
                "lv_ev p(8)",
                "lv_c string",
                "lv_cast string",
                "lt_attr table lcl=>ty_rows of structure lcl=>ty_row",
                "lv_ref_id i",
                "lv_ref_name string",
                "ls_cast structure lcl=>ty_row",
                "lv_day i",
                "lv_div i",
                "lv_mix p(8)",
                "lv_pow f",
                "lv_neg i",
                "lv_ipow i",
                "lv_max decfloat34",
                "lv_sqrt decfloat34",
                "lv_sin f",
                "lv_round decfloat34",
                "lv_bit i",
                "lv_runtime i",
                "lv_time t",
                "lv_stamp p(8) timestamp",
                "comp1 i",
                "comp2 string",
            ]
        );
    }
}
