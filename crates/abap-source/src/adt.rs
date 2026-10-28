//! Sources from an SAP system through the ABAP Development Tools REST API,
//! the same endpoints Eclipse ADT and vscode_abap_remote_fs use. Only reads.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use base64::Engine;

use crate::{
    Badi, ClassPart, Hook, ObjectKind, Result, Source, SourceError, SourceProvider, SourceRef,
};

#[derive(Debug, Clone)]
pub struct AdtConfig {
    /// e.g. `https://sap.example.com:44300`
    pub url: String,
    pub user: String,
    pub password: String,
    /// `sap-client`, e.g. `100`
    pub client: Option<String>,
    /// `sap-language`, e.g. `EN`
    pub language: Option<String>,
    /// skip TLS certificate checks; only for test systems with self-signed certificates
    pub insecure: bool,
    pub timeout: Duration,
    pub proxy: AdtProxy,
}

/// How requests reach the SAP system.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum AdtProxy {
    /// through the proxy of `ALL_PROXY`, `HTTPS_PROXY` or `HTTP_PROXY`
    /// (also lower case, the first set), except for hosts in `NO_PROXY`;
    /// directly when none is set
    #[default]
    Environment,
    /// directly, whatever the environment says
    Off,
    /// through this proxy, like `http://proxy:8080`, for every host
    Url(String),
}

/// The variables a proxy is taken from, in the order ureq reads them.
const PROXY_VARIABLES: [&str; 6] = [
    "ALL_PROXY",
    "all_proxy",
    "HTTPS_PROXY",
    "https_proxy",
    "HTTP_PROXY",
    "http_proxy",
];

/// The proxy of `config`, and how errors name it: its address without
/// user and password, and where it comes from. An explicit proxy that
/// cannot be read is an error.
fn proxy_of(config: &AdtConfig) -> std::result::Result<(Option<ureq::Proxy>, String), String> {
    let name = |p: &ureq::Proxy| {
        let scheme = p.uri().scheme_str().unwrap_or("http");
        format!("{scheme}://{}:{}", p.host(), p.port())
    };
    match &config.proxy {
        AdtProxy::Off => Ok((None, String::new())),
        AdtProxy::Url(url) => {
            let p = ureq::Proxy::new(url)
                .map_err(|e| format!("the proxy given cannot be used: {e}"))?;
            let via = format!(" (through the proxy {})", name(&p));
            Ok((Some(p), via))
        }
        AdtProxy::Environment => {
            let Some(p) = ureq::Proxy::try_from_env() else {
                return Ok((None, String::new()));
            };
            let direct = config
                .url
                .parse::<ureq::http::Uri>()
                .is_ok_and(|target| p.is_no_proxy(&target));
            if direct {
                return Ok((Some(p), String::new()));
            }
            let variable = PROXY_VARIABLES
                .iter()
                .find(|v| std::env::var(v).is_ok_and(|value| ureq::Proxy::new(&value).is_ok()))
                .copied()
                .unwrap_or("the environment");
            let via = format!(
                " (through the proxy {} from {variable}; NO_PROXY, --no-proxy or proxy: false bypass it)",
                name(&p)
            );
            Ok((Some(p), via))
        }
    }
}

impl AdtConfig {
    pub fn new(url: &str, user: &str, password: &str) -> Self {
        AdtConfig {
            url: url.trim_end_matches('/').to_string(),
            user: user.to_string(),
            password: password.to_string(),
            client: None,
            language: None,
            insecure: false,
            timeout: Duration::from_secs(60),
            proxy: AdtProxy::Environment,
        }
    }
}

pub struct AdtProvider {
    config: AdtConfig,
    agent: ureq::Agent,
    authorization: String,
    /// how network errors name the proxy, like ` (through the proxy ...)`
    via: String,
    /// an explicit proxy that cannot be used: every request fails with it
    proxy_error: Option<String>,
    /// function module -> function group, from repository searches
    groups: Mutex<HashMap<String, Option<String>>>,
    /// CSRF token with the session cookies it belongs to, for POST requests
    session: Mutex<Option<(String, String)>>,
    /// interface -> classes using it, from where-used lists
    usages: Mutex<HashMap<String, Vec<String>>>,
    /// implementing classes per interface, upper case
    implementers: Mutex<HashMap<String, Vec<String>>>,
    /// subclasses per class, upper case
    subclasses: Mutex<HashMap<String, Vec<String>>>,
    /// BAdIs by name, classic ones by interface with a leading `if:`; one
    /// slot per key, so parallel loads asking for the same BAdI wait for
    /// one lookup
    badis: Mutex<HashMap<String, BadiSlot>>,
    /// enhancement implementation -> its code, URL and whether it is
    /// switched off
    hook_sources: Mutex<HashMap<String, Option<HookSource>>>,
    /// what the system offers, from the discovery, read once
    capabilities: Mutex<Option<Capabilities>>,
    /// lookups the system cannot do, each once, for the result
    notes: Mutex<Vec<String>>,
    /// `SAP_BASIS 750 SP 0035`, read once, for messages only
    release: Mutex<Option<String>>,
    /// write method, path, status, duration and SQL of every request to
    /// stderr, set by `ABAP_ADT_LOG=1`; never headers or credentials
    log: bool,
}

/// A BAdI looked up or being looked up: empty until the lookup is done.
type BadiSlot = Arc<Mutex<Option<Option<Badi>>>>;

/// The collections of the ADT discovery with the content types they accept.
/// Empty when the discovery could not be read: then everything is assumed.
#[derive(Clone, Default)]
struct Capabilities {
    collections: Vec<(String, Vec<String>)>,
}

impl Capabilities {
    fn from_discovery(xml: &str) -> Self {
        let collections = xml
            .split("<app:collection ")
            .skip(1)
            .filter_map(|chunk| {
                let href = chunk.split("href=\"").nth(1)?.split('"').next()?;
                let body = chunk.split("</app:collection>").next()?;
                let accepts = body
                    .split("<app:accept>")
                    .skip(1)
                    .filter_map(|a| a.split('<').next())
                    .map(str::to_string)
                    .collect();
                Some((href.to_string(), accepts))
            })
            .collect();
        Capabilities { collections }
    }

    fn known(&self) -> bool {
        !self.collections.is_empty()
    }

    /// Whether the collection exists; true when the discovery is unknown.
    fn offers(&self, href: &str) -> bool {
        !self.known() || self.collections.iter().any(|(h, _)| h == href)
    }

    fn accepts(&self, href: &str) -> Vec<&str> {
        self.collections
            .iter()
            .filter(|(h, _)| h == href)
            .flat_map(|(_, a)| a.iter().map(String::as_str))
            .collect()
    }
}

const FREESTYLE: &str = "/sap/bc/adt/datapreview/freestyle";
const PREVIEW_ACCEPT: &str = "application/xml, application/vnd.sap.adt.datapreview.table.v1+xml";

const TABLE_PREVIEW: &str = "/sap/bc/adt/datapreview/ddic";
const ENHANCEMENT_IMPLEMENTATIONS: &str = "/sap/bc/adt/enhancements/enhoxhh";
const DATA_ELEMENTS: &str = "/sap/bc/adt/ddic/dataelements";
const TABLE_TYPES: &str = "/sap/bc/adt/ddic/tabletypes";
const TYPE_GROUPS: &str = "/sap/bc/adt/ddic/typegroups";

/// The message of a rejected data preview query, on one line.
fn rejection(xml: &str) -> String {
    let message = xml
        .split("<message")
        .nth(1)
        .and_then(|m| m.split_once('>'))
        .and_then(|(_, m)| m.split("</message>").next())
        .unwrap_or(xml);
    let line = message.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut cut: String = line.chars().take(160).collect();
    if cut.len() < line.len() {
        cut.push_str("...");
    }
    cut
}

#[derive(Clone)]
struct HookSource {
    text: String,
    origin: String,
    switched_off: bool,
}

/// The block `ENHANCEMENT <id> ...` to `ENDENHANCEMENT.` of the code of an
/// enhancement implementation.
fn hook_block(source: &str, id: &str) -> Option<String> {
    let mut out: Option<Vec<&str>> = None;
    for line in source.lines() {
        let words: Vec<String> = line
            .split(|c: char| c.is_whitespace() || c == '.')
            .filter(|w| !w.is_empty())
            .map(|w| w.to_ascii_uppercase())
            .collect();
        match &mut out {
            None => {
                if words.first().is_some_and(|w| w == "ENHANCEMENT")
                    && words.get(1).map(String::as_str) == Some(id)
                {
                    out = Some(vec![line]);
                }
            }
            Some(lines) => {
                lines.push(line);
                if words.first().is_some_and(|w| w == "ENDENHANCEMENT") {
                    return Some(lines.join("\n"));
                }
            }
        }
    }
    None
}

/// The newest XML version of data elements the system lists, like
/// `application/vnd.sap.adt.dataelements.v2+xml`; the system also lists
/// `text/html`. v2 where the discovery says nothing.
fn data_element_type(accepts: &[&str]) -> String {
    const PREFIX: &str = "application/vnd.sap.adt.dataelements.v";
    accepts
        .iter()
        .filter_map(|a| {
            let version = a.strip_prefix(PREFIX)?.strip_suffix("+xml")?;
            Some((version.parse::<u32>().ok()?, a))
        })
        .max_by_key(|(version, _)| *version)
        .map_or_else(|| format!("{PREFIX}2+xml"), |(_, a)| a.to_string())
}

/// The hooks in the given programs. FULL_NAME is a long string and not
/// allowed in a condition on SAP_BASIS 750, so it is only read.
fn enhancement_query(programs: &[String]) -> String {
    // the data preview takes lines up to 255 characters
    let list = programs
        .iter()
        .map(|p| format!("'{p}'"))
        .collect::<Vec<_>>()
        .join(",\n");
    format!(
        "SELECT enhname, id, enhmode, full_name, programname FROM enhincinx \
         WHERE version = 'A' AND programname IN ( {list} )"
    )
}

/// The main program of a function group, `SAPL<group>` or `/NS/SAPL<group>`.
fn group_program(group: &str) -> String {
    match group.strip_prefix('/').and_then(|g| g.split_once('/')) {
        Some((ns, name)) => format!("/{ns}/SAPL{name}").to_ascii_uppercase(),
        None => format!("SAPL{group}").to_ascii_uppercase(),
    }
}

/// Whether a name may go into SQL: only what ABAP names consist of.
fn is_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '/')
}

/// The rows of a data preview answer, which lists the values per column.
fn preview_rows(xml: &str) -> Vec<Vec<String>> {
    let columns: Vec<Vec<String>> = xml
        .split("<dataPreview:dataSet>")
        .skip(1)
        .map(|set| {
            let set = set
                .split("</dataPreview:dataSet>")
                .next()
                .unwrap_or_default();
            set.split("<dataPreview:data")
                .skip(1)
                .map(|d| match d.strip_prefix('>') {
                    Some(v) => v.split('<').next().unwrap_or_default().trim().to_string(),
                    // an empty <dataPreview:data/>
                    None => String::new(),
                })
                .collect()
        })
        .collect();
    let n = columns.iter().map(Vec::len).max().unwrap_or(0);
    (0..n)
        .map(|i| {
            columns
                .iter()
                .map(|c| c.get(i).cloned().unwrap_or_default())
                .collect()
        })
        .collect()
}

/// The values of the first column, upper case, sorted, without empty ones.
fn first_column(rows: Vec<Vec<String>>) -> Vec<String> {
    let mut out: Vec<String> = rows
        .into_iter()
        .filter_map(|r| non_empty(r.first()))
        .collect();
    out.sort();
    out.dedup();
    out
}

fn non_empty(value: Option<&String>) -> Option<String> {
    value
        .map(|v| v.trim().to_ascii_uppercase())
        .filter(|v| !v.is_empty())
}

/// Lower case, '/' of namespaces percent encoded, as ADT URIs expect.
fn encode(name: &str) -> String {
    let mut out = String::new();
    for c in name.to_ascii_lowercase().chars() {
        match c {
            'a'..='z' | '0'..='9' | '_' | '-' | '.' => out.push(c),
            _ => {
                let mut buf = [0u8; 4];
                for b in c.encode_utf8(&mut buf).bytes() {
                    out.push_str(&format!("%{b:02x}"));
                }
            }
        }
    }
    out
}

fn decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(b) = u8::from_str_radix(&s[i + 1..i + 3], 16) {
                out.push(b);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

impl AdtProvider {
    pub fn new(config: AdtConfig) -> Self {
        use ureq::tls::{RootCerts, TlsConfig};
        // the platform verifier uses the OS certificate store, where
        // corporate CAs of SAP systems usually are
        let tls = TlsConfig::builder()
            .root_certs(RootCerts::PlatformVerifier)
            .disable_verification(config.insecure)
            .build();
        let (proxy, via, proxy_error) = match proxy_of(&config) {
            Ok((proxy, via)) => (proxy, via, None),
            Err(e) => (None, String::new(), Some(e)),
        };
        let agent: ureq::Agent = ureq::Agent::config_builder()
            .tls_config(tls)
            .proxy(proxy)
            .http_status_as_error(false)
            .timeout_global(Some(config.timeout))
            .build()
            .into();
        let credentials = base64::engine::general_purpose::STANDARD
            .encode(format!("{}:{}", config.user, config.password));
        AdtProvider {
            config,
            agent,
            authorization: format!("Basic {credentials}"),
            via,
            proxy_error,
            groups: Mutex::new(HashMap::new()),
            session: Mutex::new(None),
            usages: Mutex::new(HashMap::new()),
            implementers: Mutex::new(HashMap::new()),
            subclasses: Mutex::new(HashMap::new()),
            badis: Mutex::new(HashMap::new()),
            hook_sources: Mutex::new(HashMap::new()),
            capabilities: Mutex::new(None),
            notes: Mutex::new(Vec::new()),
            release: Mutex::new(None),
            log: std::env::var("ABAP_ADT_LOG").is_ok_and(|v| !v.is_empty() && v != "0"),
        }
    }

    /// Like [`new`](Self::new), but an explicit proxy that cannot be used
    /// is an error here, not on each request.
    pub fn try_new(config: AdtConfig) -> Result<Self> {
        let adt = Self::new(config);
        match &adt.proxy_error {
            Some(e) => Err(SourceError::Other(e.clone())),
            None => Ok(adt),
        }
    }

    /// A request that got no answer, naming the proxy it went through.
    fn network(&self, url: &str, e: impl std::fmt::Display) -> SourceError {
        SourceError::Network(format!("{url}: {e}{}", self.via))
    }

    /// The error of a proxy that cannot be used, before any request.
    fn usable(&self) -> Result<()> {
        match &self.proxy_error {
            Some(e) => Err(SourceError::Other(e.clone())),
            None => Ok(()),
        }
    }

    fn log_request(
        &self,
        method: &str,
        path: &str,
        status: u16,
        since: Instant,
        sql: Option<&str>,
    ) {
        if self.log {
            let ms = since.elapsed().as_millis();
            let sql = sql
                .map(|q| format!(" sql: {}", q.replace('\n', " ")))
                .unwrap_or_default();
            eprintln!("adt {method} {path} {status} {ms}ms{sql}");
        }
    }

    /// The discovery, once. A system that does not answer it is assumed to
    /// offer everything, as before.
    fn capabilities(&self) -> Capabilities {
        if let Some(c) = self.capabilities.lock().unwrap().clone() {
            return c;
        }
        // the discovery that opens the session also tells what is offered
        {
            let mut session = self.session.lock().unwrap();
            if session.is_none() {
                if let Ok(opened) = self.fetch_session() {
                    *session = Some(opened);
                }
            }
        }
        self.capabilities
            .lock()
            .unwrap()
            .get_or_insert_with(Capabilities::default)
            .clone()
    }

    /// `SAP_BASIS 750 SP 0035`, for messages only; empty when unknown.
    fn release(&self) -> String {
        let mut slot = self.release.lock().unwrap();
        if let Some(r) = slot.as_ref() {
            return r.clone();
        }
        let xml = self
            .get("/sap/bc/adt/system/components", "*/*")
            .ok()
            .flatten()
            .unwrap_or_default();
        let release = xml
            .split("<atom:entry>")
            .find(|e| e.contains("<atom:id>SAP_BASIS</atom:id>"))
            .and_then(|e| e.split("<atom:title>").nth(1))
            .and_then(|t| t.split('<').next())
            .map(|t| {
                let mut parts = t.split(';');
                let release = parts.next().unwrap_or_default();
                let level = parts.nth(1).unwrap_or_default();
                format!("SAP_BASIS {release} SP {level}")
            })
            .unwrap_or_default();
        *slot = Some(release.clone());
        release
    }

    /// Remembers that a lookup is not available, and says why.
    fn note(&self, lookup: &str, reason: &str) {
        let release = self.release();
        let suffix = if release.is_empty() {
            String::new()
        } else {
            format!(", {release}")
        };
        let text = format!("{lookup}: not available ({reason}{suffix})");
        let mut notes = self.notes.lock().unwrap();
        if !notes.contains(&text) {
            notes.push(text);
        }
    }

    fn url(&self, path: &str) -> String {
        let mut url = format!("{}{}", self.config.url, path);
        let mut params = Vec::new();
        if let Some(c) = &self.config.client {
            params.push(format!("sap-client={c}"));
        }
        if let Some(l) = &self.config.language {
            params.push(format!("sap-language={l}"));
        }
        if !params.is_empty() {
            url.push(if url.contains('?') { '&' } else { '?' });
            url.push_str(&params.join("&"));
        }
        url
    }

    /// GET returning the body, `None` on 404.
    fn get(&self, path: &str, accept: &str) -> Result<Option<String>> {
        self.usable()?;
        let url = self.url(path);
        let started = Instant::now();
        let mut response = self
            .agent
            .get(&url)
            .header("Authorization", &self.authorization)
            .header("Accept", accept)
            .call()
            .map_err(|e| self.network(&url, e))?;
        let status = response.status().as_u16();
        self.log_request("GET", path, status, started, None);
        match status {
            200 => response
                .body_mut()
                .read_to_string()
                .map(Some)
                .map_err(|e| self.network(&url, e)),
            404 => Ok(None),
            401 | 403 => Err(SourceError::Auth { url }),
            _ => Err(SourceError::Http { status, url }),
        }
    }

    /// A CSRF token and the cookies of its session.
    fn fetch_session(&self) -> Result<(String, String)> {
        self.usable()?;
        let url = self.url("/sap/bc/adt/discovery");
        let started = Instant::now();
        let mut response = self
            .agent
            .get(&url)
            .header("Authorization", &self.authorization)
            .header("X-CSRF-Token", "Fetch")
            .header("Accept", "application/atomsvc+xml")
            .call()
            .map_err(|e| self.network(&url, e))?;
        let status = response.status().as_u16();
        self.log_request("GET", "/sap/bc/adt/discovery", status, started, None);
        match status {
            200 => {}
            401 | 403 => return Err(SourceError::Auth { url }),
            _ => return Err(SourceError::Http { status, url }),
        }
        let headers = response.headers();
        let token = headers
            .get("x-csrf-token")
            .and_then(|v| v.to_str().ok())
            .unwrap_or_default()
            .to_string();
        let cookies = headers
            .get_all("set-cookie")
            .iter()
            .filter_map(|v| v.to_str().ok())
            .filter_map(|c| c.split(';').next())
            .map(str::trim)
            .collect::<Vec<_>>()
            .join("; ");
        // the same answer tells what the system offers
        let xml = response.body_mut().read_to_string().unwrap_or_default();
        self.capabilities
            .lock()
            .unwrap()
            .get_or_insert_with(|| Capabilities::from_discovery(&xml));
        Ok((token, cookies))
    }

    /// POST with a CSRF token, returning the body; `None` on 404. The token
    /// is fetched once and again when the system asks for a new one.
    fn post(
        &self,
        path: &str,
        content_type: &str,
        accept: &str,
        body: &str,
    ) -> Result<Option<String>> {
        let (status, text) = self.post_reply(path, content_type, accept, body)?;
        match status {
            200 => Ok(Some(text)),
            404 => Ok(None),
            401 | 403 => Err(SourceError::Auth {
                url: self.url(path),
            }),
            _ => Err(SourceError::Http {
                status,
                url: self.url(path),
            }),
        }
    }

    /// The status and body of a POST, whatever the status; the token is
    /// renewed once when the system asks for it.
    fn post_reply(
        &self,
        path: &str,
        content_type: &str,
        accept: &str,
        body: &str,
    ) -> Result<(u16, String)> {
        let url = self.url(path);
        let started = Instant::now();
        // the data preview logs its SQL
        let sql = path.starts_with("/sap/bc/adt/datapreview/").then_some(body);
        for attempt in 0..2 {
            let (token, cookies) = {
                let mut session = self.session.lock().unwrap();
                if session.is_none() || attempt > 0 {
                    *session = Some(self.fetch_session()?);
                }
                session.clone().unwrap_or_default()
            };
            let mut request = self
                .agent
                .post(&url)
                .header("Authorization", &self.authorization)
                .header("Content-Type", content_type)
                .header("Accept", accept)
                .header("X-CSRF-Token", &token);
            if !cookies.is_empty() {
                request = request.header("Cookie", &cookies);
            }
            let mut response = request.send(body).map_err(|e| self.network(&url, e))?;
            let status = response.status().as_u16();
            self.log_request("POST", path, status, started, sql);
            let renew = response
                .headers()
                .get("x-csrf-token")
                .and_then(|v| v.to_str().ok())
                .is_some_and(|v| v.eq_ignore_ascii_case("required"));
            if status == 403 && renew && attempt == 0 {
                continue;
            }
            let text = response
                .body_mut()
                .read_to_string()
                .map_err(|e| self.network(&url, e))?;
            return Ok((status, text));
        }
        Err(SourceError::Auth { url })
    }

    /// Classes in the where-used list of an interface: those implementing it
    /// and those only using it.
    pub fn interface_usages(&self, interface: &str) -> Result<Vec<String>> {
        let key = interface.to_ascii_uppercase();
        if let Some(u) = self.usages.lock().unwrap().get(&key) {
            return Ok(u.clone());
        }
        let uri = format!("/sap/bc/adt/oo/interfaces/{}", encode(&key));
        let path = format!(
            "/sap/bc/adt/repository/informationsystem/usageReferences?uri={}",
            query_encode(&uri)
        );
        let xml = self
            .post(
                &path,
                "application/vnd.sap.adt.repository.usagereferences.request.v1+xml",
                "application/vnd.sap.adt.repository.usagereferences.result.v1+xml",
                USAGE_REQUEST,
            )?
            .unwrap_or_default();
        let classes = classes_in_usages(&xml);
        self.usages.lock().unwrap().insert(key, classes.clone());
        Ok(classes)
    }

    /// Classes implementing an interface directly, from the class relations
    /// through the data preview; `None` when the system has no data preview.
    pub fn implementing_classes(&self, interface: &str) -> Result<Option<Vec<String>>> {
        self.related_classes(interface, Relation::Implements)
    }

    /// Classes with a relation to `name` in the class relations (SEOMETAREL)
    /// read through the data preview; `None` when the system has no data
    /// preview.
    fn related_classes(&self, name: &str, relation: Relation) -> Result<Option<Vec<String>>> {
        let key = name.to_ascii_uppercase();
        if !is_name(&key) {
            return Ok(Some(Vec::new()));
        }
        let query = format!(
            "SELECT clsname FROM seometarel WHERE refclsname = '{key}' AND reltype = '{}' AND version = '1'",
            relation as u8
        );
        let rows = self.select("class relations", &query, || {
            self.table_select("SEOMETAREL", &query)
        })?;
        Ok(rows.map(first_column))
    }

    /// Rows of an ABAP SQL query through the data preview. When the system
    /// has no free-form SQL or rejects the query, `fallback` gets a try,
    /// usually the same lookup as single-table queries through the table
    /// preview. `None` and a note in the result when nothing works.
    fn select(
        &self,
        lookup: &str,
        query: &str,
        fallback: impl FnOnce() -> Result<Option<Vec<Vec<String>>>>,
    ) -> Result<Option<Vec<Vec<String>>>> {
        let mut reason = "the system has no data preview".to_string();
        if self.capabilities().offers(FREESTYLE) {
            let path = format!("{FREESTYLE}?rowNumber=1000");
            let (status, text) = self.post_reply(&path, "text/plain", PREVIEW_ACCEPT, query)?;
            match status {
                200 => return Ok(Some(preview_rows(&text))),
                401 | 403 => {
                    return Err(SourceError::Auth {
                        url: self.url(&path),
                    })
                }
                400 => {
                    reason = format!("the data preview rejects the query: {}", rejection(&text));
                }
                404 => {}
                _ => {
                    return Err(SourceError::Http {
                        status,
                        url: self.url(&path),
                    })
                }
            }
        }
        if let Some(rows) = fallback()? {
            return Ok(Some(rows));
        }
        self.note(lookup, &reason);
        Ok(None)
    }

    /// Rows of a full `SELECT` on one table through the table preview;
    /// `None` when the system has none or does not answer.
    fn table_select(&self, table: &str, query: &str) -> Result<Option<Vec<Vec<String>>>> {
        if !self.capabilities().offers(TABLE_PREVIEW) || !is_name(table) {
            return Ok(None);
        }
        let path = format!("{TABLE_PREVIEW}?rowNumber=1000&ddicEntityName={table}");
        let (status, text) = self.post_reply(&path, "text/plain", PREVIEW_ACCEPT, query)?;
        match status {
            200 => Ok(Some(preview_rows(&text))),
            401 | 403 => Err(SourceError::Auth {
                url: self.url(&path),
            }),
            _ => Ok(None),
        }
    }

    /// A table type from its dictionary table DD40L through the data
    /// preview, as abapGit writes it (`DD40V`), for systems whose ADT has
    /// no table types.
    fn table_type_by_table(&self, r: &SourceRef) -> Result<Option<Source>> {
        let name = r.name.to_ascii_uppercase();
        if !is_name(&name) {
            return Ok(None);
        }
        let query = format!(
            "SELECT rowtype, rowkind, datatype, leng, decimals FROM dd40l              WHERE typename = '{name}' AND as4local = 'A'"
        );
        let rows = match self.select("table types", &query, || self.table_select("DD40L", &query)) {
            Ok(rows) => rows.unwrap_or_default(),
            Err(SourceError::Http { .. }) => return Ok(None),
            Err(e) => return Err(e),
        };
        let Some(row) = rows.first() else {
            return Ok(None);
        };
        let mut text = format!("<DD40V><TYPENAME>{name}</TYPENAME>");
        for (tag, value) in ["ROWTYPE", "ROWKIND", "DATATYPE", "LENG", "DECIMALS"]
            .iter()
            .zip(row)
        {
            // empty values stay out, like abapGit leaves them out
            if !value.is_empty() {
                text.push_str(&format!("<{tag}>{value}</{tag}>"));
            }
        }
        text.push_str("</DD40V>");
        Ok(Some(Source {
            reference: r.clone(),
            text,
            origin: format!("{}{FREESTYLE} (DD40L {name})", self.config.url),
        }))
    }

    /// A BAdI of an enhancement spot: interface and fallback class from its
    /// definition, the classes of its active implementations.
    fn spot_badi(&self, name: &str) -> Result<Option<Badi>> {
        let query = format!(
            "SELECT interface_name, default_class FROM badi_main WHERE badi_name = '{name}'"
        );
        let definition = self
            .select("BAdI definitions", &query, || {
                self.table_select("BADI_MAIN", &query)
            })?
            .unwrap_or_default();
        let Some(row) = definition.first() else {
            return Ok(None);
        };
        Ok(Some(Badi {
            name: name.to_string(),
            interface: non_empty(row.first()),
            implementations: self.badi_implementations(name)?,
            fallback: non_empty(row.get(1)),
            classic: false,
        }))
    }

    /// Classes of the active implementations of a BAdI of an enhancement
    /// spot: the runtime table only has those.
    fn badi_implementations(&self, name: &str) -> Result<Vec<String>> {
        let query = format!("SELECT class_name FROM badi_impl WHERE badi_name = '{name}'");
        Ok(self
            .select("BAdI implementations", &query, || {
                self.table_select("BADI_IMPL", &query)
            })?
            .map(first_column)
            .unwrap_or_default())
    }

    /// A classic BAdI by its interface with its active implementations;
    /// those of the BAdI it was migrated to count too.
    fn exit_badi(&self, interface: &str) -> Result<Option<Badi>> {
        let joined = format!(
            "SELECT x~exit_name, a~def_clname, a~mig_badi_name FROM sxs_inter AS x \
             INNER JOIN sxs_attr AS a ON a~exit_name = x~exit_name \
             WHERE x~inter_name = '{interface}'"
        );
        let definition = self
            .select("classic BAdI definitions", &joined, || {
                self.exit_definition_by_tables(interface)
            })?
            .unwrap_or_default();
        let Some(row) = definition.first() else {
            return Ok(None);
        };
        let joined = format!(
            "SELECT c~imp_class FROM sxc_class AS c INNER JOIN sxc_attr AS a \
             ON a~imp_name = c~imp_name WHERE c~inter_name = '{interface}' AND a~active = 'X'"
        );
        let mut implementations = self
            .select("classic BAdI implementations", &joined, || {
                self.exit_implementations_by_tables(interface)
            })?
            .map(first_column)
            .unwrap_or_default();
        if let Some(migrated) = non_empty(row.get(2)).filter(|m| is_name(m)) {
            implementations.extend(self.badi_implementations(&migrated)?);
            implementations.sort();
            implementations.dedup();
        }
        Ok(Some(Badi {
            name: non_empty(row.first()).unwrap_or_default(),
            interface: Some(interface.to_string()),
            implementations,
            fallback: non_empty(row.get(1)),
            classic: true,
        }))
    }

    /// The join of the classic BAdI definition as two single-table queries
    /// through the table preview.
    fn exit_definition_by_tables(&self, interface: &str) -> Result<Option<Vec<Vec<String>>>> {
        let Some(exits) = self.table_select(
            "SXS_INTER",
            &format!("SELECT exit_name FROM sxs_inter WHERE inter_name = '{interface}'"),
        )?
        else {
            return Ok(None);
        };
        let Some(exit) = first_column(exits).into_iter().find(|e| is_name(e)) else {
            return Ok(Some(Vec::new()));
        };
        self.table_select(
            "SXS_ATTR",
            &format!(
                "SELECT exit_name, def_clname, mig_badi_name FROM sxs_attr WHERE exit_name = '{exit}'"
            ),
        )
    }

    /// The join of the active classic BAdI implementations as single-table
    /// queries through the table preview.
    fn exit_implementations_by_tables(&self, interface: &str) -> Result<Option<Vec<Vec<String>>>> {
        let Some(classes) = self.table_select(
            "SXC_CLASS",
            &format!("SELECT imp_name, imp_class FROM sxc_class WHERE inter_name = '{interface}'"),
        )?
        else {
            return Ok(None);
        };
        let mut out = Vec::new();
        for row in classes {
            let (Some(imp), Some(class)) = (non_empty(row.first()), non_empty(row.get(1))) else {
                continue;
            };
            if !is_name(&imp) {
                continue;
            }
            let active = self.table_select(
                "SXC_ATTR",
                &format!("SELECT imp_name FROM sxc_attr WHERE imp_name = '{imp}' AND active = 'X'"),
            )?;
            if active.is_some_and(|rows| !rows.is_empty()) {
                out.push(vec![class]);
            }
        }
        Ok(Some(out))
    }

    /// The code of an enhancement implementation with its URL and whether
    /// it is switched off.
    fn hook_source(&self, name: &str) -> Result<Option<HookSource>> {
        if let Some(s) = self.hook_sources.lock().unwrap().get(name) {
            return Ok(s.clone());
        }
        let path = format!("/sap/bc/adt/enhancements/enhoxhh/{}", encode(name));
        let metadata = self.get(&path, "*/*")?.unwrap_or_default();
        let switched_off = metadata.split("<enho:switch ").nth(1).is_some_and(|s| {
            s.split('>')
                .next()
                .unwrap_or_default()
                .contains("state=\"off\"")
        });
        let main = format!("{path}/source/main");
        let source = self.get(&main, "text/plain")?.map(|text| HookSource {
            text,
            origin: format!("{}{main}", self.config.url),
            switched_off,
        });
        self.hook_sources
            .lock()
            .unwrap()
            .insert(name.to_string(), source.clone());
        Ok(source)
    }

    /// A BAdI looked up once; systems without data preview have none.
    fn cached_badi(
        &self,
        key: String,
        lookup: impl Fn(&str) -> Result<Option<Badi>>,
    ) -> Result<Option<Badi>> {
        let slot = self
            .badis
            .lock()
            .unwrap()
            .entry(key.clone())
            .or_default()
            .clone();
        // held during the lookup: others asking for the same key wait
        let mut slot = slot.lock().unwrap();
        if let Some(b) = slot.as_ref() {
            return Ok(b.clone());
        }
        let name = key.trim_start_matches("if:");
        let badi = if !is_name(name) {
            None
        } else {
            match lookup(name) {
                Ok(b) => b,
                Err(SourceError::Auth { .. }) | Err(SourceError::Http { .. }) => None,
                Err(e) => return Err(e),
            }
        };
        *slot = Some(badi.clone());
        Ok(badi)
    }

    fn source(&self, reference: &SourceRef, path: &str) -> Result<Option<Source>> {
        Ok(self.get(path, "text/plain")?.map(|text| Source {
            reference: reference.clone(),
            text,
            origin: format!("{}{}", self.config.url, path),
        }))
    }

    /// Function group of a function module, from the repository search.
    pub fn function_group(&self, function_module: &str) -> Result<Option<String>> {
        let key = function_module.to_ascii_uppercase();
        if let Some(g) = self.groups.lock().unwrap().get(&key) {
            return Ok(g.clone());
        }
        let path = format!(
            "/sap/bc/adt/repository/informationsystem/search?operation=quickSearch&query={}&maxResults=10&objectType=FUGR/FF",
            encode(&key).to_ascii_uppercase().replace("%2F", "%2f")
        );
        let xml = self.get(&path, "application/xml")?.unwrap_or_default();
        let mut group = group_from_search(&xml, &key);
        // lock modules are generated and not in the search; their lock
        // object names them with their group
        if let (None, Some(lock_object)) = (
            &group,
            key.strip_prefix("ENQUEUE_")
                .or_else(|| key.strip_prefix("DEQUEUE_")),
        ) {
            let path = format!(
                "/sap/bc/adt/ddic/lockobjects/sources/{}",
                encode(lock_object)
            );
            if let Some(xml) = self.get(&path, "application/vnd.sap.adt.lockobjects.v1+xml")? {
                group = group_from_search(&xml, &key);
            }
        }
        self.groups.lock().unwrap().insert(key, group.clone());
        Ok(group)
    }
}

const USAGE_REQUEST: &str = r#"<?xml version="1.0" encoding="UTF-8"?><usagereferences:usageReferenceRequest xmlns:usagereferences="http://www.sap.com/adt/ris/usageReferences"><usagereferences:affectedObjects/></usagereferences:usageReferenceRequest>"#;

/// Relation types of the class relations, SEOMETAREL-RELTYPE.
#[derive(Clone, Copy)]
enum Relation {
    Implements = 1,
    Inherits = 2,
}

/// Percent encoding of a query parameter value.
fn query_encode(value: &str) -> String {
    let mut out = String::new();
    for b in value.bytes() {
        if b.is_ascii_alphanumeric() || b"_-.~".contains(&b) {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

/// Names of the global classes (`CLAS/OC`) of a where-used result.
fn classes_in_usages(xml: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for tag in xml.split('<') {
        if !tag.contains(r#"adtcore:type="CLAS/OC""#) {
            continue;
        }
        let Some(rest) = tag.split(r#"adtcore:name=""#).nth(1) else {
            continue;
        };
        let name = rest
            .split('"')
            .next()
            .unwrap_or_default()
            .to_ascii_uppercase();
        if !name.is_empty() && !out.contains(&name) {
            out.push(name);
        }
    }
    out
}

/// Finds `.../functions/groups/<group>/fmodules/<fm>` in the search result.
fn group_from_search(xml: &str, function_module: &str) -> Option<String> {
    for part in xml.split("/functions/groups/").skip(1) {
        let Some((group, rest)) = part.split_once("/fmodules/") else {
            continue;
        };
        let fm = rest.split(['"', '/', '?', '#']).next().unwrap_or("");
        if decode(fm).eq_ignore_ascii_case(function_module) {
            return Some(decode(group).to_ascii_uppercase());
        }
    }
    None
}

impl SourceProvider for AdtProvider {
    fn fetch(&self, r: &SourceRef) -> Result<Option<Source>> {
        let name = encode(&r.name);
        match (r.kind, r.part) {
            (ObjectKind::Prog, _) => {
                if let Some(s) = self.source(
                    r,
                    &format!("/sap/bc/adt/programs/programs/{name}/source/main"),
                )? {
                    return Ok(Some(s));
                }
                self.source(
                    r,
                    &format!("/sap/bc/adt/programs/includes/{name}/source/main"),
                )
            }
            (ObjectKind::Clas, None) => {
                self.source(r, &format!("/sap/bc/adt/oo/classes/{name}/source/main"))
            }
            (ObjectKind::Clas, Some(part)) => {
                let include = match part {
                    ClassPart::Definitions => "definitions",
                    ClassPart::Implementations => "implementations",
                    ClassPart::Macros => "macros",
                    ClassPart::TestClasses => "testclasses",
                };
                self.source(
                    r,
                    &format!("/sap/bc/adt/oo/classes/{name}/includes/{include}"),
                )
            }
            (ObjectKind::Intf, _) => {
                self.source(r, &format!("/sap/bc/adt/oo/interfaces/{name}/source/main"))
            }
            (ObjectKind::Fugr, _) => self.source(
                r,
                &format!("/sap/bc/adt/functions/groups/{name}/source/main"),
            ),
            (ObjectKind::Func, _) => {
                let Some(group) = self.function_group(&r.name)? else {
                    return Ok(None);
                };
                let group = encode(&group);
                self.source(
                    r,
                    &format!("/sap/bc/adt/functions/groups/{group}/fmodules/{name}/source/main"),
                )
            }
            (ObjectKind::Tabl, _) => {
                if let Some(s) = self.source(
                    r,
                    &format!("/sap/bc/adt/ddic/structures/{name}/source/main"),
                )? {
                    return Ok(Some(s));
                }
                self.source(r, &format!("/sap/bc/adt/ddic/tables/{name}/source/main"))
            }
            (ObjectKind::Ddls, _) => self.source(
                r,
                &format!("/sap/bc/adt/ddic/ddl/sources/{name}/source/main"),
            ),
            // only where the discovery lists them
            (ObjectKind::Ttyp, _) => {
                let capabilities = self.capabilities();
                if !capabilities.offers(TABLE_TYPES) {
                    return self.table_type_by_table(r);
                }
                let path = format!("{TABLE_TYPES}/{name}");
                let accept = capabilities
                    .accepts(TABLE_TYPES)
                    .into_iter()
                    .find(|a| a.ends_with("+xml"))
                    .unwrap_or("application/vnd.sap.adt.tabletype.v1+xml")
                    .to_string();
                Ok(self.get(&path, &accept)?.map(|text| Source {
                    reference: r.clone(),
                    text,
                    origin: format!("{}{}", self.config.url, path),
                }))
            }
            (ObjectKind::Type, _) => {
                if !self.capabilities().offers(TYPE_GROUPS) {
                    return Ok(None);
                }
                self.source(r, &format!("{TYPE_GROUPS}/{name}/source/main"))
            }
            // a data element brings the type of its domain along
            (ObjectKind::Doma, _) => Ok(None),
            (ObjectKind::Dtel, _) => {
                let path = format!("/sap/bc/adt/ddic/dataelements/{name}");
                let accept = data_element_type(&self.capabilities().accepts(DATA_ELEMENTS));
                Ok(self.get(&path, &accept)?.map(|text| Source {
                    reference: r.clone(),
                    text,
                    origin: format!("{}{}", self.config.url, path),
                }))
            }
        }
    }

    /// From the class relations through the data preview; nothing where
    /// that is not allowed.
    fn subclasses(&self, class: &str) -> Result<Vec<String>> {
        let key = class.to_ascii_uppercase();
        if let Some(c) = self.subclasses.lock().unwrap().get(&key) {
            return Ok(c.clone());
        }
        let classes = match self.related_classes(&key, Relation::Inherits) {
            Ok(classes) => classes.unwrap_or_default(),
            Err(SourceError::Auth { .. }) | Err(SourceError::Http { .. }) => Vec::new(),
            Err(e) => return Err(e),
        };
        self.subclasses.lock().unwrap().insert(key, classes.clone());
        Ok(classes)
    }

    /// The classes the class relations (SEOMETAREL) name, read through the
    /// data preview; where that is not allowed, the where-used list of the
    /// interface, a superset which callers check against the class sources.
    fn implementations(&self, interface: &str) -> Result<Vec<String>> {
        let key = interface.to_ascii_uppercase();
        if let Some(c) = self.implementers.lock().unwrap().get(&key) {
            return Ok(c.clone());
        }
        let classes = match self.implementing_classes(&key) {
            Ok(Some(classes)) => classes,
            Ok(None) | Err(SourceError::Auth { .. }) | Err(SourceError::Http { .. }) => {
                self.interface_usages(&key)?
            }
            Err(e) => return Err(e),
        };
        self.implementers
            .lock()
            .unwrap()
            .insert(key, classes.clone());
        Ok(classes)
    }

    /// From the index of enhancement includes (ENHINCINX) through the data
    /// preview, the code from each enhancement implementation.
    fn enhancements(&self, programs: &[String], function_modules: &[String]) -> Result<Vec<Hook>> {
        // without the code of the implementations the hooks are of no use
        if !self.capabilities().offers(ENHANCEMENT_IMPLEMENTATIONS) {
            self.note(
                "enhancements",
                "the ADT discovery has no enhancements/enhoxhh to read their code",
            );
            return Ok(Vec::new());
        }
        // names go into SQL: ABAP names, class pools and their includes
        let valid = |n: &&String| {
            !n.is_empty()
                && n.chars()
                    .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '/' | '='))
        };
        let mut wanted: Vec<String> = programs
            .iter()
            .filter(valid)
            .map(|p| p.to_ascii_uppercase())
            .collect();
        let named = wanted.clone();
        // Hooks of a function module sit under the main program of its
        // function group. FULL_NAME is a long string, which ABAP SQL does
        // not allow in WHERE on SAP_BASIS 750, so the module is matched here.
        let mut prefixes: Vec<String> = Vec::new();
        for module in function_modules.iter().filter(valid) {
            let module = module.to_ascii_uppercase();
            match self.function_group(&module) {
                Ok(Some(group)) => {
                    wanted.push(group_program(&group));
                    prefixes.push(format!("\\FU:{module}\\"));
                }
                Ok(None) | Err(SourceError::Http { .. }) => {}
                Err(SourceError::Auth { .. }) => return Ok(Vec::new()),
                Err(e) => return Err(e),
            }
        }
        wanted.sort();
        wanted.dedup();
        let mut rows = Vec::new();
        for chunk in wanted.chunks(50) {
            let query = enhancement_query(chunk);
            match self.select("enhancements", &query, || {
                self.table_select("ENHINCINX", &query)
            }) {
                Ok(r) => rows.extend(r.unwrap_or_default()),
                // other answers mean the system has no data preview for us
                Err(SourceError::Http { .. }) | Err(SourceError::Auth { .. }) => {
                    return Ok(Vec::new())
                }
                Err(e) => return Err(e),
            }
        }
        rows.retain(|row| {
            let (Some(position), Some(program)) = (row.get(3), row.get(4)) else {
                return false;
            };
            let position = position.trim().to_ascii_uppercase();
            named.iter().any(|n| n.eq_ignore_ascii_case(program.trim()))
                || prefixes.iter().any(|p| position.starts_with(p.as_str()))
        });
        let mut out: Vec<Hook> = Vec::new();
        for row in rows {
            let [name, id, mode, position, _program] = &row[..] else {
                continue;
            };
            let name = name.trim().to_ascii_uppercase();
            // enhancements of a function module's parameters are no code
            if !is_name(&name)
                || position.matches(r"\SE:").count() > 1
                || out
                    .iter()
                    .any(|h| h.enhancement == name && h.position == *position)
            {
                continue;
            }
            // one the system does not deliver does not stop the others
            let source = match self.hook_source(&name) {
                Ok(Some(source)) => source,
                Ok(None) | Err(SourceError::Http { .. }) => continue,
                Err(e) => return Err(e),
            };
            // switched off, only the static parts stay, like declarations
            if source.switched_off && mode.trim() == "D" {
                continue;
            }
            if let Some(code) = hook_block(&source.text, id.trim()) {
                out.push(Hook {
                    enhancement: name,
                    position: position.trim().to_string(),
                    code,
                    origin: source.origin,
                });
            }
        }
        Ok(out)
    }

    /// From the BAdI tables through the data preview.
    fn badi(&self, name: &str) -> Result<Option<Badi>> {
        self.cached_badi(name.to_ascii_uppercase(), |n| self.spot_badi(n))
    }

    fn classic_badi(&self, interface: &str) -> Result<Option<Badi>> {
        self.cached_badi(format!("if:{}", interface.to_ascii_uppercase()), |n| {
            self.exit_badi(n)
        })
    }

    fn notes(&self) -> Vec<String> {
        self.notes.lock().unwrap().clone()
    }

    fn describe(&self) -> String {
        match &self.config.client {
            Some(c) => format!("{} client {c}", self.config.url),
            None => self.config.url.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{BufRead, BufReader, Write};
    use std::net::TcpListener;

    /// Serves canned responses by path; returns the base URL and the
    /// receiver of the request lines seen.
    /// A proxy that refuses every CONNECT with 407, and tells the requests
    /// it got.
    fn proxy_server() -> (String, std::sync::mpsc::Receiver<String>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            for stream in listener.incoming() {
                let mut stream = stream.unwrap();
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                let mut request = String::new();
                reader.read_line(&mut request).unwrap();
                loop {
                    let mut line = String::new();
                    reader.read_line(&mut line).unwrap();
                    if line == "\r\n" || line.is_empty() {
                        break;
                    }
                }
                let _ = tx.send(request.trim().to_string());
                let _ = stream.write_all(
                    b"HTTP/1.1 407 Proxy Authentication Required\r\nContent-Length: 0\r\n\r\n",
                );
            }
        });
        (url, rx)
    }

    #[test]
    fn requests_go_through_the_proxy_given() {
        let (proxy, asked) = proxy_server();
        let mut config = AdtConfig::new("http://sap.example.com:50000", "user", "secret");
        config.proxy = AdtProxy::Url(proxy.clone());
        let adt = AdtProvider::try_new(config).unwrap();
        let e = adt.fetch(&SourceRef::program("ZREP")).unwrap_err();
        assert_eq!(
            asked.try_recv().unwrap(),
            "CONNECT sap.example.com:50000 HTTP/1.1"
        );
        let text = e.to_string();
        assert!(text.contains("proxy server responded 407"), "{text}");
        assert!(
            text.ends_with(&format!(" (through the proxy {proxy})")),
            "{text}"
        );
    }

    #[test]
    fn a_proxy_down_is_named_in_the_error() {
        // a port nothing listens on
        let port = TcpListener::bind("127.0.0.1:0")
            .unwrap()
            .local_addr()
            .unwrap()
            .port();
        let mut config = AdtConfig::new("http://sap.example.com:50000", "user", "secret");
        config.proxy = AdtProxy::Url(format!("http://proxyuser:proxysecret@127.0.0.1:{port}"));
        config.timeout = Duration::from_secs(5);
        let adt = AdtProvider::new(config);
        let e = adt.fetch(&SourceRef::program("ZREP")).unwrap_err();
        assert!(matches!(e, SourceError::Network(_)), "{e}");
        assert!(e.retryable());
        let text = e.to_string();
        assert!(
            text.ends_with(&format!(" (through the proxy http://127.0.0.1:{port})")),
            "{text}"
        );
        assert!(!text.contains("proxysecret"), "{text}");

        // no proxy: the error names none
        let mut config = AdtConfig::new(&format!("http://127.0.0.1:{port}"), "user", "secret");
        config.proxy = AdtProxy::Off;
        config.timeout = Duration::from_secs(5);
        let e = AdtProvider::new(config)
            .fetch(&SourceRef::program("ZREP"))
            .unwrap_err();
        assert!(!e.to_string().contains("proxy"), "{e}");
    }

    #[test]
    fn a_proxy_that_cannot_be_read() {
        let mut config = AdtConfig::new("http://sap.example.com", "user", "secret");
        config.proxy = AdtProxy::Url("ftp://proxy:21".into());
        let e = AdtProvider::try_new(config.clone()).err().unwrap();
        assert!(
            e.to_string().starts_with("the proxy given cannot be used"),
            "{e}"
        );
        // without try_new each request fails with it, nothing is sent
        let e = AdtProvider::new(config)
            .fetch(&SourceRef::program("ZREP"))
            .unwrap_err();
        assert!(matches!(e, SourceError::Other(_)), "{e}");
    }

    fn serve(
        routes: Vec<(&'static str, u16, &'static str)>,
    ) -> (String, std::sync::mpsc::Receiver<String>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            for stream in listener.incoming() {
                let mut stream = stream.unwrap();
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                let mut request = String::new();
                reader.read_line(&mut request).unwrap();
                let mut auth = false;
                let mut length = 0;
                loop {
                    let mut line = String::new();
                    reader.read_line(&mut line).unwrap();
                    let lower = line.to_ascii_lowercase();
                    if lower.starts_with("authorization: basic ") {
                        auth = true;
                    }
                    if let Some(n) = lower.strip_prefix("content-length:") {
                        length = n.trim().parse().unwrap_or(0);
                    }
                    if line == "\r\n" || line.is_empty() {
                        break;
                    }
                }
                let mut body = vec![0; length];
                std::io::Read::read_exact(&mut reader, &mut body).unwrap();
                let path = request.split_whitespace().nth(1).unwrap_or("").to_string();
                let _ = tx.send(path.clone());
                let (status, body) = if !auth {
                    (401, "")
                } else {
                    routes
                        .iter()
                        .find(|(p, _, _)| path.starts_with(p))
                        .map(|(_, s, b)| (*s, *b))
                        .unwrap_or((404, ""))
                };
                let response = format!(
                    "HTTP/1.1 {status} X\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                );
                let _ = stream.write_all(response.as_bytes());
            }
        });
        (url, rx)
    }

    #[test]
    fn fetches_programs_includes_classes_and_function_modules() {
        let (url, requests) = serve(vec![
            (
                "/sap/bc/adt/programs/programs/zreport/source/main",
                200,
                "REPORT zreport.",
            ),
            (
                "/sap/bc/adt/programs/includes/zreport_top/source/main",
                200,
                "DATA gv TYPE i.",
            ),
            (
                "/sap/bc/adt/oo/classes/%2fabc%2fcl_x/includes/implementations",
                200,
                "CLASS lcl IMPLEMENTATION.",
            ),
            (
                "/sap/bc/adt/repository/informationsystem/search",
                200,
                r#"<adtcore:objectReferences><adtcore:objectReference adtcore:uri="/sap/bc/adt/functions/groups/zgrp/fmodules/z_fm" adtcore:name="Z_FM"/></adtcore:objectReferences>"#,
            ),
            (
                "/sap/bc/adt/functions/groups/zgrp/fmodules/z_fm/source/main",
                200,
                "FUNCTION z_fm.",
            ),
            (
                "/sap/bc/adt/ddic/lockobjects/sources/ez_order",
                200,
                r#"<enqu:lockModules><enqu:lockModule adtcore:uri="/sap/bc/adt/functions/groups/%2f1bcdwben%2fsen0001/fmodules/enqueue_ez_order" adtcore:name="ENQUEUE_EZ_ORDER"/></enqu:lockModules>"#,
            ),
            (
                "/sap/bc/adt/functions/groups/%2f1bcdwben%2fsen0001/fmodules/enqueue_ez_order/source/main",
                200,
                "FUNCTION enqueue_ez_order.",
            ),
        ]);
        let mut config = AdtConfig::new(&url, "user", "secret");
        config.client = Some("100".to_string());
        let adt = AdtProvider::new(config);

        let prog = adt.fetch(&SourceRef::program("ZREPORT")).unwrap().unwrap();
        assert_eq!(prog.text, "REPORT zreport.");
        assert!(requests.recv().unwrap().ends_with("?sap-client=100"));

        // not a program, found as include
        let inc = adt
            .fetch(&SourceRef::program("ZREPORT_TOP"))
            .unwrap()
            .unwrap();
        assert_eq!(inc.text, "DATA gv TYPE i.");

        let imp = adt
            .fetch(&SourceRef::class("/ABC/CL_X").with_part(ClassPart::Implementations))
            .unwrap()
            .unwrap();
        assert_eq!(imp.text, "CLASS lcl IMPLEMENTATION.");

        let fm = adt
            .fetch(&SourceRef::function_module("Z_FM"))
            .unwrap()
            .unwrap();
        assert_eq!(fm.text, "FUNCTION z_fm.");

        // generated lock module, not in the search: group from its lock object
        let lock = adt
            .fetch(&SourceRef::function_module("ENQUEUE_EZ_ORDER"))
            .unwrap()
            .unwrap();
        assert_eq!(lock.text, "FUNCTION enqueue_ez_order.");

        assert!(adt
            .fetch(&SourceRef::class("ZCL_MISSING"))
            .unwrap()
            .is_none());
    }

    #[test]
    fn implementations_from_the_class_relations() {
        let (url, requests) = serve(vec![
            ("/sap/bc/adt/discovery", 200, ""),
            (
                "/sap/bc/adt/datapreview/freestyle",
                200,
                "<dataPreview:tableData><dataPreview:dataSet><dataPreview:data>CL_IDENTITY</dataPreview:data><dataPreview:data>CL_OTHER</dataPreview:data></dataPreview:dataSet></dataPreview:tableData>",
            ),
        ]);
        let adt = AdtProvider::new(AdtConfig::new(&url, "user", "secret"));
        assert_eq!(
            adt.implementations("if_identity").unwrap(),
            ["CL_IDENTITY", "CL_OTHER"]
        );
        // names that are no ABAP names are not put into SQL
        assert!(adt.implementations("X' OR '1' = '1").unwrap().is_empty());
        // subclasses the same way, answers cached
        assert_eq!(
            adt.subclasses("cl_base").unwrap(),
            ["CL_IDENTITY", "CL_OTHER"]
        );
        adt.subclasses("CL_BASE").unwrap();
        let paths: Vec<String> = requests.try_iter().collect();
        assert_eq!(
            paths.iter().filter(|p| p.contains("freestyle")).count(),
            2,
            "{paths:?}"
        );
    }

    #[test]
    fn blocks_of_enhancement_code() {
        let source = "ENHANCEMENT 1  .\r\n  x = 1.\r\nENDENHANCEMENT.\r\nENHANCEMENT 12  .\r\n  y = 2.\r\nENDENHANCEMENT.";
        assert_eq!(
            hook_block(source, "12").as_deref(),
            Some("ENHANCEMENT 12  .\n  y = 2.\nENDENHANCEMENT.")
        );
        assert_eq!(
            hook_block(source, "1").as_deref(),
            Some("ENHANCEMENT 1  .\n  x = 1.\nENDENHANCEMENT.")
        );
        assert_eq!(hook_block(source, "2"), None);
    }

    #[test]
    fn data_preview_rows() {
        let xml = "<dataPreview:tableData><dataPreview:columns><dataPreview:metadata dataPreview:name=\"A\"/>            <dataPreview:dataSet><dataPreview:data>X1</dataPreview:data><dataPreview:data/></dataPreview:dataSet>            </dataPreview:columns><dataPreview:columns><dataPreview:metadata dataPreview:name=\"B\"/>            <dataPreview:dataSet><dataPreview:data> y1 </dataPreview:data><dataPreview:data>Y2</dataPreview:data>            </dataPreview:dataSet></dataPreview:columns></dataPreview:tableData>";
        assert_eq!(preview_rows(xml), [["X1", "y1"], ["", "Y2"]]);
        assert_eq!(first_column(preview_rows(xml)), ["X1"]);
        // an empty answer has no rows
        assert!(preview_rows(
            "<dataPreview:tableData><dataPreview:dataSet/></dataPreview:tableData>"
        )
        .is_empty());
    }

    #[test]
    fn badis_from_the_badi_tables() {
        let (url, requests) = serve(vec![
            ("/sap/bc/adt/discovery", 200, ""),
            (
                "/sap/bc/adt/datapreview/freestyle",
                200,
                "<dataPreview:tableData><dataPreview:columns><dataPreview:dataSet><dataPreview:data>ZIF_X</dataPreview:data></dataPreview:dataSet></dataPreview:columns><dataPreview:columns><dataPreview:dataSet><dataPreview:data/></dataPreview:dataSet></dataPreview:columns></dataPreview:tableData>",
            ),
        ]);
        let adt = AdtProvider::new(AdtConfig::new(&url, "user", "secret"));
        // every query gets the same answer: the definition row, then one class
        let b = adt.badi("zbadi_x").unwrap().unwrap();
        assert_eq!(b.name, "ZBADI_X");
        assert_eq!(b.interface.as_deref(), Some("ZIF_X"));
        assert_eq!(b.implementations, ["ZIF_X"]);
        assert_eq!(b.fallback, None);
        assert!(!b.classic);
        adt.badi("ZBADI_X").unwrap();
        // names that are no ABAP names are not put into SQL
        assert!(adt.classic_badi("X' OR '1' = '1").unwrap().is_none());
        let paths: Vec<String> = requests.try_iter().collect();
        assert_eq!(
            paths.iter().filter(|p| p.contains("freestyle")).count(),
            2,
            "{paths:?}"
        );
    }

    #[test]
    fn authorization_errors_are_reported() {
        let (url, _) = serve(vec![]);
        let adt = AdtProvider::new(AdtConfig::new(&url, "", ""));
        // the test server rejects requests without credentials only; send none by
        // using a provider whose header the server does not see as basic auth
        let mut bad = adt;
        bad.authorization = String::new();
        match bad.fetch(&SourceRef::program("ZREPORT")) {
            Err(SourceError::Auth { .. }) => {}
            other => panic!("expected auth error, got {other:?}"),
        }
    }

    #[test]
    fn search_result_parsing() {
        let xml =
            r#"<x adtcore:uri="/sap/bc/adt/functions/groups/%2fabc%2fgrp/fmodules/%2fabc%2ffm"/>"#;
        assert_eq!(
            group_from_search(xml, "/ABC/FM").as_deref(),
            Some("/ABC/GRP")
        );
        assert_eq!(group_from_search(xml, "OTHER"), None);
    }

    /// The columns of a data preview answer, each with its values.
    fn preview(columns: &[&[&str]]) -> &'static str {
        let mut xml = String::from("<dataPreview:tableData>");
        for values in columns {
            xml.push_str("<dataPreview:columns><dataPreview:dataSet>");
            for v in *values {
                xml.push_str(&format!("<dataPreview:data>{v}</dataPreview:data>"));
            }
            xml.push_str("</dataPreview:dataSet></dataPreview:columns>");
        }
        xml.push_str("</dataPreview:tableData>");
        Box::leak(xml.into_boxed_str())
    }

    const REJECTED: &str = "<exc:exception><message lang=\"EN\">Field FULL_NAME is a long string</message></exc:exception>";

    #[test]
    fn rejected_free_form_sql_falls_back_to_the_table_preview() {
        let (url, requests) = serve(vec![
            ("/sap/bc/adt/discovery", 200, ""),
            ("/sap/bc/adt/datapreview/freestyle", 400, REJECTED),
            ("/sap/bc/adt/datapreview/ddic", 200, preview(&[&["ZCL_A"]])),
        ]);
        let adt = AdtProvider::new(AdtConfig::new(&url, "user", "secret"));
        assert_eq!(
            adt.implementing_classes("zif_x").unwrap(),
            Some(vec!["ZCL_A".to_string()])
        );
        assert!(adt.notes().is_empty());
        let paths: Vec<String> = requests.try_iter().collect();
        assert!(
            paths
                .iter()
                .any(|p| p.contains("ddicEntityName=SEOMETAREL")),
            "{paths:?}"
        );
    }

    #[test]
    fn unavailable_lookups_become_notes_not_errors() {
        let (url, _) = serve(vec![
            ("/sap/bc/adt/discovery", 200, ""),
            ("/sap/bc/adt/datapreview/freestyle", 400, REJECTED),
            (
                "/sap/bc/adt/system/components",
                200,
                "<atom:entry><atom:id>SAP_BASIS</atom:id><atom:title>750;SAPK-75035INSAPBASIS;0035;SAP Basis Component</atom:title></atom:entry>",
            ),
        ]);
        let adt = AdtProvider::new(AdtConfig::new(&url, "user", "secret"));
        assert_eq!(adt.implementing_classes("zif_x").unwrap(), None);
        assert_eq!(
            adt.notes(),
            ["class relations: not available (the data preview rejects the query: Field FULL_NAME is a long string, SAP_BASIS 750 SP 0035)"]
        );
    }

    #[test]
    fn table_types_from_their_dictionary_table() {
        let discovery = "<app:collection href=\"/sap/bc/adt/datapreview/freestyle\"><atom:title>x</atom:title></app:collection>";
        let (url, requests) = serve(vec![
            ("/sap/bc/adt/discovery", 200, discovery),
            (
                "/sap/bc/adt/datapreview/freestyle",
                200,
                preview(&[&["ZS_ROW"], &["S"], &["STRU"], &["000000"], &[""]]),
            ),
        ]);
        let adt = AdtProvider::new(AdtConfig::new(&url, "user", "secret"));
        let s = adt
            .fetch(&SourceRef::table_type("ztt_rows"))
            .unwrap()
            .unwrap();
        assert_eq!(
            s.text,
            "<DD40V><TYPENAME>ZTT_ROWS</TYPENAME><ROWTYPE>ZS_ROW</ROWTYPE><ROWKIND>S</ROWKIND>\
             <DATATYPE>STRU</DATATYPE><LENG>000000</LENG></DD40V>"
        );
        // names that are no ABAP names are not put into SQL
        assert!(adt
            .fetch(&SourceRef::table_type("x' OR '1"))
            .unwrap()
            .is_none());
        let paths: Vec<String> = requests.try_iter().collect();
        assert!(!paths.iter().any(|p| p.contains("tabletypes")), "{paths:?}");
        assert_eq!(paths.iter().filter(|p| p.contains("freestyle")).count(), 1);
    }

    #[test]
    fn discovery_tells_which_lookups_exist() {
        let discovery = "<app:collection href=\"/sap/bc/adt/datapreview/freestyle\"><atom:title>x</atom:title></app:collection><app:collection href=\"/sap/bc/adt/ddic/dataelements\"><app:accept>application/vnd.sap.adt.dataelements.v1+xml</app:accept></app:collection>";
        let c = Capabilities::from_discovery(discovery);
        assert!(c.offers(FREESTYLE));
        assert!(!c.offers(TABLE_PREVIEW));
        assert!(!c.offers(ENHANCEMENT_IMPLEMENTATIONS));
        assert_eq!(
            c.accepts(DATA_ELEMENTS),
            ["application/vnd.sap.adt.dataelements.v1+xml"]
        );
        // a discovery that could not be read assumes everything
        assert!(Capabilities::default().offers(ENHANCEMENT_IMPLEMENTATIONS));

        let (url, requests) = serve(vec![("/sap/bc/adt/discovery", 200, discovery)]);
        let adt = AdtProvider::new(AdtConfig::new(&url, "user", "secret"));
        assert!(adt
            .enhancements(&["ZPROG".to_string()], &[])
            .unwrap()
            .is_empty());
        assert!(adt.notes()[0].starts_with(
            "enhancements: not available (the ADT discovery has no enhancements/enhoxhh"
        ));
        let paths: Vec<String> = requests.try_iter().collect();
        assert!(
            !paths.iter().any(|p| p.contains("datapreview")),
            "{paths:?}"
        );
    }

    #[test]
    fn enhancement_query_never_filters_on_the_long_string() {
        let query = enhancement_query(&["SAPLZGRP".to_string(), "ZPROG".to_string()]);
        let condition = query.split("WHERE").nth(1).unwrap();
        assert!(!condition.to_ascii_lowercase().contains("full_name"));
        assert!(!query.contains("LIKE"));
        assert!(query.lines().all(|l| l.len() < 255));
        assert_eq!(group_program("ZGRP"), "SAPLZGRP");
        assert_eq!(group_program("/abc/grp"), "/ABC/SAPLGRP");
    }

    #[test]
    fn function_module_hooks_are_matched_on_the_client() {
        let (url, _) = serve(vec![
            ("/sap/bc/adt/discovery", 200, ""),
            (
                "/sap/bc/adt/repository/informationsystem/search",
                200,
                r#"<x adtcore:uri="/sap/bc/adt/functions/groups/zgrp/fmodules/zfm"/>"#,
            ),
            // the hook of ZFM, and one of another module of the same group
            (
                "/sap/bc/adt/datapreview/freestyle",
                200,
                preview(&[
                    &["ZENH", "ZOTHER"],
                    &["1", "1"],
                    &["D", "D"],
                    &["\\FU:ZFM\\EI", "\\FU:OTHER\\EI"],
                    &["SAPLZGRP", "SAPLZGRP"],
                ]),
            ),
            (
                "/sap/bc/adt/enhancements/enhoxhh/zenh/source/main",
                200,
                "ENHANCEMENT 1  .\n  x = 1.\nENDENHANCEMENT.",
            ),
        ]);
        let adt = AdtProvider::new(AdtConfig::new(&url, "user", "secret"));
        let hooks = adt.enhancements(&[], &["ZFM".to_string()]).unwrap();
        assert_eq!(hooks.len(), 1, "{:?}", adt.notes());
        assert_eq!(hooks[0].enhancement, "ZENH");
        assert!(hooks[0].code.contains("x = 1."));
    }

    #[test]
    fn parallel_lookups_of_a_badi_ask_once() {
        let (url, requests) = serve(vec![
            ("/sap/bc/adt/discovery", 200, ""),
            ("/sap/bc/adt/datapreview/freestyle", 200, preview(&[])),
        ]);
        let adt = AdtProvider::new(AdtConfig::new(&url, "user", "secret"));
        let barrier = std::sync::Barrier::new(8);
        std::thread::scope(|s| {
            for _ in 0..8 {
                s.spawn(|| {
                    barrier.wait();
                    assert_eq!(adt.classic_badi("ZIF_EX_X").unwrap(), None);
                });
            }
        });
        let previews = requests
            .try_iter()
            .filter(|p| p.contains("datapreview"))
            .count();
        assert_eq!(previews, 1);
    }

    #[test]
    fn data_elements_in_the_newest_xml_version() {
        let v1 = "application/vnd.sap.adt.dataelements.v1+xml";
        let v2 = "application/vnd.sap.adt.dataelements.v2+xml";
        // SAP_BASIS 816 lists HTML too, which sorts after the XML types
        assert_eq!(data_element_type(&[v2, "text/html"]), v2);
        assert_eq!(data_element_type(&[v1]), v1);
        assert_eq!(data_element_type(&[v1, v2]), v2);
        assert_eq!(data_element_type(&["text/html"]), v2);
        assert_eq!(data_element_type(&[]), v2);
    }

    #[test]
    fn rejection_message_on_one_line() {
        assert_eq!(rejection(REJECTED), "Field FULL_NAME is a long string");
        assert_eq!(rejection("no xml\n here"), "no xml here");
    }
}
