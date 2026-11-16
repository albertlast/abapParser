//! Syntax errors in SQLScript, the language of AMDP methods on SAP HANA.
//!
//! The body of an AMDP method `BY DATABASE PROCEDURE|FUNCTION FOR HDB
//! LANGUAGE SQLSCRIPT` is lexed here and matched against the grammar in
//! `grammar/sqlscript.rs`, written from SAP's SQLScript and SQL references
//! for SAP HANA 2.0 SPS 08 and held against the syntax check of the SAP
//! trial 2025, which hands the body to its HANA 2.0 SPS 08.

use crate::{File, Position, Statement, StatementKind as K, Token, TokenKind};

/// A syntax error in a SQLScript body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SqlScriptSyntaxError {
    /// where in the body, rows and columns counted from its start
    pub start: Position,
    /// byte offset in the body
    pub offset: usize,
    /// the token where the grammar got stuck, empty at the end
    pub text: String,
    pub message: &'static str,
}

/// The tokens of SQLScript: names, quoted names, `:variables`, numbers,
/// strings, operators; comments (`--`, `/* */` and ABAP's `*` in the first
/// column) as [`TokenKind::Comment`].
pub fn tokens(source: &str) -> Vec<Token> {
    let chars: Vec<(usize, char)> = source.char_indices().collect();
    let at = |i: usize| chars.get(i).map(|c| c.1);
    let offset = |i: usize| chars.get(i).map_or(source.len(), |c| c.0);
    let is_start = |c: char| c.is_alphabetic() || c == '_';
    let is_name = |c: char| c.is_alphanumeric() || matches!(c, '_' | '#' | '$');
    // row and column of each char
    let mut positions = Vec::with_capacity(chars.len() + 1);
    let (mut row, mut col) = (1u32, 1u32);
    for &(_, c) in &chars {
        positions.push(Position { row, col });
        if c == '\n' {
            row += 1;
            col = 1;
        } else {
            col += 1;
        }
    }
    positions.push(Position { row, col });
    let mut out: Vec<Token> = Vec::new();
    let mut i = 0;
    while let Some(c) = at(i) {
        if c.is_whitespace() {
            i += 1;
            continue;
        }
        let start = i;
        let line_start = i == 0 || at(i - 1) == Some('\n');
        let to_eol = |mut k: usize| {
            while at(k).is_some_and(|d| d != '\n') {
                k += 1;
            }
            k
        };
        let quoted = |mut k: usize, q: char| {
            // past the closing quote; doubled quotes inside
            k += 1;
            while let Some(d) = at(k) {
                k += 1;
                if d == q {
                    if at(k) == Some(q) {
                        k += 1;
                    } else {
                        break;
                    }
                }
            }
            k
        };
        let mut kind = TokenKind::Identifier;
        let next = at(i + 1);
        i = match c {
            '*' if line_start => {
                kind = TokenKind::Comment;
                to_eol(i)
            }
            '-' if next == Some('-') => {
                kind = TokenKind::Comment;
                to_eol(i)
            }
            '/' if next == Some('*') => {
                kind = TokenKind::Comment;
                let mut k = i + 2;
                while at(k).is_some() && !(at(k) == Some('*') && at(k + 1) == Some('/')) {
                    k += 1;
                }
                (k + 2).min(chars.len())
            }
            '\'' => {
                kind = TokenKind::String;
                quoted(i, '\'')
            }
            'N' | 'n' | 'X' | 'x' if next == Some('\'') => {
                kind = TokenKind::String;
                quoted(i + 1, '\'')
            }
            '"' => quoted(i, '"'),
            ':' if next == Some(':') && at(i + 2).is_some_and(is_start) => {
                let mut k = i + 2;
                while at(k).is_some_and(is_name) {
                    k += 1;
                }
                k
            }
            ':' if next == Some('"') => quoted(i + 1, '"'),
            ':' if next.is_some_and(is_start) => {
                let mut k = i + 1;
                while at(k).is_some_and(is_name) {
                    k += 1;
                }
                k
            }
            c if c.is_ascii_digit() || (c == '.' && next.is_some_and(|d| d.is_ascii_digit())) => {
                let mut k = i;
                while at(k).is_some_and(|d| d.is_ascii_digit()) {
                    k += 1;
                }
                if at(k) == Some('.') && at(k + 1).is_some_and(|d| d.is_ascii_digit()) {
                    k += 1;
                    while at(k).is_some_and(|d| d.is_ascii_digit()) {
                        k += 1;
                    }
                }
                let exponent = matches!(at(k), Some('e' | 'E'))
                    && (at(k + 1).is_some_and(|d| d.is_ascii_digit())
                        || (matches!(at(k + 1), Some('+' | '-'))
                            && at(k + 2).is_some_and(|d| d.is_ascii_digit())));
                if exponent {
                    k += 2;
                    while at(k).is_some_and(|d| d.is_ascii_digit()) {
                        k += 1;
                    }
                }
                k
            }
            c if is_start(c) => {
                let mut k = i;
                while at(k).is_some_and(is_name) {
                    k += 1;
                }
                k
            }
            _ => {
                let two: String = [Some(c), next].iter().flatten().collect();
                if matches!(
                    two.as_str(),
                    "<=" | ">=" | "<>" | "!=" | "||" | "=>" | ".." | ":="
                ) {
                    i + 2
                } else {
                    kind = match c {
                        '(' => TokenKind::ParenLeft,
                        ')' => TokenKind::ParenRight,
                        '[' => TokenKind::BracketLeft,
                        ']' => TokenKind::BracketRight,
                        ',' | '.' => TokenKind::Punctuation,
                        _ => TokenKind::Identifier,
                    };
                    i + 1
                }
            }
        };
        let end = i.min(chars.len());
        let (s, e) = (offset(start), offset(end));
        out.push(Token {
            kind,
            text: source[s..e].to_string(),
            start: positions[start],
            span: s..e,
            ws_before: start == 0 || at(start - 1).is_some_and(char::is_whitespace),
            ws_after: at(end).is_none_or(char::is_whitespace),
            expanded: false,
        });
    }
    out
}

/// The first syntax error in a SQLScript body, `None` when it matches the
/// grammar; an empty body has none.
pub fn syntax_error(body: &str) -> Option<SqlScriptSyntaxError> {
    let all = tokens(body);
    let code: Vec<&Token> = all
        .iter()
        .filter(|t| t.kind != TokenKind::Comment)
        .collect();
    let far = crate::grammar::sqlscript_mismatch(&code)?;
    let at = code.get(far);
    Some(SqlScriptSyntaxError {
        start: at.map_or(Position { row: 1, col: 1 }, |t| t.start),
        offset: at.map_or(body.len(), |t| t.span.start),
        text: at.map(|t| t.text.clone()).unwrap_or_default(),
        message: "does not match the SQLScript grammar",
    })
}

/// Versions of SAP HANA 2.0 that brought SQLScript and SQL constructs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Hana {
    V2Sps00,
    V2Sps01,
    V2Sps02,
    V2Sps03,
    V2Sps04,
    V2Sps05,
    V2Sps06,
    V2Sps07,
    V2Sps08,
}

impl Hana {
    /// As SAP writes it: `HANA 2.0 SPS 03`.
    pub fn name(self) -> &'static str {
        match self {
            Hana::V2Sps00 => "HANA 2.0 SPS 00",
            Hana::V2Sps01 => "HANA 2.0 SPS 01",
            Hana::V2Sps02 => "HANA 2.0 SPS 02",
            Hana::V2Sps03 => "HANA 2.0 SPS 03",
            Hana::V2Sps04 => "HANA 2.0 SPS 04",
            Hana::V2Sps05 => "HANA 2.0 SPS 05",
            Hana::V2Sps06 => "HANA 2.0 SPS 06",
            Hana::V2Sps07 => "HANA 2.0 SPS 07",
            Hana::V2Sps08 => "HANA 2.0 SPS 08",
        }
    }

    /// Short and sortable: `hana2sps03`.
    pub fn id(self) -> &'static str {
        match self {
            Hana::V2Sps00 => "hana2sps00",
            Hana::V2Sps01 => "hana2sps01",
            Hana::V2Sps02 => "hana2sps02",
            Hana::V2Sps03 => "hana2sps03",
            Hana::V2Sps04 => "hana2sps04",
            Hana::V2Sps05 => "hana2sps05",
            Hana::V2Sps06 => "hana2sps06",
            Hana::V2Sps07 => "hana2sps07",
            Hana::V2Sps08 => "hana2sps08",
        }
    }
}

/// A construct of a SQLScript body and the HANA version that brought it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HanaNeed {
    pub version: Hana,
    pub construct: &'static str,
    /// byte offset in the body
    pub offset: usize,
}

/// Built-in functions of SAP HANA 2.0, from the lists of new functions in
/// "What's New in the SAP HANA Platform 2.0" (SQL Reference, SPS 00 to 08).
const FUNCTIONS: &[(&str, Hana)] = &[
    ("ADD_MONTHS_LAST", Hana::V2Sps00),
    ("HIERARCHY", Hana::V2Sps00),
    ("HIERARCHY_ANCESTORS", Hana::V2Sps00),
    ("HIERARCHY_DESCENDANTS", Hana::V2Sps00),
    ("HIERARCHY_SIBLINGS", Hana::V2Sps00),
    ("JSON_QUERY", Hana::V2Sps00),
    ("JSON_TABLE", Hana::V2Sps00),
    ("JSON_VALUE", Hana::V2Sps00),
    ("NEWUID", Hana::V2Sps00),
    ("XMLEXTRACT", Hana::V2Sps00),
    ("XMLEXTRACTVALUE", Hana::V2Sps00),
    ("MAP_MERGE", Hana::V2Sps00),
    ("ABAP_ALPHANUM", Hana::V2Sps01),
    ("ABAP_NUMC", Hana::V2Sps01),
    ("CURRENT_MVCC_SNAPSHOT_TIMESTAMP", Hana::V2Sps01),
    ("HIERARCHY_COMPOSITE_ID", Hana::V2Sps01),
    ("HIERARCHY_LEVELED", Hana::V2Sps01),
    ("INITCAP", Hana::V2Sps01),
    ("SOUNDEX", Hana::V2Sps01),
    ("WIDTH_BUCKET", Hana::V2Sps01),
    ("HIERARCHY_SPANTREE", Hana::V2Sps02),
    ("HIERARCHY_TEMPORAL", Hana::V2Sps02),
    ("NDIV0", Hana::V2Sps02),
    ("TO_JSON_BOOLEAN", Hana::V2Sps02),
    ("XMLTABLE", Hana::V2Sps02),
    ("ADD_NANO100", Hana::V2Sps03),
    ("EXPRESSION_MACRO", Hana::V2Sps03),
    ("HIERARCHY_ANCESTORS_AGGREGATE", Hana::V2Sps03),
    ("HIERARCHY_COMPOSITE_ID_LENGTH", Hana::V2Sps03),
    ("NORMALIZE", Hana::V2Sps03),
    ("RECORD_COMMIT_TIMESTAMP", Hana::V2Sps03),
    ("RECORD_ID", Hana::V2Sps03),
    ("MAP_REDUCE", Hana::V2Sps03),
    ("ALLOW_PRECISION_LOSS", Hana::V2Sps04),
    ("CURRENT_SITE_ID", Hana::V2Sps04),
    ("CURRENT_USER_ID", Hana::V2Sps04),
    ("HEXTONUM", Hana::V2Sps04),
    ("NUMTOHEX", Hana::V2Sps04),
    ("ABAP_DF34RAW_TO_DECIMAL", Hana::V2Sps06),
    ("GROUP_SCORE", Hana::V2Sps07),
];

/// The constructs of a SQLScript body newer than SAP HANA 1.0, by the
/// SQLScript and SQL news of "What's New in the SAP HANA Platform 2.0":
/// functions, operators on table variables (SPS 01 to 03), the line number
/// variable (SPS 02), BOOLEAN, declarations with LIKE and ROW, INTO and
/// USING of dynamic SQL, SELECT INTO with DEFAULT, BETWEEN in IF and WHILE,
/// updatable cursors (SPS 03).
pub fn needs(body: &str) -> Vec<HanaNeed> {
    let all = tokens(body);
    let t: Vec<&Token> = all
        .iter()
        .filter(|t| t.kind != TokenKind::Comment)
        .collect();
    let word = |i: usize| {
        t.get(i)
            .map(|x| x.text.to_ascii_uppercase())
            .unwrap_or_default()
    };
    let is = |i: usize, w: &str| t.get(i).is_some_and(|x| x.text.eq_ignore_ascii_case(w));
    let mut out = Vec::new();
    let mut need = |i: usize, version: Hana, construct: &'static str| {
        out.push(HanaNeed {
            version,
            construct,
            offset: t[i].span.start,
        })
    };
    // the statement a token is in starts after the last ";"
    let mut statement_start = 0;
    for i in 0..t.len() {
        let w = word(i);
        if i > 0 && is(i - 1, ";") {
            statement_start = i;
        }
        if is(i + 1, "(") && !(i > 0 && is(i - 1, ".")) {
            if let Some(&(f, v)) = FUNCTIONS.iter().find(|(f, _)| *f == w) {
                need(i, v, f);
            }
        }
        // :lt.INSERT((1, 'a')), :lt.INSERT(:other), :lt.DELETE(1), :lt.SEARCH(...)
        if t[i].text.starts_with(':') && is(i + 1, ".") && is(i + 3, "(") {
            match word(i + 2).as_str() {
                "INSERT" if t.get(i + 4).is_some_and(|x| x.text.starts_with(':')) => need(
                    i,
                    Hana::V2Sps02,
                    "INSERT of a table variable into a table variable",
                ),
                "INSERT" => need(i, Hana::V2Sps01, "INSERT operator on a table variable"),
                "DELETE" => need(i, Hana::V2Sps01, "DELETE operator on a table variable"),
                "SEARCH" => need(i, Hana::V2Sps03, "SEARCH operator on a table variable"),
                _ => {}
            }
        }
        if w == "::CURRENT_LINE_NUMBER" {
            need(i, Hana::V2Sps02, "::CURRENT_LINE_NUMBER");
        }
        if w == "BOOLEAN" {
            need(i, Hana::V2Sps03, "BOOLEAN");
        }
        if w == "DECLARE" {
            // DECLARE a, b [TABLE|ROW] LIKE ..., DECLARE r ROW (...)
            let mut k = i + 1;
            while k < t.len()
                && !is(k, ";")
                && !matches!(
                    word(k).as_str(),
                    "LIKE" | "ROW" | "CURSOR" | "TABLE" | "=" | "DEFAULT"
                )
                && k < i + 20
            {
                k += 1;
            }
            match word(k).as_str() {
                "ROW" => need(k, Hana::V2Sps03, "row type variable"),
                "TABLE" if is(k + 1, "LIKE") => need(k, Hana::V2Sps03, "declaration with LIKE"),
                "LIKE" => need(k, Hana::V2Sps03, "declaration with LIKE"),
                _ => {}
            }
        }
        if (w == "INTO" || w == "USING")
            && matches!(word(statement_start).as_str(), "EXEC" | "EXECUTE")
        {
            need(i, Hana::V2Sps03, "INTO and USING of dynamic SQL");
        }
        if w == "DEFAULT"
            && (statement_start..i).any(|k| is(k, "INTO"))
            && word(statement_start) == "SELECT"
        {
            need(i, Hana::V2Sps03, "SELECT INTO with DEFAULT");
        }
        if w == "BETWEEN" && matches!(word(statement_start).as_str(), "IF" | "ELSEIF" | "WHILE") {
            // in the condition, not in a query of it
            let query = (statement_start..i).any(|k| is(k, "SELECT"));
            if !query {
                need(i, Hana::V2Sps03, "BETWEEN in IF and WHILE");
            }
        }
        if w == "CURRENT" && is(i + 1, "OF") && i > 0 && is(i - 1, "WHERE") {
            need(i, Hana::V2Sps03, "updatable cursor");
        }
    }
    out
}

/// The newest HANA version a SQLScript body needs, with the construct.
pub fn release(body: &str) -> Option<HanaNeed> {
    needs(body)
        .into_iter()
        .reduce(|a, b| if b.version > a.version { b } else { a })
}

/// The body of the AMDP method a NativeSQL statement is part of: the byte
/// range in the source, and the index of the statement.
fn body_of(file: &File, s: &Statement) -> Option<(std::ops::Range<usize>, usize)> {
    let index = file.statements.iter().position(|x| std::ptr::eq(x, s))?;
    let method = (0..index)
        .rev()
        .find(|&k| file.statements[k].kind != K::NativeSQL)?;
    Some((amdp_body(file, method)?, index))
}

/// The newest HANA version the part of a SQLScript body in a NativeSQL
/// statement needs.
pub fn statement_release(file: &File, s: &Statement) -> Option<HanaNeed> {
    if s.kind != K::NativeSQL || is_comment(file, s) {
        return None;
    }
    let (body, _) = body_of(file, s)?;
    let tokens = file.tokens();
    let first = tokens[s.tokens[0]].span.start;
    let last = tokens[*s.tokens.last()?].span.end;
    needs(&file.source()[body.clone()])
        .into_iter()
        .filter(|n| (first..last).contains(&(body.start + n.offset)))
        .reduce(|a, b| if b.version > a.version { b } else { a })
}

/// The objects after USING of an AMDP method statement, upper case, with
/// the position of their first token in [`Statement::tokens`]: tables,
/// views and `CLASS=>METHOD`.
fn using_list(file: &File, s: &Statement) -> Vec<(String, usize)> {
    let tokens = file.tokens();
    let t: Vec<&Token> = s.tokens.iter().map(|i| &tokens[*i]).collect();
    let Some(at) = t.iter().position(|x| x.upper() == "USING") else {
        return Vec::new();
    };
    let mut out: Vec<(String, usize)> = Vec::new();
    let mut k = at + 1;
    // USING SCHEMA s OBJECTS ...: objects of a logical schema, used by the
    // macro $ABAP.schema
    while k < t.len() && t[k].text != "." && t[k].upper() != "SCHEMA" {
        if t[k].kind == TokenKind::StaticArrow {
            if let (Some(last), Some(next)) = (out.last_mut(), t.get(k + 1)) {
                last.0 = format!("{}=>{}", last.0, next.upper());
                k += 2;
                continue;
            }
        }
        out.push((t[k].upper(), k));
        k += 1;
    }
    out
}

/// The names of objects in a SQLScript body: unquoted in upper case,
/// quoted as written without the quotes, with their byte offsets; not
/// those in comments and strings.
fn body_names(body: &str) -> Vec<(String, bool, usize)> {
    tokens(body)
        .into_iter()
        .filter(|t| matches!(t.kind, TokenKind::Identifier))
        .filter_map(|t| {
            if let Some(q) = t.text.strip_prefix('"').and_then(|x| x.strip_suffix('"')) {
                Some((q.replace("\"\"", "\""), true, t.span.start))
            } else if t.text.starts_with(|c: char| c.is_alphabetic() || c == '_') {
                Some((t.text.to_ascii_uppercase(), false, t.span.start))
            } else {
                None
            }
        })
        .collect()
}

/// An object after USING of an AMDP method its SQLScript body does not
/// use: SAP's syntax check reports "The user declaration for the object
/// is not needed". The position of the object in [`Statement::tokens`].
pub(crate) fn unused_using(file: &File, s: &Statement) -> Option<usize> {
    let index = file.statements.iter().position(|x| std::ptr::eq(x, s))?;
    let body = amdp_body(file, index)?;
    let names = body_names(&file.source()[body]);
    using_list(file, s)
        .into_iter()
        .find(|(object, _)| {
            !names.iter().any(|(n, quoted, _)| {
                if *quoted {
                    n == object
                } else {
                    n.eq_ignore_ascii_case(object)
                }
            })
        })
        .map(|(_, at)| at)
}

/// A use of an object in a SQLScript body that SAP's syntax check of AMDP
/// rejects, with the byte offset in the body: an AMDP method in quotes,
/// `"ZCL_X=>METH"`, not after USING, or an object after USING quoted in
/// other than upper case.
fn using_misuse(file: &File, method: usize, body: &str) -> Option<(usize, &'static str)> {
    let using = using_list(file, &file.statements[method]);
    let listed = |n: &str| using.iter().any(|(u, _)| u.eq_ignore_ascii_case(n));
    for (name, quoted, at) in body_names(body) {
        if !quoted {
            continue;
        }
        if name.contains("=>") && !listed(&name) {
            return Some((at, "an AMDP method used must be named after USING"));
        }
        if listed(&name) && name != name.to_ascii_uppercase() {
            return Some((at, "an object after USING must be in upper case in quotes"));
        }
    }
    None
}

/// The error of [`using_misuse`] when it is in this NativeSQL statement:
/// the position of its token in [`Statement::tokens`] and the message.
pub(crate) fn statement_using_error(file: &File, s: &Statement) -> Option<(usize, &'static str)> {
    if is_comment(file, s) {
        return None;
    }
    let (body, _) = body_of(file, s)?;
    let index = file.statements.iter().position(|x| std::ptr::eq(x, s))?;
    let method = (0..index)
        .rev()
        .find(|&k| file.statements[k].kind != K::NativeSQL)?;
    let (offset, message) = using_misuse(file, method, &file.source()[body.clone()])?;
    let at = body.start + offset;
    let tokens = file.tokens();
    s.tokens
        .iter()
        .position(|t| tokens[*t].span.contains(&at))
        .map(|k| (k, message))
}

/// The SQLScript body an AMDP method statement opens: the byte range of
/// the source between it and its ENDMETHOD.
pub fn amdp_body(file: &File, method: usize) -> Option<std::ops::Range<usize>> {
    let s = file.statements.get(method)?;
    if s.kind != K::MethodImplementation {
        return None;
    }
    let tokens = file.tokens();
    let words: Vec<String> = s.tokens.iter().map(|t| tokens[*t].upper()).collect();
    let sqlscript = words
        .windows(2)
        .any(|w| w[0] == "LANGUAGE" && w[1] == "SQLSCRIPT");
    let database = words.windows(2).any(|w| w[0] == "BY" && w[1] == "DATABASE");
    if !sqlscript || !database {
        return None;
    }
    let from = tokens[*s.tokens.last()?].span.end;
    let end = file.statements[method + 1..]
        .iter()
        .find(|e| e.kind == K::EndMethod)?;
    let to = tokens[end.tokens[0]].span.start;
    Some(from..to.max(from))
}

/// The syntax error of the SQLScript body a NativeSQL statement is part
/// of, when it is in this statement; `start` and `offset` in the file.
pub fn statement_syntax_error(file: &File, s: &Statement) -> Option<SqlScriptSyntaxError> {
    statement_error(file, s)?;
    let (body, _) = body_of(file, s)?;
    let mut e = syntax_error(&file.source()[body.clone()])?;
    e.offset += body.start;
    let before = &file.source()[..e.offset];
    let line_start = before.rfind('\n').map_or(0, |i| i + 1);
    e.start = Position {
        row: before.matches('\n').count() as u32 + 1,
        col: before[line_start..].chars().count() as u32 + 1,
    };
    Some(e)
}

/// The syntax error of the SQLScript body a NativeSQL statement is part
/// of, when it is in this statement: the position of its token in
/// [`Statement::tokens`].
pub(crate) fn statement_error(file: &File, s: &Statement) -> Option<usize> {
    let tokens = file.tokens();
    if s.tokens
        .iter()
        .all(|t| tokens[*t].kind == TokenKind::Comment)
    {
        return None;
    }
    let (body, index) = body_of(file, s)?;
    let error = syntax_error(&file.source()[body.clone()])?;
    let at = body.start + error.offset;
    // in the statement, or after the last statement of the body
    let span = |x: &Statement| {
        let first = tokens[x.tokens[0]].span.start;
        let last = tokens[*x.tokens.last().unwrap_or(&x.tokens[0])].span.end;
        first..last
    };
    let own = span(s);
    let last_code = file.statements[index..]
        .iter()
        .take_while(|n| n.kind == K::NativeSQL)
        .filter(|n| !is_comment(file, n))
        .last()
        .is_some_and(|n| std::ptr::eq(n, s));
    if at < own.start || (at >= own.end && !last_code) {
        return None;
    }
    Some(
        s.tokens
            .iter()
            .position(|t| tokens[*t].span.end > at)
            .unwrap_or(s.tokens.len() - 1),
    )
}

fn is_comment(file: &File, s: &Statement) -> bool {
    let tokens = file.tokens();
    s.tokens
        .iter()
        .all(|t| tokens[*t].kind == TokenKind::Comment)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lexer() {
        let t: Vec<String> = tokens("* abap\nlt = SELECT \"A\"\"B\", 'it''s', N'x', :lv, ::ROWCOUNT, 1.5e3, 1..3 -- c\n/* b */ FROM t WHERE a <= :b.c;")
            .into_iter()
            .filter(|t| t.kind != TokenKind::Comment)
            .map(|t| t.text)
            .collect();
        assert_eq!(
            t,
            [
                "lt",
                "=",
                "SELECT",
                "\"A\"\"B\"",
                ",",
                "'it''s'",
                ",",
                "N'x'",
                ",",
                ":lv",
                ",",
                "::ROWCOUNT",
                ",",
                "1.5e3",
                ",",
                "1",
                "..",
                "3",
                "FROM",
                "t",
                "WHERE",
                "a",
                "<=",
                ":b",
                ".",
                "c",
                ";"
            ]
        );
    }

    #[test]
    fn bodies() {
        for ok in [
            "",
            "lt = SELECT a, b AS c FROM ztab WHERE a = :iv;",
            "DECLARE lv INT = 0;\nIF :lv > 0 THEN\n  et = SELECT * FROM :lt;\nELSE\n  RETURN;\nEND IF;",
            "BEGIN SEQUENTIAL EXECUTION\n  CALL \"ZCL_X=>RUN\"(iv => :a, et => lt);\nEND;",
            "FOR i IN 1 .. 3 DO\n  :lt.INSERT((:i, 'x'), 1);\nEND FOR;",
            "et = SELECT CAST(a AS NVARCHAR(10)) AS b, COUNT(*) AS n FROM t GROUP BY a ORDER BY a DESC LIMIT 10;",
        ] {
            assert_eq!(syntax_error(ok), None, "{ok}");
        }
        let e = syntax_error("lt = SELECT * FRM ztab;").unwrap();
        assert_eq!(e.text, "FRM");
        // `lv` is an alias of the table
        let e = syntax_error("lt = SELECT * FROM ztab\nlv = 1;").unwrap();
        assert_eq!((e.text.as_str(), e.start.row), ("=", 2));
        assert!(syntax_error("IF :a = 1 THEN RETURN; END;").is_some());
    }

    #[test]
    fn hana_versions() {
        let body = "DECLARE b BOOLEAN = TRUE;\nDECLARE lt2 TABLE LIKE :lt;\n:lt.INSERT((1, 'a'), 1);\n:lt.INSERT(:lt2);\nEXEC 'SELECT 1 FROM DUMMY' INTO n;\nSELECT a INTO x DEFAULT 0 FROM t;\nIF :n BETWEEN 1 AND 2 THEN RETURN; END IF;\nlt3 = SELECT * FROM XMLTABLE('/a' PASSING :x COLUMNS a INT PATH 'a') AS x;\n-- JSON_VALUE( in a comment\nlt4 = SELECT JSON_VALUE(doc, '$.a') AS a FROM t WHERE a BETWEEN 1 AND 2;";
        let found: Vec<(&str, &str)> = needs(body)
            .iter()
            .map(|n| (n.version.name(), n.construct))
            .collect();
        assert_eq!(
            found,
            [
                ("HANA 2.0 SPS 03", "BOOLEAN"),
                ("HANA 2.0 SPS 03", "declaration with LIKE"),
                ("HANA 2.0 SPS 01", "INSERT operator on a table variable"),
                (
                    "HANA 2.0 SPS 02",
                    "INSERT of a table variable into a table variable"
                ),
                ("HANA 2.0 SPS 03", "INTO and USING of dynamic SQL"),
                ("HANA 2.0 SPS 03", "SELECT INTO with DEFAULT"),
                ("HANA 2.0 SPS 03", "BETWEEN in IF and WHILE"),
                ("HANA 2.0 SPS 02", "XMLTABLE"),
                ("HANA 2.0 SPS 00", "JSON_VALUE"),
            ]
        );
        assert_eq!(release(body).unwrap().construct, "BOOLEAN");
        assert_eq!(release("lt = SELECT * FROM t;"), None);
    }

    #[test]
    fn in_the_file() {
        let src = "CLASS zcl_x IMPLEMENTATION.\n  METHOD m BY DATABASE PROCEDURE FOR HDB LANGUAGE SQLSCRIPT.\n    lt = SELECT a || '.' || b FROM t;\n    et = SELECT * FRM :lt;\n  ENDMETHOD.\n  METHOD n BY DATABASE PROCEDURE FOR HDB LANGUAGE GRAPH.\n    whatever it is\n  ENDMETHOD.\nENDCLASS.";
        let file = crate::parse(src);
        let native: Vec<&Statement> = file
            .statements
            .iter()
            .filter(|s| s.kind == K::NativeSQL)
            .collect();
        assert_eq!(native.len(), 2);
        let e = statement_syntax_error(&file, native[0]).unwrap();
        assert_eq!((e.start.row, e.start.col, e.text.as_str()), (4, 19, "FRM"));
        assert_eq!(&src[e.offset..e.offset + 3], "FRM");
        // another language is not checked
        assert_eq!(statement_syntax_error(&file, native[1]), None);
        assert!(crate::check::syntax_error(&file, native[0]).is_some());
    }

    /// The rules of USING that SAP's syntax check of AMDP applies and that
    /// need no dictionary, from probes on the SAP trial 2025.
    #[test]
    fn using() {
        let check = |using: &str, body: &str| {
            let src = format!(
                "CLASS zcl_x IMPLEMENTATION.\n  METHOD m BY DATABASE PROCEDURE FOR HDB LANGUAGE SQLSCRIPT{using}.\n{body}\n  ENDMETHOD.\nENDCLASS."
            );
            let file = crate::parse(&src);
            file.statements
                .iter()
                .filter_map(|s| crate::check::syntax_error(&file, s).map(|e| e.message))
                .collect::<Vec<_>>()
        };
        for (using, body) in [
            (" USING scarr", "et = SELECT * FROM scarr;"),
            (" USING scarr", "et = SELECT * FROM \"SCARR\";"),
            (
                " USING SCARR",
                "et = SELECT s.* FROM scarr AS s WHERE carrid <> 'spfli';",
            ),
            (
                " USING zcl_y=>run",
                "CALL \"ZCL_Y=>RUN\"(iv => 1, et => et);",
            ),
            // not DDIC objects: no USING
            ("", "SELECT COUNT(*) INTO n FROM m_tables;"),
            // objects of a logical schema
            (
                " USING SCHEMA s OBJECTS scarr",
                "et = SELECT * FROM \"$ABAP.schema( S )\".\"SCARR\";",
            ),
        ] {
            assert_eq!(check(using, body), Vec::<&str>::new(), "{using} {body}");
        }
        assert_eq!(
            check(" USING scarr spfli", "-- spfli\net = SELECT * FROM scarr;"),
            ["an object after USING is not used in the body"]
        );
        assert_eq!(
            check(" USING scarr", "et = SELECT * FROM \"scarr\";"),
            [
                "an object after USING is not used in the body",
                "an object after USING must be in upper case in quotes"
            ]
        );
        assert_eq!(
            check("", "CALL \"ZCL_Y=>RUN\"(iv => 1, et => et);"),
            ["an AMDP method used must be named after USING"]
        );
    }
}
