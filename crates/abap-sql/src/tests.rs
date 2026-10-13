use super::*;

fn db(src: &str) -> m::DbAccess {
    let file = abap_syntax::parse(src);
    let s = file
        .statements
        .iter()
        .find(|s| is_db_statement(s.kind))
        .unwrap_or_else(|| panic!("no db statement in {src}"));
    parse_statement(&file, s).unwrap()
}

fn tables(a: &m::DbAccess) -> Vec<(&str, m::DbTableRole)> {
    a.tables.iter().map(|t| (t.name.as_str(), t.role)).collect()
}

fn col(c: &m::DbColumn) -> String {
    format!("{}.{}", c.table.as_deref().unwrap_or("?"), c.name)
}

#[test]
fn classic_select_single() {
    let a = db("SELECT SINGLE matnr mtart FROM mara INTO (lv_matnr, lv_mtart) WHERE matnr = lv_input AND mtart IN s_mtart.");
    assert_eq!(a.operation, m::DbOperation::Select);
    assert!(a.single && !a.is_loop);
    assert_eq!(tables(&a), [("mara", m::DbTableRole::Main)]);
    let fields: Vec<_> = a.fields.iter().map(|f| f.text.as_str()).collect();
    assert_eq!(fields, ["matnr", "mtart"]);
    assert_eq!(col(&a.fields[0].columns[0]), "mara.matnr");
    let t = a.target.unwrap();
    assert_eq!(t.kind, m::DbTargetKind::List);
    assert_eq!(t.targets, ["lv_matnr", "lv_mtart"]);
    let conds: Vec<_> = a
        .conditions
        .iter()
        .map(|c| {
            (
                c.left.as_str(),
                c.operator.as_str(),
                c.right.as_deref().unwrap(),
            )
        })
        .collect();
    assert_eq!(
        conds,
        [("matnr", "=", "lv_input"), ("mtart", "IN", "s_mtart")]
    );
    // the right side of classic syntax is a host variable, not a column
    assert_eq!(a.conditions[0].columns.len(), 1);
}

#[test]
fn strict_select_with_join() {
    let a = db("SELECT a~vbeln, a~erdat, b~posnr, COUNT(*) AS cnt \
                FROM vbak AS a INNER JOIN vbap AS b ON a~vbeln = b~vbeln \
                WHERE a~erdat >= @lv_date AND b~matnr = @lv_matnr \
                GROUP BY a~vbeln, a~erdat, b~posnr ORDER BY a~vbeln \
                INTO TABLE @DATA(lt_result) UP TO 100 ROWS.");
    assert_eq!(
        tables(&a),
        [
            ("vbak", m::DbTableRole::Main),
            ("vbap", m::DbTableRole::Join)
        ]
    );
    assert_eq!(a.joins.len(), 1);
    assert_eq!(a.joins[0].kind, "inner");
    let on: Vec<_> = a.joins[0].on[0].columns.iter().map(col).collect();
    assert_eq!(on, ["vbak.vbeln", "vbap.vbeln"]);
    assert_eq!(a.fields.len(), 4);
    assert_eq!(a.fields[3].aggregate.as_deref(), Some("count"));
    assert_eq!(a.fields[3].alias.as_deref(), Some("cnt"));
    assert_eq!(col(&a.fields[2].columns[0]), "vbap.posnr");
    assert_eq!(a.conditions.len(), 2);
    assert_eq!(
        a.conditions[1].columns.iter().map(col).collect::<Vec<_>>(),
        ["vbap.matnr"]
    );
    assert_eq!(a.group_by.len(), 3);
    assert_eq!(a.order_by, ["a~vbeln"]);
    let t = a.target.unwrap();
    assert_eq!(
        (t.kind, t.targets[0].as_str()),
        (m::DbTargetKind::Table, "@DATA(lt_result)")
    );
    assert_eq!(a.up_to.as_deref(), Some("100"));
    assert!(!a.is_loop);
}

#[test]
fn fields_clause_and_left_outer_join() {
    let a = db(
        "SELECT FROM scarr AS c LEFT OUTER JOIN spfli AS p ON p~carrid = c~carrid \
                FIELDS c~carrid, p~connid WHERE c~carrid = 'LH' INTO TABLE @FINAL(rows).",
    );
    assert_eq!(a.joins[0].kind, "left_outer");
    assert_eq!(a.fields.len(), 2);
    assert_eq!(col(&a.fields[1].columns[0]), "spfli.connid");
}

#[test]
fn select_loop_with_for_all_entries() {
    let a = db("SELECT * FROM ekpo INTO CORRESPONDING FIELDS OF ls_ekpo FOR ALL ENTRIES IN lt_ekko WHERE ebeln = lt_ekko-ebeln.");
    assert!(a.is_loop);
    assert_eq!(a.for_all_entries.as_deref(), Some("lt_ekko"));
    let t = a.target.unwrap();
    assert!(t.corresponding);
    assert_eq!(t.kind, m::DbTargetKind::Structure);
    assert_eq!(a.fields[0].text, "*");
    assert!(a.fields[0].columns.is_empty());
}

#[test]
fn subquery_and_exists() {
    let a = db("SELECT carrid FROM scarr INTO TABLE lt WHERE carrid IN ( SELECT carrid FROM spfli WHERE cityfrom = 'BERLIN' ) \
                AND EXISTS ( SELECT * FROM sflight WHERE carrid = scarr~carrid ).");
    assert_eq!(
        tables(&a),
        [
            ("scarr", m::DbTableRole::Main),
            ("spfli", m::DbTableRole::Subquery),
            ("sflight", m::DbTableRole::Subquery)
        ]
    );
    let ops: Vec<_> = a.conditions.iter().map(|c| c.operator.as_str()).collect();
    assert_eq!(ops, ["IN", "=", "EXISTS", "="]);
}

#[test]
fn union_and_cte() {
    let a = db("SELECT carrid FROM scarr WHERE carrid = 'LH' UNION SELECT carrid FROM spfli INTO TABLE @DATA(lt).");
    assert!(a.set_operation);
    assert_eq!(tables(&a).len(), 2);
    assert!(a.target.is_some());

    let w = db(
        "WITH +conn AS ( SELECT carrid, connid FROM spfli WHERE cityfrom = 'FRANKFURT' ) \
                SELECT c~carrid FROM +conn AS c INTO TABLE @DATA(lt).",
    );
    assert_eq!(
        tables(&w),
        [
            ("+conn", m::DbTableRole::Cte),
            ("spfli", m::DbTableRole::Subquery)
        ]
    );
}

#[test]
fn dynamic_parts() {
    let a = db("SELECT (lv_fields) FROM (lv_table) INTO TABLE <lt> WHERE (lv_where).");
    assert!(a.dynamic);
    assert!(a.tables[0].dynamic);
    assert_eq!(a.tables[0].name, "lv_table");
}

#[test]
fn aggregates_and_count() {
    let a = db("SELECT COUNT(*) FROM zlog INTO lv_count WHERE status = 'E'.");
    assert_eq!(a.statement, "Select");
    assert_eq!(a.fields[0].aggregate.as_deref(), Some("count"));
    let b = db("SELECT SINGLE MAX( version ) FROM zfoo INTO lv_version.");
    assert_eq!(b.fields[0].aggregate.as_deref(), Some("max"));
    assert_eq!(col(&b.fields[0].columns[0]), "zfoo.version");
}

#[test]
fn insert_update_modify_delete() {
    let i = db("INSERT zorders FROM TABLE lt_orders.");
    assert_eq!(i.operation, m::DbOperation::Insert);
    assert_eq!(tables(&i), [("zorders", m::DbTableRole::Main)]);
    let s = i.source.unwrap();
    assert!(s.is_table);
    assert_eq!(s.text.as_deref(), Some("lt_orders"));

    let v = db("INSERT INTO zlog VALUES @ls_log.");
    assert_eq!(v.tables[0].name, "zlog");
    assert_eq!(v.source.unwrap().text.as_deref(), Some("@ls_log"));

    let u = db("UPDATE zorders SET status = 'X' changed_at = sy-datum WHERE id = lv_id.");
    assert_eq!(u.operation, m::DbOperation::Update);
    let set: Vec<_> = u
        .set
        .iter()
        .map(|a| (col(&a.column), a.value.as_str()))
        .collect();
    assert_eq!(
        set,
        [
            ("zorders.status".to_string(), "'X'"),
            ("zorders.changed_at".to_string(), "sy-datum")
        ]
    );
    assert_eq!(u.conditions[0].left, "id");

    let u2 = db("UPDATE zcounter SET value = value + 1, changed = @sy-datum WHERE id = @lv_id.");
    assert_eq!(u2.set[0].value, "value + 1");
    assert_eq!(u2.set.len(), 2);

    let md = db("MODIFY zorders FROM ls_order.");
    assert_eq!(md.operation, m::DbOperation::Modify);
    assert!(!md.source.unwrap().is_table);

    let d = db("DELETE FROM zlog WHERE created < @lv_limit.");
    assert_eq!(d.operation, m::DbOperation::Delete);
    assert_eq!(d.tables[0].name, "zlog");
    assert_eq!(d.conditions.len(), 1);

    let d2 = db("DELETE zlog FROM TABLE lt_old.");
    assert_eq!(d2.tables[0].name, "zlog");
    assert!(d2.source.unwrap().is_table);
}

#[test]
fn insert_from_subquery_and_merge() {
    let i = db("INSERT zalpha FROM ( SELECT * FROM zbeta WHERE a = @lv ).");
    assert_eq!(
        tables(&i),
        [
            ("zalpha", m::DbTableRole::Main),
            ("zbeta", m::DbTableRole::Subquery)
        ]
    );
    assert!(i.source.unwrap().is_subquery);

    let m2 = db(
        "MERGE INTO ztarget AS a USING zsource AS b ON a~kf = b~kf WHEN MATCHED THEN UPDATE ALL.",
    );
    assert_eq!(m2.operation, m::DbOperation::Merge);
    assert_eq!(
        tables(&m2),
        [
            ("ztarget", m::DbTableRole::Main),
            ("zsource", m::DbTableRole::MergeSource)
        ]
    );
}

#[test]
fn open_cursor() {
    let a =
        db("OPEN CURSOR WITH HOLD @lv_cursor FOR SELECT * FROM sflight WHERE carrid = @lv_carrid.");
    assert_eq!(a.statement, "OpenCursor");
    assert_eq!(a.tables[0].name, "sflight");
    assert_eq!(a.conditions.len(), 1);
}

#[test]
fn native_tables() {
    let tables_of = |sql: &str| -> Vec<String> {
        let n = hana::parse(sql, hana::Dialect::SqlScript);
        n.accesses[0]
            .access
            .tables
            .iter()
            .map(|t| t.name.clone())
            .collect()
    };
    assert_eq!(
        tables_of("SELECT a.x FROM ztab a JOIN zother b ON a.id = b.id"),
        ["ztab", "zother"]
    );
    assert_eq!(
        tables_of("SELECT * FROM HIERARCHY ( SOURCE zh_source START WHERE parent_id IS NULL )"),
        ["zh_source"]
    );
}

#[test]
fn abap_sql_hierarchy_source() {
    let a = db("SELECT * FROM HIERARCHY_DESCENDANTS( SOURCE zhier START WHERE id = @lv_root ) AS h INTO TABLE @DATA(lt).");
    assert_eq!(tables(&a), [("zhier", m::DbTableRole::Main)]);
    assert_eq!(a.tables[0].alias.as_deref(), Some("h"));
}

#[test]
fn open_sql_host_variables() {
    let hv = |src: &str| db(src).host_variables;
    assert_eq!(
        hv("SELECT SINGLE matnr FROM mara INTO lv_matnr WHERE matnr = lv_input AND mtart IN s_mtart."),
        ["lv_input", "s_mtart"]
    );
    assert_eq!(
        hv("SELECT a FROM t INTO TABLE @DATA(r) WHERE b = @ls_key-b AND c = @( lv_c + 1 ) AND d = @ls_key-b."),
        ["ls_key-b", "lv_c"]
    );
    assert_eq!(
        hv("SELECT a FROM t FOR ALL ENTRIES IN @lt_keys WHERE b = @lt_keys-b INTO TABLE @lt_out."),
        ["lt_keys", "lt_keys-b"]
    );
    assert_eq!(hv("INSERT zlog FROM TABLE @lt_log."), ["lt_log"]);
    assert_eq!(
        hv("UPDATE zlog SET done = @abap_true WHERE id = @lv_id."),
        ["abap_true", "lv_id"]
    );
    assert!(hv("SELECT * FROM t001 INTO TABLE @DATA(lt).").is_empty());
}
