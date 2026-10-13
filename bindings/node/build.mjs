// Builds the native module with cargo and copies it next to index.js.
// usage: node build.mjs [--debug]

import {execFileSync} from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import {fileURLToPath} from "node:url";

const here = path.dirname(fileURLToPath(import.meta.url));
const debug = process.argv.includes("--debug");
const args = ["build", "-p", "abap-node"];
if (!debug) {
  args.push("--release");
}
execFileSync("cargo", args, {stdio: "inherit", cwd: here});

const lib = {
  win32: "abap_node.dll",
  darwin: "libabap_node.dylib",
}[process.platform] ?? "libabap_node.so";
const source = path.join(here, "..", "..", "target", debug ? "debug" : "release", lib);
fs.copyFileSync(source, path.join(here, "abap_parser.node"));
console.log(`built abap_parser.node from ${lib}`);
