//! Loading sources that are not local: one class is in a local folder, the
//! rest of abapGit is served by the ADT mock server. The call graph loads
//! what the class uses over ADT into the disk cache; a second run reads the
//! cache and asks the server for nothing it had.

use std::path::{Path, PathBuf};

use abap_analysis::abap_source::{
    AdtOptions, FsProvider, MockAdtConfig, MockAdtServer, MockRelease, ProviderOptions, SourceRef,
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
            proxy: None,
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

/// A call through an interface whose implementation is only in the system:
/// the where-used list of the interface names it, and the logic follows it.
#[test]
fn interface_implementations_from_the_system() {
    let work = temp_dir("adt-impl");
    let system = work.join("system");
    let local = work.join("local");
    std::fs::create_dir_all(&system).unwrap();
    std::fs::create_dir_all(&local).unwrap();
    let files = [
        (
            &system,
            "zif_store.intf.abap",
            "INTERFACE zif_store PUBLIC.\n  METHODS save.\nENDINTERFACE.",
        ),
        (
            &system,
            "zcl_db_store.clas.abap",
            "CLASS zcl_db_store DEFINITION PUBLIC.\n  PUBLIC SECTION.\n    INTERFACES zif_store.\nENDCLASS.\nCLASS zcl_db_store IMPLEMENTATION.\n  METHOD zif_store~save.\n    DELETE FROM zlog WHERE id = 1.\n  ENDMETHOD.\nENDCLASS.",
        ),
        (
            &system,
            "zcl_user.clas.abap",
            "CLASS zcl_user DEFINITION PUBLIC.\n  PRIVATE SECTION.\n    DATA mo TYPE REF TO zif_store.\nENDCLASS.\nCLASS zcl_user IMPLEMENTATION.\nENDCLASS.",
        ),
        (
            &local,
            "zcl_app.clas.abap",
            "CLASS zcl_app DEFINITION PUBLIC.\n  PUBLIC SECTION.\n    METHODS run.\n  PRIVATE SECTION.\n    DATA mo_store TYPE REF TO zif_store.\nENDCLASS.\nCLASS zcl_app IMPLEMENTATION.\n  METHOD run.\n    mo_store->save( ).\n  ENDMETHOD.\nENDCLASS.",
        ),
    ];
    for (dir, name, text) in files {
        std::fs::write(dir.join(name), text).unwrap();
    }
    let server = MockAdtServer::start(
        FsProvider::new(vec![system.clone()]).unwrap(),
        "127.0.0.1:0",
        MockAdtConfig::default(),
    )
    .unwrap();
    let options = ProviderOptions {
        src: vec![local.display().to_string()],
        cache: None,
        adt: Some(AdtOptions {
            url: server.url().to_string(),
            user: "DEVELOPER".into(),
            password: "mock".into(),
            client: None,
            language: None,
            insecure: false,
            proxy: None,
        }),
    };
    let ws = Workspace::new(options.build().unwrap());
    assert_eq!(ws.implementations("ZIF_STORE"), ["ZCL_DB_STORE"]);
    let logic = ws.logic(&SourceRef::class("ZCL_APP"), 1);
    assert!(logic.errors.is_empty(), "{:?}", logic.errors);
    let run = logic
        .procedures
        .iter()
        .find(|p| p.block.name == "run")
        .unwrap();
    let e = run.effects.iter().find(|e| e.name == "ZLOG").unwrap();
    assert_eq!(e.via[0].class.as_deref(), Some("zcl_db_store"));
    assert!(server
        .requests()
        .iter()
        .any(|r| r.contains("usageReferences")));
    drop(server);
    let _ = std::fs::remove_dir_all(work);
}

/// A system on SAP_BASIS 750 with the code of a folder: the call graph over
/// ADT is the one of the folder, with one note that the code of
/// enhancements cannot be read there.
#[test]
fn basis_750_gives_the_graph_of_the_folder() {
    let corpus = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/corpus/abapgit/src");
    if !corpus.exists() {
        println!("skipped: abapGit submodule missing");
        return;
    }
    // calls through interfaces: their implementations come from the class
    // relations through the data preview
    let root = SourceRef::class("ZCL_ABAPGIT_REPO_STATUS");
    let local = ProviderOptions {
        src: vec![corpus.display().to_string()],
        cache: None,
        adt: None,
    };
    let expected =
        Workspace::new(local.build().unwrap()).call_graph(std::slice::from_ref(&root), 2);

    let config = MockAdtConfig {
        release: MockRelease::Basis750,
        ..MockAdtConfig::default()
    };
    let server = MockAdtServer::start(
        FsProvider::new(vec![corpus]).unwrap(),
        "127.0.0.1:0",
        config,
    )
    .unwrap();
    let options = ProviderOptions {
        src: Vec::new(),
        cache: None,
        adt: Some(AdtOptions {
            url: server.url().to_string(),
            user: "DEVELOPER".into(),
            password: "mock".into(),
            client: None,
            language: None,
            insecure: false,
            proxy: None,
        }),
    };
    let graph = Workspace::new(options.build().unwrap()).call_graph(std::slice::from_ref(&root), 2);
    assert!(graph.errors.is_empty(), "{:?}", graph.errors);
    assert_eq!(
        graph.notes,
        ["enhancements: not available (the ADT discovery has no enhancements/enhoxhh to read their code, SAP_BASIS 750 SP 0035)"]
    );
    let sorted = |mut r: Vec<String>| {
        r.sort();
        r
    };
    assert_eq!(
        sorted(graph.units.iter().map(|u| u.reference.clone()).collect()),
        sorted(expected.units.iter().map(|u| u.reference.clone()).collect())
    );
    assert_eq!(graph.calls, expected.calls);
    assert_eq!(graph.missing, expected.missing);
    let requests = server.requests();
    let previews = requests
        .iter()
        .filter(|r| r.contains("datapreview"))
        .count();
    assert!(previews > 0, "no data preview query");
    println!(
        "{} units, {} calls, {} requests, {previews} data preview queries",
        graph.units.len(),
        graph.calls.len(),
        requests.len()
    );
    drop(server);
}
