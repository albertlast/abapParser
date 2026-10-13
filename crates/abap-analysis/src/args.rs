//! The actual arguments of a call: for each, the parameter it is passed to
//! and the token positions it covers, so that the data a statement reads
//! and writes can be matched to parameters.

use abap_model as m;
use abap_syntax::{File, Statement, StatementKind as K, Token};

/// Row and column of a token start.
pub(crate) type At = (u32, u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Section {
    /// passed to an importing parameter
    Exporting,
    /// receives an exporting parameter
    Importing,
    Changing,
    Receiving,
    Tables,
    /// FORM ... USING
    Using,
    Exceptions,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Actual {
    /// the parameter as written, lower case; `None` when unnamed or positional
    pub formal: Option<String>,
    pub section: Section,
    /// position in a positional list of its section, for PERFORM
    pub index: usize,
    pub first: At,
    pub last: At,
}

impl Actual {
    pub fn contains(&self, at: At) -> bool {
        self.first <= at && at <= self.last
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CallArgs {
    pub actuals: Vec<Actual>,
    /// tokens of the receiver before the method name, e.g. `lo->`
    pub receiver: Option<(At, At)>,
    /// first and last token of the call
    pub range: (At, At),
}

impl CallArgs {
    pub fn in_range(&self, at: At) -> bool {
        self.range.0 <= at && at <= self.range.1
    }

    pub fn in_receiver(&self, at: At) -> bool {
        self.receiver.is_some_and(|(a, b)| a <= at && at <= b)
    }

    /// The actuals passed to the parameters, matched by name, by position
    /// for forms, and the unnamed argument to the first importing parameter.
    pub fn bind<'a>(&'a self, parameters: &[m::ProcedureParameter]) -> Vec<(String, &'a Actual)> {
        let nth = |direction: m::Direction, i: usize| {
            parameters
                .iter()
                .filter(|p| p.direction == direction)
                .nth(i)
                .map(|p| p.name.clone())
        };
        let mut out = Vec::new();
        for a in &self.actuals {
            let formal = match (&a.formal, a.section) {
                (_, Section::Exceptions) => None,
                (Some(f), _) => Some(f.trim_start_matches('!').to_string()),
                (None, Section::Exporting) => nth(m::Direction::Importing, 0),
                (None, Section::Using) => nth(m::Direction::Using, a.index),
                (None, Section::Changing) => nth(m::Direction::Changing, a.index),
                (None, Section::Tables) => nth(m::Direction::Tables, a.index),
                (None, _) => None,
            };
            if let Some(f) = formal {
                out.push((f, a));
            }
        }
        out
    }
}

fn at(t: &Token) -> At {
    (t.start.row, t.start.col)
}

fn section_of(word: &str) -> Option<Section> {
    Some(match word {
        "EXPORTING" => Section::Exporting,
        "IMPORTING" => Section::Importing,
        "CHANGING" => Section::Changing,
        "RECEIVING" => Section::Receiving,
        "TABLES" => Section::Tables,
        "USING" => Section::Using,
        "EXCEPTIONS" => Section::Exceptions,
        _ => return None,
    })
}

/// Parses the arguments in `toks[from..to]`. Before the first section
/// keyword the default section applies; without one, tokens are skipped.
/// With `positional`, operands separated by spaces are single arguments.
fn parse(
    toks: &[&Token],
    from: usize,
    to: usize,
    default: Option<Section>,
    positional: bool,
) -> Vec<Actual> {
    let mut out: Vec<Actual> = Vec::new();
    let mut section = default;
    let mut current: Option<Actual> = None;
    let mut counts: Vec<(Section, usize)> = Vec::new();
    let mut depth = 0usize;
    let mut i = from;
    while i < to {
        let t = toks[i];
        if depth == 0 {
            let word = t.text.to_ascii_uppercase();
            let named = toks
                .get(i + 1)
                .is_some_and(|n| n.text == "=" && n.ws_before && i + 2 < to);
            if let (Some(s), false) = (section_of(&word), named) {
                out.extend(current.take());
                section = Some(s);
                i += 1;
                continue;
            }
            let Some(sec) = section else {
                i += 1;
                continue;
            };
            if named {
                out.extend(current.take());
                let first = at(toks[i + 2]);
                current = Some(Actual {
                    formal: Some(t.text.to_ascii_lowercase()),
                    section: sec,
                    index: 0,
                    first,
                    last: first,
                });
                i += 2;
                continue;
            }
            let split = positional && t.ws_before && current.is_some();
            if current.is_none() || split {
                out.extend(current.take());
                let index = match counts.iter_mut().find(|(s, _)| *s == sec) {
                    Some((_, n)) => {
                        *n += 1;
                        *n - 1
                    }
                    None => {
                        counts.push((sec, 1));
                        0
                    }
                };
                current = Some(Actual {
                    formal: None,
                    section: sec,
                    index,
                    first: at(t),
                    last: at(t),
                });
            }
        }
        if let Some(c) = current.as_mut() {
            c.last = at(t);
        }
        match t.text.as_str() {
            "(" | "[" => depth += 1,
            ")" | "]" => depth = depth.saturating_sub(1),
            _ => {}
        }
        i += 1;
    }
    out.extend(current);
    out
}

/// The token closing the parenthesis opened at `open`.
fn close_of(toks: &[&Token], open: usize) -> usize {
    let mut depth = 0usize;
    for (k, t) in toks.iter().enumerate().skip(open) {
        match t.text.as_str() {
            "(" | "[" => depth += 1,
            ")" | "]" => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return k;
                }
            }
            _ => {}
        }
    }
    toks.len().saturating_sub(1)
}

fn opens_args(toks: &[&Token], k: usize) -> bool {
    toks.get(k).is_some_and(|t| t.text == "(" && !t.ws_before)
}

/// The arguments of a call in a statement; `nth` counts the earlier calls
/// of the statement with the same kind and name.
pub(crate) fn call_args(file: &File, s: &Statement, c: &m::Call, nth: usize) -> Option<CallArgs> {
    if c.dynamic {
        return None;
    }
    let toks: Vec<&Token> = s.tokens.iter().map(|i| &file.tokens()[*i]).collect();
    let mut end = toks.len();
    while end > 0 && matches!(toks[end - 1].text.as_str(), "." | ",") {
        end -= 1;
    }
    if end == 0 {
        return None;
    }
    let whole = (at(toks[0]), at(toks[end - 1]));
    let statement = |default: Option<Section>, positional: bool| CallArgs {
        actuals: parse(&toks, 0, end, default, positional),
        receiver: None,
        range: whole,
    };
    let paren = |name: usize, open: usize, receiver: Option<(At, At)>| {
        let close = close_of(&toks, open).min(end.saturating_sub(1));
        CallArgs {
            actuals: parse(&toks, open + 1, close, Some(Section::Exporting), false),
            receiver,
            range: (receiver.map_or(at(toks[name]), |r| r.0), at(toks[close])),
        }
    };
    match c.kind {
        m::CallKind::Function if s.kind == K::CallFunction => Some(statement(None, false)),
        m::CallKind::Form if s.kind == K::Perform => Some(statement(None, true)),
        m::CallKind::Constructor if s.kind == K::CreateObject => Some(statement(None, false)),
        m::CallKind::Constructor => {
            let k = (0..end)
                .filter(|k| toks[*k].text.eq_ignore_ascii_case("NEW") && opens_args(&toks, k + 2))
                .nth(nth)?;
            Some(paren(k, k + 2, None))
        }
        m::CallKind::Method => {
            let name = c.name.rsplit(['>', ' ']).next().unwrap_or(&c.name);
            let call_method = toks[0].text.eq_ignore_ascii_case("CALL")
                && toks
                    .get(1)
                    .is_some_and(|t| t.text.eq_ignore_ascii_case("METHOD"));
            let k = (0..end)
                .filter(|k| {
                    toks[*k].text.eq_ignore_ascii_case(name)
                        && (opens_args(&toks, k + 1) || call_method)
                })
                .nth(nth)?;
            // the receiver: the tokens joined to the name, `lo->`, `zcl=>`
            let mut j = k;
            while j > 0 && !toks[j].ws_before {
                j -= 1;
            }
            let receiver = (j < k).then(|| (at(toks[j]), at(toks[k - 1])));
            if opens_args(&toks, k + 1) {
                Some(paren(k, k + 1, receiver))
            } else {
                Some(CallArgs {
                    actuals: parse(&toks, k + 1, end, None, false),
                    receiver,
                    range: (receiver.map_or(at(toks[k]), |r| r.0), at(toks[end - 1])),
                })
            }
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(src: &str, kind: m::CallKind, name: &str, nth: usize) -> CallArgs {
        let file = abap_syntax::parse(src);
        let s = file
            .statements
            .iter()
            .find(|s| !matches!(s.kind, K::Comment | K::Empty))
            .unwrap();
        let mut c = crate::calls::new_call(kind, name.to_string(), false, Vec::new());
        c.text = file.statement_text(s);
        call_args(&file, s, &c, nth).unwrap()
    }

    fn summary(a: &CallArgs) -> Vec<(Option<String>, Section, usize, At)> {
        a.actuals
            .iter()
            .map(|x| (x.formal.clone(), x.section, x.index, x.first))
            .collect()
    }

    #[test]
    fn functional_and_statement_calls() {
        let a = args(
            "x = m( a = 1 b = c + d ) + lo->n( 2 ).",
            m::CallKind::Method,
            "m",
            0,
        );
        assert_eq!(
            summary(&a),
            [
                (Some("a".into()), Section::Exporting, 0, (1, 12)),
                (Some("b".into()), Section::Exporting, 0, (1, 18)),
            ]
        );
        assert_eq!(a.range, ((1, 5), (1, 24)));
        let n = args(
            "x = m( a = 1 b = c + d ) + lo->n( 2 ).",
            m::CallKind::Method,
            "n",
            0,
        );
        assert_eq!(summary(&n), [(None, Section::Exporting, 0, (1, 35))]);
        assert_eq!(n.receiver, Some(((1, 28), (1, 30))));

        let s = args(
            "CALL METHOD lo->m EXPORTING a = 1 IMPORTING e = DATA(x) RECEIVING r = y.",
            m::CallKind::Method,
            "m",
            0,
        );
        let sections: Vec<_> = s
            .actuals
            .iter()
            .map(|a| (a.formal.as_deref().unwrap(), a.section))
            .collect();
        assert_eq!(
            sections,
            [
                ("a", Section::Exporting),
                ("e", Section::Importing),
                ("r", Section::Receiving)
            ]
        );
    }

    #[test]
    fn function_modules_and_forms() {
        let f = args(
            "CALL FUNCTION 'Z_FM' DESTINATION 'NONE' EXPORTING iv = 1 TABLES et = lt EXCEPTIONS others = 1.",
            m::CallKind::Function,
            "Z_FM",
            0,
        );
        let sections: Vec<_> = f
            .actuals
            .iter()
            .map(|a| (a.formal.as_deref().unwrap(), a.section))
            .collect();
        assert_eq!(
            sections,
            [
                ("iv", Section::Exporting),
                ("et", Section::Tables),
                ("others", Section::Exceptions)
            ]
        );
        let p = args(
            "PERFORM f IN PROGRAM zp USING a b-c 'X' CHANGING d TABLES t.",
            m::CallKind::Form,
            "f",
            0,
        );
        let positions: Vec<_> = p.actuals.iter().map(|a| (a.section, a.index)).collect();
        assert_eq!(
            positions,
            [
                (Section::Using, 0),
                (Section::Using, 1),
                (Section::Using, 2),
                (Section::Changing, 0),
                (Section::Tables, 0)
            ]
        );
        let params = |names: &[(&str, m::Direction)]| -> Vec<m::ProcedureParameter> {
            names
                .iter()
                .map(|(n, d)| m::ProcedureParameter {
                    name: n.to_string(),
                    direction: *d,
                })
                .collect()
        };
        let bound: Vec<String> = p
            .bind(&params(&[
                ("p1", m::Direction::Using),
                ("p2", m::Direction::Using),
                ("p3", m::Direction::Using),
                ("c1", m::Direction::Changing),
                ("t1", m::Direction::Tables),
            ]))
            .into_iter()
            .map(|(f, _)| f)
            .collect();
        assert_eq!(bound, ["p1", "p2", "p3", "c1", "t1"]);
    }

    #[test]
    fn constructors() {
        let n = args(
            "z = NEW lcl( 3 ).",
            m::CallKind::Constructor,
            "constructor",
            0,
        );
        assert_eq!(n.actuals.len(), 1);
        let o = args(
            "CREATE OBJECT lo EXPORTING iv = 1.",
            m::CallKind::Constructor,
            "constructor",
            0,
        );
        assert_eq!(o.actuals[0].formal.as_deref(), Some("iv"));
    }
}
