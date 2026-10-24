import type {CdsDefinition, Call, CallGraph, DbAccess, FileLogic, FileSymbols, StatementInfo} from "./types";

export * from "./types";

/** Classes, interfaces, routines and declarations of one ABAP source file. */
export function symbols(source: string): FileSymbols;

/** Database accesses of one ABAP source file: tables, fields, joins, conditions, targets. */
export function dbAccess(source: string): DbAccess[];

/**
 * Where sources come from: local folders first, then the fetch function,
 * then an SAP system through ADT.
 */
export interface SourceOptions {
  /** folders with abapGit or plain .abap files */
  src?: string[];
  /**
   * Sources of the host, e.g. from its own ADT client or cache: called with
   * objects like "CLAS:ZCL_X", "CLAS:ZCL_X/implementations", "PROG:ZREPORT_TOP",
   * "FUNC:Z_FM", it returns the source, or null or undefined when it has none.
   * A thrown error or rejected promise is listed in `errors`. Only the async
   * functions take it; the blocking ones throw a TypeError.
   */
  fetch?: (reference: string) => string | null | undefined | Promise<string | null | undefined>;
  /** cache folder for sources fetched from the SAP system */
  cache?: string;
  adt?: {
    /** e.g. https://host:44300 */
    url: string;
    user: string;
    password: string;
    /** sap-client, e.g. "100" */
    client?: string;
    language?: string;
    /** skip TLS certificate checks, test systems only */
    insecure?: boolean;
  };
}

/** Calls of one ABAP source file; types resolved within the file. */
export function calls(source: string): Call[];

/** Call graph across objects like "CLAS:ZCL_ORDERS"; blocks while loading sources. */
export function callGraph(roots: string[], options: SourceOptions, depth?: number): CallGraph;

/** Source text of one object, null when it does not exist; blocks while loading. */
export function fetchSource(reference: string, options: SourceOptions): string | null;

/**
 * Logic of the procedures of one ABAP source file: statements with conditions
 * and guards, data read and written, control flow and effects.
 */
export function logic(source: string): FileLogic;

/**
 * Logic of all sources of an object like "PROG:ZREPORT", with the global data
 * of its includes; calls into other objects followed `depth` hops (default 1).
 * Blocks while loading sources.
 */
export function logicObject(reference: string, options: SourceOptions, depth?: number): FileLogic;

/** Statements of one ABAP source file, with kind, text and position. */
export function statements(source: string): StatementInfo[];

/**
 * A CDS data definition (DDL source): its sources, joins, associations,
 * parameters and the fields each element is made of.
 */
export function cds(source: string): CdsDefinition;

/**
 * Compact text of one ABAP source file, e.g. for a language model: "symbols"
 * (definitions written like ABAP), "calls" and "db" (one line each), "logic"
 * (per procedure its effects with the statements and conditions); "cds" for a
 * CDS data definition. Rows tell where to find things.
 */
export function text(analysis: "symbols" | "calls" | "db" | "logic" | "cds", source: string): string;

/**
 * Compact text of objects like "CLAS:ZCL_ORDERS": "symbols" of all sources of
 * the first object, its "logic" with calls followed `depth` hops (default 1),
 * or the call "graph" from all of them. Blocks while loading sources.
 */
export function objectText(
  analysis: "symbols" | "logic" | "graph",
  objects: string[],
  options: SourceOptions,
  depth?: number,
): string;

/** Version of the native module. */
export function version(): string;

// The same functions on the libuv thread pool, returning promises: loading
// sources from an SAP system or analyzing large files does not block the
// event loop, and several calls run in parallel.

export function symbolsAsync(source: string): Promise<FileSymbols>;
export function dbAccessAsync(source: string): Promise<DbAccess[]>;
export function callsAsync(source: string): Promise<Call[]>;
export function callGraphAsync(roots: string[], options: SourceOptions, depth?: number): Promise<CallGraph>;
export function fetchSourceAsync(reference: string, options: SourceOptions): Promise<string | null>;
export function logicAsync(source: string): Promise<FileLogic>;
export function logicObjectAsync(reference: string, options: SourceOptions, depth?: number): Promise<FileLogic>;
export function statementsAsync(source: string): Promise<StatementInfo[]>;
export function cdsAsync(source: string): Promise<CdsDefinition>;
export function textAsync(analysis: "symbols" | "calls" | "db" | "logic" | "cds", source: string): Promise<string>;
export function objectTextAsync(
  analysis: "symbols" | "logic" | "graph",
  objects: string[],
  options: SourceOptions,
  depth?: number,
): Promise<string>;
