//! Code known to be valid at an older release: corpora whose CI checks
//! their `src/` with abaplint against a fixed ABAP release. None of their
//! statements may need a newer release, have a syntax error, or be unknown.
//!
//! Run with `cargo test -p abap-syntax --test checked_releases`.

use std::path::{Path, PathBuf};

use abap_syntax::check::syntax_error;
use abap_syntax::release::{statement_release, Release};
use abap_syntax::StatementKind;

/// Corpus folder, the release its CI checks against, files at least.
const CHECKED: [(&str, Release, usize); 6] = [
    // github.com/abap-wasm/abap-wasm: .github/workflows/test.yaml runs
    // `npm test`, which runs abaplint with abaplint.json, syntax v750
    ("abap-wasm/src", Release::V750, 300),
    // github.com/larshp/abapPGP: .github/workflows/test.yml, the same, v750
    ("abapPGP/src", Release::V750, 143),
    // github.com/larshp/abapNTLM: .github/workflows/test.yml, v740sp02
    ("abapNTLM/src", Release::V740Sp02, 14),
    // github.com/open-abap/open-abap-gui: .github/workflows/test.yml runs
    // `npm test`, which runs abaplint with abaplint.jsonc, v740sp08, on
    // these three folders
    ("open-abap-gui/src", Release::V740Sp08, 170),
    ("open-abap-gui/framework", Release::V740Sp08, 70),
    ("open-abap-gui/examples", Release::V740Sp08, 600),
];

fn abap_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            abap_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "abap")
            // screen flow logic, which the abaplint configs exclude
            && !path.to_string_lossy().contains(".prog.screen_")
        {
            out.push(path);
        }
    }
}

#[test]
fn corpora_need_no_newer_release() {
    let corpus = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/corpus");
    let mut wrong = Vec::new();
    for (folder, release, min_files) in CHECKED {
        let dir = corpus.join(folder);
        if !dir.exists() {
            println!("{folder}: skipped, submodule missing");
            continue;
        }
        let mut files = Vec::new();
        abap_files(&dir, &mut files);
        let mut statements = 0;
        for path in &files {
            let file = abap_syntax::parse(&std::fs::read_to_string(path).unwrap());
            let name = path.strip_prefix(&corpus).unwrap().display();
            for s in &file.statements {
                statements += 1;
                let row = file.tokens()[s.tokens[0]].start.row;
                if s.kind == StatementKind::Unknown {
                    wrong.push(format!("{name}:{row}: unknown statement"));
                }
                if let Some(need) = statement_release(&file, s).filter(|n| n.release > release) {
                    wrong.push(format!(
                        "{name}:{row}: needs {} for {}, checked at {}",
                        need.release.name(),
                        need.construct,
                        release.name()
                    ));
                }
                if let Some(e) = syntax_error(&file, s) {
                    wrong.push(format!("{name}:{row}: syntax error {e:?}"));
                }
            }
        }
        println!(
            "{folder}: {} files, {statements} statements, checked at {}",
            files.len(),
            release.name()
        );
        assert!(
            files.len() >= min_files,
            "{folder}: only {} files",
            files.len()
        );
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}
