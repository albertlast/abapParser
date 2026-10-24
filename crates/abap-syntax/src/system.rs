//! The system fields: the components of the structure `sy` (type SYST),
//! with their types. Taken from the structure SYST and its data elements in
//! an SAP system (SAP_BASIS 816); the same as abaplint's built-in `sy`,
//! which has `fdayw` untyped and `spono` as text.

/// A built-in elementary ABAP type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuiltinType {
    /// `i`, 4 byte integer
    I,
    /// `b`, 1 byte integer
    B,
    /// `c`, text field
    C,
    /// `n`, numeric text
    N,
    /// `d`, date `YYYYMMDD`
    D,
    /// `t`, time `HHMMSS`
    T,
    /// `x`, byte field
    X,
    /// `p`, packed number
    P,
}

impl BuiltinType {
    pub fn as_str(self) -> &'static str {
        match self {
            BuiltinType::I => "i",
            BuiltinType::B => "b",
            BuiltinType::C => "c",
            BuiltinType::N => "n",
            BuiltinType::D => "d",
            BuiltinType::T => "t",
            BuiltinType::X => "x",
            BuiltinType::P => "p",
        }
    }
}

/// A system field, like `sy-subrc`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SystemField {
    /// lower case, without `sy-`
    pub name: &'static str,
    pub kind: BuiltinType,
    /// characters of `c` and `n`, bytes of `x` and `p`; 0 for types
    /// with a fixed length
    pub length: u32,
    pub decimals: u32,
    /// the data element typing it in SYST, like `SYST_SUBRC`
    pub data_element: &'static str,
}

const fn f(
    name: &'static str,
    kind: BuiltinType,
    length: u32,
    decimals: u32,
    data_element: &'static str,
) -> SystemField {
    SystemField {
        name,
        kind,
        length,
        decimals,
        data_element,
    }
}

use BuiltinType::*;

/// The components of SYST, in their order.
pub const SYSTEM_FIELDS: &[SystemField] = &[
    f("index", I, 0, 0, "SYST_INDEX"),
    f("pagno", I, 0, 0, "SYST_PAGNO"),
    f("tabix", I, 0, 0, "SYST_TABIX"),
    f("tfill", I, 0, 0, "SYST_TFILL"),
    f("tlopc", I, 0, 0, "SYST_TLOPC"),
    f("tmaxl", I, 0, 0, "SYST_TMAXL"),
    f("toccu", I, 0, 0, "SYST_TOCCU"),
    f("ttabc", I, 0, 0, "SYST_TTABC"),
    f("tstis", I, 0, 0, "SYST_TSTIS"),
    f("ttabi", I, 0, 0, "SYST_TTABI"),
    f("dbcnt", I, 0, 0, "SYST_DBCNT"),
    f("fdpos", I, 0, 0, "SYST_FDPOS"),
    f("colno", I, 0, 0, "SYST_COLNO"),
    f("linct", I, 0, 0, "SYST_LINCT"),
    f("linno", I, 0, 0, "SYST_LINNO"),
    f("linsz", I, 0, 0, "SYST_LINSZ"),
    f("pagct", I, 0, 0, "SYST_PAGCT"),
    f("macol", I, 0, 0, "SYST_MACOL"),
    f("marow", I, 0, 0, "SYST_MAROW"),
    f("tleng", I, 0, 0, "SYST_TLENG"),
    f("sfoff", I, 0, 0, "SYST_SFOFF"),
    f("willi", I, 0, 0, "SYST_WILLI"),
    f("lilli", I, 0, 0, "SYST_LILLI"),
    f("subrc", I, 0, 0, "SYST_SUBRC"),
    f("fleng", I, 0, 0, "SYST_FLENG"),
    f("cucol", I, 0, 0, "SYST_CUCOL"),
    f("curow", I, 0, 0, "SYST_CUROW"),
    f("lsind", I, 0, 0, "SYST_LSIND"),
    f("listi", I, 0, 0, "SYST_LISTI"),
    f("stepl", I, 0, 0, "SYST_STEPL"),
    f("tpagi", I, 0, 0, "SYST_TPAGI"),
    f("winx1", I, 0, 0, "SYST_WINX1"),
    f("winy1", I, 0, 0, "SYST_WINY1"),
    f("winx2", I, 0, 0, "SYST_WINX2"),
    f("winy2", I, 0, 0, "SYST_WINY2"),
    f("winco", I, 0, 0, "SYST_WINCO"),
    f("winro", I, 0, 0, "SYST_WINRO"),
    f("windi", I, 0, 0, "SYST_WINDI"),
    f("srows", I, 0, 0, "SYST_SROWS"),
    f("scols", I, 0, 0, "SYST_SCOLS"),
    f("loopc", I, 0, 0, "SYST_LOOPC"),
    f("folen", I, 0, 0, "SYST_FOLEN"),
    f("fodec", I, 0, 0, "SYST_FODEC"),
    f("tzone", I, 0, 0, "SYST_TZONE"),
    f("dayst", C, 1, 0, "SYST_DAYST"),
    f("ftype", C, 1, 0, "SYST_FTYPE"),
    f("appli", X, 2, 0, "SYST_APPLI"),
    f("fdayw", B, 0, 0, "SYST_FDAYW"),
    f("ccurs", P, 5, 0, "SYST_CCURS"),
    f("ccurt", P, 5, 0, "SYST_CCURT"),
    f("debug", C, 1, 0, "SYST_DEBUG"),
    f("ctype", C, 1, 0, "SYST_CTYPE"),
    f("input", C, 1, 0, "SYST_INPUT"),
    f("langu", C, 1, 0, "SYST_LANGU"),
    f("modno", I, 0, 0, "SYST_MODNO"),
    f("batch", C, 1, 0, "SYST_BATCH"),
    f("binpt", C, 1, 0, "SYST_BINPT"),
    f("calld", C, 1, 0, "SYST_CALLD"),
    f("dynnr", C, 4, 0, "SYST_DYNNR"),
    f("dyngr", C, 4, 0, "SYST_DYNGR"),
    f("newpa", C, 1, 0, "SYST_NEWPA"),
    f("pri40", C, 1, 0, "SYST_PRI40"),
    f("rstrt", C, 1, 0, "SYST_RSTRT"),
    f("wtitl", C, 1, 0, "SYST_WTITL"),
    f("cpage", I, 0, 0, "SYST_CPAGE"),
    f("dbnam", C, 20, 0, "SYST_DBNAM"),
    f("mandt", C, 3, 0, "SYST_MANDT"),
    f("prefx", C, 3, 0, "SYST_PREFX"),
    f("fmkey", C, 3, 0, "SYST_FMKEY"),
    f("pexpi", N, 1, 0, "SYST_PEXPI"),
    f("prini", N, 1, 0, "SYST_PRINI"),
    f("primm", C, 1, 0, "SYST_PRIMM"),
    f("prrel", C, 1, 0, "SYST_PRREL"),
    f("playo", C, 5, 0, "SYST_PLAYO"),
    f("prbig", C, 1, 0, "SYST_PRBIG"),
    f("playp", C, 1, 0, "SYST_PLAYP"),
    f("prnew", C, 1, 0, "SYST_PRNEW"),
    f("prlog", C, 1, 0, "SYST_PRLOG"),
    f("pdest", C, 4, 0, "SYST_PDEST"),
    f("plist", C, 12, 0, "SYST_PLIST"),
    f("pauth", N, 2, 0, "SYST_PAUTH"),
    f("prdsn", C, 6, 0, "SYST_PRDSN"),
    f("pnwpa", C, 1, 0, "SYST_PNWPA"),
    f("callr", C, 8, 0, "SYST_CALLR"),
    f("repi2", C, 40, 0, "SYST_REPI2"),
    f("rtitl", C, 70, 0, "SYST_RTITL"),
    f("prrec", C, 12, 0, "SYST_PRREC"),
    f("prtxt", C, 68, 0, "SYST_PRTXT"),
    f("prabt", C, 12, 0, "SYST_PRABT"),
    f("lpass", C, 4, 0, "SYST_LPASS"),
    f("nrpag", C, 1, 0, "SYST_NRPAG"),
    f("paart", C, 16, 0, "SYST_PAART"),
    f("prcop", N, 3, 0, "SYST_PRCOP"),
    f("batzs", C, 1, 0, "SYST_BATZS"),
    f("bspld", C, 1, 0, "SYST_BSPLD"),
    f("brep4", C, 4, 0, "SYST_BREP4"),
    f("batzo", C, 1, 0, "SYST_BATZO"),
    f("batzd", C, 1, 0, "SYST_BATZD"),
    f("batzw", C, 1, 0, "SYST_BATZW"),
    f("batzm", C, 1, 0, "SYST_BATZM"),
    f("ctabl", C, 4, 0, "SYST_CTABL"),
    f("dbsys", C, 10, 0, "SYST_DBSYS"),
    f("dcsys", C, 4, 0, "SYST_DCSYS"),
    f("macdb", C, 4, 0, "SYST_MACDB"),
    f("sysid", C, 8, 0, "SYST_SYSID"),
    f("opsys", C, 10, 0, "SYST_OPSYS"),
    f("pfkey", C, 20, 0, "SYST_PFKEY"),
    f("saprl", C, 4, 0, "SYST_SAPRL"),
    f("tcode", C, 20, 0, "SYST_TCODE"),
    f("ucomm", C, 70, 0, "SYST_UCOMM"),
    f("cfwae", C, 5, 0, "SYST_CFWAE"),
    f("chwae", C, 5, 0, "SYST_CHWAE"),
    f("spono", N, 10, 0, "SYST_SPONO"),
    f("sponr", N, 10, 0, "SYST_SPONR"),
    f("waers", C, 5, 0, "SYST_WAERS"),
    f("cdate", D, 0, 0, "SYST_CDATE"),
    f("datum", D, 0, 0, "SYST_DATUM"),
    f("slset", C, 14, 0, "SYST_SLSET"),
    f("subty", X, 1, 0, "SYST_SUBTY"),
    f("subcs", C, 1, 0, "SYST_SUBCS"),
    f("group", C, 1, 0, "SYST_GROUP"),
    f("ffile", C, 8, 0, "SYST_FFILE"),
    f("uzeit", T, 0, 0, "SYST_UZEIT"),
    f("dsnam", C, 8, 0, "SYST_DSNAM"),
    f("tabid", C, 8, 0, "SYST_TABID"),
    f("tfdsn", C, 8, 0, "SYST_TFDSN"),
    f("uname", C, 12, 0, "SYST_UNAME"),
    f("lstat", C, 16, 0, "SYST_LSTAT"),
    f("abcde", C, 26, 0, "SYST_ABCDE"),
    f("marky", C, 1, 0, "SYST_MARKY"),
    f("sfnam", C, 30, 0, "SYST_SFNAM"),
    f("tname", C, 30, 0, "SYST_TNAME"),
    f("msgli", C, 60, 0, "SYST_MSGLI"),
    f("title", C, 70, 0, "SYST_TITLE"),
    f("entry", C, 72, 0, "SYST_ENTRY"),
    f("lisel", C, 255, 0, "SYST_LISEL"),
    f("uline", C, 255, 0, "SYST_ULINE"),
    f("xcode", C, 70, 0, "SYST_XCODE"),
    f("cprog", C, 40, 0, "SYST_CPROG"),
    f("xprog", C, 40, 0, "SYST_XPROG"),
    f("xform", C, 30, 0, "SYST_XFORM"),
    f("ldbpg", C, 40, 0, "SYST_LDBPG"),
    f("tvar0", C, 20, 0, "SYST_TVAR"),
    f("tvar1", C, 20, 0, "SYST_TVAR"),
    f("tvar2", C, 20, 0, "SYST_TVAR"),
    f("tvar3", C, 20, 0, "SYST_TVAR"),
    f("tvar4", C, 20, 0, "SYST_TVAR"),
    f("tvar5", C, 20, 0, "SYST_TVAR"),
    f("tvar6", C, 20, 0, "SYST_TVAR"),
    f("tvar7", C, 20, 0, "SYST_TVAR"),
    f("tvar8", C, 20, 0, "SYST_TVAR"),
    f("tvar9", C, 20, 0, "SYST_TVAR"),
    f("msgid", C, 20, 0, "SYST_MSGID"),
    f("msgty", C, 1, 0, "SYST_MSGTY"),
    f("msgno", N, 3, 0, "SYST_MSGNO"),
    f("msgv1", C, 50, 0, "SYST_MSGV"),
    f("msgv2", C, 50, 0, "SYST_MSGV"),
    f("msgv3", C, 50, 0, "SYST_MSGV"),
    f("msgv4", C, 50, 0, "SYST_MSGV"),
    f("oncom", C, 1, 0, "SYST_ONCOM"),
    f("vline", C, 1, 0, "SYST_VLINE"),
    f("winsl", C, 79, 0, "SYST_WINSL"),
    f("staco", I, 0, 0, "SYST_STACO"),
    f("staro", I, 0, 0, "SYST_STARO"),
    f("datar", C, 1, 0, "SYST_DATAR"),
    f("host", C, 32, 0, "SYST_HOST"),
    f("locdb", C, 1, 0, "SYST_LOCDB"),
    f("locop", C, 1, 0, "SYST_LOCOP"),
    f("datlo", D, 0, 0, "SYST_DATLO"),
    f("timlo", T, 0, 0, "SYST_TIMLO"),
    f("zonlo", C, 6, 0, "SYST_ZONLO"),
];

/// The system field of `sy-subrc`, `syst-subrc` or `subrc`, in any case.
pub fn system_field(name: &str) -> Option<&'static SystemField> {
    let lower = name.to_ascii_lowercase();
    let field = lower
        .strip_prefix("sy-")
        .or_else(|| lower.strip_prefix("syst-"))
        .unwrap_or(&lower);
    SYSTEM_FIELDS.iter().find(|f| f.name == field)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn system_fields() {
        assert_eq!(SYSTEM_FIELDS.len(), 171);
        let subrc = system_field("SY-SUBRC").unwrap();
        assert_eq!((subrc.kind, subrc.data_element), (I, "SYST_SUBRC"));
        let uname = system_field("syst-uname").unwrap();
        assert_eq!((uname.kind, uname.length), (C, 12));
        assert_eq!(system_field("datum").unwrap().kind, D);
        assert_eq!(system_field("sy-msgv1").unwrap().data_element, "SYST_MSGV");
        let ccurs = system_field("sy-ccurs").unwrap();
        assert_eq!((ccurs.kind, ccurs.length, ccurs.decimals), (P, 5, 0));
        assert!(system_field("sy-nope").is_none());
    }
}
