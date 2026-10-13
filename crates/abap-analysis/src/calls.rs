//! Calls of a file: method calls (functional and CALL METHOD), constructors,
//! function modules, forms, programs, transactions, includes.

use std::collections::HashMap;

use abap_model as m;
use abap_syntax::{File, Statement, StatementKind as K, Token, TokenKind};

use crate::context::statement_blocks;

/// Type information about classes and interfaces, possibly outside the file.
pub trait TypeResolver {
    /// Class or interface of the RETURNING parameter, when typed `TYPE REF TO`.
    fn returning_class(&self, class: &str, method: &str) -> Option<String>;
    /// Class or interface of an attribute typed `TYPE REF TO`.
    fn attribute_class(&self, class: &str, attribute: &str) -> Option<String>;
    fn superclass(&self, class: &str) -> Option<String>;
    /// Parameters typed `TYPE REF TO` of a method, as (name, class); `None`
    /// when the method is unknown. Follows interfaces, aliases and, for
    /// redefinitions, the superclass.
    fn method_parameters(&self, class: &str, method: &str) -> Option<Vec<(String, String)>>;
}

/// Resolves within one set of symbols, e.g. one file.
pub struct SymbolsResolver<'a> {
    pub symbols: &'a m::FileSymbols,
    /// asked when the symbols do not know a class
    pub fallback: Option<&'a dyn TypeResolver>,
}

/// Methods, attributes and interfaces of a class or interface.
pub(crate) struct TypeInfo<'a> {
    pub methods: &'a [m::Method],
    pub attributes: &'a [m::Declaration],
    pub interfaces: Vec<&'a str>,
    pub aliases: &'a [m::Alias],
    pub superclass: Option<&'a str>,
}

pub(crate) fn type_info<'a>(symbols: &'a m::FileSymbols, name: &str) -> Option<TypeInfo<'a>> {
    if let Some(c) = symbols
        .classes
        .iter()
        .find(|c| c.name.eq_ignore_ascii_case(name))
    {
        return Some(TypeInfo {
            methods: &c.methods,
            attributes: &c.attributes,
            interfaces: c.interfaces.iter().map(|i| i.name.as_str()).collect(),
            aliases: &c.aliases,
            superclass: c.superclass.as_deref(),
        });
    }
    symbols
        .interfaces
        .iter()
        .find(|i| i.name.eq_ignore_ascii_case(name))
        .map(|i| TypeInfo {
            methods: &i.methods,
            attributes: &i.attributes,
            interfaces: i.interfaces.iter().map(|x| x.name.as_str()).collect(),
            aliases: &i.aliases,
            superclass: None,
        })
}

fn ref_class(t: &Option<m::Typing>) -> Option<String> {
    match t {
        Some(t) if t.kind == m::TypingKind::TypeRefTo => t.name.clone(),
        _ => None,
    }
}

impl TypeResolver for SymbolsResolver<'_> {
    fn returning_class(&self, class: &str, method: &str) -> Option<String> {
        let Some(info) = type_info(self.symbols, class) else {
            return self.fallback.and_then(|f| f.returning_class(class, method));
        };
        let (intf, meth) = match method.split_once('~') {
            Some((i, mth)) => (Some(i), mth),
            None => (None, method),
        };
        if intf.is_none() {
            if let Some(md) = info
                .methods
                .iter()
                .find(|x| x.name.eq_ignore_ascii_case(meth))
            {
                return md
                    .parameters
                    .iter()
                    .find(|p| p.direction == m::Direction::Returning)
                    .and_then(|p| ref_class(&p.typing));
            }
            if let Some(a) = info
                .aliases
                .iter()
                .find(|a| a.name.eq_ignore_ascii_case(meth))
            {
                if let Some((i, mth)) = a.component.split_once('~') {
                    return self.returning_class(i, mth);
                }
            }
        }
        for i in &info.interfaces {
            if intf.is_none_or(|x| x.eq_ignore_ascii_case(i)) {
                if let Some(c) = self.returning_class(i, meth) {
                    return Some(c);
                }
            }
        }
        let superclass = info.superclass.map(str::to_string);
        superclass.and_then(|s| self.returning_class(&s, method))
    }

    fn attribute_class(&self, class: &str, attribute: &str) -> Option<String> {
        let Some(info) = type_info(self.symbols, class) else {
            return self
                .fallback
                .and_then(|f| f.attribute_class(class, attribute));
        };
        if let Some(a) = info
            .attributes
            .iter()
            .find(|a| a.name.eq_ignore_ascii_case(attribute))
        {
            return ref_class(&a.typing);
        }
        let superclass = info.superclass.map(str::to_string);
        superclass.and_then(|s| self.attribute_class(&s, attribute))
    }

    fn superclass(&self, class: &str) -> Option<String> {
        match type_info(self.symbols, class) {
            Some(info) => info.superclass.map(str::to_string),
            None => self.fallback.and_then(|f| f.superclass(class)),
        }
    }

    fn method_parameters(&self, class: &str, method: &str) -> Option<Vec<(String, String)>> {
        let Some(info) = type_info(self.symbols, class) else {
            return self
                .fallback
                .and_then(|f| f.method_parameters(class, method));
        };
        if let Some((intf, meth)) = method.split_once('~') {
            return self.method_parameters(intf, meth);
        }
        if let Some(md) = info
            .methods
            .iter()
            .find(|x| x.name.eq_ignore_ascii_case(method))
        {
            if !md.is_redefinition {
                return Some(
                    md.parameters
                        .iter()
                        .filter_map(|p| ref_class(&p.typing).map(|c| (p.name.clone(), c)))
                        .collect(),
                );
            }
        } else if let Some(a) = info
            .aliases
            .iter()
            .find(|a| a.name.eq_ignore_ascii_case(method))
        {
            if let Some((i, mth)) = a.component.split_once('~') {
                return self.method_parameters(i, mth);
            }
        } else {
            for i in &info.interfaces {
                if let Some(p) = self.method_parameters(i, method) {
                    return Some(p);
                }
            }
        }
        let superclass = info.superclass.map(str::to_string);
        superclass.and_then(|sc| self.method_parameters(&sc, method))
    }
}

/// Built-in functions; a bare call is one unless the class has a method of
/// that name.
const BUILTINS: &[&str] = &[
    "abs",
    "ceil",
    "floor",
    "frac",
    "sign",
    "trunc",
    "ipow",
    "nmax",
    "nmin",
    "acos",
    "asin",
    "atan",
    "cos",
    "cosh",
    "exp",
    "log",
    "log10",
    "sin",
    "sinh",
    "sqrt",
    "tan",
    "tanh",
    "round",
    "rescale",
    "boolc",
    "boolx",
    "xsdbool",
    "contains",
    "contains_any_of",
    "contains_any_not_of",
    "matches",
    "line_exists",
    "line_index",
    "lines",
    "charlen",
    "dbmaxlen",
    "numofchar",
    "strlen",
    "xstrlen",
    "char_off",
    "cmax",
    "cmin",
    "concat_lines_of",
    "condense",
    "count",
    "count_any_of",
    "count_any_not_of",
    "distance",
    "escape",
    "find",
    "find_end",
    "find_any_of",
    "find_any_not_of",
    "insert",
    "match",
    "repeat",
    "replace",
    "reverse",
    "segment",
    "shift_left",
    "shift_right",
    "substring",
    "substring_after",
    "substring_before",
    "substring_from",
    "substring_to",
    "to_lower",
    "to_mixed",
    "from_mixed",
    "to_upper",
    "translate",
    "utclong_add",
    "utclong_current",
    "utclong_diff",
    "bit-set",
    "xsdbool",
];

/// Constructor operators: `CONV type( ... )` is no method call.
const CONSTRUCTOR_OPERATORS: &[&str] = &[
    "CAST",
    "CONV",
    "VALUE",
    "REF",
    "EXACT",
    "CORRESPONDING",
    "COND",
    "SWITCH",
    "REDUCE",
    "FILTER",
];

fn is_declaration(kind: K) -> bool {
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
            | K::Constant
            | K::ConstantBegin
            | K::ConstantEnd
            | K::Type
            | K::TypeBegin
            | K::TypeEnd
            | K::TypeEnum
            | K::TypeEnumBegin
            | K::TypeEnumEnd
            | K::FieldSymbol
            | K::MethodDef
            | K::Events
            | K::Parameter
            | K::SelectOption
            | K::Comment
            | K::MacroContent
            | K::NativeSQL
            | K::Define
            | K::EndOfDefinition
    )
}

/// Variables with a known class or interface type, per processing block.
struct Scope {
    vars: HashMap<String, String>,
    class: Option<String>,
}

/// One element of a call chain: `-> name` or `=> name( ... )`.
#[derive(Debug)]
struct Unit {
    arrow: Option<TokenKind>,
    name: String,
    is_call: bool,
    text_end: usize,
}

pub(crate) struct CallExtractor<'a> {
    pub file: &'a File,
    pub symbols: &'a m::FileSymbols,
    pub resolver: &'a dyn TypeResolver,
}

impl<'a> CallExtractor<'a> {
    pub fn calls(&self) -> Vec<m::Call> {
        let blocks = statement_blocks(self.file);
        let mut scopes: HashMap<String, Scope> = HashMap::new();
        let mut out = Vec::new();
        for (i, s) in self.file.statements.iter().enumerate() {
            if is_declaration(s.kind) {
                continue;
            }
            let block = blocks[i].clone();
            let key = block
                .as_ref()
                .map(|b| format!("{}|{}|{}", b.kind, b.class.as_deref().unwrap_or(""), b.name))
                .unwrap_or_default();
            if !scopes.contains_key(&key) {
                let scope = self.scope(block.as_ref(), &blocks);
                scopes.insert(key.clone(), scope);
            }
            let scope = &scopes[&key];
            let tokens: Vec<&Token> = s
                .tokens
                .iter()
                .map(|t| &self.file.lexed.tokens[*t])
                .filter(|t| t.kind != TokenKind::Pragma)
                .collect();
            let body = match tokens.last() {
                Some(t) if t.kind == TokenKind::Punctuation => &tokens[..tokens.len() - 1],
                _ => &tokens[..],
            };
            let before = out.len();
            let skip = self.statement_call(s, body, scope, &mut out);
            if !abap_sql::is_db_statement(s.kind) {
                self.expression_calls(body, skip, scope, &mut out);
            }
            let text = self.file.statement_text(s);
            let span = span_of(self.file, s);
            for c in &mut out[before..] {
                c.from = block.clone();
                c.text = text.clone();
                c.span = span;
            }
        }
        self.amdp_calls(&blocks, &mut out);
        for c in &mut out {
            c.resolved = self.defined_here(c);
        }
        out
    }

    /// Calls of other AMDP methods in AMDP bodies: `CALL "ZCL_X=>METH"( ... )`
    /// and table functions like `FROM "ZCL_X=>FUNC"( ... )`.
    fn amdp_calls(&self, blocks: &[Option<m::BlockRef>], out: &mut Vec<m::Call>) {
        let file = self.file;
        let mut i = 0;
        while i < file.statements.len() {
            let Some(end) = crate::db::native_end(file, i) else {
                i += 1;
                continue;
            };
            if file.statements[i].kind == K::MethodImplementation {
                let (from, to) = crate::db::native_range(file, i, end);
                let text = &file.source()[from..to];
                for n in abap_sql::hana::parse(text, abap_sql::hana::Dialect::SqlScript).calls {
                    let Some((class, method)) = n.name.split_once("=>") else {
                        continue;
                    };
                    let mut c =
                        new_call(m::CallKind::Method, method.to_string(), false, Vec::new());
                    c.class = Some(class.to_string());
                    c.receiver = Some(class.to_string());
                    c.is_static = true;
                    c.from = blocks[i].clone();
                    c.text = text[n.range.clone()].to_string();
                    c.span = m::Span {
                        start: crate::db::position_at(file.source(), from + n.range.start),
                        end: crate::db::position_at(file.source(), from + n.range.end),
                    };
                    out.push(c);
                }
            }
            i = end;
        }
    }

    /// Whether the target is defined in the symbols of this file.
    fn defined_here(&self, c: &m::Call) -> bool {
        let s = self.symbols;
        match c.kind {
            m::CallKind::Function => s
                .function_modules
                .iter()
                .any(|f| f.name.eq_ignore_ascii_case(&c.name)),
            m::CallKind::Form => {
                c.program.is_none() && s.forms.iter().any(|f| f.name.eq_ignore_ascii_case(&c.name))
            }
            m::CallKind::Method | m::CallKind::Constructor => {
                // the class, then its superclasses in this source
                let constructor = c.kind == m::CallKind::Constructor;
                let mut class = c.class.as_deref();
                for _ in 0..10 {
                    let Some(cls) = class else { return false };
                    if method_defined(s, cls, &c.name, constructor) {
                        return true;
                    }
                    class = type_info(s, cls).and_then(|i| i.superclass);
                }
                false
            }
            _ => false,
        }
    }

    /// Declared variable types visible in a processing block.
    fn scope(&self, block: Option<&m::BlockRef>, blocks: &[Option<m::BlockRef>]) -> Scope {
        let mut vars: HashMap<String, String> = HashMap::new();
        add_declarations(&mut vars, &self.symbols.declarations);
        let mut class = None;
        if let Some(b) = block {
            match b.kind.as_str() {
                "method" => {
                    let cls = b.class.clone().unwrap_or_default();
                    // attributes of the class and its superclasses come from the resolver;
                    // own attributes and method signature from the symbols
                    if let Some(info) = type_info(self.symbols, &cls) {
                        add_declarations(&mut vars, info.attributes);
                    }
                    // the signature may be in an interface or a superclass of another object
                    for (name, class) in self
                        .resolver
                        .method_parameters(&cls, &b.name)
                        .unwrap_or_default()
                    {
                        vars.insert(name.to_ascii_lowercase(), class);
                    }
                    if let Some(imp) = self.implementation(&cls) {
                        if let Some(mi) = imp
                            .methods
                            .iter()
                            .find(|x| x.name.eq_ignore_ascii_case(&b.name))
                        {
                            add_declarations(&mut vars, &mi.locals);
                        }
                    }
                    class = Some(cls);
                }
                "form" => {
                    if let Some(f) = self
                        .symbols
                        .forms
                        .iter()
                        .find(|f| f.name.eq_ignore_ascii_case(&b.name))
                    {
                        add_declarations(&mut vars, &f.locals);
                        params(&mut vars, &f.parameters);
                    }
                }
                "function_module" => {
                    if let Some(f) = self
                        .symbols
                        .function_modules
                        .iter()
                        .find(|f| f.name.eq_ignore_ascii_case(&b.name))
                    {
                        add_declarations(&mut vars, &f.locals);
                        params(&mut vars, &f.parameters);
                    }
                }
                _ => {}
            }
        }
        let mut scope = Scope { vars, class };
        // inline declarations: DATA(x) = NEW zcl( ) / CAST zcl( ) / call returning a reference
        for (i, s) in self.file.statements.iter().enumerate() {
            if blocks[i].as_ref() != block {
                continue;
            }
            if s.kind == K::Catch {
                // CATCH [BEFORE UNWIND] zcx_a [zcx_b] INTO DATA(lx): typed by the first class
                let t: Vec<&Token> = s
                    .tokens
                    .iter()
                    .map(|k| &self.file.lexed.tokens[*k])
                    .collect();
                let first = t
                    .iter()
                    .skip(1)
                    .find(|x| !x.is("BEFORE") && !x.is("UNWIND"));
                let into = t.iter().position(|x| x.is("INTO"));
                if let (Some(class), Some(k)) = (first, into) {
                    let declares = t.get(k + 1).is_some_and(|x| x.is("DATA") || x.is("FINAL"));
                    if declares && t.get(k + 2).is_some_and(|x| x.kind == TokenKind::ParenLeft) {
                        if let Some(var) = t.get(k + 3) {
                            scope
                                .vars
                                .insert(var.text.to_ascii_lowercase(), class.text.clone());
                        }
                    }
                }
                continue;
            }
            if s.kind != K::Move {
                continue;
            }
            let t: Vec<&Token> = s
                .tokens
                .iter()
                .map(|k| &self.file.lexed.tokens[*k])
                .collect();
            if t.len() < 6
                || !(t[0].is("DATA") || t[0].is("FINAL"))
                || t[1].kind != TokenKind::ParenLeft
            {
                continue;
            }
            let var = t[2].text.to_ascii_lowercase();
            if !matches!(t[4].text.as_str(), "=" | "?=") {
                continue;
            }
            let rhs = &t[5..t.len() - 1];
            let class =
                if (rhs[0].is("NEW") || rhs[0].is("CAST")) && rhs.len() > 1 && rhs[1].text != "#" {
                    Some(rhs[1].text.clone())
                } else {
                    // a call chain: the type of its result
                    let end = rhs.len();
                    self.chain_class(rhs, 0, end, &scope)
                };
            if let Some(c) = class {
                scope.vars.insert(var, c);
            }
        }
        scope
    }

    fn implementation(&self, class: &str) -> Option<&m::ClassImplementation> {
        self.symbols
            .classes
            .iter()
            .find(|c| c.name.eq_ignore_ascii_case(class))
            .and_then(|c| c.implementation.as_ref())
            .or_else(|| {
                self.symbols
                    .orphan_implementations
                    .iter()
                    .find(|o| o.class.eq_ignore_ascii_case(class))
                    .map(|o| &o.implementation)
            })
    }

    /// Calls given by the statement keyword. Returns how many leading tokens
    /// the expression scan has to skip.
    fn statement_call(
        &self,
        s: &Statement,
        t: &[&Token],
        scope: &Scope,
        out: &mut Vec<m::Call>,
    ) -> usize {
        let word = |i: usize| t.get(i).map(|x| x.upper()).unwrap_or_default();
        let literal = |tok: &Token| -> Option<String> {
            let x = &tok.text;
            ((x.starts_with('\'') && x.ends_with('\'')) || (x.starts_with('`') && x.ends_with('`')))
                .then(|| x[1..x.len() - 1].trim().to_ascii_uppercase())
        };
        let operand_end = |i: usize| operand_end(t, i);
        let text = |i: usize, j: usize| join(t, i, j, &self.file.lexed.source);
        // a name or literal, else dynamic
        let target = |i: usize| -> (String, bool) {
            match t.get(i) {
                Some(tok) => match literal(tok) {
                    Some(l) => (l, false),
                    None if tok.kind == TokenKind::ParenLeft => (text(i, operand_end(i)), true),
                    None if tok.kind == TokenKind::Identifier => {
                        let j = operand_end(i);
                        // a variable holding the name, unless it is a plain name
                        (
                            text(i, j),
                            matches!(s.kind, K::CallFunction | K::CallTransaction | K::Leave),
                        )
                    }
                    None => (text(i, operand_end(i)), true),
                },
                None => (String::new(), true),
            }
        };
        let options = |phrases: &[&str]| -> Vec<String> {
            let mut found = Vec::new();
            for p in phrases {
                let words: Vec<&str> = p.split(' ').collect();
                for i in 0..t.len() {
                    if words.iter().enumerate().all(|(k, w)| word(i + k) == *w) {
                        let mut j = i + words.len();
                        if matches!(*p, "DESTINATION" | "STARTING NEW TASK") {
                            j = operand_end(j).max(j);
                        }
                        found.push(text(i, j));
                        break;
                    }
                }
            }
            found
        };
        let mut push = |kind, name: String, dynamic, opts: Vec<String>| {
            out.push(new_call(kind, name, dynamic, opts));
        };
        match s.kind {
            K::CallFunction => {
                let (name, dynamic) = target(2);
                let opts = options(&[
                    "DESTINATION",
                    "IN UPDATE TASK",
                    "STARTING NEW TASK",
                    "IN BACKGROUND TASK",
                    "IN BACKGROUND UNIT",
                ]);
                push(m::CallKind::Function, name, dynamic, opts);
                3
            }
            K::Perform => {
                if t.len() < 2 {
                    return t.len();
                }
                let mut i = 1;
                let (name, mut dynamic) = if t[1].kind == TokenKind::ParenLeft {
                    let j = operand_end(1);
                    i = j;
                    (text(1, j), true)
                } else {
                    i += 1;
                    (t[1].text.clone(), false)
                };
                let mut program = None;
                if i < t.len() && t[i].kind == TokenKind::ParenLeft && !t[i].ws_before {
                    // PERFORM form(program)
                    let close = (i..t.len())
                        .find(|k| t[*k].kind == TokenKind::ParenRight)
                        .unwrap_or(t.len());
                    program = Some(text(i + 1, close).to_ascii_uppercase());
                }
                if let Some(k) = (0..t.len()).find(|k| word(*k) == "IN" && word(k + 1) == "PROGRAM")
                {
                    if let Some(p) = t.get(k + 2) {
                        if p.kind == TokenKind::ParenLeft {
                            dynamic = true;
                            program = Some(text(k + 2, operand_end(k + 2)));
                        } else {
                            program = Some(p.text.to_ascii_uppercase());
                        }
                    }
                }
                let mut call = new_call(m::CallKind::Form, name, dynamic, options(&["IF FOUND"]));
                call.program = program;
                out.push(call);
                t.len().min(i + 1)
            }
            K::Submit => {
                let (name, dynamic) = target(1);
                let name = if dynamic {
                    name
                } else {
                    name.to_ascii_uppercase()
                };
                push(
                    m::CallKind::Submit,
                    name,
                    dynamic,
                    options(&["AND RETURN", "VIA SELECTION-SCREEN", "VIA JOB"]),
                );
                2
            }
            K::CallTransaction => {
                let (name, dynamic) = target(2);
                push(
                    m::CallKind::Transaction,
                    name,
                    dynamic,
                    options(&["AND SKIP FIRST SCREEN"]),
                );
                3
            }
            K::Leave if word(1) == "TO" && word(2) == "TRANSACTION" => {
                let (name, dynamic) = target(3);
                push(
                    m::CallKind::Transaction,
                    name,
                    dynamic,
                    vec!["LEAVE".to_string()],
                );
                4
            }
            K::Include => {
                if let Some(tok) = t.get(1) {
                    push(
                        m::CallKind::Include,
                        tok.text.to_ascii_uppercase(),
                        false,
                        options(&["IF FOUND"]),
                    );
                }
                t.len()
            }
            K::CallTransformation => {
                let (name, dynamic) = target(2);
                let name = if dynamic {
                    name
                } else {
                    name.to_ascii_uppercase()
                };
                push(m::CallKind::Transformation, name, dynamic, Vec::new());
                3
            }
            K::Call | K::CallBadi if word(0) == "CALL" => {
                // CALL METHOD chain / CALL BADI badi->meth
                let start = 2;
                let end = operand_end(start);
                if start < end {
                    let mut c = self.chain_call(t, start, end, scope, true);
                    if s.kind == K::CallBadi {
                        c.options.push("BADI".to_string());
                    }
                    out.push(c);
                }
                end
            }
            K::CreateObject => {
                let var_end = operand_end(2);
                let var = text(2, var_end);
                let ty = (var_end..t.len()).find(|k| word(*k) == "TYPE");
                let (class, dynamic) = match ty.and_then(|k| t.get(k + 1).map(|x| (k + 1, x))) {
                    Some((k, x)) if x.kind == TokenKind::ParenLeft => {
                        match literal_in_parens(t, k, operand_end(k)) {
                            Some(lit) => (Some(lit), false),
                            None => (Some(text(k, operand_end(k))), true),
                        }
                    }
                    Some((_, x)) => (Some(x.text.clone()), false),
                    None => (self.receiver_class(t, 2, var_end, scope), false),
                };
                let mut c = new_call(
                    m::CallKind::Constructor,
                    "constructor".to_string(),
                    dynamic,
                    Vec::new(),
                );
                c.receiver = Some(var);
                c.class = class;
                out.push(c);
                var_end
            }
            _ => 0,
        }
    }

    /// Functional calls: a name directly followed by `( `.
    fn expression_calls(&self, t: &[&Token], skip: usize, scope: &Scope, out: &mut Vec<m::Call>) {
        for k in skip.max(1)..t.len() {
            let p = t[k];
            if p.kind != TokenKind::ParenLeft
                || p.ws_before
                || !(p.ws_after
                    || t.get(k + 1)
                        .is_some_and(|n| n.kind == TokenKind::ParenRight))
            {
                continue;
            }
            let start = chain_start(t, k);
            if start < skip {
                continue;
            }
            let prev = start
                .checked_sub(1)
                .map(|x| t[x].upper())
                .unwrap_or_default();
            if CONSTRUCTOR_OPERATORS.contains(&prev.as_str()) {
                continue;
            }
            // NEW zcl( ), and THROW zcx( ) inside COND and SWITCH
            if prev == "NEW" || prev == "THROW" {
                let name = join(t, start, k, &self.file.lexed.source);
                let mut c = new_call(
                    m::CallKind::Constructor,
                    "constructor".to_string(),
                    name.contains('('),
                    Vec::new(),
                );
                if name != "#" {
                    c.class = Some(name);
                }
                out.push(c);
                continue;
            }
            let units = units(t, start, k);
            let Some(last) = units.last() else { continue };
            if units.len() == 1 {
                // a bare name: built-in function, or a method of the own
                // class; a method hides the built-in function of its name
                let name = last.name.to_ascii_lowercase();
                let first = t[start];
                let Some(class) = scope.class.as_deref() else {
                    continue;
                };
                if first.kind != TokenKind::Identifier
                    || name.contains('-')
                    || (BUILTINS.contains(&name.as_str())
                        && self.resolver.method_parameters(class, &name).is_none())
                {
                    continue;
                }
            }
            out.push(self.chain_call(t, start, k, scope, false));
        }
    }

    /// The call of the last element of a chain like `a->b( )->c`.
    fn chain_call(
        &self,
        t: &[&Token],
        start: usize,
        end: usize,
        scope: &Scope,
        statement: bool,
    ) -> m::Call {
        let units = units(t, start, end);
        let source = &self.file.lexed.source;
        let dynamic = t[start..end]
            .iter()
            .any(|x| x.kind == TokenKind::ParenLeft && !x.ws_after)
            && statement;
        let mut c = new_call(m::CallKind::Method, String::new(), dynamic, Vec::new());
        if dynamic {
            // lo->(name), zcl=>(name), (class)=>meth: the part before the arrow is known
            c.name = join(t, start, end, source);
            if let Some(arrow) = (start..end).find(|k| {
                matches!(
                    t[*k].kind,
                    TokenKind::InstanceArrow | TokenKind::StaticArrow
                )
            }) {
                c.receiver = Some(join(t, start, arrow, source));
                c.is_static = t[arrow].kind == TokenKind::StaticArrow;
                let receiver_literal = literal_in_parens(t, start, arrow);
                if let Some(class) = &receiver_literal {
                    c.class = Some(class.clone());
                } else if !(start..arrow)
                    .any(|k| t[k].kind == TokenKind::ParenLeft && !t[k].ws_after)
                {
                    let receiver = self::units(t, start, arrow);
                    c.class = self.units_class(&receiver, scope, Some(t[arrow].kind));
                }
                // ('CL_X')=>('METHOD'): names given as literals are static after all
                let method_literal = literal_in_parens(t, arrow + 1, end);
                let receiver_static = receiver_literal.is_some()
                    || !(start..arrow).any(|k| t[k].kind == TokenKind::ParenLeft && !t[k].ws_after);
                if let (Some(method), true) = (method_literal, receiver_static) {
                    c.name = method.to_ascii_lowercase();
                    c.dynamic = false;
                } else if receiver_literal.is_some() && end == arrow + 2 {
                    c.name = t[arrow + 1].text.clone();
                    c.dynamic = false;
                }
            }
            return c;
        }
        let Some(last) = units.last() else {
            c.name = join(t, start, end, source);
            return c;
        };
        c.name = last.name.clone();
        if units.len() == 1 {
            // own method, also `intf~meth`
            c.class = scope.class.clone();
            return c;
        }
        let receiver_end = units[units.len() - 2].text_end;
        c.receiver = Some(join(t, start, receiver_end, source));
        c.is_static = last.arrow == Some(TokenKind::StaticArrow);
        c.class = self.units_class(&units[..units.len() - 1], scope, last.arrow);
        c
    }

    /// Class of the value of a chain `t[start..end]`, like a receiver.
    fn chain_class(&self, t: &[&Token], start: usize, end: usize, scope: &Scope) -> Option<String> {
        let end = operand_end(t, start).min(end);
        let units = units(t, start, end);
        self.units_class(&units, scope, None)
    }

    fn receiver_class(
        &self,
        t: &[&Token],
        start: usize,
        end: usize,
        scope: &Scope,
    ) -> Option<String> {
        self.chain_class(t, start, end, scope)
    }

    /// Class of the value of chain elements; `next` is the arrow after them.
    fn units_class(
        &self,
        units: &[Unit],
        scope: &Scope,
        next: Option<TokenKind>,
    ) -> Option<String> {
        let first = units.first()?;
        let name = first.name.to_ascii_lowercase();
        let first_is_class = match units.get(1) {
            Some(u) => u.arrow == Some(TokenKind::StaticArrow),
            None => next == Some(TokenKind::StaticArrow),
        };
        let mut current: Option<String> = if first_is_class && !first.is_call {
            Some(first.name.clone())
        } else if name == "me" {
            scope.class.clone()
        } else if name == "super" {
            scope
                .class
                .as_deref()
                .and_then(|c| self.resolver.superclass(c))
        } else if first.is_call {
            scope
                .class
                .as_deref()
                .and_then(|c| self.resolver.returning_class(c, &first.name))
        } else {
            scope.vars.get(&name).cloned().or_else(|| {
                scope
                    .class
                    .as_deref()
                    .and_then(|c| self.resolver.attribute_class(c, &first.name))
            })
        };
        for u in &units[1..] {
            let cls = current?;
            current = if u.is_call {
                self.resolver.returning_class(&cls, &u.name)
            } else {
                self.resolver.attribute_class(&cls, &u.name)
            };
        }
        current
    }
}

fn add_declarations(vars: &mut HashMap<String, String>, decls: &[m::Declaration]) {
    for d in decls {
        if let Some(c) = ref_class(&d.typing) {
            vars.insert(d.name.to_ascii_lowercase(), c);
        }
    }
}

fn params(vars: &mut HashMap<String, String>, parameters: &[m::Parameter]) {
    for p in parameters {
        if let Some(c) = ref_class(&p.typing) {
            vars.insert(p.name.to_ascii_lowercase(), c);
        }
    }
}

/// Whether a method (or the constructor) is defined for a class in `symbols`,
/// directly, through an interface or an alias.
pub(crate) fn method_defined(
    s: &m::FileSymbols,
    class: &str,
    method: &str,
    constructor: bool,
) -> bool {
    let Some(info) = type_info(s, class) else {
        return false;
    };
    if constructor {
        // every class can be instantiated, a constructor method is optional
        return true;
    }
    let (intf, meth) = match method.split_once('~') {
        Some((i, mth)) => (Some(i), mth),
        None => (None, method),
    };
    if intf.is_none()
        && (info
            .methods
            .iter()
            .any(|x| x.name.eq_ignore_ascii_case(meth))
            || info
                .aliases
                .iter()
                .any(|a| a.name.eq_ignore_ascii_case(meth)))
    {
        return true;
    }
    info.interfaces
        .iter()
        .filter(|i| intf.is_none_or(|x| x.eq_ignore_ascii_case(i)))
        .any(|i| method_defined(s, i, meth, false))
}

pub(crate) fn new_call(
    kind: m::CallKind,
    name: String,
    dynamic: bool,
    options: Vec<String>,
) -> m::Call {
    let zero = m::Position { row: 0, col: 0 };
    m::Call {
        kind,
        from: None,
        name,
        class: None,
        program: None,
        receiver: None,
        is_static: false,
        dynamic,
        options,
        resolved: false,
        hierarchy_incomplete: false,
        unit: None,
        text: String::new(),
        span: m::Span {
            start: zero,
            end: zero,
        },
    }
}

pub(crate) fn span_of(file: &File, s: &Statement) -> m::Span {
    let tokens = file.tokens();
    let first = &tokens[s.tokens[0]];
    let last = tokens[*s.tokens.last().unwrap()].end();
    m::Span {
        start: m::Position {
            row: first.start.row,
            col: first.start.col,
        },
        end: m::Position {
            row: last.row,
            col: last.col,
        },
    }
}

/// `('NAME')`: a dynamic token holding a literal; returns the name in upper case.
fn literal_in_parens(t: &[&Token], start: usize, end: usize) -> Option<String> {
    if end != start + 3
        || t[start].kind != TokenKind::ParenLeft
        || t[end - 1].kind != TokenKind::ParenRight
    {
        return None;
    }
    let x = &t[start + 1].text;
    let quoted =
        (x.starts_with('\'') && x.ends_with('\'')) || (x.starts_with('`') && x.ends_with('`'));
    (t[start + 1].kind == TokenKind::String && quoted && x.len() >= 2)
        .then(|| x[1..x.len() - 1].trim().to_ascii_uppercase())
}

fn is_open(t: &Token) -> bool {
    matches!(t.kind, TokenKind::ParenLeft | TokenKind::BracketLeft)
}

fn is_close(t: &Token) -> bool {
    matches!(t.kind, TokenKind::ParenRight | TokenKind::BracketRight)
}

/// Index after one operand starting at `i`: connected tokens, parentheses included.
fn operand_end(t: &[&Token], i: usize) -> usize {
    let mut j = i;
    let mut depth = 0u32;
    while j < t.len() {
        if j > i && depth == 0 && t[j].ws_before {
            break;
        }
        if is_open(t[j]) {
            depth += 1;
        } else if is_close(t[j]) {
            if depth == 0 {
                break;
            }
            depth -= 1;
        }
        j += 1;
    }
    j.max(i + 1).min(t.len())
}

/// Start of the chain that ends right before the parenthesis at `k`,
/// stepping over earlier calls like `a->get( )->`.
fn chain_start(t: &[&Token], k: usize) -> usize {
    let mut start = k;
    let mut pos = k;
    while pos > 0 {
        let candidate = pos - 1;
        if !t[candidate + 1].ws_before || candidate + 1 == k {
            // connected to what follows
        } else {
            break;
        }
        let tok = t[candidate];
        if is_close(tok) {
            // jump to the matching opening parenthesis
            let mut depth = 0i32;
            let mut m = candidate;
            loop {
                if is_close(t[m]) {
                    depth += 1;
                } else if is_open(t[m]) {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                if m == 0 {
                    return start;
                }
                m -= 1;
            }
            start = m;
            if t[m].ws_before {
                break;
            }
            pos = m;
            continue;
        }
        if tok.kind == TokenKind::Punctuation || tok.kind == TokenKind::String {
            break;
        }
        start = candidate;
        if tok.ws_before {
            break;
        }
        pos = candidate;
    }
    start
}

/// Splits a chain `t[start..end]` at `->` and `=>`.
fn units(t: &[&Token], start: usize, end: usize) -> Vec<Unit> {
    let mut out: Vec<Unit> = Vec::new();
    let mut arrow = None;
    let mut i = start;
    while i < end {
        let tok = t[i];
        match tok.kind {
            TokenKind::InstanceArrow | TokenKind::StaticArrow => {
                arrow = Some(tok.kind);
                i += 1;
            }
            TokenKind::ParenLeft | TokenKind::BracketLeft => {
                let mut depth = 0i32;
                let mut j = i;
                while j < end {
                    if is_open(t[j]) {
                        depth += 1;
                    } else if is_close(t[j]) {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                    }
                    j += 1;
                }
                if tok.kind == TokenKind::ParenLeft {
                    if let Some(u) = out.last_mut() {
                        u.is_call = true;
                        u.text_end = j + 1;
                    }
                }
                i = j + 1;
            }
            TokenKind::Dash => {
                // structure component: ls_x-ref
                if let (Some(u), Some(n)) = (out.last_mut(), t.get(i + 1)) {
                    if i + 1 < end {
                        u.name.push('-');
                        u.name.push_str(&n.text);
                        u.text_end = i + 2;
                    }
                }
                i += 2;
            }
            _ => {
                out.push(Unit {
                    arrow: arrow.take(),
                    name: tok.text.clone(),
                    is_call: false,
                    text_end: i + 1,
                });
                i += 1;
            }
        }
    }
    out
}

fn join(t: &[&Token], i: usize, j: usize, _source: &str) -> String {
    abap_syntax::join_tokens(t[i..j.min(t.len())].iter().copied())
}

/// Calls of one source, types resolved within the source only.
pub fn calls(source: &str) -> Vec<m::Call> {
    let file = abap_syntax::parse(source);
    let symbols = crate::symbols_of(&file);
    let resolver = SymbolsResolver {
        symbols: &symbols,
        fallback: None,
    };
    CallExtractor {
        file: &file,
        symbols: &symbols,
        resolver: &resolver,
    }
    .calls()
}

#[cfg(test)]
mod tests {
    use super::*;

    const SRC: &str = r#"
CLASS lcl_repo DEFINITION.
  PUBLIC SECTION.
    METHODS get_item RETURNING VALUE(ro_item) TYPE REF TO lcl_item.
ENDCLASS.
CLASS lcl_item DEFINITION.
  PUBLIC SECTION.
    METHODS save.
    DATA mo_parent TYPE REF TO lcl_repo.
ENDCLASS.
CLASS lcl_main DEFINITION.
  PUBLIC SECTION.
    METHODS run IMPORTING io_repo TYPE REF TO lcl_repo.
  PRIVATE SECTION.
    DATA mo_log TYPE REF TO zcl_log.
    METHODS helper.
ENDCLASS.
CLASS lcl_main IMPLEMENTATION.
  METHOD run.
    DATA lo_item TYPE REF TO lcl_item.
    lo_item = io_repo->get_item( ).
    lo_item->save( ).
    io_repo->get_item( )->save( ).
    DATA(lo_new) = NEW lcl_item( ).
    lo_new->mo_parent->get_item( ).
    mo_log->add( iv_text = |done { lines( lt_x ) }| ).
    helper( ).
    zcl_util=>convert( iv = CONV string( 1 ) ).
    CALL METHOD me->helper.
    CALL METHOD lo_item->(lv_name).
    CALL METHOD ('ZCL_FACTORY')=>('CREATE').
    CREATE OBJECT lo_item.
    CALL FUNCTION 'Z_POST' DESTINATION 'NONE' EXPORTING iv = 1.
    CALL FUNCTION lv_fm.
    PERFORM write_log IN PROGRAM zlog_prog IF FOUND.
    SUBMIT zreport AND RETURN.
    CALL TRANSACTION 'VA01'.
  ENDMETHOD.
  METHOD helper.
  ENDMETHOD.
ENDCLASS.
"#;

    #[test]
    fn calls_with_types() {
        let c = calls(SRC);
        let got: Vec<String> = c
            .iter()
            .map(|c| {
                format!(
                    "{:?} {}{}{}{}",
                    c.kind,
                    c.class
                        .as_deref()
                        .map(|x| format!("{x}."))
                        .unwrap_or_default(),
                    c.name,
                    if c.dynamic { " dynamic" } else { "" },
                    if c.resolved { " resolved" } else { "" },
                )
            })
            .collect();
        assert_eq!(
            got,
            [
                "Method lcl_repo.get_item resolved",
                "Method lcl_item.save resolved",
                "Method lcl_repo.get_item resolved",
                "Method lcl_item.save resolved",
                "Constructor lcl_item.constructor resolved",
                "Method lcl_repo.get_item resolved",
                "Method zcl_log.add",
                "Method lcl_main.helper resolved",
                "Method zcl_util.convert",
                "Method lcl_main.helper resolved",
                "Method lcl_item.lo_item->(lv_name) dynamic",
                "Method ZCL_FACTORY.create",
                "Constructor lcl_item.constructor resolved",
                "Function Z_POST",
                "Function lv_fm dynamic",
                "Form write_log",
                "Submit ZREPORT",
                "Transaction VA01",
            ]
        );
        assert_eq!(c[13].options, ["DESTINATION 'NONE'"]);
        assert_eq!(c[15].program.as_deref(), Some("ZLOG_PROG"));
        assert_eq!(c[15].options, ["IF FOUND"]);
        assert_eq!(c[16].options, ["AND RETURN"]);
        assert!(c[8].is_static);
        let from = c[0].from.as_ref().unwrap();
        assert_eq!(
            (
                from.kind.as_str(),
                from.name.as_str(),
                from.class.as_deref()
            ),
            ("method", "run", Some("lcl_main"))
        );
    }

    #[test]
    fn forms_and_includes() {
        let c = calls("REPORT z.\nINCLUDE z_top.\nSTART-OF-SELECTION.\n  PERFORM main.\n  PERFORM other(zprog).\nFORM main.\nENDFORM.");
        let got: Vec<_> = c
            .iter()
            .map(|c| (c.kind, c.name.as_str(), c.program.as_deref(), c.resolved))
            .collect();
        assert_eq!(
            got,
            [
                (m::CallKind::Include, "Z_TOP", None, false),
                (m::CallKind::Form, "main", None, true),
                (m::CallKind::Form, "other", Some("ZPROG"), false),
            ]
        );
    }

    #[test]
    fn methods_hide_builtin_functions() {
        let c = calls(
            "CLASS lcl_base DEFINITION.
  PUBLIC SECTION.
    METHODS log IMPORTING iv TYPE string.
ENDCLASS.
CLASS lcl DEFINITION INHERITING FROM lcl_base.
  PUBLIC SECTION.
    METHODS run.
    METHODS lines RETURNING VALUE(rv) TYPE i.
ENDCLASS.
CLASS lcl IMPLEMENTATION.
  METHOD run.
    DATA(n) = lines( ).
    log( |{ n }| ).
    DATA(m) = strlen( `abc` ).
  ENDMETHOD.
  METHOD lines.
  ENDMETHOD.
ENDCLASS.
CLASS lcl_other DEFINITION.
  PUBLIC SECTION.
    METHODS run.
ENDCLASS.
CLASS lcl_other IMPLEMENTATION.
  METHOD run.
    DATA lt TYPE string_table.
    DATA(n) = lines( lt ).
    DATA(x) = log( 2 ).
  ENDMETHOD.
ENDCLASS.",
        );
        let got: Vec<_> = c
            .iter()
            .map(|c| {
                (
                    c.from.as_ref().unwrap().class.as_deref().unwrap(),
                    c.class.as_deref(),
                    c.name.as_str(),
                    c.resolved,
                )
            })
            .collect();
        assert_eq!(
            got,
            [
                ("lcl", Some("lcl"), "lines", true),
                ("lcl", Some("lcl"), "log", true),
            ]
        );
    }
}
