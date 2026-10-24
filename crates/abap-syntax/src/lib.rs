//! ABAP statements: splits a token stream into statements, expands chained
//! statements (`DATA: a, b.`) and classifies each statement.
//!
//! Splitting follows abaplint (<https://github.com/abaplint/abaplint>, MIT).

pub mod check;
mod classify;
pub mod decl;
pub mod expr;
mod kind;
pub mod operands;
pub mod release;
pub mod structure;

pub use abap_lexer::{lex, Lexed, Position, Token, TokenKind};
pub use kind::StatementKind;

use std::collections::HashMap;

/// A macro, `DEFINE name. ... END-OF-DEFINITION.`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MacroDef {
    /// upper case
    pub name: String,
    /// the source text between DEFINE and END-OF-DEFINITION
    pub body: String,
}

/// Joins tokens to text, with a blank where the source has whitespace.
pub fn join_tokens<'a>(tokens: impl IntoIterator<Item = &'a Token>) -> String {
    let mut out = String::new();
    let mut prev: Option<&Token> = None;
    for t in tokens {
        if let Some(p) = prev {
            let gap = if t.expanded || p.expanded {
                t.ws_before
            } else {
                p.span.end != t.span.start
            };
            if gap {
                out.push(' ');
            }
        }
        out.push_str(&t.text);
        prev = Some(t);
    }
    out
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Statement {
    pub kind: StatementKind,
    /// Indices into [`File::tokens`]. For a chained statement this includes
    /// the shared prefix before the colon, so tokens are not always adjacent.
    pub tokens: Vec<usize>,
    /// The colon of a chained statement.
    pub colon: Option<usize>,
    /// Pragmas (`##NEEDED`) of a recognized statement, not part of `tokens`.
    pub pragmas: Vec<usize>,
}

#[derive(Debug, Clone)]
pub struct File {
    pub lexed: Lexed,
    pub statements: Vec<Statement>,
}

impl File {
    pub fn source(&self) -> &str {
        &self.lexed.source
    }

    pub fn tokens(&self) -> &[Token] {
        &self.lexed.tokens
    }

    pub fn statement_tokens<'a>(
        &'a self,
        s: &'a Statement,
    ) -> impl Iterator<Item = &'a Token> + 'a {
        s.tokens.iter().map(move |i| &self.lexed.tokens[*i])
    }

    /// Statement text with tokens joined by single spaces.
    pub fn statement_text(&self, s: &Statement) -> String {
        join_tokens(self.statement_tokens(s))
    }

    /// Macros defined in this file, in source order.
    pub fn macros(&self) -> Vec<MacroDef> {
        let mut out = Vec::new();
        let mut current: Option<(String, Vec<usize>)> = None;
        for (i, s) in self.statements.iter().enumerate() {
            match s.kind {
                StatementKind::Define => {
                    let name = s.tokens.get(1).map(|t| self.lexed.tokens[*t].upper());
                    current = name.map(|n| (n, Vec::new()));
                }
                StatementKind::EndOfDefinition => {
                    if let Some((name, body)) = current.take() {
                        out.push(MacroDef {
                            name,
                            body: self.raw_text(&body),
                        });
                    }
                }
                _ => {
                    if let Some((_, body)) = current.as_mut() {
                        body.push(i);
                    }
                }
            }
        }
        out
    }

    /// Source text from the first to the last token of the statements.
    fn raw_text(&self, statements: &[usize]) -> String {
        let tokens = &self.lexed.tokens;
        let real = |i: &usize| {
            self.statements[*i]
                .tokens
                .iter()
                .map(|t| &tokens[*t])
                .filter(|t| !t.expanded)
                .collect::<Vec<_>>()
        };
        let first = statements
            .iter()
            .flat_map(&real)
            .map(|t| t.span.start)
            .min();
        let last = statements.iter().flat_map(&real).map(|t| t.span.end).max();
        match (first, last) {
            (Some(a), Some(b)) if a < b => self.lexed.source[a..b].to_string(),
            _ => String::new(),
        }
    }
}

/// Lexes and splits ABAP source into classified statements; macro calls are
/// followed by the statements of their expansion.
pub fn parse(source: &str) -> File {
    parse_with_macros(source, &[])
}

/// Like [`parse`], with macros defined elsewhere, e.g. in the macros include
/// of a class or an earlier include of the program.
pub fn parse_with_macros(source: &str, macros: &[MacroDef]) -> File {
    let lexed = lex(source);
    let mut table: MacroTable = HashMap::new();
    for m in macros {
        table
            .entry(m.name.clone())
            .or_default()
            .push((0, m.body.clone()));
    }
    let statements = statements_of(&lexed.tokens, &lexed.source, &mut table);
    let mut file = File { lexed, statements };
    expand_macros(&mut file, &table, 0);
    native_sql(&file.lexed.tokens, &mut file.statements);
    file
}

/// Macro name -> definitions as (source offset where it becomes valid, body).
/// A macro is valid after its END-OF-DEFINITION; a later definition replaces it.
type MacroTable = HashMap<String, Vec<(usize, String)>>;

/// The definition valid at a source offset.
fn macro_at<'a>(table: &'a MacroTable, name: &str, offset: usize) -> Option<&'a String> {
    table
        .get(name)?
        .iter()
        .rev()
        .find(|(valid_from, _)| *valid_from <= offset)
        .map(|(_, body)| body)
}

fn statements_of(tokens: &[Token], source: &str, macros: &mut MacroTable) -> Vec<Statement> {
    let mut statements = split(tokens);
    classify_all(tokens, &mut statements);
    mark_macros(tokens, source, &mut statements, macros);
    let mut statements = lazy_unknown(tokens, statements);
    extract_pragmas(tokens, &mut statements);
    statements
}

/// Nested macro calls deeper than this are not expanded.
const MAX_MACRO_DEPTH: usize = 10;

/// Inserts the expansion of every macro call after the call.
fn expand_macros(file: &mut File, macros: &MacroTable, depth: usize) {
    if depth >= MAX_MACRO_DEPTH || macros.is_empty() {
        return;
    }
    let statements = std::mem::take(&mut file.statements);
    let mut out = Vec::with_capacity(statements.len());
    for s in statements {
        let call = s.kind == StatementKind::MacroCall;
        out.push(s);
        if !call {
            continue;
        }
        let s = out.last().unwrap();
        let tokens: Vec<&Token> = s.tokens.iter().map(|t| &file.lexed.tokens[*t]).collect();
        let at = tokens[0].clone();
        let Some(body) = macro_at(macros, &at.upper(), at.span.start) else {
            continue;
        };
        let text = substitute(body, &macro_arguments(&tokens[1..]));
        // inside the expansion, the definitions valid at the call apply
        let mut inner_macros: MacroTable = macros
            .keys()
            .filter_map(|name| {
                macro_at(macros, name, at.span.start).map(|b| (name.clone(), vec![(0, b.clone())]))
            })
            .collect();
        let mut inner = {
            let lexed = lex(&text);
            let statements = statements_of(&lexed.tokens, &lexed.source, &mut inner_macros);
            File { lexed, statements }
        };
        expand_macros(&mut inner, &inner_macros, depth + 1);
        // move the tokens into the file, positioned at the call
        let offset = file.lexed.tokens.len();
        for mut t in inner.lexed.tokens {
            t.start = at.start;
            t.span = at.span.start..at.span.start;
            t.expanded = true;
            file.lexed.tokens.push(t);
        }
        for mut st in inner.statements {
            if st.kind == StatementKind::Comment {
                continue;
            }
            st.tokens.iter_mut().for_each(|t| *t += offset);
            st.pragmas.iter_mut().for_each(|t| *t += offset);
            st.colon = st.colon.map(|c| c + offset);
            out.push(st);
        }
    }
    file.statements = out;
}

/// Arguments of a macro call: blank separated, `ls_x-comp` is one argument.
fn macro_arguments(tokens: &[&Token]) -> Vec<String> {
    let mut args: Vec<String> = Vec::new();
    for t in tokens {
        if matches!(t.kind, TokenKind::Punctuation | TokenKind::Pragma) {
            continue;
        }
        match args.last_mut() {
            Some(last) if !t.ws_before => last.push_str(&t.text),
            _ => args.push(t.text.clone()),
        }
    }
    args
}

/// Replaces the placeholders &1 ... &9 in a macro body. Literals are kept.
fn substitute(body: &str, args: &[String]) -> String {
    let lexed = lex(body);
    let mut out = String::with_capacity(body.len());
    let mut pos = 0;
    for t in &lexed.tokens {
        out.push_str(&lexed.source[pos..t.span.start]);
        let literal = matches!(t.kind, TokenKind::String | TokenKind::Comment);
        if !literal && t.text.contains('&') {
            let mut text = t.text.clone();
            for (i, a) in args.iter().enumerate().take(9).rev() {
                text = text.replace(&format!("&{}", i + 1), a);
            }
            // placeholders without argument become empty
            for i in (1..=9).rev() {
                text = text.replace(&format!("&{i}"), "");
            }
            out.push_str(&text);
        } else {
            out.push_str(&lexed.source[t.span.clone()]);
        }
        pos = t.span.end;
    }
    out.push_str(&lexed.source[pos..]);
    out
}

fn new_statement(kind: StatementKind, tokens: Vec<usize>, colon: Option<usize>) -> Statement {
    Statement {
        kind,
        tokens,
        colon,
        pragmas: Vec::new(),
    }
}

/// Splits tokens into statements at "." and, inside chains, at ",".
/// Additional colons after the first one in a statement are dropped.
fn split(tokens: &[Token]) -> Vec<Statement> {
    let mut statements = Vec::new();
    let mut add: Vec<usize> = Vec::new();
    let mut pre: Vec<usize> = Vec::new();
    let mut colon: Option<usize> = None;
    // inside EXEC SQL and AMDP bodies ":" marks host variables, not chaining
    let mut native = false;

    let unknown = |pre: &[usize], add: &[usize], colon| {
        let mut t = pre.to_vec();
        t.extend_from_slice(add);
        new_statement(StatementKind::Unknown, t, colon)
    };

    for (i, token) in tokens.iter().enumerate() {
        if token.kind == TokenKind::Comment {
            statements.push(new_statement(StatementKind::Comment, vec![i], None));
            continue;
        }
        add.push(i);
        match token.text.as_str() {
            "." => {
                native = native_after(tokens, native, &add, colon);
                statements.push(unknown(&pre, &add, colon));
                add.clear();
                pre.clear();
                colon = None;
            }
            "," if !pre.is_empty() => {
                statements.push(unknown(&pre, &add, colon));
                add.clear();
            }
            ":" if !native => {
                add.pop();
                if colon.is_none() {
                    colon = Some(i);
                    pre.append(&mut add);
                }
            }
            _ => {}
        }
    }
    if !add.is_empty() {
        statements.push(unknown(&pre, &add, colon));
    }
    statements
}

/// Whether the statement that just ended opens or closes a native SQL region.
fn native_after(tokens: &[Token], native: bool, add: &[usize], colon: Option<usize>) -> bool {
    let words: Vec<String> = add
        .iter()
        .map(|i| &tokens[*i])
        .filter(|t| t.kind != TokenKind::Pragma)
        .map(|t| t.upper())
        .collect();
    let n = words.len();
    if n >= 2 && (words[n - 2] == "ENDEXEC" || words[n - 2] == "ENDMETHOD") {
        return false;
    }
    if colon.is_some() {
        return native;
    }
    if n >= 2 && words[0] == "EXEC" && words[1] == "SQL" {
        return true;
    }
    if is_amdp_method(&words) {
        return true;
    }
    native
}

fn is_amdp_method(words: &[String]) -> bool {
    words.first().is_some_and(|w| w == "METHOD")
        && ["BY", "DATABASE", "LANGUAGE"]
            .iter()
            .all(|k| words.iter().any(|w| w == k))
}

fn classify_all(tokens: &[Token], statements: &mut [Statement]) {
    for s in statements.iter_mut() {
        if s.kind == StatementKind::Unknown {
            s.kind = classify_tokens(tokens, &s.tokens);
        }
    }
}

fn classify_tokens(tokens: &[Token], indices: &[usize]) -> StatementKind {
    let last = &tokens[*indices.last().unwrap()];
    let terminated = last.kind == TokenKind::Punctuation;
    if indices.len() == 1 && (terminated || last.kind == TokenKind::Pragma) {
        return StatementKind::Empty;
    }
    if !terminated {
        return StatementKind::Unknown;
    }
    let body: Vec<&Token> = indices[..indices.len() - 1]
        .iter()
        .map(|i| &tokens[*i])
        .filter(|t| t.kind != TokenKind::Pragma)
        .collect();
    if body.is_empty() {
        StatementKind::Empty
    } else {
        classify::classify(&body)
    }
}

/// Splits an Unknown statement at the first line break after which the rest
/// is a known statement; typical while typing, when the period is still missing.
fn lazy_unknown(tokens: &[Token], statements: Vec<Statement>) -> Vec<Statement> {
    const SKIP: &[&str] = &[
        "CALL METHOD ",
        "RAISE EXCEPTION TYPE ",
        "READ TABLE ",
        "LOOP AT ",
        "SELECT SINGLE ",
        "CALL FUNCTION ",
    ];
    let mut result = Vec::with_capacity(statements.len());
    // native SQL is not ABAP, like abaplint its statements are not split
    let mut native = false;
    for mut s in statements {
        let words = || {
            s.tokens
                .iter()
                .map(|t| tokens[*t].upper())
                .collect::<Vec<_>>()
        };
        match s.kind {
            StatementKind::ExecSQL => native = true,
            StatementKind::MethodImplementation if is_amdp_method(&words()) => native = true,
            StatementKind::EndExec | StatementKind::EndMethod => native = false,
            _ => {}
        }
        if s.kind == StatementKind::Unknown && !native {
            let concat: String = s.tokens.iter().map(|i| tokens[*i].upper() + " ").collect();
            if !SKIP.iter().any(|p| concat.starts_with(p)) {
                for i in 1..s.tokens.len() {
                    let row = |k: usize| tokens[s.tokens[k]].start.row;
                    if row(i) == row(i - 1) || s.tokens.len() - i == 1 {
                        continue;
                    }
                    let kind = classify_tokens(tokens, &s.tokens[i..]);
                    if !matches!(kind, StatementKind::Unknown | StatementKind::Empty) {
                        result.push(new_statement(
                            StatementKind::Unknown,
                            s.tokens[..i].to_vec(),
                            None,
                        ));
                        s = new_statement(kind, s.tokens[i..].to_vec(), None);
                        break;
                    }
                }
            }
        }
        result.push(s);
    }
    result
}

/// Moves pragmas out of the tokens of recognized statements, like abaplint.
fn extract_pragmas(tokens: &[Token], statements: &mut [Statement]) {
    for s in statements.iter_mut() {
        if matches!(
            s.kind,
            StatementKind::Unknown | StatementKind::Empty | StatementKind::Comment
        ) {
            continue;
        }
        let last = s.tokens.len() - 1;
        let (pragmas, rest): (Vec<_>, Vec<_>) = s
            .tokens
            .iter()
            .enumerate()
            .partition(|(i, t)| *i != last && tokens[**t].kind == TokenKind::Pragma);
        if !pragmas.is_empty() {
            s.pragmas = pragmas.into_iter().map(|(_, t)| *t).collect();
            s.tokens = rest.into_iter().map(|(_, t)| *t).collect();
        }
    }
}

/// Statements inside DEFINE ... END-OF-DEFINITION become MacroContent, and
/// unknown statements starting with a defined macro name become MacroCall.
/// Definitions are added to `macros`; a later definition replaces an earlier one.
fn mark_macros(
    tokens: &[Token],
    source: &str,
    statements: &mut [Statement],
    macros: &mut MacroTable,
) {
    let mut current: Option<(String, Option<usize>, usize)> = None; // name, body start, body end
    for s in statements.iter_mut() {
        match s.kind {
            StatementKind::Define => {
                current = s.tokens.get(1).map(|i| (tokens[*i].upper(), None, 0));
            }
            StatementKind::EndOfDefinition if current.is_some() => {
                let (name, start, end) = current.take().unwrap();
                let body = start
                    .map(|a| source[a..end].to_string())
                    .unwrap_or_default();
                let valid_from = s.tokens.last().map(|t| tokens[*t].span.end).unwrap_or(end);
                macros.entry(name).or_default().push((valid_from, body));
            }
            _ if current.is_some() => {
                let (_, start, end) = current.as_mut().unwrap();
                for t in s.tokens.iter().map(|i| &tokens[*i]) {
                    if start.is_none() {
                        *start = Some(t.span.start);
                    }
                    *end = (*end).max(t.span.end);
                }
                if s.kind != StatementKind::Comment {
                    s.kind = StatementKind::MacroContent;
                }
            }
            StatementKind::Unknown | StatementKind::MacroCall => {
                let first = &tokens[s.tokens[0]];
                if macro_at(macros, &first.upper(), first.span.start).is_some() {
                    s.kind = StatementKind::MacroCall;
                }
            }
            _ => {}
        }
    }
}

/// Statements inside EXEC SQL ... ENDEXEC and AMDP method bodies are native SQL.
fn native_sql(tokens: &[Token], statements: &mut Vec<Statement>) {
    let mut sql = false;
    let mut i = 0;
    while i < statements.len() {
        let s = &statements[i];
        let words = || {
            s.tokens
                .iter()
                .map(|t| tokens[*t].upper())
                .collect::<Vec<_>>()
        };
        if s.kind == StatementKind::ExecSQL
            || (s.kind == StatementKind::MethodImplementation && is_amdp_method(&words()))
        {
            sql = true;
        } else if sql {
            if matches!(s.kind, StatementKind::EndExec | StatementKind::EndMethod) {
                sql = false;
            } else {
                let w = words();
                let n = w.len();
                let end = match w.get(n.wrapping_sub(2)).map(String::as_str) {
                    Some("ENDMETHOD") => Some(StatementKind::EndMethod),
                    Some("ENDEXEC") => Some(StatementKind::EndExec),
                    _ => None,
                };
                if let (Some(end), true) = (end, n > 2 && w[n - 1] == ".") {
                    let toks = statements[i].tokens.clone();
                    statements[i] =
                        new_statement(StatementKind::NativeSQL, toks[..n - 2].to_vec(), None);
                    statements.insert(i + 1, new_statement(end, toks[n - 2..].to_vec(), None));
                    sql = false;
                    i += 1;
                } else {
                    statements[i].kind = StatementKind::NativeSQL;
                }
            }
        }
        i += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(src: &str) -> Vec<&'static str> {
        parse(src)
            .statements
            .iter()
            .map(|s| s.kind.as_str())
            .collect()
    }

    #[test]
    fn chained_statements_share_prefix() {
        let f = parse("DATA: a TYPE i, b TYPE c.");
        assert_eq!(f.statements.len(), 2);
        assert_eq!(f.statement_text(&f.statements[0]), "DATA a TYPE i,");
        assert_eq!(f.statement_text(&f.statements[1]), "DATA b TYPE c.");
        assert!(f.statements.iter().all(|s| s.kind == StatementKind::Data));
        assert!(f.statements.iter().all(|s| s.colon == Some(1)));
    }

    #[test]
    fn chain_with_call_prefix() {
        let f = parse("WRITE: / 'a', / 'b'.");
        assert_eq!(kinds("WRITE: / 'a', / 'b'."), ["Write", "Write"]);
        assert_eq!(f.statement_text(&f.statements[1]), "WRITE / 'b'.");
    }

    #[test]
    fn comments_are_separate_statements() {
        assert_eq!(
            kinds("* header\nDATA a TYPE i. \" note\n"),
            ["Comment", "Data", "Comment"]
        );
    }

    #[test]
    fn empty_statement_and_pragmas() {
        assert_eq!(kinds("."), ["Empty"]);
        assert_eq!(kinds("DATA a TYPE c ##NEEDED."), ["Data"]);
    }

    #[test]
    fn unterminated_statement_is_unknown() {
        assert_eq!(kinds("DATA a TYPE i"), ["Unknown"]);
    }

    #[test]
    fn exec_sql_body_is_native() {
        assert_eq!(
            kinds("EXEC SQL.\n  SELECT a INTO :lv FROM t\nENDEXEC."),
            ["ExecSQL", "NativeSQL", "EndExec"]
        );
    }

    #[test]
    fn amdp_body_is_native() {
        let src = "METHOD m BY DATABASE PROCEDURE FOR HDB LANGUAGE SQLSCRIPT.\n\
                   et = SELECT * FROM t WHERE a = :iv;\nENDMETHOD.";
        assert_eq!(
            kinds(src),
            ["MethodImplementation", "NativeSQL", "EndMethod"]
        );
    }

    #[test]
    fn macros() {
        let src = "DEFINE m.\n  WRITE &1.\nEND-OF-DEFINITION.\nm 'x'.";
        assert_eq!(
            kinds(src),
            [
                "Define",
                "MacroContent",
                "EndOfDefinition",
                "MacroCall",
                "Write"
            ]
        );
    }

    #[test]
    fn macro_expansion() {
        let src = "DEFINE fill.\n  ls_x-a = &1.\n  APPEND ls_x TO &2.\n  \" note\n  SELECT SINGLE * FROM &3 INTO ls_y.\nEND-OF-DEFINITION.\n\
                   fill: 'A' lt_a ztab, ls_z-val lt_b ztab2.";
        let f = parse(src);
        let expanded: Vec<_> = f
            .statements
            .iter()
            .filter(|s| s.tokens.iter().all(|t| f.tokens()[*t].expanded))
            .map(|s| f.statement_text(s))
            .collect();
        assert_eq!(
            expanded,
            [
                "ls_x-a = 'A'.",
                "APPEND ls_x TO lt_a.",
                "SELECT SINGLE * FROM ztab INTO ls_y.",
                "ls_x-a = ls_z-val.",
                "APPEND ls_x TO lt_b.",
                "SELECT SINGLE * FROM ztab2 INTO ls_y.",
            ]
        );
        let call = f
            .statements
            .iter()
            .find(|s| s.kind == StatementKind::MacroCall)
            .unwrap();
        let first_expanded = f
            .statements
            .iter()
            .find(|s| s.kind == StatementKind::Move)
            .unwrap();
        assert_eq!(
            f.tokens()[first_expanded.tokens[0]].start,
            f.tokens()[call.tokens[0]].start
        );
    }

    #[test]
    fn macros_from_elsewhere_and_nesting() {
        let defs = [MacroDef {
            name: "LOG".to_string(),
            body: "CALL FUNCTION 'Z_LOG' EXPORTING text = &1.".to_string(),
        }];
        let f = parse_with_macros(
            "DEFINE twice.\n  log &1. log &1.\nEND-OF-DEFINITION.\ntwice 'x'.",
            &defs,
        );
        let calls = f
            .statements
            .iter()
            .filter(|s| s.kind == StatementKind::CallFunction)
            .count();
        assert_eq!(calls, 2);
        assert_eq!(f.macros()[0].name, "TWICE");
    }

    #[test]
    fn redefined_macro_applies_from_its_definition() {
        let src = "DEFINE m.\n  WRITE &1.\nEND-OF-DEFINITION.\nm 'a'.\n\
                   DEFINE m.\n  CLEAR &1.\nEND-OF-DEFINITION.\nm lv.";
        let k = kinds(src);
        let expanded: Vec<_> = k
            .iter()
            .filter(|x| **x == "Write" || **x == "Clear")
            .collect();
        assert_eq!(expanded, [&"Write", &"Clear"]);
    }
}
