//! Types of data objects, as far as the sources tell them: declared types
//! with local TYPES resolved, LIKE resolved through the data object it
//! names, system fields, and the types of expressions, for inline
//! declarations. Dictionary types stay names: `matnr`, `mara-matnr`.

use std::cell::RefCell;
use std::collections::HashMap;

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
                    if let Some(md) = c.methods.iter().find(|m| eq(&m.name, method)) {
                        parameters.extend(&md.parameters);
                    }
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
            inline: RefCell::new(HashMap::new()),
        }
    }

    /// Only the data of the program, for its declarations.
    pub(crate) fn of_program(symbols: &'a m::FileSymbols) -> Self {
        Types {
            data: vec![&symbols.declarations],
            parameters: Vec::new(),
            symbols,
            class: None,
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

    fn typing(&self, t: &m::Typing, depth: u32) -> Option<Found<'a>> {
        if depth > MAX_DEPTH {
            return None;
        }
        let name = t.name.as_deref()?;
        let like = t.text.trim_start().to_ascii_uppercase().starts_with("LIKE");
        let plain = |ty| Found { ty, decl: None };
        Some(match t.kind {
            m::TypingKind::Type => self.type_named(
                name,
                number(t.length.as_deref()),
                number(t.decimals.as_deref()),
                depth,
            ),
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
        Found {
            ty: named(&lower),
            decl: None,
        }
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
        if lower.contains("->") || lower.contains("=>") {
            return None;
        }
        if let Some(f) = system_field(lower) {
            return Some(Found {
                ty: system(f),
                decl: None,
            });
        }
        let mut parts = lower.split('-');
        let first = parts.next()?;
        let root = first.trim_end_matches('~').trim_end_matches("[]");
        let mut found = self.root(root, depth)?;
        found = self.lines(found, first)?;
        for part in parts {
            let component = part.trim_end_matches('~');
            found = match found.decl {
                Some(d) if !d.components.is_empty() => {
                    let c = d
                        .components
                        .iter()
                        .find(|c| c.name.eq_ignore_ascii_case(component))?;
                    self.declared(c, depth + 1)?
                }
                // a dictionary structure: its column, like mara-matnr
                _ if found.ty.kind == "named" => Found {
                    ty: named(&format!("{}-{component}", found.ty.name.as_deref()?)),
                    decl: None,
                },
                _ => return None,
            };
            found = self.lines(found, part)?;
        }
        Some(found)
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
        let d = self.find(name, false)?;
        self.declared(d, depth)
    }

    /// The line type of a table or range, or what a reference refers to.
    pub(crate) fn line_of(&self, ty: &m::DataType) -> Option<m::DataType> {
        match ty.kind.as_str() {
            "table" | "range" | "ref" => ty.line.as_deref().cloned(),
            _ => None,
        }
    }

    /// The RETURNING type of a method of a class of the sources.
    pub(crate) fn returning(&self, class: Option<&str>, method: &str) -> Option<m::DataType> {
        let class = class.map(str::to_string).or_else(|| self.class.clone())?;
        let c = self
            .symbols
            .classes
            .iter()
            .find(|c| c.name.eq_ignore_ascii_case(&class))?;
        let md = c
            .methods
            .iter()
            .find(|m| m.name.eq_ignore_ascii_case(method))?;
        let p = md
            .parameters
            .iter()
            .find(|p| p.direction == m::Direction::Returning)?;
        let found = Types {
            data: vec![&c.attributes, &c.types, &self.symbols.declarations],
            parameters: Vec::new(),
            symbols: self.symbols,
            class: None,
            inline: RefCell::new(HashMap::new()),
        }
        .typing(p.typing.as_ref()?, 0)?;
        Some(found.ty)
    }

    /// The type of a type name in an expression, like the type of
    /// `VALUE ty_rows( )`.
    pub(crate) fn type_of_name(&self, name: &str) -> m::DataType {
        self.type_named(name, None, None, 0).ty
    }
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

/// The result types of built-in functions.
pub(crate) fn builtin_function(name: &str) -> Option<m::DataType> {
    let lower = name.to_ascii_lowercase();
    Some(match lower.as_str() {
        "lines" | "strlen" | "numofchar" | "xstrlen" | "line_index" | "find" | "find_end"
        | "find_any_of" | "find_any_not_of" | "count" | "count_any_of" | "count_any_not_of"
        | "charlen" | "dbmaxlen" => builtin("i", None, None),
        "xsdbool" | "boolc" | "boolx" | "line_exists" => m::DataType {
            name: Some("abap_bool".to_string()),
            ..builtin("c", Some(1), None)
        },
        "condense" | "to_upper" | "to_lower" | "to_mixed" | "from_mixed" | "translate"
        | "replace" | "substring" | "substring_after" | "substring_before" | "substring_from"
        | "substring_to" | "concat_lines_of" | "reverse" | "shift_left" | "shift_right"
        | "segment" | "repeat" | "escape" | "match" | "cmax" | "cmin" => {
            builtin("string", None, None)
        }
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
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
                "lv_there c(1) abap_bool",
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
}
