"use strict";

const fs = require("node:fs");
const path = require("node:path");

/** Platform package with the prebuilt native module, e.g. `abap-parser-linux-x64-gnu`. */
function platformPackage() {
  const abi = {linux: "-gnu", win32: "-msvc"}[process.platform] ?? "";
  return `abap-parser-${process.platform}-${process.arch}${abi}`;
}

/** A module built locally with `npm run build` wins over the prebuilt one. */
function loadNative() {
  const local = path.join(__dirname, "abap_parser.node");
  if (fs.existsSync(local)) {
    return require(local);
  }
  const name = platformPackage();
  try {
    return require(name);
  } catch (e) {
    throw new Error(
      `abap-parser: no native module for ${process.platform}-${process.arch}. ` +
        `The optional dependency ${name} is missing or failed to load; ` +
        `reinstall, or build from source with "npm run build" (needs Rust). ${e.message}`,
    );
  }
}

const native = loadNative();

/** The functions and the signal of the options, which do not go to the native module as JSON. */
const HOOKS = ["fetch", "progress", "signal"];

/**
 * The options as JSON for the native module, without the hooks.
 * @param {import("./index").SourceOptions} options
 */
function json(options) {
  if (options == null) {
    return JSON.stringify(options);
  }
  const rest = {...options};
  for (const key of HOOKS) {
    delete rest[key];
  }
  return JSON.stringify(rest);
}

/**
 * The hooks of the options for the native module: the fetch function, made
 * to always return a promise; the progress function, called with
 * `{loaded, object}`; and a cancel token the signal cancels.
 * @param {import("./index").SourceOptions} options
 */
function hooksOf(options) {
  const {fetch, progress, signal} = options ?? {};
  for (const [name, f] of [
    ["fetch", fetch],
    ["progress", progress],
  ]) {
    if (f != null && typeof f !== "function") {
      throw new TypeError(`abap-parser: options.${name} must be a function`);
    }
  }
  let token;
  if (signal != null) {
    token = new native.CancelToken();
    if (signal.aborted) {
      token.cancel();
    } else {
      signal.addEventListener("abort", () => token.cancel(), {once: true});
    }
  }
  return [
    fetch == null ? undefined : async (reference) => fetch(reference),
    progress == null ? undefined : (text) => progress(JSON.parse(text)),
    token,
  ];
}

/**
 * The blocking functions wait in the JavaScript thread, so the functions of
 * the host could never run and nothing could cancel: only the async ones
 * take them.
 * @param {import("./index").SourceOptions} options @param {string} name
 */
function withoutHooks(options, name) {
  for (const key of HOOKS) {
    if (options != null && options[key] != null) {
      throw new TypeError(`abap-parser: options.${key} needs ${name}Async, ${name} blocks the event loop`);
    }
  }
  return json(options);
}

/**
 * Classes, interfaces, routines and declarations of one ABAP source file.
 * @param {string} source ABAP source code
 * @returns {import("./types").FileSymbols}
 */
function symbols(source) {
  return JSON.parse(native.symbolsJson(source));
}

/**
 * Database accesses of one ABAP source file: tables, fields, joins,
 * conditions and targets of every SELECT, INSERT, UPDATE, MODIFY, DELETE.
 * @param {string} source ABAP source code
 * @returns {import("./types").DbAccess[]}
 */
function dbAccess(source) {
  return JSON.parse(native.dbAccessJson(source));
}

/**
 * Calls of one ABAP source file: methods, constructors, function modules,
 * forms, programs, transactions; types resolved within the file.
 * @param {string} source ABAP source code
 * @returns {import("./types").Call[]}
 */
function calls(source) {
  return JSON.parse(native.callsJson(source));
}

/**
 * Call graph across objects, sources loaded from folders and/or an SAP system.
 * Blocks while loading; see callGraphAsync.
 * @param {string[]} roots objects like "CLAS:ZCL_ORDERS", "PROG:ZREPORT", "FUNC:Z_FM"
 * @param {import("./index").SourceOptions} options where the sources are
 * @param {number} [depth=1] hops to follow from the roots
 * @returns {import("./types").CallGraph}
 */
function callGraph(roots, options, depth = 1) {
  return JSON.parse(native.callGraphJson(roots, withoutHooks(options, "callGraph"), depth));
}

/**
 * Source text of one object, or null when it does not exist. Blocks while
 * loading; see fetchSourceAsync.
 * @param {string} reference object like "CLAS:ZCL_ORDERS" or "CLAS:ZCL_ORDERS/implementations"
 * @param {import("./index").SourceOptions} options where the sources are
 * @returns {string | null}
 */
function fetchSource(reference, options) {
  return native.fetchSource(reference, withoutHooks(options, "fetchSource"));
}

/**
 * Logic of the procedures of one ABAP source file: statements nested along the
 * control structures with their conditions and guards, data read and written
 * with the statements providing each value, control flow and effects.
 * @param {string} source ABAP source code
 * @returns {import("./types").FileLogic}
 */
function logic(source) {
  return JSON.parse(native.logicJson(source));
}

/**
 * Logic of all sources of an object: class parts, includes, function modules,
 * with the global data of the program typed as such. Calls are followed: what
 * called procedures write, attributes, global data and database tables, shows
 * in the effects of the caller. Blocks while loading; see logicObjectAsync.
 * @param {string} reference object like "PROG:ZREPORT", "CLAS:ZCL_ORDERS", "FUGR:ZGROUP"
 * @param {import("./index").SourceOptions} options where the sources are
 * @param {number} [depth=1] hops into other objects to follow
 * @returns {import("./types").FileLogic}
 */
function logicObject(reference, options, depth = 1) {
  return JSON.parse(native.logicObjectJson(reference, withoutHooks(options, "logicObject"), depth));
}

/**
 * Statements of one ABAP source file, with kind, text and position.
 * @param {string} source ABAP source code
 * @returns {import("./types").StatementInfo[]}
 */
function statements(source) {
  return JSON.parse(native.statementsJson(source));
}

/**
 * A CDS data definition (DDL source): its sources, joins, associations,
 * parameters and the fields each element is made of.
 * @param {string} source DDL source
 * @returns {import("./types").CdsDefinition}
 */
function cds(source) {
  return JSON.parse(native.cdsJson(source));
}

/**
 * Compact text of one ABAP source file, e.g. for a language model:
 * "symbols" (definitions written like ABAP), "calls" and "db" (one line each),
 * "logic" (per procedure its effects with the statements and conditions);
 * "cds" for a CDS data definition. Rows tell where to find things.
 * @param {"symbols" | "calls" | "db" | "logic" | "cds"} analysis
 * @param {string} source ABAP source code
 * @returns {string}
 */
function text(analysis, source) {
  return native.text(analysis, source);
}

/**
 * Compact text of objects, sources loaded from folders and/or an SAP system:
 * "symbols" of all sources of the first object, its "logic" with calls
 * followed `depth` hops, or the call "graph" from all of them. Blocks while
 * loading; see objectTextAsync.
 * @param {"symbols" | "logic" | "graph"} analysis
 * @param {string[]} objects objects like "CLAS:ZCL_ORDERS"
 * @param {import("./index").SourceOptions} options where the sources are
 * @param {number} [depth=1] hops into other objects to follow
 * @returns {string}
 */
function objectText(analysis, objects, options, depth = 1) {
  return native.objectText(analysis, objects, withoutHooks(options, "objectText"), depth);
}

// The same on the libuv thread pool: promises, the event loop keeps running.

const parsed = (promise) => promise.then((json) => JSON.parse(json));

/** @param {string} source @returns {Promise<import("./types").FileSymbols>} */
const symbolsAsync = (source) => parsed(native.symbolsJsonAsync(source));
/** @param {string} source @returns {Promise<import("./types").DbAccess[]>} */
const dbAccessAsync = (source) => parsed(native.dbAccessJsonAsync(source));
/** @param {string} source @returns {Promise<import("./types").Call[]>} */
const callsAsync = (source) => parsed(native.callsJsonAsync(source));
/** @param {string} source @returns {Promise<import("./types").FileLogic>} */
const logicAsync = (source) => parsed(native.logicJsonAsync(source));
/** @param {string} source @returns {Promise<import("./types").CdsDefinition>} */
const cdsAsync = (source) => parsed(native.cdsJsonAsync(source));
/** @param {string} source @returns {Promise<import("./types").StatementInfo[]>} */
const statementsAsync = (source) => parsed(native.statementsJsonAsync(source));

/**
 * @param {string[]} roots @param {import("./index").SourceOptions} options @param {number} [depth=1]
 * @returns {Promise<import("./types").CallGraph>}
 */
const callGraphAsync = (roots, options, depth = 1) =>
  parsed(native.callGraphJsonAsync(roots, json(options), depth, ...hooksOf(options)));

/**
 * @param {string} reference @param {import("./index").SourceOptions} options
 * @returns {Promise<string | null>}
 */
const fetchSourceAsync = (reference, options) =>
  parsed(native.fetchSourceJsonAsync(reference, json(options), ...hooksOf(options)));

/**
 * @param {string} reference @param {import("./index").SourceOptions} options @param {number} [depth=1]
 * @returns {Promise<import("./types").FileLogic>}
 */
const logicObjectAsync = (reference, options, depth = 1) =>
  parsed(native.logicObjectJsonAsync(reference, json(options), depth, ...hooksOf(options)));

/** @param {string} analysis @param {string} source @returns {Promise<string>} */
const textAsync = (analysis, source) => native.textAsync(analysis, source);

/**
 * @param {string} analysis @param {string[]} objects
 * @param {import("./index").SourceOptions} options @param {number} [depth=1]
 * @returns {Promise<string>}
 */
const objectTextAsync = (analysis, objects, options, depth = 1) =>
  native.objectTextAsync(analysis, objects, json(options), depth, ...hooksOf(options));

module.exports = {
  symbols,
  dbAccess,
  calls,
  callGraph,
  fetchSource,
  logic,
  logicObject,
  statements,
  cds,
  text,
  objectText,
  symbolsAsync,
  dbAccessAsync,
  callsAsync,
  callGraphAsync,
  fetchSourceAsync,
  logicAsync,
  logicObjectAsync,
  statementsAsync,
  cdsAsync,
  textAsync,
  objectTextAsync,
  version: native.version,
};
