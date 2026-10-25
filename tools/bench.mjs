// Compares the speed of versions on the same corpus and machine. Each
// version is checked out as a git worktree under target/bench/, gets this
// checkout's crates/abap-analysis/examples/bench.rs, is built and run; the
// versions run one after the other, and a table shows each stage.
//
// usage:
//   node tools/bench.mjs [--runs N] <ref>...
//     ref: a tag, branch or commit, or "." for this checkout as it is
//     e.g. node tools/bench.mjs v0.5.0 .   before a release
//
// The corpus is that of this checkout (git submodule update --init), the
// same input for every version. The last column compares the last version
// with the first.

import {execFileSync} from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import {fileURLToPath} from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const benchSource = path.join(root, "crates", "abap-analysis", "examples", "bench.rs");
const exe = process.platform === "win32" ? ".exe" : "";

function usage() {
  console.error("usage: node tools/bench.mjs [--runs N] <ref>...   (ref: tag, branch, commit or .)");
  process.exit(2);
}

const args = process.argv.slice(2);
let runs = 3;
const refs = [];
for (let i = 0; i < args.length; i++) {
  if (args[i] === "--runs") {
    runs = Number(args[++i]);
    if (!Number.isInteger(runs) || runs < 1) {
      usage();
    }
  } else {
    refs.push(args[i]);
  }
}
if (refs.length === 0) {
  usage();
}

function run(cmd, cmdArgs, cwd, capture = false) {
  return execFileSync(cmd, cmdArgs, {
    cwd,
    encoding: "utf8",
    stdio: capture ? ["ignore", "pipe", "inherit"] : "inherit",
    maxBuffer: 1 << 26,
  });
}

// builds the benchmark in `dir` and runs it on this checkout's corpus
function bench(dir) {
  run("cargo", ["build", "--release", "-p", "abap-analysis", "--example", "bench"], dir);
  const binary = path.join(dir, "target", "release", "examples", `bench${exe}`);
  const out = run(binary, [root, String(runs)], dir, true);
  const stages = new Map();
  for (const line of out.split(/\r?\n/)) {
    const [stage, time] = line.split("\t");
    const ms = /^(\d+) ms$/.exec(time ?? "");
    if (ms) {
      stages.set(stage, Number(ms[1]));
    } else if (stage === "input") {
      console.log(`  ${line.replaceAll("\t", ", ")}`);
    }
  }
  return stages;
}

const results = [];
for (const ref of refs) {
  console.log(`== ${ref}`);
  if (ref === ".") {
    results.push(bench(root));
    continue;
  }
  const dir = path.join(root, "target", "bench", ref.replace(/[^\w.-]/g, "_"));
  if (fs.existsSync(dir)) {
    run("git", ["worktree", "remove", "--force", dir], root);
  }
  run("git", ["worktree", "add", "--detach", dir, ref], root);
  try {
    const target = path.join(dir, "crates", "abap-analysis", "examples", "bench.rs");
    fs.mkdirSync(path.dirname(target), {recursive: true});
    fs.copyFileSync(benchSource, target);
    results.push(bench(dir));
  } finally {
    run("git", ["worktree", "remove", "--force", dir], root);
  }
}

const stages = [...results[0].keys()];
const header = ["stage", ...refs.map((r) => `${r} (ms)`)];
if (refs.length > 1) {
  header.push(`${refs.at(-1)} vs ${refs[0]}`);
}
const rows = stages.map((stage) => {
  const values = results.map((r) => r.get(stage));
  const row = [stage, ...values.map((v) => (v === undefined ? "-" : String(v)))];
  if (refs.length > 1) {
    const [first, last] = [values[0], values.at(-1)];
    const change = first && last !== undefined ? Math.round((100 * (last - first)) / first) : undefined;
    row.push(change === undefined ? "-" : `${change > 0 ? "+" : ""}${change}%`);
  }
  return row;
});
console.log(`\n| ${header.join(" | ")} |\n|${header.map(() => "---").join("|")}|`);
for (const row of rows) {
  console.log(`| ${row.join(" | ")} |`);
}
