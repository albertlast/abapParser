macro_rules! statement_kinds {
    ($($name:ident),* $(,)?) => {
        /// Kind of an ABAP statement. Variant names equal the statement class
        /// names of abaplint, so results can be compared against it.
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        pub enum StatementKind {
            $($name),*
        }

        impl StatementKind {
            pub const ALL: &'static [StatementKind] = &[$(StatementKind::$name),*];

            pub fn as_str(self) -> &'static str {
                match self {
                    $(StatementKind::$name => stringify!($name)),*
                }
            }
        }

        impl std::str::FromStr for StatementKind {
            type Err = ();
            fn from_str(s: &str) -> Result<Self, ()> {
                match s {
                    $(stringify!($name) => Ok(StatementKind::$name),)*
                    _ => Err(()),
                }
            }
        }
    };
}

statement_kinds! {
    // not real statements
    Unknown, Empty, Comment, NativeSQL, MacroCall, MacroContent,

    // program frame
    Report, Program, FunctionPool, ClassPool, InterfacePool, TypePool, TypePools,
    Include, IncludeType,

    // declarations
    Data, DataBegin, DataEnd, ClassData, ClassDataBegin, ClassDataEnd,
    Static, StaticBegin, StaticEnd,
    Type, TypeBegin, TypeEnd, TypeEnumBegin, TypeEnumEnd, TypeEnum, TypeMesh, TypeMeshBegin, TypeMeshEnd,
    Constant, ConstantBegin, ConstantEnd, FieldSymbol, Ranges, Tables, Nodes, Infotypes,
    Parameter, SelectOption, SelectionScreen, FieldGroup, Controls, Local, Contexts,
    Define, EndOfDefinition,

    // object orientation
    ClassDefinition, ClassImplementation, ClassDeferred, ClassDefinitionLoad, ClassLocalFriends, EndClass,
    Interface, InterfaceDeferred, InterfaceLoad, EndInterface, InterfaceDef,
    Public, Protected, Private, Package,
    MethodDef, MethodImplementation, EndMethod, Events, Aliases,
    CreateObject, CreateOLE, CreateData, RaiseEvent, SetHandler,

    // modularization
    Form, FormDefinition, EndForm, FunctionModule, EndFunction, Module, EndModule,
    Perform, CallFunction, Call, CallKernel, CallTransaction, CallTransformation, CallScreen,
    CallSelectionScreen, CallDialog, CallBadi, CallDatabase, CallSubscreen, CallOLE,
    Submit, Leave, Return, Exit, Continue, Check, CheckSelectOptions, Stop, Reject, Resume,

    // events
    StartOfSelection, EndOfSelection, Initialization, TopOfPage, EndOfPage, LoadOfProgram,
    AtSelectionScreen, AtLineSelection, AtUserCommand, AtPF, Get,

    // control flow
    If, ElseIf, Else, EndIf, Case, CaseType, When, WhenType, WhenOthers, EndCase,
    Do, EndDo, While, EndWhile, Loop, LoopAtScreen, LoopExtract, DynproLoop, EndLoop,
    Try, Catch, Cleanup, Retry, EndTry, CatchSystemExceptions, EndCatch,
    At, AtFirst, AtLast, EndAt, OnChange, EndOn, Provide, EndProvide,
    TestSeam, EndTestSeam, TestInjection, EndTestInjection,

    // data processing
    Move, Compute, MoveCorresponding, Clear, Free, FreeMemory, FreeObject, Refresh, RefreshControl,
    Append, InsertInternal, ModifyInternal, DeleteInternal, ReadTable, Sort, SortDataset, Collect,
    Assign, AssignLocalCopy, Unassign, GetReference,
    Concatenate, Split, Condense, Replace, Find, Shift, Translate, Overlay, Search,
    Convert, ConvertText, Pack, Unpack, Describe,
    Add, AddCorresponding, Subtract, SubtractCorresponding, Multiply, MultiplyCorresponding,
    Divide, DivideCorresponding,
    Raise, Message, Assert, Break, BreakId, LogPoint,

    // database
    Select, SelectLoop, EndSelect, With, WithLoop, EndWith,
    InsertDatabase, UpdateDatabase, ModifyDatabase, DeleteDatabase, MergeDatabase,
    OpenCursor, FetchNextCursor, CloseCursor, Commit, Rollback, ExecSQL, EndExec,
    ReadEntities, ModifyEntities, CommitEntities, RollbackEntities, GetPermissions, SetLocks,
    RaiseEntityEvent,

    // data cluster, files, memory
    Export, Import, ExportDynpro, ImportDynpro, ImportNametab, DeleteMemory, DeleteCluster,
    OpenDataset, CloseDataset, ReadDataset, Transfer, DeleteDataset, TruncateDataset,
    SetDataset, GetDataset,

    // lists and dynpro
    Write, Uline, Skip, NewLine, NewPage, Format, Position, Reserve, Back, Hide, Summary, Detail,
    Input, PrintControl, Window, ScrollList, ReadLine, ModifyLine, Sum, Extract, Field, Chain,
    EndChain, Fields, Put, ProcessBeforeOutput, ProcessAfterInput, ProcessOnValueRequest, ProcessOnHelpRequest,
    ModifyScreen, SetPFStatus, SetTitlebar, SetCursor, GetCursor, SetScreen, SetUserCommand,
    GetPFStatus, SetBlank, SetMargin, SetLeft,

    // system and misc
    AuthorityCheck, GetBadi, GetTime, GetRunTime, SetRunTime, GetParameter, SetParameter,
    GetProperty, SetProperty, GetBit, SetBit, GetLocale, SetLocale, SetCountry, SetLanguage,
    SetExtendedCheck, SetUpdateTask, Wait, Receive, Demand, Supply, Communication,
    SystemCall, SuppressDialog, VerificationMessage, EditorCall, SyntaxCheck, Scan,
    GenerateSubroutine, GenerateReport, GenerateDynpro, InsertReport, ReadReport, DeleteReport,
    LoadReport, InsertTextpool, ReadTextpool, DeleteTextpool, DeleteDynpro, InsertFieldGroup,
    Enhancement, EndEnhancement, EnhancementPoint, EnhancementSection, EndEnhancementSection,
}
