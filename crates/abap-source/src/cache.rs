//! Disk cache in front of a slow provider, typically the ADT one.

use std::path::{Path, PathBuf};

use crate::{Result, Source, SourceProvider, SourceRef};

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
        self.dir.join(r.kind.as_str()).join(format!("{name}.abap"))
    }

    /// Removes a cached object, so the next fetch asks the inner provider.
    pub fn invalidate(&self, r: &SourceRef) -> Result<()> {
        match std::fs::remove_file(self.path(r)) {
            Err(e) if e.kind() != std::io::ErrorKind::NotFound => Err(e.into()),
            _ => Ok(()),
        }
    }
}

fn write_atomic(path: &Path, text: &str) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let tmp = path.with_extension("tmp");
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

    fn implementations(&self, interface: &str) -> Result<Vec<String>> {
        self.inner.implementations(interface)
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
}
