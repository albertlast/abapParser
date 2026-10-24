# abap-parser

Parses ABAP code and tells what it does: classes, methods, forms and declarations; which database tables a statement reads or writes; which methods, function modules and forms it calls; and the logic of each procedure, from conditions and data flow to what it writes in the end, also through SUBMIT, ABAP memory and files. CDS views tell their data sources and the fields of each element. Sources come from a folder (abapGit layout or plain `.abap` files) or from an SAP system through the ADT REST API, SAP standard code and dictionary types included.

```sh
npm install abap-parser
```

Prebuilt for Windows x64, macOS x64 and arm64, Linux x64 and arm64 (glibc). Other platforms build from source with Rust: `npm run build` in this package.

```js
const abap = require("abap-parser");

const source = fs.readFileSync("zcl_orders.clas.abap", "utf8");
for (const c of abap.symbols(source).classes) {
  console.log(
    c.name,
    c.methods.map((m) => m.name),
  );
}
for (const a of abap.dbAccess(source)) {
  console.log(
    a.operation,
    a.tables.map((t) => t.name),
  );
}
for (const p of abap.logic(source).procedures) {
  console.log(
    p.block.name,
    p.effects.map((e) => `${e.name} <- ${e.inputs.join(", ")}`),
  );
}

// whole objects, missing sources loaded from an SAP system; does not block
const options = {
  src: ["./src"],
  cache: "./.abap-cache",
  adt: {url: "https://host:44300", user: process.env.ABAP_ADT_USER, password: process.env.ABAP_ADT_PASSWORD},
};
const graph = await abap.callGraphAsync(["CLAS:ZCL_ORDERS"], options, 2);
const logic = await abap.logicObjectAsync("PROG:ZREPORT", options);
// compact text, e.g. for a language model
const prompt = abap.text("logic", source);
const outline = await abap.objectTextAsync("symbols", ["CLAS:ZCL_ORDERS"], options);
// a CDS view: sources, joins, associations, the fields of each element
const view = abap.cds(fs.readFileSync("zi_orders.ddls.asddls", "utf8"));
// sources of the host, e.g. from its own ADT client: asked after the
// folders, before the SAP system; only the ...Async functions take it
const own = await abap.callGraphAsync(["CLAS:ZCL_ORDERS"], {fetch: async (ref) => client.source(ref)}, 2);
```

Every function has an `…Async` variant returning a promise; it runs on the libuv thread pool. The result types are in `types.d.ts`. `text` and `objectText` give a compact form instead: definitions written like ABAP, one line per call or database access, per procedure its effects with statements and conditions. See the [repository](https://github.com/albertlast/abapPaser) for what each result contains.
