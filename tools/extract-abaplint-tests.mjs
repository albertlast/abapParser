// Extracts the ABAP statement test cases of abaplint into JSON fixtures.
//
// abaplint keeps its statement tests as TypeScript arrays of ABAP strings,
// passed to helpers like statementType(tests, "DATA", Statements.Data). This
// script bundles every test file with esbuild, swaps the helpers for capturing
// stubs and the abaplint classes for name-only proxies, runs the bundle and
// writes what was captured to tests/corpus/abaplint/statements/<file>.json.
//
// The CDS parser tests (packages/core/test/cds/cds_parser.ts) give data
// definitions; each is parsed with the CDS parser of the installed
// @abaplint/core, and goes with that verdict into tests/corpus/abaplint/cds.json.
//
// usage: node extract-abaplint-tests.mjs [abaplint-checkout]
// Without an argument the pinned tag is sparse-cloned into tools/.cache.

import {execFileSync} from "node:child_process";
import fs from "node:fs";
import {createRequire} from "node:module";
import path from "node:path";
import {fileURLToPath} from "node:url";
import * as esbuild from "esbuild";

const ABAPLINT_TAG = "v2.120.105";
const here = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(here, "..");
const outDir = path.join(repoRoot, "tests", "corpus", "abaplint");

function checkout() {
  const dir = path.join(here, ".cache", `abaplint-${ABAPLINT_TAG}`);
  if (!fs.existsSync(path.join(dir, "packages"))) {
    fs.mkdirSync(path.dirname(dir), {recursive: true});
    execFileSync(
      "git",
      [
        "clone",
        "--depth",
        "1",
        "--branch",
        ABAPLINT_TAG,
        "--filter=blob:none",
        "--no-checkout",
        "https://github.com/abaplint/abaplint.git",
        dir,
      ],
      {stdio: "inherit"},
    );
    const paths = ["/LICENSE", "/packages/core/test/abap/", "/packages/core/test/cds/"];
    execFileSync("git", ["-C", dir, "sparse-checkout", "set", "--no-cone", ...paths], {
      stdio: "inherit",
      env: {...process.env, MSYS_NO_PATHCONV: "1"},
    });
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
  new Function("module", "exports", "describe", "it", result.outputFiles[0].text)(module, module.exports, describe, it);
  return globalThis.__records;
}

/** The data definitions of abaplint's CDS parser tests, with its verdict. */
async function extractCds(root) {
  const file = path.join(root, "packages", "core", "test", "cds", "cds_parser.ts");
  // every import answers anything; MemoryFile records the definitions
  const stubs = {
    name: "cds-stubs",
    setup(build) {
      build.onResolve({filter: /.*/}, (args) =>
        args.kind === "entry-point" ? undefined : {path: args.path, namespace: "stub"},
      );
      build.onLoad({filter: /.*/, namespace: "stub"}, () => ({
        contents: `
const any = () => new Proxy(function () {}, {
  get: (t, k) => (k === "then" || typeof k === "symbol" ? undefined : any()),
  apply: () => any(),
  construct: () => any(),
});
// in the prototype chain, as esbuild's __toESM copies only own keys
module.exports = Object.create(new Proxy({}, {
  get: (t, k) => {
    if (k === "__esModule") return true;
    if (typeof k === "symbol") return undefined;
    if (k === "MemoryFile") {
      return function (name, raw) {
        globalThis.__records.push({file: name, cds: raw});
        return any();
      };
    }
    return any();
  },
}));`,
        loader: "js",
      }));
    },
  };
  const result = await esbuild.build({
    entryPoints: [file],
    bundle: true,
    write: false,
    format: "cjs",
    platform: "node",
    logLevel: "silent",
    plugins: [stubs],
  });
  globalThis.__records = [];
  const describe = (_n, f) => f.call({timeout() {}});
  describe.skip = () => {};
  describe.only = describe;
  const it = (_n, f) => {
    try {
      f();
    } catch {
      // the stubs answer anything, assertions included
    }
  };
  it.skip = () => {};
  it.only = it;
  const module = {exports: {}};
  new Function("module", "exports", "describe", "it", result.outputFiles[0].text)(module, module.exports, describe, it);

  const require = createRequire(import.meta.url);
  const core = require("@abaplint/core");
  const corePath = path.dirname(require.resolve("@abaplint/core/package.json"));
  const {CDSParser} = require(path.join(corePath, "build/src/cds/cds_parser"));
  const seen = new Set();
  const cases = [];
  for (const r of globalThis.__records) {
    if (typeof r.cds !== "string" || !/\.(asddls|acds)$/i.test(r.file) || seen.has(r.cds)) {
      continue;
    }
    seen.add(r.cds);
    const parsed = new CDSParser().parse(new core.MemoryFile(r.file, r.cds));
    cases.push({cds: r.cds, valid: parsed !== undefined});
  }
  return cases;
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
  for (const f of fs
    .readdirSync(testDir)
    .filter((f) => f.endsWith(".ts"))
    .sort()) {
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
  const cds = await extractCds(root);
  fs.writeFileSync(path.join(outDir, "cds.json"), JSON.stringify(cds, null, 1) + "\n");
  fs.writeFileSync(
    path.join(outDir, "SOURCE.md"),
    `Extracted from https://github.com/abaplint/abaplint tag ${ABAPLINT_TAG},\n` +
      "packages/core/test/abap/statements and packages/core/test/cds (cds.json,\n" +
      "with the verdict of abaplint's CDS parser), by tools/extract-abaplint-tests.mjs.\n" +
      "abaplint is MIT licensed, see LICENSE in this folder.\n",
  );
  console.log(`extracted ${total} statement cases, ${cds.length} CDS cases`);
  if (failed.length > 0) {
    console.error(`failed files:\n  ${failed.join("\n  ")}`);
    process.exitCode = 1;
  }
}

await main();
