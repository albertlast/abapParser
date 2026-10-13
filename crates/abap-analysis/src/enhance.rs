//! Code of enhancement implementations put where it runs: after an
//! `ENHANCEMENT-POINT`, in place of an `ENHANCEMENT-SECTION`, or at the begin
//! or end of a form, function module, method or include.
//!
//! The code goes into the source on one line, the line of its position, so
//! the rows of the rest of the source stay as they are.

use abap_source::Hook;
use abap_syntax::{File, StatementKind as K, Token, TokenKind};

/// Where in a source the code of a hook goes.
struct Edit {
    /// byte offset the code is inserted at
    at: usize,
    /// bytes from `at` on that the code replaces, for sections
    replaces: usize,
    code: String,
}

/// The sources with the hooks put in, for the units that change, and the
/// hooks that found their position.
pub(crate) fn apply<'a>(
    units: &[(&str, &File)],
    hooks: &'a [Hook],
) -> (Vec<(usize, String)>, Vec<&'a Hook>) {
    let mut edits: Vec<Vec<Edit>> = units.iter().map(|_| Vec::new()).collect();
    let mut applied = Vec::new();
    for h in hooks {
        let Some((u, at, replaces)) = position(units, h) else {
            continue;
        };
        edits[u].push(Edit {
            at,
            replaces,
            code: one_line(&h.code),
        });
        applied.push(h);
    }
    let mut out = Vec::new();
    for (u, mut edits) in edits.into_iter().enumerate() {
        if edits.is_empty() {
            continue;
        }
        let mut text = units[u].1.lexed.source.clone();
        // from the end, so earlier offsets stay valid
        edits.sort_by_key(|e| std::cmp::Reverse(e.at));
        for e in edits {
            let end = e.at + e.replaces;
            // replaced code keeps its line breaks, so rows stay
            let kept: String = text[e.at..end]
                .chars()
                .map(|c| if c == '\n' { '\n' } else { ' ' })
                .collect();
            text.replace_range(e.at..end, &format!(" {} {kept}", e.code));
        }
        out.push((u, text));
    }
    (out, applied)
}

/// The code of a hook on one line, without comments.
fn one_line(code: &str) -> String {
    let lexed = abap_syntax::lex(code);
    let tokens: Vec<&Token> = lexed
        .tokens
        .iter()
        .filter(|t| t.kind != TokenKind::Comment)
        .collect();
    abap_syntax::join_tokens(tokens)
}

/// The unit, byte offset and length of replaced code for a hook.
fn position(units: &[(&str, &File)], h: &Hook) -> Option<(usize, usize, usize)> {
    // enhancements of a function module's parameters have two sides
    if h.position.matches("\\SE:").count() > 1 {
        return None;
    }
    let include = h.part("IC");
    let in_unit = |u: usize| include.as_deref().is_none_or(|i| units[u].0 == i);
    if let Some(point) = h.part("EX") {
        for (u, (_, file)) in units.iter().enumerate().filter(|(u, _)| in_unit(*u)) {
            for (i, s) in file.statements.iter().enumerate() {
                // ENHANCEMENT-POINT name SPOTS ...
                let spots = s.tokens.iter().position(|k| file.tokens()[*k].is("SPOTS"));
                let named = spots
                    .and_then(|k| k.checked_sub(1))
                    .and_then(|k| word(file, s, k))
                    .is_some_and(|w| w == point);
                match s.kind {
                    K::EnhancementPoint if named => return Some((u, end(file, i), 0)),
                    K::EnhancementSection if named => {
                        // the implementation replaces the code of the section
                        let close = (i + 1..file.statements.len())
                            .find(|k| file.statements[*k].kind == K::EndEnhancementSection)?;
                        let from = end(file, i);
                        return Some((u, from, start(file, close) - from));
                    }
                    _ => {}
                }
            }
        }
        return None;
    }
    let begin = match h.part("SE")?.as_str() {
        "BEGIN" => true,
        "END" => false,
        _ => return None,
    };
    let block = if let Some(form) = h.part("FO") {
        Some((K::Form, K::EndForm, form, None))
    } else if let Some(function) = h.part("FU") {
        Some((K::FunctionModule, K::EndFunction, function, None))
    } else {
        h.part("ME")
            .map(|method| (K::MethodImplementation, K::EndMethod, method, h.part("TY")))
    };
    let Some((open, close, name, class)) = block else {
        // the begin or end of an include
        let include = include?;
        let u = units.iter().position(|(n, _)| *n == include)?;
        let file = units[u].1;
        return Some(if begin {
            (u, 0, 0)
        } else {
            let last = (0..file.statements.len()).rev().find(|i| end(file, *i) > 0);
            (u, last.map_or(0, |i| end(file, i)), 0)
        });
    };
    for (u, (_, file)) in units.iter().enumerate().filter(|(u, _)| in_unit(*u)) {
        let mut in_class: Option<String> = None;
        for (i, s) in file.statements.iter().enumerate() {
            if s.kind == K::ClassImplementation {
                in_class = word(file, s, 1);
            }
            let found = s.kind == open
                && word(file, s, 1).is_some_and(|w| w == name)
                && class
                    .as_deref()
                    .is_none_or(|c| in_class.as_deref() == Some(c));
            if !found {
                continue;
            }
            if begin {
                return Some((u, end(file, i), 0));
            }
            let k = (i + 1..file.statements.len()).find(|k| file.statements[*k].kind == close)?;
            return Some((u, start(file, k), 0));
        }
    }
    None
}

/// The `n`th word of a statement, upper case.
fn word(file: &File, s: &abap_syntax::Statement, n: usize) -> Option<String> {
    s.tokens.get(n).map(|k| file.tokens()[*k].upper())
}

fn start(file: &File, i: usize) -> usize {
    let s = &file.statements[i];
    s.tokens.first().map_or(0, |k| file.tokens()[*k].span.start)
}

/// The end of the statement `i`, without comments; 0 for comments only.
fn end(file: &File, i: usize) -> usize {
    let s = &file.statements[i];
    s.tokens
        .iter()
        .map(|k| &file.tokens()[*k])
        .filter(|t| t.kind != TokenKind::Comment)
        .map(|t| t.span.end)
        .max()
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hook(position: &str, code: &str) -> Hook {
        Hook {
            enhancement: "ZENH".into(),
            position: position.into(),
            code: code.into(),
            origin: String::new(),
        }
    }

    fn applied(source: &str, unit: &str, hooks: &[Hook]) -> String {
        let file = abap_syntax::parse(source);
        let (texts, _) = apply(&[(unit, &file)], hooks);
        texts.into_iter().next().map(|(_, t)| t).unwrap_or_default()
    }

    #[test]
    fn hooks_go_where_they_run() {
        let src = "REPORT zr.
ENHANCEMENT-POINT ep_top SPOTS zspot STATIC.
FORM save.
  gv = 1.
ENDFORM.
ENHANCEMENT-SECTION es_calc SPOTS zspot.
  gv = 2.
END-ENHANCEMENT-SECTION.";
        let text = applied(
            src,
            "ZR",
            &[
                hook(
                    "\\PR:ZR\\EX:EP_TOP\\EI",
                    "ENHANCEMENT 1 ZENH.\n  DATA gv_x TYPE i. \" new\nENDENHANCEMENT.",
                ),
                hook(
                    "\\PR:ZR\\FO:SAVE\\SE:END\\EI",
                    "ENHANCEMENT 2 ZENH.\n  gv = 3.\nENDENHANCEMENT.",
                ),
                hook(
                    "\\PR:ZR\\FO:SAVE\\SE:BEGIN\\EI",
                    "ENHANCEMENT 3 ZENH.\n  gv = 4.\nENDENHANCEMENT.",
                ),
                hook(
                    "\\PR:ZR\\EX:ES_CALC\\EI",
                    "ENHANCEMENT 4 ZENH.\n  gv = 5.\nENDENHANCEMENT.",
                ),
                hook(
                    "\\PR:ZR\\FO:NONE\\SE:END\\EI",
                    "ENHANCEMENT 5 ZENH.\nENDENHANCEMENT.",
                ),
            ],
        );
        let lines: Vec<&str> = text.lines().map(str::trim_end).collect();
        assert_eq!(
            lines,
            [
                "REPORT zr.",
                "ENHANCEMENT-POINT ep_top SPOTS zspot STATIC. ENHANCEMENT 1 ZENH. DATA gv_x TYPE i. ENDENHANCEMENT.",
                "FORM save. ENHANCEMENT 3 ZENH. gv = 4. ENDENHANCEMENT.",
                "  gv = 1.",
                " ENHANCEMENT 2 ZENH. gv = 3. ENDENHANCEMENT. ENDFORM.",
                "ENHANCEMENT-SECTION es_calc SPOTS zspot. ENHANCEMENT 4 ZENH. gv = 5. ENDENHANCEMENT.",
                "",
                "END-ENHANCEMENT-SECTION.",
            ]
        );
    }

    #[test]
    fn methods_function_modules_and_includes() {
        let src = "CLASS lcl IMPLEMENTATION.
  METHOD run.
  ENDMETHOD.
ENDCLASS.
CLASS zcl_x IMPLEMENTATION.
  METHOD run.
  ENDMETHOD.
ENDCLASS.";
        let text = applied(
            src,
            "ZCL_X=========================CP",
            &[hook(
                "\\TY:ZCL_X\\ME:RUN\\SE:BEGIN\\EI",
                "ENHANCEMENT 1 ZENH.\nx = 1.\nENDENHANCEMENT.",
            )],
        );
        assert_eq!(
            text.lines().nth(5).unwrap(),
            "  METHOD run. ENHANCEMENT 1 ZENH. x = 1. ENDENHANCEMENT. "
        );

        let text = applied(
            "FUNCTION z_fm.\nENDFUNCTION.",
            "Z_FM",
            &[hook(
                "\\FU:Z_FM\\SE:BEGIN\\EI",
                "ENHANCEMENT 1 ZENH.\nx = 1.\nENDENHANCEMENT.",
            )],
        );
        assert_eq!(
            text.lines().next().unwrap(),
            "FUNCTION z_fm. ENHANCEMENT 1 ZENH. x = 1. ENDENHANCEMENT. "
        );
        // parameters of function modules are not code
        let text = applied(
            "FUNCTION z_fm.\nENDFUNCTION.",
            "Z_FM",
            &[hook(
                "\\FU:Z_FM\\SE:IMPORTING\\SE:END\\EI",
                "ENHANCEMENT 1 ZENH.\nENDENHANCEMENT.",
            )],
        );
        assert_eq!(text, "");

        let text = applied(
            "x = 1.\n\" end",
            "ZINC",
            &[hook(
                "\\PR:ZR\\IC:ZINC\\SE:END\\EI",
                "ENHANCEMENT 1 ZENH.\ny = 2.\nENDENHANCEMENT.",
            )],
        );
        assert_eq!(
            text,
            "x = 1. ENHANCEMENT 1 ZENH. y = 2. ENDENHANCEMENT. \n\" end"
        );
    }
}
