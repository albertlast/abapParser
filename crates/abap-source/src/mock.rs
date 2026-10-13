//! A local stand-in for the ADT REST API of an SAP system. It serves ABAP
//! sources from folders under the paths the [`AdtProvider`](crate::AdtProvider)
//! reads, so loading sources from a system can be tested without one.
//!
//! Served: programs and includes, classes with their includes, interfaces,
//! function groups and modules, and the repository quick search that finds
//! the group of a function module. Requests need basic authentication; a
//! CSRF token is handed out on request. Only the standard library is used.

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

use crate::{ClassPart, FsProvider, ObjectKind, Result, SourceProvider, SourceRef};

/// Who may log on to the mock server.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MockAdtConfig {
    pub user: String,
    pub password: String,
    /// when set, requests for another `sap-client` are rejected
    pub client: Option<String>,
}

impl Default for MockAdtConfig {
    fn default() -> Self {
        MockAdtConfig {
            user: "DEVELOPER".to_string(),
            password: "mock".to_string(),
            client: None,
        }
    }
}

/// A running mock server; stops when dropped.
pub struct MockAdtServer {
    url: String,
    requests: Arc<Mutex<Vec<String>>>,
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
    objects: usize,
}

struct State {
    provider: FsProvider,
    /// function module -> function group, upper case
    groups: HashMap<String, String>,
    authorization: String,
    client: Option<String>,
    requests: Arc<Mutex<Vec<String>>>,
}

const CSRF_TOKEN: &str = "mock-csrf-token";

impl MockAdtServer {
    /// Serves the sources of `provider` at `address`, e.g. `127.0.0.1:0`
    /// for any free port.
    pub fn start(provider: FsProvider, address: &str, config: MockAdtConfig) -> Result<Self> {
        let listener = TcpListener::bind(address)?;
        let url = format!("http://{}", listener.local_addr()?);
        let mut groups = HashMap::new();
        let fugrs: Vec<String> = provider
            .references()
            .filter(|r| r.kind == ObjectKind::Fugr)
            .map(|r| r.name.clone())
            .collect();
        for g in fugrs {
            for fm in provider.function_modules(&g)? {
                groups.insert(fm.to_ascii_uppercase(), g.clone());
            }
        }
        let objects = provider.len();
        let requests = Arc::new(Mutex::new(Vec::new()));
        let stop = Arc::new(AtomicBool::new(false));
        let state = Arc::new(State {
            provider,
            groups,
            authorization: format!(
                "Basic {}",
                base64(format!("{}:{}", config.user, config.password).as_bytes())
            ),
            client: config.client,
            requests: requests.clone(),
        });
        let stopped = stop.clone();
        let thread = std::thread::spawn(move || {
            for stream in listener.incoming() {
                if stopped.load(Ordering::SeqCst) {
                    break;
                }
                let Ok(stream) = stream else { continue };
                let state = state.clone();
                std::thread::spawn(move || {
                    let _ = handle(&state, stream);
                });
            }
        });
        Ok(MockAdtServer {
            url,
            requests,
            stop,
            thread: Some(thread),
            objects,
        })
    }

    /// Base URL, e.g. `http://127.0.0.1:50000`.
    pub fn url(&self) -> &str {
        &self.url
    }

    /// Number of objects served.
    pub fn objects(&self) -> usize {
        self.objects
    }

    /// Paths with query of the requests received so far, in order.
    pub fn requests(&self) -> Vec<String> {
        self.requests.lock().map(|r| r.clone()).unwrap_or_default()
    }

    /// Blocks until the server stops, for running it as a program.
    pub fn wait(mut self) {
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
    }
}

impl Drop for MockAdtServer {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        // wake up the accepting thread
        let _ = TcpStream::connect(self.url.trim_start_matches("http://"));
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
    }
}

struct Response {
    status: u16,
    content_type: &'static str,
    body: String,
    csrf: bool,
}

impl Response {
    fn status(status: u16) -> Self {
        Response {
            status,
            content_type: "text/plain; charset=utf-8",
            body: String::new(),
            csrf: false,
        }
    }
}

fn handle(state: &State, stream: TcpStream) -> std::io::Result<()> {
    let mut reader = BufReader::new(stream.try_clone()?);
    let mut request = String::new();
    reader.read_line(&mut request)?;
    let mut authorization = String::new();
    let mut csrf = false;
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line)? == 0 || line == "\r\n" || line == "\n" {
            break;
        }
        if let Some((name, value)) = line.split_once(':') {
            let value = value.trim();
            match name.trim().to_ascii_lowercase().as_str() {
                "authorization" => authorization = value.to_string(),
                "x-csrf-token" => csrf = value.eq_ignore_ascii_case("fetch"),
                _ => {}
            }
        }
    }
    let mut parts = request.split_whitespace();
    let method = parts.next().unwrap_or_default();
    let target = parts.next().unwrap_or_default();
    if let Ok(mut r) = state.requests.lock() {
        r.push(target.to_string());
    }
    let (path, query) = target.split_once('?').unwrap_or((target, ""));
    let params: HashMap<String, String> = query
        .split('&')
        .filter_map(|p| p.split_once('='))
        .map(|(k, v)| (k.to_ascii_lowercase(), percent_decode(v)))
        .collect();

    let client_ok = match (&state.client, params.get("sap-client")) {
        (Some(c), Some(asked)) => c == asked,
        _ => true,
    };
    let mut response = if authorization != state.authorization || !client_ok {
        Response::status(401)
    } else if method != "GET" && method != "HEAD" {
        Response::status(405)
    } else {
        route(state, path, &params)
    };
    response.csrf = csrf && response.status != 401;

    let reason = match response.status {
        200 => "OK",
        401 => "Unauthorized",
        404 => "Not Found",
        405 => "Method Not Allowed",
        _ => "Error",
    };
    let mut out = stream;
    let mut head = format!(
        "HTTP/1.1 {} {reason}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n",
        response.status,
        response.content_type,
        response.body.len()
    );
    if response.status == 401 {
        head.push_str("WWW-Authenticate: Basic realm=\"SAP NetWeaver Application Server\"\r\n");
    }
    if response.csrf {
        head.push_str(&format!("X-CSRF-Token: {CSRF_TOKEN}\r\n"));
    }
    head.push_str("\r\n");
    out.write_all(head.as_bytes())?;
    if method != "HEAD" {
        out.write_all(response.body.as_bytes())?;
    }
    out.flush()
}

fn route(state: &State, path: &str, params: &HashMap<String, String>) -> Response {
    let Some(rest) = path.strip_prefix("/sap/bc/adt/") else {
        return Response::status(404);
    };
    let seg: Vec<String> = rest.split('/').map(percent_decode).collect();
    let seg: Vec<&str> = seg.iter().map(String::as_str).collect();
    let source = |r: SourceRef, accept: &dyn Fn(&str) -> bool| match state.provider.fetch(&r) {
        Ok(Some(s)) if accept(&s.text) => Response {
            status: 200,
            content_type: "text/plain; charset=utf-8",
            body: s.text,
            csrf: false,
        },
        _ => Response::status(404),
    };
    let any = |_: &str| true;
    match seg.as_slice() {
        ["discovery"] | ["core", "discovery"] => Response {
            status: 200,
            content_type: "application/atomsvc+xml",
            body: "<?xml version=\"1.0\" encoding=\"utf-8\"?><app:service xmlns:app=\"http://www.w3.org/2007/app\"/>".to_string(),
            csrf: false,
        },
        // a report is a program, everything else an include
        ["programs", "programs", name, "source", "main"] => {
            source(SourceRef::program(name), &is_report)
        }
        ["programs", "includes", name, "source", "main"] => {
            source(SourceRef::program(name), &|t: &str| !is_report(t))
        }
        ["oo", "classes", name, "source", "main"] => source(SourceRef::class(name), &any),
        ["oo", "classes", name, "includes", include] => {
            let part = match include.to_ascii_lowercase().as_str() {
                "definitions" => ClassPart::Definitions,
                "implementations" => ClassPart::Implementations,
                "macros" => ClassPart::Macros,
                "testclasses" => ClassPart::TestClasses,
                _ => return Response::status(404),
            };
            source(SourceRef::class(name).with_part(part), &any)
        }
        ["oo", "interfaces", name, "source", "main"] => source(SourceRef::interface(name), &any),
        ["functions", "groups", name, "source", "main"] => {
            source(SourceRef::function_group(name), &any)
        }
        ["functions", "groups", group, "fmodules", name, "source", "main"] => {
            let in_group = state
                .groups
                .get(&name.to_ascii_uppercase())
                .is_some_and(|g| g.eq_ignore_ascii_case(group));
            if !in_group {
                return Response::status(404);
            }
            source(SourceRef::function_module(name), &any)
        }
        ["repository", "informationsystem", "search"] => search(state, params),
        _ => Response::status(404),
    }
}

/// The quick search of the repository information system, for function
/// modules only: `objectType=FUGR/FF`, `query` with `*` as wildcard.
fn search(state: &State, params: &HashMap<String, String>) -> Response {
    let query = params
        .get("query")
        .map(|q| q.to_ascii_uppercase())
        .unwrap_or_default();
    let max: usize = params
        .get("maxresults")
        .and_then(|m| m.parse().ok())
        .unwrap_or(100);
    let functions = params
        .get("objecttype")
        .is_none_or(|t| t.eq_ignore_ascii_case("FUGR/FF"));
    let mut found: Vec<(&String, &String)> = if functions {
        state
            .groups
            .iter()
            .filter(|(fm, _)| wildcard(&query, fm))
            .collect()
    } else {
        Vec::new()
    };
    found.sort();
    let mut body = String::from(
        "<?xml version=\"1.0\" encoding=\"utf-8\"?><adtcore:objectReferences xmlns:adtcore=\"http://www.sap.com/adt/core\">",
    );
    for (fm, group) in found.into_iter().take(max) {
        body.push_str(&format!(
            "<adtcore:objectReference adtcore:uri=\"/sap/bc/adt/functions/groups/{}/fmodules/{}\" adtcore:type=\"FUGR/FF\" adtcore:name=\"{}\" adtcore:packageName=\"$TMP\"/>",
            percent_encode(group),
            percent_encode(fm),
            xml_escape(fm)
        ));
    }
    body.push_str("</adtcore:objectReferences>");
    Response {
        status: 200,
        content_type: "application/xml",
        body,
        csrf: false,
    }
}

/// Whether the source is a report: its first statement is REPORT or PROGRAM.
fn is_report(text: &str) -> bool {
    for line in text.lines() {
        let line = line.split('"').next().unwrap_or_default().trim();
        if line.is_empty() || line.starts_with('*') {
            continue;
        }
        let word = line
            .split(|c: char| c.is_whitespace() || c == '.')
            .next()
            .unwrap_or_default()
            .to_ascii_uppercase();
        return word == "REPORT" || word == "PROGRAM";
    }
    false
}

/// `pattern` with `*` matching any characters, compared upper case.
fn wildcard(pattern: &str, name: &str) -> bool {
    let parts: Vec<&str> = pattern.split('*').collect();
    if parts.len() == 1 {
        return pattern == name;
    }
    let mut rest = name;
    for (i, part) in parts.iter().enumerate() {
        if part.is_empty() {
            continue;
        }
        if i == 0 {
            match rest.strip_prefix(part) {
                Some(r) => rest = r,
                None => return false,
            }
        } else if i == parts.len() - 1 {
            return rest.ends_with(part);
        } else {
            match rest.find(part) {
                Some(p) => rest = &rest[p + part.len()..],
                None => return false,
            }
        }
    }
    true
}

fn percent_decode(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%' && i + 2 < b.len() {
            if let Ok(v) = u8::from_str_radix(&s[i + 1..i + 3], 16) {
                out.push(v);
                i += 3;
                continue;
            }
        }
        out.push(if b[i] == b'+' { b' ' } else { b[i] });
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn percent_encode(name: &str) -> String {
    let mut out = String::new();
    for b in name.to_ascii_lowercase().bytes() {
        match b {
            b'a'..=b'z' | b'0'..=b'9' | b'_' | b'-' | b'.' => out.push(b as char),
            _ => out.push_str(&format!("%{b:02x}")),
        }
    }
    out
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn base64(data: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::new();
    for chunk in data.chunks(3) {
        let n = (u32::from(chunk[0]) << 16)
            | (u32::from(*chunk.get(1).unwrap_or(&0)) << 8)
            | u32::from(*chunk.get(2).unwrap_or(&0));
        for k in 0..4 {
            if k <= chunk.len() {
                out.push(CHARS[(n >> (18 - 6 * k)) as usize & 63] as char);
            } else {
                out.push('=');
            }
        }
    }
    out
}

#[cfg(all(test, feature = "adt"))]
mod tests {
    use super::*;
    use crate::{AdtConfig, AdtProvider, SourceError};

    /// A folder with one object of every kind, abapGit file names.
    fn folder() -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "abap-mock-adt-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let files = [
            (
                "zreport.prog.abap",
                "* demo\nREPORT zreport.\nINCLUDE zreport_top.",
            ),
            ("zreport_top.prog.abap", "DATA gv_x TYPE i."),
            (
                "zcl_a.clas.abap",
                "CLASS zcl_a DEFINITION PUBLIC.\nENDCLASS.\nCLASS zcl_a IMPLEMENTATION.\nENDCLASS.",
            ),
            (
                "zcl_a.clas.locals_imp.abap",
                "CLASS lcl DEFINITION.\nENDCLASS.",
            ),
            (
                "#abc#cl_ns.clas.abap",
                "CLASS /abc/cl_ns DEFINITION PUBLIC.\nENDCLASS.",
            ),
            ("zif_a.intf.abap", "INTERFACE zif_a PUBLIC.\nENDINTERFACE."),
            ("zgrp.fugr.saplzgrp.abap", "FUNCTION-POOL zgrp."),
            ("zgrp.fugr.z_fm.abap", "FUNCTION z_fm.\nENDFUNCTION."),
        ];
        for (name, text) in files {
            std::fs::write(dir.join(name), text).unwrap();
        }
        dir
    }

    fn server(dir: &std::path::Path) -> MockAdtServer {
        let fs = FsProvider::new(vec![dir.to_path_buf()]).unwrap();
        MockAdtServer::start(fs, "127.0.0.1:0", MockAdtConfig::default()).unwrap()
    }

    fn provider(url: &str, password: &str) -> AdtProvider {
        AdtProvider::new(AdtConfig::new(url, "DEVELOPER", password))
    }

    #[test]
    fn serves_every_kind_to_the_adt_provider() {
        let dir = folder();
        let server = server(&dir);
        let adt = provider(server.url(), "mock");
        let text = |r: &str| {
            adt.fetch(&r.parse().unwrap())
                .unwrap()
                .map(|s| s.text)
                .unwrap_or_else(|| panic!("{r} not served"))
        };
        assert!(text("PROG:ZREPORT").contains("REPORT zreport"));
        assert!(text("PROG:ZREPORT_TOP").contains("gv_x"));
        assert!(text("CLAS:ZCL_A").contains("zcl_a"));
        assert!(text("CLAS:ZCL_A/implementations").contains("lcl"));
        assert!(text("CLAS:/ABC/CL_NS").contains("/abc/cl_ns"));
        assert!(text("INTF:ZIF_A").contains("zif_a"));
        assert!(text("FUGR:ZGRP").contains("FUNCTION-POOL"));
        // the group of the function module is found by the quick search
        assert!(text("FUNC:Z_FM").contains("FUNCTION z_fm"));
        assert!(adt
            .fetch(&"CLAS:ZCL_NOPE".parse().unwrap())
            .unwrap()
            .is_none());

        let requests = server.requests();
        assert!(requests
            .iter()
            .any(|r| r.contains("/oo/classes/%2fabc%2fcl_ns/")));
        assert!(requests.iter().any(|r| r.contains("operation=quickSearch")));
        // the include was asked for as a program first
        assert!(requests
            .iter()
            .any(|r| r.contains("/programs/programs/zreport_top/")));
        drop(server);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn rejects_wrong_credentials_and_clients() {
        let dir = folder();
        let fs = FsProvider::new(vec![dir.clone()]).unwrap();
        let config = MockAdtConfig {
            client: Some("001".into()),
            ..MockAdtConfig::default()
        };
        let server = MockAdtServer::start(fs, "127.0.0.1:0", config).unwrap();
        let wrong = provider(server.url(), "wrong");
        assert!(matches!(
            wrong.fetch(&"CLAS:ZCL_A".parse().unwrap()),
            Err(SourceError::Auth { .. })
        ));
        let mut c = AdtConfig::new(server.url(), "DEVELOPER", "mock");
        c.client = Some("100".into());
        assert!(AdtProvider::new(c)
            .fetch(&"CLAS:ZCL_A".parse().unwrap())
            .is_err());
        let mut c = AdtConfig::new(server.url(), "DEVELOPER", "mock");
        c.client = Some("001".into());
        assert!(AdtProvider::new(c)
            .fetch(&"CLAS:ZCL_A".parse().unwrap())
            .unwrap()
            .is_some());
        drop(server);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn helpers() {
        assert_eq!(base64(b"DEVELOPER:mock"), "REVWRUxPUEVSOm1vY2s=");
        assert_eq!(base64(b"ab"), "YWI=");
        assert_eq!(percent_decode("%2fabc%2fcl_x"), "/abc/cl_x");
        assert!(wildcard("Z_*", "Z_FM"));
        assert!(wildcard("*FM", "Z_FM"));
        assert!(!wildcard("Y*", "Z_FM"));
        assert!(is_report("* comment\n\n  report zx line-size 80."));
        assert!(!is_report("DATA x TYPE i."));
    }
}
