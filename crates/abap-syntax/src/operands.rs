//! What a statement does with its operands: the data objects it reads,
//! writes or changes, and its conditions.
//!
//! Most statements are described by keyword rules: a keyword switches the
//! role of the operands that follow it, e.g. after `INTO` they are written.
//! Assignments, method calls, messages, data clusters and SQL have their own
//! code. Statements without rules have no operands.

use crate::expr::{self, Access, Cond, DataRef, Expr, ExprKind, Head, Link, Parser};
use crate::{File, Statement, StatementKind as K, TokenKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Operand {
    Expr { expr: Expr, access: Access },
    Cond(Cond),
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Operands {
    pub items: Vec<Operand>,
}

impl Operands {
    /// The data references in source order of the operands.
    pub fn refs(&self) -> Vec<DataRef> {
        let mut out = Vec::new();
        for item in &self.items {
            match item {
                Operand::Expr { expr, access } => expr::expr_refs(expr, *access, &mut out),
                Operand::Cond(c) => expr::cond_refs(c, &mut out),
            }
        }
        out
    }

    /// The conditions of the statement, like the one of IF or LOOP ... WHERE.
    pub fn conds(&self) -> impl Iterator<Item = &Cond> {
        self.items.iter().filter_map(|i| match i {
            Operand::Cond(c) => Some(c),
            _ => None,
        })
    }

    /// The expressions with the given access.
    pub fn exprs(&self, access: Access) -> impl Iterator<Item = &Expr> {
        self.items.iter().filter_map(move |i| match i {
            Operand::Expr { expr, access: a } if *a == access => Some(expr),
            _ => None,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    Read,
    /// one operand read, then [`Mode::Skip`]
    ReadOne,
    Write,
    Change,
    /// `name = value` pairs whose values are read, written or changed
    NamedRead,
    NamedWrite,
    NamedChange,
    Cond,
    ComponentCond,
    /// keywords, type, component and class names
    Skip,
    /// a method, possibly behind the objects to reach it: `lo->attr->meth`
    Method,
    /// `name [operator] value` of SUBMIT ... WITH
    Selection,
    /// ends the rule scan
    Stop,
}

use Mode::*;

struct Scan<'a> {
    p: Parser<'a>,
    out: Vec<Operand>,
}

/// Number of tokens the rule's words match at the cursor; words with dashes
/// like `NO-GAPS` match the three tokens `NO`, `-`, `GAPS`.
fn rule_len(p: &Parser, rule: &str) -> Option<usize> {
    let mut k = 0;
    for word in rule.split(' ') {
        for (i, part) in word.split('-').enumerate() {
            if i > 0 {
                let t = p.peek(k)?;
                if t.kind != TokenKind::Dash || t.ws_before {
                    return None;
                }
                k += 1;
            }
            let t = p.peek(k)?;
            if t.kind != TokenKind::Identifier || !t.is(part) {
                return None;
            }
            k += 1;
        }
    }
    Some(k)
}

impl<'a> Scan<'a> {
    fn new(file: &'a File, s: &'a Statement) -> Self {
        Scan {
            p: Parser::new(file, s),
            out: Vec::new(),
        }
    }

    fn rule_at(&self, p: &Parser, rules: &[(&str, Mode)]) -> Option<(usize, Mode)> {
        rules
            .iter()
            .filter_map(|(r, m)| rule_len(p, r).map(|n| (n, *m)))
            .max_by_key(|(n, _)| *n)
    }

    /// The longest rule matching at the cursor.
    fn rule(&self, rules: &[(&str, Mode)]) -> Option<(usize, Mode)> {
        self.rule_at(&self.p, rules)
    }

    /// Operands in `mode` until a rule switches the mode, up to the end or
    /// a `Stop` rule.
    fn run(&mut self, mut mode: Mode, rules: &[(&str, Mode)]) {
        // after a keyword an operand follows, even one named like a keyword:
        // `INDEX index`
        let mut operand_due = false;
        while !self.p.done() {
            let keyword = self.rule(rules).filter(|(n, _)| {
                // a single word that would leave no operand after it is the
                // operand itself
                let probe = self.p.clone_at(self.p.pos + n);
                !operand_due || *n > 1 || (!probe.done() && self.rule_at(&probe, rules).is_none())
            });
            if let Some((n, m)) = keyword {
                if m == Stop {
                    return;
                }
                self.p.pos += n;
                mode = m;
                operand_due = !matches!(m, Skip | Stop);
                continue;
            }
            operand_due = false;
            let before = self.p.pos;
            self.operand(mode);
            if self.p.pos == before {
                self.p.bump();
            }
            if mode == ReadOne {
                mode = Skip;
            }
        }
    }

    fn push(&mut self, expr: Option<Expr>, access: Access) {
        if let Some(expr) = expr {
            self.out.push(Operand::Expr { expr, access });
        }
    }

    fn operand(&mut self, mode: Mode) {
        match mode {
            Read | ReadOne => {
                let e = self.p.source();
                self.push(e, Access::Read);
            }
            Write => {
                let e = self.p.target();
                self.push(e, Access::Write);
            }
            Change => {
                let e = self.p.target();
                self.push(e, Access::Change);
            }
            NamedRead | NamedWrite | NamedChange => {
                if !self.p.named_at() {
                    // `TABLES itab` of PERFORM-like lists, `OTHERS`
                    self.p.bump();
                    return;
                }
                self.p.skip_component_name();
                self.p.bump();
                match mode {
                    NamedRead => self.operand(Read),
                    NamedWrite => self.operand(Write),
                    _ => self.operand(Change),
                }
            }
            Cond => {
                if let Some(c) = self.p.cond() {
                    self.out.push(Operand::Cond(c));
                }
            }
            ComponentCond => {
                if let Some(c) = self.p.component_cond() {
                    self.out.push(Operand::Cond(c));
                }
            }
            Skip => self.skip(),
            Method => self.method(),
            Selection => self.selection(),
            Stop => {}
        }
    }

    /// Skips a token; what is in a dynamic `(name)` is read.
    fn skip(&mut self) {
        let dynamic = self.p.peek(0).is_some_and(|t| t.ws_before) && self.p.dynamic_at(0);
        if dynamic {
            let e = self.p.source();
            self.push(e, Access::Read);
        } else {
            self.p.bump();
        }
    }

    /// A method designator: the objects before the method name are read.
    fn method(&mut self) {
        let Some(mut e) = self.p.source() else {
            self.skip();
            return;
        };
        if let ExprKind::Chain(c) = &mut e.kind {
            match c.links.last() {
                Some(Link::Attribute { .. }) => {
                    c.links.pop();
                }
                None if matches!(c.head, Head::Name(_)) => return,
                _ => {}
            }
        }
        self.push(Some(e), Access::Read);
    }

    /// `sel [=|EQ|IN|BETWEEN ...] value [AND value]` of SUBMIT ... WITH.
    fn selection(&mut self) {
        self.p.bump();
        while self.p.word(0).is_some_and(|w| {
            matches!(
                w.as_str(),
                "=" | "EQ"
                    | "NE"
                    | "CP"
                    | "NP"
                    | "GT"
                    | "GE"
                    | "LT"
                    | "LE"
                    | "IN"
                    | "BETWEEN"
                    | "NOT"
                    | "SIGN"
                    | "INCL"
            )
        }) {
            self.p.bump();
        }
        self.operand(Read);
        if self.p.eat("AND") {
            self.operand(Read);
        }
    }
}

const STRING_MODES: &[(&str, Mode)] = &[
    ("IN CHARACTER MODE", Skip),
    ("IN BYTE MODE", Skip),
    ("IGNORING CASE", Skip),
    ("RESPECTING CASE", Skip),
];

const TARGET_ADDITIONS: &[(&str, Mode)] = &[
    ("ASSIGNING", Write),
    ("REFERENCE INTO", Write),
    ("CASTING", Skip),
    ("ELSE UNASSIGN", Skip),
];

fn with(base: &[(&'static str, Mode)], more: &[(&'static str, Mode)]) -> Vec<(&'static str, Mode)> {
    base.iter().chain(more).copied().collect()
}

/// The operands of a statement.
pub fn operands(file: &File, s: &Statement) -> Operands {
    let mut scan = Scan::new(file, s);
    let sc = &mut scan;
    match s.kind {
        K::Move => assignment(sc),
        K::Compute => {
            sc.p.eat("COMPUTE");
            sc.p.eat("EXACT");
            assignment(sc);
        }
        K::Call | K::CallBadi => call(sc),
        K::If | K::ElseIf | K::Check => {
            sc.p.bump();
            sc.operand(Cond);
        }
        K::While => sc.run(
            Skip,
            &[
                ("WHILE", Cond),
                ("VARY", Write),
                ("FROM", Read),
                ("NEXT", Read),
                ("RANGE", Read),
            ],
        ),
        K::Assert => sc.run(
            Skip,
            &[
                ("ASSERT", Cond),
                ("ID", Skip),
                ("SUBKEY", Read),
                ("FIELDS", Read),
                ("CONDITION", Cond),
            ],
        ),
        K::Case | K::CaseType => sc.run(Skip, &[("CASE", Read), ("CASE TYPE OF", Read)]),
        K::When => sc.run(Skip, &[("WHEN", Read), ("OR", Read)]),
        K::WhenType => sc.run(Skip, &[("WHEN TYPE", Skip), ("INTO", Write)]),
        K::Do => sc.run(
            Skip,
            &[
                ("DO", Read),
                ("TIMES", Skip),
                ("VARYING", Write),
                ("FROM", Read),
                ("NEXT", Read),
                ("RANGE", Read),
            ],
        ),
        K::Return => sc.run(Skip, &[("RETURN", Read)]),
        K::Clear => sc.run(
            Skip,
            &[
                ("CLEAR", Write),
                ("WITH", Read),
                ("IN CHARACTER MODE", Skip),
                ("IN BYTE MODE", Skip),
            ],
        ),
        K::Free => sc.run(Skip, &[("FREE", Write)]),
        K::Refresh => sc.run(Skip, &[("REFRESH", Write), ("FROM TABLE", Skip)]),
        K::Unassign => sc.run(Skip, &[("UNASSIGN", Write)]),
        K::Append => sc.run(
            Skip,
            &with(
                TARGET_ADDITIONS,
                &[
                    ("APPEND", Read),
                    ("INITIAL LINE", Skip),
                    ("LINES OF", Read),
                    ("FROM", Read),
                    ("TO", Change),
                    ("USING KEY", Skip),
                    ("SORTED BY", Skip),
                ],
            ),
        ),
        K::InsertInternal => sc.run(
            Skip,
            &with(
                TARGET_ADDITIONS,
                &[
                    ("INSERT", Read),
                    ("INITIAL LINE", Skip),
                    ("LINES OF", Read),
                    ("FROM", Read),
                    ("TO", Read),
                    ("USING KEY", Skip),
                    ("INTO", Change),
                    ("INTO TABLE", Change),
                    ("INSERT TABLE", Change),
                    ("INDEX", Read),
                ],
            ),
        ),
        K::Collect => sc.run(
            Skip,
            &with(TARGET_ADDITIONS, &[("COLLECT", Read), ("INTO", Change)]),
        ),
        K::ModifyInternal => sc.run(
            Skip,
            &with(
                TARGET_ADDITIONS,
                &[
                    ("MODIFY", Change),
                    ("MODIFY TABLE", Change),
                    ("USING KEY", Skip),
                    ("FROM", Read),
                    ("INDEX", Read),
                    ("TRANSPORTING", Skip),
                    ("WHERE", ComponentCond),
                ],
            ),
        ),
        K::DeleteInternal => sc.run(
            Skip,
            &[
                ("DELETE", Change),
                ("DELETE TABLE", Change),
                ("DELETE ADJACENT DUPLICATES FROM", Change),
                ("USING KEY", Skip),
                ("FROM", Read),
                ("TO", Read),
                ("INDEX", Read),
                ("WHERE", ComponentCond),
                ("WITH TABLE KEY", NamedRead),
                ("COMPARING", Skip),
            ],
        ),
        K::ReadTable => read_table(sc),
        K::Loop => loop_at(sc),
        K::Sort => sc.run(Skip, &[("SORT", Change), ("STABLE", Skip), ("BY", Skip)]),
        K::Assign => assign(sc),
        K::GetReference => sc.run(Skip, &[("GET REFERENCE OF", Read), ("INTO", Write)]),
        K::CreateObject => sc.run(
            Skip,
            &[
                ("CREATE OBJECT", Write),
                ("TYPE", Skip),
                ("AREA HANDLE", Read),
                ("EXPORTING", NamedRead),
                ("EXCEPTIONS", Skip),
                ("PARAMETER-TABLE", Read),
                ("EXCEPTION-TABLE", Read),
            ],
        ),
        K::CreateData => sc.run(
            Skip,
            &[
                ("CREATE DATA", Write),
                ("TYPE", Skip),
                ("TYPE HANDLE", Read),
                ("LIKE", Read),
                ("LIKE LINE OF", Read),
                ("LIKE TABLE OF", Read),
                ("LIKE STANDARD TABLE OF", Read),
                ("LIKE REF TO", Read),
                ("LIKE RANGE OF", Read),
                ("AREA HANDLE", Read),
                ("WITH", Skip),
                ("INITIAL SIZE", Read),
            ],
        ),
        K::Concatenate => sc.run(
            Skip,
            &with(
                STRING_MODES,
                &[
                    ("CONCATENATE", Read),
                    ("LINES OF", Read),
                    ("INTO", Write),
                    ("SEPARATED BY", Read),
                    ("RESPECTING BLANKS", Skip),
                ],
            ),
        ),
        K::Split => sc.run(
            Skip,
            &with(
                STRING_MODES,
                &[
                    ("SPLIT", Read),
                    ("AT", Read),
                    ("INTO", Write),
                    ("INTO TABLE", Write),
                ],
            ),
        ),
        K::Condense => sc.run(Skip, &[("CONDENSE", Change), ("NO-GAPS", Skip)]),
        K::Translate => sc.run(
            Skip,
            &[
                ("TRANSLATE", Change),
                ("TO UPPER CASE", Skip),
                ("TO LOWER CASE", Skip),
                ("USING", Read),
            ],
        ),
        K::Shift => sc.run(
            Skip,
            &with(
                STRING_MODES,
                &[
                    ("SHIFT", Change),
                    ("BY", Read),
                    ("PLACES", Skip),
                    ("UP TO", Read),
                    ("LEFT", Skip),
                    ("RIGHT", Skip),
                    ("CIRCULAR", Skip),
                    ("DELETING LEADING", Read),
                    ("DELETING TRAILING", Read),
                ],
            ),
        ),
        K::Replace => sc.run(
            Skip,
            &with(
                STRING_MODES,
                &[
                    ("REPLACE", Read),
                    ("FIRST OCCURRENCE OF", Read),
                    ("ALL OCCURRENCES OF", Read),
                    ("SUBSTRING", Read),
                    ("REGEX", Read),
                    ("PCRE", Read),
                    ("SECTION", Skip),
                    ("OFFSET", Read),
                    ("LENGTH", Read),
                    ("OF", Change),
                    ("IN", Change),
                    ("IN SECTION", Skip),
                    ("IN TABLE", Change),
                    ("FROM", Read),
                    ("TO", Read),
                    ("WITH", Read),
                    ("INTO", Change),
                    ("REPLACEMENT COUNT", Write),
                    ("REPLACEMENT OFFSET", Write),
                    ("REPLACEMENT LENGTH", Write),
                    ("REPLACEMENT LINE", Write),
                    ("RESULTS", Write),
                ],
            ),
        ),
        K::Find => sc.run(
            Skip,
            &with(
                STRING_MODES,
                &[
                    ("FIND", Read),
                    ("FIRST OCCURRENCE OF", Read),
                    ("ALL OCCURRENCES OF", Read),
                    ("SUBSTRING", Read),
                    ("REGEX", Read),
                    ("PCRE", Read),
                    ("IN", Read),
                    ("IN SECTION", Skip),
                    ("OFFSET", Read),
                    ("LENGTH", Read),
                    ("OF", Read),
                    ("IN TABLE", Read),
                    ("FROM", Read),
                    ("TO", Read),
                    ("MATCH COUNT", Write),
                    ("MATCH LINE", Write),
                    ("MATCH OFFSET", Write),
                    ("MATCH LENGTH", Write),
                    ("RESULTS", Write),
                    ("SUBMATCHES", Write),
                ],
            ),
        ),
        K::Overlay => sc.run(
            Skip,
            &[("OVERLAY", Change), ("WITH", Read), ("USING", Read)],
        ),
        K::Convert => {
            sc.run(
                Skip,
                &[
                    ("CONVERT", Read),
                    ("TIME STAMP", Read),
                    ("UTCLONG", Read),
                    ("DATE", Read),
                    ("TIME", Read),
                    ("TIME ZONE", Read),
                    ("DAYLIGHT SAVING TIME", Read),
                    ("TEXT", Read),
                    ("INVERTED-DATE", Read),
                    ("INTO", Stop),
                ],
            );
            sc.run(
                Skip,
                &[
                    ("INTO", Write),
                    ("INTO TIME STAMP", Write),
                    ("INTO UTCLONG", Write),
                    ("INTO DATE", Write),
                    ("INTO SORTABLE CODE", Write),
                    ("TIME", Write),
                    ("DATE", Write),
                    ("TIME ZONE", Read),
                    ("DAYLIGHT SAVING TIME", Write),
                    ("FRACTIONAL SECONDS", Write),
                    ("LEAP SECOND", Write),
                ],
            );
        }
        K::Describe => sc.run(
            Skip,
            &with(
                STRING_MODES,
                &[
                    ("DESCRIBE TABLE", Read),
                    ("DESCRIBE FIELD", Read),
                    ("DESCRIBE DISTANCE BETWEEN", Read),
                    ("AND", Read),
                    ("INTO", Write),
                    ("KIND", Write),
                    ("LINES", Write),
                    ("OCCURS", Write),
                    ("TYPE", Write),
                    ("COMPONENTS", Write),
                    ("LENGTH", Write),
                    ("DECIMALS", Write),
                    ("OUTPUT-LENGTH", Write),
                    ("HELP-ID", Write),
                    ("EDIT MASK", Write),
                ],
            ),
        ),
        K::GetTime => sc.run(Skip, &[("STAMP FIELD", Write), ("FIELD", Write)]),
        K::GetRunTime => sc.run(Skip, &[("FIELD", Write)]),
        K::GetParameter => sc.run(Skip, &[("ID", Read), ("FIELD", Write)]),
        K::SetParameter => sc.run(Skip, &[("ID", Read), ("FIELD", Read)]),
        K::Add => sc.run(Skip, &[("ADD", Read), ("TO", Change)]),
        K::Subtract => sc.run(Skip, &[("SUBTRACT", Read), ("FROM", Change)]),
        K::Multiply | K::Divide => sc.run(
            Skip,
            &[("MULTIPLY", Change), ("DIVIDE", Change), ("BY", Read)],
        ),
        K::MoveCorresponding => sc.run(
            Skip,
            &[
                ("MOVE-CORRESPONDING", Read),
                ("MOVE-CORRESPONDING EXACT", Read),
                ("TO", Change),
                ("EXPANDING NESTED TABLES", Skip),
                ("KEEPING TARGET LINES", Skip),
            ],
        ),
        K::Catch => sc.run(Skip, &[("INTO", Write)]),
        K::Cleanup => sc.run(Skip, &[("INTO", Write)]),
        K::Message => message(sc),
        K::Raise => raise(sc),
        K::RaiseEvent => sc.run(Skip, &[("EXPORTING", NamedRead)]),
        K::CallFunction => sc.run(
            Skip,
            &[
                ("CALL FUNCTION", Read),
                ("DESTINATION", Read),
                ("IN UPDATE TASK", Skip),
                ("STARTING NEW TASK", Read),
                ("CALLING", Method),
                ("PERFORMING", Skip),
                ("ON END OF TASK", Skip),
                ("IN BACKGROUND TASK", Skip),
                ("AS SEPARATE UNIT", Skip),
                ("IN BACKGROUND UNIT", Read),
                ("EXPORTING", NamedRead),
                ("IMPORTING", NamedWrite),
                ("TABLES", NamedChange),
                ("CHANGING", NamedChange),
                ("EXCEPTIONS", Skip),
                ("MESSAGE", Write),
                ("PARAMETER-TABLE", Read),
                ("EXCEPTION-TABLE", Read),
            ],
        ),
        K::Receive => sc.run(
            Skip,
            &[
                ("IMPORTING", NamedWrite),
                ("TABLES", NamedChange),
                ("CHANGING", NamedChange),
                ("EXCEPTIONS", Skip),
                ("MESSAGE", Write),
            ],
        ),
        K::Perform => sc.run(
            Skip,
            &[
                ("IN PROGRAM", Skip),
                ("IF FOUND", Skip),
                ("TABLES", Change),
                ("USING", Read),
                ("CHANGING", Change),
                ("ON COMMIT", Skip),
                ("ON ROLLBACK", Skip),
                ("LEVEL", Read),
            ],
        ),
        K::Submit => sc.run(
            Skip,
            &[
                ("WITH", Selection),
                ("WITH SELECTION-TABLE", Read),
                ("VIA SELECTION-SCREEN", Skip),
                ("AND RETURN", Skip),
                ("EXPORTING LIST TO MEMORY", Skip),
                ("USING SELECTION-SET", Read),
                ("USING SELECTION-SETS OF PROGRAM", Read),
                ("LINE-SIZE", Read),
                ("LINE-COUNT", Read),
                ("TO SAP-SPOOL", Skip),
                ("SPOOL PARAMETERS", Read),
                ("ARCHIVE PARAMETERS", Read),
                ("WITHOUT SPOOL DYNPRO", Skip),
            ],
        ),
        K::CallTransaction => sc.run(
            Skip,
            &[
                ("CALL TRANSACTION", Read),
                ("WITH AUTHORITY-CHECK", Skip),
                ("WITHOUT AUTHORITY-CHECK", Skip),
                ("USING", Read),
                ("MODE", Read),
                ("UPDATE", Read),
                ("MESSAGES INTO", Write),
                ("OPTIONS FROM", Read),
                ("AND SKIP FIRST SCREEN", Skip),
            ],
        ),
        K::CallTransformation => sc.run(
            Skip,
            &[
                ("PARAMETERS", NamedRead),
                ("OBJECTS", NamedRead),
                ("OPTIONS", Skip),
                ("SOURCE", NamedRead),
                ("SOURCE XML", Read),
                ("RESULT", NamedWrite),
                ("RESULT XML", Write),
            ],
        ),
        K::CallScreen | K::SetScreen => sc.run(
            Skip,
            &[
                ("CALL SCREEN", Read),
                ("SET SCREEN", Read),
                ("STARTING AT", Read),
                ("ENDING AT", Read),
            ],
        ),
        K::SetHandler => sc.run(
            Skip,
            &[
                ("SET HANDLER", Method),
                ("FOR", Read),
                ("FOR ALL INSTANCES", Skip),
                ("ACTIVATION", Read),
            ],
        ),
        K::Wait => sc.run(Skip, &[("UNTIL", Cond), ("UP TO", Read), ("SECONDS", Skip)]),
        K::AuthorityCheck => sc.run(
            Skip,
            &[
                ("OBJECT", Read),
                ("FOR USER", Read),
                ("ID", Read),
                ("FIELD", Read),
                ("DUMMY", Skip),
            ],
        ),
        K::OpenDataset => sc.run(
            Skip,
            &[
                ("OPEN DATASET", Read),
                ("MESSAGE", Write),
                ("FILTER", Read),
                ("AT POSITION", Read),
                ("CODE PAGE", Read),
            ],
        ),
        K::ReadDataset => sc.run(
            Skip,
            &[
                ("READ DATASET", Read),
                ("INTO", Write),
                ("MAXIMUM LENGTH", Read),
                ("ACTUAL LENGTH", Write),
                ("LENGTH", Write),
            ],
        ),
        K::Transfer => sc.run(
            Skip,
            &[
                ("TRANSFER", Read),
                ("TO", Read),
                ("LENGTH", Read),
                ("NO END OF LINE", Skip),
            ],
        ),
        K::CloseDataset | K::DeleteDataset => sc.run(Skip, &[("DATASET", Read)]),
        K::GetBit => sc.run(Skip, &[("GET BIT", Read), ("OF", Read), ("INTO", Write)]),
        K::SetBit => sc.run(Skip, &[("SET BIT", Read), ("OF", Change), ("TO", Read)]),
        K::GetBadi => sc.run(
            Skip,
            &[
                ("GET BADI", Write),
                ("TYPE", Skip),
                ("FILTERS", NamedRead),
                ("CONTEXT", Read),
            ],
        ),
        K::GetLocale => sc.run(
            Skip,
            &[("LANGUAGE", Write), ("COUNTRY", Write), ("MODIFIER", Write)],
        ),
        K::SetLocale => sc.run(
            Skip,
            &[("LANGUAGE", Read), ("COUNTRY", Read), ("MODIFIER", Read)],
        ),
        K::Pack | K::Unpack => sc.run(Skip, &[("PACK", Read), ("UNPACK", Read), ("TO", Write)]),
        K::Hide | K::OnChange => sc.run(Skip, &[("HIDE", Read), ("OF", Read), ("OR", Read)]),
        // the parameter or selection table the event is for
        K::AtSelectionScreen => sc.run(
            Skip,
            &[
                ("ON", ReadOne),
                ("ON END OF", ReadOne),
                ("ON VALUE-REQUEST FOR", ReadOne),
                ("ON HELP-REQUEST FOR", ReadOne),
                ("ON BLOCK", Stop),
                ("ON RADIOBUTTON GROUP", Stop),
                ("ON EXIT-COMMAND", Stop),
            ],
        ),
        K::SetPFStatus => sc.run(
            Skip,
            &[
                ("SET PF-STATUS", Read),
                ("OF PROGRAM", Read),
                ("EXCLUDING", Read),
                ("IMMEDIATELY", Skip),
            ],
        ),
        K::SetTitlebar => sc.run(
            Skip,
            &[("SET TITLEBAR", Read), ("OF PROGRAM", Read), ("WITH", Read)],
        ),
        K::GetCursor => sc.run(
            Skip,
            &[
                ("FIELD", Write),
                ("VALUE", Write),
                ("LINE", Write),
                ("OFFSET", Write),
                ("LENGTH", Write),
            ],
        ),
        K::LogPoint => sc.run(Skip, &[("SUBKEY", Read), ("FIELDS", Read)]),
        K::ReadReport => sc.run(
            Skip,
            &[
                ("READ REPORT", Read),
                ("INTO", Write),
                ("STATE", Read),
                ("MAXIMUM WIDTH INTO", Write),
            ],
        ),
        K::InsertReport => sc.run(
            Skip,
            &[
                ("INSERT REPORT", Read),
                ("FROM", Read),
                ("STATE", Read),
                ("PROGRAM TYPE", Read),
                ("DIRECTORY ENTRY", Read),
                ("UNICODE ENABLING", Read),
                ("KEEPING DIRECTORY ENTRY", Skip),
                ("FIXED-POINT ARITHMETIC", Read),
                ("VERSION", Read),
            ],
        ),
        K::GenerateSubroutine => sc.run(
            Skip,
            &[
                ("GENERATE SUBROUTINE POOL", Read),
                ("NAME", Write),
                ("MESSAGE", Write),
                ("INCLUDE", Write),
                ("LINE", Write),
                ("WORD", Write),
                ("OFFSET", Write),
                ("MESSAGE-ID", Write),
                ("SHORTDUMP-ID", Write),
            ],
        ),
        K::Write => write(sc),
        K::MethodDef => sc.run(Skip, &[("DEFAULT", ReadOne)]),
        K::Export => export(sc),
        K::Import => import(sc),
        K::Data | K::Constant | K::Static | K::ClassData | K::Parameter => sc.run(
            Skip,
            &[
                ("VALUE", ReadOne),
                ("VALUE IS INITIAL", Skip),
                ("DEFAULT", ReadOne),
                ("WITH DEFAULT KEY", Skip),
                ("TYPE", Skip),
                ("LIKE", Skip),
                ("LENGTH", Skip),
                ("DECIMALS", Skip),
                ("READ-ONLY", Skip),
                ("OBLIGATORY", Skip),
                ("AS CHECKBOX", Skip),
                ("RADIOBUTTON GROUP", Skip),
                ("MEMORY ID", Skip),
                ("LOWER CASE", Skip),
                ("MODIF ID", Skip),
                ("USER-COMMAND", Skip),
                ("MATCHCODE OBJECT", Skip),
                ("VISIBLE LENGTH", Skip),
                ("NO-DISPLAY", Skip),
                ("AS LISTBOX", Skip),
            ],
        ),
        k if is_sql(k) => sql(sc, k),
        _ => {}
    }
    Operands { items: scan.out }
}

fn is_sql(kind: K) -> bool {
    matches!(
        kind,
        K::Select
            | K::SelectLoop
            | K::InsertDatabase
            | K::UpdateDatabase
            | K::ModifyDatabase
            | K::DeleteDatabase
            | K::MergeDatabase
            | K::OpenCursor
            | K::FetchNextCursor
            | K::CloseCursor
            | K::With
            | K::WithLoop
    )
}

/// `a = b`, `a = b = c`, `a ?= b`, `a += b`, `MOVE a TO b`.
fn assignment(sc: &mut Scan) {
    if sc.p.eat("MOVE") {
        sc.p.eat("EXACT");
        sc.run(Read, &[("TO", Write), ("?TO", Write)]);
        return;
    }
    let mut targets = Vec::new();
    let mut access = Access::Write;
    loop {
        let save = sc.p.pos;
        let Some(t) = sc.p.target() else { break };
        let op = sc.p.peek(0).map(|t| t.text.clone()).unwrap_or_default();
        // `+=` and `-=` are two tokens
        let split = matches!(op.as_str(), "+" | "-")
            && sc.p.peek(1).is_some_and(|t| t.text == "=" && !t.ws_before);
        match op.as_str() {
            "=" | "?=" => {}
            "*=" | "/=" | "&&=" => access = Access::Change,
            _ if split => {
                access = Access::Change;
                sc.p.bump();
            }
            _ => {
                sc.p.pos = save;
                break;
            }
        }
        sc.p.bump();
        targets.push(t);
    }
    let source = sc.p.source();
    sc.push(source, Access::Read);
    for t in targets {
        sc.push(Some(t), access);
    }
}

/// Method call statements: `meth( ... ).`, `CALL METHOD lo->meth EXPORTING ...`.
fn call(sc: &mut Scan) {
    if sc.p.eat_seq("CALL METHOD") || sc.p.eat_seq("CALL BADI") {
        sc.operand(Method);
        sc.run(
            Skip,
            &[
                ("EXPORTING", NamedRead),
                ("IMPORTING", NamedWrite),
                ("CHANGING", NamedChange),
                ("RECEIVING", NamedWrite),
                ("EXCEPTIONS", Skip),
                ("PARAMETER-TABLE", Read),
                ("EXCEPTION-TABLE", Read),
            ],
        );
        return;
    }
    let e = sc.p.source();
    sc.push(e, Access::Read);
}

fn read_table(sc: &mut Scan) {
    sc.run(
        Skip,
        &with(
            TARGET_ADDITIONS,
            &[
                ("READ TABLE", Read),
                ("WHERE", ComponentCond),
                ("WITH KEY", Stop),
                ("WITH TABLE KEY", Stop),
                ("INDEX", Read),
                ("FROM", Read),
                ("USING KEY", Skip),
                ("BINARY SEARCH", Skip),
                ("INTO", Write),
                ("TRANSPORTING", Skip),
                ("TRANSPORTING NO FIELDS", Skip),
                ("COMPARING", Skip),
            ],
        ),
    );
    if sc.p.done() {
        return;
    }
    // WITH [TABLE] KEY [name COMPONENTS] comp = value ..., or `WITH KEY = value`
    sc.p.eat("WITH");
    sc.p.eat("TABLE");
    sc.p.eat("KEY");
    if sc.p.word(1).as_deref() == Some("COMPONENTS") {
        sc.p.pos += 2;
    }
    let rules = with(
        TARGET_ADDITIONS,
        &[
            ("BINARY SEARCH", Skip),
            ("INTO", Write),
            ("TRANSPORTING", Skip),
            ("TRANSPORTING NO FIELDS", Skip),
            ("COMPARING", Skip),
            ("USING KEY", Skip),
        ],
    );
    while !sc.p.done() {
        if sc.rule(&rules).is_some() {
            break;
        }
        let before = sc.p.pos;
        if sc.p.at("=") {
            sc.p.bump();
            sc.operand(Read);
        } else if sc.p.named_at() {
            sc.p.skip_component_name();
            sc.p.bump();
            sc.operand(Read);
        }
        if sc.p.pos == before {
            sc.skip();
        }
    }
    sc.run(Skip, &rules);
}

fn loop_at(sc: &mut Scan) {
    sc.p.eat("LOOP");
    sc.p.eat("AT");
    sc.p.eat("GROUP");
    // the table can be changed through the field symbol or reference
    let by_reference = (0..sc.p.len()).any(|i| {
        sc.p.token(i).is_some_and(|t| {
            t.kind == TokenKind::Identifier && (t.is("ASSIGNING") || t.is("REFERENCE"))
        })
    });
    sc.operand(if by_reference { Change } else { Read });
    let mut rules = with(
        TARGET_ADDITIONS,
        &[
            ("INTO", Write),
            ("TRANSPORTING NO FIELDS", Skip),
            ("USING KEY", Skip),
            ("FROM", Read),
            ("TO", Read),
            ("STEP", Read),
            ("WHERE", ComponentCond),
            ("GROUP BY", Stop),
        ],
    );
    sc.run(Skip, &rules);
    if sc.p.eat_seq("GROUP BY") {
        let mut items = Vec::new();
        sc.p.group_by(&mut items);
        for item in items {
            match item {
                expr::Item::Value(e) => sc.push(Some(e), Access::Read),
                expr::Item::Named {
                    value: expr::Value::Expr(e),
                    ..
                } => sc.push(Some(e), Access::Read),
                _ => {}
            }
        }
        rules.extend([
            ("ASCENDING", Skip),
            ("DESCENDING", Skip),
            ("AS TEXT", Skip),
            ("WITHOUT MEMBERS", Skip),
        ]);
        sc.run(Skip, &rules);
    }
}

fn assign(sc: &mut Scan) {
    sc.p.eat("ASSIGN");
    let rules = with(
        TARGET_ADDITIONS,
        &[
            ("TO", Write),
            ("INCREMENT", Read),
            ("RANGE", Read),
            ("TYPE", Skip),
            ("LIKE", Read),
            ("TYPE HANDLE", Read),
            ("DECIMALS", Read),
        ],
    );
    if sc.p.eat_seq("COMPONENT") {
        sc.operand(Read);
        sc.p.eat_seq("OF STRUCTURE");
        sc.operand(Read);
    } else if sc.p.eat_seq("LOCAL COPY OF") {
        sc.p.eat_seq("INITIAL");
        sc.p.eat_seq("LINE OF");
        sc.operand(Read);
    } else if sc.p.eat_seq("TABLE FIELD") {
        sc.skip();
    } else {
        sc.operand(Read);
    }
    sc.run(Skip, &rules);
}

/// A message given by type and number, `e001(zz)` or `e001`.
fn message_spec_at(p: &Parser) -> bool {
    p.peek(0).is_some_and(|t| {
        let b = t.text.as_bytes();
        t.kind == TokenKind::Identifier
            && b.len() == 4
            && b"aeiswxAEISWX".contains(&b[0])
            && b[1..].iter().all(|c| c.is_ascii_alphanumeric())
    }) && (p.peek(1).is_none_or(|t| t.ws_before)
        || p.peek(1).is_some_and(|t| t.kind == TokenKind::ParenLeft))
}

/// Skips a message given by type and number at the cursor.
fn skip_message_spec(p: &mut Parser) -> bool {
    if !message_spec_at(p) {
        return false;
    }
    p.bump();
    if p.peek(0)
        .is_some_and(|t| t.kind == TokenKind::ParenLeft && !t.ws_before)
    {
        p.bump();
        p.skip_to_close();
    }
    true
}

fn message(sc: &mut Scan) {
    sc.p.eat("MESSAGE");
    if !skip_message_spec(&mut sc.p) && !sc.p.at("ID") {
        sc.operand(Read);
    }
    sc.run(
        Skip,
        &[
            ("ID", Read),
            ("TYPE", Read),
            ("NUMBER", Read),
            ("WITH", Read),
            ("INTO", Write),
            ("RAISING", Skip),
            ("DISPLAY LIKE", Read),
        ],
    );
}

fn raise(sc: &mut Scan) {
    let rules = [
        ("EXPORTING", NamedRead),
        ("MESSAGE", Skip),
        ("ID", Read),
        ("TYPE", Read),
        ("NUMBER", Read),
        ("WITH", Read),
        ("USING MESSAGE", Skip),
    ];
    sc.p.eat("RAISE");
    sc.p.eat("RESUMABLE");
    sc.p.eat("SHORTDUMP");
    if !sc.p.eat("EXCEPTION") {
        // RAISE exception of a classic exception, RAISE EVENT is RaiseEvent
        return;
    }
    if sc.p.eat("TYPE") {
        sc.skip();
    } else {
        sc.operand(Read);
    }
    if sc.p.eat("MESSAGE") {
        skip_message_spec(&mut sc.p);
    }
    sc.run(Skip, &rules);
}

/// `WRITE [AT pos] [/] dobj [TO target] [options]`.
fn write(sc: &mut Scan) {
    sc.p.eat("WRITE");
    sc.p.eat("AT");
    // positions like `/`, `/10(20)`, `10`
    while sc
        .p
        .peek(0)
        .is_some_and(|t| t.text.starts_with('/') || t.text.bytes().all(|b| b.is_ascii_digit()))
    {
        sc.p.bump();
        if sc
            .p
            .peek(0)
            .is_some_and(|t| t.kind == TokenKind::ParenLeft && !t.ws_before)
        {
            sc.p.bump();
            sc.p.skip_to_close();
        }
    }
    sc.run(
        Read,
        &[
            ("TO", Write),
            ("CURRENCY", Read),
            ("UNIT", Read),
            ("USING EDIT MASK", Read),
            ("COLOR", Read),
            ("DECIMALS", Read),
            ("ROUND", Read),
            ("TIME ZONE", Read),
            ("EXPONENT", Read),
            ("NO-GAP", Skip),
            ("NO-ZERO", Skip),
            ("NO-SIGN", Skip),
            ("LEFT-JUSTIFIED", Skip),
            ("RIGHT-JUSTIFIED", Skip),
            ("CENTERED", Skip),
            ("AS CHECKBOX", Skip),
            ("AS ICON", Skip),
            ("AS SYMBOL", Skip),
            ("AS LINE", Skip),
            ("HOTSPOT", Skip),
            ("INTENSIFIED", Skip),
            ("INVERSE", Skip),
            ("INPUT", Skip),
            ("UNDER", Read),
            ("QUICKINFO", Read),
            ("NO EDIT MASK", Skip),
        ],
    );
}

/// The parameter list of EXPORT and IMPORT: `p = dobj`, `p FROM|TO dobj`
/// or just `dobj`.
fn cluster_parameters(sc: &mut Scan, access: Access, end: &str) {
    while !sc.p.done() && !sc.p.at(end) {
        let before = sc.p.pos;
        if sc.p.named_at() {
            sc.p.skip_component_name();
            sc.p.bump();
        } else if matches!(sc.p.word(1).as_deref(), Some("FROM" | "TO")) {
            sc.p.pos += 2;
        }
        let e = if access == Access::Read {
            sc.p.source()
        } else {
            sc.p.target()
        };
        sc.push(e, access);
        if sc.p.pos == before {
            sc.skip();
        }
    }
}

fn export(sc: &mut Scan) {
    sc.p.eat("EXPORT");
    cluster_parameters(sc, Access::Read, "TO");
    sc.run(
        Skip,
        &[
            ("TO MEMORY ID", Read),
            ("TO DATA BUFFER", Write),
            ("TO INTERNAL TABLE", Write),
            ("TO DATABASE", Skip),
            ("TO SHARED BUFFER", Skip),
            ("TO SHARED MEMORY", Skip),
            ("FROM", Read),
            ("CLIENT", Read),
            ("ID", Read),
            ("COMPRESSION ON", Skip),
            ("COMPRESSION OFF", Skip),
        ],
    );
}

fn import(sc: &mut Scan) {
    sc.p.eat("IMPORT");
    cluster_parameters(sc, Access::Write, "FROM");
    sc.run(
        Skip,
        &[
            ("FROM MEMORY ID", Read),
            ("FROM DATA BUFFER", Read),
            ("FROM INTERNAL TABLE", Read),
            ("FROM DATABASE", Skip),
            ("FROM SHARED BUFFER", Skip),
            ("FROM SHARED MEMORY", Skip),
            ("TO", Write),
            ("CLIENT", Read),
            ("ID", Read),
            ("ACCEPTING PADDING", Skip),
            ("ACCEPTING TRUNCATION", Skip),
            ("IGNORING CONVERSION ERRORS", Skip),
            ("IGNORING STRUCTURE BOUNDARIES", Skip),
            ("REPLACEMENT CHARACTER", Read),
            ("IN CHAR-TO-HEX MODE", Skip),
            ("CODE PAGE INTO", Write),
            ("ENDIAN INTO", Write),
        ],
    );
}

/// SQL clauses that end a list of targets after INTO.
const SQL_CLAUSES: &[&str] = &[
    "SELECT",
    "FROM",
    "WHERE",
    "GROUP",
    "HAVING",
    "ORDER",
    "UP",
    "FOR",
    "FIELDS",
    "UNION",
    "INTERSECT",
    "EXCEPT",
    "PACKAGE",
    "OFFSET",
    "BYPASSING",
    "CONNECTION",
    "CLIENT",
    "USING",
    "INTO",
    "APPENDING",
    "SET",
    "VALUES",
    "EXTENDED",
    "PRIVILEGED",
    "ORDER",
];

/// Database statements: host variables are read, INTO targets written.
fn sql(sc: &mut Scan, kind: K) {
    let p = &sc.p;
    let new_syntax = (0..p.len()).any(|i| p.token(i).is_some_and(|t| t.text == "@"));
    let dml = matches!(
        kind,
        K::InsertDatabase | K::UpdateDatabase | K::ModifyDatabase | K::DeleteDatabase
    );
    // SELECT * FROM dbtab without INTO: the table work area of TABLES dbtab
    // is written
    if matches!(kind, K::Select | K::SelectLoop)
        && !new_syntax
        && !(0..p.len()).any(|i| p.word(i).is_some_and(|w| w == "INTO" || w == "APPENDING"))
    {
        let star = 1 + usize::from(p.word(1).as_deref() == Some("SINGLE"));
        let from = (0..p.len()).find(|&i| p.word(i).as_deref() == Some("FROM"));
        if let Some(from) = from.filter(|_| p.token(star).is_some_and(|t| t.text == "*")) {
            let at = sc.p.pos;
            sc.p.pos = from + 1;
            sc.operand(Write);
            sc.p.pos = at;
        }
    }
    let mut into = false;
    // inside WHERE, HAVING, ON or SET of old syntax: right hand sides are
    // host variables
    let mut condition = false;
    while !sc.p.done() {
        let before = sc.p.pos;
        let t = sc.p.peek(0).unwrap();
        let word = sc.p.word(0).unwrap_or_default();
        if t.text == "@" {
            sc.p.bump();
            if sc.p.peek(0).is_some_and(|t| t.kind == TokenKind::ParenLeft) {
                sc.p.bump();
                let e = sc.p.source();
                sc.push(e, Access::Read);
                sc.p.skip_to_close();
            } else if into {
                sc.operand(Write);
            } else {
                sc.operand(Read);
            }
        } else if word == "INTO" || word == "APPENDING" {
            sc.p.bump();
            into = true;
            condition = false;
            sc.p.eat_seq("CORRESPONDING FIELDS OF");
            sc.p.eat("TABLE");
        } else if sc.p.eat_seq("FOR ALL ENTRIES IN") {
            into = false;
            sc.operand(Read);
        } else if sc.p.at_seq("OPEN CURSOR")
            || sc.p.at_seq("FETCH NEXT CURSOR")
            || sc.p.at_seq("CLOSE CURSOR")
        {
            let write = sc.p.at("OPEN");
            sc.p.pos += if sc.p.at("FETCH") { 3 } else { 2 };
            sc.p.eat_seq("WITH HOLD");
            sc.p.eat("@");
            sc.operand(if write { Write } else { Read });
        } else if into && t.kind == TokenKind::ParenLeft {
            // INTO (a, b)
            sc.p.bump();
            while !sc.p.done()
                && !sc
                    .p
                    .peek(0)
                    .is_some_and(|t| t.kind == TokenKind::ParenRight)
            {
                let before = sc.p.pos;
                sc.p.eat("@");
                sc.operand(Write);
                if sc.p.peek(0).is_some_and(|t| t.text == ",") || sc.p.pos == before {
                    sc.p.bump();
                }
            }
            sc.p.bump();
        } else if into && !SQL_CLAUSES.contains(&word.as_str()) && t.kind == TokenKind::Identifier {
            sc.operand(Write);
        } else if !new_syntax && dml && word == "FROM" && before > 1 {
            // INSERT dbtab FROM wa, DELETE dbtab FROM TABLE itab
            sc.p.bump();
            sc.p.eat("TABLE");
            sc.operand(Read);
        } else if !new_syntax && dml && word == "VALUES" {
            sc.p.bump();
            sc.operand(Read);
        } else if !new_syntax && (sc.p.at_seq("UP TO") || sc.p.at_seq("PACKAGE SIZE")) {
            sc.p.pos += 2;
            sc.operand(Read);
        } else if SQL_CLAUSES.contains(&word.as_str()) || word == "JOIN" {
            into = false;
            condition = matches!(word.as_str(), "WHERE" | "HAVING" | "SET");
            sc.p.bump();
        } else if word == "ON" {
            condition = true;
            sc.p.bump();
        } else if !new_syntax && condition && is_sql_compare(&word) {
            sc.p.bump();
            sc.p.eat("ALL");
            sc.p.eat("ANY");
            sc.p.eat("SOME");
            let host = sc.p.peek(0).is_some_and(|t| {
                t.kind == TokenKind::Identifier
                    && !t.text.contains('~')
                    && !is_sql_keyword(&t.upper())
            });
            if host {
                sc.operand(Read);
            }
        } else if !new_syntax && condition && (word == "IN" || word == "BETWEEN") {
            sc.p.bump();
            if sc
                .p
                .peek(0)
                .is_some_and(|t| t.kind == TokenKind::Identifier)
            {
                sc.operand(Read);
            }
        } else if !new_syntax && condition && word == "AND" {
            sc.p.bump();
        } else {
            sc.skip();
        }
        if sc.p.pos == before {
            sc.p.bump();
        }
    }
}

fn is_sql_compare(word: &str) -> bool {
    matches!(
        word,
        "=" | "<>" | "<" | ">" | "<=" | ">=" | "EQ" | "NE" | "LT" | "GT" | "LE" | "GE" | "LIKE"
    )
}

fn is_sql_keyword(word: &str) -> bool {
    matches!(
        word,
        "SELECT" | "NOT" | "NULL" | "IS" | "IN" | "EXISTS" | "CASE" | "ALL" | "ANY" | "SOME"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `name:R|W|C` for every data reference of the first statement.
    fn refs(code: &str) -> Vec<String> {
        let file = crate::parse(code);
        let s = &file.statements[0];
        let toks: Vec<_> = file.statement_tokens(s).collect();
        operands(&file, s)
            .refs()
            .iter()
            .map(|r| {
                let a = match r.access {
                    Access::Read => "R",
                    Access::Write => "W",
                    Access::Change => "C",
                };
                format!("{}:{a}", toks[r.pos].text)
            })
            .collect()
    }

    #[test]
    fn assignments() {
        assert_eq!(refs("a = b + c."), ["b:R", "c:R", "a:W"]);
        assert_eq!(refs("a = b = c."), ["c:R", "a:W", "b:W"]);
        assert_eq!(refs("a += b."), ["b:R", "a:C"]);
        assert_eq!(refs("MOVE a TO b."), ["a:R", "b:W"]);
        assert_eq!(refs("DATA(x) = lo->get( )."), ["lo:R", "x:W"]);
    }

    #[test]
    fn internal_tables() {
        assert_eq!(
            refs("READ TABLE t INTO w WITH KEY k = v BINARY SEARCH."),
            ["t:R", "w:W", "v:R"]
        );
        assert_eq!(
            refs("READ TABLE t ASSIGNING <f> WITH TABLE KEY primary_key COMPONENTS a = b."),
            ["t:R", "<f>:W", "b:R"]
        );
        assert_eq!(
            refs("LOOP AT t INTO DATA(w) WHERE a = b AND c IS INITIAL."),
            ["t:R", "w:W", "b:R"]
        );
        assert_eq!(
            refs("APPEND VALUE #( a = b ) TO t ASSIGNING FIELD-SYMBOL(<x>)."),
            ["b:R", "t:C", "<x>:W"]
        );
        assert_eq!(refs("APPEND LINES OF a TO b."), ["a:R", "b:C"]);
        assert_eq!(refs("INSERT w INTO TABLE t."), ["w:R", "t:C"]);
        assert_eq!(
            refs("MODIFY t FROM w TRANSPORTING a b WHERE c = d."),
            ["t:C", "w:R", "d:R"]
        );
        assert_eq!(
            refs("DELETE ADJACENT DUPLICATES FROM t COMPARING a."),
            ["t:C"]
        );
        assert_eq!(refs("SORT t BY a DESCENDING."), ["t:C"]);
    }

    #[test]
    fn calls() {
        assert_eq!(
            refs("CALL METHOD lo->m EXPORTING a = b IMPORTING c = d."),
            ["lo:R", "b:R", "d:W"]
        );
        assert_eq!(
            refs("CALL FUNCTION 'X' EXPORTING a = b TABLES t = itab EXCEPTIONS OTHERS = 1."),
            ["b:R", "itab:C"]
        );
        assert_eq!(refs("PERFORM f USING a CHANGING b."), ["a:R", "b:C"]);
        assert_eq!(refs("lo->m( a )."), ["lo:R", "a:R"]);
    }

    #[test]
    fn statements_with_keywords() {
        assert_eq!(
            refs("CONCATENATE a b INTO c SEPARATED BY d."),
            ["a:R", "b:R", "c:W", "d:R"]
        );
        assert_eq!(refs("SPLIT a AT b INTO TABLE t."), ["a:R", "b:R", "t:W"]);
        assert_eq!(
            refs("FIND ALL OCCURRENCES OF REGEX r IN s MATCH COUNT n RESULTS DATA(m)."),
            ["r:R", "s:R", "n:W", "m:W"]
        );
        assert_eq!(refs("CONDENSE s NO-GAPS."), ["s:C"]);
        assert_eq!(
            refs("MESSAGE e001(zz) WITH a b INTO DATA(m)."),
            ["a:R", "b:R", "m:W"]
        );
        assert_eq!(refs("MESSAGE text TYPE 'E'."), ["text:R"]);
        assert_eq!(
            refs("RAISE EXCEPTION TYPE zcx EXPORTING textid = t previous = p."),
            ["t:R", "p:R"]
        );
        assert_eq!(
            refs("CREATE OBJECT o TYPE zcl EXPORTING a = b."),
            ["o:W", "b:R"]
        );
        assert_eq!(
            refs("ASSIGN COMPONENT c OF STRUCTURE s TO <f>."),
            ["c:R", "s:R", "<f>:W"]
        );
        assert_eq!(refs("ASSIGN (name) TO <f>."), ["name:R", "<f>:W"]);
        assert_eq!(refs("CATCH zcx_a zcx_b INTO DATA(lx)."), ["lx:W"]);
        assert_eq!(
            refs("IF a = b AND c IS NOT INITIAL."),
            ["a:R", "b:R", "c:R"]
        );
        assert_eq!(refs("CLEAR: a."), ["a:W"]);
        assert_eq!(
            refs("CONVERT TIME STAMP ts TIME ZONE tz INTO DATE d TIME t."),
            ["ts:R", "tz:R", "d:W", "t:W"]
        );
    }

    #[test]
    fn sql_host_variables() {
        assert_eq!(
            refs("SELECT a FROM t INTO TABLE @DATA(r) WHERE b = @v AND c IN @range."),
            ["r:W", "v:R", "range:R"]
        );
        assert_eq!(
            refs("SELECT SINGLE * FROM t INTO ls WHERE b = lv_b AND c = t2~c."),
            ["ls:W", "lv_b:R"]
        );
        assert_eq!(
            refs("SELECT a FROM t INTO TABLE r FOR ALL ENTRIES IN it WHERE b = it-b."),
            ["r:W", "it:R", "it:R"]
        );
        assert_eq!(refs("INSERT ztab FROM TABLE lt."), ["lt:R"]);
        assert_eq!(refs("MODIFY ztab FROM @ls."), ["ls:R"]);
        // without INTO the work area of TABLES t100 is written
        assert_eq!(
            refs("SELECT SINGLE * FROM t100 WHERE arbgb IN s_arbgb."),
            ["t100:W", "s_arbgb:R"]
        );
        assert_eq!(refs("SELECT * FROM *t100 WHERE arbgb = '00'."), ["*t100:W"]);
        assert_eq!(refs("SELECT COUNT(*) FROM t100 WHERE arbgb = v."), ["v:R"]);
    }

    #[test]
    fn selection_screen_events() {
        assert_eq!(refs("AT SELECTION-SCREEN ON p_carr."), ["p_carr:R"]);
        assert_eq!(refs("AT SELECTION-SCREEN ON END OF s_carr."), ["s_carr:R"]);
        assert_eq!(
            refs("AT SELECTION-SCREEN ON VALUE-REQUEST FOR s_carr-low."),
            ["s_carr:R"]
        );
        assert!(refs("AT SELECTION-SCREEN ON BLOCK b1.").is_empty());
        assert!(refs("AT SELECTION-SCREEN ON RADIOBUTTON GROUP g1.").is_empty());
        assert!(refs("AT SELECTION-SCREEN OUTPUT.").is_empty());
    }
}
