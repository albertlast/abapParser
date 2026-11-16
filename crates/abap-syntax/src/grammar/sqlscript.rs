//! The grammar of SQLScript, the language of AMDP methods on SAP HANA.
//!
//! Written by hand from the SAP HANA SQLScript Reference and the SAP HANA
//! SQL Reference Guide for SAP HANA Platform 2.0 SPS 08, used as references
//! only, and held against the syntax check of the SAP trial 2025, which
//! passes the body of an AMDP method to its HANA 2.0 SPS 08. The arguments
//! of functions with keywords in them, like `CAST(a AS INT)` or
//! `SUBSTR_REGEXPR(p IN s GROUP 1)`, and window specifications are only
//! taken as balanced parentheses.

use super::dsl::*;
use super::E;

/// No unquoted names: SAP HANA's reserved words (SQL Reference, Reserved
/// Words), then the words of conditions, which it does not reserve.
const NOT_NAMES: &[&str] = &[
    "ALL",
    "ALTER",
    "AS",
    "BEFORE",
    "BEGIN",
    "BOTH",
    "CASE",
    "CHAR",
    "CONDITION",
    "CONNECT",
    "CROSS",
    "CUBE",
    "CURRENT_CONNECTION",
    "CURRENT_DATE",
    "CURRENT_SCHEMA",
    "CURRENT_TIME",
    "CURRENT_TIMESTAMP",
    "CURRENT_TRANSACTION_ISOLATION_LEVEL",
    "CURRENT_USER",
    "CURRENT_UTCDATE",
    "CURRENT_UTCTIME",
    "CURRENT_UTCTIMESTAMP",
    "CURRVAL",
    "CURSOR",
    "DECLARE",
    "DISTINCT",
    "ELSE",
    "ELSEIF",
    "END",
    "EXCEPT",
    "EXCEPTION",
    "EXEC",
    "FALSE",
    "FOR",
    "FROM",
    "FULL",
    "GROUP",
    "HAVING",
    "IF",
    "IN",
    "INNER",
    "INOUT",
    "INTERSECT",
    "INTO",
    "IS",
    "JOIN",
    "LATERAL",
    "LEADING",
    "LEFT",
    "LIMIT",
    "LOOP",
    "MINUS",
    "NATURAL",
    "NCHAR",
    "NEXTVAL",
    "NULL",
    "ON",
    "ORDER",
    "OUT",
    "PRIOR",
    "RETURN",
    "RETURNS",
    "REVERSE",
    "RIGHT",
    "ROLLUP",
    "ROWID",
    "SELECT",
    "SESSION_USER",
    "SET",
    "SQL",
    "START",
    "SYSUUID",
    "TABLESAMPLE",
    "TOP",
    "TRAILING",
    "TRUE",
    "UNION",
    "UNKNOWN",
    "USING",
    "UTCTIMESTAMP",
    "VALUES",
    "WHEN",
    "WHERE",
    "WHILE",
    "WITH",
    "AND",
    "OR",
    "NOT",
    "BETWEEN",
    "LIKE",
    "EXISTS",
    "THEN",
    "ESCAPE",
];

/// The body of a procedure or function: its statements.
pub(super) fn body() -> Rule {
    statements()
}

/// The statements of a body, block, branch or loop: declarations of
/// variables, cursors, conditions and handlers come first.
fn statements() -> Rule {
    seq!(star(E::SqsDeclare), star(E::SqsStatement))
}

/// The grammar of a SQLScript expression.
pub(super) fn expression(e: E) -> Option<Rule> {
    Some(match e {
        E::SqsStatement => statement(),
        E::SqsBlock => block(),
        E::SqsDeclare => declare(),
        E::SqsType => sql_type(),
        E::SqsAssign => assign(),
        E::SqsIf => if_(),
        E::SqsLoop => loop_(),
        E::SqsCall => call(),
        E::SqsExec => exec(),
        E::SqsCursor => cursor(),
        E::SqsSql => sql(),
        E::SqsQuery => query(),
        E::SqsQueryTerm => query_term(),
        E::SqsSelect => select(),
        E::SqsSelectItem => select_item(),
        E::SqsTableRef => table_ref(),
        E::SqsTablePrimary => table_primary(),
        E::SqsCondition => condition(),
        E::SqsPredicate => predicate(),
        E::SqsExpression => expr(),
        E::SqsPrimary => primary(),
        E::SqsArgument => argument(),
        E::SqsCase => case(),
        E::SqsSoup => soup(),
        E::SqsInsert => insert(),
        E::SqsUpdate => update(),
        E::SqsDelete => delete(),
        E::SqsUpsert => upsert(),
        E::SqsMerge => merge(),
        E::SqsTarget => target(),
        E::SqsFunction => function(),
        E::SqsLogic => logic(),
        E::SqsWindow => window(),
        _ => return None,
    })
}

// ----- names and literals ----------------------------------------------------

/// A name: unquoted, not reserved, or in double quotes.
fn name() -> Rule {
    alt_prio!(quoted(), plain())
}

/// An unquoted name.
fn plain() -> Rule {
    re_not(r"^[A-Za-z_\x80-\xFF][\w#$\x80-\xFF]*$", NOT_NAMES)
}

fn quoted() -> Rule {
    re(r#"^"(?:[^"]|"")+"$"#)
}

/// `name`, `schema.name`, `"ZCL_X=>METH"`.
fn dotted() -> Rule {
    seq!(name(), star_prio(seq!(".", name())))
}

/// `:name`, `:"name"`.
fn variable() -> Rule {
    re(r#"^:(?:[A-Za-z_\x80-\xFF][\w#$\x80-\xFF]*|"(?:[^"]|"")+")$"#)
}

fn number() -> Rule {
    re(r"^(?:\d+\.?\d*|\.\d+)(?:[eE][+-]?\d+)?$")
}

/// `'text'`, `N'text'`, `X'0A'`.
fn string() -> Rule {
    re(r"^[NnXx]?'(?:[^']|'')*'$")
}

/// A comma list: the rule is given twice, Rule is no Clone.
fn list2(first: Rule, rest: Rule) -> Rule {
    seq!(first, star_prio(seq!(",", rest)))
}

fn names() -> Rule {
    list2(name(), name())
}

fn paren_names() -> Rule {
    seq!("(", names(), ")")
}

fn exprs() -> Rule {
    list2(Rule::from(E::SqsExpression), Rule::from(E::SqsExpression))
}

/// Any tokens with balanced parentheses: arguments of functions with
/// keywords.
fn soup() -> Rule {
    star_prio(alt!(seq!("(", E::SqsSoup, ")"), re(r"^(?:[^();]|..)")))
}

fn hint() -> Rule {
    seq!("WITH", "HINT", "(", E::SqsSoup, ")")
}

// ----- statements ------------------------------------------------------------

fn statement() -> Rule {
    alt!(
        E::SqsBlock,
        E::SqsIf,
        E::SqsLoop,
        seq!(alt_prio!("BREAK", "CONTINUE"), ";"),
        seq!("RETURN", opt(alt!(E::SqsQuery, E::SqsExpression)), ";"),
        seq!(
            alt_prio!("SIGNAL", "RESIGNAL"),
            opt(alt!(name(), number(), seq!("SQL_ERROR_CODE", number()))),
            opt(seq!("SET", "MESSAGE_TEXT", "=", E::SqsExpression)),
            ";"
        ),
        E::SqsCursor,
        E::SqsCall,
        E::SqsExec,
        seq!(E::SqsSql, ";"),
        E::SqsAssign
    )
}

fn block() -> Rule {
    seq!(
        "BEGIN",
        opt_prio(alt_prio!(
            seq!("SEQUENTIAL", "EXECUTION"),
            seq!("PARALLEL", "EXECUTION"),
            seq!("AUTONOMOUS", "TRANSACTION")
        )),
        statements(),
        "END",
        ";"
    )
}

fn declare() -> Rule {
    seq!(
        "DECLARE",
        alt!(
            seq!(
                alt_prio!("EXIT", "CONTINUE"),
                "HANDLER",
                "FOR",
                list2(condition_value(), condition_value()),
                E::SqsStatement
            ),
            seq!(
                "CURSOR",
                name(),
                opt(seq!(
                    "(",
                    list2(seq!(name(), E::SqsType), seq!(name(), E::SqsType)),
                    ")"
                )),
                "FOR",
                E::SqsQuery,
                ";"
            ),
            seq!(
                name(),
                "CONDITION",
                opt(seq!("FOR", "SQL_ERROR_CODE", number())),
                ";"
            ),
            seq!(
                names(),
                opt("CONSTANT"),
                E::SqsType,
                opt(seq!("NOT", "NULL")),
                opt(seq!(
                    alt_prio!("DEFAULT", "=", ":="),
                    alt!(E::SqsQuery, E::SqsExpression)
                )),
                ";"
            )
        )
    )
}

fn condition_value() -> Rule {
    alt!("SQLEXCEPTION", seq!("SQL_ERROR_CODE", number()), name())
}

/// A type: built-in with length, a table type, `TABLE (...)`,
/// `TABLE LIKE :t`, a row, an array.
fn sql_type() -> Rule {
    let column = || {
        seq!(
            name(),
            E::SqsType,
            opt(seq!("NOT", "NULL")),
            opt(seq!("PRIMARY", "KEY"))
        )
    };
    alt!(
        seq!(
            alt_prio!("TABLE", "ROW"),
            alt!(
                seq!(
                    "(",
                    list2(column(), column()),
                    opt(seq!(",", "PRIMARY", "KEY", paren_names())),
                    ")"
                ),
                seq!("LIKE", alt!(variable(), dotted()))
            )
        ),
        seq!(
            alt!(
                quoted(),
                re_not(
                    r"^[A-Za-z_][\w#$]*$",
                    &["ARRAY", "TABLE", "ROW", "CONSTANT", "DEFAULT"]
                ),
                seq!(name(), plus_prio(seq!(".", name())))
            ),
            opt(seq!("(", number(), opt(seq!(",", number())), ")")),
            opt("ARRAY"),
            opt(seq!("=", "ARRAY", "(", opt(exprs()), ")"))
        )
    )
}

fn assign() -> Rule {
    alt!(
        seq!(
            E::SqsTarget,
            alt_prio!("=", ":="),
            alt!(
                E::SqsQuery,
                E::SqsExpression,
                // lt[1] = ('a', 1)
                tuple(),
                // UNNEST(:a, :b) WITH ORDINALITY AS (x, y, i)
                seq!(
                    "UNNEST",
                    "(",
                    exprs(),
                    ")",
                    opt(seq!("WITH", "ORDINALITY")),
                    opt(seq!("AS", opt(name()), paren_names()))
                )
            ),
            ";"
        ),
        // (a, b) = f(...)
        seq!("(", names(), ")", "=", E::SqsExpression, ";"),
        // :lt.INSERT((1, 'a'), 2), :lt.DELETE(1 .. 3)
        seq!(
            variable(),
            ".",
            alt_prio!("INSERT", "UPDATE", "DELETE"),
            "(",
            opt(list2(index_range(), index_range())),
            ")",
            ";"
        )
    )
}

fn index_range() -> Rule {
    alt!(
        seq!(E::SqsExpression, opt(seq!("..", E::SqsExpression))),
        tuple()
    )
}

/// What is assigned to: `x`, `arr[1]`, `lt.col[1]`, `lt[1]`.
fn target() -> Rule {
    seq!(
        name(),
        opt(seq!(".", column())),
        opt(seq!("[", E::SqsExpression, "]"))
    )
}

/// A column of a table variable: `col`, `(col)`.
fn column() -> Rule {
    alt_prio!(name(), seq!("(", name(), ")"))
}

fn if_() -> Rule {
    let branch = || seq!(opt(seq!("SEQUENTIAL", "EXECUTION")), statements());
    seq!(
        "IF",
        E::SqsLogic,
        "THEN",
        branch(),
        star(seq!("ELSEIF", E::SqsLogic, "THEN", branch())),
        opt(seq!("ELSE", branch())),
        "END",
        "IF",
        ";"
    )
}

/// WHILE, FOR and LOOP.
fn loop_() -> Rule {
    let body = |end: &'static str| {
        seq!(
            opt(seq!("SEQUENTIAL", "EXECUTION")),
            statements(),
            "END",
            end,
            ";"
        )
    };
    alt!(
        seq!("WHILE", E::SqsLogic, "DO", body("WHILE")),
        seq!(
            "FOR",
            name(),
            alt!(
                seq!(
                    "IN",
                    opt("REVERSE"),
                    E::SqsExpression,
                    "..",
                    E::SqsExpression
                ),
                seq!("AS", name(), opt(seq!("(", opt(exprs()), ")")))
            ),
            "DO",
            body("FOR")
        ),
        seq!("LOOP", body("LOOP"))
    )
}

/// `CALL p(...)`, `CALL p`, and `p(...)` without CALL.
fn call() -> Rule {
    let arguments = || seq!("(", opt(list2(call_argument(), call_argument())), ")");
    alt!(
        seq!(
            "CALL",
            dotted(),
            opt(arguments()),
            opt(seq!("WITH", "OVERVIEW")),
            opt(hint()),
            ";"
        ),
        seq!(dotted(), arguments(), ";")
    )
}

/// An argument, maybe named; the parameters of a calculation view are
/// named like `PLACEHOLDER."$$P$$" => :v` (SAP's AMDP classes on HANA).
fn call_argument() -> Rule {
    seq!(opt(seq!(dotted(), "=>")), alt!(E::SqsExpression, "?"))
}

/// `WITH PARAMETERS ('PLACEHOLDER' = ('$$P$$', 'x'))`: options of a
/// SELECT, like the input parameters of a calculation view.
fn with_parameters() -> Rule {
    seq!("WITH", "PARAMETERS", "(", E::SqsSoup, ")")
}

fn exec() -> Rule {
    seq!(
        alt_prio!("EXEC", seq!("EXECUTE", "IMMEDIATE")),
        E::SqsExpression,
        opt(seq!(
            "INTO",
            list2(Rule::from(E::SqsTarget), Rule::from(E::SqsTarget)),
            opt(seq!("DEFAULT", exprs()))
        )),
        opt(seq!("USING", exprs())),
        opt(seq!("READS", "SQL", "DATA")),
        ";"
    )
}

/// OPEN, FETCH and CLOSE of a cursor.
fn cursor() -> Rule {
    alt!(
        seq!("OPEN", name(), opt(seq!("(", opt(exprs()), ")")), ";"),
        seq!(
            "FETCH",
            name(),
            "INTO",
            list2(Rule::from(E::SqsTarget), Rule::from(E::SqsTarget)),
            ";"
        ),
        seq!("CLOSE", name(), ";")
    )
}

/// The SQL statements of SQLScript.
fn sql() -> Rule {
    alt!(
        E::SqsQuery,
        E::SqsInsert,
        E::SqsUpdate,
        E::SqsDelete,
        E::SqsUpsert,
        E::SqsMerge,
        seq!("TRUNCATE", "TABLE", dml_table()),
        "COMMIT",
        "ROLLBACK",
        // data definition: not taken apart
        seq!(
            alt_prio!("CREATE", "DROP", "ALTER", "RENAME", "COMMENT", "GRANT", "REVOKE"),
            E::SqsSoup
        )
    )
}

fn dml_table() -> Rule {
    alt!(variable(), dotted())
}

fn alias() -> Rule {
    seq!(opt("AS"), name())
}

fn insert() -> Rule {
    seq!(
        "INSERT",
        "INTO",
        dml_table(),
        opt(paren_names()),
        alt!(seq!("VALUES", "(", exprs(), ")"), E::SqsQuery),
        opt(hint())
    )
}

fn set_list() -> Rule {
    let one = || seq!(dotted(), "=", E::SqsExpression);
    list2(one(), one())
}

fn update() -> Rule {
    seq!(
        "UPDATE",
        dml_table(),
        opt(alias()),
        "SET",
        set_list(),
        opt(seq!(
            "FROM",
            list2(Rule::from(E::SqsTableRef), Rule::from(E::SqsTableRef))
        )),
        opt(seq!("WHERE", E::SqsCondition)),
        opt(hint())
    )
}

fn delete() -> Rule {
    seq!(
        "DELETE",
        opt("HISTORY"),
        "FROM",
        dml_table(),
        opt(alias()),
        opt(seq!("WHERE", E::SqsCondition)),
        opt(hint())
    )
}

fn upsert() -> Rule {
    seq!(
        alt_prio!("UPSERT", "REPLACE"),
        dml_table(),
        opt(paren_names()),
        alt!(
            seq!(
                "VALUES",
                "(",
                exprs(),
                ")",
                opt(alt!(
                    seq!("WHERE", E::SqsCondition),
                    seq!("WITH", "PRIMARY", "KEY")
                ))
            ),
            E::SqsQuery
        ),
        opt(hint())
    )
}

fn merge() -> Rule {
    let when = seq!(
        "WHEN",
        opt("NOT"),
        "MATCHED",
        opt(seq!("AND", E::SqsCondition)),
        "THEN",
        alt!(
            seq!("UPDATE", "SET", set_list()),
            "DELETE",
            seq!("INSERT", opt(paren_names()), "VALUES", "(", exprs(), ")")
        )
    );
    seq!(
        "MERGE",
        "INTO",
        dml_table(),
        opt(alias()),
        "USING",
        E::SqsTablePrimary,
        "ON",
        E::SqsCondition,
        plus(when),
        opt(hint())
    )
}

// ----- queries ---------------------------------------------------------------

fn query() -> Rule {
    let cte = || seq!(name(), opt(paren_names()), "AS", "(", E::SqsQuery, ")");
    seq!(
        opt(seq!("WITH", list2(cte(), cte()))),
        E::SqsQueryTerm,
        star(seq!(
            alt_prio!(
                seq!("UNION", opt(alt_prio!("ALL", "DISTINCT"))),
                seq!("INTERSECT", opt("DISTINCT")),
                seq!("EXCEPT", opt("DISTINCT")),
                "MINUS"
            ),
            E::SqsQueryTerm
        )),
        opt(order_by()),
        opt(limit()),
        opt(into()),
        opt(seq!(
            "FOR",
            alt!(
                seq!("UPDATE", opt(seq!("OF", paren_names())), opt("NOWAIT")),
                seq!("SHARE", "LOCK"),
                seq!("JSON", opt(seq!("(", E::SqsSoup, ")"))),
                seq!("XML", opt(seq!("(", E::SqsSoup, ")")))
            )
        )),
        opt(hint())
    )
}

fn query_term() -> Rule {
    alt!(E::SqsSelect, seq!("(", E::SqsQuery, ")"))
}

fn into() -> Rule {
    seq!(
        "INTO",
        list2(Rule::from(E::SqsTarget), Rule::from(E::SqsTarget)),
        opt(seq!("DEFAULT", exprs()))
    )
}

fn select() -> Rule {
    seq!(
        "SELECT",
        opt(seq!("TOP", alt!(number(), variable()))),
        opt(alt_prio!("DISTINCT", "ALL")),
        list2(Rule::from(E::SqsSelectItem), Rule::from(E::SqsSelectItem)),
        opt(into()),
        "FROM",
        list2(Rule::from(E::SqsTableRef), Rule::from(E::SqsTableRef)),
        opt(seq!("WHERE", E::SqsCondition)),
        opt(seq!("GROUP", "BY", list2(group_item(), group_item()))),
        opt(seq!("HAVING", E::SqsCondition)),
        opt(order_by()),
        opt(limit()),
        opt(with_parameters())
    )
}

fn group_item() -> Rule {
    alt!(
        E::SqsExpression,
        seq!(
            alt_prio!(seq!("GROUPING", "SETS"), "ROLLUP", "CUBE"),
            "(",
            E::SqsSoup,
            ")"
        )
    )
}

fn order_by() -> Rule {
    let item = || {
        seq!(
            E::SqsExpression,
            opt(alt_prio!("ASC", "DESC")),
            opt(seq!("NULLS", alt_prio!("FIRST", "LAST")))
        )
    };
    seq!("ORDER", "BY", list2(item(), item()))
}

fn limit() -> Rule {
    seq!(
        "LIMIT",
        E::SqsExpression,
        opt(seq!("OFFSET", E::SqsExpression)),
        opt(seq!("TOTAL", "ROWCOUNT"))
    )
}

fn select_item() -> Rule {
    alt!(
        "*",
        seq!(dotted(), ".", "*"),
        seq!(E::SqsExpression, opt(alias()))
    )
}

fn table_ref() -> Rule {
    seq!(E::SqsTablePrimary, star(join()))
}

fn join() -> Rule {
    let cardinality = seq!(
        opt("EXACT"),
        alt_prio!("MANY", "ONE"),
        "TO",
        opt("EXACT"),
        alt_prio!("MANY", "ONE")
    );
    alt!(
        seq!(
            opt(alt_prio!(
                "INNER",
                seq!(alt_prio!("LEFT", "RIGHT", "FULL"), opt("OUTER"))
            )),
            opt(cardinality),
            "JOIN",
            E::SqsTablePrimary,
            "ON",
            E::SqsCondition
        ),
        seq!("CROSS", "JOIN", E::SqsTablePrimary)
    )
}

fn table_primary() -> Rule {
    seq!(
        alt!(
            seq!(
                dotted(),
                opt(seq!("(", opt(list2(call_argument(), call_argument())), ")"))
            ),
            seq!(special_function(), "(", E::SqsSoup, ")"),
            variable(),
            seq!("(", E::SqsQuery, ")"),
            seq!("(", E::SqsTableRef, ")"),
            seq!("LATERAL", "(", E::SqsQuery, ")")
        ),
        opt(seq!("PARTITION", "(", E::SqsSoup, ")")),
        opt(seq!(
            "TABLESAMPLE",
            opt(alt_prio!("BERNOULLI", "SYSTEM")),
            "(",
            number(),
            ")"
        )),
        opt(seq!(alias(), opt(paren_names()))),
        opt(with_parameters())
    )
}

// ----- conditions and expressions --------------------------------------------

/// A logical expression of predicates with NOT, AND and OR.
fn logical(atom: fn() -> Rule) -> Rule {
    let not = || seq!(star_prio("NOT"), atom());
    let and = || seq!(not(), star_prio(seq!("AND", not())));
    seq!(and(), star_prio(seq!("OR", and())))
}

/// A condition of SQL: WHERE, ON, HAVING, CASE.
fn condition() -> Rule {
    logical(|| Rule::from(E::SqsPredicate))
}

/// A condition of IF and WHILE: predicates, a function like
/// `IS_EMPTY(:lt)`, an attribute of a cursor like `c::NOTFOUND`.
fn logic() -> Rule {
    logical(|| {
        alt!(
            E::SqsPredicate,
            seq!("(", E::SqsLogic, ")"),
            E::SqsFunction,
            seq!(name(), cursor_attribute())
        )
    })
}

fn cursor_attribute() -> Rule {
    re(r"(?i)^::(?:ISCLOSED|NOTFOUND|ROWCOUNT)$")
}

fn predicate() -> Rule {
    let comparison = alt_prio!("=", "<>", "!=", "<=", ">=", "<", ">");
    alt!(
        seq!("(", E::SqsCondition, ")"),
        seq!("EXISTS", "(", E::SqsQuery, ")"),
        // the text search predicate
        seq!("CONTAINS", "(", E::SqsSoup, ")"),
        seq!(
            E::SqsExpression,
            alt!(
                seq!(
                    comparison,
                    alt!(
                        E::SqsExpression,
                        seq!(alt_prio!("ANY", "SOME", "ALL"), "(", E::SqsQuery, ")")
                    )
                ),
                seq!(
                    opt("NOT"),
                    alt!(
                        seq!("BETWEEN", E::SqsExpression, "AND", E::SqsExpression),
                        seq!("IN", "(", alt!(E::SqsQuery, in_list()), ")"),
                        seq!(
                            "LIKE",
                            E::SqsExpression,
                            opt(seq!("ESCAPE", E::SqsExpression))
                        ),
                        seq!(
                            "LIKE_REGEXPR",
                            E::SqsExpression,
                            opt(seq!("FLAG", E::SqsExpression))
                        ),
                        seq!("MEMBER", "OF", E::SqsExpression)
                    )
                ),
                seq!(
                    "IS",
                    opt("NOT"),
                    alt_prio!("NULL", "TRUE", "FALSE", "UNKNOWN")
                )
            )
        ),
        // (a, b) = (1, 2)
        seq!(
            tuple(),
            alt_prio!("=", "<>", "!=", "<=", ">=", "<", ">"),
            tuple()
        ),
        // (a, b) IN ((1, 2), (3, 4))
        seq!(
            tuple(),
            opt("NOT"),
            "IN",
            "(",
            alt!(E::SqsQuery, in_list()),
            ")"
        )
    )
}

/// `(a, b)`: only in IN lists and for rows of table variables.
fn tuple() -> Rule {
    seq!("(", exprs(), ")")
}

fn in_list() -> Rule {
    list2(
        alt!(E::SqsExpression, tuple()),
        alt!(E::SqsExpression, tuple()),
    )
}

fn expr() -> Rule {
    let operand = || seq!(star_prio(alt_prio!("+", "-")), E::SqsPrimary);
    seq!(
        operand(),
        star_prio(seq!(alt_prio!("+", "-", "*", "/", "||"), operand()))
    )
}

/// Functions with keywords among their arguments, or other forms.
fn special_function() -> Rule {
    re(concat!(
        r"(?i)^(?:TRIM|SUBSTRING|POSITION|CONVERT|TREAT|",
        r"LISTAGG|PERCENTILE_CONT|PERCENTILE_DISC|",
        r"LOCATE_REGEXPR|OCCURRENCES_REGEXPR|REPLACE_REGEXPR|SUBSTR_REGEXPR|SUBSTRING_REGEXPR|",
        r"JSON_VALUE|JSON_QUERY|JSON_TABLE|JSON_EXISTS|XMLTABLE|XMLEXTRACT|XMLEXTRACTVALUE|",
        r"CONTAINS|SCORE|HIERARCHY\w*|SERIES_\w+|MAP_MERGE|MAP_REDUCE|",
        r"CE_\w+|UNNEST|",
        r"BINNING|CUBIC_SPLINE_APPROX|LINEAR_APPROX|RANDOM_PARTITION)$"
    ))
}

/// Arguments in parentheses, maybe none.
fn parens() -> Rule {
    seq!(
        "(",
        opt(list2(
            Rule::from(E::SqsArgument),
            Rule::from(E::SqsArgument)
        )),
        ")"
    )
}

/// A function call: built-in functions by an unquoted name take arguments
/// unless they have none; AMDP functions in quotes and functions of a
/// schema may have none.
fn function() -> Rule {
    let arguments = || {
        seq!(
            "(",
            list2(Rule::from(E::SqsArgument), Rule::from(E::SqsArgument)),
            ")"
        )
    };
    seq!(
        alt!(
            seq!(
                alt!(quoted(), seq!(name(), plus_prio(seq!(".", name())))),
                "(",
                opt(list2(call_argument(), call_argument())),
                ")"
            ),
            seq!(plain(), arguments()),
            seq!(zero_arguments(), "(", ")"),
            seq!(alt_prio!("LEFT", "RIGHT", "CHAR", "NCHAR"), arguments()),
            seq!("CAST", "(", E::SqsExpression, "AS", E::SqsType, ")"),
            seq!("EXTRACT", "(", name(), "FROM", E::SqsExpression, ")"),
            // ordered aggregates: STRING_AGG(a, ',' ORDER BY b)
            seq!(
                re(r"(?i)^(?:STRING_AGG|ARRAY_AGG|FIRST_VALUE|LAST_VALUE|NTH_VALUE)$"),
                "(",
                exprs(),
                opt(order_by()),
                ")"
            ),
            seq!(
                "APPLY_FILTER",
                "(",
                alt!(variable(), dotted()),
                ",",
                E::SqsExpression,
                ")"
            ),
            seq!(special_function(), "(", E::SqsSoup, ")")
        ),
        opt(seq!("WITHIN", "GROUP", "(", order_by(), ")")),
        opt(seq!("OVER", "(", E::SqsWindow, ")")),
        methods()
    )
}

/// Built-in functions without arguments.
fn zero_arguments() -> Rule {
    re(
        r"(?i)^(?:ROW_NUMBER|RANK|DENSE_RANK|PERCENT_RANK|CUME_DIST|RAND|RAND_SECURE|NOW|NEWUID|SCORE|GROUPING_ID|CURRENT_UTCTIMESTAMP|CURRENT_TIMESTAMP)$",
    )
}

/// The window of OVER: partitions, order, frame.
fn window() -> Rule {
    let bound = || {
        alt!(
            seq!("UNBOUNDED", alt_prio!("PRECEDING", "FOLLOWING")),
            seq!("CURRENT", "ROW"),
            seq!(E::SqsExpression, alt_prio!("PRECEDING", "FOLLOWING"))
        )
    };
    seq!(
        opt(seq!("PARTITION", "BY", exprs())),
        opt(order_by()),
        opt(seq!(
            alt_prio!("ROWS", "RANGE"),
            alt!(seq!("BETWEEN", bound(), "AND", bound()), bound())
        ))
    )
}

fn primary() -> Rule {
    alt!(
        number(),
        string(),
        seq!(
            alt_prio!("DATE", "TIME", "TIMESTAMP", "SECONDDATE"),
            string()
        ),
        alt_prio!(
            "NULL",
            "TRUE",
            "FALSE",
            "?",
            "CURRENT_DATE",
            "CURRENT_TIME",
            "CURRENT_TIMESTAMP",
            "CURRENT_UTCDATE",
            "CURRENT_UTCTIME",
            "CURRENT_UTCTIMESTAMP",
            "CURRENT_USER",
            "SESSION_USER",
            "CURRENT_SCHEMA",
            "CURRENT_CONNECTION",
            "CURRENT_TRANSACTION_ISOLATION_LEVEL",
            "SYSUUID",
            "UTCTIMESTAMP"
        ),
        re(
            r"(?i)^::(?:ROWCOUNT|CURRENT_OBJECT_NAME|CURRENT_OBJECT_SCHEMA|CURRENT_LINE_NUMBER|SQL_ERROR_CODE|SQL_ERROR_MESSAGE)$"
        ),
        // :lt.col[1], :arr[2], :lt.SEARCH(...), :lt.*?
        seq!(
            variable(),
            star_prio(alt!(
                seq!(".", column(), opt(parens())),
                seq!("[", E::SqsExpression, "]")
            ))
        ),
        // a, t.a, seq.NEXTVAL
        seq!(dotted(), opt(seq!(".", alt_prio!("NEXTVAL", "CURRVAL")))),
        // c::NOTFOUND of a cursor
        seq!(name(), cursor_attribute()),
        E::SqsFunction,
        seq!("NEW", dotted(), parens()),
        seq!("(", E::SqsExpression, ")"),
        seq!("(", E::SqsQuery, ")"),
        seq!("[", opt(exprs()), "]"),
        E::SqsCase
    )
}

/// `.ST_AsText()` on a spatial value.
fn methods() -> Rule {
    star_prio(seq!(
        ".",
        name(),
        "(",
        opt(list2(
            Rule::from(E::SqsArgument),
            Rule::from(E::SqsArgument)
        )),
        ")"
    ))
}

/// An argument of a function: an expression, `name => value`, `*`,
/// `DISTINCT x`, a query.
fn argument() -> Rule {
    alt!(
        // HANA rejects names its built-in functions do not have, which
        // are not told apart from functions of the system here
        seq!(opt(seq!(name(), "=>")), E::SqsExpression),
        "*",
        seq!(alt_prio!("DISTINCT", "ALL"), E::SqsExpression),
        E::SqsQuery
    )
}

fn case() -> Rule {
    seq!(
        "CASE",
        alt!(
            seq!(
                E::SqsExpression,
                plus(seq!("WHEN", E::SqsExpression, "THEN", E::SqsExpression))
            ),
            plus(seq!("WHEN", E::SqsCondition, "THEN", E::SqsExpression))
        ),
        opt(seq!("ELSE", E::SqsExpression)),
        "END"
    )
}
