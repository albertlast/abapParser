"""ABAP parser: symbols and statements of ABAP source code.

The returned dictionaries follow the JSON model of the Rust core, see the
TypedDicts in ``abap_parser.types``.
"""

import json
from typing import List, Optional, TypedDict

from . import _native
from .types import Call, CallGraph, DbAccess, FileLogic, FileSymbols, StatementInfo

__all__ = [
    "symbols",
    "db_access",
    "calls",
    "call_graph",
    "fetch_source",
    "logic",
    "statements",
    "version",
    "AdtOptions",
    "SourceOptions",
    "Call",
    "CallGraph",
    "DbAccess",
    "FileLogic",
    "FileSymbols",
    "StatementInfo",
]


class _AdtRequired(TypedDict):
    url: str
    user: str
    password: str


class AdtOptions(_AdtRequired, total=False):
    """An SAP system reached through the ADT REST API."""

    client: str
    language: str
    insecure: bool


class SourceOptions(TypedDict, total=False):
    """Where sources come from: local folders first, then an SAP system."""

    src: List[str]
    cache: str
    adt: AdtOptions


def symbols(source: str) -> FileSymbols:
    """Classes, interfaces, routines and declarations of one ABAP source file."""
    return json.loads(_native.symbols_json(source))


def db_access(source: str) -> List[DbAccess]:
    """Database accesses of one ABAP source file: tables, fields, joins,
    conditions and targets of every SELECT, INSERT, UPDATE, MODIFY, DELETE."""
    return json.loads(_native.db_access_json(source))


def calls(source: str) -> List[Call]:
    """Calls of one ABAP source file: methods, constructors, function modules,
    forms, programs, transactions; types resolved within the file."""
    return json.loads(_native.calls_json(source))


def call_graph(roots: List[str], options: SourceOptions, depth: int = 1) -> CallGraph:
    """Call graph across objects like "CLAS:ZCL_ORDERS", sources loaded from
    folders and/or an SAP system. Raises RuntimeError for invalid options."""
    return json.loads(_native.call_graph_json(list(roots), json.dumps(options), depth))


def logic(source: str) -> FileLogic:
    """Logic of the procedures of one ABAP source file: statements nested along
    the control structures with their conditions and guards, data read and
    written with the statements providing each value, control flow and effects."""
    return json.loads(_native.logic_json(source))


def fetch_source(reference: str, options: SourceOptions) -> Optional[str]:
    """Source text of one object, None when it does not exist.
    Raises RuntimeError for provider errors like failed authorization."""
    return _native.fetch_source(reference, json.dumps(options))


def statements(source: str) -> List[StatementInfo]:
    """Statements of one ABAP source file, with kind, text and position."""
    return json.loads(_native.statements_json(source))


def version() -> str:
    """Version of the native module."""
    return _native.version()
