//! Data a statement keeps outside the program or gets from there: ABAP
//! memory, SAP memory, files, shared buffers and SAP locks. Given as table
//! accesses, so the data flow treats them like tables: what is written
//! there depends on, what is read from there. Data clusters in database
//! tables are database accesses, see [`abap_sql::parse_cluster`].

use std::collections::HashMap;

use abap_model as m;
use abap_syntax::{File, Statement, StatementKind as K};

/// The stores and cluster tables a statement reads or writes.
pub(crate) fn store_accesses(
    file: &File,
    s: &Statement,
    constants: &HashMap<String, String>,
) -> Vec<m::TableAccess> {
    let words: Vec<&str> = s
        .tokens
        .iter()
        .map(|t| file.lexed.tokens[*t].text.as_str())
        .filter(|w| *w != "." && *w != ",")
        .collect();
    let at = |phrase: &[&str]| {
        words
            .windows(phrase.len())
            .position(|w| w.iter().zip(phrase).all(|(a, b)| a.eq_ignore_ascii_case(b)))
            .map(|i| i + phrase.len())
    };
    let key = |i: usize| words.get(i).map(|w| name(w, constants));
    let access =
        |name: String, store: Option<m::Store>, written: bool, delete: bool| m::TableAccess {
            name,
            access: if written {
                m::AccessKind::Write
            } else {
                m::AccessKind::Read
            },
            operation: if delete {
                m::DbOperation::Delete
            } else if written {
                m::DbOperation::Modify
            } else {
                m::DbOperation::Select
            },
            store,
            via: None,
            depends: None,
        };
    // EXPORT ... TO MEMORY ID id / IMPORT ... FROM MEMORY ID id, and to
    // shared buffers and shared memory
    let cluster = |written: bool, delete: bool| -> Vec<m::TableAccess> {
        let mut out = Vec::new();
        if let Some(i) = at(&["MEMORY", "ID"]) {
            if let Some(k) = key(i) {
                out.push(access(
                    format!("memory:{k}"),
                    Some(m::Store::Memory),
                    written,
                    delete,
                ));
            }
        }
        for (phrase, store) in [
            (&["SHARED", "BUFFER"][..], Some(m::Store::SharedBuffer)),
            (&["SHARED", "MEMORY"][..], Some(m::Store::SharedMemory)),
        ] {
            let Some(i) = at(phrase) else { continue };
            let Some(table) = words.get(i) else { continue };
            // dbtab(ar): the table, the area follows in parentheses
            let table = table
                .split('(')
                .next()
                .unwrap_or(table)
                .to_ascii_uppercase();
            let id = at(&["ID"]).and_then(key).unwrap_or_default();
            let name = match store {
                Some(m::Store::SharedBuffer) => format!("shared_buffer:{table}:{id}"),
                _ => format!("shared_memory:{table}:{id}"),
            };
            out.push(access(name, store, written, delete));
        }
        out
    };
    match s.kind {
        K::Export => cluster(true, false),
        K::Import => cluster(false, false),
        // DELETE FROM MEMORY ID id, DELETE FROM DATABASE dbtab(ar) ID id
        K::DeleteMemory | K::DeleteCluster => cluster(true, true),
        // FREE MEMORY [ID id]: without an ID all of it
        K::FreeMemory => {
            let k = at(&["ID"]).and_then(key).unwrap_or_else(|| "*".to_string());
            vec![access(
                format!("memory:{k}"),
                Some(m::Store::Memory),
                true,
                true,
            )]
        }
        K::SetParameter | K::GetParameter => at(&["ID"])
            .and_then(key)
            .map(|k| {
                let written = s.kind == K::SetParameter;
                access(
                    format!("parameter:{k}"),
                    Some(m::Store::Parameter),
                    written,
                    false,
                )
            })
            .into_iter()
            .collect(),
        // TRANSFER dobj TO dset
        K::Transfer => at(&["TO"])
            .and_then(|i| words.get(i))
            .map(|d| access(file_name(d, constants), Some(m::Store::File), true, false))
            .into_iter()
            .collect(),
        // READ DATASET dset INTO dobj, DELETE / TRUNCATE DATASET dset
        K::ReadDataset | K::DeleteDataset | K::TruncateDataset => at(&["DATASET"])
            .and_then(|i| words.get(i))
            .map(|d| {
                let written = s.kind != K::ReadDataset;
                let delete = s.kind == K::DeleteDataset;
                access(
                    file_name(d, constants),
                    Some(m::Store::File),
                    written,
                    delete,
                )
            })
            .into_iter()
            .collect(),
        // CALL FUNCTION 'ENQUEUE_EZ_ORDER' sets a lock, DEQUEUE_ releases it
        K::CallFunction => words
            .get(2)
            .and_then(|w| lock_module(&name(w, constants)))
            .map(|(object, release)| {
                access(
                    format!("lock:{object}"),
                    Some(m::Store::Lock),
                    true,
                    release,
                )
            })
            .into_iter()
            .collect(),
        _ => Vec::new(),
    }
}

/// The lock object of a lock module, and whether the module releases the
/// lock: `ENQUEUE_<lock object>` sets it, `DEQUEUE_<lock object>` releases
/// it, `DEQUEUE_ALL` releases all locks (`*`). Lock objects start with `E`,
/// in a namespace after it, like `/ABC/ELOCK`; so `ENQUEUE_READ`, which
/// reads the lock table, is not a lock module.
pub(crate) fn lock_module(function: &str) -> Option<(String, bool)> {
    let function = function.to_ascii_uppercase();
    if function == "DEQUEUE_ALL" {
        return Some(("*".to_string(), true));
    }
    let (object, release) = match function.strip_prefix("ENQUEUE_") {
        Some(o) => (o, false),
        None => (function.strip_prefix("DEQUEUE_")?, true),
    };
    let local = match object.strip_prefix('/') {
        Some(rest) => rest.split_once('/')?.1,
        None => object,
    };
    local
        .starts_with('E')
        .then(|| (object.to_string(), release))
}

/// A key as written: the text of a literal, the value of a constant of the
/// file, else the operand in parentheses; upper case except file names.
fn name(word: &str, constants: &HashMap<String, String>) -> String {
    raw(word, constants).to_ascii_uppercase()
}

fn file_name(word: &str, constants: &HashMap<String, String>) -> String {
    format!("file:{}", raw(word, constants))
}

pub(crate) fn raw(word: &str, constants: &HashMap<String, String>) -> String {
    let literal = word
        .strip_prefix('\'')
        .and_then(|w| w.strip_suffix('\''))
        .or_else(|| word.strip_prefix('`').and_then(|w| w.strip_suffix('`')));
    match literal {
        Some(l) => l.trim().to_string(),
        None => match constants.get(&word.to_ascii_lowercase()) {
            Some(v) => v.clone(),
            None => format!("({})", word.to_ascii_lowercase()),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn accesses(source: &str) -> Vec<String> {
        let file = abap_syntax::parse(source);
        let symbols = crate::symbols_of(&file);
        let constants = crate::db::literal_constants(&symbols);
        file.statements
            .iter()
            .flat_map(|s| store_accesses(&file, s, &constants))
            .map(|t| format!("{} {:?} {:?}", t.name, t.access, t.operation))
            .collect()
    }

    #[test]
    fn memory_parameters_files_locks() {
        let got = accesses(
            "CONSTANTS c_key TYPE c LENGTH 10 VALUE 'ZORD'.
EXPORT orders = lt_orders TO MEMORY ID 'ZORDERS'.
IMPORT orders = lt_orders FROM MEMORY ID c_key.
EXPORT data = ls_data TO DATABASE indx(zo) ID lv_id.
IMPORT data = ls_data FROM SHARED BUFFER indx(zo) ID 'K1'.
DELETE FROM MEMORY ID lv_key.
FREE MEMORY.
SET PARAMETER ID 'VKO' FIELD lv_vkorg.
GET PARAMETER ID 'VKO' FIELD lv_vkorg.
OPEN DATASET lv_file FOR OUTPUT IN TEXT MODE ENCODING DEFAULT.
TRANSFER lv_line TO lv_file.
READ DATASET '/tmp/in.csv' INTO lv_line.
DELETE DATASET lv_file.
CALL FUNCTION 'ENQUEUE_EZ_ORDER' EXPORTING vbeln = lv_vbeln.
CALL FUNCTION 'DEQUEUE_/ABC/ELOCK'.
CALL FUNCTION 'DEQUEUE_ALL'.
CALL FUNCTION 'ENQUEUE_READ' TABLES enq = lt_enq.",
        );
        assert_eq!(
            got,
            [
                "memory:ZORDERS Write Modify",
                "memory:ZORD Read Select",
                "shared_buffer:INDX:K1 Read Select",
                "memory:(LV_KEY) Write Delete",
                "memory:* Write Delete",
                "parameter:VKO Write Modify",
                "parameter:VKO Read Select",
                "file:(lv_file) Write Modify",
                "file:/tmp/in.csv Read Select",
                "file:(lv_file) Write Delete",
                "lock:EZ_ORDER Write Modify",
                "lock:/ABC/ELOCK Write Delete",
                "lock:* Write Delete",
            ]
        );
    }
}
