use lazy_static::lazy_static;
use regex::Regex;
use std::str::FromStr;

lazy_static! {
    static ref LABEL_REGEX: Regex =
        Regex::new(r"iflabel\s*\{\s*(?P<label>[\w\s,]+)?\s*\}").unwrap();
}

#[derive(Debug, PartialEq, Clone, Default, Eq)]
pub(crate) struct Label {
    pub(crate) principals: Vec<String>,
}
impl Label {
    #[allow(dead_code)]
    pub(crate) fn new(principals: Vec<String>) -> Self {
        Self { principals }
    }

    pub(crate) fn is_public(&self) -> bool {
        self.principals.is_empty()
    }

    #[allow(dead_code)]
    pub(crate) fn new_public() -> Self {
        Self { principals: vec![] }
    }

    pub(crate) fn to_string(&self) -> String {
        if self.is_public() {
            return "{}".to_string();
        }
        let principals = self.principals.join(", ");
        format!("{{ {} }}", principals)
    }

    /// Check labels direction conversion i.e. you can move down in the lattice, not up
    /// ```latex
    ///       AB
    ///      / \
    ///     A   B
    ///      \ /
    ///       0
    /// ```
    pub(crate) fn is_higher_in_lattice_path(&self, label: &Label) -> bool {
        // If the test label is public, then it is never more restrictive
        if label.is_public() {
            return true;
        }

        // If self has more principals, then it is higher in the lattice
        if self.principals.len() > label.principals.len() {
            // Check if the label is a subset of the self
            for principal in &label.principals {
                if self.principals.contains(principal) {
                    return true;
                }
            }
            return false;
        }

        // If the labels have the same principals, then you can "convert" between them
        label.principals == self.principals
    }
}
#[derive(Debug, Clone)]
pub(crate) struct LabelParseError;

impl std::fmt::Display for LabelParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Label parse error")
    }
}

impl FromStr for Label {
    type Err = LabelParseError;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        match LABEL_REGEX.captures(string) {
            Some(captures) => match captures.name("label") {
                Some(label) => {
                    let principals = label
                        .as_str()
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect::<Vec<String>>();
                    Ok(Label { principals })
                }
                None => Ok(Label::new_public()),
            },
            None => Err(LabelParseError),
        }
    }
}

#[cfg(test)]
mod test_labels {
    use super::Label;
    #[test]
    fn test_label_parse_one() {
        let label = "iflabel { alice }".parse::<Label>().unwrap();
        assert_eq!(label, Label::new(vec!["alice".to_string()]));
    }

    #[test]
    fn test_label_parse_multiple() {
        let principals = ["alice", "bob", "charlie"];

        let mut label_string = String::from("iflabel {");

        let mut i = 0;
        while i < principals.len() {
            let formatted_string = format!("{}, {}", label_string, principals[i]);
            label_string = formatted_string;

            let label = (format!("{label_string}}}")).parse::<Label>().unwrap();
            assert_eq!(
                label,
                Label::new(principals[..=i].iter().map(|s| (*s).to_string()).collect())
            );
            i += 1;
        }
    }

    #[test]
    fn test_label_parse_public() {
        let label = "iflabel {}".parse::<Label>().unwrap();
        assert_eq!(label, Label::new_public());

        let label = "iflabel { }".parse::<Label>().unwrap();
        assert_eq!(label, Label::new_public());
    }

    #[test]
    fn test_failing_regex_should_throw() {
        assert!("".parse::<Label>().is_err());
    }

    #[test]
    fn test_label_conversion() {
        let label1 = Label::new(vec!["alice".to_string(), "bob".to_string()]);
        let label2 = Label::new(vec!["alice".to_string()]);
        let label3 = Label::new(vec!["bob".to_string()]);

        assert!(label1.is_higher_in_lattice_path(&label1));
        assert!(label1.is_higher_in_lattice_path(&label2));
        assert!(label1.is_higher_in_lattice_path(&label3));

        assert!(!label2.is_higher_in_lattice_path(&label1));
        assert!(!label3.is_higher_in_lattice_path(&label1));
        assert!(!label2.is_higher_in_lattice_path(&label3));
    }
}
