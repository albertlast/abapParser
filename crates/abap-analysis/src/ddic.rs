//! Dictionary types as far as calls need them: the components of structures
//! and tables, and data elements typed as references. Read from the DDL
//! source and XML an SAP system gives through ADT, or from abapGit files.

use crate::calls::ValueType;

/// A dictionary structure or table.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Structure {
    /// components with their type; `None` for built-in types
    pub components: Vec<(String, Option<ValueType>)>,
    /// included structures, upper case
    pub includes: Vec<String>,
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
                    // abap.char(10), abap.string(0): built in
                    [t, ..] if t.to_ascii_lowercase().starts_with("abap.") => None,
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
