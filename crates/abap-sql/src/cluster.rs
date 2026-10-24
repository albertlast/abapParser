//! Data clusters in database tables: `EXPORT ... TO DATABASE dbtab(ar) ID
//! id`, `IMPORT ... FROM DATABASE`, `IMPORT DIRECTORY` and `DELETE FROM
//! DATABASE`, as database accesses of the table.

use abap_model as m;
use abap_syntax::{File, Statement, StatementKind as K, Token, TokenKind};

/// The cluster access of a statement, `None` unless it reads or writes a
/// cluster in a database table. `block`, `text` and `span` are left empty
/// for the caller to fill.
pub fn parse_cluster(file: &File, s: &Statement) -> Option<m::DbAccess> {
    if !matches!(s.kind, K::Export | K::Import | K::DeleteCluster) {
        return None;
    }
    let words = words(file, s);
    let upper = |i: usize| words.get(i).map(|w| w.to_ascii_uppercase());
    let at = words
        .iter()
        .position(|w| w.eq_ignore_ascii_case("DATABASE"))?;
    // dbtab(ar)
    let (table, area) = words.get(at + 1)?.split_once('(')?;
    let area = area.trim_end_matches(')').to_string();
    let after = |keyword: &str| {
        (at + 2..words.len())
            .find(|i| upper(*i).as_deref() == Some(keyword))
            .and_then(|i| words.get(i + 1).cloned())
    };
    let directory = upper(1).as_deref() == Some("DIRECTORY");
    let mut access = crate::empty_access(s.kind);
    access.operation = match s.kind {
        K::Export => m::DbOperation::Modify,
        K::Import => m::DbOperation::Select,
        _ => m::DbOperation::Delete,
    };
    access.tables.push(m::DbTable {
        name: table.to_string(),
        alias: None,
        path: None,
        dynamic: false,
        resolved: None,
        role: m::DbTableRole::Main,
    });
    access.client_specified = after("CLIENT").is_some();
    let mut parameters = Vec::new();
    let mut data = Vec::new();
    if directory {
        // IMPORT DIRECTORY INTO itab FROM DATABASE ...
        data.extend(after_word(&words, "INTO"));
    } else if s.kind != K::DeleteCluster {
        // the end of the parameter list: TO / FROM before DATABASE
        let end = at.saturating_sub(1);
        let mut i = 1;
        while i < end {
            let next = upper(i + 1);
            let named = matches!(next.as_deref(), Some("=" | "FROM" | "TO")) && i + 2 < end;
            parameters.push(words[i].to_ascii_lowercase());
            if named {
                data.push(words[i + 2].clone());
                i += 3;
            } else {
                // the old form: the data object is the parameter
                data.push(words[i].clone());
                i += 1;
            }
        }
    }
    match s.kind {
        K::Import => {
            access.target = (!data.is_empty()).then_some(m::DbTarget {
                targets: data,
                kind: if directory {
                    m::DbTargetKind::Table
                } else {
                    m::DbTargetKind::List
                },
                appending: false,
                corresponding: false,
            });
        }
        K::Export => {
            access.source = (!data.is_empty()).then_some(m::DbSource {
                text: Some(data.join(" ")),
                is_table: false,
                is_subquery: false,
            });
        }
        _ => {}
    }
    access.host_variables = crate::host_variables(file, s);
    access.cluster = Some(m::DbCluster {
        area,
        id: after("ID"),
        parameters,
    });
    Some(access)
}

/// The words of a statement: tokens joined where no blank is between
/// them, like `indx(mo)` or `ls_key-id`; without pragmas and the period.
fn words(file: &File, s: &Statement) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let tokens: Vec<&Token> = s.tokens.iter().map(|t| &file.tokens()[*t]).collect();
    for t in tokens {
        if t.kind == TokenKind::Pragma || (t.kind == TokenKind::Punctuation && t.text == ".") {
            continue;
        }
        match out.last_mut() {
            Some(w) if !t.ws_before => w.push_str(&t.text),
            _ => out.push(t.text.clone()),
        }
    }
    out
}

fn after_word(words: &[String], keyword: &str) -> Option<String> {
    let i = words.iter().position(|w| w.eq_ignore_ascii_case(keyword))?;
    words.get(i + 1).cloned()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cluster(source: &str) -> m::DbAccess {
        let file = abap_syntax::parse(source);
        parse_cluster(&file, &file.statements[0]).unwrap()
    }

    #[test]
    fn clusters_in_database_tables() {
        let e =
            cluster("EXPORT orders = lt_orders head FROM ls_head TO DATABASE indx(mo) ID lv_id.");
        assert_eq!(e.operation, m::DbOperation::Modify);
        assert_eq!(e.tables[0].name, "indx");
        let c = e.cluster.unwrap();
        assert_eq!((c.area.as_str(), c.id.as_deref()), ("mo", Some("lv_id")));
        assert_eq!(c.parameters, ["orders", "head"]);
        assert_eq!(e.source.unwrap().text.as_deref(), Some("lt_orders ls_head"));

        let i = cluster("IMPORT orders = lt_orders FROM DATABASE indx(mo) CLIENT '100' ID 'K1'.");
        assert_eq!(i.operation, m::DbOperation::Select);
        assert!(i.client_specified);
        assert_eq!(i.target.unwrap().targets, ["lt_orders"]);
        assert_eq!(i.cluster.unwrap().id.as_deref(), Some("'K1'"));

        // the old form names the data objects only
        let old = cluster("EXPORT gt_rt gt_crt TO DATABASE pcl2(ru) ID ls_key.");
        assert_eq!(old.cluster.unwrap().parameters, ["gt_rt", "gt_crt"]);

        let d = cluster("DELETE FROM DATABASE indx(mo) ID lv_id.");
        assert_eq!(d.operation, m::DbOperation::Delete);

        let dir = cluster("IMPORT DIRECTORY INTO lt_dir FROM DATABASE indx(mo) ID lv_id.");
        assert_eq!(dir.target.unwrap().targets, ["lt_dir"]);

        let file = abap_syntax::parse("EXPORT orders = lt_orders TO MEMORY ID 'Z'.");
        assert!(parse_cluster(&file, &file.statements[0]).is_none());
    }
}
