# abap-parser

Parses ABAP code and tells what it does: classes, methods, forms and declarations; which database tables a statement reads or writes; which methods, function modules and forms it calls; and the logic of each procedure, from conditions and data flow to what it writes in the end, also through SUBMIT, ABAP memory and files. CDS views tell their data sources and the fields of each element. Sources come from a folder (abapGit layout or plain `.abap` files) or from an SAP system through the ADT REST API, SAP standard code and dictionary types included.

```sh
pip install abap-parser
```

Wheels for Windows x64, macOS x64 and arm64, Linux x64 and arm64 (manylinux), Python 3.9 and later. Other platforms build from the source distribution with Rust.

```python
import abap_parser

source = open("zcl_orders.clas.abap", encoding="utf-8").read()
for c in abap_parser.symbols(source)["classes"]:
    print(c["name"], [m["name"] for m in c["methods"]])
for a in abap_parser.db_access(source):
    print(a["operation"], [t["name"] for t in a["tables"]])
for p in abap_parser.logic(source)["procedures"]:
    print(p["block"]["name"], [(e["name"], e["inputs"]) for e in p["effects"]])

# whole objects, missing sources loaded from an SAP system
options = {
    "src": ["./src"],
    "cache": "./.abap-cache",
    "adt": {"url": "https://host:44300", "user": os.environ["ABAP_ADT_USER"],
            "password": os.environ["ABAP_ADT_PASSWORD"]},
}
graph = abap_parser.call_graph(["CLAS:ZCL_ORDERS"], options, depth=2)
logic = await abap_parser.logic_object_async("PROG:ZREPORT", options)
# compact text, e.g. for a language model
prompt = abap_parser.text("logic", source)
outline = await abap_parser.object_text_async("symbols", ["CLAS:ZCL_ORDERS"], options)
# a CDS view: sources, joins, associations, the fields of each element
view = abap_parser.cds(open("zi_orders.ddls.asddls", encoding="utf-8").read())
```

Every function has an `..._async` variant for asyncio; the native code releases the GIL. The results are dictionaries typed by the TypedDicts in `abap_parser.types`. `text` and `object_text` give a compact form instead: definitions written like ABAP, one line per call or database access, per procedure its effects with statements and conditions. See the [repository](https://github.com/albertlast/abapPaser) for what each result contains.
