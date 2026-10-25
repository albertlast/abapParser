// Runs @abaplint/core over ABAP files and writes golden JSON with the tokens
// and statements abaplint found, used by the differential tests in Rust.
//
// usage: node oracle.mjs <out-dir> <input-dir-or-file>...
//        node oracle.mjs <out-dir> --list tests/corpus/corpora.txt
// For every .abap file <out-dir>/<path relative to cwd>.json is written, so the
// golden tree mirrors the repository layout.
//
// usage: node oracle.mjs --sql-cases <statements-dir> <out-file>
// Writes abaplint's SQL details for the extracted database statement cases.
//
// usage: node oracle.mjs --releases <out-dir> --list tests/corpus/corpora.txt
// Parses every file once per on-premise release and writes, for each
// statement that needs more than 7.00, the lowest release from which
// abaplint parses it as it does on the newest: <out-dir>/<path>.json.

import fs from "node:fs";
import path from "node:path";
import abaplint from "@abaplint/core";

const {Registry, MemoryFile, ABAPObject, Version, Config, Expressions, SyntaxLogic, ReferenceType, VirtualPosition} =
  abaplint;

// Minimal definitions of the standard exception classes, so the syntax check
// of the many classes inheriting from them does not stop at the superclass.
const STUBS = {
  "cx_root.clas.abap": `CLASS cx_root DEFINITION PUBLIC ABSTRACT CREATE PUBLIC.
  PUBLIC SECTION.
    INTERFACES if_message.
    ALIASES get_text FOR if_message~get_text.
    ALIASES get_longtext FOR if_message~get_longtext.
    DATA textid TYPE c LENGTH 32 READ-ONLY.
    DATA previous TYPE REF TO cx_root READ-ONLY.
    DATA is_resumable TYPE abap_bool READ-ONLY.
    METHODS constructor IMPORTING textid LIKE textid OPTIONAL previous LIKE previous OPTIONAL.
    METHODS get_source_position EXPORTING program_name TYPE syrepid include_name TYPE syrepid
      source_line TYPE i.
ENDCLASS.
CLASS cx_root IMPLEMENTATION.
  METHOD constructor.
  ENDMETHOD.
  METHOD get_source_position.
  ENDMETHOD.
  METHOD if_message~get_text.
  ENDMETHOD.
  METHOD if_message~get_longtext.
  ENDMETHOD.
ENDCLASS.`,
  "cx_static_check.clas.abap": `CLASS cx_static_check DEFINITION PUBLIC INHERITING FROM cx_root ABSTRACT.
ENDCLASS.
CLASS cx_static_check IMPLEMENTATION.
ENDCLASS.`,
  "cx_dynamic_check.clas.abap": `CLASS cx_dynamic_check DEFINITION PUBLIC INHERITING FROM cx_root ABSTRACT.
ENDCLASS.
CLASS cx_dynamic_check IMPLEMENTATION.
ENDCLASS.`,
  "cx_no_check.clas.abap": `CLASS cx_no_check DEFINITION PUBLIC INHERITING FROM cx_root ABSTRACT.
ENDCLASS.
CLASS cx_no_check IMPLEMENTATION.
ENDCLASS.`,
  "if_message.intf.abap": `INTERFACE if_message PUBLIC.
  METHODS get_text RETURNING VALUE(result) TYPE string.
  METHODS get_longtext IMPORTING preserve_newlines TYPE abap_bool OPTIONAL RETURNING VALUE(result) TYPE string.
ENDINTERFACE.`,
  "if_t100_message.intf.abap": `INTERFACE if_t100_message PUBLIC.
  DATA t100key TYPE scx_t100key.
  CONSTANTS default_textid TYPE scx_t100key VALUE IS INITIAL.
ENDINTERFACE.`,
  "if_t100_dyn_msg.intf.abap": `INTERFACE if_t100_dyn_msg PUBLIC.
  INTERFACES if_t100_message.
  DATA msgty TYPE symsgty.
  DATA msgv1 TYPE symsgv.
  DATA msgv2 TYPE symsgv.
  DATA msgv3 TYPE symsgv.
  DATA msgv4 TYPE symsgv.
ENDINTERFACE.`,
};

/** Data read and write references found by the syntax check, by file name. */
function dataReferences(reg, obj) {
  const result = new SyntaxLogic(reg, obj).run();
  const byFile = new Map();
  const walk = (node) => {
    for (const r of node.getData().references) {
      const read = r.referenceType === ReferenceType.DataReadReference;
      if (!read && r.referenceType !== ReferenceType.DataWriteReference) {
        continue;
      }
      const start = r.position.getStart();
      if (start instanceof VirtualPosition) {
        continue;
      }
      const file = r.position.getFilename();
      if (!byFile.has(file)) {
        byFile.set(file, []);
      }
      byFile.get(file).push([start.getRow(), start.getCol(), read ? "R" : "W"]);
    }
    for (const c of node.getChildren()) {
      walk(c);
    }
  };
  walk(result.spaghetti.getTop());
  return {ok: result.issues.length === 0, byFile};
}

const DB_STATEMENTS = new Set([
  "Select",
  "SelectLoop",
  "InsertDatabase",
  "UpdateDatabase",
  "ModifyDatabase",
  "DeleteDatabase",
  "OpenCursor",
  "With",
  "WithLoop",
  "MergeDatabase",
]);

/** what abaplint knows about the SQL of a database statement */
function sqlInfo(s) {
  const fae = s.findFirstExpression(Expressions.SQLForAllEntries);
  return {
    tables: s.findAllExpressions(Expressions.DatabaseTable).map((e) => e.concatTokens()),
    targets: s.findAllExpressions(Expressions.SQLTarget).map((e) => e.concatTokens()),
    joins: s.findAllExpressions(Expressions.SQLJoin).length,
    forAllEntries: fae?.findFirstExpression(Expressions.SQLSource)?.concatTokens(),
  };
}

function* walk(p) {
  const stat = fs.statSync(p);
  if (stat.isFile()) {
    if (p.endsWith(".abap")) {
      yield p;
    }
    return;
  }
  for (const e of fs.readdirSync(p).sort()) {
    if (e === ".git" || e === "node_modules") {
      continue;
    }
    yield* walk(path.join(p, e));
  }
}

const VISIBILITY = {1: "private", 2: "protected", 3: "public"};

function interfaceInfo(i) {
  return {
    name: i.name,
    row: i.identifier.getStart().getRow(),
    isGlobal: i.isGlobal,
    interfaces: i.interfaces.map((x) => x.name),
    methods: i.methods.map((m) => ({
      name: m.name,
      visibility: VISIBILITY[m.visibility],
      isRedefinition: m.isRedefinition,
      isEventHandler: m.isEventHandler,
      isForTesting: m.isForTesting,
      isAbstract: m.isAbstract,
      isFinal: m.isFinal,
      parameters: m.parameters.map((p) => ({name: p.name, direction: p.direction})),
      exceptions: m.exceptions,
    })),
    aliases: i.aliases.map((a) => ({name: a.name, component: a.component, visibility: VISIBILITY[a.visibility]})),
    constants: i.constants.map((c) => ({name: c.name, visibility: VISIBILITY[c.visibility]})),
    attributes: i.attributes.map((a) => ({
      name: a.name,
      level: a.level,
      readOnly: a.readOnly,
      visibility: VISIBILITY[a.visibility],
    })),
  };
}

function fileInfo(info) {
  return {
    interfaces: info.listInterfaceDefinitions().map(interfaceInfo),
    classes: info.listClassDefinitions().map((c) => ({
      ...interfaceInfo(c),
      superClassName: c.superClassName,
      isAbstract: c.isAbstract,
      isFinal: c.isFinal,
      isForTesting: c.isForTesting,
    })),
    implementations: info.listClassImplementations().map((c) => ({
      name: c.name,
      methods: c.methods.map((m) => m.getName()),
    })),
    forms: info.listFormDefinitions().map((f) => f.name),
  };
}

function tokenJson(t) {
  return {text: t.getStr(), kind: t.constructor.name, row: t.getRow(), col: t.getCol()};
}

function sqlCases(dir, outFile) {
  const config = Config.getDefault(Version.v816);
  const out = [];
  for (const f of fs.readdirSync(dir).sort()) {
    for (const c of JSON.parse(fs.readFileSync(path.join(dir, f), "utf8"))) {
      if (c.expect !== "match" || !DB_STATEMENTS.has(c.type)) {
        continue;
      }
      const reg = new Registry(config).addFile(new MemoryFile("zfoo.prog.abap", c.abap)).parse();
      const file = reg.getFirstObject().getABAPFiles()[0];
      const s = file.getStatements().find((x) => DB_STATEMENTS.has(x.get().constructor.name));
      if (s !== undefined) {
        out.push({abap: c.abap, type: s.get().constructor.name, sql: sqlInfo(s)});
      }
    }
  }
  fs.mkdirSync(path.dirname(outFile), {recursive: true});
  fs.writeFileSync(outFile, JSON.stringify(out, null, 1) + "\n");
  console.log(`oracle: ${out.length} sql cases`);
}

// SAP's global macros, like those of the logical database PNP, as the
// parser has them in SAP_MACROS of crates/abap-syntax/src/lib.rs
function sapMacros() {
  const rust = fs.readFileSync(new URL("../crates/abap-syntax/src/lib.rs", import.meta.url), "utf8");
  const list = rust.match(/pub const SAP_MACROS: &\[&str\] = &\[([^\]]*)\]/);
  if (list === null) {
    throw new Error("SAP_MACROS not found in crates/abap-syntax/src/lib.rs");
  }
  return [...list[1].matchAll(/"([^"]+)"/g)].map((m) => m[1]);
}

// a configuration for the release with SAP's global macros
function configFor(version) {
  const conf = Config.getDefault(version).get();
  conf.syntax.globalMacros = sapMacros();
  return conf;
}

// on-premise releases, oldest first
const RELEASES = [
  "v700",
  "v702",
  "v740sp02",
  "v740sp05",
  "v740sp08",
  "v750",
  "v751",
  "v752",
  "v753",
  "v754",
  "v755",
  "v756",
  "v757",
  "v758",
  "v816",
];

// abaplint's file name for a path: a plain NAME.abap is a program
function objectName(f) {
  const name = path.basename(f).toLowerCase();
  return name.split(".").length === 2 ? name.replace(/\.abap$/, ".prog.abap") : name;
}

function releases(outDir, inputs) {
  let files = 0;
  let needing = 0;
  for (const folders of inputs) {
    const names = new Map();
    const sources = [];
    for (const f of walkAll(folders)) {
      names.set(objectName(f), path.relative(process.cwd(), f));
      sources.push([objectName(f), fs.readFileSync(f, "utf8")]);
    }
    // per file: statement key -> type per release
    const kinds = new Map();
    for (const [r, release] of RELEASES.entries()) {
      const reg = new Registry(new Config(JSON.stringify(configFor(Version[release]))));
      for (const [name, text] of sources) {
        reg.addFile(new MemoryFile(name, text));
      }
      reg.parse();
      for (const obj of reg.getObjects()) {
        if (!(obj instanceof ABAPObject)) {
          continue;
        }
        for (const file of obj.getABAPFiles()) {
          const rel = names.get(file.getFilename());
          if (rel === undefined) {
            continue;
          }
          if (!kinds.has(rel)) {
            kinds.set(rel, new Map());
          }
          const byKey = kinds.get(rel);
          for (const s of file.getStatements()) {
            const tokens = s.getTokens().map((t) => [t.getRow(), t.getCol()]);
            const key = JSON.stringify(tokens);
            if (!byKey.has(key)) {
              byKey.set(key, {tokens, types: new Array(RELEASES.length).fill("Unknown")});
            }
            byKey.get(key).types[r] = s.get().constructor.name;
          }
        }
      }
    }
    for (const [rel, byKey] of kinds) {
      const statements = [];
      for (const {tokens, types} of byKey.values()) {
        const newest = types[types.length - 1];
        if (newest === "Unknown" || newest === "Comment") {
          continue;
        }
        // the lowest release from which every newer one parses it alike
        let lowest = types.length - 1;
        while (lowest > 0 && types[lowest - 1] === newest) {
          lowest -= 1;
        }
        if (lowest > 0) {
          statements.push({type: newest, tokens, release: RELEASES[lowest]});
        }
      }
      const out = {file: rel.split(path.sep).join("/"), abaplint: abaplint.Registry.abaplintVersion(), statements};
      const dest = path.join(outDir, rel + ".json");
      fs.mkdirSync(path.dirname(dest), {recursive: true});
      fs.writeFileSync(dest, JSON.stringify(out) + "\n");
      files += 1;
      needing += statements.length;
    }
  }
  console.log(`oracle: ${files} files, ${needing} statements newer than 7.00`);
}

// the corpora, each a list of folders read together
function inputList(inputs) {
  if (inputs[0] === "--list") {
    // one corpus per line, its folders separated by blanks, # comments
    return fs
      .readFileSync(inputs[1], "utf8")
      .split(/\r?\n/)
      .map((l) => l.trim())
      .filter((l) => l !== "" && !l.startsWith("#"))
      .map((l) => l.split(/\s+/));
  }
  return inputs.map((i) => [i]);
}

function* walkAll(folders) {
  for (const folder of folders) {
    yield* walk(folder);
  }
}

function main() {
  if (process.argv[2] === "--sql-cases") {
    sqlCases(process.argv[3], process.argv[4]);
    return;
  }
  if (process.argv[2] === "--releases") {
    releases(process.argv[3], inputList(process.argv.slice(4)));
    return;
  }
  const [outDir, ...rest] = process.argv.slice(2);
  const inputs = inputList(rest);
  if (outDir === undefined || inputs.length === 0) {
    console.error("usage: node oracle.mjs <out-dir> <input-dir-or-file>...");
    process.exit(2);
  }

  // unknown types are void instead of errors, so the syntax check of code
  // using SAP standard types goes on
  const conf = configFor(Version.v816);
  conf.syntax.errorNamespace = "^$";
  const config = new Config(JSON.stringify(conf));
  let files = 0;
  let statements = 0;
  for (const folders of inputs) {
    const reg = new Registry(config);
    const names = new Map();
    for (const f of walkAll(folders)) {
      // abaplint derives the object type from the abapGit style file name;
      // a plain NAME.abap is a program
      const name = objectName(f);
      names.set(name, path.relative(process.cwd(), f));
      reg.addFile(new MemoryFile(name, fs.readFileSync(f, "utf8")));
    }
    for (const [name, text] of Object.entries(STUBS)) {
      if (!names.has(name)) {
        reg.addFile(new MemoryFile(name, text));
      }
    }
    reg.parse();

    for (const obj of reg.getObjects()) {
      if (!(obj instanceof ABAPObject)) {
        continue;
      }
      const refs = dataReferences(reg, obj);
      for (const file of obj.getABAPFiles()) {
        const rel = names.get(file.getFilename());
        if (rel === undefined) {
          continue;
        }
        const out = {
          file: rel.split(path.sep).join("/"),
          abaplint: abaplint.Registry.abaplintVersion(),
          release: config.getVersion(),
          tokens: file.getTokens().map(tokenJson),
          statements: file.getStatements().map((s) => ({
            type: s.get().constructor.name,
            colon: s.getColon() !== undefined,
            tokens: s.getTokens().map((t) => [t.getRow(), t.getCol()]),
            sql: DB_STATEMENTS.has(s.get().constructor.name) ? sqlInfo(s) : undefined,
          })),
          info: fileInfo(file.getInfo()),
          // data references, only complete when the object has no syntax error
          syntaxOk: refs.ok,
          refs: refs.byFile.get(file.getFilename()) ?? [],
        };
        const dest = path.join(outDir, rel + ".json");
        fs.mkdirSync(path.dirname(dest), {recursive: true});
        fs.writeFileSync(dest, JSON.stringify(out) + "\n");
        files += 1;
        statements += out.statements.length;
      }
    }
  }
  console.log(`oracle: ${files} files, ${statements} statements`);
}

main();
