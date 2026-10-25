//! Disk cache in front of a slow provider, typically the ADT one: the
//! sources, and the answers of lookups like the enhancements of a program
//! or the implementations of an interface, under `lookups/`.

use std::path::{Path, PathBuf};

use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::{Badi, Hook, ObjectKind, Result, Source, SourceError, SourceProvider, SourceRef};

pub struct CachedProvider<P> {
    dir: PathBuf,
    inner: P,
}

impl<P: SourceProvider> CachedProvider<P> {
    /// `dir` should identify the system, e.g. `~/.cache/abap/DEV-100`, since
    /// objects differ between systems and clients.
    pub fn new(dir: impl Into<PathBuf>, inner: P) -> Self {
        CachedProvider {
            dir: dir.into(),
            inner,
        }
    }

    fn path(&self, r: &SourceRef) -> PathBuf {
        let mut name = r.name.replace('/', "#");
        if let Some(p) = r.part {
            name.push('.');
            name.push_str(p.as_str());
        }
        let extension = match r.kind {
            ObjectKind::Dtel | ObjectKind::Doma | ObjectKind::Ttyp => "xml",
            ObjectKind::Tabl => "ddl",
            ObjectKind::Ddls => "asddls",
            _ => "abap",
        };
        self.dir
            .join(r.kind.as_str())
            .join(format!("{name}.{extension}"))
    }

    /// The answer to a lookup from the cache, else from `ask`, then kept.
    fn lookup<T: Serialize + DeserializeOwned>(
        &self,
        kind: &str,
        key: &str,
        ask: impl FnOnce() -> Result<T>,
    ) -> Result<T> {
        let path = self
            .dir
            .join("lookups")
            .join(kind)
            .join(format!("{}.json", file_key(key)));
        if let Some(answer) = std::fs::read_to_string(&path)
            .ok()
            .and_then(|text| serde_json::from_str(&text).ok())
        {
            return Ok(answer);
        }
        let notes = self.inner.notes().len();
        let answer = ask()?;
        // an answer the system could not give is no answer to keep
        if self.inner.notes().len() == notes {
            let text =
                serde_json::to_string(&answer).map_err(|e| SourceError::Other(e.to_string()))?;
            write_atomic(&path, &text)?;
        }
        Ok(answer)
    }

    /// Removes a cached object, so the next fetch asks the inner provider.
    pub fn invalidate(&self, r: &SourceRef) -> Result<()> {
        match std::fs::remove_file(self.path(r)) {
            Err(e) if e.kind() != std::io::ErrorKind::NotFound => Err(e.into()),
            _ => Ok(()),
        }
    }
}

/// A file name for a lookup: the name itself when it is short and plain,
/// else a hash of it.
fn file_key(key: &str) -> String {
    let plain = key
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '/' | '='));
    if plain && key.len() <= 60 {
        return key.replace('/', "#");
    }
    // FNV-1a, stable across runs and versions
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for b in key.bytes() {
        hash ^= u64::from(b);
        hash = hash.wrapping_mul(0x0100_0000_01b3);
    }
    format!("{hash:016x}")
}

fn write_atomic(path: &Path, text: &str) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    // objects load in parallel: a temp file of its own, or two loads of the
    // same object rename each other's file away
    static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let n = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let tmp = path.with_extension(format!("{}.{n}.tmp", std::process::id()));
    std::fs::write(&tmp, text)?;
    std::fs::rename(&tmp, path)
}

impl<P: SourceProvider> SourceProvider for CachedProvider<P> {
    fn fetch(&self, r: &SourceRef) -> Result<Option<Source>> {
        let path = self.path(r);
        if let Ok(text) = std::fs::read_to_string(&path) {
            return Ok(Some(Source {
                reference: r.clone(),
                text,
                origin: path.display().to_string(),
            }));
        }
        let fetched = self.inner.fetch(r)?;
        if let Some(s) = &fetched {
            write_atomic(&path, &s.text)?;
        }
        Ok(fetched)
    }

    fn function_modules(&self, group: &str) -> Result<Vec<String>> {
        self.inner.function_modules(group)
    }

    fn function_interface(&self, function: &str) -> Result<Option<String>> {
        self.inner.function_interface(function)
    }

    fn screens(&self, program: &str) -> Result<Vec<crate::Screen>> {
        self.inner.screens(program)
    }

    fn implementations(&self, interface: &str) -> Result<Vec<String>> {
        let key = interface.to_ascii_uppercase();
        self.lookup("implementations", &key, || self.inner.implementations(&key))
    }

    fn enhancements(&self, programs: &[String], function_modules: &[String]) -> Result<Vec<Hook>> {
        // one question per object: its programs and function modules
        let key = format!(
            "{}|{}",
            programs.join(",").to_ascii_uppercase(),
            function_modules.join(",").to_ascii_uppercase()
        );
        self.lookup("enhancements", &key, || {
            self.inner.enhancements(programs, function_modules)
        })
    }

    fn badi(&self, name: &str) -> Result<Option<Badi>> {
        let key = name.to_ascii_uppercase();
        self.lookup("badis", &key, || self.inner.badi(&key))
    }

    fn classic_badi(&self, interface: &str) -> Result<Option<Badi>> {
        let key = interface.to_ascii_uppercase();
        self.lookup("classic_badis", &key, || self.inner.classic_badi(&key))
    }

    fn subclasses(&self, class: &str) -> Result<Vec<String>> {
        let key = class.to_ascii_uppercase();
        self.lookup("subclasses", &key, || self.inner.subclasses(&key))
    }

    fn notes(&self) -> Vec<String> {
        self.inner.notes()
    }

    fn describe(&self) -> String {
        format!("cache {} -> {}", self.dir.display(), self.inner.describe())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MemoryProvider;

    #[test]
    fn caches_on_disk() {
        let dir = std::env::temp_dir().join(format!("abap-cache-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let mut mem = MemoryProvider::new();
        mem.insert(SourceRef::class("/ABC/CL_X"), "CLASS /abc/cl_x DEFINITION.");
        let cached = CachedProvider::new(&dir, mem);
        let first = cached
            .fetch(&SourceRef::class("/ABC/CL_X"))
            .unwrap()
            .unwrap();
        assert!(first.origin.starts_with("memory:"));
        let second = cached
            .fetch(&SourceRef::class("/ABC/CL_X"))
            .unwrap()
            .unwrap();
        assert!(second.origin.ends_with("#ABC#CL_X.abap"));
        assert_eq!(second.text, first.text);
        assert!(cached
            .fetch(&SourceRef::class("ZCL_NONE"))
            .unwrap()
            .is_none());
        cached.invalidate(&SourceRef::class("/ABC/CL_X")).unwrap();
        let third = cached
            .fetch(&SourceRef::class("/ABC/CL_X"))
            .unwrap()
            .unwrap();
        assert!(third.origin.starts_with("memory:"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Counts the lookups that reach the provider.
    struct Counting {
        inner: MemoryProvider,
        asked: std::sync::atomic::AtomicUsize,
    }

    impl SourceProvider for Counting {
        fn fetch(&self, r: &SourceRef) -> Result<Option<Source>> {
            self.inner.fetch(r)
        }

        fn enhancements(
            &self,
            programs: &[String],
            function_modules: &[String],
        ) -> Result<Vec<Hook>> {
            self.asked.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            self.inner.enhancements(programs, function_modules)
        }

        fn badi(&self, name: &str) -> Result<Option<Badi>> {
            self.asked.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            self.inner.badi(name)
        }

        fn describe(&self) -> String {
            "counting".into()
        }
    }

    #[test]
    fn caches_lookups_on_disk() {
        let dir = std::env::temp_dir().join(format!("abap-cache-lookups-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let mut mem = MemoryProvider::new();
        mem.insert_hook(Hook {
            enhancement: "ZENH".into(),
            position: r"\PR:ZORDERS\FO:SAVE\SE:END\EI".into(),
            code: "ENHANCEMENT 1 ZENH.\nENDENHANCEMENT.".into(),
            origin: "memory:ZENH".into(),
        });
        let counting = Counting {
            inner: mem,
            asked: Default::default(),
        };
        let cached = CachedProvider::new(&dir, counting);
        let programs = ["ZORDERS".to_string(), "ZORDERS_F01".to_string()];
        for _ in 0..2 {
            let hooks = cached.enhancements(&programs, &[]).unwrap();
            assert_eq!(hooks.len(), 1);
            assert_eq!(hooks[0].enhancement, "ZENH");
            // nothing found is an answer too
            assert!(cached
                .enhancements(&["ZOTHER".into()], &[])
                .unwrap()
                .is_empty());
            assert!(cached.badi("zbadi_none").unwrap().is_none());
        }
        // asked once per question, then from the disk, also by a new cache
        let again = CachedProvider::new(&dir, MemoryProvider::new());
        assert_eq!(again.enhancements(&programs, &[]).unwrap().len(), 1);
        assert_eq!(
            cached.inner.asked.load(std::sync::atomic::Ordering::SeqCst),
            3
        );
        assert_eq!(file_key("ZCL_X"), "ZCL_X");
        assert_eq!(file_key("/ABC/IF_X"), "#ABC#IF_X");
        assert_eq!(file_key("ZORDERS|").len(), 16);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
