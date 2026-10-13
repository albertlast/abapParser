//! Sources from an SAP system through the ABAP Development Tools REST API,
//! the same endpoints Eclipse ADT and vscode_abap_remote_fs use. Only reads.

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Duration;

use base64::Engine;

use crate::{ClassPart, ObjectKind, Result, Source, SourceError, SourceProvider, SourceRef};

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
        let group = group_from_search(&xml, &key);
        self.groups.lock().unwrap().insert(key, group.clone());
        Ok(group)
    }
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
        }
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
                loop {
                    let mut line = String::new();
                    reader.read_line(&mut line).unwrap();
                    if line
                        .to_ascii_lowercase()
                        .starts_with("authorization: basic ")
                    {
                        auth = true;
                    }
                    if line == "\r\n" || line.is_empty() {
                        break;
                    }
                }
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

        assert!(adt
            .fetch(&SourceRef::class("ZCL_MISSING"))
            .unwrap()
            .is_none());
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
