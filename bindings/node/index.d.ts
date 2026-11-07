import type {CdsDefinition, Call, CallGraph, DbAccess, FileLogic, FileSymbols, StatementInfo} from "./types";

export * from "./types";

/** Classes, interfaces, routines and declarations of one ABAP source file. */
export function symbols(source: string): FileSymbols;

/**
 * Database accesses of one ABAP source file: tables, fields, joins,
 * conditions, targets of its database statements. Tables read without SQL
 * (logical database, HR_READ_INFOTYPE, PNP macros), those of called
 * procedures and the data flow are in `logic`; a lineage needs both.
 */
export function dbAccess(source: string): DbAccess[];

/**
 * Where sources come from: local folders first, then the fetch function,
 * then an SAP system through ADT.
 */
export interface SourceOptions {
  /**
   * Folders with files named as abapGit names them: `zcl_x.clas.abap`,
   * `zcl_x.clas.locals_imp.abap`, `zgrp.fugr.z_fm.abap` (a function module in
   * its group), `#abc#name` for a namespace `/ABC/`, `name.tabl.xml`,
   * `name.dtel.xml`, `name.ttyp.xml` and so on. A plain `name.abap` is taken
   * as the program or include `name`, never as a function module or class.
   */
  src?: string[];
  /**
   * Sources of the host, e.g. from its own ADT client or cache. Called with
   * a reference `KIND:NAME`, it returns the text, or null or undefined when
   * it has none; a Buffer or Uint8Array is decoded as UTF-8. Each reference
   * is asked once, up to 8 at a time: the objects of a level, and the
   * dictionary types their declarations name, load several at once. Only
   * the async functions take it; the blocking ones throw a TypeError.
   *
   * A thrown error or rejected promise with the HTTP status 404 or 410
   * counts as not found; others are listed in `errors`, with the code
   * `auth` or `http` for an HTTP status (`status`, `statusCode`,
   * `response.status`), `network` for a code like `ECONNREFUSED`,
   * `ETIMEDOUT`, `ENOTFOUND` or `UND_ERR_...` (`code`, `cause.code`) and
   * the name `TimeoutError`, `io` for other `E...` codes like `ENOENT`, or
   * as the code `"network"` or `"io"` says. An answer that is no text
   * counts as not found, and a note says so. What to return, by kind:
   *
   * - `PROG:ZREPORT` (also includes, like `PROG:ZREPORT_TOP`), `INTF:ZIF_X`,
   *   `CLAS:ZCL_X`, `CLAS:ZCL_X/definitions` (`/implementations`, `/macros`,
   *   `/testclasses`: the local parts), `FUGR:ZGRP` (the main program
   *   SAPLZGRP), `FUNC:Z_FM` (`FUNCTION z_fm ... ENDFUNCTION.` as ADT gives
   *   it), `TYPE:ZTPOOL` (the type group), `DDLS:ZI_X` (the CDS source):
   *   ABAP or DDL source as in ADT's `source/main`
   * - `TABL:ZSTRUCT`, a structure or table: the DDL source ADT gives
   *   (`define structure ...`), or the abapGit `.tabl.xml` (`DD03P` rows)
   * - `DTEL:ZDTEL`: ADT's data element XML (`dtel:dataType`,
   *   `dtel:dataTypeLength`), or abapGit's `.dtel.xml`, `<DD04V>` with
   *   `DATATYPE`/`LENG`/`DECIMALS`, or `DOMNAME` and a `DOMA:` lookup
   * - `DOMA:ZDOMA`: abapGit's `.doma.xml`, `<DD01V>` with `DATATYPE`, `LENG`
   * - `TTYP:ZTT`: ADT's table type XML, or abapGit's `<DD40V>` with
   *   `ROWTYPE` and `ROWKIND` (`S` structure, `E` data element, `L` table
   *   type, `R` reference), or `DATATYPE`, `LENG` and `DECIMALS` for a
   *   built-in line type, e.g.
   *   `<DD40V><ROWTYPE>LVC_S_FCAT</ROWTYPE><ROWKIND>S</ROWKIND></DD40V>` or
   *   `<DD40V><DATATYPE>STRG</DATATYPE></DD40V>`; leave out empty values.
   *   DD40L gives these columns.
   * - `ENHO:ZENH`: the code of an enhancement implementation as ADT's
   *   `enhancements/enhoxhh/zenh/source/main` gives it, asked for the hooks
   *   `query` finds
   *
   * The dictionary kinds are asked while typing, whatever the patterns of
   * `follow` say; `follow.dictionary` bounds them. A text that cannot be
   * read leaves the type a name, and a note counts them.
   */
  fetch?: (
    reference: string,
  ) => string | Uint8Array | null | undefined | Promise<string | Uint8Array | null | undefined>;
  /**
   * Answers the lookups an SAP system answers from its tables: BAdIs,
   * classic BAdIs, enhancements (their code through `fetch` as `ENHO:`),
   * implementations of interfaces and subclasses. Called with a query on
   * one table, it returns the rows, each an array of values in the order
   * of `columns` or an object by column, or null or undefined when it
   * cannot answer, which a note says. The tables: BADI_MAIN, BADI_IMPL,
   * SXS_INTER, SXS_ATTR, SXC_CLASS, SXC_ATTR, SEOMETAREL, TFDIR and
   * ENHINCINX; `sql` suits ADT's data preview. Errors are reported as for
   * `fetch`. Without it, these lookups come only from `src` and `adt`. Only
   * the async functions take it.
   */
  query?: (query: TableQuery) => TableRows | null | undefined | Promise<TableRows | null | undefined>;
  /**
   * Which objects calls are followed into; the objects given are always
   * loaded. Those not loaded are listed in `skipped` with the reason, calls
   * into them are unresolved with the reason `not_followed` or `limit`. The
   * patterns match object names, not kinds, and apply to the objects of
   * calls and types (programs, classes, interfaces, function modules, type
   * groups), not to dictionary lookups (`TABL:`, `DTEL:`, `DOMA:`, `TTYP:`).
   */
  follow?: {
    /** name patterns of objects to load, like "Z*", or "/ABC/*" for a custom namespace; all when empty */
    include?: string[];
    /** name patterns of objects not to load, also when `include` matches */
    exclude?: string[];
    /**
     * At most this many objects are looked for, after the patterns; those
     * no source has count too, like a structure name tried as a class. They
     * are loaded level by level from the roots, so the nearest come first.
     * The note `limit: ...` says how many were found and how many more were
     * not loaded.
     */
    limit?: number;
    /**
     * At most this many dictionary lookups while typing (`TABL:`, `DTEL:`,
     * `DOMA:`, `TTYP:`), each name asked once; 0 for none. Types not looked
     * up stay names; the note `dictionary: ...` says how many were not made.
     */
    dictionary?: number;
  };
  /**
   * Called after each object loaded or looked for, without waiting for it.
   * Only the async functions take it.
   */
  progress?: (progress: Progress) => void;
  /**
   * Once aborted, nothing more is loaded and the analysis finishes with what
   * it has; calls into objects not loaded are unresolved with the reason
   * `cancelled`. Logic analyzes no more procedures: those not analyzed are
   * left out, and the note `cancelled: n procedures were not analyzed`
   * counts them. Only the async functions take it.
   */
  signal?: AbortSignal;
  /**
   * cache folder for sources fetched through `adt`, and which objects it
   * does not have; what `fetch` and `query` return is not cached
   */
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
    /**
     * A proxy URL like "http://proxy:8080" for every host, or false to
     * connect directly. Absent or true: the proxy of ALL_PROXY, HTTPS_PROXY
     * or HTTP_PROXY, except for the hosts in NO_PROXY. Errors without an
     * answer name the proxy they went through, without its credentials.
     */
    proxy?: string | boolean;
  };
}

/** A query on one table, for the `query` function of the host. */
export interface TableQuery {
  /** like "BADI_IMPL" */
  table: string;
  /** like ["CLASS_NAME"] */
  columns: string[];
  /** each column has one of its values, like {"BADI_NAME": ["ZBADI"]} */
  where: Record<string, string[]>;
  /** the same in ABAP SQL, like "SELECT class_name FROM badi_impl WHERE badi_name = 'ZBADI'" */
  sql: string;
}

/** Rows of a table query: values in the order of the columns, or by column name. */
export type TableRows = Array<
  Array<string | number | boolean | null> | Record<string, string | number | boolean | null>
>;

/** How far loading is, after each object loaded or looked for. */
export interface Progress {
  /** objects loaded or looked for so far */
  loaded: number;
  /**
   * work known to come: objects of the current level, or calls whose
   * targets are looked for, not started yet, and objects loading; what is
   * found may add more
   */
  queued: number;
  /** like "CLAS:ZCL_X" */
  object: string;
  /** like "CLAS" */
  kind: string;
  /** "cached" when read from `cache`, "missing" when no source has it, "error" when fetching failed */
  outcome: "loaded" | "cached" | "missing" | "error";
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
