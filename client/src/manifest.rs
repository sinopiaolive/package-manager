#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Manifest<'a> {
    pub entries: Vec<ManifestEntry<'a>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ManifestEntry<'a> {
    WsOrComment(WsOrComment<'a>),
    Dependencies(Dependencies<'a>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Dependencies<'a> {
    // " dependencies { // foo"
    pub opening_line: &'a str,

    pub entries: Vec<DependenciesEntry<'a>>,

    // " } // foo"
    pub closing_line: &'a str,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DependenciesEntry<'a> {
    BlankLine(WsOrComment<'a>),
    DependencyLine(DependencyLine<'a>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DependencyLine<'a> {
    pub prefix_ws: &'a str,

    pub body_source: &'a str,
    // body_source, parsed.
    pub body: DependencyBody<'a>,

    pub suffix_ws: &'a str,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DependencyBody<'a> {
    pub package_name: &'a str,
    pub version_constraint_component1: &'a str,
    pub version_constraint_component2: Option<&'a str>,
}

// WsOrComment matches "  " and "  // foo".
// It does not include the trailing newline character.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WsOrComment<'a> {
    pub ws: &'a str,
    // Includes "//". Can be empty.
    pub comment: &'a str,
}

#[cfg(test)]
mod test {
    use super::*;
    use manifest_grammar::*;

    #[test]
    fn manifest_parser() {
        assert_eq!(manifest(""), Ok(Manifest { entries: vec![
            ManifestEntry::WsOrComment(WsOrComment { ws: "", comment: "" })
        ] }));
    }
}
