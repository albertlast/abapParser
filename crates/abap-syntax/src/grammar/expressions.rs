//! Expressions of the statement grammars.
//!
//! Generated from the statement matchers of abaplint 2.120.105
//! (<https://github.com/abaplint/abaplint>, MIT, Copyright (c) 2015 Lars Hvam)
//! by tools/gen-grammar.mjs, maintained by hand since.

use super::dsl::*;

/// The expressions statement grammars are built of, named like abaplint's.
#[allow(clippy::enum_variant_names)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum E {
    Abstract,
    AbstractMethods,
    AndReturn,
    ArithOperator,
    Arrow,
    ArrowOrDash,
    AssignSource,
    AttributeChain,
    AttributeName,
    BehaviorDefinitionName,
    BlockName,
    CallTransformationOptions,
    CallTransformationParameters,
    Cast,
    ClassFinal,
    ClassFriends,
    ClassGlobal,
    ClassName,
    Color,
    Compare,
    CompareOperator,
    ComponentChain,
    ComponentChainSimple,
    ComponentCompare,
    ComponentCompareSimple,
    ComponentCompareSingle,
    ComponentCond,
    ComponentCondSub,
    ComponentName,
    ConcatenatedConstant,
    Cond,
    CondBody,
    CondSub,
    Constant,
    ConstantFieldLength,
    ConstantString,
    ConvBody,
    CorrespondingBody,
    CorrespondingBodyBase,
    CorrespondingBodyMapping,
    DatabaseConnection,
    DatabaseTable,
    DatabaseTableNoDynamic,
    DataDefinition,
    Decimals,
    Default,
    DefinitionName,
    Dereference,
    Destination,
    Dynamic,
    EMLEntityPath,
    EntityAssociation,
    EventHandler,
    EventName,
    ExceptionName,
    Field,
    FieldAll,
    FieldAssignment,
    FieldChain,
    FieldLength,
    FieldOffset,
    FieldSub,
    FieldSymbol,
    FilterBody,
    FinalMethods,
    FindType,
    For,
    FormChanging,
    FormName,
    FormParam,
    FormParamName,
    FormParamType,
    FormRaising,
    FormTables,
    FormUsing,
    FSTarget,
    FunctionExporting,
    FunctionExportingParameter,
    FunctionName,
    FunctionParameters,
    IncludeName,
    InlineData,
    InlineField,
    InlineFieldDefinition,
    InlineFS,
    InlineLoopDefinition,
    Integer,
    InterfaceName,
    KernelId,
    Language,
    Length,
    Let,
    LOBHandle,
    LoopGroupBy,
    LoopGroupByComponent,
    LoopGroupByTarget,
    LoopSource,
    LoopTarget,
    MacroName,
    MessageClass,
    MessageNumber,
    MessageSource,
    MessageSourceSource,
    MessageTypeAndNumber,
    MethodCall,
    MethodCallBody,
    MethodCallChain,
    MethodCallParam,
    MethodDefChanging,
    MethodDefExceptions,
    MethodDefExporting,
    MethodDefImporting,
    MethodDefRaising,
    MethodDefReturning,
    MethodName,
    MethodParam,
    MethodParameters,
    MethodParamName,
    MethodParamOptional,
    MethodSource,
    Modif,
    NamespaceSimpleName,
    NewObject,
    OLEExporting,
    Or,
    ParameterException,
    ParameterListExceptions,
    ParameterListS,
    ParameterListT,
    ParameterName,
    ParameterS,
    ParameterT,
    PassByValue,
    PerformChanging,
    PerformTables,
    PerformUsing,
    ProvideFieldName,
    RadioGroupName,
    RaiseWith,
    ReadTableTarget,
    ReceiveParameters,
    Redefinition,
    ReduceBody,
    ReduceNext,
    ReportName,
    Select,
    SelectCTE,
    SimpleFieldChain,
    SimpleFieldChain2,
    SimpleName,
    SimpleSource1,
    SimpleSource2,
    SimpleSource3,
    SimpleSource4,
    SimpleTarget,
    Source,
    SourceField,
    SourceFieldSymbol,
    SourceFieldSymbolChain,
    SQLAggregation,
    SQLAliasField,
    SQLAsName,
    SQLAssociationEntry,
    SQLAssociationsList,
    SQLBypassingBuffer,
    SQLCase,
    SQLCDSParameters,
    SQLClient,
    SQLCompare,
    SQLCompareOperator,
    SQLCond,
    SQLCreating,
    SQLCreatingColumns,
    SQLCreatingForScope,
    SQLCreatingTypeSpec,
    SQLCTEAssociations,
    SQLCTEHierarchy,
    SQLCTEPathPrefix,
    SQLDefiningNewAssociation,
    SQLDmlOptions,
    SQLExposeClientAs,
    SQLExtendedResult,
    SQLField,
    SQLFieldAndValue,
    SQLFieldList,
    SQLFieldListLoopGreedy,
    SQLFieldName,
    SQLFields,
    SQLFieldsLoop,
    SQLForAllEntries,
    SQLFrom,
    SQLFromBody,
    SQLFromSource,
    SQLFunction,
    SQLFunctionInput,
    SQLGroupBy,
    SQLGroupingSets,
    SQLHaving,
    SQLHierarchyAccessor,
    SQLHierarchyAggregate,
    SQLHierarchySource,
    SQLHints,
    SQLIn,
    SQLIndicators,
    SQLIntoList,
    SQLIntoStructure,
    SQLIntoTable,
    SQLJoin,
    SQLJoinSource,
    SQLLikeRegexpr,
    SQLLocateRegexpr,
    SQLMappingFromEntity,
    SQLOccurrencesRegexpr,
    SQLOffset,
    SQLOptions,
    SQLOrderBy,
    SQLOver,
    SQLPackageSize,
    SQLPathCardinality,
    SQLPathForColumn,
    SQLPathForEntity,
    SQLPathForEntityNoPrefix,
    SQLPathJoinType,
    SQLPathSegment,
    SQLPathSegmentNws,
    SQLPrivilegedAccess,
    SQLProvidedBy,
    SQLRegexprFrom,
    SQLRegexprGroup,
    SQLRegexprOccurrence,
    SQLRegexprPattern,
    SQLReplaceRegexpr,
    SQLSetOp,
    SQLSetOpGroup,
    SQLSource,
    SQLSourceNoSpace,
    SQLSourceSimple,
    SQLSubstringRegexpr,
    SQLTarget,
    SQLTypedLiteral,
    SQLUpTo,
    StringTemplate,
    StringTemplateFormatting,
    StringTemplateSource,
    SuperClassName,
    SwitchBody,
    TableBody,
    TableExpression,
    Target,
    TargetField,
    TargetFieldSymbol,
    TestSeamName,
    TextElement,
    TextElementKey,
    TextElementString,
    Throw,
    TransportingFields,
    Type,
    TypeName,
    TypeNameOrInfer,
    TypeParam,
    TypeSimpleRef,
    TypeStructure,
    TypeTable,
    TypeTableKey,
    Value,
    ValueBase,
    ValueBody,
    ValueBodyLine,
    ValueBodyLines,
    WithName,
    WriteOffsetLength,
}

impl E {
    pub(super) const ALL: [E; 272] = [
        E::Abstract,
        E::AbstractMethods,
        E::AndReturn,
        E::ArithOperator,
        E::Arrow,
        E::ArrowOrDash,
        E::AssignSource,
        E::AttributeChain,
        E::AttributeName,
        E::BehaviorDefinitionName,
        E::BlockName,
        E::CallTransformationOptions,
        E::CallTransformationParameters,
        E::Cast,
        E::ClassFinal,
        E::ClassFriends,
        E::ClassGlobal,
        E::ClassName,
        E::Color,
        E::Compare,
        E::CompareOperator,
        E::ComponentChain,
        E::ComponentChainSimple,
        E::ComponentCompare,
        E::ComponentCompareSimple,
        E::ComponentCompareSingle,
        E::ComponentCond,
        E::ComponentCondSub,
        E::ComponentName,
        E::ConcatenatedConstant,
        E::Cond,
        E::CondBody,
        E::CondSub,
        E::Constant,
        E::ConstantFieldLength,
        E::ConstantString,
        E::ConvBody,
        E::CorrespondingBody,
        E::CorrespondingBodyBase,
        E::CorrespondingBodyMapping,
        E::DatabaseConnection,
        E::DatabaseTable,
        E::DatabaseTableNoDynamic,
        E::DataDefinition,
        E::Decimals,
        E::Default,
        E::DefinitionName,
        E::Dereference,
        E::Destination,
        E::Dynamic,
        E::EMLEntityPath,
        E::EntityAssociation,
        E::EventHandler,
        E::EventName,
        E::ExceptionName,
        E::Field,
        E::FieldAll,
        E::FieldAssignment,
        E::FieldChain,
        E::FieldLength,
        E::FieldOffset,
        E::FieldSub,
        E::FieldSymbol,
        E::FilterBody,
        E::FinalMethods,
        E::FindType,
        E::For,
        E::FormChanging,
        E::FormName,
        E::FormParam,
        E::FormParamName,
        E::FormParamType,
        E::FormRaising,
        E::FormTables,
        E::FormUsing,
        E::FSTarget,
        E::FunctionExporting,
        E::FunctionExportingParameter,
        E::FunctionName,
        E::FunctionParameters,
        E::IncludeName,
        E::InlineData,
        E::InlineField,
        E::InlineFieldDefinition,
        E::InlineFS,
        E::InlineLoopDefinition,
        E::Integer,
        E::InterfaceName,
        E::KernelId,
        E::Language,
        E::Length,
        E::Let,
        E::LOBHandle,
        E::LoopGroupBy,
        E::LoopGroupByComponent,
        E::LoopGroupByTarget,
        E::LoopSource,
        E::LoopTarget,
        E::MacroName,
        E::MessageClass,
        E::MessageNumber,
        E::MessageSource,
        E::MessageSourceSource,
        E::MessageTypeAndNumber,
        E::MethodCall,
        E::MethodCallBody,
        E::MethodCallChain,
        E::MethodCallParam,
        E::MethodDefChanging,
        E::MethodDefExceptions,
        E::MethodDefExporting,
        E::MethodDefImporting,
        E::MethodDefRaising,
        E::MethodDefReturning,
        E::MethodName,
        E::MethodParam,
        E::MethodParameters,
        E::MethodParamName,
        E::MethodParamOptional,
        E::MethodSource,
        E::Modif,
        E::NamespaceSimpleName,
        E::NewObject,
        E::OLEExporting,
        E::Or,
        E::ParameterException,
        E::ParameterListExceptions,
        E::ParameterListS,
        E::ParameterListT,
        E::ParameterName,
        E::ParameterS,
        E::ParameterT,
        E::PassByValue,
        E::PerformChanging,
        E::PerformTables,
        E::PerformUsing,
        E::ProvideFieldName,
        E::RadioGroupName,
        E::RaiseWith,
        E::ReadTableTarget,
        E::ReceiveParameters,
        E::Redefinition,
        E::ReduceBody,
        E::ReduceNext,
        E::ReportName,
        E::Select,
        E::SelectCTE,
        E::SimpleFieldChain,
        E::SimpleFieldChain2,
        E::SimpleName,
        E::SimpleSource1,
        E::SimpleSource2,
        E::SimpleSource3,
        E::SimpleSource4,
        E::SimpleTarget,
        E::Source,
        E::SourceField,
        E::SourceFieldSymbol,
        E::SourceFieldSymbolChain,
        E::SQLAggregation,
        E::SQLAliasField,
        E::SQLAsName,
        E::SQLAssociationEntry,
        E::SQLAssociationsList,
        E::SQLBypassingBuffer,
        E::SQLCase,
        E::SQLCDSParameters,
        E::SQLClient,
        E::SQLCompare,
        E::SQLCompareOperator,
        E::SQLCond,
        E::SQLCreating,
        E::SQLCreatingColumns,
        E::SQLCreatingForScope,
        E::SQLCreatingTypeSpec,
        E::SQLCTEAssociations,
        E::SQLCTEHierarchy,
        E::SQLCTEPathPrefix,
        E::SQLDefiningNewAssociation,
        E::SQLDmlOptions,
        E::SQLExposeClientAs,
        E::SQLExtendedResult,
        E::SQLField,
        E::SQLFieldAndValue,
        E::SQLFieldList,
        E::SQLFieldListLoopGreedy,
        E::SQLFieldName,
        E::SQLFields,
        E::SQLFieldsLoop,
        E::SQLForAllEntries,
        E::SQLFrom,
        E::SQLFromBody,
        E::SQLFromSource,
        E::SQLFunction,
        E::SQLFunctionInput,
        E::SQLGroupBy,
        E::SQLGroupingSets,
        E::SQLHaving,
        E::SQLHierarchyAccessor,
        E::SQLHierarchyAggregate,
        E::SQLHierarchySource,
        E::SQLHints,
        E::SQLIn,
        E::SQLIndicators,
        E::SQLIntoList,
        E::SQLIntoStructure,
        E::SQLIntoTable,
        E::SQLJoin,
        E::SQLJoinSource,
        E::SQLLikeRegexpr,
        E::SQLLocateRegexpr,
        E::SQLMappingFromEntity,
        E::SQLOccurrencesRegexpr,
        E::SQLOffset,
        E::SQLOptions,
        E::SQLOrderBy,
        E::SQLOver,
        E::SQLPackageSize,
        E::SQLPathCardinality,
        E::SQLPathForColumn,
        E::SQLPathForEntity,
        E::SQLPathForEntityNoPrefix,
        E::SQLPathJoinType,
        E::SQLPathSegment,
        E::SQLPathSegmentNws,
        E::SQLPrivilegedAccess,
        E::SQLProvidedBy,
        E::SQLRegexprFrom,
        E::SQLRegexprGroup,
        E::SQLRegexprOccurrence,
        E::SQLRegexprPattern,
        E::SQLReplaceRegexpr,
        E::SQLSetOp,
        E::SQLSetOpGroup,
        E::SQLSource,
        E::SQLSourceNoSpace,
        E::SQLSourceSimple,
        E::SQLSubstringRegexpr,
        E::SQLTarget,
        E::SQLTypedLiteral,
        E::SQLUpTo,
        E::StringTemplate,
        E::StringTemplateFormatting,
        E::StringTemplateSource,
        E::SuperClassName,
        E::SwitchBody,
        E::TableBody,
        E::TableExpression,
        E::Target,
        E::TargetField,
        E::TargetFieldSymbol,
        E::TestSeamName,
        E::TextElement,
        E::TextElementKey,
        E::TextElementString,
        E::Throw,
        E::TransportingFields,
        E::Type,
        E::TypeName,
        E::TypeNameOrInfer,
        E::TypeParam,
        E::TypeSimpleRef,
        E::TypeStructure,
        E::TypeTable,
        E::TypeTableKey,
        E::Value,
        E::ValueBase,
        E::ValueBody,
        E::ValueBodyLine,
        E::ValueBodyLines,
        E::WithName,
        E::WriteOffsetLength,
    ];
}

/// The grammar of an expression.
pub(super) fn expression(e: E) -> Rule {
    match e {
        E::Abstract => abstract_(),
        E::AbstractMethods => abstract_methods(),
        E::AndReturn => and_return(),
        E::ArithOperator => arith_operator(),
        E::Arrow => arrow(),
        E::ArrowOrDash => arrow_or_dash(),
        E::AssignSource => assign_source(),
        E::AttributeChain => attribute_chain(),
        E::AttributeName => attribute_name(),
        E::BehaviorDefinitionName => behavior_definition_name(),
        E::BlockName => block_name(),
        E::CallTransformationOptions => call_transformation_options(),
        E::CallTransformationParameters => call_transformation_parameters(),
        E::Cast => cast(),
        E::ClassFinal => class_final(),
        E::ClassFriends => class_friends(),
        E::ClassGlobal => class_global(),
        E::ClassName => class_name(),
        E::Color => color(),
        E::Compare => compare(),
        E::CompareOperator => compare_operator(),
        E::ComponentChain => component_chain(),
        E::ComponentChainSimple => component_chain_simple(),
        E::ComponentCompare => component_compare(),
        E::ComponentCompareSimple => component_compare_simple(),
        E::ComponentCompareSingle => component_compare_single(),
        E::ComponentCond => component_cond(),
        E::ComponentCondSub => component_cond_sub(),
        E::ComponentName => component_name(),
        E::ConcatenatedConstant => concatenated_constant(),
        E::Cond => cond(),
        E::CondBody => cond_body(),
        E::CondSub => cond_sub(),
        E::Constant => constant(),
        E::ConstantFieldLength => constant_field_length(),
        E::ConstantString => constant_string(),
        E::ConvBody => conv_body(),
        E::CorrespondingBody => corresponding_body(),
        E::CorrespondingBodyBase => corresponding_body_base(),
        E::CorrespondingBodyMapping => corresponding_body_mapping(),
        E::DatabaseConnection => database_connection(),
        E::DatabaseTable => database_table(),
        E::DatabaseTableNoDynamic => database_table_no_dynamic(),
        E::DataDefinition => data_definition(),
        E::Decimals => decimals(),
        E::Default => default(),
        E::DefinitionName => definition_name(),
        E::Dereference => dereference(),
        E::Destination => destination(),
        E::Dynamic => dynamic(),
        E::EMLEntityPath => eml_entity_path(),
        E::EntityAssociation => entity_association(),
        E::EventHandler => event_handler(),
        E::EventName => event_name(),
        E::ExceptionName => exception_name(),
        E::Field => field(),
        E::FieldAll => field_all(),
        E::FieldAssignment => field_assignment(),
        E::FieldChain => field_chain(),
        E::FieldLength => field_length(),
        E::FieldOffset => field_offset(),
        E::FieldSub => field_sub(),
        E::FieldSymbol => field_symbol(),
        E::FilterBody => filter_body(),
        E::FinalMethods => final_methods(),
        E::FindType => find_type(),
        E::For => for_(),
        E::FormChanging => form_changing(),
        E::FormName => form_name(),
        E::FormParam => form_param(),
        E::FormParamName => form_param_name(),
        E::FormParamType => form_param_type(),
        E::FormRaising => form_raising(),
        E::FormTables => form_tables(),
        E::FormUsing => form_using(),
        E::FSTarget => fs_target(),
        E::FunctionExporting => function_exporting(),
        E::FunctionExportingParameter => function_exporting_parameter(),
        E::FunctionName => function_name(),
        E::FunctionParameters => function_parameters(),
        E::IncludeName => include_name(),
        E::InlineData => inline_data(),
        E::InlineField => inline_field(),
        E::InlineFieldDefinition => inline_field_definition(),
        E::InlineFS => inline_fs(),
        E::InlineLoopDefinition => inline_loop_definition(),
        E::Integer => integer(),
        E::InterfaceName => interface_name(),
        E::KernelId => kernel_id(),
        E::Language => language(),
        E::Length => length(),
        E::Let => let_(),
        E::LOBHandle => lob_handle(),
        E::LoopGroupBy => loop_group_by(),
        E::LoopGroupByComponent => loop_group_by_component(),
        E::LoopGroupByTarget => loop_group_by_target(),
        E::LoopSource => loop_source(),
        E::LoopTarget => loop_target(),
        E::MacroName => macro_name(),
        E::MessageClass => message_class(),
        E::MessageNumber => message_number(),
        E::MessageSource => message_source(),
        E::MessageSourceSource => message_source_source(),
        E::MessageTypeAndNumber => message_type_and_number(),
        E::MethodCall => method_call(),
        E::MethodCallBody => method_call_body(),
        E::MethodCallChain => method_call_chain(),
        E::MethodCallParam => method_call_param(),
        E::MethodDefChanging => method_def_changing(),
        E::MethodDefExceptions => method_def_exceptions(),
        E::MethodDefExporting => method_def_exporting(),
        E::MethodDefImporting => method_def_importing(),
        E::MethodDefRaising => method_def_raising(),
        E::MethodDefReturning => method_def_returning(),
        E::MethodName => method_name(),
        E::MethodParam => method_param(),
        E::MethodParameters => method_parameters(),
        E::MethodParamName => method_param_name(),
        E::MethodParamOptional => method_param_optional(),
        E::MethodSource => method_source(),
        E::Modif => modif(),
        E::NamespaceSimpleName => namespace_simple_name(),
        E::NewObject => new_object(),
        E::OLEExporting => ole_exporting(),
        E::Or => or(),
        E::ParameterException => parameter_exception(),
        E::ParameterListExceptions => parameter_list_exceptions(),
        E::ParameterListS => parameter_list_s(),
        E::ParameterListT => parameter_list_t(),
        E::ParameterName => parameter_name(),
        E::ParameterS => parameter_s(),
        E::ParameterT => parameter_t(),
        E::PassByValue => pass_by_value(),
        E::PerformChanging => perform_changing(),
        E::PerformTables => perform_tables(),
        E::PerformUsing => perform_using(),
        E::ProvideFieldName => provide_field_name(),
        E::RadioGroupName => radio_group_name(),
        E::RaiseWith => raise_with(),
        E::ReadTableTarget => read_table_target(),
        E::ReceiveParameters => receive_parameters(),
        E::Redefinition => redefinition(),
        E::ReduceBody => reduce_body(),
        E::ReduceNext => reduce_next(),
        E::ReportName => report_name(),
        E::Select => select(),
        E::SelectCTE => select_cte(),
        E::SimpleFieldChain => simple_field_chain(),
        E::SimpleFieldChain2 => simple_field_chain2(),
        E::SimpleName => simple_name(),
        E::SimpleSource1 => simple_source1(),
        E::SimpleSource2 => simple_source2(),
        E::SimpleSource3 => simple_source3(),
        E::SimpleSource4 => simple_source4(),
        E::SimpleTarget => simple_target(),
        E::Source => source(),
        E::SourceField => source_field(),
        E::SourceFieldSymbol => source_field_symbol(),
        E::SourceFieldSymbolChain => source_field_symbol_chain(),
        E::SQLAggregation => sql_aggregation(),
        E::SQLAliasField => sql_alias_field(),
        E::SQLAsName => sql_as_name(),
        E::SQLAssociationEntry => sql_association_entry(),
        E::SQLAssociationsList => sql_associations_list(),
        E::SQLBypassingBuffer => sql_bypassing_buffer(),
        E::SQLCase => sql_case(),
        E::SQLCDSParameters => sqlcds_parameters(),
        E::SQLClient => sql_client(),
        E::SQLCompare => sql_compare(),
        E::SQLCompareOperator => sql_compare_operator(),
        E::SQLCond => sql_cond(),
        E::SQLCreating => sql_creating(),
        E::SQLCreatingColumns => sql_creating_columns(),
        E::SQLCreatingForScope => sql_creating_for_scope(),
        E::SQLCreatingTypeSpec => sql_creating_type_spec(),
        E::SQLCTEAssociations => sqlcte_associations(),
        E::SQLCTEHierarchy => sqlcte_hierarchy(),
        E::SQLCTEPathPrefix => sqlcte_path_prefix(),
        E::SQLDefiningNewAssociation => sql_defining_new_association(),
        E::SQLDmlOptions => sql_dml_options(),
        E::SQLExposeClientAs => sql_expose_client_as(),
        E::SQLExtendedResult => sql_extended_result(),
        E::SQLField => sql_field(),
        E::SQLFieldAndValue => sql_field_and_value(),
        E::SQLFieldList => sql_field_list(),
        E::SQLFieldListLoopGreedy => sql_field_list_loop_greedy(),
        E::SQLFieldName => sql_field_name(),
        E::SQLFields => sql_fields(),
        E::SQLFieldsLoop => sql_fields_loop(),
        E::SQLForAllEntries => sql_for_all_entries(),
        E::SQLFrom => sql_from(),
        E::SQLFromBody => sql_from_body(),
        E::SQLFromSource => sql_from_source(),
        E::SQLFunction => sql_function(),
        E::SQLFunctionInput => sql_function_input(),
        E::SQLGroupBy => sql_group_by(),
        E::SQLGroupingSets => sql_grouping_sets(),
        E::SQLHaving => sql_having(),
        E::SQLHierarchyAccessor => sql_hierarchy_accessor(),
        E::SQLHierarchyAggregate => sql_hierarchy_aggregate(),
        E::SQLHierarchySource => sql_hierarchy_source(),
        E::SQLHints => sql_hints(),
        E::SQLIn => sql_in(),
        E::SQLIndicators => sql_indicators(),
        E::SQLIntoList => sql_into_list(),
        E::SQLIntoStructure => sql_into_structure(),
        E::SQLIntoTable => sql_into_table(),
        E::SQLJoin => sql_join(),
        E::SQLJoinSource => sql_join_source(),
        E::SQLLikeRegexpr => sql_like_regexpr(),
        E::SQLLocateRegexpr => sql_locate_regexpr(),
        E::SQLMappingFromEntity => sql_mapping_from_entity(),
        E::SQLOccurrencesRegexpr => sql_occurrences_regexpr(),
        E::SQLOffset => sql_offset(),
        E::SQLOptions => sql_options(),
        E::SQLOrderBy => sql_order_by(),
        E::SQLOver => sql_over(),
        E::SQLPackageSize => sql_package_size(),
        E::SQLPathCardinality => sql_path_cardinality(),
        E::SQLPathForColumn => sql_path_for_column(),
        E::SQLPathForEntity => sql_path_for_entity(),
        E::SQLPathForEntityNoPrefix => sql_path_for_entity_no_prefix(),
        E::SQLPathJoinType => sql_path_join_type(),
        E::SQLPathSegment => sql_path_segment(),
        E::SQLPathSegmentNws => sql_path_segment_nws(),
        E::SQLPrivilegedAccess => sql_privileged_access(),
        E::SQLProvidedBy => sql_provided_by(),
        E::SQLRegexprFrom => sql_regexpr_from(),
        E::SQLRegexprGroup => sql_regexpr_group(),
        E::SQLRegexprOccurrence => sql_regexpr_occurrence(),
        E::SQLRegexprPattern => sql_regexpr_pattern(),
        E::SQLReplaceRegexpr => sql_replace_regexpr(),
        E::SQLSetOp => sql_set_op(),
        E::SQLSetOpGroup => sql_set_op_group(),
        E::SQLSource => sql_source(),
        E::SQLSourceNoSpace => sql_source_no_space(),
        E::SQLSourceSimple => sql_source_simple(),
        E::SQLSubstringRegexpr => sql_substring_regexpr(),
        E::SQLTarget => sql_target(),
        E::SQLTypedLiteral => sql_typed_literal(),
        E::SQLUpTo => sql_up_to(),
        E::StringTemplate => string_template(),
        E::StringTemplateFormatting => string_template_formatting(),
        E::StringTemplateSource => string_template_source(),
        E::SuperClassName => super_class_name(),
        E::SwitchBody => switch_body(),
        E::TableBody => table_body(),
        E::TableExpression => table_expression(),
        E::Target => target(),
        E::TargetField => target_field(),
        E::TargetFieldSymbol => target_field_symbol(),
        E::TestSeamName => test_seam_name(),
        E::TextElement => text_element(),
        E::TextElementKey => text_element_key(),
        E::TextElementString => text_element_string(),
        E::Throw => throw(),
        E::TransportingFields => transporting_fields(),
        E::Type => type_(),
        E::TypeName => type_name(),
        E::TypeNameOrInfer => type_name_or_infer(),
        E::TypeParam => type_param(),
        E::TypeSimpleRef => type_simple_ref(),
        E::TypeStructure => type_structure(),
        E::TypeTable => type_table(),
        E::TypeTableKey => type_table_key(),
        E::Value => value(),
        E::ValueBase => value_base(),
        E::ValueBody => value_body(),
        E::ValueBodyLine => value_body_line(),
        E::ValueBodyLines => value_body_lines(),
        E::WithName => with_name(),
        E::WriteOffsetLength => write_offset_length(),
    }
}

fn abstract_() -> Rule {
    Rule::from("ABSTRACT")
}

fn abstract_methods() -> Rule {
    seq!("ABSTRACT METHODS", plus(E::MethodName))
}

fn and_return() -> Rule {
    Rule::from("AND RETURN")
}

fn arith_operator() -> Rule {
    alt_prio!(
        Tok::WPlusW,
        Tok::WDashW,
        "*",
        "**",
        "/",
        "BIT-XOR",
        "BIT-AND",
        "BIT-OR",
        "DIV",
        "MOD"
    )
}

fn arrow() -> Rule {
    alt_prio!(Tok::InstanceArrow, Tok::StaticArrow)
}

fn arrow_or_dash() -> Rule {
    alt_prio!(Tok::InstanceArrow, Tok::StaticArrow, Tok::Dash)
}

fn assign_source() -> Rule {
    alt!(
        seq!(
            E::Source,
            opt(seq!(
                alt!(Tok::InstanceArrow, not_in(Lang::KeyUser, Tok::StaticArrow)),
                E::Dynamic
            ))
        ),
        seq!(
            "COMPONENT",
            alt!(E::SimpleSource3, ver("7.40 SP02", E::Source)),
            "OF STRUCTURE",
            E::Source
        ),
        not_in(
            Lang::KeyUser,
            seq!("TABLE FIELD", alt!(E::Source, E::Dynamic))
        ),
        not_in(
            Lang::KeyUser,
            seq!(
                E::Dynamic,
                opt(seq!(
                    alt!(Tok::InstanceArrow, Tok::StaticArrow),
                    alt!(E::Field, E::Dynamic)
                ))
            )
        )
    )
}

fn attribute_chain() -> Rule {
    seq!(
        seq!(
            E::AttributeName,
            star_prio(alt_prio!(
                seq!(E::ArrowOrDash, alt_prio!("*", E::ComponentName)),
                E::TableExpression
            ))
        ),
        opt_prio(E::TableBody),
        opt_prio(E::FieldOffset),
        opt_prio(E::FieldLength)
    )
}

fn attribute_name() -> Rule {
    re(r"^(\/\w+\/)?[\w\d_\~%][\w\d_\*\~%]*$")
}

fn behavior_definition_name() -> Rule {
    re(r"^((\w*\/\w+\/)|(\w*\/\w+\/)?[\w\*$%]+)$")
}

fn block_name() -> Rule {
    seq!(
        opt_prio(Tok::WDash),
        re(r"^[\w%\$\*]+$"),
        star_prio(seq!(Tok::Dash, re(r"^[\w%\$\*]+$")))
    )
}

fn call_transformation_options() -> Rule {
    plus(seq!(alt!(E::Field, E::Integer), "=", E::Source))
}

fn call_transformation_parameters() -> Rule {
    alt!(
        plus(seq!(alt!(E::Field, E::Integer), "=", E::SimpleSource3)),
        E::Dynamic
    )
}

fn cast() -> Rule {
    ver(
        "7.40 SP02",
        seq!(
            "CAST",
            E::TypeNameOrInfer,
            Tok::ParenLeftW,
            opt_prio(E::Let),
            E::Source,
            alt_prio!(Tok::WParenRightW, Tok::WParenRight),
            opt(E::Dereference)
        ),
    )
}

fn class_final() -> Rule {
    Rule::from("FINAL")
}

fn class_friends() -> Rule {
    seq!(opt_prio("GLOBAL"), "FRIENDS", plus(E::ClassName))
}

fn class_global() -> Rule {
    Rule::from("PUBLIC")
}

fn class_name() -> Rule {
    re(r"^!?\w*(\/\w{3,}\/)?\w+$")
}

fn color() -> Rule {
    seq!(
        "COLOR",
        opt(seq!(
            alt!(
                seq!("=", E::SimpleSource3),
                alt_prio!(
                    "ON",
                    "OFF",
                    "COLOR OFF",
                    alt_prio!(
                        alt_prio!("1", "2", "3", "4", "5", "6", "7"),
                        alt_prio!(
                            "COL_BACKGROUND",
                            "COL_HEADING",
                            "COL_NORMAL",
                            "COL_TOTAL",
                            "COL_KEY",
                            "COL_POSITIVE",
                            "COL_NEGATIVE",
                            "COL_GROUP"
                        )
                    )
                )
            ),
            opt(alt_prio!("ON", "OFF"))
        ))
    )
}

fn compare() -> Rule {
    seq!(
        opt("NOT"),
        alt_prio!(
            seq!(
                E::Source,
                alt_prio!(
                    seq!(E::CompareOperator, E::Source),
                    seq!(
                        opt_prio("NOT"),
                        "IN",
                        alt_prio!(
                            E::Source,
                            seq!(
                                Tok::WParenLeft,
                                alt_prio!(E::FieldSub, E::Constant),
                                plus(seq!(",", alt_prio!(E::FieldSub, E::Constant))),
                                Tok::ParenRightW
                            )
                        )
                    ),
                    seq!(opt_prio("NOT"), "BETWEEN", E::Source, "AND", E::Source),
                    seq!(
                        "IS",
                        opt_prio("NOT"),
                        alt_prio!(
                            "SUPPLIED",
                            "BOUND",
                            ver("7.50", seq!("INSTANCE OF", E::ClassName)),
                            "REQUESTED",
                            "INITIAL"
                        )
                    )
                )
            ),
            ver("7.40 SP08", E::MethodCallChain),
            seq!(E::SourceFieldSymbolChain, "IS", opt_prio("NOT"), "ASSIGNED")
        )
    )
}

fn compare_operator() -> Rule {
    alt_prio!(
        "=", "<>", "><", "<", ">", "<=", ">=", "=>", "=<", "CA", "CO", "CP", "EQ", "NE", "CN",
        "GE", "GT", "LT", "LE", "CS", "NS", "NA", "NP", "BYTE-CO", "BYTE-CA", "BYTE-CS", "BYTE-CN",
        "BYTE-NA", "BYTE-NS", "O", "Z", "M"
    )
}

fn component_chain() -> Rule {
    seq!(
        seq!(
            E::ComponentName,
            star(alt_prio!(
                seq!(E::ArrowOrDash, alt_prio!("*", E::ComponentName)),
                E::TableExpression
            ))
        ),
        opt_prio(E::TableBody),
        opt_prio(E::FieldOffset),
        opt_prio(E::FieldLength)
    )
}

fn component_chain_simple() -> Rule {
    seq!(
        E::ComponentName,
        star_prio(alt_prio!(
            E::Dereference,
            seq!(E::ArrowOrDash, E::ComponentName)
        )),
        opt_prio(E::FieldOffset),
        opt_prio(E::FieldLength)
    )
}

fn component_compare() -> Rule {
    seq!(
        opt("NOT"),
        seq!(
            E::ComponentChainSimple,
            alt_prio!(
                seq!(E::CompareOperator, E::Source),
                seq!(
                    opt("NOT"),
                    "IN",
                    alt_prio!(
                        E::Source,
                        seq!(
                            Tok::WParenLeft,
                            alt!(E::FieldSub, E::Constant),
                            plus(seq!(",", alt!(E::FieldSub, E::Constant))),
                            Tok::ParenRightW
                        )
                    )
                ),
                seq!(opt("NOT"), "BETWEEN", E::Source, "AND", E::Source),
                seq!(
                    "IS",
                    opt("NOT"),
                    alt_prio!(
                        "SUPPLIED",
                        "BOUND",
                        ver("7.50", seq!("INSTANCE OF", E::Source)),
                        "REQUESTED",
                        "ASSIGNED",
                        "INITIAL"
                    )
                )
            )
        )
    )
}

fn component_compare_simple() -> Rule {
    plus(seq!(
        alt_prio!(E::ComponentChainSimple, E::Dynamic),
        "=",
        alt_prio!(ver("7.40 SP02", E::Source), E::SimpleSource4)
    ))
}

fn component_compare_single() -> Rule {
    seq!(
        alt_prio!(E::ComponentChainSimple, E::Dynamic),
        "=",
        E::Source
    )
}

/// A condition on components, `LOOP ... WHERE`; SAP `abenlogexp_equiv`:
/// EQUIV joins logical expressions there too.
fn component_cond() -> Rule {
    seq!(
        alt!(E::ComponentCompare, E::ComponentCondSub),
        star(seq!(
            alt!("AND", "OR", ver("7.02", "EQUIV")),
            alt!(E::ComponentCompare, E::ComponentCondSub)
        ))
    )
}

fn component_cond_sub() -> Rule {
    seq!(
        opt("NOT"),
        Tok::WParenLeftW,
        E::ComponentCond,
        alt_prio!(Tok::WParenRightW, Tok::ParenRightW)
    )
}

fn component_name() -> Rule {
    re(r"^[\/\w\d_%$\*\~]+$")
}

fn concatenated_constant() -> Rule {
    alt_prio!(
        seq!(re(r"^`.*`$"), plus_prio(seq!("&", re(r"^`.*`$")))),
        seq!(re(r"^'.*'$"), plus_prio(seq!("&", re(r"^'.*'$"))))
    )
}

fn cond() -> Rule {
    seq!(
        alt_prio!(E::Compare, E::CondSub),
        star_prio(seq!(
            alt_prio!("AND", "OR", ver("7.02", "EQUIV")),
            alt_prio!(E::Compare, E::CondSub)
        ))
    )
}

fn cond_body() -> Rule {
    seq!(
        opt_prio(E::Let),
        plus_prio(seq!(
            "WHEN",
            alt_prio!(E::Cond, E::Source),
            "THEN",
            alt_prio!(E::Throw, E::Source)
        )),
        opt_prio(seq!("ELSE", alt_prio!(E::Throw, E::Source)))
    )
}

fn cond_sub() -> Rule {
    seq!(
        opt_prio("NOT"),
        alt_prio!(Tok::WParenLeftW, Tok::WParenLeft),
        E::Cond,
        alt_prio!(Tok::WParenRightW, Tok::ParenRightW)
    )
}

fn constant() -> Rule {
    alt_prio!(
        E::TextElementString,
        E::ConcatenatedConstant,
        E::ConstantString,
        E::Integer
    )
}

fn constant_field_length() -> Rule {
    seq!(
        Tok::ParenLeft,
        alt_prio!(E::Integer, E::SimpleFieldChain),
        Tok::ParenRightW
    )
}

fn constant_string() -> Rule {
    re(r"^('.*')|(`.*`)$")
}

fn conv_body() -> Rule {
    seq!(opt_prio(E::Let), E::Source)
}

fn corresponding_body() -> Rule {
    seq!(
        opt_prio("DEEP"),
        opt_prio(E::CorrespondingBodyBase),
        E::Source,
        // SAP `abencorresponding_constr_using`: a lookup table
        opt_prio(seq!(
            "FROM",
            E::Source,
            "USING",
            opt_prio(seq!("KEY", E::SimpleName)),
            plus(seq!(E::ComponentChainSimple, "=", E::ComponentChainSimple))
        )),
        opt_prio(ver("7.51", "DISCARDING DUPLICATES")),
        opt_prio(E::CorrespondingBodyMapping),
        opt_prio("CHANGING CONTROL"),
        opt_prio(seq!("MAPPING FROM ENTITY", opt_prio("USING CONTROL"))),
        opt_prio(seq!("EXCEPT", alt!(plus(E::Field), "*")))
    )
}

/// SAP `abencorresponding_constr_arg_type`: `[APPENDING] BASE ( base )`, and
/// `APPENDING ( base )` without BASE.
fn corresponding_body_base() -> Rule {
    let base = || seq!(Tok::WParenLeftW, E::Source, Tok::WParenRightW);
    alt_prio!(
        seq!("BASE", base()),
        ver("7.56", seq!("APPENDING", opt_prio("BASE"), base()))
    )
}

fn corresponding_body_mapping() -> Rule {
    seq!(
        "MAPPING",
        plus(seq!(E::ComponentName, "=", E::ComponentChain))
    )
}

fn database_connection() -> Rule {
    seq!(
        "CONNECTION",
        alt_prio!(re(r"[\w\/]+"), E::ConstantString, E::Dynamic)
    )
}

fn database_table() -> Rule {
    alt_prio!(E::Dynamic, re(r"^\*?(\/\w+\/)?\w+$"))
}

fn database_table_no_dynamic() -> Rule {
    alt_prio!(not_in(Lang::KeyUser, E::Dynamic), re(r"^\*?(\/\w+\/)?\w+$"))
}

fn data_definition() -> Rule {
    seq!(
        E::DefinitionName,
        opt_prio(E::ConstantFieldLength),
        alt_prio!(
            ver(
                "7.31",
                seq!(
                    "TYPE",
                    E::TypeName,
                    opt("READ-ONLY"),
                    plus(seq!(
                        alt_prio!("LOCATOR", "READER", "WRITER", seq!("LOB", "HANDLE")),
                        alt_prio!(
                            seq!("FOR", opt("OTHER"), alt_prio!("BLOB", "CLOB"), "COLUMNS"),
                            seq!(
                                "FOR",
                                "ALL",
                                opt("OTHER"),
                                alt_prio!("BLOB", "CLOB"),
                                "COLUMNS"
                            ),
                            seq!("FOR", "ALL", "COLUMNS"),
                            seq!("FOR", "COLUMNS", plus(E::FieldChain))
                        )
                    ))
                )
            ),
            alt!(
                opt(per!("READ-ONLY", E::Type, E::Length, E::Decimals, E::Value)),
                seq!(E::TypeTable, opt_prio("READ-ONLY")),
                E::TypeStructure
            )
        ),
        opt_prio(ver("7.02", "BOXED"))
    )
}

fn decimals() -> Rule {
    seq!("DECIMALS", E::Integer)
}

fn default() -> Rule {
    seq!(
        "DEFAULT",
        alt_prio!(E::TextElement, E::Constant, E::FieldChain)
    )
}

fn definition_name() -> Rule {
    seq!(
        re(r"^&|&?((\w*\/\w+\/)|(\w*\/\w+\/)?[\w\*$%\?#]+)$"),
        star_prio(seq!(
            Tok::Dash,
            opt_prio(re(r"^&|&?((\w*\/\w+\/)|(\w*\/\w+\/)?[\w\*$%\?#]+)$"))
        )),
        opt_prio(Tok::DashW)
    )
}

fn dereference() -> Rule {
    seq!(Tok::InstanceArrow, "*")
}

fn destination() -> Rule {
    seq!(
        "DESTINATION",
        opt_prio("IN GROUP"),
        alt_prio!("DEFAULT", E::Source)
    )
}

fn dynamic() -> Rule {
    seq!(
        alt_prio!(Tok::WParenLeft, Tok::ParenLeft),
        alt_prio!(E::FieldChain, E::Constant),
        alt_prio!(Tok::ParenRightW, Tok::ParenRight)
    )
}

fn eml_entity_path() -> Rule {
    plus_prio(alt_prio!(Tok::AssociationName, re(r"^\\[\w]+$")))
}

fn entity_association() -> Rule {
    seq!(
        re(r"^[\/\w]+$"),
        plus_prio(alt_prio!(Tok::AssociationName, re(r"^\\[\w]+$")))
    )
}

fn event_handler() -> Rule {
    seq!(
        "FOR EVENT",
        E::EventName,
        "OF",
        E::ClassName,
        opt_prio(seq!("IMPORTING", plus_prio(E::MethodParamName)))
    )
}

fn event_name() -> Rule {
    re(r"^[&_!]?\*?\w*(\/\w+\/)?\d*[a-zA-Z_%\$][\w\*%\$\?#]*(~\w+)?$")
}

fn exception_name() -> Rule {
    seq!(E::Field, star_prio(seq!(Tok::Dash, E::Field)))
}

fn field() -> Rule {
    re(r"^[&_!?]?\*?\w*(\/\w+\/)?\d*[a-zA-Z_%\$][\w\*%\$\?#]*(~\w+)?$")
}

fn field_all() -> Rule {
    re(r"^&?\*?(\/\w+\/)?[\w\*\$]+(~\w+)?$")
}

fn field_assignment() -> Rule {
    seq!(E::FieldSub, "=", E::Source)
}

/// The parentheses of a mesh association, SAP `abenmesh_path`:
/// `\_assoc[ source [cond] ]` first, `\_assoc[ [cond] ]` after; meshes came
/// with 7.40 SP05.
fn mesh_condition() -> Rule {
    ver(
        "7.40 SP05",
        seq!(
            Tok::BracketLeftW,
            opt(E::Source),
            star(seq!(
                alt_prio!(E::ComponentChainSimple, E::Dynamic),
                "=",
                E::Source
            )),
            alt_prio!(Tok::WBracketRight, Tok::WBracketRightW)
        ),
    )
}

fn field_chain() -> Rule {
    seq!(
        alt_prio!(
            seq!(E::ClassName, Tok::StaticArrow, E::AttributeName),
            E::SourceField,
            E::SourceFieldSymbol
        ),
        star(alt_prio!(
            E::Dereference,
            ver(
                "7.57",
                seq!(
                    Tok::InstanceArrow,
                    Tok::ParenLeft,
                    alt_prio!(E::FieldChain, E::Constant),
                    alt_prio!(Tok::ParenRight, Tok::ParenRightW)
                )
            ),
            seq!(Tok::InstanceArrow, E::AttributeName),
            ver(
                "7.57",
                seq!(
                    Tok::Dash,
                    Tok::ParenLeft,
                    alt_prio!(E::FieldChain, E::Constant),
                    opt_prio(E::FieldOffset),
                    opt_prio(E::FieldLength),
                    alt_prio!(Tok::ParenRight, Tok::ParenRightW)
                )
            ),
            // SAP: a component name is expected after "-"
            seq!(Tok::Dash, E::ComponentName),
            seq!(Tok::AssociationName, opt_prio(mesh_condition())),
            E::TableExpression
        )),
        seq!(
            opt_prio(E::TableBody),
            opt_prio(E::FieldOffset),
            opt_prio(E::FieldLength)
        )
    )
}

fn field_length() -> Rule {
    seq!(
        Tok::ParenLeft,
        opt_prio(alt_prio!(
            seq!(
                opt_prio(Tok::Plus),
                alt_prio!(re(r"^\d+$"), E::SimpleFieldChain2)
            ),
            "*"
        )),
        alt_prio!(Tok::ParenRightW, Tok::ParenRight)
    )
}

fn field_offset() -> Rule {
    seq!(
        Tok::Plus,
        opt_prio(alt_prio!(re(r"^\d+$"), E::SimpleFieldChain2))
    )
}

fn field_sub() -> Rule {
    seq!(
        re(r"^\*?!?&?(\/\w+\/)?[a-zA-Z_%$\?][\w%$\$\*]*$"),
        star_prio(seq!(Tok::Dash, re(r"^[\w%$\$\*]+$"))),
        opt(E::FieldLength),
        opt(E::TableBody)
    )
}

fn field_symbol() -> Rule {
    alt_prio!(
        re(r"^<[\w\/%$\*\?]+>$"),
        seq!(re(r"^<[\w\/%$\*]+$"), Tok::Dash, re(r"^[\w\/%$\*]+>$"))
    )
}

/// SAP `abenconstructor_expression_filter`; the key may also come before
/// `EXCEPT IN ftab`, as the syntax check of the SAP trial 2025 has it.
fn filter_body() -> Rule {
    seq!(
        E::Source,
        opt_prio(ver("7.52", seq!("USING KEY", E::SimpleName))),
        opt_prio("EXCEPT"),
        opt_prio(per!(
            seq!("IN", E::Source),
            seq!("USING KEY", E::SimpleName)
        )),
        seq!("WHERE", E::ComponentCond)
    )
}

fn final_methods() -> Rule {
    seq!("FINAL METHODS", plus(E::MethodName))
}

fn find_type() -> Rule {
    opt(alt!("REGEX", "SUBSTRING", ver("7.55", "PCRE")))
}

fn for_() -> Rule {
    ver(
        "7.40 SP05",
        seq!(
            "FOR",
            alt!(
                seq!(
                    E::InlineFieldDefinition,
                    opt(seq!("THEN", E::Source)),
                    seq!(alt_prio!("UNTIL", "WHILE"), E::Cond)
                ),
                seq!(
                    E::InlineLoopDefinition,
                    opt_prio(seq!("FROM", E::Source)),
                    opt_prio(seq!("TO", E::Source)),
                    // SAP `abenfor_cond`: STEP, WHERE ( log_exp ) | (cond_syntax)
                    opt_prio(ver("7.57", seq!("STEP", E::Source))),
                    opt_prio(seq!(
                        "WHERE",
                        alt_prio!(
                            E::Dynamic,
                            seq!("(", alt_prio!(E::Dynamic, E::ComponentCond), ")")
                        )
                    ))
                ),
                ver(
                    "7.40 SP08",
                    seq!(
                        "GROUPS",
                        alt!(E::TargetField, E::TargetFieldSymbol),
                        "OF",
                        alt!(E::TargetField, E::TargetFieldSymbol),
                        "IN",
                        E::Source,
                        opt_prio(seq!(
                            "GROUP BY",
                            alt!(E::FieldChain, seq!("(", plus(E::LoopGroupByComponent), ")")),
                            opt(seq!(alt!("ASCENDING", "DESCENDING"), opt("AS TEXT"))),
                            opt("WITHOUT MEMBERS")
                        ))
                    )
                )
            ),
            opt_prio(E::Let)
        ),
    )
}

fn form_changing() -> Rule {
    seq!("CHANGING", plus(E::FormParam))
}

fn form_name() -> Rule {
    seq!(
        re(r"^!?[\w%$\*\/\?<>]+$"),
        star_prio(seq!(Tok::Dash, opt_prio(re(r"^\w+$")))),
        opt_prio(Tok::DashW)
    )
}

fn form_param() -> Rule {
    seq!(
        alt_prio!(E::PassByValue, E::FormParamName),
        opt_prio(alt_prio!(
            E::FormParamType,
            seq!("STRUCTURE", E::SimpleFieldChain)
        ))
    )
}

fn form_param_name() -> Rule {
    seq!(
        re(r"^[\w$&\*%\/]+$"),
        star_prio(seq!(Tok::Dash, re(r"^[\w$&\*%\/]+$")))
    )
}

fn form_param_type() -> Rule {
    alt!(
        seq!(
            "TYPE",
            alt_prio!(
                seq!(
                    seq!(
                        alt!("STANDARD", "HASHED", "INDEX", "SORTED", "ANY"),
                        "TABLE"
                    ),
                    opt_prio(seq!("OF", alt_prio!(seq!("REF TO", fail()), E::TypeName))),
                    opt_prio(seq!("WITH", fail()))
                ),
                ver("8.16", seq!("ANY", "STRUCTURE")),
                seq!(alt!("TABLE", "RANGE"), "OF", fail()),
                seq!(
                    opt_prio(alt!("REF TO", "LINE OF")),
                    E::TypeName,
                    opt_prio(seq!(alt!("LENGTH", "DECIMALS"), E::Integer, fail())),
                    opt(seq!("DEFAULT", alt!(E::Constant, E::FieldChain)))
                )
            )
        ),
        seq!(
            "LIKE",
            alt_prio!(
                seq!(alt!("TABLE", "RANGE"), "OF", fail()),
                seq!(
                    seq!(
                        alt!("STANDARD", "HASHED", "INDEX", "SORTED", "ANY"),
                        "TABLE"
                    ),
                    fail()
                ),
                seq!(opt_prio(alt!("REF TO", "LINE OF")), E::FieldChain)
            )
        )
    )
}

fn form_raising() -> Rule {
    seq!(
        "RAISING",
        plus(alt!(
            E::ClassName,
            seq!("RESUMABLE", Tok::ParenLeft, E::ClassName, Tok::ParenRightW)
        ))
    )
}

fn form_tables() -> Rule {
    seq!("TABLES", plus(E::FormParam))
}

fn form_using() -> Rule {
    seq!("USING", plus(E::FormParam))
}

fn fs_target() -> Rule {
    alt_prio!(E::InlineFS, E::TargetFieldSymbol)
}

fn function_exporting() -> Rule {
    plus_prio(E::FunctionExportingParameter)
}

fn function_exporting_parameter() -> Rule {
    seq!(
        E::ParameterName,
        "=",
        alt_prio!(ver("7.40 SP02", E::Source), E::SimpleSource3)
    )
}

fn function_name() -> Rule {
    alt_prio!(E::Constant, E::FieldChain)
}

fn function_parameters() -> Rule {
    seq!(
        opt_prio(seq!("EXPORTING", E::FunctionExporting)),
        opt_prio(seq!("IMPORTING", E::ParameterListT)),
        opt_prio(seq!("TABLES", E::ParameterListT)),
        opt_prio(seq!("CHANGING", E::ParameterListT)),
        opt_prio(seq!("EXCEPTIONS", E::ParameterListExceptions))
    )
}

fn include_name() -> Rule {
    seq!(
        re(r"^<?(\/\w+\/)?[\w%#]+(~\w+)?>?$"),
        star_prio(seq!(Tok::Dash, opt_prio(re(r"^\w+$"))))
    )
}

fn inline_data() -> Rule {
    alt_prio!(
        ver(
            "7.40 SP02",
            seq!(
                "DATA",
                Tok::ParenLeft,
                E::TargetField,
                alt_prio!(Tok::ParenRightW, Tok::ParenRight)
            )
        ),
        ver(
            "7.57",
            seq!(
                "FINAL",
                Tok::ParenLeft,
                E::TargetField,
                alt_prio!(Tok::ParenRightW, Tok::ParenRight)
            )
        )
    )
}

fn inline_field() -> Rule {
    seq!(E::Field, star_prio(seq!(Tok::Dash, E::FieldAll)))
}

fn inline_field_definition() -> Rule {
    alt_prio!(
        seq!(alt_prio!(E::Field, E::FieldSymbol), "=", E::Source),
        seq!(E::Field, "TYPE", E::TypeName)
    )
}

fn inline_fs() -> Rule {
    ver(
        "7.40 SP02",
        seq!(
            "FIELD-SYMBOL",
            Tok::ParenLeft,
            E::TargetFieldSymbol,
            Tok::ParenRightW
        ),
    )
}

fn inline_loop_definition() -> Rule {
    seq!(
        alt_prio!(E::TargetFieldSymbol, E::TargetField),
        "IN",
        opt("GROUP"),
        E::Source,
        opt_prio(seq!("USING KEY", alt_prio!(E::SimpleName, E::Dynamic))),
        opt_prio(seq!("INDEX INTO", E::TargetField))
    )
}

fn integer() -> Rule {
    seq!(
        opt_prio(alt_prio!(Tok::WDash, Tok::WDashW, Tok::WPlus)),
        re(r"^\d+$")
    )
}

fn interface_name() -> Rule {
    re(r"^\w*(\/\w{3,}\/)?\w+$")
}

fn kernel_id() -> Rule {
    seq!("ID", E::Source, "FIELD", E::Source)
}

fn language() -> Rule {
    seq!("LANGUAGE", alt_prio!("SQLSCRIPT", "SQL", "GRAPH"))
}

fn length() -> Rule {
    seq!(
        "LENGTH",
        alt_prio!(E::Integer, E::ConstantString, E::SimpleFieldChain)
    )
}

fn let_() -> Rule {
    seq!("LET", plus_prio(E::InlineFieldDefinition), "IN")
}

fn lob_handle() -> Rule {
    seq!("WRITER FOR COLUMNS", E::Field)
}

/// SAP `abaploop_at_itab_group_by`.
fn loop_group_by() -> Rule {
    seq!(
        alt!(
            E::Source,
            seq!(
                Tok::WParenLeftW,
                plus(E::LoopGroupByComponent),
                Tok::WParenRightW
            )
        ),
        // SAP `abaploop_at_itab_group_by`: [ASCENDING | DESCENDING [AS TEXT]]
        // [WITHOUT MEMBERS] [group_result]
        opt_prio(seq!(
            alt_prio!("ASCENDING", "DESCENDING"),
            opt_prio("AS TEXT")
        )),
        opt_prio("WITHOUT MEMBERS"),
        opt_prio(E::LoopGroupByTarget)
    )
}

fn loop_group_by_component() -> Rule {
    alt!(
        E::ComponentCompareSingle,
        seq!(E::ComponentName, "=", "GROUP SIZE"),
        seq!(E::ComponentName, "=", "GROUP INDEX")
    )
}

fn loop_group_by_target() -> Rule {
    opt_prio(alt!(
        seq!(opt("REFERENCE"), "INTO", E::Target),
        seq!("ASSIGNING", E::FSTarget)
    ))
}

fn loop_source() -> Rule {
    alt!(E::SimpleSource2, ver("7.40 SP02", E::Source))
}

fn loop_target() -> Rule {
    alt!(
        seq!(
            alt!(
                seq!(opt("REFERENCE"), "INTO", E::Target),
                seq!("ASSIGNING", E::FSTarget)
            ),
            opt_prio("CASTING")
        ),
        "TRANSPORTING NO FIELDS"
    )
}

fn macro_name() -> Rule {
    seq!(
        re(r"^(\/\w+\/)?[\w\*%\?$&]+>?$"),
        star_prio(seq!(Tok::Dash, re(r"^(\/\w+\/)?[\w\*%\?$&]+>?$")))
    )
}

fn message_class() -> Rule {
    seq!(
        re(r"^>?[\w\/]+#?@?\/?!?&?>?\$?\??<?§?~?$"),
        star_prio(Tok::Plus),
        star_prio(Tok::PlusW),
        star_prio(seq!(Tok::Dash, opt_prio(re(r"^\w+$")))),
        opt_prio(Tok::DashW)
    )
}

fn message_number() -> Rule {
    re(r"(?i)^\d\d\d$")
}

fn message_source() -> Rule {
    alt_prio!(
        seq!(
            E::MessageTypeAndNumber,
            opt(seq!(Tok::ParenLeft, E::MessageClass, Tok::ParenRightW))
        ),
        seq!(
            "ID",
            E::SimpleSource3,
            "TYPE",
            E::SimpleSource3,
            "NUMBER",
            alt_prio!(E::MessageNumber, E::SimpleSource3)
        )
    )
}

fn message_source_source() -> Rule {
    alt!(ver("7.40 SP02", E::Source), E::SimpleSource3)
}

fn message_type_and_number() -> Rule {
    re(r"(?i)^[iweaxs]\d\d\d$")
}

fn method_call() -> Rule {
    seq!(E::MethodName, E::MethodCallParam)
}

fn method_call_body() -> Rule {
    alt!(
        E::MethodCallParam,
        E::MethodParameters,
        seq!(
            seq!("PARAMETER-TABLE", E::Source),
            opt_prio(seq!("EXCEPTION-TABLE", E::Source))
        )
    )
}

fn method_call_chain() -> Rule {
    seq!(
        alt_prio!(
            seq!(
                opt_prio(alt_prio!(
                    seq!(E::FieldChain, Tok::InstanceArrow),
                    seq!(E::ClassName, Tok::StaticArrow)
                )),
                E::MethodCall
            ),
            E::NewObject,
            E::Cast
        ),
        star(seq!(
            star(alt_prio!(
                seq!(Tok::InstanceArrow, E::AttributeName),
                seq!(Tok::Dash, E::ComponentName)
            )),
            Tok::InstanceArrow,
            E::MethodCall
        ))
    )
}

fn method_call_param() -> Rule {
    alt_prio!(
        seq!(
            Tok::ParenLeftW,
            alt!(E::Source, E::ParameterListS, E::MethodParameters),
            alt_prio!(
                Tok::WParenRight,
                Tok::WParenRightW,
                Tok::ParenRight,
                Tok::ParenRightW
            )
        ),
        seq!(
            Tok::ParenLeft,
            alt!(E::Source, E::ParameterListS, E::MethodParameters),
            alt_prio!(Tok::WParenRight, Tok::WParenRightW)
        ),
        seq!(
            Tok::ParenLeft,
            E::ConstantString,
            alt_prio!(Tok::ParenRight, Tok::ParenRightW)
        )
    )
}

fn method_def_changing() -> Rule {
    seq!(
        "CHANGING",
        plus(alt_prio!(
            ver(
                "8.16",
                seq!(
                    E::MethodParamName,
                    "TYPE",
                    "ANY",
                    "STRUCTURE",
                    opt_prio("OPTIONAL")
                )
            ),
            E::MethodParamOptional
        ))
    )
}

fn method_def_exceptions() -> Rule {
    seq!(
        "EXCEPTIONS",
        plus_prio(seq!(
            stop_before_any(&[
                "IMPORTING",
                "EXPORTING",
                "CHANGING",
                "RETURNING",
                "RAISING",
                "EXCEPTIONS"
            ]),
            E::NamespaceSimpleName
        ))
    )
}

fn method_def_exporting() -> Rule {
    seq!("EXPORTING", plus(E::MethodParam))
}

fn method_def_importing() -> Rule {
    seq!(
        "IMPORTING",
        plus(alt_prio!(
            ver(
                "8.16",
                seq!(
                    E::MethodParamName,
                    "TYPE",
                    "ANY",
                    "STRUCTURE",
                    opt_prio("OPTIONAL")
                )
            ),
            E::MethodParamOptional
        )),
        opt_prio(seq!("PREFERRED PARAMETER", re(r"^!?(\/\w+\/)?\w+$")))
    )
}

fn method_def_raising() -> Rule {
    seq!(
        "RAISING",
        plus(alt_prio!(
            seq!("RESUMABLE", Tok::ParenLeft, E::ClassName, Tok::ParenRightW),
            seq!(
                stop_before_any(&[
                    "IMPORTING",
                    "EXPORTING",
                    "CHANGING",
                    "RETURNING",
                    "RAISING",
                    "EXCEPTIONS"
                ]),
                E::ClassName
            )
        ))
    )
}

fn method_def_returning() -> Rule {
    seq!(
        "RETURNING",
        seq!(
            alt_prio!("VALUE", "!VALUE"),
            Tok::ParenLeft,
            E::MethodParamName,
            Tok::ParenRightW
        ),
        E::TypeParam
    )
}

fn method_name() -> Rule {
    re(r"^(\/\w+\/)?\w+(~\w+)?$")
}

fn method_param() -> Rule {
    seq!(
        alt_prio!(
            seq!(
                alt_prio!("VALUE", "!VALUE"),
                Tok::ParenLeft,
                E::MethodParamName,
                Tok::ParenRightW
            ),
            seq!(
                alt_prio!("REFERENCE", "!REFERENCE"),
                Tok::ParenLeft,
                E::MethodParamName,
                Tok::ParenRightW
            ),
            E::MethodParamName
        ),
        E::TypeParam
    )
}

fn method_parameters() -> Rule {
    seq!(
        opt_prio(seq!("EXPORTING", E::ParameterListS)),
        opt_prio(seq!("IMPORTING", E::ParameterListT)),
        opt_prio(seq!("CHANGING", E::ParameterListT)),
        opt_prio(seq!("RECEIVING", E::ParameterT)),
        opt_prio(seq!("EXCEPTIONS", E::ParameterListExceptions))
    )
}

fn method_param_name() -> Rule {
    re(r"^!?\w*(\/\w+\/)?\w+$")
}

fn method_param_optional() -> Rule {
    seq!(E::MethodParam, opt("OPTIONAL"))
}

fn method_source() -> Rule {
    seq!(
        alt_prio!(
            seq!(
                seq!(E::ClassName, Tok::StaticArrow),
                alt!(E::AttributeName, E::MethodCall, E::Dynamic)
            ),
            E::SourceField,
            E::SourceFieldSymbol,
            E::Dynamic
        ),
        star(alt_prio!(
            seq!(
                alt_prio!(Tok::InstanceArrow, Tok::StaticArrow),
                alt!(E::AttributeName, E::MethodCall, E::Dynamic)
            ),
            seq!(Tok::Dash, E::ComponentName)
        ))
    )
}

fn modif() -> Rule {
    re(r"^[\w\*]{1,3}$")
}

fn namespace_simple_name() -> Rule {
    re(r"^!?((\w*\/\w+\/)|(\w*\/\w+\/)*[\w\*$%#]+)$")
}

fn new_object() -> Rule {
    ver(
        "7.40 SP02",
        seq!(
            "NEW",
            E::TypeNameOrInfer,
            Tok::ParenLeftW,
            // SAP `abennew_constructor_params_struct`: [let_exp] [BASE dobj]
            opt_prio(E::Let),
            opt_prio(seq!("BASE", E::Source)),
            opt_prio(alt!(
                E::Source,
                E::ParameterListS,
                plus(seq!(Tok::WParenLeftW, E::Source, Tok::WParenRightW)),
                plus(seq!(
                    Tok::WParenLeftW,
                    plus(E::FieldAssignment),
                    Tok::WParenRightW
                ))
            )),
            ")"
        ),
    )
}

fn ole_exporting() -> Rule {
    seq!(
        "EXPORTING",
        plus(seq!(re(r"^[&_!#\*]?[\w\d\*%\$\?#]+$"), "=", E::Source))
    )
}

fn or() -> Rule {
    seq!("OR", E::Source)
}

fn parameter_exception() -> Rule {
    seq!(
        alt_prio!(
            "OTHERS",
            seq!(E::ParameterName, Tok::Dash, E::ComponentName),
            E::ParameterName
        ),
        opt_prio(seq!(
            "=",
            alt_prio!(E::Integer, E::SimpleFieldChain),
            opt_prio(seq!("MESSAGE", E::Target))
        ))
    )
}

fn parameter_list_exceptions() -> Rule {
    plus(E::ParameterException)
}

fn parameter_list_s() -> Rule {
    plus_prio(E::ParameterS)
}

fn parameter_list_t() -> Rule {
    plus(E::ParameterT)
}

fn parameter_name() -> Rule {
    re(r"^[&_!]?\*?\w*(\/\w+\/)?\d*[a-zA-Z_%\$][\w\*%\$\?]*(~\w+)?$")
}

fn parameter_s() -> Rule {
    seq!(E::ParameterName, "=", E::Source)
}

fn parameter_t() -> Rule {
    seq!(E::ParameterName, "=", E::Target)
}

fn pass_by_value() -> Rule {
    seq!("VALUE", Tok::ParenLeft, E::FormParamName, Tok::ParenRightW)
}

fn perform_changing() -> Rule {
    seq!("CHANGING", plus(E::Target))
}

// SAP reads the word of a later section as that section, a declared field
// `changing` too, and any other word as an operand: `USING using` is fine.

fn perform_tables() -> Rule {
    seq!(
        "TABLES",
        plus(seq!(stop_before_any(&["USING", "CHANGING"]), E::Source))
    )
}

fn perform_using() -> Rule {
    seq!(
        "USING",
        plus(seq!(stop_before_any(&["CHANGING"]), E::SimpleSource3))
    )
}

fn provide_field_name() -> Rule {
    seq!(
        re_not(
            r"(?i)^(\/\w+\/)?(\w+~(\w+|\*)|\w+)$",
            &["FROM", "BETWEEN", "WHERE"]
        ),
        opt_prio(seq!(Tok::Dash, re(r"(?i)^\w+$")))
    )
}

fn radio_group_name() -> Rule {
    re(r"^[\w\d%]+$")
}

fn raise_with() -> Rule {
    seq!(
        "WITH",
        alt_prio!(
            seq!(E::Source, opt(E::Source), opt(E::Source), opt(E::Source)),
            seq!(
                E::SimpleSource1,
                opt(E::SimpleSource1),
                opt(E::SimpleSource1),
                opt(E::SimpleSource1)
            )
        )
    )
}

fn read_table_target() -> Rule {
    alt_prio!(
        seq!("ASSIGNING", E::FSTarget),
        seq!(opt_prio("REFERENCE"), "INTO", E::Target),
        "TRANSPORTING NO FIELDS"
    )
}

fn receive_parameters() -> Rule {
    seq!(
        opt(seq!("IMPORTING", E::ParameterListT)),
        opt(seq!("CHANGING", E::ParameterListT)),
        opt(seq!("TABLES", E::ParameterListT)),
        opt(seq!(
            "EXCEPTIONS",
            opt(E::ParameterListExceptions),
            opt(E::Field)
        ))
    )
}

fn redefinition() -> Rule {
    seq!(opt("FINAL"), "REDEFINITION")
}

fn reduce_body() -> Rule {
    seq!(
        opt(E::Let),
        seq!("INIT", plus(E::InlineFieldDefinition)),
        plus(E::For),
        E::ReduceNext
    )
}

fn reduce_next() -> Rule {
    seq!(
        "NEXT",
        plus(seq!(
            E::SimpleTarget,
            alt_prio!(
                "=",
                ver(
                    "7.54",
                    alt!(
                        seq!(Tok::WPlus, "="),
                        seq!(Tok::WDash, "="),
                        "/=",
                        "*=",
                        "&&="
                    )
                )
            ),
            E::Source
        ))
    )
}

fn report_name() -> Rule {
    seq!(re(r"^[\w/$%&=]+$"), star(seq!(Tok::Dash, re(r"^\w+$"))))
}

fn select() -> Rule {
    seq!(
        "SELECT",
        alt_prio!(
            seq!(
                alt_prio!(
                    seq!(
                        "SINGLE",
                        seq!(
                            opt_prio(E::SQLFieldList),
                            seq!(
                                E::SQLFrom,
                                opt_prio(E::SQLClient),
                                opt_prio(E::SQLBypassingBuffer),
                                opt_prio(seq!("WHERE", E::SQLCond)),
                                seq!(opt_prio(E::SQLGroupBy), opt_prio(E::SQLHaving)),
                                seq!(
                                    opt_prio(not_in(Lang::KeyUser, E::DatabaseConnection)),
                                    opt_prio(E::SQLHints),
                                    opt_prio(ver("7.58", E::SQLPrivilegedAccess)),
                                    opt_prio(E::SQLOptions)
                                )
                            )
                        )
                    ),
                    seq!(
                        opt_prio("DISTINCT"),
                        opt_prio(E::SQLFieldList),
                        opt_prio(E::SQLBypassingBuffer),
                        seq!(
                            E::SQLFrom,
                            opt_prio(E::SQLClient),
                            opt_prio(E::SQLBypassingBuffer),
                            seq!(
                                opt_prio(ver("7.50", E::SQLFields)),
                                opt_prio(E::SQLForAllEntries),
                                opt_prio(seq!("WHERE", E::SQLCond)),
                                seq!(opt_prio(E::SQLGroupBy), opt_prio(E::SQLHaving)),
                                seq!(
                                    opt_prio(not_in(Lang::KeyUser, E::DatabaseConnection)),
                                    opt_prio(E::SQLHints),
                                    opt_prio(ver("7.58", E::SQLPrivilegedAccess)),
                                    opt_prio(E::SQLOptions)
                                )
                            )
                        )
                    )
                ),
                ver("7.50", plus_prio(E::SQLSetOp)),
                opt_prio(E::SQLOrderBy),
                opt_prio(alt_prio!(
                    E::SQLIntoTable,
                    E::SQLIntoStructure,
                    E::SQLIntoList
                ))
            ),
            alt_prio!(
                seq!(
                    "SINGLE",
                    opt_prio(not_in(Lang::KeyUser, "FOR UPDATE")),
                    seq!(
                        opt_prio(E::SQLFieldList),
                        opt_prio(E::SQLBypassingBuffer),
                        alt_prio!(
                            seq!(
                                seq!(
                                    alt_prio!(E::SQLIntoStructure, E::SQLIntoList),
                                    opt_prio(ver("7.52", E::SQLExtendedResult)),
                                    opt_prio(ver("7.50", E::SQLCreating))
                                ),
                                E::SQLFrom,
                                opt_prio(E::SQLClient),
                                opt_prio(E::SQLBypassingBuffer),
                                opt_prio(not_in(Lang::KeyUser, E::DatabaseConnection)),
                                opt_prio(seq!("WHERE", E::SQLCond)),
                                seq!(opt_prio(E::SQLGroupBy), opt_prio(E::SQLHaving)),
                                seq!(
                                    opt_prio(not_in(Lang::KeyUser, E::DatabaseConnection)),
                                    opt_prio(E::SQLHints),
                                    opt_prio(ver("7.58", E::SQLPrivilegedAccess)),
                                    opt_prio(E::SQLOptions)
                                )
                            ),
                            seq!(
                                E::SQLFrom,
                                opt_prio(E::SQLClient),
                                opt_prio(E::SQLBypassingBuffer),
                                opt_prio(not_in(Lang::KeyUser, E::DatabaseConnection)),
                                alt_prio!(
                                    seq!(
                                        seq!(
                                            alt_prio!(E::SQLIntoStructure, E::SQLIntoList),
                                            opt_prio(ver("7.52", E::SQLExtendedResult)),
                                            opt_prio(ver("7.50", E::SQLCreating))
                                        ),
                                        opt_prio(E::SQLBypassingBuffer),
                                        opt_prio(seq!("WHERE", E::SQLCond)),
                                        seq!(opt_prio(E::SQLGroupBy), opt_prio(E::SQLHaving)),
                                        seq!(
                                            opt_prio(not_in(Lang::KeyUser, E::DatabaseConnection)),
                                            opt_prio(E::SQLHints),
                                            opt_prio(ver("7.58", E::SQLPrivilegedAccess)),
                                            opt_prio(E::SQLOptions)
                                        )
                                    ),
                                    seq!(
                                        opt_prio(ver("7.50", E::SQLFields)),
                                        opt_prio(seq!("WHERE", E::SQLCond)),
                                        seq!(opt_prio(E::SQLGroupBy), opt_prio(E::SQLHaving)),
                                        seq!(
                                            opt_prio(not_in(Lang::KeyUser, E::DatabaseConnection)),
                                            opt_prio(E::SQLHints),
                                            opt_prio(ver("7.58", E::SQLPrivilegedAccess)),
                                            opt_prio(E::SQLOptions)
                                        ),
                                        opt_prio(seq!(
                                            alt_prio!(E::SQLIntoStructure, E::SQLIntoList),
                                            opt_prio(ver("7.52", E::SQLExtendedResult)),
                                            opt_prio(ver("7.50", E::SQLCreating))
                                        )),
                                        opt_prio(not_in(Lang::KeyUser, E::DatabaseConnection))
                                    )
                                )
                            )
                        )
                    )
                ),
                alt_prio!(
                    seq!(
                        "SINGLE",
                        seq!(
                            opt_prio(E::SQLFieldList),
                            opt_prio(E::SQLBypassingBuffer),
                            alt_prio!(
                                seq!(
                                    seq!(
                                        alt_prio!(E::SQLIntoStructure, E::SQLIntoList),
                                        opt_prio(ver("7.52", E::SQLExtendedResult)),
                                        opt_prio(ver("7.50", E::SQLCreating))
                                    ),
                                    E::SQLFrom,
                                    opt_prio(E::SQLClient),
                                    opt_prio(E::SQLBypassingBuffer),
                                    opt_prio(not_in(Lang::KeyUser, E::DatabaseConnection)),
                                    opt_prio(seq!("WHERE", E::SQLCond)),
                                    seq!(opt_prio(E::SQLGroupBy), opt_prio(E::SQLHaving)),
                                    seq!(
                                        opt_prio(not_in(Lang::KeyUser, E::DatabaseConnection)),
                                        opt_prio(E::SQLHints),
                                        opt_prio(ver("7.58", E::SQLPrivilegedAccess)),
                                        opt_prio(E::SQLOptions)
                                    )
                                ),
                                seq!(
                                    E::SQLFrom,
                                    opt_prio(E::SQLClient),
                                    opt_prio(E::SQLBypassingBuffer),
                                    opt_prio(not_in(Lang::KeyUser, E::DatabaseConnection)),
                                    alt_prio!(
                                        seq!(
                                            seq!(
                                                alt_prio!(E::SQLIntoStructure, E::SQLIntoList),
                                                opt_prio(ver("7.52", E::SQLExtendedResult)),
                                                opt_prio(ver("7.50", E::SQLCreating))
                                            ),
                                            opt_prio(E::SQLBypassingBuffer),
                                            opt_prio(seq!("WHERE", E::SQLCond)),
                                            seq!(opt_prio(E::SQLGroupBy), opt_prio(E::SQLHaving)),
                                            seq!(
                                                opt_prio(not_in(
                                                    Lang::KeyUser,
                                                    E::DatabaseConnection
                                                )),
                                                opt_prio(E::SQLHints),
                                                opt_prio(ver("7.58", E::SQLPrivilegedAccess)),
                                                opt_prio(E::SQLOptions)
                                            )
                                        ),
                                        seq!(
                                            opt_prio(ver("7.50", E::SQLFields)),
                                            opt_prio(seq!("WHERE", E::SQLCond)),
                                            seq!(opt_prio(E::SQLGroupBy), opt_prio(E::SQLHaving)),
                                            seq!(
                                                opt_prio(not_in(
                                                    Lang::KeyUser,
                                                    E::DatabaseConnection
                                                )),
                                                opt_prio(E::SQLHints),
                                                opt_prio(ver("7.58", E::SQLPrivilegedAccess)),
                                                opt_prio(E::SQLOptions)
                                            ),
                                            opt_prio(seq!(
                                                alt_prio!(E::SQLIntoStructure, E::SQLIntoList),
                                                opt_prio(ver("7.52", E::SQLExtendedResult)),
                                                opt_prio(ver("7.50", E::SQLCreating))
                                            )),
                                            opt_prio(not_in(Lang::KeyUser, E::DatabaseConnection))
                                        )
                                    )
                                )
                            )
                        )
                    ),
                    seq!(
                        opt_prio("DISTINCT"),
                        opt_prio(E::SQLFieldList),
                        opt_prio(E::SQLUpTo),
                        opt_prio(E::SQLBypassingBuffer),
                        opt_prio(alt_prio!(
                            seq!(
                                seq!(
                                    E::SQLIntoTable,
                                    opt_prio(ver("7.52", E::SQLExtendedResult)),
                                    opt_prio(ver("7.50", E::SQLCreating))
                                ),
                                opt_prio(E::SQLPackageSize)
                            ),
                            seq!(
                                alt_prio!(E::SQLIntoStructure, E::SQLIntoList),
                                opt_prio(ver("7.52", E::SQLExtendedResult)),
                                opt_prio(ver("7.50", E::SQLCreating))
                            )
                        )),
                        opt_prio(E::SQLUpTo),
                        opt_prio(E::SQLBypassingBuffer),
                        opt_prio(not_in(Lang::KeyUser, E::DatabaseConnection)),
                        seq!(
                            opt_prio(E::SQLFrom),
                            opt_prio(E::SQLClient),
                            opt_prio(E::SQLBypassingBuffer),
                            seq!(
                                opt_prio(E::SQLPackageSize),
                                opt_prio(E::SQLUpTo),
                                opt_prio(E::SQLBypassingBuffer),
                                opt_prio(not_in(Lang::KeyUser, E::DatabaseConnection)),
                                opt_prio(E::SQLBypassingBuffer),
                                opt_prio(E::SQLUpTo)
                            ),
                            alt_prio!(
                                seq!(
                                    seq!(
                                        E::SQLIntoTable,
                                        opt_prio(ver("7.52", E::SQLExtendedResult)),
                                        opt_prio(ver("7.50", E::SQLCreating))
                                    ),
                                    opt_prio(E::SQLPackageSize),
                                    opt_prio(E::SQLBypassingBuffer),
                                    opt_prio(not_in(Lang::KeyUser, E::DatabaseConnection)),
                                    opt_prio(E::SQLUpTo),
                                    opt_prio(E::SQLBypassingBuffer),
                                    opt_prio(ver("7.51", E::SQLOffset)),
                                    opt_prio(E::SQLForAllEntries),
                                    opt_prio(seq!("WHERE", E::SQLCond)),
                                    seq!(opt_prio(E::SQLGroupBy), opt_prio(E::SQLHaving)),
                                    opt_prio(seq!(
                                        E::SQLOrderBy,
                                        opt_prio(E::SQLUpTo),
                                        opt_prio(ver("7.51", E::SQLOffset))
                                    )),
                                    seq!(
                                        opt_prio(not_in(Lang::KeyUser, E::DatabaseConnection)),
                                        opt_prio(E::SQLHints),
                                        opt_prio(ver("7.58", E::SQLPrivilegedAccess)),
                                        opt_prio(E::SQLOptions)
                                    )
                                ),
                                seq!(
                                    seq!(
                                        alt_prio!(E::SQLIntoStructure, E::SQLIntoList),
                                        opt_prio(ver("7.52", E::SQLExtendedResult)),
                                        opt_prio(ver("7.50", E::SQLCreating))
                                    ),
                                    opt_prio(E::SQLBypassingBuffer),
                                    opt_prio(E::SQLUpTo),
                                    opt_prio(E::SQLBypassingBuffer),
                                    opt_prio(E::SQLForAllEntries),
                                    opt_prio(E::SQLUpTo),
                                    opt_prio(E::SQLBypassingBuffer),
                                    opt_prio(ver("7.51", E::SQLOffset)),
                                    opt_prio(seq!("WHERE", E::SQLCond)),
                                    seq!(opt_prio(E::SQLGroupBy), opt_prio(E::SQLHaving)),
                                    opt_prio(seq!(
                                        E::SQLOrderBy,
                                        opt_prio(E::SQLUpTo),
                                        opt_prio(ver("7.51", E::SQLOffset))
                                    )),
                                    seq!(
                                        opt_prio(not_in(Lang::KeyUser, E::DatabaseConnection)),
                                        opt_prio(E::SQLHints),
                                        opt_prio(ver("7.58", E::SQLPrivilegedAccess)),
                                        opt_prio(E::SQLOptions)
                                    )
                                ),
                                seq!(
                                    opt_prio(ver("7.50", E::SQLFields)),
                                    opt_prio(E::SQLForAllEntries),
                                    opt_prio(seq!("WHERE", E::SQLCond)),
                                    seq!(opt_prio(E::SQLGroupBy), opt_prio(E::SQLHaving)),
                                    opt_prio(seq!(
                                        E::SQLOrderBy,
                                        opt_prio(E::SQLUpTo),
                                        opt_prio(ver("7.51", E::SQLOffset))
                                    )),
                                    seq!(
                                        opt_prio(not_in(Lang::KeyUser, E::DatabaseConnection)),
                                        opt_prio(E::SQLHints),
                                        opt_prio(ver("7.58", E::SQLPrivilegedAccess)),
                                        opt_prio(E::SQLOptions)
                                    ),
                                    seq!(
                                        opt_prio(alt_prio!(
                                            seq!(
                                                seq!(
                                                    E::SQLIntoTable,
                                                    opt_prio(ver("7.52", E::SQLExtendedResult)),
                                                    opt_prio(ver("7.50", E::SQLCreating))
                                                ),
                                                opt_prio(E::SQLPackageSize),
                                                opt_prio(E::SQLBypassingBuffer),
                                                opt_prio(E::SQLUpTo),
                                                opt_prio(E::SQLBypassingBuffer),
                                                opt_prio(ver("7.51", E::SQLOffset)),
                                                opt_prio(E::SQLOrderBy)
                                            ),
                                            seq!(
                                                seq!(
                                                    alt_prio!(E::SQLIntoStructure, E::SQLIntoList),
                                                    opt_prio(ver("7.52", E::SQLExtendedResult)),
                                                    opt_prio(ver("7.50", E::SQLCreating))
                                                ),
                                                opt_prio(E::SQLBypassingBuffer),
                                                opt_prio(E::SQLUpTo),
                                                opt_prio(E::SQLBypassingBuffer),
                                                opt_prio(ver("7.51", E::SQLOffset))
                                            )
                                        )),
                                        opt_prio(E::SQLOptions)
                                    )
                                )
                            )
                        )
                    )
                )
            )
        )
    )
}

fn select_cte() -> Rule {
    seq!(
        seq!(
            "SELECT",
            alt_prio!(
                seq!(
                    E::SQLFrom,
                    opt_prio(E::SQLClient),
                    opt_prio("BYPASSING BUFFER"),
                    ver("7.50", E::SQLFieldsLoop),
                    opt_prio(E::SQLForAllEntries),
                    opt_prio(seq!("WHERE", E::SQLCond)),
                    seq!(
                        seq!(opt_prio(E::SQLGroupBy), opt_prio(E::SQLHaving)),
                        opt_prio(seq!(
                            E::SQLOrderBy,
                            opt_prio(E::SQLUpTo),
                            opt_prio(ver("7.51", seq!("OFFSET", E::SQLSource)))
                        )),
                        opt_prio(E::SQLHints),
                        opt_prio(ver("7.58", E::SQLPrivilegedAccess)),
                        opt_prio(E::SQLOptions),
                        opt_prio(not_in(Lang::KeyUser, E::DatabaseConnection))
                    )
                ),
                seq!(
                    opt_prio("DISTINCT"),
                    E::SQLFieldListLoopGreedy,
                    E::SQLFrom,
                    opt_prio(E::SQLClient),
                    opt_prio("BYPASSING BUFFER"),
                    opt_prio(E::SQLForAllEntries),
                    opt_prio(seq!("WHERE", E::SQLCond)),
                    seq!(
                        seq!(opt_prio(E::SQLGroupBy), opt_prio(E::SQLHaving)),
                        opt_prio(seq!(
                            E::SQLOrderBy,
                            opt_prio(E::SQLUpTo),
                            opt_prio(ver("7.51", seq!("OFFSET", E::SQLSource)))
                        )),
                        opt_prio(E::SQLHints),
                        opt_prio(ver("7.58", E::SQLPrivilegedAccess)),
                        opt_prio(E::SQLOptions),
                        opt_prio(not_in(Lang::KeyUser, E::DatabaseConnection))
                    )
                )
            )
        ),
        star_prio(seq!(
            alt_prio!(
                ver(
                    "7.50",
                    seq!("UNION", opt_prio(alt_prio!("DISTINCT", "ALL")))
                ),
                ver(
                    "7.56",
                    alt_prio!(
                        seq!("INTERSECT", opt_prio("DISTINCT")),
                        seq!("EXCEPT", opt_prio("DISTINCT"))
                    )
                )
            ),
            alt_prio!(
                E::SQLSetOpGroup,
                seq!(
                    "SELECT",
                    alt_prio!(
                        seq!(
                            E::SQLFrom,
                            opt_prio(E::SQLClient),
                            opt_prio("BYPASSING BUFFER"),
                            ver("7.50", E::SQLFieldsLoop),
                            opt_prio(E::SQLForAllEntries),
                            opt_prio(seq!("WHERE", E::SQLCond)),
                            seq!(opt_prio(E::SQLGroupBy), opt_prio(E::SQLHaving))
                        ),
                        seq!(
                            opt_prio("DISTINCT"),
                            E::SQLFieldListLoopGreedy,
                            E::SQLFrom,
                            opt_prio(E::SQLClient),
                            opt_prio("BYPASSING BUFFER"),
                            opt_prio(E::SQLForAllEntries),
                            opt_prio(seq!("WHERE", E::SQLCond)),
                            seq!(opt_prio(E::SQLGroupBy), opt_prio(E::SQLHaving))
                        )
                    )
                )
            )
        ))
    )
}

fn simple_field_chain() -> Rule {
    seq!(
        alt_prio!(
            seq!(E::ClassName, Tok::StaticArrow, E::AttributeName),
            E::Field
        ),
        star_prio(seq!(Tok::Dash, E::ComponentName))
    )
}

fn simple_field_chain2() -> Rule {
    seq!(
        alt_prio!(
            seq!(E::ClassName, Tok::StaticArrow, E::AttributeName),
            E::SourceField,
            E::SourceFieldSymbol
        ),
        star(alt_prio!(
            seq!(Tok::InstanceArrow, E::AttributeName),
            seq!(Tok::Dash, E::ComponentName)
        ))
    )
}

fn simple_name() -> Rule {
    re(r"^[\w$%]+$")
}

fn simple_source1() -> Rule {
    alt!(
        E::Constant,
        E::TextElement,
        E::MethodCallChain,
        seq!(E::StringTemplate, star(seq!("&&", E::StringTemplate))),
        E::FieldChain
    )
}

fn simple_source2() -> Rule {
    alt_prio!(
        E::Constant,
        E::TextElement,
        seq!(E::StringTemplate, star(seq!("&&", E::StringTemplate))),
        E::FieldChain
    )
}

fn simple_source3() -> Rule {
    alt_prio!(E::Constant, E::TextElement, E::FieldChain)
}

fn simple_source4() -> Rule {
    alt!(
        E::Constant,
        E::TextElement,
        E::FieldChain,
        E::MethodCallChain
    )
}

fn simple_target() -> Rule {
    seq!(
        alt_prio!(
            seq!(alt_prio!(E::Cast, E::NewObject), E::Arrow, E::FieldAll),
            seq!(E::ClassName, Tok::StaticArrow, E::AttributeName),
            E::TargetField,
            E::TargetFieldSymbol
        ),
        star_prio(alt_prio!(
            E::Dereference,
            seq!(Tok::InstanceArrow, E::AttributeName),
            seq!(Tok::Dash, E::ComponentName),
            E::TableExpression
        )),
        alt_prio!(
            E::TableBody,
            seq!(opt_prio(E::FieldOffset), opt_prio(E::FieldLength))
        )
    )
}

fn source() -> Rule {
    seq!(
        star_prio(alt_prio!(Tok::WDashW, Tok::WPlusW)),
        alt_prio!(
            ver(
                "7.40 SP08",
                seq!(
                    "FILTER",
                    E::TypeNameOrInfer,
                    Tok::ParenLeftW,
                    E::FilterBody,
                    Tok::WParenRightW
                )
            ),
            ver(
                "7.40 SP02",
                seq!(
                    "REF",
                    E::TypeNameOrInfer,
                    Tok::ParenLeftW,
                    E::Source,
                    opt_prio("OPTIONAL"),
                    Tok::WParenRightW
                )
            ),
            ver(
                "7.40 SP05",
                seq!(
                    "CORRESPONDING",
                    E::TypeNameOrInfer,
                    Tok::ParenLeftW,
                    E::CorrespondingBody,
                    Tok::WParenRightW,
                    opt_prio(seq!(alt_prio!("&", "&&", E::ArithOperator), E::Source))
                )
            ),
            ver(
                "7.40 SP02",
                seq!(
                    "CONV",
                    E::TypeNameOrInfer,
                    alt_prio!(Tok::ParenLeft, Tok::ParenLeftW),
                    E::ConvBody,
                    alt_prio!(Tok::ParenRightW, Tok::WParenRightW),
                    opt_prio(seq!(alt_prio!("&", "&&", E::ArithOperator), E::Source))
                )
            ),
            ver(
                "7.40 SP02",
                seq!(
                    "VALUE",
                    E::TypeNameOrInfer,
                    Tok::ParenLeftW,
                    E::ValueBody,
                    alt_prio!(Tok::ParenRightW, Tok::WParenRightW),
                    opt_prio(seq!(alt_prio!("&", "&&", E::ArithOperator), E::Source))
                )
            ),
            ver(
                "7.40 SP02",
                seq!(
                    "COND",
                    E::TypeNameOrInfer,
                    Tok::ParenLeftW,
                    E::CondBody,
                    alt_prio!(Tok::ParenRightW, Tok::WParenRightW),
                    opt_prio(seq!(alt_prio!("&", "&&", E::ArithOperator), E::Source))
                )
            ),
            ver(
                "7.40 SP02",
                seq!(
                    "EXACT",
                    E::TypeNameOrInfer,
                    Tok::ParenLeftW,
                    E::Source,
                    Tok::WParenRightW,
                    opt_prio(seq!(alt_prio!("&", "&&", E::ArithOperator), E::Source))
                )
            ),
            ver(
                "7.40 SP02",
                seq!(
                    "SWITCH",
                    E::TypeNameOrInfer,
                    Tok::ParenLeftW,
                    E::SwitchBody,
                    alt_prio!(Tok::ParenRightW, Tok::WParenRightW),
                    opt_prio(seq!(alt_prio!("&", "&&", E::ArithOperator), E::Source))
                )
            ),
            ver(
                "7.40 SP08",
                seq!(
                    "REDUCE",
                    E::TypeNameOrInfer,
                    Tok::ParenLeftW,
                    E::ReduceBody,
                    Tok::WParenRightW,
                    opt_prio(seq!(alt_prio!("&", "&&", E::ArithOperator), E::Source))
                )
            ),
            seq!(
                opt_prio(alt_prio!(Tok::WPlus, "BIT-NOT")),
                alt_prio!(
                    E::Constant,
                    E::StringTemplate,
                    E::TextElement,
                    seq!(
                        alt_prio!(ver("7.02", "BOOLC"), ver("7.40 SP08", "XSDBOOL")),
                        Tok::ParenLeftW,
                        E::Cond,
                        ")"
                    ),
                    seq!(E::Cast, E::Dereference),
                    seq!(
                        E::MethodCallChain,
                        opt_prio(alt_prio!(
                            seq!(E::Arrow, E::AttributeChain),
                            seq!(Tok::Dash, E::ComponentChain)
                        )),
                        opt_prio(ver("7.56", E::Dereference)),
                        star(seq!(
                            alt_prio!(
                                ver(
                                    "7.57",
                                    seq!(
                                        Tok::InstanceArrow,
                                        Tok::ParenLeft,
                                        alt_prio!(E::FieldChain, E::Constant),
                                        alt_prio!(Tok::ParenRight, Tok::ParenRightW)
                                    )
                                ),
                                ver(
                                    "7.57",
                                    seq!(
                                        Tok::Dash,
                                        Tok::ParenLeft,
                                        alt_prio!(E::FieldChain, E::Constant),
                                        alt_prio!(Tok::ParenRight, Tok::ParenRightW)
                                    )
                                )
                            ),
                            opt_prio(E::Dereference)
                        ))
                    ),
                    seq!(E::FieldChain, opt_prio(ver("7.56", E::Dereference))),
                    seq!(Tok::WParenLeftW, E::Source, Tok::WParenRightW)
                ),
                opt_prio(seq!(alt_prio!("&", "&&", E::ArithOperator), E::Source))
            )
        )
    )
}

fn source_field() -> Rule {
    Rule::from(E::Field)
}

fn source_field_symbol() -> Rule {
    Rule::from(E::FieldSymbol)
}

fn source_field_symbol_chain() -> Rule {
    seq!(
        E::FieldSymbol,
        star_prio(seq!(E::ArrowOrDash, E::ComponentName)),
        opt_prio(E::TableBody)
    )
}

fn sql_aggregation() -> Rule {
    alt_prio!(
        ver(
            "7.57",
            seq!(
                alt_prio!("ROW_NUMBER", "RANK", "DENSE_RANK"),
                alt_prio!(Tok::ParenLeft, Tok::ParenLeftW),
                alt_prio!(Tok::WParenRightW, Tok::WParenRight, Tok::ParenRightW),
                E::SQLOver
            )
        ),
        ver(
            "7.57",
            seq!(
                alt_prio!("LEAD", "LAG"),
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                opt_prio(seq!(
                    ",",
                    E::SQLFunctionInput,
                    opt_prio(seq!(",", E::SQLFunctionInput))
                )),
                alt_prio!(Tok::WParenRightW, Tok::WParenRight, Tok::ParenRightW),
                E::SQLOver
            )
        ),
        ver(
            "7.57",
            seq!(
                alt_prio!("FIRST_VALUE", "LAST_VALUE"),
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                alt_prio!(Tok::WParenRightW, Tok::WParenRight, Tok::ParenRightW),
                E::SQLOver
            )
        ),
        ver(
            "7.57",
            seq!(
                "STRING_AGG",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                opt_prio(seq!(",", E::SQLFunctionInput)),
                alt_prio!(Tok::WParenRightW, Tok::WParenRight, Tok::ParenRightW),
                opt_prio(E::SQLOver)
            )
        ),
        ver(
            "7.57",
            seq!(
                "NTILE",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                alt_prio!(Tok::WParenRightW, Tok::WParenRight, Tok::ParenRightW),
                E::SQLOver
            )
        ),
        ver(
            "7.57",
            seq!(
                alt_prio!("CORR_SPEARMAN", "CORR"),
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                ",",
                E::SQLFunctionInput,
                alt_prio!(Tok::WParenRightW, Tok::WParenRight, Tok::ParenRightW),
                opt_prio(E::SQLOver)
            )
        ),
        ver(
            "7.57",
            seq!(
                alt_prio!("PRODUCT", "MEDIAN", "VAR", "STDDEV"),
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                alt_prio!(Tok::WParenRightW, Tok::WParenRight, Tok::ParenRightW),
                opt_prio(E::SQLOver)
            )
        ),
        seq!(
            "COUNT",
            alt_prio!(Tok::ParenLeft, Tok::ParenLeftW),
            opt_prio("DISTINCT"),
            alt_prio!(
                "*",
                alt_prio!(ver("7.40 SP08", E::SQLFunctionInput), E::SQLFieldName)
            ),
            alt_prio!(Tok::WParenRightW, Tok::WParenRight, Tok::ParenRightW),
            opt_prio(E::SQLOver)
        ),
        seq!(
            "MAX",
            alt_prio!(Tok::ParenLeft, Tok::ParenLeftW),
            opt_prio("DISTINCT"),
            alt_prio!(ver("7.40 SP08", E::SQLFunctionInput), E::SQLFieldName),
            alt_prio!(Tok::WParenRightW, Tok::WParenRight, Tok::ParenRightW),
            opt_prio(E::SQLOver)
        ),
        seq!(
            "MIN",
            alt_prio!(Tok::ParenLeft, Tok::ParenLeftW),
            opt_prio("DISTINCT"),
            alt_prio!(ver("7.40 SP08", E::SQLFunctionInput), E::SQLFieldName),
            alt_prio!(Tok::WParenRightW, Tok::WParenRight, Tok::ParenRightW),
            opt_prio(E::SQLOver)
        ),
        seq!(
            "SUM",
            alt_prio!(Tok::ParenLeft, Tok::ParenLeftW),
            opt_prio("DISTINCT"),
            alt_prio!(ver("7.40 SP08", E::SQLFunctionInput), E::SQLFieldName),
            alt_prio!(Tok::WParenRightW, Tok::WParenRight, Tok::ParenRightW),
            opt_prio(E::SQLOver)
        ),
        seq!(
            "AVG",
            Tok::ParenLeftW,
            opt_prio("DISTINCT"),
            alt_prio!(ver("7.40 SP08", E::SQLFunctionInput), E::SQLFieldName),
            opt_prio(ver(
                "7.51",
                seq!(
                    "AS",
                    alt_prio!(
                        seq!(
                            "DEC",
                            seq!(
                                Tok::ParenLeftW,
                                E::SQLFunctionInput,
                                ",",
                                E::SQLFunctionInput,
                                Tok::WParenRightW
                            )
                        ),
                        seq!(
                            "CURR",
                            seq!(
                                Tok::ParenLeftW,
                                E::SQLFunctionInput,
                                ",",
                                E::SQLFunctionInput,
                                Tok::WParenRightW
                            )
                        ),
                        seq!(
                            "QUAN",
                            seq!(
                                Tok::ParenLeftW,
                                E::SQLFunctionInput,
                                ",",
                                E::SQLFunctionInput,
                                Tok::WParenRightW
                            )
                        ),
                        "D16N",
                        "D34N",
                        "FLTP"
                    )
                )
            )),
            alt_prio!(Tok::WParenRightW, Tok::WParenRight, Tok::ParenRightW),
            opt_prio(E::SQLOver)
        )
    )
}

fn sql_alias_field() -> Rule {
    seq!(
        alt_prio!(
            seq!(Tok::WPlus, re(r"^(\/\w+\/)?\w+~(\/\w+\/)?\w+$")),
            re(r"^(\/\w+\/)?\w+~(\/\w+\/)?\w+$")
        ),
        star_prio(seq!(Tok::Dash, re(r"^\w+$")))
    )
}

fn sql_as_name() -> Rule {
    seq!(
        re(r"^[&_!]?\*?\w*(\/\w+\/)?\d*[a-zA-Z_%\$][\w\*%\$\?]*(~\w+)?$"),
        star_prio(seq!(
            Tok::Dash,
            re(r"^[&_!]?\*?\w*(\/\w+\/)?\d*[a-zA-Z_%\$][\w\*%\$\?]*(~\w+)?$")
        ))
    )
}

fn sql_association_entry() -> Rule {
    ver(
        "7.51",
        alt_prio!(
            E::SQLDefiningNewAssociation,
            seq!(
                E::SQLPathForEntity,
                opt_prio(seq!("AS", E::SQLAsName)),
                opt_prio(seq!("REDIRECTED TO", E::WithName, "VIA", E::DatabaseTable))
            )
        ),
    )
}

fn sql_associations_list() -> Rule {
    ver(
        "7.51",
        seq!(
            Tok::WParenLeftW,
            E::SQLAssociationEntry,
            star(seq!(",", E::SQLAssociationEntry)),
            Tok::WParenRightW
        ),
    )
}

fn sql_bypassing_buffer() -> Rule {
    Rule::from("BYPASSING BUFFER")
}

/// A CASE expression of ABAP SQL; a result may be a typed literal.
fn sql_case() -> Rule {
    ver(
        "7.40 SP05",
        seq!(
            "CASE",
            opt(alt_prio!(
                E::SQLFieldName,
                seq!(Tok::WAt, E::SimpleFieldChain2)
            )),
            plus_prio(seq!(
                "WHEN",
                alt_prio!(
                    E::SQLCond,
                    E::Constant,
                    seq!(Tok::WAt, E::SimpleFieldChain2)
                ),
                "THEN",
                alt_prio!(
                    seq!(
                        Tok::WParenLeftW,
                        alt_prio!(
                            E::SQLCase,
                            E::SQLAggregation,
                            E::SQLFunction,
                            E::SQLTypedLiteral,
                            E::SQLFieldName,
                            E::SQLSource,
                            E::Constant
                        ),
                        Tok::WParenRightW
                    ),
                    seq!(
                        opt_prio("-"),
                        alt_prio!(
                            E::SQLCase,
                            E::SQLAggregation,
                            E::SQLFunction,
                            E::SQLTypedLiteral,
                            E::SQLFieldName,
                            E::SQLSource,
                            E::Constant
                        )
                    )
                ),
                star_prio(seq!(
                    alt_prio!("+", "-", "*", "/", "&&"),
                    opt_prio(Tok::WParenLeftW),
                    alt_prio!(
                        E::SQLAggregation,
                        E::SQLCase,
                        E::SQLFunction,
                        E::SQLPathForColumn,
                        E::SQLFieldName,
                        E::Constant
                    ),
                    opt_prio(Tok::WParenRightW)
                ))
            )),
            opt_prio(seq!(
                "ELSE",
                alt_prio!(
                    seq!(
                        Tok::WParenLeftW,
                        alt_prio!(
                            E::SQLCase,
                            E::SQLAggregation,
                            E::SQLFunction,
                            E::SQLTypedLiteral,
                            E::SQLFieldName,
                            E::SQLSource,
                            E::Constant
                        ),
                        Tok::WParenRightW
                    ),
                    seq!(
                        opt_prio("-"),
                        alt_prio!(
                            E::SQLCase,
                            E::SQLAggregation,
                            E::SQLFunction,
                            E::SQLTypedLiteral,
                            E::SQLFieldName,
                            E::SQLSource,
                            E::Constant
                        )
                    )
                )
            )),
            "END"
        ),
    )
}

fn sqlcds_parameters() -> Rule {
    seq!(
        alt!(Tok::ParenLeft, Tok::ParenLeftW),
        seq!(
            E::Field,
            "=",
            alt!(seq!(Tok::WAt, E::FieldChain), E::Constant)
        ),
        star_prio(seq!(
            ",",
            seq!(
                E::Field,
                "=",
                alt!(seq!(Tok::WAt, E::FieldChain), E::Constant)
            )
        )),
        alt!(Tok::WParenRightW, Tok::WParenRight)
    )
}

fn sql_client() -> Rule {
    not_in(
        Lang::KeyUser,
        alt!(
            not_in(
                Lang::Cloud,
                seq!(
                    "CLIENT SPECIFIED",
                    opt_prio(ver(
                        "7.40 SP05",
                        alt!(
                            E::Dynamic,
                            seq!(E::SQLAliasField, star_prio(seq!(",", E::SQLAliasField)))
                        )
                    ))
                )
            ),
            seq!(
                "USING",
                alt!(
                    ver("7.40 SP05", seq!("CLIENT", E::SQLSourceSimple)),
                    ver("7.54", seq!("CLIENTS IN", alt!(E::SQLSourceSimple, "T000"))),
                    ver("7.54", "ALL CLIENTS")
                )
            )
        ),
    )
}

/// SAP `abenwhere_logexp_in`, operand list: `( op1, op2 ) [NOT] IN ( ( v1, v2 ),
/// ... )` or `IN ( SELECT ... )`.
fn sql_operand_list_in() -> Rule {
    let row = || {
        seq!(
            Tok::WParenLeftW,
            E::SQLFunctionInput,
            star(seq!(",", E::SQLFunctionInput)),
            Tok::WParenRightW
        )
    };
    seq!(
        Tok::WParenLeftW,
        E::SQLFunctionInput,
        plus(seq!(",", E::SQLFunctionInput)),
        Tok::WParenRightW,
        opt_prio("NOT"),
        alt_prio!(
            seq!(
                "IN",
                Tok::WParenLeftW,
                row(),
                star(seq!(",", row())),
                Tok::WParenRightW
            ),
            E::SQLIn
        )
    )
}

fn sql_compare() -> Rule {
    alt_prio!(
        sql_operand_list_in(),
        seq!(
            "EXISTS",
            alt_prio!(
                ver("7.50", E::SQLSetOpGroup),
                seq!(
                    "(",
                    "SELECT",
                    alt_prio!(
                        seq!(
                            "SINGLE",
                            seq!(
                                opt_prio(E::SQLFieldList),
                                seq!(
                                    E::SQLFrom,
                                    opt_prio(E::SQLClient),
                                    opt_prio(E::SQLBypassingBuffer),
                                    opt_prio(seq!("WHERE", E::SQLCond)),
                                    seq!(opt_prio(E::SQLGroupBy), opt_prio(E::SQLHaving)),
                                    seq!(
                                        opt_prio(not_in(Lang::KeyUser, E::DatabaseConnection)),
                                        opt_prio(E::SQLHints),
                                        opt_prio(ver("7.58", E::SQLPrivilegedAccess)),
                                        opt_prio(E::SQLOptions)
                                    )
                                )
                            )
                        ),
                        seq!(
                            opt_prio("DISTINCT"),
                            opt_prio(E::SQLFieldList),
                            opt_prio(E::SQLBypassingBuffer),
                            seq!(
                                E::SQLFrom,
                                opt_prio(E::SQLClient),
                                opt_prio(E::SQLBypassingBuffer),
                                seq!(
                                    opt_prio(ver("7.50", E::SQLFields)),
                                    opt_prio(E::SQLForAllEntries),
                                    opt_prio(seq!("WHERE", E::SQLCond)),
                                    seq!(opt_prio(E::SQLGroupBy), opt_prio(E::SQLHaving)),
                                    seq!(
                                        opt_prio(not_in(Lang::KeyUser, E::DatabaseConnection)),
                                        opt_prio(E::SQLHints),
                                        opt_prio(ver("7.58", E::SQLPrivilegedAccess)),
                                        opt_prio(E::SQLOptions)
                                    )
                                )
                            )
                        )
                    ),
                    ")"
                )
            )
        ),
        E::Dynamic,
        seq!(
            alt_prio!(
                E::SQLCase,
                E::SQLAggregation,
                E::SQLFunction,
                E::ConstantString,
                seq!(
                    alt_prio!(E::SQLPathForColumn, E::SQLAliasField, E::SQLFieldName),
                    opt_prio(ver(
                        "7.50",
                        plus_prio(seq!(alt_prio!("+", "-", "*", "/"), E::SQLFieldName))
                    ))
                ),
                ver(
                    "7.40 SP05",
                    seq!(
                        Tok::WAt,
                        alt_prio!(
                            E::SimpleSource3,
                            seq!(Tok::ParenLeftW, E::Source, Tok::WParenRightW)
                        )
                    )
                )
            ),
            alt_prio!(
                seq!(
                    E::SQLCompareOperator,
                    alt_prio!(
                        seq!(
                            opt_prio(alt_prio!("ALL", "ANY", "SOME")),
                            alt_prio!(
                                ver("7.50", E::SQLSetOpGroup),
                                ver(
                                    "7.40 SP08",
                                    seq!(
                                        "(",
                                        "SELECT",
                                        alt_prio!(
                                            seq!(
                                                "SINGLE",
                                                seq!(
                                                    opt_prio(E::SQLFieldList),
                                                    seq!(
                                                        E::SQLFrom,
                                                        opt_prio(E::SQLClient),
                                                        opt_prio(E::SQLBypassingBuffer),
                                                        opt_prio(seq!("WHERE", E::SQLCond)),
                                                        seq!(
                                                            opt_prio(E::SQLGroupBy),
                                                            opt_prio(E::SQLHaving)
                                                        ),
                                                        seq!(
                                                            opt_prio(not_in(
                                                                Lang::KeyUser,
                                                                E::DatabaseConnection
                                                            )),
                                                            opt_prio(E::SQLHints),
                                                            opt_prio(ver(
                                                                "7.58",
                                                                E::SQLPrivilegedAccess
                                                            )),
                                                            opt_prio(E::SQLOptions)
                                                        )
                                                    )
                                                )
                                            ),
                                            seq!(
                                                opt_prio("DISTINCT"),
                                                opt_prio(E::SQLFieldList),
                                                opt_prio(E::SQLBypassingBuffer),
                                                seq!(
                                                    E::SQLFrom,
                                                    opt_prio(E::SQLClient),
                                                    opt_prio(E::SQLBypassingBuffer),
                                                    seq!(
                                                        opt_prio(ver("7.50", E::SQLFields)),
                                                        opt_prio(E::SQLForAllEntries),
                                                        opt_prio(seq!("WHERE", E::SQLCond)),
                                                        seq!(
                                                            opt_prio(E::SQLGroupBy),
                                                            opt_prio(E::SQLHaving)
                                                        ),
                                                        seq!(
                                                            opt_prio(not_in(
                                                                Lang::KeyUser,
                                                                E::DatabaseConnection
                                                            )),
                                                            opt_prio(E::SQLHints),
                                                            opt_prio(ver(
                                                                "7.58",
                                                                E::SQLPrivilegedAccess
                                                            )),
                                                            opt_prio(E::SQLOptions)
                                                        )
                                                    )
                                                )
                                            )
                                        ),
                                        ")"
                                    )
                                )
                            )
                        ),
                        E::SQLCase,
                        E::SQLAggregation,
                        E::SQLFunction,
                        seq!(
                            E::SQLSource,
                            opt_prio(ver(
                                "7.50",
                                plus_prio(seq!(alt_prio!("+", "-", "*", "/"), E::SQLFieldName))
                            ))
                        )
                    )
                ),
                seq!(
                    opt_prio("NOT"),
                    alt_prio!(
                        E::SQLIn,
                        seq!("LIKE", E::SQLSource, opt_prio(seq!("ESCAPE", E::SQLSource))),
                        seq!("BETWEEN", E::SQLSource, "AND", E::SQLSource)
                    )
                ),
                E::SQLLikeRegexpr,
                seq!(
                    "IS",
                    opt_prio("NOT"),
                    alt_prio!("NULL", ver("7.53", "INITIAL"))
                )
            )
        )
    )
}

fn sql_compare_operator() -> Rule {
    alt_prio!("=", "<>", "<", ">", "<=", ">=", "EQ", "NE", "GE", "GT", "LT", "LE", "><", "=>", "=<")
}

fn sql_cond() -> Rule {
    seq!(
        seq!(
            opt_prio("NOT"),
            alt_prio!(
                E::SQLCompare,
                seq!(
                    Tok::WParenLeftW,
                    E::SQLCond,
                    alt_prio!(Tok::WParenRightW, Tok::ParenRightW)
                )
            )
        ),
        star_prio(seq!(
            alt_prio!("AND", "OR"),
            seq!(
                opt_prio("NOT"),
                alt_prio!(
                    E::SQLCompare,
                    seq!(
                        Tok::WParenLeftW,
                        E::SQLCond,
                        alt_prio!(Tok::WParenRightW, Tok::ParenRightW)
                    )
                )
            )
        ))
    )
}

fn sql_creating() -> Rule {
    seq!(
        "CREATING",
        alt_prio!(
            E::Dynamic,
            seq!(E::SQLCreatingTypeSpec, star_prio(E::SQLCreatingTypeSpec))
        )
    )
}

fn sql_creating_columns() -> Rule {
    seq!(
        "COLUMNS",
        E::SQLFieldName,
        star_prio(seq!(
            seq!(stop_before("LOCATOR", "FOR"), stop_before("READER", "FOR")),
            opt_prio(","),
            E::SQLFieldName
        ))
    )
}

fn sql_creating_for_scope() -> Rule {
    alt_prio!(
        seq!(
            "FOR ALL OTHER",
            alt_prio!("BLOB COLUMNS", "CLOB COLUMNS", "COLUMNS")
        ),
        seq!(
            "FOR ALL",
            alt_prio!("BLOB COLUMNS", "CLOB COLUMNS", "COLUMNS")
        ),
        seq!("FOR", E::SQLCreatingColumns)
    )
}

fn sql_creating_type_spec() -> Rule {
    seq!(alt_prio!("LOCATOR", "READER"), E::SQLCreatingForScope)
}

fn sqlcte_associations() -> Rule {
    ver(
        "7.51",
        seq!(
            "ASSOCIATIONS",
            alt_prio!(E::SQLAssociationsList, E::Dynamic)
        ),
    )
}

fn sqlcte_hierarchy() -> Rule {
    ver(
        "7.51",
        seq!(
            "HIERARCHY",
            E::SQLAsName,
            opt_prio(seq!("WITH", E::SQLCTEAssociations))
        ),
    )
}

fn sqlcte_path_prefix() -> Rule {
    alt_prio!(seq!(Tok::WPlus, re(r"^\w+~$")), re(r"^\w+~$"))
}

fn sql_defining_new_association() -> Rule {
    ver(
        "7.51",
        seq!(
            "JOIN",
            opt_prio(re(r"(?i)^#[A-Z][A-Z0-9_]*$")),
            opt_prio(E::SQLPathJoinType),
            alt_prio!(E::DatabaseTableNoDynamic, E::WithName),
            "AS",
            E::SQLAsName,
            "ON",
            E::SQLCond
        ),
    )
}

fn sql_dml_options() -> Rule {
    ver(
        "8.16",
        seq!(
            "OPTIONS",
            alt!(
                seq!(
                    seq!("USING CLIENT", E::SQLSourceSimple),
                    opt_prio(per!(E::SQLPrivilegedAccess, E::DatabaseConnection))
                ),
                per!(E::SQLPrivilegedAccess, E::DatabaseConnection)
            )
        ),
    )
}

fn sql_expose_client_as() -> Rule {
    seq!("EXPOSE CLIENT AS", re(r"^\w+$"))
}

fn sql_extended_result() -> Rule {
    seq!("EXTENDED RESULT", Tok::WAt, re(r"^\w+$"))
}

/// A field of the SELECT list, with arithmetic and an alias: abaplint's
/// SQLField, written out by hand (the generator unrolled it). A host
/// expression `@( expr )` takes any ABAP expression, like SAP's host
/// expressions.
fn sql_field() -> Rule {
    let ops = || alt_prio!("+", "-", "*", "/", "&&");
    let abap = || {
        ver(
            "7.40 SP05",
            seq!(
                Tok::WAt,
                alt_prio!(
                    E::SimpleFieldChain2,
                    seq!(Tok::ParenLeftW, E::Source, Tok::WParenRightW)
                )
            ),
        )
    };
    let paren_field_name = || {
        seq!(
            Tok::WParenLeftW,
            E::SQLFieldName,
            alt_prio!(Tok::WParenRightW, Tok::WParenRight)
        )
    };
    let field_no_agg = || {
        alt_prio!(
            E::SQLCase,
            E::SQLFunction,
            E::SQLPathForColumn,
            E::SQLTypedLiteral,
            // a column of a common table expression, `+cte~col`
            E::SQLAliasField,
            E::SQLFieldName,
            abap(),
            E::Constant,
            paren_field_name()
        )
    };
    let field = || alt_prio!(E::SQLAggregation, field_no_agg());
    let paren_field = || seq!(Tok::WParenLeftW, field(), Tok::WParenRightW);
    let paren_field_no_agg = || seq!(Tok::WParenLeftW, field_no_agg(), Tok::WParenRightW);
    let arith = || {
        alt_prio!(
            ver(
                "7.54",
                plus_prio(seq!(ops(), alt_prio!(paren_field(), field())))
            ),
            ver(
                "7.40 SP05",
                plus_prio(seq!(ops(), alt_prio!(paren_field_no_agg(), field_no_agg())))
            )
        )
    };
    let arith_sequence = || seq!(opt_prio("-"), field(), opt_prio(arith()));
    let paren_arith_sequence = || {
        seq!(
            Tok::WParenLeftW,
            opt_prio("-"),
            arith_sequence(),
            Tok::WParenRightW
        )
    };
    let arith_ext = || {
        alt_prio!(
            ver(
                "7.54",
                plus_prio(seq!(
                    ops(),
                    alt_prio!(paren_arith_sequence(), paren_field(), field())
                ))
            ),
            ver(
                "7.40 SP05",
                plus_prio(seq!(
                    ops(),
                    alt_prio!(paren_arith_sequence(), paren_field_no_agg(), field_no_agg())
                ))
            )
        )
    };
    seq!(
        alt_prio!(
            seq!(paren_arith_sequence(), opt_prio(arith_ext())),
            arith_sequence()
        ),
        opt_prio(seq!("AS", E::SQLAsName))
    )
}

fn sql_field_and_value() -> Rule {
    seq!(
        E::SQLFieldName,
        "=",
        alt_prio!(
            seq!(
                alt_prio!(E::Integer, E::SQLFieldName),
                alt_prio!(Tok::WPlusW, Tok::WDashW),
                E::SQLSource
            ),
            E::SQLSource
        )
    )
}

fn sql_field_list() -> Rule {
    alt_prio!(
        "*",
        E::Dynamic,
        seq!(
            E::SQLField,
            alt!(
                ver(
                    "7.40 SP05",
                    star_prio(seq!(
                        ",",
                        alt_prio!(
                            E::SQLField,
                            seq!(
                                Tok::ParenLeftW,
                                E::SQLFieldName,
                                alt_prio!(Tok::WParenRightW, Tok::WParenRight),
                                opt_prio(seq!("AS", E::SQLAsName))
                            )
                        )
                    ))
                ),
                star_prio(seq!(stop_before("UP", "TO"), E::SQLField))
            )
        )
    )
}

fn sql_field_list_loop_greedy() -> Rule {
    alt_prio!(
        "*",
        E::Dynamic,
        plus_prio(seq!(
            alt_prio!(
                E::SQLField,
                ver("7.40 SP05", seq!(Tok::WAt, E::SimpleFieldChain2)),
                E::SQLPathForColumn,
                E::SQLFieldName,
                E::Constant
            ),
            opt_prio(seq!("AS", E::SQLAsName)),
            opt(ver("7.40 SP05", ","))
        ))
    )
}

fn sql_field_name() -> Rule {
    re_not(
        r"(?i)^(\/\w+\/)?(\*?\w+~(\/\w+\/)?(\w+|\*)|\w+)$",
        &[
            "FROM",
            "INTO",
            "HAVING",
            "FOR",
            "DISTINCT",
            "UNION",
            "INTERSECT",
            "EXCEPT",
            "NOT",
            "WHEN",
            "CASE",
            "AS",
            "APPENDING",
            "WHERE",
            "CONNECTION",
            "EXISTS",
            "INSERT",
            "MODIFY",
        ],
    )
}

fn sql_fields() -> Rule {
    seq!("FIELDS", opt("DISTINCT"), E::SQLFieldList)
}

fn sql_fields_loop() -> Rule {
    seq!("FIELDS", opt("DISTINCT"), E::SQLFieldListLoopGreedy)
}

fn sql_for_all_entries() -> Rule {
    seq!("FOR ALL ENTRIES IN", E::SQLSource)
}

fn sql_from() -> Rule {
    seq!("FROM", E::SQLFromBody)
}

fn sql_from_body() -> Rule {
    alt_prio!(
        not_in(
            Lang::KeyUser,
            seq!(
                Tok::WParenLeftW,
                E::SQLFromBody,
                opt_prio(Tok::WParenRightW),
                star_prio(seq!(opt_prio(Tok::WParenRightW), E::SQLJoin))
            )
        ),
        seq!(
            E::SQLFromSource,
            star_prio(seq!(opt_prio(Tok::WParenRightW), E::SQLJoin))
        )
    )
}

fn sql_from_source() -> Rule {
    seq!(
        alt_prio!(
            E::SQLHierarchyAggregate,
            E::SQLHierarchyAccessor,
            E::SQLHierarchySource,
            seq!(
                E::WithName,
                opt_prio(not_in(Lang::KeyUser, E::SQLPathForEntityNoPrefix))
            ),
            seq!(
                E::DatabaseTableNoDynamic,
                opt_prio(E::SQLCDSParameters),
                opt_prio(not_in(Lang::KeyUser, E::SQLPathForEntityNoPrefix))
            ),
            ver("7.52", seq!(Tok::WAt, E::FieldChain))
        ),
        opt_prio(ver("8.16", E::SQLProvidedBy)),
        opt_prio(ver("7.52", seq!("WITH", E::SQLPrivilegedAccess))),
        opt_prio(seq!("AS", E::SQLAsName)),
        opt_prio(ver("8.16", E::SQLExposeClientAs))
    )
}

/// SAP `abensql_cast`: the dictionary types of a cast, with length and
/// decimals as literals or host constants.
fn sql_cast_type() -> Rule {
    let len = || alt_prio!(E::Integer, seq!(Tok::WAt, E::SimpleSource3));
    let len1 = || opt_prio(seq!(Tok::ParenLeftW, len(), Tok::WParenRightW));
    let len2 = || {
        opt_prio(seq!(
            Tok::ParenLeftW,
            len(),
            opt_prio(seq!(",", len())),
            Tok::WParenRightW
        ))
    };
    alt_prio!(
        seq!("CHAR", len1()),
        seq!("SSTRING", len1()),
        seq!("RAW", len1()),
        seq!("NUMC", len1()),
        seq!("DEC", len2()),
        seq!("CURR", len2()),
        seq!("QUAN", len2()),
        seq!("UNIT", Tok::ParenLeftW, len(), Tok::WParenRightW),
        "INT1",
        "INT2",
        "INT4",
        "INT8",
        "D16N",
        "D34N",
        "FLTP",
        "DATS",
        "TIMS",
        "DATN",
        "TIMN",
        "UTCL",
        "CLNT",
        "LANG",
        "CUKY"
    )
}

/// SAP `abensql_string_func`: the PCRE functions with named parameters,
/// `LIKE_REGEXPR( pcre = ..., value = ... )`.
/// LIKE_, REPLACE_ and OCCURRENCES_REGEXPR came with 7.55, SUBSTRING_REGEXPR
/// with 7.56 (release news).
fn sql_regexpr_named() -> Rule {
    let param = || {
        seq!(
            alt_prio!(
                "PCRE",
                "VALUE",
                "WITH",
                "OCCURRENCE",
                "CASE_SENSITIVE",
                "START",
                "GROUP"
            ),
            "=",
            E::SQLFunctionInput
        )
    };
    seq!(
        alt_prio!(
            "LIKE_REGEXPR",
            "LOCATE_REGEXPR_AFTER",
            "LOCATE_REGEXPR",
            "OCCURRENCES_REGEXPR",
            "REPLACE_REGEXPR",
            "SUBSTRING_REGEXPR"
        ),
        Tok::ParenLeftW,
        param(),
        star(seq!(",", param())),
        Tok::WParenRightW
    )
}

fn sql_function() -> Rule {
    alt_prio!(
        ver(
            "7.50",
            seq!(
                "CAST",
                Tok::ParenLeftW,
                alt_prio!(E::SQLCase, E::SQLFunctionInput),
                "AS",
                sql_cast_type(),
                Tok::WParenRightW
            )
        ),
        sql_regexpr_named(),
        E::SQLReplaceRegexpr,
        E::SQLOccurrencesRegexpr,
        E::SQLSubstringRegexpr,
        E::SQLLocateRegexpr,
        ver(
            "7.40 SP05",
            seq!(
                "ABS",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                Tok::WParenRightW
            )
        ),
        ver(
            "7.40 SP05",
            seq!(
                "CEIL",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                Tok::WParenRightW
            )
        ),
        ver(
            "7.40 SP05",
            seq!(
                "FLOOR",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                Tok::WParenRightW
            )
        ),
        ver(
            "7.40 SP05",
            seq!(
                "DIV",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                seq!(",", E::SQLFunctionInput),
                Tok::WParenRightW
            )
        ),
        ver(
            "7.40 SP05",
            seq!(
                "MOD",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                seq!(",", E::SQLFunctionInput),
                Tok::WParenRightW
            )
        ),
        ver(
            "7.40 SP05",
            seq!(
                "COALESCE",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "7.50",
            seq!(
                "CONCAT",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                seq!(",", E::SQLFunctionInput),
                Tok::WParenRightW
            )
        ),
        ver(
            "7.50",
            seq!(
                "LENGTH",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                Tok::WParenRightW
            )
        ),
        ver(
            "7.50",
            seq!(
                "LTRIM",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                seq!(",", E::SQLFunctionInput),
                Tok::WParenRightW
            )
        ),
        ver(
            "7.50",
            seq!(
                "RTRIM",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                seq!(",", E::SQLFunctionInput),
                Tok::WParenRightW
            )
        ),
        ver(
            "7.50",
            seq!(
                "RIGHT",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                seq!(",", E::SQLFunctionInput),
                Tok::WParenRightW
            )
        ),
        ver(
            "7.50",
            seq!(
                "REPLACE",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                seq!(",", E::SQLFunctionInput),
                seq!(",", E::SQLFunctionInput),
                Tok::WParenRightW
            )
        ),
        ver(
            "7.50",
            seq!(
                "ROUND",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                seq!(",", E::SQLFunctionInput),
                Tok::WParenRightW
            )
        ),
        ver(
            "7.50",
            seq!(
                "SUBSTRING",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                seq!(",", E::SQLFunctionInput),
                seq!(",", E::SQLFunctionInput),
                Tok::WParenRightW
            )
        ),
        ver(
            "7.50",
            seq!(
                "LPAD",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                seq!(",", E::SQLFunctionInput),
                seq!(",", E::SQLFunctionInput),
                Tok::WParenRightW
            )
        ),
        ver(
            "7.50",
            seq!(
                "INSTR",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                seq!(",", E::SQLFunctionInput),
                Tok::WParenRightW
            )
        ),
        ver(
            "7.51",
            seq!(
                "CONCAT_WITH_SPACE",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                seq!(",", E::SQLFunctionInput),
                seq!(",", E::SQLFunctionInput),
                Tok::WParenRightW
            )
        ),
        ver(
            "7.51",
            seq!(
                "RPAD",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                seq!(",", E::SQLFunctionInput),
                seq!(",", E::SQLFunctionInput),
                Tok::WParenRightW
            )
        ),
        ver(
            "7.51",
            seq!(
                "LEFT",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                seq!(",", E::SQLFunctionInput),
                Tok::WParenRightW
            )
        ),
        ver(
            "7.51",
            seq!(
                "LOWER",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                Tok::WParenRightW
            )
        ),
        ver(
            "7.51",
            seq!(
                "UPPER",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                Tok::WParenRightW
            )
        ),
        ver(
            "7.51",
            seq!(
                "DIVISION",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                seq!(",", E::SQLFunctionInput),
                seq!(",", E::SQLFunctionInput),
                Tok::WParenRightW
            )
        ),
        ver(
            "7.52",
            seq!(
                "BINTOHEX",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                Tok::WParenRightW
            )
        ),
        ver(
            "7.52",
            seq!(
                "HEXTOBIN",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                Tok::WParenRightW
            )
        ),
        ver(
            "7.54",
            seq!(
                "DATS_IS_VALID",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                Tok::WParenRightW
            )
        ),
        ver(
            "7.54",
            seq!(
                "DATS_DAYS_BETWEEN",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                seq!(",", E::SQLFunctionInput),
                Tok::WParenRightW
            )
        ),
        ver(
            "7.54",
            seq!(
                "DATS_ADD_DAYS",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                seq!(",", E::SQLFunctionInput),
                Tok::WParenRightW
            )
        ),
        ver(
            "7.54",
            seq!(
                "DATS_ADD_MONTHS",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                seq!(",", E::SQLFunctionInput),
                Tok::WParenRightW
            )
        ),
        ver("7.54", seq!("UUID", Tok::ParenLeftW, Tok::WParenRightW)),
        ver(
            "7.54",
            seq!("UTCL_CURRENT", Tok::ParenLeftW, Tok::WParenRightW)
        ),
        ver(
            "7.57",
            seq!(
                "TSTMP_CURRENT_UTCTIMESTAMP",
                Tok::ParenLeftW,
                Tok::WParenRightW
            )
        ),
        ver(
            "7.57",
            seq!(
                "TSTMP_IS_VALID",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                Tok::WParenRightW
            )
        ),
        ver(
            "7.57",
            seq!(
                "TIMS_IS_VALID",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                Tok::WParenRightW
            )
        ),
        ver(
            "7.57",
            seq!(
                "UTCL_ADD_SECONDS",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                seq!(",", E::SQLFunctionInput),
                Tok::WParenRightW
            )
        ),
        ver(
            "7.57",
            seq!(
                "UTCL_SECONDS_BETWEEN",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                seq!(",", E::SQLFunctionInput),
                Tok::WParenRightW
            )
        ),
        ver(
            "7.57",
            seq!(
                "DATN_DAYS_BETWEEN",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                seq!(",", E::SQLFunctionInput),
                Tok::WParenRightW
            )
        ),
        ver(
            "7.57",
            seq!(
                "DATN_ADD_DAYS",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                seq!(",", E::SQLFunctionInput),
                Tok::WParenRightW
            )
        ),
        ver(
            "7.57",
            seq!(
                "DATN_ADD_MONTHS",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                seq!(",", E::SQLFunctionInput),
                Tok::WParenRightW
            )
        ),
        ver(
            "7.57",
            seq!(
                "AS_GEO_JSON",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                Tok::WParenRightW
            )
        ),
        ver(
            "7.57",
            seq!(
                "ALLOW_PRECISION_LOSS",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                Tok::WParenRightW
            )
        ),
        ver(
            "7.57",
            seq!(
                "TO_CLOB",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                Tok::WParenRightW
            )
        ),
        ver(
            "7.57",
            seq!(
                "TO_BLOB",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                Tok::WParenRightW
            )
        ),
        ver(
            "7.57",
            seq!(
                "GROUPING",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                Tok::WParenRightW
            )
        ),
        ver(
            "7.58",
            seq!(
                "EXTRACT_YEAR",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                Tok::WParenRightW
            )
        ),
        ver(
            "7.58",
            seq!(
                "EXTRACT_MONTH",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                Tok::WParenRightW
            )
        ),
        ver(
            "7.58",
            seq!(
                "EXTRACT_DAY",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                Tok::WParenRightW
            )
        ),
        ver(
            "7.58",
            seq!(
                "EXTRACT_HOUR",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                Tok::WParenRightW
            )
        ),
        ver(
            "7.58",
            seq!(
                "EXTRACT_MINUTE",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                Tok::WParenRightW
            )
        ),
        ver(
            "7.58",
            seq!(
                "EXTRACT_SECOND",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                Tok::WParenRightW
            )
        ),
        ver(
            "7.58",
            seq!(
                "DAYS_BETWEEN",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                seq!(",", E::SQLFunctionInput),
                Tok::WParenRightW
            )
        ),
        ver(
            "7.58",
            seq!(
                "ADD_DAYS",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                seq!(",", E::SQLFunctionInput),
                Tok::WParenRightW
            )
        ),
        ver(
            "7.58",
            seq!(
                "ADD_MONTHS",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                seq!(",", E::SQLFunctionInput),
                Tok::WParenRightW
            )
        ),
        ver(
            "7.58",
            seq!(
                "MONTHNAME",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                Tok::WParenRightW
            )
        ),
        ver(
            "7.58",
            seq!(
                "DAYNAME",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                Tok::WParenRightW
            )
        ),
        ver(
            "7.58",
            seq!(
                "INITCAP",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                Tok::WParenRightW
            )
        ),
        ver(
            "7.58",
            seq!(
                "IS_VALID",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                Tok::WParenRightW
            )
        ),
        ver(
            "7.58",
            seq!(
                "WEEKDAY",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                Tok::WParenRightW
            )
        ),
        ver(
            "7.58",
            seq!(
                "LOCATE",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "7.58",
            seq!(
                "GREATEST",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "7.58",
            seq!(
                "LEAST",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "7.58",
            seq!(
                "HIERARCHY_COMPOSITE_ID",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "7.58",
            seq!(
                "SIN",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                Tok::WParenRightW
            )
        ),
        ver(
            "7.58",
            seq!(
                "COS",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                Tok::WParenRightW
            )
        ),
        ver(
            "7.58",
            seq!(
                "TAN",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                Tok::WParenRightW
            )
        ),
        ver(
            "7.58",
            seq!(
                "ASIN",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                Tok::WParenRightW
            )
        ),
        ver(
            "7.58",
            seq!(
                "ACOS",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                Tok::WParenRightW
            )
        ),
        ver(
            "7.58",
            seq!(
                "ATAN",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                Tok::WParenRightW
            )
        ),
        ver(
            "7.58",
            seq!(
                "SINH",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                Tok::WParenRightW
            )
        ),
        ver(
            "7.58",
            seq!(
                "COSH",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                Tok::WParenRightW
            )
        ),
        ver(
            "7.58",
            seq!(
                "TANH",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                Tok::WParenRightW
            )
        ),
        ver(
            "7.58",
            seq!(
                "SQRT",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                Tok::WParenRightW
            )
        ),
        ver(
            "7.58",
            seq!(
                "POWER",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                seq!(",", E::SQLFunctionInput),
                Tok::WParenRightW
            )
        ),
        ver(
            "7.58",
            seq!(
                "EXP",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                Tok::WParenRightW
            )
        ),
        ver(
            "7.58",
            seq!(
                "LN",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                Tok::WParenRightW
            )
        ),
        ver(
            "7.58",
            seq!(
                "LOG10",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                Tok::WParenRightW
            )
        ),
        ver(
            "7.58",
            seq!(
                "LOG",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                seq!(",", E::SQLFunctionInput),
                Tok::WParenRightW
            )
        ),
        ver(
            "7.58",
            seq!(
                "UTCTOLOCAL",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                seq!(",", E::SQLFunctionInput),
                Tok::WParenRightW
            )
        ),
        ver(
            "7.58",
            seq!(
                "LOCALTOUTC",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                seq!(",", E::SQLFunctionInput),
                Tok::WParenRightW
            )
        ),
        ver(
            "7.58",
            seq!(
                "IS_MASKED",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                Tok::WParenRightW
            )
        ),
        ver(
            "7.57",
            seq!(
                "ABAP_USER_TIMEZONE",
                Tok::ParenLeftW,
                opt_prio(seq!(
                    alt_prio!(
                        seq!("USER", "=", E::SQLFunctionInput),
                        seq!("CLIENT", "=", E::SQLFunctionInput),
                        seq!(
                            "ON_ERROR",
                            "=",
                            alt_prio!(
                                E::Constant,
                                seq!(
                                    Tok::WAt,
                                    alt_prio!(
                                        E::SimpleSource3,
                                        seq!(
                                            Tok::ParenLeftW,
                                            E::SQLFunctionInput,
                                            Tok::WParenRightW
                                        )
                                    )
                                )
                            )
                        )
                    ),
                    star_prio(seq!(
                        ",",
                        alt_prio!(
                            seq!("USER", "=", E::SQLFunctionInput),
                            seq!("CLIENT", "=", E::SQLFunctionInput),
                            seq!(
                                "ON_ERROR",
                                "=",
                                alt_prio!(
                                    E::Constant,
                                    seq!(
                                        Tok::WAt,
                                        alt_prio!(
                                            E::SimpleSource3,
                                            seq!(
                                                Tok::ParenLeftW,
                                                E::SQLFunctionInput,
                                                Tok::WParenRightW
                                            )
                                        )
                                    )
                                )
                            )
                        )
                    ))
                )),
                Tok::WParenRightW
            )
        ),
        ver(
            "7.57",
            seq!(
                "ABAP_SYSTEM_TIMEZONE",
                Tok::ParenLeftW,
                opt_prio(seq!(
                    alt_prio!(
                        seq!("CLIENT", "=", E::SQLFunctionInput),
                        seq!(
                            "ON_ERROR",
                            "=",
                            alt_prio!(
                                E::Constant,
                                seq!(
                                    Tok::WAt,
                                    alt_prio!(
                                        E::SimpleSource3,
                                        seq!(
                                            Tok::ParenLeftW,
                                            E::SQLFunctionInput,
                                            Tok::WParenRightW
                                        )
                                    )
                                )
                            )
                        )
                    ),
                    star_prio(seq!(
                        ",",
                        alt_prio!(
                            seq!("CLIENT", "=", E::SQLFunctionInput),
                            seq!(
                                "ON_ERROR",
                                "=",
                                alt_prio!(
                                    E::Constant,
                                    seq!(
                                        Tok::WAt,
                                        alt_prio!(
                                            E::SimpleSource3,
                                            seq!(
                                                Tok::ParenLeftW,
                                                E::SQLFunctionInput,
                                                Tok::WParenRightW
                                            )
                                        )
                                    )
                                )
                            )
                        )
                    ))
                )),
                Tok::WParenRightW
            )
        ),
        ver(
            "7.57",
            seq!(
                "DATS_TIMS_TO_TSTMP",
                Tok::ParenLeftW,
                opt_prio(seq!(
                    alt_prio!(
                        seq!("DATE", "=", E::SQLFunctionInput),
                        seq!("TIME", "=", E::SQLFunctionInput),
                        seq!("TZONE", "=", E::SQLFunctionInput),
                        seq!("CLIENT", "=", E::SQLFunctionInput),
                        seq!(
                            "ON_ERROR",
                            "=",
                            alt_prio!(
                                E::Constant,
                                seq!(
                                    Tok::WAt,
                                    alt_prio!(
                                        E::SimpleSource3,
                                        seq!(
                                            Tok::ParenLeftW,
                                            E::SQLFunctionInput,
                                            Tok::WParenRightW
                                        )
                                    )
                                )
                            )
                        )
                    ),
                    star_prio(seq!(
                        ",",
                        alt_prio!(
                            seq!("DATE", "=", E::SQLFunctionInput),
                            seq!("TIME", "=", E::SQLFunctionInput),
                            seq!("TZONE", "=", E::SQLFunctionInput),
                            seq!("CLIENT", "=", E::SQLFunctionInput),
                            seq!(
                                "ON_ERROR",
                                "=",
                                alt_prio!(
                                    E::Constant,
                                    seq!(
                                        Tok::WAt,
                                        alt_prio!(
                                            E::SimpleSource3,
                                            seq!(
                                                Tok::ParenLeftW,
                                                E::SQLFunctionInput,
                                                Tok::WParenRightW
                                            )
                                        )
                                    )
                                )
                            )
                        )
                    ))
                )),
                Tok::WParenRightW
            )
        ),
        ver(
            "7.57",
            seq!(
                "TSTMP_ADD_SECONDS",
                Tok::ParenLeftW,
                opt_prio(seq!(
                    alt_prio!(
                        seq!("TSTMP", "=", E::SQLFunctionInput),
                        seq!("SECONDS", "=", E::SQLFunctionInput),
                        seq!(
                            "ON_ERROR",
                            "=",
                            alt_prio!(
                                E::Constant,
                                seq!(
                                    Tok::WAt,
                                    alt_prio!(
                                        E::SimpleSource3,
                                        seq!(
                                            Tok::ParenLeftW,
                                            E::SQLFunctionInput,
                                            Tok::WParenRightW
                                        )
                                    )
                                )
                            )
                        )
                    ),
                    star_prio(seq!(
                        ",",
                        alt_prio!(
                            seq!("TSTMP", "=", E::SQLFunctionInput),
                            seq!("SECONDS", "=", E::SQLFunctionInput),
                            seq!(
                                "ON_ERROR",
                                "=",
                                alt_prio!(
                                    E::Constant,
                                    seq!(
                                        Tok::WAt,
                                        alt_prio!(
                                            E::SimpleSource3,
                                            seq!(
                                                Tok::ParenLeftW,
                                                E::SQLFunctionInput,
                                                Tok::WParenRightW
                                            )
                                        )
                                    )
                                )
                            )
                        )
                    ))
                )),
                Tok::WParenRightW
            )
        ),
        ver(
            "7.57",
            seq!(
                "TSTMP_SECONDS_BETWEEN",
                Tok::ParenLeftW,
                opt_prio(seq!(
                    alt_prio!(
                        seq!("TSTMP1", "=", E::SQLFunctionInput),
                        seq!("TSTMP2", "=", E::SQLFunctionInput),
                        seq!(
                            "ON_ERROR",
                            "=",
                            alt_prio!(
                                E::Constant,
                                seq!(
                                    Tok::WAt,
                                    alt_prio!(
                                        E::SimpleSource3,
                                        seq!(
                                            Tok::ParenLeftW,
                                            E::SQLFunctionInput,
                                            Tok::WParenRightW
                                        )
                                    )
                                )
                            )
                        )
                    ),
                    star_prio(seq!(
                        ",",
                        alt_prio!(
                            seq!("TSTMP1", "=", E::SQLFunctionInput),
                            seq!("TSTMP2", "=", E::SQLFunctionInput),
                            seq!(
                                "ON_ERROR",
                                "=",
                                alt_prio!(
                                    E::Constant,
                                    seq!(
                                        Tok::WAt,
                                        alt_prio!(
                                            E::SimpleSource3,
                                            seq!(
                                                Tok::ParenLeftW,
                                                E::SQLFunctionInput,
                                                Tok::WParenRightW
                                            )
                                        )
                                    )
                                )
                            )
                        )
                    ))
                )),
                Tok::WParenRightW
            )
        ),
        ver(
            "7.57",
            seq!(
                "TSTMP_TO_DATS",
                Tok::ParenLeftW,
                opt_prio(seq!(
                    alt_prio!(
                        seq!("TSTMP", "=", E::SQLFunctionInput),
                        seq!("TZONE", "=", E::SQLFunctionInput),
                        seq!("CLIENT", "=", E::SQLFunctionInput),
                        seq!(
                            "ON_ERROR",
                            "=",
                            alt_prio!(
                                E::Constant,
                                seq!(
                                    Tok::WAt,
                                    alt_prio!(
                                        E::SimpleSource3,
                                        seq!(
                                            Tok::ParenLeftW,
                                            E::SQLFunctionInput,
                                            Tok::WParenRightW
                                        )
                                    )
                                )
                            )
                        )
                    ),
                    star_prio(seq!(
                        ",",
                        alt_prio!(
                            seq!("TSTMP", "=", E::SQLFunctionInput),
                            seq!("TZONE", "=", E::SQLFunctionInput),
                            seq!("CLIENT", "=", E::SQLFunctionInput),
                            seq!(
                                "ON_ERROR",
                                "=",
                                alt_prio!(
                                    E::Constant,
                                    seq!(
                                        Tok::WAt,
                                        alt_prio!(
                                            E::SimpleSource3,
                                            seq!(
                                                Tok::ParenLeftW,
                                                E::SQLFunctionInput,
                                                Tok::WParenRightW
                                            )
                                        )
                                    )
                                )
                            )
                        )
                    ))
                )),
                Tok::WParenRightW
            )
        ),
        ver(
            "7.57",
            seq!(
                "TSTMP_TO_TIMS",
                Tok::ParenLeftW,
                opt_prio(seq!(
                    alt_prio!(
                        seq!("TSTMP", "=", E::SQLFunctionInput),
                        seq!("TZONE", "=", E::SQLFunctionInput),
                        seq!("CLIENT", "=", E::SQLFunctionInput),
                        seq!(
                            "ON_ERROR",
                            "=",
                            alt_prio!(
                                E::Constant,
                                seq!(
                                    Tok::WAt,
                                    alt_prio!(
                                        E::SimpleSource3,
                                        seq!(
                                            Tok::ParenLeftW,
                                            E::SQLFunctionInput,
                                            Tok::WParenRightW
                                        )
                                    )
                                )
                            )
                        )
                    ),
                    star_prio(seq!(
                        ",",
                        alt_prio!(
                            seq!("TSTMP", "=", E::SQLFunctionInput),
                            seq!("TZONE", "=", E::SQLFunctionInput),
                            seq!("CLIENT", "=", E::SQLFunctionInput),
                            seq!(
                                "ON_ERROR",
                                "=",
                                alt_prio!(
                                    E::Constant,
                                    seq!(
                                        Tok::WAt,
                                        alt_prio!(
                                            E::SimpleSource3,
                                            seq!(
                                                Tok::ParenLeftW,
                                                E::SQLFunctionInput,
                                                Tok::WParenRightW
                                            )
                                        )
                                    )
                                )
                            )
                        )
                    ))
                )),
                Tok::WParenRightW
            )
        ),
        ver(
            "7.57",
            seq!(
                "TSTMP_TO_DST",
                Tok::ParenLeftW,
                opt_prio(seq!(
                    alt_prio!(
                        seq!("TSTMP", "=", E::SQLFunctionInput),
                        seq!("TZONE", "=", E::SQLFunctionInput),
                        seq!("CLIENT", "=", E::SQLFunctionInput),
                        seq!(
                            "ON_ERROR",
                            "=",
                            alt_prio!(
                                E::Constant,
                                seq!(
                                    Tok::WAt,
                                    alt_prio!(
                                        E::SimpleSource3,
                                        seq!(
                                            Tok::ParenLeftW,
                                            E::SQLFunctionInput,
                                            Tok::WParenRightW
                                        )
                                    )
                                )
                            )
                        )
                    ),
                    star_prio(seq!(
                        ",",
                        alt_prio!(
                            seq!("TSTMP", "=", E::SQLFunctionInput),
                            seq!("TZONE", "=", E::SQLFunctionInput),
                            seq!("CLIENT", "=", E::SQLFunctionInput),
                            seq!(
                                "ON_ERROR",
                                "=",
                                alt_prio!(
                                    E::Constant,
                                    seq!(
                                        Tok::WAt,
                                        alt_prio!(
                                            E::SimpleSource3,
                                            seq!(
                                                Tok::ParenLeftW,
                                                E::SQLFunctionInput,
                                                Tok::WParenRightW
                                            )
                                        )
                                    )
                                )
                            )
                        )
                    ))
                )),
                Tok::WParenRightW
            )
        ),
        ver(
            "7.57",
            seq!(
                "TSTMPL_TO_UTCL",
                Tok::ParenLeftW,
                opt_prio(seq!(
                    alt_prio!(
                        seq!("TSTMPL", "=", E::SQLFunctionInput),
                        seq!(
                            "ON_ERROR",
                            "=",
                            alt_prio!(
                                E::Constant,
                                seq!(
                                    Tok::WAt,
                                    alt_prio!(
                                        E::SimpleSource3,
                                        seq!(
                                            Tok::ParenLeftW,
                                            E::SQLFunctionInput,
                                            Tok::WParenRightW
                                        )
                                    )
                                )
                            )
                        ),
                        seq!(
                            "ON_INITIAL",
                            "=",
                            alt_prio!(
                                E::Constant,
                                seq!(
                                    Tok::WAt,
                                    alt_prio!(
                                        E::SimpleSource3,
                                        seq!(
                                            Tok::ParenLeftW,
                                            E::SQLFunctionInput,
                                            Tok::WParenRightW
                                        )
                                    )
                                )
                            )
                        )
                    ),
                    star_prio(seq!(
                        ",",
                        alt_prio!(
                            seq!("TSTMPL", "=", E::SQLFunctionInput),
                            seq!(
                                "ON_ERROR",
                                "=",
                                alt_prio!(
                                    E::Constant,
                                    seq!(
                                        Tok::WAt,
                                        alt_prio!(
                                            E::SimpleSource3,
                                            seq!(
                                                Tok::ParenLeftW,
                                                E::SQLFunctionInput,
                                                Tok::WParenRightW
                                            )
                                        )
                                    )
                                )
                            ),
                            seq!(
                                "ON_INITIAL",
                                "=",
                                alt_prio!(
                                    E::Constant,
                                    seq!(
                                        Tok::WAt,
                                        alt_prio!(
                                            E::SimpleSource3,
                                            seq!(
                                                Tok::ParenLeftW,
                                                E::SQLFunctionInput,
                                                Tok::WParenRightW
                                            )
                                        )
                                    )
                                )
                            )
                        )
                    ))
                )),
                Tok::WParenRightW
            )
        ),
        ver(
            "7.57",
            seq!(
                "TSTMPL_FROM_UTCL",
                Tok::ParenLeftW,
                opt_prio(seq!(
                    alt_prio!(
                        seq!("UTCL", "=", E::SQLFunctionInput),
                        seq!(
                            "ON_NULL",
                            "=",
                            alt_prio!(
                                E::Constant,
                                seq!(
                                    Tok::WAt,
                                    alt_prio!(
                                        E::SimpleSource3,
                                        seq!(
                                            Tok::ParenLeftW,
                                            E::SQLFunctionInput,
                                            Tok::WParenRightW
                                        )
                                    )
                                )
                            )
                        )
                    ),
                    star_prio(seq!(
                        ",",
                        alt_prio!(
                            seq!("UTCL", "=", E::SQLFunctionInput),
                            seq!(
                                "ON_NULL",
                                "=",
                                alt_prio!(
                                    E::Constant,
                                    seq!(
                                        Tok::WAt,
                                        alt_prio!(
                                            E::SimpleSource3,
                                            seq!(
                                                Tok::ParenLeftW,
                                                E::SQLFunctionInput,
                                                Tok::WParenRightW
                                            )
                                        )
                                    )
                                )
                            )
                        )
                    ))
                )),
                Tok::WParenRightW
            )
        ),
        ver(
            "7.57",
            seq!(
                "DATS_TO_DATN",
                Tok::ParenLeftW,
                opt_prio(seq!(
                    alt_prio!(
                        seq!("DATS", "=", E::SQLFunctionInput),
                        seq!(
                            "ON_ERROR",
                            "=",
                            alt_prio!(
                                E::Constant,
                                seq!(
                                    Tok::WAt,
                                    alt_prio!(
                                        E::SimpleSource3,
                                        seq!(
                                            Tok::ParenLeftW,
                                            E::SQLFunctionInput,
                                            Tok::WParenRightW
                                        )
                                    )
                                )
                            )
                        ),
                        seq!(
                            "ON_INITIAL",
                            "=",
                            alt_prio!(
                                E::Constant,
                                seq!(
                                    Tok::WAt,
                                    alt_prio!(
                                        E::SimpleSource3,
                                        seq!(
                                            Tok::ParenLeftW,
                                            E::SQLFunctionInput,
                                            Tok::WParenRightW
                                        )
                                    )
                                )
                            )
                        )
                    ),
                    star_prio(seq!(
                        ",",
                        alt_prio!(
                            seq!("DATS", "=", E::SQLFunctionInput),
                            seq!(
                                "ON_ERROR",
                                "=",
                                alt_prio!(
                                    E::Constant,
                                    seq!(
                                        Tok::WAt,
                                        alt_prio!(
                                            E::SimpleSource3,
                                            seq!(
                                                Tok::ParenLeftW,
                                                E::SQLFunctionInput,
                                                Tok::WParenRightW
                                            )
                                        )
                                    )
                                )
                            ),
                            seq!(
                                "ON_INITIAL",
                                "=",
                                alt_prio!(
                                    E::Constant,
                                    seq!(
                                        Tok::WAt,
                                        alt_prio!(
                                            E::SimpleSource3,
                                            seq!(
                                                Tok::ParenLeftW,
                                                E::SQLFunctionInput,
                                                Tok::WParenRightW
                                            )
                                        )
                                    )
                                )
                            )
                        )
                    ))
                )),
                Tok::WParenRightW
            )
        ),
        ver(
            "7.57",
            seq!(
                "DATS_FROM_DATN",
                Tok::ParenLeftW,
                opt_prio(seq!(
                    alt_prio!(
                        seq!("DATN", "=", E::SQLFunctionInput),
                        seq!(
                            "ON_NULL",
                            "=",
                            alt_prio!(
                                E::Constant,
                                seq!(
                                    Tok::WAt,
                                    alt_prio!(
                                        E::SimpleSource3,
                                        seq!(
                                            Tok::ParenLeftW,
                                            E::SQLFunctionInput,
                                            Tok::WParenRightW
                                        )
                                    )
                                )
                            )
                        )
                    ),
                    star_prio(seq!(
                        ",",
                        alt_prio!(
                            seq!("DATN", "=", E::SQLFunctionInput),
                            seq!(
                                "ON_NULL",
                                "=",
                                alt_prio!(
                                    E::Constant,
                                    seq!(
                                        Tok::WAt,
                                        alt_prio!(
                                            E::SimpleSource3,
                                            seq!(
                                                Tok::ParenLeftW,
                                                E::SQLFunctionInput,
                                                Tok::WParenRightW
                                            )
                                        )
                                    )
                                )
                            )
                        )
                    ))
                )),
                Tok::WParenRightW
            )
        ),
        ver(
            "7.57",
            seq!(
                "TIMS_TO_TIMN",
                Tok::ParenLeftW,
                opt_prio(seq!(
                    alt_prio!(
                        seq!("TIMS", "=", E::SQLFunctionInput),
                        seq!(
                            "ON_ERROR",
                            "=",
                            alt_prio!(
                                E::Constant,
                                seq!(
                                    Tok::WAt,
                                    alt_prio!(
                                        E::SimpleSource3,
                                        seq!(
                                            Tok::ParenLeftW,
                                            E::SQLFunctionInput,
                                            Tok::WParenRightW
                                        )
                                    )
                                )
                            )
                        )
                    ),
                    star_prio(seq!(
                        ",",
                        alt_prio!(
                            seq!("TIMS", "=", E::SQLFunctionInput),
                            seq!(
                                "ON_ERROR",
                                "=",
                                alt_prio!(
                                    E::Constant,
                                    seq!(
                                        Tok::WAt,
                                        alt_prio!(
                                            E::SimpleSource3,
                                            seq!(
                                                Tok::ParenLeftW,
                                                E::SQLFunctionInput,
                                                Tok::WParenRightW
                                            )
                                        )
                                    )
                                )
                            )
                        )
                    ))
                )),
                Tok::WParenRightW
            )
        ),
        ver(
            "7.57",
            seq!(
                "TIMS_FROM_TIMN",
                Tok::ParenLeftW,
                opt_prio(seq!(
                    alt_prio!(
                        seq!("TIMN", "=", E::SQLFunctionInput),
                        seq!(
                            "ON_NULL",
                            "=",
                            alt_prio!(
                                E::Constant,
                                seq!(
                                    Tok::WAt,
                                    alt_prio!(
                                        E::SimpleSource3,
                                        seq!(
                                            Tok::ParenLeftW,
                                            E::SQLFunctionInput,
                                            Tok::WParenRightW
                                        )
                                    )
                                )
                            )
                        )
                    ),
                    star_prio(seq!(
                        ",",
                        alt_prio!(
                            seq!("TIMN", "=", E::SQLFunctionInput),
                            seq!(
                                "ON_NULL",
                                "=",
                                alt_prio!(
                                    E::Constant,
                                    seq!(
                                        Tok::WAt,
                                        alt_prio!(
                                            E::SimpleSource3,
                                            seq!(
                                                Tok::ParenLeftW,
                                                E::SQLFunctionInput,
                                                Tok::WParenRightW
                                            )
                                        )
                                    )
                                )
                            )
                        )
                    ))
                )),
                Tok::WParenRightW
            )
        ),
        ver(
            "7.57",
            seq!(
                "CURRENCY_CONVERSION",
                Tok::ParenLeftW,
                opt_prio(seq!(
                    alt_prio!(
                        seq!("AMOUNT", "=", E::SQLFunctionInput),
                        seq!("SOURCE_CURRENCY", "=", E::SQLFunctionInput),
                        seq!("TARGET_CURRENCY", "=", E::SQLFunctionInput),
                        seq!("EXCHANGE_RATE_DATE", "=", E::SQLFunctionInput),
                        seq!("EXCHANGE_RATE_TYPE", "=", E::SQLFunctionInput),
                        seq!("CLIENT", "=", E::SQLFunctionInput),
                        seq!(
                            "ROUND",
                            "=",
                            alt_prio!(
                                E::Constant,
                                seq!(
                                    Tok::WAt,
                                    alt_prio!(
                                        E::SimpleSource3,
                                        seq!(
                                            Tok::ParenLeftW,
                                            E::SQLFunctionInput,
                                            Tok::WParenRightW
                                        )
                                    )
                                )
                            )
                        ),
                        seq!(
                            "DECIMAL_SHIFT",
                            "=",
                            alt_prio!(
                                E::Constant,
                                seq!(
                                    Tok::WAt,
                                    alt_prio!(
                                        E::SimpleSource3,
                                        seq!(
                                            Tok::ParenLeftW,
                                            E::SQLFunctionInput,
                                            Tok::WParenRightW
                                        )
                                    )
                                )
                            )
                        ),
                        seq!(
                            "DECIMAL_SHIFT_BACK",
                            "=",
                            alt_prio!(
                                E::Constant,
                                seq!(
                                    Tok::WAt,
                                    alt_prio!(
                                        E::SimpleSource3,
                                        seq!(
                                            Tok::ParenLeftW,
                                            E::SQLFunctionInput,
                                            Tok::WParenRightW
                                        )
                                    )
                                )
                            )
                        ),
                        seq!(
                            "ON_ERROR",
                            "=",
                            alt_prio!(
                                E::Constant,
                                seq!(
                                    Tok::WAt,
                                    alt_prio!(
                                        E::SimpleSource3,
                                        seq!(
                                            Tok::ParenLeftW,
                                            E::SQLFunctionInput,
                                            Tok::WParenRightW
                                        )
                                    )
                                )
                            )
                        )
                    ),
                    star_prio(seq!(
                        ",",
                        alt_prio!(
                            seq!("AMOUNT", "=", E::SQLFunctionInput),
                            seq!("SOURCE_CURRENCY", "=", E::SQLFunctionInput),
                            seq!("TARGET_CURRENCY", "=", E::SQLFunctionInput),
                            seq!("EXCHANGE_RATE_DATE", "=", E::SQLFunctionInput),
                            seq!("EXCHANGE_RATE_TYPE", "=", E::SQLFunctionInput),
                            seq!("CLIENT", "=", E::SQLFunctionInput),
                            seq!(
                                "ROUND",
                                "=",
                                alt_prio!(
                                    E::Constant,
                                    seq!(
                                        Tok::WAt,
                                        alt_prio!(
                                            E::SimpleSource3,
                                            seq!(
                                                Tok::ParenLeftW,
                                                E::SQLFunctionInput,
                                                Tok::WParenRightW
                                            )
                                        )
                                    )
                                )
                            ),
                            seq!(
                                "DECIMAL_SHIFT",
                                "=",
                                alt_prio!(
                                    E::Constant,
                                    seq!(
                                        Tok::WAt,
                                        alt_prio!(
                                            E::SimpleSource3,
                                            seq!(
                                                Tok::ParenLeftW,
                                                E::SQLFunctionInput,
                                                Tok::WParenRightW
                                            )
                                        )
                                    )
                                )
                            ),
                            seq!(
                                "DECIMAL_SHIFT_BACK",
                                "=",
                                alt_prio!(
                                    E::Constant,
                                    seq!(
                                        Tok::WAt,
                                        alt_prio!(
                                            E::SimpleSource3,
                                            seq!(
                                                Tok::ParenLeftW,
                                                E::SQLFunctionInput,
                                                Tok::WParenRightW
                                            )
                                        )
                                    )
                                )
                            ),
                            seq!(
                                "ON_ERROR",
                                "=",
                                alt_prio!(
                                    E::Constant,
                                    seq!(
                                        Tok::WAt,
                                        alt_prio!(
                                            E::SimpleSource3,
                                            seq!(
                                                Tok::ParenLeftW,
                                                E::SQLFunctionInput,
                                                Tok::WParenRightW
                                            )
                                        )
                                    )
                                )
                            )
                        )
                    ))
                )),
                Tok::WParenRightW
            )
        ),
        ver(
            "7.58",
            seq!(
                "UNIT_CONVERSION",
                Tok::ParenLeftW,
                opt_prio(seq!(
                    alt_prio!(
                        seq!("QUANTITY", "=", E::SQLFunctionInput),
                        seq!("SOURCE_UNIT", "=", E::SQLFunctionInput),
                        seq!("TARGET_UNIT", "=", E::SQLFunctionInput),
                        seq!("CLIENT", "=", E::SQLFunctionInput),
                        seq!(
                            "ON_ERROR",
                            "=",
                            alt_prio!(
                                E::Constant,
                                seq!(
                                    Tok::WAt,
                                    alt_prio!(
                                        E::SimpleSource3,
                                        seq!(
                                            Tok::ParenLeftW,
                                            E::SQLFunctionInput,
                                            Tok::WParenRightW
                                        )
                                    )
                                )
                            )
                        )
                    ),
                    star_prio(seq!(
                        ",",
                        alt_prio!(
                            seq!("QUANTITY", "=", E::SQLFunctionInput),
                            seq!("SOURCE_UNIT", "=", E::SQLFunctionInput),
                            seq!("TARGET_UNIT", "=", E::SQLFunctionInput),
                            seq!("CLIENT", "=", E::SQLFunctionInput),
                            seq!(
                                "ON_ERROR",
                                "=",
                                alt_prio!(
                                    E::Constant,
                                    seq!(
                                        Tok::WAt,
                                        alt_prio!(
                                            E::SimpleSource3,
                                            seq!(
                                                Tok::ParenLeftW,
                                                E::SQLFunctionInput,
                                                Tok::WParenRightW
                                            )
                                        )
                                    )
                                )
                            )
                        )
                    ))
                )),
                Tok::WParenRightW
            )
        ),
        ver(
            "7.58",
            seq!(
                "CURR_TO_DECFLOAT_AMOUNT",
                Tok::ParenLeftW,
                opt_prio(seq!(
                    alt_prio!(
                        seq!("CURR", "=", E::SQLFunctionInput),
                        seq!("CUKY", "=", E::SQLFunctionInput)
                    ),
                    star_prio(seq!(
                        ",",
                        alt_prio!(
                            seq!("CURR", "=", E::SQLFunctionInput),
                            seq!("CUKY", "=", E::SQLFunctionInput)
                        )
                    ))
                )),
                Tok::WParenRightW
            )
        ),
        ver(
            "7.58",
            seq!(
                "LANG_TO_CHAR",
                Tok::ParenLeftW,
                opt_prio(seq!(
                    seq!("LANG", "=", E::SQLFunctionInput),
                    star_prio(seq!(",", seq!("LANG", "=", E::SQLFunctionInput)))
                )),
                Tok::WParenRightW
            )
        ),
        ver(
            "7.58",
            seq!(
                "IPOW",
                Tok::ParenLeftW,
                opt_prio(seq!(
                    alt_prio!(
                        seq!("BASE", "=", E::SQLFunctionInput),
                        seq!("EXP", "=", E::SQLFunctionInput)
                    ),
                    star_prio(seq!(
                        ",",
                        alt_prio!(
                            seq!("BASE", "=", E::SQLFunctionInput),
                            seq!("EXP", "=", E::SQLFunctionInput)
                        )
                    ))
                )),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_NEW_POINT",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_NEW_POINTZ",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_NEW_POINTM",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_NEW_POINTZM",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_NEW_LINESTRING",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_NEW_MULTILINESTRING",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_NEW_POLYGON",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_NEW_MULTIPOLYGON",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_NEW_MULTIPOINT",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_NEW_CIRCULARSTRING",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_NEW_GEOMETRYCOLLECTION",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_SRID",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_RELATE",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_GEOMFROMTEXT",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_GEOMFROMWKT",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_GEOMFROMEWKT",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_GEOMFROMWKB",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_GEOMFROMEWKB",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_GEOMFROMGEOJSON",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_GEOMFROMESRIJSON",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_GEOMFROMGEOHASH",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_POINTFROMTEXT",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_POINTFROMGEOHASH",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_MAKELINE",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_MAKEPOLYGON",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_X",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_Y",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_Z",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_M",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_XMAX",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_XMIN",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_YMAX",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_YMIN",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_ZMAX",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_ZMIN",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_MMAX",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_MMIN",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_AREA",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_LENGTH",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_PERIMETER",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_DIMENSION",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_COORDDIM",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_GEOMETRYTYPE",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_ISEMPTY",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_ISSIMPLE",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_ISCLOSED",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_ISRING",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_ISVALID",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_ISVALIDTRAJECTORY",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_IS3D",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_ISMEASURED",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_INVALIDREASON",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_NUMPOINTS",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_NUMGEOMETRIES",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_NUMINTERIORRING",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_NUMINTERIORRINGS",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_ENVELOPE",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_BOUNDARY",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_EXTERIORRING",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_STARTPOINT",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_ENDPOINT",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_CENTROID",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_POINTONSURFACE",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_CONVEXHULL",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_CONCAVEHULL",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_REVERSE",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_FORCE2D",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_FORCE3DZ",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_FORCE3DM",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_FORCE4D",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_MAKEVALID",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_GEOHASH",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_ASWKT",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_ASWKB",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_ASEWKT",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_ASEWKB",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_ASBINARY",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_ASTEXT",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_ASGEOJSON",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_ASESRIJSON",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_ASSVG",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_DISTANCE",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_WITHINDISTANCE",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_CONTAINS",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_COVERS",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_COVEREDBY",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_CROSSES",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_DISJOINT",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_EQUALS",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_ORDERINGEQUALS",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_INTERSECTS",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_INTERSECTSFILTER",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_INTERSECTSRECT",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_INTERSECTSRECTPLANAR",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_OVERLAPS",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_TOUCHES",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_WITHIN",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_INTERSECTION",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_UNION",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_DIFFERENCE",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_SYMDIFFERENCE",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_COLLECT",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_BUFFER",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_TRANSFORM",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_ROTATE",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_SCALE",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_TRANSLATE",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_TRANSLATE3D",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_SIMPLIFY",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_SNAPTOGRID",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_ADDMEASURE",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_ADDPOINT",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_REMOVEPOINT",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_POINTN",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_GEOMETRYN",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_INTERIORRINGN",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_LINEINTERPOLATEPOINT",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_LINELOCATEPOINT",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_LINESUBSTRING",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_LOCATEALONG",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_LOCATEBETWEEN",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_ALPHASHAPE",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_ALPHASHAPEAREA",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_ALPHASHAPEEDGE",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_CLOSESTPOINTOFAPPROACH",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_FRECHETDISTANCE",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_HAUSDORFFDISTANCE",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_3DLENGTH",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_ASMVTGEOM",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_ASSVGAGGR",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_COLLECTAGGR",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_CONCAVEHULLAGGR",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_CONVEXHULLAGGR",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_ENVELOPEAGGR",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_INTERSECTIONAGGR",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_UNIONAGGR",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_ALPHASHAPEAGGR",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_ALPHASHAPEAREAAGGR",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        ),
        ver(
            "8.16",
            seq!(
                "ST_ALPHASHAPEEDGEAGGR",
                Tok::ParenLeftW,
                E::SQLFunctionInput,
                star_prio(seq!(",", E::SQLFunctionInput)),
                Tok::WParenRightW
            )
        )
    )
}

fn sql_function_input() -> Rule {
    seq!(
        alt_prio!(
            seq!("(", E::SQLFunctionInput, ")"),
            E::SQLCase,
            E::SQLFunction,
            E::SQLAggregation,
            // before the name a typed literal starts with
            E::SQLTypedLiteral,
            // a path, `coalesce( _assoc-col, ... )`
            E::SQLPathForColumn,
            E::SQLFieldName,
            E::SQLAliasField,
            E::Constant,
            ver(
                "7.40 SP05",
                seq!(
                    Tok::WAt,
                    alt_prio!(
                        E::SimpleSource3,
                        seq!(Tok::ParenLeftW, E::Source, Tok::WParenRightW)
                    )
                )
            )
        ),
        star_prio(seq!(
            alt_prio!("+", "-", "*", "/", "&&"),
            alt_prio!(
                seq!("(", E::SQLFunctionInput, ")"),
                E::SQLCase,
                E::SQLFunction,
                E::SQLAggregation,
                E::SQLFieldName,
                E::SQLAliasField,
                E::Constant,
                ver(
                    "7.40 SP05",
                    seq!(
                        Tok::WAt,
                        alt_prio!(
                            E::SimpleSource3,
                            seq!(Tok::ParenLeftW, E::Source, Tok::WParenRightW)
                        )
                    )
                )
            )
        ))
    )
}

fn sql_group_by() -> Rule {
    alt_prio!(
        ver("7.53", seq!("GROUP BY", E::SQLGroupingSets)),
        ver(
            "7.40 SP02",
            seq!(
                "GROUP BY",
                ver("7.40 SP02", E::SQLField),
                alt_prio!(
                    plus(seq!(",", ver("7.40 SP02", E::SQLField))),
                    opt_prio(plus(seq!(
                        stop_before("ORDER", "BY"),
                        alt!(E::SQLFieldName, E::Dynamic)
                    )))
                )
            )
        ),
        seq!(
            "GROUP BY",
            alt_prio!(
                seq!(
                    plus(seq!(
                        seq!(
                            stop_before("ORDER", "BY"),
                            alt!(E::SQLFieldName, E::Dynamic)
                        ),
                        ","
                    )),
                    seq!(
                        stop_before("ORDER", "BY"),
                        alt!(E::SQLFieldName, E::Dynamic)
                    )
                ),
                plus(seq!(
                    stop_before("ORDER", "BY"),
                    alt!(E::SQLFieldName, E::Dynamic)
                ))
            )
        )
    )
}

fn sql_grouping_sets() -> Rule {
    seq!(
        "GROUPING SETS",
        Tok::WParenLeftW,
        seq!(
            seq!(
                Tok::WParenLeftW,
                opt_prio(seq!(E::SQLField, star_prio(seq!(",", E::SQLField)))),
                Tok::WParenRightW
            ),
            star_prio(seq!(
                ",",
                seq!(
                    Tok::WParenLeftW,
                    opt_prio(seq!(E::SQLField, star_prio(seq!(",", E::SQLField)))),
                    Tok::WParenRightW
                )
            ))
        ),
        Tok::WParenRightW
    )
}

fn sql_having() -> Rule {
    seq!("HAVING", E::SQLCond)
}

fn sql_hierarchy_accessor() -> Rule {
    ver(
        "7.50",
        seq!(
            alt_prio!(
                "HIERARCHY_DESCENDANTS",
                "HIERARCHY_ANCESTORS",
                "HIERARCHY_SIBLINGS"
            ),
            Tok::ParenLeftW,
            seq!(
                "SOURCE",
                E::DatabaseTable,
                "START",
                "WHERE",
                E::SQLCond,
                opt_prio(seq!(
                    "DISTANCE",
                    alt_prio!(
                        seq!("FROM", E::SQLSource, opt_prio(seq!("TO", E::SQLSource))),
                        seq!("TO", E::SQLSource)
                    )
                ))
            ),
            Tok::WParenRightW
        ),
    )
}

fn sql_hierarchy_aggregate() -> Rule {
    ver(
        "7.50",
        seq!(
            alt_prio!(
                "HIERARCHY_DESCENDANTS_AGGREGATE",
                "HIERARCHY_ANCESTORS_AGGREGATE"
            ),
            Tok::ParenLeftW,
            seq!(
                "SOURCE",
                E::DatabaseTable,
                opt_prio(seq!("START", "WHERE", E::SQLCond)),
                opt_prio(seq!("JOIN", E::DatabaseTable, "ON", E::SQLCond)),
                seq!(
                    "MEASURES",
                    seq!(E::SQLAggregation, "AS", E::SQLAsName),
                    star(seq!(",", seq!(E::SQLAggregation, "AS", E::SQLAsName)))
                ),
                opt_prio(seq!("WHERE", E::SQLCond)),
                opt_prio(seq!(
                    "WITH",
                    alt_prio!("SUBTOTAL", "BALANCE", seq!("NOT", "MATCHED"), "TOTAL"),
                    star(seq!(
                        "WITH",
                        alt_prio!("SUBTOTAL", "BALANCE", seq!("NOT", "MATCHED"), "TOTAL")
                    ))
                ))
            ),
            Tok::WParenRightW
        ),
    )
}

fn sql_hierarchy_source() -> Rule {
    ver(
        "7.50",
        seq!(
            "HIERARCHY",
            Tok::ParenLeftW,
            "SOURCE",
            seq!(
                alt_prio!(E::WithName, E::DatabaseTable),
                opt_prio(seq!("AS", E::SQLAsName))
            ),
            alt_prio!(
                seq!(
                    seq!(
                        "LEVELS",
                        Tok::WParenLeftW,
                        E::SQLFieldName,
                        star(seq!(",", E::SQLFieldName)),
                        Tok::WParenRightW
                    ),
                    opt_prio(seq!(
                        "SIBLINGS",
                        "ORDER",
                        "BY",
                        alt_prio!(
                            "PRIMARY KEY",
                            E::Dynamic,
                            seq!(
                                seq!(
                                    re_not(
                                        r"(?i)^(\/\w+\/)?(\*?\w+~(\/\w+\/)?(\w+|\*)|\w+)$",
                                        &[
                                            "DEPTH", "MULTIPLE", "ORPHAN", "CACHE", "CYCLE",
                                            "LOAD", "GENERATE", "RETAIN"
                                        ]
                                    ),
                                    opt_prio(alt_prio!("ASCENDING", "DESCENDING")),
                                    opt_prio(",")
                                ),
                                star_prio(seq!(
                                    re_not(
                                        r"(?i)^(\/\w+\/)?(\*?\w+~(\/\w+\/)?(\w+|\*)|\w+)$",
                                        &[
                                            "DEPTH", "MULTIPLE", "ORPHAN", "CACHE", "CYCLE",
                                            "LOAD", "GENERATE", "RETAIN"
                                        ]
                                    ),
                                    opt_prio(alt_prio!("ASCENDING", "DESCENDING")),
                                    opt_prio(",")
                                ))
                            )
                        )
                    )),
                    opt_prio(seq!("CACHE", alt_prio!("FORCE", "OFF", "ON"))),
                    opt_prio(seq!("RETAIN", "NULLS", E::SQLSource))
                ),
                seq!(
                    seq!("CHILD", "TO", "PARENT", "ASSOCIATION", E::SQLFieldName),
                    opt_prio(seq!(
                        seq!("PERIOD", "FROM", E::SQLFieldName, "TO", E::SQLFieldName),
                        seq!("VALID", "FROM", E::SQLSource, "TO", E::SQLSource)
                    )),
                    seq!("START", "WHERE", E::SQLCond),
                    opt_prio(seq!(
                        "SIBLINGS",
                        "ORDER",
                        "BY",
                        alt_prio!(
                            "PRIMARY KEY",
                            E::Dynamic,
                            seq!(
                                seq!(
                                    re_not(
                                        r"(?i)^(\/\w+\/)?(\*?\w+~(\/\w+\/)?(\w+|\*)|\w+)$",
                                        &[
                                            "DEPTH", "MULTIPLE", "ORPHAN", "CACHE", "CYCLE",
                                            "LOAD", "GENERATE", "RETAIN"
                                        ]
                                    ),
                                    opt_prio(alt_prio!("ASCENDING", "DESCENDING")),
                                    opt_prio(",")
                                ),
                                star_prio(seq!(
                                    re_not(
                                        r"(?i)^(\/\w+\/)?(\*?\w+~(\/\w+\/)?(\w+|\*)|\w+)$",
                                        &[
                                            "DEPTH", "MULTIPLE", "ORPHAN", "CACHE", "CYCLE",
                                            "LOAD", "GENERATE", "RETAIN"
                                        ]
                                    ),
                                    opt_prio(alt_prio!("ASCENDING", "DESCENDING")),
                                    opt_prio(",")
                                ))
                            )
                        )
                    )),
                    opt_prio(seq!("DEPTH", E::SQLSource)),
                    opt_prio(seq!(
                        "MULTIPLE",
                        "PARENTS",
                        alt_prio!(seq!("NOT", "ALLOWED"), seq!("LEAVES", "ONLY"), "ALLOWED")
                    )),
                    opt_prio(seq!("ORPHAN", alt_prio!("IGNORE", "ERROR", "ROOT"))),
                    opt_prio(seq!("CACHE", alt_prio!("FORCE", "OFF", "ON"))),
                    opt_prio(seq!("CYCLE", alt_prio!("BREAKUP", "ERROR"))),
                    opt_prio(seq!("LOAD", alt_prio!("INCREMENTAL", "BULK", E::SQLSource))),
                    opt_prio(seq!("GENERATE", "SPANTREE"))
                )
            ),
            Tok::WParenRightW
        ),
    )
}

fn sql_hints() -> Rule {
    seq!(
        "%_HINTS",
        plus(seq!(
            alt_prio!(
                "ORACLE", "ADABAS", "AS400", "DB2", "HDB", "MSSQLNT", "SYBASE", "DB6", "INFORMIX"
            ),
            E::SQLSource
        ))
    )
}

fn sql_in() -> Rule {
    seq!(
        "IN",
        alt_prio!(
            E::SQLSetOpGroup,
            seq!(
                "(",
                "SELECT",
                alt_prio!(
                    seq!(
                        "SINGLE",
                        seq!(
                            opt_prio(E::SQLFieldList),
                            seq!(
                                E::SQLFrom,
                                opt_prio(E::SQLClient),
                                opt_prio(E::SQLBypassingBuffer),
                                opt_prio(seq!("WHERE", E::SQLCond)),
                                seq!(opt_prio(E::SQLGroupBy), opt_prio(E::SQLHaving)),
                                seq!(
                                    opt_prio(not_in(Lang::KeyUser, E::DatabaseConnection)),
                                    opt_prio(E::SQLHints),
                                    opt_prio(ver("7.58", E::SQLPrivilegedAccess)),
                                    opt_prio(E::SQLOptions)
                                )
                            )
                        )
                    ),
                    seq!(
                        opt_prio("DISTINCT"),
                        opt_prio(E::SQLFieldList),
                        opt_prio(E::SQLBypassingBuffer),
                        seq!(
                            E::SQLFrom,
                            opt_prio(E::SQLClient),
                            opt_prio(E::SQLBypassingBuffer),
                            seq!(
                                opt_prio(ver("7.50", E::SQLFields)),
                                opt_prio(E::SQLForAllEntries),
                                opt_prio(seq!("WHERE", E::SQLCond)),
                                seq!(opt_prio(E::SQLGroupBy), opt_prio(E::SQLHaving)),
                                seq!(
                                    opt_prio(not_in(Lang::KeyUser, E::DatabaseConnection)),
                                    opt_prio(E::SQLHints),
                                    opt_prio(ver("7.58", E::SQLPrivilegedAccess)),
                                    opt_prio(E::SQLOptions)
                                )
                            )
                        )
                    )
                ),
                ")"
            ),
            seq!(
                Tok::WParenLeft,
                alt!(ver("7.40 SP05", E::SQLSourceNoSpace), E::SQLSource),
                star_prio(seq!(",", E::SQLSource)),
                alt_prio!(Tok::ParenRight, Tok::ParenRightW, Tok::WParenRightW)
            ),
            ver(
                "7.40 SP02",
                seq!(
                    Tok::WParenLeftW,
                    E::SQLSource,
                    star_prio(seq!(",", alt_prio!(E::SQLSourceNoSpace, E::SQLSource))),
                    alt_prio!(Tok::WParenRight, Tok::WParenRightW)
                )
            ),
            E::SQLSource
        )
    )
}

fn sql_indicators() -> Rule {
    ver(
        "8.16",
        seq!(
            "INDICATORS",
            alt_prio!(
                seq!(
                    opt_prio("NOT"),
                    "NULL",
                    alt!(
                        seq!("STRUCTURE", re(r"^%?\w+$")),
                        seq!("BITFIELD", re(r"^%?\w+$"))
                    )
                ),
                seq!(
                    alt_prio!(Tok::WParenLeftW, Tok::WParenLeft),
                    re(r"^%?\w+$"),
                    alt_prio!(Tok::WParenRightW, Tok::ParenRightW)
                )
            )
        ),
    )
}

fn sql_into_list() -> Rule {
    seq!(
        "INTO",
        seq!(
            alt_prio!(Tok::WParenLeft, ver("7.40 SP02", Tok::WParenLeftW)),
            star_prio(seq!(E::SQLTarget, ",")),
            E::SQLTarget,
            ")"
        )
    )
}

fn sql_into_structure() -> Rule {
    seq!(
        "INTO",
        seq!(opt_prio("CORRESPONDING FIELDS OF"), E::SQLTarget),
        opt_prio(ver("8.16", E::SQLIndicators))
    )
}

fn sql_into_table() -> Rule {
    seq!(
        alt_prio!("INTO", "APPENDING"),
        opt_prio("CORRESPONDING FIELDS OF"),
        "TABLE",
        E::SQLTarget,
        opt_prio(ver("8.16", E::SQLIndicators))
    )
}

fn sql_join() -> Rule {
    alt_prio!(
        ver(
            "7.50",
            seq!(
                "CROSS JOIN",
                opt_prio(ver("8.16", re(r"(?i)^#[A-Z][A-Z0-9_]*$"))),
                E::SQLJoinSource
            )
        ),
        seq!(
            seq!(
                opt_prio(alt_prio!(
                    "INNER",
                    "LEFT OUTER",
                    "LEFT",
                    "RIGHT OUTER",
                    "RIGHT"
                )),
                "JOIN"
            ),
            opt_prio(ver("8.16", re(r"(?i)^#[A-Z][A-Z0-9_]*$"))),
            E::SQLJoinSource,
            "ON",
            E::SQLCond
        )
    )
}

fn sql_join_source() -> Rule {
    alt_prio!(
        seq!(
            Tok::WParenLeftW,
            alt_prio!(E::SQLJoinSource, E::SQLFromSource),
            star_prio(seq!(opt_prio(Tok::WParenRightW), E::SQLJoin)),
            opt_prio(Tok::WParenRightW)
        ),
        E::SQLFromSource
    )
}

fn sql_like_regexpr() -> Rule {
    ver(
        "7.57",
        seq!(
            "LIKE_REGEXPR",
            E::SQLFunctionInput,
            opt_prio(seq!(
                "FLAG",
                alt_prio!(
                    seq!(
                        Tok::WAt,
                        alt_prio!(
                            E::SimpleSource3,
                            seq!(Tok::ParenLeftW, E::Source, Tok::WParenRightW)
                        )
                    ),
                    E::SQLFunctionInput
                )
            ))
        ),
    )
}

fn sql_locate_regexpr() -> Rule {
    ver(
        "7.58",
        seq!(
            "LOCATE_REGEXPR",
            Tok::ParenLeftW,
            opt_prio(alt_prio!("START", "AFTER")),
            E::SQLRegexprPattern,
            opt_prio(E::SQLRegexprFrom),
            opt_prio(E::SQLRegexprOccurrence),
            opt_prio(E::SQLRegexprGroup),
            Tok::WParenRightW
        ),
    )
}

fn sql_mapping_from_entity() -> Rule {
    ver("7.57", "MAPPING FROM ENTITY")
}

fn sql_occurrences_regexpr() -> Rule {
    ver(
        "7.57",
        seq!(
            "OCCURRENCES_REGEXPR",
            Tok::ParenLeftW,
            E::SQLRegexprPattern,
            opt_prio(E::SQLRegexprFrom),
            Tok::WParenRightW
        ),
    )
}

fn sql_offset() -> Rule {
    seq!("OFFSET", E::SQLSource)
}

fn sql_options() -> Rule {
    ver(
        "8.16",
        seq!(
            "OPTIONS",
            alt!(
                seq!(
                    "USING",
                    alt!(
                        seq!("CLIENTS IN", alt!(E::SQLSourceSimple, "T000")),
                        "ALL CLIENTS",
                        seq!("CLIENT", E::SQLSourceSimple)
                    ),
                    opt_prio(per!(
                        E::SQLPrivilegedAccess,
                        E::SQLBypassingBuffer,
                        E::DatabaseConnection
                    ))
                ),
                per!(
                    E::SQLPrivilegedAccess,
                    E::SQLBypassingBuffer,
                    E::DatabaseConnection
                )
            )
        ),
    )
}

fn sql_order_by() -> Rule {
    seq!(
        "ORDER BY",
        alt_prio!(
            "PRIMARY KEY",
            E::Dynamic,
            plus(seq!(
                alt_prio!(ver("7.57", E::SQLField), E::SQLFieldName),
                opt(alt!("ASCENDING", "DESCENDING")),
                opt(ver("7.55", seq!("NULLS", alt!("FIRST", "LAST")))),
                // after a comma a column may be named like a keyword,
                // `ORDER BY name, connection`
                opt(seq!(",", opt_prio(re(r"(?i)^CONNECTION$"))))
            ))
        )
    )
}

fn sql_over() -> Rule {
    ver(
        "7.57",
        seq!(
            "OVER",
            Tok::ParenLeftW,
            opt_prio(seq!(
                "PARTITION",
                "BY",
                plus(seq!(E::SQLFunctionInput, opt(",")))
            )),
            opt_prio(E::SQLOrderBy),
            opt_prio(ver(
                "7.57",
                seq!(
                    "ROWS",
                    "BETWEEN",
                    alt_prio!(
                        seq!("UNBOUNDED", "PRECEDING"),
                        seq!("UNBOUNDED", "FOLLOWING"),
                        seq!("CURRENT", "ROW"),
                        seq!(E::SQLSource, alt_prio!("PRECEDING", "FOLLOWING"))
                    ),
                    "AND",
                    alt_prio!(
                        seq!("UNBOUNDED", "PRECEDING"),
                        seq!("UNBOUNDED", "FOLLOWING"),
                        seq!("CURRENT", "ROW"),
                        seq!(E::SQLSource, alt_prio!("PRECEDING", "FOLLOWING"))
                    )
                )
            )),
            alt_prio!(Tok::WParenRightW, Tok::WParenRight)
        ),
    )
}

fn sql_package_size() -> Rule {
    seq!("PACKAGE SIZE", E::SQLSource)
}

/// The cardinality of a path, `( 1 )` or `(*)`: with or without blanks inside
/// the parentheses, as SAP's cheat sheets write it.
fn sql_path_cardinality() -> Rule {
    seq!(
        alt_prio!(Tok::WParenLeftW, Tok::WParenLeft),
        alt_prio!(re(r"^\d+$"), "*"),
        alt_prio!(Tok::WParenRightW, Tok::ParenRightW)
    )
}

fn sql_path_for_column() -> Rule {
    ver(
        "7.40 SP05",
        alt_prio!(
            seq!(
                re(r"^(\/\w+\/)?\w+~$"),
                plus(E::SQLPathSegmentNws),
                Tok::Dash,
                E::Field
            ),
            seq!(plus(E::SQLPathSegment), Tok::Dash, E::Field)
        ),
    )
}

fn sql_path_for_entity() -> Rule {
    ver(
        "7.40 SP05",
        seq!(
            opt_prio(alt_prio!(E::SQLCTEPathPrefix, E::DatabaseTable)),
            plus(E::SQLPathSegment)
        ),
    )
}

fn sql_path_for_entity_no_prefix() -> Rule {
    ver("7.40 SP05", plus(E::SQLPathSegment))
}

fn sql_path_join_type() -> Rule {
    alt_prio!(
        seq!(
            alt_prio!("INNER", "LEFT OUTER", "RIGHT OUTER"),
            opt_prio(seq!(
                alt_prio!("EXACT ONE", "MANY", "ONE"),
                "TO",
                alt_prio!("EXACT ONE", "MANY", "ONE")
            ))
        ),
        seq!(
            alt_prio!("EXACT ONE", "MANY", "ONE"),
            "TO",
            alt_prio!("EXACT ONE", "MANY", "ONE")
        )
    )
}

fn sql_path_segment() -> Rule {
    ver(
        "7.40 SP05",
        seq!(
            alt_prio!(Tok::AssociationName, re(r"^\\[\w]+$")),
            opt_prio(alt_prio!(
                seq!(
                    ver("7.51", E::SQLCDSParameters),
                    opt_prio(ver(
                        "7.51",
                        seq!(
                            Tok::BracketLeftW,
                            opt_prio(ver("8.16", re(r"(?i)^#[A-Z][A-Z0-9_]*$"))),
                            opt_prio(E::SQLPathCardinality),
                            opt_prio(E::SQLPathJoinType),
                            opt_prio(seq!(opt_prio("WHERE"), E::SQLCond)),
                            alt_prio!(Tok::WBracketRightW, Tok::WBracketRight)
                        )
                    ))
                ),
                ver(
                    "7.51",
                    seq!(
                        Tok::BracketLeftW,
                        opt_prio(ver("8.16", re(r"(?i)^#[A-Z][A-Z0-9_]*$"))),
                        opt_prio(E::SQLPathCardinality),
                        opt_prio(E::SQLPathJoinType),
                        opt_prio(seq!(opt_prio("WHERE"), E::SQLCond)),
                        alt_prio!(Tok::WBracketRightW, Tok::WBracketRight)
                    )
                )
            ))
        ),
    )
}

fn sql_path_segment_nws() -> Rule {
    ver(
        "7.40 SP05",
        seq!(
            Tok::AssociationName,
            opt_prio(alt_prio!(
                seq!(
                    ver("7.51", E::SQLCDSParameters),
                    opt_prio(ver(
                        "7.51",
                        seq!(
                            Tok::BracketLeftW,
                            opt_prio(ver("8.16", re(r"(?i)^#[A-Z][A-Z0-9_]*$"))),
                            opt_prio(E::SQLPathCardinality),
                            opt_prio(E::SQLPathJoinType),
                            opt_prio(seq!(opt_prio("WHERE"), E::SQLCond)),
                            alt_prio!(Tok::WBracketRightW, Tok::WBracketRight)
                        )
                    ))
                ),
                ver(
                    "7.51",
                    seq!(
                        Tok::BracketLeftW,
                        opt_prio(ver("8.16", re(r"(?i)^#[A-Z][A-Z0-9_]*$"))),
                        opt_prio(E::SQLPathCardinality),
                        opt_prio(E::SQLPathJoinType),
                        opt_prio(seq!(opt_prio("WHERE"), E::SQLCond)),
                        alt_prio!(Tok::WBracketRightW, Tok::WBracketRight)
                    )
                )
            ))
        ),
    )
}

fn sql_privileged_access() -> Rule {
    seq!(
        "PRIVILEGED ACCESS",
        opt_prio(ver("after 8.16", seq!("LEVEL", E::SQLSourceSimple)))
    )
}

fn sql_provided_by() -> Rule {
    seq!(
        "PROVIDED BY",
        alt_prio!(E::DatabaseTableNoDynamic, re(r"^[\w\/]+$"))
    )
}

fn sql_regexpr_from() -> Rule {
    seq!(
        "FROM",
        alt_prio!(
            seq!(
                Tok::WAt,
                alt_prio!(
                    E::SimpleSource3,
                    seq!(Tok::ParenLeftW, E::Source, Tok::WParenRightW)
                )
            ),
            E::Integer
        )
    )
}

fn sql_regexpr_group() -> Rule {
    seq!(
        "GROUP",
        alt_prio!(
            seq!(
                Tok::WAt,
                alt_prio!(
                    E::SimpleSource3,
                    seq!(Tok::ParenLeftW, E::Source, Tok::WParenRightW)
                )
            ),
            E::Integer
        )
    )
}

fn sql_regexpr_occurrence() -> Rule {
    seq!(
        "OCCURRENCE",
        alt_prio!(
            seq!(
                Tok::WAt,
                alt_prio!(
                    E::SimpleSource3,
                    seq!(Tok::ParenLeftW, E::Source, Tok::WParenRightW)
                )
            ),
            E::Integer,
            "ALL"
        )
    )
}

fn sql_regexpr_pattern() -> Rule {
    seq!(
        E::SQLFunctionInput,
        opt_prio(seq!(
            "FLAG",
            alt_prio!(
                seq!(
                    Tok::WAt,
                    alt_prio!(
                        E::SimpleSource3,
                        seq!(Tok::ParenLeftW, E::Source, Tok::WParenRightW)
                    )
                ),
                E::SQLFunctionInput
            )
        )),
        "IN",
        E::SQLFunctionInput
    )
}

fn sql_replace_regexpr() -> Rule {
    ver(
        "7.57",
        seq!(
            "REPLACE_REGEXPR",
            Tok::ParenLeftW,
            E::SQLRegexprPattern,
            opt_prio(seq!(
                "WITH",
                alt_prio!(
                    seq!(
                        Tok::WAt,
                        alt_prio!(
                            E::SimpleSource3,
                            seq!(Tok::ParenLeftW, E::Source, Tok::WParenRightW)
                        )
                    ),
                    E::SQLFunctionInput
                )
            )),
            opt_prio(E::SQLRegexprFrom),
            opt_prio(E::SQLRegexprOccurrence),
            Tok::WParenRightW
        ),
    )
}

fn sql_set_op() -> Rule {
    seq!(
        alt_prio!(
            ver(
                "7.50",
                seq!("UNION", opt_prio(alt_prio!("DISTINCT", "ALL")))
            ),
            ver(
                "7.56",
                alt_prio!(
                    seq!("INTERSECT", opt_prio("DISTINCT")),
                    seq!("EXCEPT", opt_prio("DISTINCT"))
                )
            )
        ),
        alt_prio!(
            E::SQLSetOpGroup,
            seq!(
                "SELECT",
                alt_prio!(
                    seq!(
                        "SINGLE",
                        seq!(
                            opt_prio(E::SQLFieldList),
                            seq!(
                                E::SQLFrom,
                                opt_prio(E::SQLClient),
                                opt_prio(E::SQLBypassingBuffer),
                                opt_prio(seq!("WHERE", E::SQLCond)),
                                seq!(opt_prio(E::SQLGroupBy), opt_prio(E::SQLHaving)),
                                seq!(
                                    opt_prio(not_in(Lang::KeyUser, E::DatabaseConnection)),
                                    opt_prio(E::SQLHints),
                                    opt_prio(ver("7.58", E::SQLPrivilegedAccess)),
                                    opt_prio(E::SQLOptions)
                                )
                            )
                        )
                    ),
                    seq!(
                        opt_prio("DISTINCT"),
                        opt_prio(E::SQLFieldList),
                        opt_prio(E::SQLBypassingBuffer),
                        seq!(
                            E::SQLFrom,
                            opt_prio(E::SQLClient),
                            opt_prio(E::SQLBypassingBuffer),
                            seq!(
                                opt_prio(ver("7.50", E::SQLFields)),
                                opt_prio(E::SQLForAllEntries),
                                opt_prio(seq!("WHERE", E::SQLCond)),
                                seq!(opt_prio(E::SQLGroupBy), opt_prio(E::SQLHaving)),
                                seq!(
                                    opt_prio(not_in(Lang::KeyUser, E::DatabaseConnection)),
                                    opt_prio(E::SQLHints),
                                    opt_prio(ver("7.58", E::SQLPrivilegedAccess)),
                                    opt_prio(E::SQLOptions)
                                )
                            )
                        )
                    )
                )
            )
        )
    )
}

fn sql_set_op_group() -> Rule {
    ver(
        "7.50",
        seq!(
            "(",
            seq!(
                alt_prio!(
                    E::SQLSetOpGroup,
                    seq!(
                        "SELECT",
                        alt_prio!(
                            seq!(
                                "SINGLE",
                                seq!(
                                    opt_prio(E::SQLFieldList),
                                    seq!(
                                        E::SQLFrom,
                                        opt_prio(E::SQLClient),
                                        opt_prio(E::SQLBypassingBuffer),
                                        opt_prio(seq!("WHERE", E::SQLCond)),
                                        seq!(opt_prio(E::SQLGroupBy), opt_prio(E::SQLHaving)),
                                        seq!(
                                            opt_prio(not_in(Lang::KeyUser, E::DatabaseConnection)),
                                            opt_prio(E::SQLHints),
                                            opt_prio(ver("7.58", E::SQLPrivilegedAccess)),
                                            opt_prio(E::SQLOptions)
                                        )
                                    )
                                )
                            ),
                            seq!(
                                opt_prio("DISTINCT"),
                                opt_prio(E::SQLFieldList),
                                opt_prio(E::SQLBypassingBuffer),
                                seq!(
                                    E::SQLFrom,
                                    opt_prio(E::SQLClient),
                                    opt_prio(E::SQLBypassingBuffer),
                                    seq!(
                                        opt_prio(ver("7.50", E::SQLFields)),
                                        opt_prio(E::SQLForAllEntries),
                                        opt_prio(seq!("WHERE", E::SQLCond)),
                                        seq!(opt_prio(E::SQLGroupBy), opt_prio(E::SQLHaving)),
                                        seq!(
                                            opt_prio(not_in(Lang::KeyUser, E::DatabaseConnection)),
                                            opt_prio(E::SQLHints),
                                            opt_prio(ver("7.58", E::SQLPrivilegedAccess)),
                                            opt_prio(E::SQLOptions)
                                        )
                                    )
                                )
                            )
                        )
                    )
                ),
                opt_prio(seq!(
                    ver("7.50", plus_prio(E::SQLSetOp)),
                    opt_prio(E::SQLOrderBy)
                ))
            ),
            ")"
        ),
    )
}

fn sql_source() -> Rule {
    alt_prio!(
        E::SQLTypedLiteral,
        E::SQLAliasField,
        E::SimpleSource3,
        ver(
            "7.40 SP05",
            seq!(
                Tok::WAt,
                alt_prio!(
                    E::SimpleSource3,
                    seq!(Tok::ParenLeftW, E::Source, Tok::WParenRightW)
                )
            )
        )
    )
}

fn sql_source_no_space() -> Rule {
    alt!(
        E::SQLAliasField,
        E::SimpleSource3,
        ver(
            "7.40 SP05",
            seq!(
                Tok::At,
                alt_prio!(
                    E::SimpleSource3,
                    seq!(Tok::ParenLeftW, E::Source, Tok::WParenRightW)
                )
            )
        )
    )
}

fn sql_source_simple() -> Rule {
    alt!(
        E::SimpleSource3,
        ver(
            "7.40 SP05",
            seq!(
                Tok::WAt,
                alt!(
                    E::SimpleSource3,
                    seq!(Tok::ParenLeftW, E::Source, Tok::WParenRightW)
                )
            )
        )
    )
}

fn sql_substring_regexpr() -> Rule {
    ver(
        "7.58",
        seq!(
            alt_prio!("SUBSTRING_REGEXPR", "SUBSTR_REGEXPR"),
            Tok::ParenLeftW,
            E::SQLRegexprPattern,
            opt_prio(E::SQLRegexprFrom),
            opt_prio(E::SQLRegexprOccurrence),
            opt_prio(E::SQLRegexprGroup),
            Tok::WParenRightW
        ),
    )
}

fn sql_target() -> Rule {
    alt_prio!(
        ver(
            "7.40 SP05",
            seq!(
                opt(ver("7.54", "NEW")),
                alt_prio!(Tok::WAt, Tok::At),
                E::Target
            )
        ),
        E::Target
    )
}

/// SAP `abenabap_sql_typed_literals`: a dictionary type right before a
/// backquote literal, with the DF16 and DF34 names too.
/// Typed literals came with 7.55 (release news).
fn sql_typed_literal() -> Rule {
    seq!(
        re(
            r"(?i)^(CHAR|CLNT|CUKY|CURR|DATN|DATS|DEC|DECFLOAT16|DECFLOAT34|D16N|D34N|D16D|D34D|D16R|D34R|DF16_RAW|DF34_RAW|DF16_DEC|DF34_DEC|FLTP|INT1|INT2|INT4|INT8|LANG|NUMC|QUAN|RAW|RAWSTRING|SSTRING|STRING|TIMN|TIMS|UNIT|UTCLONG)$"
        ),
        re(r"^`.*`$")
    )
}

fn sql_up_to() -> Rule {
    seq!("UP TO", E::SQLSource, "ROWS")
}

fn string_template() -> Rule {
    ver(
        "7.02",
        alt_prio!(
            seq!(
                Tok::StringTemplateBegin,
                E::StringTemplateSource,
                star_prio(seq!(Tok::StringTemplateMiddle, E::StringTemplateSource)),
                Tok::StringTemplateEnd
            ),
            Tok::StringTemplate
        ),
    )
}

/// SAP `abapcompute_string_format_options`: each option at most once, in any
/// order; which ones fit together depends on the type of the value.
fn string_template_formatting() -> Rule {
    // a keyword value, `(dobj)` or an expression
    let value = |words: &'static [&'static str]| {
        let mut list: Vec<Rule> = words.iter().map(|w| Rule::Word(w)).collect();
        list.push(E::Dynamic.into());
        list.push(E::Source.into());
        Rule::AltPrio(list)
    };
    per!(
        seq!("WIDTH =", E::Source),
        seq!("ALIGN =", value(&["LEFT", "RIGHT", "CENTER"])),
        seq!("PAD =", E::Source),
        seq!("CASE =", value(&["RAW", "UPPER", "LOWER"])),
        seq!(
            "SIGN =",
            value(&[
                "LEFT",
                "LEFTPLUS",
                "LEFTSPACE",
                "RIGHT",
                "RIGHTPLUS",
                "RIGHTSPACE"
            ])
        ),
        seq!("EXPONENT =", E::Source),
        seq!("DECIMALS =", E::Source),
        seq!("ZERO =", value(&["YES", "NO"])),
        ver("7.40 SP02", seq!("XSD =", value(&["YES", "NO"]))),
        seq!(
            "STYLE =",
            value(&[
                "SIMPLE",
                "SIGN_AS_POSTFIX",
                "SCALE_PRESERVING",
                "SCIENTIFIC",
                "SCIENTIFIC_WITH_LEADING_ZERO",
                "SCALE_PRESERVING_SCIENTIFIC",
                "ENGINEERING"
            ])
        ),
        seq!("CURRENCY =", E::Source),
        seq!("NUMBER =", value(&["RAW", "USER", "ENVIRONMENT"])),
        ver("7.40 SP02", seq!("ALPHA =", value(&["IN", "OUT", "RAW"]))),
        seq!("DATE =", value(&["RAW", "ISO", "USER", "ENVIRONMENT"])),
        seq!("TIME =", value(&["RAW", "ISO", "USER", "ENVIRONMENT"])),
        seq!(
            "TIMESTAMP =",
            value(&["SPACE", "ISO", "USER", "ENVIRONMENT"])
        ),
        seq!("TIMEZONE =", E::Source),
        seq!("COUNTRY =", E::Source)
    )
}

fn string_template_source() -> Rule {
    seq!(E::Source, opt_prio(E::StringTemplateFormatting))
}

fn super_class_name() -> Rule {
    Rule::from(E::ClassName)
}

fn switch_body() -> Rule {
    seq!(
        opt(E::Let),
        E::Source,
        plus(seq!(
            "WHEN",
            E::Source,
            star(seq!("OR", E::Source)),
            "THEN",
            alt!(E::Source, E::Throw)
        )),
        opt(seq!("ELSE", alt!(E::Source, E::Throw)))
    )
}

fn table_body() -> Rule {
    seq!(
        Tok::BracketLeft,
        alt_prio!(Tok::BracketRightW, Tok::BracketRight)
    )
}

/// SAP: `abentable_exp_itab_line`; the key name may be dynamic, `KEY ('SK')`;
/// TABLE KEY came with 8.16 (release news).
fn table_expression() -> Rule {
    let components = || {
        plus(seq!(
            alt_prio!(E::ComponentChainSimple, E::Dynamic),
            "=",
            E::Source
        ))
    };
    ver(
        "7.40 SP02",
        seq!(
            Tok::BracketLeftW,
            alt!(
                seq!(
                    "TABLE KEY",
                    alt_prio!(E::SimpleName, E::Dynamic),
                    opt("COMPONENTS"),
                    components()
                ),
                E::Source,
                seq!(
                    opt_prio(seq!("KEY", alt_prio!(E::SimpleName, E::Dynamic))),
                    opt("COMPONENTS"),
                    alt_prio!(components(), seq!("INDEX", E::Source))
                )
            ),
            alt_prio!(Tok::WBracketRight, Tok::WBracketRightW)
        ),
    )
}

fn target() -> Rule {
    alt_prio!(
        E::InlineData,
        E::InlineFS,
        seq!(
            alt_prio!(
                E::Cast,
                E::NewObject,
                seq!(E::ClassName, Tok::StaticArrow, E::AttributeName),
                E::TargetField,
                E::TargetFieldSymbol
            ),
            star_prio(alt_prio!(
                E::Dereference,
                ver(
                    "7.57",
                    seq!(
                        Tok::InstanceArrow,
                        Tok::ParenLeft,
                        alt_prio!(E::FieldChain, E::Constant),
                        alt_prio!(Tok::ParenRight, Tok::ParenRightW)
                    )
                ),
                seq!(Tok::InstanceArrow, E::AttributeName),
                ver(
                    "7.57",
                    seq!(
                        Tok::Dash,
                        Tok::ParenLeft,
                        alt_prio!(E::FieldChain, E::Constant),
                        alt_prio!(Tok::ParenRight, Tok::ParenRightW)
                    )
                ),
                seq!(Tok::Dash, E::ComponentName),
                E::TableExpression
            )),
            alt_prio!(
                E::TableBody,
                seq!(opt_prio(E::FieldOffset), opt_prio(E::FieldLength))
            )
        )
    )
}

fn target_field() -> Rule {
    Rule::from(E::Field)
}

fn target_field_symbol() -> Rule {
    seq!(E::FieldSymbol, opt_prio(E::TableBody))
}

fn test_seam_name() -> Rule {
    seq!(
        re(r"^[\w%\$\*]+$"),
        star_prio(seq!(Tok::Dash, re(r"^[\w%\$\*]+$")))
    )
}

fn text_element() -> Rule {
    seq!("TEXT", Tok::Dash, E::TextElementKey)
}

fn text_element_key() -> Rule {
    re(r"^\w{3}$")
}

fn text_element_string() -> Rule {
    seq!(
        re(r"^('.*')|(`.*`)$"),
        seq!(Tok::ParenLeft, E::TextElementKey, Tok::ParenRightW)
    )
}

/// SAP `abenconditional_expression_result`: `THROW [RESUMABLE|SHORTDUMP]
/// cx_class( [message] [p1 = a1 p2 = a2 ...] )`, message also `USING MESSAGE`;
/// MESSAGE came with 7.50 (release news).
fn throw() -> Rule {
    seq!(
        "THROW",
        opt(alt_prio!("RESUMABLE", "SHORTDUMP")),
        E::ClassName,
        alt_prio!(Tok::ParenLeftW, Tok::ParenLeft),
        opt(alt!(
            E::Source,
            E::ParameterListS,
            seq!(
                alt_prio!(
                    "USING MESSAGE",
                    seq!("MESSAGE", E::MessageSource, opt(E::RaiseWith))
                ),
                opt(E::ParameterListS)
            )
        )),
        alt_prio!(Tok::WParenRightW, Tok::ParenRightW)
    )
}

/// SAP: each component may be dynamic, `TRANSPORTING ('A') ('B')`.
fn transporting_fields() -> Rule {
    alt_prio!(
        plus(E::Dynamic),
        plus(alt!(seq!("INTO", fail_star()), E::FieldSub))
    )
}

fn type_() -> Rule {
    seq!(
        alt_prio!(
            seq!(
                "LIKE",
                alt_prio!(
                    seq!("LINE OF", E::FieldChain),
                    seq!("REF TO", E::FieldChain),
                    E::FieldChain
                )
            ),
            seq!(
                "TYPE",
                alt_prio!(
                    seq!("LINE OF", seq!(E::TypeName, opt_prio(E::Default))),
                    seq!("REF TO", seq!(E::TypeName, opt_prio(E::Default))),
                    seq!(
                        seq!(E::TypeName, opt_prio(E::Default)),
                        opt_prio(E::LOBHandle)
                    )
                )
            )
        ),
        opt_prio(ver(
            "7.55",
            seq!(
                "WITH INDICATORS",
                E::ComponentName,
                opt_prio(alt_prio!(
                    ver("7.56", "AS BITFIELD"),
                    seq!("TYPE", E::TypeName)
                ))
            )
        ))
    )
}

fn type_name() -> Rule {
    seq!(
        opt(seq!(
            re(r"^[\w~\/%$]+$"),
            alt!(Tok::StaticArrow, Tok::InstanceArrow)
        )),
        re(r"^[\w~\/%$]+$"),
        star_prio(seq!(Tok::Dash, re(r"^[\w~\/%$]+$")))
    )
}

fn type_name_or_infer() -> Rule {
    alt_prio!("#", E::TypeName)
}

fn type_param() -> Rule {
    alt_prio!(
        ver(
            "7.53",
            seq!(
                "TYPE",
                "TABLE FOR",
                alt_prio!(
                    ver(
                        "7.53",
                        seq!("ACTION IMPORT", alt!(E::TypeName, E::EntityAssociation))
                    ),
                    ver(
                        "7.54",
                        seq!("READ LINK", alt!(E::TypeName, E::EntityAssociation))
                    ),
                    ver(
                        "7.57",
                        seq!("EVENT", alt!(E::TypeName, E::EntityAssociation))
                    ),
                    ver(
                        "7.54",
                        seq!("FAILED LATE", alt!(E::TypeName, E::EntityAssociation))
                    ),
                    ver(
                        "7.54",
                        seq!("MAPPED LATE", alt!(E::TypeName, E::EntityAssociation))
                    ),
                    ver(
                        "7.54",
                        seq!("REPORTED LATE", alt!(E::TypeName, E::EntityAssociation))
                    ),
                    ver(
                        "7.54",
                        seq!("FAILED EARLY", alt!(E::TypeName, E::EntityAssociation))
                    ),
                    ver(
                        "7.54",
                        seq!("MAPPED EARLY", alt!(E::TypeName, E::EntityAssociation))
                    ),
                    ver(
                        "7.54",
                        seq!("REPORTED EARLY", alt!(E::TypeName, E::EntityAssociation))
                    ),
                    ver(
                        "7.54",
                        seq!("FAILED", alt!(E::TypeName, E::EntityAssociation))
                    ),
                    ver(
                        "7.54",
                        seq!("MAPPED", alt!(E::TypeName, E::EntityAssociation))
                    ),
                    ver(
                        "7.54",
                        seq!("REPORTED", alt!(E::TypeName, E::EntityAssociation))
                    ),
                    ver(
                        "7.53",
                        seq!("FUNCTION IMPORT", alt!(E::TypeName, E::EntityAssociation))
                    ),
                    ver(
                        "7.53",
                        seq!("FUNCTION RESULT", alt!(E::TypeName, E::EntityAssociation))
                    ),
                    ver(
                        "7.53",
                        seq!("ACTION RESULT", alt!(E::TypeName, E::EntityAssociation))
                    ),
                    ver(
                        "7.55",
                        seq!(
                            "INSTANCE AUTHORIZATION KEY",
                            alt!(E::TypeName, E::EntityAssociation)
                        )
                    ),
                    ver(
                        "7.55",
                        seq!(
                            "INSTANCE AUTHORIZATION REQUEST",
                            alt!(E::TypeName, E::EntityAssociation)
                        )
                    ),
                    ver(
                        "7.55",
                        seq!(
                            "INSTANCE AUTHORIZATION RESULT",
                            alt!(E::TypeName, E::EntityAssociation)
                        )
                    ),
                    ver(
                        "7.55",
                        seq!(
                            "INSTANCE FEATURES KEY",
                            alt!(E::TypeName, E::EntityAssociation)
                        )
                    ),
                    ver(
                        "7.55",
                        seq!(
                            "INSTANCE FEATURES REQUEST",
                            alt!(E::TypeName, E::EntityAssociation)
                        )
                    ),
                    ver(
                        "7.55",
                        seq!(
                            "INSTANCE FEATURES RESULT",
                            alt!(E::TypeName, E::EntityAssociation)
                        )
                    ),
                    ver(
                        "7.55",
                        seq!(
                            "AUTHORIZATION RESULT",
                            alt!(E::TypeName, E::EntityAssociation)
                        )
                    ),
                    ver(
                        "7.55",
                        seq!("AUTHORIZATION KEY", alt!(E::TypeName, E::EntityAssociation))
                    ),
                    ver(
                        "7.54",
                        seq!("FEATURES RESULT", alt!(E::TypeName, E::EntityAssociation))
                    ),
                    ver(
                        "7.54",
                        seq!("FEATURES KEY", alt!(E::TypeName, E::EntityAssociation))
                    ),
                    ver(
                        "7.55",
                        seq!("PERMISSIONS KEY", alt!(E::TypeName, E::EntityAssociation))
                    ),
                    seq!("READ IMPORT", alt!(E::TypeName, E::EntityAssociation)),
                    seq!("READ RESULT", alt!(E::TypeName, E::EntityAssociation)),
                    ver(
                        "8.16",
                        seq!("READ CHANGES", alt!(E::TypeName, E::EntityAssociation))
                    ),
                    seq!("CREATE", alt!(E::TypeName, E::EntityAssociation)),
                    seq!("DELETE", alt!(E::TypeName, E::EntityAssociation)),
                    seq!("UPDATE", alt!(E::TypeName, E::EntityAssociation)),
                    ver(
                        "7.54",
                        seq!("LOCK", alt!(E::TypeName, E::EntityAssociation))
                    ),
                    ver(
                        "7.54",
                        seq!("KEY OF", alt!(E::TypeName, E::EntityAssociation))
                    ),
                    ver(
                        "7.54",
                        seq!("DETERMINATION", alt!(E::TypeName, E::EntityAssociation))
                    ),
                    ver(
                        "7.54",
                        seq!("VALIDATION", alt!(E::TypeName, E::EntityAssociation))
                    ),
                    ver(
                        "7.55",
                        seq!("CHANGE", alt!(E::TypeName, E::EntityAssociation))
                    )
                ),
                opt_prio("VALUE IS INITIAL")
            )
        ),
        E::TypeStructure,
        seq!(
            "TYPE",
            alt!(
                seq!(
                    alt_prio!("STANDARD", "HASHED", "INDEX", "SORTED", "ANY"),
                    "TABLE"
                ),
                seq!(
                    alt!(
                        seq!(
                            opt_prio(seq!(
                                seq!(
                                    alt_prio!("STANDARD", "HASHED", "INDEX", "SORTED", "ANY"),
                                    "TABLE"
                                ),
                                "OF"
                            )),
                            opt_prio("REF TO")
                        ),
                        "LINE OF"
                    ),
                    E::TypeNameOrInfer,
                    opt(E::Default)
                )
            )
        ),
        // SAP `abentyping_complete`: LIKE { [LINE OF] dobj } | { REF TO dobj }
        seq!(
            "LIKE",
            opt(alt_prio!("LINE OF", "REF TO")),
            E::FieldChain,
            opt_prio(E::Default)
        )
    )
}

fn type_simple_ref() -> Rule {
    alt_prio!(
        seq!("LINE", "OF", E::TypeName),
        seq!("REF", "TO", E::TypeName),
        seq!(
            alt!("ANY", "STANDARD", "HASHED", "SORTED", "INDEX"),
            "TABLE"
        ),
        seq!("ANY", "STRUCTURE"),
        E::TypeName
    )
}

fn type_structure() -> Rule {
    seq!(
        "TYPE",
        alt_prio!(
            ver(
                "7.54",
                seq!(
                    "STRUCTURE FOR",
                    alt_prio!(
                        ver(
                            "7.55",
                            seq!("FUNCTION REQUEST", alt!(E::TypeName, E::EntityAssociation))
                        ),
                        ver(
                            "7.55",
                            seq!("ACTION REQUEST", alt!(E::TypeName, E::EntityAssociation))
                        ),
                        seq!("ACTION IMPORT", E::Source),
                        ver(
                            "7.55",
                            seq!(
                                "GLOBAL AUTHORIZATION REQUEST",
                                alt!(E::TypeName, E::EntityAssociation)
                            )
                        ),
                        ver(
                            "7.55",
                            seq!(
                                "GLOBAL AUTHORIZATION RESULT",
                                alt!(E::TypeName, E::EntityAssociation)
                            )
                        ),
                        ver(
                            "7.55",
                            seq!(
                                "GLOBAL FEATURES REQUEST",
                                alt!(E::TypeName, E::EntityAssociation)
                            )
                        ),
                        ver(
                            "7.55",
                            seq!(
                                "GLOBAL FEATURES RESULT",
                                alt!(E::TypeName, E::EntityAssociation)
                            )
                        ),
                        ver(
                            "7.55",
                            seq!(
                                "AUTHORIZATION REQUEST",
                                alt!(E::TypeName, E::EntityAssociation)
                            )
                        ),
                        ver(
                            "7.54",
                            seq!("FEATURES REQUEST", alt!(E::TypeName, E::EntityAssociation))
                        ),
                        seq!(
                            "PERMISSIONS REQUEST",
                            alt!(E::TypeName, E::EntityAssociation)
                        ),
                        ver(
                            "7.55",
                            seq!(
                                "PERMISSIONS RESULT",
                                alt!(E::TypeName, E::EntityAssociation)
                            )
                        ),
                        seq!("READ LINK", E::EntityAssociation),
                        seq!("HIERARCHY", E::NamespaceSimpleName),
                        seq!("EVENT", E::EventName),
                        ver(
                            "7.54",
                            seq!("FAILED LATE", alt!(E::TypeName, E::EntityAssociation))
                        ),
                        ver(
                            "7.54",
                            seq!("MAPPED LATE", alt!(E::TypeName, E::EntityAssociation))
                        ),
                        ver(
                            "7.54",
                            seq!("REPORTED LATE", alt!(E::TypeName, E::EntityAssociation))
                        ),
                        ver(
                            "7.54",
                            seq!("FAILED EARLY", alt!(E::TypeName, E::EntityAssociation))
                        ),
                        ver(
                            "7.54",
                            seq!("MAPPED EARLY", alt!(E::TypeName, E::EntityAssociation))
                        ),
                        ver(
                            "7.54",
                            seq!("REPORTED EARLY", alt!(E::TypeName, E::EntityAssociation))
                        ),
                        ver(
                            "7.54",
                            seq!("FAILED", alt!(E::TypeName, E::EntityAssociation))
                        ),
                        ver(
                            "7.54",
                            seq!("MAPPED", alt!(E::TypeName, E::EntityAssociation))
                        ),
                        ver(
                            "7.54",
                            seq!("REPORTED", alt!(E::TypeName, E::EntityAssociation))
                        ),
                        ver(
                            "7.53",
                            seq!("FUNCTION IMPORT", alt!(E::TypeName, E::EntityAssociation))
                        ),
                        ver(
                            "7.53",
                            seq!("FUNCTION RESULT", alt!(E::TypeName, E::EntityAssociation))
                        ),
                        ver(
                            "7.53",
                            seq!("ACTION RESULT", alt!(E::TypeName, E::EntityAssociation))
                        ),
                        ver(
                            "7.55",
                            seq!(
                                "INSTANCE AUTHORIZATION KEY",
                                alt!(E::TypeName, E::EntityAssociation)
                            )
                        ),
                        ver(
                            "7.55",
                            seq!(
                                "INSTANCE AUTHORIZATION REQUEST",
                                alt!(E::TypeName, E::EntityAssociation)
                            )
                        ),
                        ver(
                            "7.55",
                            seq!(
                                "INSTANCE AUTHORIZATION RESULT",
                                alt!(E::TypeName, E::EntityAssociation)
                            )
                        ),
                        ver(
                            "7.55",
                            seq!(
                                "INSTANCE FEATURES KEY",
                                alt!(E::TypeName, E::EntityAssociation)
                            )
                        ),
                        ver(
                            "7.55",
                            seq!(
                                "INSTANCE FEATURES REQUEST",
                                alt!(E::TypeName, E::EntityAssociation)
                            )
                        ),
                        ver(
                            "7.55",
                            seq!(
                                "INSTANCE FEATURES RESULT",
                                alt!(E::TypeName, E::EntityAssociation)
                            )
                        ),
                        ver(
                            "7.55",
                            seq!(
                                "AUTHORIZATION RESULT",
                                alt!(E::TypeName, E::EntityAssociation)
                            )
                        ),
                        ver(
                            "7.55",
                            seq!("AUTHORIZATION KEY", alt!(E::TypeName, E::EntityAssociation))
                        ),
                        ver(
                            "7.54",
                            seq!("FEATURES RESULT", alt!(E::TypeName, E::EntityAssociation))
                        ),
                        ver(
                            "7.54",
                            seq!("FEATURES KEY", alt!(E::TypeName, E::EntityAssociation))
                        ),
                        ver(
                            "7.55",
                            seq!("PERMISSIONS KEY", alt!(E::TypeName, E::EntityAssociation))
                        ),
                        seq!("READ IMPORT", alt!(E::TypeName, E::EntityAssociation)),
                        seq!("READ RESULT", alt!(E::TypeName, E::EntityAssociation)),
                        ver(
                            "8.16",
                            seq!("READ CHANGES", alt!(E::TypeName, E::EntityAssociation))
                        ),
                        seq!("CREATE", alt!(E::TypeName, E::EntityAssociation)),
                        seq!("DELETE", alt!(E::TypeName, E::EntityAssociation)),
                        seq!("UPDATE", alt!(E::TypeName, E::EntityAssociation)),
                        ver(
                            "7.54",
                            seq!("LOCK", alt!(E::TypeName, E::EntityAssociation))
                        ),
                        ver(
                            "7.54",
                            seq!("KEY OF", alt!(E::TypeName, E::EntityAssociation))
                        ),
                        ver(
                            "7.54",
                            seq!("DETERMINATION", alt!(E::TypeName, E::EntityAssociation))
                        ),
                        ver(
                            "7.54",
                            seq!("VALIDATION", alt!(E::TypeName, E::EntityAssociation))
                        ),
                        ver(
                            "7.55",
                            seq!("CHANGE", alt!(E::TypeName, E::EntityAssociation))
                        )
                    )
                )
            ),
            ver(
                "7.54",
                seq!(
                    "RESPONSE FOR",
                    alt_prio!(
                        seq!("FAILED EARLY", E::NamespaceSimpleName),
                        seq!("FAILED LATE", E::NamespaceSimpleName),
                        seq!("FAILED", E::NamespaceSimpleName),
                        seq!("MAPPED EARLY", E::NamespaceSimpleName),
                        seq!("MAPPED LATE", E::NamespaceSimpleName),
                        seq!("MAPPED", E::NamespaceSimpleName),
                        seq!("REPORTED EARLY", E::NamespaceSimpleName),
                        seq!("REPORTED LATE", E::NamespaceSimpleName),
                        seq!("REPORTED", E::NamespaceSimpleName)
                    )
                )
            ),
            ver(
                "7.55",
                seq!(
                    "REQUEST FOR",
                    alt!("CHANGE", "DELETE"),
                    E::NamespaceSimpleName
                )
            )
        ),
        opt_prio("VALUE IS INITIAL")
    )
}

fn type_table() -> Rule {
    alt_prio!(
        seq!(
            seq!("OCCURS", alt_prio!(E::Integer, E::FieldChain)),
            opt("WITH HEADER LINE")
        ),
        seq!(
            "LIKE",
            alt!(
                seq!(
                    opt("REF TO"),
                    E::FieldChain,
                    alt!(
                        seq!(
                            seq!("OCCURS", alt_prio!(E::Integer, E::FieldChain)),
                            opt("WITH HEADER LINE")
                        ),
                        "WITH HEADER LINE"
                    )
                ),
                seq!(
                    opt(alt!("STANDARD", "HASHED", "INDEX", "SORTED", "ANY")),
                    "TABLE OF",
                    opt_prio("REF TO"),
                    opt(E::FieldChain),
                    opt(per!(
                        "WITH HEADER LINE",
                        seq!("INITIAL SIZE", E::Constant),
                        plus_prio(E::TypeTableKey)
                    ))
                ),
                seq!(
                    "RANGE OF",
                    E::FieldChain,
                    opt_prio("WITH HEADER LINE"),
                    opt_prio(seq!("INITIAL SIZE", E::Constant)),
                    opt_prio("VALUE IS INITIAL")
                ),
                seq!(
                    "LINE OF",
                    E::TypeName,
                    seq!("OCCURS", alt_prio!(E::Integer, E::FieldChain)),
                    "WITH HEADER LINE"
                )
            )
        ),
        seq!(
            "TYPE",
            alt!(
                seq!(
                    opt("REF TO"),
                    E::TypeName,
                    alt!(
                        seq!(
                            seq!("OCCURS", alt_prio!(E::Integer, E::FieldChain)),
                            opt("WITH HEADER LINE")
                        ),
                        "WITH HEADER LINE"
                    )
                ),
                alt!(
                    seq!(
                        opt(alt!("STANDARD", "HASHED", "INDEX", "SORTED", "ANY")),
                        "TABLE"
                    ),
                    seq!(
                        seq!(
                            opt(alt!("STANDARD", "HASHED", "INDEX", "SORTED", "ANY")),
                            "TABLE OF",
                            opt_prio("REF TO"),
                            E::TypeName
                        ),
                        alt!(
                            opt(per!(
                                "WITH HEADER LINE",
                                seq!("INITIAL SIZE", E::Constant),
                                plus_prio(E::TypeTableKey)
                            )),
                            seq!(
                                plus(E::TypeTableKey),
                                opt_prio(seq!("INITIAL SIZE", E::Constant))
                            )
                        ),
                        opt_prio("VALUE IS INITIAL")
                    )
                ),
                seq!(
                    "RANGE OF",
                    E::TypeName,
                    opt_prio("WITH HEADER LINE"),
                    opt_prio(seq!("INITIAL SIZE", E::Constant)),
                    opt_prio("VALUE IS INITIAL")
                ),
                seq!(
                    "LINE OF",
                    E::TypeName,
                    seq!("OCCURS", alt_prio!(E::Integer, E::FieldChain)),
                    "WITH HEADER LINE"
                ),
                ver(
                    "7.53",
                    seq!(
                        "TABLE FOR",
                        alt_prio!(
                            ver(
                                "7.53",
                                seq!("ACTION IMPORT", alt!(E::TypeName, E::EntityAssociation))
                            ),
                            ver(
                                "7.54",
                                seq!("READ LINK", alt!(E::TypeName, E::EntityAssociation))
                            ),
                            ver(
                                "7.57",
                                seq!("EVENT", alt!(E::TypeName, E::EntityAssociation))
                            ),
                            ver(
                                "7.54",
                                seq!("FAILED LATE", alt!(E::TypeName, E::EntityAssociation))
                            ),
                            ver(
                                "7.54",
                                seq!("MAPPED LATE", alt!(E::TypeName, E::EntityAssociation))
                            ),
                            ver(
                                "7.54",
                                seq!("REPORTED LATE", alt!(E::TypeName, E::EntityAssociation))
                            ),
                            ver(
                                "7.54",
                                seq!("FAILED EARLY", alt!(E::TypeName, E::EntityAssociation))
                            ),
                            ver(
                                "7.54",
                                seq!("MAPPED EARLY", alt!(E::TypeName, E::EntityAssociation))
                            ),
                            ver(
                                "7.54",
                                seq!("REPORTED EARLY", alt!(E::TypeName, E::EntityAssociation))
                            ),
                            ver(
                                "7.54",
                                seq!("FAILED", alt!(E::TypeName, E::EntityAssociation))
                            ),
                            ver(
                                "7.54",
                                seq!("MAPPED", alt!(E::TypeName, E::EntityAssociation))
                            ),
                            ver(
                                "7.54",
                                seq!("REPORTED", alt!(E::TypeName, E::EntityAssociation))
                            ),
                            ver(
                                "7.53",
                                seq!("FUNCTION IMPORT", alt!(E::TypeName, E::EntityAssociation))
                            ),
                            ver(
                                "7.53",
                                seq!("FUNCTION RESULT", alt!(E::TypeName, E::EntityAssociation))
                            ),
                            ver(
                                "7.53",
                                seq!("ACTION RESULT", alt!(E::TypeName, E::EntityAssociation))
                            ),
                            ver(
                                "7.55",
                                seq!(
                                    "INSTANCE AUTHORIZATION KEY",
                                    alt!(E::TypeName, E::EntityAssociation)
                                )
                            ),
                            ver(
                                "7.55",
                                seq!(
                                    "INSTANCE AUTHORIZATION REQUEST",
                                    alt!(E::TypeName, E::EntityAssociation)
                                )
                            ),
                            ver(
                                "7.55",
                                seq!(
                                    "INSTANCE AUTHORIZATION RESULT",
                                    alt!(E::TypeName, E::EntityAssociation)
                                )
                            ),
                            ver(
                                "7.55",
                                seq!(
                                    "INSTANCE FEATURES KEY",
                                    alt!(E::TypeName, E::EntityAssociation)
                                )
                            ),
                            ver(
                                "7.55",
                                seq!(
                                    "INSTANCE FEATURES REQUEST",
                                    alt!(E::TypeName, E::EntityAssociation)
                                )
                            ),
                            ver(
                                "7.55",
                                seq!(
                                    "INSTANCE FEATURES RESULT",
                                    alt!(E::TypeName, E::EntityAssociation)
                                )
                            ),
                            ver(
                                "7.55",
                                seq!(
                                    "AUTHORIZATION RESULT",
                                    alt!(E::TypeName, E::EntityAssociation)
                                )
                            ),
                            ver(
                                "7.55",
                                seq!("AUTHORIZATION KEY", alt!(E::TypeName, E::EntityAssociation))
                            ),
                            ver(
                                "7.54",
                                seq!("FEATURES RESULT", alt!(E::TypeName, E::EntityAssociation))
                            ),
                            ver(
                                "7.54",
                                seq!("FEATURES KEY", alt!(E::TypeName, E::EntityAssociation))
                            ),
                            ver(
                                "7.55",
                                seq!("PERMISSIONS KEY", alt!(E::TypeName, E::EntityAssociation))
                            ),
                            seq!("READ IMPORT", alt!(E::TypeName, E::EntityAssociation)),
                            seq!("READ RESULT", alt!(E::TypeName, E::EntityAssociation)),
                            ver(
                                "8.16",
                                seq!("READ CHANGES", alt!(E::TypeName, E::EntityAssociation))
                            ),
                            seq!("CREATE", alt!(E::TypeName, E::EntityAssociation)),
                            seq!("DELETE", alt!(E::TypeName, E::EntityAssociation)),
                            seq!("UPDATE", alt!(E::TypeName, E::EntityAssociation)),
                            ver(
                                "7.54",
                                seq!("LOCK", alt!(E::TypeName, E::EntityAssociation))
                            ),
                            ver(
                                "7.54",
                                seq!("KEY OF", alt!(E::TypeName, E::EntityAssociation))
                            ),
                            ver(
                                "7.54",
                                seq!("DETERMINATION", alt!(E::TypeName, E::EntityAssociation))
                            ),
                            ver(
                                "7.54",
                                seq!("VALIDATION", alt!(E::TypeName, E::EntityAssociation))
                            ),
                            ver(
                                "7.55",
                                seq!("CHANGE", alt!(E::TypeName, E::EntityAssociation))
                            )
                        ),
                        opt_prio("VALUE IS INITIAL")
                    )
                )
            )
        )
    )
}

fn type_table_key() -> Rule {
    seq!(
        "WITH",
        opt_prio(alt!("NON-UNIQUE", "UNIQUE")),
        alt_prio!(
            "DEFAULT KEY",
            ver("7.40 SP02", "EMPTY KEY"),
            seq!(
                opt(alt!("SORTED", "HASHED")),
                "KEY",
                alt_prio!(
                    seq!(
                        E::Field,
                        opt(seq!("ALIAS", E::Field)),
                        "COMPONENTS",
                        plus_prio(seq!(
                            stop_before_any(&["WITH", "INITIAL", "WITHOUT"]),
                            stop_before("READ", "-"),
                            E::FieldSub
                        ))
                    ),
                    plus_prio(seq!(
                        stop_before_any(&["WITH", "INITIAL", "WITHOUT"]),
                        stop_before("READ", "-"),
                        E::FieldSub
                    ))
                )
            )
        ),
        opt_prio(seq!(alt!("WITHOUT", "WITH"), "FURTHER SECONDARY KEYS")),
        opt_prio("READ-ONLY")
    )
}

fn value() -> Rule {
    seq!(
        "VALUE",
        alt!(E::Constant, E::SimpleFieldChain, "IS INITIAL")
    )
}

fn value_base() -> Rule {
    seq!("BASE", E::Source)
}

fn value_body() -> Rule {
    opt_prio(alt_prio!(
        seq!(
            opt_prio(E::Let),
            opt_prio(E::ValueBase),
            star_prio(E::For),
            plus_prio(alt_prio!(E::FieldAssignment, E::ValueBodyLine))
        ),
        seq!(
            E::Source,
            opt_prio(ver(
                "7.40 SP08",
                alt_prio!("OPTIONAL", seq!("DEFAULT", E::Source))
            ))
        )
    ))
}

fn value_body_line() -> Rule {
    seq!(
        alt_prio!(Tok::WParenLeftW, Tok::WParenLeft),
        opt_prio(alt_prio!(
            plus_prio(E::FieldAssignment),
            E::ValueBodyLines,
            E::Source
        )),
        alt_prio!(Tok::WParenRightW, Tok::ParenRightW)
    )
}

fn value_body_lines() -> Rule {
    seq!(
        "LINES OF",
        E::Source,
        seq!(
            opt_prio(seq!("FROM", E::Source)),
            opt_prio(seq!("TO", E::Source))
        )
    )
}

/// The name of a common table expression, with its column names, which may
/// be named like keywords, `+result( name, connection )`.
fn with_name() -> Rule {
    seq!(
        Tok::WPlus,
        re(r"^\w+$"),
        opt_prio(seq!(
            Tok::ParenLeftW,
            re(r"^\w+$"),
            star(seq!(",", re(r"^\w+$"))),
            Tok::WParenRightW
        ))
    )
}

fn write_offset_length() -> Rule {
    // SAP `abapwrite-`: "If position and length are only specified as
    // numeric literals, the addition AT can be omitted."
    let literal = || alt!(re(r"^\d+$"), "*", "**");
    alt!(
        seq!(
            Tok::WParenLeft,
            literal(),
            alt!(Tok::ParenRightW, Tok::ParenRight)
        ),
        seq!(
            alt!(re(r"^\/?\d+$"), "/"),
            opt(seq!(
                alt!(Tok::WParenLeft, Tok::ParenLeft),
                literal(),
                alt!(Tok::ParenRightW, Tok::ParenRight)
            ))
        ),
        seq!("AT", write_position())
    )
}

/// The position and length after AT.
fn write_position() -> Rule {
    seq!(alt!(
        seq!(
            Tok::WParenLeft,
            seq!(
                alt!(E::SimpleFieldChain2, re(r"^[\d]+$"), "*"),
                alt!(Tok::ParenRightW, Tok::ParenRight)
            )
        ),
        seq!(
            alt!(E::SimpleFieldChain2, re(r"^\/?[\w\d]+$"), "/"),
            opt(seq!(
                alt!(Tok::WParenLeft, Tok::ParenLeft),
                seq!(
                    alt!(E::SimpleFieldChain2, re(r"^[\d]+$"), "*"),
                    alt!(Tok::ParenRightW, Tok::ParenRight)
                )
            ))
        )
    ))
}
