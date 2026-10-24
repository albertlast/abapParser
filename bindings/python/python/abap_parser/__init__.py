"""ABAP parser: symbols and statements of ABAP source code.

The returned dictionaries follow the JSON model of the Rust core, see the
TypedDicts in ``abap_parser.types``.

Every function has an ``..._async`` variant for asyncio. It runs in a worker
thread; the native code releases the GIL, so loading sources from an SAP
system does not block the event loop and several calls run in parallel.
"""

import asyncio
import json
from typing import Any, Callable, List, Optional, Tuple, TypedDict

from . import _native
from .types import CdsDefinition, Call, CallGraph, DbAccess, FileLogic, FileSymbols, StatementInfo

__all__ = [
    "symbols",
    "db_access",
    "calls",
    "call_graph",
    "fetch_source",
    "logic",
    "logic_object",
    "statements",
    "version",
    "symbols_async",
    "db_access_async",
    "calls_async",
    "call_graph_async",
    "fetch_source_async",
    "logic_async",
    "logic_object_async",
    "statements_async",
    "AdtOptions",
    "SourceOptions",
    "FollowOptions",
    "CancelToken",
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


class FollowOptions(TypedDict, total=False):
    """Which objects calls are followed into; the objects given are always
    loaded. Those not loaded are listed in ``skipped``, calls into them are
    unresolved with the reason ``not_followed`` or ``limit``.

    ``include``: name patterns of objects to load, like "Z*", or "/ABC/*" for a custom namespace,
    all when empty; ``exclude``: name patterns not to load, also when
    ``include`` matches; ``limit``: at most this many objects are loaded or
    looked for."""

    include: List[str]
    exclude: List[str]
    limit: int


CancelToken = _native.CancelToken
"""Stops an analysis loading sources: ``token.cancel()``, also from another
thread or the event loop; nothing more is loaded and the analysis finishes
with what it has."""


class SourceOptions(TypedDict, total=False):
    """Where sources come from: local folders first, then the fetch function,
    then an SAP system.

    ``fetch`` gives sources of the host, e.g. from its own ADT client or cache:
    called with objects like "CLAS:ZCL_X", "CLAS:ZCL_X/implementations",
    "PROG:ZREPORT_TOP" or "FUNC:Z_FM", it returns the source, or None when it
    has none. Exceptions it raises are listed in ``errors``. It is called from
    worker threads, also several at once.

    ``follow`` limits which objects calls are followed into, see
    ``FollowOptions``. ``progress`` is called with ``{"loaded": n, "object":
    "CLAS:ZCL_X"}`` after each object loaded or looked for, from worker
    threads. ``cancel`` is a ``CancelToken``."""

    src: List[str]
    cache: str
    adt: AdtOptions
    follow: FollowOptions
    fetch: Callable[[str], Optional[str]]
    progress: Callable[[dict], Any]
    cancel: Any


def _split(options: SourceOptions) -> Tuple[str, Any, Any, Any]:
    """The options as JSON for the native module, and the fetch function, the
    progress function and the cancel token."""
    rest = dict(options)
    fetch = rest.pop("fetch", None)
    progress = rest.pop("progress", None)
    cancel = rest.pop("cancel", None)
    for name, f in (("fetch", fetch), ("progress", progress)):
        if f is not None and not callable(f):
            raise TypeError(f"options['{name}'] must be callable")
    if cancel is not None and not isinstance(cancel, CancelToken):
        raise TypeError("options['cancel'] must be a CancelToken")
    return json.dumps(rest), fetch, progress, cancel


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


def _with_depth(options: SourceOptions, depth: int) -> Tuple[str, int, Any, Any, Any]:
    text, fetch, progress, cancel = _split(options)
    return text, depth, fetch, progress, cancel


def call_graph(roots: List[str], options: SourceOptions, depth: int = 1) -> CallGraph:
    """Call graph across objects like "CLAS:ZCL_ORDERS", sources loaded from
    folders and/or an SAP system. Raises RuntimeError for invalid options."""
    return json.loads(_native.call_graph_json(list(roots), *_with_depth(options, depth)))


def logic(source: str) -> FileLogic:
    """Logic of the procedures of one ABAP source file: statements nested along
    the control structures with their conditions and guards, data read and
    written with the statements providing each value, control flow and effects."""
    return json.loads(_native.logic_json(source))


def logic_object(reference: str, options: SourceOptions, depth: int = 1) -> FileLogic:
    """Logic of all sources of an object like "PROG:ZREPORT": class parts,
    includes, function modules, with the global data of the program typed as
    such. Calls are followed; what called procedures write, attributes, global
    data and database tables, shows in the effects of the caller. Calls into
    other objects are followed `depth` hops. Raises RuntimeError for invalid
    options."""
    return json.loads(_native.logic_object_json(reference, *_with_depth(options, depth)))


def fetch_source(reference: str, options: SourceOptions) -> Optional[str]:
    """Source text of one object, None when it does not exist.
    Raises RuntimeError for provider errors like failed authorization."""
    return _native.fetch_source(reference, *_split(options))


def statements(source: str) -> List[StatementInfo]:
    """Statements of one ABAP source file, with kind, text and position."""
    return json.loads(_native.statements_json(source))


def cds(source: str) -> CdsDefinition:
    """A CDS data definition (DDL source): its sources, joins, associations,
    parameters and the fields each element is made of."""
    return json.loads(_native.cds_json(source))


def text(analysis: str, source: str) -> str:
    """Compact text of one ABAP source file, e.g. for a language model:
    "symbols" (definitions written like ABAP), "calls" and "db" (one line
    each), "logic" (per procedure its effects with the statements and
    conditions), "cds" for a CDS data definition. Rows tell where to find
    things. Raises ValueError for an unknown analysis."""
    return _native.text(analysis, source)


def object_text(analysis: str, objects: List[str], options: SourceOptions, depth: int = 1) -> str:
    """Compact text of objects like "CLAS:ZCL_ORDERS", sources loaded from
    folders and/or an SAP system: "symbols" of all sources of the first
    object, its "logic" with calls followed `depth` hops, or the call "graph"
    from all of them. Raises RuntimeError for invalid options or a missing
    object."""
    return _native.object_text(analysis, list(objects), *_with_depth(options, depth))


def version() -> str:
    """Version of the native module."""
    return _native.version()


async def symbols_async(source: str) -> FileSymbols:
    """:func:`symbols` in a worker thread."""
    return await asyncio.to_thread(symbols, source)


async def db_access_async(source: str) -> List[DbAccess]:
    """:func:`db_access` in a worker thread."""
    return await asyncio.to_thread(db_access, source)


async def calls_async(source: str) -> List[Call]:
    """:func:`calls` in a worker thread."""
    return await asyncio.to_thread(calls, source)


async def call_graph_async(roots: List[str], options: SourceOptions, depth: int = 1) -> CallGraph:
    """:func:`call_graph` in a worker thread."""
    return await asyncio.to_thread(call_graph, roots, options, depth)


async def fetch_source_async(reference: str, options: SourceOptions) -> Optional[str]:
    """:func:`fetch_source` in a worker thread."""
    return await asyncio.to_thread(fetch_source, reference, options)


async def logic_async(source: str) -> FileLogic:
    """:func:`logic` in a worker thread."""
    return await asyncio.to_thread(logic, source)


async def logic_object_async(reference: str, options: SourceOptions, depth: int = 1) -> FileLogic:
    """:func:`logic_object` in a worker thread."""
    return await asyncio.to_thread(logic_object, reference, options, depth)


async def statements_async(source: str) -> List[StatementInfo]:
    """:func:`statements` in a worker thread."""
    return await asyncio.to_thread(statements, source)


async def cds_async(source: str) -> CdsDefinition:
    """:func:`cds` in a worker thread."""
    return await asyncio.to_thread(cds, source)


async def text_async(analysis: str, source: str) -> str:
    """:func:`text` in a worker thread."""
    return await asyncio.to_thread(text, analysis, source)


async def object_text_async(analysis: str, objects: List[str], options: SourceOptions, depth: int = 1) -> str:
    """:func:`object_text` in a worker thread."""
    return await asyncio.to_thread(object_text, analysis, objects, options, depth)
