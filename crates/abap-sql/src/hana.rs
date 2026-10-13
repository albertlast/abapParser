//! HANA SQL and SQLScript: the native SQL of EXEC SQL blocks and AMDP
//! methods.
//!
//! Lenient like the rest of the parser: SQLScript blocks (IF, WHILE, FOR,
//! BEGIN ... END) are followed to find the statements inside; queries and
//! data changes are taken apart into tables, fields, joins and conditions;
//! what is not understood is skipped. Nothing fails.
//!
//! Grammar reference: the SAP HANA SQL Reference Guide and the SQLScript
//! Reference, used as a reference only.

use std::collections::HashMap;
use std::ops::Range;

use abap_model as m;
use abap_syntax::StatementKind as K;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dialect {
    /// EXEC SQL ... ENDEXEC: ABAP comments with `*` and `"`, host
    /// variables like `:wa-field`
    ExecSql,
    /// the body of an AMDP method: SQLScript, `"` quotes names, `*` in the
    /// first column starts a comment line
    SqlScript,
}

/// A database access with the byte range of its statement in the text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeAccess {
    pub access: m::DbAccess,
    pub range: Range<usize>,
}

/// A call of a procedure, or of a table function in FROM.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeCall {
    /// as written without quotes, e.g. `ZCL_X=>GET_ROWS`, `SYS.TRUNCATE_X`
    pub name: String,
    /// a table function in FROM, not CALL
    pub function: bool,
    pub range: Range<usize>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Native {
    pub accesses: Vec<NativeAccess>,
    pub calls: Vec<NativeCall>,
}

/// Parses native SQL.
pub fn parse(text: &str, dialect: Dialect) -> Native {
    let toks = lex(text, dialect);
    let mut p = P {
        t: &toks,
        pos: 0,
        end: toks.len(),
        text,
        dialect,
    };
    let mut out = Native::default();
    p.block(&mut out, &[]);
    out
}

// ----- tokens ----------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    /// unquoted name or keyword
    Word,
    /// `"Name"`, the text without quotes
    Quoted,
    /// `'text'` as written
    Str,
    Num,
    /// `:name`, the text without colon
    Var,
    Sym,
}

#[derive(Debug, Clone)]
struct Tok {
    kind: Kind,
    text: String,
    start: usize,
    end: usize,
}

impl Tok {
    fn is(&self, word: &str) -> bool {
        self.kind == Kind::Word && self.text.eq_ignore_ascii_case(word)
    }
    fn sym(&self, s: &str) -> bool {
        self.kind == Kind::Sym && self.text == s
    }
    fn upper(&self) -> String {
        self.text.to_ascii_uppercase()
    }
    fn is_name(&self) -> bool {
        matches!(self.kind, Kind::Word | Kind::Quoted)
    }
}

fn lex(text: &str, dialect: Dialect) -> Vec<Tok> {
    let chars: Vec<(usize, char)> = text.char_indices().collect();
    let at = |i: usize| chars.get(i).map(|c| c.1);
    let offset = |i: usize| chars.get(i).map(|c| c.0).unwrap_or(text.len());
    let is_name = |c: char| c.is_alphanumeric() || c == '_' || c == '#' || c == '$';
    let mut out = Vec::new();
    let mut i = 0;
    // the text starts at a line start or after an ABAP statement, where no
    // SQL statement starts with `*`
    let mut line_start = true;
    let skip_line = |mut i: usize| {
        while at(i).is_some_and(|c| c != '\n') {
            i += 1;
        }
        i
    };
    while let Some(c) = at(i) {
        if c == '\n' {
            line_start = true;
            i += 1;
            continue;
        }
        if line_start && c == '*' {
            i = skip_line(i);
            continue;
        }
        line_start = false;
        if c.is_whitespace() {
            i += 1;
            continue;
        }
        let start = i;
        let push = |out: &mut Vec<Tok>, kind, text: String, from: usize, to: usize| {
            out.push(Tok {
                kind,
                text,
                start: offset(from),
                end: offset(to),
            })
        };
        match c {
            '-' if at(i + 1) == Some('-') => i = skip_line(i),
            '/' if at(i + 1) == Some('*') => {
                i += 2;
                while at(i).is_some() && !(at(i) == Some('*') && at(i + 1) == Some('/')) {
                    i += 1;
                }
                i += 2;
            }
            '"' if dialect == Dialect::ExecSql => i = skip_line(i),
            '"' | '\'' => {
                let mut value = String::new();
                i += 1;
                while let Some(d) = at(i) {
                    if d == c {
                        if at(i + 1) == Some(c) {
                            value.push(c);
                            i += 2;
                            continue;
                        }
                        i += 1;
                        break;
                    }
                    value.push(d);
                    i += 1;
                }
                if c == '"' {
                    push(&mut out, Kind::Quoted, value, start, i);
                } else {
                    let written = text[offset(start)..offset(i)].to_string();
                    push(&mut out, Kind::Str, written, start, i);
                }
            }
            ':' if at(i + 1) == Some('=') => {
                i += 2;
                push(&mut out, Kind::Sym, ":=".into(), start, i);
            }
            ':' if at(i + 1).is_some_and(|d| is_name(d) || d == '<' || d == '"') => {
                i += 1;
                let name_start = i;
                if at(i) == Some('"') {
                    i += 1;
                    while at(i).is_some_and(|d| d != '"') {
                        i += 1;
                    }
                    i += 1;
                } else {
                    while let Some(d) = at(i) {
                        let abap = dialect == Dialect::ExecSql
                            && (d == '<'
                                || d == '>'
                                || d == '~'
                                || (d == '-' && at(i + 1).is_some_and(is_name)));
                        if is_name(d) || abap {
                            i += 1;
                        } else {
                            break;
                        }
                    }
                }
                let name = text[offset(name_start)..offset(i)]
                    .trim_matches('"')
                    .to_string();
                push(&mut out, Kind::Var, name, start, i);
            }
            c if c.is_ascii_digit() => {
                while at(i).is_some_and(|d| d.is_ascii_digit()) {
                    i += 1;
                }
                if at(i) == Some('.') && at(i + 1).is_some_and(|d| d.is_ascii_digit()) {
                    i += 1;
                    while at(i).is_some_and(|d| d.is_ascii_digit()) {
                        i += 1;
                    }
                }
                if matches!(at(i), Some('e' | 'E'))
                    && at(i + 1).is_some_and(|d| d.is_ascii_digit() || d == '-' || d == '+')
                {
                    i += 2;
                    while at(i).is_some_and(|d| d.is_ascii_digit()) {
                        i += 1;
                    }
                }
                let t = text[offset(start)..offset(i)].to_string();
                push(&mut out, Kind::Num, t, start, i);
            }
            c if is_name(c) => {
                while at(i).is_some_and(is_name) {
                    i += 1;
                }
                let t = text[offset(start)..offset(i)].to_string();
                push(&mut out, Kind::Word, t, start, i);
            }
            _ => {
                let two: String = [Some(c), at(i + 1)].iter().flatten().collect();
                let len = if matches!(two.as_str(), "<=" | ">=" | "<>" | "!=" | "||" | "=>" | "..")
                {
                    2
                } else {
                    1
                };
                i += len;
                let t = text[offset(start)..offset(i)].to_string();
                push(&mut out, Kind::Sym, t, start, i);
            }
        }
    }
    out
}

// ----- parser ----------------------------------------------------------------

/// Keywords that end a clause of a query.
const CLAUSES: &[&str] = &[
    "FROM",
    "WHERE",
    "GROUP",
    "HAVING",
    "ORDER",
    "LIMIT",
    "OFFSET",
    "UNION",
    "INTERSECT",
    "EXCEPT",
    "MINUS",
    "INTO",
    "WITH",
    "FOR",
];

const JOIN_WORDS: &[&str] = &["INNER", "LEFT", "RIGHT", "FULL", "CROSS", "JOIN", "ON"];

/// Words after a table that are no alias.
const NO_ALIAS: &[&str] = &[
    "SET", "USING", "VALUES", "WHEN", "OUTER", "SELECT", "START", "MANY", "ONE", "EXACT",
];

/// Words in expressions that are no column names.
const EXPRESSION_WORDS: &[&str] = &[
    "CASE",
    "WHEN",
    "THEN",
    "ELSE",
    "END",
    "AND",
    "OR",
    "NOT",
    "NULL",
    "IS",
    "IN",
    "AS",
    "DISTINCT",
    "ALL",
    "TRUE",
    "FALSE",
    "BETWEEN",
    "LIKE",
    "ESCAPE",
    "OVER",
    "PARTITION",
    "BY",
    "ORDER",
    "ASC",
    "DESC",
    "NULLS",
    "FIRST",
    "LAST",
    "ROWS",
    "RANGE",
    "UNBOUNDED",
    "PRECEDING",
    "FOLLOWING",
    "CURRENT",
    "ROW",
    "INTERVAL",
    "DATE",
    "TIME",
    "TIMESTAMP",
    "EXISTS",
    "ANY",
    "SOME",
    "LIKE_REGEXPR",
    "FLAG",
    "WITHIN",
    "GROUP",
    "SECOND",
    "MINUTE",
    "HOUR",
    "DAY",
    "MONTH",
    "YEAR",
    "LEADING",
    "TRAILING",
    "BOTH",
    "FROM",
    "FOR",
    "CURRENT_DATE",
    "CURRENT_TIME",
    "CURRENT_TIMESTAMP",
    "CURRENT_UTCTIMESTAMP",
    "CURRENT_USER",
    "SESSION_USER",
];

const AGGREGATES: &[&str] = &[
    "COUNT",
    "SUM",
    "MIN",
    "MAX",
    "AVG",
    "STRING_AGG",
    "STDDEV",
    "VAR",
    "MEDIAN",
    "FIRST_VALUE",
    "LAST_VALUE",
    "NTH_VALUE",
    "CORR",
    "STDDEV_POP",
    "STDDEV_SAMP",
    "VAR_POP",
    "VAR_SAMP",
    "ARRAY_AGG",
];

/// Built-in table functions, which read no table by themselves.
fn is_builtin_table_function(name: &str) -> bool {
    let up = name.to_ascii_uppercase();
    up.starts_with("SERIES_GENERATE")
        || matches!(
            up.as_str(),
            "UNNEST"
                | "JSON_TABLE"
                | "XMLTABLE"
                | "MAP_MERGE"
                | "MAP_REDUCE"
                | "SERIES_DISAGGREGATE"
        )
}

struct P<'a> {
    t: &'a [Tok],
    pos: usize,
    end: usize,
    text: &'a str,
    dialect: Dialect,
}

/// Tables seen in a statement, to resolve the qualifiers of columns.
#[derive(Default)]
struct Scope {
    /// upper case alias or name -> table name
    names: HashMap<String, String>,
}

impl<'a> P<'a> {
    fn sub(&self, range: Range<usize>) -> P<'a> {
        P {
            t: self.t,
            pos: range.start,
            end: range.end,
            text: self.text,
            dialect: self.dialect,
        }
    }

    fn done(&self) -> bool {
        self.pos >= self.end
    }

    fn peek(&self, k: usize) -> Option<&'a Tok> {
        let i = self.pos + k;
        (i < self.end).then(|| &self.t[i])
    }

    fn at(&self, word: &str) -> bool {
        self.peek(0).is_some_and(|t| t.is(word))
    }

    fn at_sym(&self, s: &str) -> bool {
        self.peek(0).is_some_and(|t| t.sym(s))
    }

    fn at_seq(&self, words: &str) -> bool {
        words
            .split(' ')
            .enumerate()
            .all(|(i, w)| self.peek(i).is_some_and(|t| t.is(w)))
    }

    fn eat(&mut self, word: &str) -> bool {
        let found = self.at(word);
        if found {
            self.pos += 1;
        }
        found
    }

    fn eat_seq(&mut self, words: &str) -> bool {
        let found = self.at_seq(words);
        if found {
            self.pos += words.split(' ').count();
        }
        found
    }

    fn word(&self) -> String {
        self.peek(0)
            .filter(|t| t.kind == Kind::Word)
            .map(Tok::upper)
            .unwrap_or_default()
    }

    /// Text of tokens `range`, as written.
    fn text_of(&self, range: Range<usize>) -> String {
        if range.is_empty() {
            return String::new();
        }
        self.text[self.t[range.start].start..self.t[range.end - 1].end].to_string()
    }

    /// The position after the parenthesis closing the one at `open`.
    fn close_of(&self, open: usize) -> usize {
        let mut depth = 0usize;
        let mut i = open;
        while i < self.end {
            let t = &self.t[i];
            if t.sym("(") {
                depth += 1;
            } else if t.sym(")") {
                depth -= 1;
                if depth == 0 {
                    return i + 1;
                }
            }
            i += 1;
        }
        self.end
    }

    /// The position of the first token at depth 0 from the cursor on that
    /// `stop` accepts, or the end.
    fn find(&self, stop: impl Fn(&Tok) -> bool) -> usize {
        let mut i = self.pos;
        while i < self.end {
            let t = &self.t[i];
            if t.sym("(") {
                i = self.close_of(i);
                continue;
            }
            if stop(t) {
                return i;
            }
            i += 1;
        }
        self.end
    }

    // ----- SQLScript blocks --------------------------------------------------

    /// Statements up to one of the end keywords, which is not consumed.
    fn block(&mut self, out: &mut Native, ends: &[&str]) {
        loop {
            while self.at_sym(";") {
                self.pos += 1;
            }
            if self.done() || ends.iter().any(|e| self.at_seq(e)) {
                return;
            }
            let before = self.pos;
            self.statement(out);
            if self.pos == before {
                self.pos += 1;
            }
        }
    }

    /// Skips to the keyword at depth 0, scanning what is skipped for queries.
    fn condition_until(&mut self, out: &mut Native, word: &str) {
        let end = self.find(|t| t.is(word));
        let mut acc = super::empty_access(K::NativeSQL);
        let mut scope = Scope::default();
        let mut sub = self.sub(self.pos..end);
        sub.scan(&mut acc, &mut scope, out);
        self.push_if_tables(out, acc, self.pos..end);
        self.pos = end;
        self.eat(word);
    }

    fn statement(&mut self, out: &mut Native) {
        match self.word().as_str() {
            "IF" => {
                self.pos += 1;
                self.condition_until(out, "THEN");
                self.block(out, &["ELSEIF", "ELSE", "END IF"]);
                loop {
                    if self.eat("ELSEIF") {
                        self.condition_until(out, "THEN");
                        self.block(out, &["ELSEIF", "ELSE", "END IF"]);
                    } else if self.eat("ELSE") {
                        self.block(out, &["END IF"]);
                    } else {
                        break;
                    }
                }
                self.eat_seq("END IF");
            }
            "WHILE" => {
                self.pos += 1;
                self.condition_until(out, "DO");
                self.block(out, &["END WHILE"]);
                self.eat_seq("END WHILE");
            }
            "FOR" => {
                self.pos += 1;
                self.condition_until(out, "DO");
                self.block(out, &["END FOR"]);
                self.eat_seq("END FOR");
            }
            "BEGIN" => {
                self.pos += 1;
                for addition in [
                    "AUTONOMOUS TRANSACTION",
                    "PARALLEL EXECUTION",
                    "SEQUENTIAL EXECUTION",
                ] {
                    self.eat_seq(addition);
                }
                self.block(out, &["END"]);
                self.eat("END");
            }
            "DECLARE" if self.at_seq("DECLARE EXIT HANDLER FOR") => {
                // the handled conditions, then one statement
                self.pos += 4;
                while self.at("SQLEXCEPTION")
                    || self.at("SQL_ERROR_CODE")
                    || self.at_sym(",")
                    || self.peek(0).is_some_and(|t| t.kind == Kind::Num)
                {
                    self.pos += 1;
                }
                self.statement(out);
            }
            _ => self.simple(out),
        }
    }

    /// A statement up to `;`, or up to a keyword ending its block.
    fn simple(&mut self, out: &mut Native) {
        let start = self.pos;
        let mut case = 0usize;
        let mut i = start;
        while i < self.end {
            let t = &self.t[i];
            if t.sym("(") {
                i = self.close_of(i);
                continue;
            }
            if t.sym(";") {
                break;
            }
            if t.is("CASE") {
                case += 1;
            } else if t.is("END") {
                let block_end = self
                    .t
                    .get(i + 1)
                    .is_some_and(|n| ["IF", "FOR", "WHILE"].iter().any(|w| n.is(w)));
                if case > 0 && !block_end {
                    case -= 1;
                } else {
                    break;
                }
            } else if case == 0 && (t.is("ELSE") || t.is("ELSEIF")) {
                break;
            }
            i += 1;
        }
        self.pos = i;
        if i > start {
            self.analyze(start..i, out);
        }
    }

    fn push_if_tables(&self, out: &mut Native, acc: m::DbAccess, range: Range<usize>) {
        if !acc.tables.is_empty() {
            self.push(out, acc, range);
        }
    }

    fn push(&self, out: &mut Native, mut acc: m::DbAccess, range: Range<usize>) {
        if range.is_empty() {
            return;
        }
        let text = self.text_of(range.clone());
        acc.native_sql = Some(text.clone());
        acc.text = text;
        acc.host_variables.dedup();
        let mut seen = std::collections::HashSet::new();
        acc.host_variables
            .retain(|v| seen.insert(v.to_ascii_lowercase()));
        out.accesses.push(NativeAccess {
            access: acc,
            range: self.t[range.start].start..self.t[range.end - 1].end,
        });
    }

    // ----- statements --------------------------------------------------------

    fn analyze(&self, range: Range<usize>, out: &mut Native) {
        let mut p = self.sub(range.clone());
        let mut acc = super::empty_access(K::NativeSQL);
        let mut scope = Scope::default();
        let first = p.word();
        match first.as_str() {
            "SELECT" | "WITH" => {
                p.query(&mut acc, &mut scope, out, m::DbTableRole::Main, true);
            }
            "INSERT" => {
                acc.operation = m::DbOperation::Insert;
                p.pos += 1;
                p.eat("INTO");
                p.changed_table(&mut acc, &mut scope, out);
                p.insert_source(&mut acc, &mut scope, out);
            }
            "UPSERT" | "REPLACE" => {
                acc.operation = m::DbOperation::Modify;
                p.pos += 1;
                p.changed_table(&mut acc, &mut scope, out);
                p.insert_source(&mut acc, &mut scope, out);
                if p.eat("WHERE") {
                    p.where_clause(&mut acc, &mut scope, out, true);
                }
            }
            "UPDATE" => {
                acc.operation = m::DbOperation::Update;
                p.pos += 1;
                p.changed_table(&mut acc, &mut scope, out);
                if p.eat("SET") {
                    p.set_list(&mut acc, &mut scope, out);
                }
                if p.eat("FROM") {
                    p.from(&mut acc, &mut scope, out, m::DbTableRole::Join);
                }
                if p.eat("WHERE") {
                    p.where_clause(&mut acc, &mut scope, out, true);
                }
            }
            "DELETE" => {
                acc.operation = m::DbOperation::Delete;
                p.pos += 1;
                p.eat("HISTORY");
                p.eat("FROM");
                p.changed_table(&mut acc, &mut scope, out);
                if p.eat("WHERE") {
                    p.where_clause(&mut acc, &mut scope, out, true);
                }
            }
            "TRUNCATE" => {
                acc.operation = m::DbOperation::Delete;
                p.pos += 1;
                p.eat("TABLE");
                p.changed_table(&mut acc, &mut scope, out);
            }
            "MERGE" => {
                acc.operation = m::DbOperation::Merge;
                p.pos += 1;
                p.eat("INTO");
                p.changed_table(&mut acc, &mut scope, out);
                if p.eat("USING") {
                    p.table_ref(&mut acc, &mut scope, out, m::DbTableRole::MergeSource);
                }
                if p.eat("ON") {
                    let end = p.find(|t| t.is("WHEN"));
                    let conds = p.sub(p.pos..end).conditions(&mut acc, &mut scope, out);
                    acc.conditions.extend(conds);
                    p.pos = end;
                }
                while !p.done() {
                    if p.eat("SET") {
                        p.set_list(&mut acc, &mut scope, out);
                    } else {
                        let before = p.pos;
                        p.scan_one(&mut acc, &mut scope, out);
                        if p.pos == before {
                            p.pos += 1;
                        }
                    }
                }
            }
            "CALL" => {
                acc.operation = m::DbOperation::Native;
                p.pos += 1;
                let name_start = p.pos;
                let name = p.dotted_name();
                if let Some(name) = name {
                    out.calls.push(NativeCall {
                        name,
                        function: false,
                        range: self.t[name_start].start..self.t[p.pos - 1].end,
                    });
                }
                p.scan(&mut acc, &mut scope, out);
            }
            "OPEN" => {
                // EXEC SQL: OPEN c FOR SELECT ...
                p.pos += 1;
                acc.cursor = p.peek(0).map(|t| t.text.clone());
                let at = p.find(|t| t.is("FOR"));
                p.pos = (at + 1).min(p.end);
                p.query(&mut acc, &mut scope, out, m::DbTableRole::Main, true);
            }
            "DECLARE" => {
                // DECLARE CURSOR c FOR SELECT, DECLARE x ... = (SELECT ...)
                let cursor = (0..4).any(|k| p.peek(k).is_some_and(|t| t.is("CURSOR")));
                if cursor {
                    let at = p.find(|t| t.is("FOR"));
                    acc.cursor = (at > p.pos).then(|| p.t[at - 1].text.clone());
                    p.pos = (at + 1).min(p.end);
                    p.query(&mut acc, &mut scope, out, m::DbTableRole::Main, true);
                } else {
                    p.scan(&mut acc, &mut scope, out);
                    self.push_if_tables(out, acc, range);
                    return;
                }
            }
            "RETURN" => {
                p.pos += 1;
                if p.query_ahead() {
                    p.query(&mut acc, &mut scope, out, m::DbTableRole::Main, true);
                    acc.target = Some(m::DbTarget {
                        targets: vec!["RETURN".into()],
                        kind: m::DbTargetKind::Table,
                        appending: false,
                        corresponding: false,
                    });
                } else {
                    p.scan(&mut acc, &mut scope, out);
                    self.push_if_tables(out, acc, range);
                    return;
                }
            }
            "FETCH" | "CLOSE" | "COMMIT" | "ROLLBACK" | "BREAK" | "CONTINUE" | "SIGNAL"
            | "RESIGNAL" | "CONNECT" | "DISCONNECT" | "SET" => return,
            "CREATE" | "DROP" | "ALTER" | "GRANT" | "REVOKE" | "LOCK" | "EXEC" | "EXECUTE"
            | "RENAME" | "COMMENT" | "LOAD" | "UNLOAD" | "IMPORT" | "EXPORT" => {
                acc.operation = m::DbOperation::Native;
                p.scan(&mut acc, &mut scope, out);
            }
            _ => {
                // `lt_rows = SELECT ...`, `lv_x := ...`, `:lt.INSERT( ... )`
                let assign = p.find(|t| t.sym("=") || t.sym(":="));
                let target = p.t[p.pos..assign]
                    .iter()
                    .find(|t| t.is_name())
                    .map(|t| t.text.clone());
                if assign < p.end && p.pos < assign && target.is_some() {
                    p.pos = assign + 1;
                    if p.query_ahead() {
                        p.query(&mut acc, &mut scope, out, m::DbTableRole::Main, true);
                        acc.target = Some(m::DbTarget {
                            targets: target.into_iter().collect(),
                            kind: m::DbTargetKind::Table,
                            appending: false,
                            corresponding: false,
                        });
                        self.push(out, acc, range);
                        return;
                    }
                }
                p.pos = range.start;
                p.scan(&mut acc, &mut scope, out);
                if !acc.tables.is_empty() {
                    if let Some(t) = target.filter(|_| assign < p.end) {
                        acc.target = Some(m::DbTarget {
                            targets: vec![t],
                            kind: m::DbTargetKind::Table,
                            appending: false,
                            corresponding: false,
                        });
                    }
                }
                self.push_if_tables(out, acc, range);
                return;
            }
        }
        self.push(out, acc, range);
    }

    /// `SELECT` or `WITH` at the cursor, possibly in parentheses.
    fn query_ahead(&self) -> bool {
        let mut k = 0;
        while self.peek(k).is_some_and(|t| t.sym("(")) {
            k += 1;
        }
        self.peek(k).is_some_and(|t| t.is("SELECT") || t.is("WITH"))
    }

    /// `name`, `schema.name`, `"ZCL=>METH"`; None at anything else.
    fn dotted_name(&mut self) -> Option<String> {
        let mut parts = Vec::new();
        while let Some(t) = self.peek(0) {
            if !t.is_name() {
                break;
            }
            parts.push(t.text.clone());
            self.pos += 1;
            if self.at_sym(".") && self.peek(1).is_some_and(Tok::is_name) {
                self.pos += 1;
            } else {
                break;
            }
        }
        (!parts.is_empty()).then(|| parts.join("."))
    }

    /// The table changed by INSERT, UPDATE, DELETE, UPSERT, MERGE; a
    /// parenthesis after it is a column list.
    fn changed_table(&mut self, acc: &mut m::DbAccess, scope: &mut Scope, out: &mut Native) {
        self.table_or_columns(acc, scope, out, m::DbTableRole::Main, false);
    }

    /// `[(columns)] VALUES (...)`, `[(columns)] SELECT ...`, `WITH PRIMARY KEY`.
    fn insert_source(&mut self, acc: &mut m::DbAccess, scope: &mut Scope, out: &mut Native) {
        if self.at_sym("(") && !self.query_ahead() {
            self.pos = self.close_of(self.pos);
        }
        if self.at("VALUES") {
            let start = self.pos;
            self.pos += 1;
            let end = if self.at_sym("(") {
                self.close_of(self.pos)
            } else {
                self.find(|t| t.is("WHERE") || t.is("WITH"))
            };
            self.sub(self.pos..end).scan(acc, scope, out);
            acc.source = Some(m::DbSource {
                text: Some(self.text_of(start..end)),
                is_table: false,
                is_subquery: false,
            });
            self.pos = end;
        } else if self.query_ahead() {
            let mut sub = self.sub(self.pos..self.end);
            sub.query(acc, scope, out, m::DbTableRole::Subquery, false);
            self.pos = sub.pos;
            acc.source = Some(m::DbSource {
                text: None,
                is_table: false,
                is_subquery: true,
            });
        }
        self.eat_seq("WITH PRIMARY KEY");
    }

    /// `col = expr, ...` of UPDATE ... SET and MERGE.
    fn set_list(&mut self, acc: &mut m::DbAccess, scope: &mut Scope, out: &mut Native) {
        let end = self.find(|t| t.is("FROM") || t.is("WHERE") || t.is("WHEN"));
        for item in self.split(self.pos..end) {
            let mut q = self.sub(item.clone());
            let eq = q.find(|t| t.sym("="));
            if eq >= item.end {
                continue;
            }
            let column = column_of(&self.t[item.start..eq], scope);
            let value = self.text_of(eq + 1..item.end);
            q.pos = eq + 1;
            q.scan(acc, scope, out);
            if let Some(column) = column {
                acc.set.push(m::DbAssignment { column, value });
            }
        }
        self.pos = end;
    }

    /// Ranges of the comma separated parts of `range` at depth 0.
    fn split(&self, range: Range<usize>) -> Vec<Range<usize>> {
        let mut out = Vec::new();
        let mut q = self.sub(range.clone());
        let mut start = range.start;
        loop {
            let comma = q.find(|t| t.sym(","));
            if comma > start {
                out.push(start..comma);
            }
            if comma >= range.end {
                break;
            }
            start = comma + 1;
            q.pos = start;
        }
        out
    }

    // ----- queries -----------------------------------------------------------

    /// A query at the cursor. `top`: the clauses describe the statement, else
    /// only its tables and variables are taken.
    fn query(
        &mut self,
        acc: &mut m::DbAccess,
        scope: &mut Scope,
        out: &mut Native,
        role: m::DbTableRole,
        top: bool,
    ) {
        if self.at_sym("(") {
            let close = self.close_of(self.pos);
            let mut inner = self.sub(self.pos + 1..close - 1);
            inner.query(acc, scope, out, role, top);
            self.pos = close;
            self.query_rest(acc, scope, out, top);
            return;
        }
        if self.eat("WITH") {
            while let Some(name) = self.peek(0).filter(|t| t.is_name()) {
                self.pos += 1;
                scope.names.insert(name.upper(), name.text.clone());
                acc.tables.push(table(&name.text, m::DbTableRole::Cte));
                if self.at_sym("(") {
                    self.pos = self.close_of(self.pos);
                }
                self.eat("AS");
                if self.at_sym("(") {
                    let close = self.close_of(self.pos);
                    let mut inner = self.sub(self.pos + 1..close - 1);
                    inner.query(acc, scope, out, m::DbTableRole::Subquery, false);
                    self.pos = close;
                }
                if !self.at_sym(",") {
                    break;
                }
                self.pos += 1;
            }
        }
        if !self.eat("SELECT") {
            self.scan(acc, scope, out);
            return;
        }
        if self.eat("TOP") {
            if top {
                acc.up_to = self.peek(0).map(|t| t.text.clone());
            }
            self.pos += 1;
        }
        if self.eat("DISTINCT") {
            acc.distinct |= top;
        }
        self.eat("ALL");

        // the select list, then INTO of SQLScript and EXEC SQL
        let list_end = self.find(|t| t.is("FROM") || t.is("INTO"));
        let list = self.pos..list_end;
        self.pos = list_end;
        if self.eat("INTO") {
            let end = self.find(|t| t.is("FROM"));
            if top {
                let targets: Vec<String> = self
                    .split(self.pos..end)
                    .into_iter()
                    .map(|r| {
                        self.t[r]
                            .iter()
                            .map(|t| t.text.as_str())
                            .collect::<String>()
                    })
                    .collect();
                acc.target = Some(m::DbTarget {
                    kind: if targets.len() > 1 {
                        m::DbTargetKind::List
                    } else {
                        m::DbTargetKind::Structure
                    },
                    targets,
                    appending: false,
                    corresponding: false,
                });
            }
            self.pos = end;
        }
        if self.eat("FROM") {
            self.from(acc, scope, out, role);
        }
        // columns are resolved once the tables are known
        let mut fields = Vec::new();
        for item in self.split(list) {
            if top {
                fields.push(field(self, item.clone(), scope));
            }
            self.sub(item).scan_variables(acc, scope, out);
        }
        if top {
            acc.fields = fields;
        }
        self.query_rest(acc, scope, out, top);
    }

    /// WHERE, GROUP BY, HAVING, set operations, ORDER BY, LIMIT.
    fn query_rest(
        &mut self,
        acc: &mut m::DbAccess,
        scope: &mut Scope,
        out: &mut Native,
        top: bool,
    ) {
        while !self.done() {
            let before = self.pos;
            if self.eat("WHERE") {
                self.where_clause(acc, scope, out, top);
            } else if self.eat_seq("GROUP BY") {
                let end = self.clause_end();
                for item in self.split(self.pos..end) {
                    if top {
                        if let Some(c) = column_of(&self.t[item.clone()], scope) {
                            acc.group_by.push(c);
                        }
                    }
                    self.sub(item).scan_variables(acc, scope, out);
                }
                self.pos = end;
            } else if self.eat("HAVING") {
                let end = self.clause_end();
                let conds = self.sub(self.pos..end).conditions(acc, scope, out);
                if top {
                    acc.having = conds;
                }
                self.pos = end;
            } else if ["UNION", "INTERSECT", "EXCEPT", "MINUS"]
                .iter()
                .any(|w| self.at(w))
            {
                acc.set_operation = true;
                self.pos += 1;
                self.eat("ALL");
                self.eat("DISTINCT");
                self.query(acc, scope, out, m::DbTableRole::Main, false);
            } else if self.eat_seq("ORDER BY") {
                let end = self.clause_end();
                if top {
                    acc.order_by = self
                        .split(self.pos..end)
                        .into_iter()
                        .map(|r| self.text_of(r))
                        .collect();
                }
                self.pos = end;
            } else if self.eat("LIMIT") {
                if top {
                    acc.up_to = self.peek(0).map(|t| t.text.clone());
                }
                self.pos += 1;
            } else {
                self.scan_one(acc, scope, out);
            }
            if self.pos == before {
                self.pos += 1;
            }
        }
    }

    fn clause_end(&self) -> usize {
        self.find(|t| CLAUSES.iter().any(|c| t.is(c)))
    }

    fn where_clause(
        &mut self,
        acc: &mut m::DbAccess,
        scope: &mut Scope,
        out: &mut Native,
        top: bool,
    ) {
        let end = self.clause_end();
        let conds = self.sub(self.pos..end).conditions(acc, scope, out);
        if top {
            acc.where_text = Some(self.text_of(self.pos..end));
            acc.conditions.extend(conds);
        }
        self.pos = end;
    }

    /// The FROM list with joins.
    fn from(
        &mut self,
        acc: &mut m::DbAccess,
        scope: &mut Scope,
        out: &mut Native,
        role: m::DbTableRole,
    ) {
        let end = self.clause_end();
        let saved = self.end;
        self.end = end;
        loop {
            self.table_ref(acc, scope, out, role);
            loop {
                let kind = if self.eat("CROSS") {
                    "cross"
                } else if self.eat("INNER") {
                    "inner"
                } else if self.eat("LEFT") {
                    "left_outer"
                } else if self.eat("RIGHT") {
                    "right_outer"
                } else if self.eat("FULL") {
                    "full_outer"
                } else if self.at("JOIN") {
                    "inner"
                } else {
                    break;
                };
                // OUTER, MANY TO ONE and other cardinalities up to JOIN
                while !self.done() && !self.at("JOIN") {
                    self.pos += 1;
                }
                self.eat("JOIN");
                let at = acc.tables.len();
                self.table_ref(acc, scope, out, m::DbTableRole::Join);
                let joined = acc.tables.get(at).cloned();
                let mut on = Vec::new();
                if self.eat("ON") {
                    let on_end =
                        self.find(|t| JOIN_WORDS[..5].iter().any(|w| t.is(w)) || t.sym(","));
                    on = self.sub(self.pos..on_end).conditions(acc, scope, out);
                    self.pos = on_end;
                }
                if let Some(table) = joined {
                    acc.joins.push(m::DbJoin {
                        kind: kind.to_string(),
                        table,
                        on,
                    });
                }
            }
            if !self.at_sym(",") {
                break;
            }
            self.pos += 1;
        }
        self.end = saved;
        self.pos = end;
    }

    /// A table, table variable, table function, hierarchy function or
    /// subquery, with its alias.
    fn table_ref(
        &mut self,
        acc: &mut m::DbAccess,
        scope: &mut Scope,
        out: &mut Native,
        role: m::DbTableRole,
    ) {
        self.table_or_columns(acc, scope, out, role, true);
    }

    fn table_or_columns(
        &mut self,
        acc: &mut m::DbAccess,
        scope: &mut Scope,
        out: &mut Native,
        role: m::DbTableRole,
        functions: bool,
    ) {
        let Some(t) = self.peek(0) else { return };
        let name = if t.sym("(") {
            let close = self.close_of(self.pos);
            let mut inner = self.sub(self.pos + 1..close - 1);
            if inner.query_ahead() {
                inner.query(acc, scope, out, m::DbTableRole::Subquery, false);
            } else {
                inner.from(acc, scope, out, role);
            }
            self.pos = close;
            None
        } else if t.kind == Kind::Var {
            self.pos += 1;
            acc.host_variables.push(t.text.clone());
            let name = format!(":{}", t.text);
            acc.tables.push(table(&name, m::DbTableRole::InternalTable));
            Some(name)
        } else if t.is_name() {
            let start = self.pos;
            let name = self.dotted_name().unwrap_or_default();
            if functions && self.at_sym("(") {
                // table function, hierarchy function
                let close = self.close_of(self.pos);
                let args = self.pos + 1..close - 1;
                let upper = name.to_ascii_uppercase();
                if upper.starts_with("HIERARCHY") {
                    let mut a = self.sub(args);
                    while !a.done() {
                        if a.eat("SOURCE") {
                            a.table_ref(acc, scope, out, role);
                        } else {
                            let before = a.pos;
                            a.scan_one(acc, scope, out);
                            if a.pos == before {
                                a.pos += 1;
                            }
                        }
                    }
                } else {
                    if !is_builtin_table_function(&name) {
                        out.calls.push(NativeCall {
                            name: name.clone(),
                            function: true,
                            range: self.t[start].start..self.t[close - 1].end,
                        });
                        acc.tables.push(table(&name, role));
                    }
                    self.sub(args).scan(acc, scope, out);
                }
                self.pos = close;
            } else if !name.eq_ignore_ascii_case("DUMMY") {
                acc.tables.push(table(&name, role));
            }
            Some(name)
        } else {
            return;
        };
        // alias
        self.eat("AS");
        let alias = self.peek(0).filter(|t| {
            t.is_name()
                && !(t.kind == Kind::Word
                    && CLAUSES
                        .iter()
                        .chain(JOIN_WORDS)
                        .chain(NO_ALIAS)
                        .any(|w| t.is(w)))
        });
        let alias = alias.map(|t| t.text.clone());
        if alias.is_some() {
            self.pos += 1;
        }
        if let Some(name) = name {
            let table_name = name.clone();
            scope
                .names
                .insert(name.to_ascii_uppercase(), table_name.clone());
            if let Some(last) = name.rsplit('.').next() {
                scope
                    .names
                    .insert(last.to_ascii_uppercase(), table_name.clone());
            }
            if let Some(a) = &alias {
                scope
                    .names
                    .insert(a.to_ascii_uppercase(), table_name.clone());
                if let Some(t) = acc.tables.iter_mut().rev().find(|t| t.name == table_name) {
                    t.alias = Some(a.clone());
                }
            }
        }
    }

    // ----- conditions and expressions ---------------------------------------

    /// The comparisons of a logical expression.
    fn conditions(
        &mut self,
        acc: &mut m::DbAccess,
        scope: &mut Scope,
        out: &mut Native,
    ) -> Vec<m::DbCondition> {
        let mut result = Vec::new();
        // split at AND and OR on depth 0, not at the AND of BETWEEN
        let mut parts: Vec<Range<usize>> = Vec::new();
        let mut start = self.pos;
        let mut between = false;
        let mut i = self.pos;
        while i < self.end {
            let t = &self.t[i];
            if t.sym("(") {
                i = self.close_of(i);
                continue;
            }
            if t.is("BETWEEN") {
                between = true;
            } else if t.is("AND") && between {
                between = false;
            } else if t.is("AND") || t.is("OR") {
                parts.push(start..i);
                start = i + 1;
            }
            i += 1;
        }
        parts.push(start..self.end);
        for part in parts {
            let mut q = self.sub(part.clone());
            while q.eat("NOT") {}
            if q.done() {
                continue;
            }
            // `( cond )`: a parenthesized logical expression
            if q.at_sym("(") && q.close_of(q.pos) == q.end && !q.query_ahead() {
                let mut inner = self.sub(q.pos + 1..q.end - 1);
                result.extend(inner.conditions(acc, scope, out));
                continue;
            }
            if q.at("EXISTS") {
                q.pos += 1;
                let text = q.text_of(q.pos..q.end);
                q.scan(acc, scope, out);
                result.push(m::DbCondition {
                    left: String::new(),
                    operator: "EXISTS".into(),
                    right: Some(text),
                    columns: Vec::new(),
                });
                continue;
            }
            let op_at = q.find(|t| {
                (t.kind == Kind::Sym
                    && matches!(t.text.as_str(), "=" | "<>" | "!=" | "<" | ">" | "<=" | ">="))
                    || [
                        "LIKE",
                        "IN",
                        "BETWEEN",
                        "IS",
                        "NOT",
                        "LIKE_REGEXPR",
                        "MEMBER",
                    ]
                    .iter()
                    .any(|w| t.is(w))
            });
            let left = part.start..op_at;
            let mut op_end = op_at;
            let mut operator = String::new();
            while op_end < part.end {
                let t = &self.t[op_end];
                let is_op = (t.kind == Kind::Sym
                    && matches!(t.text.as_str(), "=" | "<>" | "!=" | "<" | ">" | "<=" | ">="))
                    || [
                        "LIKE",
                        "IN",
                        "BETWEEN",
                        "IS",
                        "NOT",
                        "NULL",
                        "LIKE_REGEXPR",
                        "MEMBER",
                        "OF",
                        "ANY",
                        "SOME",
                        "ALL",
                    ]
                    .iter()
                    .any(|w| t.is(w));
                if !is_op
                    || (t.is("NULL") && !operator.ends_with("IS") && !operator.ends_with("NOT"))
                {
                    break;
                }
                if !operator.is_empty() {
                    operator.push(' ');
                }
                operator.push_str(&t.text.to_ascii_uppercase());
                op_end += 1;
                if t.is("NULL") {
                    break;
                }
            }
            let right = op_end..part.end;
            let mut columns = self.columns(left.clone(), scope);
            columns.extend(self.columns(right.clone(), scope));
            self.sub(part.clone()).scan_variables(acc, scope, out);
            result.push(m::DbCondition {
                left: self.text_of(left),
                operator: if operator.is_empty() {
                    "PREDICATE".into()
                } else {
                    operator
                },
                right: (!right.is_empty()).then(|| self.text_of(right)),
                columns,
            });
        }
        self.pos = self.end;
        result
    }

    /// The columns of an expression.
    fn columns(&self, range: Range<usize>, scope: &Scope) -> Vec<m::DbColumn> {
        let mut out = Vec::new();
        let mut i = range.start;
        while i < range.end {
            let t = &self.t[i];
            if !t.is_name() || (t.kind == Kind::Word && EXPRESSION_WORDS.iter().any(|w| t.is(w))) {
                i += 1;
                continue;
            }
            // a chain `a.b.c`
            let mut parts = vec![t];
            let mut k = i + 1;
            while k + 1 < range.end && self.t[k].sym(".") && self.t[k + 1].is_name() {
                parts.push(&self.t[k + 1]);
                k += 2;
            }
            let call = k < range.end && self.t[k].sym("(");
            let previous_dot = i > range.start && self.t[i - 1].sym(".");
            if !call && !previous_dot {
                let toks: Vec<Tok> = parts.into_iter().cloned().collect();
                if let Some(c) = column_of(&toks, scope) {
                    if !out.contains(&c) {
                        out.push(c);
                    }
                }
            }
            i = k;
        }
        out
    }

    /// Variables and subqueries of the tokens up to the end, nothing else.
    fn scan_variables(&mut self, acc: &mut m::DbAccess, scope: &mut Scope, out: &mut Native) {
        self.scan(acc, scope, out);
    }

    /// Finds variables and queries in the tokens up to the end.
    fn scan(&mut self, acc: &mut m::DbAccess, scope: &mut Scope, out: &mut Native) {
        while !self.done() {
            let before = self.pos;
            self.scan_one(acc, scope, out);
            if self.pos == before {
                self.pos += 1;
            }
        }
    }

    fn scan_one(&mut self, acc: &mut m::DbAccess, scope: &mut Scope, out: &mut Native) {
        let Some(t) = self.peek(0) else { return };
        if t.kind == Kind::Var {
            acc.host_variables.push(t.text.clone());
            self.pos += 1;
        } else if t.sym("(") {
            let close = self.close_of(self.pos);
            let mut inner = self.sub(self.pos + 1..close.saturating_sub(1).max(self.pos + 1));
            if inner.query_ahead() {
                inner.query(acc, scope, out, m::DbTableRole::Subquery, false);
            } else {
                inner.scan(acc, scope, out);
            }
            self.pos = close;
        } else if t.is("SELECT") {
            self.query(acc, scope, out, m::DbTableRole::Subquery, false);
        } else if t.is_name() && t.text.contains("=>") && self.peek(1).is_some_and(|n| n.sym("(")) {
            // an AMDP function in an expression
            let close = self.close_of(self.pos + 1);
            out.calls.push(NativeCall {
                name: t.text.clone(),
                function: true,
                range: t.start..self.t[close - 1].end,
            });
            self.pos += 1;
        } else {
            self.pos += 1;
        }
    }
}

fn table(name: &str, role: m::DbTableRole) -> m::DbTable {
    m::DbTable {
        name: name.to_string(),
        alias: None,
        path: None,
        dynamic: false,
        resolved: None,
        role,
    }
}

/// `col`, `alias.col`, `schema.table.col` as a column.
fn column_of(toks: &[Tok], scope: &Scope) -> Option<m::DbColumn> {
    let names: Vec<&Tok> = toks.iter().filter(|t| t.is_name()).collect();
    if names.is_empty() || toks.iter().any(|t| t.kind == Kind::Sym && t.text != ".") {
        return None;
    }
    let name = names.last()?.text.clone();
    let qualifier = (names.len() >= 2).then(|| names[names.len() - 2].text.clone());
    let table = match &qualifier {
        Some(q) => scope.names.get(&q.to_ascii_uppercase()).cloned(),
        None => {
            let mut tables: Vec<&String> = scope.names.values().collect();
            tables.sort();
            tables.dedup();
            (tables.len() == 1).then(|| tables[0].clone())
        }
    };
    Some(m::DbColumn {
        name,
        qualifier,
        table: table.filter(|t| !t.starts_with(':')),
    })
}

/// One element of a select list.
fn field(p: &P, range: Range<usize>, scope: &Scope) -> m::DbField {
    let toks = &p.t[range.clone()];
    let mut expr_end = range.end;
    let mut alias = None;
    let n = toks.len();
    if n >= 2 && toks[n - 2].is("AS") && toks[n - 1].is_name() {
        alias = Some(toks[n - 1].text.clone());
        expr_end = range.end - 2;
    } else if n >= 2
        && toks[n - 1].is_name()
        && !(toks[n - 1].kind == Kind::Word && EXPRESSION_WORDS.iter().any(|w| toks[n - 1].is(w)))
        && !toks[n - 2].sym(".")
        && (toks[n - 2].sym(")")
            || toks[n - 2].is_name()
            || toks[n - 2].kind == Kind::Str
            || toks[n - 2].kind == Kind::Num)
        && !toks[n - 2].is("END")
    {
        alias = Some(toks[n - 1].text.clone());
        expr_end = range.end - 1;
    }
    let first = &toks[0];
    let aggregate = (first.kind == Kind::Word
        && AGGREGATES.iter().any(|a| first.is(a))
        && n > 1
        && toks[1].sym("(")
        && p.close_of(range.start + 1) == expr_end)
        .then(|| first.text.to_ascii_lowercase());
    m::DbField {
        text: p.text_of(range.start..expr_end),
        alias,
        aggregate,
        columns: p.columns(range.start..expr_end, scope),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn amdp(text: &str) -> Native {
        parse(text, Dialect::SqlScript)
    }

    fn names(a: &m::DbAccess) -> Vec<(String, m::DbTableRole)> {
        a.tables.iter().map(|t| (t.name.clone(), t.role)).collect()
    }

    #[test]
    fn lexer() {
        let t = lex(
            "* comment line\nSELECT \"A\"\"B\", 'it''s', :lv, 1.5e3 -- rest\n/* block */ FROM t;",
            Dialect::SqlScript,
        );
        let kinds: Vec<(Kind, &str)> = t.iter().map(|t| (t.kind, t.text.as_str())).collect();
        assert_eq!(
            kinds,
            [
                (Kind::Word, "SELECT"),
                (Kind::Quoted, "A\"B"),
                (Kind::Sym, ","),
                (Kind::Str, "'it''s'"),
                (Kind::Sym, ","),
                (Kind::Var, "lv"),
                (Kind::Sym, ","),
                (Kind::Num, "1.5e3"),
                (Kind::Word, "FROM"),
                (Kind::Word, "t"),
                (Kind::Sym, ";"),
            ]
        );
        let e = lex("SELECT a INTO :wa-b FROM t \" comment\n", Dialect::ExecSql);
        assert_eq!(
            e.iter().map(|t| t.text.as_str()).collect::<Vec<_>>(),
            ["SELECT", "a", "INTO", "wa-b", "FROM", "t"]
        );
    }

    #[test]
    fn amdp_procedure() {
        let n = amdp(
            r#"
* select
    lt_flights = SELECT f.carrid, f.connid, SUM( f.price ) AS total
                FROM "ZDEMO_ABAP_FLI" AS f
                INNER JOIN zdemo_abap_carr AS c ON c.carrid = f.carrid
                WHERE f.carrid = :iv_carrid AND f.fldate BETWEEN :iv_from AND :iv_to
                GROUP BY f.carrid, f.connid
                ORDER BY f.carrid;
    IF :iv_mode = 'X' THEN
      et_result = SELECT * FROM :lt_flights WHERE total > 100;
    ELSE
      et_result = SELECT * FROM :lt_flights;
      DELETE FROM zlog WHERE carrid = :iv_carrid;
    END IF;
    CALL "ZCL_X=>WRITE_LOG"( iv_text => 'done' );
"#,
        );
        assert_eq!(n.accesses.len(), 5, "{:#?}", n.accesses);
        let a = &n.accesses[0].access;
        assert_eq!(a.operation, m::DbOperation::Select);
        assert_eq!(
            names(a),
            [
                ("ZDEMO_ABAP_FLI".into(), m::DbTableRole::Main),
                ("zdemo_abap_carr".into(), m::DbTableRole::Join)
            ]
        );
        assert_eq!(a.target.as_ref().unwrap().targets, ["lt_flights"]);
        assert_eq!(a.host_variables, ["iv_carrid", "iv_from", "iv_to"]);
        assert_eq!(a.fields.len(), 3);
        assert_eq!(a.fields[2].aggregate.as_deref(), Some("sum"));
        assert_eq!(a.fields[2].alias.as_deref(), Some("total"));
        assert_eq!(
            a.fields[0].columns[0].table.as_deref(),
            Some("ZDEMO_ABAP_FLI")
        );
        assert_eq!(a.joins[0].on[0].columns.len(), 2);
        let ops: Vec<&str> = a.conditions.iter().map(|c| c.operator.as_str()).collect();
        assert_eq!(ops, ["=", "BETWEEN"]);
        assert_eq!(a.group_by.len(), 2);
        assert_eq!(a.order_by, ["f.carrid"]);

        let b = &n.accesses[1].access;
        assert_eq!(
            names(b),
            [(":lt_flights".into(), m::DbTableRole::InternalTable)]
        );
        assert_eq!(b.host_variables, ["lt_flights"]);
        let d = &n.accesses[3].access;
        assert_eq!(d.operation, m::DbOperation::Delete);
        assert_eq!(names(d), [("zlog".into(), m::DbTableRole::Main)]);
        let c = &n.accesses[4].access;
        assert_eq!(c.operation, m::DbOperation::Native);
        assert_eq!(n.calls.len(), 1);
        assert_eq!(n.calls[0].name, "ZCL_X=>WRITE_LOG");
        assert!(!n.calls[0].function);
    }

    #[test]
    fn table_functions_cte_and_set_operations() {
        let n = amdp(
            r#"RETURN WITH recent AS ( SELECT * FROM zorders WHERE created > ADD_DAYS( CURRENT_DATE, -7 ) )
  SELECT r.id FROM recent AS r
  UNION ALL
  SELECT id FROM "ZCL_X=>GET_OLD"( p_days => :iv_days )
  WHERE EXISTS ( SELECT 1 FROM zitems AS i WHERE i.id = id );"#,
        );
        let a = &n.accesses[0].access;
        assert!(a.set_operation);
        assert_eq!(a.target.as_ref().unwrap().targets, ["RETURN"]);
        assert_eq!(
            names(a),
            [
                ("recent".into(), m::DbTableRole::Cte),
                ("zorders".into(), m::DbTableRole::Subquery),
                ("recent".into(), m::DbTableRole::Main),
                ("ZCL_X=>GET_OLD".into(), m::DbTableRole::Main),
                ("zitems".into(), m::DbTableRole::Subquery),
            ]
        );
        assert!(n
            .calls
            .iter()
            .any(|c| c.function && c.name == "ZCL_X=>GET_OLD"));
        assert_eq!(a.host_variables, ["iv_days"]);
    }

    #[test]
    fn hierarchy_function() {
        let n = amdp(
            "lt_tree = SELECT * FROM HIERARCHY_DESCENDANTS( SOURCE HIERARCHY( SOURCE ( SELECT id AS node_id, parent AS parent_id FROM znodes ) ) START WHERE node_id = :iv_root );",
        );
        let a = &n.accesses[0].access;
        assert_eq!(names(a), [("znodes".into(), m::DbTableRole::Subquery)]);
        assert_eq!(a.host_variables, ["iv_root"]);
    }

    #[test]
    fn data_changes() {
        let n = amdp(
            "INSERT INTO zlog (id, text) VALUES (:lv_id, 'x');\n\
             UPDATE zorders SET status = 'C', changed = CURRENT_TIMESTAMP WHERE id IN (SELECT id FROM :lt_done);\n\
             UPSERT ztotals SELECT carrid, SUM(price) FROM zflights GROUP BY carrid;\n\
             MERGE INTO zt AS t USING zsrc AS s ON t.id = s.id WHEN MATCHED THEN UPDATE SET t.v = s.v WHEN NOT MATCHED THEN INSERT VALUES (s.id, s.v);\n\
             TRUNCATE TABLE ztmp;",
        );
        let ops: Vec<m::DbOperation> = n.accesses.iter().map(|a| a.access.operation).collect();
        use m::DbOperation::*;
        assert_eq!(ops, [Insert, Update, Modify, Merge, Delete]);
        let ins = &n.accesses[0].access;
        assert_eq!(ins.host_variables, ["lv_id"]);
        assert!(ins
            .source
            .as_ref()
            .unwrap()
            .text
            .as_deref()
            .unwrap()
            .starts_with("VALUES"));
        let upd = &n.accesses[1].access;
        assert_eq!(upd.set.len(), 2);
        assert_eq!(upd.set[0].column.name, "status");
        assert_eq!(
            names(upd),
            [
                ("zorders".into(), m::DbTableRole::Main),
                (":lt_done".into(), m::DbTableRole::InternalTable)
            ]
        );
        let ups = &n.accesses[2].access;
        assert_eq!(
            names(ups),
            [
                ("ztotals".into(), m::DbTableRole::Main),
                ("zflights".into(), m::DbTableRole::Subquery)
            ]
        );
        let merge = &n.accesses[3].access;
        assert_eq!(
            names(merge),
            [
                ("zt".into(), m::DbTableRole::Main),
                ("zsrc".into(), m::DbTableRole::MergeSource)
            ]
        );
        assert_eq!(merge.set.len(), 1);
    }

    #[test]
    fn exec_sql() {
        let n = parse(
            "\n  SELECT cityfrom, cityto\n         INTO :wa\n         FROM spfli\n         WHERE mandt  = :sy-mandt AND\n               carrid = :carrid \" a comment\n",
            Dialect::ExecSql,
        );
        let a = &n.accesses[0].access;
        assert_eq!(names(a), [("spfli".into(), m::DbTableRole::Main)]);
        assert_eq!(a.target.as_ref().unwrap().targets, ["wa"]);
        assert_eq!(a.host_variables, ["sy-mandt", "carrid"]);
        assert_eq!(a.conditions.len(), 2);
        assert!(!a.text.contains("comment"));
    }

    #[test]
    fn select_into_variables_and_blocks() {
        let n = amdp(
            "DECLARE lv_count INTEGER;\nBEGIN\n  SELECT COUNT(*) INTO lv_count FROM zorders WHERE status = 'O';\n  WHILE :lv_count > 0 DO\n    lv_count = :lv_count - 1;\n  END WHILE;\n  FOR i IN 1 .. 3 DO\n    INSERT INTO zlog VALUES (:i);\n  END FOR;\nEND;",
        );
        let ops: Vec<m::DbOperation> = n.accesses.iter().map(|a| a.access.operation).collect();
        assert_eq!(ops, [m::DbOperation::Select, m::DbOperation::Insert]);
        let a = &n.accesses[0].access;
        assert_eq!(a.target.as_ref().unwrap().targets, ["lv_count"]);
        assert_eq!(a.fields[0].aggregate.as_deref(), Some("count"));
    }
}
