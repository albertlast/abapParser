# Changelog

## 0.1.0

First release: the Rust core with a command line tool, Node.js and Python packages.

- **Symbols:** classes, interfaces, methods with signatures, attributes, forms, function modules, dialog modules, event blocks, macros, includes and declarations.
- **Database access:** Open SQL in classic and strict syntax, with tables, fields, joins, conditions, targets and the ABAP data read. Native SQL in EXEC SQL and AMDP methods, parsed as HANA SQL and SQLScript.
- **Calls and call graph:** methods, function modules, forms, programs, transactions, with receiver types, across objects.
- **Logic:** steps nested along the control structures, with conditions, guards, a control flow graph and data flow. Effects list the parameters, attributes, globals and database tables a procedure writes, and what the values depend on. Calls are followed into the procedures they reach: through interfaces, dynamic names with known values, field symbols and references, with arguments matched to parameters.
- **Sources:** abapGit or plain folders, a disk cache, and SAP systems through ADT. Includes and function modules are loaded with their object. A local ADT mock server is included for tests.
- **Bindings:** every function also as a promise (Node.js) or coroutine (Python) that does not block the event loop. Types are generated for TypeScript and Python.

The JSON results have `schema_version` 1. It changes when a result changes in a way that breaks readers; new optional fields do not change it.
