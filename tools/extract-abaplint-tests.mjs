// Extracts the ABAP statement test cases of abaplint into JSON fixtures.
//
// abaplint keeps its statement tests as TypeScript arrays of ABAP strings,
// passed to helpers like statementType(tests, "DATA", Statements.Data). This
// script bundles every test file with esbuild, swaps the helpers for capturing
// stubs and the abaplint classes for name-only proxies, runs the bundle and
// writes what was captured to tests/corpus/abaplint/statements/<file>.json.
//
// usage: node extract-abaplint-tests.mjs [abaplint-checkout]
// Without an argument the pinned tag is sparse-cloned into tools/.cache.

import {execFileSync} from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import {fileURLToPath} from "node:url";
import * as esbuild from "esbuild";

const ABAPLINT_TAG = "v2.120.60";
const here = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(here, "..");
const outDir = path.join(repoRoot, "tests", "corpus", "abaplint");

function checkout() {
  const dir = path.join(here, ".cache", `abaplint-${ABAPLINT_TAG}`);
  if (!fs.existsSync(path.join(dir, "packages"))) {
    fs.mkdirSync(path.dirname(dir), {recursive: true});
    execFileSync("git", ["clone", "--depth", "1", "--branch", ABAPLINT_TAG, "--filter=blob:none",
      "--no-checkout", "https://github.com/abaplint/abaplint.git", dir], {stdio: "inherit"});
    execFileSync("git", ["-C", dir, "sparse-checkout", "set", "--no-cone",
      "/LICENSE", "/packages/core/test/abap/"], {stdio: "inherit", env: {...process.env, MSYS_NO_PATHCONV: "1"}});
    execFileSync("git", ["-C", dir, "checkout"], {stdio: "inherit"});
  }
  return dir;
}

const captureModule = `
const rec = (r) => globalThis.__records.push(r);
const name = (x) => x === undefined ? undefined : (typeof x === "string" ? x : x.__name);
const each = (tests, f) => tests.forEach((t) => f(typeof t === "string" ? {abap: t} : t));
module.exports = {
  statementType: (tests, description, type) => each(tests, (t) =>
    rec({abap: t.abap, expect: "match", type: name(type)})),
  statementExpectFail: (tests) => each(tests, (t) =>
    rec({abap: t.abap, expect: "unknown"})),
  statementVersion: (tests, description, type) => each(tests, (t) =>
    rec({abap: t.abap, expect: "match", type: name(type), minRelease: name(t.rel), lang: name(t.langVer)})),
  statementVersionOk: (tests, description, type) => each(tests, (t) =>
    rec({abap: t.abap, expect: "match", type: name(type), release: name(t.rel), lang: name(t.langVer)})),
  statementVersionFail: (tests) => each(tests, (t) =>
    rec({abap: t.abap, expect: "unknown", release: name(t.rel), lang: name(t.langVer)})),
};
`;

// every other import resolves to a proxy where each property is a named stub
const proxyModule = `
function named(n) {
  const f = function () {};
  return new Proxy(f, {
    get: (t, k) => k === "__name" ? n : (typeof k === "symbol" ? undefined : named(String(k))),
    construct: () => named(n),
    apply: () => named(n),
  });
}
// esbuild's __toESM copies own keys into an object inheriting from the
// module's prototype, so the proxy goes into the prototype chain
module.exports = Object.create(new Proxy({}, {
  get: (t, k) => k === "__esModule" ? true : (typeof k === "symbol" ? undefined : named(String(k))),
}));
`;

const redirect = {
  name: "redirect",
  setup(build) {
    build.onResolve({filter: /.*/}, (args) => {
      if (args.kind === "entry-point") {
        return undefined;
      }
      if (/(^|\/)_utils$/.test(args.path)) {
        return {path: "capture", namespace: "stub"};
      }
      return {path: "proxy", namespace: "stub"};
    });
    build.onLoad({filter: /.*/, namespace: "stub"}, (args) => ({
      contents: args.path === "capture" ? captureModule : proxyModule,
      loader: "js",
    }));
  },
};

async function extractFile(file) {
  const result = await esbuild.build({
    entryPoints: [file],
    bundle: true,
    write: false,
    format: "cjs",
    platform: "node",
    logLevel: "silent",
    plugins: [redirect],
  });
  globalThis.__records = [];
  const describe = (_n, f) => f.call({timeout() {}});
  describe.skip = () => {};
  describe.only = describe;
  const it = () => {};
  it.skip = it;
  it.only = it;
  const module = {exports: {}};
  new Function("module", "exports", "describe", "it", result.outputFiles[0].text)(
    module, module.exports, describe, it);
  return globalThis.__records;
}

async function main() {
  const root = process.argv[2] ?? checkout();
  const testDir = path.join(root, "packages", "core", "test", "abap", "statements");
  const target = path.join(outDir, "statements");
  fs.rmSync(target, {recursive: true, force: true});
  fs.mkdirSync(target, {recursive: true});
  fs.copyFileSync(path.join(root, "LICENSE"), path.join(outDir, "LICENSE"));

  let total = 0;
  const failed = [];
  for (const f of fs.readdirSync(testDir).filter((f) => f.endsWith(".ts")).sort()) {
    try {
      const records = await extractFile(path.join(testDir, f));
      if (records.length === 0) {
        continue;
      }
      total += records.length;
      fs.writeFileSync(path.join(target, f.replace(/\.ts$/, ".json")), JSON.stringify(records, null, 1) + "\n");
    } catch (e) {
      failed.push(`${f}: ${e.message}`);
    }
  }
  fs.writeFileSync(path.join(outDir, "SOURCE.md"),
    `Extracted from https://github.com/abaplint/abaplint tag ${ABAPLINT_TAG},\n` +
    "packages/core/test/abap/statements, by tools/extract-abaplint-tests.mjs.\n" +
    "abaplint is MIT licensed, see LICENSE in this folder.\n");
  console.log(`extracted ${total} statement cases`);
  if (failed.length > 0) {
    console.error(`failed files:\n  ${failed.join("\n  ")}`);
    process.exitCode = 1;
  }
}

await main();
