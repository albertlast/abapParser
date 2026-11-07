//! Sources and lookups from functions of the caller, like the `fetch` and
//! `query` callbacks of the Node.js and Python packages.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use serde_json::Value;

use crate::tables::{self, HookSource, Query, Relation, Rows};
use crate::{Badi, Hook, ObjectKind, Result, Source, SourceError, SourceProvider, SourceRef};

/// The function of [`FnProvider`]: the source of an object, `None` when
/// there is none.
pub type FetchFn = dyn Fn(&SourceRef) -> Result<Option<String>> + Send + Sync;

/// The query function of [`FnProvider`]: the rows of a query on one table,
/// `None` when it cannot answer it.
pub type QueryFn = dyn Fn(&Query) -> Result<Option<Rows>> + Send + Sync;

/// A function of a host answering in JSON, see [`FnProvider::json`]; an
/// `Err` when calling it failed.
pub type HostFn = Box<dyn Fn(String) -> std::result::Result<String, String> + Send + Sync>;

/// Sources from a function of the caller, like a callback of the Node.js or
/// Python package that reads them from its own ADT client or cache. It gets
/// the object as written, like `CLAS:ZCL_X/implementations` or `FUNC:Z_FM`.
///
/// With a query function ([`with_query`](Self::with_query)) it also answers
/// the lookups an SAP system answers from its tables: BAdIs (BADI_MAIN,
/// BADI_IMPL), classic BAdIs (SXS_INTER, SXS_ATTR, SXC_CLASS, SXC_ATTR),
/// implementations and subclasses (SEOMETAREL) and enhancements (TFDIR,
/// ENHINCINX, the code fetched as `ENHO:<name>`).
pub struct FnProvider {
    name: String,
    fetch: Box<FetchFn>,
    query: Option<Box<QueryFn>>,
    badis: Mutex<HashMap<String, Option<Badi>>>,
    notes: Mutex<Vec<String>>,
    /// answers of the fetch function that are no text: how many, the first
    not_text: Arc<Mutex<(usize, String)>>,
}

impl FnProvider {
    /// `name` describes the provider in messages and in the origin of sources.
    pub fn new(
        name: impl Into<String>,
        fetch: impl Fn(&SourceRef) -> Result<Option<String>> + Send + Sync + 'static,
    ) -> Self {
        FnProvider {
            name: name.into(),
            fetch: Box::new(fetch),
            query: None,
            badis: Mutex::new(HashMap::new()),
            notes: Mutex::new(Vec::new()),
            not_text: Arc::default(),
        }
    }

    /// Lookups through queries on single tables, see [`Query`].
    pub fn with_query(
        mut self,
        query: impl Fn(&Query) -> Result<Option<Rows>> + Send + Sync + 'static,
    ) -> Self {
        self.query = Some(Box::new(query));
        self
    }

    /// A provider of host functions answering in JSON, as the language
    /// bindings call them: `fetch` gets the reference like `CLAS:ZCL_X`,
    /// `query` the JSON of a [`Query`] (see [`Query::to_json`]). Both answer
    /// `{"value": ...}`, the text or `null` for `fetch`, the rows or `null`
    /// for `query`, or `{"error": {"message": ..., "status": 500, "code":
    /// "ECONNRESET"}}`, status and code when known (see
    /// [`SourceError::from_host`]).
    pub fn json(fetch: Option<HostFn>, query: Option<HostFn>) -> Self {
        let not_text: Arc<Mutex<(usize, String)>> = Arc::default();
        let seen = not_text.clone();
        let mut provider = FnProvider::new("fetch", move |r| {
            let Some(fetch) = &fetch else {
                return Ok(None);
            };
            let what = format!("fetch {r}");
            match answer(&what, fetch(r.to_string()))? {
                None => Ok(None),
                Some(Value::String(text)) => Ok(Some(text)),
                Some(other) => {
                    let mut n = seen.lock().unwrap();
                    if n.0 == 0 {
                        n.1 = format!("{r}, {}", kind(&other));
                    }
                    n.0 += 1;
                    Ok(None)
                }
            }
        });
        provider.not_text = not_text;
        if let Some(query) = query {
            provider = provider.with_query(move |q| {
                let what = format!("query {}", q.table);
                match answer(&what, query(q.to_json().to_string()))? {
                    None => Ok(None),
                    Some(value) => q
                        .rows_of(&value)
                        .map_err(|e| SourceError::Other(format!("{what}: {e}"))),
                }
            });
        }
        provider
    }

    /// Rows of a query for a lookup; `None` without a query function, and
    /// with a note when the function does not answer it.
    fn select(&self, lookup: &str, query: &Query) -> Result<Option<Rows>> {
        let Some(f) = &self.query else {
            return Ok(None);
        };
        let rows = f(query)?;
        if rows.is_none() {
            let text = format!(
                "{lookup}: not available (the query function does not answer {})",
                query.table
            );
            let mut notes = self.notes.lock().unwrap();
            if !notes.contains(&text) {
                notes.push(text);
            }
        }
        Ok(rows)
    }

    /// A BAdI looked up once.
    fn cached_badi(
        &self,
        key: String,
        lookup: impl FnOnce() -> Result<Option<Badi>>,
    ) -> Result<Option<Badi>> {
        if self.query.is_none() {
            return Ok(None);
        }
        if let Some(b) = self.badis.lock().unwrap().get(&key) {
            return Ok(b.clone());
        }
        let badi = lookup()?;
        self.badis.lock().unwrap().insert(key, badi.clone());
        Ok(badi)
    }
}

/// The value of an answer of a host function, `None` for `null` or a
/// status saying not found.
fn answer(what: &str, reply: std::result::Result<String, String>) -> Result<Option<Value>> {
    let text = reply.map_err(|e| SourceError::Other(format!("{what}: {e}")))?;
    let reply: Value = serde_json::from_str(&text)
        .map_err(|e| SourceError::Other(format!("{what}: answer is no JSON: {e}")))?;
    if let Some(error) = reply.get("error") {
        let message = error
            .get("message")
            .and_then(Value::as_str)
            .unwrap_or("failed");
        let status = error
            .get("status")
            .and_then(Value::as_u64)
            .and_then(|s| u16::try_from(s).ok());
        let code = error.get("code").and_then(Value::as_str);
        return match SourceError::from_host(what, message, status, code) {
            Some(e) => Err(e),
            None => Ok(None),
        };
    }
    Ok(reply.get("value").filter(|v| !v.is_null()).cloned())
}

fn kind(v: &Value) -> &'static str {
    match v {
        Value::Null => "null",
        Value::Bool(_) => "a boolean",
        Value::Number(_) => "a number",
        Value::String(_) => "a string",
        Value::Array(_) => "a list",
        Value::Object(_) => "an object",
    }
}

impl SourceProvider for FnProvider {
    fn fetch(&self, reference: &SourceRef) -> Result<Option<Source>> {
        Ok((self.fetch)(reference)?.map(|text| Source {
            reference: reference.clone(),
            text,
            origin: format!("{}:{reference}", self.name),
            cached: false,
        }))
    }

    fn implementations(&self, interface: &str) -> Result<Vec<String>> {
        Ok(
            tables::related_classes(&|l, q| self.select(l, q), interface, Relation::Implements)?
                .unwrap_or_default(),
        )
    }

    fn subclasses(&self, class: &str) -> Result<Vec<String>> {
        Ok(
            tables::related_classes(&|l, q| self.select(l, q), class, Relation::Inherits)?
                .unwrap_or_default(),
        )
    }

    fn enhancements(&self, programs: &[String], function_modules: &[String]) -> Result<Vec<Hook>> {
        if self.query.is_none() {
            return Ok(Vec::new());
        }
        tables::enhancements(
            &|l, q| self.select(l, q),
            programs,
            function_modules,
            |name| {
                Ok(self
                    .fetch(&SourceRef::new(ObjectKind::Enho, name))?
                    .map(|s| HookSource {
                        text: s.text,
                        origin: s.origin,
                        // the host gives the code of active implementations
                        switched_off: false,
                    }))
            },
        )
    }

    fn badi(&self, name: &str) -> Result<Option<Badi>> {
        let key = name.to_ascii_uppercase();
        self.cached_badi(key.clone(), || {
            tables::spot_badi(&|l, q| self.select(l, q), &key)
        })
    }

    fn classic_badi(&self, interface: &str) -> Result<Option<Badi>> {
        let key = interface.to_ascii_uppercase();
        self.cached_badi(format!("if:{key}"), || {
            tables::classic_badi(&|l, q| self.select(l, q), &key)
        })
    }

    fn notes(&self) -> Vec<String> {
        let mut notes = self.notes.lock().unwrap().clone();
        let (count, first) = self.not_text.lock().unwrap().clone();
        if count > 0 {
            notes.push(format!(
                "fetch: {count} answers were no text and are taken as not found (first: {first})"
            ));
        }
        notes
    }

    fn describe(&self) -> String {
        self.name.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A host answering the tables of a small system.
    fn system(q: &Query) -> Result<Option<Rows>> {
        let value = |c: &str| {
            q.conditions
                .iter()
                .find(|(k, _)| k == c)
                .map(|(_, v)| v.clone())
                .unwrap_or_default()
        };
        let rows: Vec<Vec<&str>> = match q.table.as_str() {
            "BADI_MAIN" if value("BADI_NAME") == ["ZBADI"] => {
                vec![vec!["ZIF_BADI", "ZCL_FALLBACK"]]
            }
            "BADI_IMPL" if value("BADI_NAME") == ["ZBADI"] => vec![vec!["ZCL_IMPL"]],
            "BADI_IMPL" if value("BADI_NAME") == ["ZMIGRATED"] => vec![vec!["ZCL_NEW"]],
            "SXS_INTER" => vec![vec!["ZEXIT"]],
            "SXS_ATTR" => vec![vec!["ZEXIT", "ZCL_EXIT_DEF", "ZMIGRATED"]],
            "SXC_CLASS" => vec![vec!["ZIMP_ON", "ZCL_ON"], vec!["ZIMP_OFF", "ZCL_OFF"]],
            "SXC_ATTR" => {
                assert_eq!(value("IMP_NAME"), ["ZIMP_ON", "ZIMP_OFF"]);
                vec![vec!["ZIMP_ON"]]
            }
            "SEOMETAREL" if value("RELTYPE") == ["1"] => vec![vec!["ZCL_B"], vec!["ZCL_A"]],
            "SEOMETAREL" => vec![vec!["ZCL_SUB"]],
            "TFDIR" => vec![vec!["Z_FM", "SAPLZGRP"]],
            "ENHINCINX" => vec![
                vec!["ZENH", "1", "D", r"\PR:ZREPORT\EX:ZPOINT\EI", "ZREPORT"],
                vec!["ZENH", "2", "D", r"\FU:Z_FM\SE:END\EI", "SAPLZGRP"],
                vec!["ZENH", "3", "D", r"\FU:Z_OTHER\SE:END\EI", "SAPLZGRP"],
            ],
            // a table the host does not have
            "BADI_MAIN" => return Ok(None),
            _ => vec![],
        };
        Ok(Some(
            rows.into_iter()
                .map(|r| r.into_iter().map(String::from).collect())
                .collect(),
        ))
    }

    fn provider() -> FnProvider {
        FnProvider::new("host", |r| {
            Ok((r.kind == ObjectKind::Enho && r.name == "ZENH").then(|| {
                "ENHANCEMENT 1 .\n  x = 1.\nENDENHANCEMENT.\nENHANCEMENT 2 .\n  y = 2.\nENDENHANCEMENT.\nENHANCEMENT 3 .\n  z = 3.\nENDENHANCEMENT.".to_string()
            }))
        })
        .with_query(system)
    }

    #[test]
    fn lookups_through_the_query_function() {
        let p = provider();
        let b = p.badi("zbadi").unwrap().unwrap();
        assert_eq!(b.interface.as_deref(), Some("ZIF_BADI"));
        assert_eq!(b.fallback.as_deref(), Some("ZCL_FALLBACK"));
        assert_eq!(b.implementations, ["ZCL_IMPL"]);
        let c = p.classic_badi("ZIF_EX_X").unwrap().unwrap();
        assert!(c.classic);
        assert_eq!(c.name, "ZEXIT");
        assert_eq!(c.fallback.as_deref(), Some("ZCL_EXIT_DEF"));
        // active ones only, and those of the BAdI it was migrated to
        assert_eq!(c.implementations, ["ZCL_NEW", "ZCL_ON"]);
        assert_eq!(p.implementations("zif_x").unwrap(), ["ZCL_A", "ZCL_B"]);
        assert_eq!(p.subclasses("ZCL_BASE").unwrap(), ["ZCL_SUB"]);
        let hooks = p
            .enhancements(&["ZREPORT".to_string()], &["Z_FM".to_string()])
            .unwrap();
        let codes: Vec<(&str, &str)> = hooks
            .iter()
            .map(|h| (h.position.as_str(), h.code.as_str()))
            .collect();
        assert_eq!(
            codes,
            [
                (
                    r"\PR:ZREPORT\EX:ZPOINT\EI",
                    "ENHANCEMENT 1 .\n  x = 1.\nENDENHANCEMENT."
                ),
                (
                    r"\FU:Z_FM\SE:END\EI",
                    "ENHANCEMENT 2 .\n  y = 2.\nENDENHANCEMENT."
                ),
            ]
        );
        assert_eq!(hooks[0].origin, "host:ENHO:ZENH");
        assert!(p.notes().is_empty(), "{:?}", p.notes());
        // a table the host does not answer is a note, not an error
        assert_eq!(p.badi("ZOTHER").unwrap(), None);
        assert_eq!(
            p.notes(),
            ["BAdI definitions: not available (the query function does not answer BADI_MAIN)"]
        );
        // without a query function nothing is asked
        let plain = FnProvider::new("host", |_| Ok(None));
        assert_eq!(plain.badi("ZBADI").unwrap(), None);
        assert!(plain.implementations("ZIF_X").unwrap().is_empty());
        assert!(plain.notes().is_empty());
    }

    #[test]
    fn host_functions_answering_in_json() {
        let fetch: HostFn = Box::new(|r| {
            Ok(match r.as_str() {
                "PROG:ZA" => r#"{"value": "REPORT za."}"#.to_string(),
                "PROG:ZBYTES" => r#"{"value": {"type": "Buffer"}}"#.to_string(),
                "PROG:ZGONE" => r#"{"error": {"message": "Not Found", "status": 404}}"#.to_string(),
                "PROG:ZAUTH" => {
                    r#"{"error": {"message": "Unauthorized", "status": 401}}"#.to_string()
                }
                "PROG:ZDOWN" => {
                    r#"{"error": {"message": "connect ECONNREFUSED", "code": "ECONNREFUSED"}}"#
                        .to_string()
                }
                "PROG:ZBROKEN" => "not json".to_string(),
                "PROG:ZTHROWN" => return Err("host gone".to_string()),
                _ => r#"{"value": null}"#.to_string(),
            })
        });
        let query: HostFn = Box::new(|q| {
            let q: Value = serde_json::from_str(&q).unwrap();
            assert_eq!(q["where"]["REFCLSNAME"], serde_json::json!(["ZIF_X"]));
            Ok(r#"{"value": [{"clsname": "ZCL_X"}]}"#.to_string())
        });
        let p = FnProvider::json(Some(fetch), Some(query));
        let text = |r: &str| p.fetch(&r.parse().unwrap()).map(|s| s.map(|s| s.text));
        assert_eq!(text("PROG:ZA").unwrap().as_deref(), Some("REPORT za."));
        assert_eq!(text("PROG:ZNONE").unwrap(), None);
        assert_eq!(text("PROG:ZGONE").unwrap(), None);
        assert_eq!(text("PROG:ZBYTES").unwrap(), None);
        assert!(matches!(text("PROG:ZAUTH"), Err(SourceError::Auth { .. })));
        let down = text("PROG:ZDOWN").unwrap_err();
        assert!(matches!(down, SourceError::Network(_)));
        assert!(down.retryable());
        assert_eq!(down.to_string(), "fetch PROG:ZDOWN: connect ECONNREFUSED");
        assert!(matches!(text("PROG:ZBROKEN"), Err(SourceError::Other(_))));
        assert_eq!(
            text("PROG:ZTHROWN").unwrap_err().to_string(),
            "fetch PROG:ZTHROWN: host gone"
        );
        assert_eq!(
            p.notes(),
            ["fetch: 1 answers were no text and are taken as not found (first: PROG:ZBYTES, an object)"]
        );
        assert_eq!(p.implementations("ZIF_X").unwrap(), ["ZCL_X"]);
    }
}
