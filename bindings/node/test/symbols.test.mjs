import assert from "node:assert/strict";
import fs from "node:fs";
import {createRequire} from "node:module";
import path from "node:path";
import test from "node:test";
import {fileURLToPath} from "node:url";

const require = createRequire(import.meta.url);
const abap = require("../index.js");
const corpus = path.join(path.dirname(fileURLToPath(import.meta.url)), "..", "..", "..", "tests", "corpus");

const SOURCE = `
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
ENDCLASS.`;

test("symbols of a class", () => {
  const s = abap.symbols(SOURCE);
  assert.equal(s.schema_version, 1);
  const c = s.classes[0];
  assert.equal(c.name, "zcl_demo");
  assert.equal(c.is_global, true);
  assert.deepEqual(c.methods[0].parameters.map((p) => [p.name, p.direction]),
    [["iv_id", "importing"], ["rv_ok", "returning"]]);
  assert.equal(c.attributes[0].typing.kind, "table");
  assert.equal(c.attributes[0].visibility, "private");
  assert.deepEqual(c.implementation.methods.map((m) => m.name), ["run"]);
});

test("database access", () => {
  const acc = abap.dbAccess(`
FORM read.
  SELECT a~vbeln, b~posnr FROM vbak AS a INNER JOIN vbap AS b ON a~vbeln = b~vbeln
    WHERE a~erdat >= @lv_date INTO TABLE @DATA(lt_items).
ENDFORM.`);
  assert.equal(acc.length, 1);
  const a = acc[0];
  assert.equal(a.operation, "select");
  assert.deepEqual(a.tables.map((t) => [t.name, t.role]), [["vbak", "main"], ["vbap", "join"]]);
  assert.deepEqual(a.conditions[0].columns.map((c) => `${c.table}.${c.name}`), ["vbak.erdat"]);
  assert.deepEqual(a.block, {kind: "form", name: "read"});
});

test("calls and call graph", () => {
  const c = abap.calls("FORM a.\n  CALL FUNCTION 'Z_FM'.\n  PERFORM b.\nENDFORM.\nFORM b.\nENDFORM.");
  assert.deepEqual(c.map((x) => [x.kind, x.name, x.resolved]), [["function", "Z_FM", false], ["form", "b", true]]);
  const src = path.join(corpus, "abapgit", "src");
  if (fs.existsSync(src)) {
    const g = abap.callGraph(["CLAS:ZCL_ABAPGIT_APACK_MIGRATION"], {src: [src]});
    assert.ok(g.calls.length > 10);
    assert.ok(g.calls.some((x) => x.resolved));
    assert.match(abap.fetchSource("CLAS:ZCL_ABAPGIT_APACK_MIGRATION", {src: [src]}), /zcl_abapgit_apack_migration/i);
    assert.equal(abap.fetchSource("CLAS:ZCL_NOPE", {src: [src]}), null);
  }
  assert.throws(() => abap.callGraph(["CLAS:X"], {}), /no source/);
});

test("statements", () => {
  const st = abap.statements("DATA: a TYPE i, b TYPE c.\nWRITE a.");
  assert.deepEqual(st.map((s) => s.kind), ["Data", "Data", "Write"]);
  assert.equal(st[1].text, "DATA b TYPE c.");
  assert.deepEqual(st[2].span.start, {row: 2, col: 1});
});

test("logic", () => {
  const l = abap.logic("FORM f USING p TYPE i.\n  CHECK p > 0.\n  x = p * 2.\nENDFORM.");
  const [proc] = l.procedures;
  assert.equal(proc.block.name, "f");
  const step = proc.body[1];
  assert.deepEqual(step.guards.map((g) => g.text), ["p > 0"]);
  assert.equal(step.reads[0].scope, "parameter");
  assert.deepEqual(proc.effects.map((e) => [e.name, e.inputs]), [["x", ["p"]]]);
});

test("logic follows calls", () => {
  const l = abap.logic("FORM a.\n  PERFORM b.\nENDFORM.\nFORM b.\n  DELETE FROM zlog WHERE id = 1.\nENDFORM.");
  const a = l.procedures[0];
  assert.deepEqual(
    a.effects.map((e) => [e.name, e.scope, (e.via ?? []).map((v) => v.name)]),
    [["ZLOG", "database", ["b"]]],
  );
});

test("logic of an object", {skip: !fs.existsSync(path.join(corpus, "abap-sdk-for-azure", "src"))}, () => {
  const src = path.join(corpus, "abap-sdk-for-azure", "src");
  const l = abap.logicObject("FUGR:ZREST", {src: [src]});
  assert.equal(l.object, "FUGR:ZREST");
  const form = l.procedures.find((p) => p.block.name === "set_log");
  assert.equal(form.unit, "PROG:LZRESTF01");
  assert.ok(form.effects.some((e) => e.name === "gwa_log" && e.scope === "global"));
  const fm = l.procedures.find((p) => p.block.name === "zsave_rest_log");
  const db = fm.effects.filter((e) => e.scope === "database");
  assert.ok(db.length > 0 && db.every((e) => e.via?.length));
});

test("a real abapGit class", {skip: !fs.existsSync(path.join(corpus, "abapgit", "src"))}, () => {
  const file = path.join(corpus, "abapgit", "src", "apack", "zcl_abapgit_apack_migration.clas.abap");
  const s = abap.symbols(fs.readFileSync(file, "utf8"));
  assert.equal(s.classes[0].name.toLowerCase(), "zcl_abapgit_apack_migration");
  assert.ok(s.classes[0].methods.length > 5);
});

test("async variants give the same results", async () => {
  assert.deepEqual(await abap.symbolsAsync(SOURCE), abap.symbols(SOURCE));
  assert.deepEqual(await abap.logicAsync(SOURCE), abap.logic(SOURCE));
  assert.deepEqual(await abap.statementsAsync(SOURCE), abap.statements(SOURCE));
  assert.deepEqual(await abap.callsAsync(SOURCE), abap.calls(SOURCE));
  assert.deepEqual(await abap.dbAccessAsync(SOURCE), abap.dbAccess(SOURCE));
  await assert.rejects(abap.callGraphAsync(["CLAS:X"], {}), /no source/);
  await assert.rejects(abap.logicObjectAsync("NOPE", {}), /KIND:NAME/);
});

test("loading objects does not block the event loop", {skip: !fs.existsSync(path.join(corpus, "abapgit", "src"))}, async () => {
  const src = path.join(corpus, "abapgit", "src");
  let ticks = 0;
  const timer = setInterval(() => ticks++, 1);
  const [graph, text, missing, l] = await Promise.all([
    abap.callGraphAsync(["CLAS:ZCL_ABAPGIT_REPO_ONLINE"], {src: [src]}, 2),
    abap.fetchSourceAsync("CLAS:ZCL_ABAPGIT_APACK_MIGRATION", {src: [src]}),
    abap.fetchSourceAsync("CLAS:ZCL_NOPE", {src: [src]}),
    abap.logicObjectAsync("CLAS:ZCL_ABAPGIT_REPO_ONLINE", {src: [src]}),
  ]);
  clearInterval(timer);
  assert.ok(graph.calls.length > 100);
  assert.ok(text.toLowerCase().includes("zcl_abapgit_apack_migration"));
  assert.equal(missing, null);
  assert.ok(l.procedures.length > 10);
  // the timer ran while the native work was going on
  assert.ok(ticks > 0, `${ticks} ticks`);
});
