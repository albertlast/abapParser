//! A local stand-in for the ADT REST API of an SAP system. It serves ABAP
//! sources from folders under the paths the [`AdtProvider`](crate::AdtProvider)
//! reads, so loading sources from a system can be tested without one.
//!
//! Served: programs and includes, classes with their includes, interfaces,
//! function groups and modules, the repository quick search that finds the
//! group of a function module, and the where-used list of an interface.
//! Requests need basic authentication. A CSRF token with a session cookie is
//! handed out on request; POST requests need both. Only the standard
//! library is used.
//!
//! With [`MockRelease::Basis750`] it acts like an ERP on SAP_BASIS 750 as
//! measured on one: its discovery and component list, data elements in XML
//! version 1 only (406 for others), structures but no tables or lock
//! objects, no enhancement implementations, and the data preview answering
//! ABAP SQL on the class relations, BAdI tables and enhancement index of the
//! folders, rejecting what 7.50 rejects: long strings in `WHERE` and lines
//! over 255 characters. Dictionary sources come as abapGit XML, which the
//! analysis reads like the system's formats.

mod tables;

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

use crate::badi::field;
use crate::{ClassPart, FsProvider, ObjectKind, Result, SourceProvider, SourceRef};
use tables::Tables;

/// Which system the mock acts like.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum MockRelease {
    /// sources only: an empty discovery, no data preview, no dictionary
    #[default]
    Plain,
    /// an ERP on SAP_BASIS 750 SP 35
    Basis750,
}

impl std::str::FromStr for MockRelease {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, String> {
        match s {
            "plain" => Ok(MockRelease::Plain),
            "750" => Ok(MockRelease::Basis750),
            _ => Err(format!("unknown release {s}, known: plain, 750")),
        }
    }
}

/// Who may log on to the mock server.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MockAdtConfig {
    pub user: String,
    pub password: String,
    /// when set, requests for another `sap-client` are rejected
    pub client: Option<String>,
    pub release: MockRelease,
}

impl Default for MockAdtConfig {
    fn default() -> Self {
        MockAdtConfig {
            user: "DEVELOPER".to_string(),
            password: "mock".to_string(),
            client: None,
            release: MockRelease::Plain,
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
    release: MockRelease,
    /// the tables the data preview reads, for releases that have one
    tables: Option<Tables>,
    requests: Arc<Mutex<Vec<String>>>,
}

const CSRF_TOKEN: &str = "mock-csrf-token";
const SESSION_COOKIE: &str = "SAP_SESSIONID_MCK_001=mock-session";

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
        let tables = (config.release != MockRelease::Plain).then(|| Tables::of(&provider, &groups));
        let state = Arc::new(State {
            provider,
            groups,
            release: config.release,
            tables,
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
    /// the CSRF token of the request was missing or wrong
    token_required: bool,
}

impl Response {
    fn status(status: u16) -> Self {
        Response {
            status,
            content_type: "text/plain; charset=utf-8",
            body: String::new(),
            csrf: false,
            token_required: false,
        }
    }

    fn ok(content_type: &'static str, body: String) -> Self {
        Response {
            status: 200,
            content_type,
            body,
            csrf: false,
            token_required: false,
        }
    }
}

fn handle(state: &State, stream: TcpStream) -> std::io::Result<()> {
    let mut reader = BufReader::new(stream.try_clone()?);
    let mut request = String::new();
    reader.read_line(&mut request)?;
    let mut authorization = String::new();
    let mut token = String::new();
    let mut cookie = String::new();
    let mut accept = String::new();
    let mut length = 0usize;
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line)? == 0 || line == "\r\n" || line == "\n" {
            break;
        }
        if let Some((name, value)) = line.split_once(':') {
            let value = value.trim();
            match name.trim().to_ascii_lowercase().as_str() {
                "authorization" => authorization = value.to_string(),
                "x-csrf-token" => token = value.to_string(),
                "cookie" => cookie = value.to_string(),
                "accept" => accept = value.to_string(),
                "content-length" => length = value.parse().unwrap_or(0),
                _ => {}
            }
        }
    }
    let mut body = vec![0u8; length.min(1 << 20)];
    reader.read_exact(&mut body)?;
    let csrf = token.eq_ignore_ascii_case("fetch");
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
    let session = token == CSRF_TOKEN && cookie.split(';').any(|c| c.trim() == SESSION_COOKIE);
    let mut response = if authorization != state.authorization || !client_ok {
        Response::status(401)
    } else if method == "POST" && path.ends_with("/informationsystem/usageReferences") {
        if session {
            usage_references(state, &params)
        } else {
            Response {
                token_required: true,
                ..Response::status(403)
            }
        }
    } else if method == "POST" && state.tables.is_some() && path.starts_with(PREVIEW) {
        if session {
            preview(state, path, &params, &String::from_utf8_lossy(&body))
        } else {
            Response {
                token_required: true,
                ..Response::status(403)
            }
        }
    } else if method != "GET" && method != "HEAD" {
        Response::status(405)
    } else {
        route(state, path, &params, &accept)
    };
    response.csrf = csrf && response.status != 401;

    let reason = match response.status {
        200 => "OK",
        400 => "Bad Request",
        401 => "Unauthorized",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        406 => "Not Acceptable",
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
        head.push_str(&format!("Set-Cookie: {SESSION_COOKIE}; path=/\r\n"));
    }
    if response.token_required {
        head.push_str("X-CSRF-Token: Required\r\n");
    }
    head.push_str("\r\n");
    out.write_all(head.as_bytes())?;
    if method != "HEAD" {
        out.write_all(response.body.as_bytes())?;
    }
    out.flush()
}

fn route(state: &State, path: &str, params: &HashMap<String, String>, accept: &str) -> Response {
    let Some(rest) = path.strip_prefix("/sap/bc/adt/") else {
        return Response::status(404);
    };
    let seg: Vec<String> = rest.split('/').map(percent_decode).collect();
    let seg: Vec<&str> = seg.iter().map(String::as_str).collect();
    if state.release == MockRelease::Basis750 {
        if let Some(response) = route_750(state, &seg, accept) {
            return response;
        }
    }
    let source = |r: SourceRef, accept: &dyn Fn(&str) -> bool| match state.provider.fetch(&r) {
        Ok(Some(s)) if accept(&s.text) => Response {
            status: 200,
            content_type: "text/plain; charset=utf-8",
            body: s.text,
            csrf: false,
            token_required: false,
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
            token_required: false,
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
        // the abapGit XML; the provider only passes the text on
        ["ddic", "tabletypes", name] => match state.provider.fetch(&SourceRef::table_type(name)) {
            Ok(Some(s)) => Response::ok("application/vnd.sap.adt.tabletype.v1+xml", s.text),
            _ => Response::status(404),
        },
        ["ddic", "typegroups", name, "source", "main"] => {
            source(SourceRef::type_group(name), &any)
        }
        _ => Response::status(404),
    }
}

const PREVIEW: &str = "/sap/bc/adt/datapreview/";

/// The collections of the discovery of SAP_BASIS 750 the provider looks at,
/// with the content types they accept.
const DISCOVERY_750: [(&str, &str); 12] = [
    (
        "programs/programs",
        "application/vnd.sap.adt.programs.programs.v2+xml",
    ),
    (
        "programs/includes",
        "application/vnd.sap.adt.programs.includes.v2+xml",
    ),
    ("oo/classes", "application/vnd.sap.adt.oo.classes.v2+xml"),
    (
        "oo/interfaces",
        "application/vnd.sap.adt.oo.interfaces.v2+xml",
    ),
    (
        "functions/groups",
        "application/vnd.sap.adt.functions.groups.v2+xml",
    ),
    ("repository/informationsystem/search", ""),
    (
        "repository/informationsystem/usageReferences",
        "application/vnd.sap.adt.repository.usagereferences.request.v1+xml",
    ),
    ("datapreview/freestyle", ""),
    ("datapreview/ddic", ""),
    ("ddic/structures", "application/vnd.sap.adt.blues.v1+xml"),
    (
        "ddic/dataelements",
        "application/vnd.sap.adt.dataelements.v1+xml",
    ),
    ("ddic/ddl/sources", "application/vnd.sap.adt.ddlsource+xml"),
];

/// What SAP_BASIS 750 answers differently from the plain mock; `None` for
/// the requests both answer alike.
fn route_750(state: &State, seg: &[&str], accept: &str) -> Option<Response> {
    let dictionary = |r: SourceRef| match state.provider.fetch(&r) {
        Ok(Some(s)) => Some(s.text),
        _ => None,
    };
    Some(match seg {
        ["discovery"] | ["core", "discovery"] => {
            let mut body = String::from(
                r#"<?xml version="1.0" encoding="utf-8"?><app:service xmlns:app="http://www.w3.org/2007/app" xmlns:atom="http://www.w3.org/2005/Atom"><app:workspace><atom:title>ABAP</atom:title>"#,
            );
            for (href, accepts) in DISCOVERY_750 {
                body.push_str(&format!(
                    r#"<app:collection href="/sap/bc/adt/{href}"><atom:title>{href}</atom:title>"#
                ));
                if !accepts.is_empty() {
                    body.push_str(&format!("<app:accept>{accepts}</app:accept>"));
                }
                body.push_str("</app:collection>");
            }
            body.push_str("</app:workspace></app:service>");
            Response::ok("application/atomsvc+xml", body)
        }
        ["system", "components"] => Response::ok(
            "application/atom+xml;type=feed",
            r#"<?xml version="1.0" encoding="utf-8"?><atom:feed xmlns:atom="http://www.w3.org/2005/Atom"><atom:title>Installed Components</atom:title><atom:entry><atom:id>SAP_ABA</atom:id><atom:title>750;SAPK-75035INSAPABA;0035;Cross-Application Component</atom:title></atom:entry><atom:entry><atom:id>SAP_BASIS</atom:id><atom:title>750;SAPK-75035INSAPBASIS;0035;SAP Basis Component</atom:title></atom:entry></atom:feed>"#.to_string(),
        ),
        ["ddic", "dataelements", name] => {
            let v1 = accept
                .split(',')
                .any(|a| a.trim() == "application/vnd.sap.adt.dataelements.v1+xml");
            if !v1 {
                return Some(Response::status(406));
            }
            match dictionary(SourceRef::data_element(name)) {
                Some(xml) => Response::ok(
                    "application/vnd.sap.adt.dataelements.v1+xml",
                    data_element_v1(name, &xml),
                ),
                None => Response::status(404),
            }
        }
        // structures yes, transparent tables not: 7.50 has no collection
        ["ddic", "structures", name, "source", "main"] => {
            match dictionary(SourceRef::table(name)) {
                Some(xml) if field(&xml, "TABCLASS").as_deref() == Some("INTTAB") => {
                    Response::ok("text/plain; charset=utf-8", xml)
                }
                _ => Response::status(404),
            }
        }
        ["ddic", "ddl", "sources", name, "source", "main"] => {
            match dictionary(SourceRef::data_definition(name)) {
                Some(text) => Response::ok("text/plain; charset=utf-8", text),
                None => Response::status(404),
            }
        }
        _ => return None,
    })
}

/// A data element in the XML of ADT, version 1, from its abapGit file: what
/// it is typed with.
fn data_element_v1(name: &str, xml: &str) -> String {
    let domain = field(xml, "DOMNAME").unwrap_or_default();
    let reference = (field(xml, "REFKIND"), field(xml, "REFTYPE"));
    let (kind, type_name) = match (reference.0.as_deref(), reference.1.as_deref()) {
        (Some("R"), Some("C" | "I")) => ("refToClifType", domain),
        (Some("R"), _) => ("refToDictionaryType", domain),
        _ if !domain.is_empty() => ("domain", domain),
        _ => ("predefinedAbapType", String::new()),
    };
    format!(
        r#"<?xml version="1.0" encoding="utf-8"?><blue:wbobj xmlns:blue="http://www.sap.com/wbobj/dictionary/dtel" xmlns:adtcore="http://www.sap.com/adt/core" adtcore:name="{}" adtcore:type="DTEL/DE"><dtel:dataElement xmlns:dtel="http://www.sap.com/adt/dictionary/dataelements"><dtel:typeKind>{kind}</dtel:typeKind><dtel:typeName>{}</dtel:typeName><dtel:dataType>{}</dtel:dataType></dtel:dataElement></blue:wbobj>"#,
        xml_escape(&name.to_ascii_uppercase()),
        xml_escape(&type_name),
        xml_escape(&field(xml, "DATATYPE").unwrap_or_default())
    )
}

/// The data preview: free-form ABAP SQL, or a `SELECT` on the dictionary
/// object the table preview is opened on.
fn preview(state: &State, path: &str, params: &HashMap<String, String>, sql: &str) -> Response {
    let Some(tables) = &state.tables else {
        return Response::status(404);
    };
    let max = params
        .get("rownumber")
        .and_then(|n| n.parse().ok())
        .unwrap_or(100);
    let answer = match path.strip_prefix(PREVIEW) {
        Some("freestyle") => tables.select(sql, None, max),
        Some("ddic") => match params.get("ddicentityname") {
            Some(entity) => tables.select(sql, Some(&entity.to_ascii_uppercase()), max),
            None => return Response::status(400),
        },
        _ => return Response::status(404),
    };
    match answer {
        Ok((columns, rows)) => {
            let mut body = format!(
                r#"<?xml version="1.0" encoding="utf-8"?><dataPreview:tableData xmlns:dataPreview="http://www.sap.com/adt/dataPreview"><dataPreview:totalRows>{}</dataPreview:totalRows>"#,
                rows.len()
            );
            for (i, column) in columns.iter().enumerate() {
                body.push_str(&format!(
                    r#"<dataPreview:columns><dataPreview:metadata dataPreview:name="{}" dataPreview:type="C"/><dataPreview:dataSet>"#,
                    xml_escape(column)
                ));
                for row in &rows {
                    match row[i].as_str() {
                        "" => body.push_str("<dataPreview:data/>"),
                        v => body.push_str(&format!(
                            "<dataPreview:data>{}</dataPreview:data>",
                            xml_escape(v)
                        )),
                    }
                }
                body.push_str("</dataPreview:dataSet></dataPreview:columns>");
            }
            body.push_str("</dataPreview:tableData>");
            Response::ok("application/vnd.sap.adt.datapreview.table.v1+xml", body)
        }
        Err(message) => Response {
            status: 400,
            content_type: "application/xml",
            body: format!(
                r#"<?xml version="1.0" encoding="utf-8"?><exc:exception xmlns:exc="http://www.sap.com/abapxml/types/communicationframework"><namespace id="com.sap.adt.dataPreview"/><type id="Exception"/><message lang="EN">{0}</message><localizedMessage lang="EN">{0}</localizedMessage></exc:exception>"#,
                xml_escape(&message)
            ),
            csrf: false,
            token_required: false,
        },
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
        token_required: false,
    }
}

/// The where-used list of an interface, `uri=/sap/bc/adt/oo/interfaces/<name>`:
/// the classes and programs whose source names it, like a system lists
/// implementations and users alike.
fn usage_references(state: &State, params: &HashMap<String, String>) -> Response {
    let uri = params.get("uri").cloned().unwrap_or_default();
    let Some(name) = uri.strip_prefix("/sap/bc/adt/oo/interfaces/") else {
        return Response::status(404);
    };
    let name =
        percent_decode(name.split(['#', '?']).next().unwrap_or_default()).to_ascii_uppercase();
    let mut found: Vec<(&'static str, String, String)> = Vec::new();
    let mut refs: Vec<&SourceRef> = state
        .provider
        .references()
        .filter(|r| r.part.is_none() && matches!(r.kind, ObjectKind::Clas | ObjectKind::Prog))
        .collect();
    refs.sort();
    for r in refs {
        let Ok(Some(source)) = state.provider.fetch(r) else {
            continue;
        };
        if !names(&source.text.to_ascii_uppercase(), &name) {
            continue;
        }
        let lower = percent_encode(&r.name.to_ascii_lowercase());
        found.push(match r.kind {
            ObjectKind::Clas => (
                "CLAS/OC",
                r.name.clone(),
                format!("/sap/bc/adt/oo/classes/{lower}"),
            ),
            _ => (
                "PROG/P",
                r.name.clone(),
                format!("/sap/bc/adt/programs/programs/{lower}"),
            ),
        });
    }
    let mut body = format!(
        r#"<?xml version="1.0" encoding="utf-8"?><usageReferences:usageReferenceResult xmlns:usageReferences="http://www.sap.com/adt/ris/usageReferences" xmlns:adtcore="http://www.sap.com/adt/core" numberOfResults="{}"><usageReferences:referencedObjects>"#,
        found.len()
    );
    // the package the objects are in comes first, as in a system's answer
    body.push_str(r#"<usageReferences:referencedObject uri="/sap/bc/adt/packages/%24tmp" isResult="false" canHaveChildren="true"><usageReferences:adtObject adtcore:name="$TMP" adtcore:type="DEVC/K"/></usageReferences:referencedObject>"#);
    for (kind, object, uri) in &found {
        body.push_str(&format!(
            r#"<usageReferences:referencedObject uri="{uri}" parentUri="/sap/bc/adt/packages/%24tmp" isResult="true" canHaveChildren="false" usageInformation="gradeDirect,includeProductive"><usageReferences:adtObject adtcore:name="{}" adtcore:type="{kind}" adtcore:responsible="DEVELOPER"><adtcore:packageRef adtcore:name="$TMP"/></usageReferences:adtObject></usageReferences:referencedObject>"#,
            xml_escape(object)
        ));
    }
    body.push_str("</usageReferences:referencedObjects></usageReferences:usageReferenceResult>");
    Response::ok(
        "application/vnd.sap.adt.repository.usagereferences.result.v1+xml",
        body,
    )
}

/// Whether `text` names `name` as a whole word.
fn names(text: &str, name: &str) -> bool {
    let word = |c: char| c.is_ascii_alphanumeric() || c == '_' || c == '/';
    text.match_indices(name).any(|(i, _)| {
        let before = text[..i].chars().next_back().is_none_or(|c| !word(c));
        let after = text[i + name.len()..]
            .chars()
            .next()
            .is_none_or(|c| !word(c));
        before && after
    })
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
            (
                "ztt_a.ttyp.xml",
                "<DD40V><TYPENAME>ZTT_A</TYPENAME><ROWTYPE>ZS_A</ROWTYPE><ROWKIND>S</ROWKIND></DD40V>",
            ),
            ("zabc.type.abap", "TYPE-POOL zabc.\nTYPES zabc_flag TYPE c LENGTH 1."),
            (
                "zcl_impl.clas.abap",
                "CLASS zcl_impl DEFINITION PUBLIC.\n  PUBLIC SECTION.\n    INTERFACES zif_a.\nENDCLASS.\nCLASS zcl_impl IMPLEMENTATION.\nENDCLASS.",
            ),
            (
                "zcl_user.clas.abap",
                "CLASS zcl_user DEFINITION PUBLIC.\n  PRIVATE SECTION.\n    DATA mo TYPE REF TO zif_a.\nENDCLASS.\nCLASS zcl_user IMPLEMENTATION.\nENDCLASS.",
            ),
            ("zuse.prog.abap", "REPORT zuse.\nDATA go TYPE REF TO zif_a."),
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
        assert!(text("TTYP:ZTT_A").contains("<ROWTYPE>ZS_A<"));
        assert!(text("TYPE:ZABC").contains("TYPE-POOL zabc"));
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
    fn where_used_list_of_an_interface() {
        let dir = folder();
        let server = server(&dir);
        let adt = provider(server.url(), "mock");
        // classes implementing or using it; the program is no class
        let classes = adt.implementations("zif_a").unwrap();
        assert_eq!(classes, ["ZCL_IMPL", "ZCL_USER"]);
        assert!(adt.implementations("ZIF_NONE").unwrap().is_empty());
        let requests = server.requests();
        assert!(requests[0].starts_with("/sap/bc/adt/discovery"));
        assert!(
            requests
                .iter()
                .any(|r| r
                    .contains("usageReferences?uri=%2Fsap%2Fbc%2Fadt%2Foo%2Finterfaces%2Fzif_a"))
        );
        // one session for both requests, answers cached
        assert_eq!(
            requests.iter().filter(|r| r.contains("discovery")).count(),
            1
        );
        let before = server.requests().len();
        adt.implementations("ZIF_A").unwrap();
        assert_eq!(server.requests().len(), before);
        drop(server);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn post_needs_token_and_session() {
        let dir = folder();
        let server = server(&dir);
        let mut stream = TcpStream::connect(server.url().trim_start_matches("http://")).unwrap();
        let auth = base64(b"DEVELOPER:mock");
        write!(
            stream,
            "POST /sap/bc/adt/repository/informationsystem/usageReferences?uri=%2Fsap%2Fbc%2Fadt%2Foo%2Finterfaces%2Fzif_a HTTP/1.1\r\nAuthorization: Basic {auth}\r\nX-CSRF-Token: {CSRF_TOKEN}\r\nContent-Length: 0\r\n\r\n"
        )
        .unwrap();
        let mut answer = String::new();
        stream.read_to_string(&mut answer).unwrap();
        // without the session cookie the token is not valid
        assert!(answer.starts_with("HTTP/1.1 403"), "{answer}");
        assert!(answer.contains("X-CSRF-Token: Required"));
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
    fn acts_like_basis_750() {
        let dir = folder();
        let files = [
            (
                "zspot.enhs.xml",
                "<BADI_DATA><ENH_BADI_DATA><BADI_NAME>ZBADI</BADI_NAME><INTERFACE_NAME>ZIF_A</INTERFACE_NAME>\
                 <DEFAULT_CLASS>ZCL_A</DEFAULT_CLASS></ENH_BADI_DATA></BADI_DATA>",
            ),
            (
                "zimpl.enho.xml",
                "<IMPL><ENH_BADI_IMPL_DATA><BADI_NAME>ZBADI</BADI_NAME><IMPL_CLASS>ZCL_IMPL</IMPL_CLASS>\
                 <ACTIVE>X</ACTIVE></ENH_BADI_IMPL_DATA></IMPL>",
            ),
            (
                "zexit.sxsd.xml",
                "<BADI><EXIT_NAME>ZEXIT</EXIT_NAME><INTER_NAME>ZIF_EX</INTER_NAME><DEF_CLNAME>ZCL_DEF</DEF_CLNAME></BADI>",
            ),
            (
                "zimp.sxci.xml",
                "<IMPLEMENTATION_DATA><IMP_CLASS>ZCL_IM</IMP_CLASS><EXIT_NAME>ZEXIT</EXIT_NAME>\
                 <INTER_NAME>ZIF_EX</INTER_NAME><ACTIVE>X</ACTIVE></IMPLEMENTATION_DATA>",
            ),
            (
                "zenh.enho.a1b2.abap",
                "\"Name: \\PR:ZREPORT\\EX:ZPOINT\\EI\nENHANCEMENT 1 zenh.\n  gv_x = 1.\nENDENHANCEMENT.",
            ),
            (
                "zid_ref.dtel.xml",
                "<DD04V><ROLLNAME>ZID_REF</ROLLNAME><DOMNAME>ZIF_A</DOMNAME><DATATYPE>REF</DATATYPE>\
                 <REFKIND>R</REFKIND><REFTYPE>I</REFTYPE></DD04V>",
            ),
            (
                "zst.tabl.xml",
                "<DD02V><TABNAME>ZST</TABNAME><TABCLASS>INTTAB</TABCLASS></DD02V>\
                 <DD03P><FIELDNAME>ID</FIELDNAME><ROLLNAME>ZID_REF</ROLLNAME></DD03P>",
            ),
            (
                "ztab.tabl.xml",
                "<DD02V><TABNAME>ZTAB</TABNAME><TABCLASS>TRANSP</TABCLASS></DD02V>",
            ),
        ];
        for (name, text) in files {
            std::fs::write(dir.join(name), text).unwrap();
        }
        let fs = FsProvider::new(vec![dir.clone()]).unwrap();
        let config = MockAdtConfig {
            release: MockRelease::Basis750,
            ..MockAdtConfig::default()
        };
        let server = MockAdtServer::start(fs, "127.0.0.1:0", config).unwrap();
        let adt = provider(server.url(), "mock");

        // the class relations, not the where-used list, which also names users
        assert_eq!(adt.implementations("ZIF_A").unwrap(), ["ZCL_IMPL"]);
        let badi = adt.badi("zbadi").unwrap().unwrap();
        assert_eq!(badi.implementations, ["ZCL_IMPL"]);
        assert_eq!(badi.fallback.as_deref(), Some("ZCL_A"));
        // the join of the classic BAdI tables
        let exit = adt.classic_badi("ZIF_EX").unwrap().unwrap();
        assert_eq!(exit.name, "ZEXIT");
        assert_eq!(exit.implementations, ["ZCL_IM"]);
        assert_eq!(exit.fallback.as_deref(), Some("ZCL_DEF"));

        // the system has the hook, but no way to read its code
        assert!(adt
            .enhancements(&["ZREPORT".to_string()], &[])
            .unwrap()
            .is_empty());
        assert_eq!(
            adt.notes(),
            ["enhancements: not available (the ADT discovery has no enhancements/enhoxhh to read their code, SAP_BASIS 750 SP 0035)"]
        );

        let text = |r: &str| adt.fetch(&r.parse().unwrap()).unwrap().map(|s| s.text);
        let dtel = text("DTEL:ZID_REF").unwrap();
        assert!(dtel.contains("<dtel:typeKind>refToClifType</dtel:typeKind><dtel:typeName>ZIF_A<"));
        assert!(text("TABL:ZST").unwrap().contains("ZID_REF"));
        assert_eq!(text("TABL:ZTAB"), None);

        let requests = server.requests();
        assert!(!requests.iter().any(|r| r.contains("usageReferences")));
        assert!(!requests.iter().any(|r| r.contains("enhoxhh")));
        assert!(requests.iter().any(|r| r.contains("datapreview/freestyle")));
        drop(server);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn data_elements_only_in_version_1_on_750() {
        let dir = folder();
        std::fs::write(
            dir.join("zd.dtel.xml"),
            "<DD04V><ROLLNAME>ZD</ROLLNAME><DOMNAME>ZDOM</DOMNAME></DD04V>",
        )
        .unwrap();
        let fs = FsProvider::new(vec![dir.clone()]).unwrap();
        let config = MockAdtConfig {
            release: MockRelease::Basis750,
            ..MockAdtConfig::default()
        };
        let server = MockAdtServer::start(fs, "127.0.0.1:0", config).unwrap();
        let get = |accept: &str| {
            let mut stream =
                TcpStream::connect(server.url().trim_start_matches("http://")).unwrap();
            let auth = base64(b"DEVELOPER:mock");
            write!(
                stream,
                "GET /sap/bc/adt/ddic/dataelements/zd HTTP/1.1\r\nAuthorization: Basic {auth}\r\nAccept: {accept}\r\n\r\n"
            )
            .unwrap();
            let mut answer = String::new();
            stream.read_to_string(&mut answer).unwrap();
            answer
        };
        assert!(get("application/vnd.sap.adt.dataelements.v2+xml").starts_with("HTTP/1.1 406"));
        let v1 = get("application/vnd.sap.adt.dataelements.v1+xml");
        assert!(v1.starts_with("HTTP/1.1 200"), "{v1}");
        assert!(v1.contains("<dtel:typeKind>domain</dtel:typeKind><dtel:typeName>ZDOM<"));
        assert_eq!("750".parse(), Ok(MockRelease::Basis750));
        assert!("7.50".parse::<MockRelease>().is_err());
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
