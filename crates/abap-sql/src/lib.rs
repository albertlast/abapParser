//! Open SQL: tables, columns, joins, conditions and targets of ABAP database
//! statements, classic and strict syntax.
//!
//! The parser works on clause level and is lenient: what it cannot place is
//! skipped, so new syntax additions degrade the result instead of losing it.

use std::collections::HashMap;

use abap_model as m;
use abap_syntax::{File, Statement, StatementKind as K, Token, TokenKind};

pub mod cds;
mod cluster;
pub mod hana;

pub use cluster::parse_cluster;

/// Statement kinds that access the database.
pub fn is_db_statement(kind: K) -> bool {
    matches!(
        kind,
        K::Select
            | K::SelectLoop
            | K::InsertDatabase
            | K::UpdateDatabase
            | K::ModifyDatabase
            | K::DeleteDatabase
            | K::MergeDatabase
            | K::OpenCursor
            | K::With
            | K::WithLoop
    )
}

/// Parses a database statement. `block`, `text` and `span` of the result are
/// left empty for the caller to fill.
pub fn parse_statement(file: &File, s: &Statement) -> Option<m::DbAccess> {
    if !is_db_statement(s.kind) {
        return None;
    }
    let mut tokens: Vec<&Token> = s.tokens.iter().map(|i| &file.lexed.tokens[*i]).collect();
    if tokens
        .last()
        .is_some_and(|t| t.kind == TokenKind::Punctuation)
    {
        tokens.pop();
    }
    tokens.retain(|t| t.kind != TokenKind::Pragma);
    let mut access = parse(s.kind, &tokens);
    access.host_variables = host_variables(file, s);
    Some(access)
}

/// ABAP data the statement reads, as written: `@lv_x`, the operands of
/// `@( ... )`, and the unmarked host variables of the classic syntax.
pub(crate) fn host_variables(file: &File, s: &Statement) -> Vec<String> {
    let all = file.tokens();
    let mut out: Vec<String> = Vec::new();
    for r in abap_syntax::operands::operands(file, s).refs() {
        if r.access != abap_syntax::expr::Access::Read {
            continue;
        }
        let path = abap_syntax::join_tokens((r.start..=r.last).map(|i| &all[s.tokens[i]]));
        if !out.iter().any(|v| v.eq_ignore_ascii_case(&path)) {
            out.push(path);
        }
    }
    out
}

/// Parses the tokens of a database statement, without terminator and pragmas.
pub fn parse(kind: K, tokens: &[&Token]) -> m::DbAccess {
    let p = Parser::new(tokens);
    let mut access = empty_access(kind);
    let end = tokens.len();
    match kind {
        K::Select | K::SelectLoop => {
            let q = p.select(0, end);
            p.apply_query(&mut access, q);
        }
        K::With | K::WithLoop => {
            let q = p.with(0, end);
            p.apply_query(&mut access, q);
        }
        K::OpenCursor => {
            let start =
                (0..end).find(|i| p.depth[*i] == 0 && (p.kw(*i, "SELECT") || p.kw(*i, "WITH")));
            if let Some(start) = start {
                let q = if p.kw(start, "WITH") {
                    p.with(start, end)
                } else {
                    p.select(start, end)
                };
                p.apply_query(&mut access, q);
            }
            // OPEN CURSOR [WITH HOLD] cursor FOR ...
            let mut i = 2;
            if p.seq(i, end, "WITH HOLD").is_some() {
                i += 2;
            }
            if i < end && !p.kw(i, "FOR") {
                let j = p.operand_end(i, end);
                access.cursor = Some(p.text(i, j));
            }
        }
        K::InsertDatabase => p.dml(&mut access, m::DbOperation::Insert, end),
        K::UpdateDatabase => p.dml(&mut access, m::DbOperation::Update, end),
        K::ModifyDatabase => p.dml(&mut access, m::DbOperation::Modify, end),
        K::DeleteDatabase => p.dml(&mut access, m::DbOperation::Delete, end),
        K::MergeDatabase => p.merge(&mut access, end),
        _ => {}
    }
    access.is_loop = matches!(kind, K::SelectLoop | K::WithLoop);
    access
}

pub(crate) fn empty_access(kind: K) -> m::DbAccess {
    let zero = m::Position { row: 0, col: 0 };
    m::DbAccess {
        operation: m::DbOperation::Select,
        statement: kind.as_str().to_string(),
        tables: Vec::new(),
        fields: Vec::new(),
        joins: Vec::new(),
        conditions: Vec::new(),
        where_text: None,
        group_by: Vec::new(),
        having: Vec::new(),
        order_by: Vec::new(),
        target: None,
        source: None,
        set: Vec::new(),
        for_all_entries: None,
        cursor: None,
        single: false,
        distinct: false,
        is_loop: false,
        up_to: None,
        set_operation: false,
        dynamic: false,
        client_specified: false,
        bypassing_buffer: false,
        connection: None,
        native_sql: None,
        cluster: None,
        host_variables: Vec::new(),
        block: None,
        text: String::new(),
        span: m::Span {
            start: zero,
            end: zero,
        },
    }
}

/// Result of one query, subqueries and set operations merged in.
#[derive(Default)]
struct Query {
    tables: Vec<m::DbTable>,
    fields: Vec<m::DbField>,
    joins: Vec<m::DbJoin>,
    conditions: Vec<m::DbCondition>,
    where_text: Option<String>,
    group_by: Vec<m::DbColumn>,
    having: Vec<m::DbCondition>,
    order_by: Vec<String>,
    target: Option<m::DbTarget>,
    for_all_entries: Option<String>,
    single: bool,
    distinct: bool,
    up_to: Option<String>,
    set_operation: bool,
    dynamic: bool,
    client_specified: bool,
    bypassing_buffer: bool,
    connection: Option<String>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Clause {
    Fields,
    From,
    Into,
    Appending,
    Where,
    GroupBy,
    Having,
    OrderBy,
    UpTo,
    Offset,
    ForAllEntries,
    PackageSize,
    BypassingBuffer,
    ClientSpecified,
    UsingClient,
    Connection,
    ForUpdate,
    SetOperation,
    Ignored,
}

const CLAUSES: &[(&str, Clause)] = &[
    ("FOR ALL ENTRIES IN", Clause::ForAllEntries),
    ("FOR UPDATE", Clause::ForUpdate),
    ("GROUP BY", Clause::GroupBy),
    ("ORDER BY", Clause::OrderBy),
    ("UP TO", Clause::UpTo),
    ("PACKAGE SIZE", Clause::PackageSize),
    ("BYPASSING BUFFER", Clause::BypassingBuffer),
    ("CLIENT SPECIFIED", Clause::ClientSpecified),
    ("USING CLIENT", Clause::UsingClient),
    ("USING CLIENTS", Clause::UsingClient),
    ("USING ALL CLIENTS", Clause::UsingClient),
    ("FIELDS", Clause::Fields),
    ("FROM", Clause::From),
    ("INTO", Clause::Into),
    ("APPENDING", Clause::Appending),
    ("WHERE", Clause::Where),
    ("HAVING", Clause::Having),
    ("OFFSET", Clause::Offset),
    ("CONNECTION", Clause::Connection),
    ("UNION", Clause::SetOperation),
    ("INTERSECT", Clause::SetOperation),
    ("EXCEPT", Clause::SetOperation),
    ("INDICATORS", Clause::Ignored),
    ("EXTENDED RESULT", Clause::Ignored),
    ("OPTIONS", Clause::Ignored),
    ("PRIVILEGED ACCESS", Clause::Ignored),
];

const AGGREGATES: &[&str] = &[
    "COUNT",
    "MAX",
    "MIN",
    "SUM",
    "AVG",
    "STDDEV",
    "VAR",
    "MEDIAN",
    "PRODUCT",
    "STRING_AGG",
    "CORR",
    "CORR_SPEARMAN",
    "GROUPING",
];

/// Words that are never column names in conditions and select lists.
const SQL_WORDS: &[&str] = &[
    "AS",
    "CASE",
    "WHEN",
    "THEN",
    "ELSE",
    "END",
    "AND",
    "OR",
    "NOT",
    "IS",
    "NULL",
    "IN",
    "BETWEEN",
    "LIKE",
    "ESCAPE",
    "DISTINCT",
    "ALL",
    "ANY",
    "SOME",
    "ASCENDING",
    "DESCENDING",
    "INITIAL",
    "EXISTS",
    "EQ",
    "NE",
    "LT",
    "GT",
    "LE",
    "GE",
    "CAST",
    "OVER",
    "PARTITION",
    "BY",
    "ORDER",
    "ROWS",
    "SINGLE",
    "PRIMARY",
    "KEY",
    "DIV",
    "MOD",
    "TRUE",
    "FALSE",
    "UNKNOWN",
    "INTO",
    "FROM",
];

const COMPARISONS: &[&str] = &[
    "=", "<>", "<", ">", "<=", ">=", "><", "=<", "=>", "EQ", "NE", "LT", "GT", "LE", "GE",
];

struct Parser<'a> {
    t: &'a [&'a Token],
    /// nesting depth in front of each token, parentheses have the outer depth
    depth: Vec<u32>,
}

impl<'a> Parser<'a> {
    fn new(t: &'a [&'a Token]) -> Self {
        let mut depth = Vec::with_capacity(t.len());
        let mut d: u32 = 0;
        for tok in t {
            if matches!(tok.kind, TokenKind::ParenRight | TokenKind::BracketRight) {
                d = d.saturating_sub(1);
            }
            depth.push(d);
            if matches!(tok.kind, TokenKind::ParenLeft | TokenKind::BracketLeft) {
                d += 1;
            }
        }
        Parser { t, depth }
    }

    fn kw(&self, i: usize, word: &str) -> bool {
        self.t
            .get(i)
            .is_some_and(|t| t.kind == TokenKind::Identifier && t.text.eq_ignore_ascii_case(word))
    }

    /// Words starting at `i`, all at the depth of `i`; returns the index after them.
    fn seq(&self, i: usize, end: usize, words: &str) -> Option<usize> {
        let mut j = i;
        for w in words.split(' ') {
            if j >= end || !self.kw(j, w) || self.depth[j] != self.depth[i] {
                return None;
            }
            j += 1;
        }
        Some(j)
    }

    fn is_open(&self, i: usize) -> bool {
        matches!(
            self.t[i].kind,
            TokenKind::ParenLeft | TokenKind::BracketLeft
        )
    }

    fn is_close(&self, i: usize) -> bool {
        matches!(
            self.t[i].kind,
            TokenKind::ParenRight | TokenKind::BracketRight
        )
    }

    /// Index of the closing parenthesis for the opening one at `i`.
    fn close(&self, i: usize, end: usize) -> usize {
        let d = self.depth[i];
        (i + 1..end)
            .find(|j| self.depth[*j] == d && self.is_close(*j))
            .unwrap_or(end)
    }

    /// Index after one operand: tokens connected without whitespace, parentheses included.
    fn operand_end(&self, i: usize, end: usize) -> usize {
        let mut j = i;
        let mut open = 0u32;
        while j < end {
            let t = self.t[j];
            if j > i && open == 0 && (t.ws_before || t.kind == TokenKind::Punctuation) {
                break;
            }
            if self.is_open(j) {
                open += 1;
            } else if self.is_close(j) {
                if open == 0 {
                    break;
                }
                open -= 1;
            }
            j += 1;
        }
        j.max(i + 1).min(end.max(i + 1))
    }

    fn text(&self, i: usize, j: usize) -> String {
        abap_syntax::join_tokens(self.t[i..j.min(self.t.len())].iter().copied())
    }

    /// A clause keyword at `i`; returns the clause and the index after the keyword.
    fn clause_at(&self, i: usize, end: usize) -> Option<(Clause, usize)> {
        if self.t[i].kind != TokenKind::Identifier {
            return None;
        }
        // INTO CORRESPONDING FIELDS OF: FIELDS belongs to INTO
        if self.kw(i, "FIELDS") && i > 0 && self.kw(i - 1, "CORRESPONDING") {
            return None;
        }
        CLAUSES
            .iter()
            .find_map(|(words, clause)| self.seq(i, end, words).map(|j| (*clause, j)))
    }

    fn is_subquery(&self, i: usize, end: usize) -> bool {
        self.is_open(i) && (self.kw(i + 1, "SELECT") || self.kw(i + 1, "WITH")) && i + 1 < end
    }

    /// `(name)`, a dynamic token: a single operand directly inside the
    /// parentheses. With blanks, `( name )` is a parenthesized expression.
    fn is_dynamic(&self, i: usize, end: usize) -> bool {
        if !self.is_open(i) || self.t[i].ws_after || i + 1 >= end || self.is_open(i + 1) {
            return false;
        }
        let close = self.close(i, end);
        close > i + 1
            && close < end
            && !self.t[close].ws_before
            && self.operand_end(i + 1, close) == close
            && !self.kw(i + 1, "SELECT")
    }

    /// `( SELECT ... )`, also with more parentheses inside: `( ( SELECT ... ) UNION ... )`.
    fn is_query(&self, i: usize, e: usize) -> bool {
        self.is_subquery(i, e) || (i + 1 < e && self.is_open(i) && self.is_query(i + 1, e))
    }

    /// A query expression: SELECT, WITH or a parenthesized query, each
    /// possibly followed by UNION, INTERSECT or EXCEPT and another query.
    fn query_expr(&self, s: usize, e: usize) -> Query {
        if s >= e {
            return Query::default();
        }
        if self.kw(s, "SELECT") {
            return self.select(s, e);
        }
        if self.kw(s, "WITH") {
            return self.with(s, e);
        }
        if !self.is_open(s) {
            return Query::default();
        }
        let close = self.close(s, e);
        let mut q = self.query_expr(s + 1, close);
        let mut k = close + 1;
        if k < e && (self.kw(k, "UNION") || self.kw(k, "INTERSECT") || self.kw(k, "EXCEPT")) {
            k += 1;
            while k < e && (self.kw(k, "ALL") || self.kw(k, "DISTINCT")) {
                k += 1;
            }
            let other = self.query_expr(k, e);
            q.set_operation = true;
            q.tables.extend(other.tables);
            q.joins.extend(other.joins);
            q.conditions.extend(other.conditions);
            q.target = q.target.or(other.target);
            q.dynamic |= other.dynamic;
        } else if k < e {
            // INTO etc. after a parenthesized query belongs to the whole statement
            let tail = self.select(k, e);
            q.target = q.target.or(tail.target);
        }
        q
    }

    // ---------------------------------------------------------------- queries

    /// WITH +cte AS ( SELECT ... ) [, ...] SELECT ...
    fn with(&self, start: usize, end: usize) -> Query {
        let mut q = Query::default();
        let mut i = start + 1;
        while i < end && !self.kw(i, "SELECT") {
            if self.is_query(i, end) {
                let close = self.close(i, end);
                let sub = self.query_expr(i + 1, close);
                let conditions = self.merge_subquery(&mut q, sub);
                q.conditions.extend(conditions);
                i = close + 1;
            } else if let Some(open) = self.seq(i, end, "WITH ASSOCIATIONS") {
                // WITH ASSOCIATIONS ( JOIN target AS x ON ... ) defines associations:
                // their tables take part, but they are no joins of the query.
                // Exposed paths like ( \_assoc AS x ) name no table.
                if open < end && self.is_open(open) {
                    let close = self.close(open, end);
                    if (open + 1..close).any(|k| self.kw(k, "JOIN")) {
                        let joins = q.joins.len();
                        self.from(&mut q, open + 1, close);
                        q.joins.truncate(joins);
                    }
                    i = close + 1;
                } else {
                    i = open;
                }
            } else {
                i += 1;
            }
        }
        if i < end {
            let main = self.select(i, end);
            let cte_tables = std::mem::take(&mut q.tables);
            let cte_conditions = std::mem::take(&mut q.conditions);
            let cte_joins = std::mem::take(&mut q.joins);
            q = main;
            q.tables.extend(cte_tables);
            q.conditions.extend(cte_conditions);
            q.joins.extend(cte_joins);
        }
        q
    }

    fn select(&self, start: usize, end: usize) -> Query {
        let mut q = Query::default();
        let mut i = start;
        if self.kw(i, "SELECT") {
            i += 1;
        }
        if self.kw(i, "SINGLE") {
            q.single = true;
            i += 1;
        }
        if let Some(j) = self.seq(i, end, "FOR UPDATE") {
            i = j;
        }
        if self.kw(i, "DISTINCT") {
            q.distinct = true;
            i += 1;
        }

        // split into clauses at depth of the SELECT
        let base = self.depth.get(start).copied().unwrap_or(0);
        let mut clauses: Vec<(Clause, usize, usize)> = Vec::new(); // clause, body start, body end
        let list_start = i;
        let mut list_end = end;
        let mut j = i;
        while j < end {
            if self.depth[j] == base {
                if let Some((clause, body)) = self.clause_at(j, end) {
                    if clauses.is_empty() {
                        list_end = j;
                    }
                    if let Some(last) = clauses.last_mut() {
                        last.2 = j;
                    }
                    clauses.push((clause, body, end));
                    if clause == Clause::SetOperation {
                        break;
                    }
                    j = body;
                    continue;
                }
            }
            j += 1;
        }

        let mut set_rest: Option<usize> = None;
        for (clause, s, e) in &clauses {
            let (s, e) = (*s, *e);
            match clause {
                Clause::Fields => q.fields = self.select_list(s, e, &mut q.dynamic),
                Clause::From => self.from(&mut q, s, e),
                Clause::Into | Clause::Appending => {
                    q.target = Some(self.target(s, e, *clause == Clause::Appending))
                }
                Clause::Where => {
                    q.where_text = Some(self.text(s, e));
                    let mut conditions = Vec::new();
                    self.conditions(&mut q, s, e, &mut conditions);
                    q.conditions.extend(conditions);
                }
                Clause::Having => {
                    let mut having = Vec::new();
                    self.conditions(&mut q, s, e, &mut having);
                    q.having = having;
                }
                Clause::GroupBy => {
                    q.group_by = self.column_list(s, e, &mut q.dynamic);
                }
                Clause::OrderBy => {
                    q.order_by = self.order_by(s, e, &mut q.dynamic);
                }
                Clause::UpTo => {
                    let oe = self.operand_end(s, e);
                    q.up_to = Some(self.text(s, oe));
                }
                Clause::ForAllEntries => {
                    let oe = self.operand_end(s, e);
                    q.for_all_entries = Some(self.text(s, oe));
                }
                Clause::BypassingBuffer => q.bypassing_buffer = true,
                Clause::ClientSpecified | Clause::UsingClient => q.client_specified = true,
                Clause::Connection => {
                    let oe = self.operand_end(s, e);
                    q.connection = Some(self.text(s, oe));
                }
                Clause::SetOperation => set_rest = Some(s),
                Clause::Offset | Clause::PackageSize | Clause::ForUpdate | Clause::Ignored => {}
            }
        }
        if clauses.iter().all(|c| c.0 != Clause::Fields) {
            q.fields = self.select_list(list_start, list_end, &mut q.dynamic);
        }
        resolve(&mut q);

        if let Some(rest) = set_rest {
            // UNION [ALL|DISTINCT] SELECT ... or ( SELECT ... )
            let mut k = rest;
            while k < end && (self.kw(k, "ALL") || self.kw(k, "DISTINCT")) {
                k += 1;
            }
            let other = self.query_expr(k, end);
            q.set_operation = true;
            q.tables.extend(other.tables);
            q.joins.extend(other.joins);
            q.conditions.extend(other.conditions);
            q.target = q.target.or(other.target);
            q.dynamic |= other.dynamic;
        }
        q
    }

    /// Subquery tables count as such, joins are kept; returns the conditions
    /// of the subquery for the caller to place.
    #[must_use]
    fn merge_subquery(&self, q: &mut Query, sub: Query) -> Vec<m::DbCondition> {
        for mut t in sub.tables {
            if t.role != m::DbTableRole::Cte && t.role != m::DbTableRole::InternalTable {
                t.role = m::DbTableRole::Subquery;
            }
            q.tables.push(t);
        }
        for mut j in sub.joins {
            j.table.role = m::DbTableRole::Subquery;
            q.joins.push(j);
        }
        q.dynamic |= sub.dynamic;
        sub.conditions
    }

    fn select_list(&self, s: usize, e: usize, dynamic: &mut bool) -> Vec<m::DbField> {
        if s >= e {
            return Vec::new();
        }
        let base = self.depth[s];
        let comma_separated = (s..e).any(|i| self.depth[i] == base && self.t[i].text == ",");
        let mut items: Vec<(usize, usize)> = Vec::new();
        if comma_separated {
            let mut start = s;
            for i in s..e {
                if self.depth[i] == base && self.t[i].text == "," {
                    items.push((start, i));
                    start = i + 1;
                }
            }
            items.push((start, e));
        } else {
            let mut i = s;
            while i < e {
                let mut j = self.operand_end(i, e);
                if self.kw(i, "CASE") {
                    j = (i..e)
                        .find(|k| self.kw(*k, "END") && self.depth[*k] == base)
                        .map_or(e, |k| k + 1);
                }
                if self.kw(j, "AS") && j + 1 < e {
                    j = self.operand_end(j + 1, e);
                }
                items.push((i, j));
                i = j;
            }
        }
        items
            .into_iter()
            .filter(|(a, b)| a < b)
            .map(|(a, b)| {
                if self.is_dynamic(a, b) && self.close(a, b) + 1 == b {
                    *dynamic = true;
                }
                let as_at = (a..b)
                    .rev()
                    .find(|k| self.depth[*k] == base && self.kw(*k, "AS"));
                let (expr_end, alias) = match as_at {
                    Some(k) if k > a => (k, Some(self.text(k + 1, b))),
                    _ => (b, None),
                };
                let aggregate = AGGREGATES
                    .iter()
                    .find(|f| self.kw(a, f) && a + 1 < expr_end && self.is_open(a + 1))
                    .map(|f| f.to_ascii_lowercase());
                m::DbField {
                    text: self.text(a, expr_end),
                    alias,
                    aggregate,
                    columns: self.columns_in(a, expr_end),
                }
            })
            .collect()
    }

    /// Column references in a token range; host variables, literals,
    /// function names and subqueries are skipped.
    fn columns_in(&self, s: usize, e: usize) -> Vec<m::DbColumn> {
        let mut out: Vec<m::DbColumn> = Vec::new();
        let mut i = s;
        while i < e {
            if self.is_subquery(i, e) {
                i = self.close(i, e) + 1;
                continue;
            }
            let t = self.t[i];
            if t.kind == TokenKind::At {
                i = self.operand_end(i, e);
                continue;
            }
            if t.kind == TokenKind::Identifier {
                let next_is_call = i + 1 < e && self.is_open(i + 1) && !self.t[i + 1].ws_before;
                let after_dash = i > s && self.t[i - 1].kind == TokenKind::Dash && !t.ws_before;
                let next_is_dash =
                    i + 1 < e && self.t[i + 1].kind == TokenKind::Dash && !self.t[i + 1].ws_before;
                if !next_is_call && !after_dash && !next_is_dash {
                    if let Some(c) = column(&t.text) {
                        if !out.contains(&c) {
                            out.push(c);
                        }
                    }
                }
            }
            i += 1;
        }
        out
    }

    fn column_list(&self, s: usize, e: usize, dynamic: &mut bool) -> Vec<m::DbColumn> {
        if self.is_dynamic(s, e) {
            *dynamic = true;
        }
        self.columns_in(s, e)
    }

    fn order_by(&self, s: usize, e: usize, dynamic: &mut bool) -> Vec<String> {
        if s >= e {
            return Vec::new();
        }
        if self.is_dynamic(s, e) {
            *dynamic = true;
        }
        let base = self.depth[s];
        if (s..e).any(|x| self.depth[x] == base && self.t[x].text == ",") {
            let mut out = Vec::new();
            let mut start = s;
            for i in s..=e {
                if i == e || (self.depth[i] == base && self.t[i].text == ",") {
                    if start < i {
                        out.push(self.text(start, i));
                    }
                    start = i + 1;
                }
            }
            return out;
        }
        // classic syntax: elements separated by blanks
        let mut out = Vec::new();
        let mut i = s;
        while i < e {
            let mut j = if self.seq(i, e, "PRIMARY KEY").is_some() {
                i + 2
            } else {
                self.operand_end(i, e)
            };
            if self.kw(j, "ASCENDING") || self.kw(j, "DESCENDING") {
                j += 1;
            }
            out.push(self.text(i, j));
            i = j;
        }
        out
    }

    fn target(&self, s: usize, e: usize, appending: bool) -> m::DbTarget {
        let mut i = s;
        let mut corresponding = false;
        let mut kind = m::DbTargetKind::Structure;
        if let Some(j) = self.seq(i, e, "CORRESPONDING FIELDS OF") {
            corresponding = true;
            i = j;
        }
        if self.kw(i, "TABLE") {
            kind = m::DbTargetKind::Table;
            i += 1;
        }
        let mut targets = Vec::new();
        if i < e && self.is_open(i) {
            // INTO ( a, b, c )
            let close = self.close(i, e);
            kind = m::DbTargetKind::List;
            let mut k = i + 1;
            while k < close {
                if self.t[k].text == "," {
                    k += 1;
                    continue;
                }
                let j = self.operand_end(k, close);
                targets.push(self.text(k, j));
                k = j;
            }
        } else if i < e {
            // NEW @DATA(x) creates the target
            let operand = if self.kw(i, "NEW") { i + 1 } else { i };
            let j = self.operand_end(operand.min(e - 1), e);
            targets.push(self.text(i, j));
        }
        m::DbTarget {
            targets,
            kind,
            appending,
            corresponding,
        }
    }

    /// FROM source [AS alias] { [INNER|LEFT|RIGHT|CROSS] JOIN source [AS alias] ON cond }
    fn from(&self, q: &mut Query, s: usize, e: usize) {
        let mut i = s;
        let mut first = true;
        while i < e {
            if let Some((kind, after)) = self.join_keyword(i, e) {
                let (mut table, next) = self.source_at(q, after, e, m::DbTableRole::Join);
                let mut on = Vec::new();
                let mut k = next;
                if self.kw(k, "ON") {
                    let cond_end = (k + 1..e)
                        .find(|x| {
                            self.depth[*x] <= self.depth[k] && self.join_keyword(*x, e).is_some()
                        })
                        .unwrap_or(e);
                    self.conditions(q, k + 1, cond_end, &mut on);
                    k = cond_end;
                }
                if let Some(t) = table.take() {
                    q.tables.push(t.clone());
                    q.joins.push(m::DbJoin { kind, table: t, on });
                }
                i = k;
                continue;
            }
            if first {
                let (table, next) = self.source_at(q, i, e, m::DbTableRole::Main);
                if let Some(t) = table {
                    q.tables.push(t);
                }
                first = false;
                i = next;
                continue;
            }
            i += 1;
        }
    }

    fn join_keyword(&self, i: usize, e: usize) -> Option<(String, usize)> {
        let kind = if self.kw(i, "INNER") || self.kw(i, "JOIN") {
            "inner"
        } else if self.kw(i, "LEFT") {
            "left_outer"
        } else if self.kw(i, "RIGHT") {
            "right_outer"
        } else if self.kw(i, "CROSS") {
            "cross"
        } else if self.kw(i, "FULL") {
            "full_outer"
        } else {
            return None;
        };
        // skip OUTER, cardinality like MANY TO ONE, up to JOIN
        let j = (i..e.min(i + 8)).find(|k| self.kw(*k, "JOIN"))?;
        Some((kind.to_string(), j + 1))
    }

    /// One data source; parenthesized joins are flattened into `q`.
    fn source_at(
        &self,
        q: &mut Query,
        i: usize,
        e: usize,
        role: m::DbTableRole,
    ) -> (Option<m::DbTable>, usize) {
        let mut i = i;
        // join hints like #TAG, association cardinality like MANY TO ONE
        while i + 1 < e
            && (self.t[i].text.starts_with('#')
                || ["EXACT", "ONE", "MANY", "TO"].iter().any(|w| self.kw(i, w)))
        {
            i += 1;
        }
        if i >= e {
            return (None, e);
        }
        if self.t[i].text.to_ascii_uppercase().starts_with("HIERARCHY")
            && i + 1 < e
            && self.is_open(i + 1)
        {
            return self.hierarchy(q, i, e, role);
        }
        if self.is_open(i) && !self.is_dynamic(i, e) {
            // ( a INNER JOIN b ON ... )
            let close = self.close(i, e);
            let before = q.tables.len();
            self.from(q, i + 1, close);
            if role == m::DbTableRole::Join && q.tables.len() > before {
                // the first table inside the group is the one being joined
                let t = q.tables.remove(before);
                return (Some(m::DbTable { role, ..t }), self.alias_end(close + 1, e));
            }
            return (None, close + 1);
        }
        let (name, dynamic, mut next) = if self.is_open(i) {
            let close = self.close(i, e);
            (self.text(i + 1, close), true, close + 1)
        } else {
            let j = self.operand_end(i, e);
            // CDS parameters: zview( p = @x ) keep the view name only
            let name_end = (i..j).find(|k| self.is_open(*k)).unwrap_or(j);
            (self.text(i, name_end), false, j)
        };
        // CDS path expression: entity\_assoc
        let (name, path) = match name.find('\\') {
            Some(p) if p > 0 => (name[..p].to_string(), Some(name[p..].to_string())),
            _ => (name, None),
        };
        if dynamic {
            q.dynamic = true;
        }
        let mut alias = None;
        if self.kw(next, "AS") && next + 1 < e {
            let j = self.operand_end(next + 1, e);
            alias = Some(self.text(next + 1, j));
            next = j;
        }
        let role = if name.starts_with('@') {
            m::DbTableRole::InternalTable
        } else if name.starts_with('+') {
            m::DbTableRole::Cte
        } else {
            role
        };
        let name = name.trim_start_matches('@').to_string();
        (
            Some(m::DbTable {
                name,
                alias,
                path,
                dynamic,
                resolved: None,
                role,
            }),
            next,
        )
    }

    /// HIERARCHY( SOURCE src ... ), HIERARCHY_DESCENDANTS( SOURCE src ... ) and
    /// the other hierarchy functions: the data source is the table after
    /// SOURCE, aggregate functions may join a further table.
    fn hierarchy(
        &self,
        q: &mut Query,
        i: usize,
        e: usize,
        role: m::DbTableRole,
    ) -> (Option<m::DbTable>, usize) {
        let open = i + 1;
        let close = self.close(open, e);
        let mut source = None;
        let mut k = open + 1;
        while k < close {
            if self.kw(k, "SOURCE") && k + 1 < close {
                if self.is_query(k + 1, close) {
                    let sub_close = self.close(k + 1, close);
                    let sub = self.query_expr(k + 2, sub_close);
                    let conditions = self.merge_subquery(q, sub);
                    q.conditions.extend(conditions);
                    k = sub_close + 1;
                    continue;
                }
                let (t, next) = self.source_at(q, k + 1, close, role);
                source = source.or(t);
                k = next;
                continue;
            }
            if self.kw(k, "JOIN") && k + 1 < close {
                let (t, next) = self.source_at(q, k + 1, close, m::DbTableRole::Join);
                if let Some(t) = t {
                    q.tables.push(t);
                }
                k = next;
                continue;
            }
            k += 1;
        }
        let mut next = close + 1;
        let mut alias = None;
        if self.kw(next, "AS") && next + 1 < e {
            let j = self.operand_end(next + 1, e);
            alias = Some(self.text(next + 1, j));
            next = j;
        }
        (
            source.map(|t| m::DbTable {
                alias: alias.or(t.alias),
                ..t
            }),
            next,
        )
    }

    fn alias_end(&self, i: usize, e: usize) -> usize {
        if self.kw(i, "AS") && i + 1 < e {
            self.operand_end(i + 1, e)
        } else {
            i
        }
    }

    /// Comparisons of a condition; subqueries are parsed and merged into `q`.
    fn conditions(&self, q: &mut Query, s: usize, e: usize, out: &mut Vec<m::DbCondition>) {
        let mut i = s;
        while i < e {
            if self.is_query(i, e) {
                let close = self.close(i, e);
                let sub = self.query_expr(i + 1, close);
                let conditions = self.merge_subquery(q, sub);
                out.extend(conditions);
                i = close + 1;
                continue;
            }
            if self.is_open(i) {
                if self.is_dynamic(i, e) {
                    let close = self.close(i, e);
                    // a dynamic condition stands alone, not as a comparison operand
                    let follows_logic = close + 1 >= e
                        || self.kw(close + 1, "AND")
                        || self.kw(close + 1, "OR")
                        || self.is_close(close + 1);
                    if follows_logic {
                        q.dynamic = true;
                        i = close + 1;
                        continue;
                    }
                } else {
                    i += 1;
                    continue;
                }
            }
            if self.is_close(i) || self.kw(i, "AND") || self.kw(i, "OR") || self.kw(i, "NOT") {
                i += 1;
                continue;
            }
            if self.kw(i, "EXISTS") && i + 1 < e && self.is_subquery(i + 1, e) {
                let close = self.close(i + 1, e);
                out.push(m::DbCondition {
                    left: String::new(),
                    operator: "EXISTS".to_string(),
                    right: Some(self.text(i + 1, close + 1)),
                    columns: Vec::new(),
                });
                let sub = self.select(i + 2, close);
                let conditions = self.merge_subquery(q, sub);
                out.extend(conditions);
                i = close + 1;
                continue;
            }

            let left_end = self.operand_end(i, e);
            let mut pending: Vec<m::DbCondition> = Vec::new();
            let Some((operator, mut k)) = self.operator(left_end, e) else {
                i = left_end;
                continue;
            };
            let right_start = k;
            let mut right: Option<String> = None;
            match operator.trim_start_matches("NOT ") {
                "IS NULL" | "IS NOT NULL" | "IS INITIAL" | "IS NOT INITIAL" => {}
                "BETWEEN" => {
                    k = self.operand_end(k, e);
                    if self.kw(k, "AND") {
                        k = self.operand_end(k + 1, e);
                    }
                    right = Some(self.text(right_start, k));
                }
                _ => {
                    if k < e && self.is_open(k) {
                        let close = self.close(k, e);
                        if self.is_query(k, e) {
                            let sub = self.query_expr(k + 1, close);
                            pending = self.merge_subquery(q, sub);
                        }
                        k = close + 1;
                    } else if k < e {
                        k = self.operand_end(k, e);
                        // arithmetic continuation: a = b + 1
                        while k + 1 < e
                            && matches!(self.t[k].text.as_str(), "+" | "-" | "*" | "/")
                            && self.t[k].ws_before
                        {
                            k = self.operand_end(k + 1, e);
                        }
                    }
                    if self.kw(k, "ESCAPE") {
                        k = self.operand_end(k + 1, e);
                    }
                    right = Some(self.text(right_start, k));
                }
            }
            let mut columns = self.columns_in(i, left_end);
            if columns.len() > 1 || self.t[i].kind == TokenKind::At {
                columns.retain(|c| c.qualifier.is_some());
            }
            if right_start < k && !self.is_subquery(right_start, k) {
                let strict = self.t.iter().any(|t| t.kind == TokenKind::At);
                for c in self.columns_in(right_start, k) {
                    // classic syntax has unmarked host variables on the right
                    if (c.qualifier.is_some() || strict) && !columns.contains(&c) {
                        columns.push(c);
                    }
                }
            }
            out.push(m::DbCondition {
                left: self.text(i, left_end),
                operator,
                right,
                columns,
            });
            out.append(&mut pending);
            i = k.max(left_end);
        }
    }

    /// Comparison operator at `i`; returns it normalized and the index after it.
    fn operator(&self, i: usize, e: usize) -> Option<(String, usize)> {
        if i >= e {
            return None;
        }
        let word = |k: usize| {
            self.t
                .get(k)
                .map(|t| t.text.to_ascii_uppercase())
                .unwrap_or_default()
        };
        let mut k = i;
        let mut not = false;
        if self.kw(k, "NOT") {
            not = true;
            k += 1;
        }
        let w = word(k);
        let op = if COMPARISONS.contains(&w.as_str()) {
            let mut op = w.clone();
            k += 1;
            // "= ANY ( SELECT ... )"
            if self.kw(k, "ANY") || self.kw(k, "ALL") || self.kw(k, "SOME") {
                op = format!("{op} {}", word(k));
                k += 1;
            }
            op
        } else if matches!(w.as_str(), "LIKE" | "IN" | "BETWEEN") {
            k += 1;
            w
        } else if w == "IS" {
            k += 1;
            let mut op = "IS".to_string();
            if self.kw(k, "NOT") {
                op.push_str(" NOT");
                k += 1;
            }
            op.push(' ');
            op.push_str(&word(k));
            k += 1;
            op
        } else {
            return None;
        };
        Some((if not { format!("NOT {op}") } else { op }, k))
    }

    // ------------------------------------------------------------ DML

    /// INSERT, UPDATE, MODIFY, DELETE.
    fn dml(&self, a: &mut m::DbAccess, operation: m::DbOperation, end: usize) {
        a.operation = operation;
        let mut q = Query::default();
        let mut i = 1;
        if self.kw(i, "INTO") || (operation == m::DbOperation::Delete && self.kw(i, "FROM")) {
            i += 1;
        }
        let (table, mut i2) = self.source_at(&mut q, i, end, m::DbTableRole::Main);
        if let Some(t) = table {
            q.tables.push(t);
        }
        let mut set = Vec::new();
        let mut source = None;
        while i2 < end {
            if self.depth[i2] != 0 {
                i2 += 1;
                continue;
            }
            if self.kw(i2, "FROM") || self.kw(i2, "VALUES") {
                let mut k = i2 + 1;
                let is_table = self.kw(k, "TABLE");
                if is_table {
                    k += 1;
                }
                if self.is_subquery(k, end) {
                    let close = self.close(k, end);
                    let sub = self.select(k + 1, close);
                    let conditions = self.merge_subquery(&mut q, sub);
                    q.conditions.extend(conditions);
                    source = Some(m::DbSource {
                        text: None,
                        is_table: false,
                        is_subquery: true,
                    });
                    i2 = close + 1;
                } else {
                    let j = self.operand_end(k, end);
                    source = Some(m::DbSource {
                        text: Some(self.text(k, j)),
                        is_table,
                        is_subquery: false,
                    });
                    i2 = j;
                }
                continue;
            }
            if self.kw(i2, "SET") {
                let set_end = (i2 + 1..end)
                    .find(|k| self.depth[*k] == 0 && self.kw(*k, "WHERE"))
                    .unwrap_or(end);
                set = self.assignments(i2 + 1, set_end, &mut q.dynamic);
                i2 = set_end;
                continue;
            }
            if self.kw(i2, "WHERE") {
                let where_end = (i2 + 1..end)
                    .find(|k| {
                        self.depth[*k] == 0
                            && (self.seq(*k, end, "UP TO").is_some()
                                || self.kw(*k, "OFFSET")
                                || self.seq(*k, end, "ORDER BY").is_some()
                                || self.kw(*k, "OPTIONS"))
                    })
                    .unwrap_or(end);
                q.where_text = Some(self.text(i2 + 1, where_end));
                let mut conditions = Vec::new();
                self.conditions(&mut q, i2 + 1, where_end, &mut conditions);
                q.conditions.extend(conditions);
                i2 = where_end;
                continue;
            }
            if let Some((clause, body)) = self.clause_at(i2, end) {
                match clause {
                    Clause::ClientSpecified | Clause::UsingClient => q.client_specified = true,
                    Clause::Connection => {
                        let j = self.operand_end(body, end);
                        q.connection = Some(self.text(body, j));
                    }
                    Clause::UpTo => {
                        let j = self.operand_end(body, end);
                        q.up_to = Some(self.text(body, j));
                    }
                    _ => {}
                }
                i2 = body;
                continue;
            }
            i2 += 1;
        }
        resolve(&mut q);
        self.apply_query(a, q);
        a.operation = operation;
        a.source = source;
        let table = a.tables.first().map(|t| t.name.clone());
        a.set = set
            .into_iter()
            .map(|(c, v)| m::DbAssignment {
                column: m::DbColumn {
                    table: c.table.or_else(|| table.clone()),
                    ..c
                },
                value: v,
            })
            .collect();
    }

    /// `col = value` pairs of UPDATE ... SET, comma or blank separated.
    fn assignments(&self, s: usize, e: usize, dynamic: &mut bool) -> Vec<(m::DbColumn, String)> {
        let mut out = Vec::new();
        let mut i = s;
        while i < e {
            if self.t[i].text == "," {
                i += 1;
                continue;
            }
            if self.is_dynamic(i, e) {
                *dynamic = true;
                i = self.close(i, e) + 1;
                continue;
            }
            let col_end = self.operand_end(i, e);
            if col_end >= e || self.t[col_end].text != "=" {
                i = col_end;
                continue;
            }
            let v_start = col_end + 1;
            let mut k = self.operand_end(v_start, e);
            while k + 1 < e && matches!(self.t[k].text.as_str(), "+" | "-" | "*" | "/") {
                k = self.operand_end(k + 1, e);
            }
            if let Some(c) = column(&self.text(i, col_end)) {
                out.push((c, self.text(v_start, k)));
            }
            i = k;
        }
        out
    }

    /// MERGE INTO target [AS a] USING source [AS b] ON cond WHEN ...
    fn merge(&self, a: &mut m::DbAccess, end: usize) {
        let mut q = Query::default();
        let i = if self.kw(1, "INTO") { 2 } else { 1 };
        let (target, next) = self.source_at(&mut q, i, end, m::DbTableRole::Main);
        if let Some(t) = target {
            q.tables.push(t);
        }
        if let Some(u) = (next..end).find(|k| self.depth[*k] == 0 && self.kw(*k, "USING")) {
            let (src, after) = self.source_at(&mut q, u + 1, end, m::DbTableRole::MergeSource);
            if let Some(t) = src {
                q.tables.push(t);
            }
            if self.kw(after, "ON") {
                let cond_end = (after + 1..end)
                    .find(|k| self.depth[*k] == 0 && self.kw(*k, "WHEN"))
                    .unwrap_or(end);
                let mut on = Vec::new();
                self.conditions(&mut q, after + 1, cond_end, &mut on);
                q.conditions.extend(on);
            }
        }
        resolve(&mut q);
        self.apply_query(a, q);
        a.operation = m::DbOperation::Merge;
    }

    fn apply_query(&self, a: &mut m::DbAccess, q: Query) {
        a.tables = q.tables;
        a.fields = q.fields;
        a.joins = q.joins;
        a.conditions = q.conditions;
        a.where_text = q.where_text;
        a.group_by = q.group_by;
        a.having = q.having;
        a.order_by = q.order_by;
        a.target = q.target;
        a.for_all_entries = q.for_all_entries;
        a.single = q.single;
        a.distinct = q.distinct;
        a.up_to = q.up_to;
        a.set_operation = q.set_operation;
        a.dynamic = q.dynamic;
        a.client_specified = q.client_specified;
        a.bypassing_buffer = q.bypassing_buffer;
        a.connection = q.connection;
    }
}

/// A column reference from one token: `name`, `qualifier~name`, `qualifier~*`.
fn column(text: &str) -> Option<m::DbColumn> {
    let first = text.chars().next()?;
    if !(first.is_alphabetic() || first == '_' || first == '/' || first == '+' || first == '*') {
        return None;
    }
    if text.contains('-') && !text.contains('~') {
        return None;
    }
    let upper = text.to_ascii_uppercase();
    if SQL_WORDS.contains(&upper.as_str()) || AGGREGATES.contains(&upper.as_str()) {
        return None;
    }
    match text.split_once('~') {
        Some((qualifier, name)) => Some(m::DbColumn {
            name: name.to_string(),
            qualifier: Some(qualifier.to_string()),
            table: None,
        }),
        None if first == '*' => None,
        None => Some(m::DbColumn {
            name: text.to_string(),
            qualifier: None,
            table: None,
        }),
    }
}

/// Fills `DbColumn::table` from aliases, or from the only table of the query.
fn resolve(q: &mut Query) {
    let scope: Vec<&m::DbTable> = q
        .tables
        .iter()
        .filter(|t| {
            matches!(
                t.role,
                m::DbTableRole::Main | m::DbTableRole::Join | m::DbTableRole::Cte
            )
        })
        .collect();
    let mut names: HashMap<String, String> = HashMap::new();
    for t in &scope {
        names.insert(t.name.to_ascii_lowercase(), t.name.clone());
        if let Some(a) = &t.alias {
            names.insert(a.to_ascii_lowercase(), t.name.clone());
        }
    }
    let only = (scope.len() == 1 && !scope[0].dynamic).then(|| scope[0].name.clone());
    let fix = |c: &mut m::DbColumn| {
        if c.table.is_some() {
            return;
        }
        c.table = match &c.qualifier {
            Some(qual) => names.get(&qual.to_ascii_lowercase()).cloned(),
            None => only.clone(),
        };
    };
    for f in q.fields.iter_mut() {
        f.columns.iter_mut().for_each(fix);
    }
    for c in q.conditions.iter_mut().chain(q.having.iter_mut()) {
        c.columns.iter_mut().for_each(fix);
    }
    for j in q.joins.iter_mut() {
        for c in j.on.iter_mut() {
            c.columns.iter_mut().for_each(fix);
        }
    }
    q.group_by.iter_mut().for_each(fix);
}

#[cfg(test)]
mod tests;
