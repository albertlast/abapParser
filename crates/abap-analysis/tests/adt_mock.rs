//! Loading sources that are not local: one class is in a local folder, the
//! rest of abapGit is served by the ADT mock server. The call graph loads
//! what the class uses over ADT into the disk cache; a second run reads the
//! cache and asks the server for nothing it had.

use std::path::{Path, PathBuf};

use abap_analysis::abap_source::{
    AdtOptions, FsProvider, MockAdtConfig, MockAdtServer, ProviderOptions, SourceRef,
};
use abap_analysis::Workspace;

const ROOT: &str = "zcl_abapgit_apack_migration";

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("abap-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// The files of one class, copied from the corpus.
fn copy_class(from: &Path, to: &Path, class: &str) {
    for e in std::fs::read_dir(from).unwrap().flatten() {
        let name = e.file_name().to_string_lossy().to_string();
        if name.starts_with(&format!("{class}.clas.")) && name.ends_with(".abap") {
            std::fs::copy(e.path(), to.join(&name)).unwrap();
        }
    }
}

#[test]
fn sources_missing_locally_are_loaded_over_adt() {
    let corpus = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/corpus/abapgit/src");
    if !corpus.exists() {
        println!("skipped: abapGit submodule missing");
        return;
    }
    let server = MockAdtServer::start(
        FsProvider::new(vec![corpus.clone()]).unwrap(),
        "127.0.0.1:0",
        MockAdtConfig::default(),
    )
    .unwrap();

    let work = temp_dir("adt-mock");
    let local = work.join("src");
    std::fs::create_dir_all(&local).unwrap();
    copy_class(&corpus.join("apack"), &local, ROOT);
    let options = ProviderOptions {
        src: vec![local.display().to_string()],
        cache: Some(work.join("cache").display().to_string()),
        adt: Some(AdtOptions {
            url: server.url().to_string(),
            user: "DEVELOPER".into(),
            password: "mock".into(),
            client: None,
            language: None,
            insecure: false,
        }),
    };
    let root = SourceRef::class(&ROOT.to_ascii_uppercase());

    let first = Workspace::new(options.build().unwrap()).call_graph(std::slice::from_ref(&root), 2);
    assert!(first.errors.is_empty(), "{:?}", first.errors);
    let local_units: Vec<_> = first
        .units
        .iter()
        .filter(|u| !u.origin.starts_with(server.url()))
        .collect();
    let remote_units: Vec<_> = first
        .units
        .iter()
        .filter(|u| u.origin.starts_with(server.url()))
        .collect();
    assert!(local_units.iter().all(|u| u
        .reference
        .starts_with(&format!("CLAS:{}", ROOT.to_ascii_uppercase()))));
    assert!(remote_units.len() >= 2, "{:#?}", first.units);
    // calls into classes that only came over ADT resolve to their methods
    let remote_classes: Vec<String> = remote_units
        .iter()
        .filter_map(|u| u.reference.strip_prefix("CLAS:"))
        .map(|r| r.split('/').next().unwrap().to_string())
        .collect();
    let resolved_remote = first
        .calls
        .iter()
        .filter(|c| {
            c.resolved
                && c.class
                    .as_deref()
                    .is_some_and(|k| remote_classes.contains(&k.to_ascii_uppercase()))
        })
        .count();
    assert!(resolved_remote > 0, "no resolved call into a loaded class");
    let requests_first = server.requests().len();
    assert!(requests_first >= remote_units.len());

    // the second run reads the loaded sources from the cache
    let second =
        Workspace::new(options.build().unwrap()).call_graph(std::slice::from_ref(&root), 2);
    let requests: Vec<String> = server.requests()[requests_first..].to_vec();
    for u in &remote_units {
        let path = u.origin.trim_start_matches(server.url());
        assert!(
            !requests.iter().any(|r| r.starts_with(path)),
            "{} fetched again",
            u.reference
        );
    }
    assert!(second
        .units
        .iter()
        .all(|u| !u.origin.starts_with(server.url())));
    assert_eq!(first.calls.len(), second.calls.len());
    println!(
        "{} units, {} over ADT, {resolved_remote} resolved calls into them; second run: {} requests, all for objects the server does not have",
        first.units.len(),
        remote_units.len(),
        requests.len()
    );
    drop(server);
    let _ = std::fs::remove_dir_all(work);
}
