# Changelog

## Unreleased

- **BAdIs:** `CALL BADI` resolves to the interface of the BAdI, and calls through the interface of a classic BAdI (`IF_EX_...`) reach only its active implementations, not the adapter class. The call graph loads the classes that run, logic follows them. They come from the BAdI tables of an SAP system or from abapGit `.enhs.xml`, `.enho.xml`, `.sxsd.xml` and `.sxci.xml` files. New `badi` on calls, MCP tool `find_badi_implementations`.
- **Enhancements:** code of enhancement implementations is put where it runs when an object is loaded: at enhancement points, in place of enhancement sections, and at the begin or end of forms, function modules, methods and includes. It is on the line of its position, so rows stay. From an SAP system through the index of enhancement includes, without implementations switched off, or from abapGit `.enho.<hash>.abap` files. The call graph lists them as units `ENHO:<name>`.
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
