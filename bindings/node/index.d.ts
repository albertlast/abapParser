import type {Call, CallGraph, DbAccess, FileLogic, FileSymbols, StatementInfo} from "./types";

export * from "./types";

/** Classes, interfaces, routines and declarations of one ABAP source file. */
export function symbols(source: string): FileSymbols;

/** Database accesses of one ABAP source file: tables, fields, joins, conditions, targets. */
export function dbAccess(source: string): DbAccess[];

/** Where sources come from: local folders first, then an SAP system through ADT. */
export interface SourceOptions {
  /** folders with abapGit or plain .abap files */
  src?: string[];
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

/** Statements of one ABAP source file, with kind, text and position. */
export function statements(source: string): StatementInfo[];

/** Version of the native module. */
export function version(): string;
