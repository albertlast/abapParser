//! Expressions and logical expressions: operands with their data
//! references, arithmetic, method calls, constructor expressions and
//! conditions.
//!
//! The parser works on the tokens of one statement. Positions in the AST are
//! indices into [`Statement::tokens`](crate::Statement::tokens), so text and
//! source positions are found through the statement. It is lenient: what it
//! does not understand is skipped, and it never fails a whole statement.

use crate::{File, Statement, Token, TokenKind};

/// Position in [`Statement::tokens`](crate::Statement::tokens).
pub type Pos = usize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Expr {
    pub kind: ExprKind,
    /// first and last token
    pub first: Pos,
    pub last: Pos,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExprKind {
    /// `'text'`, `` `text` ``, `42`, `|text|`, `TEXT-001`, `'text'(001)`
    Literal,
    /// `|text { expr } text|`, the embedded expressions
    Template(Vec<Expr>),
    /// a data object, a method call, or a chain of both
    Chain(Box<Chain>),
    /// `VALUE #( ... )`, `NEW zcl( ... )`, `COND #( ... )` ...
    Constructor(Box<Constructor>),
    Binary {
        op: String,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Unary {
        op: String,
        operand: Box<Expr>,
    },
    Paren(Box<Expr>),
    /// a component of the current table line, as in `LOOP ... WHERE comp = 1`
    Component,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chain {
    pub head: Head,
    pub links: Vec<Link>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Head {
    /// data object, field symbol or `me`
    Name(Pos),
    /// the class or interface before `=>`
    Class(Pos),
    /// `name( ... )`: function, method of the own class or built-in function
    Call {
        name: Pos,
        args: Args,
    },
    /// `(name)` or `('NAME')` before `=>` or `->`, with what is inside
    Dynamic(Option<Box<Expr>>),
    /// `DATA(x)`, `FINAL(x)`, `FIELD-SYMBOL(<x>)`: the declared name
    Inline(Pos),
    Constructor(Box<Constructor>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Link {
    /// `-comp`
    Component(Pos),
    /// `->attr`, `=>attr`
    Attribute { name: Pos, is_static: bool },
    /// `->meth( )`, `=>meth( )`
    Method {
        name: Pos,
        is_static: bool,
        args: Args,
    },
    /// `->*`
    Deref,
    /// `[ ... ]`
    Table(Vec<Item>),
    /// `[]`, the table body
    Body,
    /// `+off(len)`
    Substring(Vec<Expr>),
    /// `->(name)`, `=>(name)`, `-(name)`, with what is inside
    Dynamic(Option<Box<Expr>>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Section {
    Exporting,
    Importing,
    Changing,
    Receiving,
    Exceptions,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Args {
    pub items: Vec<Arg>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Arg {
    pub section: Section,
    /// the parameter name, `None` for the single unnamed argument
    pub name: Option<Pos>,
    pub value: Value,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    Expr(Expr),
    Cond(Cond),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Constructor {
    /// upper case: VALUE, NEW, CONV, CORRESPONDING, CAST, REF, EXACT,
    /// REDUCE, FILTER, COND, SWITCH
    pub keyword: String,
    /// the type, `#` when inferred
    pub type_name: Pos,
    /// the last token of the type, after `type_name` for `zif_x=>ty_name`
    pub type_end: Pos,
    pub items: Vec<Item>,
}

/// Parts of a constructor expression or table expression, in source order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Item {
    /// a value that is read
    Value(Expr),
    /// a local name defined by FOR, LET, INIT or INDEX INTO, or assigned by
    /// NEXT, with its value
    Define {
        name: Expr,
        value: Option<Expr>,
    },
    /// `comp = value`: a component, parameter or key name with its value
    Named {
        name: Pos,
        value: Value,
    },
    Cond(Cond),
    /// `( ... )`, a table line
    Row(Vec<Item>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cond {
    pub kind: CondKind,
    pub first: Pos,
    pub last: Pos,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CondKind {
    And(Vec<Cond>),
    Or(Vec<Cond>),
    Equiv(Vec<Cond>),
    Not(Box<Cond>),
    Paren(Box<Cond>),
    /// `a = b`, `a CS b`, ...; `op` upper case as written
    Compare {
        left: Expr,
        op: String,
        right: Expr,
    },
    /// `a IS [NOT] INITIAL|BOUND|ASSIGNED|SUPPLIED|REQUESTED|INSTANCE OF c`
    Is {
        operand: Expr,
        negated: bool,
        /// upper case, `INSTANCE OF` for the type check
        predicate: String,
        class: Option<Pos>,
    },
    /// `a [NOT] IN range`, `a [NOT] IN ( b, c )`
    In {
        operand: Expr,
        negated: bool,
        values: Vec<Expr>,
    },
    /// `a [NOT] BETWEEN b AND c`
    Between {
        operand: Expr,
        negated: bool,
        low: Expr,
        high: Expr,
    },
    /// predicate method call or predicate function, `line_exists( ... )`
    Predicate(Expr),
}

const CONSTRUCTORS: &[&str] = &[
    "VALUE",
    "NEW",
    "CONV",
    "CORRESPONDING",
    "CAST",
    "REF",
    "EXACT",
    "REDUCE",
    "FILTER",
    "COND",
    "SWITCH",
];

const COMPARE_OPS: &[&str] = &[
    "=", "<>", "<", ">", "<=", ">=", "><", "=<", "EQ", "NE", "LT", "GT", "LE", "GE", "CO", "CN",
    "CA", "NA", "CS", "NS", "CP", "NP", "O", "Z", "M",
];

/// Built-in functions whose single argument is a logical expression.
const BOOLEAN_FUNCTIONS: &[&str] = &["BOOLC", "BOOLX", "XSDBOOL"];

/// Values of string template formatting options that are keywords, not data.
const FORMAT_KEYWORDS: &[&str] = &[
    "LEFT",
    "RIGHT",
    "CENTER",
    "UPPER",
    "LOWER",
    "RAW",
    "USER",
    "ISO",
    "ENVIRONMENT",
    "IN",
    "OUT",
    "YES",
    "NO",
    "SPACE",
    "SIMPLE",
    "SIGN",
    "SCIENTIFIC",
    "ENGINEERING",
    "SCALE_PRESERVING",
    "SCIENTIFIC_WITH_LEADING_ZERO",
    "SCALE_PRESERVING_SCIENTIFIC",
    "LEFTPLUS",
    "LEFTSPACE",
    "RIGHTPLUS",
    "RIGHTSPACE",
    "UTC",
];

pub struct Parser<'a> {
    toks: Vec<&'a Token>,
    pub pos: Pos,
}

impl<'a> Parser<'a> {
    /// The tokens of a statement without its terminating period or comma.
    pub fn new(file: &'a File, s: &Statement) -> Self {
        let mut toks: Vec<&Token> = s.tokens.iter().map(|i| &file.tokens()[*i]).collect();
        if toks
            .last()
            .is_some_and(|t| t.kind == TokenKind::Punctuation)
        {
            toks.pop();
        }
        Parser { toks, pos: 0 }
    }

    /// A parser on the same tokens at another position.
    pub fn clone_at(&self, pos: Pos) -> Self {
        Parser {
            toks: self.toks.clone(),
            pos,
        }
    }

    pub fn len(&self) -> usize {
        self.toks.len()
    }

    pub fn is_empty(&self) -> bool {
        self.toks.is_empty()
    }

    pub fn done(&self) -> bool {
        self.pos >= self.toks.len()
    }

    pub fn token(&self, pos: Pos) -> Option<&'a Token> {
        self.toks.get(pos).copied()
    }

    pub fn peek(&self, ahead: usize) -> Option<&'a Token> {
        self.toks.get(self.pos + ahead).copied()
    }

    /// The upper case text of the identifier `ahead` tokens on.
    pub fn word(&self, ahead: usize) -> Option<String> {
        self.peek(ahead)
            .filter(|t| t.kind == TokenKind::Identifier)
            .map(|t| t.upper())
    }

    pub fn at(&self, word: &str) -> bool {
        self.peek(0)
            .is_some_and(|t| t.kind == TokenKind::Identifier && t.is(word))
    }

    /// Whether the next tokens are the blank separated words.
    pub fn at_seq(&self, words: &str) -> bool {
        words.split(' ').enumerate().all(|(i, w)| {
            self.peek(i)
                .is_some_and(|t| t.kind == TokenKind::Identifier && t.is(w))
        })
    }

    pub fn eat(&mut self, word: &str) -> bool {
        let found = self.at(word);
        if found {
            self.pos += 1;
        }
        found
    }

    pub fn eat_seq(&mut self, words: &str) -> bool {
        let found = self.at_seq(words);
        if found {
            self.pos += words.split(' ').count();
        }
        found
    }

    pub fn bump(&mut self) {
        self.pos += 1;
    }

    pub(crate) fn kind_at(&self, ahead: usize) -> Option<TokenKind> {
        self.peek(ahead).map(|t| t.kind)
    }

    pub(crate) fn is_close(&self) -> bool {
        self.kind_at(0) == Some(TokenKind::ParenRight)
    }

    /// `(` directly after a name, opening an argument list: `meth( `.
    pub(crate) fn call_paren_at(&self, ahead: usize) -> bool {
        self.peek(ahead).is_some_and(|t| {
            t.kind == TokenKind::ParenLeft
                && !t.ws_before
                && (t.ws_after
                    || self
                        .peek(ahead + 1)
                        .is_some_and(|n| n.kind == TokenKind::ParenRight))
        })
    }

    /// `(` with no blank before and after: `lv(3)`, `meth(`... is not one.
    fn tight_paren_at(&self, ahead: usize) -> bool {
        self.peek(ahead).is_some_and(|t| {
            t.kind == TokenKind::ParenLeft
                && !t.ws_before
                && !t.ws_after
                && self
                    .peek(ahead + 1)
                    .is_some_and(|n| n.kind != TokenKind::ParenRight)
        })
    }

    /// `(name)` or `(lo->name)`: a dynamic name, no blank after `(`.
    pub(crate) fn dynamic_at(&self, ahead: usize) -> bool {
        self.dynamic_len(ahead).is_some()
    }

    /// Token count of the dynamic name at `ahead`, parentheses included.
    pub(crate) fn dynamic_len(&self, ahead: usize) -> Option<usize> {
        let open = self.peek(ahead)?;
        if open.kind != TokenKind::ParenLeft || open.ws_after {
            return None;
        }
        let mut depth = 0usize;
        let mut k = ahead;
        while let Some(t) = self.peek(k) {
            match t.kind {
                TokenKind::ParenLeft => depth += 1,
                TokenKind::ParenRight => {
                    depth -= 1;
                    if depth == 0 {
                        return (!t.ws_before && k > ahead + 1).then_some(k + 1 - ahead);
                    }
                }
                _ => {}
            }
            k += 1;
        }
        None
    }

    /// The dynamic name at the cursor, `(` up to and including `)`.
    fn dynamic(&mut self) -> Option<Box<Expr>> {
        let end = self.pos + self.dynamic_len(0)?;
        self.bump();
        let inner = self.source();
        self.pos = end;
        inner.map(Box::new)
    }

    /// Skips to after the `)` closing the current parenthesis level.
    pub(crate) fn skip_to_close(&mut self) {
        let mut depth = 0usize;
        while let Some(t) = self.peek(0) {
            self.pos += 1;
            match t.kind {
                TokenKind::ParenLeft => depth += 1,
                TokenKind::ParenRight => {
                    if depth == 0 {
                        return;
                    }
                    depth -= 1;
                }
                _ => {}
            }
        }
    }

    fn skip_to_bracket_close(&mut self) {
        let mut depth = 0usize;
        while let Some(t) = self.peek(0) {
            self.pos += 1;
            match t.kind {
                TokenKind::BracketLeft => depth += 1,
                TokenKind::BracketRight => {
                    if depth == 0 {
                        return;
                    }
                    depth -= 1;
                }
                _ => {}
            }
        }
    }

    fn expr(&self, kind: ExprKind, first: Pos) -> Expr {
        Expr {
            kind,
            first,
            last: self.pos.saturating_sub(1).max(first),
        }
    }

    // ----- sources ----------------------------------------------------------

    /// An expression that is read: operands combined by arithmetic,
    /// string and bit operators.
    pub fn source(&mut self) -> Option<Expr> {
        self.binary(0)
    }

    fn binary(&mut self, min: u8) -> Option<Expr> {
        let first = self.pos;
        let mut left = self.unary()?;
        while let Some((op, len, power)) = self.binary_op() {
            if power < min {
                break;
            }
            let save = self.pos;
            self.pos += len;
            // `**` binds to the right
            let next = if op == "**" { power } else { power + 1 };
            let Some(right) = self.binary(next) else {
                self.pos = save;
                break;
            };
            left = self.expr(
                ExprKind::Binary {
                    op,
                    left: Box::new(left),
                    right: Box::new(right),
                },
                first,
            );
        }
        Some(left)
    }

    /// The binary operator at the cursor: text, token count, binding power.
    fn binary_op(&self) -> Option<(String, usize, u8)> {
        let t = self.peek(0)?;
        match t.kind {
            TokenKind::Plus if t.ws_before && t.ws_after => return Some(("+".into(), 1, 5)),
            TokenKind::Dash if t.ws_before && t.ws_after => return Some(("-".into(), 1, 5)),
            TokenKind::Identifier => {}
            _ => return None,
        }
        let up = t.upper();
        let power = match up.as_str() {
            "&&" | "&" => 1,
            "*" | "/" | "DIV" | "MOD" => 6,
            "**" => 7,
            "BIT" => {
                let dash = self.peek(1)?;
                let op = self.word(2)?;
                if dash.kind != TokenKind::Dash || dash.ws_before {
                    return None;
                }
                let power = match op.as_str() {
                    "OR" => 2,
                    "XOR" => 3,
                    "AND" => 4,
                    _ => return None,
                };
                return Some((format!("BIT-{op}"), 3, power));
            }
            _ => return None,
        };
        Some((up, 1, power))
    }

    fn unary(&mut self) -> Option<Expr> {
        let first = self.pos;
        let t = self.peek(0)?;
        if matches!(t.kind, TokenKind::Dash | TokenKind::Plus) && self.peek(1).is_some() {
            self.bump();
            let operand = self.unary()?;
            return Some(self.expr(
                ExprKind::Unary {
                    op: t.text.clone(),
                    operand: Box::new(operand),
                },
                first,
            ));
        }
        if self.at("BIT")
            && self
                .peek(1)
                .is_some_and(|d| d.kind == TokenKind::Dash && !d.ws_before)
            && self.word(2).as_deref() == Some("NOT")
        {
            self.pos += 3;
            let operand = self.unary()?;
            return Some(self.expr(
                ExprKind::Unary {
                    op: "BIT-NOT".into(),
                    operand: Box::new(operand),
                },
                first,
            ));
        }
        self.operand()
    }

    fn operand(&mut self) -> Option<Expr> {
        let first = self.pos;
        let t = self.peek(0)?;
        match t.kind {
            TokenKind::String => {
                self.bump();
                // text symbol `'text'(001)`
                if self.tight_paren_at(0)
                    && self
                        .peek(2)
                        .is_some_and(|c| c.kind == TokenKind::ParenRight)
                {
                    self.pos += 3;
                }
                Some(self.expr(ExprKind::Literal, first))
            }
            TokenKind::StringTemplate => {
                self.bump();
                Some(self.expr(ExprKind::Literal, first))
            }
            TokenKind::StringTemplateBegin => Some(self.template()),
            TokenKind::ParenLeft if t.ws_after => {
                // `( a + b )`
                self.bump();
                let inner = self.source();
                if self.is_close() {
                    self.bump();
                } else {
                    self.skip_to_close();
                }
                let inner = inner?;
                Some(self.expr(ExprKind::Paren(Box::new(inner)), first))
            }
            TokenKind::ParenLeft if self.dynamic_at(0) => self.chain(),
            TokenKind::Identifier => {
                if t.text.bytes().all(|b| b.is_ascii_digit()) {
                    self.bump();
                    return Some(self.expr(ExprKind::Literal, first));
                }
                // text symbol `TEXT-001`
                if t.is("TEXT")
                    && self
                        .peek(1)
                        .is_some_and(|d| d.kind == TokenKind::Dash && !d.ws_before && !d.ws_after)
                    && self.peek(2).is_some_and(|n| {
                        n.text.len() == 3 && n.text.bytes().all(|b| b.is_ascii_alphanumeric())
                    })
                {
                    self.pos += 3;
                    return Some(self.expr(ExprKind::Literal, first));
                }
                self.chain()
            }
            _ => None,
        }
    }

    fn template(&mut self) -> Expr {
        let first = self.pos;
        self.bump();
        let mut parts = Vec::new();
        while let Some(t) = self.peek(0) {
            match t.kind {
                TokenKind::StringTemplateEnd => {
                    self.bump();
                    break;
                }
                TokenKind::StringTemplateMiddle => {
                    self.bump();
                    continue;
                }
                _ => {}
            }
            let before = self.pos;
            if let Some(e) = self.source() {
                parts.push(e);
            }
            // formatting options `WIDTH = 10 ALIGN = LEFT`
            while self.kind_at(0) == Some(TokenKind::Identifier)
                && self.word(1).as_deref() == Some("=")
            {
                self.pos += 2;
                if self
                    .word(0)
                    .is_some_and(|w| FORMAT_KEYWORDS.contains(&w.as_str()))
                {
                    self.bump();
                } else if let Some(e) = self.source() {
                    parts.push(e);
                }
            }
            if self.pos == before {
                self.bump();
            }
        }
        self.expr(ExprKind::Template(parts), first)
    }

    /// A target: a data object written to, or an inline declaration.
    pub fn target(&mut self) -> Option<Expr> {
        let t = self.peek(0)?;
        match t.kind {
            TokenKind::Identifier => self.chain(),
            TokenKind::ParenLeft if self.dynamic_at(0) => self.chain(),
            _ => None,
        }
    }

    /// An inline declaration at the cursor: `DATA(x)`, `FINAL(x)`,
    /// `FIELD-SYMBOL(<x>)`; returns the declared name's position and the
    /// token count.
    fn inline_at(&self) -> Option<(Pos, usize)> {
        let is_open = |k: usize| {
            self.peek(k)
                .is_some_and(|t| t.kind == TokenKind::ParenLeft && !t.ws_before)
        };
        let is_close = |k: usize| {
            self.peek(k)
                .is_some_and(|t| t.kind == TokenKind::ParenRight)
        };
        if (self.at("DATA") || self.at("FINAL")) && is_open(1) && is_close(3) {
            return Some((self.pos + 2, 4));
        }
        if self.at("FIELD")
            && self
                .peek(1)
                .is_some_and(|d| d.kind == TokenKind::Dash && !d.ws_before)
            && self.word(2).as_deref() == Some("SYMBOL")
            && is_open(3)
            && is_close(5)
        {
            return Some((self.pos + 4, 6));
        }
        None
    }

    fn constructor_at(&self) -> bool {
        self.word(0)
            .is_some_and(|w| CONSTRUCTORS.contains(&w.as_str()))
            && self
                .peek(1)
                .is_some_and(|t| t.kind == TokenKind::Identifier && t.ws_before)
            && self.call_paren_at(2 + self.qualified_len(1))
    }

    /// The tokens after the name at `ahead` that qualify it, as in a type
    /// `zif_x=>ty_name`: two per `=>name`.
    fn qualified_len(&self, ahead: usize) -> usize {
        let mut len = 0;
        while self
            .peek(ahead + len + 1)
            .is_some_and(|t| t.kind == TokenKind::StaticArrow && !t.ws_before)
            && self
                .peek(ahead + len + 2)
                .is_some_and(|t| t.kind == TokenKind::Identifier && !t.ws_before)
        {
            len += 2;
        }
        len
    }

    /// A data object or method call with components, attributes, method
    /// calls, dereferences, table expressions and substrings.
    fn chain(&mut self) -> Option<Expr> {
        let first = self.pos;
        let t = self.peek(0)?;
        let head = if let Some((name, len)) = self.inline_at() {
            self.pos += len;
            return Some(self.expr(
                ExprKind::Chain(Box::new(Chain {
                    head: Head::Inline(name),
                    links: Vec::new(),
                })),
                first,
            ));
        } else if self.constructor_at() {
            let c = self.constructor();
            if self.peek(0).is_some_and(|t| {
                matches!(
                    t.kind,
                    TokenKind::InstanceArrow | TokenKind::Dash | TokenKind::BracketLeft
                ) && !t.ws_before
            }) {
                Head::Constructor(Box::new(c))
            } else {
                return Some(self.expr(ExprKind::Constructor(Box::new(c)), first));
            }
        } else if t.kind == TokenKind::ParenLeft {
            Head::Dynamic(self.dynamic())
        } else if t.kind != TokenKind::Identifier {
            return None;
        } else if self.call_paren_at(1) {
            let name = self.pos;
            self.pos += 2;
            let args = self.args(&t.upper());
            Head::Call { name, args }
        } else if self
            .peek(1)
            .is_some_and(|a| a.kind == TokenKind::StaticArrow && !a.ws_before)
        {
            self.bump();
            Head::Class(first)
        } else {
            self.bump();
            Head::Name(first)
        };
        let links = self.links();
        Some(self.expr(ExprKind::Chain(Box::new(Chain { head, links })), first))
    }

    fn links(&mut self) -> Vec<Link> {
        let mut links = Vec::new();
        while let Some(t) = self.peek(0) {
            if t.ws_before {
                break;
            }
            match t.kind {
                TokenKind::Dash if !t.ws_after => {
                    if self.dynamic_at(1) {
                        self.bump();
                        let inner = self.dynamic();
                        links.push(Link::Dynamic(inner));
                    } else if self.kind_at(1) == Some(TokenKind::Identifier) {
                        links.push(Link::Component(self.pos + 1));
                        self.pos += 2;
                    } else {
                        break;
                    }
                }
                TokenKind::InstanceArrow | TokenKind::StaticArrow if !t.ws_after => {
                    let is_static = t.kind == TokenKind::StaticArrow;
                    if self.peek(1).is_some_and(|n| n.text == "*") {
                        links.push(Link::Deref);
                        self.pos += 2;
                    } else if self.dynamic_at(1) {
                        self.bump();
                        let inner = self.dynamic();
                        links.push(Link::Dynamic(inner));
                        if self.call_paren_at(0) {
                            self.bump();
                            self.args("");
                        }
                    } else if self.kind_at(1) == Some(TokenKind::Identifier) {
                        let name = self.pos + 1;
                        if self.call_paren_at(2) {
                            let upper = self.toks[name].upper();
                            self.pos += 3;
                            let args = self.args(&upper);
                            links.push(Link::Method {
                                name,
                                is_static,
                                args,
                            });
                        } else {
                            links.push(Link::Attribute { name, is_static });
                            self.pos += 2;
                        }
                    } else {
                        break;
                    }
                }
                TokenKind::BracketLeft => {
                    if self.kind_at(1) == Some(TokenKind::BracketRight) {
                        links.push(Link::Body);
                        self.pos += 2;
                    } else {
                        self.bump();
                        let items = self.table_expression();
                        links.push(Link::Table(items));
                    }
                }
                TokenKind::Plus if !t.ws_after => {
                    // `+off(len)`
                    self.bump();
                    let mut parts = Vec::new();
                    if let Some(off) = self.substring_part() {
                        parts.push(off);
                    }
                    if self.tight_paren_at(0) {
                        self.bump();
                        if let Some(len) = self.substring_part() {
                            parts.push(len);
                        }
                        self.skip_to_close();
                    }
                    links.push(Link::Substring(parts));
                }
                TokenKind::ParenLeft if !t.ws_after => {
                    // `(len)`
                    self.bump();
                    let mut parts = Vec::new();
                    if let Some(len) = self.substring_part() {
                        parts.push(len);
                    }
                    self.skip_to_close();
                    links.push(Link::Substring(parts));
                }
                _ => break,
            }
        }
        links
    }

    /// Offset or length: a number, `*` or a data object.
    fn substring_part(&mut self) -> Option<Expr> {
        let first = self.pos;
        let t = self.peek(0)?;
        if t.kind != TokenKind::Identifier {
            return None;
        }
        if t.text == "*" || t.text.bytes().all(|b| b.is_ascii_digit()) {
            self.bump();
            return Some(self.expr(ExprKind::Literal, first));
        }
        self.bump();
        let mut links = Vec::new();
        while self
            .peek(0)
            .is_some_and(|d| d.kind == TokenKind::Dash && !d.ws_before && !d.ws_after)
            && self.kind_at(1) == Some(TokenKind::Identifier)
        {
            links.push(Link::Component(self.pos + 1));
            self.pos += 2;
        }
        Some(self.expr(
            ExprKind::Chain(Box::new(Chain {
                head: Head::Name(first),
                links,
            })),
            first,
        ))
    }

    /// The inside of `[ ... ]` up to and including `]`.
    fn table_expression(&mut self) -> Vec<Item> {
        let mut items = Vec::new();
        while let Some(t) = self.peek(0) {
            if t.kind == TokenKind::BracketRight {
                self.bump();
                break;
            }
            let before = self.pos;
            if self.eat("KEY") {
                if let Some(n) = self.dynamic_len(0) {
                    self.pos += n;
                } else {
                    self.bump();
                }
                self.eat("COMPONENTS");
            } else if self.eat("INDEX") {
                if let Some(e) = self.source() {
                    items.push(Item::Value(e));
                }
            } else if self.named_at() {
                let name = self.pos;
                self.skip_component_name();
                self.bump(); // =
                if let Some(e) = self.source() {
                    items.push(Item::Named {
                        name,
                        value: Value::Expr(e),
                    });
                }
            } else if let Some(e) = self.source() {
                items.push(Item::Value(e));
            }
            if self.pos == before {
                self.skip_to_bracket_close();
                break;
            }
        }
        items
    }

    /// `name =` at the cursor; the name may be a component path like `a-b`
    /// or dynamic `(name)`.
    pub(crate) fn named_at(&self) -> bool {
        let mut k = 0;
        if let Some(n) = self.dynamic_len(0) {
            k = n;
        } else if self.kind_at(0) == Some(TokenKind::Identifier) {
            k = 1;
            while self
                .peek(k)
                .is_some_and(|d| d.kind == TokenKind::Dash && !d.ws_before && !d.ws_after)
                && self.kind_at(k + 1) == Some(TokenKind::Identifier)
            {
                k += 2;
            }
        }
        k > 0
            && self
                .peek(k)
                .is_some_and(|t| t.kind == TokenKind::Identifier && t.text == "=")
    }

    pub(crate) fn skip_component_name(&mut self) {
        if let Some(n) = self.dynamic_len(0) {
            self.pos += n;
            return;
        }
        self.bump();
        while self
            .peek(0)
            .is_some_and(|d| d.kind == TokenKind::Dash && !d.ws_before && !d.ws_after)
            && self.kind_at(1) == Some(TokenKind::Identifier)
        {
            self.pos += 2;
        }
    }

    /// A component name of a table line as operand: `comp`, `comp-sub`,
    /// `table_line`, `(name)`.
    pub fn component(&mut self) -> Option<Expr> {
        let first = self.pos;
        let t = self.peek(0)?;
        if !(t.kind == TokenKind::Identifier || self.dynamic_at(0)) {
            return None;
        }
        self.skip_component_name();
        // `comp->attr`, `table_line->attr`
        while self
            .peek(0)
            .is_some_and(|a| a.kind == TokenKind::InstanceArrow && !a.ws_before)
            && self.kind_at(1) == Some(TokenKind::Identifier)
        {
            self.pos += 2;
        }
        Some(self.expr(ExprKind::Component, first))
    }

    /// Arguments after `name(` up to and including `)`.
    fn args(&mut self, name: &str) -> Args {
        let mut args = Args::default();
        if self.is_close() {
            self.bump();
            return args;
        }
        let section_at = |p: &Self| -> Option<Section> {
            Some(match p.word(0)?.as_str() {
                "EXPORTING" => Section::Exporting,
                "IMPORTING" => Section::Importing,
                "CHANGING" => Section::Changing,
                "RECEIVING" => Section::Receiving,
                "EXCEPTIONS" => Section::Exceptions,
                _ => return None,
            })
        };
        let boolean = BOOLEAN_FUNCTIONS.contains(&name);
        // boolc( a = b ) and xsdbool( a = b ) take one logical expression
        let single_cond = boolean && name != "BOOLX";
        if single_cond || (section_at(self).is_none() && !self.named_at()) {
            let value = if boolean {
                self.cond().map(Value::Cond)
            } else {
                self.source().map(Value::Expr)
            };
            if let Some(value) = value {
                args.items.push(Arg {
                    section: Section::Exporting,
                    name: None,
                    value,
                });
            }
            self.skip_to_close();
            return args;
        }
        let mut section = Section::Exporting;
        loop {
            if self.done() {
                break;
            }
            if self.is_close() {
                self.bump();
                break;
            }
            if let Some(s) = section_at(self) {
                section = s;
                self.bump();
                continue;
            }
            if self.named_at() {
                let name = self.pos;
                self.skip_component_name();
                self.bump(); // =
                let value = match section {
                    Section::Importing | Section::Changing | Section::Receiving => {
                        self.target().map(Value::Expr)
                    }
                    Section::Exceptions => {
                        self.bump();
                        None
                    }
                    Section::Exporting if boolean => self.cond().map(Value::Cond),
                    Section::Exporting => self.source().map(Value::Expr),
                };
                if let Some(value) = value {
                    args.items.push(Arg {
                        section,
                        name: Some(name),
                        value,
                    });
                }
                continue;
            }
            // `EXCEPTIONS OTHERS = 1`, unknown parts
            self.bump();
        }
        args
    }

    // ----- constructor expressions -----------------------------------------

    fn constructor(&mut self) -> Constructor {
        let keyword = self.toks[self.pos].upper();
        let type_name = self.pos + 1;
        let type_end = type_name + self.qualified_len(1);
        self.pos = type_end + 2;
        let items = self.constructor_items(&keyword);
        Constructor {
            keyword,
            type_name,
            type_end,
            items,
        }
    }

    /// Items up to and including the closing `)`.
    fn constructor_items(&mut self, keyword: &str) -> Vec<Item> {
        let mut items = Vec::new();
        // names assigned after LET, INIT and NEXT are local data, not
        // components
        let mut defining = false;
        while let Some(t) = self.peek(0) {
            if t.kind == TokenKind::ParenRight {
                self.bump();
                break;
            }
            let before = self.pos;
            let word = self.word(0).unwrap_or_default();
            match word.as_str() {
                "LET" | "INIT" | "NEXT" => {
                    self.bump();
                    defining = true;
                }
                "IN" if defining => {
                    self.bump();
                    defining = false;
                }
                "FOR" => {
                    self.bump();
                    defining = false;
                    self.for_items(&mut items);
                }
                "WHEN" if keyword == "COND" => {
                    self.bump();
                    defining = false;
                    if let Some(c) = self.cond() {
                        items.push(Item::Cond(c));
                    }
                }
                "WHEN" | "THEN" | "ELSE" | "BASE" | "DEFAULT" => {
                    self.bump();
                    defining = false;
                    self.eat("THROW");
                    if let Some(e) = self.source() {
                        items.push(Item::Value(e));
                    }
                }
                "OR" if keyword == "SWITCH" => {
                    self.bump();
                    if let Some(e) = self.source() {
                        items.push(Item::Value(e));
                    }
                }
                "WHERE" => {
                    self.bump();
                    if let Some(c) = self.component_cond() {
                        items.push(Item::Cond(c));
                    }
                }
                "LINES" if self.word(1).as_deref() == Some("OF") => {
                    self.pos += 2;
                    if let Some(e) = self.source() {
                        items.push(Item::Value(e));
                    }
                }
                "FROM" | "TO" | "STEP" if keyword != "CORRESPONDING" => {
                    self.bump();
                    if let Some(e) = self.source() {
                        items.push(Item::Value(e));
                    }
                }
                "IN" if keyword == "FILTER" => {
                    self.bump();
                    if let Some(e) = self.source() {
                        items.push(Item::Value(e));
                    }
                }
                "MAPPING" | "EXCEPT" => {
                    // component names up to the next part
                    self.bump();
                    while let Some(t) = self.peek(0) {
                        if t.kind == TokenKind::ParenRight
                            || self.word(0).is_some_and(|w| {
                                matches!(w.as_str(), "EXCEPT" | "DISCARDING" | "MAPPING" | "WHERE")
                            })
                        {
                            break;
                        }
                        if t.kind == TokenKind::ParenLeft {
                            self.bump();
                            self.skip_to_close();
                        } else {
                            self.bump();
                        }
                    }
                }
                "USING" if self.word(1).as_deref() == Some("KEY") => {
                    self.pos += 3;
                }
                "OPTIONAL" | "DEEP" | "APPENDING" | "DISCARDING" | "DUPLICATES" | "EXACT"
                | "CHANGING" | "CONTROLS" | "THROW" => {
                    self.bump();
                }
                "TYPE" if defining => {
                    self.pos += 2;
                }
                _ => {
                    if t.kind == TokenKind::ParenLeft && t.ws_after {
                        // a table line `( ... )`, or a parenthesized value
                        if keyword == "VALUE" || keyword == "NEW" {
                            self.bump();
                            let row = self.constructor_items(keyword);
                            items.push(Item::Row(row));
                        } else if let Some(e) = self.source() {
                            items.push(Item::Value(e));
                        }
                    } else if defining && self.named_at() {
                        let name = self.target();
                        if self.word(0).as_deref() == Some("TYPE") {
                            self.pos += 2;
                        }
                        // `=`, `+=`, `-=`, ...
                        self.bump();
                        let value = self.source();
                        if let Some(name) = name {
                            items.push(Item::Define { name, value });
                        }
                    } else if defining
                        && self.kind_at(0) == Some(TokenKind::Identifier)
                        && self.word(1).as_deref() == Some("TYPE")
                    {
                        // `INIT x TYPE i = 0`
                        let name = self.target();
                        self.pos += 2;
                        self.bump();
                        let value = self.source();
                        if let Some(name) = name {
                            items.push(Item::Define { name, value });
                        }
                    } else if defining
                        && self.kind_at(0) == Some(TokenKind::Identifier)
                        && self.peek(1).is_some_and(|o| {
                            matches!(o.text.as_str(), "+=" | "-=" | "*=" | "/=" | "&&=")
                        })
                    {
                        let name = self.target();
                        self.bump();
                        let value = self.source();
                        if let Some(name) = name {
                            items.push(Item::Define { name, value });
                        }
                    } else if self.named_at() {
                        let name = self.pos;
                        self.skip_component_name();
                        self.bump();
                        if let Some(e) = self.source() {
                            items.push(Item::Named {
                                name,
                                value: Value::Expr(e),
                            });
                        }
                    } else if let Some(e) = self.source() {
                        items.push(Item::Value(e));
                    }
                }
            }
            if self.pos == before {
                self.bump();
            }
        }
        items
    }

    /// After FOR: the iteration variables and what they iterate.
    fn for_items(&mut self, items: &mut Vec<Item>) {
        if self.eat("GROUPS") {
            // FOR GROUPS [group] OF wa IN itab ... GROUP BY key
            if !self.at("OF") {
                if let Some(g) = self.target() {
                    items.push(Item::Define {
                        name: g,
                        value: None,
                    });
                }
            }
            self.eat("OF");
        }
        let Some(var) = self.target() else { return };
        if self.at("=") {
            // FOR i = 0 [THEN expr] UNTIL|WHILE cond
            self.bump();
            let value = self.source();
            items.push(Item::Define { name: var, value });
            if self.eat("THEN") {
                if let Some(e) = self.source() {
                    items.push(Item::Value(e));
                }
            }
            if self.eat("UNTIL") || self.eat("WHILE") {
                if let Some(c) = self.cond() {
                    items.push(Item::Cond(c));
                }
            }
            return;
        }
        items.push(Item::Define {
            name: var,
            value: None,
        });
        if !self.eat("IN") {
            return;
        }
        if self.eat("GROUP") {
            if let Some(e) = self.source() {
                items.push(Item::Value(e));
            }
        } else if let Some(e) = self.source() {
            items.push(Item::Value(e));
        }
        loop {
            if self.eat_seq("INDEX INTO") {
                if let Some(i) = self.target() {
                    items.push(Item::Define {
                        name: i,
                        value: None,
                    });
                }
            } else if self.eat_seq("USING KEY") {
                self.bump();
            } else if self.eat("WHERE") {
                if let Some(c) = self.component_cond() {
                    items.push(Item::Cond(c));
                }
            } else if self.eat_seq("GROUP BY") {
                self.group_by(items);
            } else if self.at("FROM") || self.at("TO") || self.at("STEP") {
                self.bump();
                if let Some(e) = self.source() {
                    items.push(Item::Value(e));
                }
            } else if self.eat("ASCENDING") || self.eat("DESCENDING") {
                self.eat_seq("AS TEXT");
            } else if self.eat_seq("WITHOUT MEMBERS") {
            } else {
                break;
            }
        }
    }

    /// `GROUP BY key` or `GROUP BY ( name = key ... GROUP SIZE size )`.
    pub(crate) fn group_by(&mut self, items: &mut Vec<Item>) {
        let open = self.peek(0).is_some_and(|t| t.kind == TokenKind::ParenLeft);
        if !open {
            if let Some(e) = self.source() {
                items.push(Item::Value(e));
            }
            return;
        }
        self.bump();
        loop {
            if self.done() {
                break;
            }
            if self.is_close() {
                self.bump();
                break;
            }
            let before = self.pos;
            if self.named_at() {
                let name = self.pos;
                self.bump();
                self.bump();
                if self.at_seq("GROUP SIZE") || self.at_seq("GROUP INDEX") {
                    self.pos += 2;
                } else if let Some(e) = self.source() {
                    items.push(Item::Named {
                        name,
                        value: Value::Expr(e),
                    });
                }
            }
            if self.pos == before {
                self.bump();
            }
        }
    }

    // ----- conditions -------------------------------------------------------

    /// A logical expression.
    pub fn cond(&mut self) -> Option<Cond> {
        self.cond_with(false)
    }

    /// A logical expression whose left operands are components of a table
    /// line, as in `LOOP ... WHERE` and `DELETE ... WHERE`.
    pub fn component_cond(&mut self) -> Option<Cond> {
        self.cond_with(true)
    }

    fn cond_with(&mut self, component: bool) -> Option<Cond> {
        self.junction("EQUIV", component)
    }

    fn junction(&mut self, word: &str, component: bool) -> Option<Cond> {
        let first = self.pos;
        let lower = |p: &mut Self| match word {
            "EQUIV" => p.junction("OR", component),
            "OR" => p.junction("AND", component),
            _ => p.negation(component),
        };
        let one = lower(self)?;
        let mut all = vec![one];
        while self.at(word) {
            let save = self.pos;
            self.bump();
            match lower(self) {
                Some(c) => all.push(c),
                None => {
                    self.pos = save;
                    break;
                }
            }
        }
        if all.len() == 1 {
            return all.pop();
        }
        let kind = match word {
            "EQUIV" => CondKind::Equiv(all),
            "OR" => CondKind::Or(all),
            _ => CondKind::And(all),
        };
        Some(self.cond_node(kind, first))
    }

    fn negation(&mut self, component: bool) -> Option<Cond> {
        let first = self.pos;
        if self.at("NOT") {
            self.bump();
            let inner = self.negation(component)?;
            return Some(self.cond_node(CondKind::Not(Box::new(inner)), first));
        }
        self.cond_primary(component)
    }

    fn cond_node(&self, kind: CondKind, first: Pos) -> Cond {
        Cond {
            kind,
            first,
            last: self.pos.saturating_sub(1).max(first),
        }
    }

    fn cond_primary(&mut self, component: bool) -> Option<Cond> {
        let first = self.pos;
        if self
            .peek(0)
            .is_some_and(|t| t.kind == TokenKind::ParenLeft && t.ws_after)
        {
            // `( cond )`, unless it is an arithmetic operand like `( a + b ) > c`
            self.bump();
            if let Some(inner) = self.cond_with(component) {
                if self.is_close() {
                    self.bump();
                    if !self.compare_continues() {
                        return Some(self.cond_node(CondKind::Paren(Box::new(inner)), first));
                    }
                }
            }
            self.pos = first;
        }
        self.compare(component)
    }

    /// Whether the cursor is at something that continues a comparison
    /// operand, so a parenthesis before it was arithmetic.
    fn compare_continues(&self) -> bool {
        let Some(w) = self.word(0) else {
            return false;
        };
        COMPARE_OPS.contains(&w.as_str())
            || matches!(w.as_str(), "IS" | "IN" | "BETWEEN" | "BYTE")
            || (w == "NOT" && matches!(self.word(1).as_deref(), Some("IN" | "BETWEEN")))
            || self.binary_op().is_some()
    }

    fn compare(&mut self, component: bool) -> Option<Cond> {
        let first = self.pos;
        let left = if component {
            self.component()?
        } else {
            self.source()?
        };
        let negated_word = self.at("NOT");
        let w = self.word(usize::from(negated_word)).unwrap_or_default();
        if !negated_word {
            let op = if w == "BYTE"
                && self
                    .peek(1)
                    .is_some_and(|d| d.kind == TokenKind::Dash && !d.ws_before)
            {
                let op = format!("BYTE-{}", self.word(2).unwrap_or_default());
                self.pos += 3;
                Some(op)
            } else if COMPARE_OPS.contains(&w.as_str()) {
                self.bump();
                Some(w.clone())
            } else {
                None
            };
            if let Some(op) = op {
                let right = self.source()?;
                return Some(self.cond_node(CondKind::Compare { left, op, right }, first));
            }
            if w == "IS" {
                self.bump();
                let negated = self.eat("NOT");
                let mut class = None;
                let predicate = if self.eat_seq("INSTANCE OF") {
                    class = Some(self.pos);
                    self.bump();
                    "INSTANCE OF".to_string()
                } else {
                    let p = self.word(0).unwrap_or_default();
                    self.bump();
                    p
                };
                return Some(self.cond_node(
                    CondKind::Is {
                        operand: left,
                        negated,
                        predicate,
                        class,
                    },
                    first,
                ));
            }
        }
        match w.as_str() {
            "IN" => {
                self.pos += 1 + usize::from(negated_word);
                let mut values = Vec::new();
                if self
                    .peek(0)
                    .is_some_and(|t| t.kind == TokenKind::ParenLeft && t.ws_after)
                    && self.list_ahead()
                {
                    self.bump();
                    loop {
                        if let Some(v) = self.source() {
                            values.push(v);
                        }
                        if self.peek(0).is_some_and(|t| t.text == ",") {
                            self.bump();
                            continue;
                        }
                        break;
                    }
                    self.skip_to_close();
                } else if let Some(v) = self.source() {
                    values.push(v);
                }
                Some(self.cond_node(
                    CondKind::In {
                        operand: left,
                        negated: negated_word,
                        values,
                    },
                    first,
                ))
            }
            "BETWEEN" => {
                self.pos += 1 + usize::from(negated_word);
                let low = self.source()?;
                if !self.eat("AND") {
                    return None;
                }
                let high = self.source()?;
                Some(self.cond_node(
                    CondKind::Between {
                        operand: left,
                        negated: negated_word,
                        low,
                        high,
                    },
                    first,
                ))
            }
            _ => Some(self.cond_node(CondKind::Predicate(left), first)),
        }
    }

    /// `( a, b )` ahead: a value list of IN.
    fn list_ahead(&self) -> bool {
        let mut depth = 0usize;
        let mut k = 0;
        while let Some(t) = self.peek(k) {
            match t.kind {
                TokenKind::ParenLeft => depth += 1,
                TokenKind::ParenRight => {
                    depth -= 1;
                    if depth == 0 {
                        return false;
                    }
                }
                TokenKind::Punctuation if depth == 1 && t.text == "," => return true,
                _ => {}
            }
            k += 1;
        }
        false
    }
}

// ----- data references -------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Access {
    Read,
    /// replaced, or partly when the reference ends in a component
    Write,
    /// read and changed in place, e.g. `APPEND ... TO itab`, `CONDENSE s`
    Change,
}

/// A data object used by an expression.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataRef {
    /// the name token: variable, field symbol or attribute
    pub pos: Pos,
    pub access: Access,
    /// the first and last token of the whole operand, e.g. of `ls-a-b`
    /// for `ls`, of `lo->attr` for `attr`
    pub start: Pos,
    pub last: Pos,
    /// the variable itself is replaced, not one of its components
    pub whole: bool,
    /// an attribute after `->` or `=>`
    pub attribute: bool,
    /// declared inline: `DATA(x)`, a FOR or LET variable
    pub declared: bool,
}

/// The data references of an expression read or written as `access`.
pub fn expr_refs(e: &Expr, access: Access, out: &mut Vec<DataRef>) {
    match &e.kind {
        ExprKind::Literal | ExprKind::Component => {}
        ExprKind::Template(parts) => {
            for p in parts {
                expr_refs(p, Access::Read, out);
            }
        }
        ExprKind::Chain(c) => chain_refs(c, e, access, out),
        ExprKind::Constructor(c) => constructor_refs(c, out),
        ExprKind::Binary { left, right, .. } => {
            expr_refs(left, Access::Read, out);
            expr_refs(right, Access::Read, out);
        }
        ExprKind::Unary { operand, .. } | ExprKind::Paren(operand) => {
            expr_refs(operand, Access::Read, out)
        }
    }
}

fn chain_refs(c: &Chain, e: &Expr, access: Access, out: &mut Vec<DataRef>) {
    // the data objects of the chain: the head and attributes; all but the
    // last one are only read to reach the next
    let mut data: Vec<(Pos, bool)> = Vec::new();
    match &c.head {
        Head::Name(p) => data.push((*p, false)),
        Head::Inline(p) => {
            out.push(DataRef {
                pos: *p,
                access: Access::Write,
                start: e.first,
                last: e.last,
                whole: true,
                attribute: false,
                declared: true,
            });
            return;
        }
        Head::Call { args, .. } => args_refs(args, out),
        Head::Constructor(k) => constructor_refs(k, out),
        Head::Dynamic(inner) => {
            if let Some(e) = inner {
                expr_refs(e, Access::Read, out);
            }
        }
        Head::Class(_) => {}
    }
    let mut whole = true;
    for link in &c.links {
        match link {
            Link::Attribute { name, .. } => {
                data.push((*name, true));
                whole = true;
            }
            Link::Method { args, .. } => {
                // the objects before the call are read to call it
                for (pos, attribute) in data.drain(..) {
                    out.push(DataRef {
                        pos,
                        access: Access::Read,
                        start: e.first,
                        last: e.last,
                        whole: false,
                        attribute,
                        declared: false,
                    });
                }
                args_refs(args, out);
                whole = false;
            }
            Link::Table(items) => {
                items_refs(items, out);
                whole = false;
            }
            Link::Substring(parts) => {
                for p in parts {
                    expr_refs(p, Access::Read, out);
                }
                whole = false;
            }
            Link::Dynamic(inner) => {
                if let Some(e) = inner {
                    expr_refs(e, Access::Read, out);
                }
                whole = false;
            }
            Link::Component(_) | Link::Deref => whole = false,
            Link::Body => {}
        }
    }
    let n = data.len();
    for (i, (pos, attribute)) in data.into_iter().enumerate() {
        let last = i + 1 == n;
        out.push(DataRef {
            pos,
            access: if last { access } else { Access::Read },
            start: e.first,
            last: e.last,
            whole: last && whole,
            attribute,
            declared: false,
        });
    }
}

fn args_refs(args: &Args, out: &mut Vec<DataRef>) {
    for a in &args.items {
        let access = match a.section {
            Section::Exporting | Section::Exceptions => Access::Read,
            Section::Importing | Section::Receiving => Access::Write,
            Section::Changing => Access::Change,
        };
        value_refs(&a.value, access, out);
    }
}

fn value_refs(v: &Value, access: Access, out: &mut Vec<DataRef>) {
    match v {
        Value::Expr(e) => expr_refs(e, access, out),
        Value::Cond(c) => cond_refs(c, out),
    }
}

fn constructor_refs(c: &Constructor, out: &mut Vec<DataRef>) {
    items_refs(&c.items, out);
}

fn items_refs(items: &[Item], out: &mut Vec<DataRef>) {
    for item in items {
        match item {
            Item::Value(e) => expr_refs(e, Access::Read, out),
            Item::Define { name, value } => {
                if let Some(v) = value {
                    expr_refs(v, Access::Read, out);
                }
                let at = out.len();
                expr_refs(name, Access::Write, out);
                for r in &mut out[at..] {
                    if r.access == Access::Write {
                        r.declared = true;
                    }
                }
            }
            Item::Named { value, .. } => value_refs(value, Access::Read, out),
            Item::Cond(c) => cond_refs(c, out),
            Item::Row(row) => items_refs(row, out),
        }
    }
}

/// The data references read by a condition.
pub fn cond_refs(c: &Cond, out: &mut Vec<DataRef>) {
    match &c.kind {
        CondKind::And(all) | CondKind::Or(all) | CondKind::Equiv(all) => {
            for c in all {
                cond_refs(c, out);
            }
        }
        CondKind::Not(c) | CondKind::Paren(c) => cond_refs(c, out),
        CondKind::Compare { left, right, .. } => {
            expr_refs(left, Access::Read, out);
            expr_refs(right, Access::Read, out);
        }
        CondKind::Is { operand, .. } | CondKind::Predicate(operand) => {
            expr_refs(operand, Access::Read, out)
        }
        CondKind::In {
            operand, values, ..
        } => {
            expr_refs(operand, Access::Read, out);
            for v in values {
                expr_refs(v, Access::Read, out);
            }
        }
        CondKind::Between {
            operand, low, high, ..
        } => {
            for e in [operand, low, high] {
                expr_refs(e, Access::Read, out);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_source(code: &str) -> (crate::File, Option<Expr>, usize) {
        let file = crate::parse(code);
        let s = file.statements[0].clone();
        let mut p = Parser::new(&file, &s);
        let e = p.source();
        let rest = p.len() - p.pos;
        drop(p);
        (file, e, rest)
    }

    fn refs_of(code: &str, skip: usize, target: bool) -> Vec<(String, Access)> {
        let file = crate::parse(code);
        let s = &file.statements[0];
        let mut p = Parser::new(&file, s);
        p.pos = skip;
        let e = if target { p.target() } else { p.source() }.unwrap();
        assert!(target || p.done(), "{code}: rest from {}", p.pos);
        let mut out = Vec::new();
        expr_refs(
            &e,
            if target { Access::Write } else { Access::Read },
            &mut out,
        );
        let toks: Vec<&Token> = file.statement_tokens(s).collect();
        out.iter()
            .map(|r| (toks[r.pos].text.clone(), r.access))
            .collect()
    }

    #[test]
    fn arithmetic_precedence() {
        let (_, e, rest) = parse_source("a + b * c ** d ** e - f.");
        assert_eq!(rest, 0);
        let ExprKind::Binary { op, left, .. } = e.unwrap().kind else {
            panic!()
        };
        assert_eq!(op, "-");
        let ExprKind::Binary { op, right, .. } = left.kind else {
            panic!()
        };
        assert_eq!(op, "+");
        let ExprKind::Binary { op, right, .. } = right.kind else {
            panic!()
        };
        assert_eq!(op, "*");
        // right associative power
        let ExprKind::Binary { op, right, .. } = right.kind else {
            panic!()
        };
        assert_eq!(op, "**");
        assert!(matches!(right.kind, ExprKind::Binary { .. }));
    }

    #[test]
    fn data_references() {
        use Access::*;
        let r = |code: &str| refs_of(code, 0, false);
        assert_eq!(r("ls_a-comp-sub."), [("ls_a".into(), Read)]);
        assert_eq!(
            r("lo->attr->meth( iv = x )->other."),
            [
                ("lo".into(), Read),
                ("attr".into(), Read),
                ("x".into(), Read),
                ("other".into(), Read)
            ]
        );
        assert_eq!(r("zcl_x=>attr-c."), [("attr".into(), Read)]);
        assert_eq!(
            r("itab[ key = v ]-c + lv+off(len)."),
            [
                ("v".into(), Read),
                ("itab".into(), Read),
                ("off".into(), Read),
                ("len".into(), Read),
                ("lv".into(), Read)
            ]
        );
        assert_eq!(
            r("VALUE #( BASE b FOR wa IN itab WHERE ( c = d ) ( x = wa-y ) )."),
            [
                ("b".into(), Read),
                ("wa".into(), Write),
                ("itab".into(), Read),
                ("d".into(), Read),
                ("wa".into(), Read)
            ]
        );
        assert_eq!(
            r("|a{ b WIDTH = w ALIGN = LEFT }c{ to_upper( d ) }|."),
            [("b".into(), Read), ("w".into(), Read), ("d".into(), Read)]
        );
        assert_eq!(
            r("xsdbool( a = b AND c IS INITIAL )."),
            [("a".into(), Read), ("b".into(), Read), ("c".into(), Read)]
        );
        assert_eq!(
            r("COND #( WHEN a > 1 THEN b ELSE THROW zcx( c ) )."),
            [("a".into(), Read), ("b".into(), Read), ("c".into(), Read)]
        );
        assert_eq!(
            r("REDUCE i( INIT s = 0 FOR l IN t NEXT s = s + l )."),
            [
                ("s".into(), Write),
                ("l".into(), Write),
                ("t".into(), Read),
                ("s".into(), Read),
                ("l".into(), Read),
                ("s".into(), Write)
            ]
        );
        assert_eq!(r("TEXT-001 && 'a'(002) && `b`."), []);
        assert_eq!(r("-1."), []);
        assert_eq!(
            r("CAST zif( lo )->attr."),
            [("lo".into(), Read), ("attr".into(), Read)]
        );
        // a type of an interface or class
        assert_eq!(
            r("CONV zif_x=>ty_name( lv_name )."),
            [("lv_name".into(), Read)]
        );
        assert_eq!(
            r("COND zif_x=>ty_nr( WHEN a IS INITIAL THEN b )."),
            [("a".into(), Read), ("b".into(), Read)]
        );
    }

    #[test]
    fn target_references() {
        use Access::*;
        let w = |code: &str| refs_of(code, 0, true);
        assert_eq!(w("ls-a = 1."), [("ls".into(), Write)]);
        assert_eq!(
            w("lo->attr = 1."),
            [("lo".into(), Read), ("attr".into(), Write)]
        );
        assert_eq!(w("DATA(x) = 1."), [("x".into(), Write)]);
        assert_eq!(w("FIELD-SYMBOL(<f>) = 1."), [("<f>".into(), Write)]);
    }

    #[test]
    fn method_call_arguments() {
        use Access::*;
        let r = |code: &str| refs_of(code, 0, false);
        assert_eq!(
            r("meth( EXPORTING a = x IMPORTING b = y CHANGING c = z EXCEPTIONS e = 1 )."),
            [
                ("x".into(), Read),
                ("y".into(), Write),
                ("z".into(), Change)
            ]
        );
        assert_eq!(
            r("lo->m( IMPORTING r = DATA(v) )."),
            [("lo".into(), Read), ("v".into(), Write)]
        );
    }

    fn cond_of(code: &str) -> (crate::File, Cond, bool) {
        let file = crate::parse(code);
        let s = file.statements[0].clone();
        let mut p = Parser::new(&file, &s);
        p.bump();
        let c = p.cond().unwrap();
        let done = p.done();
        drop(p);
        (file, c, done)
    }

    #[test]
    fn conditions() {
        let (_, c, done) = cond_of("IF a = 1 AND NOT b IS INITIAL OR c NOT IN r EQUIV d.");
        assert!(done);
        let CondKind::Equiv(parts) = c.kind else {
            panic!("{c:?}")
        };
        let CondKind::Or(or) = &parts[0].kind else {
            panic!()
        };
        let CondKind::And(and) = &or[0].kind else {
            panic!()
        };
        assert!(matches!(and[0].kind, CondKind::Compare { .. }));
        let CondKind::Not(not) = &and[1].kind else {
            panic!()
        };
        assert!(matches!(not.kind, CondKind::Is { negated: false, .. }));
        assert!(matches!(or[1].kind, CondKind::In { negated: true, .. }));
    }

    #[test]
    fn parenthesis_is_condition_or_arithmetic() {
        let (_, c, done) = cond_of("IF ( a + b ) > c AND ( d = e OR f BETWEEN 1 AND 2 ).");
        assert!(done);
        let CondKind::And(and) = c.kind else { panic!() };
        let CondKind::Compare { left, op, .. } = &and[0].kind else {
            panic!("{:?}", and[0])
        };
        assert_eq!(op, ">");
        assert!(matches!(left.kind, ExprKind::Paren(_)));
        let CondKind::Paren(inner) = &and[1].kind else {
            panic!()
        };
        let CondKind::Or(or) = &inner.kind else {
            panic!()
        };
        assert!(matches!(or[1].kind, CondKind::Between { .. }));
    }

    #[test]
    fn in_list_and_predicates() {
        let (_, c, done) =
            cond_of("IF a IN ( 'X', b ) AND line_exists( t[ k = 1 ] ) AND o IS INSTANCE OF zcl.");
        assert!(done);
        let CondKind::And(and) = c.kind else { panic!() };
        let CondKind::In { values, .. } = &and[0].kind else {
            panic!()
        };
        assert_eq!(values.len(), 2);
        assert!(matches!(and[1].kind, CondKind::Predicate(_)));
        assert!(matches!(
            &and[2].kind,
            CondKind::Is { predicate, class: Some(_), .. } if predicate == "INSTANCE OF"
        ));
    }
}
