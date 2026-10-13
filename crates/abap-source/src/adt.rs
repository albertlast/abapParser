//! Sources from an SAP system through the ABAP Development Tools REST API,
//! the same endpoints Eclipse ADT and vscode_abap_remote_fs use. Only reads.

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Duration;

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
        }
    }
}

pub struct AdtProvider {
    config: AdtConfig,
    agent: ureq::Agent,
    authorization: String,
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
    /// BAdIs by name, classic ones by interface with a leading `if:`
    badis: Mutex<HashMap<String, Option<Badi>>>,
    /// enhancement implementation -> its code, URL and whether it is
    /// switched off
    hook_sources: Mutex<HashMap<String, Option<HookSource>>>,
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
        let agent: ureq::Agent = ureq::Agent::config_builder()
            .tls_config(tls)
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
            groups: Mutex::new(HashMap::new()),
            session: Mutex::new(None),
            usages: Mutex::new(HashMap::new()),
            implementers: Mutex::new(HashMap::new()),
            subclasses: Mutex::new(HashMap::new()),
            badis: Mutex::new(HashMap::new()),
            hook_sources: Mutex::new(HashMap::new()),
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
        let url = self.url(path);
        let mut response = self
            .agent
            .get(&url)
            .header("Authorization", &self.authorization)
            .header("Accept", accept)
            .call()
            .map_err(|e| SourceError::Other(format!("{url}: {e}")))?;
        let status = response.status().as_u16();
        match status {
            200 => response
                .body_mut()
                .read_to_string()
                .map(Some)
                .map_err(|e| SourceError::Other(format!("{url}: {e}"))),
            404 => Ok(None),
            401 | 403 => Err(SourceError::Auth { url }),
            _ => Err(SourceError::Http { status, url }),
        }
    }

    /// A CSRF token and the cookies of its session.
    fn fetch_session(&self) -> Result<(String, String)> {
        let url = self.url("/sap/bc/adt/discovery");
        let response = self
            .agent
            .get(&url)
            .header("Authorization", &self.authorization)
            .header("X-CSRF-Token", "Fetch")
            .header("Accept", "application/atomsvc+xml")
            .call()
            .map_err(|e| SourceError::Other(format!("{url}: {e}")))?;
        let status = response.status().as_u16();
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
        let url = self.url(path);
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
            let mut response = request
                .send(body)
                .map_err(|e| SourceError::Other(format!("{url}: {e}")))?;
            let status = response.status().as_u16();
            let renew = response
                .headers()
                .get("x-csrf-token")
                .and_then(|v| v.to_str().ok())
                .is_some_and(|v| v.eq_ignore_ascii_case("required"));
            match status {
                200 => {
                    return response
                        .body_mut()
                        .read_to_string()
                        .map(Some)
                        .map_err(|e| SourceError::Other(format!("{url}: {e}")))
                }
                404 => return Ok(None),
                403 if renew && attempt == 0 => continue,
                401 | 403 => return Err(SourceError::Auth { url }),
                _ => return Err(SourceError::Http { status, url }),
            }
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
        let rows = self.select(&format!(
            "SELECT clsname FROM seometarel WHERE refclsname = '{key}' AND reltype = '{}' AND version = '1'",
            relation as u8
        ))?;
        Ok(rows.map(first_column))
    }

    /// Rows of an ABAP SQL query through the data preview; `None` when the
    /// system has no data preview.
    fn select(&self, query: &str) -> Result<Option<Vec<Vec<String>>>> {
        let xml = self.post(
            "/sap/bc/adt/datapreview/freestyle?rowNumber=1000",
            "text/plain",
            "application/xml, application/vnd.sap.adt.datapreview.table.v1+xml",
            query,
        )?;
        Ok(xml.map(|xml| preview_rows(&xml)))
    }

    /// A BAdI of an enhancement spot: interface and fallback class from its
    /// definition, the classes of its active implementations.
    fn spot_badi(&self, name: &str) -> Result<Option<Badi>> {
        let definition = self
            .select(&format!(
                "SELECT interface_name, default_class FROM badi_main WHERE badi_name = '{name}'"
            ))?
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
        Ok(self
            .select(&format!(
                "SELECT class_name FROM badi_impl WHERE badi_name = '{name}'"
            ))?
            .map(first_column)
            .unwrap_or_default())
    }

    /// A classic BAdI by its interface with its active implementations;
    /// those of the BAdI it was migrated to count too.
    fn exit_badi(&self, interface: &str) -> Result<Option<Badi>> {
        let definition = self
            .select(&format!(
                "SELECT x~exit_name, a~def_clname, a~mig_badi_name FROM sxs_inter AS x \
                 INNER JOIN sxs_attr AS a ON a~exit_name = x~exit_name \
                 WHERE x~inter_name = '{interface}'"
            ))?
            .unwrap_or_default();
        let Some(row) = definition.first() else {
            return Ok(None);
        };
        let mut implementations = self
            .select(&format!(
                "SELECT c~imp_class FROM sxc_class AS c INNER JOIN sxc_attr AS a \
                 ON a~imp_name = c~imp_name WHERE c~inter_name = '{interface}' AND a~active = 'X'"
            ))?
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
        if let Some(b) = self.badis.lock().unwrap().get(&key) {
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
        self.badis.lock().unwrap().insert(key, badi.clone());
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
            (ObjectKind::Dtel, _) => {
                let path = format!("/sap/bc/adt/ddic/dataelements/{name}");
                Ok(self
                    .get(&path, "application/vnd.sap.adt.dataelements.v2+xml")?
                    .map(|text| Source {
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
        // names go into SQL: ABAP names, class pools and their includes
        let valid = |n: &&String| {
            !n.is_empty()
                && n.chars()
                    .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '/' | '='))
        };
        let programs: Vec<String> = programs
            .iter()
            .filter(valid)
            .map(|p| format!("'{}'", p.to_ascii_uppercase()))
            .collect();
        let modules: Vec<String> = function_modules
            .iter()
            .filter(valid)
            .map(|f| f.to_ascii_uppercase())
            .collect();
        let mut conditions: Vec<String> = programs
            .chunks(50)
            // the data preview takes lines up to 255 characters
            .map(|c| format!("programname IN ( {} )", c.join(",\n")))
            .collect();
        conditions.extend(
            modules
                .iter()
                .map(|f| format!("full_name LIKE '\\FU:{f}\\%'")),
        );
        let mut rows = Vec::new();
        for c in conditions.chunks(20) {
            let query = format!(
                "SELECT enhname, id, enhmode, full_name FROM enhincinx WHERE version = 'A' AND ( {} )",
                c.join("\nOR ")
            );
            match self.select(&query) {
                Ok(r) => rows.extend(r.unwrap_or_default()),
                // a query the system rejects is a mistake here; other answers
                // mean the system has no data preview for us
                Err(SourceError::Http { status, .. }) if status != 400 => return Ok(Vec::new()),
                Err(SourceError::Auth { .. }) => return Ok(Vec::new()),
                Err(e) => return Err(e),
            }
        }
        let mut out: Vec<Hook> = Vec::new();
        for row in rows {
            let [name, id, mode, position] = &row[..] else {
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
}
