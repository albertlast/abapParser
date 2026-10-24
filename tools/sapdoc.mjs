// Looks up statement syntax in the ABAP Keyword Documentation
// (https://help.sap.com/doc/abapdocu_latest_index_htm/latest/en-US/ABENABAP_REFERENCE.html)
// and prints it as plain text, as a reference while writing statement grammars.
//
// The documentation is copyrighted by SAP SE: pages are cached locally in
// tools/.cache/sapdoc (gitignored) and must not be committed to this repository.
//
// usage: node sapdoc.mjs <PAGE> [--all]
//   PAGE   page name, e.g. ABAPMODIFY_ITAB or ABAPMODIFY_ITAB_SHORTREF
//   --all  print descriptions and examples too, not only the syntax blocks

import fs from "node:fs";
import path from "node:path";
import {fileURLToPath} from "node:url";

const BASE = "https://help.sap.com/doc/abapdocu_latest_index_htm/latest/en-US/";
const cacheDir = path.join(path.dirname(fileURLToPath(import.meta.url)), ".cache", "sapdoc");

async function fetchPage(name) {
  const file = path.join(cacheDir, name + ".html");
  if (fs.existsSync(file)) {
    return fs.readFileSync(file, "utf8");
  }
  const res = await fetch(BASE + name + ".html");
  if (!res.ok) {
    throw new Error(`${name}: HTTP ${res.status}`);
  }
  const text = await res.text();
  fs.mkdirSync(cacheDir, {recursive: true});
  fs.writeFileSync(file, text);
  return text;
}

/** the page content lives in a UI5 JSONModel literal: {par1: "<html>", code1: "...", ...} */
function modelEntries(page) {
  const start = page.indexOf("new sap.ui.model.json.JSONModel({");
  if (start < 0) {
    return [];
  }
  const body = page.slice(start);
  const entries = [];
  const re = /(\w+): "((?:[^"\\]|\\.)*)"/g;
  let m;
  while ((m = re.exec(body)) !== null) {
    entries.push({key: m[1], value: m[2].replace(/\\(.)/g, "$1")});
    if (m[1].startsWith("breadcrumb")) {
      break;
    }
  }
  return entries;
}

function toText(html) {
  return html
    .replace(/<br\s*\/?>/g, "\n")
    .replace(/<\/p>/g, "\n")
    .replace(/<li>/g, "- ")
    .replace(/<[^>]+>/g, "")
    .replace(/&lt;/g, "<")
    .replace(/&gt;/g, ">")
    .replace(/&amp;/g, "&")
    .replace(/&quot;/g, '"')
    .replace(/&#x9;/g, "\t")
    .replace(/&nbsp;/g, " ")
    .replace(/\\n/g, "\n")
    .trim();
}

function links(html) {
  return [...html.matchAll(/href='([A-Z0-9_]+)\.html'/g)].map((m) => m[1]);
}

async function main() {
  const [name, flag] = process.argv.slice(2);
  if (name === undefined) {
    console.error("usage: node sapdoc.mjs <PAGE> [--all]");
    process.exit(2);
  }
  const all = flag === "--all";
  const entries = modelEntries(await fetchPage(name.toUpperCase().replace(/\.HTML$/, "")));
  const related = new Set();
  for (const {key, value} of entries) {
    const isSyntax = value.startsWith("<pre>") && !key.startsWith("code");
    if (isSyntax || all) {
      console.log((key.startsWith("code") ? "--- example\n" : "") + toText(value) + "\n");
    }
    links(value).forEach((l) => related.add(l));
  }
  if (related.size > 0) {
    console.log("related: " + [...related].join(" "));
  }
}

await main();
