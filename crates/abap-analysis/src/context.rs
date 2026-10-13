//! The processing block every statement belongs to: method, form, function
//! module, dialog module, event block or the program itself.

use abap_model as m;
use abap_syntax::decl;
use abap_syntax::structure::{self, Block};
use abap_syntax::{File, StatementKind as K};

fn block_ref(kind: &str, name: String, class: Option<String>) -> m::BlockRef {
    m::BlockRef {
        kind: kind.to_string(),
        name,
        class,
    }
}

fn is_event(kind: K) -> bool {
    matches!(
        kind,
        K::StartOfSelection
            | K::EndOfSelection
            | K::Initialization
            | K::TopOfPage
            | K::EndOfPage
            | K::LoadOfProgram
            | K::AtSelectionScreen
            | K::AtLineSelection
            | K::AtUserCommand
            | K::AtPF
            | K::Get
    )
}

/// One entry per statement; `None` for declarative parts like class
/// definitions and for statements before any processing block.
pub fn statement_blocks(file: &File) -> Vec<Option<m::BlockRef>> {
    let n = file.statements.len();
    let mut out: Vec<Option<m::BlockRef>> = vec![None; n];
    let name = |i: usize, skip: usize| {
        decl::first_name(file, &file.statements[i], skip)
            .map(|x| x.0)
            .unwrap_or_default()
    };
    let fill = |out: &mut Vec<Option<m::BlockRef>>, b: &Block, r: m::BlockRef| {
        for slot in out.iter_mut().take(b.last() + 1).skip(b.start) {
            *slot = Some(r.clone());
        }
    };

    let blocks = structure::blocks(&file.statements);
    let mut processing: Vec<(usize, usize)> = Vec::new();
    for b in &blocks {
        let kind = file.statements[b.start].kind;
        match kind {
            K::ClassImplementation => {
                let class = name(b.start, 1);
                for c in &b.children {
                    if file.statements[c.start].kind == K::MethodImplementation {
                        fill(
                            &mut out,
                            c,
                            block_ref("method", name(c.start, 1), Some(class.clone())),
                        );
                    }
                }
            }
            K::Form => fill(&mut out, b, block_ref("form", name(b.start, 1), None)),
            K::FunctionModule => fill(
                &mut out,
                b,
                block_ref("function_module", name(b.start, 1), None),
            ),
            K::Module => fill(&mut out, b, block_ref("module", name(b.start, 1), None)),
            K::ClassDefinition | K::Interface | K::Define => {}
            _ => continue,
        }
        processing.push((b.start, b.last()));
    }

    // event blocks and the program level: everything not inside a block above
    let inside = |i: usize| processing.iter().any(|(s, e)| (*s..=*e).contains(&i));
    let mut current: Option<m::BlockRef> = None;
    for (i, (s, slot)) in file.statements.iter().zip(out.iter_mut()).enumerate() {
        if inside(i) {
            if matches!(
                s.kind,
                K::Form | K::FunctionModule | K::Module | K::ClassImplementation
            ) {
                current = None;
            }
            continue;
        }
        if is_event(s.kind) {
            let event = file.statement_text(s).trim_end_matches('.').to_string();
            current = Some(block_ref("event", event, None));
        }
        if !matches!(s.kind, K::Comment | K::Empty) {
            *slot = Some(
                current
                    .clone()
                    .unwrap_or_else(|| block_ref("program", String::new(), None)),
            );
        }
    }
    out
}
