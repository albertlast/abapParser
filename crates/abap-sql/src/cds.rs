//! CDS data definitions (DDL sources): what a view reads, its joins,
//! associations and parameters, and what each element is made of.
//!
//! Tolerant: what is not understood is skipped, the rest still comes out.
//! Covers DEFINE [ROOT] VIEW [ENTITY], projection views, table functions,
//! abstract and custom entities, hierarchies and EXTEND VIEW [ENTITY].

use abap_model as m;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    Word,
    Text,
    Number,
    Punct,
}

#[derive(Debug, Clone)]
struct Token {
    kind: Kind,
    text: String,
    start: usize,
    end: usize,
}

impl Token {
    fn is(&self, word: &str) -> bool {
        self.kind == Kind::Word && self.text.eq_ignore_ascii_case(word)
    }

    fn punct(&self, p: &str) -> bool {
        self.kind == Kind::Punct && self.text == p
    }
}

fn lex(src: &str) -> Vec<Token> {
    let b = src.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < b.len() {
        let c = b[i];
        if c.is_ascii_whitespace() {
            i += 1;
        } else if (c == b'/' || c == b'-') && b.get(i + 1) == Some(&c) {
            // `//` and `--` comments
            while i < b.len() && b[i] != b'\n' {
                i += 1;
            }
        } else if c == b'/' && b.get(i + 1) == Some(&b'*') {
            i += 2;
            while i + 1 < b.len() && !(b[i] == b'*' && b[i + 1] == b'/') {
                i += 1;
            }
            i = (i + 2).min(b.len());
        } else if c == b'\'' {
            let start = i;
            i += 1;
            while i < b.len() {
                if b[i] == b'\'' {
                    // '' inside a literal
                    if b.get(i + 1) == Some(&b'\'') {
                        i += 2;
                        continue;
                    }
                    i += 1;
                    break;
                }
                i += 1;
            }
            out.push(token(src, Kind::Text, start, i));
        } else if c.is_ascii_digit() {
            let start = i;
            while i < b.len()
                && (b[i].is_ascii_digit()
                    || b[i] == b'.' && b.get(i + 1).is_some_and(u8::is_ascii_digit))
            {
                i += 1;
            }
            out.push(token(src, Kind::Number, start, i));
        } else if c.is_ascii_alphabetic()
            || c == b'_'
            || c == b'$'
            || c == b'#'
            || (c == b'/'
                && b.get(i + 1).is_some_and(|n| n.is_ascii_alphanumeric())
                && namespace(b, i))
        {
            let start = i;
            if c == b'/' {
                // /ABC/NAME
                i += 1;
                while i < b.len() && b[i] != b'/' {
                    i += 1;
                }
                i += 1;
            } else {
                i += 1;
            }
            while i < b.len() && (b[i].is_ascii_alphanumeric() || b[i] == b'_') {
                i += 1;
            }
            out.push(token(src, Kind::Word, start, i));
        } else {
            let two = src.get(i..i + 2).unwrap_or("");
            let len = if matches!(two, ".." | "<=" | ">=" | "<>" | "=>" | "!=") {
                2
            } else {
                // one character, whatever it is
                src[i..].chars().next().map_or(1, char::len_utf8)
            };
            out.push(token(src, Kind::Punct, i, i + len));
            i += len;
        }
    }
    out
}

/// A namespace like `/ABC/` starts here: letters up to the next '/'.
fn namespace(b: &[u8], i: usize) -> bool {
    let mut j = i + 1;
    while j < b.len() && (b[j].is_ascii_alphanumeric() || b[j] == b'_') {
        j += 1;
    }
    b.get(j) == Some(&b'/') && j > i + 1
}

fn token(src: &str, kind: Kind, start: usize, end: usize) -> Token {
    Token {
        kind,
        text: src[start..end].to_string(),
        start,
        end,
    }
}

/// Words ending the clause of a view at the top level.
const CLAUSE_END: &[&str] = &[
    "inner",
    "left",
    "right",
    "cross",
    "join",
    "association",
    "composition",
    "where",
    "group",
    "having",
    "union",
    "except",
    "intersect",
    "redirected",
];

struct Parser<'a> {
    src: &'a str,
    t: Vec<Token>,
}

impl Parser<'_> {
    fn text(&self, from: usize, to: usize) -> String {
        if from >= to || from >= self.t.len() {
            return String::new();
        }
        let to = to.min(self.t.len());
        self.src[self.t[from].start..self.t[to - 1].end]
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    }

    fn is(&self, i: usize, word: &str) -> bool {
        self.t.get(i).is_some_and(|t| t.is(word))
    }

    fn punct(&self, i: usize, p: &str) -> bool {
        self.t.get(i).is_some_and(|t| t.punct(p))
    }

    /// The index after the bracket closing the one at `i`.
    fn close(&self, i: usize) -> usize {
        let mut depth = 0i32;
        let mut j = i;
        while j < self.t.len() {
            let x = &self.t[j];
            if x.kind == Kind::Punct {
                match x.text.as_str() {
                    "(" | "[" | "{" => depth += 1,
                    ")" | "]" | "}" => {
                        depth -= 1;
                        if depth == 0 {
                            return j + 1;
                        }
                    }
                    _ => {}
                }
            }
            j += 1;
        }
        self.t.len()
    }

    /// Skips an annotation at `i` (`@`), returning its name, its value as
    /// written and the index after it.
    fn annotation(&self, i: usize) -> (String, String, usize) {
        let mut j = i + 1;
        if self.punct(j, "<") {
            j += 1;
        }
        let mut name = String::new();
        while let Some(x) = self.t.get(j) {
            if x.kind == Kind::Word || x.punct(".") {
                name.push_str(&x.text);
                j += 1;
            } else {
                break;
            }
        }
        if !self.punct(j, ":") {
            return (name, String::new(), j);
        }
        j += 1;
        let from = j;
        if self.punct(j, "{") || self.punct(j, "[") {
            j = self.close(j);
        } else if j < self.t.len() {
            j += 1;
            // #ENUM.VALUE or a negative number
            while self.punct(j, ".") && self.t.get(j + 1).is_some_and(|x| x.kind == Kind::Word) {
                j += 2;
            }
        }
        (name, self.text(from, j), j)
    }

    /// The end of a clause starting at `i`: the next top level word of
    /// `ends`, an opening brace, or a semicolon.
    fn clause_end(&self, i: usize, ends: &[&str]) -> usize {
        let mut j = i;
        while j < self.t.len() {
            let x = &self.t[j];
            if x.punct("(") || x.punct("[") {
                j = self.close(j);
                continue;
            }
            if x.punct("{") || x.punct(";") || x.punct("}") {
                return j;
            }
            if x.kind == Kind::Word && ends.iter().any(|e| x.is(e)) {
                return j;
            }
            j += 1;
        }
        j
    }
}

/// Parses a DDL source.
pub fn parse(src: &str) -> m::CdsDefinition {
    let p = Parser { src, t: lex(src) };
    let mut d = m::CdsDefinition {
        schema_version: m::SCHEMA_VERSION,
        kind: String::new(),
        name: String::new(),
        sql_view: None,
        label: None,
        parameters: Vec::new(),
        sources: Vec::new(),
        associations: Vec::new(),
        elements: Vec::new(),
        where_text: None,
        implemented_by: None,
        span: span_of(src, 0, src.len()),
    };
    let mut i = 0;
    // annotations of the entity
    while i < p.t.len() && !p.is(i, "define") && !p.is(i, "extend") {
        if p.punct(i, "@") {
            let (name, value, next) = p.annotation(i);
            let value = value.trim_matches('\'').to_string();
            if name.eq_ignore_ascii_case("AbapCatalog.sqlViewName") {
                d.sql_view = Some(value.to_ascii_uppercase());
            } else if name.eq_ignore_ascii_case("EndUserText.label") {
                d.label = Some(value);
            }
            i = next;
        } else {
            i += 1;
        }
    }
    let Some(start) = p.t.get(i) else {
        return d;
    };
    d.span = span_of(src, start.start, src.len());
    let extend = start.is("extend");
    i += 1;
    // the kind: the words up to the name
    const KIND_WORDS: &[&str] = &[
        "root",
        "view",
        "entity",
        "table",
        "function",
        "abstract",
        "custom",
        "hierarchy",
        "transient",
        "external",
        "projection",
    ];
    let mut kind = vec![if extend { "extend" } else { "" }.to_string()];
    while let Some(x) = p.t.get(i) {
        if x.kind == Kind::Word
            && KIND_WORDS.iter().any(|w| x.is(w))
            && p.t.get(i + 1).is_some_and(|n| n.kind == Kind::Word)
        {
            kind.push(x.text.to_ascii_lowercase());
            i += 1;
        } else {
            break;
        }
    }
    d.kind = kind
        .into_iter()
        .filter(|k| !k.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    let Some(name) = p.t.get(i) else { return d };
    d.name = name.text.clone();
    i += 1;
    if extend {
        // EXTEND VIEW [ENTITY] name WITH { elements }
        if let Some(open) = (i..p.t.len()).find(|j| p.punct(*j, "{")) {
            d.sources.push(m::CdsSource {
                name: d.name.to_ascii_uppercase(),
                alias: None,
                role: "extended".to_string(),
                on: None,
            });
            let close = p.close(open);
            elements(&p, open + 1, close - 1, &mut d);
        }
        return d;
    }
    // WITH PARAMETERS p : type, ...
    if p.is(i, "with") && p.is(i + 1, "parameters") {
        i += 2;
        loop {
            while p.punct(i, "@") {
                i = p.annotation(i).2;
            }
            let Some(n) = p.t.get(i).filter(|x| x.kind == Kind::Word) else {
                break;
            };
            if !p.punct(i + 1, ":") {
                break;
            }
            let end = (i + 2..p.t.len())
                .find(|j| {
                    p.punct(*j, ",") || p.is(*j, "as") || p.is(*j, "returns") || p.punct(*j, "{")
                })
                .unwrap_or(p.t.len());
            d.parameters.push(m::CdsParameter {
                name: n.text.clone(),
                typing: p.text(i + 2, end),
            });
            i = end;
            if p.punct(i, ",") {
                i += 1;
            } else {
                break;
            }
        }
    }
    // table function: RETURNS { fields } IMPLEMENTED BY METHOD cl=>m
    if p.is(i, "returns") {
        if let Some(open) = (i..p.t.len()).find(|j| p.punct(*j, "{")) {
            let close = p.close(open);
            elements(&p, open + 1, close - 1, &mut d);
            i = close;
        }
        if let Some(k) = (i..p.t.len()).find(|j| p.is(*j, "method")) {
            let end = (k + 1..p.t.len())
                .find(|j| p.punct(*j, ";"))
                .unwrap_or(p.t.len());
            d.implemented_by = Some(p.text(k + 1, end).replace(' ', ""));
        }
        return d;
    }
    // abstract and custom entities: { fields }
    if p.punct(i, "{") {
        let close = p.close(i);
        elements(&p, i + 1, close - 1, &mut d);
        return d;
    }
    // PROVIDER CONTRACT ... AS SELECT / AS PROJECTION ON / AS PARENT CHILD HIERARCHY
    while i < p.t.len() && !p.is(i, "as") {
        i += 1;
    }
    let mut role = "from";
    let mut first = true;
    while i < p.t.len() {
        if p.is(i, "as") && p.is(i + 1, "projection") && p.is(i + 2, "on") {
            i = source(&p, i + 3, "projection", &mut d);
        } else if p.is(i, "as") && p.is(i + 1, "select") || p.is(i, "select") {
            i += if p.is(i, "as") { 2 } else { 1 };
            if p.is(i, "distinct") {
                i += 1;
            }
            if p.is(i, "from") {
                i = source(&p, i + 1, role, &mut d);
            }
        } else if p.is(i, "source") && p.t.get(i + 1).is_some_and(|x| x.kind == Kind::Word) {
            // hierarchy: SOURCE name CHILD TO PARENT ASSOCIATION _x
            i = source(&p, i + 1, "from", &mut d);
        } else if let Some(join) = join_at(&p, i) {
            let (kind, at) = join;
            i = source(&p, at, &kind, &mut d);
            if p.is(i, "on") {
                let end = p.clause_end(i + 1, CLAUSE_END);
                if let Some(s) = d.sources.last_mut() {
                    s.on = Some(p.text(i + 1, end));
                }
                i = end;
            }
        } else if p.is(i, "association") || p.is(i, "composition") {
            i = association(&p, i, &mut d);
        } else if p.punct(i, "{") {
            let close = p.close(i);
            if first {
                elements(&p, i + 1, close - 1, &mut d);
                first = false;
            }
            i = close;
        } else if p.is(i, "where") {
            let end = p.clause_end(i + 1, &["group", "having", "union", "except", "intersect"]);
            if role == "from" {
                d.where_text = Some(p.text(i + 1, end));
            }
            i = end;
        } else if p.is(i, "union") || p.is(i, "except") || p.is(i, "intersect") {
            role = "union";
            i += 1;
        } else {
            i += 1;
        }
    }
    resolve_fields(&mut d);
    d
}

/// A join at `i`: its kind and where its data source starts.
fn join_at(p: &Parser, i: usize) -> Option<(String, usize)> {
    let mut words = Vec::new();
    let mut j = i;
    while j < p.t.len() && j < i + 8 {
        let x = &p.t[j];
        if x.is("join") {
            if words.is_empty() {
                words.push("inner".to_string());
            }
            // the cardinality of a join says nothing about where data comes from
            let kind: Vec<String> = words
                .into_iter()
                .filter(|w| matches!(w.as_str(), "inner" | "left" | "right" | "cross" | "outer"))
                .collect();
            return Some((format!("{} join", kind.join(" ")), j + 1));
        }
        let allowed = [
            "inner", "left", "right", "cross", "outer", "to", "one", "many", "exact",
        ];
        if x.kind != Kind::Word || !allowed.iter().any(|a| x.is(a)) {
            return None;
        }
        words.push(x.text.to_ascii_lowercase());
        j += 1;
    }
    None
}

/// A data source at `i`: name, parameters in parentheses, alias.
fn source(p: &Parser, i: usize, role: &str, d: &mut m::CdsDefinition) -> usize {
    let Some(name) = p.t.get(i).filter(|x| x.kind == Kind::Word) else {
        return i;
    };
    let mut j = i + 1;
    if p.punct(j, "(") {
        j = p.close(j);
    }
    let mut alias = None;
    if p.is(j, "as") && p.t.get(j + 1).is_some_and(|x| x.kind == Kind::Word) {
        alias = Some(p.t[j + 1].text.clone());
        j += 2;
    }
    d.sources.push(m::CdsSource {
        name: name.text.to_ascii_uppercase(),
        alias,
        role: role.to_string(),
        on: None,
    });
    j
}

/// ASSOCIATION [card] TO [PARENT] target AS _x ON cond, COMPOSITION [card]
/// OF target AS _x, and the newer ASSOCIATION OF EXACT ONE TO MANY target.
fn association(p: &Parser, i: usize, d: &mut m::CdsDefinition) -> usize {
    let composition = p.is(i, "composition");
    let mut j = i + 1;
    let mut cardinality = None;
    if p.punct(j, "[") {
        let close = p.close(j);
        cardinality = Some(p.text(j, close).replace(' ', ""));
        j = close;
    }
    let mut kind = if composition {
        "composition"
    } else {
        "association"
    }
    .to_string();
    // the words up to the target: TO, OF, PARENT, EXACT ONE TO MANY ...
    let skip = ["to", "of", "exact", "one", "many", "redirected"];
    while let Some(x) = p.t.get(j) {
        if x.is("parent") {
            kind = "to parent".to_string();
            j += 1;
        } else if x.kind == Kind::Word && skip.iter().any(|s| x.is(s)) {
            j += 1;
        } else {
            break;
        }
    }
    let Some(target) = p.t.get(j).filter(|x| x.kind == Kind::Word) else {
        return j;
    };
    j += 1;
    let mut name = target.text.clone();
    if p.is(j, "as") && p.t.get(j + 1).is_some_and(|x| x.kind == Kind::Word) {
        name = p.t[j + 1].text.clone();
        j += 2;
    }
    let mut on = None;
    if p.is(j, "on") {
        let end = p.clause_end(j + 1, CLAUSE_END);
        on = Some(p.text(j + 1, end));
        j = end;
    }
    d.associations.push(m::CdsAssociation {
        name,
        target: target.text.to_ascii_uppercase(),
        cardinality,
        kind,
        on,
    });
    j
}

/// The elements between `from` and `to`, split at top level commas and
/// semicolons.
fn elements(p: &Parser, from: usize, to: usize, d: &mut m::CdsDefinition) {
    let mut start = from;
    let mut j = from;
    while j <= to {
        // fields of entities and table functions end with ';'
        let end_here = j == to || p.punct(j, ",") || p.punct(j, ";");
        if end_here {
            element(p, start, j, d);
            start = j + 1;
            j += 1;
            continue;
        }
        if p.punct(j, "(") || p.punct(j, "[") || p.punct(j, "{") {
            j = p.close(j);
            continue;
        }
        if p.punct(j, "@") {
            j = p.annotation(j).2;
            continue;
        }
        j += 1;
    }
}

fn element(p: &Parser, from: usize, to: usize, d: &mut m::CdsDefinition) {
    let mut i = from;
    while i < to && p.punct(i, "@") {
        i = p.annotation(i).2;
    }
    let mut key = false;
    while i < to && (p.is(i, "key") || p.is(i, "virtual")) {
        key |= p.is(i, "key");
        i += 1;
    }
    if i >= to {
        return;
    }
    // trailing annotations `@<...`
    let mut end = to;
    if let Some(k) = (i..to).find(|k| p.punct(*k, "@")) {
        end = k;
    }
    // name : type, or _assoc : redirected to x
    if p.punct(i + 1, ":") && p.t[i].kind == Kind::Word {
        let assoc = p.is(i + 2, "redirected") || p.is(i + 2, "association");
        d.elements.push(m::CdsElement {
            name: p.t[i].text.clone(),
            key,
            text: p.text(i + 2, end),
            fields: Vec::new(),
            association: assoc,
        });
        return;
    }
    // expression [AS alias]
    let mut alias = None;
    let mut expr_end = end;
    if end >= i + 2
        && p.is(end - 2, "as")
        && p.t[end - 1].kind == Kind::Word
        && depth_at(p, i, end - 2) == 0
    {
        alias = Some(p.t[end - 1].text.clone());
        expr_end = end - 2;
    }
    let text = p.text(i, expr_end);
    let paths = paths(p, i, expr_end);
    let name = alias.clone().unwrap_or_else(|| {
        paths
            .last()
            .map(|x| x.rsplit('.').next().unwrap_or(x).to_string())
            .unwrap_or_else(|| text.clone())
    });
    d.elements.push(m::CdsElement {
        name,
        key,
        text,
        fields: paths,
        association: false,
    });
}

/// Bracket depth of `at` counted from `from`.
fn depth_at(p: &Parser, from: usize, at: usize) -> i32 {
    let mut depth = 0;
    for x in &p.t[from..at] {
        if x.punct("(") || x.punct("[") {
            depth += 1;
        } else if x.punct(")") || x.punct("]") {
            depth -= 1;
        }
    }
    depth
}

/// Words of expressions that are no fields.
const EXPRESSION_WORDS: &[&str] = &[
    "case", "when", "then", "else", "end", "and", "or", "not", "is", "null", "initial", "between",
    "like", "escape", "distinct", "all", "true", "false", "as", "key",
];

/// The fields an expression reads, as written: `alias.field`, `field`,
/// `_assoc.field`, `$parameters.p`.
fn paths(p: &Parser, from: usize, to: usize) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut j = from;
    while j < to {
        let x = &p.t[j];
        // CAST( x AS type ): the type is no field
        if x.is("as") {
            while j < to && !p.punct(j, ")") {
                j += 1;
            }
            continue;
        }
        if x.kind != Kind::Word
            || x.text.starts_with('#')
            || EXPRESSION_WORDS.iter().any(|w| x.is(w))
            || p.punct(j + 1, "(")
        {
            j += 1;
            continue;
        }
        let mut path = x.text.clone();
        let mut k = j + 1;
        while p.punct(k, ".") && p.t.get(k + 1).is_some_and(|n| n.kind == Kind::Word) {
            path.push('.');
            path.push_str(&p.t[k + 1].text);
            k += 2;
        }
        // a filter or parameters of a path: _assoc[...].field
        while p.punct(k, "[") {
            k = p.close(k);
            while p.punct(k, ".") && p.t.get(k + 1).is_some_and(|n| n.kind == Kind::Word) {
                path.push('.');
                path.push_str(&p.t[k + 1].text);
                k += 2;
            }
        }
        // a typed literal like abap.int4'1'
        if !p.t.get(k).is_some_and(|n| n.kind == Kind::Text) && !out.contains(&path) {
            out.push(path);
        }
        j = k;
    }
    out
}

/// Fields as `SOURCE.FIELD`: aliases resolved, unqualified fields given the
/// only source of the view.
fn resolve_fields(d: &mut m::CdsDefinition) {
    let main: Vec<&m::CdsSource> = d.sources.iter().filter(|s| s.role != "union").collect();
    let lookup = |first: &str| -> Option<String> {
        main.iter()
            .find(|s| {
                s.alias
                    .as_deref()
                    .is_some_and(|a| a.eq_ignore_ascii_case(first))
                    || s.name.eq_ignore_ascii_case(first)
            })
            .map(|s| s.name.clone())
    };
    let assocs: Vec<String> = d
        .associations
        .iter()
        .map(|a| a.name.to_ascii_uppercase())
        .collect();
    let single = (main.len() == 1).then(|| main[0].name.clone());
    for e in &mut d.elements {
        let mut fields = Vec::new();
        for path in &e.fields {
            let upper = path.to_ascii_uppercase();
            let resolved = match upper.split_once('.') {
                Some((first, rest)) => match lookup(first) {
                    Some(s) => format!("{s}.{rest}"),
                    None => upper.clone(),
                },
                None if assocs.contains(&upper) => {
                    e.association = true;
                    upper.clone()
                }
                None => match &single {
                    Some(s) => format!("{s}.{upper}"),
                    None => upper.clone(),
                },
            };
            if !fields.contains(&resolved) {
                fields.push(resolved);
            }
        }
        e.fields = fields;
    }
}

fn span_of(src: &str, start: usize, end: usize) -> m::Span {
    let pos = |at: usize| {
        let before = &src[..at.min(src.len())];
        let row = before.matches('\n').count() as u32 + 1;
        let col = before.rsplit('\n').next().map_or(0, |l| l.chars().count()) as u32 + 1;
        m::Position { row, col }
    };
    let end = src[..end].trim_end().len();
    m::Span {
        start: pos(start),
        end: pos(end),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn view_with_association() {
        let d = parse(
            "@EndUserText.label: 'Country/Region'
@Analytics: { dataCategory: #DIMENSION, dataExtraction.enabled: true }
@AbapCatalog.sqlViewName: 'IFICOUNTRY'
@ObjectModel.supportedCapabilities: [#SQL_DATA_SOURCE,
                                     #CDS_MODELING_DATA_SOURCE]
define view I_Country
  as select from t005
  association [0..*] to I_CountryText as _Text on $projection.Country = _Text.Country
{
      @Search.defaultSearchElement: true
      @ObjectModel.text.association: '_Text'
  key land1  as Country,
      intca3 as CountryThreeLetterISOCode, // comment
      cast( xegld as abap.char(1) ) as IsEuropeanUnionMember,
      _Text
}",
        );
        assert_eq!(d.kind, "view");
        assert_eq!(d.name, "I_Country");
        assert_eq!(d.sql_view.as_deref(), Some("IFICOUNTRY"));
        assert_eq!(d.label.as_deref(), Some("Country/Region"));
        assert_eq!(d.sources.len(), 1);
        assert_eq!(
            (d.sources[0].name.as_str(), d.sources[0].role.as_str()),
            ("T005", "from")
        );
        let a = &d.associations[0];
        assert_eq!(
            (
                a.name.as_str(),
                a.target.as_str(),
                a.cardinality.as_deref(),
                a.kind.as_str()
            ),
            ("_Text", "I_COUNTRYTEXT", Some("[0..*]"), "association")
        );
        assert_eq!(a.on.as_deref(), Some("$projection.Country = _Text.Country"));
        let e: Vec<(&str, bool, Vec<&str>, bool)> = d
            .elements
            .iter()
            .map(|e| {
                (
                    e.name.as_str(),
                    e.key,
                    e.fields.iter().map(String::as_str).collect(),
                    e.association,
                )
            })
            .collect();
        assert_eq!(
            e,
            [
                ("Country", true, vec!["T005.LAND1"], false),
                (
                    "CountryThreeLetterISOCode",
                    false,
                    vec!["T005.INTCA3"],
                    false
                ),
                ("IsEuropeanUnionMember", false, vec!["T005.XEGLD"], false),
                ("_Text", false, vec!["_TEXT"], true),
            ]
        );
    }

    #[test]
    fn joins_parameters_where_union() {
        let d = parse(
            "define view entity zdemo_joins
  with parameters p_carrid : s_carr_id, @Consumption.hidden: true p_date : abap.dats
  as select from zdemo_abap_carr as _carr
    inner join zdemo_abap_flsch as _fl on _carr.carrid = _fl.carrid
    left outer to one join zdemo_abap_fli as _f on _f.connid = _fl.connid
  association [1] to zdemo_abap_carr as _c on _c.carrid = _carr.carrid
{
  key _carr.carrid,
      _fl.connid as Connection,
      concat( _carr.carrname, _fl.cityfrom ) as Text,
      case when _f.price > 100 then 'X' else '' end as Expensive,
      $parameters.p_date as KeyDate,
      _c.carrname as Name
}
where _carr.carrid = $parameters.p_carrid
union all select from zother as o { o.carrid, o.connid, o.x, o.y, o.z, o.n }",
        );
        assert_eq!(d.kind, "view entity");
        let params: Vec<(&str, &str)> = d
            .parameters
            .iter()
            .map(|x| (x.name.as_str(), x.typing.as_str()))
            .collect();
        assert_eq!(params, [("p_carrid", "s_carr_id"), ("p_date", "abap.dats")]);
        let sources: Vec<(&str, &str)> = d
            .sources
            .iter()
            .map(|s| (s.name.as_str(), s.role.as_str()))
            .collect();
        assert_eq!(
            sources,
            [
                ("ZDEMO_ABAP_CARR", "from"),
                ("ZDEMO_ABAP_FLSCH", "inner join"),
                ("ZDEMO_ABAP_FLI", "left outer join"),
                ("ZOTHER", "union"),
            ]
        );
        assert_eq!(
            d.sources[1].on.as_deref(),
            Some("_carr.carrid = _fl.carrid")
        );
        assert_eq!(
            d.where_text.as_deref(),
            Some("_carr.carrid = $parameters.p_carrid")
        );
        let fields: Vec<(&str, Vec<&str>)> = d
            .elements
            .iter()
            .map(|e| {
                (
                    e.name.as_str(),
                    e.fields.iter().map(String::as_str).collect(),
                )
            })
            .collect();
        assert_eq!(
            fields,
            [
                ("carrid", vec!["ZDEMO_ABAP_CARR.CARRID"]),
                ("Connection", vec!["ZDEMO_ABAP_FLSCH.CONNID"]),
                (
                    "Text",
                    vec!["ZDEMO_ABAP_CARR.CARRNAME", "ZDEMO_ABAP_FLSCH.CITYFROM"]
                ),
                ("Expensive", vec!["ZDEMO_ABAP_FLI.PRICE"]),
                ("KeyDate", vec!["$PARAMETERS.P_DATE"]),
                ("Name", vec!["_C.CARRNAME"]),
            ]
        );
        assert!(d.elements[0].key);
    }

    #[test]
    fn projections_table_functions_entities() {
        let d = parse(
            "define root view entity ZC_Travel provider contract transactional_query
  as projection on ZI_Travel as Travel
{
  key TravelId,
      _Booking : redirected to composition child ZC_Booking
}",
        );
        assert_eq!(d.kind, "root view entity");
        assert_eq!(
            (d.sources[0].name.as_str(), d.sources[0].role.as_str()),
            ("ZI_TRAVEL", "projection")
        );
        assert_eq!(d.elements[0].fields, ["ZI_TRAVEL.TRAVELID"]);
        assert!(d.elements[1].association);

        let d = parse(
            "@EndUserText.label: 'Function'
define table function zdemo_tf
  with parameters @Environment.systemField: #CLIENT p_clnt : abap.clnt
  returns {
    client : abap.clnt;
    carrid : s_carr_id;
  }
  implemented by method zcl_demo_amdp=>get_rows;",
        );
        assert_eq!(d.kind, "table function");
        assert_eq!(d.implemented_by.as_deref(), Some("zcl_demo_amdp=>get_rows"));
        assert_eq!(d.parameters[0].name, "p_clnt");

        let d = parse("define abstract entity zabs { col1: abap.char(25); col2 : abap.int4; }");
        assert_eq!(d.kind, "abstract entity");
        let fields: Vec<(&str, &str)> = d
            .elements
            .iter()
            .map(|e| (e.name.as_str(), e.text.as_str()))
            .collect();
        assert_eq!(fields, [("col1", "abap.char(25)"), ("col2", "abap.int4")]);

        let d = parse(
            "extend view entity zdemo_base with {
  _carr.carrname as CarrierName
}",
        );
        assert_eq!(d.kind, "extend view entity");
        assert_eq!(d.name, "zdemo_base");
        assert_eq!(d.elements[0].name, "CarrierName");

        let d = parse(
            "define view entity zc { key a.x as X }
  -- comment
  /* block */",
        );
        assert_eq!(d.kind, "view entity");
    }

    #[test]
    fn composition_and_parent() {
        let d = parse(
            "define root view entity ZI_Travel as select from ztravel
  composition [0..*] of ZI_Booking as _Booking
  association to parent ZI_Agency as _Agency on $projection.AgencyId = _Agency.AgencyId
{ key travel_id as TravelId, agency_id as AgencyId, _Booking, _Agency }",
        );
        let a: Vec<(&str, &str, &str)> = d
            .associations
            .iter()
            .map(|a| (a.name.as_str(), a.target.as_str(), a.kind.as_str()))
            .collect();
        assert_eq!(
            a,
            [
                ("_Booking", "ZI_BOOKING", "composition"),
                ("_Agency", "ZI_AGENCY", "to parent")
            ]
        );
        assert!(d.elements[2].association && d.elements[3].association);
    }
}
