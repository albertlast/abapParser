# abapPaser

An ABAP parser and analysis library written in Rust, with bindings for Node.js and Python.
The goal is to answer structured questions about ABAP code, so tools don't have to half-parse it ad hoc or pass it to an LLM:

- **Structure and symbols**: classes, methods, forms, function modules, signatures, data and types
- **Database access**: which tables and fields are read or written
- **Call graph**: CALL FUNCTION, method calls, PERFORM, INCLUDE, SUBMIT
- **Logic**: conditions, assignments, data flow

Includes and referenced objects that are missing locally are fetched from the SAP system through the ADT REST API.

```sh
npm install abap-parser     # Node.js 24+, prebuilt for Windows x64, macOS, Linux x64/arm64
pip install abap-parser           # Python 3.9+, wheels for the same platforms
```

The command line tool `abap` is attached to each [GitHub release](https://github.com/albertlast/abapPaser/releases). See [Usage](#usage).

## Status

| Milestone | State |
|---|---|
| 1. Workspace, corpus, abaplint oracle | done |
| 2. Lexer, statement splitting, chain expansion, statement kinds | done |
| 3. Declarations + structures → symbols API, Node/Python bindings | done |
| 4. Open SQL → DB access API | done |
| 5. Source providers (abapGit folders, ADT) + call graph | done |
| 6. Expressions, control flow, data flow → logic API | done |
| 6b. Native SQL (HANA SQL, SQLScript) for EXEC SQL and AMDP | done |
| 7. Optional HANA metadata via rust-hdbconnect | next |

Results against [abaplint](https://github.com/abaplint/abaplint), the reference implementation:

The corpus is 1,897 files from eleven open source projects (MIT/Apache-2.0), see [tests/corpus](tests/corpus/README.md):

- **Tokens:** identical in all files.
- **Statement boundaries:** identical in 99.8% of the files, macro expansion included.
- **Statement kinds:** 99.84% of 303,374 statements classified the same. Most of the rest are statements abaplint cannot parse, or macros defined twice, where we follow the ABAP documentation and abaplint does not.
- **abaplint statement test cases:** 99.9% of 4,363 classified the same.
- **Symbols:** identical to abaplint's file information in every compared category across 1,785 files: classes, interfaces, methods, parameters, attributes, aliases, implemented interfaces, forms, implementations. Files where abaplint itself fails to parse a statement are skipped.

- **Database access:** for 1,431 of the corpus's 1,432 database statements, tables, INTO targets, joins and FOR ALL ENTRIES are identical to abaplint. The remaining one is `MODIFY lt_x FROM ls_x` in a loop, which abaplint takes for a database statement. Over abaplint's 1,668 SQL test cases, 99.8% or more agree per category. The remaining differences are intended: nested parenthesized joins count every join, and `PROVIDED BY schema` is not a table.

- **Call graph:** each corpus is loaded through the folder provider. 97.1% of 58,178 method calls get a receiver class; most of the rest are SAP standard types that are not in the sources. Every call into a class of the same corpus resolves to an existing method, inherited and interface methods included, unless the class inherits from an SAP standard class; those calls are marked `hierarchy_incomplete`.

- **Data references:** abaplint's syntax check resolves every variable a statement reads or writes. In the 1,432 files whose objects pass that check (155k references), we find 99.85% of abaplint's references, and 94.8% of ours are also abaplint's. Most of the rest are names abaplint cannot resolve, like constants from SAP includes missing in the corpus. abaplint's conventions are translated before comparing, see `crates/abap-syntax/tests/refs_diff.rs`. For example, in `lo->attr = x` abaplint writes `lo`, while we read `lo` and write `attr`.

- **Native SQL:** abaplint does not parse HANA SQL, so AMDP methods are checked against their own declarations instead. ABAP requires every table, view and AMDP method that an AMDP body uses to be listed after `USING`. In all 33 AMDP methods of the corpus, the tables and AMDP calls the parser finds are exactly the 35 entries of their `USING` lists.

Statement kinds are still decided by keywords. Declaration statements are parsed by grammar. Rejecting invalid syntax in general needs grammars for the remaining statements.

## Symbols

`symbols` returns everything defined in one source file, as JSON with this shape (abridged):

```json
{
  "schema_version": 1,
  "program": {"kind": "report", "name": "zdemo", "span": {...}},
  "classes": [{
    "name": "zcl_demo", "is_global": true, "superclass": "zcl_base",
    "interfaces": [{"name": "zif_demo", ...}],
    "methods": [{"name": "run", "visibility": "public", "is_static": false,
                 "parameters": [{"name": "iv_id", "direction": "importing",
                                 "typing": {"kind": "type", "name": "i", "text": "TYPE i"}}],
                 "raising": ["zcx_error"], ...}],
    "attributes": [...], "types": [...], "events": [...], "aliases": [...],
    "implementation": {"methods": [{"name": "run", "locals": [...], "span": {...}}]}
  }],
  "interfaces": [...], "forms": [...], "function_modules": [...],
  "modules": [...], "event_blocks": [...], "macros": [...], "includes": [...],
  "declarations": [...]
}
```

Function module signatures are read from the statement or from the generated `*"` "Local Interface" comment block.
Structures declared with `BEGIN OF ... END OF` carry their `components`.
`abap schema` prints the full JSON Schema. TypeScript and Python types are generated from it.

## Database access

`db_access` lists every SELECT, INSERT, UPDATE, MODIFY, DELETE, MERGE, OPEN CURSOR and WITH, and also native SQL. For each one it reports:

- the tables, with alias and role (main, join, subquery, CTE, internal table)
- the SELECT list, with the columns each element reads, aggregates and aliases
- joins with their ON conditions, and WHERE and HAVING comparisons
- GROUP BY and ORDER BY
- the INTO/APPENDING target, or the source and SET assignments of DML
- the ABAP data the statement reads (`host_variables`): `@lv_x`, the operands of `@( ... )` and the host variables of the classic syntax
- FOR ALL ENTRIES, UP TO, SINGLE, DISTINCT, set operations, the CLIENT, BYPASSING BUFFER and CONNECTION options, and dynamic parts
- the processing block it is in: method and class, form, function module, module, event

Classic and strict syntax are both supported. Column references are resolved to tables through their aliases. For a dynamic table name like `(c_tabname)`, the value is resolved when it is a constant of the same file.

```json
{
  "operation": "select", "statement": "Select",
  "tables": [{"name": "vbak", "alias": "a", "role": "main"}, {"name": "vbap", "alias": "b", "role": "join"}],
  "fields": [{"text": "a~vbeln", "columns": [{"name": "vbeln", "qualifier": "a", "table": "vbak"}]}],
  "joins": [{"kind": "inner", "table": {...}, "on": [...]}],
  "conditions": [{"left": "a~erdat", "operator": ">=", "right": "@lv_date",
                  "columns": [{"name": "erdat", "qualifier": "a", "table": "vbak"}]}],
  "target": {"targets": ["@DATA(lt_items)"], "kind": "table", "appending": false, "corresponding": false},
  "block": {"kind": "method", "name": "read_items", "class": "zcl_orders"},
  "span": {...}
}
```

### Native SQL

Native SQL is HANA SQL: the statements between `EXEC SQL` and `ENDEXEC`, and the SQLScript bodies of AMDP methods (`METHOD ... BY DATABASE PROCEDURE|FUNCTION FOR HDB LANGUAGE SQLSCRIPT`). It has its own parser. Each statement becomes one database access, with the same details as Open SQL:
- tables, joins, conditions, the SELECT list, GROUP BY, ORDER BY, SET of UPDATE and MERGE
- the target: a table variable (`lt_rows = SELECT ...`), `SELECT ... INTO var`, `RETURN SELECT ...` of a table function, or `INTO :wa` of EXEC SQL
- `host_variables`: the `:name` variables read, which are the ABAP host variables of EXEC SQL, or the parameters and table variables of an AMDP

Supported are:
- SELECT with WITH, joins, subqueries, set operations, TOP and LIMIT
- table variables in FROM (role `internal_table`), table functions and the hierarchy functions
- INSERT, UPDATE, DELETE, UPSERT, REPLACE, MERGE, TRUNCATE and CALL
- cursors
- the SQLScript blocks IF, WHILE, FOR, BEGIN ... END and exit handlers, followed to find the statements inside them

Names in quotes keep their case. In AMDP bodies `"` quotes names; in EXEC SQL it starts an ABAP comment.

AMDP methods have more information in other APIs:
- In symbols, the method implementation lists the AMDP kind, language, OPTIONS and USING.
- In calls, calls of other AMDP methods (`CALL "ZCL_X=>M"( ... )`) and their table functions in FROM are method calls.

Not covered: the data flow inside SQLScript in the logic API; dynamic SQL (`EXEC :lv_sql`) is a `native` access with its text.

## Calls and sources

`calls` lists the calls of one source:
- method calls, functional and `CALL METHOD`
- constructors, from `NEW` and `CREATE OBJECT`
- `CALL FUNCTION`, `PERFORM`, `SUBMIT`, transactions, `CALL TRANSFORMATION` and `INCLUDE`

Each call has its processing block, receiver, target class, options like `DESTINATION` or `IN UPDATE TASK`, and a `dynamic` flag. Receiver types come from declarations, parameters, attributes and inline `DATA(x) = NEW ...` or `x = NEW #( )`. References inside structures are followed, `ls_node-idref->get( )` or `lr_node->idref->get( )`, with local types and dictionary structures. Data elements typed as a reference, `io_id TYPE zid_ref`, lead to their class or interface. Chains like `zcl_factory=>get( )->run( )` are typed through returning parameters. Dynamic names written as literals, `('CL_X')=>('METH')`, count as static.

`call_graph` follows calls across objects. It loads them from source providers:

| Provider | Reads |
|---|---|
| folders | abapGit layout (`zcl_x.clas.abap`, `zcl_x.clas.locals_imp.abap`, `zgrp.fugr.z_fm.abap`, `#ns#name` namespaces) or plain `name.abap` files; dictionary types from `name.tabl.xml` and `name.dtel.xml` |
| SAP system | ADT REST API: programs, includes, classes and their local parts, interfaces, function groups and modules, dictionary structures, tables and data elements. A function module's group is found through the repository search, a generated lock module's group (`ENQUEUE_x`) through its lock object. Uses the operating system's certificate store, so corporate CAs work |
| cache | sources fetched from the SAP system, one folder per system and client |

Folders are asked first, then the SAP system. Includes are expanded. Types of other classes are loaded when needed, for example the superclass of an inherited method or the interface of a returned reference. Each call is marked `resolved` when its target exists. Objects that no provider had are listed in `missing`.

```sh
abap graph CLAS:ZCL_ORDERS --src ./src --depth 2
ABAP_ADT_USER=me ABAP_ADT_PASSWORD=... abap graph PROG:ZREPORT --adt-url https://sap:44300 --client 100 --cache ~/.cache/abap
abap fetch CLAS:ZCL_ORDERS/implementations --adt-url https://sap:44300 --client 100
```

```js
const graph = abap.callGraph(["CLAS:ZCL_ORDERS"], {
  src: ["./src"],
  adt: {url: "https://sap:44300", user, password, client: "100"},
  cache: ".abap-cache",
}, 2);
```

The ADT provider only reads. It is tested against the mock server below and by hand against the SAP ABAP Cloud Developer Trial.

### Testing without an SAP system

`abap mock-adt` serves folders of ABAP code like an SAP system does through ADT. It covers:
- programs and includes
- classes with their includes, interfaces, function groups and modules
- the repository search that finds a function module's group
- the where-used list of an interface, a POST that needs the CSRF token and the session cookie handed out with it

Requests need basic authentication, and with `--client` only that client is accepted. The logon comes from `ABAP_ADT_USER` and `ABAP_ADT_PASSWORD`, or is `DEVELOPER`/`mock`, so a client started with the same environment logs on. Your own projects can use it to test code that loads ABAP from a system: keep part of the code locally and let the rest come over ADT.

```sh
abap mock-adt --src ./abap-src --port 50000
ABAP_ADT_USER=DEVELOPER ABAP_ADT_PASSWORD=mock abap graph CLAS:ZCL_ORDERS --src ./local --adt-url http://127.0.0.1:50000 --cache ./.abap-cache
```

In Rust it is `abap_source::MockAdtServer`. Bind it to port 0 for a free port; `requests()` lists what was asked for. `crates/abap-analysis/tests/adt_mock.rs` uses it: one class is local, the classes it uses come over ADT into the cache, and a second run reads them from the cache.

## Logic

`logic` describes each method, form, function module, dialog module and event block of a source:

- **Steps:** the statements, nested along IF, CASE, LOOP, DO, WHILE and TRY. ELSEIF, ELSE, CATCH and CLEANUP follow their IF or TRY. The WHENs of a CASE are its children.
- **Conditions:** parsed into a tree of AND, OR, NOT, comparisons, `IS INITIAL`/`BOUND`/`ASSIGNED`/`INSTANCE OF`, `IN` and `BETWEEN`. Operands are classified as literal (with its value), data, call, constructor or arithmetic. A WHEN becomes `operand = value OR ...`.
- **Guards:** every condition that holds when a step runs, outermost first. ELSE and ELSEIF get the earlier conditions negated. Statements after `CHECK c` are guarded by `c`. Statements after `IF c. RETURN. ENDIF.` are guarded by `NOT c`.
- **Reads and writes:** each data object a statement reads, writes or changes in place (`APPEND ... TO itab`), with its position and scope: local, parameter, attribute, global (declared at program level, also in event blocks) or other.
- **Field symbols and references:** a write through `<fs>` or `lr->` is a write of the data they point to. That is the data bound to them on the way to the statement, by `ASSIGN`, `LOOP AT ... ASSIGNING`, `READ TABLE ... REFERENCE INTO`, `GET REFERENCE OF`, `REF #( )`, or a call returning a reference. Such accesses name the field symbol or reference in `through`. Binding a field symbol to a table does not change the table. A write through a reference parameter is a write of what the caller passed: `set( lr )` changes what `lr` points to, `touch( lo )` changes `lo->mv_x` when `touch` writes `io->mv_x`.
- **Database tables:** each step lists the tables it reads, and the table that INSERT, UPDATE, MODIFY, DELETE or MERGE writes. Open SQL and native SQL are both covered.
- **Data flow:** each read lists the steps whose writes may provide its value, and whether the value may come from before the procedure. Computed by reaching definitions over the control flow graph, which is also part of the output.
- **Calls:** a call into a procedure that is in the sources is followed. The calling step then also reads and writes what the called procedure does: attributes, global data and database tables. These accesses name the procedure in `via`. Attributes are renamed for the caller: through `lo->run( )` the attribute `mv_x` becomes `lo->mv_x`, through `zcl_x=>run( )` it becomes `zcl_x=>mv_x`. Arguments are matched to parameters by name, by position for PERFORM, and the single unnamed argument to the first importing parameter. Each value a call writes, into an argument, the receiving variable or through the called procedure, carries in `depends` what it depends on there: the caller's data passed to the parameters it uses, data read there, tables read there. A call through an interface is followed into each implementation, if there are no more than 10. Calls through aliases resolve to the interface method. A dynamic call like `CALL METHOD (lv_name)` or `CALL FUNCTION lv_fm` is followed into the names its operand can have on the way to the call, from literals assigned to it and from constants; the operand then counts as an input of what the call writes.
- **Effects:** every parameter, attribute, global or database table the procedure writes, directly or through calls. Each effect lists the inputs and tables the written values depend on, through assignments and through the conditions that decide them, and the procedures it is written through.

```abap
FORM price USING iv_qty TYPE i CHANGING cv_price TYPE p.
  CHECK iv_qty > 0.
  IF iv_qty >= 100.
    cv_price = cv_price * '0.9'.
  ENDIF.
ENDFORM.
```

For this form, the step `cv_price = cv_price * '0.9'` has the guards `iv_qty > 0` and `iv_qty >= 100`. Its read of `cv_price` comes from the entry. The effects are `cv_price` with the inputs `cv_price` and `iv_qty`.

```abap
METHOD total.    " IMPORTING iv_vbeln EXPORTING ev_total
  SELECT SUM( netwr ) FROM vbap INTO @DATA(lv_sum) WHERE vbeln = @iv_vbeln.
  ev_total = lv_sum.
ENDMETHOD.
METHOD run.
  total( EXPORTING iv_vbeln = gv_vbeln IMPORTING ev_total = DATA(lv_total) ).
  IF lv_total > 100.
    write_log( |big| ).    " INSERT zlog ..., mv_count = mv_count + 1
  ENDIF.
ENDMETHOD.
```

`total` has the effect `ev_total` with the input `iv_vbeln` and the table `VBAP`. `run` has the effects `ZLOG` (scope database) and `mv_count`, both through `write_log`, and both depend on `VBAP` because the IF decides on the result of `total`.

For one source, the calls between its own procedures are followed. Through a workspace, `logic` works on a whole object: all class parts, the includes of a program or function group, and its function modules. Data declared in the TOP include is then global in every other include. Calls into other objects are followed up to `--depth` hops, default 1. Global data of another object is named with that object, like `(zgroup)gv_buffer` for function group ZGROUP. A procedure calling itself, directly or through others, is followed once.

```sh
abap logic FUGR:ZORDERS --src ./src --depth 2
```

Not covered yet:
- A method redefined in a subclass is followed into the class the call names, not into the redefinitions.
- Dynamic names computed at runtime, e.g. by concatenation or read from a table, are not followed.
- Implementations of global interfaces come from the source folders, or from an SAP system through ADT: from the class relations (table SEOMETAREL) read through the data preview, or, where that is not allowed, from the where-used list. The where-used index of a system may be incomplete for SAP standard code.
- A field symbol or reference stored in a structure, table or attribute is not followed.

## Language models (MCP)

`abap mcp` is an [MCP](https://modelcontextprotocol.io) server over stdio. It gives language models the analyses as tools, with sources from the same options as the other commands: folders, an SAP system, a cache. Loaded objects are kept for the session.

| Tool | Answers |
|---|---|
| `get_source` | source of an object with row numbers, optionally a row range |
| `get_symbols` | classes, interfaces, methods with signatures, attributes, types, forms, declarations, written like ABAP |
| `get_call_graph` | calls from objects with calling procedure and row, followed `depth` hops; one line per call, `detail: "full"` for JSON |
| `get_logic` | per procedure its parameters and effects: what it writes, in which statements, under which conditions, from which inputs; `procedure` for one, `detail: "full"` for every statement and the control flow |
| `get_db_access` | Open SQL and native SQL statements, one line each; `detail: "full"` adds fields, joins, conditions |
| `find_implementations` | global classes implementing an interface |
| `analyze_source` | the analyses of ABAP code given as text, without a system |

Answers are in the compact text form unless `detail: "full"` asks for JSON, and are cut at 80,000 characters with a hint to ask for less.

With Claude Code:

```sh
claude mcp add abap -e ABAP_ADT_URL=https://sap:44300 -e ABAP_ADT_CLIENT=100 \
  -e ABAP_ADT_USER=me -e ABAP_ADT_PASSWORD=... -- abap mcp --src ./src --cache ~/.cache/abap
```

Other clients take the same as JSON:

```json
{
  "mcpServers": {
    "abap": {
      "command": "abap",
      "args": ["mcp", "--src", "./src", "--cache", ".abap-cache"],
      "env": {"ABAP_ADT_URL": "https://sap:44300", "ABAP_ADT_CLIENT": "100",
              "ABAP_ADT_USER": "me", "ABAP_ADT_PASSWORD": "..."}
    }
  }
}
```

The `abap` binary is in the archives of the GitHub releases.

## Layout

```
crates/abap-lexer     tokenizer, token boundaries identical to abaplint
crates/abap-syntax    statements (splitting, chains, pragmas, macros, native SQL, kinds),
                      block structure, declaration grammars, expressions and conditions,
                      operands read and written per statement
crates/abap-model     serializable output model and its JSON Schema
crates/abap-sql       Open SQL parser; HANA SQL and SQLScript parser for EXEC SQL and AMDP
crates/abap-source    source providers: folders, disk cache, SAP ADT; ADT mock server
crates/abap-analysis  symbols, database access, calls, logic, workspace and call graph,
                      compact text forms
crates/abap-cli       `abap tokens|statements|symbols|db|calls|logic <file>`, `abap graph|logic|fetch <object>`, `abap mcp`, `abap mock-adt`, `abap schema`
bindings/node         Node.js package (napi-rs); npm/ builds the per-platform packages of a release
bindings/python       Python package (PyO3, maturin)
tools/                corpus extraction, abaplint oracle, type generation, release version,
                      keyword documentation lookup
tests/corpus/         test corpus, see tests/corpus/README.md
```

## Usage

```sh
cargo run -p abap-cli -- symbols zcl_demo.clas.abap
cargo run -p abap-cli -- db zcl_demo.clas.abap
cargo run -p abap-cli -- calls zcl_demo.clas.abap
cargo run -p abap-cli -- logic zcl_demo.clas.abap
cargo run -p abap-cli -- logic PROG:ZREPORT --src ./src --depth 1
cargo run -p abap-cli -- statements zprogram.prog.abap --json
cargo run -p abap-cli -- logic zcl_demo.clas.abap --text
```

### Compact text

The JSON is the complete form. For language models, and for reading, `--text` and the `text` functions of the packages give a compact form: definitions written like ABAP, one line per call or database access, and per procedure its effects with the statements writing them and the conditions they run under. Rows tell where to find things. Over the corpus it is 6.7 times shorter than the JSON for symbols and 9.2 times for logic. The MCP server answers in it by default.

```
zcl_orders->save 21-27
  parameters: importing iv_id, importing iv_status, exporting ev_count
  reads tables: ZORDERS
  writes ZORDERS (database table) at 24; from iv_id, iv_status
  writes ev_count (parameter) at 25; from iv_id, sy
  statements:
       24 UPDATE zorders SET status = @iv_status WHERE id = @iv_id.  when iv_id > 0
       25 ev_count = sy-dbcnt.  when iv_id > 0
```

Node.js (`npm install abap-parser`, or from this repository `cd bindings/node && npm run build`):

```js
const abap = require("abap-parser");
const s = abap.symbols(source);
for (const c of s.classes) {
  console.log(c.name, c.methods.map((m) => m.name));
}
for (const a of abap.dbAccess(source)) {
  console.log(a.operation, a.tables.map((t) => t.name));
}
for (const p of abap.logic(source).procedures) {
  console.log(p.block.name, p.effects.map((e) => `${e.name} <- ${e.inputs.join(", ")}`));
}
// a whole object with its includes, calls into other objects followed 1 hop
const l = abap.logicObject("PROG:ZREPORT", {src: ["./src"]}, 1);
// the same as a promise, without blocking the event loop
const graph = await abap.callGraphAsync(["CLAS:ZCL_ORDERS"], {src: ["./src"]}, 2);
// compact text, e.g. for a prompt: "symbols", "calls", "db" or "logic"
const prompt = abap.text("logic", source);
// of objects: "symbols", "logic" or "graph"
const outline = abap.objectText("symbols", ["CLAS:ZCL_ORDERS"], {src: ["./src"]});
```

Every function has an `…Async` variant. It runs on the libuv thread pool, so loading sources from an SAP system or analyzing large files does not block the event loop, and several calls run in parallel.

Python (`pip install abap-parser`, or from this repository `cd bindings/python && maturin develop --release`):

```python
import abap_parser
s = abap_parser.symbols(source)
for c in s["classes"]:
    print(c["name"], [m["name"] for m in c["methods"]])
for a in abap_parser.db_access(source):
    print(a["operation"], [t["name"] for t in a["tables"]])
for p in abap_parser.logic(source)["procedures"]:
    print(p["block"]["name"], [(e["name"], e["inputs"]) for e in p["effects"]])
lg = abap_parser.logic_object("PROG:ZREPORT", {"src": ["./src"]}, depth=1)
# in asyncio code
graph = await abap_parser.call_graph_async(["CLAS:ZCL_ORDERS"], {"src": ["./src"]}, 2)
# compact text, e.g. for a prompt
prompt = abap_parser.text("logic", source)
outline = abap_parser.object_text("symbols", ["CLAS:ZCL_ORDERS"], {"src": ["./src"]})
```

Every function has an `..._async` variant for asyncio, running in a worker thread. The native code releases the GIL.

Rust:

```rust
let symbols = abap_analysis::symbols(source);
let file = abap_syntax::parse(source);
for s in &file.statements {
    println!("{} {}", s.kind.as_str(), file.statement_text(s));
}
```

## Development

```sh
git submodule update --init --depth 1
npm ci --prefix tools
node tools/oracle.mjs tests/corpus/golden --list tests/corpus/corpora.txt
node tools/oracle.mjs --sql-cases tests/corpus/abaplint/statements tests/corpus/golden/sql_cases.json
cargo test --release -- --nocapture             # prints the coverage reports
node tools/gen-types.mjs                          # after changing crates/abap-model
node tools/sapdoc.mjs ABAPMODIFY_ITAB_SHORTREF    # syntax lookup in the ABAP Keyword Documentation
```

### Releasing

The version is kept in the Cargo workspace; `bindings/node/package.json` follows it and the Python package reads it through maturin.

```sh
node tools/version.mjs 0.2.0          # set it in Cargo.toml, Cargo.lock, package.json
# add a "## 0.2.0" section to CHANGELOG.md, commit, then
git tag v0.2.0 && git push origin v0.2.0
```

The tag starts `.github/workflows/release.yml`. It checks that the tag, the packages and the changelog agree, runs CI, and builds the packages:
- the native Node.js module per platform, tested where the runner can load it;
- Python wheels and the source distribution;
- the command line tool per platform.

It then publishes:
- `abap-parser` and its platform packages (`abap-parser-linux-x64-gnu` and so on) to npm through trusted publishing, with provenance;
- `abap-parser` to PyPI through trusted publishing, which needs a PyPI trusted publisher for this repository and the environment `pypi`;
- a GitHub release with the command line tool and the changelog section.

Started by hand from the Actions tab, the workflow builds and tests everything and keeps the results as artifacts, without publishing.

npm and PyPI trust the workflow through OIDC, no tokens are stored. Set up once:
- PyPI: a pending trusted publisher for the project `abap-parser`, repository `albertlast/abapPaser`, workflow `release.yml`, environment `pypi`; and the environment `pypi` in the repository settings.
- npm: a trusted publisher for each of the six packages, repository `albertlast/abapPaser`, workflow `release.yml`. npm sets this up only for packages that exist, so before the first release publish placeholders by hand:

  ```sh
  node bindings/node/npm/placeholders.mjs npm-placeholders
  # in each of the six folders, logged in with npm login:
  npm publish --access public
  ```

  Then in the settings of each package on npmjs.com, add the trusted publisher, and under publishing access require two-factor authentication and disallow tokens.

`schema_version` in the JSON results changes only when a result changes in a way that breaks readers. New optional fields leave it unchanged.
