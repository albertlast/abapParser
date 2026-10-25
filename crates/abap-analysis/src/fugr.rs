//! The signature of a function module from the abapGit XML of its function
//! group (`<group>.fugr.xml`), for modules whose source has none, like the
//! generated ones of table maintenance.

/// Sections of the signature: keyword, abapGit list and entry.
const SECTIONS: [(&str, &str, &str); 5] = [
    ("IMPORTING", "IMPORT", "RSIMP"),
    ("EXPORTING", "EXPORT", "RSEXP"),
    ("CHANGING", "CHANGING", "RSCHA"),
    ("TABLES", "TABLES", "RSTBL"),
    ("EXCEPTIONS", "EXCEPTION", "RSEXC"),
];

/// The function module with its signature as the `*"` comment block SAP
/// writes after `FUNCTION name.`; the parser reads the signature from it.
/// `None` if the XML does not have the module.
pub fn interface_source(xml: &str, function: &str) -> Option<String> {
    let item = xml
        .split("<item>")
        .skip(1)
        .map(|i| i.split("</item>").next().unwrap_or_default())
        .find(|i| tag(i, "FUNCNAME").is_some_and(|n| n.eq_ignore_ascii_case(function)))?;
    let mut out = format!("FUNCTION {function}.\n");
    for (keyword, list, entry) in SECTIONS {
        // up to the last closing tag: the list of exceptions, EXCEPTION,
        // has entries with an EXCEPTION
        let Some(list) = item.find(&format!("<{list}>")).and_then(|start| {
            let start = start + list.len() + 2;
            let end = item.rfind(&format!("</{list}>"))?;
            item.get(start..end)
        }) else {
            continue;
        };
        let entries: Vec<&str> = list
            .split(&format!("<{entry}>"))
            .skip(1)
            .map(|e| e.split(&format!("</{entry}>")).next().unwrap_or_default())
            .collect();
        if entries.is_empty() {
            continue;
        }
        out.push_str(&format!("*\"  {keyword}\n"));
        for e in entries {
            if let Some(line) = parameter(keyword, e) {
                out.push_str(&format!("*\"     {line}\n"));
            }
        }
    }
    out.push_str("ENDFUNCTION.\n");
    Some(out)
}

/// One parameter as the comment block writes it, like
/// `VALUE(IV_ID) TYPE I DEFAULT 0` or `T_DATA STRUCTURE ZROW OPTIONAL`.
fn parameter(keyword: &str, e: &str) -> Option<String> {
    if keyword == "EXCEPTIONS" {
        return tag(e, "EXCEPTION");
    }
    let name = tag(e, "PARAMETER")?;
    let mut line = match keyword {
        "TABLES" => name,
        _ if tag(e, "REFERENCE").is_some() => format!("REFERENCE({name})"),
        _ => format!("VALUE({name})"),
    };
    if let Some(t) = tag(e, "TYP") {
        let r = if tag(e, "REF_CLASS").is_some() {
            "REF TO "
        } else {
            ""
        };
        line.push_str(&format!(" TYPE {r}{t}"));
    } else if let Some(f) = tag(e, "DBFIELD") {
        line.push_str(&format!(" LIKE {f}"));
    } else if let Some(s) = tag(e, "DBSTRUCT") {
        line.push_str(&format!(" STRUCTURE {s}"));
    }
    if let Some(d) = tag(e, "DEFAULT") {
        line.push_str(&format!(" DEFAULT {d}"));
    } else if tag(e, "OPTIONAL").is_some() {
        line.push_str(" OPTIONAL");
    }
    Some(line)
}

/// The content of `<name>...</name>`.
fn section<'t>(text: &'t str, name: &str) -> Option<&'t str> {
    let start = text.find(&format!("<{name}>"))? + name.len() + 2;
    let end = start + text[start..].find(&format!("</{name}>"))?;
    Some(&text[start..end])
}

/// Text of the first `<name>...</name>`, trimmed, XML entities decoded;
/// `None` if missing or empty.
fn tag(text: &str, name: &str) -> Option<String> {
    let value = section(text, name)?.trim();
    (!value.is_empty()).then(|| {
        value
            .replace("&apos;", "'")
            .replace("&quot;", "\"")
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&amp;", "&")
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signature_from_the_function_group() {
        let xml = "<abapGit><asx:abap><asx:values><AREAT>x</AREAT><FUNCTIONS><item>\
<FUNCNAME>Z_OTHER</FUNCNAME></item><item>\
<FUNCNAME>VIEWFRAME_ZV</FUNCNAME><SHORT_TEXT>Maintenance</SHORT_TEXT>\
<IMPORT><RSIMP><PARAMETER>VIEW_ACTION</PARAMETER><DEFAULT>&apos;S&apos;</DEFAULT></RSIMP>\
<RSIMP><PARAMETER>VIEW_NAME</PARAMETER><DBFIELD>DD02V-TABNAME</DBFIELD></RSIMP>\
<RSIMP><PARAMETER>IO_X</PARAMETER><TYP>ZCL_X</TYP><REF_CLASS>X</REF_CLASS><REFERENCE>X</REFERENCE><OPTIONAL>X</OPTIONAL></RSIMP></IMPORT>\
<EXPORT><RSEXP><PARAMETER>EV_COUNT</PARAMETER><TYP>I</TYP></RSEXP></EXPORT>\
<TABLES><RSTBL><PARAMETER>DBA_SELLIST</PARAMETER><DBSTRUCT>VIMSELLIST</DBSTRUCT></RSTBL></TABLES>\
<EXCEPTION><RSEXC><EXCEPTION>NO_VALUE</EXCEPTION></RSEXC></EXCEPTION>\
</item></FUNCTIONS></asx:values></asx:abap></abapGit>";
        assert_eq!(
            interface_source(xml, "viewframe_zv").unwrap(),
            "FUNCTION viewframe_zv.
*\"  IMPORTING
*\"     VALUE(VIEW_ACTION) DEFAULT 'S'
*\"     VALUE(VIEW_NAME) LIKE DD02V-TABNAME
*\"     REFERENCE(IO_X) TYPE REF TO ZCL_X OPTIONAL
*\"  EXPORTING
*\"     VALUE(EV_COUNT) TYPE I
*\"  TABLES
*\"     DBA_SELLIST STRUCTURE VIMSELLIST
*\"  EXCEPTIONS
*\"     NO_VALUE
ENDFUNCTION.
"
        );
        let symbols = crate::symbols_of(&abap_syntax::parse(
            &interface_source(xml, "VIEWFRAME_ZV").unwrap(),
        ));
        let f = &symbols.function_modules[0];
        let params: Vec<String> = f
            .parameters
            .iter()
            .map(|p| {
                format!(
                    "{} {}",
                    p.name,
                    p.typing.as_ref().map(|t| t.text.as_str()).unwrap_or("-")
                )
            })
            .collect();
        assert_eq!(
            params,
            [
                "VIEW_ACTION -",
                "VIEW_NAME LIKE DD02V-TABNAME",
                "IO_X TYPE REF TO ZCL_X",
                "EV_COUNT TYPE I",
                "DBA_SELLIST STRUCTURE VIMSELLIST"
            ]
        );
        assert_eq!(f.exceptions, ["NO_VALUE"]);
        assert!(interface_source(xml, "Z_MISSING").is_none());
    }
}
