// Generates Rust statement grammars from abaplint's statement matchers.
//
// Used once, for crates/abap-syntax/src/grammar/{statements,expressions}.rs;
// those files are maintained by hand since. Run it again into another folder
// to see what changed in abaplint's grammar after an update:
//
// usage: node tools/gen-grammar.mjs <out-dir>
//
// abaplint (https://github.com/abaplint/abaplint) is MIT licensed,
// Copyright (c) 2015 Lars Hvam. Its matchers are walked as objects: words,
// tokens, regexes and combinators become the DSL of grammar/mod.rs, every
// expression becomes a function. Release (`ver`) and language version
// annotations are kept; the check runs for the newest release and standard
// ABAP.

import {execFileSync} from "node:child_process";
import fs from "node:fs";
import {createRequire} from "node:module";
import path from "node:path";

const require = createRequire(import.meta.url);
const corePath = path.dirname(require.resolve("@abaplint/core/package.json"));
const {version} = require("@abaplint/core/package.json");
const combi = require(path.join(corePath, "build/src/abap/2_statements/combi"));
const {ArtifactsABAP} = require(path.join(corePath, "build/src/abap/artifacts"));

const out = process.argv[2];
if (!out) {
  console.error("usage: node tools/gen-grammar.mjs <out-dir>");
  process.exit(1);
}

const snake = (name) =>
  name
    .replace(/([a-z0-9])([A-Z])/g, "$1_$2")
    .replace(/([A-Z])([A-Z][a-z])/g, "$1_$2")
    .toLowerCase();
const KEYWORDS = new Set(
  (
    "as async await break const continue crate dyn else enum extern false fn for gen if impl in let loop match mod move " +
    "mut pub ref return self static struct super trait true type unsafe use where while abstract become box do final " +
    "macro override priv try typeof unsized virtual yield"
  ).split(" "),
);
const fnName = (name) => {
  const s = snake(name);
  return KEYWORDS.has(s) ? s + "_" : s;
};
const rustStr = (s) => JSON.stringify(s);
const raw = (s) => {
  let hashes = "";
  while (s.includes('"' + hashes)) hashes += "#";
  return `r${hashes}"${s}"${hashes}`;
};

// regexes without a Rust equivalent, rewritten by hand
const REWRITE = {
  "^(\\/\\w+\\/)?(?!\\*)[\\w\\d_\\*\\~%]+$": "^(\\/\\w+\\/)?[\\w\\d_\\~%][\\w\\d_\\*\\~%]*$",
};

/** A regex as a word when it is one, else as a pattern. */
function regex(r) {
  const flags = r.flags.replace("g", "");
  let source = r.source;
  const word = /^\^([A-Za-z0-9_]+)\$$/.exec(source);
  if (word && (flags === "i" || !/[a-z]/i.test(word[1]))) {
    return rustStr(word[1].toUpperCase());
  }
  const literal = /^\^\\?([*])\$$/.exec(source);
  if (literal) {
    return rustStr(literal[1]);
  }
  if (REWRITE[source]) {
    source = REWRITE[source];
  }
  let not = [];
  const look = /^\^\(\?!\(\?:([A-Z|]+)\)\$\)(.*)$/.exec(source);
  if (look) {
    not = look[1].split("|").filter((x, i, a) => a.indexOf(x) === i);
    source = "^" + look[2];
  }
  if (source.includes("(?")) {
    throw new Error("regex without Rust equivalent: " + r);
  }
  if (flags !== "" && flags !== "i") {
    throw new Error("regex flags: " + r);
  }
  const pattern = raw((flags === "i" ? "(?i)" : "") + source);
  return not.length > 0 ? `re_not(${pattern}, &[${not.map(rustStr).join(", ")}])` : `re(${pattern})`;
}

// expressions built with a constructor argument other than the default
const VARIANTS = {
  'DatabaseTable{"noDynamic":true}': "DatabaseTableNoDynamic",
  'SQLPathForEntity{"withPrefix":false}': "SQLPathForEntityNoPrefix",
  'SQLPathSegment{"nws":true}': "SQLPathSegmentNws",
};

/** The name of an expression: its class, or its variant. */
function expressionName(n) {
  const fields = (o) =>
    Object.fromEntries(
      Object.keys(o)
        .filter((k) => k !== "runnable")
        .map((k) => [k, o[k]]),
    );
  const own = fields(n);
  const plain = fields(new n.constructor());
  const differ = Object.fromEntries(Object.entries(own).filter(([k, v]) => plain[k] !== v));
  if (Object.keys(differ).length === 0) {
    return n.constructor.name;
  }
  const key = n.constructor.name + JSON.stringify(differ);
  if (!VARIANTS[key]) {
    throw new Error("unnamed expression variant " + key);
  }
  return VARIANTS[key];
}

/**
 * The on-premise release that shipped an abaplint release: abaplint dates
 * syntax by cloud release (7.77 is on-premise 7.54), "after 8.16" for cloud
 * releases no on-premise release has yet.
 */
function onPremise(release) {
  if (release.op == null) {
    return release.abap == null ? "newest" : "after 8.16";
  }
  if (/^740SP/.test(release.abap)) {
    return "7.40 " + release.abap.slice(3);
  }
  const op = String(release.op);
  return op[0] + "." + op.slice(1);
}

const expressions = new Map();

function emit(n) {
  if (n instanceof combi.Expression) {
    const name = expressionName(n);
    if (!expressions.has(name)) {
      expressions.set(name, null);
      expressions.set(name, emit(n.getRunnable()));
    }
    return `E::${name}`;
  }
  const c = n.constructor.name;
  const list = () => n.list.map(emit).join(", ");
  switch (c) {
    case "Word":
      return rustStr(n.s);
    case "WordSequence":
      return rustStr(n.stri.toUpperCase());
    case "Token":
      return `Tok::${n.name}`;
    case "Regex":
      return regex(n.regexp);
    case "Sequence":
      return `seq!(${list()})`;
    case "Alternative":
      return `alt!(${list()})`;
    case "AlternativePriority":
      return `alt_prio!(${list()})`;
    case "Permutation":
      return `per!(${list()})`;
    case "Optional":
      return `opt(${emit(n.optional)})`;
    case "OptionalPriority":
      return `opt_prio(${emit(n.optional)})`;
    case "Star":
      return `star(${emit(n.sta)})`;
    case "StarPriority":
      return `star_prio(${emit(n.sta)})`;
    case "Plus":
      return `plus(${emit(n.plu)})`;
    case "PlusPriority":
      return `plus_prio(${emit(n.plu)})`;
    case "Vers":
      return `ver(${rustStr(onPremise(n.release))}, ${emit(n.runnable)})`;
    case "LangVers":
      return `only_in(Lang::${n.langVersion}, ${emit(n.runnable)})`;
    case "LangVersNot":
      return `not_in(Lang::${n.langVersion}, ${emit(n.runnable)})`;
    case "FailCombinator":
      return "fail()";
    case "FailStar":
      return "fail_star()";
    case "StopBefore2":
      return `stop_before(${rustStr(n.t1)}, ${rustStr(n.t2)})`;
    case "StopBefore1":
      return `stop_before_any(&[${n.words.map(rustStr).join(", ")}])`;
    default:
      throw new Error("unknown combinator " + c);
  }
}

/** A function body: a rule, converted when it is a word, token or expression. */
const body = (b) => (/^("|E::|Tok::)/.test(b) ? `Rule::from(${b})` : b);

const header = (name) => `//! ${name} of the statement grammars.
//!
//! Generated from the statement matchers of abaplint ${version}
//! (<https://github.com/abaplint/abaplint>, MIT, Copyright (c) 2015 Lars Hvam)
//! by tools/gen-grammar.mjs, maintained by hand since.
`;

const statements = ArtifactsABAP.getStatements()
  .map((s) => [s.constructor.name, emit(s.getMatcher())])
  .sort((a, b) => a[0].localeCompare(b[0]));

let st =
  header("Statements") +
  `
use super::dsl::*;
use super::E;
use crate::StatementKind as K;

/// The grammar of a statement kind.
pub(super) fn statement(kind: K) -> Option<Rule> {
    Some(match kind {
${statements.map(([n]) => `        K::${n} => ${fnName(n)}(),`).join("\n")}
        _ => return None,
    })
}
`;
for (const [n, rule] of statements) {
  st += `\nfn ${fnName(n)}() -> Rule {\n    ${body(rule)}\n}\n`;
}

const names = [...expressions.keys()].sort((a, b) => a.localeCompare(b));
let ex =
  header("Expressions") +
  `
use super::dsl::*;

/// The expressions statement grammars are built of, named like abaplint's.
#[allow(clippy::enum_variant_names)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum E {
${names.map((n) => `    ${n},`).join("\n")}
}

impl E {
    pub(super) const ALL: [E; ${names.length}] = [${names.map((n) => `E::${n}`).join(", ")}];
}

/// The grammar of an expression.
pub(super) fn expression(e: E) -> Rule {
    match e {
${names.map((n) => `        E::${n} => ${fnName(n)}(),`).join("\n")}
    }
}
`;
for (const n of names) {
  ex += `\nfn ${fnName(n)}() -> Rule {\n    ${body(expressions.get(n))}\n}\n`;
}

fs.mkdirSync(out, {recursive: true});
const files = [path.join(out, "statements.rs"), path.join(out, "expressions.rs")];
fs.writeFileSync(files[0], st);
fs.writeFileSync(files[1], ex);
execFileSync("rustfmt", ["--edition", "2021", ...files], {stdio: "inherit"});
console.log(`gen-grammar: ${statements.length} statements, ${names.length} expressions`);
