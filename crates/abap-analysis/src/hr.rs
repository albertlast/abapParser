//! HR infotypes: the database tables of infotypes, `PAnnnn`, `PBnnnn` and
//! `HRPnnnn`, read through the logical databases PNP and PCH,
//! `HR_READ_INFOTYPE`, `RH_READ_INFTY` and the macros of PNP, and what those
//! macros read and write.

use std::collections::HashMap;

use abap_model as m;
use abap_syntax::{File, Statement, StatementKind as K, Token, TokenKind};

use crate::stores::raw;

/// What reads an infotype: it decides the table of customer infotypes,
/// which may be of either kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Reader {
    /// personnel administration: PNP, `HR_READ_INFOTYPE`
    Personnel,
    /// applicants: `HR_READ_INFOTYPE` with `tclas = 'B'`
    Applicant,
    /// personnel planning: PCH, `RH_READ_INFTY`
    Planning,
}

/// The database table of an infotype; `None` unless `number` is four
/// digits. The number range decides: personnel planning (1000 to 1999, and
/// 5000 to 5999) is in `HRPnnnn`, recruitment (4000 to 4999) in `PBnnnn`,
/// personnel administration and time management (0000 to 0999, 2000 to
/// 2999) in `PAnnnn`, or `PBnnnn` for applicants. Others, like customer
/// infotypes 9000 to 9999, can be of either kind: the reader decides.
pub(crate) fn table(number: &str, reader: Reader) -> Option<String> {
    if number.len() != 4 || !number.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let n: u16 = number.parse().ok()?;
    let prefix = match (n, reader) {
        (1000..=1999 | 5000..=5999, _) => "HRP",
        (4000..=4999, _) => "PB",
        (_, Reader::Applicant) => "PB",
        (0..=999 | 2000..=2999, _) | (_, Reader::Personnel) => "PA",
        (_, Reader::Planning) => "HRP",
    };
    Some(format!("{prefix}{number}"))
}

/// An infotype declared with INFOTYPES.
pub(crate) struct Infotype {
    /// the internal table, lower case, like `p0001`
    pub table: String,
    pub number: String,
    /// `MODE N`: the logical database does not fill the table
    pub mode_n: bool,
}

/// The infotypes a program declares.
pub(crate) fn infotypes(symbols: &m::FileSymbols) -> Vec<Infotype> {
    symbols
        .declarations
        .iter()
        .filter(|d| d.kind == m::DeclarationKind::Infotype)
        .filter_map(|d| {
            let typing = d.typing.as_ref()?;
            let number = typing.name.as_deref()?.get(1..)?.to_string();
            let words: Vec<String> = typing
                .text
                .split_whitespace()
                .map(str::to_ascii_uppercase)
                .collect();
            Some(Infotype {
                table: d.name.to_ascii_lowercase(),
                number,
                mode_n: words.windows(2).any(|w| w[0] == "MODE" && w[1] == "N"),
            })
        })
        .collect()
}

fn read(name: String) -> m::TableAccess {
    m::TableAccess {
        name,
        access: m::AccessKind::Read,
        operation: m::DbOperation::Select,
        store: None,
        via: None,
        depends: None,
    }
}

/// The node of `GET node`, lower case, unless it is `GET node LATE`, which
/// reads nothing.
pub(crate) fn get_node(file: &File, s: &Statement) -> Option<String> {
    if s.kind != K::Get {
        return None;
    }
    let words: Vec<&Token> = s.tokens.iter().map(|t| &file.tokens()[*t]).collect();
    if words.iter().any(|t| t.is("LATE")) {
        return None;
    }
    words.get(1).map(|t| t.text.to_ascii_lowercase())
}

/// The infotype tables the logical database fills at `GET node`, with the
/// database table of each: PNP at `GET PERNR` and PNPCE at `GET PERAS`
/// those in `PAnnnn`, PCH at `GET OBJEC` those in `HRPnnnn`; not those
/// declared with `MODE N`.
pub(crate) fn filled<'a>(node: &str, declared: &'a [Infotype]) -> Vec<(&'a Infotype, String)> {
    let (reader, prefix) = match node {
        "pernr" | "peras" => (Reader::Personnel, "PA"),
        "objec" => (Reader::Planning, "HRP"),
        _ => return Vec::new(),
    };
    declared
        .iter()
        .filter(|i| !i.mode_n)
        .filter_map(|i| table(&i.number, reader).map(|t| (i, t)))
        .filter(|(_, t)| t.starts_with(prefix))
        .collect()
}

/// An infotype number as written: digits, a literal or a constant.
fn number(word: &str, constants: &HashMap<String, String>) -> String {
    if word.bytes().all(|b| b.is_ascii_digit()) {
        word.to_string()
    } else {
        raw(word, constants)
    }
}

/// Function modules reading infotypes: name, reader, and the infotype when
/// the name fixes it; else it is the parameter `infty`.
const READERS: &[(&str, Reader, Option<&str>)] = &[
    ("HR_READ_INFOTYPE", Reader::Personnel, None),
    ("RH_READ_INFTY", Reader::Planning, None),
    ("RH_READ_INFTY_1001", Reader::Planning, Some("1001")),
];

/// The infotype tables a statement reads from the database: `GET PERNR`
/// and `GET OBJEC` those of the declared infotypes, `HR_READ_INFOTYPE`,
/// `RH_READ_INFTY` and `RP-READ-INFOTYPE` the one they name.
pub(crate) fn infotype_reads(
    file: &File,
    s: &Statement,
    constants: &HashMap<String, String>,
    declared: &[Infotype],
) -> Vec<m::TableAccess> {
    let words: Vec<&str> = s
        .tokens
        .iter()
        .map(|t| file.tokens()[*t].text.as_str())
        .collect();
    match s.kind {
        K::Get => get_node(file, s)
            .map(|n| filled(&n, declared))
            .unwrap_or_default()
            .into_iter()
            .map(|(_, t)| read(t))
            .collect(),
        // CALL FUNCTION 'HR_READ_INFOTYPE' EXPORTING ... infty = '0001'
        K::CallFunction => {
            let Some(&(_, reader, fixed)) = words.get(2).and_then(|w| {
                let name = raw(w, constants);
                READERS.iter().find(|r| r.0.eq_ignore_ascii_case(&name))
            }) else {
                return Vec::new();
            };
            let value = |parameter: &str| {
                words
                    .windows(3)
                    .find(|w| w[0].eq_ignore_ascii_case(parameter) && w[1] == "=")
                    .map(|w| raw(w[2], constants))
            };
            let applicant = reader == Reader::Personnel
                && value("tclas").is_some_and(|c| c.eq_ignore_ascii_case("B"));
            let reader = if applicant { Reader::Applicant } else { reader };
            fixed
                .map(str::to_string)
                .or_else(|| value("infty"))
                .and_then(|n| table(&n, reader))
                .map(read)
                .into_iter()
                .collect()
        }
        // RP-READ-INFOTYPE pernr infty table begda endda
        K::MacroCall => match abap_syntax::sap_macro(file, s) {
            Some(("RP-READ-INFOTYPE", used)) => arguments(file, s, used)
                .get(1)
                .and_then(|a| table(&number(&join(a), constants), Reader::Personnel))
                .map(read)
                .into_iter()
                .collect(),
            _ => Vec::new(),
        },
        _ => Vec::new(),
    }
}

/// What a macro of PNP reads and writes, by the position of its
/// arguments, counted from 0; the macros as SAP documents their use:
///
/// - `RP-PROVIDE-FROM-LAST table subtype begda endda` and
///   `RP-PROVIDE-FROM-FRST`: the header line of the infotype table gets the
///   last (first) record of the period, `pnp-sw-found` whether there is one
/// - `RP-READ-INFOTYPE pernr infty table begda endda`: the table gets the
///   records of the infotype of the period
/// - `RP-SET-DATA-INTERVAL infty begda endda`: the period the logical
///   database reads
pub(crate) struct MacroAccess {
    pub reads: Vec<usize>,
    /// (argument, whole)
    pub writes: Vec<(usize, bool)>,
    /// data the macro writes that is not one of its arguments
    pub sets: &'static [&'static str],
}

pub(crate) fn macro_access(name: &str) -> Option<MacroAccess> {
    Some(match name {
        "RP-PROVIDE-FROM-LAST" | "RP-PROVIDE-FROM-FRST" => MacroAccess {
            reads: vec![0, 1, 2, 3],
            writes: vec![(0, false)],
            sets: &["pnp-sw-found"],
        },
        "RP-READ-INFOTYPE" => MacroAccess {
            reads: vec![0, 3, 4],
            writes: vec![(2, true)],
            sets: &[],
        },
        "RP-SET-DATA-INTERVAL" => MacroAccess {
            reads: vec![1, 2],
            writes: Vec::new(),
            sets: &[],
        },
        _ => return None,
    })
}

/// The arguments of a macro call after the `used` tokens of its name:
/// tokens joined without blanks, like `pn-begda`.
pub(crate) fn arguments<'a>(file: &'a File, s: &Statement, used: usize) -> Vec<Vec<&'a Token>> {
    let mut out: Vec<Vec<&Token>> = Vec::new();
    for t in s.tokens.iter().skip(used).map(|t| &file.tokens()[*t]) {
        if t.kind == TokenKind::Punctuation {
            continue;
        }
        match out.last_mut() {
            Some(arg) if !t.ws_before => arg.push(t),
            _ => out.push(vec![t]),
        }
    }
    out
}

pub(crate) fn join(tokens: &[&Token]) -> String {
    tokens.iter().map(|t| t.text.as_str()).collect()
}

/// Whether a macro argument is data, not a literal, a number or `space`.
pub(crate) fn is_data(arg: &[&Token]) -> bool {
    let first = arg[0];
    first.kind == TokenKind::Identifier
        && !first.text.bytes().all(|b| b.is_ascii_digit())
        && !(arg.len() == 1 && first.is("SPACE"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reads(source: &str) -> Vec<String> {
        let file = abap_syntax::parse(source);
        let symbols = crate::symbols_of(&file);
        let constants = crate::db::literal_constants(&symbols);
        let declared = infotypes(&symbols);
        file.statements
            .iter()
            .flat_map(|s| infotype_reads(&file, s, &constants, &declared))
            .map(|t| t.name)
            .collect()
    }

    #[test]
    fn tables_by_number_range() {
        let t = |n, r| table(n, r).unwrap();
        assert_eq!(t("0001", Reader::Personnel), "PA0001");
        assert_eq!(t("2001", Reader::Personnel), "PA2001");
        assert_eq!(t("0002", Reader::Applicant), "PB0002");
        assert_eq!(t("4000", Reader::Personnel), "PB4000");
        // personnel planning, whoever reads it
        assert_eq!(t("1001", Reader::Personnel), "HRP1001");
        assert_eq!(t("5007", Reader::Personnel), "HRP5007");
        // customer infotypes: the reader decides
        assert_eq!(t("9001", Reader::Personnel), "PA9001");
        assert_eq!(t("9001", Reader::Planning), "HRP9001");
        assert_eq!(table("001", Reader::Personnel), None);
    }

    #[test]
    fn infotype_tables() {
        let got = reads(
            "REPORT z.
NODES pernr.
INFOTYPES: 0001, 0002 MODE N, 1001, 0008 NAME lt_basic.
CONSTANTS c_addr TYPE infty VALUE '0006'.
GET pernr.
  CALL FUNCTION 'HR_READ_INFOTYPE'
    EXPORTING pernr = pernr-pernr infty = '0185'
    TABLES infty_tab = lt_0185.
  CALL FUNCTION 'HR_READ_INFOTYPE'
    EXPORTING tclas = 'B' pernr = lv_appl infty = c_addr
    TABLES infty_tab = lt_0006.
  rp-read-infotype pernr-pernr 0007 p0007 pn-begda pn-endda.
  CALL FUNCTION 'HR_READ_INFOTYPE' EXPORTING pernr = pernr-pernr infty = '5007'
    TABLES infty_tab = lt_5007.
GET pernr LATE.",
        );
        assert_eq!(
            got,
            ["PA0001", "PA0008", "PA0185", "PB0006", "PA0007", "HRP5007"]
        );
        let got = reads(
            "REPORT z.
NODES objec.
INFOTYPES: 1000, 1001, 9010, 0001.
GET objec.
  CALL FUNCTION 'RH_READ_INFTY'
    EXPORTING plvar = '01' otype = 'S' objid = objec-objid infty = '9020'
    TABLES innnn = lt_9020.
  CALL FUNCTION 'RH_READ_INFTY_1001' EXPORTING plvar = '01'
    TABLES i1001 = lt_1001.",
        );
        // PCH reads only planning infotypes; customer ones are of planning there
        assert_eq!(got, ["HRP1000", "HRP1001", "HRP9010", "HRP9020", "HRP1001"]);
    }
}
