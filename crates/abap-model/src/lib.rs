//! Output model of the ABAP parser. Everything here serializes to JSON with
//! snake_case keys; Node.js and Python bindings return exactly this shape.
//! The JSON Schema is generated with [`symbols_schema`].

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Bumped on incompatible changes of the JSON shape.
pub const SCHEMA_VERSION: u32 = 2;

/// 1-based row and column.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Position {
    pub row: u32,
    pub col: u32,
}

/// Source range from the first character of the first token to the end of the last token.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Span {
    pub start: Position,
    pub end: Position,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Visibility {
    Public,
    Protected,
    Private,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TypingKind {
    /// `TYPE t`
    Type,
    /// `LIKE dobj`
    Like,
    /// `TYPE REF TO t`
    TypeRefTo,
    /// `LIKE REF TO dobj`
    LikeRefTo,
    /// internal table
    Table,
    /// `RANGE OF`
    Range,
    /// `LINE OF`
    LineOf,
    /// `STRUCTURE s`
    Structure,
    /// `FOR dobj`, SELECT-OPTIONS
    For,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct TableTyping {
    /// `standard`, `sorted`, `hashed`, `any` or `index`; absent for `TYPE TABLE OF`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    /// primary key uniqueness, when stated
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unique: Option<bool>,
    /// key definition as written, e.g. `WITH UNIQUE KEY name`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    /// the line type is `REF TO ...`
    pub line_is_ref: bool,
    /// obsolete `WITH HEADER LINE`
    pub header_line: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Typing {
    pub kind: TypingKind,
    /// Referenced type or data object; for tables the line type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub table: Option<TableTyping>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub length: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decimals: Option<String>,
    /// The typing clause as written.
    pub text: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DeclarationKind {
    /// DATA
    Data,
    /// CLASS-DATA
    ClassData,
    /// STATICS
    Static,
    /// CONSTANTS
    Constant,
    /// TYPES
    Type,
    /// FIELD-SYMBOLS
    FieldSymbol,
    /// PARAMETERS on a selection screen
    Parameter,
    /// SELECT-OPTIONS
    SelectOption,
    /// RANGES
    Range,
    /// TABLES work area
    Tables,
    /// NODES
    Node,
    /// INFOTYPES: an internal table with header line, named `Pnnnn` after
    /// the infotype unless NAME gives another name, with line type `Pnnnn`.
    /// The logical database fills it, unless the clause has `MODE N`: PNP at
    /// `GET PERNR` from `PAnnnn`, PCH at `GET OBJEC` from `HRPnnnn`. The
    /// clause as written is in `typing.text`
    Infotype,
    /// `INCLUDE TYPE|STRUCTURE` inside a structure
    Include,
    /// value of an enumerated type
    EnumValue,
}

/// A data object or type declaration. Structures declared with
/// `BEGIN OF ... END OF` list their parts in `components`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Declaration {
    pub name: String,
    pub kind: DeclarationKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visibility: Option<Visibility>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub typing: Option<Typing>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    /// `READ-ONLY` addition, always true for constants
    pub read_only: bool,
    /// declared with BEGIN OF ... END OF
    pub is_structure: bool,
    /// declared with BEGIN OF ENUM ... END OF ENUM
    pub is_enum: bool,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub components: Vec<Declaration>,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    Importing,
    Exporting,
    Changing,
    Returning,
    /// FORM ... USING
    Using,
    /// FORM and function module TABLES
    Tables,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Parameter {
    pub name: String,
    pub direction: Direction,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub typing: Option<Typing>,
    /// `VALUE(name)`
    pub by_value: bool,
    pub optional: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,
    /// `PREFERRED PARAMETER`
    pub preferred: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct EventHandler {
    pub class: String,
    pub event: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Method {
    pub name: String,
    pub visibility: Visibility,
    pub is_static: bool,
    pub is_abstract: bool,
    pub is_final: bool,
    pub is_redefinition: bool,
    pub is_for_testing: bool,
    /// `DEFAULT IGNORE` or `DEFAULT FAIL`: `ignore` / `fail`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_handler: Option<EventHandler>,
    /// RAP handler method, the text after `FOR`, e.g. `MODIFY`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rap_handler: Option<String>,
    pub parameters: Vec<Parameter>,
    /// class based exceptions, RAISING
    pub raising: Vec<String>,
    /// classic exceptions, EXCEPTIONS
    pub exceptions: Vec<String>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Event {
    pub name: String,
    pub visibility: Visibility,
    pub is_static: bool,
    pub parameters: Vec<Parameter>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Alias {
    pub name: String,
    /// `interface~component`
    pub component: String,
    pub visibility: Visibility,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ImplementedInterface {
    pub name: String,
    pub visibility: Visibility,
    pub partially_implemented: bool,
    pub abstract_methods: Vec<String>,
    pub final_methods: Vec<String>,
    pub all_abstract: bool,
    pub all_final: bool,
}

/// `BY DATABASE ...` of an AMDP method, whose body is SQLScript.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Amdp {
    /// `procedure`, `function` or `graph_workspace`
    pub kind: String,
    /// database system, `HDB`
    pub database: String,
    /// e.g. `SQLSCRIPT`
    pub language: String,
    /// OPTIONS as written, e.g. `READ-ONLY`, `CDS SESSION CLIENT current`
    pub options: Vec<String>,
    /// USING: tables, views and AMDP methods used, e.g. `zcl_x=>get_rows`
    pub using: Vec<String>,
}

/// METHOD ... ENDMETHOD in a class implementation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct MethodImplementation {
    /// as written, e.g. `zif_intf~method` for interface methods
    pub name: String,
    /// `BY DATABASE PROCEDURE|FUNCTION ...`, AMDP
    pub is_amdp: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amdp: Option<Amdp>,
    pub locals: Vec<Declaration>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ClassImplementation {
    pub methods: Vec<MethodImplementation>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Class {
    pub name: String,
    /// `PUBLIC`: a global class
    pub is_global: bool,
    pub is_final: bool,
    pub is_abstract: bool,
    pub is_for_testing: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub superclass: Option<String>,
    /// `CREATE PUBLIC|PROTECTED|PRIVATE`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub create: Option<String>,
    pub friends: Vec<String>,
    /// RAP behavior pool, `FOR BEHAVIOR OF`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub behavior_of: Option<String>,
    pub interfaces: Vec<ImplementedInterface>,
    pub methods: Vec<Method>,
    /// DATA, CLASS-DATA and CONSTANTS
    pub attributes: Vec<Declaration>,
    pub types: Vec<Declaration>,
    pub events: Vec<Event>,
    pub aliases: Vec<Alias>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub implementation: Option<ClassImplementation>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Interface {
    pub name: String,
    pub is_global: bool,
    pub interfaces: Vec<ImplementedInterface>,
    pub methods: Vec<Method>,
    pub attributes: Vec<Declaration>,
    pub types: Vec<Declaration>,
    pub events: Vec<Event>,
    pub aliases: Vec<Alias>,
    pub span: Span,
}

/// FORM subroutine or function module.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Routine {
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub raising: Vec<String>,
    /// classic exceptions of function modules
    pub exceptions: Vec<String>,
    pub locals: Vec<Declaration>,
    pub span: Span,
}

/// Dynpro module, `MODULE name INPUT|OUTPUT`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DialogModule {
    pub name: String,
    /// `input` or `output`, absent for the dynpro flow logic call
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    pub span: Span,
}

/// Event block like START-OF-SELECTION, up to the next processing block.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct EventBlock {
    /// the statement as written, e.g. `AT SELECTION-SCREEN ON p_file`
    pub event: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct NamedSpan {
    pub name: String,
    pub span: Span,
}

/// REPORT, PROGRAM, FUNCTION-POOL, CLASS-POOL, INTERFACE-POOL or TYPE-POOL.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ProgramHeader {
    /// the keyword in lower case, e.g. `report`
    pub kind: String,
    pub name: String,
    pub span: Span,
}

/// Symbols defined in one ABAP source file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct FileSymbols {
    pub schema_version: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub program: Option<ProgramHeader>,
    pub classes: Vec<Class>,
    pub interfaces: Vec<Interface>,
    pub forms: Vec<Routine>,
    pub function_modules: Vec<Routine>,
    pub modules: Vec<DialogModule>,
    pub event_blocks: Vec<EventBlock>,
    pub macros: Vec<NamedSpan>,
    /// INCLUDE statements, in source order
    pub includes: Vec<NamedSpan>,
    /// declarations outside classes and routines
    pub declarations: Vec<Declaration>,
    /// class implementations without a definition in this file, e.g.
    /// the implementation part of a global class in its own include
    pub orphan_implementations: Vec<OrphanImplementation>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct OrphanImplementation {
    pub class: String,
    pub implementation: ClassImplementation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DbOperation {
    /// SELECT, also inside OPEN CURSOR and WITH
    Select,
    Insert,
    Update,
    Modify,
    Delete,
    Merge,
    /// other native SQL, e.g. CALL of a database procedure or DDL
    Native,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DbTableRole {
    /// FROM of a query, or the changed table of INSERT, UPDATE, MODIFY, DELETE, MERGE
    Main,
    /// joined in FROM
    Join,
    /// FROM of a subquery or of a common table expression
    Subquery,
    /// source table of MERGE ... USING
    MergeSource,
    /// common table expression, `+name`
    Cte,
    /// internal table as data source, `FROM @itab`; in SQLScript a table
    /// variable, `FROM :lt_rows`
    InternalTable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DbTable {
    /// database table or view as written; for dynamic tables the operand
    /// inside the parentheses
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alias: Option<String>,
    /// CDS path expression after the name, e.g. `\_items`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// `(lv_table)`: name known only at runtime
    pub dynamic: bool,
    /// for a dynamic name that is a constant of the same file: its value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolved: Option<String>,
    pub role: DbTableRole,
}

/// A column reference, `qualifier~name` or `name`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DbColumn {
    pub name: String,
    /// table alias or table name as written before `~`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub qualifier: Option<String>,
    /// resolved database table, from the qualifier or because there is only one table
    #[serde(skip_serializing_if = "Option::is_none")]
    pub table: Option<String>,
}

/// One element of the SELECT list.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DbField {
    /// as written, e.g. `a~matnr`, `COUNT(*)`, `CASE ... END`
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alias: Option<String>,
    /// `count`, `max`, `min`, `sum`, `avg`, ... when the element is an aggregate
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aggregate: Option<String>,
    /// columns read by the element; empty for `*` and literals
    pub columns: Vec<DbColumn>,
}

/// A comparison in WHERE, HAVING or a join condition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DbCondition {
    pub left: String,
    /// upper case, e.g. `=`, `IN`, `LIKE`, `BETWEEN`, `IS NULL`, `EXISTS`
    pub operator: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub right: Option<String>,
    /// columns compared; host variables and literals are not included
    pub columns: Vec<DbColumn>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DbJoin {
    /// `inner`, `left_outer`, `right_outer` or `cross`
    pub kind: String,
    pub table: DbTable,
    pub on: Vec<DbCondition>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DbTargetKind {
    /// INTO TABLE / APPENDING TABLE
    Table,
    /// a single work area
    Structure,
    /// `INTO (a, b, c)`
    List,
}

/// Where selected data goes, INTO or APPENDING.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DbTarget {
    /// data objects as written, e.g. `@DATA(lt_rows)`, or each element of a list
    pub targets: Vec<String>,
    pub kind: DbTargetKind,
    pub appending: bool,
    /// INTO CORRESPONDING FIELDS OF
    pub corresponding: bool,
}

/// Data source of INSERT, UPDATE and MODIFY, and of DELETE with a work area.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DbSource {
    /// the work area or internal table as written, absent for a subquery
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// FROM TABLE itab
    pub is_table: bool,
    /// FROM ( SELECT ... )
    pub is_subquery: bool,
}

/// `column = value` in UPDATE ... SET.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DbAssignment {
    pub column: DbColumn,
    pub value: String,
}

/// A data cluster in a database table, `EXPORT ... TO DATABASE dbtab(ar)
/// ID id`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DbCluster {
    /// the area, as written, like `mo`
    pub area: String,
    /// the ID as written: a literal with its quotes, or a data object
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// the names in the cluster, lower case: `orders` of
    /// `orders = lt_orders`; in the old form `EXPORT gt_rt TO ...` the data
    /// objects
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub parameters: Vec<String>,
}

/// The processing block a statement belongs to.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct BlockRef {
    /// `method`, `form`, `function_module`, `module`, `event` or `program`
    pub kind: String,
    pub name: String,
    /// class of a method
    #[serde(skip_serializing_if = "Option::is_none")]
    pub class: Option<String>,
}

/// One database access statement.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DbAccess {
    pub operation: DbOperation,
    /// statement kind, e.g. `Select`, `SelectLoop`, `ModifyDatabase`, `OpenCursor`
    pub statement: String,
    /// every table taking part, joins and subqueries included
    pub tables: Vec<DbTable>,
    pub fields: Vec<DbField>,
    pub joins: Vec<DbJoin>,
    /// comparisons of WHERE, subqueries included
    pub conditions: Vec<DbCondition>,
    /// WHERE as written
    #[serde(skip_serializing_if = "Option::is_none")]
    pub where_text: Option<String>,
    pub group_by: Vec<DbColumn>,
    pub having: Vec<DbCondition>,
    /// ORDER BY elements as written, e.g. `matnr DESCENDING`, `PRIMARY KEY`
    pub order_by: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<DbTarget>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<DbSource>,
    pub set: Vec<DbAssignment>,
    /// internal table of FOR ALL ENTRIES IN
    #[serde(skip_serializing_if = "Option::is_none")]
    pub for_all_entries: Option<String>,
    /// cursor variable of OPEN CURSOR
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
    pub single: bool,
    pub distinct: bool,
    /// SELECT ... ENDSELECT loop
    pub is_loop: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub up_to: Option<String>,
    /// UNION, INTERSECT or EXCEPT used
    pub set_operation: bool,
    /// some part is only known at runtime: table, columns or condition
    pub dynamic: bool,
    pub client_specified: bool,
    pub bypassing_buffer: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connection: Option<String>,
    /// native SQL (HANA SQL or SQLScript): the statement as written
    #[serde(skip_serializing_if = "Option::is_none")]
    pub native_sql: Option<String>,
    /// a data cluster: `EXPORT ... TO DATABASE` (`modify`), `IMPORT ...
    /// FROM DATABASE` and `IMPORT DIRECTORY` (`select`), `DELETE FROM
    /// DATABASE` (`delete`); the exported data objects are in `source`,
    /// the imported ones in `target`
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub cluster: Option<DbCluster>,
    /// ABAP data read by the statement, as written: in Open SQL `@lv_x`
    /// without the `@`, the operands of `@( ... )` and the host variables of
    /// the classic syntax; in native SQL `:name` without the colon, the ABAP
    /// host variables of EXEC SQL, parameters and table variables of AMDP
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub host_variables: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block: Option<BlockRef>,
    /// the statement as written
    pub text: String,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CallKind {
    /// method call, `obj->meth( )`, `zcl=>meth( )`, `CALL METHOD`
    Method,
    /// instance creation, `NEW zcl( )`, `CREATE OBJECT`
    Constructor,
    /// `CALL FUNCTION`
    Function,
    /// `PERFORM`
    Form,
    /// `SUBMIT`
    Submit,
    /// `CALL TRANSACTION`, `LEAVE TO TRANSACTION`
    Transaction,
    /// `INCLUDE`
    Include,
    /// `CALL TRANSFORMATION`
    Transformation,
}

/// A call from one processing block to another, or a dependency on an include.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Call {
    pub kind: CallKind,
    /// the calling processing block
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<BlockRef>,
    /// method, form, function module, program, transaction or include name;
    /// for dynamic calls the operand as written
    pub name: String,
    /// class or interface of a method or constructor, when known
    #[serde(skip_serializing_if = "Option::is_none")]
    pub class: Option<String>,
    /// BAdI called with `CALL BADI`, when its definition is known; `class`
    /// is then its interface
    #[serde(skip_serializing_if = "Option::is_none")]
    pub badi: Option<String>,
    /// program of `PERFORM ... IN PROGRAM`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub program: Option<String>,
    /// receiver as written, e.g. `lo_order`, `zcl_x`, `me`, `lo_a->get( )`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub receiver: Option<String>,
    /// `=>`, a static method
    pub is_static: bool,
    /// target known only at runtime
    pub dynamic: bool,
    /// additions like `DESTINATION 'NONE'`, `IN UPDATE TASK`, `AND RETURN`, `IF FOUND`
    pub options: Vec<String>,
    /// the target was found in the analyzed sources; always false for
    /// single file analysis of targets outside the file
    pub resolved: bool,
    /// the class of the target inherits from or implements types that are
    /// not in the sources, e.g. SAP standard classes; an unresolved method
    /// may be defined there
    pub hierarchy_incomplete: bool,
    /// why the target was not found, when `resolved` is false; set by the
    /// call graph
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub unresolved: Option<Unresolved>,
    /// object the call is in, like `CLAS:ZCL_X`; set by the call graph
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
    pub text: String,
    pub span: Span,
}

/// Why the target of a call was not found.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Unresolved {
    /// the target is known only at runtime
    Dynamic,
    /// the class or interface of the receiver is not known
    UnknownType,
    /// no source provider has the target
    Missing,
    /// loading the target failed, see `errors`
    LookupFailed,
    /// the method is not in the class, which inherits from or implements
    /// types that are not in the sources: it may be defined there
    HierarchyIncomplete,
    /// the target does not match the follow patterns, see `skipped`
    NotFollowed,
    /// the target was not loaded: the maximum number of objects was reached
    Limit,
    /// the target was not loaded: the analysis was cancelled
    Cancelled,
    /// a transaction or transformation, which is not analyzed
    NotLoaded,
}

/// What kind of failure an error is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    /// not authorized, HTTP 401 or 403
    Auth,
    /// another HTTP status, in `status`
    Http,
    /// no answer: connection, timeout, proxy, or the answer broke off
    Network,
    /// reading or writing a file, e.g. the cache
    Io,
    Other,
}

/// A failure of a source provider; the analysis went on without what it
/// could not load.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct LoadError {
    pub code: ErrorCode,
    /// the object, like `CLAS:ZCL_X`, or the lookup, like
    /// `implementations:ZIF_X`
    pub object: String,
    /// what failed, like `HTTP 500 for https://host/sap/bc/adt/...`
    pub cause: String,
    /// trying again may succeed: network failures, HTTP 408, 429 and 5xx
    pub retryable: bool,
    /// the HTTP status of `http`
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub status: Option<u16>,
}

impl std::fmt::Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.object, self.cause)
    }
}

/// One loaded source of a call graph.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct GraphUnit {
    /// like `CLAS:ZCL_X`, `CLAS:ZCL_X/implementations`, `PROG:ZREPORT_TOP`
    pub reference: String,
    /// file path or URL
    pub origin: String,
    /// hops from the roots
    pub depth: u32,
}

/// Calls across objects, loaded through the source providers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct CallGraph {
    pub schema_version: u32,
    pub roots: Vec<String>,
    pub units: Vec<GraphUnit>,
    pub calls: Vec<Call>,
    /// referenced objects no provider had; those whose loading failed are
    /// in `errors`
    pub missing: Vec<String>,
    /// referenced objects not loaded: not matching the follow patterns,
    /// beyond the maximum number of objects, or after cancelling
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub skipped: Vec<String>,
    /// provider errors, e.g. authorization failures
    pub errors: Vec<LoadError>,
    /// lookups the system could not do, like `enhancements: not available
    /// (...)`; the graph is complete apart from them
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub notes: Vec<String>,
}

/// One statement, as listed by the statements API.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct StatementInfo {
    /// statement kind, e.g. `Data`, `Select`, `CallFunction`; same names as abaplint
    pub kind: String,
    /// tokens joined by single spaces where the source has whitespace
    pub text: String,
    pub span: Span,
    /// the oldest release that has all the statement uses, when that is
    /// newer than 7.00
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub release: Option<ReleaseNeed>,
    /// a syntax error found in the statement, like `keyword as a column`;
    /// statements without one are not necessarily correct
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub error: Option<String>,
}

/// The release a statement or source needs and the construct that needs it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ReleaseNeed {
    /// on premise, as SAP writes it: `7.02`, `7.40 SP05`, `7.50` to `7.58`, `8.16`
    pub release: String,
    /// the same as abaplint names it, for comparing: `v702`, `v740sp05`, `v750`
    pub id: String,
    /// like `inline declaration`, `host variable with @`, `string template`
    pub construct: String,
}

/// The release a source needs, and what needs more than a given release.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ReleaseReport {
    pub schema_version: u32,
    /// the highest release a statement needs; absent when 7.00 has it all
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub release: Option<ReleaseNeed>,
    /// the release asked about, like `7.50`; absent for 7.00
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub target: Option<String>,
    /// the statements that need a newer release than the target
    pub statements: Vec<StatementInfo>,
    /// the statements with a syntax error found
    pub errors: Vec<StatementInfo>,
}

/// What an operand of a condition is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum OperandKind {
    /// `'text'`, `` `text` ``, `42`, `|text|`, `TEXT-001`
    Literal,
    /// a data object, e.g. `lv_x`, `ls_x-comp`, `lo->attr`, `zcl=>const`
    Data,
    /// a method or function call, e.g. `lines( itab )`, `lo->get( )`
    Call,
    /// `VALUE #( ... )`, `COND #( ... )`, `NEW zcl( )` ...
    Constructor,
    /// arithmetic, string or bit operators
    Arithmetic,
    /// a string template with embedded expressions
    Template,
    /// a component of the current table line, as in `LOOP ... WHERE comp = 1`
    Component,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Operand {
    pub kind: OperandKind,
    /// as written
    pub text: String,
    /// the value of a literal, without quotes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ConditionKind {
    /// `children` combined with AND
    And,
    Or,
    Equiv,
    /// the single child negated
    Not,
    /// `left operator right`, e.g. `a = b`, `a CS b`
    Compare,
    /// `left IS [NOT] operator`, e.g. `INITIAL`, `BOUND`, `INSTANCE OF` with
    /// the class in `right`
    Is,
    /// `left [NOT] IN values`, a range table or a value list
    In,
    /// `left [NOT] BETWEEN right AND high`
    Between,
    /// a predicate method call or function, e.g. `line_exists( ... )`, in `left`
    Predicate,
    /// the exception classes of CATCH, as written in `text`
    Catch,
}

/// A logical expression.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Condition {
    pub kind: ConditionKind,
    /// as written
    pub text: String,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub children: Vec<Condition>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub left: Option<Operand>,
    /// upper case: the comparison operator, or the predicate of IS
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operator: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub right: Option<Operand>,
    /// upper bound of BETWEEN
    #[serde(skip_serializing_if = "Option::is_none")]
    pub high: Option<Operand>,
    /// value list or range table of IN
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub values: Vec<Operand>,
    /// `IS NOT`, `NOT IN`, `NOT BETWEEN`
    #[serde(skip_serializing_if = "std::ops::Not::not", default)]
    pub negated: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AccessKind {
    Read,
    /// replaced, or partly when only a component is written
    Write,
    /// read and changed in place, e.g. the table of `APPEND ... TO itab`
    Change,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DataScope {
    /// declared in the procedure, inline declarations included
    Local,
    Parameter,
    /// attribute of the class, also through `me->` or another object
    Attribute,
    /// global data of the program: declared outside classes and routines,
    /// in the same source or, for objects loaded through a workspace, in
    /// another include such as the TOP include; written by a procedure of
    /// another object it is named `(object)name`, e.g. `(zreport)gv_count`
    Global,
    /// a database table, view or CDS entity; names are upper case
    Database,
    /// data kept outside the program: ABAP memory, SAP memory, files,
    /// shared buffers; named with the kind, e.g. `memory:ZORDERS`
    External,
    /// a system field, like `sy-datum`: set by the runtime, or implicitly
    /// by statements, like `sy-tabix` by READ TABLE
    System,
    /// constants of other classes, or not known
    Other,
}

/// Where data outside of the program and the database is kept.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Store {
    /// ABAP memory, `EXPORT ... TO MEMORY ID`: shared by the programs of a
    /// call sequence, e.g. through SUBMIT ... AND RETURN
    Memory,
    /// SAP memory, `SET/GET PARAMETER ID`: shared by the sessions of a user
    Parameter,
    /// a file on the application server, `OPEN/READ/TRANSFER DATASET`
    File,
    /// `EXPORT ... TO SHARED BUFFER`
    SharedBuffer,
    /// `EXPORT ... TO SHARED MEMORY`
    SharedMemory,
    /// an SAP lock, set by the lock module `ENQUEUE_<lock object>` and
    /// released by `DEQUEUE_<lock object>`
    Lock,
}

/// A built-in elementary type, like that of a system field.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DataType {
    /// `i`, `b`, `c`, `n`, `d`, `t`, `x` or `p`
    pub kind: String,
    /// characters of `c` and `n`, bytes of `x` and `p`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub length: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decimals: Option<u32>,
    /// the data element typing it, like `SYST_DATUM`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_element: Option<String>,
}

/// A data object read or written by a statement.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DataAccess {
    /// lower case; attributes of other objects with the path to them, e.g.
    /// `lo_order->items`, `zcl_x=>gv_count`; `me->` is left out
    pub name: String,
    /// the operand as written, e.g. `ls_item-matnr`
    pub path: String,
    pub access: AccessKind,
    pub scope: DataScope,
    /// writes: the whole data object is replaced, not just a component
    #[serde(skip_serializing_if = "std::ops::Not::not", default)]
    pub whole: bool,
    /// declared here, e.g. `DATA(x)`, `FIELD-SYMBOL(<x>)`, a FOR variable
    #[serde(skip_serializing_if = "std::ops::Not::not", default)]
    pub declared: bool,
    /// set by the statement without being named in it, like `sy-subrc` by
    /// READ TABLE or a call
    #[serde(skip_serializing_if = "std::ops::Not::not", default)]
    pub implicit: bool,
    /// the type, when known; for now that of system fields
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub data_type: Option<DataType>,
    /// reads: statements whose writes may provide the value
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub sources: Vec<u32>,
    /// reads: the value may come from before the procedure, e.g. from a
    /// parameter or attribute
    #[serde(skip_serializing_if = "std::ops::Not::not", default)]
    pub from_entry: bool,
    /// accessed by the called procedure, not by the statement itself
    #[serde(skip_serializing_if = "Option::is_none")]
    pub via: Option<BlockRef>,
    /// accessed through a field symbol or data reference, e.g. `<ls_item>`
    /// for `<ls_item>-amount = 0` after `LOOP AT mt_items ASSIGNING <ls_item>`;
    /// `name` is then the data it points to, `path` the operand as written
    #[serde(skip_serializing_if = "Option::is_none")]
    pub through: Option<String>,
    /// writes through a call: what the written value depends on in the
    /// called procedure
    #[serde(skip_serializing_if = "Option::is_none")]
    pub depends: Option<Dependency>,
    pub position: Position,
}

/// What a value written by a called procedure depends on there, in names
/// of the caller.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Dependency {
    /// data read through the call: attributes, global data
    pub reads: Vec<String>,
    /// the value depends on arguments that could not be matched to the
    /// parameters, e.g. of a dynamic call; matched arguments are in `reads`
    pub arguments: bool,
    /// database tables read, upper case
    pub tables: Vec<String>,
}

/// A database table, or data kept outside the program (`store`), read or
/// written by a statement.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct TableAccess {
    /// table, view or CDS entity, upper case; a dynamic name resolved from
    /// a constant, or the operand as written in parentheses. For a store
    /// its kind and key: `memory:ZORDERS`, `parameter:VKO`,
    /// `file:/tmp/out.csv`, `file:(lv_file)`, `lock:EZ_ORDER`
    pub name: String,
    /// `read`, or `write` for the changed table of INSERT, UPDATE, MODIFY,
    /// DELETE and MERGE, and for stores written by EXPORT, SET PARAMETER,
    /// TRANSFER and their deletes
    pub access: AccessKind,
    /// for stores: `select` when read, `modify` when written, `delete`
    pub operation: DbOperation,
    /// data kept outside the program and the database
    #[serde(skip_serializing_if = "Option::is_none")]
    pub store: Option<Store>,
    /// accessed by the called procedure, not by the statement itself
    #[serde(skip_serializing_if = "Option::is_none")]
    pub via: Option<BlockRef>,
    /// writes through a call: what the written rows depend on in the
    /// called procedure
    #[serde(skip_serializing_if = "Option::is_none")]
    pub depends: Option<Dependency>,
}

/// A condition that holds when a step runs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Guard {
    /// the step with the condition, e.g. an IF, WHEN or CHECK
    pub statement: u32,
    /// the condition is false, e.g. in ELSE or after `IF x. RETURN. ENDIF.`
    pub negated: bool,
    /// the condition as written
    pub text: String,
    /// the condition checks `sy-subrc`: the statements that may have set
    /// the value it checks, like the READ TABLE before `IF sy-subrc = 0`
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub subrc: Vec<u32>,
}

/// One statement of a procedure.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Step {
    /// index of the statement in the file, as in the statements API
    pub id: u32,
    /// statement kind, e.g. `If`, `Move`, `Loop`; same names as abaplint
    pub kind: String,
    pub text: String,
    pub span: Span,
    /// condition of IF, ELSEIF, WHEN, WHILE, CHECK, LOOP ... WHERE, CATCH ...
    #[serde(skip_serializing_if = "Option::is_none")]
    pub condition: Option<Condition>,
    /// all conditions that hold when the step runs, outermost first
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub guards: Vec<Guard>,
    /// data read, changed operands included
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub reads: Vec<DataAccess>,
    /// data written, changed operands included
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub writes: Vec<DataAccess>,
    /// database tables read or written, also by called procedures
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub tables: Vec<TableAccess>,
    /// the statements of a block: the body of IF, ELSEIF, ELSE, LOOP, TRY,
    /// CATCH, WHEN ...; the steps of CASE are its WHENs. ELSEIF, ELSE, CATCH
    /// and CLEANUP follow their IF or TRY as siblings, end statements are
    /// left out
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub children: Vec<Step>,
}

/// Control flow from one step to the next.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct FlowEdge {
    /// step id, absent for the entry of the procedure
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<u32>,
    /// step id, absent for leaving the procedure
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to: Option<u32>,
    /// `true`, `false`, `when`, `other`, `iterate`, `done`, `exception`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

/// A data object outside the procedure, or a database table, that it
/// writes, with what the written values depend on.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Effect {
    pub name: String,
    pub scope: DataScope,
    /// the steps writing it
    pub statements: Vec<u32>,
    /// data read from before the procedure that the values depend on,
    /// through assignments and through the conditions deciding them
    pub inputs: Vec<String>,
    /// database tables and stores read that the values depend on, upper
    /// case, stores named like `memory:ZORDERS`
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub tables: Vec<String>,
    /// the called procedures that write it, when not written directly
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub via: Vec<BlockRef>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ProcedureParameter {
    /// lower case
    pub name: String,
    pub direction: Direction,
}

/// The logic of a method, form, function module, dialog module or event
/// block, or of the statements of a program outside of them.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Procedure {
    pub block: BlockRef,
    /// the source it is in, like `PROG:ZREPORT_F01`; for objects loaded
    /// through a workspace
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
    pub span: Span,
    pub parameters: Vec<ProcedureParameter>,
    /// data declared in the procedure, lower case
    pub locals: Vec<String>,
    pub body: Vec<Step>,
    pub flow: Vec<FlowEdge>,
    pub effects: Vec<Effect>,
}

/// Control flow, conditions and data flow of the procedures of a file, or
/// of all sources of an object loaded through a workspace.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct FileLogic {
    pub schema_version: u32,
    /// the object, like `PROG:ZREPORT`; for objects loaded through a workspace
    #[serde(skip_serializing_if = "Option::is_none")]
    pub object: Option<String>,
    pub procedures: Vec<Procedure>,
    /// objects no provider had: includes, and called objects; those whose
    /// loading failed are in `errors`
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub missing: Vec<String>,
    /// called objects not loaded: not matching the follow patterns, beyond
    /// the maximum number of objects, or after cancelling
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub skipped: Vec<String>,
    /// provider errors, e.g. authorization failures
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub errors: Vec<LoadError>,
    /// lookups the system could not do, like `enhancements: not available
    /// (...)`; the result is complete apart from them
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub notes: Vec<String>,
}

/// A CDS definition, from its DDL source: what it reads and what its
/// elements are made of.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct CdsDefinition {
    pub schema_version: u32,
    /// as written after DEFINE or EXTEND, lower case: `view entity`, `root
    /// view entity`, `view`, `table function`, `abstract entity`, `custom
    /// entity`, `hierarchy`, `extend view entity` ...
    pub kind: String,
    /// the entity as written
    pub name: String,
    /// `@AbapCatalog.sqlViewName` of a classic view, upper case
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sql_view: Option<String>,
    /// `@EndUserText.label`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// `WITH PARAMETERS`
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub parameters: Vec<CdsParameter>,
    /// data sources: SELECT FROM, joins, PROJECTION ON, and the selects
    /// joined by UNION; upper case
    pub sources: Vec<CdsSource>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub associations: Vec<CdsAssociation>,
    pub elements: Vec<CdsElement>,
    /// WHERE as written
    #[serde(skip_serializing_if = "Option::is_none")]
    pub where_text: Option<String>,
    /// a table function's AMDP method, `IMPLEMENTED BY METHOD cl=>meth`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub implemented_by: Option<String>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct CdsParameter {
    pub name: String,
    /// as written
    pub typing: String,
}

/// A data source of a CDS view.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct CdsSource {
    /// table, view or entity, upper case
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alias: Option<String>,
    /// `from`, `inner join`, `left outer join`, `right outer join`, `cross
    /// join`, `projection`, or `union` for the FROM of a following select
    pub role: String,
    /// ON of a join as written
    #[serde(skip_serializing_if = "Option::is_none")]
    pub on: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct CdsAssociation {
    /// the name it is used with, e.g. `_Text`
    pub name: String,
    /// the entity it leads to, upper case
    pub target: String,
    /// e.g. `[0..*]`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cardinality: Option<String>,
    /// COMPOSITION OF instead of ASSOCIATION TO; TO PARENT for the other way
    pub kind: String,
    /// ON as written
    #[serde(skip_serializing_if = "Option::is_none")]
    pub on: Option<String>,
}

/// An element of the select list, or a field of an entity without select.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct CdsElement {
    /// the name it has in the entity: the alias, else the last part
    pub name: String,
    pub key: bool,
    /// the expression as written, or the type of a field
    pub text: String,
    /// the fields it is made of, as `SOURCE.FIELD` upper case with the
    /// alias resolved; through an association `_ASSOC.FIELD`; parameters
    /// as `$PARAMETERS.P`
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub fields: Vec<String>,
    /// an association exposed in the element list
    pub association: bool,
}

/// JSON Schema of [`CdsDefinition`].
pub fn cds_schema() -> serde_json::Value {
    serde_json::to_value(schemars::schema_for!(CdsDefinition)).expect("schema serializes")
}

/// JSON Schema of [`FileSymbols`].
pub fn symbols_schema() -> serde_json::Value {
    serde_json::to_value(schemars::schema_for!(FileSymbols)).expect("schema serializes")
}

/// JSON Schema of the database access API, an array of [`DbAccess`].
pub fn db_access_schema() -> serde_json::Value {
    serde_json::to_value(schemars::schema_for!(Vec<DbAccess>)).expect("schema serializes")
}

/// JSON Schema of the calls API, an array of [`Call`].
pub fn calls_schema() -> serde_json::Value {
    serde_json::to_value(schemars::schema_for!(Vec<Call>)).expect("schema serializes")
}

/// JSON Schema of the call graph API.
pub fn call_graph_schema() -> serde_json::Value {
    serde_json::to_value(schemars::schema_for!(CallGraph)).expect("schema serializes")
}

/// JSON Schema of the statements API, an array of [`StatementInfo`].
pub fn statements_schema() -> serde_json::Value {
    serde_json::to_value(schemars::schema_for!(Vec<StatementInfo>)).expect("schema serializes")
}

/// JSON Schema of the release API.
pub fn release_schema() -> serde_json::Value {
    serde_json::to_value(schemars::schema_for!(ReleaseReport)).expect("schema serializes")
}

/// JSON Schema of the logic API.
pub fn logic_schema() -> serde_json::Value {
    serde_json::to_value(schemars::schema_for!(FileLogic)).expect("schema serializes")
}
