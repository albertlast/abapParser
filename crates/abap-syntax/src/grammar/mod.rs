//! Statement grammars: whether the tokens of a statement form a statement
//! of ABAP.
//!
//! The grammars in [`statements`] and [`expressions`] were generated from
//! the statement matchers of abaplint (<https://github.com/abaplint/abaplint>,
//! MIT) by `tools/gen-grammar.mjs`, and are maintained by hand since; that
//! script, run into another folder, shows what changed in abaplint. Rules
//! are written with the building blocks of [`dsl`] and compiled by [`run`],
//! whose engine follows abaplint's combinators: a rule takes the positions
//! where it may start and returns every position where it may end. The
//! priority variants keep only some of them, so they decide like abaplint
//! does. One difference: a permutation (`per!`) takes each of its rules at
//! most once, in any order; abaplint's may repeat one and miss orders.
//!
//! The check is for the newest release and standard ABAP: release and
//! language version annotations of the rules are kept as information,
//! dating is done by [`crate::release`].

mod expressions;
mod run;
mod statements;

use crate::{StatementKind as K, Token, TokenKind};
use expressions::E;

/// A grammar rule.
pub(crate) enum Rule {
    /// a keyword, case-insensitive
    Word(&'static str),
    Tok(Tok),
    Regex(Box<Pattern>),
    Seq(Vec<Rule>),
    /// all alternatives
    Alt(Vec<Rule>),
    /// the first alternative that matches
    AltPrio(Vec<Rule>),
    Opt(Box<Rule>),
    /// with the rule when it matches
    OptPrio(Box<Rule>),
    Star(Box<Rule>),
    /// as often as possible
    StarPrio(Box<Rule>),
    Plus(Box<Rule>),
    PlusPrio(Box<Rule>),
    /// each at most once, in any order, at least one
    Per(Vec<Rule>),
    Expr(E),
    /// a rule of another language version than standard ABAP
    Never(Box<Rule>),
    /// the statement does not match
    Fail,
    /// ends the enclosing [`Rule::Star`]
    FailStar,
    /// not before these two words
    StopBefore(&'static str, &'static str),
    /// not before any of these words
    StopBeforeAny(&'static [&'static str]),
}

/// A regular expression on the text of a token, but not these words.
pub(crate) struct Pattern {
    /// a regular expression of the `regex` crate, for bytes, `w` as ASCII
    re: &'static str,
    not: &'static [&'static str],
}

/// ABAP language versions besides standard ABAP.
#[derive(Debug, Clone, Copy)]
pub(crate) enum Lang {
    Cloud,
    KeyUser,
}

macro_rules! tokens {
    ($($kind:ident: $plain:ident $w:ident $after:ident $both:ident;)* $(; $($single:ident),*)?) => {
        /// A token by kind and, for some kinds, whitespace before (`W...`)
        /// and after (`...W`); abaplint's token classes.
        /// Not every class is used by the rules.
        #[allow(clippy::enum_variant_names, dead_code)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub(crate) enum Tok {
            $($plain, $w, $after, $both,)*
            $($($single,)*)?
        }

        impl Tok {
            const ALL: &'static [Tok] = &[$(Tok::$plain, Tok::$w, Tok::$after, Tok::$both,)* $($(Tok::$single,)*)?];

            /// The class of a token.
            fn of(t: &Token) -> Tok {
                match t.kind {
                    $(
                        TokenKind::$kind => match (t.ws_before, t.ws_after) {
                            (false, false) => Tok::$plain,
                            (true, false) => Tok::$w,
                            (false, true) => Tok::$after,
                            (true, true) => Tok::$both,
                        },
                    )*
                    $($(TokenKind::$single => Tok::$single,)*)?
                }
            }
        }
    };
}

tokens! {
    ParenLeft: ParenLeft WParenLeft ParenLeftW WParenLeftW;
    ParenRight: ParenRight WParenRight ParenRightW WParenRightW;
    BracketLeft: BracketLeft WBracketLeft BracketLeftW WBracketLeftW;
    BracketRight: BracketRight WBracketRight BracketRightW WBracketRightW;
    Dash: Dash WDash DashW WDashW;
    Plus: Plus WPlus PlusW WPlusW;
    At: At WAt AtW WAtW;
    InstanceArrow: InstanceArrow WInstanceArrow InstanceArrowW WInstanceArrowW;
    StaticArrow: StaticArrow WStaticArrow StaticArrowW WStaticArrowW;
    ; Identifier, String, StringTemplate, StringTemplateBegin, StringTemplateMiddle,
    StringTemplateEnd, Comment, Pragma, Punctuation, AssociationName
}

impl From<&'static str> for Rule {
    /// A keyword, or several separated by blanks or dashes like abaplint's
    /// `"ACCORDING TO"` and `"READ-ONLY"`.
    fn from(s: &'static str) -> Rule {
        let several = s[1.min(s.len())..].contains([' ', '-']);
        if !several {
            return Rule::Word(s);
        }
        let mut words = Vec::new();
        for part in s.split(' ') {
            let mut rest = part;
            while let Some(i) = rest.find('-') {
                if i > 0 {
                    words.push(Rule::Word(&rest[..i]));
                }
                words.push(Rule::Word("-"));
                rest = &rest[i + 1..];
            }
            if !rest.is_empty() || part.is_empty() {
                words.push(Rule::Word(rest));
            }
        }
        Rule::Seq(words)
    }
}

impl From<E> for Rule {
    fn from(e: E) -> Rule {
        Rule::Expr(e)
    }
}

impl From<Tok> for Rule {
    fn from(t: Tok) -> Rule {
        Rule::Tok(t)
    }
}

/// The building blocks of the grammars, named like abaplint's.
mod dsl {
    use super::Pattern;
    pub(crate) use super::{Lang, Rule, Tok};

    macro_rules! seq {
        ($($x:expr),+ $(,)?) => { Rule::Seq(vec![$(Rule::from($x)),+]) };
    }
    macro_rules! alt {
        ($($x:expr),+ $(,)?) => { Rule::Alt(vec![$(Rule::from($x)),+]) };
    }
    macro_rules! alt_prio {
        ($($x:expr),+ $(,)?) => { Rule::AltPrio(vec![$(Rule::from($x)),+]) };
    }
    macro_rules! per {
        ($($x:expr),+ $(,)?) => { Rule::Per(vec![$(Rule::from($x)),+]) };
    }
    pub(crate) use {alt, alt_prio, per, seq};

    pub(crate) fn opt(x: impl Into<Rule>) -> Rule {
        Rule::Opt(Box::new(x.into()))
    }
    pub(crate) fn opt_prio(x: impl Into<Rule>) -> Rule {
        Rule::OptPrio(Box::new(x.into()))
    }
    pub(crate) fn star(x: impl Into<Rule>) -> Rule {
        Rule::Star(Box::new(x.into()))
    }
    pub(crate) fn star_prio(x: impl Into<Rule>) -> Rule {
        Rule::StarPrio(Box::new(x.into()))
    }
    pub(crate) fn plus(x: impl Into<Rule>) -> Rule {
        Rule::Plus(Box::new(x.into()))
    }
    pub(crate) fn plus_prio(x: impl Into<Rule>) -> Rule {
        Rule::PlusPrio(Box::new(x.into()))
    }

    /// A token whose text matches the regular expression.
    pub(crate) fn re(pattern: &'static str) -> Rule {
        re_not(pattern, &[])
    }

    /// Like [`re`], but not the words (case-insensitive).
    pub(crate) fn re_not(pattern: &'static str, not: &'static [&'static str]) -> Rule {
        Rule::Regex(Box::new(Pattern { re: pattern, not }))
    }

    /// Available from the release on; information only.
    pub(crate) fn ver(_release: &'static str, x: impl Into<Rule>) -> Rule {
        x.into()
    }

    /// Not available in the language version; standard ABAP has it.
    pub(crate) fn not_in(_lang: Lang, x: impl Into<Rule>) -> Rule {
        x.into()
    }

    /// Only available in the language version, not in standard ABAP.
    pub(crate) fn only_in(_lang: Lang, x: impl Into<Rule>) -> Rule {
        Rule::Never(Box::new(x.into()))
    }

    pub(crate) fn fail() -> Rule {
        Rule::Fail
    }
    pub(crate) fn fail_star() -> Rule {
        Rule::FailStar
    }
    pub(crate) fn stop_before(a: &'static str, b: &'static str) -> Rule {
        Rule::StopBefore(a, b)
    }
    pub(crate) fn stop_before_any(words: &'static [&'static str]) -> Rule {
        Rule::StopBeforeAny(words)
    }
}

/// Statements longer than this are not checked, like abaplint.
const MAX_TOKENS: usize = 20000;

/// Statements longer than this many tokens are matched in a thread of their
/// own, with a stack of [`STACK_PER_TOKEN`] bytes per token: rules recurse
/// about three levels deep per token (`a && b && ...`), more than the stack
/// of the caller may hold.
const LONG: usize = 128;
/// Frames of a debug build are several times larger.
const STACK_PER_TOKEN: usize = if cfg!(debug_assertions) { 32768 } else { 4096 };

/// Whether the statement does not match its kind's grammar nor that of any
/// other statement starting with its first word: the position in
/// [`Statement::tokens`](crate::Statement::tokens) of the token where the
/// grammar of its kind got stuck. `None` also for kinds without a grammar.
pub(crate) fn mismatch(tokens: &[&Token], kind: K) -> Option<usize> {
    let g = run::grammar();
    if !g.has(kind) {
        return None;
    }
    let last = tokens.last()?;
    if last.kind != TokenKind::Punctuation {
        return None;
    }
    // without the period and pragmas, like abaplint
    let at: Vec<usize> = (0..tokens.len() - 1)
        .filter(|&i| tokens[i].kind != TokenKind::Pragma)
        .collect();
    if at.is_empty() || at.len() > MAX_TOKENS {
        return None;
    }
    let body: Vec<&Token> = at.iter().map(|&i| tokens[i]).collect();
    let far = if body.len() <= LONG {
        g.stuck(kind, &body)?
    } else {
        std::thread::scope(|s| {
            std::thread::Builder::new()
                .stack_size((2 << 20) + body.len() * STACK_PER_TOKEN)
                .spawn_scoped(s, || g.stuck(kind, &body))
                .ok()?
                .join()
                .unwrap_or_else(|e| std::panic::resume_unwind(e))
        })?
    };
    Some(at.get(far).copied().unwrap_or(tokens.len() - 1))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check(source: &str) -> Option<usize> {
        let file = crate::parse(source);
        let s = file
            .statements
            .iter()
            .find(|s| s.kind != K::Comment)
            .expect("a statement");
        let tokens: Vec<&Token> = file.statement_tokens(s).collect();
        mismatch(&tokens, s.kind)
    }

    #[test]
    fn statements_match() {
        for ok in [
            "ALIASES foo FOR zif_bar~foo.",
            "ADD 2 TO lv_count.",
            "DATA lt_tab TYPE STANDARD TABLE OF string WITH EMPTY KEY.",
            "lv_a = lv_b + 1.",
            "LOOP AT lt_tab ASSIGNING FIELD-SYMBOL(<ls_row>) WHERE id = 1.",
            "CONSTANTS lc_x TYPE c LENGTH 1 VALUE 'X'.",
            "PARAMETERS p_file TYPE string LOWER CASE OBLIGATORY.",
            "lo_obj->method( EXPORTING iv_a = 1 IMPORTING ev_b = lv_b ).",
        ] {
            assert_eq!(check(ok), None, "{ok}");
        }
    }

    /// Syntax abaplint does not know, from SAP's documentation, its ABAP
    /// cheat sheets and the syntax check of the SAP trial 2025.
    #[test]
    fn sap_syntax_beyond_abaplint() {
        for ok in [
            "READ TABLE itab INTO wa WHERE a > 1 AND b IS INITIAL.",
            "READ TABLE itab INTO wa WHERE (cond).",
            "READ TABLE itab ASSIGNING <fs> CASTING ELSE UNASSIGN INDEX 1.",
            "READ TABLE itab INTO wa INDEX 1 TRANSPORTING ('A') ('B').",
            "wa = itab[ TABLE KEY sk COMPONENTS a = 1 ].",
            "wa = itab[ KEY ('SK') ('A') = 1 ].",
            "MODIFY TABLE itab FROM wa USING KEY ('SK') TRANSPORTING ('B').",
            "DELETE TABLE itab WITH TABLE KEY ('SK') COMPONENTS ('A') = 1.",
            "t = VALUE #( FOR w IN itab STEP -1 WHERE (cond) ( w ) ).",
            "SORT itab BY VALUE abap_sortorder_tab( ( name = 'A' ) ).",
            "wa = CORRESPONDING #( DEEP APPENDING ( wa ) wa2 ).",
            "it = CORRESPONDING #( it FROM lookup USING KEY sk c = a ).",
            "f = FILTER #( itab USING KEY sk EXCEPT IN ftab WHERE a = table_line ).",
            "x = COND #( WHEN a = 1 THEN 1 ELSE THROW cx( MESSAGE e005(zd) p = 1 ) ).",
            "s = |{ a TIMEZONE = 'CET' COUNTRY = 'DE ' }|.",
            "n = NEW st( LET k = 2 IN a = k ).",
            "CALL TRANSFORMATION id SOURCE var = num RESULT JSON DATA(json).",
            "REPLACE ALL OCCURRENCES OF 'b' IN TABLE t FROM 1 OFFSET 1 TO 2 WITH 'x' RESULTS DATA(r).",
            "LOOP AT itab INTO wa GROUP BY wa-a DESCENDING.",
            "LOOP AT itab INTO wa WHERE a > 2 EQUIV b < 10.",
            "CONVERT UTCLONG u INTO DATE d TIME t FRACTIONAL SECONDS f TIME ZONE tz.",
            "READ ENTITIES OPERATIONS op_tab.",
            "MODIFY ENTITY zent CREATE AUTO FILL CID FIELDS ( a ) WITH tab.",
            "SELECT SINGLE ceil( decfloat34`1.3` ) AS c FROM t INTO @DATA(x).",
            "SELECT SINGLE like_regexpr( pcre = '.', value = url ) AS c FROM t INTO @DATA(x).",
            "SELECT SINGLE CAST( 1 AS D34N ) AS c FROM t INTO @DATA(x).",
            "SELECT a FROM t WHERE ( a, b ) IN ( ( 1, 'x' ), ( 2, 'y' ) ) INTO TABLE @DATA(x).",
            r"SELECT a, \_assoc[ (*) ]-b AS c FROM t INTO TABLE @DATA(x).",
        ] {
            assert_eq!(check(ok), None, "{ok}");
        }
        // the syntax check of the SAP trial 2025: ELSE UNASSIGN right after
        // ASSIGNING only
        assert!(check("READ TABLE itab ASSIGNING <fs> INDEX 1 ELSE UNASSIGN.").is_some());
        assert!(check("READ TABLE itab INTO wa ELSE UNASSIGN INDEX 1.").is_some());
    }

    /// Statements abaplint accepts and the syntax check of the SAP trial
    /// 2025 rejects, with names declared, next to the valid forms.
    #[test]
    fn sap_rejects_beyond_abaplint() {
        for ok in [
            // a field named like a word of the statement is an operand
            "PERFORM f USING using CHANGING CHANGING lv.",
            "PERFORM f TABLES lt USING lv CHANGING lv USING lv.",
            "INSERT lv table INTO fg.",
            "SELECT-OPTIONS foo-bar FOR foo-uname.",
            "SHIFT s LEFT DELETING LEADING ' ' IN CHARACTER MODE.",
            "SHIFT s RIGHT BY 2 PLACES CIRCULAR.",
            "WRITE /5(10) lv.",
            "WRITE AT /pos(len) lv.",
            "WRITE / lv.",
            "WRITE c10(5).",
            "ULINE /5(20).",
            "ULINE AT (20).",
            r"x = mesh-node1\_assoc2[ mesh-node1[ 1 ] k = 1 ]\_assoc3[ ]-col.",
        ] {
            assert_eq!(check(ok), None, "{ok}");
        }
        for bad in [
            // A component name was expected after "-".
            "IF foo- = 2.",
            "lv = ls-.",
            "MODIFY t000 FROM TABLE lt-[].",
            // Invalid syntax for a table row selection.
            "lv = lt[ lv tabix = 1 ]-index.",
            // ".", "RESPECTING BLANKS", ... expected after "INTO".
            "CONCATENATE a into INTO b.",
            // "field" expected after "USING" (or "CHANGING").
            "PERFORM f USING changing CHANGING lv.",
            "PERFORM f USING lv lv CHANGING.",
            // Unable to interpret "USING".
            "PERFORM f TABLES using USING lv CHANGING lv.",
            // "field" expected after "FOR", a field named FOR too.
            "SET HANDLER o->h o FOR.",
            "SET HANDLER o->h for FOR o.",
            // "." expected after "RIGHT".
            "SHIFT s RIGHT DELETING LEADING ' '.",
            "SHIFT s DELETING LEADING ' '.",
            "SHIFT s IN CHARACTER MODE LEFT DELETING LEADING ' '.",
            // Unable to interpret "LV": a position that is no literal needs AT.
            "WRITE lv lv.",
            "ULINE c10(5).",
            "ULINE AT(20).",
            // Arithmetic calculation not permitted here.
            "WRITE lv /.",
        ] {
            assert!(check(bad).is_some(), "{bad}");
        }
    }

    #[test]
    fn long_statements_do_not_overflow_the_stack() {
        let parts = vec!["`x`"; 2000].join(" && ");
        assert_eq!(check(&format!("lv = {parts}.")), None);
        // the operand is missing at the period
        assert_eq!(check(&format!("lv = {parts} &&.")), Some(4002));
    }

    #[test]
    fn mutants_do_not() {
        for (bad, at) in [
            // FOR can name a field
            ("ALIASES foo FOR FOR zif_bar~foo.", 4),
            ("ADD 2 TO.", 3),
            ("ALIASES foo zif_bar~foo.", 2),
        ] {
            assert_eq!(check(bad), Some(at), "{bad}");
        }
    }
}
