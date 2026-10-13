//! MCP server: the analyses as tools for language models, over stdio.
//!
//! JSON-RPC 2.0, one message per line. Objects are loaded through the
//! source options of the command line (folders, SAP system, cache) and kept
//! for the session, so later questions about the same objects are fast.

use std::io::{BufRead, Write};

use abap_analysis::abap_source::{ObjectKind, SourceRef};
use abap_analysis::{text, Workspace};
use abap_model as m;
use serde_json::{json, Value};

/// Protocol versions this server speaks, newest first.
const PROTOCOL_VERSIONS: &[&str] = &["2025-06-18", "2025-03-26", "2024-11-05"];

/// Longest tool answer in characters; longer ones are cut, with a hint how
/// to ask for less. Keeps answers within what a model can take in.
const MAX_ANSWER: usize = 80_000;

const INSTRUCTIONS: &str = "Static analysis of ABAP code. Objects are named \
KIND:NAME: CLAS:ZCL_ORDERS, INTF:ZIF_X, PROG:ZREPORT (also includes), \
FUNC:Z_FM, FUGR:ZGROUP, TABL:BAPIRET2 (dictionary structures and tables), \
DTEL:XUBNAME. Sources come from local folders and/or an SAP system and \
include SAP standard code. Start with get_symbols or get_logic for an \
object; follow calls with get_call_graph; read code with get_source using \
the rows the other tools report.";

pub struct Server {
    workspace: Option<Workspace>,
    /// why there is no workspace
    no_workspace: String,
    sources: String,
}

impl Server {
    pub fn new(
        provider: Result<Box<dyn abap_analysis::abap_source::SourceProvider>, String>,
    ) -> Self {
        match provider {
            Ok(p) => Server {
                sources: p.describe(),
                workspace: Some(Workspace::new(p)),
                no_workspace: String::new(),
            },
            Err(e) => Server {
                sources: "none".to_string(),
                workspace: None,
                no_workspace: e,
            },
        }
    }

    /// Serves requests from `input` until it ends.
    pub fn run(&self, input: impl BufRead, mut output: impl Write) -> std::io::Result<()> {
        for line in input.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            let response = match serde_json::from_str::<Value>(&line) {
                Ok(message) => self.handle(&message),
                Err(e) => Some(error(Value::Null, -32700, &format!("parse error: {e}"))),
            };
            if let Some(r) = response {
                writeln!(output, "{r}")?;
                output.flush()?;
            }
        }
        Ok(())
    }

    /// The response to one message; `None` for notifications.
    pub fn handle(&self, message: &Value) -> Option<Value> {
        let id = message.get("id").cloned()?;
        let method = message.get("method").and_then(Value::as_str).unwrap_or("");
        let params = message.get("params").cloned().unwrap_or(json!({}));
        let result = match method {
            "initialize" => {
                let asked = params.get("protocolVersion").and_then(Value::as_str);
                let version = asked
                    .filter(|v| PROTOCOL_VERSIONS.contains(v))
                    .unwrap_or(PROTOCOL_VERSIONS[0]);
                json!({
                    "protocolVersion": version,
                    "capabilities": {"tools": {}},
                    "serverInfo": {
                        "name": "abap-parser",
                        "title": "ABAP parser",
                        "version": env!("CARGO_PKG_VERSION"),
                    },
                    "instructions": format!("{INSTRUCTIONS} Sources: {}.", self.sources),
                })
            }
            "ping" => json!({}),
            "tools/list" => json!({"tools": tools()}),
            "tools/call" => {
                let name = params.get("name").and_then(Value::as_str).unwrap_or("");
                let args = params.get("arguments").cloned().unwrap_or(json!({}));
                match self.call(name, &args) {
                    Ok(text) => tool_result(&text, false),
                    Err(Failure::Tool(e)) => tool_result(&e, true),
                    Err(Failure::Unknown) => {
                        return Some(error(id, -32602, &format!("unknown tool {name}")))
                    }
                }
            }
            _ => return Some(error(id, -32601, &format!("method not found: {method}"))),
        };
        Some(json!({"jsonrpc": "2.0", "id": id, "result": result}))
    }

    fn call(&self, name: &str, args: &Value) -> Result<String, Failure> {
        match name {
            "get_source" => self.source(args),
            "get_symbols" => {
                let object = self.object(args)?;
                Ok(match Detail::of(args) {
                    Detail::Full => compact(&object.symbols),
                    Detail::Summary => cut_text(text::symbols(&object.symbols)),
                })
            }
            "get_call_graph" => self.call_graph(args),
            "get_logic" => self.logic(args),
            "get_db_access" => {
                let object = self.object(args)?;
                let units: Vec<(String, Vec<m::DbAccess>)> = object
                    .units
                    .iter()
                    .map(|u| {
                        (
                            u.reference.to_string(),
                            abap_analysis::db_access_of(&u.file),
                        )
                    })
                    .filter(|(_, a)| !a.is_empty())
                    .collect();
                if Detail::of(args) == Detail::Full {
                    let units: Vec<Value> = units
                        .iter()
                        .map(|(u, a)| json!({"unit": u, "accesses": a}))
                        .collect();
                    return Ok(compact(&units));
                }
                let mut out = String::new();
                for (unit, accesses) in &units {
                    out.push_str(&format!("{unit}\n{}", text::db_access(accesses)));
                }
                if out.is_empty() {
                    out.push_str("no database access");
                }
                Ok(cut_text(out))
            }
            "find_implementations" => {
                let interface = string(args, "interface")?;
                let classes = self.workspace()?.implementations(&interface);
                Ok(compact(&classes))
            }
            "analyze_source" => {
                let source = string(args, "source")?;
                let analysis = args
                    .get("analysis")
                    .and_then(Value::as_str)
                    .unwrap_or("symbols");
                let full = Detail::of(args) == Detail::Full;
                Ok(match analysis {
                    "symbols" if full => compact(&abap_analysis::symbols(&source)),
                    "symbols" => cut_text(text::symbols(&abap_analysis::symbols(&source))),
                    "calls" if full => compact(&abap_analysis::calls(&source)),
                    "calls" => cut_text(text::calls(&abap_analysis::calls(&source))),
                    "db" if full => compact(&abap_analysis::db_access(&source)),
                    "db" => cut_text(text::db_access(&abap_analysis::db_access(&source))),
                    "logic" => {
                        let mut logic = abap_analysis::logic(&source);
                        filter_procedures(&mut logic, args);
                        logic_answer(&logic, Detail::of(args))
                    }
                    "statements" => compact(&abap_analysis::statements(&source)),
                    other => return Err(Failure::Tool(format!("unknown analysis {other}"))),
                })
            }
            _ => Err(Failure::Unknown),
        }
    }

    fn workspace(&self) -> Result<&Workspace, Failure> {
        self.workspace.as_ref().ok_or_else(|| {
            Failure::Tool(format!(
                "no sources configured ({}); start the server with --src folders and/or \
                 an SAP system (--adt-url, ABAP_ADT_USER, ABAP_ADT_PASSWORD), or use analyze_source",
                self.no_workspace
            ))
        })
    }

    fn reference(&self, args: &Value) -> Result<SourceRef, Failure> {
        string(args, "object")?.parse().map_err(Failure::Tool)
    }

    fn object(&self, args: &Value) -> Result<std::sync::Arc<abap_analysis::Object>, Failure> {
        let r = self.reference(args)?;
        self.workspace()?
            .object(&r)
            .ok_or_else(|| self.not_found(&r))
    }

    fn not_found(&self, r: &SourceRef) -> Failure {
        let errors = self
            .workspace
            .as_ref()
            .map(|w| w.errors())
            .unwrap_or_default();
        let mut text = format!("{r} not found in {}", self.sources);
        if let Some(e) = errors.iter().rev().find(|e| e.starts_with(&r.to_string())) {
            text.push_str(&format!(": {e}"));
        }
        Failure::Tool(text)
    }

    fn source(&self, args: &Value) -> Result<String, Failure> {
        let r = self.reference(args)?;
        // dictionary types come as their DDL source or XML
        let unit = self
            .workspace()?
            .unit(&r)
            .ok_or_else(|| self.not_found(&r))?;
        let first = number(args, "start_row").unwrap_or(1).max(1);
        let last = number(args, "end_row").unwrap_or(u64::MAX);
        let mut out = format!("{} ({})\n", r, unit.origin);
        for (i, line) in unit.file.lexed.source.lines().enumerate() {
            let row = i as u64 + 1;
            if row >= first && row <= last {
                out.push_str(&format!("{row:>5}| {line}\n"));
            }
        }
        if r.kind == ObjectKind::Clas && r.part.is_none() {
            out.push_str(
                "\nlocal classes and types of a global class are in its parts: \
                 CLAS:NAME/definitions, /implementations, /macros, /testclasses\n",
            );
        }
        Ok(out)
    }

    fn call_graph(&self, args: &Value) -> Result<String, Failure> {
        let objects: Vec<SourceRef> = match args.get("objects") {
            Some(Value::Array(a)) => a
                .iter()
                .map(|v| v.as_str().unwrap_or("").parse())
                .collect::<Result<_, _>>()
                .map_err(Failure::Tool)?,
            _ => vec![string(args, "objects")?.parse().map_err(Failure::Tool)?],
        };
        let depth = number(args, "depth").unwrap_or(1).min(5) as u32;
        let graph = self.workspace()?.call_graph(&objects, depth);
        if Detail::of(args) == Detail::Full {
            return Ok(compact(&graph));
        }
        Ok(cut_text(format!(
            "depth {depth}; {}",
            text::call_graph(&graph)
        )))
    }

    fn logic(&self, args: &Value) -> Result<String, Failure> {
        let r = self.reference(args)?;
        // answers grow fast with depth: only the object itself by default
        let depth = number(args, "depth").unwrap_or(0).min(5) as u32;
        let mut logic = self.workspace()?.logic(&r, depth);
        if logic.procedures.is_empty() && self.workspace()?.object(&r).is_none() {
            return Err(self.not_found(&r));
        }
        filter_procedures(&mut logic, args);
        Ok(logic_answer(&logic, Detail::of(args)))
    }
}

enum Failure {
    /// reported to the model as a failed tool call
    Tool(String),
    Unknown,
}

#[derive(PartialEq, Clone, Copy)]
enum Detail {
    Summary,
    Full,
}

impl Detail {
    fn of(args: &Value) -> Detail {
        match args.get("detail").and_then(Value::as_str) {
            Some("full") => Detail::Full,
            _ => Detail::Summary,
        }
    }
}

/// Keeps the procedures named by the `procedure` argument: `name`,
/// `class->name` or `class=>name`, case insensitive.
fn filter_procedures(logic: &mut m::FileLogic, args: &Value) {
    let Some(wanted) = args.get("procedure").and_then(Value::as_str) else {
        return;
    };
    let wanted = wanted.to_ascii_lowercase();
    let (class, name) = match wanted.split_once("->").or_else(|| wanted.split_once("=>")) {
        Some((c, n)) => (Some(c.to_string()), n.to_string()),
        None => (None, wanted),
    };
    logic.procedures.retain(|p| {
        p.block.name.eq_ignore_ascii_case(&name)
            && class.as_deref().is_none_or(|c| {
                p.block
                    .class
                    .as_deref()
                    .is_some_and(|x| x.eq_ignore_ascii_case(c))
            })
    });
}

fn logic_answer(logic: &m::FileLogic, detail: Detail) -> String {
    match detail {
        Detail::Full => compact(logic),
        Detail::Summary if logic.procedures.is_empty() => "no procedures".to_string(),
        Detail::Summary => cut_text(text::logic(logic)),
    }
}

/// Compact JSON, cut at [`MAX_ANSWER`].
fn compact(value: &impl serde::Serialize) -> String {
    cut_text(serde_json::to_string(value).expect("serializes"))
}

/// The text, cut at [`MAX_ANSWER`] with a hint how to ask for less.
fn cut_text(text: String) -> String {
    if text.len() <= MAX_ANSWER {
        return text;
    }
    let mut end = MAX_ANSWER;
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    format!(
        "{}\n... cut at {MAX_ANSWER} of {} characters; ask for less, e.g. one procedure, \
         a smaller depth or a row range",
        &text[..end],
        text.len()
    )
}

fn string(args: &Value, name: &str) -> Result<String, Failure> {
    args.get(name)
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| Failure::Tool(format!("missing argument {name}")))
}

fn number(args: &Value, name: &str) -> Option<u64> {
    args.get(name).and_then(Value::as_u64)
}

fn tool_result(text: &str, is_error: bool) -> Value {
    json!({"content": [{"type": "text", "text": text}], "isError": is_error})
}

fn error(id: Value, code: i32, message: &str) -> Value {
    json!({"jsonrpc": "2.0", "id": id, "error": {"code": code, "message": message}})
}

fn tools() -> Value {
    let object = json!({
        "type": "string",
        "description": "KIND:NAME, e.g. CLAS:ZCL_ORDERS, INTF:ZIF_X, PROG:ZREPORT, FUNC:Z_FM, FUGR:ZGROUP",
    });
    let detail = json!({
        "type": "string",
        "enum": ["summary", "full"],
        "description": "summary (default): compact text, definitions written like ABAP, one line per call or access, rows to read with get_source; full: the complete model as JSON",
    });
    let depth = json!({
        "type": "integer",
        "minimum": 0,
        "maximum": 5,
        "description": "how many hops calls into other objects are followed (default 1)",
    });
    let read_only = json!({"readOnlyHint": true, "openWorldHint": false});
    json!([
        {
            "name": "get_source",
            "title": "ABAP source",
            "description": "Source code of an object with row numbers. A global class's local classes are in CLAS:NAME/implementations and /definitions.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "object": object,
                    "start_row": {"type": "integer", "minimum": 1},
                    "end_row": {"type": "integer", "minimum": 1},
                },
                "required": ["object"],
            },
            "annotations": read_only,
        },
        {
            "name": "get_symbols",
            "title": "Structure of an object",
            "description": "Classes, interfaces, methods with signatures, attributes, types, forms, function modules and declarations of all sources of an object, written like their ABAP definitions with rows.",
            "inputSchema": {"type": "object", "properties": {"object": object, "detail": detail}, "required": ["object"]},
            "annotations": read_only,
        },
        {
            "name": "get_call_graph",
            "title": "Calls",
            "description": "Calls from the given objects: methods, function modules, forms, programs, transactions, with the calling procedure and row. Calls into other objects are followed up to depth hops.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "objects": {"type": "array", "items": object, "minItems": 1},
                    "depth": depth,
                    "detail": detail,
                },
                "required": ["objects"],
            },
            "annotations": read_only,
        },
        {
            "name": "get_logic",
            "title": "What procedures do",
            "description": "Per procedure (method, form, function module): parameters and effects, i.e. the data, parameters and database tables written, by which statements, under which conditions, from which inputs; effects of called procedures included. detail=full gives every statement with conditions, data read and written, and the control flow.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "object": object,
                    "procedure": {"type": "string", "description": "only this procedure: name, class->name or class=>name"},
                    "depth": {
                        "type": "integer",
                        "minimum": 0,
                        "maximum": 5,
                        "description": "how many hops calls into other objects are followed for their effects (default 0: calls within the object only)",
                    },
                    "detail": detail,
                },
                "required": ["object"],
            },
            "annotations": read_only,
        },
        {
            "name": "get_db_access",
            "title": "Database access",
            "description": "Open SQL and native SQL statements of an object, one line each: row, procedure, operation, tables, statement. detail=full adds fields, joins, conditions and targets as JSON.",
            "inputSchema": {"type": "object", "properties": {"object": object, "detail": detail}, "required": ["object"]},
            "annotations": read_only,
        },
        {
            "name": "find_implementations",
            "title": "Implementations of an interface",
            "description": "Global classes implementing an interface.",
            "inputSchema": {
                "type": "object",
                "properties": {"interface": {"type": "string", "description": "e.g. ZIF_REPOSITORY"}},
                "required": ["interface"],
            },
            "annotations": read_only,
        },
        {
            "name": "analyze_source",
            "title": "Analyze ABAP text",
            "description": "Analyzes ABAP code given as text, without loading anything: symbols, calls, db, logic or statements.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "source": {"type": "string"},
                    "analysis": {"type": "string", "enum": ["symbols", "calls", "db", "logic", "statements"]},
                    "procedure": {"type": "string"},
                    "detail": detail,
                },
                "required": ["source"],
            },
            "annotations": read_only,
        },
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use abap_analysis::abap_source::MemoryProvider;

    fn server() -> Server {
        let mut p = MemoryProvider::new();
        p.insert(
            SourceRef::class("ZCL_ORDERS"),
            "CLASS zcl_orders DEFINITION PUBLIC.
  PUBLIC SECTION.
    METHODS save IMPORTING iv_id TYPE i.
ENDCLASS.
CLASS zcl_orders IMPLEMENTATION.
  METHOD save.
    IF iv_id > 0.
      UPDATE zorders SET status = 'S' WHERE id = iv_id.
      CALL FUNCTION 'Z_LOG'.
    ENDIF.
  ENDMETHOD.
ENDCLASS.",
        );
        Server::new(Ok(Box::new(p)))
    }

    fn call(s: &Server, tool: &str, args: Value) -> (String, bool) {
        let r = s
            .handle(&json!({"jsonrpc": "2.0", "id": 1, "method": "tools/call",
                "params": {"name": tool, "arguments": args}}))
            .unwrap();
        let result = &r["result"];
        (
            result["content"][0]["text"].as_str().unwrap().to_string(),
            result["isError"].as_bool().unwrap(),
        )
    }

    #[test]
    fn handshake_and_tool_list() {
        let s = server();
        let r = s
            .handle(&json!({"jsonrpc": "2.0", "id": 0, "method": "initialize",
                "params": {"protocolVersion": "2025-03-26", "capabilities": {}}}))
            .unwrap();
        assert_eq!(r["result"]["protocolVersion"], "2025-03-26");
        assert!(r["result"]["capabilities"]["tools"].is_object());
        // notifications get no answer
        assert!(s
            .handle(&json!({"jsonrpc": "2.0", "method": "notifications/initialized"}))
            .is_none());
        let r = s
            .handle(&json!({"jsonrpc": "2.0", "id": 1, "method": "tools/list"}))
            .unwrap();
        let names: Vec<&str> = r["result"]["tools"]
            .as_array()
            .unwrap()
            .iter()
            .map(|t| t["name"].as_str().unwrap())
            .collect();
        assert_eq!(
            names,
            [
                "get_source",
                "get_symbols",
                "get_call_graph",
                "get_logic",
                "get_db_access",
                "find_implementations",
                "analyze_source"
            ]
        );
        let r = s
            .handle(&json!({"jsonrpc": "2.0", "id": 2, "method": "nope"}))
            .unwrap();
        assert_eq!(r["error"]["code"], -32601);
    }

    #[test]
    fn tools_answer() {
        let s = server();
        let (text, err) = call(
            &s,
            "get_source",
            json!({"object": "CLAS:ZCL_ORDERS", "start_row": 6, "end_row": 7}),
        );
        assert!(!err);
        assert!(
            text.contains("    6|   METHOD save.\n    7|     IF iv_id > 0.\n"),
            "{text}"
        );
        assert!(!text.contains("ENDCLASS"));

        let (text, _) = call(
            &s,
            "get_call_graph",
            json!({"objects": ["CLAS:ZCL_ORDERS"]}),
        );
        assert!(
            text.contains("9 zcl_orders->save: function Z_LOG (unresolved)"),
            "{text}"
        );
        assert!(text.contains("missing: FUNC:Z_LOG"), "{text}");

        let (text, _) = call(
            &s,
            "get_logic",
            json!({"object": "CLAS:ZCL_ORDERS", "procedure": "zcl_orders->save"}),
        );
        assert!(
            text.starts_with("zcl_orders->save 6-11 in CLAS:ZCL_ORDERS\n"),
            "{text}"
        );
        assert!(
            text.contains(
                "  writes ZORDERS (database table) at 8; from iv_id\n  statements:\n        8 UPDATE zorders SET status = 'S' WHERE id = iv_id.  when iv_id > 0\n"
            ),
            "{text}"
        );
        let (text, _) = call(
            &s,
            "get_logic",
            json!({"object": "CLAS:ZCL_ORDERS", "detail": "full"}),
        );
        let v: Value = serde_json::from_str(&text).unwrap();
        assert_eq!(v["procedures"][0]["block"]["name"], "save");

        let (text, _) = call(&s, "get_symbols", json!({"object": "CLAS:ZCL_ORDERS"}));
        assert!(
            text.contains("    METHODS save IMPORTING iv_id TYPE i.  \" 3\n"),
            "{text}"
        );

        let (text, _) = call(&s, "get_db_access", json!({"object": "CLAS:ZCL_ORDERS"}));
        assert!(
            text.contains("CLAS:ZCL_ORDERS\n      8 zcl_orders->save: update zorders | UPDATE"),
            "{text}"
        );

        let (text, err) = call(&s, "get_symbols", json!({"object": "CLAS:ZCL_MISSING"}));
        assert!(err);
        assert!(text.starts_with("CLAS:ZCL_MISSING not found"), "{text}");

        let (text, err) = call(
            &s,
            "analyze_source",
            json!({"source": "FORM f.\n  CALL FUNCTION 'X'.\nENDFORM.", "analysis": "calls"}),
        );
        assert!(!err);
        assert_eq!(text, "      2 form f: function X (unresolved)\n");
    }

    #[test]
    fn without_sources() {
        let s = Server::new(Err("no source".to_string()));
        let (text, err) = call(&s, "get_symbols", json!({"object": "CLAS:ZCL_X"}));
        assert!(err);
        assert!(text.contains("--src"), "{text}");
        let (_, err) = call(&s, "analyze_source", json!({"source": "REPORT z."}));
        assert!(!err);
    }
}
