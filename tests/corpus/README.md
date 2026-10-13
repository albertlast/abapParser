# Test corpus

| Folder | Source | License | Used for |
|---|---|---|---|
| `abaplint/statements/` | statement tests of [abaplint](https://github.com/abaplint/abaplint) `v2.120.60`, extracted by `tools/extract-abaplint-tests.mjs` | MIT, see `abaplint/LICENSE` | `crates/abap-syntax/tests/abaplint_statements.rs` |
| `abapgit/` (submodule) | [abapGit](https://github.com/abapGit/abapGit) | MIT | whole-file differential tests |
| `abap-cheat-sheets/` (submodule) | [SAP-samples/abap-cheat-sheets](https://github.com/SAP-samples/abap-cheat-sheets) | Apache-2.0 | whole-file differential tests, modern syntax |
| `abap2xlsx/` (submodule) | [abap2xlsx/abap2xlsx](https://github.com/abap2xlsx/abap2xlsx) | Apache-2.0 | large classes, older style, demo reports |
| `code-pal-for-abap/` (submodule) | [SAP/code-pal-for-abap](https://github.com/SAP/code-pal-for-abap) | Apache-2.0 | check framework, inherits from SAP standard classes |
| `abapOpenChecks/` (submodule) | [larshp/abapOpenChecks](https://github.com/larshp/abapOpenChecks) | MIT | checks, macros in test classes |
| `abap-sdk-for-azure/` (submodule) | [microsoft/ABAP-SDK-for-Azure](https://github.com/microsoft/ABAP-SDK-for-Azure) | MIT | classic enterprise style, function groups, macros |
| `abap_fm_json/` (submodule) | [cesar-sap/abap_fm_json](https://github.com/cesar-sap/abap_fm_json) | Apache-2.0 | function module heavy, redefined macros |
| `abap2UI5/` (submodule) | [abap2UI5/abap2UI5](https://github.com/abap2UI5/abap2UI5) | MIT | very modern syntax |
| `ajson/` (submodule) | [sbcgua/ajson](https://github.com/sbcgua/ajson) | MIT | modern library with many unit tests |
| `abap-logger/` (submodule) | [ABAP-Logger/ABAP-Logger](https://github.com/ABAP-Logger/ABAP-Logger) | MIT | small, macros |
| `spma-2025-08/` (submodule) | [keremkoseoglu/spma-2025-08](https://github.com/keremkoseoglu/spma-2025-08) | MIT | AMDP: SQLScript procedures and table functions |
| `golden/` (generated, gitignored) | output of `tools/oracle.mjs` (@abaplint/core): tokens, statements, symbols, SQL details and data references of the syntax check per corpus file, plus `sql_cases.json` for the SQL statement cases | - | `oracle_diff.rs`, `symbols_diff.rs`, `sql_diff.rs`, `refs_diff.rs` |
| `private/` (gitignored) | your own ABAP code, never committed | - | local experiments |

The source folders used by the oracle and the corpus tests are listed in `corpora.txt`.
Only MIT and Apache-2.0 licensed repositories are included; repositories without a license are not.

Each extracted statement case looks like `{"abap": "...", "expect": "match" | "unknown", "type": "Select", "release"?: "v751"}`.
`expect: "unknown"` cases are syntax that abaplint rejects.

## Regenerating

```sh
git submodule update --init --depth 1
npm ci --prefix tools
node tools/extract-abaplint-tests.mjs        # tests/corpus/abaplint/statements
node tools/oracle.mjs tests/corpus/golden --list tests/corpus/corpora.txt
node tools/oracle.mjs --sql-cases tests/corpus/abaplint/statements tests/corpus/golden/sql_cases.json
```

The ABAP Keyword Documentation is not part of the corpus. It is copyrighted by SAP SE.
`node tools/sapdoc.mjs <PAGE>` looks up statement syntax there for reference, and caches pages under `tools/.cache/` only.
