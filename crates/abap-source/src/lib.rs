//! Where ABAP source code comes from: abapGit style folders, a disk cache,
//! or an SAP system through the ADT REST API.
//!
//! Objects are addressed with [`SourceRef`], written like `PROG:ZREPORT`,
//! `CLAS:ZCL_ORDERS`, `CLAS:ZCL_ORDERS/implementations`, `INTF:ZIF_X`,
//! `FUGR:ZORDERS` or `FUNC:Z_ORDER_CREATE`; dictionary types as
//! `TABL:BAPIRET2` (structures and tables) and `DTEL:XUBNAME`, CDS data
//! definitions as `DDLS:I_COUNTRY`.

#[cfg(feature = "adt")]
mod adt;
mod cache;
mod fs;
mod mock;
mod options;

#[cfg(feature = "adt")]
pub use adt::{AdtConfig, AdtProvider};
pub use cache::CachedProvider;
pub use fs::FsProvider;
pub use mock::{MockAdtConfig, MockAdtServer};
pub use options::{AdtOptions, ProviderOptions};

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ObjectKind {
    /// program or include
    Prog,
    Clas,
    Intf,
    /// function group, its main program SAPL<group>
    Fugr,
    /// function module
    Func,
    /// dictionary structure or table: ADT gives its DDL source, abapGit
    /// its `.tabl.xml`
    Tabl,
    /// dictionary data element: ADT and abapGit give XML
    Dtel,
    /// CDS data definition, its DDL source; abapGit `.ddls.asddls`
    Ddls,
}

impl ObjectKind {
    pub fn as_str(self) -> &'static str {
        match self {
            ObjectKind::Prog => "PROG",
            ObjectKind::Clas => "CLAS",
            ObjectKind::Intf => "INTF",
            ObjectKind::Fugr => "FUGR",
            ObjectKind::Func => "FUNC",
            ObjectKind::Tabl => "TABL",
            ObjectKind::Dtel => "DTEL",
            ObjectKind::Ddls => "DDLS",
        }
    }
}

/// Source parts of a global class besides the main part.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ClassPart {
    /// local type definitions, abapGit `.clas.locals_def.abap`
    Definitions,
    /// local class implementations, abapGit `.clas.locals_imp.abap`
    Implementations,
    Macros,
    TestClasses,
}

impl ClassPart {
    pub const ALL: [ClassPart; 4] = [
        ClassPart::Definitions,
        ClassPart::Implementations,
        ClassPart::Macros,
        ClassPart::TestClasses,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            ClassPart::Definitions => "definitions",
            ClassPart::Implementations => "implementations",
            ClassPart::Macros => "macros",
            ClassPart::TestClasses => "testclasses",
        }
    }
}

/// An ABAP source object. Names are kept upper case.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SourceRef {
    pub kind: ObjectKind,
    pub name: String,
    /// class part, only for [`ObjectKind::Clas`]
    pub part: Option<ClassPart>,
}

impl SourceRef {
    pub fn new(kind: ObjectKind, name: &str) -> Self {
        SourceRef {
            kind,
            name: name.trim().to_ascii_uppercase(),
            part: None,
        }
    }

    pub fn program(name: &str) -> Self {
        Self::new(ObjectKind::Prog, name)
    }

    pub fn class(name: &str) -> Self {
        Self::new(ObjectKind::Clas, name)
    }

    pub fn interface(name: &str) -> Self {
        Self::new(ObjectKind::Intf, name)
    }

    pub fn function_group(name: &str) -> Self {
        Self::new(ObjectKind::Fugr, name)
    }

    pub fn function_module(name: &str) -> Self {
        Self::new(ObjectKind::Func, name)
    }

    pub fn table(name: &str) -> Self {
        Self::new(ObjectKind::Tabl, name)
    }

    pub fn data_element(name: &str) -> Self {
        Self::new(ObjectKind::Dtel, name)
    }

    pub fn data_definition(name: &str) -> Self {
        Self::new(ObjectKind::Ddls, name)
    }

    pub fn with_part(mut self, part: ClassPart) -> Self {
        self.part = Some(part);
        self
    }

    /// The object without a class part.
    pub fn object(&self) -> SourceRef {
        SourceRef {
            part: None,
            ..self.clone()
        }
    }
}

impl fmt::Display for SourceRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.kind.as_str(), self.name)?;
        if let Some(p) = self.part {
            write!(f, "/{}", p.as_str())?;
        }
        Ok(())
    }
}

impl FromStr for SourceRef {
    type Err = String;

    /// `KIND:NAME[/part]`, kind one of PROG, CLAS, INTF, FUGR, FUNC, TABL, DTEL, DDLS
    fn from_str(s: &str) -> std::result::Result<Self, String> {
        let (kind, rest) = s
            .split_once(':')
            .ok_or_else(|| format!("expected KIND:NAME, got {s:?}"))?;
        let kind = match kind.trim().to_ascii_uppercase().as_str() {
            "PROG" => ObjectKind::Prog,
            "CLAS" => ObjectKind::Clas,
            "INTF" => ObjectKind::Intf,
            "FUGR" => ObjectKind::Fugr,
            "FUNC" => ObjectKind::Func,
            "TABL" => ObjectKind::Tabl,
            "DTEL" => ObjectKind::Dtel,
            "DDLS" => ObjectKind::Ddls,
            other => return Err(format!("unknown object kind {other:?}")),
        };
        // names may contain namespaces like /ABC/ZCL_X; a part follows the
        // last '/', and without one a name starting with '/' is a namespace
        let (name, part) = match rest.rsplit_once('/') {
            Some((n, p)) if kind == ObjectKind::Clas && !n.is_empty() && !n.ends_with('/') => {
                match ClassPart::ALL
                    .into_iter()
                    .find(|x| x.as_str().eq_ignore_ascii_case(p))
                {
                    Some(part) => (n, Some(part)),
                    None if rest.trim_start().starts_with('/') => (rest, None),
                    None => return Err(format!("unknown class part {p:?}")),
                }
            }
            _ => (rest, None),
        };
        if name.trim().is_empty() {
            return Err(format!("missing name in {s:?}"));
        }
        Ok(SourceRef {
            kind,
            name: name.trim().to_ascii_uppercase(),
            part,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Source {
    pub reference: SourceRef,
    pub text: String,
    /// file path or URL the text came from
    pub origin: String,
}

#[derive(Debug)]
pub enum SourceError {
    Io(std::io::Error),
    /// HTTP status other than 404
    Http {
        status: u16,
        url: String,
    },
    /// authentication failed, HTTP 401 or 403
    Auth {
        url: String,
    },
    Other(String),
}

impl fmt::Display for SourceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SourceError::Io(e) => write!(f, "{e}"),
            SourceError::Http { status, url } => write!(f, "HTTP {status} for {url}"),
            SourceError::Auth { url } => write!(f, "not authorized for {url}"),
            SourceError::Other(m) => write!(f, "{m}"),
        }
    }
}

impl std::error::Error for SourceError {}

impl From<std::io::Error> for SourceError {
    fn from(e: std::io::Error) -> Self {
        SourceError::Io(e)
    }
}

pub type Result<T> = std::result::Result<T, SourceError>;

/// Delivers source code of ABAP objects; `Ok(None)` when the object does not exist.
pub trait SourceProvider: Send + Sync {
    fn fetch(&self, reference: &SourceRef) -> Result<Option<Source>>;

    /// Function modules of a function group, when the provider can list them.
    /// Needed where the group's include structure is not available, e.g. abapGit.
    fn function_modules(&self, _group: &str) -> Result<Vec<String>> {
        Ok(Vec::new())
    }

    /// Global classes implementing an interface, upper case, when the
    /// provider can list them. May name more classes, e.g. an SAP system
    /// answers with its where-used list; callers check the class sources.
    fn implementations(&self, _interface: &str) -> Result<Vec<String>> {
        Ok(Vec::new())
    }

    /// Global classes inheriting directly from a class, upper case, when the
    /// provider can list them.
    fn subclasses(&self, _class: &str) -> Result<Vec<String>> {
        Ok(Vec::new())
    }

    /// Short description for messages, e.g. the folder or URL.
    fn describe(&self) -> String;
}

/// Asks providers in order; the first one that has the object wins.
pub struct ChainProvider {
    providers: Vec<Box<dyn SourceProvider>>,
}

impl ChainProvider {
    pub fn new(providers: Vec<Box<dyn SourceProvider>>) -> Self {
        ChainProvider { providers }
    }
}

impl SourceProvider for ChainProvider {
    fn fetch(&self, reference: &SourceRef) -> Result<Option<Source>> {
        for p in &self.providers {
            if let Some(s) = p.fetch(reference)? {
                return Ok(Some(s));
            }
        }
        Ok(None)
    }

    fn function_modules(&self, group: &str) -> Result<Vec<String>> {
        for p in &self.providers {
            let fms = p.function_modules(group)?;
            if !fms.is_empty() {
                return Ok(fms);
            }
        }
        Ok(Vec::new())
    }

    fn implementations(&self, interface: &str) -> Result<Vec<String>> {
        let mut out: Vec<String> = Vec::new();
        for p in &self.providers {
            for c in p.implementations(interface)? {
                if !out.contains(&c) {
                    out.push(c);
                }
            }
        }
        Ok(out)
    }

    fn subclasses(&self, class: &str) -> Result<Vec<String>> {
        let mut out: Vec<String> = Vec::new();
        for p in &self.providers {
            for c in p.subclasses(class)? {
                if !out.contains(&c) {
                    out.push(c);
                }
            }
        }
        Ok(out)
    }

    fn describe(&self) -> String {
        self.providers
            .iter()
            .map(|p| p.describe())
            .collect::<Vec<_>>()
            .join(" -> ")
    }
}

/// A provider with fixed sources, for tests and for sources passed in by callers.
#[derive(Default)]
pub struct MemoryProvider {
    sources: std::collections::HashMap<SourceRef, String>,
}

impl MemoryProvider {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, reference: SourceRef, text: impl Into<String>) {
        self.sources.insert(reference, text.into());
    }

    /// Global classes whose main source matches, sorted.
    fn classes(&self, matches: impl Fn(&str, &str) -> bool) -> Vec<String> {
        let mut out: Vec<String> = self
            .sources
            .iter()
            .filter(|(r, text)| {
                r.kind == ObjectKind::Clas && r.part.is_none() && matches(text, &r.name)
            })
            .map(|(r, _)| r.name.clone())
            .collect();
        out.sort();
        out
    }
}

impl SourceProvider for MemoryProvider {
    fn fetch(&self, reference: &SourceRef) -> Result<Option<Source>> {
        Ok(self.sources.get(reference).map(|t| Source {
            reference: reference.clone(),
            text: t.clone(),
            origin: format!("memory:{reference}"),
        }))
    }

    fn function_modules(&self, _group: &str) -> Result<Vec<String>> {
        Ok(Vec::new())
    }

    fn implementations(&self, interface: &str) -> Result<Vec<String>> {
        Ok(self
            .classes(|text, _| fs::interfaces_of(text).contains(&interface.to_ascii_uppercase())))
    }

    fn subclasses(&self, class: &str) -> Result<Vec<String>> {
        Ok(self.classes(|text, name| {
            fs::superclass_of(text, name).is_some_and(|s| s.eq_ignore_ascii_case(class))
        }))
    }

    fn describe(&self) -> String {
        "memory".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_ref_roundtrip() {
        for s in [
            "PROG:ZREPORT",
            "CLAS:ZCL_X",
            "CLAS:ZCL_X/implementations",
            "CLAS:/ABC/CL_X/testclasses",
            "CLAS:/ABC/CL_X",
            "INTF:/ABC/IF_X",
            "FUNC:Z_FM",
        ] {
            let r: SourceRef = s.parse().unwrap();
            assert_eq!(r.to_string(), s);
        }
        let r: SourceRef = "clas:zcl_x".parse().unwrap();
        assert_eq!(r, SourceRef::class("ZCL_X"));
        assert!("XYZ:A".parse::<SourceRef>().is_err());
        assert!("CLAS:ZCL_X/nope".parse::<SourceRef>().is_err());
    }
}
