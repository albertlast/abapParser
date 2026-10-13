//! Provider setup from options, shared by the CLI and the language bindings.

use std::path::PathBuf;

use serde::Deserialize;

use crate::{ChainProvider, Result, SourceError, SourceProvider};

/// Where to look for sources: local folders first, then an SAP system
/// through ADT, optionally behind a disk cache.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ProviderOptions {
    /// folders with abapGit or plain `.abap` files
    pub src: Vec<String>,
    /// cache folder for sources fetched from the SAP system
    pub cache: Option<String>,
    pub adt: Option<AdtOptions>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct AdtOptions {
    pub url: String,
    pub user: String,
    pub password: String,
    pub client: Option<String>,
    pub language: Option<String>,
    /// skip TLS certificate checks, only for test systems
    pub insecure: bool,
}

impl ProviderOptions {
    pub fn build(&self) -> Result<Box<dyn SourceProvider>> {
        let mut providers: Vec<Box<dyn SourceProvider>> = Vec::new();
        if !self.src.is_empty() {
            let roots = self.src.iter().map(PathBuf::from).collect();
            providers.push(Box::new(crate::FsProvider::new(roots)?));
        }
        if let Some(adt) = &self.adt {
            providers.push(adt_provider(adt, self.cache.as_deref())?);
        }
        if providers.is_empty() {
            return Err(SourceError::Other(
                "no source: give folders (src) or an SAP system (adt)".to_string(),
            ));
        }
        Ok(if providers.len() == 1 {
            providers.pop().unwrap()
        } else {
            Box::new(ChainProvider::new(providers))
        })
    }
}

#[cfg(feature = "adt")]
fn adt_provider(o: &AdtOptions, cache: Option<&str>) -> Result<Box<dyn SourceProvider>> {
    let mut config = crate::AdtConfig::new(&o.url, &o.user, &o.password);
    config.client = o.client.clone();
    config.language = o.language.clone();
    config.insecure = o.insecure;
    let adt = crate::AdtProvider::new(config);
    Ok(match cache {
        Some(dir) => {
            // one cache folder per system and client
            let host: String = o
                .url
                .trim_start_matches("https://")
                .trim_start_matches("http://")
                .chars()
                .map(|c| {
                    if c.is_ascii_alphanumeric() || c == '.' || c == '-' {
                        c
                    } else {
                        '_'
                    }
                })
                .collect();
            let folder = format!("{host}-{}", o.client.as_deref().unwrap_or("default"));
            Box::new(crate::CachedProvider::new(
                PathBuf::from(dir).join(folder),
                adt,
            ))
        }
        None => Box::new(adt),
    })
}

#[cfg(not(feature = "adt"))]
fn adt_provider(_: &AdtOptions, _: Option<&str>) -> Result<Box<dyn SourceProvider>> {
    Err(SourceError::Other("built without ADT support".to_string()))
}
