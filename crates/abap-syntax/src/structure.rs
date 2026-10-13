//! Nesting of statements into blocks: CLASS ... ENDCLASS, METHOD ... ENDMETHOD,
//! IF ... ENDIF and so on. Error tolerant: an unexpected closing statement is
//! ignored, a missing one leaves the block unclosed.

use crate::{Statement, StatementKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    /// Statement index of the opening statement.
    pub start: usize,
    /// Statement index of the closing statement, `None` if missing.
    pub end: Option<usize>,
    pub children: Vec<Block>,
}

impl Block {
    /// Index of the last statement belonging to the block.
    pub fn last(&self) -> usize {
        self.end
            .unwrap_or_else(|| self.children.last().map(|c| c.last()).unwrap_or(self.start))
    }
}

/// The closing statement kind for a block opening statement.
pub fn closer(kind: StatementKind) -> Option<StatementKind> {
    use StatementKind::*;
    Some(match kind {
        ClassDefinition | ClassImplementation => EndClass,
        Interface => EndInterface,
        MethodImplementation => EndMethod,
        Form => EndForm,
        FunctionModule => EndFunction,
        Module => EndModule,
        If => EndIf,
        Case | CaseType => EndCase,
        Loop | LoopAtScreen | LoopExtract | DynproLoop => EndLoop,
        Do => EndDo,
        While => EndWhile,
        Try => EndTry,
        SelectLoop => EndSelect,
        WithLoop => EndWith,
        At | AtFirst | AtLast => EndAt,
        OnChange => EndOn,
        Provide => EndProvide,
        Define => EndOfDefinition,
        TestSeam => EndTestSeam,
        TestInjection => EndTestInjection,
        CatchSystemExceptions => EndCatch,
        Enhancement => EndEnhancement,
        EnhancementSection => EndEnhancementSection,
        ExecSQL => EndExec,
        Chain => EndChain,
        DataBegin => DataEnd,
        ClassDataBegin => ClassDataEnd,
        StaticBegin => StaticEnd,
        ConstantBegin => ConstantEnd,
        TypeBegin => TypeEnd,
        TypeEnumBegin => TypeEnumEnd,
        TypeMeshBegin => TypeMeshEnd,
        _ => return None,
    })
}

fn is_closer(kind: StatementKind) -> bool {
    use StatementKind::*;
    matches!(
        kind,
        EndClass
            | EndInterface
            | EndMethod
            | EndForm
            | EndFunction
            | EndModule
            | EndIf
            | EndCase
            | EndLoop
            | EndDo
            | EndWhile
            | EndTry
            | EndSelect
            | EndWith
            | EndAt
            | EndOn
            | EndProvide
            | EndOfDefinition
            | EndTestSeam
            | EndTestInjection
            | EndCatch
            | EndEnhancement
            | EndEnhancementSection
            | EndExec
            | EndChain
            | DataEnd
            | ClassDataEnd
            | StaticEnd
            | ConstantEnd
            | TypeEnd
            | TypeEnumEnd
            | TypeMeshEnd
    )
}

/// Builds the block tree. Statements that open no block are not included,
/// they are found by index ranges between `start` and `end`.
pub fn blocks(statements: &[Statement]) -> Vec<Block> {
    let mut roots: Vec<Block> = Vec::new();
    let mut stack: Vec<Block> = Vec::new();

    fn attach(stack: &mut [Block], roots: &mut Vec<Block>, block: Block) {
        match stack.last_mut() {
            Some(parent) => parent.children.push(block),
            None => roots.push(block),
        }
    }

    for (i, s) in statements.iter().enumerate() {
        if is_closer(s.kind) {
            let open = stack
                .iter()
                .rposition(|b| closer(statements[b.start].kind) == Some(s.kind));
            if let Some(pos) = open {
                while stack.len() > pos + 1 {
                    let unclosed = stack.pop().unwrap();
                    attach(&mut stack, &mut roots, unclosed);
                }
                let mut block = stack.pop().unwrap();
                block.end = Some(i);
                attach(&mut stack, &mut roots, block);
            }
        } else if closer(s.kind).is_some() {
            stack.push(Block {
                start: i,
                end: None,
                children: Vec::new(),
            });
        }
    }
    while let Some(unclosed) = stack.pop() {
        attach(&mut stack, &mut roots, unclosed);
    }
    roots
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nesting() {
        let f = crate::parse(
            "CLASS c IMPLEMENTATION.\n  METHOD m.\n    IF a = b.\n      x = 1.\n    ENDIF.\n  ENDMETHOD.\nENDCLASS.",
        );
        let b = blocks(&f.statements);
        assert_eq!(b.len(), 1);
        assert_eq!((b[0].start, b[0].end), (0, Some(6)));
        let m = &b[0].children[0];
        assert_eq!((m.start, m.end), (1, Some(5)));
        assert_eq!((m.children[0].start, m.children[0].end), (2, Some(4)));
    }

    #[test]
    fn missing_closer_is_tolerated() {
        let f = crate::parse("FORM a.\n  IF x = 1.\n    y = 2.\nENDFORM.\nENDIF.");
        let b = blocks(&f.statements);
        assert_eq!(b.len(), 1);
        assert_eq!(b[0].end, Some(3));
        assert_eq!(b[0].children[0].end, None);
        assert_eq!(b[0].children[0].last(), 1);
    }
}
