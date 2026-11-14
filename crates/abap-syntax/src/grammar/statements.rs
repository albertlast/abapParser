//! Statements of the statement grammars.
//!
//! Generated from the statement matchers of abaplint 2.120.105
//! (<https://github.com/abaplint/abaplint>, MIT, Copyright (c) 2015 Lars Hvam)
//! by tools/gen-grammar.mjs, maintained by hand since.

use super::dsl::*;
use super::E;
use crate::StatementKind as K;

/// The grammar of a statement kind.
pub(super) fn statement(kind: K) -> Option<Rule> {
    Some(match kind {
        K::Add => add(),
        K::AddCorresponding => add_corresponding(),
        K::Aliases => aliases(),
        K::Append => append(),
        K::Assert => assert(),
        K::Assign => assign(),
        K::AssignLocalCopy => assign_local_copy(),
        K::At => at(),
        K::AtFirst => at_first(),
        K::AtLast => at_last(),
        K::AtLineSelection => at_line_selection(),
        K::AtPF => at_pf(),
        K::AtSelectionScreen => at_selection_screen(),
        K::AtUserCommand => at_user_command(),
        K::AuthorityCheck => authority_check(),
        K::Back => back(),
        K::Break => break_(),
        K::BreakId => break_id(),
        K::Call => call(),
        K::CallBadi => call_badi(),
        K::CallDatabase => call_database(),
        K::CallDialog => call_dialog(),
        K::CallFunction => call_function(),
        K::CallKernel => call_kernel(),
        K::CallOLE => call_ole(),
        K::CallScreen => call_screen(),
        K::CallSelectionScreen => call_selection_screen(),
        K::CallSubscreen => call_subscreen(),
        K::CallTransaction => call_transaction(),
        K::CallTransformation => call_transformation(),
        K::Case => case(),
        K::CaseType => case_type(),
        K::Catch => catch(),
        K::CatchSystemExceptions => catch_system_exceptions(),
        K::Chain => chain(),
        K::Check => check(),
        K::CheckSelectOptions => check_select_options(),
        K::ClassData => class_data(),
        K::ClassDataBegin => class_data_begin(),
        K::ClassDataEnd => class_data_end(),
        K::ClassDeferred => class_deferred(),
        K::ClassDefinition => class_definition(),
        K::ClassDefinitionLoad => class_definition_load(),
        K::ClassImplementation => class_implementation(),
        K::ClassLocalFriends => class_local_friends(),
        K::Cleanup => cleanup(),
        K::Clear => clear(),
        K::CloseCursor => close_cursor(),
        K::CloseDataset => close_dataset(),
        K::Collect => collect(),
        K::Commit => commit(),
        K::CommitEntities => commit_entities(),
        K::Communication => communication(),
        K::Compute => compute(),
        K::Concatenate => concatenate(),
        K::Condense => condense(),
        K::Constant => constant(),
        K::ConstantBegin => constant_begin(),
        K::ConstantEnd => constant_end(),
        K::Contexts => contexts(),
        K::Continue => continue_(),
        K::Controls => controls(),
        K::Convert => convert(),
        K::ConvertText => convert_text(),
        K::CreateData => create_data(),
        K::CreateObject => create_object(),
        K::CreateOLE => create_ole(),
        K::Data => data(),
        K::DataBegin => data_begin(),
        K::DataEnd => data_end(),
        K::Define => define(),
        K::DeleteCluster => delete_cluster(),
        K::DeleteDatabase => delete_database(),
        K::DeleteDataset => delete_dataset(),
        K::DeleteDynpro => delete_dynpro(),
        K::DeleteInternal => delete_internal(),
        K::DeleteMemory => delete_memory(),
        K::DeleteReport => delete_report(),
        K::DeleteTextpool => delete_textpool(),
        K::Demand => demand(),
        K::Describe => describe(),
        K::Detail => detail(),
        K::Divide => divide(),
        K::Do => do_(),
        K::DynproLoop => dynpro_loop(),
        K::EditorCall => editor_call(),
        K::Else => else_(),
        K::ElseIf => else_if(),
        K::EndAt => end_at(),
        K::EndCase => end_case(),
        K::EndCatch => end_catch(),
        K::EndChain => end_chain(),
        K::EndClass => end_class(),
        K::EndDo => end_do(),
        K::EndEnhancement => end_enhancement(),
        K::EndEnhancementSection => end_enhancement_section(),
        K::EndExec => end_exec(),
        K::EndForm => end_form(),
        K::EndFunction => end_function(),
        K::EndIf => end_if(),
        K::EndInterface => end_interface(),
        K::EndLoop => end_loop(),
        K::EndMethod => end_method(),
        K::EndModule => end_module(),
        K::EndOfDefinition => end_of_definition(),
        K::EndOfPage => end_of_page(),
        K::EndOfSelection => end_of_selection(),
        K::EndOn => end_on(),
        K::EndProvide => end_provide(),
        K::EndSelect => end_select(),
        K::EndTestInjection => end_test_injection(),
        K::EndTestSeam => end_test_seam(),
        K::EndTry => end_try(),
        K::EndWhile => end_while(),
        K::EndWith => end_with(),
        K::Enhancement => enhancement(),
        K::EnhancementPoint => enhancement_point(),
        K::EnhancementSection => enhancement_section(),
        K::Events => events(),
        K::ExecSQL => exec_sql(),
        K::Exit => exit(),
        K::Export => export(),
        K::ExportDynpro => export_dynpro(),
        K::Extract => extract(),
        K::FetchNextCursor => fetch_next_cursor(),
        K::Field => field(),
        K::FieldGroup => field_group(),
        K::Fields => fields(),
        K::FieldSymbol => field_symbol(),
        K::Find => find(),
        K::Form => form(),
        K::Format => format(),
        K::FormDefinition => form_definition(),
        K::Free => free(),
        K::FreeMemory => free_memory(),
        K::FreeObject => free_object(),
        K::FunctionModule => function_module(),
        K::FunctionPool => function_pool(),
        K::GenerateDynpro => generate_dynpro(),
        K::GenerateReport => generate_report(),
        K::GenerateSubroutine => generate_subroutine(),
        K::Get => get(),
        K::GetBadi => get_badi(),
        K::GetBit => get_bit(),
        K::GetCursor => get_cursor(),
        K::GetDataset => get_dataset(),
        K::GetLocale => get_locale(),
        K::GetParameter => get_parameter(),
        K::GetPermissions => get_permissions(),
        K::GetPFStatus => get_pf_status(),
        K::GetProperty => get_property(),
        K::GetReference => get_reference(),
        K::GetRunTime => get_run_time(),
        K::GetTime => get_time(),
        K::Hide => hide(),
        K::If => if_(),
        K::Import => import(),
        K::ImportDynpro => import_dynpro(),
        K::ImportNametab => import_nametab(),
        K::Include => include(),
        K::IncludeType => include_type(),
        K::Infotypes => infotypes(),
        K::Initialization => initialization(),
        K::Input => input(),
        K::InsertDatabase => insert_database(),
        K::InsertFieldGroup => insert_field_group(),
        K::InsertInternal => insert_internal(),
        K::InsertReport => insert_report(),
        K::InsertTextpool => insert_textpool(),
        K::Interface => interface(),
        K::InterfaceDef => interface_def(),
        K::InterfaceDeferred => interface_deferred(),
        K::InterfaceLoad => interface_load(),
        K::Leave => leave(),
        K::LoadOfProgram => load_of_program(),
        K::LoadReport => load_report(),
        K::Local => local(),
        K::LogPoint => log_point(),
        K::Loop => loop_(),
        K::LoopAtScreen => loop_at_screen(),
        K::LoopExtract => loop_extract(),
        K::MergeDatabase => merge_database(),
        K::Message => message(),
        K::MethodDef => method_def(),
        K::MethodImplementation => method_implementation(),
        K::ModifyDatabase => modify_database(),
        K::ModifyEntities => modify_entities(),
        K::ModifyInternal => modify_internal(),
        K::ModifyLine => modify_line(),
        K::ModifyScreen => modify_screen(),
        K::Module => module(),
        K::Move => move_(),
        K::MoveCorresponding => move_corresponding(),
        K::Multiply => multiply(),
        K::NewLine => new_line(),
        K::NewPage => new_page(),
        K::Nodes => nodes(),
        K::OnChange => on_change(),
        K::OpenCursor => open_cursor(),
        K::OpenDataset => open_dataset(),
        K::Overlay => overlay(),
        K::Pack => pack(),
        K::Parameter => parameter(),
        K::Perform => perform(),
        K::Position => position(),
        K::PrintControl => print_control(),
        K::Private => private(),
        K::ProcessAfterInput => process_after_input(),
        K::ProcessBeforeOutput => process_before_output(),
        K::ProcessOnHelpRequest => process_on_help_request(),
        K::ProcessOnValueRequest => process_on_value_request(),
        K::Program => program(),
        K::Protected => protected(),
        K::Provide => provide(),
        K::Public => public(),
        K::Put => put(),
        K::Raise => raise(),
        K::RaiseEntityEvent => raise_entity_event(),
        K::RaiseEvent => raise_event(),
        K::Ranges => ranges(),
        K::ReadDataset => read_dataset(),
        K::ReadEntities => read_entities(),
        K::ReadLine => read_line(),
        K::ReadReport => read_report(),
        K::ReadTable => read_table(),
        K::ReadTextpool => read_textpool(),
        K::Receive => receive(),
        K::Refresh => refresh(),
        K::RefreshControl => refresh_control(),
        K::Reject => reject(),
        K::Replace => replace(),
        K::Report => report(),
        K::Reserve => reserve(),
        K::Resume => resume(),
        K::Retry => retry(),
        K::Return => return_(),
        K::Rollback => rollback(),
        K::RollbackEntities => rollback_entities(),
        K::Scan => scan(),
        K::ScrollList => scroll_list(),
        K::Search => search(),
        K::Select => select(),
        K::SelectionScreen => selection_screen(),
        K::SelectLoop => select_loop(),
        K::SelectOption => select_option(),
        K::SetBit => set_bit(),
        K::SetBlank => set_blank(),
        K::SetCountry => set_country(),
        K::SetCursor => set_cursor(),
        K::SetDataset => set_dataset(),
        K::SetExtendedCheck => set_extended_check(),
        K::SetHandler => set_handler(),
        K::SetLanguage => set_language(),
        K::SetLeft => set_left(),
        K::SetLocale => set_locale(),
        K::SetLocks => set_locks(),
        K::SetMargin => set_margin(),
        K::SetParameter => set_parameter(),
        K::SetPFStatus => set_pf_status(),
        K::SetProperty => set_property(),
        K::SetRunTime => set_run_time(),
        K::SetScreen => set_screen(),
        K::SetTitlebar => set_titlebar(),
        K::SetUpdateTask => set_update_task(),
        K::SetUserCommand => set_user_command(),
        K::Shift => shift(),
        K::Skip => skip(),
        K::Sort => sort(),
        K::SortDataset => sort_dataset(),
        K::Split => split(),
        K::StartOfSelection => start_of_selection(),
        K::Static => static_(),
        K::StaticBegin => static_begin(),
        K::StaticEnd => static_end(),
        K::Stop => stop(),
        K::Submit => submit(),
        K::Subtract => subtract(),
        K::SubtractCorresponding => subtract_corresponding(),
        K::Sum => sum(),
        K::Summary => summary(),
        K::Supply => supply(),
        K::SuppressDialog => suppress_dialog(),
        K::SyntaxCheck => syntax_check(),
        K::SystemCall => system_call(),
        K::Tables => tables(),
        K::TestInjection => test_injection(),
        K::TestSeam => test_seam(),
        K::TopOfPage => top_of_page(),
        K::Transfer => transfer(),
        K::Translate => translate(),
        K::TruncateDataset => truncate_dataset(),
        K::Try => try_(),
        K::Type => type_(),
        K::TypeBegin => type_begin(),
        K::TypeEnd => type_end(),
        K::TypeEnum => type_enum(),
        K::TypeEnumBegin => type_enum_begin(),
        K::TypeEnumEnd => type_enum_end(),
        K::TypeMesh => type_mesh(),
        K::TypeMeshBegin => type_mesh_begin(),
        K::TypeMeshEnd => type_mesh_end(),
        K::TypePool => type_pool(),
        K::TypePools => type_pools(),
        K::Uline => uline(),
        K::Unassign => unassign(),
        K::Unpack => unpack(),
        K::UpdateDatabase => update_database(),
        K::VerificationMessage => verification_message(),
        K::Wait => wait(),
        K::When => when(),
        K::WhenOthers => when_others(),
        K::WhenType => when_type(),
        K::While => while_(),
        K::Window => window(),
        K::With => with(),
        K::WithLoop => with_loop(),
        K::Write => write(),
        _ => return None,
    })
}

fn add() -> Rule {
    not_in(
        Lang::KeyUser,
        seq!(
            "ADD",
            E::Source,
            alt_prio!(
                seq!("TO", E::Target),
                seq!(
                    "THEN",
                    E::Source,
                    "UNTIL",
                    E::Source,
                    opt(per!(
                        seq!("GIVING", E::Source),
                        seq!("ACCORDING TO", E::Source)
                    )),
                    opt(seq!("TO", E::Target))
                )
            )
        ),
    )
}

fn add_corresponding() -> Rule {
    not_in(
        Lang::Cloud,
        seq!("ADD-CORRESPONDING", E::Source, "TO", E::Target),
    )
}

fn aliases() -> Rule {
    seq!("ALIASES", E::SimpleName, "FOR", E::Field)
}

fn append() -> Rule {
    seq!(
        "APPEND",
        alt_prio!(
            "INITIAL LINE",
            seq!(
                opt_prio("LINES OF"),
                alt!(E::SimpleSource4, ver("7.40 SP02", E::Source))
            )
        ),
        opt_prio(seq!("FROM", E::Source)),
        opt(alt!(
            seq!(seq!("TO", E::Source), seq!("TO", E::Target)),
            seq!("TO", E::Target)
        )),
        opt(alt_prio!(
            seq!("ASSIGNING", E::FSTarget),
            seq!("REFERENCE INTO", E::Target)
        )),
        opt_prio("CASTING"),
        opt_prio(seq!("SORTED BY", E::Field))
    )
}

fn assert() -> Rule {
    not_in(
        Lang::KeyUser,
        seq!(
            "ASSERT",
            opt_prio(seq!("ID", E::NamespaceSimpleName)),
            opt_prio(seq!("SUBKEY", E::Source)),
            opt(seq!("FIELDS", plus(E::Source))),
            opt_prio("CONDITION"),
            E::Cond
        ),
    )
}

fn assign() -> Rule {
    seq!(
        "ASSIGN",
        opt(not_in(Lang::KeyUser, seq!(E::Target, "INCREMENT"))),
        E::AssignSource,
        "TO",
        E::FSTarget,
        opt(alt_prio!(
            not_in(
                Lang::KeyUser,
                seq!(
                    "CASTING",
                    opt(alt!(
                        seq!("LIKE", alt_prio!(E::Dynamic, E::Source)),
                        seq!("TYPE HANDLE", E::Source),
                        per!(
                            seq!(
                                "TYPE",
                                alt_prio!(not_in(Lang::KeyUser, E::Dynamic), E::TypeName)
                            ),
                            seq!("DECIMALS", E::Source)
                        )
                    ))
                )
            ),
            seq!("TYPE", E::Source, opt_prio(seq!("DECIMALS", E::Source))),
            seq!("DECIMALS", E::Source)
        )),
        opt(seq!("RANGE", E::Source)),
        opt(ver("7.57", "ELSE UNASSIGN"))
    )
}

fn assign_local_copy() -> Rule {
    not_in(
        Lang::Cloud,
        seq!(
            "ASSIGN LOCAL COPY OF",
            alt!(
                seq!("INITIAL", alt!(E::Source, E::Dynamic)),
                seq!("INITIAL LINE OF", alt!(E::Source, E::Dynamic)),
                seq!("MAIN TABLE FIELD", E::Dynamic),
                E::Source
            ),
            "TO",
            E::TargetFieldSymbol
        ),
    )
}

fn at() -> Rule {
    not_in(
        Lang::KeyUser,
        seq!(
            "AT",
            alt_prio!(
                seq!(
                    "NEW",
                    alt!(
                        seq!(E::FieldSub, opt(E::FieldOffset), opt(E::FieldLength)),
                        E::Dynamic,
                        E::SourceFieldSymbol
                    )
                ),
                seq!(
                    "END OF",
                    alt!(
                        seq!(E::FieldSub, opt(E::FieldOffset), opt(E::FieldLength)),
                        E::Dynamic,
                        E::SourceFieldSymbol
                    )
                ),
                re(r"^[%\w]+$")
            )
        ),
    )
}

fn at_first() -> Rule {
    Rule::from("AT FIRST")
}

fn at_last() -> Rule {
    Rule::from("AT LAST")
}

fn at_line_selection() -> Rule {
    not_in(Lang::Cloud, "AT LINE-SELECTION")
}

fn at_pf() -> Rule {
    not_in(Lang::Cloud, seq!("AT", re(r"(?i)^PF\d\d?$")))
}

fn at_selection_screen() -> Rule {
    not_in(
        Lang::Cloud,
        seq!(
            "AT SELECTION-SCREEN",
            opt(alt!(
                "OUTPUT",
                seq!("ON VALUE-REQUEST FOR", E::FieldSub),
                seq!("ON RADIOBUTTON GROUP", E::Field),
                "ON EXIT-COMMAND",
                seq!("ON", E::FieldSub),
                seq!("ON END OF", E::Field),
                seq!("ON HELP-REQUEST FOR", E::FieldSub),
                seq!("ON BLOCK", E::BlockName)
            ))
        ),
    )
}

fn at_user_command() -> Rule {
    not_in(Lang::Cloud, "AT USER-COMMAND")
}

fn authority_check() -> Rule {
    not_in(
        Lang::KeyUser,
        seq!(
            "AUTHORITY-CHECK OBJECT",
            E::Source,
            opt(seq!("FOR USER", E::Source)),
            plus(seq!(
                "ID",
                E::Source,
                alt!(seq!("FIELD", E::Source), "DUMMY")
            ))
        ),
    )
}

fn back() -> Rule {
    not_in(Lang::Cloud, "BACK")
}

fn break_() -> Rule {
    not_in(
        Lang::Cloud,
        alt_prio!(
            seq!(
                "BREAK-POINT",
                opt_prio(alt_prio!("AT NEXT APPLICATION STATEMENT", E::Source))
            ),
            seq!("BREAK", star_prio(re(r".*")))
        ),
    )
}

fn break_id() -> Rule {
    not_in(Lang::Cloud, seq!("BREAK-POINT", seq!("ID", E::Field)))
}

fn call() -> Rule {
    alt_prio!(
        not_in(
            Lang::KeyUser,
            seq!("CALL METHOD", E::MethodSource, E::MethodCallBody)
        ),
        E::MethodCallChain
    )
}

fn call_badi() -> Rule {
    not_in(
        Lang::Cloud,
        seq!("CALL BADI", E::MethodSource, E::MethodCallBody),
    )
}

fn call_database() -> Rule {
    not_in(
        Lang::Cloud,
        seq!(
            "CALL DATABASE PROCEDURE",
            E::Dynamic,
            opt(E::DatabaseConnection),
            alt!(
                seq!(
                    opt(seq!("EXPORTING", E::ParameterListS)),
                    opt(seq!("IMPORTING", E::ParameterListT))
                ),
                seq!("PARAMETER-TABLE", E::Source)
            )
        ),
    )
}

fn call_dialog() -> Rule {
    not_in(
        Lang::Cloud,
        seq!(
            "CALL DIALOG",
            alt!(E::Constant, E::FieldSub),
            opt(seq!(
                "EXPORTING",
                plus(seq!(E::FieldSub, opt_prio(seq!("FROM", E::Source))))
            )),
            opt(seq!(
                "IMPORTING",
                plus(seq!(E::Field, opt_prio(seq!("TO", E::Field))))
            ))
        ),
    )
}

fn call_function() -> Rule {
    not_in(
        Lang::KeyUser,
        seq!(
            "CALL",
            alt_prio!("FUNCTION", not_in(Lang::Cloud, "CUSTOMER-FUNCTION")),
            E::FunctionName,
            opt(per!(
                not_in(Lang::Cloud, seq!("STARTING NEW TASK", E::SimpleSource2)),
                not_in(Lang::Cloud, "IN UPDATE TASK"),
                not_in(
                    Lang::Cloud,
                    seq!("IN BACKGROUND", alt_prio!("TASK", seq!("UNIT", E::Source)))
                ),
                E::Destination,
                seq!("CALLING", E::MethodSource, "ON END OF TASK"),
                seq!("PERFORMING", E::FormName, "ON END OF TASK"),
                "AS SEPARATE UNIT",
                "KEEPING LOGICAL UNIT OF WORK"
            )),
            alt!(
                E::FunctionParameters,
                alt!(
                    seq!(
                        "PARAMETER-TABLE",
                        E::Source,
                        opt(seq!("EXCEPTION-TABLE", E::Source))
                    ),
                    seq!("EXCEPTION-TABLE", E::Source)
                )
            )
        ),
    )
}

fn call_kernel() -> Rule {
    not_in(
        Lang::Cloud,
        seq!(
            "CALL",
            alt_prio!(E::Constant, E::Field),
            star_prio(E::KernelId)
        ),
    )
}

fn call_ole() -> Rule {
    not_in(
        Lang::Cloud,
        seq!(
            "CALL METHOD OF",
            E::Source,
            E::Source,
            opt(seq!("=", E::Target)),
            opt("NO FLUSH"),
            opt(alt!("QUEUE-ONLY", "QUEUEONLY")),
            opt(E::OLEExporting)
        ),
    )
}

fn call_screen() -> Rule {
    not_in(
        Lang::Cloud,
        seq!(
            "CALL SCREEN",
            E::Source,
            opt_prio(seq!(
                seq!("STARTING AT", E::Source, E::Source),
                opt_prio(seq!("ENDING AT", E::Source, E::Source))
            ))
        ),
    )
}

fn call_selection_screen() -> Rule {
    not_in(
        Lang::Cloud,
        seq!(
            "CALL SELECTION-SCREEN",
            E::Source,
            opt(seq!(
                seq!("STARTING AT", E::Source, E::Source),
                opt(seq!("ENDING AT", E::Source, E::Source))
            )),
            opt(seq!("USING SELECTION-SET", E::Source))
        ),
    )
}

fn call_subscreen() -> Rule {
    not_in(
        Lang::Cloud,
        seq!(
            "CALL SUBSCREEN",
            E::Source,
            opt_prio(seq!("INCLUDING", E::Source, E::Source))
        ),
    )
}

fn call_transaction() -> Rule {
    not_in(
        Lang::Cloud,
        seq!(
            "CALL TRANSACTION",
            E::Source,
            opt_prio(ver(
                "7.40 SP02",
                seq!(alt_prio!("WITH", "WITHOUT"), "AUTHORITY-CHECK")
            )),
            opt_prio(seq!("USING", E::Source)),
            opt(per!(
                seq!("UPDATE", E::Source),
                "AND SKIP FIRST SCREEN",
                seq!("OPTIONS FROM", E::Source),
                seq!("MESSAGES INTO", E::Target),
                seq!("MODE", E::Source)
            ))
        ),
    )
}

fn call_transformation() -> Rule {
    not_in(
        Lang::KeyUser,
        seq!(
            "CALL TRANSFORMATION",
            alt!(E::NamespaceSimpleName, E::Dynamic),
            per!(
                seq!("OPTIONS", E::CallTransformationOptions),
                seq!("PARAMETERS", E::CallTransformationParameters),
                seq!("OBJECTS", E::CallTransformationParameters),
                // SOURCE JSON and RESULT JSON, as the syntax check of the SAP
                // trial 2025 has them
                seq!(
                    "SOURCE",
                    alt!(
                        E::CallTransformationParameters,
                        seq!("XML", E::SimpleSource3),
                        seq!("JSON", E::SimpleSource3)
                    )
                ),
                seq!(
                    "RESULT",
                    alt!(
                        E::CallTransformationParameters,
                        seq!("XML", E::Target),
                        seq!("JSON", E::Target)
                    )
                )
            )
        ),
    )
}

fn case() -> Rule {
    seq!("CASE", E::Source)
}

fn case_type() -> Rule {
    ver("7.50", seq!("CASE TYPE OF", E::Source))
}

fn catch() -> Rule {
    seq!(
        "CATCH",
        opt_prio("BEFORE UNWIND"),
        plus(E::ClassName),
        opt(seq!("INTO", E::Target))
    )
}

fn catch_system_exceptions() -> Rule {
    not_in(
        Lang::Cloud,
        seq!(
            "CATCH SYSTEM-EXCEPTIONS",
            plus(seq!(E::Field, "=", E::Source))
        ),
    )
}

fn chain() -> Rule {
    not_in(Lang::Cloud, "CHAIN")
}

fn check() -> Rule {
    seq!("CHECK", alt_prio!(E::Cond, E::FieldSub))
}

fn check_select_options() -> Rule {
    not_in(Lang::Cloud, "CHECK SELECT-OPTIONS")
}

fn class_data() -> Rule {
    seq!("CLASS-DATA", E::DataDefinition)
}

fn class_data_begin() -> Rule {
    seq!(
        "CLASS-DATA",
        seq!(
            "BEGIN OF",
            opt_prio("COMMON PART"),
            E::NamespaceSimpleName,
            opt_prio("READ-ONLY"),
            opt_prio(seq!("OCCURS", E::Integer))
        )
    )
}

fn class_data_end() -> Rule {
    seq!(
        "CLASS-DATA",
        seq!(
            "END OF",
            alt_prio!(
                seq!("COMMON PART", opt_prio(E::NamespaceSimpleName)),
                E::NamespaceSimpleName
            )
        )
    )
}

fn class_deferred() -> Rule {
    not_in(
        Lang::KeyUser,
        seq!(
            "CLASS",
            E::ClassName,
            "DEFINITION DEFERRED",
            opt_prio("PUBLIC")
        ),
    )
}

fn class_definition() -> Rule {
    seq!(
        "CLASS",
        E::ClassName,
        seq!(
            "DEFINITION",
            opt_prio(per!(
                E::ClassGlobal,
                E::ClassFinal,
                "ABSTRACT",
                seq!("INHERITING FROM", E::SuperClassName),
                seq!("CREATE", alt_prio!("PUBLIC", "PROTECTED", "PRIVATE")),
                "FOR TESTING",
                seq!("RISK LEVEL", alt_prio!("CRITICAL", "HARMLESS", "DANGEROUS")),
                "SHARED MEMORY ENABLED",
                seq!("DURATION", alt_prio!("LONG", "MEDIUM", "SHORT")),
                ver("7.54", seq!("FOR BEHAVIOR OF", E::BehaviorDefinitionName)),
                ver("7.58", seq!("FOR EVENTS OF", E::BehaviorDefinitionName)),
                E::ClassFriends
            ))
        )
    )
}

fn class_definition_load() -> Rule {
    not_in(
        Lang::KeyUser,
        seq!("CLASS", E::ClassName, "DEFINITION LOAD"),
    )
}

fn class_implementation() -> Rule {
    seq!("CLASS", E::ClassName, "IMPLEMENTATION")
}

fn class_local_friends() -> Rule {
    seq!(
        "CLASS",
        E::ClassName,
        "DEFINITION LOCAL FRIENDS",
        plus_prio(E::ClassName)
    )
}

fn cleanup() -> Rule {
    seq!("CLEANUP", opt(seq!("INTO", E::Target)))
}

fn clear() -> Rule {
    seq!(
        "CLEAR",
        E::Target,
        opt_prio(seq!("WITH", E::Source)),
        opt_prio(alt_prio!("IN CHARACTER MODE", "IN BYTE MODE"))
    )
}

fn close_cursor() -> Rule {
    not_in(Lang::Cloud, seq!("CLOSE CURSOR", E::SQLSourceSimple))
}

fn close_dataset() -> Rule {
    not_in(Lang::Cloud, seq!("CLOSE DATASET", E::Source))
}

fn collect() -> Rule {
    seq!(
        "COLLECT",
        E::Source,
        opt(seq!("INTO", E::Target)),
        opt(seq!("ASSIGNING", E::FSTarget))
    )
}

fn commit() -> Rule {
    not_in(
        Lang::KeyUser,
        seq!(
            "COMMIT",
            alt!(seq!("WORK", opt("AND WAIT")), E::DatabaseConnection)
        ),
    )
}

fn commit_entities() -> Rule {
    not_in(
        Lang::KeyUser,
        ver(
            "7.54",
            seq!(
                "COMMIT ENTITIES",
                opt_prio("IN SIMULATION MODE"),
                opt(seq!(
                    "RESPONSES",
                    seq!("FAILED", E::Target),
                    seq!("REPORTED", E::Target)
                )),
                opt(seq!(
                    "RESPONSE OF",
                    E::NamespaceSimpleName,
                    per!(seq!("FAILED", E::Target), seq!("REPORTED", E::Target))
                ))
            ),
        ),
    )
}

fn communication() -> Rule {
    not_in(
        Lang::Cloud,
        seq!(
            "COMMUNICATION",
            alt!(
                seq!("INIT ID", E::Source, "DESTINATION", E::Target),
                seq!("ALLOCATE ID", E::Source, opt(seq!("RETURNCODE", E::Source))),
                seq!(
                    "SEND ID",
                    E::Source,
                    opt(per!(seq!("BUFFER", E::Target), seq!("LENGTH", E::Target))),
                    opt(seq!("RETURNCODE", E::Source))
                ),
                seq!(
                    "DEALLOCATE ID",
                    E::Source,
                    opt(seq!("RETURNCODE", E::Source))
                ),
                seq!(
                    "RECEIVE ID",
                    E::Source,
                    "BUFFER",
                    E::Source,
                    opt(seq!("LENGTH", E::Target)),
                    "DATAINFO",
                    E::Target,
                    "STATUSINFO",
                    E::Target,
                    "RECEIVED",
                    E::Target
                ),
                seq!("ACCEPT ID", E::Source)
            )
        ),
    )
}

fn compute() -> Rule {
    not_in(
        Lang::Cloud,
        seq!("COMPUTE", opt("EXACT"), E::Target, "=", E::Source),
    )
}

fn concatenate() -> Rule {
    seq!(
        "CONCATENATE",
        alt_prio!(
            seq!("LINES OF", E::Source),
            // SAP: INTO ends the operands
            seq!(
                E::SimpleSource3,
                plus(seq!(stop_before_any(&["INTO"]), E::SimpleSource3))
            )
        ),
        "INTO",
        E::Target,
        opt_prio(per!(
            seq!("IN", alt_prio!("BYTE", "CHARACTER"), "MODE"),
            "RESPECTING BLANKS",
            seq!("SEPARATED BY", E::SimpleSource3)
        ))
    )
}

fn condense() -> Rule {
    seq!("CONDENSE", E::Target, opt("NO-GAPS"))
}

fn constant() -> Rule {
    seq!(
        alt!("CONSTANT", "CONSTANTS"),
        seq!(
            E::DefinitionName,
            opt(E::ConstantFieldLength),
            per!(E::Type, E::Value, E::Decimals, E::Length)
        ),
        opt_prio("%_PREDEFINED")
    )
}

fn constant_begin() -> Rule {
    seq!(
        "CONSTANTS BEGIN OF",
        E::DefinitionName,
        opt_prio("%_PREDEFINED")
    )
}

fn constant_end() -> Rule {
    seq!("CONSTANTS", "END", "OF", E::DefinitionName)
}

fn contexts() -> Rule {
    not_in(Lang::Cloud, seq!("CONTEXTS", E::Field))
}

fn continue_() -> Rule {
    Rule::from("CONTINUE")
}

fn controls() -> Rule {
    not_in(
        Lang::Cloud,
        seq!(
            "CONTROLS",
            E::NamespaceSimpleName,
            seq!(
                "TYPE",
                alt!(seq!("TABLEVIEW USING SCREEN", E::Source), "TABSTRIP")
            )
        ),
    )
}

fn convert() -> Rule {
    seq!(
        "CONVERT",
        alt!(
            seq!(
                "TIME STAMP",
                E::Source,
                per!(
                    seq!("TIME ZONE", E::Source),
                    seq!(
                        "INTO",
                        per!(seq!("TIME", E::Target), seq!("DATE", E::Target))
                    ),
                    seq!("DAYLIGHT SAVING TIME", E::Target)
                )
            ),
            // SAP `abapconvert_date_utclong`: TIME tim [FRACTIONAL SECONDS fs]
            seq!(
                per!(
                    seq!("DATE", E::Source),
                    seq!(
                        "TIME",
                        E::Source,
                        opt_prio(seq!("FRACTIONAL SECONDS", E::Source))
                    )
                ),
                per!(
                    seq!("DAYLIGHT SAVING TIME", E::Source),
                    seq!("INTO TIME STAMP", E::Target),
                    seq!("TIME ZONE", E::Source),
                    seq!("INTO INVERTED-DATE", E::Target),
                    ver("7.54", seq!("INTO UTCLONG", E::Target))
                )
            ),
            seq!("INVERTED-DATE", E::Source, "INTO DATE", E::Target),
            ver(
                "7.54",
                // SAP `abapconvert_utclong`: TIME tim [FRACTIONAL SECONDS fs]
                seq!(
                    "UTCLONG",
                    E::Source,
                    per!(
                        seq!("TIME ZONE", E::Source),
                        seq!(
                            "INTO",
                            per!(
                                seq!(
                                    "TIME",
                                    E::Target,
                                    opt_prio(seq!("FRACTIONAL SECONDS", E::Target))
                                ),
                                seq!("DATE", E::Target)
                            )
                        ),
                        seq!("DAYLIGHT SAVING TIME", E::Target)
                    )
                )
            )
        )
    )
}

fn convert_text() -> Rule {
    seq!("CONVERT TEXT", E::Source, "INTO SORTABLE CODE", E::Target)
}

fn create_data() -> Rule {
    not_in(
        Lang::KeyUser,
        seq!(
            "CREATE DATA",
            E::Target,
            opt(alt!(
                seq!("TYPE HANDLE", E::SimpleSource3),
                seq!(
                    opt(seq!("AREA HANDLE", E::Source)),
                    seq!(
                        alt!(
                            seq!(
                                alt!(
                                    "TYPE",
                                    "TYPE REF TO",
                                    "TYPE TABLE OF",
                                    "TYPE TABLE OF REF TO",
                                    "TYPE SORTED TABLE OF",
                                    "TYPE HASHED TABLE OF",
                                    "TYPE STANDARD TABLE OF",
                                    "TYPE LINE OF"
                                ),
                                alt!(E::TypeName, E::Dynamic)
                            ),
                            seq!(
                                alt!(
                                    "LIKE",
                                    "LIKE HASHED TABLE OF",
                                    "LIKE LINE OF",
                                    "LIKE STANDARD TABLE OF",
                                    "LIKE SORTED TABLE OF",
                                    "LIKE TABLE OF"
                                ),
                                alt!(E::Source, E::Dynamic)
                            )
                        ),
                        opt(seq!(
                            "WITH",
                            alt!(
                                seq!(
                                    opt(alt!("UNIQUE", "NON-UNIQUE")),
                                    alt!("DEFAULT KEY", ver("7.40 SP02", "EMPTY KEY"))
                                ),
                                seq!(
                                    opt(alt!("UNIQUE", "NON-UNIQUE")),
                                    "KEY",
                                    alt!(plus(E::Field), E::Dynamic)
                                )
                            )
                        )),
                        opt(seq!("INITIAL SIZE", E::Source)),
                        opt(seq!("LENGTH", E::Source)),
                        opt(seq!("DECIMALS", E::Source))
                    )
                )
            ))
        ),
    )
}

fn create_object() -> Rule {
    not_in(
        Lang::KeyUser,
        seq!(
            "CREATE OBJECT",
            E::Target,
            opt_prio(per!(
                seq!("TYPE", alt_prio!(E::ClassName, E::Dynamic)),
                not_in(Lang::Cloud, seq!("AREA HANDLE", E::Source))
            )),
            opt_prio(alt_prio!(
                seq!("EXPORTING", E::ParameterListS),
                seq!("PARAMETER-TABLE", E::Source)
            )),
            opt_prio(alt_prio!(
                seq!("EXCEPTIONS", E::ParameterListExceptions),
                seq!("EXCEPTION-TABLE", E::Source)
            ))
        ),
    )
}

fn create_ole() -> Rule {
    not_in(
        Lang::Cloud,
        seq!(
            "CREATE OBJECT",
            E::Target,
            E::Source,
            opt("NO FLUSH"),
            opt("QUEUE-ONLY")
        ),
    )
}

fn data() -> Rule {
    seq!(
        "DATA",
        alt_prio!(
            E::DataDefinition,
            seq!(
                alt_prio!(Tok::WParenLeft, Tok::ParenLeft),
                E::Integer,
                alt_prio!(Tok::ParenRightW, Tok::ParenRight)
            )
        ),
        opt_prio("%_PREDEFINED")
    )
}

fn data_begin() -> Rule {
    seq!(
        "DATA",
        seq!(
            "BEGIN OF",
            alt_prio!(
                seq!("COMMON PART", opt_prio(E::DefinitionName)),
                seq!(
                    E::DefinitionName,
                    opt_prio("READ-ONLY"),
                    opt_prio(seq!("OCCURS", alt_prio!(E::Integer, E::FieldChain)))
                )
            )
        )
    )
}

fn data_end() -> Rule {
    seq!(
        "DATA",
        seq!(
            "END OF",
            alt_prio!(
                seq!("COMMON PART", opt_prio(E::DefinitionName)),
                E::DefinitionName
            )
        ),
        opt_prio(seq!(
            "VALID BETWEEN",
            E::ComponentName,
            "AND",
            E::ComponentName
        ))
    )
}

fn define() -> Rule {
    not_in(Lang::Cloud, seq!("DEFINE", E::MacroName))
}

fn delete_cluster() -> Rule {
    not_in(
        Lang::Cloud,
        seq!(
            "DELETE FROM DATABASE",
            E::NamespaceSimpleName,
            Tok::ParenLeft,
            E::SimpleName,
            Tok::ParenRightW,
            opt(seq!("CLIENT", E::Source)),
            "ID",
            E::Source
        ),
    )
}

fn delete_database() -> Rule {
    not_in(
        Lang::KeyUser,
        seq!(
            "DELETE",
            alt_prio!(
                seq!(
                    "FROM",
                    E::DatabaseTable,
                    opt_prio(E::SQLClient),
                    opt_prio(E::DatabaseConnection),
                    opt(seq!("WHERE", alt_prio!(E::SQLCond, E::Dynamic))),
                    opt_prio(alt_prio!(
                        ver(
                            "7.51",
                            seq!(
                                E::SQLOrderBy,
                                opt_prio(ver(
                                    "7.51",
                                    alt_prio!(
                                        seq!(E::SQLUpTo, opt_prio(E::SQLOffset)),
                                        seq!(E::SQLOffset, opt_prio(E::SQLUpTo))
                                    )
                                ))
                            )
                        ),
                        ver(
                            "7.51",
                            alt_prio!(
                                seq!(E::SQLUpTo, opt_prio(E::SQLOffset)),
                                seq!(E::SQLOffset, opt_prio(E::SQLUpTo))
                            )
                        )
                    )),
                    opt_prio(E::SQLDmlOptions)
                ),
                seq!(
                    E::DatabaseTable,
                    opt_prio(E::SQLClient),
                    opt_prio(E::DatabaseConnection),
                    opt_prio(seq!(
                        "FROM",
                        opt("TABLE"),
                        E::SQLSourceSimple,
                        opt_prio(E::SQLIndicators),
                        opt_prio(E::SQLMappingFromEntity)
                    )),
                    opt_prio(E::SQLDmlOptions)
                )
            )
        ),
    )
}

fn delete_dataset() -> Rule {
    not_in(Lang::Cloud, seq!("DELETE DATASET", E::Source))
}

fn delete_dynpro() -> Rule {
    not_in(Lang::Cloud, seq!("DELETE DYNPRO", E::Source))
}

fn delete_internal() -> Rule {
    seq!(
        "DELETE",
        alt!(
            seq!(
                "TABLE",
                E::Target,
                alt!(
                    per!(
                        seq!("INDEX", E::Source),
                        seq!("USING KEY", alt_prio!(E::SimpleName, E::Dynamic))
                    ),
                    seq!(
                        opt_prio(seq!("FROM", E::Source)),
                        opt_prio(seq!("USING KEY", alt_prio!(E::SimpleName, E::Dynamic)))
                    ),
                    // SAP `abapdelete_itab_key`: `{ comp | (name) } = operand`
                    seq!(
                        "WITH TABLE KEY",
                        opt(seq!(alt_prio!(E::SimpleName, E::Dynamic), "COMPONENTS")),
                        plus(alt!(E::ComponentCompare, seq!(E::Dynamic, "=", E::Source)))
                    )
                )
            ),
            seq!(
                "ADJACENT DUPLICATES FROM",
                E::Target,
                opt_prio(seq!("USING KEY", alt_prio!(E::SimpleName, E::Dynamic))),
                opt(seq!(
                    "COMPARING",
                    alt_prio!(
                        "ALL FIELDS",
                        plus(alt_prio!(
                            seq!(
                                E::SimpleFieldChain2,
                                opt_prio(E::FieldOffset),
                                opt_prio(E::FieldLength)
                            ),
                            E::Dynamic
                        ))
                    )
                )),
                opt_prio(seq!("USING KEY", alt_prio!(E::SimpleName, E::Dynamic)))
            ),
            seq!(
                E::Target,
                alt!(
                    per!(
                        seq!("INDEX", E::Source),
                        seq!("USING KEY", alt_prio!(E::SimpleName, E::Dynamic))
                    ),
                    seq!(
                        opt_prio(seq!("FROM", E::Source)),
                        opt_prio(seq!("TO", E::Source))
                    ),
                    // SAP `abapdelete_itab_key`: `{ comp | (name) } = operand`
                    seq!(
                        "WITH TABLE KEY",
                        opt(seq!(alt_prio!(E::SimpleName, E::Dynamic), "COMPONENTS")),
                        plus(alt!(E::ComponentCompare, seq!(E::Dynamic, "=", E::Source)))
                    )
                ),
                opt(seq!(
                    "WHERE",
                    alt!(E::ComponentCond, not_in(Lang::KeyUser, E::Dynamic))
                ))
            )
        )
    )
}

fn delete_memory() -> Rule {
    not_in(
        Lang::Cloud,
        seq!(
            "DELETE FROM",
            alt!(
                seq!("MEMORY ID", E::Source),
                seq!(
                    "SHARED",
                    alt_prio!("MEMORY", "BUFFER"),
                    E::Field,
                    "(",
                    re(r"^[\w%]+$"),
                    ")",
                    opt_prio(seq!("CLIENT", E::Source)),
                    seq!("ID", E::Source)
                )
            )
        ),
    )
}

fn delete_report() -> Rule {
    not_in(
        Lang::Cloud,
        seq!(
            "DELETE REPORT",
            E::Source,
            opt_prio(seq!("STATE", E::Source))
        ),
    )
}

fn delete_textpool() -> Rule {
    not_in(
        Lang::Cloud,
        seq!(
            "DELETE TEXTPOOL",
            E::Source,
            opt(seq!("LANGUAGE", E::Source)),
            opt(seq!("STATE", E::Source))
        ),
    )
}

fn demand() -> Rule {
    not_in(
        Lang::Cloud,
        seq!(
            "DEMAND",
            plus(seq!(E::Field, "=", E::Target)),
            "FROM CONTEXT",
            E::Field,
            opt(seq!("MESSAGES INTO", E::Target))
        ),
    )
}

fn describe() -> Rule {
    not_in(
        Lang::Cloud,
        seq!(
            "DESCRIBE",
            alt_prio!(
                seq!(
                    "TABLE",
                    E::FieldChain,
                    opt(per!(
                        seq!("LINES", E::Target),
                        seq!("KIND", E::Target),
                        seq!("OCCURS", E::Target)
                    ))
                ),
                seq!(
                    "FIELD",
                    E::Source,
                    per!(
                        seq!("TYPE", E::Target),
                        seq!("COMPONENTS", E::Target),
                        seq!(
                            "LENGTH",
                            E::Target,
                            opt(seq!("IN", alt!("BYTE", "CHARACTER"), "MODE"))
                        ),
                        seq!("DECIMALS", E::Target),
                        seq!("HELP-ID", E::Target),
                        seq!("OUTPUT-LENGTH", E::Target),
                        seq!("EDIT MASK", E::Target),
                        seq!("INTO", E::Target)
                    )
                ),
                seq!(
                    "DISTANCE BETWEEN",
                    E::Source,
                    "AND",
                    E::Source,
                    "INTO",
                    E::Target,
                    opt(seq!("IN", alt!("BYTE", "CHARACTER"), "MODE"))
                ),
                seq!(
                    "LIST",
                    per!(
                        seq!("NUMBER OF LINES", E::Target),
                        seq!("NUMBER OF PAGES", E::Target),
                        seq!("INDEX", E::Source),
                        seq!("LINE", E::Source),
                        seq!("PAGE", E::Source),
                        seq!("TOP-LINES", E::Target),
                        seq!("FIRST-LINE", E::Target),
                        seq!("LINES", E::Target),
                        seq!("LINE-SIZE", E::Target),
                        seq!("LINE-COUNT", E::Target)
                    )
                )
            )
        ),
    )
}

fn detail() -> Rule {
    not_in(Lang::Cloud, "DETAIL")
}

fn divide() -> Rule {
    not_in(Lang::KeyUser, seq!("DIVIDE", E::Target, "BY", E::Source))
}

fn do_() -> Rule {
    seq!(
        "DO",
        opt_prio(per!(
            plus(seq!(
                "VARYING",
                E::Target,
                "FROM",
                E::Source,
                "NEXT",
                E::Source,
                opt_prio(seq!("RANGE", E::Source))
            )),
            seq!(E::Source, "TIMES")
        ))
    )
}

fn dynpro_loop() -> Rule {
    seq!(
        "LOOP",
        alt_prio!(
            seq!(
                "AT",
                E::SimpleSource2,
                per!(
                    seq!("INTO", E::SimpleSource2),
                    seq!("WITH CONTROL", E::SimpleSource2),
                    seq!("CURSOR", E::SimpleSource2),
                    seq!("FROM", E::SimpleSource2),
                    seq!("TO", E::SimpleSource2)
                )
            ),
            seq!("WITH CONTROL", E::SimpleSource2)
        )
    )
}

fn editor_call() -> Rule {
    not_in(
        Lang::Cloud,
        seq!(
            "EDITOR-CALL FOR",
            opt_prio("REPORT"),
            E::Source,
            opt_prio(per!("DISPLAY-MODE", seq!("TITLE", E::SimpleSource3)))
        ),
    )
}

fn else_() -> Rule {
    Rule::from("ELSE")
}

fn else_if() -> Rule {
    seq!("ELSEIF", E::Cond)
}

fn end_at() -> Rule {
    Rule::from("ENDAT")
}

fn end_case() -> Rule {
    Rule::from("ENDCASE")
}

fn end_catch() -> Rule {
    not_in(Lang::Cloud, "ENDCATCH")
}

fn end_chain() -> Rule {
    not_in(Lang::Cloud, "ENDCHAIN")
}

fn end_class() -> Rule {
    Rule::from("ENDCLASS")
}

fn end_do() -> Rule {
    Rule::from("ENDDO")
}

fn end_enhancement() -> Rule {
    not_in(Lang::Cloud, "ENDENHANCEMENT")
}

fn end_enhancement_section() -> Rule {
    not_in(Lang::Cloud, "END-ENHANCEMENT-SECTION")
}

fn end_exec() -> Rule {
    not_in(Lang::Cloud, "ENDEXEC")
}

fn end_form() -> Rule {
    not_in(Lang::KeyUser, "ENDFORM")
}

fn end_function() -> Rule {
    not_in(Lang::KeyUser, "ENDFUNCTION")
}

fn end_if() -> Rule {
    Rule::from("ENDIF")
}

fn end_interface() -> Rule {
    Rule::from("ENDINTERFACE")
}

fn end_loop() -> Rule {
    Rule::from("ENDLOOP")
}

fn end_method() -> Rule {
    Rule::from("ENDMETHOD")
}

fn end_module() -> Rule {
    not_in(Lang::Cloud, "ENDMODULE")
}

fn end_of_definition() -> Rule {
    not_in(Lang::Cloud, "END-OF-DEFINITION")
}

fn end_of_page() -> Rule {
    not_in(Lang::Cloud, "END-OF-PAGE")
}

fn end_of_selection() -> Rule {
    not_in(Lang::Cloud, "END-OF-SELECTION")
}

fn end_on() -> Rule {
    not_in(Lang::Cloud, "ENDON")
}

fn end_provide() -> Rule {
    not_in(Lang::Cloud, "ENDPROVIDE")
}

fn end_select() -> Rule {
    Rule::from("ENDSELECT")
}

fn end_test_injection() -> Rule {
    Rule::from("END-TEST-INJECTION")
}

fn end_test_seam() -> Rule {
    ver("7.50", "END-TEST-SEAM")
}

fn end_try() -> Rule {
    Rule::from("ENDTRY")
}

fn end_while() -> Rule {
    Rule::from("ENDWHILE")
}

fn end_with() -> Rule {
    not_in(Lang::KeyUser, ver("7.51", "ENDWITH"))
}

fn enhancement() -> Rule {
    not_in(
        Lang::Cloud,
        seq!("ENHANCEMENT", plus(E::NamespaceSimpleName)),
    )
}

fn enhancement_point() -> Rule {
    not_in(
        Lang::Cloud,
        seq!(
            "ENHANCEMENT-POINT",
            E::FieldSub,
            "SPOTS",
            E::Field,
            opt("STATIC"),
            opt("INCLUDE BOUND")
        ),
    )
}

fn enhancement_section() -> Rule {
    not_in(
        Lang::Cloud,
        seq!(
            "ENHANCEMENT-SECTION",
            seq!(E::Field, star_prio(seq!(Tok::Dash, E::Field))),
            "SPOTS",
            E::Field,
            opt("STATIC"),
            opt("INCLUDE BOUND")
        ),
    )
}

fn events() -> Rule {
    seq!(
        alt_prio!("CLASS-EVENTS", "EVENTS"),
        E::EventName,
        opt_prio(seq!("EXPORTING", plus(E::MethodParamOptional)))
    )
}

fn exec_sql() -> Rule {
    not_in(
        Lang::Cloud,
        seq!("EXEC SQL", opt(seq!("PERFORMING", E::SimpleName))),
    )
}

fn exit() -> Rule {
    seq!("EXIT", opt(alt!("FROM SQL", "FROM STEP-LOOP")))
}

fn export() -> Rule {
    seq!(
        "EXPORT",
        alt!(
            plus(alt_prio!(
                seq!(
                    alt!(E::FieldSub, E::FieldSymbol),
                    seq!(alt_prio!("FROM", "="), E::Source)
                ),
                alt!(E::FieldSub, E::FieldSymbol)
            )),
            E::Dynamic,
            E::Constant
        ),
        "TO",
        alt!(
            seq!("DATA BUFFER", E::Target),
            not_in(Lang::KeyUser, seq!("MEMORY", opt(seq!("ID", E::Source)))),
            not_in(
                Lang::KeyUser,
                seq!(
                    "DATABASE",
                    seq!(
                        E::NamespaceSimpleName,
                        Tok::ParenLeft,
                        re(r"^[\w$%\^~]{2}$"),
                        Tok::ParenRightW
                    ),
                    per!(
                        seq!(alt_prio!("FROM", "="), E::Source),
                        seq!("CLIENT", E::Source),
                        seq!("ID", E::Source),
                        seq!("USING", E::Source)
                    )
                )
            ),
            seq!("INTERNAL TABLE", E::Target),
            not_in(
                Lang::KeyUser,
                seq!(
                    alt!("SHARED MEMORY", "SHARED BUFFER"),
                    seq!(
                        E::NamespaceSimpleName,
                        Tok::ParenLeft,
                        re(r"^[\w$%\^~]{2}$"),
                        Tok::ParenRightW
                    ),
                    per!(
                        seq!(alt_prio!("FROM", "="), E::Source),
                        seq!("CLIENT", E::Source),
                        seq!("ID", E::Source)
                    )
                )
            )
        ),
        opt(seq!("COMPRESSION", alt!("ON", "OFF"))),
        opt(seq!("CODE PAGE HINT", E::Source))
    )
}

fn export_dynpro() -> Rule {
    not_in(
        Lang::Cloud,
        seq!(
            "EXPORT DYNPRO",
            E::Source,
            E::Source,
            E::Source,
            E::Source,
            "ID",
            E::Source
        ),
    )
}

fn extract() -> Rule {
    not_in(Lang::Cloud, seq!("EXTRACT", opt(E::Field)))
}

fn fetch_next_cursor() -> Rule {
    not_in(
        Lang::Cloud,
        seq!(
            "FETCH NEXT CURSOR",
            E::SQLSourceSimple,
            alt!(E::SQLIntoStructure, E::SQLIntoTable, E::SQLIntoList),
            opt_prio(ver("7.52", E::SQLExtendedResult)),
            opt_prio(seq!("PACKAGE SIZE", E::SQLSourceSimple))
        ),
    )
}

fn field() -> Rule {
    not_in(
        Lang::Cloud,
        seq!(
            "FIELD",
            E::FieldChain,
            opt(alt_prio!(
                seq!(
                    "MODULE",
                    E::FormName,
                    opt(alt!(
                        "ON INPUT",
                        "ON REQUEST",
                        "ON CHAIN-REQUEST",
                        "ON CHAIN-INPUT",
                        "AT CURSOR-SELECTION"
                    ))
                ),
                seq!(
                    alt!(
                        "ON INPUT",
                        "ON REQUEST",
                        "ON CHAIN-REQUEST",
                        "ON CHAIN-INPUT",
                        "AT CURSOR-SELECTION"
                    ),
                    "MODULE",
                    E::FormName
                ),
                seq!(
                    "VALUES",
                    Tok::WParenLeft,
                    "BETWEEN",
                    E::Constant,
                    "AND",
                    E::Constant,
                    Tok::ParenRightW
                ),
                seq!("WITH", alt_prio!(E::FieldChain, E::Constant)),
                seq!(
                    "SELECT * FROM",
                    E::FieldChain,
                    "WHERE",
                    seq!(
                        seq!(E::FieldChain, "=", E::FieldChain),
                        star_prio(seq!("AND", seq!(E::FieldChain, "=", E::FieldChain)))
                    ),
                    opt(seq!("INTO", E::FieldChain)),
                    opt("WHENEVER NOT FOUND SEND ERRORMESSAGE")
                )
            ))
        ),
    )
}

fn field_group() -> Rule {
    not_in(Lang::Cloud, seq!("FIELD-GROUPS", plus(E::Field)))
}

fn fields() -> Rule {
    not_in(Lang::Cloud, seq!("FIELDS", E::FieldSub))
}

fn field_symbol() -> Rule {
    seq!(
        "FIELD-SYMBOLS",
        E::FieldSymbol,
        opt(alt!(
            ver("8.16", seq!("TYPE", "ANY", "STRUCTURE")),
            E::Type,
            E::TypeTable,
            seq!("STRUCTURE", E::TypeName, "DEFAULT", E::Field)
        ))
    )
}

fn find() -> Rule {
    seq!(
        "FIND",
        opt(alt_prio!("FIRST OCCURRENCE OF", "ALL OCCURRENCES OF")),
        E::FindType,
        E::Source,
        "IN",
        seq!(
            opt_prio(alt_prio!(
                "TABLE",
                "SECTION OFFSET",
                seq!("SECTION LENGTH", E::Source, "OF")
            )),
            E::Source
        ),
        opt(per!(
            "IGNORING CASE",
            "RESPECTING CASE",
            "IN BYTE MODE",
            "IN CHARACTER MODE",
            seq!("OF", E::Source),
            seq!("FROM", E::Source),
            seq!("TO", E::Source),
            seq!("MATCH OFFSET", E::Target),
            seq!("MATCH LINE", E::Target),
            seq!("MATCH COUNT", E::Target),
            seq!("MATCH LENGTH", E::Target),
            seq!("LENGTH", E::Source),
            seq!("RESULTS", E::Target),
            seq!("SUBMATCHES", E::Target),
            seq!("SUBMATCHES", E::Target, E::Target),
            seq!("SUBMATCHES", plus(E::Target))
        ))
    )
}

fn form() -> Rule {
    not_in(
        Lang::KeyUser,
        seq!(
            "FORM",
            E::FormName,
            alt!(
                "IMPLEMENTATION",
                seq!(
                    opt(E::FormTables),
                    opt(E::FormUsing),
                    opt(E::FormChanging),
                    opt(E::FormRaising)
                )
            )
        ),
    )
}

fn format() -> Rule {
    not_in(
        Lang::Cloud,
        seq!(
            "FORMAT",
            per!(
                "RESET",
                seq!(
                    "INTENSIFIED",
                    opt(alt_prio!(seq!("=", E::Source), "ON", "OFF"))
                ),
                seq!("INVERSE", opt(alt_prio!(seq!("=", E::Source), "ON", "OFF"))),
                seq!("HOTSPOT", opt(alt_prio!(seq!("=", E::Source), "ON", "OFF"))),
                seq!("FRAMES", alt_prio!(seq!("=", E::Source), "ON", "OFF")),
                seq!("INPUT", opt(alt_prio!(seq!("=", E::Source), "ON", "OFF"))),
                E::Color
            )
        ),
    )
}

fn form_definition() -> Rule {
    seq!(
        "FORM",
        E::FormName,
        "DEFINITION",
        opt(E::FormTables),
        opt(E::FormUsing),
        opt(E::FormChanging),
        opt(E::FormRaising)
    )
}

fn free() -> Rule {
    not_in(Lang::KeyUser, seq!("FREE", E::Target))
}

fn free_memory() -> Rule {
    not_in(
        Lang::Cloud,
        seq!("FREE MEMORY", opt_prio(seq!("ID", E::Source))),
    )
}

fn free_object() -> Rule {
    not_in(Lang::Cloud, seq!("FREE OBJECT", E::Target, opt("NO FLUSH")))
}

fn function_module() -> Rule {
    not_in(Lang::KeyUser, seq!("FUNCTION", E::Field))
}

fn function_pool() -> Rule {
    not_in(
        Lang::KeyUser,
        seq!(
            "FUNCTION-POOL",
            E::IncludeName,
            opt(per!(
                seq!("MESSAGE-ID", E::MessageClass),
                seq!("LINE-SIZE", E::Integer),
                "NO STANDARD PAGE HEADING"
            ))
        ),
    )
}

fn generate_dynpro() -> Rule {
    not_in(
        Lang::Cloud,
        seq!(
            "GENERATE DYNPRO",
            E::Source,
            E::Source,
            E::Source,
            E::Source,
            "ID",
            E::Source,
            "MESSAGE",
            E::Target,
            per!(seq!("LINE", E::Target), seq!("WORD", E::Target))
        ),
    )
}

fn generate_report() -> Rule {
    not_in(
        Lang::Cloud,
        seq!(
            "GENERATE REPORT",
            E::Source,
            opt(per!(
                "WITHOUT SELECTION-SCREEN",
                seq!("MESSAGE", E::Target),
                seq!("INCLUDE", E::Target),
                seq!("TRACE-FILE", E::Target),
                seq!("LINE", E::Target),
                seq!("WORD", E::Target),
                seq!("OFFSET", E::Target),
                "WITH PRECOMPILED HEADERS",
                "WITH TEST CODE",
                seq!("MESSAGE-ID", E::Target),
                seq!("SHORTDUMP-ID", E::Target),
                seq!("DIRECTORY ENTRY", E::Target)
            ))
        ),
    )
}

fn generate_subroutine() -> Rule {
    not_in(
        Lang::Cloud,
        seq!(
            "GENERATE SUBROUTINE POOL",
            E::Source,
            per!(
                seq!("NAME", E::Source),
                seq!("MESSAGE", E::Target),
                seq!("LINE", E::Target),
                seq!("WORD", E::Target),
                seq!("INCLUDE", E::Target),
                seq!("OFFSET", E::Target),
                seq!("MESSAGE-ID", E::Target),
                seq!("SHORTDUMP-ID", E::Target)
            )
        ),
    )
}

fn get() -> Rule {
    not_in(
        Lang::Cloud,
        seq!(
            "GET",
            E::Target,
            opt(per!("LATE", seq!("FIELDS", plus(E::Field))))
        ),
    )
}

fn get_badi() -> Rule {
    not_in(
        Lang::Cloud,
        seq!(
            "GET BADI",
            E::Target,
            opt(seq!("TYPE", E::Dynamic)),
            opt(seq!("FILTERS", E::ParameterListS)),
            opt(seq!("CONTEXT", E::Source))
        ),
    )
}

fn get_bit() -> Rule {
    seq!("GET BIT", E::Source, "OF", E::Source, "INTO", E::Target)
}

fn get_cursor() -> Rule {
    not_in(
        Lang::Cloud,
        seq!(
            "GET CURSOR",
            per!(
                seq!("LINE", E::Target),
                opt("DISPLAY"),
                seq!("FIELD", E::Target),
                seq!("OFFSET", E::Target),
                seq!("VALUE", E::Target),
                seq!("LENGTH", E::Target),
                seq!("AREA", E::Target)
            )
        ),
    )
}

fn get_dataset() -> Rule {
    not_in(
        Lang::Cloud,
        seq!(
            "GET DATASET",
            E::Target,
            opt(per!(
                seq!("POSITION", E::Target),
                seq!("ATTRIBUTES", E::Target)
            ))
        ),
    )
}

fn get_locale() -> Rule {
    not_in(
        Lang::Cloud,
        seq!(
            "GET LOCALE LANGUAGE",
            E::Target,
            seq!("COUNTRY", E::Target),
            seq!("MODIFIER", E::Target)
        ),
    )
}

fn get_parameter() -> Rule {
    not_in(
        Lang::Cloud,
        seq!("GET PARAMETER ID", E::Source, "FIELD", E::Target),
    )
}

fn get_permissions() -> Rule {
    ver(
        "7.54",
        seq!(
            "GET PERMISSIONS",
            opt_prio(seq!(opt("FORWARDING"), "PRIVILEGED")),
            opt_prio(seq!(
                "ONLY",
                alt_prio!(
                    seq!("GLOBAL", opt_prio(alt_prio!("FEATURES", "AUTHORIZATION"))),
                    seq!("INSTANCE", opt_prio(alt_prio!("FEATURES", "AUTHORIZATION"))),
                    alt_prio!("FEATURES", "AUTHORIZATION"),
                    E::Dynamic
                )
            )),
            alt_prio!(
                seq!(
                    "OF",
                    E::NamespaceSimpleName,
                    plus(seq!(
                        "ENTITY",
                        alt!(E::NamespaceSimpleName, E::EntityAssociation),
                        opt_prio(seq!("FROM", E::Source)),
                        "REQUEST",
                        E::Source,
                        "RESULT",
                        E::Target
                    ))
                ),
                seq!("OPERATIONS", E::Source),
                seq!(
                    "ENTITY",
                    alt!(E::NamespaceSimpleName, E::EntityAssociation),
                    opt_prio(seq!("FROM", E::Source)),
                    "REQUEST",
                    E::Source,
                    "RESULT",
                    E::Target
                )
            ),
            opt_prio(per!(seq!("FAILED", E::Target), seq!("REPORTED", E::Target)))
        ),
    )
}

fn get_pf_status() -> Rule {
    not_in(
        Lang::Cloud,
        seq!(
            "GET PF-STATUS",
            E::Target,
            opt(seq!("PROGRAM", E::Source)),
            opt(seq!("EXCLUDING", E::Source))
        ),
    )
}

fn get_property() -> Rule {
    not_in(
        Lang::Cloud,
        seq!(
            "GET PROPERTY OF",
            E::FieldSub,
            alt!(E::Source, E::ConstantString),
            "=",
            E::Source,
            opt("NO FLUSH"),
            opt(alt!("QUEUE-ONLY", "QUEUEONLY")),
            opt(E::OLEExporting)
        ),
    )
}

fn get_reference() -> Rule {
    not_in(
        Lang::KeyUser,
        seq!("GET REFERENCE OF", E::Source, "INTO", E::Target),
    )
}

fn get_run_time() -> Rule {
    not_in(Lang::Cloud, seq!("GET RUN TIME FIELD", E::Target))
}

fn get_time() -> Rule {
    not_in(
        Lang::KeyUser,
        seq!(
            "GET TIME",
            opt(seq!(alt!("STAMP FIELD", "FIELD"), E::Target))
        ),
    )
}

fn hide() -> Rule {
    not_in(Lang::Cloud, seq!("HIDE", E::Source))
}

fn if_() -> Rule {
    seq!("IF", E::Cond)
}

fn import() -> Rule {
    seq!(
        "IMPORT",
        alt!(
            plus(seq!(
                alt!(E::ComponentChainSimple, E::FieldSymbol),
                "=",
                E::Target
            )),
            plus(seq!(E::ComponentChainSimple, alt!("TO", "INTO"), E::Target)),
            E::Dynamic,
            plus(E::Target)
        ),
        "FROM",
        alt!(
            seq!("DATA BUFFER", E::Source),
            not_in(Lang::KeyUser, seq!("MEMORY", opt(seq!("ID", E::Source)))),
            not_in(
                Lang::KeyUser,
                seq!(
                    "DATABASE",
                    seq!(
                        E::NamespaceSimpleName,
                        Tok::ParenLeft,
                        re(r"^[\w$%\^]{2}$"),
                        Tok::ParenRightW
                    ),
                    per!(
                        seq!("TO", E::Target),
                        seq!("CLIENT", E::Source),
                        seq!("ID", E::Source),
                        seq!("USING", E::Source)
                    )
                )
            ),
            seq!("INTERNAL TABLE", E::Source),
            not_in(
                Lang::KeyUser,
                seq!(
                    alt!("SHARED MEMORY", "SHARED BUFFER"),
                    seq!(
                        E::NamespaceSimpleName,
                        Tok::ParenLeft,
                        re(r"^[\w$%\^]{2}$"),
                        Tok::ParenRightW
                    ),
                    per!(
                        seq!("TO", E::Target),
                        seq!("CLIENT", E::Source),
                        seq!("ID", E::Source)
                    )
                )
            ),
            not_in(Lang::KeyUser, seq!("LOGFILE ID", E::Source))
        ),
        opt(per!(
            "ACCEPTING PADDING",
            "IGNORING CONVERSION ERRORS",
            "IN CHAR-TO-HEX MODE",
            "IGNORING STRUCTURE BOUNDARIES",
            "ACCEPTING TRUNCATION",
            seq!("REPLACEMENT CHARACTER", E::Source),
            seq!("CODE PAGE INTO", E::Source),
            seq!("ENDIAN INTO", E::Source)
        ))
    )
}

fn import_dynpro() -> Rule {
    not_in(
        Lang::Cloud,
        seq!(
            "IMPORT DYNPRO",
            E::Target,
            E::Target,
            E::Target,
            E::Target,
            "ID",
            E::Source
        ),
    )
}

fn import_nametab() -> Rule {
    not_in(
        Lang::Cloud,
        seq!("IMPORT NAMETAB", E::Target, E::Target, "ID", E::Source),
    )
}

fn include() -> Rule {
    seq!("INCLUDE", E::IncludeName, opt("IF FOUND"))
}

fn include_type() -> Rule {
    seq!(
        "INCLUDE",
        alt!("TYPE", "STRUCTURE"),
        E::TypeName,
        opt(seq!("AS", E::Field)),
        opt(seq!("RENAMING WITH SUFFIX", E::ComponentName))
    )
}

fn infotypes() -> Rule {
    not_in(
        Lang::Cloud,
        seq!(
            "INFOTYPES",
            re(r"^\d\d\d\d$"),
            opt_prio(seq!("VALID FROM", E::Source, "TO", E::Source)),
            opt_prio(seq!("NAME", E::FieldSub)),
            opt_prio(seq!("OCCURS", E::Constant)),
            opt_prio("MODE N")
        ),
    )
}

fn initialization() -> Rule {
    not_in(Lang::Cloud, "INITIALIZATION")
}

fn input() -> Rule {
    not_in(Lang::Cloud, "INPUT")
}

fn insert_database() -> Rule {
    not_in(
        Lang::KeyUser,
        seq!(
            "INSERT",
            alt!(
                seq!(
                    E::DatabaseTable,
                    opt(alt!(
                        seq!(
                            opt(E::SQLClient),
                            opt(E::DatabaseConnection),
                            alt!(
                                seq!(
                                    "FROM",
                                    opt("TABLE"),
                                    E::SQLSource,
                                    opt_prio(E::SQLIndicators),
                                    opt_prio(E::SQLMappingFromEntity)
                                ),
                                seq!("FROM", seq!(Tok::WParenLeftW, E::Select, Tok::WParenRightW))
                            ),
                            opt("ACCEPTING DUPLICATE KEYS"),
                            opt_prio(E::SQLDmlOptions)
                        ),
                        E::SQLClient,
                        E::DatabaseConnection
                    ))
                ),
                seq!(
                    "INTO",
                    E::DatabaseTable,
                    opt(E::SQLClient),
                    opt(E::DatabaseConnection),
                    "VALUES",
                    E::SQLSource,
                    opt_prio(E::SQLIndicators),
                    opt_prio(E::SQLMappingFromEntity),
                    opt_prio(E::SQLDmlOptions)
                )
            )
        ),
    )
}

fn insert_field_group() -> Rule {
    seq!(
        "INSERT",
        alt!(ver("7.40 SP02", plus(E::Source)), plus(E::SimpleSource1)),
        seq!("INTO", E::Target)
    )
}

fn insert_internal() -> Rule {
    seq!(
        "INSERT",
        alt_prio!(
            seq!("TABLE", E::Source),
            seq!(
                alt_prio!(
                    "INITIAL LINE",
                    seq!(
                        "LINES OF",
                        alt_prio!(E::Source, E::Dynamic),
                        // SAP `abapinsert_itab_lines`: [USING KEY keyname]
                        opt(per!(
                            seq!("FROM", E::Source),
                            seq!("TO", E::Source),
                            seq!("USING KEY", alt_prio!(E::SimpleName, E::Dynamic))
                        ))
                    ),
                    alt!(E::SimpleSource4, ver("7.40 SP02", E::Source))
                ),
                alt!(
                    seq!(
                        "INTO TABLE",
                        E::Target,
                        opt(alt!(
                            seq!("REFERENCE INTO", E::Target),
                            seq!("ASSIGNING", E::FSTarget)
                        ))
                    ),
                    seq!(
                        seq!("INTO", E::Target),
                        opt(per!(
                            seq!("INDEX", E::Source),
                            alt!(
                                seq!("REFERENCE INTO", E::Target),
                                seq!("ASSIGNING", E::FSTarget)
                            )
                        ))
                    ),
                    per!(
                        seq!("INDEX", E::Source),
                        alt!(
                            seq!("REFERENCE INTO", E::Target),
                            seq!("ASSIGNING", E::FSTarget)
                        )
                    )
                )
            )
        )
    )
}

fn insert_report() -> Rule {
    not_in(
        Lang::Cloud,
        seq!(
            "INSERT REPORT",
            E::Source,
            "FROM",
            E::Source,
            opt(per!(
                seq!("STATE", E::Source),
                seq!("EXTENSION TYPE", E::Source),
                seq!("DIRECTORY ENTRY", E::Source),
                seq!("UNICODE ENABLING", E::Source),
                seq!("PROGRAM TYPE", E::Source),
                ver("7.50", seq!("VERSION", E::Source)),
                seq!("FIXED-POINT ARITHMETIC", E::Source),
                "KEEPING DIRECTORY ENTRY"
            ))
        ),
    )
}

fn insert_textpool() -> Rule {
    not_in(
        Lang::Cloud,
        seq!(
            "INSERT TEXTPOOL",
            E::Source,
            "FROM",
            E::Source,
            opt(seq!("LANGUAGE", E::Source)),
            opt(seq!("STATE", E::Source))
        ),
    )
}

fn interface() -> Rule {
    seq!("INTERFACE", E::InterfaceName, opt(E::ClassGlobal))
}

fn interface_def() -> Rule {
    seq!(
        "INTERFACES",
        E::InterfaceName,
        opt(alt!(
            seq!(E::AbstractMethods, opt(E::FinalMethods)),
            E::FinalMethods,
            "ALL METHODS ABSTRACT",
            "ALL METHODS FINAL",
            ver("7.40 SP02", "PARTIALLY IMPLEMENTED")
        )),
        opt(seq!(
            "DATA VALUES",
            plus(seq!(E::AttributeName, "=", E::Source))
        ))
    )
}

fn interface_deferred() -> Rule {
    not_in(
        Lang::KeyUser,
        seq!("INTERFACE", E::InterfaceName, "DEFERRED", opt("PUBLIC")),
    )
}

fn interface_load() -> Rule {
    not_in(Lang::KeyUser, seq!("INTERFACE", E::InterfaceName, "LOAD"))
}

fn leave() -> Rule {
    not_in(
        Lang::Cloud,
        seq!(
            "LEAVE",
            opt(alt!(
                seq!("TO CURRENT TRANSACTION", opt("AND SKIP FIRST SCREEN")),
                seq!(
                    opt("TO"),
                    "LIST-PROCESSING",
                    opt(seq!(E::AndReturn, "TO SCREEN", E::Source))
                ),
                "LIST-PROCESSING",
                "SCREEN",
                seq!("TO TRANSACTION", E::Source, opt("AND SKIP FIRST SCREEN")),
                "PROGRAM",
                seq!("TO SCREEN", E::Source)
            ))
        ),
    )
}

fn load_of_program() -> Rule {
    not_in(Lang::Cloud, "LOAD-OF-PROGRAM")
}

fn load_report() -> Rule {
    not_in(
        Lang::Cloud,
        seq!(
            "LOAD REPORT",
            E::Source,
            "PART",
            E::Source,
            "INTO",
            E::Target
        ),
    )
}

fn local() -> Rule {
    not_in(Lang::Cloud, seq!("LOCAL", E::FieldSub, opt(E::TableBody)))
}

fn log_point() -> Rule {
    not_in(
        Lang::Cloud,
        seq!(
            "LOG-POINT ID",
            E::NamespaceSimpleName,
            opt(seq!("SUBKEY", E::Source)),
            opt(seq!("FIELDS", plus(E::Source)))
        ),
    )
}

fn loop_() -> Rule {
    seq!(
        "LOOP AT",
        seq!(
            opt(seq!("SCREEN", fail())),
            opt(ver("7.40 SP08", "GROUP")),
            E::LoopSource,
            opt(per!(
                E::LoopTarget,
                seq!("FROM", E::Source),
                seq!("TO", E::Source),
                seq!(
                    "WHERE",
                    alt!(E::ComponentCond, not_in(Lang::KeyUser, E::Dynamic))
                ),
                seq!("USING KEY", alt_prio!(E::SimpleName, E::Dynamic)),
                ver("7.40 SP08", seq!("GROUP BY", E::LoopGroupBy)),
                ver("7.57", seq!("STEP", E::Source))
            ))
        )
    )
}

fn loop_at_screen() -> Rule {
    not_in(
        Lang::Cloud,
        seq!("LOOP AT SCREEN", opt(seq!("INTO", E::Target))),
    )
}

fn loop_extract() -> Rule {
    not_in(Lang::KeyUser, "LOOP")
}

fn merge_database() -> Rule {
    not_in(
        Lang::KeyUser,
        ver(
            "after 8.16",
            seq!(
                "MERGE",
                alt!(
                    seq!(
                        "INTO",
                        seq!(E::DatabaseTable, opt_prio(seq!("AS", E::SQLAsName))),
                        opt_prio(seq!(
                            "USING",
                            alt_prio!(
                                seq!(
                                    Tok::WParenLeftW,
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
                                    Tok::WParenRightW,
                                    opt_prio(seq!("AS", E::SQLAsName))
                                ),
                                seq!(
                                    Tok::WAt,
                                    E::SQLAsName,
                                    "AS",
                                    E::SQLAsName,
                                    opt_prio(seq!("DECLARE CLIENT", E::SQLFieldName))
                                ),
                                seq!(E::DatabaseTable, opt_prio(seq!("AS", E::SQLAsName)))
                            )
                        )),
                        opt_prio(seq!("ON", E::SQLCond)),
                        opt_prio(alt_prio!(
                            seq!(
                                seq!(
                                    "WHEN MATCHED",
                                    opt_prio(seq!("AND", E::SQLCond)),
                                    "THEN",
                                    alt_prio!(
                                        "DELETE",
                                        seq!(
                                            "UPDATE",
                                            alt_prio!(
                                                "ALL",
                                                seq!(
                                                    "SET",
                                                    seq!(
                                                        seq!(E::SQLFieldName, "=", E::SQLField),
                                                        star_prio(seq!(
                                                            ",",
                                                            seq!(E::SQLFieldName, "=", E::SQLField)
                                                        ))
                                                    )
                                                )
                                            )
                                        )
                                    )
                                ),
                                opt_prio(seq!(
                                    "WHEN NOT MATCHED",
                                    opt_prio(seq!("AND", E::SQLCond)),
                                    "THEN",
                                    seq!(
                                        "INSERT",
                                        alt_prio!(
                                            "ALL",
                                            seq!(
                                                seq!(
                                                    Tok::WParenLeftW,
                                                    E::SQLFieldName,
                                                    star_prio(seq!(",", E::SQLFieldName)),
                                                    Tok::WParenRightW
                                                ),
                                                "VALUES",
                                                seq!(
                                                    Tok::WParenLeftW,
                                                    E::SQLField,
                                                    star_prio(seq!(",", E::SQLField)),
                                                    Tok::WParenRightW
                                                )
                                            )
                                        )
                                    )
                                ))
                            ),
                            seq!(
                                "WHEN NOT MATCHED",
                                opt_prio(seq!("AND", E::SQLCond)),
                                "THEN",
                                seq!(
                                    "INSERT",
                                    alt_prio!(
                                        "ALL",
                                        seq!(
                                            seq!(
                                                Tok::WParenLeftW,
                                                E::SQLFieldName,
                                                star_prio(seq!(",", E::SQLFieldName)),
                                                Tok::WParenRightW
                                            ),
                                            "VALUES",
                                            seq!(
                                                Tok::WParenLeftW,
                                                E::SQLField,
                                                star_prio(seq!(",", E::SQLField)),
                                                Tok::WParenRightW
                                            )
                                        )
                                    )
                                )
                            )
                        )),
                        opt_prio(seq!(
                            "OPTIONS",
                            alt_prio!(
                                E::SQLPrivilegedAccess,
                                E::SQLClient,
                                seq!(E::SQLClient, E::SQLPrivilegedAccess),
                                seq!(E::SQLPrivilegedAccess, E::SQLClient)
                            )
                        ))
                    ),
                    E::Dynamic
                )
            ),
        ),
    )
}

fn message() -> Rule {
    seq!(
        "MESSAGE",
        alt_prio!(
            not_in(
                Lang::Cloud,
                seq!(
                    E::MessageSource,
                    opt(per!(
                        seq!("DISPLAY LIKE", E::Source),
                        seq!("INTO", E::Target),
                        not_in(Lang::KeyUser, seq!("RAISING", E::ExceptionName))
                    )),
                    opt(seq!(
                        "WITH",
                        E::MessageSourceSource,
                        opt(alt!(
                            alt_prio!(
                                per!(
                                    seq!("DISPLAY LIKE", E::Source),
                                    seq!("INTO", E::Target),
                                    not_in(Lang::KeyUser, seq!("RAISING", E::ExceptionName))
                                ),
                                E::MessageSourceSource
                            ),
                            seq!(
                                E::MessageSourceSource,
                                alt_prio!(
                                    per!(
                                        seq!("DISPLAY LIKE", E::Source),
                                        seq!("INTO", E::Target),
                                        not_in(Lang::KeyUser, seq!("RAISING", E::ExceptionName))
                                    ),
                                    E::MessageSourceSource
                                )
                            ),
                            seq!(
                                E::MessageSourceSource,
                                E::MessageSourceSource,
                                alt_prio!(
                                    per!(
                                        seq!("DISPLAY LIKE", E::Source),
                                        seq!("INTO", E::Target),
                                        not_in(Lang::KeyUser, seq!("RAISING", E::ExceptionName))
                                    ),
                                    E::MessageSourceSource
                                )
                            ),
                            seq!(
                                E::MessageSourceSource,
                                E::MessageSourceSource,
                                E::MessageSourceSource,
                                per!(
                                    seq!("DISPLAY LIKE", E::Source),
                                    seq!("INTO", E::Target),
                                    not_in(Lang::KeyUser, seq!("RAISING", E::ExceptionName))
                                )
                            )
                        ))
                    ))
                )
            ),
            not_in(
                Lang::Cloud,
                seq!(
                    E::MessageSourceSource,
                    seq!("TYPE", E::Source),
                    opt_prio(seq!("DISPLAY LIKE", E::Source)),
                    opt_prio(not_in(Lang::KeyUser, seq!("RAISING", E::ExceptionName)))
                )
            ),
            only_in(
                Lang::Cloud,
                seq!(
                    E::MessageSource,
                    alt!(
                        seq!(
                            opt(seq!(
                                "WITH",
                                E::Source,
                                opt(E::Source),
                                opt(E::Source),
                                opt(E::Source)
                            )),
                            alt_prio!(
                                seq!("INTO", E::Target),
                                not_in(Lang::KeyUser, seq!("RAISING", E::ExceptionName))
                            )
                        ),
                        seq!(
                            alt_prio!(
                                seq!("INTO", E::Target),
                                not_in(Lang::KeyUser, seq!("RAISING", E::ExceptionName))
                            ),
                            opt(seq!(
                                "WITH",
                                E::Source,
                                opt(E::Source),
                                opt(E::Source),
                                opt(E::Source)
                            ))
                        )
                    )
                )
            )
        )
    )
}

fn method_def() -> Rule {
    alt!(
        seq!(
            "METHODS",
            E::MethodName,
            not_in(
                Lang::KeyUser,
                seq!(
                    opt_prio("FINAL"),
                    "FOR ENTITY EVENT",
                    opt_prio("IMPORTING"),
                    plus(seq!(
                        alt_prio!(
                            seq!(
                                "REFERENCE",
                                Tok::ParenLeft,
                                E::MethodParamName,
                                Tok::ParenRightW
                            ),
                            E::MethodParamName
                        ),
                        "FOR",
                        E::TypeName
                    ))
                )
            )
        ),
        seq!(
            alt_prio!("CLASS-METHODS", "METHODS"),
            E::MethodName,
            alt!(
                seq!(
                    opt_prio(E::Abstract),
                    opt_prio(ver(
                        "7.40 SP08",
                        seq!("DEFAULT", alt_prio!("FAIL", "IGNORE"))
                    )),
                    E::EventHandler
                ),
                seq!(
                    opt_prio(alt_prio!(
                        "FINAL",
                        ver("7.40 SP08", seq!("DEFAULT", alt_prio!("FAIL", "IGNORE"))),
                        E::Abstract
                    )),
                    opt_prio(E::MethodDefImporting),
                    opt_prio(E::MethodDefExporting),
                    opt_prio(E::MethodDefChanging),
                    opt_prio(E::MethodDefReturning),
                    opt_prio(alt_prio!(
                        E::MethodDefRaising,
                        not_in(Lang::KeyUser, E::MethodDefExceptions)
                    ))
                ),
                seq!(
                    opt_prio(E::Abstract),
                    "FOR TESTING",
                    opt_prio(alt_prio!(
                        E::MethodDefRaising,
                        not_in(Lang::KeyUser, E::MethodDefExceptions)
                    ))
                ),
                not_in(
                    Lang::KeyUser,
                    seq!(
                        "FOR",
                        alt_prio!(
                            "DDL OBJECT OPTIONS CDS SESSION CLIENT REQUIRED",
                            seq!("TABLE FUNCTION", E::NamespaceSimpleName),
                            seq!(
                                "VALIDATE ON SAVE IMPORTING",
                                E::MethodParamName,
                                "FOR",
                                E::TypeName
                            ),
                            seq!(
                                "MODIFY",
                                opt("IMPORTING"),
                                plus(seq!(
                                    E::MethodParamName,
                                    alt!(
                                        seq!(
                                            "FOR ACTION",
                                            E::TypeName,
                                            opt_prio(seq!("RESULT", E::MethodParamName))
                                        ),
                                        seq!("FOR CREATE", alt!(E::TypeName, E::EntityAssociation)),
                                        seq!("FOR DELETE", E::TypeName),
                                        seq!("FOR UPDATE", E::TypeName)
                                    )
                                ))
                            ),
                            seq!(
                                "PRECHECK IMPORTING",
                                E::MethodParamName,
                                alt!(
                                    seq!(
                                        "FOR ACTION",
                                        E::TypeName,
                                        opt_prio(seq!("RESULT", E::MethodParamName))
                                    ),
                                    seq!("FOR CREATE", alt!(E::TypeName, E::EntityAssociation)),
                                    seq!("FOR DELETE", E::TypeName),
                                    seq!("FOR UPDATE", E::TypeName)
                                )
                            ),
                            seq!(
                                "DETERMINATION",
                                E::TypeName,
                                "IMPORTING",
                                E::MethodParamName,
                                "FOR",
                                E::TypeName
                            ),
                            seq!(
                                "VALIDATION",
                                E::TypeName,
                                "IMPORTING",
                                E::MethodParamName,
                                "FOR",
                                E::TypeName
                            ),
                            seq!(
                                "NUMBERING IMPORTING",
                                E::MethodParamName,
                                alt!(
                                    seq!(
                                        "FOR ACTION",
                                        E::TypeName,
                                        opt_prio(seq!("RESULT", E::MethodParamName))
                                    ),
                                    seq!("FOR CREATE", alt!(E::TypeName, E::EntityAssociation)),
                                    seq!("FOR DELETE", E::TypeName),
                                    seq!("FOR UPDATE", E::TypeName)
                                )
                            ),
                            seq!(
                                "READ IMPORTING",
                                E::MethodParamName,
                                alt_prio!(
                                    seq!(
                                        "FOR READ",
                                        alt!(E::TypeName, E::EntityAssociation),
                                        opt_prio(seq!("FULL", E::MethodParamName)),
                                        seq!("RESULT", E::MethodParamName),
                                        opt_prio(seq!("LINK", E::MethodParamName))
                                    ),
                                    seq!(
                                        "FOR FUNCTION",
                                        E::TypeName,
                                        seq!("RESULT", E::MethodParamName)
                                    )
                                )
                            ),
                            seq!(
                                "FEATURES IMPORTING",
                                E::MethodParamName,
                                "REQUEST",
                                E::NamespaceSimpleName,
                                "FOR",
                                E::NamespaceSimpleName,
                                seq!("RESULT", E::MethodParamName)
                            ),
                            seq!(
                                "BEHAVIOR IMPORTING",
                                E::MethodParamName,
                                "FOR CREATE",
                                E::TypeName,
                                E::MethodParamName,
                                "FOR UPDATE",
                                E::TypeName,
                                E::MethodParamName,
                                "FOR DELETE",
                                E::TypeName
                            ),
                            seq!(
                                "BEHAVIOR IMPORTING",
                                E::MethodParamName,
                                "FOR READ",
                                E::TypeName,
                                seq!("RESULT", E::MethodParamName)
                            ),
                            seq!(
                                "BEHAVIOR IMPORTING",
                                E::MethodParamName,
                                "FOR UPDATE",
                                E::TypeName
                            ),
                            seq!(
                                alt!("BEHAVIOR", "LOCK"),
                                "IMPORTING",
                                E::MethodParamName,
                                "FOR LOCK",
                                E::TypeName
                            ),
                            seq!(
                                "DETERMINE",
                                alt!("ON MODIFY", "ON SAVE"),
                                "IMPORTING",
                                E::MethodParamName,
                                "FOR",
                                E::TypeName
                            ),
                            seq!(
                                "GLOBAL AUTHORIZATION IMPORTING REQUEST",
                                E::MethodParamName,
                                "FOR",
                                E::TypeName,
                                seq!("RESULT", E::MethodParamName)
                            ),
                            seq!(
                                "GLOBAL FEATURES IMPORTING REQUEST",
                                E::MethodParamName,
                                "FOR",
                                E::TypeName,
                                seq!("RESULT", E::MethodParamName)
                            ),
                            seq!(
                                seq!(opt("INSTANCE"), "AUTHORIZATION IMPORTING"),
                                E::MethodParamName,
                                "REQUEST",
                                E::MethodParamName,
                                "FOR",
                                E::TypeName,
                                seq!("RESULT", E::MethodParamName)
                            ),
                            seq!(
                                "INSTANCE FEATURES IMPORTING",
                                E::MethodParamName,
                                "REQUEST",
                                E::MethodParamName,
                                "FOR",
                                E::TypeName,
                                seq!("RESULT", E::MethodParamName)
                            )
                        )
                    )
                ),
                not_in(
                    Lang::KeyUser,
                    seq!(
                        // SAP `abapmethods_amdp_options`,
                        // `abapmethods_amdp_options_client`; RETURNING for
                        // AMDP functions
                        ver("7.52", "AMDP OPTIONS"),
                        per!(
                            "READ-ONLY",
                            alt!(
                                seq!(
                                    "CDS SESSION CLIENT",
                                    alt_prio!("CURRENT", ver("8.16", "DEPENDENT"), E::Source)
                                ),
                                ver("7.58", "CLIENT INDEPENDENT")
                            )
                        ),
                        opt_prio(E::MethodDefImporting),
                        opt_prio(E::MethodDefExporting),
                        opt_prio(E::MethodDefChanging),
                        opt_prio(E::MethodDefReturning),
                        opt_prio(E::MethodDefRaising)
                    )
                ),
                "NOT AT END OF MODE",
                opt_prio(E::Redefinition)
            )
        )
    )
}

fn method_implementation() -> Rule {
    seq!(
        "METHOD",
        E::MethodName,
        opt_prio(not_in(
            Lang::KeyUser,
            seq!(
                "BY",
                alt!(
                    seq!(
                        "KERNEL MODULE",
                        plus(re(r"[\w~]+")),
                        opt_prio(alt_prio!("FAIL", "IGNORE"))
                    ),
                    seq!(
                        "DATABASE",
                        alt!("PROCEDURE", "FUNCTION", "GRAPH WORKSPACE"),
                        "FOR HDB",
                        E::Language,
                        opt(seq!(
                            "OPTIONS",
                            per!("READ-ONLY", "SUPPRESS SYNTAX ERRORS", "DETERMINISTIC")
                        )),
                        opt(seq!("USING", plus(E::SimpleFieldChain)))
                    )
                )
            )
        ))
    )
}

fn modify_database() -> Rule {
    not_in(
        Lang::KeyUser,
        seq!(
            "MODIFY",
            E::DatabaseTable,
            per!(
                E::DatabaseConnection,
                alt!(
                    seq!(
                        "FROM",
                        opt("TABLE"),
                        E::SQLSource,
                        opt_prio(E::SQLIndicators),
                        opt_prio(E::SQLMappingFromEntity)
                    ),
                    seq!("FROM", seq!(Tok::WParenLeftW, E::Select, Tok::WParenRightW))
                ),
                E::SQLClient
            ),
            opt_prio(E::SQLDmlOptions)
        ),
    )
}

/// SAP `abapmodify_entity_entities_fields`: `FROM tab | AUTO FILL CID WITH tab |
/// [AUTO FILL CID] FIELDS ( comp ... ) WITH tab | [AUTO FILL CID] SET FIELDS
/// WITH tab`.
fn eml_field_spec() -> Rule {
    alt!(
        seq!("FROM", E::Source),
        seq!(
            opt_prio("AUTO FILL CID"),
            alt!(
                seq!("WITH", E::Source),
                seq!("FIELDS (", plus(E::SimpleName), ")", "WITH", E::Source),
                seq!("SET FIELDS WITH", E::Source)
            )
        )
    )
}

fn modify_entities() -> Rule {
    ver(
        "7.54",
        seq!(
            "MODIFY",
            alt!(
                seq!(
                    opt_prio("AUGMENTING"),
                    "ENTITIES OF",
                    E::NamespaceSimpleName,
                    opt_prio(alt_prio!(
                        "IN LOCAL MODE",
                        seq!(opt("FORWARDING"), "PRIVILEGED")
                    )),
                    plus_prio(seq!(
                        "ENTITY",
                        E::NamespaceSimpleName,
                        plus(alt!(
                            seq!("UPDATE SET FIELDS WITH", E::Source),
                            seq!("CREATE SET FIELDS WITH", E::Source),
                            seq!(
                                "UPDATE",
                                seq!(
                                    "FIELDS (",
                                    plus(E::SimpleName),
                                    ")",
                                    seq!("WITH", E::Source)
                                )
                            ),
                            seq!("DELETE FROM", E::Source),
                            seq!(
                                "UPDATE FROM",
                                E::Source,
                                opt(seq!(
                                    "RELATING TO",
                                    E::NamespaceSimpleName,
                                    "BY",
                                    E::NamespaceSimpleName
                                ))
                            ),
                            seq!(
                                "CREATE",
                                opt(seq!("BY", E::EMLEntityPath)),
                                "FROM",
                                E::Source,
                                opt(seq!(
                                    "RELATING TO",
                                    E::NamespaceSimpleName,
                                    "BY",
                                    E::NamespaceSimpleName
                                ))
                            ),
                            seq!("EXECUTE", E::NamespaceSimpleName, "FROM", E::Source),
                            seq!("EXECUTE", E::NamespaceSimpleName, eml_field_spec()),
                            seq!(
                                "CREATE",
                                opt(seq!("BY", E::EMLEntityPath)),
                                opt_prio("AUTO FILL CID"),
                                alt_prio!(
                                    seq!("WITH", E::Source),
                                    seq!(
                                        "FIELDS (",
                                        plus(E::SimpleName),
                                        ")",
                                        seq!("WITH", E::Source)
                                    )
                                )
                            )
                        ))
                    ))
                ),
                seq!(
                    "ENTITIES",
                    opt_prio(alt_prio!(
                        "IN LOCAL MODE",
                        seq!(opt("FORWARDING"), "PRIVILEGED")
                    )),
                    "OPERATIONS",
                    E::Source
                ),
                seq!(
                    "ENTITY",
                    opt("IN LOCAL MODE"),
                    alt!(E::NamespaceSimpleName, E::EntityAssociation),
                    alt!(
                        seq!("EXECUTE", E::NamespaceSimpleName, "FROM", E::Source),
                        seq!(
                            "CREATE",
                            opt(seq!("BY", E::EMLEntityPath)),
                            "FROM",
                            E::Source,
                            opt(seq!(
                                "RELATING TO",
                                E::NamespaceSimpleName,
                                "BY",
                                E::NamespaceSimpleName
                            ))
                        ),
                        seq!(
                            "UPDATE",
                            seq!(
                                "FIELDS (",
                                plus(E::SimpleName),
                                ")",
                                seq!("WITH", E::Source)
                            )
                        ),
                        seq!("DELETE FROM", E::Source),
                        seq!("UPDATE SET FIELDS WITH", E::Source),
                        seq!(
                            "UPDATE FROM",
                            E::Source,
                            opt(seq!(
                                "RELATING TO",
                                E::NamespaceSimpleName,
                                "BY",
                                E::NamespaceSimpleName
                            ))
                        ),
                        seq!(
                            "CREATE",
                            seq!(
                                "FIELDS (",
                                plus(E::SimpleName),
                                ")",
                                seq!("WITH", E::Source)
                            ),
                            opt(seq!(
                                "CREATE BY",
                                E::EMLEntityPath,
                                seq!(
                                    "FIELDS (",
                                    plus(E::SimpleName),
                                    ")",
                                    seq!("WITH", E::Source)
                                )
                            ))
                        ),
                        seq!(
                            "CREATE BY",
                            E::EMLEntityPath,
                            seq!(
                                "FIELDS (",
                                plus(E::SimpleName),
                                ")",
                                seq!("WITH", E::Source)
                            )
                        ),
                        seq!(
                            "CREATE FROM",
                            E::Source,
                            plus(seq!("CREATE BY", E::EMLEntityPath, "FROM", E::Source))
                        ),
                        seq!(
                            "CREATE",
                            opt(seq!("BY", E::EMLEntityPath)),
                            eml_field_spec()
                        ),
                        seq!("EXECUTE", E::NamespaceSimpleName, eml_field_spec())
                    )
                )
            ),
            opt_prio(per!(
                seq!("FAILED", E::Target),
                seq!("RESULT", E::Target),
                seq!("MAPPED", E::Target),
                seq!("REPORTED", E::Target)
            ))
        ),
    )
}

/// SAP: `abapmodify_itab_single`, `abapmodify_itab_multiple`,
/// `abapmodify_itab_index`, `abapmodify_itab_table_key`,
/// `abapmodify_itab_result`; a key name may be dynamic (`abenkeyname`). The
/// additions may come in any order, like in abaplint.
fn modify_internal() -> Rule {
    let key = || seq!("USING KEY", alt_prio!(E::SimpleName, E::Dynamic));
    let transporting = || {
        seq!(
            "TRANSPORTING",
            plus(alt!(E::ComponentChainSimple, E::Dynamic))
        )
    };
    let index = || per!(seq!("INDEX", E::Source), transporting(), key());
    let result = || {
        per!(
            seq!("WHERE", alt_prio!(E::Dynamic, E::ComponentCond)),
            seq!(
                "ASSIGNING",
                E::FSTarget,
                opt_prio("CASTING"),
                opt_prio(ver("7.57", "ELSE UNASSIGN"))
            ),
            seq!("REFERENCE INTO", E::Target),
            key()
        )
    };
    alt!(
        seq!(
            "MODIFY",
            opt("TABLE"),
            alt!(E::Target, E::Dynamic),
            opt(alt!(
                index(),
                seq!(seq!("FROM", E::Source), opt(index())),
                seq!(index(), seq!("FROM", E::Source), opt(index()))
            )),
            opt(result())
        ),
        seq!(
            "MODIFY TABLE",
            alt!(E::Target, E::Dynamic),
            per!(seq!("FROM", E::Source), key(), transporting()),
            opt(result())
        )
    )
}

fn modify_line() -> Rule {
    not_in(
        Lang::Cloud,
        seq!(
            "MODIFY",
            alt_prio!("CURRENT LINE", seq!("LINE", E::Source)),
            opt(per!(
                seq!("INDEX", E::Source),
                seq!(
                    "FIELD VALUE",
                    plus(seq!(E::Source, opt_prio(seq!("FROM", E::Source))))
                ),
                seq!(
                    "FIELD FORMAT",
                    E::Source,
                    opt(seq!(
                        alt!("INVERSE", "INPUT", "COLOR", "HOTSPOT"),
                        opt(alt!(seq!("=", E::Source), alt!("ON", "OFF")))
                    ))
                ),
                seq!("OF PAGE", E::Source),
                seq!(
                    "LINE FORMAT",
                    per!(
                        seq!("INPUT", opt(alt!(seq!("=", E::Source), alt!("ON", "OFF")))),
                        seq!(
                            "INVERSE",
                            opt(alt!(seq!("=", E::Source), alt!("ON", "OFF")))
                        ),
                        "RESET",
                        seq!("INTENSIFIED", opt(alt!("ON", "OFF"))),
                        E::Color
                    )
                ),
                seq!("LINE VALUE FROM", E::Source),
                "OF CURRENT PAGE",
                seq!("INTENSIFIED", alt!("ON", "OFF")),
                E::Color
            ))
        ),
    )
}

fn modify_screen() -> Rule {
    not_in(
        Lang::Cloud,
        seq!("MODIFY SCREEN", opt(seq!("FROM", E::Source))),
    )
}

fn module() -> Rule {
    not_in(
        Lang::Cloud,
        seq!(
            "MODULE",
            E::FormName,
            opt(alt!(
                "INPUT",
                "OUTPUT",
                "ON CHAIN-REQUEST",
                "ON CHAIN-INPUT",
                "AT EXIT-COMMAND",
                seq!("SWITCH", E::NamespaceSimpleName)
            ))
        ),
    )
}

fn move_() -> Rule {
    alt!(
        seq!(
            not_in(Lang::Cloud, "MOVE"),
            alt_prio!(
                seq!("EXACT", E::Source, "TO", E::Target),
                seq!(E::Source, alt_prio!("?TO", "TO"), E::Target)
            ),
            opt(seq!("PERCENTAGE", E::Source, opt(alt!("LEFT", "RIGHT"))))
        ),
        seq!(
            E::Target,
            alt_prio!(
                alt_prio!(seq!("=", star(seq!(E::Target, "="))), "?="),
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
        )
    )
}

fn move_corresponding() -> Rule {
    seq!(
        "MOVE-CORRESPONDING",
        opt_prio("EXACT"),
        E::Source,
        "TO",
        E::SimpleTarget,
        opt_prio(ver("7.40 SP05", "EXPANDING NESTED TABLES")),
        opt_prio(ver("7.40 SP05", "KEEPING TARGET LINES"))
    )
}

fn multiply() -> Rule {
    not_in(Lang::KeyUser, seq!("MULTIPLY", E::Target, "BY", E::Source))
}

fn new_line() -> Rule {
    not_in(
        Lang::Cloud,
        seq!("NEW-LINE", opt(alt!("SCROLLING", "NO-SCROLLING"))),
    )
}

fn new_page() -> Rule {
    not_in(
        Lang::Cloud,
        seq!(
            "NEW-PAGE",
            opt(per!(
                seq!("PRINT", alt!("OFF", "ON")),
                alt!("NO-TITLE", "WITH-TITLE"),
                alt!("NO-HEADING", "WITH-HEADING"),
                "NO DIALOG",
                "NO-TOPOFPAGE",
                seq!("PARAMETERS", E::Source),
                seq!("LIST AUTHORITY", E::Source),
                seq!("IMMEDIATELY", E::Source),
                seq!("LIST DATASET", E::Source),
                seq!("SAP COVER PAGE", E::Source),
                seq!("NEW LIST IDENTIFICATION", E::Source),
                seq!("KEEP IN SPOOL", E::Source),
                seq!("DEPARTMENT", E::Source),
                seq!("LIST NAME", E::Source),
                seq!("LAYOUT", E::Source),
                seq!("DESTINATION", E::Source),
                seq!("RECEIVER", E::Source),
                seq!("COPIES", E::Source),
                seq!("COVER TEXT", E::Source),
                seq!("ARCHIVE PARAMETERS", E::Source),
                "NEW-SECTION",
                seq!("LINE-COUNT", E::Source),
                seq!("LINE-SIZE", E::Source)
            ))
        ),
    )
}

fn nodes() -> Rule {
    not_in(Lang::Cloud, seq!("NODES", E::Field))
}

fn on_change() -> Rule {
    not_in(
        Lang::Cloud,
        seq!("ON CHANGE OF", E::Target, star(seq!("OR", E::Target))),
    )
}

fn open_cursor() -> Rule {
    not_in(
        Lang::Cloud,
        seq!(
            "OPEN CURSOR",
            opt_prio("WITH HOLD"),
            E::SQLTarget,
            "FOR",
            E::Select,
            opt_prio(ver("7.52", seq!("WITH", E::SQLPrivilegedAccess))),
            opt_prio(E::SQLOptions)
        ),
    )
}

fn open_dataset() -> Rule {
    not_in(
        Lang::Cloud,
        seq!(
            "OPEN DATASET",
            E::Source,
            opt(per!(
                seq!("FOR", alt_prio!("OUTPUT", "INPUT", "UPDATE", "APPENDING")),
                seq!("TYPE", E::Source),
                seq!("IN", opt("LEGACY"), alt_prio!("BINARY MODE", "TEXT MODE")),
                "WITH BYTE-ORDER MARK",
                seq!("REPLACEMENT CHARACTER", E::Source),
                seq!("FILTER", E::Source),
                seq!("ENCODING", alt_prio!("DEFAULT", "UTF-8", "NON-UNICODE")),
                seq!("AT POSITION", E::Source),
                seq!("MESSAGE", E::Target),
                "IGNORING CONVERSION ERRORS",
                "SKIPPING BYTE-ORDER MARK",
                seq!("CODE PAGE", E::Source),
                seq!("WITH", alt_prio!("SMART", "NATIVE", "UNIX"), "LINEFEED"),
                "WITH WINDOWS LINEFEED"
            ))
        ),
    )
}

fn overlay() -> Rule {
    seq!(
        "OVERLAY",
        E::Target,
        "WITH",
        E::Source,
        opt(seq!("ONLY", E::Source))
    )
}

fn pack() -> Rule {
    not_in(Lang::Cloud, seq!("PACK", E::Source, "TO", E::Target))
}

fn parameter() -> Rule {
    not_in(
        Lang::Cloud,
        seq!(
            alt_prio!("PARAMETER", "PARAMETERS"),
            E::FieldSub,
            opt(E::ConstantFieldLength),
            opt(per!(
                seq!(
                    alt_prio!("TYPE", "LIKE"),
                    alt_prio!(E::TypeName, E::Dynamic),
                    opt_prio(E::FieldOffset),
                    opt_prio(E::FieldLength)
                ),
                seq!("DEFAULT", alt_prio!(E::Constant, E::FieldChain)),
                "OBLIGATORY",
                seq!("MATCHCODE OBJECT", E::Field),
                seq!("USER-COMMAND", re(r"^[\w\?\/]+$")),
                seq!("LENGTH", E::Constant),
                seq!("DECIMALS", E::Source),
                seq!("RADIOBUTTON GROUP", E::RadioGroupName),
                seq!("MEMORY ID", E::SimpleSource1),
                seq!("MODIF ID", E::Modif),
                "AS LISTBOX",
                seq!("VISIBLE LENGTH", E::Constant),
                seq!("FOR TABLE", E::DatabaseTable, opt("VALUE-REQUEST")),
                "VALUE CHECK",
                "NO-DISPLAY",
                "AS CHECKBOX",
                "LOWER CASE"
            ))
        ),
    )
}

fn perform() -> Rule {
    not_in(
        Lang::KeyUser,
        seq!(
            "PERFORM",
            alt!(
                not_in(
                    Lang::Cloud,
                    seq!(
                        E::FormName,
                        Tok::ParenLeft,
                        E::IncludeName,
                        Tok::ParenRightW
                    )
                ),
                seq!(
                    alt!(E::FormName, E::Dynamic),
                    opt(not_in(
                        Lang::Cloud,
                        seq!("IN PROGRAM", opt(alt!(E::Dynamic, E::IncludeName)))
                    ))
                )
            ),
            alt_prio!(
                seq!("OF", plus(E::FormName)),
                seq!(
                    opt("IF FOUND"),
                    opt(E::PerformTables),
                    opt(E::PerformUsing),
                    opt(E::PerformChanging),
                    opt("IF FOUND"),
                    opt(alt!(
                        seq!("ON COMMIT", opt(seq!("LEVEL", E::Source))),
                        "ON ROLLBACK"
                    ))
                )
            )
        ),
    )
}

fn position() -> Rule {
    not_in(Lang::Cloud, seq!("POSITION", E::Source))
}

fn print_control() -> Rule {
    not_in(
        Lang::Cloud,
        seq!(
            "PRINT-CONTROL",
            per!(
                seq!("INDEX-LINE", E::Source),
                seq!("FUNCTION", E::Source),
                seq!("LINE", E::Source),
                seq!("POSITION", E::Source),
                seq!("SIZE", E::Source),
                seq!("CPI", E::Source),
                seq!("LPI", E::Source),
                seq!("FONT", E::Source)
            )
        ),
    )
}

fn private() -> Rule {
    Rule::from("PRIVATE SECTION")
}

fn process_after_input() -> Rule {
    not_in(Lang::Cloud, "PROCESS AFTER INPUT")
}

fn process_before_output() -> Rule {
    not_in(Lang::Cloud, "PROCESS BEFORE OUTPUT")
}

fn process_on_help_request() -> Rule {
    not_in(Lang::Cloud, "PROCESS ON HELP-REQUEST")
}

fn process_on_value_request() -> Rule {
    not_in(Lang::Cloud, "PROCESS ON VALUE-REQUEST")
}

fn program() -> Rule {
    not_in(
        Lang::Cloud,
        seq!(
            "PROGRAM",
            opt(E::ReportName),
            opt(per!(
                seq!("MESSAGE-ID", E::MessageClass),
                seq!("LINE-SIZE", E::Source),
                "NO STANDARD PAGE HEADING",
                seq!("LINE-COUNT", E::Source, opt(E::FieldLength))
            ))
        ),
    )
}

fn protected() -> Rule {
    Rule::from("PROTECTED SECTION")
}

fn provide() -> Rule {
    not_in(
        Lang::Cloud,
        seq!(
            "PROVIDE",
            alt_prio!(
                plus_prio(seq!(
                    "FIELDS",
                    plus_prio(alt_prio!("*", E::ProvideFieldName)),
                    "FROM",
                    E::Source,
                    "INTO",
                    E::Target,
                    "VALID",
                    E::Field,
                    "BOUNDS",
                    E::Field,
                    "AND",
                    E::Field
                )),
                plus_prio(seq!(
                    plus(plus_prio(alt_prio!("*", E::ProvideFieldName))),
                    seq!("FROM", E::Source)
                ))
            ),
            opt(per!(
                seq!("BETWEEN", E::SimpleSource3, "AND", E::SimpleSource3),
                seq!("WHERE", E::Cond)
            ))
        ),
    )
}

fn public() -> Rule {
    Rule::from("PUBLIC SECTION")
}

fn put() -> Rule {
    not_in(Lang::Cloud, seq!("PUT", E::Field))
}

fn raise() -> Rule {
    seq!(
        "RAISE",
        alt_prio!(
            seq!(
                alt_prio!(seq!(opt_prio("RESUMABLE"), "EXCEPTION"), "SHORTDUMP"),
                alt_prio!(
                    seq!(
                        "TYPE",
                        E::ClassName,
                        opt(alt!(
                            ver(
                                "7.50",
                                alt!(
                                    seq!("MESSAGE", E::MessageSource, opt(E::RaiseWith)),
                                    seq!(
                                        "MESSAGE ID",
                                        E::Source,
                                        "NUMBER",
                                        alt_prio!(E::MessageNumber, E::Source),
                                        opt_prio(E::RaiseWith)
                                    )
                                )
                            ),
                            ver("7.52", "USING MESSAGE")
                        )),
                        opt_prio(seq!("EXPORTING", E::ParameterListS))
                    ),
                    ver("7.52", E::Source),
                    E::SimpleSource2
                )
            ),
            not_in(Lang::KeyUser, E::ExceptionName)
        )
    )
}

fn raise_entity_event() -> Rule {
    not_in(
        Lang::KeyUser,
        seq!("RAISE ENTITY EVENT", E::EventName, "FROM", E::Source),
    )
}

fn raise_event() -> Rule {
    not_in(
        Lang::KeyUser,
        seq!(
            "RAISE EVENT",
            E::EventName,
            opt(seq!("EXPORTING", E::ParameterListS))
        ),
    )
}

fn ranges() -> Rule {
    not_in(
        Lang::Cloud,
        seq!(
            "RANGES",
            E::DefinitionName,
            "FOR",
            E::SimpleFieldChain2,
            opt_prio(seq!("OCCURS", E::Source)),
            opt_prio(E::FieldLength)
        ),
    )
}

fn read_dataset() -> Rule {
    not_in(
        Lang::Cloud,
        seq!(
            "READ DATASET",
            E::Source,
            "INTO",
            E::Target,
            opt(per!(
                seq!("MAXIMUM LENGTH", E::Source),
                seq!("ACTUAL LENGTH", E::Target),
                seq!("LENGTH", E::Target)
            ))
        ),
    )
}

fn read_entities() -> Rule {
    ver(
        "7.54",
        seq!(
            "READ",
            alt!(
                // SAP `abapread_entities_operations`
                seq!(
                    "ENTITIES",
                    opt_prio(alt_prio!(
                        "IN LOCAL MODE",
                        seq!(opt("FORWARDING"), "PRIVILEGED")
                    )),
                    "OPERATIONS",
                    E::Source,
                    opt_prio(per!(seq!("FAILED", E::Target), seq!("REPORTED", E::Target)))
                ),
                seq!(
                    "ENTITIES OF",
                    E::NamespaceSimpleName,
                    opt("IN LOCAL MODE"),
                    plus(seq!(
                        "ENTITY",
                        E::NamespaceSimpleName,
                        plus(seq!(
                            opt(seq!("BY", E::EMLEntityPath)),
                            alt!(
                                seq!(
                                    "FIELDS",
                                    Tok::WParenLeftW,
                                    plus(E::SimpleName),
                                    Tok::WParenRightW,
                                    "WITH",
                                    E::Source
                                ),
                                seq!("FROM", E::Source),
                                seq!("ALL FIELDS WITH", E::Source),
                                seq!("EXECUTE", E::SimpleName, seq!("FROM", E::Source))
                            ),
                            opt_prio(seq!("RESULT", E::Target))
                        ))
                    )),
                    opt_prio(seq!("LINK", E::Target)),
                    opt_prio(per!(seq!("FAILED", E::Target), seq!("REPORTED", E::Target)))
                ),
                seq!(
                    "ENTITY",
                    opt("IN LOCAL MODE"),
                    alt!(E::NamespaceSimpleName, E::EntityAssociation),
                    plus(seq!(
                        alt!(
                            seq!("ALL FIELDS WITH", E::Source),
                            seq!(
                                "FIELDS",
                                Tok::WParenLeftW,
                                plus(E::SimpleName),
                                Tok::WParenRightW,
                                "WITH",
                                E::Source
                            ),
                            seq!("FROM", E::Source),
                            seq!(
                                "BY",
                                E::EMLEntityPath,
                                seq!(
                                    "FIELDS",
                                    Tok::WParenLeftW,
                                    plus(E::SimpleName),
                                    Tok::WParenRightW,
                                    "WITH",
                                    E::Source
                                )
                            ),
                            seq!("BY", E::EMLEntityPath, seq!("ALL FIELDS WITH", E::Source))
                        ),
                        seq!("RESULT", E::Target)
                    )),
                    opt_prio(seq!("FAILED", E::Target)),
                    opt_prio(seq!("REPORTED", E::Target))
                )
            )
        ),
    )
}

fn read_line() -> Rule {
    not_in(
        Lang::Cloud,
        seq!(
            "READ",
            alt!("CURRENT LINE", seq!("LINE", E::Source)),
            opt(per!(
                seq!("LINE VALUE INTO", E::Target),
                seq!("INDEX", E::Source),
                seq!(
                    "FIELD VALUE",
                    plus(seq!(E::Target, opt(seq!("INTO", E::Target))))
                ),
                seq!("OF PAGE", E::Source),
                "OF CURRENT PAGE"
            ))
        ),
    )
}

fn read_report() -> Rule {
    not_in(
        Lang::Cloud,
        seq!(
            "READ REPORT",
            E::Source,
            per!(
                seq!("STATE", E::Source),
                seq!("INTO", E::Target),
                seq!("MAXIMUM WIDTH INTO", E::Target)
            )
        ),
    )
}

/// What READ TABLE reads into and how, in any order with what selects the
/// line. SAP: `abapread_table_outdesc`, `abapread_table_transport_options`;
/// ELSE UNASSIGN only right after `ASSIGNING <fs> [CASTING]`, as the syntax
/// check of the SAP trial 2025 has it.
fn read_table_output() -> Vec<Rule> {
    vec![
        alt!(
            seq!(
                "ASSIGNING",
                E::FSTarget,
                opt_prio("CASTING"),
                ver("7.57", "ELSE UNASSIGN")
            ),
            E::ReadTableTarget
        ),
        seq!("COMPARING", alt!(plus(E::FieldSub), plus(E::Dynamic))),
        "CASTING".into(),
        seq!(
            "TRANSPORTING",
            alt_prio!("ALL FIELDS", "NO FIELDS", E::TransportingFields)
        ),
        "BINARY SEARCH".into(),
    ]
}

/// SAP: `abapread_table`, with `WHERE log_exp | (cond_syntax)` of
/// `abapread_table_where`.
fn read_table() -> Rule {
    let with = |mut select: Vec<Rule>| {
        select.extend(read_table_output());
        Rule::Per(select)
    };
    seq!(
        "READ TABLE",
        alt!(E::SimpleSource2, ver("7.40 SP02", E::Source)),
        opt(alt!(
            with(vec![
                alt!(
                    seq!("INDEX", E::Source),
                    seq!("FROM", E::Source),
                    ver("8.16", seq!("WHERE", alt_prio!(E::Dynamic, E::Cond)))
                ),
                seq!(
                    "USING KEY",
                    alt!(E::Field, not_in(Lang::KeyUser, E::Dynamic))
                ),
            ]),
            with(vec![seq!(
                alt_prio!("WITH KEY", "WITH TABLE KEY"),
                alt!(
                    E::ComponentCompareSimple,
                    seq!(
                        not_in(Lang::KeyUser, alt!(E::Field, E::Dynamic)),
                        "COMPONENTS",
                        E::ComponentCompareSimple
                    ),
                    seq!(opt_prio("="), E::Source)
                )
            )]),
            Rule::Per(read_table_output())
        ))
    )
}

fn read_textpool() -> Rule {
    not_in(
        Lang::Cloud,
        seq!(
            "READ TEXTPOOL",
            E::Source,
            per!(
                seq!("INTO", E::SimpleTarget),
                seq!("LANGUAGE", E::Source),
                seq!("STATE", E::Source)
            )
        ),
    )
}

fn receive() -> Rule {
    not_in(
        Lang::Cloud,
        seq!(
            "RECEIVE RESULTS FROM FUNCTION",
            E::FunctionName,
            opt("KEEPING TASK"),
            E::ReceiveParameters
        ),
    )
}

fn refresh() -> Rule {
    not_in(
        Lang::Cloud,
        seq!(
            "REFRESH",
            E::Target,
            opt_prio(seq!("FROM TABLE", E::DatabaseTable))
        ),
    )
}

fn refresh_control() -> Rule {
    not_in(
        Lang::Cloud,
        seq!("REFRESH CONTROL", E::Source, "FROM SCREEN", E::Source),
    )
}

fn reject() -> Rule {
    not_in(Lang::Cloud, seq!("REJECT", opt(E::Source)))
}

fn replace() -> Rule {
    seq!(
        "REPLACE",
        per!(
            seq!(
                opt("IN"),
                "SECTION",
                per!(seq!("OFFSET", E::Source), seq!("LENGTH", E::Source)),
                "OF",
                E::Target
            ),
            seq!(
                opt(alt_prio!(
                    "ALL OCCURRENCES",
                    "ALL OCCURENCES",
                    "FIRST OCCURENCE",
                    "FIRST OCCURRENCE"
                )),
                seq!(opt("OF"), E::FindType, E::Source)
            )
        ),
        opt(seq!("IN", opt("TABLE"), E::Target)),
        // SAP `abapreplace_itab`, `abapreplace_options`: the table range,
        // VERBATIM and RESULTS too
        opt(per!(
            seq!("WITH", E::Source),
            seq!("INTO", E::Target),
            alt_prio!("IGNORING CASE", "RESPECTING CASE"),
            alt!("IN CHARACTER MODE", "IN BYTE MODE"),
            seq!("REPLACEMENT COUNT", E::Target),
            seq!("REPLACEMENT OFFSET", E::Target),
            seq!("REPLACEMENT LENGTH", E::Target),
            seq!("REPLACEMENT LINE", E::Target),
            seq!("LENGTH", E::Source),
            seq!("RESULTS", E::Target),
            "VERBATIM",
            seq!("FROM", E::Source, opt_prio(seq!("OFFSET", E::Source))),
            seq!("TO", E::Source, opt_prio(seq!("OFFSET", E::Source)))
        ))
    )
}

fn report() -> Rule {
    not_in(
        Lang::Cloud,
        seq!(
            "REPORT",
            opt(E::ReportName),
            opt(per!(
                "NO STANDARD PAGE HEADING",
                seq!("LINE-SIZE", E::Integer),
                seq!(
                    "LINE-COUNT",
                    E::Integer,
                    opt(seq!(
                        Tok::ParenLeft,
                        E::Integer,
                        alt!(Tok::ParenRightW, Tok::ParenRight)
                    ))
                ),
                seq!("USING DATABASE", E::Field),
                seq!("MESSAGE-ID", E::MessageClass)
            ))
        ),
    )
}

fn reserve() -> Rule {
    not_in(Lang::Cloud, seq!("RESERVE", E::Source, "LINES"))
}

fn resume() -> Rule {
    Rule::from("RESUME")
}

fn retry() -> Rule {
    ver("7.02", "RETRY")
}

fn return_() -> Rule {
    seq!("RETURN", opt_prio(ver("7.58", E::Source)))
}

fn rollback() -> Rule {
    not_in(
        Lang::KeyUser,
        seq!("ROLLBACK", alt_prio!("WORK", E::DatabaseConnection)),
    )
}

fn rollback_entities() -> Rule {
    not_in(Lang::KeyUser, ver("7.54", "ROLLBACK ENTITIES"))
}

fn scan() -> Rule {
    not_in(
        Lang::Cloud,
        seq!(
            "SCAN ABAP-SOURCE",
            E::Source,
            per!(
                seq!("TOKENS INTO", E::Target),
                seq!("LEVELS INTO", E::Target),
                seq!("FROM", E::Source),
                seq!("TO", E::Source),
                seq!("STATEMENTS INTO", E::Target),
                seq!("STRUCTURES INTO", E::Target),
                seq!("KEYWORDS FROM", E::Source),
                seq!("WORD INTO", E::Target),
                seq!("LINE INTO", E::Target),
                seq!("OFFSET INTO", E::Target),
                seq!("OVERFLOW INTO", E::Target),
                seq!("MESSAGE INTO", E::Target),
                seq!("INCLUDE PROGRAM FROM", E::Source),
                seq!("INCLUDE INTO", E::Target),
                seq!("FRAME PROGRAM FROM", E::Source),
                seq!("ENHANCEMENT OPTIONS INTO", E::Target),
                seq!("ENHANCEMENTS INTO", E::Target),
                seq!("PROGRAM FROM", E::Source),
                seq!("REPLACING", E::Source),
                "WITH ANALYSIS",
                "WITH COMMENTS",
                "WITH TYPE-POOLS",
                "WITH INCLUDES",
                "WITHOUT TRMAC",
                "WITH DECLARATIONS",
                "WITH BLOCKS",
                "PRESERVING IDENTIFIER ESCAPING",
                "WITH LIST TOKENIZATION",
                "WITH EXPLICIT ENHANCEMENTS",
                "WITH IMPLICIT ENHANCEMENTS",
                "WITH INACTIVE ENHANCEMENTS",
                seq!("WITH PRAGMAS", E::Source),
                seq!("ID", E::Source, "TABLE", E::Source)
            )
        ),
    )
}

fn scroll_list() -> Rule {
    not_in(
        Lang::Cloud,
        seq!(
            "SCROLL LIST",
            per!(
                seq!("INDEX", E::Source),
                seq!(
                    alt!(
                        seq!(
                            "TO",
                            alt!("FIRST PAGE", "LAST PAGE", seq!("PAGE", E::Source))
                        ),
                        "BACKWARD",
                        "FORWARD"
                    ),
                    opt(seq!(E::Source, "PAGES"))
                ),
                seq!(alt!("LEFT", "RIGHT"), opt(seq!("BY", E::Source, "PLACES"))),
                seq!("TO COLUMN", E::Source),
                seq!("LINE", E::Source)
            )
        ),
    )
}

fn search() -> Rule {
    not_in(
        Lang::Cloud,
        seq!(
            "SEARCH",
            E::Source,
            "FOR",
            E::Source,
            opt(per!(
                alt_prio!("IN BYTE MODE", "IN CHARACTER MODE"),
                seq!("STARTING AT", E::Source),
                seq!("ENDING AT", E::Source),
                "AND MARK"
            ))
        ),
    )
}

fn select() -> Rule {
    Rule::from(E::Select)
}

fn selection_screen() -> Rule {
    not_in(
        Lang::Cloud,
        seq!(
            "SELECTION-SCREEN",
            alt_prio!(
                seq!(
                    "COMMENT",
                    seq!(
                        opt(re(r"^\/?[\d\w]+$")),
                        alt_prio!(Tok::ParenLeft, Tok::WParenLeft),
                        E::Integer,
                        alt_prio!(Tok::ParenRightW, Tok::ParenRight)
                    ),
                    opt(alt_prio!(E::TextElement, E::InlineField)),
                    opt(per!(
                        seq!("FOR FIELD", E::FieldSub, opt_prio(seq!("ID", re(r"^\w+$")))),
                        seq!("MODIF ID", E::Modif),
                        seq!("VISIBLE LENGTH", re(r"^\d+$"))
                    ))
                ),
                seq!("FUNCTION KEY", E::Integer),
                seq!("SKIP", opt(E::Integer)),
                seq!(
                    "POSITION",
                    alt_prio!(
                        re(r"^(0?[1-9]|[1234567][0-9]|8[0-3])$"),
                        alt_prio!("POS_LOW", "POS_HIGH")
                    ),
                    opt(seq!("FOR TABLE", E::Field))
                ),
                seq!("INCLUDE BLOCKS", E::BlockName),
                seq!("INCLUDE SELECT-OPTIONS", E::Field),
                seq!(
                    "PUSHBUTTON",
                    seq!(
                        opt(re(r"^\/?[\d\w]+$")),
                        alt_prio!(Tok::ParenLeft, Tok::WParenLeft),
                        E::Integer,
                        alt_prio!(Tok::ParenRightW, Tok::ParenRight)
                    ),
                    alt_prio!(E::TextElement, E::InlineField),
                    seq!("USER-COMMAND", alt!(E::Field, E::Constant)),
                    opt(seq!("MODIF ID", E::Modif)),
                    opt(seq!("VISIBLE LENGTH", re(r"^\d+$")))
                ),
                seq!(
                    "TAB",
                    Tok::WParenLeft,
                    E::Integer,
                    Tok::ParenRightW,
                    alt_prio!(E::TextElement, E::InlineField),
                    seq!("USER-COMMAND", alt!(E::Field, E::Constant)),
                    opt(seq!(
                        "DEFAULT",
                        opt(seq!("PROGRAM", E::Field)),
                        "SCREEN",
                        E::Integer
                    )),
                    opt(seq!("MODIF ID", E::Modif))
                ),
                seq!(
                    "ULINE",
                    opt(seq!(
                        opt(re(r"^\/?[\d\w]+$")),
                        alt_prio!(Tok::ParenLeft, Tok::WParenLeft),
                        E::Integer,
                        alt_prio!(Tok::ParenRightW, Tok::ParenRight)
                    )),
                    opt(seq!("MODIF ID", E::Modif))
                ),
                seq!(
                    "BEGIN OF BLOCK",
                    E::BlockName,
                    opt_prio("WITH FRAME"),
                    opt_prio(seq!("TITLE", alt_prio!(E::TextElement, E::InlineField))),
                    opt_prio("NO INTERVALS")
                ),
                seq!(
                    "BEGIN OF TABBED BLOCK",
                    E::BlockName,
                    "FOR",
                    E::Integer,
                    "LINES",
                    opt_prio("NO INTERVALS")
                ),
                seq!("END OF BLOCK", E::BlockName),
                "BEGIN OF LINE",
                "END OF LINE",
                seq!("INCLUDE PARAMETERS", E::Field),
                seq!(
                    "BEGIN OF SCREEN",
                    E::Integer,
                    opt(per!(
                        seq!("AS", alt!("WINDOW", "SUBSCREEN")),
                        seq!("TITLE", alt_prio!(E::TextElement, E::InlineField)),
                        "NO INTERVALS",
                        seq!("NESTING LEVEL", E::Source)
                    ))
                ),
                seq!("END OF SCREEN", E::Integer),
                seq!("EXCLUDE", alt!("IDS", "PARAMETERS"), re(r"^\w+$")),
                seq!("BEGIN OF VERSION", re(r"^\w+$"), E::TextElement),
                seq!("END OF VERSION", re(r"^\w+$")),
                seq!(
                    "DEFAULT",
                    opt(seq!("PROGRAM", E::Field)),
                    "SCREEN",
                    E::Integer
                )
            )
        ),
    )
}

fn select_loop() -> Rule {
    Rule::from(E::Select)
}

fn select_option() -> Rule {
    not_in(
        Lang::Cloud,
        seq!(
            "SELECT-OPTIONS",
            E::FieldSub,
            "FOR",
            alt!(E::FieldChain, E::Dynamic),
            opt(per!(
                seq!(
                    "DEFAULT",
                    alt!(E::Constant, E::FieldChain),
                    opt(seq!("TO", alt!(E::Constant, E::FieldChain)))
                ),
                seq!("OPTION", alt!("CP", E::Field)),
                seq!("SIGN", E::Field),
                seq!("MEMORY ID", E::SimpleSource1),
                seq!("MATCHCODE OBJECT", E::SimpleFieldChain),
                seq!("VISIBLE LENGTH", E::Source),
                seq!("MODIF ID", E::Modif),
                "NO DATABASE SELECTION",
                "LOWER CASE",
                "NO-EXTENSION",
                "NO INTERVALS",
                "NO-DISPLAY",
                "OBLIGATORY"
            ))
        ),
    )
}

fn set_bit() -> Rule {
    seq!(
        "SET BIT",
        E::Source,
        "OF",
        E::Target,
        opt(seq!("TO", E::Source))
    )
}

fn set_blank() -> Rule {
    not_in(Lang::Cloud, seq!("SET BLANK LINES", alt!("ON", "OFF")))
}

fn set_country() -> Rule {
    not_in(Lang::Cloud, seq!("SET COUNTRY", E::Source))
}

fn set_cursor() -> Rule {
    not_in(
        Lang::Cloud,
        seq!(
            "SET CURSOR",
            alt_prio!(
                per!(
                    seq!("FIELD", E::Source),
                    seq!("OFFSET", E::Source),
                    seq!("LINE", E::Source)
                ),
                seq!(E::Source, E::Source)
            )
        ),
    )
}

fn set_dataset() -> Rule {
    not_in(
        Lang::Cloud,
        seq!(
            "SET DATASET",
            E::Source,
            "POSITION",
            alt!(E::Source, "END OF FILE")
        ),
    )
}

fn set_extended_check() -> Rule {
    not_in(Lang::Cloud, seq!("SET EXTENDED CHECK", alt!("OFF", "ON")))
}

fn set_handler() -> Rule {
    not_in(
        Lang::KeyUser,
        seq!(
            "SET HANDLER",
            // SAP: "field" expected after "FOR"
            plus(seq!(stop_before_any(&["FOR"]), E::MethodSource)),
            opt_prio(seq!("FOR", alt_prio!("ALL INSTANCES", E::Source))),
            opt_prio(seq!("ACTIVATION", E::Source))
        ),
    )
}

fn set_language() -> Rule {
    not_in(Lang::Cloud, seq!("SET LANGUAGE", E::Source))
}

fn set_left() -> Rule {
    not_in(
        Lang::Cloud,
        seq!("SET LEFT SCROLL-BOUNDARY", opt(seq!("COLUMN", E::Source))),
    )
}

fn set_locale() -> Rule {
    not_in(
        Lang::Cloud,
        seq!(
            "SET LOCALE LANGUAGE",
            E::Source,
            opt(seq!("COUNTRY", E::Source)),
            opt(seq!("MODIFIER", E::Source))
        ),
    )
}

fn set_locks() -> Rule {
    not_in(
        Lang::KeyUser,
        ver(
            "7.54",
            seq!(
                "SET LOCKS OF",
                E::SimpleName,
                "ENTITY",
                E::SimpleName,
                "FROM",
                E::Source,
                "FAILED",
                E::Target,
                "REPORTED",
                E::Target
            ),
        ),
    )
}

fn set_margin() -> Rule {
    not_in(Lang::Cloud, seq!("SET MARGIN", E::Source, opt(E::Source)))
}

fn set_parameter() -> Rule {
    not_in(
        Lang::Cloud,
        seq!("SET PARAMETER ID", E::Source, "FIELD", E::Source),
    )
}

fn set_pf_status() -> Rule {
    not_in(
        Lang::Cloud,
        seq!(
            "SET PF-STATUS",
            E::Source,
            opt(per!(
                seq!("OF PROGRAM", E::Source),
                "IMMEDIATELY",
                seq!("EXCLUDING", E::Source)
            ))
        ),
    )
}

fn set_property() -> Rule {
    not_in(
        Lang::Cloud,
        seq!(
            "SET PROPERTY OF",
            E::Source,
            alt!(E::Constant, E::Field),
            "=",
            E::Source,
            opt_prio("NO FLUSH"),
            opt(E::OLEExporting),
            opt("QUEUEONLY")
        ),
    )
}

fn set_run_time() -> Rule {
    not_in(
        Lang::Cloud,
        seq!(
            "SET RUN TIME",
            alt!(
                seq!("CLOCK RESOLUTION", alt!("LOW", "HIGH")),
                seq!("ANALYZER", alt!("ON", "OFF"))
            )
        ),
    )
}

fn set_screen() -> Rule {
    not_in(Lang::Cloud, seq!("SET SCREEN", E::Source))
}

fn set_titlebar() -> Rule {
    not_in(
        Lang::Cloud,
        seq!(
            "SET TITLEBAR",
            E::Source,
            opt(seq!("OF PROGRAM", E::Source)),
            opt(seq!("WITH", plus(E::Source)))
        ),
    )
}

fn set_update_task() -> Rule {
    not_in(Lang::Cloud, "SET UPDATE TASK LOCAL")
}

fn set_user_command() -> Rule {
    not_in(Lang::Cloud, seq!("SET USER-COMMAND", E::Source))
}

fn shift() -> Rule {
    seq!(
        "SHIFT",
        E::Target,
        // SAP `abapshift`: SHIFT dobj [{[places] [direction]} | deleting]
        // [IN {CHARACTER|BYTE} MODE], deleting is
        // {LEFT DELETING LEADING} | {RIGHT DELETING TRAILING} mask
        opt(alt!(
            seq!(
                alt_prio!("LEFT DELETING LEADING", "RIGHT DELETING TRAILING"),
                E::Source,
                opt(seq!("IN", alt_prio!("CHARACTER", "BYTE"), "MODE"))
            ),
            per!(
                seq!("UP TO", E::Source),
                seq!("IN", alt_prio!("CHARACTER", "BYTE"), "MODE"),
                alt_prio!("LEFT", "RIGHT"),
                seq!("BY", E::Source, opt_prio("PLACES")),
                "CIRCULAR"
            )
        ))
    )
}

fn skip() -> Rule {
    not_in(Lang::Cloud, seq!("SKIP", opt("TO LINE"), opt(E::Source)))
}

fn sort() -> Rule {
    seq!(
        "SORT",
        seq!(
            E::Target,
            opt(per!(
                alt_prio!("ASCENDING", "DESCENDING"),
                alt!(
                    seq!(
                        "BY",
                        plus(seq!(
                            alt!(E::ComponentChain, E::Dynamic, E::SourceFieldSymbol),
                            opt_prio("AS TEXT"),
                            opt_prio(alt_prio!("ASCENDING", "DESCENDING")),
                            opt_prio("AS TEXT")
                        ))
                    ),
                    // SAP `abapsort_itab`: BY expr, a sort table
                    seq!("BY", E::Source)
                ),
                "STABLE",
                "AS TEXT"
            ))
        )
    )
}

fn sort_dataset() -> Rule {
    not_in(
        Lang::Cloud,
        seq!(
            "SORT",
            opt("AS TEXT"),
            opt(seq!(
                "BY",
                plus(seq!(
                    alt!(E::ComponentChain, E::SourceFieldSymbol, E::Dynamic),
                    opt_prio(alt!("ASCENDING", "DESCENDING"))
                ))
            ))
        ),
    )
}

fn split() -> Rule {
    seq!(
        "SPLIT",
        E::Source,
        "AT",
        E::Source,
        "INTO",
        alt_prio!(
            seq!(
                "TABLE",
                E::Target,
                opt(seq!("IN", alt!("CHARACTER", "BYTE"), "MODE"))
            ),
            seq!(
                plus(E::Target),
                opt(seq!("IN", alt!("CHARACTER", "BYTE"), "MODE"))
            )
        )
    )
}

fn start_of_selection() -> Rule {
    not_in(Lang::Cloud, "START-OF-SELECTION")
}

fn static_() -> Rule {
    alt_prio!(
        seq!(
            "STATIC",
            seq!(
                E::DefinitionName,
                opt_prio(E::ConstantFieldLength),
                alt!(
                    seq!(
                        opt(E::ConstantFieldLength),
                        opt(per!(E::Type, E::Value, E::Length, E::Decimals))
                    ),
                    E::TypeTable
                )
            )
        ),
        not_in(
            Lang::KeyUser,
            seq!(
                "STATICS",
                seq!(
                    E::DefinitionName,
                    opt_prio(E::ConstantFieldLength),
                    alt!(
                        seq!(
                            opt(E::ConstantFieldLength),
                            opt(per!(E::Type, E::Value, E::Length, E::Decimals))
                        ),
                        E::TypeTable
                    )
                )
            )
        )
    )
}

fn static_begin() -> Rule {
    seq!(
        alt!("STATIC", "STATICS"),
        "BEGIN OF",
        E::DefinitionName,
        opt(seq!("OCCURS", E::Integer))
    )
}

fn static_end() -> Rule {
    seq!(alt!("STATIC", "STATICS"), "END OF", E::DefinitionName)
}

fn stop() -> Rule {
    not_in(Lang::Cloud, "STOP")
}

fn submit() -> Rule {
    not_in(
        Lang::Cloud,
        seq!(
            "SUBMIT",
            alt_prio!(E::IncludeName, E::Dynamic),
            opt(per!(
                plus_prio(seq!(
                    "WITH",
                    E::FieldSub,
                    alt!(
                        seq!(
                            alt!("=", "EQ", "IN", "NE", "CP", "GE", "LE", "INCL", "GT", "LT"),
                            E::Source
                        ),
                        seq!("BETWEEN", E::Source, "AND", E::Source)
                    ),
                    opt_prio(seq!("SIGN", E::Source))
                )),
                seq!("WITH SELECTION-TABLE", E::Source),
                plus_prio(seq!(
                    "WITH",
                    E::FieldSub,
                    alt!(
                        seq!(
                            alt!("=", "EQ", "IN", "NE", "CP", "GE", "LE", "INCL", "GT", "LT"),
                            E::Source
                        ),
                        seq!("BETWEEN", E::Source, "AND", E::Source)
                    ),
                    opt_prio(seq!("SIGN", E::Source))
                )),
                plus_prio(seq!(
                    "WITH",
                    E::FieldSub,
                    alt!(
                        seq!(
                            alt!("=", "EQ", "IN", "NE", "CP", "GE", "LE", "INCL", "GT", "LT"),
                            E::Source
                        ),
                        seq!("BETWEEN", E::Source, "AND", E::Source)
                    ),
                    opt_prio(seq!("SIGN", E::Source))
                )),
                seq!("SPOOL PARAMETERS", E::Source),
                seq!("LINE-SIZE", E::Source),
                seq!("LINE-COUNT", E::Source),
                seq!("ARCHIVE PARAMETERS", E::Source),
                seq!("ARCHIVE MODE", E::Source),
                seq!("RECEIVER", E::Source),
                seq!("USER", E::Source),
                seq!("USING SELECTION-SET", E::Source),
                seq!("USING SELECTION-SETS OF PROGRAM", E::Source),
                seq!("KEEP IN SPOOL", E::Source),
                seq!("SAP COVER PAGE", E::Source),
                seq!("IMMEDIATELY", E::Source),
                seq!("LAYOUT", E::Source),
                seq!("DESTINATION", E::Source),
                seq!("COVER TEXT", E::Source),
                seq!("LIST NAME", E::Source),
                seq!("LIST DATASET", E::Source),
                seq!("LIST AUTHORITY", E::Source),
                seq!("COVER PAGE", E::Source),
                seq!("DEPARTMENT", E::Source),
                seq!("LANGUAGE", E::Source),
                seq!("WITH FREE SELECTIONS", E::Source),
                seq!("NEW LIST IDENTIFICATION", E::Source),
                seq!("USING SELECTION-SCREEN", E::Source),
                seq!("COPIES", E::Source),
                seq!("DATASET EXPIRATION", E::Source),
                "TO SAP-SPOOL",
                "WITHOUT SPOOL DYNPRO",
                "VIA SELECTION-SCREEN",
                "EXPORTING LIST TO MEMORY",
                E::AndReturn,
                seq!("VIA JOB", E::Source, "NUMBER", E::Source)
            ))
        ),
    )
}

fn subtract() -> Rule {
    not_in(
        Lang::KeyUser,
        seq!("SUBTRACT", E::Source, "FROM", E::Target),
    )
}

fn subtract_corresponding() -> Rule {
    not_in(
        Lang::Cloud,
        seq!("SUBTRACT-CORRESPONDING", E::Source, "FROM", E::Target),
    )
}

fn sum() -> Rule {
    not_in(Lang::Cloud, "SUM")
}

fn summary() -> Rule {
    not_in(Lang::Cloud, "SUMMARY")
}

fn supply() -> Rule {
    not_in(
        Lang::Cloud,
        seq!(
            "SUPPLY",
            plus(seq!(E::Field, "=", E::Source)),
            "TO CONTEXT",
            E::Field
        ),
    )
}

fn suppress_dialog() -> Rule {
    not_in(Lang::Cloud, "SUPPRESS DIALOG")
}

fn syntax_check() -> Rule {
    not_in(
        Lang::Cloud,
        seq!(
            "SYNTAX-CHECK FOR",
            alt!(
                seq!(
                    opt_prio("PROGRAM"),
                    E::Source,
                    per!(
                        seq!("MESSAGE", E::Target),
                        seq!("LINE", E::Target),
                        seq!("WORD", E::Target),
                        seq!("OFFSET", E::Target),
                        seq!("PROGRAM", E::Source),
                        seq!("REPLACING", E::Target),
                        seq!("DIRECTORY ENTRY", E::Source),
                        seq!("FRAME ENTRY", E::Target),
                        seq!("INCLUDE", E::Target),
                        seq!("MESSAGE-ID", E::Target),
                        seq!("TRACE-TABLE", E::Target),
                        seq!("SHORTDUMP-ID", E::Source),
                        seq!("FILTER", E::Source),
                        plus(seq!("ID", E::Source, "TABLE", E::Target))
                    )
                ),
                seq!(
                    "DYNPRO",
                    E::Source,
                    E::Source,
                    E::Source,
                    E::Source,
                    per!(
                        seq!("MESSAGE", E::Target),
                        seq!("LINE", E::Target),
                        seq!("WORD", E::Target),
                        seq!("OFFSET", E::Target),
                        seq!("MESSAGE-ID", E::Target),
                        seq!("TRACE-TABLE", E::Target)
                    )
                )
            )
        ),
    )
}

fn system_call() -> Rule {
    not_in(
        Lang::Cloud,
        seq!(
            "SYSTEM-CALL",
            alt_prio!(
                seq!("OBJMGR CLONE", E::Source, "TO", E::Target),
                seq!(re(r"^.+$"), "DID", E::Source, "PARAMETERS", plus(E::Source)),
                plus(re(r"^.+$"))
            )
        ),
    )
}

fn tables() -> Rule {
    not_in(Lang::Cloud, seq!("TABLES", E::Field))
}

fn test_injection() -> Rule {
    not_in(Lang::KeyUser, seq!("TEST-INJECTION", E::TestSeamName))
}

fn test_seam() -> Rule {
    not_in(
        Lang::KeyUser,
        ver("7.50", seq!("TEST-SEAM", E::TestSeamName)),
    )
}

fn top_of_page() -> Rule {
    not_in(
        Lang::Cloud,
        seq!("TOP-OF-PAGE", opt("DURING LINE-SELECTION")),
    )
}

fn transfer() -> Rule {
    not_in(
        Lang::Cloud,
        seq!(
            "TRANSFER",
            E::Source,
            "TO",
            E::Source,
            opt(seq!("LENGTH", E::Source)),
            opt("NO END OF LINE")
        ),
    )
}

fn translate() -> Rule {
    seq!(
        "TRANSLATE",
        E::Target,
        alt_prio!(
            seq!("TO", alt_prio!("UPPER", "LOWER"), "CASE"),
            seq!("USING", E::Source)
        )
    )
}

fn truncate_dataset() -> Rule {
    not_in(
        Lang::Cloud,
        seq!("TRUNCATE DATASET", E::Source, "AT CURRENT POSITION"),
    )
}

fn try_() -> Rule {
    Rule::from("TRY")
}

fn type_() -> Rule {
    seq!(
        "TYPES",
        seq!(
            E::NamespaceSimpleName,
            opt(E::ConstantFieldLength),
            opt(alt_prio!(
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
                ver(
                    "7.50",
                    seq!(
                        "TYPE",
                        E::TypeName,
                        "CLIENT",
                        "SPECIFIED",
                        E::FieldChain,
                        opt(plus(seq!(
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
                        )))
                    )
                ),
                ver(
                    "8.16",
                    seq!(
                        "TYPE",
                        "ANY",
                        "STRUCTURE",
                        opt(seq!(
                            "CONTAINING",
                            plus(seq!(E::DefinitionName, "TYPE", E::TypeSimpleRef))
                        ))
                    )
                ),
                alt!(
                    per!(E::Type, E::Decimals, E::Length),
                    E::TypeTable,
                    E::TypeStructure
                )
            ))
        ),
        opt(ver("7.02", "BOXED"))
    )
}

fn type_begin() -> Rule {
    seq!(
        "TYPES",
        "BEGIN OF",
        E::NamespaceSimpleName,
        opt_prio(not_in(Lang::Cloud, "%_FINAL"))
    )
}

fn type_end() -> Rule {
    seq!("TYPES", "END OF", E::NamespaceSimpleName)
}

fn type_enum() -> Rule {
    ver("7.51", seq!("TYPES", E::NamespaceSimpleName, E::Value))
}

fn type_enum_begin() -> Rule {
    ver(
        "7.51",
        seq!(
            "TYPES",
            "BEGIN OF",
            seq!(
                "ENUM",
                E::NamespaceSimpleName,
                opt(seq!("STRUCTURE", E::NamespaceSimpleName)),
                opt(seq!("BASE TYPE", E::NamespaceSimpleName))
            )
        ),
    )
}

fn type_enum_end() -> Rule {
    ver(
        "7.51",
        seq!(
            "TYPES",
            "END OF",
            "ENUM",
            E::NamespaceSimpleName,
            opt(seq!("STRUCTURE", E::NamespaceSimpleName))
        ),
    )
}

fn type_mesh() -> Rule {
    ver(
        "7.40 SP05",
        seq!(
            "TYPES",
            E::NamespaceSimpleName,
            "TYPE",
            opt("REF TO"),
            E::TypeName,
            plus(seq!(
                "ASSOCIATION",
                E::NamespaceSimpleName,
                "TO",
                E::NamespaceSimpleName,
                plus(seq!(
                    "ON",
                    E::NamespaceSimpleName,
                    "=",
                    E::NamespaceSimpleName,
                    star(seq!(
                        "AND",
                        E::NamespaceSimpleName,
                        "=",
                        E::NamespaceSimpleName
                    ))
                ))
            )),
            opt(seq!("USING KEY", E::NamespaceSimpleName))
        ),
    )
}

fn type_mesh_begin() -> Rule {
    ver(
        "7.40 SP05",
        seq!("TYPES", "BEGIN OF MESH", E::NamespaceSimpleName),
    )
}

fn type_mesh_end() -> Rule {
    ver(
        "7.40 SP05",
        seq!("TYPES", "END OF MESH", E::NamespaceSimpleName),
    )
}

fn type_pool() -> Rule {
    not_in(Lang::Cloud, seq!("TYPE-POOL", re(r"^\w+$")))
}

fn type_pools() -> Rule {
    not_in(Lang::Cloud, seq!("TYPE-POOLS", re(r"^\w+$")))
}

fn uline() -> Rule {
    not_in(
        Lang::Cloud,
        seq!("ULINE", opt_prio(E::WriteOffsetLength), opt_prio("NO-GAP")),
    )
}

fn unassign() -> Rule {
    seq!("UNASSIGN", E::TargetFieldSymbol)
}

fn unpack() -> Rule {
    not_in(Lang::Cloud, seq!("UNPACK", E::Source, "TO", E::Target))
}

fn update_database() -> Rule {
    not_in(
        Lang::KeyUser,
        seq!(
            "UPDATE",
            E::DatabaseTable,
            opt(E::SQLClient),
            opt(E::DatabaseConnection),
            opt(alt!(
                seq!(
                    "FROM",
                    opt("TABLE"),
                    E::SQLSource,
                    opt_prio(alt_prio!(
                        ver("8.16", E::SQLIndicators),
                        ver("7.55", seq!("INDICATORS SET STRUCTURE", E::ComponentName))
                    )),
                    opt_prio(E::SQLMappingFromEntity)
                ),
                seq!(
                    "SET",
                    alt!(
                        seq!(
                            E::SQLFieldAndValue,
                            star(seq!(opt(","), E::SQLFieldAndValue))
                        ),
                        E::Dynamic
                    ),
                    opt(seq!("WHERE", E::SQLCond))
                )
            )),
            opt_prio(E::SQLDmlOptions)
        ),
    )
}

fn verification_message() -> Rule {
    not_in(
        Lang::Cloud,
        seq!(
            "VERIFICATION-MESSAGE",
            E::Source,
            E::Source,
            opt(seq!("PRIORITY", E::Source))
        ),
    )
}

fn wait() -> Rule {
    not_in(
        Lang::KeyUser,
        seq!(
            "WAIT",
            alt!(
                seq!(
                    opt(seq!(
                        "FOR",
                        per!(
                            "MESSAGING CHANNELS",
                            ver("7.50", "PUSH CHANNELS"),
                            "ASYNCHRONOUS TASKS"
                        )
                    )),
                    "UNTIL",
                    E::Cond,
                    opt(seq!("UP TO", E::Source, "SECONDS"))
                ),
                seq!("UP TO", E::Source, "SECONDS")
            )
        ),
    )
}

fn when() -> Rule {
    seq!("WHEN", seq!(E::Source, star_prio(E::Or)))
}

fn when_others() -> Rule {
    Rule::from("WHEN OTHERS")
}

fn when_type() -> Rule {
    ver(
        "7.50",
        seq!(
            "WHEN TYPE",
            seq!(E::ClassName, opt_prio(seq!("INTO", E::Target)))
        ),
    )
}

fn while_() -> Rule {
    seq!(
        "WHILE",
        E::Cond,
        star(seq!(
            "VARY",
            E::Target,
            "FROM",
            E::Source,
            "NEXT",
            E::Source
        ))
    )
}

fn window() -> Rule {
    not_in(
        Lang::Cloud,
        seq!(
            "WINDOW STARTING AT",
            E::Source,
            E::Source,
            opt(seq!("ENDING AT", E::Source, E::Source))
        ),
    )
}

fn with() -> Rule {
    not_in(
        Lang::KeyUser,
        ver(
            "7.51",
            seq!(
                "WITH",
                seq!(
                    E::WithName,
                    "AS",
                    Tok::WParenLeftW,
                    E::SelectCTE,
                    Tok::WParenRightW,
                    opt_prio(seq!(
                        "WITH",
                        alt_prio!(E::SQLCTEHierarchy, E::SQLCTEAssociations)
                    ))
                ),
                star(seq!(
                    ",",
                    seq!(
                        E::WithName,
                        "AS",
                        Tok::WParenLeftW,
                        E::SelectCTE,
                        Tok::WParenRightW,
                        opt_prio(seq!(
                            "WITH",
                            alt_prio!(E::SQLCTEHierarchy, E::SQLCTEAssociations)
                        ))
                    )
                )),
                E::Select
            ),
        ),
    )
}

fn with_loop() -> Rule {
    not_in(
        Lang::KeyUser,
        ver(
            "7.51",
            seq!(
                "WITH",
                seq!(
                    E::WithName,
                    "AS",
                    Tok::WParenLeftW,
                    E::SelectCTE,
                    Tok::WParenRightW,
                    opt_prio(seq!(
                        "WITH",
                        alt_prio!(E::SQLCTEHierarchy, E::SQLCTEAssociations)
                    ))
                ),
                star(seq!(
                    ",",
                    seq!(
                        E::WithName,
                        "AS",
                        Tok::WParenLeftW,
                        E::SelectCTE,
                        Tok::WParenRightW,
                        opt_prio(seq!(
                            "WITH",
                            alt_prio!(E::SQLCTEHierarchy, E::SQLCTEAssociations)
                        ))
                    )
                )),
                E::Select
            ),
        ),
    )
}

fn write() -> Rule {
    not_in(
        Lang::Cloud,
        seq!(
            "WRITE",
            alt!(
                seq!(
                    "AT /",
                    opt(E::Source),
                    opt(per!(
                        seq!(
                            "USING",
                            alt_prio!("NO EDIT MASK", seq!("EDIT MASK", E::Source))
                        ),
                        seq!("TO", E::Target),
                        seq!("EXPONENT", E::Source),
                        "NO-GROUPING",
                        "NO-ZERO",
                        "CENTERED",
                        seq!(
                            "INPUT",
                            opt(alt!(alt_prio!("ON", "OFF"), seq!("=", E::FieldSub)))
                        ),
                        "NO-GAP",
                        "LEFT-JUSTIFIED",
                        seq!("AS", alt_prio!("LINE", "ICON", "CHECKBOX", "SYMBOL")),
                        seq!(
                            "FRAMES",
                            alt!(alt_prio!("ON", "OFF"), seq!("=", E::FieldSub))
                        ),
                        seq!(
                            "HOTSPOT",
                            opt(alt!(alt_prio!("ON", "OFF"), seq!("=", E::FieldSub)))
                        ),
                        "RIGHT-JUSTIFIED",
                        seq!("TIME ZONE", E::Source),
                        seq!("UNDER", E::Source),
                        seq!("STYLE", E::Source),
                        seq!("ROUND", E::Source),
                        seq!("QUICKINFO", E::Source),
                        "ENVIRONMENT TIME FORMAT",
                        alt_prio!(
                            "DD/MM/YY",
                            "MM/DD/YY",
                            "DD/MM/YYYY",
                            "MM/DD/YYYY",
                            "DDMMYY",
                            "MMDDYY",
                            "YYMMDD"
                        ),
                        seq!("UNIT", E::Source),
                        seq!(
                            "INTENSIFIED",
                            opt(alt!(alt_prio!("ON", "OFF"), seq!("=", E::FieldSub)))
                        ),
                        seq!("INDEX", E::Source),
                        seq!("DECIMALS", E::Source),
                        seq!(
                            "INVERSE",
                            opt(alt!(alt_prio!("ON", "OFF"), seq!("=", E::FieldSub)))
                        ),
                        E::Color,
                        seq!("CURRENCY", E::Source),
                        "RESET",
                        "NO-SIGN"
                    ))
                ),
                seq!(
                    alt!(
                        seq!(opt(E::WriteOffsetLength), alt!(E::Source, E::Dynamic)),
                        "/"
                    ),
                    opt(per!(
                        seq!(
                            "USING",
                            alt_prio!("NO EDIT MASK", seq!("EDIT MASK", E::Source))
                        ),
                        seq!("TO", E::Target),
                        seq!("EXPONENT", E::Source),
                        "NO-GROUPING",
                        "NO-ZERO",
                        "CENTERED",
                        seq!(
                            "INPUT",
                            opt(alt!(alt_prio!("ON", "OFF"), seq!("=", E::FieldSub)))
                        ),
                        "NO-GAP",
                        "LEFT-JUSTIFIED",
                        seq!("AS", alt_prio!("LINE", "ICON", "CHECKBOX", "SYMBOL")),
                        seq!(
                            "FRAMES",
                            alt!(alt_prio!("ON", "OFF"), seq!("=", E::FieldSub))
                        ),
                        seq!(
                            "HOTSPOT",
                            opt(alt!(alt_prio!("ON", "OFF"), seq!("=", E::FieldSub)))
                        ),
                        "RIGHT-JUSTIFIED",
                        seq!("TIME ZONE", E::Source),
                        seq!("UNDER", E::Source),
                        seq!("STYLE", E::Source),
                        seq!("ROUND", E::Source),
                        seq!("QUICKINFO", E::Source),
                        "ENVIRONMENT TIME FORMAT",
                        alt_prio!(
                            "DD/MM/YY",
                            "MM/DD/YY",
                            "DD/MM/YYYY",
                            "MM/DD/YYYY",
                            "DDMMYY",
                            "MMDDYY",
                            "YYMMDD"
                        ),
                        seq!("UNIT", E::Source),
                        seq!(
                            "INTENSIFIED",
                            opt(alt!(alt_prio!("ON", "OFF"), seq!("=", E::FieldSub)))
                        ),
                        seq!("INDEX", E::Source),
                        seq!("DECIMALS", E::Source),
                        seq!(
                            "INVERSE",
                            opt(alt!(alt_prio!("ON", "OFF"), seq!("=", E::FieldSub)))
                        ),
                        E::Color,
                        seq!("CURRENCY", E::Source),
                        "RESET",
                        "NO-SIGN"
                    ))
                )
            )
        ),
    )
}
