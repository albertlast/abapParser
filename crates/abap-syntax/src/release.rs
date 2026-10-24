//! The ABAP release a statement needs: the oldest on-premise release whose
//! syntax has everything the statement uses, with the construct that needs
//! it, like `inline declaration` for `DATA(x) = 1.`
//!
//! Releases a construct came with follow abaplint's grammar
//! (<https://github.com/abaplint/abaplint>, MIT). Constructs abaplint dates by
//! a cloud release (7.60 to 9.18) count from the on-premise release that
//! shipped it, 7.76 from 7.54 for example. Statements that only exist in an
//! older form are not told apart: a statement needs at least 7.00.

use crate::expr::{Access, Chain, Cond, CondKind, Expr, ExprKind, Head, Item, Link, Value};
use crate::operands::{operands, Operand};
use crate::{File, Statement, StatementKind as K, Token, TokenKind};

/// An on-premise release of SAP NetWeaver or the ABAP Platform, in order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Release {
    V700,
    V702,
    V740Sp02,
    V740Sp05,
    V740Sp08,
    V750,
    V751,
    V752,
    V753,
    V754,
    V755,
    V756,
    V757,
    V758,
    V816,
}

impl Release {
    pub const ALL: [Release; 15] = [
        Release::V700,
        Release::V702,
        Release::V740Sp02,
        Release::V740Sp05,
        Release::V740Sp08,
        Release::V750,
        Release::V751,
        Release::V752,
        Release::V753,
        Release::V754,
        Release::V755,
        Release::V756,
        Release::V757,
        Release::V758,
        Release::V816,
    ];

    /// As SAP writes it: `7.40 SP05`, `7.50`, `8.16`.
    pub fn name(self) -> &'static str {
        match self {
            Release::V700 => "7.00",
            Release::V702 => "7.02",
            Release::V740Sp02 => "7.40 SP02",
            Release::V740Sp05 => "7.40 SP05",
            Release::V740Sp08 => "7.40 SP08",
            Release::V750 => "7.50",
            Release::V751 => "7.51",
            Release::V752 => "7.52",
            Release::V753 => "7.53",
            Release::V754 => "7.54",
            Release::V755 => "7.55",
            Release::V756 => "7.56",
            Release::V757 => "7.57",
            Release::V758 => "7.58",
            Release::V816 => "8.16",
        }
    }

    /// As abaplint names it: `v740sp05`, `v750`.
    pub fn id(self) -> &'static str {
        match self {
            Release::V700 => "v700",
            Release::V702 => "v702",
            Release::V740Sp02 => "v740sp02",
            Release::V740Sp05 => "v740sp05",
            Release::V740Sp08 => "v740sp08",
            Release::V750 => "v750",
            Release::V751 => "v751",
            Release::V752 => "v752",
            Release::V753 => "v753",
            Release::V754 => "v754",
            Release::V755 => "v755",
            Release::V756 => "v756",
            Release::V757 => "v757",
            Release::V758 => "v758",
            Release::V816 => "v816",
        }
    }

    /// The on-premise release that shipped a release of abaplint's list,
    /// cloud ones included: `v776` is 7.54. `None` for other names and for
    /// cloud releases not shipped on premise.
    pub fn of_abaplint(id: &str) -> Option<Release> {
        if let Some(r) = Release::ALL.iter().find(|r| r.id() == id) {
            return Some(*r);
        }
        let number: u32 = id.strip_prefix('v')?.parse().ok()?;
        Some(match number {
            701..=731 => Release::V702,
            760..=762 => Release::V750,
            763..=765 => Release::V751,
            766..=769 => Release::V752,
            770..=773 => Release::V753,
            774..=777 => Release::V754,
            778..=781 => Release::V755,
            782..=785 => Release::V756,
            786..=789 => Release::V757,
            790..=793 => Release::V758,
            794..=916 => Release::V816,
            _ => return None,
        })
    }
}

impl std::fmt::Display for Release {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}

impl std::str::FromStr for Release {
    type Err = String;

    /// `750`, `7.50`, `v750`, `740sp05`, `7.40 SP05`, `816`.
    fn from_str(s: &str) -> Result<Self, String> {
        let key: String = s
            .chars()
            .filter(|c| c.is_ascii_alphanumeric())
            .collect::<String>()
            .to_ascii_lowercase();
        let key = key.strip_prefix('v').unwrap_or(&key);
        Release::ALL
            .iter()
            .find(|r| &r.id()[1..] == key)
            .copied()
            .ok_or_else(|| {
                format!("unknown release {s}, known: 700, 702, 740sp02, 740sp05, 740sp08, 750 to 758, 816")
            })
    }
}

/// What a statement needs: the release and the construct that needs it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Need {
    pub release: Release,
    /// like `inline declaration` or `host variable with @`
    pub construct: &'static str,
}

/// The release the statement needs; `None` when 7.00 has all it uses.
pub fn statement_release(file: &File, s: &Statement) -> Option<Need> {
    statement_needs(file, s)
        .into_iter()
        .reduce(|a, b| if b.release > a.release { b } else { a })
}

/// Every construct of the statement newer than 7.00 with its release, each
/// once, in the order found.
pub fn statement_needs(file: &File, s: &Statement) -> Vec<Need> {
    if matches!(s.kind, K::Comment | K::Unknown | K::NativeSQL) {
        return Vec::new();
    }
    let mut scan = Scan::new(file, s);
    scan.tokens();
    scan.keywords();
    if is_sql(s.kind) {
        scan.sql();
    }
    scan.operands(file, s);
    scan.needs
}

/// The release a file needs: the highest of its statements, with the index
/// of the first statement that needs it.
pub fn file_release(file: &File) -> Option<(usize, Need)> {
    let mut best: Option<(usize, Need)> = None;
    for (i, s) in file.statements.iter().enumerate() {
        if let Some(n) = statement_release(file, s) {
            if best.is_none_or(|(_, b)| n.release > b.release) {
                best = Some((i, n));
            }
        }
    }
    best
}

fn is_sql(kind: K) -> bool {
    matches!(
        kind,
        K::Select
            | K::SelectLoop
            | K::With
            | K::WithLoop
            | K::OpenCursor
            | K::FetchNextCursor
            | K::InsertDatabase
            | K::UpdateDatabase
            | K::ModifyDatabase
            | K::DeleteDatabase
    )
}

struct Scan<'a> {
    kind: K,
    t: Vec<&'a Token>,
    /// upper case text of each token
    w: Vec<String>,
    /// parenthesis and bracket depth before each token
    depth: Vec<u32>,
    needs: Vec<Need>,
}

/// Constructor operators and the release that brought each.
const CONSTRUCTORS: [(&str, Release); 11] = [
    ("NEW", Release::V740Sp02),
    ("VALUE", Release::V740Sp02),
    ("CONV", Release::V740Sp02),
    ("CAST", Release::V740Sp02),
    ("REF", Release::V740Sp02),
    ("EXACT", Release::V740Sp02),
    ("COND", Release::V740Sp02),
    ("SWITCH", Release::V740Sp02),
    ("CORRESPONDING", Release::V740Sp05),
    ("REDUCE", Release::V740Sp08),
    ("FILTER", Release::V740Sp08),
];

/// Keyword sequences of statements and the release that brought them;
/// `?` matches any word.
const KEYWORDS: &[(&[K], &str, Release, &str)] = &[
    (
        &[
            K::Type,
            K::Data,
            K::CreateData,
            K::Static,
            K::ClassData,
            K::TypeBegin,
            K::DataBegin,
        ],
        "EMPTY KEY",
        Release::V740Sp02,
        "EMPTY KEY",
    ),
    (
        &[K::InterfaceDef],
        "PARTIALLY IMPLEMENTED",
        Release::V740Sp02,
        "PARTIALLY IMPLEMENTED",
    ),
    (
        &[K::CallTransaction],
        "WITH AUTHORITY-CHECK",
        Release::V740Sp02,
        "WITH AUTHORITY-CHECK",
    ),
    (
        &[K::CallTransaction],
        "WITHOUT AUTHORITY-CHECK",
        Release::V740Sp02,
        "WITHOUT AUTHORITY-CHECK",
    ),
    (
        &[K::MoveCorresponding],
        "EXPANDING NESTED TABLES",
        Release::V740Sp05,
        "EXPANDING NESTED TABLES",
    ),
    (
        &[K::MoveCorresponding],
        "KEEPING TARGET LINES",
        Release::V740Sp05,
        "KEEPING TARGET LINES",
    ),
    (
        &[K::MethodDef],
        "DEFAULT FAIL",
        Release::V740Sp08,
        "DEFAULT FAIL",
    ),
    (
        &[K::MethodDef],
        "DEFAULT IGNORE",
        Release::V740Sp08,
        "DEFAULT IGNORE",
    ),
    (
        &[K::Loop],
        "GROUP BY",
        Release::V740Sp08,
        "LOOP AT GROUP BY",
    ),
    (
        &[K::Loop],
        "LOOP AT GROUP",
        Release::V740Sp08,
        "LOOP AT GROUP",
    ),
    (
        &[K::Wait],
        "PUSH CHANNELS",
        Release::V750,
        "WAIT FOR PUSH CHANNELS",
    ),
    (
        &[K::InsertReport],
        "VERSION",
        Release::V750,
        "INSERT REPORT VERSION",
    ),
    (
        &[K::Raise],
        "USING MESSAGE",
        Release::V752,
        "RAISE EXCEPTION USING MESSAGE",
    ),
    (&[K::Find, K::Replace], "PCRE", Release::V755, "PCRE"),
    (
        &[K::Assign],
        "ELSE UNASSIGN",
        Release::V757,
        "ELSE UNASSIGN",
    ),
    (&[K::Loop], "STEP", Release::V757, "LOOP STEP"),
    (
        &[K::ClassDefinition],
        "FOR BEHAVIOR OF",
        Release::V754,
        "behavior pool",
    ),
    (
        &[K::ClassDefinition],
        "FOR EVENTS OF",
        Release::V758,
        "RAP event handler",
    ),
    (&[K::Convert], "UTCLONG", Release::V754, "UTCLONG"),
    (
        &[K::Type, K::Data, K::FieldSymbol, K::MethodDef, K::Form],
        "TYPE ANY STRUCTURE",
        Release::V816,
        "TYPE ANY STRUCTURE",
    ),
    (
        &[K::DeleteDatabase],
        "ORDER BY",
        Release::V751,
        "DELETE ORDER BY",
    ),
];

/// RAP derived types, `TYPE TABLE FOR CREATE entity`: the word after FOR
/// and the release; the base forms come first.
const DERIVED: &[(&str, Release)] = &[
    ("ACTION IMPORT", Release::V753),
    ("ACTION RESULT", Release::V753),
    ("FUNCTION IMPORT", Release::V753),
    ("FUNCTION RESULT", Release::V753),
    ("FAILED LATE", Release::V754),
    ("MAPPED LATE", Release::V754),
    ("REPORTED LATE", Release::V754),
    ("KEY OF", Release::V754),
    ("READ LINK", Release::V754),
    ("DETERMINATION", Release::V754),
    ("VALIDATION", Release::V754),
    ("FEATURES KEY", Release::V754),
    ("FEATURES RESULT", Release::V754),
    ("FEATURES REQUEST", Release::V754),
    ("FAILED", Release::V754),
    ("MAPPED", Release::V754),
    ("REPORTED", Release::V754),
    ("LOCK", Release::V754),
    ("CHANGE", Release::V755),
    ("ACTION REQUEST", Release::V755),
    ("FUNCTION REQUEST", Release::V755),
    ("AUTHORIZATION", Release::V755),
    ("PERMISSIONS", Release::V755),
    ("INSTANCE", Release::V755),
    ("GLOBAL", Release::V755),
    ("EVENT", Release::V757),
    ("READ CHANGES", Release::V816),
];

impl<'a> Scan<'a> {
    fn new(file: &'a File, s: &'a Statement) -> Self {
        let t: Vec<&Token> = file.statement_tokens(s).collect();
        let w = t.iter().map(|t| t.upper()).collect();
        let mut depth = Vec::with_capacity(t.len());
        let mut d: u32 = 0;
        for token in &t {
            if matches!(token.kind, TokenKind::ParenRight | TokenKind::BracketRight) {
                d = d.saturating_sub(1);
            }
            depth.push(d);
            if matches!(token.kind, TokenKind::ParenLeft | TokenKind::BracketLeft) {
                d += 1;
            }
        }
        Scan {
            kind: s.kind,
            t,
            w,
            depth,
            needs: Vec::new(),
        }
    }

    fn need(&mut self, release: Release, construct: &'static str) {
        if !self.needs.iter().any(|n| n.construct == construct) {
            self.needs.push(Need { release, construct });
        }
    }

    fn word(&self, i: usize) -> &str {
        self.w.get(i).map(String::as_str).unwrap_or("")
    }

    fn kind_at(&self, i: usize) -> Option<TokenKind> {
        self.t.get(i).map(|t| t.kind)
    }

    /// `(` right after the previous token, without blank.
    fn tight_paren(&self, i: usize) -> bool {
        self.t
            .get(i)
            .is_some_and(|t| t.kind == TokenKind::ParenLeft && !t.ws_before)
    }

    /// Position where the words of `pattern` follow each other.
    fn find_seq(&self, pattern: &str) -> Option<usize> {
        // words with dashes are several tokens: AUTHORITY - CHECK
        let p: Vec<&str> = pattern
            .split(' ')
            .flat_map(|w| {
                let mut parts = Vec::new();
                for (k, part) in w.split('-').enumerate() {
                    if k > 0 {
                        parts.push("-");
                    }
                    parts.push(part);
                }
                parts
            })
            .collect();
        (0..self.w.len()).find(|&i| {
            p.iter()
                .enumerate()
                .all(|(k, x)| *x == "?" || self.word(i + k) == *x)
        })
    }

    /// Constructs seen in the tokens alone.
    fn tokens(&mut self) {
        for i in 0..self.t.len() {
            let t = self.t[i];
            match t.kind {
                TokenKind::StringTemplate
                | TokenKind::StringTemplateBegin
                | TokenKind::StringTemplateMiddle
                | TokenKind::StringTemplateEnd => self.need(Release::V702, "string template"),
                TokenKind::BracketLeft
                    if t.ws_after && self.kind_at(i + 1) != Some(TokenKind::BracketRight) =>
                {
                    self.need(Release::V740Sp02, "table expression");
                }
                // get_ref( )->*
                TokenKind::InstanceArrow
                    if self.word(i + 1) == "*"
                        && i > 0
                        && self.t[i - 1].kind == TokenKind::ParenRight
                        && !self.closes_constructor(i - 1) =>
                {
                    self.need(Release::V756, "dereference of a method result");
                }
                // ref->( name ), struct-( name )
                TokenKind::InstanceArrow | TokenKind::Dash if self.tight_paren(i + 1) => {
                    // ASSIGN struc-(name) and CALL METHOD oref->(name) are old
                    let old = match self.kind {
                        K::Call => true,
                        K::Assign => {
                            self.kind_at(i + 2) != Some(TokenKind::String)
                                && i > 0
                                && !matches!(self.word(i - 1), "]" | "*")
                        }
                        _ => false,
                    };
                    if !old {
                        self.need(Release::V757, "dynamic component");
                    }
                }
                _ => {}
            }
            let word = self.word(i).to_string();
            let word = word.as_str();
            match word {
                "BOOLC" if self.tight_paren(i + 1) => self.need(Release::V702, "BOOLC"),
                "XSDBOOL" if self.tight_paren(i + 1) => self.need(Release::V740Sp08, "XSDBOOL"),
                "DATA" if self.tight_paren(i + 1) => {
                    self.need(Release::V740Sp02, "inline declaration")
                }
                "FINAL" if self.tight_paren(i + 1) => self.need(Release::V757, "FINAL"),
                "FIELD"
                    if self.kind_at(i + 1) == Some(TokenKind::Dash)
                        && self.word(i + 2) == "SYMBOL"
                        && self.tight_paren(i + 3) =>
                {
                    self.need(Release::V740Sp02, "inline declaration")
                }
                "ALPHA" | "XSD" if self.word(i + 1) == "=" && self.depth_in_template(i) => {
                    self.need(Release::V740Sp02, "string template formatting")
                }
                _ => {}
            }
            // VALUE #( ), NEW zcl_x( ), CONV string( )
            if let Some((_, release)) = CONSTRUCTORS.iter().find(|(k, _)| *k == word) {
                let after_arrow = i > 0
                    && matches!(
                        self.t[i - 1].kind,
                        TokenKind::InstanceArrow | TokenKind::StaticArrow | TokenKind::Dash
                    );
                let typed = self
                    .t
                    .get(i + 1)
                    .is_some_and(|n| n.kind == TokenKind::Identifier && n.ws_before);
                if let (false, true, Some(open)) = (after_arrow, typed, self.type_then_paren(i + 1))
                {
                    let release = *release;
                    self.need(release, constructor_name(word));
                    self.constructor_parts(word, open);
                }
            }
        }
        // x += 1, x &&= y, also NEXT s += x in REDUCE
        {
            for i in 1..self.t.len() {
                let split = matches!(self.t[i].kind, TokenKind::Plus | TokenKind::Dash)
                    && self.word(i + 1) == "="
                    && !self.t[i + 1].ws_before;
                if split || matches!(self.word(i), "*=" | "/=" | "&&=") {
                    self.need(Release::V754, "calculation assignment");
                    break;
                }
            }
        }
    }

    /// The `(` right after a type name starting at `i`, like `zcl_x=>ty(`
    /// or `#(`.
    fn type_then_paren(&self, i: usize) -> Option<usize> {
        let mut k = i + 1;
        loop {
            match self.t.get(k) {
                Some(t) if t.kind == TokenKind::ParenLeft && !t.ws_before => return Some(k),
                Some(t)
                    if !t.ws_before
                        && matches!(
                            t.kind,
                            TokenKind::StaticArrow | TokenKind::Dash | TokenKind::InstanceArrow
                        )
                        || (!t.ws_before && t.text == "~") =>
                {
                    let next = self.t.get(k + 1)?;
                    if next.ws_before || next.kind != TokenKind::Identifier {
                        return None;
                    }
                    k += 2;
                }
                _ => return None,
            }
        }
    }

    /// Whether the `)` at `close` ends a constructor expression like
    /// `CAST ty( ... )`.
    fn closes_constructor(&self, close: usize) -> bool {
        let Some(open) = (0..close).rev().find(|&k| {
            self.depth[k] == self.depth[close] && self.t[k].kind == TokenKind::ParenLeft
        }) else {
            return false;
        };
        // back over the type name, zcl_x=>ty or #
        let mut k = open;
        while k > 1 && !self.t[k - 1].ws_before && self.t[k - 1].kind != TokenKind::Identifier {
            k -= 2;
        }
        k > 1 && CONSTRUCTORS.iter().any(|(c, _)| *c == self.word(k - 2))
    }

    /// Whether the token is inside the `{ }` of a string template.
    fn depth_in_template(&self, i: usize) -> bool {
        self.t[..i]
            .iter()
            .rev()
            .find(|t| {
                matches!(
                    t.kind,
                    TokenKind::StringTemplateBegin
                        | TokenKind::StringTemplateMiddle
                        | TokenKind::StringTemplateEnd
                )
            })
            .is_some_and(|t| t.kind != TokenKind::StringTemplateEnd)
    }

    /// FOR iterations and table defaults inside a constructor expression
    /// whose `(` is at `open`.
    fn constructor_parts(&mut self, keyword: &str, open: usize) {
        let inside = self.depth[open] + 1;
        let mut i = open + 1;
        while i < self.t.len() && self.depth[i] >= inside {
            if self.depth[i] == inside {
                match self.word(i) {
                    "FOR" if self.word(i + 1) == "GROUPS" => {
                        self.need(Release::V740Sp08, "FOR GROUPS")
                    }
                    "FOR" if matches!(self.word(i + 2), "IN" | "=") => {
                        self.need(Release::V740Sp05, "FOR iteration")
                    }
                    "OPTIONAL" | "DEFAULT"
                        if keyword == "VALUE"
                            && (open..i).any(|k| self.t[k].kind == TokenKind::BracketRight) =>
                    {
                        self.need(Release::V740Sp08, "table expression with default")
                    }
                    "DISCARDING" if self.word(i + 1) == "DUPLICATES" => {
                        self.need(Release::V751, "DISCARDING DUPLICATES")
                    }
                    _ => {}
                }
            }
            i += 1;
        }
    }

    /// Keyword sequences and statements that came with a release.
    fn keywords(&mut self) {
        let by_kind = match self.kind {
            K::CaseType => Some((Release::V750, "CASE TYPE OF")),
            K::WhenType => Some((Release::V750, "WHEN TYPE")),
            K::TestSeam | K::EndTestSeam => Some((Release::V750, "test seam")),
            K::TypeEnum | K::TypeEnumBegin | K::TypeEnumEnd => {
                Some((Release::V751, "enumerated type"))
            }
            K::TypeMesh | K::TypeMeshBegin | K::TypeMeshEnd => Some((Release::V740Sp05, "mesh")),
            K::With | K::WithLoop | K::EndWith => Some((Release::V751, "common table expression")),
            K::ReadEntities
            | K::ModifyEntities
            | K::CommitEntities
            | K::RollbackEntities
            | K::GetPermissions
            | K::SetLocks => Some((Release::V754, "EML")),
            K::Retry => Some((Release::V702, "RETRY")),
            _ => None,
        };
        if let Some((release, construct)) = by_kind {
            self.need(release, construct);
        }
        for (kinds, pattern, release, construct) in KEYWORDS {
            if kinds.contains(&self.kind) && self.find_seq(pattern).is_some() {
                self.need(*release, construct);
            }
        }
        if self.find_seq("IS INSTANCE OF").is_some()
            || self.find_seq("IS NOT INSTANCE OF").is_some()
        {
            self.need(Release::V750, "IS INSTANCE OF");
        }
        if matches!(self.kind, K::Data | K::Type | K::Static | K::ClassData)
            && self.find_seq("BOXED").is_some()
        {
            self.need(Release::V702, "BOXED");
        }
        if self.kind == K::Raise {
            // RAISE EXCEPTION with an object built in place, or MESSAGE
            if self.word(0) == "RAISE" && self.word(1) == "EXCEPTION" {
                let i = if self.word(2) == "RESUMABLE" { 3 } else { 2 };
                if self.word(i) != "TYPE"
                    && (CONSTRUCTORS.iter().any(|(k, _)| *k == self.word(i))
                        || self.tight_paren(i + 1))
                {
                    self.need(Release::V752, "RAISE EXCEPTION with an expression");
                }
                if self.word(i) == "TYPE" && self.word(i + 2) == "MESSAGE" {
                    self.need(Release::V750, "RAISE EXCEPTION MESSAGE");
                }
            }
        }
        let words = self
            .t
            .iter()
            .filter(|t| t.kind != TokenKind::Punctuation)
            .count();
        if matches!(self.kind, K::Type | K::Data) && self.find_seq("WITH INDICATORS").is_some() {
            self.need(Release::V755, "WITH INDICATORS");
        }
        if self.kind == K::Return && words > 1 {
            self.need(Release::V758, "RETURN with a value");
        }
        self.derived_types();
    }

    fn derived_types(&mut self) {
        if !matches!(
            self.kind,
            K::Type
                | K::Data
                | K::Static
                | K::ClassData
                | K::FieldSymbol
                | K::MethodDef
                | K::CreateData
        ) {
            return;
        }
        for (start, base) in [
            ("TABLE FOR", Release::V753),
            ("STRUCTURE FOR", Release::V754),
            ("RESPONSE FOR", Release::V754),
            ("REQUEST FOR", Release::V755),
        ] {
            let Some(at) = self.find_seq(start) else {
                continue;
            };
            let after = at + 2;
            let mut release = base;
            for (words, r) in DERIVED {
                let matches = words
                    .split(' ')
                    .enumerate()
                    .all(|(k, x)| self.word(after + k) == x);
                if matches {
                    release = release.max(*r);
                    break;
                }
            }
            self.need(release, "RAP derived type");
        }
    }

    /// Strict mode and the later additions of ABAP SQL.
    fn sql(&mut self) {
        let select = matches!(
            self.kind,
            K::Select | K::SelectLoop | K::With | K::WithLoop | K::OpenCursor
        );
        // the clause the token is in, by the last keyword at depth 0
        let mut clause = String::new();
        for i in 0..self.t.len() {
            let t = self.t[i];
            if t.kind == TokenKind::At {
                self.need(Release::V740Sp05, "host variable with @");
            }
            if t.kind == TokenKind::AssociationName {
                self.need(Release::V740Sp05, "path expression");
                if matches!(self.word(i + 1), "(" | "[") {
                    self.need(Release::V751, "path expression with parameters or filter");
                }
            }
            let word = self.word(i).to_string();
            let word = word.as_str();
            if self.depth[i] == 0
                && matches!(
                    word,
                    "SELECT"
                        | "FROM"
                        | "FIELDS"
                        | "WHERE"
                        | "INTO"
                        | "APPENDING"
                        | "ORDER"
                        | "GROUP"
                        | "HAVING"
                        | "UP"
                        | "OFFSET"
                )
            {
                clause = word.to_string();
            }
            if self.tight_paren(i + 1) || self.word(i + 1) == "(" {
                if let Some((_, release)) = SQL_FUNCTIONS.iter().find(|(f, _)| *f == word) {
                    self.need(*release, "SQL function");
                }
            }
            match word {
                // a comma-separated list of columns
                "," if self.depth[i] == 0 && i + 1 < self.t.len() => {
                    self.need(Release::V740Sp05, "comma-separated list")
                }
                "AVG" if self.word(i + 1) == "(" => {
                    let close = (i + 2..self.t.len())
                        .find(|&k| self.depth[k] == self.depth[i] && self.word(k) == ")");
                    if close.is_some_and(|c| {
                        (i + 2..c)
                            .any(|k| self.word(k) == "AS" && self.depth[k] == self.depth[i] + 1)
                    }) {
                        self.need(Release::V751, "AVG AS type");
                    }
                }
                "CASE" if select && clause == "ORDER" => {
                    self.need(Release::V757, "ORDER BY an expression")
                }
                "CASE" => self.need(Release::V740Sp05, "SQL CASE"),
                "FIELDS"
                    if self.depth[i] == 0 && (i == 0 || self.word(i - 1) != "CORRESPONDING") =>
                {
                    self.need(Release::V750, "FIELDS")
                }
                "UNION" if self.starts_select(i + 1) => self.need(Release::V750, "UNION"),
                "INTERSECT" | "EXCEPT" if self.starts_select(i + 1) => {
                    self.need(Release::V756, "INTERSECT or EXCEPT")
                }
                "CROSS" if self.word(i + 1) == "JOIN" => self.need(Release::V750, "CROSS JOIN"),
                "CAST" if self.tight_paren(i + 1) || self.word(i + 1) == "(" => {
                    self.need(Release::V750, "SQL CAST")
                }
                "HIERARCHY"
                | "HIERARCHY_DESCENDANTS"
                | "HIERARCHY_ANCESTORS"
                | "HIERARCHY_SIBLINGS"
                | "HIERARCHY_DESCENDANTS_AGGREGATE"
                | "HIERARCHY_ANCESTORS_AGGREGATE"
                    if self.word(i + 1) == "(" =>
                {
                    self.need(Release::V750, "SQL hierarchy")
                }
                "OFFSET" => self.need(Release::V751, "OFFSET"),
                "OVER" if self.word(i + 1) == "(" => self.need(Release::V757, "window function"),
                "PRIVILEGED" if self.word(i + 1) == "ACCESS" => {
                    if i > 0 && self.word(i - 1) == "WITH" {
                        self.need(Release::V752, "WITH PRIVILEGED ACCESS")
                    } else {
                        self.need(Release::V758, "PRIVILEGED ACCESS")
                    }
                }
                "OPTIONS" if self.depth[i] == 0 => self.need(Release::V816, "SQL OPTIONS"),
                "IN" if self.word(i + 1) == "("
                    && self.t.get(i + 1).is_some_and(|t| t.ws_after)
                    && self.word(i + 2) != "SELECT" =>
                {
                    self.need(Release::V740Sp02, "IN list with blanks")
                }
                "INDICATORS" if self.word(i + 1) == "SET" => {
                    self.need(Release::V755, "INDICATORS SET STRUCTURE")
                }
                "INDICATORS" => self.need(Release::V816, "INDICATORS"),
                "INTO"
                    if self.word(i + 1) == "(" && self.t.get(i + 1).is_some_and(|t| t.ws_after) =>
                {
                    self.need(Release::V740Sp02, "INTO list with blanks")
                }
                "NULLS" if matches!(self.word(i + 1), "FIRST" | "LAST") => {
                    self.need(Release::V755, "NULLS FIRST or LAST")
                }
                "USING" if self.word(i + 1) == "ALL" && self.word(i + 2) == "CLIENTS" => {
                    self.need(Release::V754, "USING ALL CLIENTS")
                }
                "USING" if self.word(i + 1) == "CLIENTS" && self.word(i + 2) == "IN" => {
                    self.need(Release::V754, "USING CLIENTS IN")
                }
                "USING" if self.word(i + 1) == "CLIENT" => {
                    self.need(Release::V740Sp05, "USING CLIENT")
                }
                "NEW" if self.kind_at(i + 1) == Some(TokenKind::At) => {
                    self.need(Release::V754, "INTO NEW")
                }
                "FROM" | "JOIN"
                    if self.kind_at(i + 1) == Some(TokenKind::At)
                        && (select || self.depth[i] > 0)
                        && (i == 0 || !matches!(self.word(i - 1), "VALID" | "PERIOD")) =>
                {
                    self.need(Release::V752, "SELECT FROM an internal table")
                }
                "IS" if self.word(i + 1) == "INITIAL"
                    || (self.word(i + 1) == "NOT" && self.word(i + 2) == "INITIAL") =>
                {
                    self.need(Release::V753, "IS INITIAL in SQL")
                }
                "LIKE_REGEXPR" | "OCCURRENCES_REGEXPR" | "REPLACE_REGEXPR" => {
                    self.need(Release::V757, "SQL regular expression")
                }
                "LOCATE_REGEXPR" | "SUBSTRING_REGEXPR" => {
                    self.need(Release::V758, "SQL regular expression")
                }
                "JOIN" if self.word(i + 1).starts_with('#') => self.need(Release::V816, "join tag"),
                "+" | "-" | "*" | "/"
                    if select
                        && matches!(clause.as_str(), "SELECT" | "FIELDS" | "GROUP" | "HAVING")
                        && self.is_operator(i) =>
                {
                    if self.inside_aggregate(i) {
                        self.need(Release::V740Sp08, "SQL aggregate of an expression");
                    } else if AGGREGATES.contains(&self.word(i + 1)) {
                        self.need(Release::V754, "SQL expression with an aggregate");
                    } else {
                        self.need(Release::V740Sp05, "SQL arithmetic");
                    }
                }
                "+" | "-" | "*" | "/" if select && clause == "WHERE" && self.is_operator(i) => {
                    self.need(Release::V750, "SQL arithmetic in a condition")
                }
                "+" | "-" | "*" | "/" | "("
                    if select && clause == "ORDER" && self.is_operator_or_call(i) =>
                {
                    self.need(Release::V757, "ORDER BY an expression")
                }
                _ => {}
            }
        }
    }

    /// An arithmetic operator between two operands, not a sign or a dash
    /// of a component.
    fn is_operator(&self, i: usize) -> bool {
        let t = self.t[i];
        t.ws_before
            && t.ws_after
            && i > 0
            && !matches!(
                self.word(i - 1),
                "(" | "," | "=" | "SELECT" | "FIELDS" | "SINGLE" | "UPDATE" | "DISTINCT" | "~"
            )
            && !matches!(
                self.word(i + 1),
                "FROM" | "INTO" | "," | ")" | "APPENDING" | "WHERE"
            )
    }

    fn is_operator_or_call(&self, i: usize) -> bool {
        match self.word(i) {
            "(" => !self.t[i].ws_before,
            "CASE" => true,
            _ => self.is_operator(i),
        }
    }

    /// Inside the parentheses of SUM, MIN, MAX, AVG or COUNT.
    fn inside_aggregate(&self, i: usize) -> bool {
        let d = self.depth[i];
        if d == 0 {
            return false;
        }
        let open = (0..i)
            .rev()
            .find(|&k| self.depth[k] < d && self.word(k) == "(");
        open.is_some_and(|k| k > 0 && AGGREGATES.contains(&self.word(k - 1)))
    }

    /// Constructs seen in the expression tree.
    fn operands(&mut self, file: &File, s: &Statement) {
        let ops = operands(file, s);
        let place = self.restricted_place();
        for item in &ops.items {
            match item {
                Operand::Cond(c) => self.cond(c),
                Operand::Expr { expr, access } => {
                    self.expr(expr);
                    if *access == Access::Read {
                        // READ TABLE ... WITH KEY comp = value
                        let key_value = self.kind == K::ReadTable
                            && expr.first > 0
                            && self.word(expr.first - 1) == "=";
                        self.operand_position(expr, Some(expr.first) == place, key_value);
                    }
                }
            }
        }
    }

    fn cond(&mut self, c: &Cond) {
        match &c.kind {
            CondKind::And(cs) | CondKind::Or(cs) => cs.iter().for_each(|c| self.cond(c)),
            CondKind::Equiv(cs) => {
                self.need(Release::V702, "EQUIV");
                cs.iter().for_each(|c| self.cond(c));
            }
            CondKind::Not(c) | CondKind::Paren(c) => self.cond(c),
            CondKind::Predicate(e) => {
                if self.predicate_function(e) {
                    self.need(Release::V702, "predicate function");
                } else if is_call(e) {
                    self.need(Release::V740Sp08, "predicate method call");
                }
                self.expr(e);
            }
            _ => {}
        }
    }

    /// `contains( )`, `matches( )`, `line_exists( )`: built-in predicate
    /// functions, older than predicate method calls.
    fn predicate_function(&self, e: &Expr) -> bool {
        let ExprKind::Chain(chain) = &e.kind else {
            return false;
        };
        let Head::Call { name, .. } = &chain.head else {
            return false;
        };
        chain.links.is_empty()
            && matches!(
                self.word(*name),
                "CONTAINS" | "CONTAINS_ANY_OF" | "CONTAINS_ANY_NOT_OF" | "MATCHES" | "LINE_EXISTS"
            )
    }

    /// A SELECT or a parenthesized query starts at `i`, as after UNION.
    fn starts_select(&self, i: usize) -> bool {
        matches!(self.word(i), "SELECT" | "(" | "DISTINCT" | "ALL")
    }

    fn expr(&mut self, e: &Expr) {
        match &e.kind {
            ExprKind::Template(parts) => parts.iter().for_each(|p| self.expr(p)),
            ExprKind::Chain(chain) => self.chain(chain),
            ExprKind::Constructor(c) => self.items(&c.items),
            ExprKind::Binary { left, right, .. } => {
                self.expr(left);
                self.expr(right);
            }
            ExprKind::Unary { operand, .. } => self.expr(operand),
            ExprKind::Paren(inner) => self.expr(inner),
            _ => {}
        }
    }

    fn chain(&mut self, chain: &Chain) {
        if let Head::Call { args, .. } = &chain.head {
            for arg in &args.items {
                match &arg.value {
                    Value::Expr(e) => self.expr(e),
                    Value::Cond(c) => self.cond(c),
                }
            }
        }
        for link in &chain.links {
            match link {
                Link::Method { args, .. } => {
                    for arg in &args.items {
                        match &arg.value {
                            Value::Expr(e) => self.expr(e),
                            Value::Cond(c) => self.cond(c),
                        }
                    }
                }
                Link::Table(items) => self.items(items),
                _ => {}
            }
        }
    }

    fn items(&mut self, items: &[Item]) {
        for item in items {
            match item {
                Item::Value(e) => self.expr(e),
                Item::Define { value, .. } => {
                    if let Some(v) = value {
                        self.expr(v);
                    }
                }
                Item::Named { value, .. } => match value {
                    Value::Expr(e) => self.expr(e),
                    Value::Cond(c) => self.cond(c),
                },
                Item::Cond(c) => self.cond(c),
                Item::Row(items) => self.items(items),
            }
        }
    }

    /// Where the operand starts whose form 7.40 SP02 widened: the table of
    /// LOOP and READ TABLE, the line of APPEND and INSERT, the source of
    /// ASSIGN, the message of MESSAGE.
    fn restricted_place(&self) -> Option<usize> {
        let after = |word: &str| {
            (0..self.w.len())
                .find(|&i| self.word(i) == word)
                .map(|i| i + 1)
        };
        match self.kind {
            K::Loop => after("AT"),
            K::ReadTable => after("TABLE"),
            K::Append | K::InsertInternal => match self.word(1) {
                "LINES" => after("OF"),
                "INITIAL" => None,
                _ => Some(1),
            },
            K::Assign => match self.word(1) {
                "COMPONENT" => Some(2),
                _ => Some(1),
            },
            K::Message => Some(1),
            _ => None,
        }
    }

    /// Statements whose operands were data objects, constants and method
    /// calls only before 7.40 SP02: an operand calculated in place needs it.
    /// `first`: the operand at [`Self::restricted_place`].
    fn operand_position(&mut self, e: &Expr, first: bool, key_value: bool) {
        // what the operand could be before: method calls, string templates
        let (calls, templates) = match self.kind {
            K::Loop | K::ReadTable if first => (false, true),
            K::ReadTable if key_value => (true, false),
            K::Append | K::InsertInternal if first => (true, false),
            K::InsertFieldGroup => (true, true),
            K::Assign if first => (false, false),
            K::Message | K::CallFunction => (false, false),
            _ => return,
        };
        let calculated = matches!(
            e.kind,
            ExprKind::Binary { .. } | ExprKind::Unary { .. } | ExprKind::Paren(_)
        );
        let call = !calls && is_call(e);
        let template = !templates
            && (matches!(e.kind, ExprKind::Template(_))
                || (e.kind == ExprKind::Literal
                    && self
                        .t
                        .get(e.first)
                        .is_some_and(|t| t.kind == TokenKind::StringTemplate)));
        if calculated || call || template {
            self.need(Release::V740Sp02, "expression in an operand position");
        }
    }
}

/// SQL functions and the release that brought each, as abaplint has them.
const SQL_FUNCTIONS: &[(&str, Release)] = &[
    ("ABS", Release::V740Sp05),
    ("CEIL", Release::V740Sp05),
    ("FLOOR", Release::V740Sp05),
    ("DIV", Release::V740Sp05),
    ("MOD", Release::V740Sp05),
    ("COALESCE", Release::V740Sp05),
    ("CONCAT", Release::V750),
    ("LENGTH", Release::V750),
    ("LTRIM", Release::V750),
    ("RTRIM", Release::V750),
    ("RIGHT", Release::V750),
    ("REPLACE", Release::V750),
    ("ROUND", Release::V750),
    ("SUBSTRING", Release::V750),
    ("LPAD", Release::V750),
    ("INSTR", Release::V750),
    ("CONCAT_WITH_SPACE", Release::V751),
    ("RPAD", Release::V751),
    ("LEFT", Release::V751),
    ("LOWER", Release::V751),
    ("UPPER", Release::V751),
    ("DIVISION", Release::V751),
    ("BINTOHEX", Release::V752),
    ("HEXTOBIN", Release::V752),
    ("DATS_IS_VALID", Release::V754),
    ("DATS_DAYS_BETWEEN", Release::V754),
    ("DATS_ADD_DAYS", Release::V754),
    ("DATS_ADD_MONTHS", Release::V754),
    ("UUID", Release::V754),
    ("UTCL_CURRENT", Release::V754),
    ("TSTMP_CURRENT_UTCTIMESTAMP", Release::V757),
    ("TSTMP_IS_VALID", Release::V757),
    ("TIMS_IS_VALID", Release::V757),
    ("UTCL_ADD_SECONDS", Release::V757),
    ("UTCL_SECONDS_BETWEEN", Release::V757),
    ("DATN_DAYS_BETWEEN", Release::V757),
    ("DATN_ADD_DAYS", Release::V757),
    ("DATN_ADD_MONTHS", Release::V757),
    ("AS_GEO_JSON", Release::V757),
    ("ALLOW_PRECISION_LOSS", Release::V757),
    ("TO_CLOB", Release::V757),
    ("TO_BLOB", Release::V757),
    ("GROUPING", Release::V757),
    ("ABAP_USER_TIMEZONE", Release::V757),
    ("ABAP_SYSTEM_TIMEZONE", Release::V757),
    ("DATS_TIMS_TO_TSTMP", Release::V757),
    ("TSTMP_ADD_SECONDS", Release::V757),
    ("TSTMP_SECONDS_BETWEEN", Release::V757),
    ("TSTMP_TO_DATS", Release::V757),
    ("TSTMP_TO_TIMS", Release::V757),
    ("TSTMP_TO_DST", Release::V757),
    ("TSTMPL_TO_UTCL", Release::V757),
    ("TSTMPL_FROM_UTCL", Release::V757),
    ("DATS_TO_DATN", Release::V757),
    ("DATS_FROM_DATN", Release::V757),
    ("TIMS_TO_TIMN", Release::V757),
    ("TIMS_FROM_TIMN", Release::V757),
    ("CURRENCY_CONVERSION", Release::V757),
    ("EXTRACT_YEAR", Release::V758),
    ("EXTRACT_MONTH", Release::V758),
    ("EXTRACT_DAY", Release::V758),
    ("EXTRACT_HOUR", Release::V758),
    ("EXTRACT_MINUTE", Release::V758),
    ("EXTRACT_SECOND", Release::V758),
    ("DAYS_BETWEEN", Release::V758),
    ("ADD_DAYS", Release::V758),
    ("ADD_MONTHS", Release::V758),
    ("MONTHNAME", Release::V758),
    ("DAYNAME", Release::V758),
    ("INITCAP", Release::V758),
    ("IS_VALID", Release::V758),
    ("WEEKDAY", Release::V758),
    ("LOCATE", Release::V758),
    ("GREATEST", Release::V758),
    ("LEAST", Release::V758),
    ("HIERARCHY_COMPOSITE_ID", Release::V758),
    ("SIN", Release::V758),
    ("COS", Release::V758),
    ("TAN", Release::V758),
    ("ASIN", Release::V758),
    ("ACOS", Release::V758),
    ("ATAN", Release::V758),
    ("SINH", Release::V758),
    ("COSH", Release::V758),
    ("TANH", Release::V758),
    ("SQRT", Release::V758),
    ("POWER", Release::V758),
    ("EXP", Release::V758),
    ("LN", Release::V758),
    ("LOG10", Release::V758),
    ("LOG", Release::V758),
    ("UTCTOLOCAL", Release::V758),
    ("LOCALTOUTC", Release::V758),
    ("IS_MASKED", Release::V758),
    ("UNIT_CONVERSION", Release::V758),
    ("CURR_TO_DECFLOAT_AMOUNT", Release::V758),
    ("LANG_TO_CHAR", Release::V758),
    ("IPOW", Release::V758),
];

const AGGREGATES: [&str; 5] = ["SUM", "MIN", "MAX", "AVG", "COUNT"];

fn is_call(e: &Expr) -> bool {
    let ExprKind::Chain(chain) = &e.kind else {
        return false;
    };
    matches!(chain.head, Head::Call { .. })
        || chain.links.iter().any(|l| matches!(l, Link::Method { .. }))
}

fn constructor_name(keyword: &str) -> &'static str {
    match keyword {
        "NEW" => "NEW",
        "VALUE" => "VALUE",
        "CONV" => "CONV",
        "CAST" => "CAST",
        "REF" => "REF",
        "EXACT" => "EXACT",
        "COND" => "COND",
        "SWITCH" => "SWITCH",
        "CORRESPONDING" => "CORRESPONDING",
        "REDUCE" => "REDUCE",
        _ => "FILTER",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn need(abap: &str) -> Option<(Release, &'static str)> {
        let file = crate::parse(abap);
        let s = file
            .statements
            .iter()
            .find(|s| s.kind != K::Comment)
            .unwrap();
        statement_release(&file, s).map(|n| (n.release, n.construct))
    }

    #[test]
    fn constructs_and_their_releases() {
        assert_eq!(need("x = y."), None);
        assert_eq!(
            need("x = |a{ y }b|."),
            Some((Release::V702, "string template"))
        );
        assert_eq!(
            need("DATA(x) = 1."),
            Some((Release::V740Sp02, "inline declaration"))
        );
        assert_eq!(
            need("x = VALUE #( a = 1 )."),
            Some((Release::V740Sp02, "VALUE"))
        );
        assert_eq!(
            need("x = itab[ 1 ]."),
            Some((Release::V740Sp02, "table expression"))
        );
        assert_eq!(need("x = itab[]."), None);
        assert_eq!(
            need("x = REDUCE i( INIT s = 0 FOR wa IN itab NEXT s = s + 1 )."),
            Some((Release::V740Sp08, "REDUCE"))
        );
        assert_eq!(
            need("SELECT a, b FROM t INTO TABLE @lt."),
            Some((Release::V740Sp05, "comma-separated list"))
        );
        assert_eq!(need("SELECT a b FROM t INTO TABLE lt."), None);
        assert_eq!(
            need("x += 1."),
            Some((Release::V754, "calculation assignment"))
        );
        assert_eq!(
            need("TYPES BEGIN OF ENUM e."),
            Some((Release::V751, "enumerated type"))
        );
        assert_eq!(need("FINAL(x) = 1."), Some((Release::V757, "FINAL")));
        assert_eq!(
            need("IF foo( ). ENDIF."),
            Some((Release::V740Sp08, "predicate method call"))
        );
        assert_eq!(
            need("APPEND a + 1 TO itab."),
            Some((Release::V740Sp02, "expression in an operand position"))
        );
        assert_eq!(need("APPEND foo( ) TO itab."), None);
        assert_eq!(need("DATA x TYPE i VALUE 1."), None);
        assert_eq!(need("METHODS m RETURNING VALUE(r) TYPE i."), None);
    }

    #[test]
    fn release_names() {
        assert_eq!("750".parse(), Ok(Release::V750));
        assert_eq!("7.50".parse(), Ok(Release::V750));
        assert_eq!("7.40 SP05".parse(), Ok(Release::V740Sp05));
        assert_eq!("v816".parse(), Ok(Release::V816));
        assert!("749".parse::<Release>().is_err());
        assert_eq!(Release::of_abaplint("v776"), Some(Release::V754));
        assert_eq!(Release::of_abaplint("v916"), Some(Release::V816));
        assert_eq!(Release::of_abaplint("v917"), None);
        assert_eq!(Release::V740Sp08.to_string(), "7.40 SP08");
    }
}
