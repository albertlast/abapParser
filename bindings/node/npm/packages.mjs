// Builds the npm packages of a release: one per platform with its prebuilt
// native module, and the main package depending on them optionally, so npm
// installs only the one matching the machine.
//
// usage: node npm/packages.mjs <artifacts> <out>
//   <artifacts>  folder with abap_parser.<platform>.node for every platform
//   <out>        receives <platform>/ per platform package and core/

import fs from "node:fs";
import path from "node:path";
import {fileURLToPath} from "node:url";

const here = path.dirname(fileURLToPath(import.meta.url));
const main = path.resolve(here, "..");

/** Platforms with prebuilt modules: npm `os`, `cpu` and for Linux `libc`. */
export const PLATFORMS = [
  {name: "win32-x64-msvc", os: "win32", cpu: "x64"},
  {name: "darwin-x64", os: "darwin", cpu: "x64"},
  {name: "darwin-arm64", os: "darwin", cpu: "arm64"},
  {name: "linux-x64-gnu", os: "linux", cpu: "x64", libc: "glibc"},
  {name: "linux-arm64-gnu", os: "linux", cpu: "arm64", libc: "glibc"},
];

function build(artifacts, out) {
  const pkg = JSON.parse(fs.readFileSync(path.join(main, "package.json"), "utf8"));
  const missing = PLATFORMS.filter((p) => !fs.existsSync(path.join(artifacts, `abap_parser.${p.name}.node`)));
  if (missing.length) {
    throw new Error(`missing native modules: ${missing.map((p) => p.name).join(", ")}`);
  }
  fs.rmSync(out, {recursive: true, force: true});
  for (const p of PLATFORMS) {
    const dir = path.join(out, p.name);
    fs.mkdirSync(dir, {recursive: true});
    fs.copyFileSync(path.join(artifacts, `abap_parser.${p.name}.node`), path.join(dir, "abap_parser.node"));
    const platformPkg = {
      name: `${pkg.name}-${p.name}`,
      version: pkg.version,
      description: `Native module of ${pkg.name} for ${p.name}`,
      license: pkg.license,
      repository: pkg.repository,
      main: "abap_parser.node",
      files: ["abap_parser.node", "LICENSE"],
      os: [p.os],
      cpu: [p.cpu],
      ...(p.libc ? {libc: [p.libc]} : {}),
      engines: pkg.engines,
    };
    fs.writeFileSync(path.join(dir, "package.json"), JSON.stringify(platformPkg, null, 2) + "\n");
    fs.copyFileSync(path.join(main, "LICENSE"), path.join(dir, "LICENSE"));
    fs.writeFileSync(path.join(dir, "README.md"),
      `# ${platformPkg.name}\n\nThe native module of [${pkg.name}](https://www.npmjs.com/package/${pkg.name}) for ${p.name}. Install ${pkg.name}, not this package.\n`);
  }
  // the main package without a native module of its own
  const core = path.join(out, "core");
  fs.mkdirSync(core, {recursive: true});
  for (const f of pkg.files.filter((f) => f !== "abap_parser.node").concat(["package.json"])) {
    fs.copyFileSync(path.join(main, f), path.join(core, f));
  }
  const corePkg = JSON.parse(fs.readFileSync(path.join(core, "package.json"), "utf8"));
  corePkg.files = corePkg.files.filter((f) => f !== "abap_parser.node");
  corePkg.optionalDependencies = Object.fromEntries(PLATFORMS.map((p) => [`${pkg.name}-${p.name}`, pkg.version]));
  delete corePkg.scripts;
  fs.writeFileSync(path.join(core, "package.json"), JSON.stringify(corePkg, null, 2) + "\n");
  console.log(`packages of ${pkg.name} ${pkg.version} in ${out}: core, ${PLATFORMS.map((p) => p.name).join(", ")}`);
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const [artifacts, out] = process.argv.slice(2);
  if (!artifacts || !out) {
    console.error("usage: node npm/packages.mjs <artifacts> <out>");
    process.exit(2);
  }
  build(path.resolve(artifacts), path.resolve(out));
}
