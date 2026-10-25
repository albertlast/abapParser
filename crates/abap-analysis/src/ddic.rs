//! Dictionary types: the components of structures and tables, the types of
//! data elements and domains, the lines of table types. Read from the DDL
//! source and XML an SAP system gives through ADT, or from abapGit files.

use abap_model as m;

use crate::calls::ValueType;

/// A dictionary structure or table.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Structure {
    /// components with their type; `None` for built-in types
    pub components: Vec<(String, Option<ValueType>)>,
    /// included structures, upper case
    pub includes: Vec<String>,
    /// the components of a built-in type with that type, like `c(10)`
    pub builtins: Vec<(String, m::DataType)>,
}

impl Structure {
    pub fn component(&self, name: &str) -> Option<&Option<ValueType>> {
        self.components
            .iter()
            .find(|(n, _)| n.eq_ignore_ascii_case(name))
            .map(|(_, t)| t)
    }
}

/// Parses a structure or table: DDL (`define structure x { ... }`) or
/// abapGit XML (`DD03P` entries).
pub fn structure(text: &str) -> Option<Structure> {
    if text.trim_start().starts_with('<') || text.starts_with('\u{feff}') {
        return structure_xml(text);
    }
    let body = &text[text.find('{')? + 1..text.rfind('}')?];
    let mut out = Structure::default();
    for entry in body.split(';') {
        // annotations precede a component on lines of their own
        let entry: Vec<&str> = entry
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty() && !l.starts_with('@') && !l.starts_with("//"))
            .collect();
        let entry = entry.join(" ");
        let words: Vec<&str> = entry.split_whitespace().collect();
        match words.as_slice() {
            [] => {}
            [include, name, ..] if include.eq_ignore_ascii_case("include") => {
                out.includes.push(name.to_ascii_uppercase());
            }
            _ => {
                let Some((name, typing)) = entry.split_once(':') else {
                    continue;
                };
                let name = name.split_whitespace().last().unwrap_or_default();
                let words: Vec<&str> = typing.split_whitespace().collect();
                let ty = match words.as_slice() {
                    [r, t, target, ..]
                        if r.eq_ignore_ascii_case("reference") && t.eq_ignore_ascii_case("to") =>
                    {
                        Some(ValueType::Ref(target.to_string()))
                    }
                    // abap.char(10), abap.dec(15,2), abap.string(0): built in
                    [t, ..] if t.to_ascii_lowercase().starts_with("abap.") => {
                        if let Some(b) = ddl_builtin(&t[5..]) {
                            out.builtins.push((name.to_ascii_lowercase(), b));
                        }
                        None
                    }
                    [t, ..] => Some(ValueType::Data(t.to_string())),
                    [] => None,
                };
                if !name.is_empty() {
                    out.components.push((name.to_ascii_lowercase(), ty));
                }
            }
        }
    }
    Some(out)
}

fn structure_xml(text: &str) -> Option<Structure> {
    if !text.contains("<DD03P") {
        return None;
    }
    let mut out = Structure::default();
    for entry in text.split("<DD03P>").skip(1) {
        let entry = entry.split("</DD03P>").next().unwrap_or_default();
        let field = tag(entry, "FIELDNAME").unwrap_or_default();
        if field.starts_with(".INCLU") {
            if let Some(s) = tag(entry, "PRECFIELD") {
                out.includes.push(s.to_ascii_uppercase());
            }
            continue;
        }
        if field.is_empty() || field.starts_with('.') {
            continue;
        }
        let ty = tag(entry, "ROLLNAME").map(|r| {
            if tag(entry, "COMPTYPE").as_deref() == Some("R") {
                ValueType::Ref(r)
            } else {
                ValueType::Data(r)
            }
        });
        if ty.is_none() {
            let builtin = dictionary_builtin(
                &tag(entry, "DATATYPE").unwrap_or_default(),
                number(tag(entry, "LENG")),
                number(tag(entry, "DECIMALS")),
            );
            if let Some(b) = builtin {
                out.builtins.push((field.to_ascii_lowercase(), b));
            }
        }
        out.components.push((field.to_ascii_lowercase(), ty));
    }
    Some(out)
}

/// What a data element refers to, when it is typed as a reference: from the
/// ADT XML (`typeKind` refToClifType / refToDictionaryType) or abapGit
/// (`REFKIND` R, the target in `DOMNAME`).
pub fn data_element_reference(text: &str) -> Option<String> {
    if let Some(kind) = tag(text, "dtel:typeKind") {
        return kind
            .starts_with("refTo")
            .then(|| tag(text, "dtel:typeName"))
            .flatten();
    }
    (tag(text, "REFKIND").as_deref() == Some("R"))
        .then(|| tag(text, "DOMNAME"))
        .flatten()
}

/// The type of a DDL built-in type after `abap.`, like `char(10)` or
/// `dec(15,2)`.
fn ddl_builtin(text: &str) -> Option<m::DataType> {
    let (name, rest) = text.split_once('(').unwrap_or((text, ""));
    let mut numbers = rest
        .trim_end_matches(|c: char| c == ')' || c.is_whitespace() || c == ';')
        .split(',')
        .map(|n| n.trim().parse::<u32>().ok().filter(|n| *n > 0));
    let (length, decimals) = (numbers.next().flatten(), numbers.next().flatten());
    dictionary_builtin(name.trim(), length, decimals)
}

/// What a data element is.
#[derive(Debug, Clone, PartialEq)]
pub enum Element {
    /// of a built-in type, its own or that of its domain as ADT gives it
    Type(m::DataType),
    /// of the domain, whose type abapGit keeps in the domain
    Domain(String),
    /// a reference, to a class, an interface or data
    Reference(m::DataType),
}

/// The type of a data element: from the ADT XML (`dtel:typeKind`, with the
/// type of the domain in `dtel:dataType`) or abapGit (`DD04V`).
pub fn data_element(text: &str) -> Option<Element> {
    use crate::types;
    if let Some(kind) = tag(text, "dtel:typeKind") {
        let name = tag(text, "dtel:typeName");
        let builtin = || {
            dictionary_builtin(
                &tag(text, "dtel:dataType")?,
                number(tag(text, "dtel:dataTypeLength")),
                number(tag(text, "dtel:dataTypeDecimals")),
            )
        };
        return match kind.as_str() {
            "refToClifType" => Some(Element::Reference(types::reference(&name?))),
            "refToDictionaryType" => Some(Element::Reference(m::DataType {
                line: Some(Box::new(types::named(&name?))),
                ..types::reference("data")
            })),
            "refToPredefinedAbapType" => Some(Element::Reference(m::DataType {
                line: builtin().map(Box::new),
                ..types::reference("data")
            })),
            _ => builtin().map(Element::Type),
        };
    }
    let entry = text.split("<DD04V>").nth(1)?.split("</DD04V>").next()?;
    let target = tag(entry, "DOMNAME");
    if tag(entry, "REFKIND").as_deref() == Some("R") {
        return Some(Element::Reference(match tag(entry, "REFTYPE").as_deref() {
            Some("C" | "I") | None => types::reference(&target?),
            _ => m::DataType {
                line: Some(Box::new(types::named(&target?))),
                ..types::reference("data")
            },
        }));
    }
    if let Some(b) = tag(entry, "DATATYPE").and_then(|t| {
        dictionary_builtin(
            &t,
            number(tag(entry, "LENG")),
            number(tag(entry, "DECIMALS")),
        )
    }) {
        return Some(Element::Type(b));
    }
    target.map(Element::Domain)
}

/// The type of a domain: abapGit `DD01V`.
pub fn domain_type(text: &str) -> Option<m::DataType> {
    let entry = text.split("<DD01V>").nth(1)?.split("</DD01V>").next()?;
    dictionary_builtin(
        &tag(entry, "DATATYPE")?,
        number(tag(entry, "LENG")),
        number(tag(entry, "DECIMALS")),
    )
}

/// The line type of a dictionary table type: from the ADT XML
/// (`ttyp:rowType`) or abapGit (`DD40V`). Dictionary types stay names,
/// built-in ones are the ABAP types SAP maps them to.
pub fn table_type_line(text: &str) -> Option<m::DataType> {
    use crate::types;
    if let Some(row) = text
        .split("<ttyp:rowType>")
        .nth(1)
        .and_then(|r| r.split("</ttyp:rowType>").next())
    {
        let name = tag(row, "ttyp:typeName");
        let builtin = || {
            dictionary_builtin(
                &tag(row, "ttyp:dataType")?,
                number(tag(row, "ttyp:length")),
                number(tag(row, "ttyp:decimals")),
            )
        };
        return match tag(row, "ttyp:typeKind")?.as_str() {
            "predefinedAbapType" => builtin(),
            "dictionaryType" => Some(types::named(&name?)),
            "refToPredefinedAbapType" => Some(m::DataType {
                line: builtin().map(Box::new),
                ..types::reference("data")
            }),
            "refToDictionaryType" => Some(m::DataType {
                line: Some(Box::new(types::named(&name?))),
                ..types::reference("data")
            }),
            "refToClassOrInterfaceType" => Some(types::reference(&name?)),
            "rangeTypeOnPredefinedType" => Some(m::DataType {
                line: builtin().map(Box::new),
                ..types::range()
            }),
            "rangeTypeOnDataelement" => Some(m::DataType {
                line: Some(Box::new(types::named(&name?))),
                ..types::range()
            }),
            _ => None,
        };
    }
    let entry = text.split("<DD40V>").nth(1)?.split("</DD40V>").next()?;
    let row = tag(entry, "ROWTYPE");
    match tag(entry, "ROWKIND").as_deref() {
        // a data element, structure or table type
        Some("E" | "S" | "L") => Some(types::named(&row?)),
        Some("R") => Some(types::reference(&row?)),
        None => dictionary_builtin(
            &tag(entry, "DATATYPE")?,
            number(tag(entry, "LENG")),
            number(tag(entry, "DECIMALS")),
        ),
        _ => None,
    }
}

/// The ABAP type of a built-in dictionary type, as SAP's overview of the
/// dictionary types gives it: `CHAR` of length m is `c` of length m, `DEC`,
/// `CURR` and `QUAN` are `p` of length m DIV 2 + 1, and so on. Also the
/// short names abapGit writes, like `STRG` and `RSTR`.
fn dictionary_builtin(
    name: &str,
    length: Option<u32>,
    decimals: Option<u32>,
) -> Option<m::DataType> {
    use crate::types::builtin;
    let upper = name.to_ascii_uppercase();
    Some(match upper.as_str() {
        "INT1" => builtin("b", None, None),
        "INT2" | "PREC" => builtin("s", None, None),
        "INT4" => builtin("i", None, None),
        "INT8" => builtin("int8", None, None),
        "DEC" | "CURR" | "QUAN" => builtin("p", length.map(|m| m / 2 + 1), decimals),
        "FLTP" => builtin("f", None, None),
        "CHAR" | "LCHR" | "UNIT" => builtin("c", length, None),
        "CLNT" => builtin("c", Some(3), None),
        "LANG" => builtin("c", Some(1), None),
        "CUKY" => builtin("c", Some(5), None),
        "NUMC" => builtin("n", length, None),
        "ACCP" => builtin("n", Some(6), None),
        "SSTRING" | "SSTR" | "STRING" | "STRG" => builtin("string", None, None),
        "RAW" | "LRAW" => builtin("x", length, None),
        "RAWSTRING" | "RSTR" | "GEOM_EWKB" | "GGM1" => builtin("xstring", None, None),
        "DATS" | "DATN" => builtin("d", None, None),
        "TIMS" | "TIMN" => builtin("t", None, None),
        "UTCLONG" | "UTCL" => builtin("utclong", None, None),
        u if u.starts_with("DECFLOAT16") || u.starts_with("DF16") || u.starts_with("D16") => {
            builtin("decfloat16", None, None)
        }
        u if u.starts_with("DECFLOAT34") || u.starts_with("DF34") || u.starts_with("D34") => {
            builtin("decfloat34", None, None)
        }
        _ => return None,
    })
}

fn number(text: Option<String>) -> Option<u32> {
    text?.trim().parse().ok().filter(|n| *n > 0)
}

/// Text of the first `<name>...</name>`, trimmed; `None` if missing or empty.
fn tag(text: &str, name: &str) -> Option<String> {
    let start = text.find(&format!("<{name}>"))? + name.len() + 2;
    let end = start + text[start..].find(&format!("</{name}>"))?;
    let value = text[start..end].trim();
    (!value.is_empty()).then(|| value.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_type_lines() {
        let short = crate::types::tests::short;
        let adt = |kind: &str, name: &str, ty: &str, len: &str| {
            format!("<ttyp:tableType><ttyp:rowType><ttyp:typeKind>{kind}</ttyp:typeKind><ttyp:typeName>{name}</ttyp:typeName><ttyp:builtInType><ttyp:dataType>{ty}</ttyp:dataType><ttyp:length>{len}</ttyp:length><ttyp:decimals>000002</ttyp:decimals></ttyp:builtInType><ttyp:rangeType/></ttyp:rowType></ttyp:tableType>")
        };
        let line = |t: String| table_type_line(&t).as_ref().map(short);
        assert_eq!(
            line(adt("predefinedAbapType", "", "STRING", "000000")).as_deref(),
            Some("string")
        );
        assert_eq!(
            line(adt("predefinedAbapType", "", "CHAR", "000010")).as_deref(),
            Some("c(10)")
        );
        assert_eq!(
            line(adt("predefinedAbapType", "", "DEC", "000015")).as_deref(),
            Some("p(8,2)")
        );
        assert_eq!(
            line(adt("dictionaryType", "BAPIRET2", "STRU", "000000")).as_deref(),
            Some("named bapiret2")
        );
        assert_eq!(
            line(adt("refToClassOrInterfaceType", "IF_X", "", "")).as_deref(),
            Some("ref if_x")
        );
        let git = "<DD40V><TYPENAME>ZTT</TYPENAME><ROWTYPE>ZS_ROW</ROWTYPE><ROWKIND>S</ROWKIND><DATATYPE>STRU</DATATYPE></DD40V>";
        assert_eq!(line(git.to_string()).as_deref(), Some("named zs_row"));
        let git = "<DD40V><TYPENAME>ZTT</TYPENAME><DATATYPE>STRG</DATATYPE></DD40V>";
        assert_eq!(line(git.to_string()).as_deref(), Some("string"));
    }

    #[test]
    fn ddl_structure() {
        let s = structure(
            "@EndUserText.label : 'Root'
define structure suid_st_node_root {
  include suid_st_key;
  @AbapCatalog.foreignKey.screenCheck : true
  bname         : xubname
    with value help user_comp
      where bname = suid_st_node_root.bname;
  idref         : suid_idref;
  key mandt     : mandt not null;
  name          : abap.string(0);
  value         : reference to cl_xms_headerfield;
}",
        )
        .unwrap();
        assert_eq!(s.includes, ["SUID_ST_KEY"]);
        assert_eq!(
            s.components,
            [
                ("bname".into(), Some(ValueType::Data("xubname".into()))),
                ("idref".into(), Some(ValueType::Data("suid_idref".into()))),
                ("mandt".into(), Some(ValueType::Data("mandt".into()))),
                ("name".into(), None),
                (
                    "value".into(),
                    Some(ValueType::Ref("cl_xms_headerfield".into()))
                ),
            ]
        );
    }

    #[test]
    fn abapgit_structure() {
        let s = structure(
            "<abapGit><DD03P_TABLE>
<DD03P><FIELDNAME>.INCLUDE</FIELDNAME><PRECFIELD>ZS_KEY</PRECFIELD></DD03P>
<DD03P><FIELDNAME>TABLE</FIELDNAME><ROLLNAME>ZCL_EXCEL_TABLE</ROLLNAME><DATATYPE>REF</DATATYPE><COMPTYPE>R</COMPTYPE><REFTYPE>C</REFTYPE></DD03P>
<DD03P><FIELDNAME>NAME</FIELDNAME><ROLLNAME>ZEXCEL_FIELDNAME</ROLLNAME><COMPTYPE>E</COMPTYPE></DD03P>
<DD03P><FIELDNAME>RAW</FIELDNAME><DATATYPE>CHAR</DATATYPE></DD03P>
</DD03P_TABLE></abapGit>",
        )
        .unwrap();
        assert_eq!(s.includes, ["ZS_KEY"]);
        assert_eq!(
            s.components,
            [
                (
                    "table".into(),
                    Some(ValueType::Ref("ZCL_EXCEL_TABLE".into()))
                ),
                (
                    "name".into(),
                    Some(ValueType::Data("ZEXCEL_FIELDNAME".into()))
                ),
                ("raw".into(), None),
            ]
        );
    }

    #[test]
    fn types_of_data_elements_and_domains() {
        let short = crate::types::tests::short;
        let ty = |text: &str| match data_element(text) {
            Some(Element::Type(t)) | Some(Element::Reference(t)) => short(&t),
            Some(Element::Domain(d)) => format!("domain {d}"),
            None => "none".into(),
        };
        // ADT with the type of the domain
        assert_eq!(
            ty("<dtel:typeKind>domain</dtel:typeKind><dtel:typeName>XUBNAME</dtel:typeName><dtel:dataType>CHAR</dtel:dataType><dtel:dataTypeLength>000012</dtel:dataTypeLength><dtel:dataTypeDecimals>000000</dtel:dataTypeDecimals>"),
            "c(12)"
        );
        assert_eq!(
            ty("<dtel:typeKind>refToClifType</dtel:typeKind><dtel:typeName>IF_IDENTITY</dtel:typeName><dtel:dataType/>"),
            "ref if_identity"
        );
        // abapGit: own type, domain, reference to data
        assert_eq!(
            ty("<DD04V><ROLLNAME>ZD</ROLLNAME><DATATYPE>DEC</DATATYPE><LENG>000013</LENG><DECIMALS>000003</DECIMALS></DD04V>"),
            "p(7,3)"
        );
        assert_eq!(
            ty("<DD04V><ROLLNAME>ZD</ROLLNAME><DOMNAME>ZDOM</DOMNAME><REFKIND>D</REFKIND></DD04V>"),
            "domain ZDOM"
        );
        assert_eq!(
            ty("<DD04V><ROLLNAME>ZD</ROLLNAME><DOMNAME>ZS</DOMNAME><REFKIND>R</REFKIND><REFTYPE>S</REFTYPE></DD04V>"),
            "ref data of named zs"
        );
        let domain = "<DD01V><DOMNAME>ZDOM</DOMNAME><DATATYPE>STRG</DATATYPE></DD01V>";
        assert_eq!(
            domain_type(domain).as_ref().map(short).as_deref(),
            Some("string")
        );
        // built-in columns of DDL and abapGit structures
        let s = structure("define structure zs { a : abap.dec(15,2); b : abap.char(3) not null; }")
            .unwrap();
        let b: Vec<String> = s
            .builtins
            .iter()
            .map(|(n, t)| format!("{n} {}", short(t)))
            .collect();
        assert_eq!(b, ["a p(8,2)", "b c(3)"]);
    }

    #[test]
    fn data_elements() {
        let adt = r#"<blue:wbobj><dtel:dataElement xmlns:dtel="x"><dtel:typeKind>refToClifType</dtel:typeKind><dtel:typeName>IF_IDENTITY</dtel:typeName></dtel:dataElement></blue:wbobj>"#;
        assert_eq!(data_element_reference(adt).as_deref(), Some("IF_IDENTITY"));
        let adt = r#"<dtel:typeKind>domain</dtel:typeKind><dtel:typeName>XUBNAME</dtel:typeName>"#;
        assert_eq!(data_element_reference(adt), None);
        let git = "<DD04V><ROLLNAME>ZREF</ROLLNAME><DOMNAME>ZCL_X</DOMNAME><DATATYPE>REF</DATATYPE><REFKIND>R</REFKIND><REFTYPE>C</REFTYPE></DD04V>";
        assert_eq!(data_element_reference(git).as_deref(), Some("ZCL_X"));
        let git =
            "<DD04V><ROLLNAME>ZD</ROLLNAME><DOMNAME>ZDOM</DOMNAME><REFKIND>D</REFKIND></DD04V>";
        assert_eq!(data_element_reference(git), None);
    }
}
