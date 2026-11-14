//! CDS data definitions (DDL sources): tokens and syntax errors.
//!
//! The lexer follows abaplint's CDS lexer (<https://github.com/abaplint/abaplint>,
//! MIT): every token is an identifier, apart from comments; strings in
//! single and double quotes are one token each, `@` starts a token, and
//! `; : , . { } ( ) [ ] ! = < > + - * /` are tokens of their own. Unlike
//! abaplint, a name with a namespace written without blanks, `/DMO/I_Travel`,
//! is one token: SAP rejects blanks inside it. Whether
//! the tokens form a data definition is decided by the grammar of CDS in
//! `grammar/cds.rs`, generated from abaplint's CDS parser.

use crate::release::{Need, Release};
use crate::{Position, Token, TokenKind};

/// A syntax error of a data definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CdsSyntaxError {
    /// where the grammar got stuck: the token, or the end of the source
    pub start: Position,
    /// byte offset of `start`
    pub offset: usize,
    /// the token there, empty at the end
    pub text: String,
    pub message: &'static str,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    Default,
    String,
    DoubleQuoteString,
    LineComment,
    BlockComment,
}

struct Lexer<'a> {
    src: &'a str,
    out: Vec<Token>,
    /// start of the token being built, byte offset and position
    start: Option<(usize, Position)>,
    /// whitespace or a comment since the last token
    ws: bool,
}

impl Lexer<'_> {
    /// Ends the token being built at `end`.
    fn flush(&mut self, end: usize, kind: TokenKind) {
        if let Some((s, pos)) = self.start.take() {
            if s < end {
                if let Some(last) = self.out.last_mut() {
                    last.ws_after = self.ws;
                }
                self.out.push(Token {
                    kind,
                    text: self.src[s..end].to_string(),
                    start: pos,
                    span: s..end,
                    ws_before: self.ws,
                    ws_after: false,
                    expanded: false,
                });
                self.ws = false;
            }
        }
    }

    fn begin(&mut self, at: usize, pos: Position) {
        if self.start.is_none() {
            self.start = Some((at, pos));
        }
    }
}

/// The length of `/ns/name` at the start of `s`: a slash, letters, digits
/// or underscores, a slash, and at least one more of them.
fn namespace_len(s: &str) -> Option<usize> {
    let b = s.as_bytes();
    let word = |c: u8| c.is_ascii_alphanumeric() || c == b'_';
    let ns = b[1..].iter().take_while(|&&c| word(c)).count();
    if ns == 0 || b.get(1 + ns) != Some(&b'/') {
        return None;
    }
    let name = b[2 + ns..].iter().take_while(|&&c| word(c)).count();
    (name > 0).then_some(2 + ns + name)
}

/// The tokens of a data definition, comments included.
pub fn tokens(source: &str) -> Vec<Token> {
    let mut lx = Lexer {
        src: source,
        out: Vec::new(),
        start: None,
        ws: false,
    };
    let mut mode = Mode::Default;
    let (mut row, mut col) = (1u32, 1u32);
    let mut chars = source.char_indices().peekable();
    while let Some((i, c)) = chars.next() {
        let next = chars.peek().map(|&(_, n)| n);
        let pos = Position { row, col };
        let after = i + c.len_utf8();
        if c == '\n' {
            row += 1;
            col = 1;
        } else {
            col += 1;
        }
        match mode {
            Mode::String | Mode::DoubleQuoteString => {
                let quote = if mode == Mode::String { '\'' } else { '"' };
                // doubled quote, or an escape with a backslash
                let escaped = (c == quote && next == Some(quote))
                    || (c == '\\'
                        && (next == Some(quote) || (quote == '\'' && next == Some('\\'))));
                if escaped {
                    chars.next();
                    col += 1;
                } else if c == quote {
                    lx.flush(after, TokenKind::String);
                    mode = Mode::Default;
                }
                continue;
            }
            Mode::LineComment => {
                if c == '\n' {
                    lx.flush(i, TokenKind::Comment);
                    lx.ws = true;
                    mode = Mode::Default;
                }
                continue;
            }
            Mode::BlockComment => {
                if c == '*' && next == Some('/') {
                    chars.next();
                    col += 1;
                    lx.ws = true;
                    mode = Mode::Default;
                }
                continue;
            }
            Mode::Default => {}
        }
        match c {
            '/' if lx.start.is_none() && namespace_len(&source[i..]).is_some() => {
                let len = namespace_len(&source[i..]).unwrap_or(1);
                lx.begin(i, pos);
                while chars.peek().is_some_and(|&(j, _)| j < i + len) {
                    chars.next();
                    col += 1;
                }
                lx.flush(i + len, TokenKind::Identifier);
            }
            '/' | '-' if next == Some(c) => {
                lx.flush(i, TokenKind::Identifier);
                lx.begin(i, pos);
                mode = Mode::LineComment;
            }
            '/' if next == Some('*') => {
                lx.flush(i, TokenKind::Identifier);
                chars.next();
                col += 1;
                mode = Mode::BlockComment;
            }
            '\'' | '"' => {
                lx.flush(i, TokenKind::Identifier);
                lx.begin(i, pos);
                mode = if c == '\'' {
                    Mode::String
                } else {
                    Mode::DoubleQuoteString
                };
            }
            // like abaplint: blanks, tabs, no-break and vertical spaces
            ' ' | '\t' | '\r' | '\n' | '\u{a0}' | '\u{b}' => {
                lx.flush(i, TokenKind::Identifier);
                lx.ws = true;
            }
            ';' | ':' | ',' | '.' | '{' | '}' | '(' | ')' | '[' | ']' | '!' | '=' | '<' | '>'
            | '+' | '-' | '*' | '/' => {
                lx.flush(i, TokenKind::Identifier);
                lx.begin(i, pos);
                lx.flush(after, TokenKind::Identifier);
            }
            '@' => {
                lx.flush(i, TokenKind::Identifier);
                lx.begin(i, pos);
            }
            _ => lx.begin(i, pos),
        }
    }
    let kind = match mode {
        Mode::LineComment => TokenKind::Comment,
        Mode::String | Mode::DoubleQuoteString => TokenKind::String,
        _ => TokenKind::Identifier,
    };
    lx.flush(source.len(), kind);
    lx.out
}

/// The syntax error of a data definition, `None` when it is one of the kinds
/// of definition: views and view entities, projection views, abstract,
/// custom, external and table entities, table functions, hierarchies,
/// extensions and metadata extensions (`annotate`).
pub fn syntax_error(source: &str) -> Option<CdsSyntaxError> {
    let all = tokens(source);
    let code: Vec<&Token> = all
        .iter()
        .filter(|t| t.kind != TokenKind::Comment)
        .collect();
    let (far, message) = match crate::grammar::cds_mismatch(&code) {
        Some(far) if code.is_empty() => (far, "no data definition"),
        Some(far) => (far, "does not match the CDS grammar"),
        None => rules(&code)?,
    };
    let at = code.get(far);
    let end = || {
        let row = source.lines().count().max(1) as u32;
        let col = source.lines().last().map_or(0, |l| l.chars().count()) as u32 + 1;
        Position { row, col }
    };
    Some(CdsSyntaxError {
        start: at.map_or_else(end, |t| t.start),
        offset: at.map_or(source.len(), |t| t.span.start),
        text: at.map(|t| t.text.clone()).unwrap_or_default(),
        message,
    })
}

/// The kind of view a data definition is: SAP's syntax rules differ.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ViewKind {
    /// `DEFINE [ROOT] VIEW name`, a DDIC-based view (obsolete since 7.57)
    Ddic,
    /// `DEFINE [ROOT] [TRANSIENT|WRITABLE] VIEW ENTITY name`
    Entity,
    Other,
}

/// The session variables of SAP's documentation
/// (`abencds_session_variable_v2`).
const SESSION_VARIABLES: [&str; 8] = [
    "USER",
    "CLIENT",
    "SYSTEM_LANGUAGE",
    "SYSTEM_DATE",
    "USER_TIMEZONE",
    "USER_DATE",
    "BS_INSTANCE_ID",
    "BS_ZONE_ID",
];

/// Words after a data source that are no alias.
const AFTER_SOURCE: [&str; 17] = [
    "BIND",
    "AS",
    "ON",
    "INNER",
    "LEFT",
    "RIGHT",
    "CROSS",
    "JOIN",
    "WHERE",
    "GROUP",
    "HAVING",
    "UNION",
    "EXCEPT",
    "INTERSECT",
    "ASSOCIATION",
    "COMPOSITION",
    "WITH",
];

/// The tokens of annotations, whose values may hold any word.
fn annotation_mask(t: &[&Token]) -> Vec<bool> {
    let is = |i: usize, w: &str| t.get(i).is_some_and(|x| x.text.eq_ignore_ascii_case(w));
    let mut annotation = vec![false; t.len()];
    let mut i = 0;
    while i < t.len() {
        if !t[i].text.starts_with('@') {
            i += 1;
            continue;
        }
        let start = i;
        i += 1;
        while is(i, "<") || (is(i, ".") && t.get(i + 1).is_some()) {
            i += if is(i, "<") { 1 } else { 2 };
        }
        if is(i, ":") {
            i += 1;
            // a value: one token, `#( ... )`, `{ ... }`, `[ ... ]`, `-1.5`
            let mut depth = 0i32;
            while let Some(x) = t.get(i) {
                match x.text.as_str() {
                    "{" | "[" | "(" => depth += 1,
                    "}" | "]" | ")" => depth -= 1,
                    _ => {}
                }
                i += 1;
                let open = matches!(x.text.as_str(), "#" | "-" | "+" | ".") || is(i, ".");
                if depth <= 0 && !open {
                    break;
                }
            }
        }
        annotation[start..i.min(t.len())]
            .iter_mut()
            .for_each(|x| *x = true);
    }
    annotation
}

/// The DEFINE of the data definition and the kind of view it defines:
/// `DEFINE [ROOT] [TRANSIENT|WRITABLE] VIEW [ENTITY]`.
fn view_kind(t: &[&Token], annotation: &[bool]) -> (Option<usize>, ViewKind) {
    let is = |i: usize, w: &str| t.get(i).is_some_and(|x| x.text.eq_ignore_ascii_case(w));
    let define = (0..t.len()).find(|&i| !annotation[i] && is(i, "DEFINE"));
    let kind = define.map_or(ViewKind::Other, |d| {
        let mut k = d + 1;
        while ["ROOT", "TRANSIENT", "WRITABLE"].iter().any(|w| is(k, w)) {
            k += 1;
        }
        match (is(k, "VIEW"), is(k + 1, "ENTITY")) {
            (true, true) => ViewKind::Entity,
            (true, false) => ViewKind::Ddic,
            _ => ViewKind::Other,
        }
    });
    (define, kind)
}

/// The rules the grammar does not hold, as SAP's documentation states them
/// and the syntax check of the SAP ABAP Cloud Developer Trial 2025 reports
/// them: the position of the first violation and its message. Annotations
/// are not checked beyond their syntax: SAP evaluates them by their
/// definitions, like `@AbapCatalog.sqlViewName`, required in DDIC-based
/// views and not allowed in view entities.
fn rules(t: &[&Token]) -> Option<(usize, &'static str)> {
    let word = |i: usize| {
        t.get(i)
            .map(|x| x.text.to_ascii_uppercase())
            .unwrap_or_default()
    };
    let is = |i: usize, w: &str| t.get(i).is_some_and(|x| x.text.eq_ignore_ascii_case(w));
    let annotation = annotation_mask(t);
    let (_, kind) = view_kind(t, &annotation);
    for i in 0..t.len() {
        let text = t[i].text.as_str();
        if annotation[i] {
            // SAP: "Post-annotations are not supported" in view entities
            if kind == ViewKind::Entity && text == "@" && is(i + 1, "<") {
                return Some((i, "annotation after an element in a view entity"));
            }
            continue;
        }
        let upper = text.to_ascii_uppercase();
        // keywords all upper, all lower or capitalized
        // (`abencds_general_syntax_rules`); reserved words are keywords
        if crate::grammar::CDS_RESERVED.contains(&upper.as_str()) {
            let lower = text.to_ascii_lowercase();
            let capital = text[..1].to_ascii_uppercase() + &lower[1..];
            if text != upper && text != lower && text != capital {
                return Some((i, "keyword neither upper case, lower case nor capitalized"));
            }
        }
        if upper == "$SESSION"
            && is(i + 1, ".")
            && !SESSION_VARIABLES.contains(&word(i + 2).as_str())
        {
            return Some((i + 2, "unknown session variable"));
        }
        match kind {
            ViewKind::Entity => {
                // `:p`: SAP "Use expression "$parameters" instead of ":""
                if text == ":" && i > 0 && !is(i - 1, ":") {
                    let prev = word(i - 1);
                    let operand = [
                        "=", "<", ">", "(", ",", "AND", "OR", "NOT", "THEN", "ELSE", "WHEN",
                        "BETWEEN", "LIKE", "+", "-", "*", "/",
                    ]
                    .contains(&prev.as_str());
                    if operand
                        && t.get(i + 1).is_some_and(|x| {
                            x.text
                                .starts_with(|c: char| c.is_ascii_alphabetic() || c == '_')
                        })
                    {
                        return Some((i, "parameter with : in a view entity, $parameters instead"));
                    }
                }
                // `FROM source alias`: the keyword or character "AS" was expected
                if (upper == "FROM" || upper == "JOIN") && i + 1 < t.len() {
                    let mut k = i + 2;
                    for (open, close) in [("(", ")"), ("[", "]")] {
                        if is(k, open) {
                            let mut depth = 0;
                            while let Some(x) = t.get(k) {
                                depth += i32::from(x.text == open) - i32::from(x.text == close);
                                k += 1;
                                if depth == 0 {
                                    break;
                                }
                            }
                        }
                    }
                    let alias = t.get(k).is_some_and(|x| {
                        x.text
                            .starts_with(|c: char| c.is_ascii_alphabetic() || c == '_')
                            && !AFTER_SOURCE.contains(&x.text.to_ascii_uppercase().as_str())
                    });
                    if alias && !annotation.get(k).copied().unwrap_or(true) {
                        return Some((k, "alias without AS in a view entity"));
                    }
                }
            }
            ViewKind::Ddic => {
                // abap.char'X': "Typed literals are not allowed in DDLS type
                // V", but as the operand of CAST they are, as in SAP's views
                let cast = i > 1 && is(i - 1, "(") && is(i - 2, "CAST");
                if upper == "ABAP"
                    && !cast
                    && is(i + 1, ".")
                    && t.get(i + 3).is_some_and(|x| {
                        x.kind == TokenKind::String && x.span.start == t[i + 2].span.end
                    })
                {
                    return Some((i, "typed literal in a DDIC-based view"));
                }
            }
            ViewKind::Other => {}
        }
    }
    None
}

/// Built-in functions of CDS by the release whose CDS release news name
/// them (`abennews-740_sp08-abap_cds` to `abennews-816-abap_cds`).
const FUNCTIONS: &[(&str, Release)] = &[
    ("CONCAT", Release::V740Sp08),
    ("REPLACE", Release::V740Sp08),
    ("ABS", Release::V740Sp08),
    ("DIV", Release::V740Sp08),
    ("DIVISION", Release::V740Sp08),
    ("FLOOR", Release::V740Sp08),
    ("MOD", Release::V740Sp08),
    ("ROUND", Release::V740Sp08),
    ("COALESCE", Release::V740Sp08),
    ("UNIT_CONVERSION", Release::V740Sp08),
    ("CURRENCY_CONVERSION", Release::V740Sp08),
    ("DECIMAL_SHIFT", Release::V740Sp08),
    ("CONCAT_WITH_SPACE", Release::V750),
    ("INSTR", Release::V750),
    ("LEFT", Release::V750),
    ("LENGTH", Release::V750),
    ("LTRIM", Release::V750),
    ("RIGHT", Release::V750),
    ("RPAD", Release::V750),
    ("RTRIM", Release::V750),
    ("BINTOHEX", Release::V750),
    ("HEXTOBIN", Release::V750),
    ("DATS_DAYS_BETWEEN", Release::V750),
    ("DATS_ADD_DAYS", Release::V750),
    ("DATS_ADD_MONTHS", Release::V750),
    ("DATS_IS_VALID", Release::V750),
    ("TIMS_IS_VALID", Release::V750),
    ("TSTMP_IS_VALID", Release::V750),
    ("TSTMP_CURRENT_UTCTIMESTAMP", Release::V750),
    ("TSTMP_SECONDS_BETWEEN", Release::V750),
    ("TSTMP_ADD_SECONDS", Release::V750),
    ("UPPER", Release::V751),
    ("LOWER", Release::V751),
    ("ABAP_SYSTEM_TIMEZONE", Release::V751),
    ("ABAP_USER_TIMEZONE", Release::V751),
    ("TSTMP_TO_DATS", Release::V751),
    ("TSTMP_TO_TIMS", Release::V751),
    ("TSTMP_TO_DST", Release::V751),
    ("DATS_TIMS_TO_TSTMP", Release::V751),
    ("FLTP_TO_DEC", Release::V751),
    ("DATN_DAYS_BETWEEN", Release::V754),
    ("DATN_ADD_DAYS", Release::V754),
    ("DATN_ADD_MONTHS", Release::V754),
    ("UTCL_CURRENT", Release::V754),
    ("UTCL_ADD_SECONDS", Release::V754),
    ("UTCL_SECONDS_BETWEEN", Release::V754),
    ("TSTMPL_TO_UTCL", Release::V754),
    ("TSTMPL_FROM_UTCL", Release::V754),
    ("DATS_TO_DATN", Release::V754),
    ("DATS_FROM_DATN", Release::V754),
    ("TIMS_TO_TIMN", Release::V754),
    ("TIMS_FROM_TIMN", Release::V754),
    ("REPLACE_REGEXPR", Release::V755),
    ("GET_NUMERIC_VALUE", Release::V756),
    ("CURR_TO_DECFLOAT_AMOUNT", Release::V756),
    ("SERIES_GENERATE_DATE", Release::V756),
    ("SERIES_GENERATE_INTEGER", Release::V756),
    ("SERIES_GENERATE_TIME", Release::V756),
    ("SERIES_GENERATE_TIMESTAMP", Release::V756),
    ("LN", Release::V816),
    ("LOG", Release::V816),
    ("EXPONENTIAL", Release::V816),
    ("POWER", Release::V816),
];

/// What a data definition needs: every construct newer than CDS itself
/// (7.40 SP05), each once, with the release whose CDS release news
/// introduce it (`abennews-740_sp08-abap_cds` to `abennews-816-abap_cds`).
/// Constructs whose release depends on the kind of view, like UNION,
/// DISTINCT or typed literals in view entities, take the release that
/// brought them to that kind.
pub fn needs(source: &str) -> Vec<Need> {
    let all = tokens(source);
    let t: Vec<&Token> = all
        .iter()
        .filter(|t| t.kind != TokenKind::Comment)
        .collect();
    let annotation = annotation_mask(&t);
    let (define, kind) = view_kind(&t, &annotation);
    let is = |i: usize, w: &str| t.get(i).is_some_and(|x| x.text.eq_ignore_ascii_case(w));
    let mut out: Vec<Need> = Vec::new();
    let mut need = |release: Release, construct: &'static str| {
        if !out.iter().any(|n| n.construct == construct) {
            out.push(Need { release, construct });
        }
    };
    // the kind of definition: DEFINE [ROOT] [TRANSIENT|WRITABLE] ...
    if let Some(d) = define {
        let mut k = d + 1;
        while ["ROOT", "TRANSIENT", "WRITABLE"].iter().any(|w| is(k, w)) {
            if is(k, "ROOT") {
                need(Release::V754, "root entity");
            } else if is(k, "TRANSIENT") {
                need(Release::V757, "transient view entity");
            } else {
                need(Release::V816, "writable entity");
            }
            k += 1;
        }
        match (word(&t, k).as_str(), word(&t, k + 1).as_str()) {
            ("VIEW", "ENTITY") => need(Release::V755, "view entity"),
            ("TABLE", "FUNCTION") => need(Release::V750, "table function"),
            ("HIERARCHY", _) => need(Release::V753, "hierarchy"),
            ("ABSTRACT", _) => need(Release::V753, "abstract entity"),
            ("CUSTOM", _) => need(Release::V754, "custom entity"),
            ("TABLE", "ENTITY") => need(Release::V816, "table entity"),
            ("EXTERNAL", _) => need(Release::V816, "external entity"),
            ("ASPECT", _) => need(Release::V816, "aspect"),
            _ => {}
        }
    }
    for i in 0..t.len() {
        if annotation[i] {
            // `@<` and annotation arrays and objects: 7.40 SP08
            if t[i].text == "@" && is(i + 1, "<") {
                need(Release::V740Sp08, "annotation after an element");
            }
            if (t[i].text == "[" || t[i].text == "{") && i > 0 && is(i - 1, ":") {
                need(Release::V740Sp08, "annotation array or object");
            }
            continue;
        }
        let w = word(&t, i);
        let next = word(&t, i + 1);
        if is(i + 1, "(") {
            if let Some((_, r)) = FUNCTIONS.iter().find(|(f, _)| *f == w) {
                need(*r, "CDS function");
            }
        }
        match w.as_str() {
            "WITH" if next == "PARAMETERS" => need(Release::V740Sp08, "parameters"),
            "EXTEND" => match (next.as_str(), word(&t, i + 2).as_str()) {
                ("VIEW", "ENTITY") => need(Release::V755, "EXTEND VIEW ENTITY"),
                ("ABSTRACT", _) => need(Release::V756, "EXTEND ABSTRACT ENTITY"),
                ("CUSTOM", _) => need(Release::V757, "EXTEND CUSTOM ENTITY"),
                _ => need(Release::V740Sp08, "EXTEND VIEW"),
            },
            "/" => need(Release::V740Sp08, "division"),
            "CASE" if next == "WHEN" => need(Release::V740Sp08, "searched CASE"),
            // path attributes [INNER], [LEFT OUTER], [1: ...]
            "[" if matches!(next.as_str(), "INNER" | "LEFT") => {
                need(Release::V740Sp08, "join type of a path")
            }
            "[" if is(i + 2, ":") && next == "1" => need(Release::V740Sp08, "unique filter"),
            "[" if is(i + 2, ":") && next == "*" => need(Release::V751, "non-unique filter"),
            "$SESSION" => {
                need(Release::V750, "session variable");
                match word(&t, i + 2).as_str() {
                    "SYSTEM_DATE" => need(Release::V751, "session variable of 7.51"),
                    "USER_TIMEZONE" | "USER_DATE" => {
                        need(Release::V755, "session variable of 7.55")
                    }
                    "BS_INSTANCE_ID" | "BS_ZONE_ID" => {
                        need(Release::V757, "application session variable")
                    }
                    _ => {}
                }
            }
            "PRESERVING" if next == "TYPE" => need(Release::V751, "PRESERVING TYPE"),
            "CROSS" if next == "JOIN" => need(Release::V751, "CROSS JOIN"),
            "DEFAULT" if next == "FILTER" => need(Release::V751, "WITH DEFAULT FILTER"),
            "AVG" if is(i + 1, "(") => {
                let close = (i + 2..t.len()).find(|&k| is(k, ")"));
                if close.is_some_and(|c| (i + 2..c).any(|k| is(k, "AS"))) {
                    need(Release::V751, "AVG AS");
                }
            }
            "OUTER" if next == "TO" => need(Release::V752, "cardinality of LEFT OUTER JOIN"),
            "IS" if next == "INITIAL" || (next == "NOT" && is(i + 2, "INITIAL")) => {
                need(Release::V753, "IS INITIAL")
            }
            "PROJECTION" if i > 0 && is(i - 1, "AS") && next == "ON" => {
                need(Release::V754, "projection view")
            }
            "PERIOD" if kind == ViewKind::Other => need(Release::V754, "temporal hierarchy"),
            "LOAD" if matches!(next.as_str(), "BULK" | "INCREMENTAL") => {
                need(Release::V754, "hierarchy load option")
            }
            "COMPOSITION" => need(Release::V754, "composition"),
            "PARENT" if i > 0 && is(i - 1, "TO") => need(Release::V754, "to-parent association"),
            "UNION" | "DISTINCT" if kind == ViewKind::Entity => {
                need(Release::V756, "UNION or DISTINCT in a view entity")
            }
            "EXCEPT" | "INTERSECT" => need(Release::V756, "EXCEPT or INTERSECT"),
            "REDEFINE" if next == "ASSOCIATION" => need(Release::V756, "REDEFINE ASSOCIATION"),
            "PROVIDER" if next == "CONTRACT" => need(Release::V756, "PROVIDER CONTRACT"),
            "CACHE" if matches!(next.as_str(), "ON" | "OFF" | "FORCE") => {
                need(Release::V756, "hierarchy cache policy")
            }
            // abap.char'X'
            "ABAP"
                if is(i + 1, ".")
                    && t.get(i + 3).is_some_and(|x| {
                        x.kind == TokenKind::String && x.span.start == t[i + 2].span.end
                    }) =>
            {
                need(Release::V756, "typed literal")
            }
            "ELSE" if next == "NULL" => need(Release::V757, "ELSE NULL"),
            // {MANY | ONE | EXACT ONE} TO {MANY | ONE | EXACT ONE}
            "MANY" | "ONE" if next == "TO" => need(Release::V758, "cardinality in words"),
            "BIND" if next == "ASPECT" => need(Release::V816, "aspect"),
            _ => {}
        }
    }
    out
}

/// The release a data definition needs: the newest of [`needs`], at least
/// 7.40 SP05, which brought CDS.
pub fn release(source: &str) -> Need {
    needs(source).into_iter().fold(
        Need {
            release: Release::V740Sp05,
            construct: "CDS data definition",
        },
        |a, b| if b.release > a.release { b } else { a },
    )
}

fn word(t: &[&Token], i: usize) -> String {
    t.get(i)
        .map(|x| x.text.to_ascii_uppercase())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn releases_by_the_release_news() {
        let r = |src: &str| release(src).release;
        assert_eq!(
            r("@AbapCatalog.sqlViewName: 'ZV'\ndefine view zv as select from t { a }"),
            Release::V740Sp05
        );
        assert_eq!(
            r("define view zv with parameters p : abap.char(1) as select from t { a }"),
            Release::V740Sp08
        );
        assert_eq!(
            r("define view zv as select from t { $session.user as u }"),
            Release::V750
        );
        assert_eq!(
            r("define view zv as select from t cross join u { t.a }"),
            Release::V751
        );
        assert_eq!(
            r("define view entity zv as select from t { key a }"),
            Release::V755
        );
        assert_eq!(
            r("define view entity zv as select distinct from t { key a }"),
            Release::V756
        );
        assert_eq!(
            r("define view entity zv as select from t { key a, case a when 'X' then 'Y' else null end as b }"),
            Release::V757
        );
        assert_eq!(
            r("define view entity zv as select from t association of many to one u as _u on _u.a = t.a { key a, _u }"),
            Release::V758
        );
        assert_eq!(
            r("define table entity zt { key a : abap.char(1); }"),
            Release::V816
        );
        // annotations hold any word
        assert_eq!(
            r("@Anno.x: 'except'\n@AbapCatalog.sqlViewName: 'ZV'\ndefine view zv as select from t { a }"),
            Release::V740Sp05
        );
        assert_eq!(
            release("define view zv as select from t { upper( a ) as u }").construct,
            "CDS function"
        );
    }

    fn texts(src: &str) -> Vec<String> {
        tokens(src)
            .into_iter()
            .filter(|t| t.kind != TokenKind::Comment)
            .map(|t| t.text)
            .collect()
    }

    #[test]
    fn lexer_like_abaplint() {
        // abaplint's first CDS parser test has 18 tokens
        let src = "@AbapCatalog.sqlViewName: 'ZSDF'\ndefine view zhvamfoocust as select from zhvam_cust {\n  key foo as sdfdsf\n}";
        assert_eq!(texts(src).len(), 18);
        assert_eq!(
            texts("a.b -- c\n/* d */ 'it''s' \"x\"\"y\" // e"),
            ["a", ".", "b", "'it''s'", "\"x\"\"y\""]
        );
        assert_eq!(texts("@<Anno: #X"), ["@", "<", "Anno", ":", "#X"]);
        assert_eq!(
            texts("from /DMO/I_Travel as t"),
            ["from", "/DMO/I_Travel", "as", "t"]
        );
        assert_eq!(texts("a/b / /x/y"), ["a", "/", "b", "/", "/x/y"]);
        assert_eq!(texts("key / 1bs/f"), ["key", "/", "1bs", "/", "f"]);
        assert_eq!(
            texts("ltrim( a, '\\\\' )"),
            ["ltrim", "(", "a", ",", "'\\\\'", ")"]
        );
        let t = tokens("define\n  view");
        assert_eq!((t[1].start.row, t[1].start.col), (2, 3));
        assert!(t[1].ws_before && t[0].ws_after);
    }

    #[test]
    fn data_definitions() {
        assert_eq!(
            syntax_error(
                "@AbapCatalog.sqlViewName: 'ZSDF'\ndefine view zfoo as select from ztab {\n  key foo as bar,\n  baz\n}"
            ),
            None
        );
        assert_eq!(
            syntax_error("define view entity ZI_X as select from ztab { key a, b }"),
            None
        );
        let e = syntax_error("define view entity ZI_X as select from ztab { key a b }").unwrap();
        assert_eq!((e.text.as_str(), e.start.row), ("b", 1));
        assert!(syntax_error("").is_some());
    }
}
