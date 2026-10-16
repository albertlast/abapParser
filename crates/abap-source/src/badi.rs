//! BAdIs: which classes run when code calls one. A BAdI of an enhancement
//! spot is called with `GET BADI` / `CALL BADI` by its name; a classic BAdI
//! through `CL_EXITHANDLER` and its interface, like `IF_EX_<name>`.
//!
//! abapGit keeps them as XML: enhancement spots `.enhs.xml` define BAdIs,
//! enhancement implementations `.enho.xml` implement them; classic ones are
//! `.sxsd.xml` (definition) and `.sxci.xml` (implementation).

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// A BAdI with the classes of its active implementations.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Badi {
    /// the BAdI, for a classic one its exit name, upper case
    pub name: String,
    /// interface the implementations implement, when the definition is known
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interface: Option<String>,
    /// classes of the active implementations, upper case
    pub implementations: Vec<String>,
    /// class that runs when no implementation is active: the fallback class
    /// of a BAdI, the default implementation of a classic one
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fallback: Option<String>,
    /// classic BAdI, called through `CL_EXITHANDLER`
    pub classic: bool,
}

impl Badi {
    /// Adds what another source knows: a definition, more implementations.
    pub fn merge(&mut self, other: Badi) {
        if self.interface.is_none() {
            self.interface = other.interface;
        }
        if self.fallback.is_none() {
            self.fallback = other.fallback;
        }
        for c in other.implementations {
            if !self.implementations.contains(&c) {
                self.implementations.push(c);
            }
        }
    }
}

/// BAdIs of abapGit files, by name and classic ones by interface.
#[derive(Default)]
pub(crate) struct BadiIndex {
    badis: HashMap<String, Badi>,
    classic: HashMap<String, Badi>,
}

impl BadiIndex {
    /// Reads an abapGit file; `kind` is its object type like `enho`.
    pub fn add(&mut self, kind: &str, xml: &str) {
        match kind {
            "enhs" => {
                for row in rows(xml, "BADI_NAME") {
                    let Some(name) = field(row, "BADI_NAME") else {
                        continue;
                    };
                    let b = self.badi(&name, false);
                    b.interface = field(row, "INTERFACE_NAME").or(b.interface.take());
                    b.fallback = field(row, "DEFAULT_CLASS").or(b.fallback.take());
                }
            }
            "enho" => {
                for row in rows(xml, "BADI_NAME") {
                    let (Some(name), Some(class)) =
                        (field(row, "BADI_NAME"), field(row, "IMPL_CLASS"))
                    else {
                        continue;
                    };
                    if field(row, "ACTIVE").as_deref() == Some("X") {
                        add_class(self.badi(&name, false), class);
                    }
                }
            }
            "sxsd" => {
                let (Some(exit), Some(interface)) =
                    (field(xml, "EXIT_NAME"), field(xml, "INTER_NAME"))
                else {
                    return;
                };
                let b = self
                    .classic
                    .entry(interface.clone())
                    .or_insert_with(|| Badi {
                        name: exit.clone(),
                        classic: true,
                        ..Badi::default()
                    });
                b.name = exit;
                b.interface = Some(interface);
                b.fallback = field(xml, "DEF_CLNAME");
            }
            "sxci" => {
                let (Some(interface), Some(class)) =
                    (field(xml, "INTER_NAME"), field(xml, "IMP_CLASS"))
                else {
                    return;
                };
                let exit = field(xml, "EXIT_NAME").unwrap_or_default();
                let b = self
                    .classic
                    .entry(interface.clone())
                    .or_insert_with(|| Badi {
                        name: exit,
                        interface: Some(interface),
                        classic: true,
                        ..Badi::default()
                    });
                if field(xml, "ACTIVE").as_deref() == Some("X") {
                    add_class(b, class);
                }
            }
            _ => {}
        }
    }

    fn badi(&mut self, name: &str, classic: bool) -> &mut Badi {
        self.badis.entry(name.to_string()).or_insert_with(|| Badi {
            name: name.to_string(),
            classic,
            ..Badi::default()
        })
    }

    pub fn by_name(&self, name: &str) -> Option<Badi> {
        self.badis.get(&name.to_ascii_uppercase()).cloned()
    }

    pub fn classic(&self, interface: &str) -> Option<Badi> {
        self.classic.get(&interface.to_ascii_uppercase()).cloned()
    }

    /// All BAdIs, those of enhancement spots and the classic ones.
    pub fn all(&self) -> impl Iterator<Item = &Badi> {
        self.badis.values().chain(self.classic.values())
    }
}

fn add_class(b: &mut Badi, class: String) {
    if !b.implementations.contains(&class) {
        b.implementations.push(class);
        b.implementations.sort();
    }
}

/// The parts of `xml` holding one table row each, from one element `<key>`
/// to the next; `key` is the first field of the row read.
fn rows<'a>(xml: &'a str, key: &str) -> Vec<&'a str> {
    let open = format!("<{key}>");
    let starts: Vec<usize> = xml.match_indices(&open).map(|(i, _)| i).collect();
    starts
        .iter()
        .enumerate()
        .map(|(n, i)| &xml[*i..starts.get(n + 1).copied().unwrap_or(xml.len())])
        .collect()
}

/// The text of the first element `<name>`, upper case; `None` when missing or empty.
pub(crate) fn field(xml: &str, name: &str) -> Option<String> {
    let open = format!("<{name}>");
    let start = xml.find(&open)? + open.len();
    let end = start + xml[start..].find('<')?;
    let value = xml[start..end].trim();
    (!value.is_empty()).then(|| value.to_ascii_uppercase())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn abapgit_files() {
        let mut index = BadiIndex::default();
        index.add(
            "enhs",
            "<asx:values><TOOL>BADI_DEF</TOOL><BADI_DATA><ENH_BADI_DATA><BADI_NAME>ZBADI_A</BADI_NAME>\
             <INTERFACE_NAME>ZIF_BADI_A</INTERFACE_NAME><DEFAULT_CLASS>ZCL_A_FALLBACK</DEFAULT_CLASS>\
             </ENH_BADI_DATA><ENH_BADI_DATA><BADI_NAME>ZBADI_B</BADI_NAME>\
             <INTERFACE_NAME>ZIF_BADI_B</INTERFACE_NAME></ENH_BADI_DATA></BADI_DATA></asx:values>",
        );
        index.add(
            "enho",
            "<asx:values><TOOL>BADI_IMPL</TOOL><SPOT_NAME>ZSPOT</SPOT_NAME><IMPL>\
             <ENH_BADI_IMPL_DATA><SPOT_NAME>ZSPOT</SPOT_NAME><BADI_NAME>ZBADI_A</BADI_NAME>\
             <IMPL_NAME>ZIMPL_1</IMPL_NAME><IMPL_CLASS>zcl_impl_1</IMPL_CLASS><ACTIVE>X</ACTIVE>\
             </ENH_BADI_IMPL_DATA><ENH_BADI_IMPL_DATA><SPOT_NAME>ZSPOT</SPOT_NAME>\
             <BADI_NAME>ZBADI_A</BADI_NAME><IMPL_NAME>ZIMPL_2</IMPL_NAME>\
             <IMPL_CLASS>ZCL_IMPL_2</IMPL_CLASS></ENH_BADI_IMPL_DATA></IMPL></asx:values>",
        );
        let a = index.by_name("zbadi_a").unwrap();
        assert_eq!(a.interface.as_deref(), Some("ZIF_BADI_A"));
        assert_eq!(a.fallback.as_deref(), Some("ZCL_A_FALLBACK"));
        // the inactive implementation is left out
        assert_eq!(a.implementations, ["ZCL_IMPL_1"]);
        let b = index.by_name("ZBADI_B").unwrap();
        assert_eq!(b.interface.as_deref(), Some("ZIF_BADI_B"));
        assert_eq!(b.fallback, None);
        assert!(b.implementations.is_empty());

        index.add(
            "sxci",
            "<SXCI><IMPLEMENTATION_DATA><IMP_NAME>ZIMP</IMP_NAME><IMP_CLASS>ZCL_IM_X</IMP_CLASS>\
             <EXIT_NAME>ZEXIT</EXIT_NAME><INTER_NAME>ZIF_EX_EXIT</INTER_NAME><ACTIVE>X</ACTIVE>\
             </IMPLEMENTATION_DATA></SXCI>",
        );
        index.add(
            "sxsd",
            "<SXSD><BADI><EXIT_NAME>ZEXIT</EXIT_NAME><INTER_NAME>ZIF_EX_EXIT</INTER_NAME>\
             <DEF_CLNAME>ZCL_DEF</DEF_CLNAME></BADI></SXSD>",
        );
        let c = index.classic("zif_ex_exit").unwrap();
        assert_eq!(c.name, "ZEXIT");
        assert!(c.classic);
        assert_eq!(c.implementations, ["ZCL_IM_X"]);
        assert_eq!(c.fallback.as_deref(), Some("ZCL_DEF"));
    }
}
