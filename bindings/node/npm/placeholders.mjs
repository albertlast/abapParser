// Placeholder packages for setting up npm trusted publishing: npm configures
// a trusted publisher only for a package that exists, so before the first
// release every package name is published once by hand as version 0.0.0.
// The release workflow then publishes the real versions.
//
// usage: node npm/placeholders.mjs <out>
//   then in each folder of <out>: npm publish --access public

import fs from "node:fs";
import path from "node:path";
import {fileURLToPath} from "node:url";

import {PLATFORMS} from "./packages.mjs";

const here = path.dirname(fileURLToPath(import.meta.url));
const main = JSON.parse(fs.readFileSync(path.join(here, "..", "package.json"), "utf8"));

const out = process.argv[2];
if (!out) {
  console.error("usage: node npm/placeholders.mjs <out>");
  process.exit(2);
}
const names = [main.name, ...PLATFORMS.map((p) => `${main.name}-${p.name}`)];
for (const name of names) {
  const dir = path.resolve(out, name);
  fs.mkdirSync(dir, {recursive: true});
  fs.writeFileSync(path.join(dir, "package.json"), JSON.stringify({
    name,
    version: "0.0.0",
    description: `Placeholder for setting up publishing; use ${main.name} 0.1.0 or later`,
    license: main.license,
    repository: main.repository,
  }, null, 2) + "\n");
  fs.writeFileSync(path.join(dir, "README.md"),
    `# ${name}\n\nPlaceholder for setting up publishing. Use [${main.name}](https://www.npmjs.com/package/${main.name}) 0.1.0 or later.\n`);
}
console.log(`placeholders in ${path.resolve(out)}:`);
for (const name of names) {
  console.log(`  ${name}`);
}
