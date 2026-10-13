// The version of the release: Cargo workspace, npm packages, Python package.
// The Cargo workspace version is the source; package.json follows it, the
// Python package reads it through maturin.
//
// usage:
//   node tools/version.mjs              print the version
//   node tools/version.mjs 0.2.0        set it everywhere
//   node tools/version.mjs --check v0.2.0
//                                       fail unless everything has this
//                                       version, e.g. the pushed tag
//   node tools/version.mjs --notes v0.2.0
//                                       the section of CHANGELOG.md for it

import fs from "node:fs";
import path from "node:path";
import {fileURLToPath} from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const cargoToml = path.join(root, "Cargo.toml");
const cargoLock = path.join(root, "Cargo.lock");
const packageJson = path.join(root, "bindings", "node", "package.json");
const pyproject = path.join(root, "bindings", "python", "pyproject.toml");

const WORKSPACE_VERSION = /(\[workspace\.package\][^[]*?\nversion = ")([^"]+)(")/;
// the crates of the workspace in Cargo.lock, they have no source
const MEMBERS = ["abap-lexer", "abap-syntax", "abap-model", "abap-sql", "abap-source",
  "abap-analysis", "abap-cli", "abap-node", "abap-python"];

function cargoVersion() {
  const m = fs.readFileSync(cargoToml, "utf8").match(WORKSPACE_VERSION);
  if (!m) {
    throw new Error("no version in [workspace.package] of Cargo.toml");
  }
  return m[2];
}

function versions() {
  const pkg = JSON.parse(fs.readFileSync(packageJson, "utf8"));
  const out = {"Cargo.toml": cargoVersion(), "bindings/node/package.json": pkg.version};
  for (const [name, v] of Object.entries(pkg.optionalDependencies ?? {})) {
    out[`bindings/node/package.json ${name}`] = v;
  }
  const py = fs.readFileSync(pyproject, "utf8");
  if (!/dynamic = \[[^\]]*"version"/.test(py)) {
    out["bindings/python/pyproject.toml"] = "(not dynamic)";
  }
  return out;
}

function set(version) {
  if (!/^\d+\.\d+\.\d+(-[0-9A-Za-z.-]+)?$/.test(version)) {
    throw new Error(`not a version: ${version}`);
  }
  fs.writeFileSync(cargoToml, fs.readFileSync(cargoToml, "utf8").replace(WORKSPACE_VERSION, `$1${version}$3`));
  let lock = fs.readFileSync(cargoLock, "utf8");
  for (const name of MEMBERS) {
    // a checkout on Windows may have CRLF line ends
    lock = lock.replace(new RegExp(`(name = "${name}"\\r?\\nversion = ")[^"]+(")`), `$1${version}$2`);
  }
  fs.writeFileSync(cargoLock, lock);
  const pkg = JSON.parse(fs.readFileSync(packageJson, "utf8"));
  pkg.version = version;
  for (const name of Object.keys(pkg.optionalDependencies ?? {})) {
    pkg.optionalDependencies[name] = version;
  }
  fs.writeFileSync(packageJson, JSON.stringify(pkg, null, 2) + "\n");
}

/** CHANGELOG.md with LF line ends, also from a checkout with CRLF */
function changelog() {
  return fs.readFileSync(path.join(root, "CHANGELOG.md"), "utf8").replace(/\r\n/g, "\n");
}

const args = process.argv.slice(2);
if (args[0] === "--notes") {
  const want = (args[1] ?? cargoVersion()).replace(/^v/, "");
  const log = changelog();
  const sections = log.split(/^## /m).slice(1);
  const section = sections.find((s) => s.split("\n")[0].trim() === want);
  if (!section) {
    console.error(`no section "## ${want}" in CHANGELOG.md`);
    process.exit(1);
  }
  console.log(section.split("\n").slice(1).join("\n").trim());
} else if (args[0] === "--check") {
  const want = (args[1] ?? "").replace(/^v/, "");
  const wrong = Object.entries(versions()).filter(([, v]) => v !== want);
  if (wrong.length) {
    for (const [where, v] of wrong) {
      console.error(`${where}: ${v}, expected ${want}`);
    }
    process.exit(1);
  }
  // a release needs its notes
  const log = changelog();
  if (!log.includes(`\n## ${want}\n`)) {
    console.error(`CHANGELOG.md has no section "## ${want}"`);
    process.exit(1);
  }
  console.log(`version ${want} everywhere`);
} else if (args[0]) {
  set(args[0]);
  console.log(`version set to ${args[0]}`);
} else {
  console.log(cargoVersion());
}
