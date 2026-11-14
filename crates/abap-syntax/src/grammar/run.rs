//! The grammars compiled into one vector of nodes, and matching the tokens
//! of a statement against them.
//!
//! Each node knows the tokens it can start with: keywords, token classes and
//! patterns. A node is not run where the next token cannot begin it, which
//! changes no result, also of the priority rules: a start where a rule
//! cannot begin adds nothing to any of its alternatives.

use std::collections::HashMap;
use std::hash::{BuildHasherDefault, Hasher};
use std::sync::OnceLock;

use super::expressions::{self, E};
use super::{statements, Pattern, Rule, Tok};
use crate::{StatementKind as K, Token};

type Id = u32;

/// Not a keyword of the grammars.
const NONE: Id = Id::MAX;

/// What a node does. Lists of children are ranges into [`Grammar::kids`].
enum Op {
    Word(Id),
    Tok(Tok),
    Regex(Id),
    Seq(Id, Id),
    Alt(Id, Id),
    AltPrio(Id, Id),
    Per(Id, Id),
    Opt(Id),
    OptPrio(Id),
    Star(Id),
    StarPrio(Id),
    Plus(Id),
    PlusPrio(Id),
    /// the index of an expression
    Expr(Id),
    Never,
    Fail,
    FailStar,
    /// two keywords
    StopBefore(Id, Id),
    /// a range of keywords in [`Grammar::kids`]
    StopBeforeAny(Id, Id),
}

/// The tokens a node can start with.
#[derive(Clone, Default, PartialEq)]
struct First {
    /// it can match no token
    empty: bool,
    /// it is always run: [`Op::Fail`] and [`Op::FailStar`] act when run
    always: bool,
    /// keywords, sorted
    words: Vec<Id>,
    /// token classes, one bit per [`Tok::ALL`]
    toks: u64,
    /// patterns, one bit per [`Grammar::patterns`]
    patterns: u128,
}

struct Node {
    op: Op,
    first: First,
}

/// All grammars, compiled.
pub(super) struct Grammar {
    nodes: Vec<Node>,
    kids: Vec<Id>,
    /// distinct patterns
    patterns: Vec<Pattern>,
    /// all patterns, to test a token against all at once
    set: regex::bytes::RegexSet,
    /// upper case keyword to its id
    keywords: HashMap<String, Id>,
    /// the node of each expression
    expressions: Vec<Id>,
    statements: HashMap<K, Id>,
    /// statements by the first keyword they can start with, like abaplint
    /// picks candidates; `""` for those that can start with any token
    by_first: HashMap<String, Vec<K>>,
}

/// The grammars, built on first use.
pub(super) fn grammar() -> &'static Grammar {
    static GRAMMAR: OnceLock<Grammar> = OnceLock::new();
    GRAMMAR.get_or_init(Grammar::build)
}

impl Grammar {
    fn build() -> Grammar {
        let exprs: Vec<Rule> = E::ALL.iter().map(|e| expressions::expression(*e)).collect();
        let stats: Vec<(K, Rule)> = K::ALL
            .iter()
            .filter_map(|k| statements::statement(*k).map(|r| (*k, r)))
            .collect();
        let mut by_first: HashMap<String, Vec<K>> = HashMap::new();
        for (k, rule) in &stats {
            for word in dispatch_first(rule, &exprs) {
                by_first
                    .entry(word.to_ascii_uppercase())
                    .or_default()
                    .push(*k);
            }
        }
        let mut b = Builder {
            g: Grammar {
                nodes: Vec::new(),
                kids: Vec::new(),
                patterns: Vec::new(),
                set: regex::bytes::RegexSet::empty(),
                keywords: HashMap::new(),
                expressions: Vec::new(),
                statements: HashMap::new(),
                by_first,
            },
            pattern_ids: HashMap::new(),
        };
        b.g.expressions = exprs.into_iter().map(|r| b.compile(r)).collect();
        b.g.statements = stats.into_iter().map(|(k, r)| (k, b.compile(r))).collect();
        let mut g = b.g;
        assert!(g.patterns.len() <= 128, "more patterns than bits");
        g.set = regex::bytes::RegexSetBuilder::new(g.patterns.iter().map(|p| p.re))
            .unicode(false)
            .build()
            .expect("grammar patterns");
        assert!(Tok::ALL.len() <= 64, "more token classes than bits");
        g.firsts();
        g
    }

    fn kids(&self, a: Id, b: Id) -> &[Id] {
        &self.kids[a as usize..b as usize]
    }

    /// What each node can start with; repeated until nothing changes, as
    /// expressions refer to each other.
    fn firsts(&mut self) {
        loop {
            let mut changed = false;
            for i in 0..self.nodes.len() {
                let f = self.first_of(i);
                if f != self.nodes[i].first {
                    self.nodes[i].first = f;
                    changed = true;
                }
            }
            if !changed {
                break;
            }
        }
    }

    fn first_of(&self, i: usize) -> First {
        let of = |id: Id| &self.nodes[id as usize].first;
        let add = |f: &mut First, c: &First| {
            f.always |= c.always;
            f.words.extend(&c.words);
            f.toks |= c.toks;
            f.patterns |= c.patterns;
        };
        let mut f = First::default();
        match self.nodes[i].op {
            Op::Word(w) => f.words.push(w),
            Op::Tok(t) => f.toks = 1 << t as u32,
            Op::Regex(r) => f.patterns = 1 << r,
            Op::Seq(a, b) => {
                f.empty = true;
                for id in self.kids(a, b) {
                    let c = of(*id);
                    add(&mut f, c);
                    if !c.empty {
                        f.empty = false;
                        break;
                    }
                }
            }
            Op::Alt(a, b) | Op::AltPrio(a, b) | Op::Per(a, b) => {
                for id in self.kids(a, b) {
                    let c = of(*id);
                    add(&mut f, c);
                    f.empty |= c.empty;
                }
            }
            Op::Opt(x) | Op::OptPrio(x) | Op::Star(x) | Op::StarPrio(x) => {
                f = of(x).clone();
                f.empty = true;
            }
            Op::Plus(x) | Op::PlusPrio(x) => f = of(x).clone(),
            Op::Expr(e) => f = of(self.expressions[e as usize]).clone(),
            Op::Never => {}
            Op::Fail | Op::FailStar => {
                f.empty = true;
                f.always = true;
            }
            Op::StopBefore(..) | Op::StopBeforeAny(..) => f.empty = true,
        }
        f.words.sort_unstable();
        f.words.dedup();
        f
    }

    /// Whether the statement kind has a grammar.
    pub(super) fn has(&self, kind: K) -> bool {
        self.statements.contains_key(&kind)
    }

    /// Where the grammar of the statement's kind got stuck, `None` when it or
    /// that of another statement starting with its first word matches all
    /// tokens.
    pub(super) fn stuck(&self, kind: K, body: &[&Token]) -> Option<usize> {
        let mut run = Run::new(self, body);
        if run.statement(self.statements[&kind]) {
            return None;
        }
        let far = run.far;
        let first = body[0].upper();
        let others = [first.as_str(), ""]
            .into_iter()
            .filter_map(|w| self.by_first.get(w))
            .flatten()
            .filter(|k| **k != kind);
        for k in others {
            if run.statement(self.statements[k]) {
                return None;
            }
        }
        Some(far as usize)
    }
}

/// Compiles rules into the nodes of a grammar.
struct Builder {
    g: Grammar,
    /// patterns by their regex and excluded words
    pattern_ids: HashMap<(String, &'static [&'static str]), Id>,
}

impl Builder {
    fn keyword(&mut self, w: &str) -> Id {
        let next = self.g.keywords.len() as Id;
        *self
            .g
            .keywords
            .entry(w.to_ascii_uppercase())
            .or_insert(next)
    }

    fn compile(&mut self, rule: Rule) -> Id {
        let op = match rule {
            Rule::Word(w) => Op::Word(self.keyword(w)),
            Rule::Tok(t) => Op::Tok(t),
            Rule::Regex(p) => {
                let next = self.g.patterns.len() as Id;
                let key = (p.re.to_string(), p.not);
                let id = *self.pattern_ids.entry(key).or_insert(next);
                if id == next {
                    self.g.patterns.push(*p);
                }
                Op::Regex(id)
            }
            Rule::Seq(l) => {
                let (a, b) = self.list(l);
                Op::Seq(a, b)
            }
            Rule::Alt(l) => {
                let (a, b) = self.list(l);
                Op::Alt(a, b)
            }
            Rule::AltPrio(l) => {
                let (a, b) = self.list(l);
                Op::AltPrio(a, b)
            }
            Rule::Per(l) => {
                let (a, b) = self.list(l);
                Op::Per(a, b)
            }
            Rule::Opt(x) => Op::Opt(self.compile(*x)),
            Rule::OptPrio(x) => Op::OptPrio(self.compile(*x)),
            Rule::Star(x) => Op::Star(self.compile(*x)),
            Rule::StarPrio(x) => Op::StarPrio(self.compile(*x)),
            Rule::Plus(x) => Op::Plus(self.compile(*x)),
            Rule::PlusPrio(x) => Op::PlusPrio(self.compile(*x)),
            Rule::Expr(e) => Op::Expr(e as Id),
            Rule::Never(_) => Op::Never,
            Rule::Fail => Op::Fail,
            Rule::FailStar => Op::FailStar,
            Rule::StopBefore(a, b) => Op::StopBefore(self.keyword(a), self.keyword(b)),
            Rule::StopBeforeAny(words) => {
                let ids: Vec<Id> = words.iter().map(|w| self.keyword(w)).collect();
                let a = self.g.kids.len() as Id;
                self.g.kids.extend(ids);
                Op::StopBeforeAny(a, self.g.kids.len() as Id)
            }
        };
        self.g.nodes.push(Node {
            op,
            first: First::default(),
        });
        self.g.nodes.len() as Id - 1
    }

    fn list(&mut self, rules: Vec<Rule>) -> (Id, Id) {
        let ids: Vec<Id> = rules.into_iter().map(|r| self.compile(r)).collect();
        let a = self.g.kids.len() as Id;
        self.g.kids.extend(ids);
        (a, self.g.kids.len() as Id)
    }
}

/// The keywords a rule can start with, `""` for any token: abaplint's
/// `first()`, by which it picks the statements to try.
fn dispatch_first(rule: &Rule, exprs: &[Rule]) -> Vec<&'static str> {
    match rule {
        Rule::Word(w) => vec![w],
        Rule::Seq(list) => dispatch_first(&list[0], exprs),
        Rule::Alt(list) | Rule::AltPrio(list) => {
            if list.len() != 2 {
                return vec![""];
            }
            let mut f1 = dispatch_first(&list[0], exprs);
            let f2 = dispatch_first(&list[1], exprs);
            if f1 == [""] {
                return f1;
            }
            if f2 == [""] {
                return f2;
            }
            if f1.len() == 1 && f1 == f2 {
                return f1;
            }
            f1.extend(f2);
            f1
        }
        Rule::Plus(x) | Rule::PlusPrio(x) | Rule::Never(x) => dispatch_first(x, exprs),
        Rule::Expr(e) => dispatch_first(&exprs[*e as usize], exprs),
        Rule::Fail | Rule::FailStar => vec![],
        _ => vec![""],
    }
}

/// Why a run stopped early.
#[derive(Debug, Clone, Copy)]
enum Abort {
    /// [`Rule::Fail`]: the statement does not match
    Fail,
    /// [`Rule::FailStar`]
    FailStar,
}

type Ends = Result<Set, Abort>;

/// Positions; up to four without allocating, which is nearly always.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Set {
    Small(u8, [u32; 4]),
    Big(Vec<u32>),
}

impl Default for Set {
    fn default() -> Self {
        Set::Small(0, [0; 4])
    }
}

impl Set {
    fn one(p: u32) -> Set {
        Set::Small(1, [p, 0, 0, 0])
    }

    fn as_slice(&self) -> &[u32] {
        match self {
            Set::Small(n, a) => &a[..*n as usize],
            Set::Big(v) => v,
        }
    }

    fn is_empty(&self) -> bool {
        self.as_slice().is_empty()
    }

    fn push(&mut self, p: u32) {
        match self {
            Set::Small(n, a) if (*n as usize) < a.len() => {
                a[*n as usize] = p;
                *n += 1;
            }
            Set::Small(_, a) => {
                let mut v = a.to_vec();
                v.push(p);
                *self = Set::Big(v);
            }
            Set::Big(v) => v.push(p),
        }
    }

    fn extend(&mut self, ps: &[u32]) {
        for p in ps {
            self.push(*p);
        }
    }

    /// Sorted, without duplicates.
    fn done(mut self) -> Set {
        match &mut self {
            Set::Small(n, a) => {
                let s = &mut a[..*n as usize];
                s.sort_unstable();
                let mut k = 0;
                for i in 0..s.len() {
                    if k == 0 || s[i] != s[k - 1] {
                        s[k] = s[i];
                        k += 1;
                    }
                }
                *n = k as u8;
            }
            Set::Big(v) => {
                v.sort_unstable();
                v.dedup();
            }
        }
        self
    }
}

/// Hashes the memo keys, numbers, by one multiplication.
#[derive(Default)]
struct KeyHasher(u64);

impl Hasher for KeyHasher {
    fn finish(&self) -> u64 {
        self.0
    }
    fn write(&mut self, _: &[u8]) {
        unreachable!("only u64 keys")
    }
    fn write_u64(&mut self, n: u64) {
        self.0 = n.wrapping_mul(0x9E37_79B9_7F4A_7C15);
    }
}

/// What is known about a token of the statement.
struct Facts {
    /// its keyword id
    kw: Id,
    /// the token classes it is of
    toks: u64,
    /// the patterns that match it, once tested
    matches: Option<u128>,
}

/// Matching one statement: positions are indices into its tokens. A rule
/// takes the positions where it may start and returns every position where
/// it may end, like abaplint's combinators.
struct Run<'a> {
    g: &'a Grammar,
    t: &'a [&'a Token],
    facts: Vec<Facts>,
    /// the farthest position a token was matched up to
    far: u32,
    /// ends of an expression from a start
    memo: HashMap<u64, Ends, BuildHasherDefault<KeyHasher>>,
}

impl<'a> Run<'a> {
    fn new(g: &'a Grammar, t: &'a [&'a Token]) -> Self {
        let facts = t
            .iter()
            .map(|token| {
                let upper = token.text.to_ascii_uppercase();
                Facts {
                    kw: g.keywords.get(&upper).copied().unwrap_or(NONE),
                    toks: 1 << Tok::of(token) as u32,
                    matches: None,
                }
            })
            .collect();
        Run {
            g,
            t,
            facts,
            far: 0,
            memo: HashMap::default(),
        }
    }

    fn len(&self) -> u32 {
        self.t.len() as u32
    }

    /// Whether the node matches all tokens; the memo is reset, the farthest
    /// position not.
    fn statement(&mut self, id: Id) -> bool {
        self.memo.clear();
        let n = self.len();
        matches!(self.run(id, &[0]), Ok(ends) if ends.as_slice().contains(&n))
    }

    fn kw(&self, p: u32) -> Id {
        self.facts.get(p as usize).map_or(NONE, |f| f.kw)
    }

    /// Which of the patterns match the token at `p`; all are tested at
    /// once, the first time.
    fn patterns(&mut self, p: u32, patterns: u128) -> u128 {
        let f = &mut self.facts[p as usize];
        let matches = *f.matches.get_or_insert_with(|| {
            let text = &self.t[p as usize].text;
            self.g
                .set
                .matches(text.as_bytes())
                .iter()
                .filter(|r| {
                    !self.g.patterns[*r]
                        .not
                        .iter()
                        .any(|w| text.eq_ignore_ascii_case(w))
                })
                .fold(0, |bits, r| bits | 1 << r)
        });
        matches & patterns
    }

    /// Whether the token at `p` can begin a node with this start.
    fn can(&mut self, first: &First, p: u32) -> bool {
        if p >= self.len() {
            return false;
        }
        let f = &self.facts[p as usize];
        f.toks & first.toks != 0
            || (f.kw != NONE && first.words.binary_search(&f.kw).is_ok())
            || (first.patterns != 0 && self.patterns(p, first.patterns) != 0)
    }

    /// Runs the node from the starts where it can begin.
    fn run(&mut self, id: Id, starts: &[u32]) -> Ends {
        let g = self.g;
        let first = &g.nodes[id as usize].first;
        if first.empty || first.always {
            return self.exec(id, starts);
        }
        match starts {
            [] => Ok(Set::default()),
            [p] => {
                if self.can(first, *p) {
                    self.exec(id, starts)
                } else {
                    Ok(Set::default())
                }
            }
            _ => {
                let mut kept = Set::default();
                for p in starts {
                    if self.can(first, *p) {
                        kept.push(*p);
                    }
                }
                self.exec(id, kept.as_slice())
            }
        }
    }

    /// The single tokens at the starts that satisfy `ok`.
    fn token(&mut self, starts: &[u32], ok: impl Fn(&Self, usize) -> bool) -> Ends {
        let mut ends = Set::default();
        for &p in starts {
            if p < self.len() && ok(self, p as usize) {
                ends.push(p + 1);
                self.far = self.far.max(p + 1);
            }
        }
        Ok(ends)
    }

    fn exec(&mut self, id: Id, starts: &[u32]) -> Ends {
        if starts.is_empty() {
            return Ok(Set::default());
        }
        let g = self.g;
        match g.nodes[id as usize].op {
            Op::Word(w) => self.token(starts, |r, p| r.facts[p].kw == w),
            Op::Tok(k) => self.token(starts, |r, p| r.facts[p].toks & 1 << k as u32 != 0),
            Op::Regex(x) => {
                for &p in starts {
                    if p < self.len() {
                        self.patterns(p, 1 << x);
                    }
                }
                self.token(starts, |r, p| r.facts[p].matches.unwrap_or(0) & 1 << x != 0)
            }
            // each start on its own, like abaplint: priority rules inside
            // decide per start
            Op::Seq(a, b) => {
                let list = g.kids(a, b);
                let mut out = Set::default();
                for &p in starts {
                    let mut temp = Set::one(p);
                    for r in list {
                        temp = self.run(*r, temp.as_slice())?;
                        if temp.is_empty() {
                            break;
                        }
                    }
                    out.extend(temp.as_slice());
                }
                Ok(out.done())
            }
            Op::Alt(a, b) => {
                let mut out = Set::default();
                for r in g.kids(a, b) {
                    out.extend(self.run(*r, starts)?.as_slice());
                }
                Ok(out.done())
            }
            Op::AltPrio(a, b) => {
                for r in g.kids(a, b) {
                    let ends = self.run(*r, starts)?;
                    if !ends.is_empty() {
                        return Ok(ends);
                    }
                }
                Ok(Set::default())
            }
            Op::Opt(x) => {
                let mut out = Set::default();
                out.extend(starts);
                for &p in starts {
                    out.extend(self.run(x, &[p])?.as_slice());
                }
                Ok(out.done())
            }
            Op::OptPrio(x) => {
                let mut out = Set::default();
                for &p in starts {
                    let ends = self.run(x, &[p])?;
                    match ends.as_slice() {
                        [] => out.push(p),
                        [e] => out.push((*e).max(p)),
                        many => out.extend(many),
                    }
                }
                Ok(out.done())
            }
            Op::Star(x) => self.star(x, starts),
            Op::StarPrio(x) => self.star_prio(x, starts),
            Op::Plus(x) | Op::PlusPrio(x) => {
                let prio = matches!(g.nodes[id as usize].op, Op::PlusPrio(_));
                let mut out = Set::default();
                for &p in starts {
                    let first = self.run(x, &[p])?;
                    if first.is_empty() {
                        continue;
                    }
                    let more = if prio {
                        self.star_prio(x, first.as_slice())?
                    } else {
                        self.star(x, first.as_slice())?
                    };
                    out.extend(more.as_slice());
                }
                Ok(out.done())
            }
            Op::Per(a, b) => self.per(g.kids(a, b), starts),
            Op::Expr(e) => {
                let node = g.expressions[e as usize];
                let mut out = Set::default();
                for &p in starts {
                    let key = (u64::from(e) << 32) | u64::from(p);
                    let ends = match self.memo.get(&key) {
                        Some(ends) => ends.clone(),
                        None => {
                            let ends = self.run(node, &[p]);
                            self.memo.insert(key, ends.clone());
                            ends
                        }
                    };
                    out.extend(ends?.as_slice());
                }
                Ok(out.done())
            }
            Op::Never => Ok(Set::default()),
            Op::Fail => Err(Abort::Fail),
            Op::FailStar => Err(Abort::FailStar),
            Op::StopBefore(a, b) => {
                let mut out = Set::default();
                for &p in starts {
                    if p < self.len() && !(self.kw(p) == a && self.kw(p + 1) == b) {
                        out.push(p);
                    }
                }
                Ok(out)
            }
            Op::StopBeforeAny(a, b) => {
                let words = g.kids(a, b);
                let mut out = Set::default();
                for &p in starts {
                    if p >= self.len() || !words.contains(&self.kw(p)) {
                        out.push(p);
                    }
                }
                Ok(out)
            }
        }
    }

    /// The starts and every end of one or more repetitions; a
    /// [`Rule::FailStar`] inside ends the repetitions.
    fn star(&mut self, x: Id, starts: &[u32]) -> Ends {
        let mut out = Set::default();
        out.extend(starts);
        let mut ends = Set::default();
        ends.extend(starts);
        loop {
            let next = match self.run(x, ends.as_slice()) {
                Ok(next) => next,
                Err(Abort::FailStar) => return Ok(out.done()),
                Err(e) => return Err(e),
            };
            // no progress: the same ends again
            if next.is_empty() || next == ends {
                break;
            }
            out.extend(next.as_slice());
            ends = next;
        }
        Ok(out.done())
    }

    /// The farthest end of the last round of repetitions that matched, or
    /// the starts when none did.
    fn star_prio(&mut self, x: Id, starts: &[u32]) -> Ends {
        let mut ends = Set::default();
        ends.extend(starts);
        let mut last: Option<u32> = None;
        loop {
            let next = self.run(x, ends.as_slice())?;
            if next.is_empty() || next == ends {
                break;
            }
            last = next.as_slice().iter().max().copied();
            ends = next;
        }
        Ok(match last {
            Some(l) => Set::one(l),
            None => {
                let mut s = Set::default();
                s.extend(starts);
                s
            }
        })
    }

    /// Each rule at most once, in any order, at least one.
    fn per(&mut self, list: &[Id], starts: &[u32]) -> Ends {
        let mut out = Set::default();
        for i in 0..list.len() {
            let ends = self.run(list[i], starts)?;
            if ends.is_empty() {
                continue;
            }
            out.extend(ends.as_slice());
            let rest: Vec<Id> = list
                .iter()
                .enumerate()
                .filter(|(k, _)| *k != i)
                .map(|(_, r)| *r)
                .collect();
            let more = match rest.as_slice() {
                [r] => self.run(*r, ends.as_slice())?,
                _ => self.per(&rest, ends.as_slice())?,
            };
            out.extend(more.as_slice());
        }
        Ok(out.done())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_rules_build() {
        let g = grammar();
        assert_eq!(g.expressions.len(), E::ALL.len());
        assert!(g.statements.len() > 300);
        assert!(g.by_first[""].contains(&K::Move));
        // a statement starts with its keyword
        let data = &g.nodes[g.statements[&K::Data] as usize].first;
        assert!(!data.empty && data.words.len() == 1 && data.toks == 0 && data.patterns == 0);
    }
}
