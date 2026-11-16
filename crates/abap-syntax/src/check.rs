//! Syntax errors in statements whose kind is known. Checks for single errors
//! come first, with a message of their own, like a keyword as a column
//! (`SELECT WHERE FROM t`) or parameter sections out of order; they cover
//! errors seen in abaplint's test cases (<https://github.com/abaplint/abaplint>,
//! MIT), SAP's documentation and the syntax check of an SAP system. Then the
//! statement is matched against the grammar of its kind, see `src/grammar`.
//!
//! Each check is held against the corpus: no statement abaplint accepts may
//! be reported.

use crate::expr::{ExprKind, Head, Link};
use crate::operands::{operands, Operand};
use crate::{File, Statement, StatementKind as K, Token, TokenKind};

/// A syntax error of a statement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SyntaxError {
    /// position in [`Statement::tokens`](crate::Statement::tokens) of the
    /// token where the error is
    pub token: usize,
    pub message: &'static str,
}

/// The first syntax error found in the statement.
pub fn syntax_error(file: &File, s: &Statement) -> Option<SyntaxError> {
    if s.kind == K::NativeSQL {
        return crate::sqlscript::statement_error(file, s)
            .map(|token| SyntaxError {
                token,
                message: "does not match the SQLScript grammar",
            })
            .or_else(|| {
                crate::sqlscript::statement_using_error(file, s)
                    .map(|(token, message)| SyntaxError { token, message })
            });
    }
    if s.kind == K::MethodImplementation {
        if let Some(token) = crate::sqlscript::unused_using(file, s) {
            return error(token, "an object after USING is not used in the body");
        }
    }
    if matches!(s.kind, K::Comment | K::Unknown) {
        return None;
    }
    let c = Check::new(file, s);
    let found = match s.kind {
        K::Select | K::SelectLoop | K::With | K::WithLoop | K::OpenCursor => c.select(),
        K::MethodDef => c.method_def(),
        K::Data | K::Static | K::ClassData | K::Type => c.data_type(),
        K::Concatenate => c.concatenate(file, s),
        K::DeleteInternal => c.delete_internal(),
        K::InsertInternal => c.insert_internal(),
        K::Describe => c.describe(file, s),
        K::Perform => c.perform(file, s),
        K::Raise => c.raise(),
        K::UpdateDatabase => c.update_database(),
        K::Append => c.append(),
        K::Format | K::EditorCall => c.data_objects_only(file, s),
        K::CallTransformation => c.call_transformation(),
        K::CreateData => c.create_data(),
        K::MergeDatabase => c.merge(),
        K::ReadTable => c.read_table(),
        K::Message => c.message(file, s),
        K::Form => c.form_parameters(),
        K::GetPermissions => c.get_permissions(),
        K::Parameter | K::SelectOption => c.selection_name(),
        _ => None,
    };
    found
        .or_else(|| c.strict_sql(s.kind))
        .or_else(|| c.dynamic_component())
        .or_else(|| c.builtin_without_argument(file))
        .or_else(|| {
            // the first token of a part of a chain follows the colon or a
            // comma, which separate like a blank: `WRITE: (7) a ,(13) b.` is
            // `WRITE (13) b.`, as the SAP_BASIS 7.50 system has it
            let first = chain_part_start(file, s).filter(|&i| !c.t[i].ws_before);
            let separated = first.map(|i| Token {
                ws_before: true,
                ..c.t[i].clone()
            });
            let mut tokens = c.t.clone();
            if let (Some(i), Some(t)) = (first, separated.as_ref()) {
                tokens[i] = t;
            }
            crate::grammar::mismatch(&tokens, s.kind)
                .and_then(|token| error(token, "does not match the statement grammar"))
        })
}

/// The position in the statement of the first token after the colon of a
/// chained statement or a comma of the chain.
fn chain_part_start(file: &File, s: &Statement) -> Option<usize> {
    let colon = s.colon?;
    let pos = s.tokens.iter().position(|&t| t > colon)?;
    let before = file.tokens().get(s.tokens[pos].checked_sub(1)?)?;
    matches!(before.text.as_str(), "," | ":").then_some(pos)
}

struct Check<'a> {
    t: Vec<&'a Token>,
    w: Vec<String>,
    depth: Vec<u32>,
    /// a database statement in a strict mode of ABAP SQL
    strict: bool,
}

fn error(token: usize, message: &'static str) -> Option<SyntaxError> {
    Some(SyntaxError { token, message })
}

/// What ends a list of the SELECT statement.
const LIST_END: [&str; 23] = [
    "FROM",
    "INTO",
    "APPENDING",
    "WHERE",
    "GROUP",
    "HAVING",
    "ORDER",
    "UNION",
    "INTERSECT",
    "EXCEPT",
    "UP",
    "OFFSET",
    "BYPASSING",
    "CONNECTION",
    "FOR",
    "PACKAGE",
    "CLIENT",
    "USING",
    "WITH",
    "PRIVILEGED",
    "INDICATORS",
    "EXTENDED",
    "%_HINTS",
];
/// Words inside an element of a list, not columns.
const LIST_KEYWORDS: [&str; 26] = [
    "AS",
    "CASE",
    "WHEN",
    "THEN",
    "ELSE",
    "END",
    "IS",
    "NOT",
    "NULL",
    "AND",
    "OR",
    "IN",
    "LIKE",
    "BETWEEN",
    "ESCAPE",
    "DISTINCT",
    "OVER",
    "ORDER",
    "BY",
    "PARTITION",
    "ROWS",
    "ASCENDING",
    "DESCENDING",
    "DIV",
    "MOD",
    "SINGLE",
];
/// Words of ABAP SQL that cannot name a column.
const SQL_RESERVED: [&str; 17] = [
    "APPENDING",
    "AS",
    "CASE",
    "CONNECTION",
    "EXCEPT",
    "EXISTS",
    "FOR",
    "FROM",
    "HAVING",
    "INSERT",
    "INTERSECT",
    "INTO",
    "MODIFY",
    "NOT",
    "UNION",
    "WHEN",
    "WHERE",
];

impl<'a> Check<'a> {
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
        let strict = crate::release::sql_strict_mode(file, s).is_some();
        Check {
            t,
            w,
            depth,
            strict,
        }
    }

    fn word(&self, i: usize) -> &str {
        self.w.get(i).map(String::as_str).unwrap_or("")
    }

    /// Positions of the word at depth 0.
    fn top(&self, word: &str) -> Vec<usize> {
        (0..self.w.len())
            .filter(|&i| self.depth[i] == 0 && self.word(i) == word)
            .collect()
    }

    /// A name, not an operator like `/`, `&&` or `=`, which are identifiers
    /// to the lexer too, and not a number.
    fn is_name(&self, k: usize) -> bool {
        let Some(t) = self.t.get(k) else {
            return false;
        };
        let text = t.text.as_str();
        t.kind == TokenKind::Identifier
            && (text.starts_with(|c: char| c.is_ascii_alphabetic() || c == '_')
                || (text.starts_with('/') && text.len() > 1))
    }

    /// The rules of the strict modes of ABAP SQL that need no dictionary,
    /// as SAP documents them (`abenabap_sql_strictmode_740_sp05`) and the
    /// syntax check of the SAP ABAP Cloud Developer Trial 2025 reports
    /// them: lists separated by commas after GROUP BY, ORDER BY and SET,
    /// `@` in front of host variables, and no SELECT without INTO.
    fn strict_sql(&self, kind: K) -> Option<SyntaxError> {
        if !self.strict {
            return None;
        }
        self.by_list("GROUP")
            .or_else(|| self.by_list("ORDER"))
            .or_else(|| self.set_list(kind))
            .or_else(|| self.host_variable_without_at(kind))
            .or_else(|| self.select_without_into(kind))
    }

    /// The columns after GROUP BY or ORDER BY, separated by blanks.
    fn by_list(&self, word: &str) -> Option<SyntaxError> {
        /// words of the list that are no columns
        const KEYWORDS: [&str; 11] = [
            "ASCENDING",
            "DESCENDING",
            "PRIMARY",
            "KEY",
            "NULLS",
            "FIRST",
            "LAST",
            "GROUPING",
            "SETS",
            "ROLLUP",
            "CUBE",
        ];
        for k in 0..self.w.len() {
            if self.word(k) != word || self.word(k + 1) != "BY" {
                continue;
            }
            // the list of a query or subquery, not of `OVER( ORDER BY ... )`
            // or `HIERARCHY( ... SIBLINGS ORDER BY ... )`
            let d = self.depth[k];
            if d > 0 {
                let open = (0..k).rev().find(|&j| self.depth[j] < d);
                if !open.is_some_and(|o| self.word(o + 1) == "SELECT") {
                    continue;
                }
            }
            let stop = (k + 2..self.w.len())
                .find(|&j| {
                    self.depth[j] < d
                        || (self.depth[j] == d
                            && (LIST_END.contains(&self.word(j)) || self.word(j) == "."))
                })
                .unwrap_or(self.w.len());
            let column = |j: usize| {
                self.is_name(j)
                    && !KEYWORDS.contains(&self.word(j))
                    && !LIST_KEYWORDS.contains(&self.word(j))
            };
            // the end of an element: a column, or `a DESCENDING`, `a NULLS LAST`
            let element_end = |j: usize| {
                column(j)
                    || matches!(self.word(j), "ASCENDING" | "DESCENDING")
                    || (matches!(self.word(j), "FIRST" | "LAST") && self.word(j - 1) == "NULLS")
            };
            for j in k + 3..stop {
                if self.depth[j] == d
                    && self.depth[j - 1] == d
                    && self.t[j].ws_before
                    && element_end(j - 1)
                    && column(j)
                {
                    let message = if word == "GROUP" {
                        "GROUP BY list without commas in strict mode"
                    } else {
                        "ORDER BY list without commas in strict mode"
                    };
                    return error(j, message);
                }
            }
        }
        None
    }

    /// The assignments after SET of UPDATE, separated by blanks.
    fn set_list(&self, kind: K) -> Option<SyntaxError> {
        if kind != K::UpdateDatabase {
            return None;
        }
        let set = self.top("SET").first().copied()?;
        let end = self.top("WHERE").first().copied().unwrap_or(self.w.len());
        let assignments: Vec<usize> = (set + 1..end)
            .filter(|&k| self.depth[k] == 0 && self.word(k) == "=")
            .collect();
        for pair in assignments.windows(2) {
            let comma = (pair[0]..pair[1]).any(|k| self.depth[k] == 0 && self.word(k) == ",");
            if !comma {
                return error(pair[1] - 1, "SET list without commas in strict mode");
            }
        }
        None
    }

    /// A host variable without `@`: the targets after INTO and APPENDING,
    /// the work area or table of a write statement, the operand of UP TO,
    /// OFFSET and PACKAGE SIZE, the table of FOR ALL ENTRIES, and a
    /// component of a structure (`wa-field`), which cannot be a column.
    /// Dynamic tokens like `(wa-where)` and the operands inside a host
    /// expression `@( ... )` are ABAP, without `@`.
    fn host_variable_without_at(&self, kind: K) -> Option<SyntaxError> {
        const MESSAGE: &str = "host variable without @ in strict mode";
        let n = self.w.len();
        // an operand at `k` that must be a host variable
        let unescaped = |k: usize| self.is_name(k) && !self.word(k).starts_with('\\');
        let query = matches!(
            kind,
            K::Select | K::SelectLoop | K::With | K::WithLoop | K::FetchNextCursor
        );
        for k in 0..n {
            if self.depth[k] != 0 {
                continue;
            }
            let operand = match self.word(k) {
                "INTO" | "APPENDING" if query => {
                    let mut j = k + 1;
                    while matches!(
                        self.word(j),
                        "CORRESPONDING" | "FIELDS" | "OF" | "TABLE" | "NEW"
                    ) {
                        j += 1;
                    }
                    if self
                        .t
                        .get(j)
                        .is_some_and(|t| t.kind == TokenKind::ParenLeft)
                    {
                        // INTO (a, b): each element
                        let inner = self.depth[j] + 1;
                        let close = (j + 1..n).find(|&i| self.depth[i] < inner).unwrap_or(n);
                        let mut first = std::iter::once(j + 1).chain(
                            (j + 1..close)
                                .filter(|&i| self.depth[i] == inner && self.word(i) == ",")
                                .map(|i| i + 1),
                        );
                        if let Some(e) = first.find(|&e| unescaped(e)) {
                            return error(e, MESSAGE);
                        }
                        continue;
                    }
                    Some(j)
                }
                // the work area or table of INSERT, UPDATE, MODIFY, DELETE;
                // not `DELETE FROM dbtab` or `MAPPING FROM ENTITY`
                "FROM" | "VALUES"
                    if matches!(
                        kind,
                        K::InsertDatabase
                            | K::UpdateDatabase
                            | K::ModifyDatabase
                            | K::DeleteDatabase
                    ) && k > 1
                        && self.word(k - 1) != "MAPPING" =>
                {
                    Some(if self.word(k + 1) == "TABLE" {
                        k + 2
                    } else {
                        k + 1
                    })
                }
                "TO" if k > 0 && self.word(k - 1) == "UP" => Some(k + 1),
                "SIZE" if k > 0 && self.word(k - 1) == "PACKAGE" => Some(k + 1),
                // UP TO n ROWS OFFSET o; a column may be named OFFSET
                "OFFSET" if k > 0 && self.word(k - 1) == "ROWS" => Some(k + 1),
                "IN" if k > 1 && self.word(k - 1) == "ENTRIES" && self.word(k - 2) == "ALL" => {
                    Some(k + 1)
                }
                _ => None,
            };
            if let Some(e) = operand.filter(|&e| unescaped(e)) {
                return error(e, MESSAGE);
            }
        }
        // `@( ... )`: ABAP inside
        let mut host_expression = vec![false; n];
        for k in 0..n {
            let open = k + 1;
            if self.t[k].kind == TokenKind::At
                && self
                    .t
                    .get(open)
                    .is_some_and(|t| t.kind == TokenKind::ParenLeft && !t.ws_before)
            {
                let close = (open + 1..n)
                    .find(|&i| self.depth[i] <= self.depth[open])
                    .unwrap_or(n);
                host_expression[open..close]
                    .iter_mut()
                    .for_each(|x| *x = true);
            }
        }
        for k in 0..n.saturating_sub(2) {
            let dash = self.t[k + 1];
            let component = unescaped(k)
                && dash.kind == TokenKind::Dash
                && !dash.ws_before
                && !dash.ws_after
                && self.is_name(k + 2);
            if !component || host_expression[k] {
                continue;
            }
            // back to the start of `a-b-c`, `me->tab-comp`, `cl=>const-comp`
            let mut start = k;
            while start >= 2
                && matches!(
                    self.t[start - 1].kind,
                    TokenKind::Dash | TokenKind::InstanceArrow | TokenKind::StaticArrow
                )
                && !self.t[start - 1].ws_before
                && !self.t[start - 1].ws_after
            {
                start -= 2;
            }
            // `@wa-field`, `(dynamic-token)`, an alias `AS a-b`
            let before = start.checked_sub(1).map(|i| self.t[i]);
            let escaped = before.is_some_and(|t| {
                t.kind == TokenKind::At
                    || (t.kind == TokenKind::ParenLeft && !t.ws_after)
                    || t.text.eq_ignore_ascii_case("AS")
            });
            if !escaped {
                return error(k, MESSAGE);
            }
        }
        None
    }

    /// The obsolete short form without INTO, into the work area of TABLES.
    fn select_without_into(&self, kind: K) -> Option<SyntaxError> {
        if !matches!(kind, K::Select | K::SelectLoop) {
            return None;
        }
        let into = (0..self.w.len())
            .any(|k| self.depth[k] == 0 && matches!(self.word(k), "INTO" | "APPENDING"));
        // `SELECT COUNT( * ) FROM ...` sets only sy-dbcnt
        let words: String = self.w.iter().map(String::as_str).collect();
        if into || words.contains("COUNT(*)") {
            return None;
        }
        error(0, "SELECT without INTO in strict mode")
    }

    fn select(&self) -> Option<SyntaxError> {
        for i in 0..self.w.len() {
            // `SELECT select FROM`: a column, SAP reports it as unknown
            let column = i > 0 && matches!(self.word(i - 1), "SELECT" | "SINGLE");
            if self.word(i) != "SELECT" || column {
                continue;
            }
            // SELECT [SINGLE] <keyword> FROM
            let mut k = i + 1;
            if self.word(k) == "SINGLE" {
                k += 1;
            }
            if SQL_RESERVED.contains(&self.word(k)) && self.word(k + 1) == "FROM" {
                return error(k, "keyword as a column");
            }
            // a subquery reads a set, never a single row
            if self.depth[i] > 0 && i > 0 && self.word(i - 1) == "(" && self.word(i + 1) == "SINGLE"
            {
                return error(i + 1, "SINGLE in a subquery");
            }
            if let Some(e) = self.single_order_by(i).or_else(|| self.select_list(i)) {
                return Some(e);
            }
        }
        if let Some(e) = self.dynamic_aggregate() {
            return Some(e);
        }
        let into = self.top("INTO");
        if into.len() > 1 {
            return error(into[1], "INTO twice");
        }
        if let Some(&size) = self.top("PACKAGE").first() {
            // INTO TABLE, INTO CORRESPONDING FIELDS OF TABLE, before or after
            let table = into
                .iter()
                .chain(self.top("APPENDING").iter())
                .any(|&i| (i + 1..(i + 5).min(self.w.len())).any(|k| self.word(k) == "TABLE"));
            if !table && self.word(size + 1) == "SIZE" {
                return error(size, "PACKAGE SIZE without a table");
            }
        }
        None
    }

    /// The end of the query of the SELECT at `select`: where its
    /// parenthesis closes, or the end of the statement.
    fn query_end(&self, select: usize) -> usize {
        let d = self.depth[select];
        (select + 1..self.w.len())
            .find(|&k| self.depth[k] < d)
            .unwrap_or(self.w.len())
    }

    /// `SELECT SINGLE` takes no ORDER BY, before or after INTO.
    fn single_order_by(&self, select: usize) -> Option<SyntaxError> {
        if self.word(select + 1) != "SINGLE" {
            return None;
        }
        let d = self.depth[select];
        (select + 2..self.query_end(select))
            .find(|&k| self.depth[k] == d && self.word(k) == "ORDER" && self.word(k + 1) == "BY")
            .and_then(|k| error(k, "ORDER BY with SELECT SINGLE"))
    }

    /// The SELECT list: separated by commas throughout or by blanks
    /// throughout (`a b, c` is neither), and a parenthesis around an
    /// expression holds neither an alias nor a list (`( a AS b )`,
    /// `( a, b )`); `( a ) AS b` is fine.
    fn select_list(&self, select: usize) -> Option<SyntaxError> {
        let d = self.depth[select];
        let end = self.query_end(select);
        let mut start = select + 1;
        while matches!(self.word(start), "SINGLE" | "DISTINCT") {
            start += 1;
        }
        if self.word(start) == "FROM" {
            // SELECT FROM ... FIELDS list; without FIELDS SAP reports the
            // INTO after FROM as "invalid here (due to grammar)"
            let fields = (start..end)
                .filter(|&k| self.depth[k] == d)
                .take_while(|&k| !matches!(self.word(k), "UNION" | "INTERSECT" | "EXCEPT"))
                .find(|&k| self.word(k) == "FIELDS");
            match fields {
                Some(k) => start = k + 1,
                None => return error(start, "SELECT FROM without FIELDS"),
            }
        }
        let stop = (start..end)
            .find(|&k| {
                self.depth[k] == d && (LIST_END.contains(&self.word(k)) || self.word(k) == ".")
            })
            .unwrap_or(end);
        let top: Vec<usize> = (start..stop).filter(|&k| self.depth[k] == d).collect();
        let column = |k: usize| self.is_name(k) && !LIST_KEYWORDS.contains(&self.word(k));
        // SAP `abenabap_sql_strictmode_740_sp05`: in strict mode "all lists
        // of operands must be comma-separated lists"
        let commas = top.iter().any(|&k| self.word(k) == ",");
        if commas || self.strict {
            for pair in top.windows(2) {
                let (a, b) = (pair[0], pair[1]);
                if b == a + 1 && self.t[b].ws_before && column(a) && column(b) {
                    let message = if commas {
                        "blanks and commas mixed in the SELECT list"
                    } else {
                        "SELECT list without commas in strict mode"
                    };
                    return error(b, message);
                }
            }
        }
        for &k in &top {
            let open = self.t[k];
            // `( expression )`, not `(dynamic)` and not the parenthesis of a call
            let bare = open.kind == TokenKind::ParenLeft
                && open.ws_after
                && (open.ws_before || k == start);
            if !bare {
                continue;
            }
            let close = (k + 1..stop).find(|&j| self.depth[j] == d).unwrap_or(stop);
            if let Some(j) = (k + 1..close)
                .find(|&j| self.depth[j] == d + 1 && matches!(self.word(j), "AS" | ","))
            {
                return error(j, "alias or list inside parentheses");
            }
        }
        None
    }

    /// An aggregate function takes no dynamic column: `SUM( (name) )`,
    /// `COUNT( DISTINCT (name) )`.
    fn dynamic_aggregate(&self) -> Option<SyntaxError> {
        const AGGREGATES: [&str; 8] = [
            "SUM",
            "COUNT",
            "AVG",
            "MIN",
            "MAX",
            "MEDIAN",
            "STRING_AGG",
            "GROUPING",
        ];
        for k in 0..self.t.len() {
            if !AGGREGATES.contains(&self.word(k)) || !self.call_at(k) {
                continue;
            }
            let mut j = k + 2;
            if self.word(j) == "DISTINCT" {
                j += 1;
            }
            if self
                .t
                .get(j)
                .is_some_and(|t| t.kind == TokenKind::ParenLeft && !t.ws_after)
            {
                return error(j, "dynamic column in an aggregate");
            }
        }
        None
    }

    /// A method call starts at `i`: a name right before `(` followed by a
    /// blank or by `)`; not an offset or length like `lv(10)`, not a
    /// dynamic name like `(lt_stab)`.
    fn call_at(&self, i: usize) -> bool {
        let (Some(name), Some(open)) = (self.t.get(i), self.t.get(i + 1)) else {
            return false;
        };
        name.kind == TokenKind::Identifier
            && open.kind == TokenKind::ParenLeft
            && !open.ws_before
            && (open.ws_after
                || self
                    .t
                    .get(i + 2)
                    .is_some_and(|t| t.kind == TokenKind::ParenRight))
    }

    /// `CALL TRANSFORMATION name` takes PARAMETERS, OBJECTS, OPTIONS, SOURCE
    /// and RESULT; the SOURCE operands are data objects.
    fn call_transformation(&self) -> Option<SyntaxError> {
        const ADDITIONS: [&str; 5] = ["PARAMETERS", "OBJECTS", "OPTIONS", "SOURCE", "RESULT"];
        // the name, or (name)
        let after = if self.word(2) == "(" {
            (3..self.w.len()).find(|&k| self.depth[k] == 0 && self.word(k) == ")")? + 1
        } else {
            3
        };
        let next = self.word(after);
        if !next.is_empty() && next != "." && !ADDITIONS.contains(&next) {
            return error(after, "unknown addition");
        }
        let source = self.top("SOURCE").first().copied()?;
        let end = (source + 1..self.w.len())
            .find(|&k| self.depth[k] == 0 && ADDITIONS.contains(&self.word(k)))
            .unwrap_or(self.w.len());
        (source + 1..end)
            .find(|&k| self.call_at(k))
            .and_then(|k| error(k, "method call in SOURCE"))
    }

    /// `CREATE DATA ... TYPE HANDLE handle`: the handle is a data object.
    fn create_data(&self) -> Option<SyntaxError> {
        let at =
            (1..self.w.len()).find(|&k| self.word(k - 1) == "TYPE" && self.word(k) == "HANDLE")?;
        (at + 1..self.w.len())
            .find(|&k| self.call_at(k))
            .and_then(|k| error(k, "TYPE HANDLE takes a data object"))
    }

    /// MERGE: an internal table as source has an alias, `WHEN MATCHED`
    /// comes before `WHEN NOT MATCHED`.
    fn merge(&self) -> Option<SyntaxError> {
        for &u in &self.top("USING") {
            if self.word(u + 1) != "@" {
                continue;
            }
            // @itab, written without blanks
            let mut k = u + 2;
            while self.t.get(k + 1).is_some_and(|t| !t.ws_before) {
                k += 1;
            }
            if self.word(k + 1) != "AS" {
                return error(u + 1, "internal table without an alias");
            }
        }
        let mut not_matched = false;
        for &w in &self.top("WHEN") {
            match (self.word(w + 1), self.word(w + 2)) {
                ("NOT", "MATCHED") => not_matched = true,
                ("MATCHED", _) if not_matched => {
                    return error(w, "WHEN MATCHED after WHEN NOT MATCHED");
                }
                _ => {}
            }
        }
        None
    }

    /// `USING KEY` goes with an index or a work area, not with `WITH KEY`.
    fn read_table(&self) -> Option<SyntaxError> {
        let using = self
            .top("USING")
            .into_iter()
            .any(|u| self.word(u + 1) == "KEY");
        if !using {
            return None;
        }
        self.top("WITH")
            .into_iter()
            .find(|&w| self.word(w + 1) == "KEY" || self.word(w + 1) == "TABLE")
            .and_then(|w| error(w, "USING KEY with WITH KEY"))
    }

    fn method_def(&self) -> Option<SyntaxError> {
        // an event handler of an entity is an instance method
        let class_method = self.word(0) == "CLASS-METHODS"
            || (self.word(0) == "CLASS" && self.word(1) == "-" && self.word(2) == "METHODS");
        if class_method {
            let entity = (0..self.w.len()).find(|&k| {
                self.word(k) == "FOR" && self.word(k + 1) == "ENTITY" && self.word(k + 2) == "EVENT"
            });
            if let Some(k) = entity {
                return error(k, "entity event handler as a static method");
            }
        }
        const ORDER: [&str; 6] = [
            "IMPORTING",
            "EXPORTING",
            "CHANGING",
            "RETURNING",
            "RAISING",
            "EXCEPTIONS",
        ];
        let mut last = 0;
        let mut first_section = None;
        for i in 0..self.w.len() {
            if self.depth[i] != 0 {
                continue;
            }
            let Some(rank) = ORDER.iter().position(|o| *o == self.word(i)) else {
                continue;
            };
            // RAISING and EXCEPTIONS rank alike
            let rank = rank.min(4);
            if rank < last {
                return error(i, "parameter sections out of order");
            }
            last = rank;
            // a test method may raise, but has no parameters
            if rank < 4 {
                first_section.get_or_insert(i);
            }
        }
        if let Some(i) = first_section {
            let testing = (0..i).any(|k| self.word(k) == "FOR" && self.word(k + 1) == "TESTING");
            if testing {
                return error(i, "test method with parameters");
            }
        }
        // TYPE ANY STRUCTURE only for IMPORTING and CHANGING parameters
        for i in 0..self.w.len() {
            if i > 0
                && self.word(i) == "ANY"
                && self.word(i + 1) == "STRUCTURE"
                && self.word(i - 1) == "TYPE"
            {
                let section = (0..i)
                    .rev()
                    .find(|&k| self.depth[k] == 0 && ORDER.contains(&self.word(k)))
                    .map(|k| self.word(k));
                if matches!(section, Some("EXPORTING" | "RETURNING")) {
                    return error(i, "TYPE ANY STRUCTURE only for IMPORTING and CHANGING");
                }
            }
        }
        None
    }

    fn data_type(&self) -> Option<SyntaxError> {
        // WITH ends a key or header clause, it cannot end the statement
        let last = self.w.iter().rposition(|w| w != ".").unwrap_or_default();
        if last > 0 && self.word(last) == "WITH" && self.depth[last] == 0 {
            return error(last, "WITH without a key");
        }
        for i in 1..self.w.len() {
            match self.word(i) {
                // TABLE OF without its line type
                "OF" if self.word(i - 1) == "TABLE" && self.word(i + 1) == "WITH" => {
                    return error(i + 1, "line type missing");
                }
                "STRUCTURE"
                    if i > 1
                        && self.word(i - 1) == "ANY"
                        && self.word(i - 2) == "TYPE"
                        && self.word(0) != "TYPES" =>
                {
                    return error(i, "TYPE ANY STRUCTURE only for parameters");
                }
                "TABLE"
                    if matches!(self.word(i - 1), "STANDARD" | "SORTED" | "HASHED")
                        && !matches!(self.word(i + 1), "OF" | "WITH" | "" | ".")
                        && self
                            .t
                            .get(i + 1)
                            .is_some_and(|t| t.kind == TokenKind::Identifier) =>
                {
                    return error(i + 1, "OF missing");
                }
                _ => {}
            }
        }
        None
    }

    fn concatenate(&self, file: &File, s: &Statement) -> Option<SyntaxError> {
        let Some(&into) = self.top("INTO").first() else {
            return error(self.t.len().saturating_sub(1), "INTO missing");
        };
        // the operands are data objects and constants; LINES OF takes a
        // table from anywhere
        if self.word(1) == "LINES" {
            return None;
        }

        for item in operands(file, s).items {
            let Operand::Expr { expr, .. } = item else {
                continue;
            };
            let call = is_call(&expr.kind);
            let template = matches!(expr.kind, ExprKind::Template(_))
                || self
                    .t
                    .get(expr.first)
                    .is_some_and(|t| t.kind == TokenKind::StringTemplate);
            if expr.first < into && (call || template) {
                return error(expr.first, "only data objects can be concatenated");
            }
            if template && self.word(expr.first.saturating_sub(1)) == "BY" {
                return error(expr.first, "only data objects can be concatenated");
            }
        }
        None
    }

    fn delete_internal(&self) -> Option<SyntaxError> {
        if self.word(1) == "TABLE" {
            for word in ["WHERE", "TO"] {
                if let Some(&i) = self.top(word).first() {
                    return error(i, "DELETE TABLE with WHERE or TO");
                }
            }
        }
        None
    }

    fn insert_internal(&self) -> Option<SyntaxError> {
        let into: Vec<usize> = self
            .top("INTO")
            .into_iter()
            .filter(|&i| self.word(i - 1) != "REFERENCE")
            .collect();
        let reference =
            (0..self.w.len()).find(|&i| self.word(i) == "REFERENCE" && self.word(i + 1) == "INTO");
        let assigning = self.top("ASSIGNING").first().copied();
        if let (Some(r), Some(_)) = (reference, assigning) {
            return error(r, "REFERENCE INTO and ASSIGNING");
        }
        if let (Some(target), Some(&into)) = (reference.or(assigning), into.first()) {
            if target < into {
                return error(target, "REFERENCE INTO or ASSIGNING before INTO");
            }
        }
        if let Some(&into) = into.first() {
            if self.word(into + 1) == "TABLE" {
                if let Some(&i) = self.top("INDEX").first() {
                    return error(i, "INDEX with INTO TABLE");
                }
            }
        }
        None
    }

    fn describe(&self, file: &File, s: &Statement) -> Option<SyntaxError> {
        if self.word(1) != "TABLE" {
            return None;
        }
        let first = operands(file, s).items.into_iter().find_map(|i| match i {
            Operand::Expr { expr, .. } => Some(expr),
            _ => None,
        })?;
        is_call(&first.kind).then_some(SyntaxError {
            token: first.first,
            message: "DESCRIBE TABLE of a method call",
        })
    }

    fn perform(&self, file: &File, s: &Statement) -> Option<SyntaxError> {
        for item in operands(file, s).items {
            if let Operand::Expr { expr, .. } = item {
                if matches!(expr.kind, ExprKind::Binary { .. }) && !expr.is_literal_concatenation()
                {
                    return error(expr.first, "only data objects as PERFORM parameters");
                }
            }
        }
        None
    }

    fn raise(&self) -> Option<SyntaxError> {
        // MESSAGE goes with RAISE EXCEPTION TYPE only
        if self.word(1) == "EXCEPTION" && self.word(2) != "TYPE" && self.word(3) != "TYPE" {
            if let Some(&i) = self.top("MESSAGE").first() {
                return error(i, "MESSAGE without TYPE");
            }
        }
        None
    }

    fn update_database(&self) -> Option<SyntaxError> {
        // SET col = col + n: only + and - change a column in place
        let set = self.top("SET").first().copied()?;
        let end = self.top("WHERE").first().copied().unwrap_or(self.w.len());
        for i in set + 1..end {
            if matches!(self.word(i), "*" | "/")
                && self.depth[i] == 0
                && self.t[i].ws_before
                && self.t[i].ws_after
            {
                let column = self.word(i - 1);
                let assigned = (set + 1..i)
                    .rev()
                    .find(|&k| self.word(k) == "=")
                    .map(|k| self.word(k - 1));
                if assigned == Some(column) {
                    return error(i, "only + and - for a column in SET");
                }
            }
        }
        None
    }

    fn append(&self) -> Option<SyntaxError> {
        // APPEND ... TO <literal>
        let to = self.top("TO");
        let last = *to.last()?;
        let target = self.t.get(last + 1)?;
        let literal =
            target.kind == TokenKind::String || target.text.chars().all(|c| c.is_ascii_digit());
        (literal && self.word(1) == "LINES").then_some(SyntaxError {
            token: last + 1,
            message: "APPEND to a literal",
        })
    }

    /// FORMAT and EDITOR-CALL take data objects and constants only.
    fn data_objects_only(&self, file: &File, s: &Statement) -> Option<SyntaxError> {
        for item in operands(file, s).items {
            let Operand::Expr { expr, .. } = item else {
                continue;
            };
            let built = matches!(expr.kind, ExprKind::Constructor(_) | ExprKind::Template(_))
                || self
                    .t
                    .get(expr.first)
                    .is_some_and(|t| t.kind == TokenKind::StringTemplate);
            if built {
                return error(expr.first, "only data objects and constants");
            }
        }
        // operands without rules: a constructor or template in the tokens
        for i in 0..self.t.len() {
            let template = self.t[i].kind == TokenKind::StringTemplate;
            let constructor = matches!(self.word(i), "COND" | "SWITCH" | "VALUE" | "CONV" | "NEW")
                && self.word(i + 1) == "#";
            if template || constructor {
                return error(i, "only data objects and constants");
            }
        }
        None
    }

    /// MESSAGE: `ID mid`, `TYPE mtype` and `NUMBER num` take data objects,
    /// not calls or constructor expressions (`oref` of `MESSAGE oref` and
    /// the operands of `WITH` are other positions).
    fn message(&self, file: &File, s: &Statement) -> Option<SyntaxError> {
        for item in operands(file, s).items {
            let Operand::Expr { expr, .. } = item else {
                continue;
            };
            let after = expr.first.checked_sub(1).map_or("", |i| self.word(i));
            if !matches!(after, "ID" | "TYPE" | "NUMBER") {
                continue;
            }
            let computed = is_call(&expr.kind)
                || matches!(expr.kind, ExprKind::Constructor(_) | ExprKind::Template(_))
                || self
                    .t
                    .get(expr.first)
                    .is_some_and(|t| t.kind == TokenKind::StringTemplate);
            if computed {
                return error(expr.first, "MESSAGE ID, TYPE and NUMBER take data objects");
            }
        }
        None
    }

    /// FORM parameters are typed with an existing type, a generic one, `REF
    /// TO` or `LINE OF`; they cannot build one, as `TYPE TABLE OF i`, `TYPE
    /// c LENGTH 10` or `TYPE STANDARD TABLE WITH DEFAULT KEY` do.
    fn form_parameters(&self) -> Option<SyntaxError> {
        const MESSAGE: &str = "FORM parameter typed with a type built in place";
        let table_kind = |k: usize| {
            matches!(
                self.word(k),
                "STANDARD" | "SORTED" | "HASHED" | "INDEX" | "ANY"
            ) && self.word(k + 1) == "TABLE"
        };
        let of = |k: usize| matches!(self.word(k), "TABLE" | "RANGE") && self.word(k + 1) == "OF";
        for i in 0..self.w.len() {
            if self.depth[i] != 0 {
                continue;
            }
            let k = i + 1;
            match self.word(i) {
                "TYPE" => {
                    if of(k) {
                        return error(k, MESSAGE);
                    }
                    if table_kind(k) {
                        // `TYPE STANDARD TABLE [OF type]`, without a key
                        let mut end = k + 2;
                        if self.word(end) == "OF" {
                            if self.word(end + 1) == "REF" && self.word(end + 2) == "TO" {
                                return error(end, MESSAGE);
                            }
                            end += 2;
                        }
                        if self.word(end) == "WITH" {
                            return error(end, MESSAGE);
                        }
                        continue;
                    }
                    let name = if matches!(self.word(k), "REF" | "LINE") {
                        k + 2
                    } else {
                        k
                    };
                    // `TYPE c length TYPE i` has a parameter named length
                    let number = !self.word(name + 2).is_empty()
                        && self.word(name + 2).bytes().all(|b| b.is_ascii_digit());
                    if matches!(self.word(name + 1), "LENGTH" | "DECIMALS") && number {
                        return error(name + 1, MESSAGE);
                    }
                }
                "LIKE" if of(k) || table_kind(k) => return error(k, MESSAGE),
                _ => {}
            }
        }
        None
    }

    /// GET PERMISSIONS: the variants of `ONLY`, `FROM keys` only and always
    /// for instances, and `REQUEST` for every entity.
    fn get_permissions(&self) -> Option<SyntaxError> {
        let mut i = 2;
        if self.word(i) == "FORWARDING" {
            i += 1;
        }
        if self.word(i) == "PRIVILEGED" {
            i += 1;
        }
        let mut global = None;
        if self.word(i) == "ONLY" {
            i += 1;
            match self.word(i) {
                "GLOBAL" | "INSTANCE" => {
                    global = Some(self.word(i) == "GLOBAL");
                    i += 1;
                    if matches!(self.word(i), "FEATURES" | "AUTHORIZATION") {
                        i += 1;
                    }
                }
                "FEATURES" | "AUTHORIZATION" => {
                    global = Some(false);
                    i += 1;
                }
                "(" => {
                    i = (i + 1..self.w.len())
                        .find(|&k| self.depth[k] == 0 && self.word(k) == ")")?
                        + 1;
                }
                _ => return error(i, "unknown ONLY clause"),
            }
            if !matches!(self.word(i), "ENTITY" | "OF" | "OPERATIONS") {
                return error(i, "unknown ONLY clause");
            }
        }
        // the short and the long form: each ENTITY with its REQUEST
        let entities = self.top("ENTITY");
        for (n, &e) in entities.iter().enumerate() {
            let end = entities.get(n + 1).copied().unwrap_or(self.w.len());
            let has =
                |word: &str| (e + 2..end).find(|&k| self.depth[k] == 0 && self.word(k) == word);
            if has("REQUEST").is_none() {
                return error(e, "REQUEST missing");
            }
            match (global, has("FROM")) {
                (Some(true), Some(from)) => return error(from, "FROM with ONLY GLOBAL"),
                (Some(false), None) => return error(e, "FROM missing for instances"),
                _ => {}
            }
        }
        None
    }

    /// `ref->( name )`: no blanks inside the parentheses of a dynamic
    /// component.
    fn dynamic_component(&self) -> Option<SyntaxError> {
        for i in 1..self.t.len() {
            let t = self.t[i];
            let arrow = matches!(
                self.t[i - 1].kind,
                TokenKind::InstanceArrow | TokenKind::Dash
            );
            if arrow && t.kind == TokenKind::ParenLeft && !t.ws_before {
                let close = (i + 1..self.t.len()).find(|&k| {
                    self.depth[k] == self.depth[i] && self.t[k].kind == TokenKind::ParenRight
                })?;
                if t.ws_after || self.t[close].ws_before {
                    return error(i, "blank inside a dynamic component");
                }
            }
        }
        None
    }

    /// The name of a parameter or selection criterion has up to eight
    /// characters, dashes included (SAP: "The parameter name ... can be up to
    /// eight characters long").
    fn selection_name(&self) -> Option<SyntaxError> {
        // after the keyword, `SELECT-OPTIONS` is three tokens
        let start = (1..self.t.len()).find(|&k| self.t[k].ws_before)?;
        let mut name = self.t[start].text.trim_start_matches('!').len();
        for t in &self.t[start + 1..] {
            if t.ws_before || !matches!(t.kind, TokenKind::Dash | TokenKind::Identifier) {
                break;
            }
            name += t.text.len();
        }
        if name > 8 {
            return error(start, "name longer than eight characters");
        }
        None
    }

    /// A built-in function called without an argument, `strlen( )`, unless
    /// the file declares a method of that name, which hides the function.
    fn builtin_without_argument(&self, file: &File) -> Option<SyntaxError> {
        let k = (0..self.t.len().saturating_sub(2)).find(|&k| {
            let (name, open, close) = (self.t[k], self.t[k + 1], self.t[k + 2]);
            let component = k > 0
                && matches!(
                    self.t[k - 1].kind,
                    TokenKind::InstanceArrow | TokenKind::StaticArrow | TokenKind::Dash
                );
            !component
                && name.kind == TokenKind::Identifier
                && open.kind == TokenKind::ParenLeft
                && !open.ws_before
                && close.kind == TokenKind::ParenRight
                && BUILTINS_WITH_ARGUMENT.contains(&self.word(k))
        })?;
        let method = file
            .statements
            .iter()
            .filter(|s| matches!(s.kind, K::MethodDef | K::MethodImplementation | K::Aliases))
            .flat_map(|s| file.statement_tokens(s))
            .any(|t| t.upper() == self.word(k));
        if method {
            return None;
        }
        error(k, "built-in function without an argument")
    }
}

/// Built-in functions that take at least one argument; SAP rejects
/// `strlen( )`. Names that are common method names too, like `count` or
/// `find`, are left out: a method of a superclass would hide the function.
const BUILTINS_WITH_ARGUMENT: [&str; 31] = [
    "STRLEN",
    "XSTRLEN",
    "NUMOFCHAR",
    "CHARLEN",
    "DBMAXLEN",
    "LINES",
    "LINE_EXISTS",
    "LINE_INDEX",
    "BOOLC",
    "BOOLX",
    "XSDBOOL",
    "TO_UPPER",
    "TO_LOWER",
    "TO_MIXED",
    "FROM_MIXED",
    "CONDENSE",
    "CONCAT_LINES_OF",
    "SUBSTRING",
    "SUBSTRING_AFTER",
    "SUBSTRING_BEFORE",
    "SUBSTRING_FROM",
    "SUBSTRING_TO",
    "SHIFT_LEFT",
    "SHIFT_RIGHT",
    "CONTAINS",
    "CONTAINS_ANY_OF",
    "CONTAINS_ANY_NOT_OF",
    "IPOW",
    "NMAX",
    "NMIN",
    "UTCLONG_DIFF",
];

fn is_call(kind: &ExprKind) -> bool {
    let ExprKind::Chain(chain) = kind else {
        return false;
    };
    matches!(chain.head, Head::Call { .. })
        || chain.links.iter().any(|l| matches!(l, Link::Method { .. }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check(abap: &str) -> Option<&'static str> {
        let file = crate::parse(abap);
        let s = file
            .statements
            .iter()
            .find(|s| s.kind != K::Comment)
            .unwrap();
        syntax_error(&file, s).map(|e| e.message)
    }

    /// Code the syntax check of the SAP_BASIS 7.50 system accepts, from
    /// active objects there and its check of unsaved text (2026-11-16): no
    /// statement may be reported.
    #[test]
    fn accepted_by_the_750_system() {
        for abap in [
            // the interface of a function module in the statement, as ADT
            // serves every function module
            "FUNCTION z_a\n  IMPORTING\n    VALUE(iv_x) TYPE i DEFAULT 5\n    REFERENCE(ir) TYPE REF TO data OPTIONAL\n    iv_l LIKE sy-uname DEFAULT sy-uname\n    VALUE(iv_u)\n  EXPORTING\n    VALUE(ev_y) TYPE string\n  CHANGING\n    cv TYPE c\n  TABLES\n    t STRUCTURE bapiret2 OPTIONAL\n    t3\n  EXCEPTIONS\n    not_found.\nENDFUNCTION.",
            "FUNCTION z_b\n  EXPORTING\n    VALUE(r) TYPE i\n  RAISING\n    cx_static_check.\nENDFUNCTION.",
            // literals joined by &, the last with a text symbol
            "DATA s TYPE string.\ns = 'a' & 'b'(001).\nWRITE 'a' & 'b' & 'c'(001).",
            "PERFORM f USING 'a' & 'b'.\nFORM f USING p. ENDFORM.",
            "CALL FUNCTION 'X' EXPORTING p = 'a' & 'b'(001).",
            // a parenthesised expression as a CASE result or WHERE operand
            "SELECT CASE carrid WHEN 'AA' THEN ( 'a' && carrname ) ELSE ( 'b' && carrname ) END AS x FROM scarr INTO TABLE @DATA(t).",
            "SELECT CASE WHEN carrid = 'AA' THEN ( 1 + 2 ) ELSE 0 END AS x FROM scarr INTO TABLE @DATA(t).",
            "SELECT carrid FROM scarr WHERE ( carrid && carrname ) = 'x' INTO TABLE @DATA(t).",
            // literals as fillers of a structure
            "DATA: BEGIN OF x,\n        name(30),\n        '(',\n        id(5),\n        ') ',\n      END OF x.",
            // a part of a chain right after the comma
            "DATA s TYPE string.\nWRITE: (7) 'a'\n ,(13) s.",
            // a field as position after a slash: a warning only
            "WRITE: /lv(10) 'x'.",
            "GENERATE DYNPRO h f e m ID n MESSAGE ms LINE l OFFSET o WORD w TRACE-FILE tf.",
            "TYPES: BEGIN OF MESH m,\n  a TYPE ta ASSOCIATION _b TO b ON x = y USING KEY k1\n            ASSOCIATION _c TO c ON x = z,\n  b TYPE tb ASSOCIATION _a TO a ON y = x USING KEY primary_key,\nEND OF MESH m.",
            "CLASS zcl DEFINITION.\n PUBLIC SECTION.\n  INTERFACES if_amdp_marker_hdb.\n  METHODS m.\nENDCLASS.\nCLASS zcl IMPLEMENTATION.\n METHOD m BY DATABASE PROCEDURE FOR HDB LANGUAGE LLANG OPTIONS READ-ONLY.\n  any text\n ENDMETHOD.\nENDCLASS.",
        ] {
            let file = crate::parse(abap);
            for s in &file.statements {
                if let Some(e) = syntax_error(&file, s) {
                    panic!("{}: {}", file.statement_text(s), e.message);
                }
            }
        }
    }

    /// What the syntax check of an SAP system (ABAP Platform 2025) rejects
    /// and accepts.
    #[test]
    fn select_rules_of_the_sap_system() {
        for (abap, expected) in [
            (
                "SELECT SINGLE * FROM t100 INTO @DATA(ls) ORDER BY arbgb.",
                "ORDER BY with SELECT SINGLE",
            ),
            (
                "SELECT SINGLE * FROM t100 ORDER BY arbgb INTO @DATA(ls).",
                "ORDER BY with SELECT SINGLE",
            ),
            (
                "SELECT SINGLE * FROM t100 INTO ls ORDER BY arbgb.",
                "ORDER BY with SELECT SINGLE",
            ),
            (
                "SELECT arbgb msgnr, text FROM t100 INTO TABLE @DATA(lt).",
                "blanks and commas mixed in the SELECT list",
            ),
            (
                "SELECT arbgb, msgnr text FROM t100 INTO TABLE @DATA(lt).",
                "blanks and commas mixed in the SELECT list",
            ),
            (
                "SELECT foo bar, moo FROM ztab INTO TABLE @DATA(tab) ORDER BY PRIMARY KEY.",
                "blanks and commas mixed in the SELECT list",
            ),
            (
                "SELECT ( arbgb AS bar ) FROM t100 INTO TABLE @DATA(lt).",
                "alias or list inside parentheses",
            ),
            (
                "SELECT ( arbgb, text ) AS bar FROM t100 INTO TABLE @DATA(lt).",
                "alias or list inside parentheses",
            ),
            (
                "SELECT FROM t100 FIELDS ( arbgb AS bar ) INTO TABLE @DATA(lt).",
                "alias or list inside parentheses",
            ),
            (
                "SELECT SUM( (l_field) ) INTO l_value FROM (l_table).",
                "dynamic column in an aggregate",
            ),
            (
                "SELECT COUNT( DISTINCT (l_field) ) INTO l_value FROM t100.",
                "dynamic column in an aggregate",
            ),
        ] {
            assert_eq!(check(abap), Some(expected), "{abap}");
        }
        for ok in [
            "SELECT * FROM t100 INTO TABLE @DATA(lt) ORDER BY arbgb.",
            "SELECT * FROM t100 INTO TABLE lt ORDER BY arbgb.",
            "SELECT * FROM t100 ORDER BY arbgb INTO @DATA(ls) UP TO 1 ROWS.",
            "SELECT arbgb msgnr text FROM t100 INTO CORRESPONDING FIELDS OF TABLE lt.",
            "SELECT arbgb, msgnr, text FROM t100 INTO TABLE @DATA(lt).",
            "SELECT ( arbgb ) FROM t100 INTO TABLE @DATA(lt).",
            "SELECT ( arbgb ) AS bar FROM t100 INTO TABLE @DATA(lt).",
            "SELECT (l_cols) INTO CORRESPONDING FIELDS OF TABLE lt FROM (l_table).",
            "SELECT SUM( msgnr ) INTO l_value FROM (l_table).",
            "SELECT a AS x, CASE b WHEN 1 THEN 'A' ELSE 'B' END AS y, c + d AS z, cast( e AS CHAR( 10 ) ) AS w FROM t INTO TABLE @lt.",
            "SELECT a, ROW_NUMBER( ) OVER( PARTITION BY b ORDER BY c DESCENDING ) AS n FROM t INTO TABLE @lt.",
            "SELECT t1~a, t2~b FROM t1 INNER JOIN t2 ON t2~k = t1~k INTO TABLE @lt.",
            "SELECT SINGLE a FROM t WHERE b IN ( SELECT b FROM u ) INTO @lv.",
        ] {
            assert_eq!(check(ok), None, "{ok}");
        }
    }

    /// Checked on the SAP ABAP Cloud Developer Trial 2025 (8.16) on
    /// 2026-11-14, with the same statements on T100.
    #[test]
    fn strict_mode_rules_of_the_sap_system() {
        const NO_AT: &str = "host variable without @ in strict mode";
        for (abap, expected) in [
            // strict from @, commas, FIELDS, SQL expressions, USING CLIENT,
            // RIGHT OUTER JOIN, INTO as the last clause
            ("SELECT a, b FROM t INTO TABLE lt.", NO_AT),
            ("SELECT FROM t FIELDS a WHERE b = 'X' INTO TABLE lt.", NO_AT),
            (
                "SELECT CASE a WHEN 'A' THEN 'B' ELSE a END AS a FROM t INTO TABLE lt.",
                NO_AT,
            ),
            ("SELECT * FROM t USING CLIENT '001' INTO TABLE lt.", NO_AT),
            (
                "SELECT t~a FROM t RIGHT OUTER JOIN u ON t~a = u~a INTO TABLE lt.",
                NO_AT,
            ),
            ("SELECT * FROM t WHERE a = lv INTO TABLE lt.", NO_AT),
            // every host variable once one has @
            ("SELECT * FROM t WHERE a = wa-a INTO TABLE @lt.", NO_AT),
            (
                "SELECT * FROM t WHERE a = @lv INTO TABLE @lt UP TO n ROWS.",
                NO_AT,
            ),
            ("SELECT * FROM t WHERE a = @lv INTO TABLE lt.", NO_AT),
            ("SELECT * FROM t WHERE a = @lv APPENDING TABLE lt.", NO_AT),
            (
                "SELECT SINGLE a, b FROM t WHERE a = @lv INTO (@x, y).",
                NO_AT,
            ),
            (
                "SELECT a, b FROM t FOR ALL ENTRIES IN lt WHERE a = @lt-a INTO TABLE @lt2.",
                NO_AT,
            ),
            (
                "SELECT a, b FROM t FOR ALL ENTRIES IN @lt WHERE a = lt-a INTO TABLE @lt2.",
                NO_AT,
            ),
            ("UPDATE t SET a = wa-a WHERE b = @lv.", NO_AT),
            ("UPDATE t USING CLIENT '001' FROM wa.", NO_AT),
            ("MODIFY t USING CLIENT '001' FROM TABLE lt.", NO_AT),
            ("DELETE FROM t WHERE a = wa-a AND b = @lv.", NO_AT),
            // lists separated by commas
            (
                "SELECT a, b FROM t WHERE a = @lv GROUP BY a b INTO TABLE @lt.",
                "GROUP BY list without commas in strict mode",
            ),
            (
                "SELECT a, b FROM t WHERE a = @lv ORDER BY a b INTO TABLE @lt.",
                "ORDER BY list without commas in strict mode",
            ),
            (
                "SELECT * FROM t WHERE a = @lv ORDER BY a DESCENDING b INTO TABLE @lt.",
                "ORDER BY list without commas in strict mode",
            ),
            (
                "UPDATE t SET a = @lv b = @lv WHERE c = @lv.",
                "SET list without commas in strict mode",
            ),
            (
                "SELECT a b FROM t INTO TABLE lt GROUP BY a, b.",
                "SELECT list without commas in strict mode",
            ),
            // the obsolete short form
            (
                "SELECT * FROM t WHERE a = @lv.",
                "SELECT without INTO in strict mode",
            ),
        ] {
            assert_eq!(check(abap), Some(expected), "{abap}");
        }
        for ok in [
            // classic
            "SELECT a b FROM t INTO TABLE lt WHERE a = lv ORDER BY a b.",
            "SELECT * FROM t WHERE a = lv.",
            "UPDATE t SET a = lv b = lv WHERE c = lv.",
            "INSERT t FROM wa.",
            "DELETE t FROM TABLE lt.",
            // strict
            "SELECT a, b FROM t INTO TABLE @lt WHERE a = @lv ORDER BY a, b.",
            "SELECT * FROM t WHERE a = @lv ORDER BY PRIMARY KEY INTO TABLE @lt.",
            "SELECT * FROM t WHERE a = @lv ORDER BY a DESCENDING, b ASCENDING INTO TABLE @lt.",
            "SELECT * FROM t WHERE a = @lv ORDER BY a NULLS FIRST, b INTO TABLE @lt.",
            "SELECT * FROM t WHERE a = @lv INTO TABLE @lt UP TO 5 ROWS.",
            "SELECT * FROM t WHERE a = @( wa-a ) INTO TABLE @lt.",
            "SELECT * FROM (dyn-tab) WHERE (dyn-where) INTO TABLE @lt.",
            "SELECT a, b FROM t FOR ALL ENTRIES IN @me->tab WHERE a = @me->tab-a INTO TABLE @lt.",
            "SELECT @cl=>const-comp AS a, b FROM t INTO TABLE @lt.",
            "SELECT SINGLE a AS t-a FROM t INTO @wa.",
            "SELECT SINGLE a AS offset FROM t INTO @lv.",
            "SELECT COUNT(*) FROM t WHERE a = @lv.",
            "SELECT a, COUNT( * ) OVER( ORDER BY a, b ROWS BETWEEN UNBOUNDED PRECEDING AND CURRENT ROW ) AS c FROM t INTO TABLE @lt.",
            "SELECT CASE WHEN a = 'A' THEN 'X' ELSE 'Y' END AS g, COUNT(*) AS c FROM t GROUP BY CASE WHEN a = 'A' THEN 'X' ELSE 'Y' END INTO TABLE @lt.",
            "UPDATE t SET a = @lv, b = @lv WHERE c = @lv.",
            "INSERT t FROM @wa.",
            "MODIFY t FROM TABLE @lt MAPPING FROM ENTITY.",
            "SELECT FROM i_country FIELDS \\_text-countryname WHERE country = @lv INTO TABLE @lt.",
        ] {
            assert_eq!(check(ok), None, "{ok}");
        }
    }

    #[test]
    fn form_parameters_take_existing_types() {
        let e = Some("FORM parameter typed with a type built in place");
        for abap in [
            "FORM f USING p TYPE TABLE OF i.",
            "FORM f TABLES p TYPE TABLE OF i.",
            "FORM f USING VALUE(p) TYPE RANGE OF i.",
            "FORM f USING p TYPE STANDARD TABLE OF REF TO i.",
            "FORM f USING p TYPE STANDARD TABLE WITH DEFAULT KEY.",
            "FORM f USING p TYPE SORTED TABLE OF i WITH UNIQUE KEY table_line.",
            "FORM f USING p LIKE TABLE OF gt_foo.",
            "FORM f USING p LIKE STANDARD TABLE OF gt_foo.",
            "FORM f USING p TYPE c LENGTH 10.",
            "FORM f CHANGING p TYPE p DECIMALS 2.",
        ] {
            assert_eq!(check(abap), e, "{abap}");
        }
        for ok in [
            "FORM f USING p TYPE STANDARD TABLE.",
            "FORM f USING p TYPE STANDARD TABLE OF ty_line.",
            "FORM f USING p TYPE ANY TABLE q TYPE INDEX TABLE.",
            "FORM f USING p TYPE REF TO zcl_x q LIKE LINE OF gt_foo r TYPE LINE OF ty_tab.",
            "FORM f TABLES t STRUCTURE /foo/bar USING VALUE(type) TYPE c CHANGING x LIKE sy-tabix.",
            "FORM f USING p TYPE c length TYPE i decimals.",
        ] {
            assert_eq!(check(ok), None, "{ok}");
        }
    }

    #[test]
    fn get_permissions_variants() {
        for (abap, expected) in [
            (
                "GET PERMISSIONS ONLY LOCAL ENTITY e REQUEST r RESULT res.",
                "unknown ONLY clause",
            ),
            (
                "GET PERMISSIONS ONLY GLOBAL INSTANCE ENTITY e REQUEST r RESULT res.",
                "unknown ONLY clause",
            ),
            (
                "GET PERMISSIONS ONLY GLOBAL ENTITY e RESULT res.",
                "REQUEST missing",
            ),
            (
                "GET PERMISSIONS ONLY GLOBAL ENTITY e FROM keys REQUEST r RESULT res.",
                "FROM with ONLY GLOBAL",
            ),
            (
                "GET PERMISSIONS ONLY INSTANCE FEATURES ENTITY e REQUEST r RESULT res.",
                "FROM missing for instances",
            ),
        ] {
            assert_eq!(check(abap), Some(expected), "{abap}");
        }
        for ok in [
            "GET PERMISSIONS ONLY GLOBAL AUTHORIZATION ENTITY e REQUEST r RESULT res FAILED f.",
            "GET PERMISSIONS PRIVILEGED ONLY INSTANCE ENTITY e FROM keys REQUEST r RESULT res.",
            "GET PERMISSIONS ONLY (if_abap_behv=>permissions_only-authorization) ENTITY e FROM keys REQUEST r RESULT res.",
            "GET PERMISSIONS FORWARDING PRIVILEGED ENTITY e FROM keys REQUEST r RESULT res.",
            "GET PERMISSIONS ONLY INSTANCE FEATURES OF root ENTITY root FROM k1 REQUEST r1 RESULT res1 ENTITY child FROM k2 REQUEST r2 RESULT res2 FAILED f.",
            "GET PERMISSIONS ONLY INSTANCE FEATURES OPERATIONS perm_tab FAILED f REPORTED r.",
        ] {
            assert_eq!(check(ok), None, "{ok}");
        }
    }

    #[test]
    fn message_takes_data_objects() {
        let e = Some("MESSAGE ID, TYPE and NUMBER take data objects");
        assert_eq!(check("MESSAGE ID get_msg_id( ) TYPE 'E' NUMBER '004'."), e);
        assert_eq!(check("MESSAGE ID 'ZX' TYPE lo->kind( ) NUMBER '004'."), e);
        assert_eq!(check("MESSAGE ID 'ZX' TYPE 'E' NUMBER |{ n }|."), e);
        for ok in [
            "MESSAGE ID sy-msgid TYPE sy-msgty NUMBER sy-msgno WITH sy-msgv1.",
            "MESSAGE ID ls_msg-id TYPE 'E' NUMBER ls_msg-number WITH lo->text( ).",
            "MESSAGE e001(zx) WITH get( ) INTO DATA(lv).",
            "MESSAGE lo_error->get_text( ) TYPE 'I' DISPLAY LIKE 'E'.",
            "MESSAGE NEW zcx_x( ) TYPE 'E'.",
        ] {
            assert_eq!(check(ok), None, "{ok}");
        }
    }

    #[test]
    fn errors_found() {
        assert_eq!(
            check("SELECT SINGLE WHERE FROM ztab INTO @wa."),
            Some("keyword as a column")
        );
        assert_eq!(
            check("SELECT col FROM ztab INTO @wa PACKAGE SIZE 10."),
            Some("PACKAGE SIZE without a table")
        );
        assert_eq!(
            check("METHODS foo RETURNING VALUE(r) TYPE i IMPORTING b TYPE i."),
            Some("parameter sections out of order")
        );
        assert_eq!(check("CONCATENATE asdf."), Some("INTO missing"));
        assert_eq!(
            check("INSERT row INTO tab REFERENCE INTO ref ASSIGNING <fs>."),
            Some("REFERENCE INTO and ASSIGNING")
        );
        assert_eq!(
            check("ASSIGN ref->( comp ) TO <fs>."),
            Some("blank inside a dynamic component")
        );
        assert_eq!(
            check("CALL TRANSFORMATION id MOO."),
            Some("unknown addition")
        );
        assert_eq!(
            check("CALL TRANSFORMATION id SOURCE data = <fs> fields = list_key_fields( ) RESULT XML writer."),
            Some("method call in SOURCE")
        );
        assert_eq!(
            check("CREATE DATA lr_data TYPE HANDLE cl_abap_structdescr=>create( lt_comp )."),
            Some("TYPE HANDLE takes a data object")
        );
        assert_eq!(
            check("MERGE INTO ztarget AS a USING @itab ON 1 = 1 WHEN MATCHED THEN DELETE."),
            Some("internal table without an alias")
        );
        assert_eq!(
            check("MERGE INTO zt AS a USING zs AS b ON 1 = 1 WHEN NOT MATCHED THEN INSERT ALL WHEN MATCHED THEN DELETE."),
            Some("WHEN MATCHED after WHEN NOT MATCHED")
        );
        assert_eq!(
            check("CLASS-METHODS on_event FOR ENTITY EVENT events FOR root~raised."),
            Some("entity event handler as a static method")
        );
        assert_eq!(
            check("READ TABLE lt INTO ls USING KEY sha WITH KEY sha1 = lv."),
            Some("USING KEY with WITH KEY")
        );
        assert_eq!(
            check("TYPES moo TYPE SORTED TABLE OF foo_bar WITH NON-UNIQUE KEY and with."),
            Some("WITH without a key")
        );
    }

    #[test]
    fn correct_statements_pass() {
        for abap in [
            "SELECT FROM ztab FIELDS col INTO TABLE @DATA(lt).",
            "SELECT SINGLE FOR UPDATE * FROM ztab INTO @wa WHERE a = 1.",
            "SELECT col FROM ztab INTO TABLE @lt PACKAGE SIZE 10.",
            "METHODS foo IMPORTING a TYPE i EXPORTING b TYPE i RAISING cx.",
            "CONCATENATE a b INTO c SEPARATED BY space.",
            "CALL TRANSFORMATION (lv_name) PARAMETERS (lt_par) SOURCE (lt_stab) RESULT XML lv.",
            "CALL TRANSFORMATION id SOURCE data = lv_x(10) RESULT XML lv.",
            "CREATE DATA lr_data TYPE HANDLE lr_type->*.",
            "MERGE INTO zt AS a USING @itab AS it ON a~kf = it~kf WHEN MATCHED THEN DELETE.",
            "MERGE INTO zt AS a USING zs AS b ON 1 = 1 WHEN MATCHED THEN UPDATE ALL WHEN NOT MATCHED THEN INSERT ALL.",
            "METHODS on_event FOR ENTITY EVENT events FOR root~raised.",
            "READ TABLE lt INTO ls INDEX 1 USING KEY name.",
            "READ TABLE lt INTO ls WITH KEY id = 1.",
            "TYPES ty TYPE SORTED TABLE OF ts WITH NON-UNIQUE KEY and.",
            "INSERT row INTO TABLE tab REFERENCE INTO ref.",
            "ASSIGN ref->(comp) TO <fs>.",
            "UPDATE ztab SET column = column + 2 WHERE val = 1.",
            "APPEND LINES OF a FROM 1 TO 2 TO b.",
        ] {
            assert_eq!(check(abap), None, "{abap}");
        }
    }
}
