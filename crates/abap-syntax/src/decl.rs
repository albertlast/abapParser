//! Parsers for declaration statements: DATA, TYPES, CONSTANTS, METHODS,
//! CLASS ... DEFINITION, FORM, FUNCTION and friends.
//!
//! The parsers are lenient: they extract what they recognize and skip the
//! rest, so new or exotic additions never make a whole declaration disappear.

use crate::{File, Statement, Token, TokenKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TypingKind {
    /// `TYPE t`
    Type,
    /// `LIKE dobj`
    Like,
    /// `TYPE REF TO t`
    TypeRefTo,
    /// `LIKE REF TO dobj`
    LikeRefTo,
    /// `TYPE|LIKE [STANDARD|SORTED|HASHED|ANY|INDEX] TABLE [OF ...]`, `OCCURS n`
    Table,
    /// `TYPE|LIKE RANGE OF t`
    Range,
    /// `TYPE|LIKE LINE OF t`
    LineOf,
    /// `STRUCTURE s` (FORM and function module parameters)
    Structure,
    /// `FOR dobj` (SELECT-OPTIONS)
    For,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TableTyping {
    /// `standard`, `sorted`, `hashed`, `any` or `index`, lower case
    pub kind: Option<String>,
    /// primary key uniqueness, when stated
    pub unique: Option<bool>,
    /// key definition text, e.g. `WITH UNIQUE KEY name`
    pub key: Option<String>,
    /// line type is `REF TO ...`
    pub line_is_ref: bool,
    pub header_line: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Typing {
    pub kind: TypingKind,
    /// The referenced type or data object, for tables the line type.
    pub name: Option<String>,
    pub table: Option<TableTyping>,
    pub length: Option<String>,
    pub decimals: Option<String>,
    /// Source text of the typing clause.
    pub text: String,
}

/// DATA, CLASS-DATA, STATICS, CONSTANTS, TYPES, FIELD-SYMBOLS, PARAMETERS,
/// SELECT-OPTIONS, TABLES and the BEGIN OF / END OF statements.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Variable {
    pub name: String,
    pub token: usize,
    pub typing: Option<Typing>,
    pub value: Option<String>,
    pub read_only: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    Importing,
    Exporting,
    Changing,
    Returning,
    /// FORM ... USING
    Using,
    /// FORM and function module TABLES
    Tables,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Parameter {
    pub name: String,
    pub direction: Direction,
    pub typing: Option<Typing>,
    pub by_value: bool,
    pub optional: bool,
    pub default: Option<String>,
    pub preferred: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MethodDef {
    pub name: String,
    pub token: usize,
    pub is_static: bool,
    pub is_abstract: bool,
    pub is_final: bool,
    pub is_redefinition: bool,
    pub is_for_testing: bool,
    /// `DEFAULT IGNORE` / `DEFAULT FAIL`, lower case
    pub default: Option<String>,
    /// `FOR EVENT event OF class`
    pub event: Option<(String, String)>,
    /// RAP handler kind, text after `FOR`, e.g. `MODIFY` or `INSTANCE FEATURES`
    pub rap_handler: Option<String>,
    pub parameters: Vec<Parameter>,
    /// class based exceptions, `RAISING`
    pub raising: Vec<String>,
    /// classic exceptions, `EXCEPTIONS`
    pub exceptions: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ClassDef {
    pub name: String,
    pub token: usize,
    pub is_public: bool,
    pub superclass: Option<String>,
    pub is_abstract: bool,
    pub is_final: bool,
    /// `public`, `protected` or `private`
    pub create: Option<String>,
    pub is_for_testing: bool,
    pub risk_level: Option<String>,
    pub duration: Option<String>,
    pub friends: Vec<String>,
    pub shared_memory: bool,
    pub behavior_of: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct InterfacesDef {
    pub name: String,
    pub partially_implemented: bool,
    pub abstract_methods: Vec<String>,
    pub final_methods: Vec<String>,
    pub all_abstract: bool,
    pub all_final: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AliasDef {
    pub name: String,
    pub component: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventDef {
    pub name: String,
    pub is_static: bool,
    pub parameters: Vec<Parameter>,
}

/// FORM, FUNCTION and EVENTS signatures.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Signature {
    pub name: String,
    pub token: usize,
    pub parameters: Vec<Parameter>,
    pub raising: Vec<String>,
    pub exceptions: Vec<String>,
}

pub struct Cursor<'a> {
    tokens: Vec<(usize, &'a Token)>,
    pos: usize,
}

impl<'a> Cursor<'a> {
    /// Statement tokens without the terminating `.` or `,`.
    pub fn new(file: &'a File, s: &Statement) -> Self {
        let mut tokens: Vec<(usize, &Token)> = s
            .tokens
            .iter()
            .map(|i| (*i, &file.lexed.tokens[*i]))
            .collect();
        if tokens
            .last()
            .is_some_and(|(_, t)| t.kind == TokenKind::Punctuation)
        {
            tokens.pop();
        }
        tokens.retain(|(_, t)| t.kind != TokenKind::Pragma);
        Cursor { tokens, pos: 0 }
    }

    pub fn from_tokens(tokens: &'a [Token]) -> Self {
        Cursor {
            tokens: tokens
                .iter()
                .enumerate()
                .filter(|(_, t)| !matches!(t.kind, TokenKind::Comment | TokenKind::Pragma))
                .collect(),
            pos: 0,
        }
    }

    pub fn done(&self) -> bool {
        self.pos >= self.tokens.len()
    }

    fn token(&self, offset: usize) -> Option<&'a Token> {
        self.tokens.get(self.pos + offset).map(|(_, t)| *t)
    }

    fn index(&self) -> usize {
        self.tokens.get(self.pos).map(|(i, _)| *i).unwrap_or(0)
    }

    /// The upper case keyword word at an offset, dash-joined words like
    /// `READ-ONLY` count as one word. Returns the word and its token count.
    fn word_at(&self, offset: usize) -> Option<(String, usize)> {
        let mut k = self.pos + offset;
        let first = self.tokens.get(k)?.1;
        let mut text = first.upper();
        let mut n = 1;
        while first.kind == TokenKind::Identifier
            && k + 2 < self.tokens.len()
            && self.tokens[k + 1].1.kind == TokenKind::Dash
            && !self.tokens[k + 1].1.ws_before
            && !self.tokens[k + 1].1.ws_after
            && self.tokens[k + 2].1.kind == TokenKind::Identifier
        {
            text.push('-');
            text.push_str(&self.tokens[k + 2].1.upper());
            k += 2;
            n += 2;
        }
        Some((text, n))
    }

    pub fn at(&self, word: &str) -> bool {
        self.word_at(0).is_some_and(|(w, _)| w == word)
    }

    /// Words following each other, space separated in `words`.
    pub fn at_seq(&self, words: &str) -> bool {
        let mut offset = 0;
        for w in words.split(' ') {
            match self.word_at(offset) {
                Some((text, n)) if text == w => offset += n,
                _ => return false,
            }
        }
        true
    }

    pub fn eat(&mut self, word: &str) -> bool {
        self.eat_seq(word)
    }

    pub fn eat_seq(&mut self, words: &str) -> bool {
        let mut offset = 0;
        for w in words.split(' ') {
            match self.word_at(offset) {
                Some((text, n)) if text == w => offset += n,
                _ => return false,
            }
        }
        self.pos += offset;
        true
    }

    /// Skips one word.
    pub fn bump(&mut self) {
        if let Some((_, n)) = self.word_at(0) {
            self.pos += n;
        }
    }

    fn text(&self, from: usize, to: usize) -> String {
        crate::join_tokens(
            self.tokens[from..to.min(self.tokens.len())]
                .iter()
                .map(|(_, t)| *t),
        )
    }

    /// One operand: tokens connected without whitespace, including the
    /// contents of attached parentheses, e.g. `zcl_x=>ty`, `ls-comp`, `foo( a )`.
    pub fn operand(&mut self) -> Option<String> {
        let start = self.pos;
        self.token(0)?;
        let mut depth = 0u32;
        while let Some(t) = self.token(0) {
            if self.pos > start && depth == 0 && (t.ws_before || t.kind == TokenKind::Punctuation) {
                break;
            }
            match t.kind {
                TokenKind::ParenLeft | TokenKind::BracketLeft => depth += 1,
                TokenKind::ParenRight | TokenKind::BracketRight => depth = depth.saturating_sub(1),
                _ => {}
            }
            self.pos += 1;
        }
        Some(self.text(start, self.pos))
    }

    /// A name, plus an attached length like `foo(10)`; returns the name and token index.
    fn name(&mut self) -> Option<(String, usize, Option<String>)> {
        let t = self.token(0)?;
        let index = self.index();
        let name = unescape(&t.text);
        self.pos += 1;
        let mut length = None;
        if self
            .token(0)
            .is_some_and(|p| p.kind == TokenKind::ParenLeft && !p.ws_before)
        {
            let start = self.pos + 1;
            while self
                .token(0)
                .is_some_and(|p| p.kind != TokenKind::ParenRight)
            {
                self.pos += 1;
            }
            length = Some(self.text(start, self.pos));
            self.pos += 1;
        }
        Some((name, index, length))
    }

    fn rest(&mut self) -> String {
        let s = self.text(self.pos, self.tokens.len());
        self.pos = self.tokens.len();
        s
    }
}

/// `!name` escapes names that equal keywords.
fn unescape(name: &str) -> String {
    name.strip_prefix('!').unwrap_or(name).to_string()
}

const PARAM_SECTIONS: &[&str] = &[
    "IMPORTING",
    "EXPORTING",
    "CHANGING",
    "RETURNING",
    "RAISING",
    "EXCEPTIONS",
    "PREFERRED",
    "TABLES",
    "USING",
    "RESUMABLE",
];

/// Words that end a typing clause.
const TYPING_STOP: &[&str] = &[
    "VALUE",
    "READ-ONLY",
    "OPTIONAL",
    "DEFAULT",
    "INITIAL",
    "BOXED",
    "OBLIGATORY",
    "LENGTH",
    "DECIMALS",
    "IMPORTING",
    "EXPORTING",
    "CHANGING",
    "RETURNING",
    "RAISING",
    "EXCEPTIONS",
    "PREFERRED",
];

fn is_stop(c: &Cursor, stops: &[&str]) -> bool {
    c.word_at(0)
        .is_some_and(|(w, _)| stops.contains(&w.as_str()))
}

/// Parses a typing clause at the cursor, `None` if there is none.
pub fn typing(c: &mut Cursor) -> Option<Typing> {
    let start = c.pos;
    let mut t = Typing {
        kind: TypingKind::Type,
        name: None,
        table: None,
        length: None,
        decimals: None,
        text: String::new(),
    };
    if c.eat("TYPE") || c.eat("LIKE") {
        let like = c.text(start, start + 1).eq_ignore_ascii_case("LIKE");
        if c.eat_seq("REF TO") {
            t.kind = if like {
                TypingKind::LikeRefTo
            } else {
                TypingKind::TypeRefTo
            };
            t.name = c.operand();
        } else if c.eat_seq("LINE OF") {
            t.kind = TypingKind::LineOf;
            t.name = c.operand();
        } else if c.eat_seq("RANGE OF") {
            t.kind = TypingKind::Range;
            t.name = c.operand();
        } else if table_ahead(c) {
            table(c, &mut t);
        } else {
            t.kind = if like {
                TypingKind::Like
            } else {
                TypingKind::Type
            };
            t.name = c.operand();
        }
    } else if c.eat("STRUCTURE") {
        t.kind = TypingKind::Structure;
        t.name = c.operand();
    } else if c.eat("FOR") {
        t.kind = TypingKind::For;
        t.name = c.operand();
    } else {
        return None;
    }
    // LENGTH n / DECIMALS n; not a parameter that happens to be called so,
    // like `decimals TYPE i` in a parameter list
    let addition = |c: &Cursor, word: &str| {
        c.at(word)
            && !c
                .word_at(1)
                .is_some_and(|(w, _)| w == "TYPE" || w == "LIKE")
    };
    loop {
        if addition(c, "LENGTH") {
            c.bump();
            t.length = c.operand();
        } else if addition(c, "DECIMALS") {
            c.bump();
            t.decimals = c.operand();
        } else {
            break;
        }
    }
    t.text = c.text(start, c.pos);
    Some(t)
}

fn table_ahead(c: &Cursor) -> bool {
    c.at("TABLE")
        || ["STANDARD", "SORTED", "HASHED", "ANY", "INDEX"]
            .iter()
            .any(|k| c.at_seq(&format!("{k} TABLE")))
}

fn table(c: &mut Cursor, t: &mut Typing) {
    t.kind = TypingKind::Table;
    let mut tt = TableTyping::default();
    if !c.at("TABLE") {
        tt.kind = c.word_at(0).map(|(w, _)| w.to_ascii_lowercase());
        c.bump();
    }
    c.eat("TABLE");
    if c.eat("OF") {
        tt.line_is_ref = c.eat_seq("REF TO");
        t.name = c.operand();
    }
    let mut keys: Vec<String> = Vec::new();
    while c.at("WITH") {
        if c.at_seq("WITH HEADER LINE") {
            c.eat_seq("WITH HEADER LINE");
            tt.header_line = true;
            continue;
        }
        let start = c.pos;
        c.bump();
        if keys.is_empty() {
            if c.at("UNIQUE") {
                tt.unique = Some(true);
            } else if c.at("NON-UNIQUE") {
                tt.unique = Some(false);
            }
        }
        c.bump();
        // "DEFAULT" belongs to the key in "WITH NON-UNIQUE DEFAULT KEY"
        while !c.done() && !c.at("WITH") && !(is_stop(c, TYPING_STOP) && !c.at("DEFAULT")) {
            c.bump();
        }
        keys.push(c.text(start, c.pos));
    }
    if c.eat_seq("INITIAL SIZE") {
        c.operand();
    }
    if !keys.is_empty() {
        tt.key = Some(keys.join(" "));
    }
    t.table = Some(tt);
}

/// DATA, CLASS-DATA, STATICS, CONSTANTS, TYPES, FIELD-SYMBOLS, PARAMETERS,
/// SELECT-OPTIONS, RANGES, TABLES, NODES and the BEGIN OF/END OF variants.
pub fn variable(file: &File, s: &Statement) -> Option<Variable> {
    let mut c = Cursor::new(file, s);
    c.bump(); // the keyword
    if c.eat_seq("BEGIN OF") || c.eat_seq("END OF") {
        c.eat("ENUM");
        c.eat("MESH");
        c.eat_seq("COMMON PART");
    }
    let (name, token, length) = c.name()?;
    let mut v = Variable {
        name,
        token,
        typing: typing(&mut c),
        value: None,
        read_only: false,
    };
    if c.eat("OCCURS") {
        let text = c.operand().unwrap_or_default();
        v.typing = Some(Typing {
            kind: TypingKind::Table,
            name: None,
            table: Some(TableTyping::default()),
            length: None,
            decimals: None,
            text: format!("OCCURS {text}"),
        });
    }
    if let (Some(len), Some(t)) = (&length, v.typing.as_mut()) {
        t.length.get_or_insert(len.clone());
    }
    while !c.done() {
        if c.eat("VALUE") {
            v.value = if c.eat_seq("IS INITIAL") {
                Some("IS INITIAL".to_string())
            } else {
                c.operand()
            };
        } else if c.eat("READ-ONLY") {
            v.read_only = true;
        } else if c.eat("LENGTH") {
            let len = c.operand();
            if let Some(t) = v.typing.as_mut() {
                t.length = len;
            }
        } else if c.eat("DECIMALS") {
            let dec = c.operand();
            if let Some(t) = v.typing.as_mut() {
                t.decimals = dec;
            }
        } else if c.eat_seq("WITH HEADER LINE") {
            if let Some(tt) = v.typing.as_mut().and_then(|t| t.table.as_mut()) {
                tt.header_line = true;
            }
        } else if v.typing.is_none() && (c.at("TYPE") || c.at("LIKE") || c.at("FOR")) {
            v.typing = typing(&mut c);
        } else {
            c.bump();
        }
    }
    Some(v)
}

/// A parameter list after IMPORTING, EXPORTING, USING, ...
fn parameters(c: &mut Cursor, direction: Direction, out: &mut Vec<Parameter>) {
    while !c.done() && !is_stop(c, PARAM_SECTIONS) {
        let mut by_value = false;
        let name = if (c.at("VALUE") || c.at("REFERENCE"))
            && c.token(1)
                .is_some_and(|t| t.kind == TokenKind::ParenLeft && !t.ws_before)
        {
            by_value = c.at("VALUE");
            c.pos += 2;
            let n = c.token(0).map(|t| unescape(&t.text));
            c.pos += 1;
            if c.token(0).is_some_and(|t| t.kind == TokenKind::ParenRight) {
                c.pos += 1;
            }
            n
        } else {
            let n = c.token(0).map(|t| unescape(&t.text));
            c.pos += 1;
            n
        };
        let Some(name) = name else { break };
        let mut p = Parameter {
            name,
            direction,
            typing: typing(c),
            by_value,
            optional: false,
            default: None,
            preferred: false,
        };
        loop {
            if c.eat("OPTIONAL") {
                p.optional = true;
            } else if c.eat("DEFAULT") {
                p.default = c.operand();
                p.optional = true;
            } else {
                break;
            }
        }
        out.push(p);
    }
}

/// Exception names after RAISING or EXCEPTIONS, `RESUMABLE( cx )` included.
fn names(c: &mut Cursor, out: &mut Vec<String>) {
    while !c.done() && !is_stop(c, &PARAM_SECTIONS[..PARAM_SECTIONS.len() - 1]) {
        if c.at("RESUMABLE") || c.at("MESSAGE") {
            c.pos += 1;
            if c.token(0).is_some_and(|t| t.kind == TokenKind::ParenLeft) {
                c.pos += 1;
                if let Some(t) = c.token(0) {
                    out.push(t.text.clone());
                }
                c.pos += 2;
            }
            continue;
        }
        if let Some(t) = c.token(0) {
            if t.kind == TokenKind::Identifier {
                out.push(t.text.clone());
            }
        }
        c.pos += 1;
    }
}

/// METHODS and CLASS-METHODS.
pub fn method_def(file: &File, s: &Statement) -> Option<MethodDef> {
    let mut c = Cursor::new(file, s);
    let is_static = c.at("CLASS-METHODS");
    c.bump();
    let (name, token, _) = c.name()?;
    let mut m = MethodDef {
        name,
        token,
        is_static,
        ..Default::default()
    };
    while !c.done() {
        if c.eat("ABSTRACT") {
            m.is_abstract = true;
        } else if c.eat("FINAL") {
            m.is_final = true;
        } else if c.eat("REDEFINITION") {
            m.is_redefinition = true;
        } else if c.eat_seq("FOR TESTING") {
            m.is_for_testing = true;
        } else if c.eat("DEFAULT") {
            m.default = c.word_at(0).map(|(w, _)| w.to_ascii_lowercase());
            c.bump();
        } else if c.eat_seq("FOR EVENT") {
            let event = c.operand().unwrap_or_default();
            c.eat("OF");
            let class = c.operand().unwrap_or_default();
            m.event = Some((class, event));
        } else if c.eat("FOR") {
            // RAP handler methods: FOR MODIFY, FOR DETERMINE ON SAVE, FOR INSTANCE FEATURES, ...
            let start = c.pos;
            while !c.done() && !is_stop(&c, PARAM_SECTIONS) {
                c.bump();
            }
            m.rap_handler = Some(c.text(start, c.pos));
            rap_parameters(&mut c, &mut m);
        } else if c.eat("IMPORTING") {
            parameters(&mut c, Direction::Importing, &mut m.parameters);
        } else if c.eat("EXPORTING") {
            parameters(&mut c, Direction::Exporting, &mut m.parameters);
        } else if c.eat("CHANGING") {
            parameters(&mut c, Direction::Changing, &mut m.parameters);
        } else if c.eat("RETURNING") {
            parameters(&mut c, Direction::Returning, &mut m.parameters);
        } else if c.eat_seq("PREFERRED PARAMETER") {
            if let Some(t) = c.token(0) {
                let name = t.text.clone();
                for p in m.parameters.iter_mut() {
                    p.preferred = p.name.eq_ignore_ascii_case(&name);
                }
            }
            c.pos += 1;
        } else if c.eat("RAISING") {
            names(&mut c, &mut m.raising);
        } else if c.eat("EXCEPTIONS") {
            names(&mut c, &mut m.exceptions);
        } else {
            c.bump();
        }
    }
    Some(m)
}

/// RAP handler signatures: `IMPORTING keys FOR entity REQUEST req RESULT result`.
fn rap_parameters(c: &mut Cursor, m: &mut MethodDef) {
    const RAP_WORDS: &[&str] = &[
        "IMPORTING",
        "REQUEST",
        "RESULT",
        "CHANGING",
        "FOR",
        "RAISING",
    ];
    let mut direction = None;
    while !c.done() {
        if c.eat("RAISING") {
            names(c, &mut m.raising);
            continue;
        }
        let next_direction = if c.eat("IMPORTING") || c.eat("REQUEST") {
            Some(Direction::Importing)
        } else if c.eat("RESULT") {
            Some(Direction::Exporting)
        } else if c.eat("CHANGING") {
            Some(Direction::Changing)
        } else {
            None
        };
        if let Some(d) = next_direction {
            direction = Some(d);
            if let Some((name, _, _)) = c.name() {
                m.parameters.push(Parameter {
                    name,
                    direction: d,
                    typing: None,
                    by_value: false,
                    optional: false,
                    default: None,
                    preferred: false,
                });
            }
        } else if c.eat("FOR") {
            while !c.done() && !is_stop(c, RAP_WORDS) {
                c.bump();
            }
        } else if direction == Some(Direction::Changing) {
            // CHANGING failed TYPE ... reported TYPE ...
            if let Some((name, _, _)) = c.name() {
                m.parameters.push(Parameter {
                    name,
                    direction: Direction::Changing,
                    typing: None,
                    by_value: false,
                    optional: false,
                    default: None,
                    preferred: false,
                });
            }
        } else {
            c.bump();
        }
    }
}

/// CLASS ... DEFINITION.
pub fn class_def(file: &File, s: &Statement) -> Option<ClassDef> {
    let mut c = Cursor::new(file, s);
    c.bump(); // CLASS
    let (name, token, _) = c.name()?;
    c.eat("DEFINITION");
    let mut d = ClassDef {
        name,
        token,
        ..Default::default()
    };
    while !c.done() {
        if c.eat("PUBLIC") {
            d.is_public = true;
        } else if c.eat_seq("INHERITING FROM") {
            d.superclass = c.operand();
        } else if c.eat("ABSTRACT") {
            d.is_abstract = true;
        } else if c.eat("FINAL") {
            d.is_final = true;
        } else if c.eat("CREATE") {
            d.create = c.word_at(0).map(|(w, _)| w.to_ascii_lowercase());
            c.bump();
        } else if c.eat_seq("SHARED MEMORY ENABLED") {
            d.shared_memory = true;
        } else if c.eat_seq("FOR TESTING") {
            d.is_for_testing = true;
        } else if c.eat_seq("RISK LEVEL") {
            d.risk_level = c.word_at(0).map(|(w, _)| w.to_ascii_lowercase());
            c.bump();
        } else if c.eat("DURATION") {
            d.duration = c.word_at(0).map(|(w, _)| w.to_ascii_lowercase());
            c.bump();
        } else if c.eat_seq("FOR BEHAVIOR OF") {
            d.behavior_of = c.operand();
        } else if c.eat_seq("GLOBAL FRIENDS") || c.eat("FRIENDS") {
            const OPTIONS: &[&str] = &[
                "PUBLIC",
                "INHERITING",
                "ABSTRACT",
                "FINAL",
                "CREATE",
                "SHARED",
                "FOR",
                "RISK",
                "DURATION",
                "GLOBAL",
                "FRIENDS",
            ];
            while !c.done() && !is_stop(&c, OPTIONS) {
                if let Some(n) = c.operand() {
                    d.friends.push(n);
                }
            }
        } else {
            c.bump();
        }
    }
    Some(d)
}

/// INTERFACE name [PUBLIC]: returns the name and whether it is global.
pub fn interface_def(file: &File, s: &Statement) -> Option<(String, usize, bool)> {
    let mut c = Cursor::new(file, s);
    c.bump();
    let (name, token, _) = c.name()?;
    let mut public = false;
    while !c.done() {
        public |= c.eat("PUBLIC");
        c.bump();
    }
    Some((name, token, public))
}

/// INTERFACES name [PARTIALLY IMPLEMENTED] [ABSTRACT METHODS ...] [FINAL METHODS ...]
pub fn interfaces_def(file: &File, s: &Statement) -> Option<InterfacesDef> {
    let mut c = Cursor::new(file, s);
    c.bump();
    let (name, _, _) = c.name()?;
    let mut d = InterfacesDef {
        name,
        ..Default::default()
    };
    let mut list: Option<bool> = None; // Some(true) abstract, Some(false) final
    while !c.done() {
        if c.eat_seq("PARTIALLY IMPLEMENTED") {
            d.partially_implemented = true;
            list = None;
        } else if c.eat_seq("ABSTRACT METHODS") {
            list = Some(true);
        } else if c.eat_seq("FINAL METHODS") {
            list = Some(false);
        } else if c.eat_seq("ALL METHODS ABSTRACT") {
            d.all_abstract = true;
            list = None;
        } else if c.eat_seq("ALL METHODS FINAL") {
            d.all_final = true;
            list = None;
        } else if c.eat_seq("DATA VALUES") {
            break;
        } else {
            match (list, c.operand()) {
                (Some(true), Some(n)) => d.abstract_methods.push(n),
                (Some(false), Some(n)) => d.final_methods.push(n),
                _ => {}
            }
        }
    }
    Some(d)
}

/// ALIASES name FOR intf~component.
pub fn alias_def(file: &File, s: &Statement) -> Option<AliasDef> {
    let mut c = Cursor::new(file, s);
    c.bump();
    let (name, _, _) = c.name()?;
    c.eat("FOR");
    Some(AliasDef {
        name,
        component: c.operand()?,
    })
}

/// EVENTS and CLASS-EVENTS.
pub fn event_def(file: &File, s: &Statement) -> Option<EventDef> {
    let mut c = Cursor::new(file, s);
    let is_static = c.at("CLASS-EVENTS");
    c.bump();
    let (name, _, _) = c.name()?;
    let mut params = Vec::new();
    if c.eat("EXPORTING") {
        parameters(&mut c, Direction::Exporting, &mut params);
    }
    Some(EventDef {
        name,
        is_static,
        parameters: params,
    })
}

/// FORM name [TABLES ...] [USING ...] [CHANGING ...] [RAISING ...].
pub fn form_def(file: &File, s: &Statement) -> Option<Signature> {
    let mut c = Cursor::new(file, s);
    c.bump();
    let (name, token, _) = c.name()?;
    let mut sig = Signature {
        name,
        token,
        ..Default::default()
    };
    signature_sections(&mut c, &mut sig);
    Some(sig)
}

fn signature_sections(c: &mut Cursor, sig: &mut Signature) {
    while !c.done() {
        if c.eat("TABLES") {
            parameters(c, Direction::Tables, &mut sig.parameters);
        } else if c.eat("USING") {
            parameters(c, Direction::Using, &mut sig.parameters);
        } else if c.eat("IMPORTING") {
            parameters(c, Direction::Importing, &mut sig.parameters);
        } else if c.eat("EXPORTING") {
            parameters(c, Direction::Exporting, &mut sig.parameters);
        } else if c.eat("CHANGING") {
            parameters(c, Direction::Changing, &mut sig.parameters);
        } else if c.eat("RAISING") {
            names(c, &mut sig.raising);
        } else if c.eat("EXCEPTIONS") {
            names(c, &mut sig.exceptions);
        } else {
            c.bump();
        }
    }
}

/// FUNCTION name, with the signature either in the statement itself or in
/// the generated `*"` comment block ("Local Interface") that follows it.
pub fn function_def(file: &File, statement_index: usize) -> Option<Signature> {
    let s = &file.statements[statement_index];
    let mut c = Cursor::new(file, s);
    c.bump();
    let (name, token, _) = c.name()?;
    let mut sig = Signature {
        name,
        token,
        ..Default::default()
    };
    signature_sections(&mut c, &mut sig);
    if sig.parameters.is_empty() && sig.raising.is_empty() && sig.exceptions.is_empty() {
        let mut text = String::new();
        for st in &file.statements[statement_index + 1..] {
            if st.kind != crate::StatementKind::Comment {
                break;
            }
            let comment = &file.lexed.tokens[st.tokens[0]].text;
            if let Some(line) = comment.strip_prefix("*\"") {
                if !line.starts_with('*') && !line.starts_with('-') {
                    text.push_str(line);
                    text.push('\n');
                }
            }
        }
        if !text.is_empty() {
            let lexed = crate::lex(&text);
            let mut c = Cursor::from_tokens(&lexed.tokens);
            signature_sections(&mut c, &mut sig);
        }
    }
    Some(sig)
}

/// The first name after the keyword(s), e.g. for METHOD, MODULE, INCLUDE,
/// REPORT, DEFINE. `skip` keywords are consumed first.
pub fn first_name(file: &File, s: &Statement, skip: usize) -> Option<(String, usize)> {
    let mut c = Cursor::new(file, s);
    for _ in 0..skip {
        c.bump();
    }
    let index = c.index();
    c.operand().map(|n| (n, index))
}

/// Remaining source text of a statement after `skip` words.
pub fn text_after(file: &File, s: &Statement, skip: usize) -> String {
    let mut c = Cursor::new(file, s);
    for _ in 0..skip {
        c.bump();
    }
    c.rest()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{parse, StatementKind};

    fn first(f: &File, kind: StatementKind) -> &Statement {
        f.statements.iter().find(|s| s.kind == kind).unwrap()
    }

    #[test]
    fn data_with_table_typing() {
        let f = parse("DATA lt_x TYPE SORTED TABLE OF zline WITH UNIQUE KEY a b.");
        let v = variable(&f, first(&f, StatementKind::Data)).unwrap();
        assert_eq!(v.name, "lt_x");
        let t = v.typing.unwrap();
        assert_eq!(t.kind, TypingKind::Table);
        assert_eq!(t.name.as_deref(), Some("zline"));
        let tt = t.table.unwrap();
        assert_eq!(tt.kind.as_deref(), Some("sorted"));
        assert_eq!(tt.unique, Some(true));
        assert_eq!(tt.key.as_deref(), Some("WITH UNIQUE KEY a b"));
    }

    #[test]
    fn data_variants() {
        let f = parse(
            "DATA: foo(10) TYPE c VALUE 'X', r TYPE REF TO zcl_a READ-ONLY, s LIKE sy-tabix, \
             t TYPE zcl_x=>ty_t, e.",
        );
        let vs: Vec<_> = f
            .statements
            .iter()
            .map(|s| variable(&f, s).unwrap())
            .collect();
        assert_eq!(vs[0].typing.as_ref().unwrap().length.as_deref(), Some("10"));
        assert_eq!(vs[0].value.as_deref(), Some("'X'"));
        assert_eq!(vs[1].typing.as_ref().unwrap().kind, TypingKind::TypeRefTo);
        assert!(vs[1].read_only);
        assert_eq!(
            vs[2].typing.as_ref().unwrap().name.as_deref(),
            Some("sy-tabix")
        );
        assert_eq!(
            vs[3].typing.as_ref().unwrap().name.as_deref(),
            Some("zcl_x=>ty_t")
        );
        assert_eq!(vs[4].name, "e");
        assert!(vs[4].typing.is_none());
    }

    #[test]
    fn method_signature() {
        let f = parse(
            "CLASS-METHODS run IMPORTING iv_a TYPE string VALUE(iv_b) TYPE i DEFAULT 5 \
             it_c TYPE STANDARD TABLE EXPORTING ev_d TYPE REF TO zcl_d \
             RETURNING VALUE(rv_e) TYPE abap_bool RAISING zcx_a RESUMABLE(zcx_b) EXCEPTIONS not_found.",
        );
        let m = method_def(&f, first(&f, StatementKind::MethodDef)).unwrap();
        assert!(m.is_static);
        assert_eq!(m.name, "run");
        let names: Vec<_> = m
            .parameters
            .iter()
            .map(|p| (p.name.as_str(), p.direction))
            .collect();
        assert_eq!(
            names,
            [
                ("iv_a", Direction::Importing),
                ("iv_b", Direction::Importing),
                ("it_c", Direction::Importing),
                ("ev_d", Direction::Exporting),
                ("rv_e", Direction::Returning)
            ]
        );
        assert!(m.parameters[1].by_value && m.parameters[1].optional);
        assert_eq!(m.parameters[1].default.as_deref(), Some("5"));
        assert_eq!(
            m.parameters[2].typing.as_ref().unwrap().kind,
            TypingKind::Table
        );
        assert_eq!(m.raising, ["zcx_a", "zcx_b"]);
        assert_eq!(m.exceptions, ["not_found"]);
    }

    #[test]
    fn parameter_named_like_an_addition() {
        let f = parse(
            "METHODS conv IMPORTING val TYPE numeric decimals TYPE i DEFAULT -1 length TYPE i.",
        );
        let m = method_def(&f, first(&f, StatementKind::MethodDef)).unwrap();
        let names: Vec<_> = m.parameters.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(names, ["val", "decimals", "length"]);
        assert_eq!(m.parameters[1].default.as_deref(), Some("-1"));
        let d = parse("DATA amount TYPE p LENGTH 10 DECIMALS 2.");
        let v = variable(&d, &d.statements[0]).unwrap();
        let t = v.typing.unwrap();
        assert_eq!(
            (t.length.as_deref(), t.decimals.as_deref()),
            (Some("10"), Some("2"))
        );
    }

    #[test]
    fn event_handler() {
        let f = parse("METHODS on_click FOR EVENT clicked OF zcl_btn IMPORTING sender e_row.");
        let m = method_def(&f, first(&f, StatementKind::MethodDef)).unwrap();
        assert_eq!(m.event, Some(("zcl_btn".into(), "clicked".into())));
        assert_eq!(m.parameters.len(), 2);
        assert!(m.parameters.iter().all(|p| p.typing.is_none()));
    }

    #[test]
    fn class_definition() {
        let f = parse(
            "CLASS zcl_a DEFINITION PUBLIC INHERITING FROM zcl_b FINAL CREATE PRIVATE GLOBAL FRIENDS zcl_c zcl_d.",
        );
        let d = class_def(&f, first(&f, StatementKind::ClassDefinition)).unwrap();
        assert!(d.is_public && d.is_final);
        assert_eq!(d.superclass.as_deref(), Some("zcl_b"));
        assert_eq!(d.create.as_deref(), Some("private"));
        assert_eq!(d.friends, ["zcl_c", "zcl_d"]);
    }

    #[test]
    fn form_signature() {
        let f = parse(
            "FORM f TABLES t STRUCTURE zs USING p1 VALUE(p2) TYPE i CHANGING c LIKE sy-subrc.",
        );
        let s = form_def(&f, first(&f, StatementKind::Form)).unwrap();
        let p: Vec<_> = s
            .parameters
            .iter()
            .map(|p| (p.name.as_str(), p.direction))
            .collect();
        assert_eq!(
            p,
            [
                ("t", Direction::Tables),
                ("p1", Direction::Using),
                ("p2", Direction::Using),
                ("c", Direction::Changing)
            ]
        );
        assert_eq!(
            s.parameters[0].typing.as_ref().unwrap().kind,
            TypingKind::Structure
        );
        assert!(s.parameters[1].typing.is_none());
    }

    #[test]
    fn function_module_comment_signature() {
        let src = "FUNCTION z_fm.\n\
*\"----------------------------------------------------------------------\n\
*\"*\"Local Interface:\n\
*\"  IMPORTING\n\
*\"     VALUE(IV_A) TYPE  STRING\n\
*\"     REFERENCE(IV_B) TYPE  I OPTIONAL\n\
*\"  TABLES\n\
*\"      ET_X STRUCTURE  ZSTRUC\n\
*\"  EXCEPTIONS\n\
*\"      NOT_FOUND\n\
*\"----------------------------------------------------------------------\n\
ENDFUNCTION.";
        let f = parse(src);
        let i = f
            .statements
            .iter()
            .position(|s| s.kind == StatementKind::FunctionModule)
            .unwrap();
        let s = function_def(&f, i).unwrap();
        assert_eq!(s.name, "z_fm");
        let p: Vec<_> = s
            .parameters
            .iter()
            .map(|p| (p.name.as_str(), p.by_value, p.optional))
            .collect();
        assert_eq!(
            p,
            [
                ("IV_A", true, false),
                ("IV_B", false, true),
                ("ET_X", false, false)
            ]
        );
        assert_eq!(s.exceptions, ["NOT_FOUND"]);
    }
}
