//! ABAP tokenizer.
//!
//! Token boundaries and token kinds follow the lexer of abaplint
//! (<https://github.com/abaplint/abaplint>, MIT) so results can be compared
//! one to one against that reference implementation.
//!
//! ABAP is whitespace sensitive in a few places, e.g. `foo( )` vs `foo ( )`
//! or `a-b` vs `a - b`. Every token therefore records whether whitespace
//! directly precedes and follows it, see [`Token::ws_before`] and
//! [`Token::ws_after`].

use std::ops::Range;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum TokenKind {
    Identifier,
    /// `'text'` or `` `text` ``
    String,
    /// complete template without embedded expressions, `|text|`
    StringTemplate,
    /// `|text{`
    StringTemplateBegin,
    /// `}text{`
    StringTemplateMiddle,
    /// `}text|`
    StringTemplateEnd,
    Comment,
    /// `##NAME` or `##NAME[ARG]`
    Pragma,
    /// `.` or `,`
    Punctuation,
    ParenLeft,
    ParenRight,
    BracketLeft,
    BracketRight,
    Dash,
    Plus,
    At,
    /// `->`
    InstanceArrow,
    /// `=>`
    StaticArrow,
    /// `\_association` in CDS path expressions
    AssociationName,
}

impl TokenKind {
    /// Whether abaplint distinguishes whitespace around this kind with
    /// separate token classes (`WParenLeftW`, `DashW`, ...).
    pub fn is_whitespace_sensitive(self) -> bool {
        matches!(
            self,
            TokenKind::ParenLeft
                | TokenKind::ParenRight
                | TokenKind::BracketLeft
                | TokenKind::BracketRight
                | TokenKind::Dash
                | TokenKind::Plus
                | TokenKind::At
                | TokenKind::InstanceArrow
                | TokenKind::StaticArrow
        )
    }
}

/// 1-based row and column, columns counted in characters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Position {
    pub row: u32,
    pub col: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Token {
    pub kind: TokenKind,
    pub text: String,
    pub start: Position,
    /// Byte range into [`Lexed::source`].
    pub span: Range<usize>,
    pub ws_before: bool,
    pub ws_after: bool,
    /// Comes from a macro expansion: `start` is the position of the macro
    /// call, `span` is empty.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "std::ops::Not::not")
    )]
    pub expanded: bool,
}

impl Token {
    pub fn upper(&self) -> String {
        self.text.to_ascii_uppercase()
    }

    /// Case-insensitive comparison of the token text.
    pub fn is(&self, text: &str) -> bool {
        self.text.eq_ignore_ascii_case(text)
    }

    pub fn end(&self) -> Position {
        Position {
            row: self.start.row,
            col: self.start.col + self.text.chars().count() as u32,
        }
    }

    /// Name of the corresponding abaplint token class, e.g. `WParenLeftW`.
    pub fn abaplint_name(&self) -> String {
        let base = match self.kind {
            TokenKind::Identifier => "Identifier",
            TokenKind::String => "StringToken",
            TokenKind::StringTemplate => "StringTemplate",
            TokenKind::StringTemplateBegin => "StringTemplateBegin",
            TokenKind::StringTemplateMiddle => "StringTemplateMiddle",
            TokenKind::StringTemplateEnd => "StringTemplateEnd",
            TokenKind::Comment => "Comment",
            TokenKind::Pragma => "Pragma",
            TokenKind::Punctuation => "Punctuation",
            TokenKind::ParenLeft => "ParenLeft",
            TokenKind::ParenRight => "ParenRight",
            TokenKind::BracketLeft => "BracketLeft",
            TokenKind::BracketRight => "BracketRight",
            TokenKind::Dash => "Dash",
            TokenKind::Plus => "Plus",
            TokenKind::At => "At",
            TokenKind::InstanceArrow => "InstanceArrow",
            TokenKind::StaticArrow => "StaticArrow",
            TokenKind::AssociationName => "AssociationName",
        };
        if !self.kind.is_whitespace_sensitive() {
            return base.to_string();
        }
        format!(
            "{}{}{}",
            if self.ws_before { "W" } else { "" },
            base,
            if self.ws_after { "W" } else { "" }
        )
    }
}

#[derive(Debug, Clone)]
pub struct Lexed {
    /// The input with carriage returns removed; token spans index into this.
    pub source: String,
    pub tokens: Vec<Token>,
}

/// Tokenizes ABAP source code. Never fails: malformed input still yields tokens.
pub fn lex(source: &str) -> Lexed {
    let source = source.replace('\r', "");
    let tokens = Lexer::new(&source).run();
    Lexed { source, tokens }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    Normal,
    Ping,
    Str,
    Template,
    Comment,
    Pragma,
}

const EOF: i32 = -1;
const TAB: i32 = '\t' as i32;
const NL: i32 = '\n' as i32;
const SPACE: i32 = ' ' as i32;
const DQUOTE: i32 = '"' as i32;
const HASH: i32 = '#' as i32;
const QUOTE: i32 = '\'' as i32;
const STAR: i32 = '*' as i32;
const DASH: i32 = '-' as i32;
const COMMA: i32 = ',' as i32;
const DOT: i32 = '.' as i32;
const COLON: i32 = ':' as i32;
const EQUALS: i32 = '=' as i32;
const GT: i32 = '>' as i32;
const AT: i32 = '@' as i32;
const BACKSLASH: i32 = '\\' as i32;
const BACKTICK: i32 = '`' as i32;
const LBRACE: i32 = '{' as i32;
const PIPE: i32 = '|' as i32;
const RBRACE: i32 = '}' as i32;
const PLUS: i32 = '+' as i32;
const LPAREN: i32 = '(' as i32;
const RPAREN: i32 = ')' as i32;
const LBRACKET: i32 = '[' as i32;
const RBRACKET: i32 = ']' as i32;

/// characters that terminate the current token
fn is_split(c: i32) -> bool {
    matches!(
        c,
        SPACE
            | COLON
            | DOT
            | COMMA
            | DASH
            | PLUS
            | LPAREN
            | RPAREN
            | LBRACKET
            | RBRACKET
            | BACKSLASH
            | TAB
            | NL
    )
}

/// single characters that are emitted as their own token
fn is_buf(c: i32) -> bool {
    matches!(
        c,
        DOT | COMMA | COLON | LPAREN | RPAREN | LBRACKET | RBRACKET | PLUS | AT
    )
}

/// separators accepted directly after literals and string templates
fn is_after_literal(c: i32) -> bool {
    matches!(
        c,
        EOF | SPACE
            | TAB
            | NL
            | DOT
            | COMMA
            | COLON
            | DQUOTE
            | LPAREN
            | RPAREN
            | LBRACKET
            | RBRACKET
    )
}

struct Lexer {
    chars: Vec<char>,
    /// byte offset of every char, plus one trailing entry for the end
    byte_of: Vec<usize>,
    line_starts: Vec<usize>,
    offset: isize,
    mode: Mode,
    buf_start: isize,
    buf_end: isize,
    buf_empty: bool,
    tokens: Vec<Token>,
}

impl Lexer {
    fn new(source: &str) -> Self {
        let chars: Vec<char> = source.chars().collect();
        let mut byte_of = Vec::with_capacity(chars.len() + 1);
        let mut line_starts = vec![0];
        let mut b = 0;
        for (i, c) in chars.iter().enumerate() {
            byte_of.push(b);
            b += c.len_utf8();
            if *c == '\n' {
                line_starts.push(i + 1);
            }
        }
        byte_of.push(b);
        Lexer {
            chars,
            byte_of,
            line_starts,
            offset: -1,
            mode: Mode::Normal,
            buf_start: 0,
            buf_end: 0,
            buf_empty: true,
            tokens: Vec::new(),
        }
    }

    fn len(&self) -> isize {
        self.chars.len() as isize
    }

    fn ch(&self, o: isize) -> i32 {
        if o < 0 || o >= self.len() {
            EOF
        } else {
            self.chars[o as usize] as i32
        }
    }

    fn current(&self) -> i32 {
        if self.offset < 0 {
            // simulate a newline before the first character, handles "*" comments
            NL
        } else {
            self.ch(self.offset)
        }
    }

    fn advance(&mut self) -> bool {
        if self.offset == self.len() {
            return false;
        }
        self.offset += 1;
        true
    }

    fn count_previous(&self, c: i32) -> usize {
        let mut count = 0;
        let mut o = self.offset - 1;
        while o >= 0 && self.ch(o) == c {
            count += 1;
            o -= 1;
        }
        count
    }

    fn buf_add(&mut self, offset: isize) {
        if self.buf_empty {
            self.buf_start = offset.max(0);
            self.buf_empty = false;
        }
        self.buf_end = offset + 1;
    }

    fn buf_len(&self) -> isize {
        self.buf_end - self.buf_start
    }

    fn buf_range(&self) -> (usize, usize) {
        let start = self.buf_start.clamp(0, self.len()) as usize;
        let end = self.buf_end.clamp(0, self.len()) as usize;
        (start, end.max(start))
    }

    fn buf_is_blank(&self) -> bool {
        let (s, e) = self.buf_range();
        self.chars[s..e].iter().all(|c| c.is_whitespace())
    }

    fn buf_count_is_even(&self, c: i32) -> bool {
        let (s, e) = self.buf_range();
        self.chars[s..e].iter().filter(|x| **x as i32 == c).count() % 2 == 0
    }

    fn buf_clear(&mut self) {
        self.buf_start = 0;
        self.buf_end = 0;
        self.buf_empty = true;
    }

    fn position(&self, char_offset: usize) -> Position {
        let line = match self.line_starts.binary_search(&char_offset) {
            Ok(i) => i,
            Err(i) => i - 1,
        };
        Position {
            row: line as u32 + 1,
            col: (char_offset - self.line_starts[line]) as u32 + 1,
        }
    }

    fn add(&mut self) {
        let (mut s, mut e) = self.buf_range();
        while s < e && self.chars[s].is_whitespace() {
            s += 1;
        }
        while e > s && self.chars[e - 1].is_whitespace() {
            e -= 1;
        }
        if s == e {
            self.buf_clear();
            return;
        }

        let text: String = self.chars[s..e].iter().collect();
        let prev = self.ch(s as isize - 1);
        let ws_before = matches!(prev, SPACE | NL | TAB | COLON);
        let next = self.ch(e as isize);
        let ws_after = matches!(next, SPACE | NL | TAB | COLON | COMMA | DOT | EOF | DQUOTE);
        let first = self.chars[s];
        let last = self.chars[e - 1];
        let length = e - s;

        let kind = match self.mode {
            Mode::Comment => TokenKind::Comment,
            Mode::Ping | Mode::Str => TokenKind::String,
            Mode::Template => {
                if first == '|' && last == '|' {
                    TokenKind::StringTemplate
                } else if first == '|' && last == '{' && ws_after {
                    TokenKind::StringTemplateBegin
                } else if first == '}' && last == '|' && ws_before {
                    TokenKind::StringTemplateEnd
                } else if first == '}' && last == '{' && ws_after && ws_before {
                    TokenKind::StringTemplateMiddle
                } else {
                    TokenKind::Identifier
                }
            }
            Mode::Normal | Mode::Pragma => {
                if length > 2 && text.starts_with("##") {
                    TokenKind::Pragma
                } else if length == 1 {
                    match first {
                        '.' | ',' => TokenKind::Punctuation,
                        '[' => TokenKind::BracketLeft,
                        ']' => TokenKind::BracketRight,
                        '(' => TokenKind::ParenLeft,
                        ')' => TokenKind::ParenRight,
                        '-' => TokenKind::Dash,
                        '+' => TokenKind::Plus,
                        '@' => TokenKind::At,
                        _ => TokenKind::Identifier,
                    }
                } else if text == "->" {
                    TokenKind::InstanceArrow
                } else if text == "=>" {
                    TokenKind::StaticArrow
                } else if self.mode == Mode::Normal
                    && first == '\\'
                    && !matches!(prev, SPACE | NL | TAB | COLON)
                {
                    TokenKind::AssociationName
                } else {
                    TokenKind::Identifier
                }
            }
        };

        self.tokens.push(Token {
            kind,
            text,
            start: self.position(s),
            span: self.byte_of[s]..self.byte_of[e],
            ws_before,
            ws_after,
            expanded: false,
        });
        self.buf_clear();
    }

    fn run(mut self) -> Vec<Token> {
        loop {
            let current = self.current();
            self.buf_add(self.offset);
            let ahead = self.ch(self.offset + 1);
            let aahead = self.ch(self.offset + 2);

            if self.mode == Mode::Normal {
                if is_split(ahead) {
                    self.add();
                } else if ahead == QUOTE {
                    self.add();
                    self.mode = Mode::Str;
                } else if ahead == PIPE || ahead == RBRACE {
                    self.add();
                    self.mode = Mode::Template;
                } else if ahead == BACKTICK {
                    self.add();
                    self.mode = Mode::Ping;
                } else if ahead == HASH && aahead == HASH {
                    self.add();
                    self.mode = Mode::Pragma;
                } else if ahead == DQUOTE || (ahead == STAR && current == NL) {
                    self.add();
                    self.mode = Mode::Comment;
                } else if (ahead == AT && self.buf_is_blank())
                    // "->" and "=>" start next
                    || ((ahead == DASH || ahead == EQUALS) && aahead == GT)
                    // "->" and "=>" end here
                    || (current == GT
                        && ahead != SPACE
                        && matches!(self.ch(self.offset - 1), DASH | EQUALS))
                    // single character tokens
                    || (self.buf_len() == 1
                        && (is_buf(current) || (current == DASH && ahead != GT)))
                {
                    self.add();
                }
            } else if self.mode == Mode::Pragma && matches!(ahead, COMMA | COLON | DOT | SPACE | NL)
            {
                self.add();
                self.mode = Mode::Normal;
            } else if self.mode == Mode::Ping
                && self.buf_len() > 1
                && current == BACKTICK
                && ahead != BACKTICK
                && self.buf_count_is_even(BACKTICK)
            {
                // end of ping literal; without a separator after it the
                // following characters stay glued to the token
                if is_after_literal(ahead) {
                    self.add();
                }
                self.mode = if ahead == DQUOTE {
                    Mode::Comment
                } else {
                    Mode::Normal
                };
            } else if self.mode == Mode::Template
                && self.buf_len() > 1
                && (current == PIPE || current == LBRACE)
                && self.count_previous(BACKSLASH).is_multiple_of(2)
            {
                // end of template part
                if current == LBRACE || is_after_literal(ahead) {
                    self.add();
                }
                self.mode = Mode::Normal;
            } else if self.mode == Mode::Template && ahead == RBRACE && current != BACKSLASH {
                self.add();
            } else if self.mode == Mode::Str
                && current == QUOTE
                && self.buf_len() > 1
                && ahead != QUOTE
                && self.buf_count_is_even(QUOTE)
            {
                // end of string; anything but a separator continues the token
                if is_after_literal(ahead) {
                    self.add();
                }
                self.mode = if ahead == DQUOTE {
                    Mode::Comment
                } else {
                    Mode::Normal
                };
            } else if ahead == NL && self.mode != Mode::Template {
                self.add();
                self.mode = Mode::Normal;
            } else if self.mode == Mode::Template && current == NL {
                self.add();
            }

            if !self.advance() {
                break;
            }
        }
        self.add();
        self.tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn texts(src: &str) -> Vec<String> {
        lex(src).tokens.into_iter().map(|t| t.text).collect()
    }

    fn names(src: &str) -> Vec<String> {
        lex(src).tokens.iter().map(|t| t.abaplint_name()).collect()
    }

    #[test]
    fn simple_statement() {
        assert_eq!(texts("DATA foo TYPE i."), ["DATA", "foo", "TYPE", "i", "."]);
    }

    #[test]
    fn chained_statement() {
        assert_eq!(
            texts("DATA: a TYPE i, b TYPE c."),
            ["DATA", ":", "a", "TYPE", "i", ",", "b", "TYPE", "c", "."]
        );
    }

    #[test]
    fn comments() {
        let l = lex("* full line\nWRITE 'a'. \" trailing\n");
        let kinds: Vec<_> = l.tokens.iter().map(|t| t.kind).collect();
        assert_eq!(
            kinds,
            [
                TokenKind::Comment,
                TokenKind::Identifier,
                TokenKind::String,
                TokenKind::Punctuation,
                TokenKind::Comment
            ]
        );
        assert_eq!(l.tokens[0].text, "* full line");
        assert_eq!(l.tokens[4].text, "\" trailing");
    }

    #[test]
    fn star_not_in_first_column_is_operator() {
        assert_eq!(texts("a = b * c."), ["a", "=", "b", "*", "c", "."]);
    }

    #[test]
    fn strings_with_escapes() {
        assert_eq!(texts("WRITE 'it''s'."), ["WRITE", "'it''s'", "."]);
        assert_eq!(texts("x = `a``b`."), ["x", "=", "`a``b`", "."]);
        assert_eq!(texts("WRITE 'a.b:c'."), ["WRITE", "'a.b:c'", "."]);
    }

    #[test]
    fn string_templates() {
        let l = lex("x = |a{ b }c{ d }e|.");
        let kinds: Vec<_> = l.tokens.iter().map(|t| t.kind).collect();
        assert_eq!(
            kinds,
            [
                TokenKind::Identifier,
                TokenKind::Identifier,
                TokenKind::StringTemplateBegin,
                TokenKind::Identifier,
                TokenKind::StringTemplateMiddle,
                TokenKind::Identifier,
                TokenKind::StringTemplateEnd,
                TokenKind::Punctuation
            ]
        );
        assert_eq!(texts("x = |plain|."), ["x", "=", "|plain|", "."]);
    }

    #[test]
    fn pragmas() {
        let l = lex("DATA a TYPE c ##NEEDED.");
        assert_eq!(l.tokens[4].kind, TokenKind::Pragma);
        assert_eq!(l.tokens[4].text, "##NEEDED");
    }

    #[test]
    fn whitespace_sensitive_tokens() {
        assert_eq!(
            names("foo( ).")[1..3],
            ["ParenLeftW".to_string(), "WParenRightW".to_string()]
        );
        assert_eq!(names("x = a-b.")[3], "Dash");
        assert_eq!(names("x = a - b.")[3], "WDashW");
        assert_eq!(
            texts("lo_obj->meth( )."),
            ["lo_obj", "->", "meth", "(", ")", "."]
        );
        assert_eq!(
            texts("zcl_x=>meth( )."),
            ["zcl_x", "=>", "meth", "(", ")", "."]
        );
        assert_eq!(names("zcl_x=>meth( ).")[1], "StaticArrow");
    }

    #[test]
    fn field_symbols_and_host_variables() {
        assert_eq!(
            texts("ASSIGN <fs> TO <bar>."),
            ["ASSIGN", "<fs>", "TO", "<bar>", "."]
        );
        assert_eq!(
            texts("SELECT * FROM t INTO @DATA(r)."),
            ["SELECT", "*", "FROM", "t", "INTO", "@", "DATA", "(", "r", ")", "."]
        );
    }

    #[test]
    fn positions_and_spans() {
        let l = lex("REPORT z.\r\n  WRITE 'ä'.");
        let w = &l.tokens[3];
        assert_eq!(w.text, "WRITE");
        assert_eq!(w.start, Position { row: 2, col: 3 });
        let s = &l.tokens[4];
        assert_eq!(&l.source[s.span.clone()], "'ä'");
        assert_eq!(s.start, Position { row: 2, col: 9 });
    }
}
