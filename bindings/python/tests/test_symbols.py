import unittest
from pathlib import Path

import abap_parser

CORPUS = Path(__file__).resolve().parents[3] / "tests" / "corpus"

SOURCE = """
CLASS zcl_demo DEFINITION PUBLIC FINAL.
  PUBLIC SECTION.
    METHODS run IMPORTING iv_id TYPE i RETURNING VALUE(rv_ok) TYPE abap_bool.
  PRIVATE SECTION.
    DATA mt_rows TYPE STANDARD TABLE OF string WITH EMPTY KEY.
ENDCLASS.
CLASS zcl_demo IMPLEMENTATION.
  METHOD run.
    rv_ok = abap_true.
  ENDMETHOD.
ENDCLASS.
"""


class SymbolsTest(unittest.TestCase):
    def test_class(self):
        s = abap_parser.symbols(SOURCE)
        self.assertEqual(s["schema_version"], 1)
        c = s["classes"][0]
        self.assertEqual(c["name"], "zcl_demo")
        self.assertTrue(c["is_global"])
        self.assertEqual(
            [(p["name"], p["direction"]) for p in c["methods"][0]["parameters"]],
            [("iv_id", "importing"), ("rv_ok", "returning")],
        )
        self.assertEqual(c["attributes"][0]["typing"]["kind"], "table")
        self.assertEqual(c["attributes"][0]["visibility"], "private")
        self.assertEqual([m["name"] for m in c["implementation"]["methods"]], ["run"])

    def test_db_access(self):
        acc = abap_parser.db_access(
            "FORM read.\n"
            "  SELECT a~vbeln, b~posnr FROM vbak AS a INNER JOIN vbap AS b ON a~vbeln = b~vbeln\n"
            "    WHERE a~erdat >= @lv_date INTO TABLE @DATA(lt_items).\n"
            "ENDFORM."
        )
        self.assertEqual(len(acc), 1)
        a = acc[0]
        self.assertEqual(a["operation"], "select")
        self.assertEqual([(t["name"], t["role"]) for t in a["tables"]], [("vbak", "main"), ("vbap", "join")])
        self.assertEqual([c["table"] + "." + c["name"] for c in a["conditions"][0]["columns"]], ["vbak.erdat"])
        self.assertEqual(a["block"], {"kind": "form", "name": "read"})

    def test_calls_and_graph(self):
        c = abap_parser.calls("FORM a.\n  CALL FUNCTION 'Z_FM'.\n  PERFORM b.\nENDFORM.\nFORM b.\nENDFORM.")
        self.assertEqual(
            [(x["kind"], x["name"], x["resolved"]) for x in c],
            [("function", "Z_FM", False), ("form", "b", True)],
        )
        src = CORPUS / "abapgit" / "src"
        if src.exists():
            g = abap_parser.call_graph(["CLAS:ZCL_ABAPGIT_APACK_MIGRATION"], {"src": [str(src)]})
            self.assertGreater(len(g["calls"]), 10)
            text = abap_parser.fetch_source("CLAS:ZCL_ABAPGIT_APACK_MIGRATION", {"src": [str(src)]})
            self.assertIn("zcl_abapgit_apack_migration", text.lower())
            self.assertIsNone(abap_parser.fetch_source("CLAS:ZCL_NOPE", {"src": [str(src)]}))
        with self.assertRaises(RuntimeError):
            abap_parser.call_graph(["CLAS:X"], {})

    def test_statements(self):
        st = abap_parser.statements("DATA: a TYPE i, b TYPE c.\nWRITE a.")
        self.assertEqual([s["kind"] for s in st], ["Data", "Data", "Write"])
        self.assertEqual(st[1]["text"], "DATA b TYPE c.")
        self.assertEqual(st[2]["span"]["start"], {"row": 2, "col": 1})

    def test_logic(self):
        lg = abap_parser.logic("FORM f USING p TYPE i.\n  CHECK p > 0.\n  x = p * 2.\nENDFORM.")
        proc = lg["procedures"][0]
        self.assertEqual(proc["block"]["name"], "f")
        step = proc["body"][1]
        self.assertEqual([g["text"] for g in step["guards"]], ["p > 0"])
        self.assertEqual(step["reads"][0]["scope"], "parameter")
        self.assertEqual([(e["name"], e["inputs"]) for e in proc["effects"]], [("x", ["p"])])

    def test_logic_follows_calls(self):
        lg = abap_parser.logic(
            "FORM a.\n  PERFORM b.\nENDFORM.\nFORM b.\n  DELETE FROM zlog WHERE id = 1.\nENDFORM."
        )
        a = lg["procedures"][0]
        self.assertEqual(
            [(e["name"], e["scope"], [v["name"] for v in e["via"]]) for e in a["effects"]],
            [("ZLOG", "database", ["b"])],
        )

    @unittest.skipUnless((CORPUS / "abap-sdk-for-azure" / "src").exists(), "corpus submodule missing")
    def test_logic_object(self):
        src = str(CORPUS / "abap-sdk-for-azure" / "src")
        lg = abap_parser.logic_object("FUGR:ZREST", {"src": [src]})
        self.assertEqual(lg["object"], "FUGR:ZREST")
        form = next(p for p in lg["procedures"] if p["block"]["name"] == "set_log")
        self.assertEqual(form["unit"], "PROG:LZRESTF01")
        self.assertIn(("gwa_log", "global"), [(e["name"], e["scope"]) for e in form["effects"]])
        fm = next(p for p in lg["procedures"] if p["block"]["name"] == "zsave_rest_log")
        db = [e for e in fm["effects"] if e["scope"] == "database"]
        self.assertTrue(db and all(e["via"] for e in db))

    @unittest.skipUnless((CORPUS / "abapgit" / "src").exists(), "corpus submodule missing")
    def test_real_class(self):
        path = CORPUS / "abapgit" / "src" / "apack" / "zcl_abapgit_apack_migration.clas.abap"
        s = abap_parser.symbols(path.read_text(encoding="utf-8"))
        self.assertEqual(s["classes"][0]["name"].lower(), "zcl_abapgit_apack_migration")
        self.assertGreater(len(s["classes"][0]["methods"]), 5)


if __name__ == "__main__":
    unittest.main()


class AsyncTest(unittest.IsolatedAsyncioTestCase):
    async def test_same_results(self):
        self.assertEqual(await abap_parser.symbols_async(SOURCE), abap_parser.symbols(SOURCE))
        self.assertEqual(await abap_parser.logic_async(SOURCE), abap_parser.logic(SOURCE))
        self.assertEqual(await abap_parser.statements_async(SOURCE), abap_parser.statements(SOURCE))
        with self.assertRaises(RuntimeError):
            await abap_parser.call_graph_async(["CLAS:X"], {})

    @unittest.skipUnless((CORPUS / "abapgit" / "src").exists(), "corpus submodule missing")
    async def test_event_loop_keeps_running(self):
        import asyncio

        src = str(CORPUS / "abapgit" / "src")
        ticks = 0

        async def ticker():
            nonlocal ticks
            while True:
                await asyncio.sleep(0.001)
                ticks += 1

        task = asyncio.create_task(ticker())
        graph, text, logic = await asyncio.gather(
            abap_parser.call_graph_async(["CLAS:ZCL_ABAPGIT_REPO_ONLINE"], {"src": [src]}, 2),
            abap_parser.fetch_source_async("CLAS:ZCL_ABAPGIT_APACK_MIGRATION", {"src": [src]}),
            abap_parser.logic_object_async("CLAS:ZCL_ABAPGIT_REPO_ONLINE", {"src": [src]}),
        )
        task.cancel()
        self.assertGreater(len(graph["calls"]), 100)
        self.assertIn("zcl_abapgit_apack_migration", text.lower())
        self.assertGreater(len(logic["procedures"]), 10)
        self.assertGreater(ticks, 0)
