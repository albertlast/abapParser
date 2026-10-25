//! Symbols of one source file: classes, interfaces, routines, declarations.

use std::collections::HashMap;

use abap_model as m;
use abap_syntax::decl::{self, Direction, TypingKind};
use abap_syntax::structure::{self, Block};
use abap_syntax::{File, Statement, StatementKind as K};

/// Parses `source` and extracts its symbols.
pub fn symbols(source: &str) -> m::FileSymbols {
    symbols_of(&abap_syntax::parse(source))
}

/// The AMDP additions after `METHOD name`, e.g. `BY DATABASE PROCEDURE FOR
/// HDB LANGUAGE SQLSCRIPT OPTIONS READ-ONLY USING ztab zcl_x=>meth.`
fn amdp(text: &str) -> Option<m::Amdp> {
    let words: Vec<&str> = text
        .trim_end_matches(['.', ','])
        .split_whitespace()
        .collect();
    let by = words
        .windows(2)
        .position(|w| w[0].eq_ignore_ascii_case("BY") && w[1].eq_ignore_ascii_case("DATABASE"))?;
    let mut a = m::Amdp {
        kind: String::new(),
        database: String::new(),
        language: String::new(),
        options: Vec::new(),
        using: Vec::new(),
    };
    let mut section = "KIND";
    for w in &words[by + 2..] {
        let up = w.to_ascii_uppercase();
        match up.as_str() {
            "FOR" | "LANGUAGE" | "OPTIONS" | "USING" if section != "USING" => {
                section = match up.as_str() {
                    "FOR" => "FOR",
                    "LANGUAGE" => "LANGUAGE",
                    "OPTIONS" => "OPTIONS",
                    _ => "USING",
                };
                continue;
            }
            _ => {}
        }
        match section {
            "KIND" => {
                if !a.kind.is_empty() {
                    a.kind.push('_');
                }
                a.kind.push_str(&w.to_ascii_lowercase());
            }
            "FOR" => a.database = w.to_string(),
            "LANGUAGE" => a.language = w.to_string(),
            "OPTIONS" => {
                // an option starts with a known word, others continue it
                let starts = ["READ-ONLY", "DETERMINISTIC", "SUPPRESS", "CDS"];
                match a.options.last_mut() {
                    Some(last) if !starts.contains(&up.as_str()) => {
                        last.push(' ');
                        last.push_str(w);
                    }
                    _ => a.options.push(w.to_string()),
                }
            }
            _ => a.using.push(w.to_string()),
        }
    }
    Some(a)
}

pub fn symbols_of(file: &File) -> m::FileSymbols {
    Builder { file }.build()
}

struct Builder<'a> {
    file: &'a File,
}

fn typing(t: decl::Typing) -> m::Typing {
    m::Typing {
        kind: match t.kind {
            TypingKind::Type => m::TypingKind::Type,
            TypingKind::Like => m::TypingKind::Like,
            TypingKind::TypeRefTo => m::TypingKind::TypeRefTo,
            TypingKind::LikeRefTo => m::TypingKind::LikeRefTo,
            TypingKind::Table => m::TypingKind::Table,
            TypingKind::Range => m::TypingKind::Range,
            TypingKind::LineOf => m::TypingKind::LineOf,
            TypingKind::Structure => m::TypingKind::Structure,
            TypingKind::For => m::TypingKind::For,
        },
        name: t.name,
        table: t.table.map(|tt| m::TableTyping {
            kind: tt.kind,
            unique: tt.unique,
            key: tt.key,
            line_is_ref: tt.line_is_ref,
            header_line: tt.header_line,
        }),
        length: t.length,
        decimals: t.decimals,
        text: t.text,
    }
}

fn parameter(p: decl::Parameter) -> m::Parameter {
    m::Parameter {
        name: p.name,
        direction: match p.direction {
            Direction::Importing => m::Direction::Importing,
            Direction::Exporting => m::Direction::Exporting,
            Direction::Changing => m::Direction::Changing,
            Direction::Returning => m::Direction::Returning,
            Direction::Using => m::Direction::Using,
            Direction::Tables => m::Direction::Tables,
        },
        typing: p.typing.map(typing),
        by_value: p.by_value,
        optional: p.optional,
        default: p.default,
        preferred: p.preferred,
    }
}

fn declaration_kind(kind: K) -> Option<m::DeclarationKind> {
    use m::DeclarationKind as D;
    Some(match kind {
        K::Data | K::DataBegin | K::DataEnd => D::Data,
        K::ClassData | K::ClassDataBegin | K::ClassDataEnd => D::ClassData,
        K::Static | K::StaticBegin | K::StaticEnd => D::Static,
        K::Constant | K::ConstantBegin | K::ConstantEnd => D::Constant,
        K::Type
        | K::TypeBegin
        | K::TypeEnd
        | K::TypeEnumBegin
        | K::TypeEnumEnd
        | K::TypeMesh
        | K::TypeMeshBegin
        | K::TypeMeshEnd => D::Type,
        K::TypeEnum => D::EnumValue,
        K::FieldSymbol => D::FieldSymbol,
        K::Parameter => D::Parameter,
        K::SelectOption => D::SelectOption,
        K::Ranges => D::Range,
        K::Tables => D::Tables,
        K::Nodes => D::Node,
        K::Infotypes => D::Infotype,
        K::IncludeType => D::Include,
        _ => return None,
    })
}

fn is_begin(kind: K) -> bool {
    matches!(
        kind,
        K::DataBegin
            | K::ClassDataBegin
            | K::StaticBegin
            | K::ConstantBegin
            | K::TypeBegin
            | K::TypeEnumBegin
            | K::TypeMeshBegin
    )
}

fn is_end(kind: K) -> bool {
    matches!(
        kind,
        K::DataEnd
            | K::ClassDataEnd
            | K::StaticEnd
            | K::ConstantEnd
            | K::TypeEnd
            | K::TypeEnumEnd
            | K::TypeMeshEnd
    )
}

/// Collects declarations, nesting BEGIN OF ... END OF structures.
#[derive(Default)]
struct Declarations {
    open: Vec<m::Declaration>,
    done: Vec<m::Declaration>,
}

impl Declarations {
    fn add(&mut self, b: &Builder, index: usize, visibility: Option<m::Visibility>) {
        let s = &b.file.statements[index];
        let Some(kind) = declaration_kind(s.kind) else {
            return;
        };
        if is_end(s.kind) {
            if let Some(mut d) = self.open.pop() {
                d.span.end = b.span(index, index).end;
                self.finish(d);
            }
            return;
        }
        let mut d = if s.kind == K::IncludeType {
            let (name, _) = decl::first_name(b.file, s, 2).unwrap_or_default();
            m::Declaration {
                typing: Some(m::Typing {
                    kind: m::TypingKind::Type,
                    name: Some(name.clone()),
                    table: None,
                    length: None,
                    decimals: None,
                    text: decl::text_after(b.file, s, 1),
                }),
                ..empty_declaration(name, kind, b.span(index, index))
            }
        } else if s.kind == K::Infotypes {
            let Some(d) = infotype(b.file, s, b.span(index, index)) else {
                return;
            };
            d
        } else {
            let Some(v) = decl::variable(b.file, s) else {
                return;
            };
            m::Declaration {
                typing: v.typing.map(typing),
                value: v.value,
                read_only: v.read_only || kind == m::DeclarationKind::Constant,
                ..empty_declaration(v.name, kind, b.span(index, index))
            }
        };
        d.visibility = visibility;
        if s.kind == K::TypeEnum && !self.open.last().is_some_and(|o| o.is_enum) {
            d.kind = m::DeclarationKind::Type;
        }
        if is_begin(s.kind) {
            d.is_structure = s.kind != K::TypeEnumBegin;
            d.is_enum = s.kind == K::TypeEnumBegin;
            self.open.push(d);
        } else {
            self.finish(d);
        }
    }

    fn finish(&mut self, d: m::Declaration) {
        match self.open.last_mut() {
            Some(parent) => parent.components.push(d),
            None => self.done.push(d),
        }
    }

    fn into_vec(mut self) -> Vec<m::Declaration> {
        while let Some(d) = self.open.pop() {
            self.finish(d);
        }
        self.done
    }
}

/// `INFOTYPES nnnn [NAME name] ...`: the table `pnnnn`, or `name`, of
/// infotype structures `Pnnnn` with header line.
fn infotype(file: &File, s: &Statement, span: m::Span) -> Option<m::Declaration> {
    let words: Vec<&str> = s
        .tokens
        .iter()
        .map(|t| file.tokens()[*t].text.as_str())
        .collect();
    let number = words
        .get(1)
        .filter(|w| w.bytes().all(|b| b.is_ascii_digit()))?;
    let name = words
        .iter()
        .position(|w| w.eq_ignore_ascii_case("NAME"))
        .and_then(|i| words.get(i + 1))
        .map(|n| n.to_string())
        .unwrap_or_else(|| format!("p{number}"));
    Some(m::Declaration {
        typing: Some(m::Typing {
            kind: m::TypingKind::Table,
            name: Some(format!("P{number}")),
            table: Some(m::TableTyping {
                kind: None,
                unique: None,
                key: None,
                line_is_ref: false,
                header_line: true,
            }),
            length: None,
            decimals: None,
            text: decl::text_after(file, s, 1),
        }),
        ..empty_declaration(name, m::DeclarationKind::Infotype, span)
    })
}

fn empty_declaration(name: String, kind: m::DeclarationKind, span: m::Span) -> m::Declaration {
    m::Declaration {
        name,
        kind,
        visibility: None,
        typing: None,
        value: None,
        read_only: false,
        is_structure: false,
        is_enum: false,
        components: Vec::new(),
        data_type: None,
        span,
    }
}

/// Fills `data_type` of all declarations: in the scope of their program,
/// class, method, form or function module, LIKE and local types resolved.
pub(crate) fn resolve_types(s: &mut m::FileSymbols) {
    use crate::types::Types;
    fn fill(out: &mut [m::Declaration], types: &Types, decls: &[m::Declaration]) {
        for (d, from) in out.iter_mut().zip(decls) {
            d.data_type = types.of_declaration(from);
            fill(&mut d.components, types, &from.components);
        }
    }
    let block = |kind: &str, name: &str, class: Option<&str>| m::BlockRef {
        kind: kind.to_string(),
        name: name.to_string(),
        class: class.map(str::to_string),
    };
    let snapshot = s.clone();
    let program = Types::of_program(&snapshot);
    fill(&mut s.declarations, &program, &snapshot.declarations);
    for (c, from) in s.classes.iter_mut().zip(&snapshot.classes) {
        let attributes = Types::of_program(&snapshot)
            .with(&from.types)
            .with(&from.attributes);
        fill(&mut c.attributes, &attributes, &from.attributes);
        fill(&mut c.types, &attributes, &from.types);
        if let (Some(imp), Some(imp_from)) = (&mut c.implementation, &from.implementation) {
            for (mi, mi_from) in imp.methods.iter_mut().zip(&imp_from.methods) {
                let types = Types::of_block(&snapshot, &block("method", &mi.name, Some(&c.name)));
                fill(&mut mi.locals, &types, &mi_from.locals);
            }
        }
    }
    for (i, from) in s.interfaces.iter_mut().zip(&snapshot.interfaces) {
        let attributes = Types::of_program(&snapshot)
            .with(&from.types)
            .with(&from.attributes);
        fill(&mut i.attributes, &attributes, &from.attributes);
        fill(&mut i.types, &attributes, &from.types);
    }
    for (kind, routines, routines_from) in [
        ("form", &mut s.forms, &snapshot.forms),
        (
            "function_module",
            &mut s.function_modules,
            &snapshot.function_modules,
        ),
    ] {
        for (r, from) in routines.iter_mut().zip(routines_from) {
            let types = Types::of_block(&snapshot, &block(kind, &r.name, None));
            fill(&mut r.locals, &types, &from.locals);
        }
    }
    for (o, from) in s
        .orphan_implementations
        .iter_mut()
        .zip(&snapshot.orphan_implementations)
    {
        for (mi, mi_from) in o
            .implementation
            .methods
            .iter_mut()
            .zip(&from.implementation.methods)
        {
            let types = Types::of_block(&snapshot, &block("method", &mi.name, Some(&o.class)));
            fill(&mut mi.locals, &types, &mi_from.locals);
        }
    }
}

/// Members of a class or interface definition.
#[derive(Default)]
struct Members {
    interfaces: Vec<m::ImplementedInterface>,
    methods: Vec<m::Method>,
    declarations: Declarations,
    events: Vec<m::Event>,
    aliases: Vec<m::Alias>,
}

impl<'a> Builder<'a> {
    fn statement(&self, i: usize) -> &'a Statement {
        &self.file.statements[i]
    }

    fn span(&self, from: usize, to: usize) -> m::Span {
        let tokens = &self.file.lexed.tokens;
        let first = &tokens[self.statement(from).tokens[0]];
        let last = &tokens[*self.statement(to).tokens.last().unwrap()];
        let end = last.end();
        m::Span {
            start: m::Position {
                row: first.start.row,
                col: first.start.col,
            },
            end: m::Position {
                row: end.row,
                col: end.col,
            },
        }
    }

    fn block_span(&self, b: &Block) -> m::Span {
        self.span(b.start, b.last())
    }

    fn build(&self) -> m::FileSymbols {
        let file = self.file;
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

        let blocks = structure::blocks(&file.statements);
        let processing: HashMap<usize, &Block> = blocks
            .iter()
            .filter(|b| {
                matches!(
                    self.statement(b.start).kind,
                    K::ClassDefinition
                        | K::ClassImplementation
                        | K::Interface
                        | K::Form
                        | K::FunctionModule
                        | K::Module
                        | K::Define
                )
            })
            .map(|b| (b.start, b))
            .collect();

        let mut implementations: Vec<(String, m::ClassImplementation)> = Vec::new();
        let mut globals = Declarations::default();
        let mut event: Option<(usize, String)> = None;
        let close_event =
            |event: &mut Option<(usize, String)>, out: &mut m::FileSymbols, last: usize| {
                if let Some((start, name)) = event.take() {
                    out.event_blocks.push(m::EventBlock {
                        event: name,
                        span: self.span(start, last.max(start)),
                    });
                }
            };

        let mut i = 0;
        while i < file.statements.len() {
            let s = self.statement(i);
            if let Some(b) = processing.get(&i) {
                if matches!(
                    s.kind,
                    K::ClassImplementation | K::Form | K::FunctionModule | K::Module
                ) {
                    close_event(&mut event, &mut out, i.saturating_sub(1));
                }
                match s.kind {
                    K::ClassDefinition => {
                        if let Some(c) = self.class(b) {
                            out.classes.push(c);
                        }
                    }
                    K::Interface => {
                        if let Some(intf) = self.interface(b) {
                            out.interfaces.push(intf);
                        }
                    }
                    K::ClassImplementation => {
                        let name = decl::first_name(file, s, 1)
                            .map(|n| n.0)
                            .unwrap_or_default();
                        implementations.push((name, self.class_implementation(b)));
                    }
                    K::Form => {
                        if let Some(sig) = decl::form_def(file, s) {
                            out.forms.push(self.routine(sig, b));
                        }
                    }
                    K::FunctionModule => {
                        if let Some(sig) = decl::function_def(file, i) {
                            out.function_modules.push(self.routine(sig, b));
                        }
                    }
                    K::Module => {
                        let name = decl::first_name(file, s, 1)
                            .map(|n| n.0)
                            .unwrap_or_default();
                        let kind = decl::text_after(file, s, 2).to_ascii_lowercase();
                        out.modules.push(m::DialogModule {
                            name,
                            kind: (!kind.is_empty()).then_some(kind),
                            span: self.block_span(b),
                        });
                    }
                    K::Define => {
                        let name = decl::first_name(file, s, 1)
                            .map(|n| n.0)
                            .unwrap_or_default();
                        out.macros.push(m::NamedSpan {
                            name,
                            span: self.block_span(b),
                        });
                    }
                    _ => {}
                }
                i = b.last() + 1;
                continue;
            }

            match s.kind {
                K::Report
                | K::Program
                | K::FunctionPool
                | K::ClassPool
                | K::InterfacePool
                | K::TypePool => {
                    let text = decl::text_after(file, s, 0);
                    out.program = Some(m::ProgramHeader {
                        kind: text
                            .split_whitespace()
                            .next()
                            .unwrap_or("")
                            .to_ascii_lowercase(),
                        name: decl::first_name(file, s, 1)
                            .map(|n| n.0)
                            .unwrap_or_default(),
                        span: self.span(i, i),
                    });
                }
                K::StartOfSelection
                | K::EndOfSelection
                | K::Initialization
                | K::TopOfPage
                | K::EndOfPage
                | K::LoadOfProgram
                | K::AtSelectionScreen
                | K::AtLineSelection
                | K::AtUserCommand
                | K::AtPF
                | K::Get => {
                    close_event(&mut event, &mut out, i.saturating_sub(1));
                    event = Some((i, file.statement_text(s).trim_end_matches('.').to_string()));
                }
                _ => globals.add(self, i, None),
            }
            i += 1;
        }
        close_event(
            &mut event,
            &mut out,
            file.statements.len().saturating_sub(1),
        );
        out.declarations = globals.into_vec();

        for (index, s) in file.statements.iter().enumerate() {
            if s.kind == K::Include {
                if let Some((name, _)) = decl::first_name(file, s, 1) {
                    out.includes.push(m::NamedSpan {
                        name,
                        span: self.span(index, index),
                    });
                }
            }
        }

        for (name, imp) in implementations {
            match out
                .classes
                .iter_mut()
                .find(|c| c.name.eq_ignore_ascii_case(&name))
            {
                Some(c) if c.implementation.is_none() => c.implementation = Some(imp),
                _ => out.orphan_implementations.push(m::OrphanImplementation {
                    class: name,
                    implementation: imp,
                }),
            }
        }
        resolve_types(&mut out);
        out
    }

    fn members(&self, b: &Block, default_visibility: m::Visibility) -> Members {
        let mut members = Members::default();
        let mut visibility = default_visibility;
        let end = b.end.unwrap_or(b.last() + 1);
        for i in b.start + 1..end {
            let s = self.statement(i);
            match s.kind {
                K::Public => visibility = m::Visibility::Public,
                K::Protected => visibility = m::Visibility::Protected,
                K::Private => visibility = m::Visibility::Private,
                K::MethodDef => {
                    if let Some(d) = decl::method_def(self.file, s) {
                        members.methods.push(m::Method {
                            name: d.name,
                            visibility,
                            is_static: d.is_static,
                            is_abstract: d.is_abstract,
                            is_final: d.is_final,
                            is_redefinition: d.is_redefinition,
                            is_for_testing: d.is_for_testing,
                            default: d.default,
                            event_handler: d
                                .event
                                .map(|(class, event)| m::EventHandler { class, event }),
                            rap_handler: d.rap_handler,
                            parameters: d.parameters.into_iter().map(parameter).collect(),
                            raising: d.raising,
                            exceptions: d.exceptions,
                            span: self.span(i, i),
                        });
                    }
                }
                K::Events => {
                    if let Some(e) = decl::event_def(self.file, s) {
                        members.events.push(m::Event {
                            name: e.name,
                            visibility,
                            is_static: e.is_static,
                            parameters: e.parameters.into_iter().map(parameter).collect(),
                            span: self.span(i, i),
                        });
                    }
                }
                K::Aliases => {
                    if let Some(a) = decl::alias_def(self.file, s) {
                        members.aliases.push(m::Alias {
                            name: a.name,
                            component: a.component,
                            visibility,
                            span: self.span(i, i),
                        });
                    }
                }
                K::InterfaceDef => {
                    if let Some(d) = decl::interfaces_def(self.file, s) {
                        members.interfaces.push(m::ImplementedInterface {
                            name: d.name,
                            visibility,
                            partially_implemented: d.partially_implemented,
                            abstract_methods: d.abstract_methods,
                            final_methods: d.final_methods,
                            all_abstract: d.all_abstract,
                            all_final: d.all_final,
                        });
                    }
                }
                _ => members.declarations.add(self, i, Some(visibility)),
            }
        }
        members
    }

    fn class(&self, b: &Block) -> Option<m::Class> {
        let d = decl::class_def(self.file, self.statement(b.start))?;
        let members = self.members(b, m::Visibility::Public);
        let (types, attributes) = split_types(members.declarations.into_vec());
        Some(m::Class {
            name: d.name,
            is_global: d.is_public,
            is_final: d.is_final,
            is_abstract: d.is_abstract,
            is_for_testing: d.is_for_testing,
            superclass: d.superclass,
            create: d.create,
            friends: d.friends,
            behavior_of: d.behavior_of,
            interfaces: members.interfaces,
            methods: members.methods,
            attributes,
            types,
            events: members.events,
            aliases: members.aliases,
            implementation: None,
            span: self.block_span(b),
        })
    }

    fn interface(&self, b: &Block) -> Option<m::Interface> {
        let (name, _, is_global) = decl::interface_def(self.file, self.statement(b.start))?;
        let members = self.members(b, m::Visibility::Public);
        let (types, attributes) = split_types(members.declarations.into_vec());
        Some(m::Interface {
            name,
            is_global,
            interfaces: members.interfaces,
            methods: members.methods,
            attributes,
            types,
            events: members.events,
            aliases: members.aliases,
            span: self.block_span(b),
        })
    }

    fn locals(&self, b: &Block) -> Vec<m::Declaration> {
        let mut d = Declarations::default();
        let end = b.end.unwrap_or(b.last() + 1);
        for i in b.start + 1..end {
            d.add(self, i, None);
        }
        d.into_vec()
    }

    fn class_implementation(&self, b: &Block) -> m::ClassImplementation {
        let methods = b
            .children
            .iter()
            .filter(|c| self.statement(c.start).kind == K::MethodImplementation)
            .map(|c| {
                let s = self.statement(c.start);
                let name = decl::first_name(self.file, s, 1)
                    .map(|n| n.0)
                    .unwrap_or_default();
                let rest = decl::text_after(self.file, s, 2);
                let amdp = amdp(&rest);
                m::MethodImplementation {
                    name,
                    is_amdp: amdp.is_some(),
                    amdp,
                    locals: self.locals(c),
                    span: self.block_span(c),
                }
            })
            .collect();
        m::ClassImplementation {
            methods,
            span: self.block_span(b),
        }
    }

    fn routine(&self, sig: decl::Signature, b: &Block) -> m::Routine {
        m::Routine {
            name: sig.name,
            parameters: sig.parameters.into_iter().map(parameter).collect(),
            raising: sig.raising,
            exceptions: sig.exceptions,
            locals: self.locals(b),
            span: self.block_span(b),
        }
    }
}

fn split_types(all: Vec<m::Declaration>) -> (Vec<m::Declaration>, Vec<m::Declaration>) {
    all.into_iter()
        .partition(|d| d.kind == m::DeclarationKind::Type)
}

#[cfg(test)]
mod tests {
    use super::*;

    const CLASS: &str = r#"
CLASS zcl_demo DEFINITION PUBLIC FINAL CREATE PUBLIC INHERITING FROM zcl_base.
  PUBLIC SECTION.
    INTERFACES zif_demo.
    ALIASES run FOR zif_demo~run.
    TYPES: BEGIN OF ty_row,
             id   TYPE i,
             name TYPE string,
           END OF ty_row,
           ty_rows TYPE STANDARD TABLE OF ty_row WITH DEFAULT KEY.
    CONSTANTS c_max TYPE i VALUE 10.
    METHODS constructor IMPORTING iv_id TYPE i.
    CLASS-METHODS create RETURNING VALUE(ro_obj) TYPE REF TO zcl_demo.
    EVENTS changed EXPORTING VALUE(ev_id) TYPE i.
  PROTECTED SECTION.
    DATA mv_id TYPE i READ-ONLY.
  PRIVATE SECTION.
    CLASS-DATA gt_cache TYPE ty_rows.
ENDCLASS.

CLASS zcl_demo IMPLEMENTATION.
  METHOD constructor.
    DATA lv_x TYPE i.
    super->constructor( ).
    mv_id = iv_id.
  ENDMETHOD.
  METHOD create.
    ro_obj = NEW #( 1 ).
  ENDMETHOD.
  METHOD zif_demo~run.
  ENDMETHOD.
ENDCLASS.
"#;

    #[test]
    fn class_symbols() {
        let s = symbols(CLASS);
        assert_eq!(s.classes.len(), 1);
        let c = &s.classes[0];
        assert_eq!(c.name, "zcl_demo");
        assert!(c.is_global && c.is_final);
        assert_eq!(c.superclass.as_deref(), Some("zcl_base"));
        assert_eq!(c.interfaces[0].name, "zif_demo");
        assert_eq!(c.aliases[0].component, "zif_demo~run");

        let names: Vec<_> = c
            .methods
            .iter()
            .map(|m| (m.name.as_str(), m.is_static))
            .collect();
        assert_eq!(names, [("constructor", false), ("create", true)]);
        assert_eq!(
            c.methods[1].parameters[0].direction,
            m::Direction::Returning
        );

        let types: Vec<_> = c.types.iter().map(|t| t.name.as_str()).collect();
        assert_eq!(types, ["ty_row", "ty_rows"]);
        assert!(c.types[0].is_structure);
        assert_eq!(c.types[0].components.len(), 2);

        let attrs: Vec<_> = c
            .attributes
            .iter()
            .map(|a| (a.name.as_str(), a.kind, a.visibility.unwrap()))
            .collect();
        assert_eq!(
            attrs,
            [
                ("c_max", m::DeclarationKind::Constant, m::Visibility::Public),
                ("mv_id", m::DeclarationKind::Data, m::Visibility::Protected),
                (
                    "gt_cache",
                    m::DeclarationKind::ClassData,
                    m::Visibility::Private
                ),
            ]
        );
        assert!(c.attributes[1].read_only);
        assert_eq!(c.events[0].name, "changed");

        let imp = c.implementation.as_ref().unwrap();
        let impls: Vec<_> = imp.methods.iter().map(|m| m.name.as_str()).collect();
        assert_eq!(impls, ["constructor", "create", "zif_demo~run"]);
        assert_eq!(imp.methods[0].locals[0].name, "lv_x");
        assert_eq!(c.span.start, m::Position { row: 2, col: 1 });
        assert_eq!(imp.span.end.row, 32);
    }

    #[test]
    fn report_symbols() {
        let src = "REPORT zdemo.\nTABLES t001.\nPARAMETERS p_bukrs TYPE bukrs.\nSELECT-OPTIONS s_date FOR sy-datum.\n\
                   DATA gv_count TYPE i.\nINCLUDE zdemo_top.\nSTART-OF-SELECTION.\n  PERFORM run USING p_bukrs.\n\
                   FORM run USING iv_bukrs TYPE bukrs.\n  DATA lv_x TYPE i.\nENDFORM.";
        let s = symbols(src);
        let p = s.program.as_ref().unwrap();
        assert_eq!((p.kind.as_str(), p.name.as_str()), ("report", "zdemo"));
        let decls: Vec<_> = s
            .declarations
            .iter()
            .map(|d| (d.name.as_str(), d.kind))
            .collect();
        assert_eq!(
            decls,
            [
                ("t001", m::DeclarationKind::Tables),
                ("p_bukrs", m::DeclarationKind::Parameter),
                ("s_date", m::DeclarationKind::SelectOption),
                ("gv_count", m::DeclarationKind::Data),
            ]
        );
        assert_eq!(s.includes[0].name, "zdemo_top");
        assert_eq!(s.event_blocks[0].event, "START-OF-SELECTION");
        assert_eq!(s.event_blocks[0].span.end.row, 8);
        assert_eq!(s.forms[0].name, "run");
        assert_eq!(s.forms[0].locals[0].name, "lv_x");
    }
}
