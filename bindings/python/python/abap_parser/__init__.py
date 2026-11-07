"""ABAP parser: symbols and statements of ABAP source code.

The returned dictionaries follow the JSON model of the Rust core, see the
TypedDicts in ``abap_parser.types``.

Every function has an ``..._async`` variant for asyncio. It runs in a worker
thread; the native code releases the GIL, so loading sources from an SAP
system does not block the event loop and several calls run in parallel.
"""

import asyncio
import errno
import json
from typing import Any, Callable, List, Optional, Tuple, TypedDict, Union

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
    """An SAP system reached through the ADT REST API.

    ``proxy``: a proxy URL like "http://proxy:8080" for every host, or False
    to connect directly; absent or True: the proxy of ``ALL_PROXY``,
    ``HTTPS_PROXY`` or ``HTTP_PROXY``, except for the hosts in ``NO_PROXY``.
    Errors without an answer name the proxy they went through, without its
    credentials."""

    client: str
    language: str
    insecure: bool
    proxy: Union[str, bool]


class FollowOptions(TypedDict, total=False):
    """Which objects calls are followed into; the objects given are always
    loaded. Those not loaded are listed in ``skipped`` with the reason, calls
    into them are unresolved with the reason ``not_followed`` or ``limit``.
    The patterns match object names, not kinds, and apply to the objects of
    calls and types (programs, classes, interfaces, function modules, type
    groups), not to dictionary lookups (``TABL:``, ``DTEL:``, ``DOMA:``,
    ``TTYP:``).

    ``include``: name patterns of objects to load, like "Z*", or "/ABC/*" for a custom namespace,
    all when empty; ``exclude``: name patterns not to load, also when
    ``include`` matches; ``limit``: at most this many objects are looked for,
    after the patterns, those no source has included (like a structure name
    tried as a class), level by level from the roots; the note ``limit: ...``
    says how many were found and how many more were not loaded;
    ``dictionary``: at most this many dictionary lookups while typing, each
    name asked once, 0 for none; types not looked up stay names, the note
    ``dictionary: ...`` says how many were not made."""

    include: List[str]
    exclude: List[str]
    limit: int
    dictionary: int


CancelToken = _native.CancelToken
"""Stops an analysis loading sources: ``token.cancel()``, also from another
thread or the event loop; nothing more is loaded and the analysis finishes
with what it has. Logic analyzes no more procedures: those not analyzed are
left out, and the note ``cancelled: n procedures were not analyzed`` counts
them."""


class SourceOptions(TypedDict, total=False):
    """Where sources come from: local folders first, then the fetch function,
    then an SAP system.

    ``src`` are folders with files named as abapGit names them
    (``zcl_x.clas.abap``, ``zgrp.fugr.z_fm.abap``, ``name.ttyp.xml`` ...); a
    plain ``name.abap`` is taken as the program or include ``name``.

    ``fetch`` gives sources of the host, e.g. from its own ADT client or cache:
    called with a reference ``KIND:NAME``, it returns the text, or None when
    it has none. Exceptions it raises are listed in ``errors``. It is called
    from worker threads, also several at once. By kind: ``PROG``, ``INTF``,
    ``CLAS`` (and ``CLAS:ZCL_X/definitions``, ``/implementations``,
    ``/macros``, ``/testclasses``), ``FUGR``, ``FUNC``, ``TYPE`` and ``DDLS``
    the source as ADT's ``source/main`` gives it; ``TABL`` the DDL source of
    ADT or abapGit's ``.tabl.xml``; ``DTEL`` the data element XML of ADT or
    abapGit's ``<DD04V>``; ``DOMA`` abapGit's ``<DD01V>``; ``TTYP`` the table
    type XML of ADT or abapGit's ``<DD40V>`` with ``ROWTYPE`` and ``ROWKIND``
    (``S``, ``E``, ``L``, ``R``) or ``DATATYPE``, ``LENG`` and ``DECIMALS``,
    like ``<DD40V><ROWTYPE>LVC_S_FCAT</ROWTYPE><ROWKIND>S</ROWKIND></DD40V>``,
    empty values left out; ``ENHO`` the code of an enhancement implementation
    as ADT's ``enhancements/enhoxhh/<name>/source/main`` gives it, asked for
    the hooks ``query`` finds. The dictionary kinds are asked while typing,
    whatever the patterns of ``follow`` say; ``follow["dictionary"]`` bounds
    them. Bytes are decoded as UTF-8; an answer that is no text counts as not
    found and a note says so, as does a dictionary text that cannot be read.
    An exception counts as not found when it carries the HTTP status 404 or
    410; otherwise it is listed in ``errors``: code ``auth`` or ``http`` for
    an HTTP status (``status``, ``status_code``, an int ``code``,
    ``response.status_code``), ``network`` for ``ConnectionError``,
    ``TimeoutError`` and the connection errors of requests and urllib3,
    ``io`` for other ``OSError``, or as a string attribute ``code`` says
    (``"network"``, ``"io"``, ``"ECONNRESET"`` ...).

    ``query`` answers the lookups an SAP system answers from its tables:
    BAdIs, classic BAdIs, enhancements, implementations of interfaces and
    subclasses. It is called with a query on one table, ``{"table":
    "BADI_IMPL", "columns": ["CLASS_NAME"], "where": {"BADI_NAME":
    ["ZBADI"]}, "sql": "SELECT class_name FROM badi_impl WHERE badi_name =
    'ZBADI'"}`` (each column of ``where`` has one of its values), and returns
    the rows, each a list of values in the order of ``columns`` or a dict by
    column, or None when it cannot answer, which a note says. The tables:
    BADI_MAIN, BADI_IMPL, SXS_INTER, SXS_ATTR, SXC_CLASS, SXC_ATTR,
    SEOMETAREL, TFDIR and ENHINCINX; ``sql`` suits ADT's data preview.
    Without it these lookups come only from ``src`` and ``adt``. ``cache``
    holds what ``adt`` fetched, and which objects it does not have, not what
    ``fetch`` or ``query`` return.

    ``follow`` limits which objects calls are followed into, see
    ``FollowOptions``. ``progress`` is called with ``{"loaded": n, "queued":
    n, "object": "CLAS:ZCL_X", "kind": "CLAS", "outcome": "loaded"}`` after
    each object loaded or looked for, from worker threads; ``queued`` counts
    the work known to come (objects of the current level, or calls whose
    targets are looked for, not started yet, and objects loading; what is
    found may add more), ``outcome`` is ``loaded``, ``cached`` (from
    ``cache``), ``missing`` or ``error``.
    ``cancel`` is a ``CancelToken``."""

    src: List[str]
    cache: str
    adt: AdtOptions
    follow: FollowOptions
    fetch: Callable[[str], Optional[str]]
    query: Callable[[dict], Optional[List[Any]]]
    progress: Callable[[dict], Any]
    cancel: Any


_NETWORK = {
    "ConnectionError",
    "ConnectTimeout",
    "ConnectTimeoutError",
    "IncompleteRead",
    "NewConnectionError",
    "ProtocolError",
    "ProxyError",
    "ReadTimeout",
    "ReadTimeoutError",
    "RemoteDisconnected",
    "Timeout",
    "URLError",
}


def _failure(e: BaseException) -> dict:
    """What an exception of a function of the host tells: its message, an
    HTTP status and a code for the kind of error."""
    response = getattr(e, "response", None)
    status = None
    for v in (
        getattr(e, "status", None),
        getattr(e, "status_code", None),
        getattr(e, "code", None),
        getattr(response, "status_code", None),
        getattr(response, "status", None),
    ):
        if isinstance(v, int) and not isinstance(v, bool) and 100 <= v <= 599:
            status = v
            break
    code = getattr(e, "code", None)
    if not isinstance(code, str):
        code = None
        if isinstance(e, (ConnectionError, TimeoutError)) or any(
            c.__name__ in _NETWORK for c in type(e).__mro__
        ):
            code = "network"
        elif isinstance(e, OSError):
            code = errno.errorcode.get(e.errno or 0, "io")
    return {"message": str(e) or type(e).__name__, "status": status, "code": code}


def _host(f: Optional[Callable[[Any], Any]], decode: Callable[[str], Any], encode: Callable[[Any], Any]):
    """A function of the host for the native module: it gets the argument as
    a string and answers in JSON, ``{"value": ...}`` or ``{"error": ...}``."""
    if f is None:
        return None

    def call(argument: str) -> str:
        try:
            return json.dumps({"value": encode(f(decode(argument)))})
        except Exception as e:  # reported to the analysis, not raised
            return json.dumps({"error": _failure(e)})

    return call


def _text(value: Any) -> Any:
    """Text as the native module takes it: bytes decoded as UTF-8; anything
    else that is no text is noted by the native module."""
    if isinstance(value, (bytes, bytearray)):
        return bytes(value).decode("utf-8")
    if value is None or isinstance(value, (str, int, float, bool, list, dict)):
        return value
    return repr(value)


def _split(options: SourceOptions) -> Tuple[str, Any, Any, Any, Any]:
    """The options as JSON for the native module, the fetch and query
    functions answering in JSON, the progress function and the cancel
    token."""
    rest = dict(options)
    fetch = rest.pop("fetch", None)
    query = rest.pop("query", None)
    progress = rest.pop("progress", None)
    cancel = rest.pop("cancel", None)
    for name, f in (("fetch", fetch), ("query", query), ("progress", progress)):
        if f is not None and not callable(f):
            raise TypeError(f"options['{name}'] must be callable")
    if cancel is not None and not isinstance(cancel, CancelToken):
        raise TypeError("options['cancel'] must be a CancelToken")
    return (
        json.dumps(rest),
        _host(fetch, lambda reference: reference, _text),
        _host(query, json.loads, lambda rows: rows),
        progress,
        cancel,
    )


def symbols(source: str) -> FileSymbols:
    """Classes, interfaces, routines and declarations of one ABAP source file."""
    return json.loads(_native.symbols_json(source))


def db_access(source: str) -> List[DbAccess]:
    """Database accesses of one ABAP source file: tables, fields, joins,
    conditions and targets of every SELECT, INSERT, UPDATE, MODIFY, DELETE.
    Tables read without SQL (logical database, HR_READ_INFOTYPE, PNP
    macros), those of called procedures and the data flow are in
    :func:`logic`; a lineage needs both."""
    return json.loads(_native.db_access_json(source))


def calls(source: str) -> List[Call]:
    """Calls of one ABAP source file: methods, constructors, function modules,
    forms, programs, transactions; types resolved within the file."""
    return json.loads(_native.calls_json(source))


def _with_depth(options: SourceOptions, depth: int) -> Tuple[str, int, Any, Any, Any, Any]:
    text, fetch, query, progress, cancel = _split(options)
    return text, depth, fetch, query, progress, cancel


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
