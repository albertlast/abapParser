//! The grammar of CDS data definitions.
//!
//! Generated from the CDS parser of abaplint 2.120.105
//! (<https://github.com/abaplint/abaplint>, MIT, Copyright (c) 2015 Lars Hvam)
//! by tools/gen-grammar.mjs --cds, maintained by hand since.

use super::dsl::*;
use super::E;

/// A whole data definition: one of the kinds of definition.
pub(super) fn definition() -> Rule {
    alt!(
        E::CDSDefineView,
        E::CDSDefineAbstract,
        E::CDSDefineProjection,
        E::CDSAnnotate,
        E::CDSDefineCustom,
        E::CDSDefineTableFunction,
        E::CDSExtendView,
        E::CDSDefineHierarchy,
        E::CDSDefineExternalEntity,
        E::CDSDefineTableEntity
    )
}

/// The grammar of a CDS expression.
pub(super) fn expression(e: E) -> Option<Rule> {
    Some(match e {
        E::CDSAggregate => cds_aggregate(),
        E::CDSAnnotate => cds_annotate(),
        E::CDSAnnotation => cds_annotation(),
        E::CDSAnnotationArray => cds_annotation_array(),
        E::CDSAnnotationObject => cds_annotation_object(),
        E::CDSAnnotationSimple => cds_annotation_simple(),
        E::CDSArithmetics => cds_arithmetics(),
        E::CDSArithParen => cds_arith_paren(),
        E::CDSAs => cds_as(),
        E::CDSAssociation => cds_association(),
        E::CDSCardinality => cds_cardinality(),
        E::CDSCase => cds_case(),
        E::CDSCast => cds_cast(),
        E::CDSComposition => cds_composition(),
        E::CDSCondition => cds_condition(),
        E::CDSDefineAbstract => cds_define_abstract(),
        E::CDSDefineCustom => cds_define_custom(),
        E::CDSDefineExternalEntity => cds_define_external_entity(),
        E::CDSDefineHierarchy => cds_define_hierarchy(),
        E::CDSDefineProjection => cds_define_projection(),
        E::CDSDefineTableEntity => cds_define_table_entity(),
        E::CDSDefineTableFunction => cds_define_table_function(),
        E::CDSDefineView => cds_define_view(),
        E::CDSElement => cds_element(),
        E::CDSExtendView => cds_extend_view(),
        E::CDSFunction => cds_function(),
        E::CDSFunctionInput => cds_function_input(),
        E::CDSGroupBy => cds_group_by(),
        E::CDSHaving => cds_having(),
        E::CDSInteger => cds_integer(),
        E::CDSJoin => cds_join(),
        E::CDSName => cds_name(),
        E::CDSParameters => cds_parameters(),
        E::CDSParametersSelect => cds_parameters_select(),
        E::CDSParenSelect => cds_paren_select(),
        E::CDSPrefixedName => cds_prefixed_name(),
        E::CDSProviderContract => cds_provider_contract(),
        E::CDSRelation => cds_relation(),
        E::CDSSelect => cds_select(),
        E::CDSSource => cds_source(),
        E::CDSString => cds_string(),
        E::CDSTableField => cds_table_field(),
        E::CDSType => cds_type(),
        E::CDSWhere => cds_where(),
        E::CDSWithParameters => cds_with_parameters(),
        _ => return None,
    })
}

fn cds_aggregate() -> Rule {
    seq!(
        alt_prio!("MAX", "MIN", "SUM", "AVG", "COUNT"),
        "(",
        opt_prio(alt_prio!("DISTINCT", "ALL")),
        alt_prio!(
            E::CDSArithmetics,
            E::CDSArithParen,
            E::CDSCast,
            E::CDSCase,
            seq!(E::CDSFunction, "AS", E::CDSType),
            E::CDSFunction,
            seq!(E::CDSPrefixedName, "AS", E::CDSType),
            E::CDSPrefixedName,
            E::CDSString,
            "*"
        ),
        opt(seq!("AS", E::CDSType)),
        ")"
    )
}

fn cds_annotate() -> Rule {
    seq!(
        star(E::CDSAnnotation),
        "ANNOTATE",
        alt!("ENTITY", "VIEW"),
        E::CDSName,
        "WITH",
        "{",
        plus(seq!(E::CDSElement, ";")),
        "}",
        opt(";")
    )
}

fn cds_annotation() -> Rule {
    seq!(
        alt!(
            re(r"^@[A-Za-z]\w*$"),
            seq!("@", annotation_part()),
            seq!("@", "<", annotation_part())
        ),
        star(seq!(".", annotation_part())),
        opt_prio(seq!(
            ":",
            alt!(
                E::CDSAnnotationArray,
                E::CDSAnnotationObject,
                E::CDSAnnotationSimple
            )
        ))
    )
}

fn cds_annotation_array() -> Rule {
    seq!(
        "[",
        alt!(
            E::CDSAnnotationSimple,
            E::CDSAnnotationObject,
            E::CDSAnnotationArray
        ),
        star(seq!(
            ",",
            alt!(
                E::CDSAnnotationSimple,
                E::CDSAnnotationObject,
                E::CDSAnnotationArray
            )
        )),
        "]"
    )
}

fn cds_annotation_object() -> Rule {
    seq!(
        "{",
        seq!(annotation_part(), star(seq!(".", annotation_part()))),
        opt_prio(seq!(
            ":",
            alt!(
                E::CDSAnnotationObject,
                E::CDSAnnotationArray,
                E::CDSAnnotationSimple
            )
        )),
        star(seq!(
            ",",
            seq!(annotation_part(), star(seq!(".", annotation_part()))),
            opt_prio(seq!(
                ":",
                alt!(
                    E::CDSAnnotationObject,
                    E::CDSAnnotationArray,
                    E::CDSAnnotationSimple
                )
            ))
        )),
        "}"
    )
}

fn cds_annotation_simple() -> Rule {
    alt_prio!(
        E::CDSString,
        "TRUE",
        "FALSE",
        "NULL",
        seq!(
            "-",
            re(r"^\d+$"),
            ".",
            re(r"^\d+[eE]$"),
            alt_prio!("+", "-"),
            re(r"^\d+$")
        ),
        seq!(
            re(r"^\d+$"),
            ".",
            re(r"^\d+[eE]$"),
            alt_prio!("+", "-"),
            re(r"^\d+$")
        ),
        seq!("-", re(r"^\d+$"), ".", re(r"^\d+$")),
        seq!("-", re(r"^\d+$")),
        seq!(re(r"^\d+$"), ".", re(r"^\d+$")),
        re(r"^\d+$"),
        seq!(
            "#",
            "(",
            seq!(
                alt_prio!(
                    E::CDSString,
                    seq!(
                        re(r"^[\w_]+$"),
                        "(",
                        alt_prio!(E::CDSString, E::CDSPrefixedName, re(r"^[\w_]+$")),
                        ")"
                    ),
                    E::CDSPrefixedName,
                    re(r"^[\w_]+$")
                ),
                star_prio(seq!(
                    "+",
                    alt_prio!(
                        E::CDSString,
                        seq!(
                            re(r"^[\w_]+$"),
                            "(",
                            alt_prio!(E::CDSString, E::CDSPrefixedName, re(r"^[\w_]+$")),
                            ")"
                        ),
                        E::CDSPrefixedName,
                        re(r"^[\w_]+$")
                    )
                ))
            ),
            ")"
        ),
        re(r"^#[\w_]+$")
    )
}

fn cds_arithmetics() -> Rule {
    alt_prio!(
        seq!(
            seq!(
                "(",
                alt_prio!(
                    seq!(
                        alt_prio!(
                            E::CDSArithParen,
                            seq!(
                                alt_prio!("-", "+"),
                                alt_prio!(
                                    E::CDSInteger,
                                    E::CDSFunction,
                                    E::CDSCase,
                                    E::CDSCast,
                                    E::CDSString,
                                    E::CDSAggregate,
                                    E::CDSPrefixedName
                                )
                            ),
                            alt_prio!(
                                E::CDSInteger,
                                E::CDSFunction,
                                E::CDSCase,
                                E::CDSCast,
                                E::CDSString,
                                E::CDSAggregate,
                                E::CDSPrefixedName
                            )
                        ),
                        plus_prio(seq!(
                            alt_prio!("+", "-", "*", "/"),
                            alt_prio!(
                                E::CDSArithParen,
                                seq!(
                                    alt_prio!("-", "+"),
                                    alt_prio!(
                                        E::CDSInteger,
                                        E::CDSFunction,
                                        E::CDSCase,
                                        E::CDSCast,
                                        E::CDSString,
                                        E::CDSAggregate,
                                        E::CDSPrefixedName
                                    )
                                ),
                                alt_prio!(
                                    E::CDSInteger,
                                    E::CDSFunction,
                                    E::CDSCase,
                                    E::CDSCast,
                                    E::CDSString,
                                    E::CDSAggregate,
                                    E::CDSPrefixedName
                                )
                            )
                        ))
                    ),
                    seq!(
                        alt_prio!("-", "+"),
                        alt_prio!(
                            E::CDSInteger,
                            E::CDSFunction,
                            E::CDSCase,
                            E::CDSCast,
                            E::CDSString,
                            E::CDSAggregate,
                            E::CDSPrefixedName
                        )
                    )
                ),
                ")"
            ),
            star_prio(seq!(
                alt_prio!("+", "-", "*", "/"),
                alt_prio!(
                    E::CDSArithParen,
                    seq!(
                        alt_prio!("-", "+"),
                        alt_prio!(
                            E::CDSInteger,
                            E::CDSFunction,
                            E::CDSCase,
                            E::CDSCast,
                            E::CDSString,
                            E::CDSAggregate,
                            E::CDSPrefixedName
                        )
                    ),
                    alt_prio!(
                        E::CDSInteger,
                        E::CDSFunction,
                        E::CDSCase,
                        E::CDSCast,
                        E::CDSString,
                        E::CDSAggregate,
                        E::CDSPrefixedName
                    )
                )
            ))
        ),
        seq!(
            seq!(alt_prio!("-", "+"), E::CDSArithParen),
            star_prio(seq!(
                alt_prio!("+", "-", "*", "/"),
                alt_prio!(
                    E::CDSArithParen,
                    seq!(
                        alt_prio!("-", "+"),
                        alt_prio!(
                            E::CDSInteger,
                            E::CDSFunction,
                            E::CDSCase,
                            E::CDSCast,
                            E::CDSString,
                            E::CDSAggregate,
                            E::CDSPrefixedName
                        )
                    ),
                    alt_prio!(
                        E::CDSInteger,
                        E::CDSFunction,
                        E::CDSCase,
                        E::CDSCast,
                        E::CDSString,
                        E::CDSAggregate,
                        E::CDSPrefixedName
                    )
                )
            ))
        ),
        seq!(
            seq!(
                alt_prio!("-", "+"),
                alt_prio!(
                    E::CDSInteger,
                    E::CDSFunction,
                    E::CDSCase,
                    E::CDSCast,
                    E::CDSString,
                    E::CDSAggregate,
                    E::CDSPrefixedName
                )
            ),
            plus_prio(seq!(
                alt_prio!("+", "-", "*", "/"),
                alt_prio!(
                    E::CDSArithParen,
                    seq!(
                        alt_prio!("-", "+"),
                        alt_prio!(
                            E::CDSInteger,
                            E::CDSFunction,
                            E::CDSCase,
                            E::CDSCast,
                            E::CDSString,
                            E::CDSAggregate,
                            E::CDSPrefixedName
                        )
                    ),
                    alt_prio!(
                        E::CDSInteger,
                        E::CDSFunction,
                        E::CDSCase,
                        E::CDSCast,
                        E::CDSString,
                        E::CDSAggregate,
                        E::CDSPrefixedName
                    )
                )
            ))
        ),
        seq!(
            alt_prio!("-", "+"),
            alt_prio!(
                E::CDSInteger,
                E::CDSFunction,
                E::CDSCase,
                E::CDSCast,
                E::CDSString,
                E::CDSAggregate,
                E::CDSPrefixedName
            )
        ),
        seq!(alt_prio!("-", "+"), E::CDSArithParen),
        seq!(
            alt_prio!(
                E::CDSArithParen,
                seq!(
                    alt_prio!("-", "+"),
                    alt_prio!(
                        E::CDSInteger,
                        E::CDSFunction,
                        E::CDSCase,
                        E::CDSCast,
                        E::CDSString,
                        E::CDSAggregate,
                        E::CDSPrefixedName
                    )
                ),
                alt_prio!(
                    E::CDSInteger,
                    E::CDSFunction,
                    E::CDSCase,
                    E::CDSCast,
                    E::CDSString,
                    E::CDSAggregate,
                    E::CDSPrefixedName
                )
            ),
            plus_prio(seq!(
                alt_prio!("+", "-", "*", "/"),
                alt_prio!(
                    E::CDSArithParen,
                    seq!(
                        alt_prio!("-", "+"),
                        alt_prio!(
                            E::CDSInteger,
                            E::CDSFunction,
                            E::CDSCase,
                            E::CDSCast,
                            E::CDSString,
                            E::CDSAggregate,
                            E::CDSPrefixedName
                        )
                    ),
                    alt_prio!(
                        E::CDSInteger,
                        E::CDSFunction,
                        E::CDSCase,
                        E::CDSCast,
                        E::CDSString,
                        E::CDSAggregate,
                        E::CDSPrefixedName
                    )
                )
            ))
        )
    )
}

fn cds_arith_paren() -> Rule {
    seq!(
        "(",
        alt_prio!(
            E::CDSArithmetics,
            E::CDSArithParen,
            alt_prio!(
                E::CDSInteger,
                E::CDSFunction,
                E::CDSCase,
                E::CDSCast,
                E::CDSString,
                E::CDSAggregate,
                E::CDSPrefixedName
            )
        ),
        ")"
    )
}

fn cds_as() -> Rule {
    seq!(
        "AS",
        E::CDSName,
        opt_prio(alt_prio!(
            seq!(
                ": REDIRECTED TO",
                opt_prio(alt_prio!("PARENT", "COMPOSITION CHILD")),
                E::CDSName
            ),
            seq!(":", alt_prio!(E::CDSType, E::CDSName, "LOCALIZED"))
        ))
    )
}

fn cds_association() -> Rule {
    alt_prio!(
        seq!(
            "ASSOCIATION",
            "OF",
            alt_prio!(seq!("EXACT", "ONE"), "ONE", "MANY"),
            "TO",
            alt_prio!(seq!("EXACT", "ONE"), "ONE", "MANY"),
            E::CDSRelation,
            "ON",
            E::CDSCondition,
            opt(seq!("WITH", "DEFAULT", "FILTER", E::CDSCondition))
        ),
        seq!(
            "ASSOCIATION",
            "OF",
            seq!(
                "[",
                alt_prio!(re(r"^\d+$"), "*"),
                opt_prio(seq!(".", ".", alt_prio!(re(r"^\d+$"), "*"))),
                "]"
            ),
            "TO",
            E::CDSRelation,
            "ON",
            E::CDSCondition,
            opt(seq!("WITH", "DEFAULT", "FILTER", E::CDSCondition))
        ),
        seq!(
            "ASSOCIATION",
            opt_prio(E::CDSCardinality),
            "TO",
            opt(alt_prio!(
                seq!(opt("EXACT"), alt_prio!("ONE", "MANY")),
                "PARENT"
            )),
            E::CDSRelation,
            "ON",
            E::CDSCondition,
            opt(seq!("WITH", "DEFAULT", "FILTER", E::CDSCondition))
        ),
        seq!("ASSOCIATION", "TO", "PARENT", E::CDSRelation)
    )
}

fn cds_cardinality() -> Rule {
    alt_prio!(
        seq!(
            "[",
            alt_prio!(re(r"^\d+$"), "*"),
            opt_prio(seq!(".", ".", alt_prio!(re(r"^\d+$"), "*"))),
            "]"
        ),
        seq!(alt_prio!("ONE", "MANY"), "TO", alt_prio!("ONE", "MANY"))
    )
}

fn cds_case() -> Rule {
    seq!(
        "CASE",
        alt_prio!(
            plus_prio(seq!(
                "WHEN",
                E::CDSCondition,
                "THEN",
                alt_prio!(
                    E::CDSString,
                    E::CDSCase,
                    seq!("(", E::CDSCase, ")"),
                    E::CDSArithmetics,
                    E::CDSCast,
                    E::CDSAggregate,
                    E::CDSArithParen,
                    E::CDSFunction,
                    E::CDSInteger,
                    E::CDSPrefixedName
                )
            )),
            seq!(
                alt_prio!(
                    E::CDSString,
                    E::CDSCast,
                    E::CDSArithmetics,
                    E::CDSArithParen,
                    E::CDSAggregate,
                    E::CDSFunction,
                    E::CDSPrefixedName
                ),
                plus_prio(seq!(
                    "WHEN",
                    alt_prio!(
                        E::CDSString,
                        E::CDSCase,
                        seq!("(", E::CDSCase, ")"),
                        E::CDSArithmetics,
                        E::CDSCast,
                        E::CDSAggregate,
                        E::CDSArithParen,
                        E::CDSFunction,
                        E::CDSInteger,
                        E::CDSPrefixedName
                    ),
                    "THEN",
                    alt_prio!(
                        E::CDSString,
                        E::CDSCase,
                        seq!("(", E::CDSCase, ")"),
                        E::CDSArithmetics,
                        E::CDSCast,
                        E::CDSAggregate,
                        E::CDSArithParen,
                        E::CDSFunction,
                        E::CDSInteger,
                        E::CDSPrefixedName
                    )
                ))
            )
        ),
        opt_prio(seq!(
            "ELSE",
            alt_prio!(
                "NULL",
                E::CDSString,
                E::CDSCase,
                seq!("(", E::CDSCase, ")"),
                E::CDSArithmetics,
                E::CDSCast,
                E::CDSAggregate,
                E::CDSArithParen,
                E::CDSFunction,
                E::CDSInteger,
                E::CDSPrefixedName
            )
        )),
        "END"
    )
}

fn cds_cast() -> Rule {
    seq!(
        "CAST",
        "(",
        alt_prio!(
            E::CDSArithmetics,
            E::CDSFunction,
            E::CDSArithParen,
            E::CDSCase,
            E::CDSAggregate,
            E::CDSCast,
            E::CDSInteger,
            E::CDSString,
            E::CDSPrefixedName
        ),
        "AS",
        E::CDSType,
        opt_prio(seq!("PRESERVING", "TYPE")),
        ")"
    )
}

fn cds_composition() -> Rule {
    seq!(
        "COMPOSITION",
        opt_prio(seq!(
            "[",
            alt_prio!(re(r"^\d+$"), "*"),
            opt_prio(seq!(".", ".", alt_prio!(re(r"^\d+$"), "*"))),
            "]"
        )),
        "OF",
        opt(alt_prio!(
            E::CDSCardinality,
            alt_prio!(
                seq!(
                    seq!(opt("EXACT"), alt_prio!("ONE", "MANY")),
                    "TO",
                    seq!(opt("EXACT"), alt_prio!("ONE", "MANY"))
                ),
                seq!(opt("EXACT"), alt_prio!("ONE", "MANY"))
            )
        )),
        E::CDSRelation
    )
}

fn cds_condition() -> Rule {
    seq!(
        alt_prio!(
            seq!(
                opt_prio("NOT"),
                alt_prio!(
                    E::CDSString,
                    E::CDSArithmetics,
                    E::CDSCast,
                    E::CDSFunction,
                    E::CDSAggregate,
                    E::CDSCase,
                    E::CDSArithParen,
                    E::CDSInteger,
                    E::CDSPrefixedName
                ),
                alt_prio!(
                    alt_prio!(
                        seq!(
                            alt_prio!("LIKE", "NOT LIKE"),
                            alt_prio!(
                                E::CDSArithParen,
                                alt_prio!(
                                    E::CDSString,
                                    E::CDSArithmetics,
                                    E::CDSCast,
                                    E::CDSFunction,
                                    E::CDSAggregate,
                                    E::CDSCase,
                                    E::CDSArithParen,
                                    E::CDSInteger,
                                    E::CDSPrefixedName
                                ),
                                E::CDSInteger,
                                E::CDSArithmetics
                            ),
                            opt_prio(seq!("ESCAPE", E::CDSString))
                        ),
                        seq!(
                            alt_prio!(
                                "=",
                                seq!("!", "="),
                                seq!("<", ">"),
                                seq!(">", "="),
                                seq!("<", "="),
                                "<",
                                ">"
                            ),
                            alt_prio!(
                                E::CDSArithParen,
                                alt_prio!(
                                    E::CDSString,
                                    E::CDSArithmetics,
                                    E::CDSCast,
                                    E::CDSFunction,
                                    E::CDSAggregate,
                                    E::CDSCase,
                                    E::CDSArithParen,
                                    E::CDSInteger,
                                    E::CDSPrefixedName
                                ),
                                E::CDSInteger,
                                E::CDSArithmetics
                            )
                        )
                    ),
                    seq!("IS", opt_prio("NOT"), alt_prio!("INITIAL", "NULL")),
                    seq!(
                        alt_prio!("NOT BETWEEN", "BETWEEN"),
                        alt_prio!(
                            E::CDSString,
                            E::CDSArithmetics,
                            E::CDSCast,
                            E::CDSFunction,
                            E::CDSAggregate,
                            E::CDSCase,
                            E::CDSArithParen,
                            E::CDSInteger,
                            E::CDSPrefixedName
                        ),
                        "AND",
                        alt_prio!(
                            E::CDSString,
                            E::CDSArithmetics,
                            E::CDSCast,
                            E::CDSFunction,
                            E::CDSAggregate,
                            E::CDSCase,
                            E::CDSArithParen,
                            E::CDSInteger,
                            E::CDSPrefixedName
                        )
                    )
                )
            ),
            seq!("NOT", seq!("(", E::CDSCondition, ")")),
            seq!("(", E::CDSCondition, ")")
        ),
        star(seq!(
            alt_prio!("AND", "OR"),
            alt_prio!(
                seq!(
                    opt_prio("NOT"),
                    alt_prio!(
                        E::CDSString,
                        E::CDSArithmetics,
                        E::CDSCast,
                        E::CDSFunction,
                        E::CDSAggregate,
                        E::CDSCase,
                        E::CDSArithParen,
                        E::CDSInteger,
                        E::CDSPrefixedName
                    ),
                    alt_prio!(
                        alt_prio!(
                            seq!(
                                alt_prio!("LIKE", "NOT LIKE"),
                                alt_prio!(
                                    E::CDSArithParen,
                                    alt_prio!(
                                        E::CDSString,
                                        E::CDSArithmetics,
                                        E::CDSCast,
                                        E::CDSFunction,
                                        E::CDSAggregate,
                                        E::CDSCase,
                                        E::CDSArithParen,
                                        E::CDSInteger,
                                        E::CDSPrefixedName
                                    ),
                                    E::CDSInteger,
                                    E::CDSArithmetics
                                ),
                                opt_prio(seq!("ESCAPE", E::CDSString))
                            ),
                            seq!(
                                alt_prio!(
                                    "=",
                                    seq!("!", "="),
                                    seq!("<", ">"),
                                    seq!(">", "="),
                                    seq!("<", "="),
                                    "<",
                                    ">"
                                ),
                                alt_prio!(
                                    E::CDSArithParen,
                                    alt_prio!(
                                        E::CDSString,
                                        E::CDSArithmetics,
                                        E::CDSCast,
                                        E::CDSFunction,
                                        E::CDSAggregate,
                                        E::CDSCase,
                                        E::CDSArithParen,
                                        E::CDSInteger,
                                        E::CDSPrefixedName
                                    ),
                                    E::CDSInteger,
                                    E::CDSArithmetics
                                )
                            )
                        ),
                        seq!("IS", opt_prio("NOT"), alt_prio!("INITIAL", "NULL")),
                        seq!(
                            alt_prio!("NOT BETWEEN", "BETWEEN"),
                            alt_prio!(
                                E::CDSString,
                                E::CDSArithmetics,
                                E::CDSCast,
                                E::CDSFunction,
                                E::CDSAggregate,
                                E::CDSCase,
                                E::CDSArithParen,
                                E::CDSInteger,
                                E::CDSPrefixedName
                            ),
                            "AND",
                            alt_prio!(
                                E::CDSString,
                                E::CDSArithmetics,
                                E::CDSCast,
                                E::CDSFunction,
                                E::CDSAggregate,
                                E::CDSCase,
                                E::CDSArithParen,
                                E::CDSInteger,
                                E::CDSPrefixedName
                            )
                        )
                    )
                ),
                seq!("NOT", seq!("(", E::CDSCondition, ")")),
                seq!("(", E::CDSCondition, ")")
            )
        ))
    )
}

fn cds_define_abstract() -> Rule {
    seq!(
        star(E::CDSAnnotation),
        opt("DEFINE"),
        opt("ROOT"),
        "ABSTRACT",
        "ENTITY",
        E::CDSName,
        opt(E::CDSWithParameters),
        "{",
        plus(alt!(
            seq!(
                star(E::CDSAnnotation),
                opt_prio("KEY"),
                name(),
                ":",
                E::CDSType,
                ";"
            ),
            seq!(
                star(E::CDSAnnotation),
                name(),
                ":",
                alt!(E::CDSComposition, E::CDSAssociation),
                ";"
            )
        )),
        "}",
        opt(";")
    )
}

fn cds_define_custom() -> Rule {
    seq!(
        star(E::CDSAnnotation),
        opt("DEFINE"),
        opt("ROOT"),
        "CUSTOM ENTITY",
        E::CDSName,
        opt(E::CDSWithParameters),
        "{",
        plus(seq!(
            star(E::CDSAnnotation),
            alt!(
                seq!(opt_prio("KEY"), name(), ":", E::CDSType, ";"),
                seq!(name(), ":", alt!(E::CDSComposition, E::CDSAssociation), ";")
            )
        )),
        "}",
        opt(";")
    )
}

fn cds_define_external_entity() -> Rule {
    seq!(
        star(E::CDSAnnotation),
        opt("DEFINE"),
        opt("ROOT"),
        alt!(
            seq!(
                opt("WRITABLE"),
                "EXTERNAL",
                "ENTITY",
                E::CDSName,
                opt(seq!(
                    "EXTERNAL",
                    "NAME",
                    alt!(E::CDSName, re(r#"^"(?:[^"]|"")*"$"#))
                )),
                opt(E::CDSWithParameters)
            ),
            seq!(
                "STATIC",
                "ENTITY",
                E::CDSName,
                opt(seq!(
                    "EXTERNAL",
                    "NAME",
                    alt!(E::CDSName, re(r#"^"(?:[^"]|"")*"$"#))
                ))
            )
        ),
        "{",
        plus(alt!(
            seq!(
                star(E::CDSAnnotation),
                opt_prio("KEY"),
                E::CDSName,
                ":",
                E::CDSType,
                opt_prio(alt!("NOT NULL", "NULL")),
                opt_prio(seq!(
                    "EXTERNAL",
                    "NAME",
                    alt!(E::CDSName, re(r#"^"(?:[^"]|"")*"$"#))
                )),
                ";"
            ),
            seq!(
                star(E::CDSAnnotation),
                E::CDSName,
                ":",
                alt!(E::CDSComposition, E::CDSAssociation),
                ";"
            )
        )),
        "}",
        opt(seq!(
            "WITH",
            "FEDERATED",
            "DATA",
            opt_prio(alt!(
                seq!("PROVIDED", "AT", "RUNTIME"),
                seq!("PROVIDED", "BY", E::CDSName)
            )),
            opt_prio(seq!("ON", "UNLINKED", alt_prio!("FAIL", "IGNORE")))
        )),
        opt(";")
    )
}

fn cds_define_hierarchy() -> Rule {
    seq!(
        star(E::CDSAnnotation),
        opt("DEFINE"),
        "HIERARCHY",
        E::CDSName,
        opt(E::CDSWithParameters),
        "AS",
        "PARENT",
        "CHILD",
        "HIERARCHY",
        "(",
        seq!(
            "SOURCE",
            E::CDSName,
            opt(E::CDSParametersSelect),
            "CHILD",
            "TO",
            "PARENT",
            "ASSOCIATION",
            E::CDSName,
            opt(alt_prio!(
                seq!(
                    opt(seq!(
                        "DIRECTORY",
                        E::CDSName,
                        "FILTER",
                        "BY",
                        E::CDSCondition
                    )),
                    seq!(
                        "PERIOD",
                        "FROM",
                        E::CDSName,
                        "TO",
                        E::CDSName,
                        opt(seq!(
                            "VALID",
                            "FROM",
                            alt_prio!(E::CDSString, E::CDSPrefixedName),
                            "TO",
                            alt_prio!(E::CDSString, E::CDSPrefixedName)
                        ))
                    ),
                    opt(seq!(
                        "DIRECTORY",
                        E::CDSName,
                        "FILTER",
                        "BY",
                        E::CDSCondition
                    ))
                ),
                seq!(
                    seq!(
                        "PERIOD",
                        "FROM",
                        E::CDSName,
                        "TO",
                        E::CDSName,
                        opt(seq!(
                            "VALID",
                            "FROM",
                            alt_prio!(E::CDSString, E::CDSPrefixedName),
                            "TO",
                            alt_prio!(E::CDSString, E::CDSPrefixedName)
                        ))
                    ),
                    opt(seq!(
                        "DIRECTORY",
                        E::CDSName,
                        "FILTER",
                        "BY",
                        E::CDSCondition
                    ))
                ),
                seq!("DIRECTORY", E::CDSName, "FILTER", "BY", E::CDSCondition)
            )),
            opt(seq!("START", "WHERE", E::CDSCondition)),
            opt(seq!(
                "SIBLINGS",
                "ORDER",
                "BY",
                seq!(
                    E::CDSPrefixedName,
                    opt_prio(alt_prio!("ASCENDING", "DESCENDING"))
                ),
                star(seq!(
                    ",",
                    seq!(
                        E::CDSPrefixedName,
                        opt_prio(alt_prio!("ASCENDING", "DESCENDING"))
                    )
                ))
            )),
            opt(seq!(
                "DEPTH",
                alt_prio!(E::CDSString, E::CDSInteger, E::CDSPrefixedName)
            )),
            opt(seq!("NODETYPE", E::CDSName)),
            opt(seq!(
                "LOAD",
                alt_prio!("BULK", "INCREMENTAL", E::CDSPrefixedName)
            )),
            opt(seq!(
                "MULTIPLE",
                "PARENTS",
                alt_prio!("NOT ALLOWED", "ALLOWED", seq!("LEAVES", opt_prio("ONLY")))
            )),
            opt(seq!("ORPHANS", alt_prio!("IGNORE", "ROOT", "ERROR"))),
            opt(seq!("CYCLES", alt_prio!("BREAKUP", "ERROR"))),
            opt(seq!("GENERATE", "SPANTREE")),
            opt(seq!("CACHE", alt_prio!("ON", "OFF", "FORCE")))
        ),
        ")",
        "{",
        seq!(
            seq!(
                star(E::CDSAnnotation),
                opt_prio("KEY"),
                E::CDSPrefixedName,
                opt_prio(E::CDSAs)
            ),
            star(seq!(
                ",",
                seq!(
                    star(E::CDSAnnotation),
                    opt_prio("KEY"),
                    E::CDSPrefixedName,
                    opt_prio(E::CDSAs)
                )
            ))
        ),
        "}",
        opt(";")
    )
}

fn cds_define_projection() -> Rule {
    seq!(
        star(E::CDSAnnotation),
        opt("DEFINE"),
        opt("TRANSIENT"),
        opt("ROOT"),
        "VIEW",
        ver("7.55", opt("ENTITY")),
        E::CDSName,
        opt(E::CDSProviderContract),
        opt(E::CDSWithParameters),
        "AS PROJECTION ON",
        E::CDSName,
        opt(E::CDSParametersSelect),
        opt(E::CDSAs),
        star(alt_prio!(
            E::CDSAssociation,
            seq!(
                "REDEFINE",
                "ASSOCIATION",
                E::CDSPrefixedName,
                opt_prio(seq!(
                    "[",
                    opt_prio(seq!(
                        alt_prio!(
                            "EXACT ONE TO EXACT ONE",
                            "EXACT ONE TO MANY",
                            "EXACT ONE TO ONE",
                            "MANY TO EXACT ONE",
                            "MANY TO MANY",
                            "MANY TO ONE",
                            "ONE TO EXACT ONE",
                            "ONE TO MANY",
                            "ONE TO ONE",
                            "TO EXACT ONE",
                            "TO ONE",
                            "TO MANY",
                            E::CDSInteger,
                            "*"
                        ),
                        ":"
                    )),
                    E::CDSCondition,
                    "]"
                )),
                opt(E::CDSAs),
                "REDIRECTED",
                "TO",
                opt_prio(alt_prio!(seq!("COMPOSITION", "CHILD"), "PARENT")),
                E::CDSName
            )
        )),
        star_prio(seq!(
            "BIND",
            "ASPECT",
            E::CDSName,
            "(",
            seq!(
                E::CDSPrefixedName,
                "=",
                ">",
                alt_prio!(
                    seq!(E::CDSType, E::CDSString),
                    E::CDSString,
                    E::CDSPrefixedName
                )
            ),
            star_prio(seq!(
                ",",
                seq!(
                    E::CDSPrefixedName,
                    "=",
                    ">",
                    alt_prio!(
                        seq!(E::CDSType, E::CDSString),
                        E::CDSString,
                        E::CDSPrefixedName
                    )
                )
            )),
            ")",
            opt_prio(seq!("AS", E::CDSName))
        )),
        "{",
        seq!(E::CDSElement, star(seq!(",", E::CDSElement)), opt(",")),
        "}",
        opt(E::CDSWhere),
        opt(";")
    )
}

fn cds_define_table_entity() -> Rule {
    seq!(
        star(E::CDSAnnotation),
        opt("DEFINE"),
        opt("ROOT"),
        "TABLE",
        "ENTITY",
        E::CDSName,
        opt(E::CDSWithParameters),
        alt_prio!(
            seq!("{", plus(E::CDSTableField), "}"),
            seq!(
                "AS",
                "SELECT",
                "FROM",
                E::CDSSource,
                star(E::CDSJoin),
                star(alt_prio!(E::CDSComposition, E::CDSAssociation)),
                opt(seq!(
                    "{",
                    alt_prio!(
                        "*",
                        seq!(E::CDSElement, star(seq!(",", E::CDSElement)), opt(","))
                    ),
                    "}"
                )),
                opt_prio(E::CDSWhere)
            )
        ),
        opt(";")
    )
}

fn cds_define_table_function() -> Rule {
    seq!(
        star(E::CDSAnnotation),
        "DEFINE TABLE FUNCTION",
        E::CDSName,
        opt_prio(E::CDSWithParameters),
        "RETURNS {",
        plus(seq!(
            star(E::CDSAnnotation),
            opt_prio("KEY"),
            E::CDSName,
            ":",
            E::CDSType,
            star(E::CDSAnnotation),
            ";"
        )),
        "} IMPLEMENTED BY METHOD",
        seq!(E::CDSName, "=", ">", E::CDSName),
        opt(";")
    )
}

fn cds_define_view() -> Rule {
    seq!(
        star(E::CDSAnnotation),
        opt("DEFINE"),
        opt("ROOT"),
        opt("WRITABLE"),
        "VIEW",
        ver("7.55", opt("ENTITY")),
        E::CDSName,
        opt(seq!("(", E::CDSName, star(seq!(",", E::CDSName)), ")")),
        opt(E::CDSWithParameters),
        "AS",
        alt_prio!(
            seq!(
                E::CDSParenSelect,
                star(alt_prio!(
                    seq!(
                        "UNION",
                        opt("ALL"),
                        alt_prio!(E::CDSParenSelect, E::CDSSelect)
                    ),
                    seq!("EXCEPT", alt_prio!(E::CDSParenSelect, E::CDSSelect)),
                    seq!("INTERSECT", alt_prio!(E::CDSParenSelect, E::CDSSelect))
                ))
            ),
            seq!(
                E::CDSSelect,
                star(alt_prio!(
                    seq!(
                        "UNION",
                        opt("ALL"),
                        alt_prio!(E::CDSParenSelect, E::CDSSelect)
                    ),
                    seq!("EXCEPT", alt_prio!(E::CDSParenSelect, E::CDSSelect)),
                    seq!("INTERSECT", alt_prio!(E::CDSParenSelect, E::CDSSelect))
                ))
            )
        ),
        opt(seq!("WITH", "HIERARCHY", E::CDSName)),
        opt(";")
    )
}

fn cds_element() -> Rule {
    seq!(
        star_prio(E::CDSAnnotation),
        alt_prio!(
            seq!("VIRTUAL", E::CDSName, ":", E::CDSType),
            seq!(
                alt_prio!("KEY", "VIRTUAL"),
                alt_prio!(
                    seq!("$EXTENSION", ".", "*"),
                    seq!(
                        "INCLUDE",
                        stop_before_any(&["AS", "KEY", "VIRTUAL", "}"]),
                        E::CDSPrefixedName,
                        opt_prio(alt_prio!(
                            seq!(
                                "SIGNATURE ONLY",
                                opt_prio(seq!(
                                    "EXCLUDING",
                                    "{",
                                    seq!(E::CDSName, star_prio(seq!(",", E::CDSName))),
                                    "}"
                                ))
                            ),
                            seq!(
                                seq!(
                                    "EXCLUDING",
                                    "{",
                                    seq!(E::CDSName, star_prio(seq!(",", E::CDSName))),
                                    "}"
                                ),
                                opt_prio("SIGNATURE ONLY")
                            )
                        ))
                    ),
                    E::CDSArithmetics,
                    E::CDSAggregate,
                    E::CDSString,
                    E::CDSArithParen,
                    seq!(
                        E::CDSFunction,
                        plus_prio(seq!(
                            ".",
                            alt_prio!(E::CDSString, E::CDSName, "*"),
                            opt_prio(alt_prio!(E::CDSParametersSelect, E::CDSParameters))
                        ))
                    ),
                    E::CDSFunction,
                    E::CDSCast,
                    E::CDSCase,
                    seq!("(", E::CDSCase, ")"),
                    E::CDSInteger,
                    seq!(
                        E::CDSPrefixedName,
                        opt_prio(E::CDSAs),
                        opt_prio(alt_prio!(
                            seq!(
                                ": REDIRECTED TO",
                                opt_prio(alt_prio!("PARENT", "COMPOSITION CHILD")),
                                E::CDSName,
                                opt_prio(E::CDSParametersSelect)
                            ),
                            seq!(":", alt_prio!(E::CDSType, E::CDSName, "LOCALIZED"))
                        ))
                    )
                )
            ),
            alt_prio!(
                seq!("$EXTENSION", ".", "*"),
                seq!(
                    "INCLUDE",
                    stop_before_any(&["AS", "KEY", "VIRTUAL", "}"]),
                    E::CDSPrefixedName,
                    opt_prio(alt_prio!(
                        seq!(
                            "SIGNATURE ONLY",
                            opt_prio(seq!(
                                "EXCLUDING",
                                "{",
                                seq!(E::CDSName, star_prio(seq!(",", E::CDSName))),
                                "}"
                            ))
                        ),
                        seq!(
                            seq!(
                                "EXCLUDING",
                                "{",
                                seq!(E::CDSName, star_prio(seq!(",", E::CDSName))),
                                "}"
                            ),
                            opt_prio("SIGNATURE ONLY")
                        )
                    ))
                ),
                E::CDSArithmetics,
                E::CDSAggregate,
                E::CDSString,
                E::CDSArithParen,
                seq!(
                    E::CDSFunction,
                    plus_prio(seq!(
                        ".",
                        alt_prio!(E::CDSString, E::CDSName, "*"),
                        opt_prio(alt_prio!(E::CDSParametersSelect, E::CDSParameters))
                    ))
                ),
                E::CDSFunction,
                E::CDSCast,
                E::CDSCase,
                seq!("(", E::CDSCase, ")"),
                E::CDSInteger,
                seq!(
                    E::CDSPrefixedName,
                    opt_prio(E::CDSAs),
                    opt_prio(alt_prio!(
                        seq!(
                            ": REDIRECTED TO",
                            opt_prio(alt_prio!("PARENT", "COMPOSITION CHILD")),
                            E::CDSName,
                            opt_prio(E::CDSParametersSelect)
                        ),
                        seq!(":", alt_prio!(E::CDSType, E::CDSName, "LOCALIZED"))
                    ))
                )
            )
        ),
        opt_prio(E::CDSAs),
        star_prio(E::CDSAnnotation)
    )
}

fn cds_extend_view() -> Rule {
    alt!(
        seq!(
            star(E::CDSAnnotation),
            "EXTEND",
            alt!("ABSTRACT", "CUSTOM"),
            "ENTITY",
            E::CDSName,
            opt(E::CDSWithParameters),
            "WITH",
            seq!(
                "{",
                plus(alt!(
                    seq!(
                        star(E::CDSAnnotation),
                        opt_prio("KEY"),
                        E::CDSName,
                        ":",
                        E::CDSType,
                        ";"
                    ),
                    seq!(
                        star(E::CDSAnnotation),
                        E::CDSName,
                        ":",
                        alt!(E::CDSComposition, E::CDSAssociation),
                        ";"
                    )
                )),
                "}"
            ),
            opt(";")
        ),
        seq!(
            star(E::CDSAnnotation),
            "EXTEND VIEW",
            opt("ENTITY"),
            E::CDSName,
            "WITH",
            opt(E::CDSName),
            star(seq!(
                "REDEFINE",
                "ASSOCIATION",
                E::CDSPrefixedName,
                opt_prio(seq!(
                    "[",
                    opt_prio(seq!(
                        alt_prio!(
                            "EXACT ONE TO EXACT ONE",
                            "EXACT ONE TO MANY",
                            "EXACT ONE TO ONE",
                            "MANY TO EXACT ONE",
                            "MANY TO MANY",
                            "MANY TO ONE",
                            "ONE TO EXACT ONE",
                            "ONE TO MANY",
                            "ONE TO ONE",
                            "TO EXACT ONE",
                            "TO ONE",
                            "TO MANY",
                            E::CDSInteger,
                            "*"
                        ),
                        ":"
                    )),
                    E::CDSCondition,
                    "]"
                )),
                opt(E::CDSAs),
                "REDIRECTED",
                "TO",
                opt_prio(alt_prio!(seq!("COMPOSITION", "CHILD"), "PARENT")),
                E::CDSName
            )),
            star(alt_prio!(E::CDSAssociation, E::CDSComposition)),
            star_prio(seq!(
                "BIND",
                "ASPECT",
                E::CDSName,
                "(",
                seq!(
                    E::CDSPrefixedName,
                    "=",
                    ">",
                    alt_prio!(
                        seq!(E::CDSType, E::CDSString),
                        E::CDSString,
                        E::CDSPrefixedName
                    )
                ),
                star_prio(seq!(
                    ",",
                    seq!(
                        E::CDSPrefixedName,
                        "=",
                        ">",
                        alt_prio!(
                            seq!(E::CDSType, E::CDSString),
                            E::CDSString,
                            E::CDSPrefixedName
                        )
                    )
                )),
                ")",
                opt_prio(seq!("AS", E::CDSName))
            )),
            alt_prio!(
                seq!(
                    "(",
                    star(alt_prio!(E::CDSAssociation, E::CDSComposition)),
                    "{",
                    seq!(E::CDSElement, star(seq!(",", E::CDSElement)), opt(",")),
                    "}",
                    star(seq!(
                        "UNION",
                        opt("ALL"),
                        star(alt_prio!(E::CDSAssociation, E::CDSComposition)),
                        "{",
                        seq!(E::CDSElement, star(seq!(",", E::CDSElement)), opt(",")),
                        "}"
                    )),
                    ")"
                ),
                seq!(
                    "{",
                    alt_prio!(
                        "*",
                        seq!(E::CDSElement, star(seq!(",", E::CDSElement)), opt(","))
                    ),
                    "}"
                ),
                seq!(
                    "{",
                    seq!(
                        E::CDSName,
                        opt_prio(seq!(".", E::CDSName)),
                        opt_prio(E::CDSAs)
                    ),
                    star(seq!(
                        ",",
                        seq!(
                            E::CDSName,
                            opt_prio(seq!(".", E::CDSName)),
                            opt_prio(E::CDSAs)
                        )
                    )),
                    "}"
                )
            ),
            seq!(
                opt_prio(E::CDSGroupBy),
                star(seq!(
                    "UNION",
                    opt("ALL"),
                    alt_prio!(
                        seq!(
                            "(",
                            star(alt_prio!(E::CDSAssociation, E::CDSComposition)),
                            "{",
                            seq!(E::CDSElement, star(seq!(",", E::CDSElement)), opt(",")),
                            "}",
                            star(seq!(
                                "UNION",
                                opt("ALL"),
                                star(alt_prio!(E::CDSAssociation, E::CDSComposition)),
                                "{",
                                seq!(E::CDSElement, star(seq!(",", E::CDSElement)), opt(",")),
                                "}"
                            )),
                            ")"
                        ),
                        seq!(
                            star(alt_prio!(E::CDSAssociation, E::CDSComposition)),
                            "{",
                            alt_prio!(
                                "*",
                                seq!(E::CDSElement, star(seq!(",", E::CDSElement)), opt(","))
                            ),
                            "}",
                            opt_prio(E::CDSGroupBy)
                        )
                    )
                ))
            ),
            opt(";")
        )
    )
}

fn cds_function() -> Rule {
    alt_prio!(
        seq!(
            "SUBSTRING",
            "(",
            E::CDSFunctionInput,
            ",",
            E::CDSFunctionInput,
            ",",
            E::CDSFunctionInput,
            ")"
        ),
        seq!(
            "COALESCE",
            "(",
            E::CDSFunctionInput,
            ",",
            E::CDSFunctionInput,
            ")"
        ),
        seq!(
            "TSTMP_TO_DATS",
            "(",
            E::CDSFunctionInput,
            ",",
            E::CDSFunctionInput,
            ",",
            E::CDSFunctionInput,
            ",",
            E::CDSFunctionInput,
            ")"
        ),
        seq!(
            "CONCAT",
            "(",
            E::CDSFunctionInput,
            ",",
            E::CDSFunctionInput,
            ")"
        ),
        seq!(
            "TSTMP_TO_TIMS",
            "(",
            E::CDSFunctionInput,
            ",",
            E::CDSFunctionInput,
            ",",
            E::CDSFunctionInput,
            ",",
            E::CDSFunctionInput,
            ")"
        ),
        seq!("UPPER", "(", E::CDSFunctionInput, ")"),
        seq!("LOWER", "(", E::CDSFunctionInput, ")"),
        seq!("ABS", "(", E::CDSFunctionInput, ")"),
        seq!("CEIL", "(", E::CDSFunctionInput, ")"),
        seq!("FLOOR", "(", E::CDSFunctionInput, ")"),
        seq!(
            "ROUND",
            "(",
            E::CDSFunctionInput,
            ",",
            E::CDSFunctionInput,
            ")"
        ),
        seq!(
            "DIV",
            "(",
            E::CDSFunctionInput,
            ",",
            E::CDSFunctionInput,
            ")"
        ),
        seq!(
            "DIVISION",
            "(",
            E::CDSFunctionInput,
            ",",
            E::CDSFunctionInput,
            ",",
            E::CDSFunctionInput,
            ")"
        ),
        seq!(
            "CONCAT_WITH_SPACE",
            "(",
            E::CDSFunctionInput,
            ",",
            E::CDSFunctionInput,
            ",",
            E::CDSFunctionInput,
            ")"
        ),
        seq!("DATS_IS_VALID", "(", E::CDSFunctionInput, ")"),
        seq!("TIMS_IS_VALID", "(", E::CDSFunctionInput, ")"),
        seq!(
            "DATS_DAYS_BETWEEN",
            "(",
            E::CDSFunctionInput,
            ",",
            E::CDSFunctionInput,
            ")"
        ),
        seq!(
            "TSTMP_ADD_SECONDS",
            "(",
            E::CDSFunctionInput,
            ",",
            E::CDSFunctionInput,
            ",",
            E::CDSFunctionInput,
            ")"
        ),
        seq!(
            "TSTMP_SECONDS_BETWEEN",
            "(",
            E::CDSFunctionInput,
            ",",
            E::CDSFunctionInput,
            ",",
            E::CDSFunctionInput,
            ")"
        ),
        seq!("TSTMP_CURRENT_UTCTIMESTAMP", "(", ")"),
        seq!("TSTMP_IS_VALID", "(", E::CDSFunctionInput, ")"),
        seq!("UTCL_CURRENT", "(", ")"),
        seq!(
            "ABAP_SYSTEM_TIMEZONE",
            "(",
            E::CDSFunctionInput,
            ",",
            E::CDSFunctionInput,
            ")"
        ),
        seq!(
            "ABAP_USER_TIMEZONE",
            "(",
            E::CDSFunctionInput,
            ",",
            E::CDSFunctionInput,
            ",",
            E::CDSFunctionInput,
            ")"
        ),
        seq!("BINTOHEX", "(", E::CDSFunctionInput, ")"),
        seq!("HEXTOBIN", "(", E::CDSFunctionInput, ")"),
        seq!(
            "DATS_ADD_DAYS",
            "(",
            E::CDSFunctionInput,
            ",",
            E::CDSFunctionInput,
            ",",
            E::CDSFunctionInput,
            ")"
        ),
        seq!(
            "DATS_ADD_MONTHS",
            "(",
            E::CDSFunctionInput,
            ",",
            E::CDSFunctionInput,
            ",",
            E::CDSFunctionInput,
            ")"
        ),
        seq!(
            "TSTMP_TO_DST",
            "(",
            E::CDSFunctionInput,
            ",",
            E::CDSFunctionInput,
            ",",
            E::CDSFunctionInput,
            ",",
            E::CDSFunctionInput,
            ")"
        ),
        seq!(
            "DATS_TIMS_TO_TSTMP",
            "(",
            E::CDSFunctionInput,
            ",",
            E::CDSFunctionInput,
            ",",
            E::CDSFunctionInput,
            ",",
            E::CDSFunctionInput,
            ",",
            E::CDSFunctionInput,
            ")"
        ),
        seq!(
            "MOD",
            "(",
            E::CDSFunctionInput,
            ",",
            E::CDSFunctionInput,
            ")"
        ),
        seq!(
            "LEFT",
            "(",
            E::CDSFunctionInput,
            ",",
            E::CDSFunctionInput,
            ")"
        ),
        seq!(
            "RIGHT",
            "(",
            E::CDSFunctionInput,
            ",",
            E::CDSFunctionInput,
            ")"
        ),
        seq!(
            "LPAD",
            "(",
            E::CDSFunctionInput,
            ",",
            E::CDSFunctionInput,
            ",",
            E::CDSFunctionInput,
            ")"
        ),
        seq!(
            "RPAD",
            "(",
            E::CDSFunctionInput,
            ",",
            E::CDSFunctionInput,
            ",",
            E::CDSFunctionInput,
            ")"
        ),
        seq!(
            "INSTR",
            "(",
            E::CDSFunctionInput,
            ",",
            E::CDSFunctionInput,
            ")"
        ),
        seq!("LENGTH", "(", E::CDSFunctionInput, ")"),
        seq!(
            "LTRIM",
            "(",
            E::CDSFunctionInput,
            ",",
            E::CDSFunctionInput,
            ")"
        ),
        seq!(
            "RTRIM",
            "(",
            E::CDSFunctionInput,
            ",",
            E::CDSFunctionInput,
            ")"
        ),
        seq!(
            "REPLACE",
            "(",
            E::CDSFunctionInput,
            ",",
            E::CDSFunctionInput,
            ",",
            E::CDSFunctionInput,
            ")"
        ),
        seq!(
            "UNIT_CONVERSION",
            "(",
            seq!(
                seq!(E::CDSName, "=", ">", E::CDSFunctionInput),
                star_prio(seq!(",", seq!(E::CDSName, "=", ">", E::CDSFunctionInput)))
            ),
            ")"
        ),
        seq!(
            "CURRENCY_CONVERSION",
            "(",
            seq!(
                seq!(E::CDSName, "=", ">", E::CDSFunctionInput),
                star_prio(seq!(",", seq!(E::CDSName, "=", ">", E::CDSFunctionInput)))
            ),
            ")"
        ),
        seq!(
            "DECIMAL_SHIFT",
            "(",
            seq!(
                seq!(E::CDSName, "=", ">", E::CDSFunctionInput),
                star_prio(seq!(",", seq!(E::CDSName, "=", ">", E::CDSFunctionInput)))
            ),
            ")"
        ),
        seq!(
            "FLTP_TO_DEC",
            "(",
            E::CDSFunctionInput,
            "AS",
            E::CDSType,
            ")"
        ),
        seq!(
            "RATIO_OF",
            "(",
            seq!(
                seq!(E::CDSName, "=", ">", E::CDSFunctionInput),
                star_prio(seq!(",", seq!(E::CDSName, "=", ">", E::CDSFunctionInput)))
            ),
            ")"
        ),
        seq!(
            "REPLACE_REGEXPR",
            "(",
            seq!(
                seq!(E::CDSName, "=", ">", E::CDSFunctionInput),
                star_prio(seq!(",", seq!(E::CDSName, "=", ">", E::CDSFunctionInput)))
            ),
            ")"
        ),
        seq!(
            "MATCHES_REGEXPR",
            "(",
            seq!(
                seq!(E::CDSName, "=", ">", E::CDSFunctionInput),
                star_prio(seq!(",", seq!(E::CDSName, "=", ">", E::CDSFunctionInput)))
            ),
            ")"
        ),
        seq!(
            "OCCURRENCES_REGEXPR",
            "(",
            seq!(
                seq!(E::CDSName, "=", ">", E::CDSFunctionInput),
                star_prio(seq!(",", seq!(E::CDSName, "=", ">", E::CDSFunctionInput)))
            ),
            ")"
        ),
        seq!(
            "SUBSTR_REGEXPR",
            "(",
            seq!(
                seq!(E::CDSName, "=", ">", E::CDSFunctionInput),
                star_prio(seq!(",", seq!(E::CDSName, "=", ">", E::CDSFunctionInput)))
            ),
            ")"
        ),
        seq!(
            "LOCATE_REGEXPR",
            "(",
            seq!(
                seq!(E::CDSName, "=", ">", E::CDSFunctionInput),
                star_prio(seq!(",", seq!(E::CDSName, "=", ">", E::CDSFunctionInput)))
            ),
            ")"
        ),
        seq!("CURR_TO_DECFLOAT_AMOUNT", "(", E::CDSFunctionInput, ")"),
        seq!("SQRT", "(", E::CDSFunctionInput, ")"),
        seq!(
            re(r"(?i)^[A-Z][A-Z0-9]*(?:_[A-Z0-9]+)+$"),
            "(",
            opt_prio(alt_prio!(
                seq!(
                    alt_prio!(
                        seq!(
                            E::CDSName,
                            "=",
                            ">",
                            alt_prio!(
                                seq!("INTERVAL", E::CDSFunctionInput, E::CDSName),
                                E::CDSFunctionInput,
                                E::CDSName
                            )
                        ),
                        seq!(
                            E::CDSName,
                            ":",
                            alt_prio!(
                                seq!("INTERVAL", E::CDSFunctionInput, E::CDSName),
                                E::CDSFunctionInput,
                                E::CDSName
                            )
                        )
                    ),
                    star_prio(seq!(
                        ",",
                        alt_prio!(
                            seq!(
                                E::CDSName,
                                "=",
                                ">",
                                alt_prio!(
                                    seq!("INTERVAL", E::CDSFunctionInput, E::CDSName),
                                    E::CDSFunctionInput,
                                    E::CDSName
                                )
                            ),
                            seq!(
                                E::CDSName,
                                ":",
                                alt_prio!(
                                    seq!("INTERVAL", E::CDSFunctionInput, E::CDSName),
                                    E::CDSFunctionInput,
                                    E::CDSName
                                )
                            )
                        )
                    ))
                ),
                seq!(
                    E::CDSFunctionInput,
                    star_prio(seq!(",", E::CDSFunctionInput))
                )
            )),
            ")"
        )
    )
}

fn cds_function_input() -> Rule {
    alt_prio!(
        seq!(
            E::CDSName,
            "=",
            ">",
            alt_prio!(
                E::CDSArithmetics,
                E::CDSArithParen,
                E::CDSAggregate,
                E::CDSCast,
                E::CDSFunction,
                E::CDSCase,
                E::CDSString,
                E::CDSPrefixedName,
                E::CDSInteger
            )
        ),
        alt_prio!(
            E::CDSArithmetics,
            E::CDSArithParen,
            E::CDSAggregate,
            E::CDSCast,
            E::CDSFunction,
            E::CDSCase,
            E::CDSString,
            E::CDSPrefixedName,
            E::CDSInteger
        )
    )
}

fn cds_group_by() -> Rule {
    seq!(
        "GROUP BY",
        E::CDSPrefixedName,
        star(seq!(",", E::CDSPrefixedName))
    )
}

fn cds_having() -> Rule {
    seq!("HAVING", E::CDSCondition)
}

fn cds_integer() -> Rule {
    seq!(
        opt("-"),
        alt_prio!(
            seq!(
                re(r"^\d+$"),
                ".",
                re(r"(?i)^\d+E$"),
                alt_prio!("+", "-"),
                re(r"^\d+$")
            ),
            // 1E-2 and 1E99, in SAP's views
            seq!(re(r"(?i)^\d+E$"), alt_prio!("+", "-"), re(r"^\d+$")),
            re(r"(?i)^\d+E\d+$"),
            seq!(re(r"^\d+$"), ".", re(r"^\d+$")),
            re(r"^\d+$")
        )
    )
}

fn cds_join() -> Rule {
    seq!(
        opt_prio(alt_prio!(
            seq!(
                "LEFT OUTER",
                alt_prio!("EXACT ONE", "ONE", "MANY"),
                "TO",
                alt_prio!("EXACT ONE", "ONE", "MANY")
            ),
            seq!("LEFT OUTER TO", alt_prio!("EXACT ONE", "ONE", "MANY")),
            "LEFT OUTER",
            seq!(
                "INNER",
                alt_prio!("EXACT ONE", "ONE", "MANY"),
                "TO",
                alt_prio!("EXACT ONE", "ONE", "MANY")
            ),
            seq!("INNER TO", alt_prio!("EXACT ONE", "ONE", "MANY")),
            "INNER",
            "CROSS",
            "RIGHT OUTER"
        )),
        "JOIN",
        alt_prio!(
            seq!(
                "(",
                E::CDSSource,
                star(seq!(
                    opt_prio(alt_prio!(
                        seq!(
                            "LEFT OUTER",
                            alt_prio!("EXACT ONE", "ONE", "MANY"),
                            "TO",
                            alt_prio!("EXACT ONE", "ONE", "MANY")
                        ),
                        seq!("LEFT OUTER TO", alt_prio!("EXACT ONE", "ONE", "MANY")),
                        "LEFT OUTER",
                        seq!(
                            "INNER",
                            alt_prio!("EXACT ONE", "ONE", "MANY"),
                            "TO",
                            alt_prio!("EXACT ONE", "ONE", "MANY")
                        ),
                        seq!("INNER TO", alt_prio!("EXACT ONE", "ONE", "MANY")),
                        "INNER",
                        "CROSS",
                        "RIGHT OUTER"
                    )),
                    "JOIN",
                    alt_prio!(
                        seq!("(", seq!(E::CDSSource, "ON", E::CDSCondition), ")"),
                        seq!(E::CDSSource, "ON", E::CDSCondition)
                    )
                )),
                ")",
                "ON",
                E::CDSCondition
            ),
            seq!(
                E::CDSSource,
                star(seq!(
                    opt_prio(alt_prio!(
                        seq!(
                            "LEFT OUTER",
                            alt_prio!("EXACT ONE", "ONE", "MANY"),
                            "TO",
                            alt_prio!("EXACT ONE", "ONE", "MANY")
                        ),
                        seq!("LEFT OUTER TO", alt_prio!("EXACT ONE", "ONE", "MANY")),
                        "LEFT OUTER",
                        seq!(
                            "INNER",
                            alt_prio!("EXACT ONE", "ONE", "MANY"),
                            "TO",
                            alt_prio!("EXACT ONE", "ONE", "MANY")
                        ),
                        seq!("INNER TO", alt_prio!("EXACT ONE", "ONE", "MANY")),
                        "INNER",
                        "CROSS",
                        "RIGHT OUTER"
                    )),
                    "JOIN",
                    E::CDSSource,
                    "ON",
                    E::CDSCondition
                )),
                "ON",
                E::CDSCondition
            ),
            alt_prio!(
                seq!("(", seq!(E::CDSSource, "ON", E::CDSCondition), ")"),
                seq!(E::CDSSource, "ON", E::CDSCondition)
            ),
            E::CDSSource
        )
    )
}

fn cds_name() -> Rule {
    alt_prio!(
        re(r#"^"(?:[^"]|"")*"$"#),
        seq!(opt_prio(":"), name()),
        re(r"(?i)^\$(projection|parameters|session|node|extension)$"),
        re(r"^#\w+$")
    )
}

/// The words SAP's general syntax rules of CDS forbid as names
/// (`abencds_general_syntax_rules`).
pub(crate) const RESERVED: &[&str] = &[
    "ALL",
    "AND",
    "AS",
    "ASSOCIATION",
    "AVG",
    "CASE",
    "CAST",
    "COUNT",
    "CROSS",
    "DISTINCT",
    "EXISTS",
    "EXTEND",
    "FROM",
    "FULL",
    "GROUP",
    "HAVING",
    "INNER",
    "JOIN",
    "KEY",
    "LEFT",
    "MAX",
    "MIN",
    "NOT",
    "NULL",
    "ON",
    "OR",
    "ORDER",
    "RIGHT",
    "SELECT",
    "SUM",
    "UNION",
    "VIEW",
    "WHEN",
    "WHERE",
];

/// A name: letters, digits, underscores, starting with a letter or an
/// underscore, or with a namespace, `/ns/name`, which the lexer gives as one
/// token; not a reserved word (`abencds_general_syntax_rules`).
fn name() -> Rule {
    alt_prio!(re(r"^/\w+/\w+$"), re_not(r"^[A-Za-z_]\w*$", RESERVED))
}

/// A part of an annotation name: letters, digits and underscores, starting
/// with a letter (`abencds_annotations_syntax_name`).
fn annotation_part() -> Rule {
    re(r"^[A-Za-z]\w*$")
}

fn cds_parameters() -> Rule {
    seq!(
        "[",
        re(r"\d+"),
        ":",
        seq!(E::CDSName, opt(seq!(".", E::CDSName))),
        "=",
        alt!(seq!(E::CDSName, opt(seq!(".", E::CDSName))), E::CDSString),
        star(seq!(
            "AND",
            seq!(E::CDSName, opt(seq!(".", E::CDSName))),
            "=",
            alt!(seq!(E::CDSName, opt(seq!(".", E::CDSName))), E::CDSString)
        )),
        "]"
    )
}

fn cds_parameters_select() -> Rule {
    seq!(
        "(",
        alt_prio!(
            seq!(
                seq!(E::CDSName, opt(seq!(".", E::CDSName))),
                ":",
                alt_prio!(
                    seq!(E::CDSType, E::CDSString),
                    E::CDSInteger,
                    seq!(E::CDSName, opt(seq!(".", E::CDSName))),
                    E::CDSString
                ),
                opt_prio("DEFAULT")
            ),
            seq!(
                seq!(E::CDSName, opt(seq!(".", E::CDSName))),
                "=",
                ">",
                alt_prio!(
                    seq!(E::CDSType, E::CDSString),
                    E::CDSInteger,
                    seq!(E::CDSName, opt(seq!(".", E::CDSName))),
                    E::CDSString
                )
            )
        ),
        star(seq!(
            ",",
            alt_prio!(
                seq!(
                    seq!(E::CDSName, opt(seq!(".", E::CDSName))),
                    ":",
                    alt_prio!(
                        seq!(E::CDSType, E::CDSString),
                        E::CDSInteger,
                        seq!(E::CDSName, opt(seq!(".", E::CDSName))),
                        E::CDSString
                    ),
                    opt_prio("DEFAULT")
                ),
                seq!(
                    seq!(E::CDSName, opt(seq!(".", E::CDSName))),
                    "=",
                    ">",
                    alt_prio!(
                        seq!(E::CDSType, E::CDSString),
                        E::CDSInteger,
                        seq!(E::CDSName, opt(seq!(".", E::CDSName))),
                        E::CDSString
                    )
                )
            )
        )),
        ")"
    )
}

fn cds_paren_select() -> Rule {
    seq!(
        "(",
        alt_prio!(
            seq!(
                E::CDSParenSelect,
                star(alt_prio!(
                    seq!(
                        "UNION",
                        opt("ALL"),
                        alt_prio!(E::CDSParenSelect, E::CDSSelect)
                    ),
                    seq!("EXCEPT", alt_prio!(E::CDSParenSelect, E::CDSSelect)),
                    seq!("INTERSECT", alt_prio!(E::CDSParenSelect, E::CDSSelect))
                ))
            ),
            seq!(
                E::CDSSelect,
                star(alt_prio!(
                    seq!(
                        "UNION",
                        opt("ALL"),
                        alt_prio!(E::CDSParenSelect, E::CDSSelect)
                    ),
                    seq!("EXCEPT", alt_prio!(E::CDSParenSelect, E::CDSSelect)),
                    seq!("INTERSECT", alt_prio!(E::CDSParenSelect, E::CDSSelect))
                ))
            )
        ),
        ")"
    )
}

fn cds_prefixed_name() -> Rule {
    seq!(
        E::CDSName,
        opt_prio(alt_prio!(E::CDSParameters, E::CDSParametersSelect)),
        opt_prio(alt_prio!(
            seq!(
                "[",
                alt_prio!(E::CDSInteger, "*"),
                ":",
                alt_prio!("LEFT OUTER", "INNER", "CROSS"),
                "WHERE",
                E::CDSCondition,
                "]"
            ),
            seq!(
                "[",
                alt_prio!("LEFT OUTER", "INNER", "CROSS"),
                "WHERE",
                E::CDSCondition,
                "]"
            ),
            seq!(
                "[",
                alt_prio!(E::CDSInteger, "*"),
                ":",
                alt_prio!("LEFT OUTER", "INNER", "CROSS"),
                "]"
            ),
            seq!("[", alt_prio!("LEFT OUTER", "INNER", "CROSS"), "]"),
            seq!(
                "[",
                E::CDSInteger,
                "TO",
                E::CDSInteger,
                opt_prio(seq!(":", E::CDSCondition)),
                "]"
            ),
            seq!(
                "[",
                seq!(
                    E::CDSInteger,
                    alt_prio!("TO EXACT ONE", "TO ONE", "TO MANY")
                ),
                opt_prio(seq!(":", E::CDSCondition)),
                "]"
            ),
            seq!(
                "[",
                alt_prio!(
                    "EXACT ONE TO EXACT ONE",
                    "EXACT ONE TO MANY",
                    "EXACT ONE TO ONE",
                    "MANY TO EXACT ONE",
                    "MANY TO MANY",
                    "MANY TO ONE",
                    "ONE TO EXACT ONE",
                    "ONE TO MANY",
                    "ONE TO ONE",
                    "TO EXACT ONE",
                    "TO ONE",
                    "TO MANY"
                ),
                opt_prio(seq!(
                    ":",
                    alt_prio!(
                        seq!(
                            alt_prio!("LEFT OUTER", "INNER", "CROSS"),
                            "WHERE",
                            E::CDSCondition
                        ),
                        alt_prio!("LEFT OUTER", "INNER", "CROSS"),
                        E::CDSCondition
                    )
                )),
                "]"
            ),
            seq!(
                "[",
                seq!(E::CDSInteger, ".", ".", alt_prio!(E::CDSInteger, "*")),
                ":",
                E::CDSCondition,
                "]"
            ),
            seq!(
                "[",
                seq!(E::CDSInteger, ".", ".", alt_prio!(E::CDSInteger, "*")),
                "]"
            ),
            seq!(
                "[",
                alt_prio!(E::CDSInteger, "*"),
                ":",
                E::CDSCondition,
                "]"
            ),
            seq!("[", alt_prio!(E::CDSInteger, "*"), "]"),
            seq!("[", E::CDSCondition, "]")
        )),
        star(seq!(
            ".",
            alt_prio!(E::CDSString, E::CDSInteger, E::CDSName, "*"),
            opt_prio(alt_prio!(E::CDSParametersSelect, E::CDSParameters)),
            opt_prio(alt_prio!(
                seq!(
                    "[",
                    alt_prio!(E::CDSInteger, "*"),
                    ":",
                    alt_prio!("LEFT OUTER", "INNER", "CROSS"),
                    "WHERE",
                    E::CDSCondition,
                    "]"
                ),
                seq!(
                    "[",
                    alt_prio!("LEFT OUTER", "INNER", "CROSS"),
                    "WHERE",
                    E::CDSCondition,
                    "]"
                ),
                seq!(
                    "[",
                    alt_prio!(E::CDSInteger, "*"),
                    ":",
                    alt_prio!("LEFT OUTER", "INNER", "CROSS"),
                    "]"
                ),
                seq!("[", alt_prio!("LEFT OUTER", "INNER", "CROSS"), "]"),
                seq!(
                    "[",
                    E::CDSInteger,
                    "TO",
                    E::CDSInteger,
                    opt_prio(seq!(":", E::CDSCondition)),
                    "]"
                ),
                seq!(
                    "[",
                    seq!(
                        E::CDSInteger,
                        alt_prio!("TO EXACT ONE", "TO ONE", "TO MANY")
                    ),
                    opt_prio(seq!(":", E::CDSCondition)),
                    "]"
                ),
                seq!(
                    "[",
                    alt_prio!(
                        "EXACT ONE TO EXACT ONE",
                        "EXACT ONE TO MANY",
                        "EXACT ONE TO ONE",
                        "MANY TO EXACT ONE",
                        "MANY TO MANY",
                        "MANY TO ONE",
                        "ONE TO EXACT ONE",
                        "ONE TO MANY",
                        "ONE TO ONE",
                        "TO EXACT ONE",
                        "TO ONE",
                        "TO MANY"
                    ),
                    opt_prio(seq!(
                        ":",
                        alt_prio!(
                            seq!(
                                alt_prio!("LEFT OUTER", "INNER", "CROSS"),
                                "WHERE",
                                E::CDSCondition
                            ),
                            alt_prio!("LEFT OUTER", "INNER", "CROSS"),
                            E::CDSCondition
                        )
                    )),
                    "]"
                ),
                seq!(
                    "[",
                    seq!(E::CDSInteger, ".", ".", alt_prio!(E::CDSInteger, "*")),
                    ":",
                    E::CDSCondition,
                    "]"
                ),
                seq!(
                    "[",
                    seq!(E::CDSInteger, ".", ".", alt_prio!(E::CDSInteger, "*")),
                    "]"
                ),
                seq!(
                    "[",
                    alt_prio!(E::CDSInteger, "*"),
                    ":",
                    E::CDSCondition,
                    "]"
                ),
                seq!("[", alt_prio!(E::CDSInteger, "*"), "]"),
                seq!("[", E::CDSCondition, "]")
            ))
        ))
    )
}

fn cds_provider_contract() -> Rule {
    seq!(
        "PROVIDER CONTRACT",
        alt!(
            "TRANSACTIONAL_QUERY",
            "TRANSACTIONAL_INTERFACE",
            "ANALYTICAL_QUERY",
            "SQL_QUERY",
            "STRUCTURED_QUERY"
        )
    )
}

fn cds_relation() -> Rule {
    seq!(name(), opt(E::CDSParametersSelect), opt(E::CDSAs))
}

fn cds_select() -> Rule {
    seq!(
        "SELECT",
        opt_prio("DISTINCT"),
        opt(alt_prio!(
            "*",
            seq!(
                stop_before_any(&["FROM"]),
                star(seq!(E::CDSElement, ",")),
                E::CDSElement
            )
        )),
        "FROM",
        E::CDSSource,
        star(E::CDSJoin),
        star(alt_prio!(E::CDSComposition, E::CDSAssociation)),
        star_prio(seq!(
            "BIND",
            "ASPECT",
            E::CDSName,
            "(",
            seq!(
                E::CDSPrefixedName,
                "=",
                ">",
                alt_prio!(
                    seq!(E::CDSType, E::CDSString),
                    E::CDSString,
                    E::CDSPrefixedName
                )
            ),
            star_prio(seq!(
                ",",
                seq!(
                    E::CDSPrefixedName,
                    "=",
                    ">",
                    alt_prio!(
                        seq!(E::CDSType, E::CDSString),
                        E::CDSString,
                        E::CDSPrefixedName
                    )
                )
            )),
            ")",
            opt_prio(seq!("AS", E::CDSName))
        )),
        opt(seq!(
            "{",
            alt_prio!(
                "*",
                seq!(E::CDSElement, star(seq!(",", E::CDSElement)), opt(","))
            ),
            "}"
        )),
        opt_prio(E::CDSWhere),
        opt_prio(E::CDSGroupBy),
        opt_prio(E::CDSHaving),
        opt_prio(alt_prio!(
            seq!(
                "UNION",
                opt_prio("ALL"),
                alt_prio!(seq!("(", E::CDSSelect, ")"), E::CDSSelect)
            ),
            seq!(
                "EXCEPT",
                alt_prio!(seq!("(", E::CDSSelect, ")"), E::CDSSelect)
            ),
            seq!(
                "INTERSECT",
                alt_prio!(seq!("(", E::CDSSelect, ")"), E::CDSSelect)
            )
        ))
    )
}

// an alias without AS only in DDIC-based views: a rule of view entities,
// checked in crate::cds
fn cds_source() -> Rule {
    alt_prio!(
        seq!(
            "(",
            alt_prio!(
                E::CDSSource,
                seq!(
                    E::CDSPrefixedName,
                    opt_prio(E::CDSParametersSelect),
                    opt_prio(seq!("[", E::CDSCondition, "]")),
                    opt(alt_prio!(E::CDSAs, E::CDSName))
                )
            ),
            star(E::CDSJoin),
            ")"
        ),
        seq!(E::CDSFunction, opt(alt_prio!(E::CDSAs, E::CDSName))),
        seq!(
            E::CDSPrefixedName,
            opt_prio(E::CDSParametersSelect),
            opt_prio(seq!("[", E::CDSCondition, "]")),
            opt(alt_prio!(E::CDSAs, E::CDSName))
        )
    )
}

fn cds_string() -> Rule {
    alt_prio!(
        seq!(
            "ABAP",
            ".",
            re(
                r"(?i)^(?:int[1-9]|int8|sstring|sstr|char|numc|dats|datn|tims|timn|utcl|utclong|fltp|decfloat\d+|dec|string|rawstring|rstr|raw|xstring|clnt|lang|unit|cuky|curr|quan|geom_ewkb|d34n|d16n|d34d|d16d|d34s|d16s|d34r|d16r|d|t|p|n|c|x|f)$"
            ),
            opt_prio(seq!("(", re(r"^\d+$"), ")")),
            re(r"^'(?:[^'\\]|''|\\'|\\\\|\\[^'])*'$")
        ),
        re(r"^'(?:[^'\\]|''|\\'|\\\\|\\[^'])*'$")
    )
}

fn cds_table_field() -> Rule {
    seq!(
        star(E::CDSAnnotation),
        alt_prio!(
            seq!(
                "INCLUDE",
                E::CDSPrefixedName,
                opt_prio(alt_prio!(
                    seq!(
                        "SIGNATURE ONLY",
                        opt_prio(seq!(
                            "EXCLUDING",
                            "{",
                            seq!(E::CDSName, star(seq!(",", E::CDSName))),
                            "}"
                        ))
                    ),
                    seq!(
                        seq!(
                            "EXCLUDING",
                            "{",
                            seq!(E::CDSName, star(seq!(",", E::CDSName))),
                            "}"
                        ),
                        opt_prio("SIGNATURE ONLY")
                    )
                ))
            ),
            seq!(E::CDSName, ":", alt!(E::CDSComposition, E::CDSAssociation)),
            seq!(
                opt_prio("KEY"),
                E::CDSName,
                ":",
                E::CDSType,
                opt_prio(alt!("NOT NULL", "NULL"))
            )
        ),
        ";"
    )
}

fn cds_type() -> Rule {
    seq!(
        name(),
        opt_prio(seq!(".", name())),
        opt_prio(seq!("(", re(r"\d+"), opt_prio(seq!(",", re(r"\d+"))), ")"))
    )
}

fn cds_where() -> Rule {
    seq!("WHERE", E::CDSCondition)
}

fn cds_with_parameters() -> Rule {
    seq!(
        "WITH PARAMETERS",
        seq!(
            star(E::CDSAnnotation),
            E::CDSName,
            ":",
            E::CDSType,
            opt(seq!(
                "EXTERNAL",
                "NAME",
                alt_prio!(E::CDSName, re(r#"^"(?:[^"]|"")*"$"#))
            )),
            star(E::CDSAnnotation)
        ),
        star(seq!(
            ",",
            seq!(
                star(E::CDSAnnotation),
                E::CDSName,
                ":",
                E::CDSType,
                opt(seq!(
                    "EXTERNAL",
                    "NAME",
                    alt_prio!(E::CDSName, re(r#"^"(?:[^"]|"")*"$"#))
                )),
                star(E::CDSAnnotation)
            )
        ))
    )
}
