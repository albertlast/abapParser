//! The logic of procedures: statements as a tree along the control
//! structures, the conditions every statement runs under, a control flow
//! graph, and the data flow from writes to reads (reaching definitions).

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::sync::OnceLock;

use abap_model as m;
use abap_syntax::expr::{Access, Cond, CondKind, DataRef, Expr, ExprKind, Head, Link};
use abap_syntax::operands::operands;
use abap_syntax::structure;
use abap_syntax::{decl, join_tokens, File, Statement, StatementKind as K, Token};

use crate::calls::span_of;
use crate::context::statement_blocks;

/// The logic of the procedures of an ABAP source.
pub fn logic(source: &str) -> m::FileLogic {
    logic_of(&abap_syntax::parse(source))
}

/// Like [`logic`], for a parsed file.
pub fn logic_of(file: &File) -> m::FileLogic {
    let symbols = crate::symbols_of(file);
    // consecutive statements of the same processing block
    let mut groups: Vec<(m::BlockRef, Vec<usize>)> = Vec::new();
    for (i, b) in statement_blocks(file).into_iter().enumerate() {
        let Some(b) = b else { continue };
        match groups.last_mut() {
            Some((r, v)) if *r == b => v.push(i),
            _ => groups.push((b, vec![i])),
        }
    }
    let procedures = groups
        .into_iter()
        .filter_map(|(block, statements)| procedure(file, &symbols, block, &statements))
        .collect();
    m::FileLogic {
        schema_version: m::SCHEMA_VERSION,
        procedures,
    }
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
            | K::Get
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
}

fn procedure(
    file: &File,
    symbols: &m::FileSymbols,
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

    let parameters = parameters_of(symbols, &block);
    let mut ctx = Ctx {
        file,
        locals: HashSet::new(),
        parameters: parameters.iter().map(|p| p.name.clone()).collect(),
        attributes: attributes_of(symbols, &block),
    };
    for i in statements {
        let s = &file.statements[*i];
        if matches!(
            s.kind,
            K::Data | K::Static | K::Constant | K::FieldSymbol | K::Ranges | K::DataBegin
        ) {
            if let Some(v) = decl::variable(file, s) {
                ctx.locals.insert(v.name.to_ascii_lowercase());
            }
        }
    }

    let mut body = tree(&ctx, statements);
    // inline declarations are local data too
    let mut declared = BTreeSet::new();
    walk(&body, &mut |s| {
        for w in &s.writes {
            if w.declared {
                declared.insert(w.name.clone());
            }
        }
    });
    ctx.locals.extend(declared);
    walk_mut(&mut body, &mut |s| {
        for a in s.reads.iter_mut().chain(s.writes.iter_mut()) {
            a.scope = ctx.scope(&a.name, a.scope);
        }
    });
    case_conditions(&mut body);
    guards(&mut body, &[]);
    let flow = flow(&body);
    data_flow(&mut body, &flow);
    let effects = effects(&body);

    let mut locals: Vec<String> = ctx.locals.into_iter().collect();
    locals.sort();
    Some(m::Procedure {
        block,
        span,
        parameters,
        locals,
        body,
        flow,
        effects,
    })
}

impl Ctx<'_> {
    fn scope(&self, name: &str, found: m::DataScope) -> m::DataScope {
        if self.locals.contains(name) {
            m::DataScope::Local
        } else if self.parameters.contains(name) {
            m::DataScope::Parameter
        } else if found == m::DataScope::Attribute || self.attributes.contains(name) {
            m::DataScope::Attribute
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

fn attributes_of(symbols: &m::FileSymbols, block: &m::BlockRef) -> HashSet<String> {
    let Some(class) = block.class.as_deref() else {
        return HashSet::new();
    };
    symbols
        .classes
        .iter()
        .filter(|c| c.name.eq_ignore_ascii_case(class))
        .flat_map(|c| &c.attributes)
        .map(|a| a.name.to_ascii_lowercase())
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
                sources: Vec::new(),
                from_entry: false,
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
        children: Vec::new(),
    }
}

fn access(toks: &[&Token], r: &DataRef) -> m::DataAccess {
    let name = if r.attribute {
        let full = text(toks, r.start, r.pos).to_ascii_lowercase();
        full.strip_prefix("me->")
            .map(str::to_string)
            .unwrap_or(full)
    } else {
        toks[r.pos].text.to_ascii_lowercase()
    };
    let t = toks[r.pos];
    m::DataAccess {
        name: name.trim_start_matches('!').to_string(),
        path: text(toks, r.start, r.last),
        access: match r.access {
            Access::Read => m::AccessKind::Read,
            Access::Write => m::AccessKind::Write,
            Access::Change => m::AccessKind::Change,
        },
        scope: if r.attribute {
            m::DataScope::Attribute
        } else {
            m::DataScope::Other
        },
        whole: r.access == Access::Write && r.whole,
        declared: r.declared,
        sources: Vec::new(),
        from_entry: false,
        position: m::Position {
            row: t.start.row,
            col: t.start.col,
        },
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
        if is_frame(kind) {
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

// ----- guards ----------------------------------------------------------------

fn guard_of(step: &m::Step, negated: bool) -> Option<m::Guard> {
    step.condition.as_ref().map(|c| m::Guard {
        statement: step.id,
        negated,
        text: c.text.clone(),
    })
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

/// Non-local data written, and the inputs their values depend on.
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
    let inputs_of = |start: u32| -> BTreeSet<String> {
        let mut inputs = BTreeSet::new();
        let mut seen = HashSet::new();
        let mut todo = vec![start];
        while let Some(id) = todo.pop() {
            if !seen.insert(id) {
                continue;
            }
            let Some(s) = by_id.get(&id) else { continue };
            for r in &s.reads {
                if r.from_entry && r.scope != m::DataScope::Local {
                    inputs.insert(r.name.clone());
                }
                todo.extend(&r.sources);
            }
            // the conditions deciding whether the step runs
            todo.extend(s.guards.iter().map(|g| g.statement));
            todo.extend(case_of.get(&id));
        }
        inputs
    };

    let mut out: BTreeMap<String, m::Effect> = BTreeMap::new();
    walk(body, &mut |s| {
        for w in &s.writes {
            if w.scope == m::DataScope::Local || w.declared {
                continue;
            }
            let e = out.entry(w.name.clone()).or_insert_with(|| m::Effect {
                name: w.name.clone(),
                scope: w.scope,
                statements: Vec::new(),
                inputs: Vec::new(),
            });
            if !e.statements.contains(&s.id) {
                e.statements.push(s.id);
            }
        }
    });
    for e in out.values_mut() {
        let mut inputs = BTreeSet::new();
        for s in &e.statements {
            inputs.extend(inputs_of(*s));
        }
        e.inputs = inputs.into_iter().collect();
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
}
