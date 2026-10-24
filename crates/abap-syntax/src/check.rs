//! Syntax errors in statements whose kind is known: what the kind's grammar
//! does not allow, like a keyword as a column (`SELECT WHERE FROM t`) or
//! parameter sections out of order. The checks cover errors seen in
//! abaplint's test cases (<https://github.com/abaplint/abaplint>, MIT); a
//! statement without an error found is not necessarily correct.
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
    if matches!(s.kind, K::Comment | K::Unknown | K::NativeSQL) {
        return None;
    }
    let c = Check::new(file, s);
    let error = match s.kind {
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
        _ => None,
    };
    error.or_else(|| c.dynamic_component())
}

struct Check<'a> {
    t: Vec<&'a Token>,
    w: Vec<String>,
    depth: Vec<u32>,
}

fn error(token: usize, message: &'static str) -> Option<SyntaxError> {
    Some(SyntaxError { token, message })
}

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
        Check { t, w, depth }
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

    fn select(&self) -> Option<SyntaxError> {
        for i in 0..self.w.len() {
            if self.word(i) != "SELECT" {
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

    fn method_def(&self) -> Option<SyntaxError> {
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
                if matches!(expr.kind, ExprKind::Binary { .. }) {
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
}

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
    }

    #[test]
    fn correct_statements_pass() {
        for abap in [
            "SELECT FROM ztab FIELDS col INTO TABLE @DATA(lt).",
            "SELECT SINGLE FOR UPDATE * FROM ztab INTO @wa WHERE a = 1.",
            "SELECT col FROM ztab INTO TABLE @lt PACKAGE SIZE 10.",
            "METHODS foo IMPORTING a TYPE i EXPORTING b TYPE i RAISING cx.",
            "CONCATENATE a b INTO c SEPARATED BY space.",
            "INSERT row INTO TABLE tab REFERENCE INTO ref.",
            "ASSIGN ref->(comp) TO <fs>.",
            "UPDATE ztab SET column = column + 2 WHERE val = 1.",
            "APPEND LINES OF a FROM 1 TO 2 TO b.",
        ] {
            assert_eq!(check(abap), None, "{abap}");
        }
    }
}
