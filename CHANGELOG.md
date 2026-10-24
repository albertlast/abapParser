# Changelog

## 0.4.0

The release each statement needs, syntax checks, and lineage for SAP_BASIS 7.50 systems and HR code: sources from the host, `sy-subrc` and system fields per statement, infotypes, locks and data clusters.

Changes to existing output: system fields are named by field (`sy-dbcnt`, before `sy`), steps carry implicit writes of system fields, and `GET` statements are steps. In Rust, the functions of `abap_analysis::json` that load sources take a source provider of the caller as a new last argument.

- **Sources from the host:** `fetch` in the source options of Node.js and Python is a function of the host program, called with each object the analysis needs (`CLAS:ZCL_X/implementations`, `FUNC:Z_FM`, ...) and returning its source or nothing, e.g. from the host's own ADT client or cache. It is asked after the folders and before the SAP system; what it throws is listed in `errors`. In Node.js only the `…Async` functions take it, a promise as its result included; the blocking ones throw a `TypeError`. In Rust it is `abap_source::FnProvider`, with `ProviderOptions::build_with`. The JSON entry points in `abap_analysis::json` that load sources take it as a new last argument.
- **`sy-subrc` per statement:** guards on `sy-subrc` name in the new `subrc` the statements that may have set the value they check. `IF sy-subrc = 0` after a READ TABLE and `IF sy-subrc NE 0` after a call inside it were one contradicting condition before. Statements that set `sy-subrc`, as SAP documents for each, write it (new `implicit` on data accesses), so the data flow gives its sources: a write under `IF sy-subrc = 0` after a READ TABLE now depends on the table read. Method calls set it, PERFORM may. The compact text writes `sy-subrc = 0 [sy-subrc of 70]`. System fields are each their own data object, `sy-subrc`, `sy-dbcnt`, no longer `sy`.
- **System fields:** the 171 fields of SYST are known with their types, as the structure SYST of an SAP system has them (abaplint agrees on 167; it has `sy-fdayw` untyped and `sy-spono` as text). Their accesses have the new scope `system` and the new `data_type` (kind, length, decimals, data element). Statements set them as SAP documents: `sy-tabix` (READ TABLE, LOOP AT, APPEND, COLLECT), `sy-index` (DO, WHILE), `sy-dbcnt` (database statements, FETCH, EXEC SQL), `sy-fdpos` (SEARCH, CO, CN, CA, NA, CS, NS, CP, NP), `sy-tfill` and `sy-tleng` (DESCRIBE TABLE), the message fields (MESSAGE). Reads of them come from those statements, so a value written from `sy-dbcnt` depends on what the SELECT before read; fields of the runtime, like `sy-datum`, are no inputs of effects, since callers do not pass them.
- **Clusters in database access:** `EXPORT ... TO DATABASE dbtab(ar) ID id`, `IMPORT ... FROM DATABASE`, `IMPORT DIRECTORY` and `DELETE FROM DATABASE` are database accesses of the table (`modify`, `select`, `delete`), with the new `cluster`: area, ID and the names in the cluster; exported data in `source`, imported data in `target`. They were only in the logic effects. The compact text writes `modify pcl2(ru) cluster`.
- **Locks:** lock modules are no longer followed into their generated code; `ENQUEUE_<lock object>` writes the new store `lock:<lock object>` (`Store::Lock`), `DEQUEUE_` deletes it, `DEQUEUE_ALL` deletes `lock:*`. Lock objects start with `E`, so `ENQUEUE_READ` is no lock module.
- **HR infotypes:** `INFOTYPES nnnn` is a declaration (kind `infotype`) of the table `pnnnn`, so the table is global data. The infotype number decides the table: `HRPnnnn` for personnel planning (1000 to 1999, 5000 to 5999), `PBnnnn` for recruitment (4000 to 4999) and applicants, else `PAnnnn`; customer infotypes (9000 to 9999) after the reader, `HRP` for PCH and `RH_READ_INFTY`. At `GET PERNR` and `GET PERAS` the logical database fills the infotype tables in `PA`, at `GET OBJEC` those in `HRP`, each from its own, apart from `MODE N`. `HR_READ_INFOTYPE`, `RH_READ_INFTY`, `RH_READ_INFTY_1001` and `RP-READ-INFOTYPE` read the table of the infotype they name. The macros `RP-PROVIDE-FROM-LAST`/`-FRST`, `RP-READ-INFOTYPE` and `RP-SET-DATA-INTERVAL` read and write their arguments, after SAP's description of their use; their definitions are not read. `GET` statements are steps now.
- **Releases:** each statement gets the oldest on-premise ABAP release that has everything it uses, from 7.02 to 8.16, with the construct that needs it, like `7.40 SP02: inline declaration`. New `release` and `error` on statements. New command `abap release <file> [--release 750]` gives the release a file needs and the statements newer than a given release. 98.2% of the corpus statements get the release abaplint gives; all but 3 of the rest are cases where abaplint reads newer syntax as older or dates predicate functions later. The oracle has a new mode, `--releases`, that parses the corpus once per release.
- **Releases from SAP's release news:** where abaplint dates an ABAP SQL construct differently from SAP's ABAP release news, the release news win. `INTO @DATA(x)` needs 7.40 SP08 (abaplint: SP05), searched `CASE WHEN` SP08, path expressions 7.50 and with parameters or filters 7.52, `CROSS JOIN` 7.51, hierarchies 7.53, window expressions (`OVER`) and `INTO … INDICATORS` 7.54 (abaplint: 7.57 and 8.16), `LIKE_REGEXPR`, `OCCURRENCES_REGEXPR` and `REPLACE_REGEXPR` 7.55, `SUBSTRING_REGEXPR` 7.56. New: `FIRST_VALUE`, `LAST_VALUE`, `NTILE` and window frames (`ROWS BETWEEN`) 7.55, `INDICATORS … NULL BITFIELD` 7.56, inline declaration in `OPEN CURSOR` 7.51. The comparison with abaplint counts these apart: 146 corpus statements and 90 abaplint cases. abaplint cases met: 99.0%, from 98.0%.
- **Syntax errors:** checks for errors of known kinds, like a keyword as a column, parameter sections out of order or `PACKAGE SIZE` without a table. They report 69 of abaplint's 77 cases that are invalid at every release, and none of the corpus statements abaplint accepts.
- **SAP macros:** calls of the macros of the logical database PNP (`RP-PROVIDE-FROM-LAST`, `RP-PROVIDE-FROM-FRST`, `RP-READ-INFOTYPE`, `RP-SET-DATA-INTERVAL`), which HR reports use without defining them, are `MacroCall` statements instead of `Unknown`. Macros with dashes in their names, `DEFINE my-mac.`, are expanded; only the part before the first dash was the name.
- **Field groups:** `INSERT f1 f2 INTO header` is `InsertFieldGroup`, not an internal table insert.
- **Text literals:** a literal in quotes ends at its closing quote also when no blank follows, as in SQLScript `'x';`, like abaplint 2.120.86.
- **abaplint 2.120.86:** the reference is updated from 2.120.60, with its statement test cases extracted again.
- **Older SAP systems:** tested against an ERP on SAP_BASIS 750 SP35.
  - What a system offers is read from its ADT discovery, not assumed from its release.
  - Data preview queries the system rejects fall back to the table preview, with single-table queries.
  - The enhancement lookup no longer filters on `ENHINCINX-FULL_NAME`, a long string that SAP_BASIS 750 rejects in a condition. This was the HTTP 400 for every function module.
  - Data elements are requested in the version the system lists.
- **Notes:** a lookup the system cannot do is no longer an error. It is a note in the new `notes` of call graph and logic results, like `enhancements: not available (the ADT discovery has no enhancements/enhoxhh to read their code, SAP_BASIS 750 SP 0035)`. Answers that could not be given are not cached.
- **Mock of SAP_BASIS 750:** `abap mock-adt --release 750` (`MockRelease::Basis750` in Rust) acts like the ERP measured. It has that system's discovery and component list, and serves data elements only in XML version 1 (406 otherwise), structures but no tables, and no enhancement code. Its data preview answers the queries of the ADT provider from the class relations, BAdIs and enhancements of the served folders. It rejects what 7.50 rejects: long strings in `WHERE` and lines over 255 characters. A test checks that the call graph over it matches the one read from the folder, with one note.
- **Request log:** `ABAP_ADT_LOG=1` writes every ADT request to stderr: method, path, status, duration, and the SQL of data preview queries. It never writes headers, cookies, tokens or credentials.
- **Fix:** two parallel loads of the same object could make a cache write fail.
- **Fix:** parallel loads asking for the same BAdI each sent the lookup to the system; now one asks and the others wait. The 7.50 mock showed it: one query went out six times in one call graph.
- **Formatting:** Markdown, JavaScript and JSON are formatted with prettier 3.9.10 (`.prettierrc.json`, `npm run --prefix tools format`), checked in CI, so editors formatting on save no longer change them.
- **Corpus:** three HR projects with logical database PNP, `INFOTYPES` and payroll clusters. The oracle reads plain `NAME.abap` files as programs.

## 0.3.0

BAdIs, enhancements and customer exits are followed to the code that runs; loading from an SAP system is faster.

- **BAdIs:** `CALL BADI` resolves to the interface of the BAdI, and calls through the interface of a classic BAdI (`IF_EX_...`) reach only its active implementations, not the adapter class. The call graph loads the classes that run, logic follows them. They come from the BAdI tables of an SAP system or from abapGit `.enhs.xml`, `.enho.xml`, `.sxsd.xml` and `.sxci.xml` files. New `badi` on calls, MCP tool `find_badi_implementations`.
- **Enhancements:** code of enhancement implementations is put where it runs when an object is loaded: at enhancement points, in place of enhancement sections, and at the begin or end of forms, function modules, methods and includes. It is on the line of its position, so rows stay. From an SAP system through the index of enhancement includes, without implementations switched off, or from abapGit `.enho.<hash>.abap` files. The call graph lists them as units `ENHO:<name>`.
- **Parallel loading:** objects, class parts and includes are loaded up to 8 at once; the call graph works level by level, logic loads the objects its calls reach before. From an SAP system with an empty cache, the call graph of `BAPI_USER_GET_DETAIL` two hops deep takes 3 s instead of 14 s, its logic 6.3 s instead of 8.2 s; results are the same.
- **Cache:** the cache also keeps the answers of lookups, under `lookups/`: enhancements, implementations, subclasses and BAdIs. A second call graph of `BAPI_USER_GET_DETAIL` two hops deep takes 0.6 s.
- **Customer exits:** `CALL CUSTOMER-FUNCTION '001'` calls `EXIT_<program>_001`, named by the program; it was a dynamic call of `-FUNCTION` before.

## 0.2.0

Tested against a real SAP system (ABAP Cloud Developer Trial 2025), and the pieces for language models and for following data through code.

- **MCP server:** `abap mcp` gives language models the analyses as tools over stdio: source, symbols, call graph, logic, database access, implementations, subclasses, and the analyses of pasted code. Sources come from folders, an SAP system and a cache.
- **Compact text:** symbols written like ABAP definitions, one line per call or database access, and per procedure its effects with the statements writing them and their conditions. 6.7 times shorter than the JSON for symbols, 9 times for logic. CLI `--text`, the default of the MCP server, `text` and `objectText` in Node.js, `text` and `object_text` in Python.
- **CDS views:** data definitions are parsed: sources with joins, associations, parameters, WHERE, and the fields each element is made of. Objects `DDLS:NAME`, CLI `cds`, `cds()` in both packages.
- **Dictionary types:** structures, tables (`TABL:NAME`) and data elements (`DTEL:NAME`) from ADT and abapGit resolve receivers like `ls_node-ref->get( )` and parameters typed with a data element referring to a class.
- **Logic:**
  - `SUBMIT` is followed: the tables of the submitted program count for the caller, connected through the values passed with `WITH`.
  - Data kept outside the program is handled like tables: ABAP memory, SAP memory (`SET/GET PARAMETER`), files, shared buffers; data clusters as accesses of their table. New effect scope `external` and `store` on table accesses.
  - Calls of instance methods also reach their redefinitions in subclasses.
- **Calls:** `x = NEW #( )` takes its class from `x`. Generated lock modules (`ENQUEUE_x`) are found through their lock object.
- **SAP systems:** implementations of interfaces and subclasses come from the class relations through the data preview, the where-used list is the fallback. The root class `OBJECT` is no longer requested.
- **Tools:** `tools/version.mjs` and `tools/gen-types.mjs --check` work on a checkout with CRLF line ends.

## 0.1.0

First release: the Rust core with a command line tool, Node.js and Python packages.

- **Symbols:** classes, interfaces, methods with signatures, attributes, forms, function modules, dialog modules, event blocks, macros, includes and declarations.
- **Database access:** Open SQL in classic and strict syntax, with tables, fields, joins, conditions, targets and the ABAP data read. Native SQL in EXEC SQL and AMDP methods, parsed as HANA SQL and SQLScript.
- **Calls and call graph:** methods, function modules, forms, programs, transactions, with receiver types, across objects.
- **Logic:** steps nested along the control structures, with conditions, guards, a control flow graph and data flow. Effects list the parameters, attributes, globals and database tables a procedure writes, and what the values depend on. Calls are followed into the procedures they reach: through interfaces, dynamic names with known values, field symbols and references, with arguments matched to parameters.
- **Sources:** abapGit or plain folders, a disk cache, and SAP systems through ADT. Includes and function modules are loaded with their object. A local ADT mock server is included for tests.
- **Bindings:** every function also as a promise (Node.js) or coroutine (Python) that does not block the event loop. Types are generated for TypeScript and Python.

The JSON results have `schema_version` 1. It changes when a result changes in a way that breaks readers; new optional fields do not change it.
