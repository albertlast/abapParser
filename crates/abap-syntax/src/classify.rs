//! Statement classification by leading keywords and statement shape.
//!
//! This decides the statement kind only. Grammar-level parsing of the
//! statement contents is done per kind, on top of this.

use crate::StatementKind::{self, *};
use abap_lexer::{Token, TokenKind};

/// A keyword-level word: either one token, or several identifiers joined by
/// dashes without whitespace (`END-OF-SELECTION`, `ls_struct-field`).
struct Word {
    text: String,
    /// parenthesis/bracket nesting depth
    depth: u32,
}

struct Words(Vec<Word>);

impl Words {
    fn new(body: &[&Token]) -> Self {
        let mut words: Vec<Word> = Vec::new();
        let mut depth: u32 = 0;
        let mut i = 0;
        while i < body.len() {
            let t = body[i];
            let mut text = t.upper();
            let word_depth = depth;
            match t.kind {
                TokenKind::ParenLeft | TokenKind::BracketLeft => depth += 1,
                TokenKind::ParenRight | TokenKind::BracketRight => depth = depth.saturating_sub(1),
                _ => {}
            }
            while t.kind == TokenKind::Identifier
                && i + 2 < body.len()
                && body[i + 1].kind == TokenKind::Dash
                && !body[i + 1].ws_before
                && !body[i + 1].ws_after
                && body[i + 2].kind == TokenKind::Identifier
            {
                text.push('-');
                text.push_str(&body[i + 2].upper());
                i += 2;
            }
            words.push(Word {
                text,
                depth: word_depth,
            });
            i += 1;
        }
        Words(words)
    }

    fn len(&self) -> usize {
        self.0.len()
    }

    fn at(&self, i: usize) -> &str {
        self.0.get(i).map(|w| w.text.as_str()).unwrap_or("")
    }

    /// Prefix match, `?` matches any single word.
    fn starts(&self, pattern: &str) -> bool {
        pattern
            .split(' ')
            .enumerate()
            .all(|(i, p)| i < self.len() && (p == "?" || self.0[i].text == p))
    }

    /// Word present at nesting depth 0.
    fn has(&self, word: &str) -> bool {
        self.0.iter().any(|w| w.depth == 0 && w.text == word)
    }

    /// Consecutive words present at nesting depth 0.
    fn has_seq(&self, pattern: &str) -> bool {
        let pat: Vec<&str> = pattern.split(' ').collect();
        self.0.windows(pat.len()).any(|win| {
            win.iter()
                .zip(&pat)
                .all(|(w, p)| w.depth == 0 && w.text == *p)
        })
    }

    fn position(&self, word: &str) -> Option<usize> {
        self.0.iter().position(|w| w.depth == 0 && w.text == word)
    }
}

pub(crate) fn classify(body: &[&Token]) -> StatementKind {
    if is_move(body) {
        return Move;
    }
    if is_call_chain(body) {
        return Call;
    }
    keyword(&Words::new(body), body)
}

const ASSIGNMENT_OPS: &[&str] = &["=", "?=", "+=", "-=", "*=", "/=", "&&="];

/// `target = expression`, the target being one connected chain of tokens
/// (`ls_a-b`, `lo->attr`, `<fs>`, `DATA(x)`, `lt_tab[ 1 ]-f`).
fn is_move(body: &[&Token]) -> bool {
    let start = usize::from(body.len() > 1 && (body[0].is("CAST") || body[0].is("NEW")));
    let mut depth = 0u32;
    for (i, t) in body.iter().enumerate().skip(start) {
        let closing = matches!(t.kind, TokenKind::ParenRight | TokenKind::BracketRight);
        if depth == 0 && i > start && t.ws_before && !closing {
            // "+=" and "-=" are lexed as two tokens
            let split_op = matches!(t.kind, TokenKind::Plus | TokenKind::Dash)
                && body
                    .get(i + 1)
                    .is_some_and(|n| n.text == "=" && !n.ws_before);
            return (ASSIGNMENT_OPS.contains(&t.text.as_str()) || split_op) && i + 1 < body.len();
        }
        match t.kind {
            TokenKind::ParenLeft | TokenKind::BracketLeft => depth += 1,
            TokenKind::ParenRight | TokenKind::BracketRight => depth = depth.saturating_sub(1),
            _ => {}
        }
    }
    false
}

/// Functional method call statement: `meth( )`, `lo->a( )->b( x )`,
/// `zcl=>m( )`, `NEW zcl( )->run( )`.
fn is_call_chain(body: &[&Token]) -> bool {
    let start = if body[0].is("NEW") && body.len() > 1 {
        1
    } else {
        0
    };
    if body[start].kind != TokenKind::Identifier {
        return false;
    }
    let mut depth = 0u32;
    let mut has_paren = false;
    for (i, t) in body.iter().enumerate().skip(start) {
        let closing = matches!(t.kind, TokenKind::ParenRight | TokenKind::BracketRight);
        if depth == 0 && i > start && t.ws_before && !closing {
            return false;
        }
        match t.kind {
            TokenKind::ParenLeft => {
                if depth == 0 {
                    has_paren = true;
                }
                depth += 1;
            }
            TokenKind::BracketLeft => depth += 1,
            TokenKind::ParenRight | TokenKind::BracketRight => depth = depth.saturating_sub(1),
            _ => {}
        }
    }
    has_paren && depth == 0 && body.last().unwrap().kind == TokenKind::ParenRight
}

/// Heuristic for statements that are syntactically identical for internal
/// tables and database tables, e.g. `MODIFY x FROM y`: decided by the name.
fn looks_internal(name: &str) -> bool {
    if name.starts_with('<') {
        return true;
    }
    let lower = name.to_ascii_lowercase();
    let prefix = lower.split('_').next().unwrap_or("");
    lower.contains('_')
        && (2..=3).contains(&prefix.len())
        && prefix.ends_with('t')
        && prefix.chars().all(|c| c.is_ascii_alphabetic())
}

fn is_dynamic_or_db(w: &Words, body: &[&Token]) -> bool {
    w.at(1) == "("
        || w.has("CONNECTION")
        || w.has("CLIENT")
        || body.iter().any(|t| t.kind == TokenKind::At)
        || w.has_seq("FROM (")
}

fn keyword(w: &Words, body: &[&Token]) -> StatementKind {
    let first = w.at(0);
    match first {
        "REPORT" => Report,
        "PROGRAM" => Program,
        "FUNCTION-POOL" => FunctionPool,
        "CLASS-POOL" => ClassPool,
        "INTERFACE-POOL" => InterfacePool,
        "TYPE-POOL" => TypePool,
        "TYPE-POOLS" => TypePools,
        "INCLUDE" if w.starts("INCLUDE TYPE") || w.starts("INCLUDE STRUCTURE") => IncludeType,
        "INCLUDE" => Include,

        "DATA" if w.starts("DATA BEGIN OF") => DataBegin,
        "DATA" if w.starts("DATA END OF") => DataEnd,
        "DATA" => Data,
        "CLASS-DATA" if w.starts("CLASS-DATA BEGIN OF") => ClassDataBegin,
        "CLASS-DATA" if w.starts("CLASS-DATA END OF") => ClassDataEnd,
        "CLASS-DATA" => ClassData,
        "STATICS" if w.starts("STATICS BEGIN OF") => StaticBegin,
        "STATICS" if w.starts("STATICS END OF") => StaticEnd,
        "STATICS" => Static,
        "CONSTANTS" if w.starts("CONSTANTS BEGIN OF") => ConstantBegin,
        "CONSTANTS" if w.starts("CONSTANTS END OF") => ConstantEnd,
        "CONSTANTS" => Constant,
        "TYPES" => types(w),
        "FIELD-SYMBOLS" | "FIELD-SYMBOL" => FieldSymbol,
        "RANGES" => Ranges,
        "TABLES" => Tables,
        "NODES" => Nodes,
        "INFOTYPES" => Infotypes,
        "PARAMETERS" | "PARAMETER" => Parameter,
        "SELECT-OPTIONS" => SelectOption,
        "SELECTION-SCREEN" => SelectionScreen,
        "FIELD-GROUPS" => FieldGroup,
        "CONTROLS" => Controls,
        "LOCAL" => Local,
        "CONTEXTS" => Contexts,
        "DEFINE" => Define,
        "END-OF-DEFINITION" => EndOfDefinition,

        "CLASS" if w.starts("CLASS ? DEFINITION DEFERRED") => ClassDeferred,
        "CLASS" if w.starts("CLASS ? DEFINITION LOAD") => ClassDefinitionLoad,
        "CLASS" if w.starts("CLASS ? DEFINITION LOCAL FRIENDS") => ClassLocalFriends,
        "CLASS" if w.starts("CLASS ? DEFINITION") => ClassDefinition,
        "CLASS" if w.starts("CLASS ? IMPLEMENTATION") => ClassImplementation,
        "ENDCLASS" => EndClass,
        "INTERFACE" if w.starts("INTERFACE ? DEFERRED") => InterfaceDeferred,
        "INTERFACE" if w.starts("INTERFACE ? LOAD") => InterfaceLoad,
        "INTERFACE" => Interface,
        "ENDINTERFACE" => EndInterface,
        "INTERFACES" => InterfaceDef,
        "PUBLIC" if w.starts("PUBLIC SECTION") => Public,
        "PROTECTED" if w.starts("PROTECTED SECTION") => Protected,
        "PRIVATE" if w.starts("PRIVATE SECTION") => Private,
        "PACKAGE" if w.starts("PACKAGE SECTION") => Package,
        "METHODS" | "CLASS-METHODS" => MethodDef,
        "METHOD" => MethodImplementation,
        "ENDMETHOD" => EndMethod,
        "EVENTS" | "CLASS-EVENTS" => Events,
        "ALIASES" => Aliases,
        "CREATE" if w.starts("CREATE DATA") => CreateData,
        "CREATE" if w.starts("CREATE OBJECT") => {
            let ole = w.len() > 3
                && ![
                    "TYPE",
                    "EXPORTING",
                    "EXCEPTIONS",
                    "AREA",
                    "FOR",
                    "PUBLIC",
                    "->",
                    "=>",
                ]
                .iter()
                .any(|k| w.has(k));
            if ole {
                CreateOLE
            } else {
                CreateObject
            }
        }
        "RAISE" if w.starts("RAISE EVENT") => RaiseEvent,
        "RAISE" if w.starts("RAISE ENTITY EVENT") => RaiseEntityEvent,
        "RAISE" => Raise,
        "SET" => set(w),
        "GET" => get(w),

        "FORM" if w.starts("FORM ? DEFINITION") => FormDefinition,
        "FORM" => Form,
        "ENDFORM" => EndForm,
        "FUNCTION" => FunctionModule,
        "ENDFUNCTION" => EndFunction,
        "MODULE" => Module,
        "ENDMODULE" => EndModule,
        "PERFORM" => Perform,
        "CALL" => call(w),
        "SUBMIT" => Submit,
        "LEAVE" => Leave,
        "RETURN" => Return,
        "EXIT" => Exit,
        "CONTINUE" => Continue,
        "CHECK" if w.starts("CHECK SELECT-OPTIONS") => CheckSelectOptions,
        "CHECK" => Check,
        "STOP" => Stop,
        "REJECT" => Reject,
        "RESUME" => Resume,

        "START-OF-SELECTION" => StartOfSelection,
        "END-OF-SELECTION" => EndOfSelection,
        "INITIALIZATION" => Initialization,
        "TOP-OF-PAGE" => TopOfPage,
        "END-OF-PAGE" => EndOfPage,
        "LOAD-OF-PROGRAM" => LoadOfProgram,
        "AT" => at(w),

        "IF" => If,
        "ELSEIF" => ElseIf,
        "ELSE" => Else,
        "ENDIF" => EndIf,
        "CASE" if w.starts("CASE TYPE OF") => CaseType,
        "CASE" => Case,
        "WHEN" if w.starts("WHEN TYPE") => WhenType,
        "WHEN" if w.starts("WHEN OTHERS") && w.len() == 2 => WhenOthers,
        "WHEN" => When,
        "ENDCASE" => EndCase,
        "DO" => Do,
        "ENDDO" => EndDo,
        "WHILE" => While,
        "ENDWHILE" => EndWhile,
        "LOOP" if w.len() == 1 => LoopExtract,
        "LOOP" if w.starts("LOOP AT SCREEN") => LoopAtScreen,
        "LOOP" if w.has("CURSOR") || w.has_seq("WITH CONTROL") => DynproLoop,
        "LOOP" => Loop,
        "ENDLOOP" => EndLoop,
        "TRY" => Try,
        "CATCH" if w.starts("CATCH SYSTEM-EXCEPTIONS") => CatchSystemExceptions,
        "CATCH" => Catch,
        "CLEANUP" => Cleanup,
        "RETRY" => Retry,
        "ENDTRY" => EndTry,
        "ENDCATCH" => EndCatch,
        "ENDAT" => EndAt,
        "ON" if w.starts("ON CHANGE OF") => OnChange,
        "ENDON" => EndOn,
        "PROVIDE" => Provide,
        "ENDPROVIDE" => EndProvide,
        "TEST-SEAM" => TestSeam,
        "END-TEST-SEAM" => EndTestSeam,
        "TEST-INJECTION" => TestInjection,
        "END-TEST-INJECTION" => EndTestInjection,

        "MOVE" => Move,
        "COMPUTE" => Compute,
        "MOVE-CORRESPONDING" => MoveCorresponding,
        "CLEAR" => Clear,
        "FREE" if w.starts("FREE MEMORY") => FreeMemory,
        "FREE" if w.starts("FREE OBJECT") => FreeObject,
        "FREE" => Free,
        "REFRESH" if w.starts("REFRESH CONTROL") => RefreshControl,
        "REFRESH" => Refresh,
        "APPEND" => Append,
        "INSERT" => insert(w, body),
        "MODIFY" => modify(w, body),
        "DELETE" => delete(w, body),
        "READ" => read(w),
        "SORT" if w.len() == 1 || w.starts("SORT BY") || w.starts("SORT AS") => SortDataset,
        "SORT" => Sort,
        "COLLECT" => Collect,
        "ASSIGN" if w.starts("ASSIGN LOCAL COPY") => AssignLocalCopy,
        "ASSIGN" => Assign,
        "UNASSIGN" => Unassign,
        "CONCATENATE" => Concatenate,
        "SPLIT" => Split,
        "CONDENSE" => Condense,
        "REPLACE" => Replace,
        "FIND" => Find,
        "SHIFT" => Shift,
        "TRANSLATE" => Translate,
        "OVERLAY" => Overlay,
        "SEARCH" => Search,
        "CONVERT" if w.starts("CONVERT TEXT") => ConvertText,
        "CONVERT" => Convert,
        "PACK" => Pack,
        "UNPACK" => Unpack,
        "DESCRIBE" => Describe,
        "ADD" => Add,
        "ADD-CORRESPONDING" => AddCorresponding,
        "SUBTRACT" => Subtract,
        "SUBTRACT-CORRESPONDING" => SubtractCorresponding,
        "MULTIPLY" => Multiply,
        "MULTIPLY-CORRESPONDING" => MultiplyCorresponding,
        "DIVIDE" => Divide,
        "DIVIDE-CORRESPONDING" => DivideCorresponding,
        "MESSAGE" => Message,
        "ASSERT" => Assert,
        "BREAK-POINT" if w.starts("BREAK-POINT ID") => BreakId,
        "BREAK-POINT" | "BREAK" => Break,
        "LOG-POINT" => LogPoint,

        "SELECT" => select(w),
        "ENDSELECT" => EndSelect,
        "WITH" => {
            if w.has_seq("PACKAGE SIZE") {
                WithLoop
            } else if into_table(w) {
                With
            } else {
                WithLoop
            }
        }
        "ENDWITH" => EndWith,
        "UPDATE" => UpdateDatabase,
        "MERGE" => MergeDatabase,
        "OPEN" if w.starts("OPEN CURSOR") => OpenCursor,
        "OPEN" if w.starts("OPEN DATASET") => OpenDataset,
        "FETCH" => FetchNextCursor,
        "CLOSE" if w.starts("CLOSE CURSOR") => CloseCursor,
        "CLOSE" if w.starts("CLOSE DATASET") => CloseDataset,
        "COMMIT" if w.starts("COMMIT ENTITIES") => CommitEntities,
        "COMMIT" => Commit,
        "ROLLBACK" if w.starts("ROLLBACK ENTITIES") => RollbackEntities,
        "ROLLBACK" => Rollback,
        "EXEC" if w.starts("EXEC SQL") => ExecSQL,
        "ENDEXEC" => EndExec,

        "EXPORT" if w.starts("EXPORT DYNPRO") => ExportDynpro,
        "EXPORT" => Export,
        "IMPORT" if w.starts("IMPORT DYNPRO") => ImportDynpro,
        "IMPORT" if w.starts("IMPORT NAMETAB") => ImportNametab,
        "IMPORT" => Import,
        "TRANSFER" => Transfer,
        "TRUNCATE" => TruncateDataset,

        "WRITE" => Write,
        "ULINE" => Uline,
        "SKIP" => Skip,
        "NEW-LINE" => NewLine,
        "NEW-PAGE" => NewPage,
        "FORMAT" => Format,
        "POSITION" => Position,
        "RESERVE" => Reserve,
        "BACK" => Back,
        "HIDE" => Hide,
        "SUMMARY" => Summary,
        "DETAIL" => Detail,
        "INPUT" => Input,
        "PRINT-CONTROL" => PrintControl,
        "WINDOW" => Window,
        "SCROLL" => ScrollList,
        "SUM" => Sum,
        "EXTRACT" => Extract,
        "FIELD" => Field,
        "FIELDS" => Fields,
        "PUT" => Put,
        "CHAIN" => Chain,
        "ENDCHAIN" => EndChain,
        "PROCESS" if w.starts("PROCESS BEFORE OUTPUT") => ProcessBeforeOutput,
        "PROCESS" if w.starts("PROCESS AFTER INPUT") => ProcessAfterInput,
        "PROCESS" if w.starts("PROCESS ON VALUE-REQUEST") => ProcessOnValueRequest,
        "PROCESS" if w.starts("PROCESS ON HELP-REQUEST") => ProcessOnHelpRequest,

        "AUTHORITY-CHECK" => AuthorityCheck,
        "WAIT" => Wait,
        "RECEIVE" => Receive,
        "DEMAND" => Demand,
        "SUPPLY" => Supply,
        "COMMUNICATION" => Communication,
        "SYSTEM-CALL" => SystemCall,
        "SUPPRESS" => SuppressDialog,
        "VERIFICATION-MESSAGE" => VerificationMessage,
        "EDITOR-CALL" => EditorCall,
        "SYNTAX-CHECK" => SyntaxCheck,
        "SCAN" => Scan,
        "GENERATE" if w.starts("GENERATE SUBROUTINE POOL") => GenerateSubroutine,
        "GENERATE" if w.starts("GENERATE REPORT") => GenerateReport,
        "GENERATE" if w.starts("GENERATE DYNPRO") => GenerateDynpro,
        "LOAD" if w.starts("LOAD REPORT") => LoadReport,
        "ENHANCEMENT" => Enhancement,
        "ENDENHANCEMENT" => EndEnhancement,
        "ENHANCEMENT-POINT" => EnhancementPoint,
        "ENHANCEMENT-SECTION" => EnhancementSection,
        "END-ENHANCEMENT-SECTION" => EndEnhancementSection,
        _ => Unknown,
    }
}

fn types(w: &Words) -> StatementKind {
    if w.starts("TYPES BEGIN OF ENUM") {
        TypeEnumBegin
    } else if w.starts("TYPES END OF ENUM") {
        TypeEnumEnd
    } else if w.starts("TYPES BEGIN OF MESH") {
        TypeMeshBegin
    } else if w.starts("TYPES END OF MESH") {
        TypeMeshEnd
    } else if w.starts("TYPES BEGIN OF") {
        TypeBegin
    } else if w.starts("TYPES END OF") {
        TypeEnd
    } else if w.at(2) == "VALUE" {
        TypeEnum
    } else if w.has("ASSOCIATION") {
        TypeMesh
    } else {
        Type
    }
}

fn set(w: &Words) -> StatementKind {
    match w.at(1) {
        "HANDLER" => SetHandler,
        "PF-STATUS" => SetPFStatus,
        "TITLEBAR" => SetTitlebar,
        "CURSOR" => SetCursor,
        "PARAMETER" => SetParameter,
        "SCREEN" => SetScreen,
        "LOCALE" => SetLocale,
        "COUNTRY" => SetCountry,
        "LANGUAGE" => SetLanguage,
        "BIT" => SetBit,
        "BLANK" => SetBlank,
        "MARGIN" => SetMargin,
        "LEFT" => SetLeft,
        "EXTENDED" => SetExtendedCheck,
        "RUN" => SetRunTime,
        "UPDATE" => SetUpdateTask,
        "USER-COMMAND" => SetUserCommand,
        "PROPERTY" => SetProperty,
        "DATASET" => SetDataset,
        "LOCKS" => SetLocks,
        _ => Unknown,
    }
}

fn get(w: &Words) -> StatementKind {
    match w.at(1) {
        "REFERENCE" => GetReference,
        "BADI" => GetBadi,
        "TIME" => GetTime,
        "RUN" => GetRunTime,
        "PARAMETER" => GetParameter,
        "CURSOR" => GetCursor,
        "PF-STATUS" => GetPFStatus,
        "PROPERTY" => GetProperty,
        "BIT" => GetBit,
        "LOCALE" => GetLocale,
        "DATASET" => GetDataset,
        "PERMISSIONS" => GetPermissions,
        _ => Get,
    }
}

fn call(w: &Words) -> StatementKind {
    match w.at(1) {
        "FUNCTION" | "CUSTOMER-FUNCTION" => CallFunction,
        "METHOD" if w.at(2) == "OF" => CallOLE,
        "METHOD" => Call,
        "TRANSACTION" => CallTransaction,
        "TRANSFORMATION" => CallTransformation,
        "SCREEN" => CallScreen,
        "SELECTION-SCREEN" => CallSelectionScreen,
        "DIALOG" => CallDialog,
        "BADI" => CallBadi,
        "DATABASE" => CallDatabase,
        "SUBSCREEN" => CallSubscreen,
        _ => CallKernel,
    }
}

fn at(w: &Words) -> StatementKind {
    match w.at(1) {
        "SELECTION-SCREEN" => AtSelectionScreen,
        "LINE-SELECTION" => AtLineSelection,
        "USER-COMMAND" => AtUserCommand,
        "FIRST" => AtFirst,
        "LAST" => AtLast,
        s if s.starts_with("PF") && s[2..].chars().all(|c| c.is_ascii_digit()) && s.len() > 2 => {
            AtPF
        }
        _ => At,
    }
}

fn read(w: &Words) -> StatementKind {
    match w.at(1) {
        "TABLE" => ReadTable,
        "LINE" | "CURRENT" => ReadLine,
        "REPORT" => ReadReport,
        "DATASET" => ReadDataset,
        "TEXTPOOL" => ReadTextpool,
        "ENTITIES" | "ENTITY" => ReadEntities,
        _ => Unknown,
    }
}

fn into_table(w: &Words) -> bool {
    w.has_seq("INTO TABLE")
        || w.has_seq("APPENDING TABLE")
        || w.has_seq("INTO CORRESPONDING FIELDS OF TABLE")
        || w.has_seq("APPENDING CORRESPONDING FIELDS OF TABLE")
}

fn has_aggregation(w: &Words) -> bool {
    let items = &w.0;
    (0..items.len().saturating_sub(1)).any(|i| {
        let is_aggregate = items[i].depth == 0
            && matches!(
                items[i].text.as_str(),
                "COUNT" | "MAX" | "MIN" | "SUM" | "AVG"
            )
            && items[i + 1].text == "(";
        // a window function, AGG( ... ) OVER( ... ), returns one row per input row
        let over = items[i + 2..]
            .iter()
            .find(|x| x.depth == 0)
            .is_some_and(|x| x.text == "OVER");
        is_aggregate && !over
    })
}

/// SELECT returning a single result vs SELECT ... ENDSELECT loop, same
/// decision table as abaplint's select reclassification.
fn select(w: &Words) -> StatementKind {
    if w.starts("SELECT SINGLE") {
        return Select;
    }
    if w.has_seq("PACKAGE SIZE") {
        return SelectLoop;
    }
    if into_table(w) {
        return Select;
    }
    let aggregation = has_aggregation(w);
    let grouped = w.has_seq("GROUP BY");
    let has_into = w.has("INTO") || w.has("APPENDING");
    if !has_into && aggregation && !grouped && !w.has("HAVING") && !w.has_seq("ORDER BY") {
        return Select;
    }
    if w.has("UNION") || w.has("INTERSECT") || w.has("EXCEPT") {
        return SelectLoop;
    }
    if !grouped && aggregation {
        return Select;
    }
    SelectLoop
}

fn insert(w: &Words, body: &[&Token]) -> StatementKind {
    if w.starts("INSERT REPORT") {
        return InsertReport;
    }
    if w.starts("INSERT TEXTPOOL") {
        return InsertTextpool;
    }
    if w.starts("INSERT LINES OF")
        || w.starts("INSERT INITIAL LINE")
        || w.starts("INSERT TABLE")
        || w.has_seq("INTO TABLE")
        || w.has("INDEX")
        || w.has("ASSIGNING")
        || w.has_seq("REFERENCE INTO")
    {
        return InsertInternal;
    }
    if w.starts("INSERT INTO") || w.has("VALUES") || w.has("FROM") || is_dynamic_or_db(w, body) {
        return InsertDatabase;
    }
    if w.has("INTO") {
        // INSERT f1 f2 INTO fg: several fields into a field group
        if fields_before_into(body) > 1 {
            return InsertFieldGroup;
        }
        return InsertInternal;
    }
    InsertDatabase
}

/// Operands between the keyword and INTO at depth 0: tokens after a blank
/// that are names or literals.
fn fields_before_into(body: &[&Token]) -> usize {
    let mut depth = 0u32;
    let mut count = 0;
    for t in body.iter().skip(1) {
        match t.kind {
            TokenKind::ParenLeft | TokenKind::BracketLeft => depth += 1,
            TokenKind::ParenRight | TokenKind::BracketRight => depth = depth.saturating_sub(1),
            _ => {}
        }
        if depth == 0 && t.is("INTO") {
            break;
        }
        if depth == 0 && t.ws_before && matches!(t.kind, TokenKind::Identifier | TokenKind::String)
        {
            count += 1;
        }
    }
    count
}

fn modify(w: &Words, body: &[&Token]) -> StatementKind {
    if w.starts("MODIFY ENTITIES") || w.starts("MODIFY ENTITY") || w.starts("MODIFY AUGMENTING") {
        return ModifyEntities;
    }
    if w.starts("MODIFY SCREEN") {
        return ModifyScreen;
    }
    if w.len() == 2 {
        // MODIFY itab. on a table with header line, e.g. TOTAL in table maintenance
        return ModifyInternal;
    }
    if w.has("CONNECTION") {
        return ModifyDatabase;
    }
    if w.starts("MODIFY LINE") || w.starts("MODIFY CURRENT LINE") {
        return ModifyLine;
    }
    if w.starts("MODIFY TABLE")
        || w.has("INDEX")
        || w.has("TRANSPORTING")
        || w.has("ASSIGNING")
        || w.has_seq("USING KEY")
    {
        return ModifyInternal;
    }
    if is_dynamic_or_db(w, body) {
        return ModifyDatabase;
    }
    if looks_internal(w.at(1)) {
        ModifyInternal
    } else {
        ModifyDatabase
    }
}

fn delete(w: &Words, body: &[&Token]) -> StatementKind {
    if w.has("CONNECTION") {
        return DeleteDatabase;
    }
    if w.starts("DELETE ADJACENT DUPLICATES") || w.starts("DELETE TABLE") {
        return DeleteInternal;
    }
    match w.at(1) {
        "DATASET" => return DeleteDataset,
        "REPORT" => return DeleteReport,
        "TEXTPOOL" => return DeleteTextpool,
        "DYNPRO" => return DeleteDynpro,
        _ => {}
    }
    if w.starts("DELETE FROM MEMORY") {
        return DeleteMemory;
    }
    if w.starts("DELETE FROM SHARED") {
        return DeleteMemory;
    }
    if w.starts("DELETE FROM DATABASE") {
        return DeleteCluster;
    }
    if w.starts("DELETE FROM") {
        return DeleteDatabase;
    }
    if w.has("INDEX") || w.has("WHERE") || w.has("TO") || w.has_seq("USING KEY") {
        return DeleteInternal;
    }
    if let Some(p) = w.position("FROM") {
        let next = w.at(p + 1);
        if next.chars().next().is_some_and(|c| c.is_ascii_digit()) {
            return DeleteInternal;
        }
    }
    if is_dynamic_or_db(w, body) {
        return DeleteDatabase;
    }
    if looks_internal(w.at(1)) || !w.has("FROM") {
        DeleteInternal
    } else {
        DeleteDatabase
    }
}
